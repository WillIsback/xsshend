// Module grep — recherche parallèle dans les logs de serveurs distants
//
// Use-case principal : déboguer un utilisateur derrière un load balancer
// sans savoir a priori sur quel nœud (WebLogic, Tomcat, etc.) il a été redirigé.
//
// Exemples :
//   # Trouver le nœud WebLogic qui a des logs pour "jdupont"
//   xsshend grep jdupont --log-path "/u01/oracle/wls/logs/*.log" \
//     --env Production --type WebLogic --first-match
//
//   # Grep exhaustif avec 10 lignes de contexte
//   xsshend grep "ERROR.*jdupont" --log-path "/opt/app/logs/server.log" \
//     --env Production --context 10 --output-format json

use crate::config::HostEntry;
use crate::core::uploader::Uploader;
use crate::ssh::keys::PassphraseCache;
use crate::ssh::pool::ConnectionPool;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use std::time::Duration;

/// Résultat d'un grep sur un serveur
#[derive(Debug, Clone, serde::Serialize)]
pub struct GrepResult {
    /// Nom du serveur (clé dans hosts.json)
    pub host: String,
    /// Lignes matchées (incluant lignes de contexte séparées par "--")
    pub matches: Vec<String>,
    /// Nombre de lignes contenant réellement le pattern (hors séparateurs "--")
    pub match_count: usize,
    /// Exit code de grep (0 = match trouvé, 1 = pas de match, >1 = erreur)
    pub exit_code: i32,
}

impl GrepResult {
    pub fn found(&self) -> bool {
        self.match_count > 0
    }
}

/// Exécuteur de grep parallèle avec pool de connexions SSH
pub struct GrepExecutor {
    pool: ConnectionPool,
}

impl GrepExecutor {
    pub fn new() -> Self {
        GrepExecutor {
            pool: ConnectionPool::new(PassphraseCache::new()),
        }
    }

    /// Point d'entrée principal : grep sur tous les hôtes filtrés
    pub async fn grep(
        &self,
        pattern: &str,
        log_path: &str,
        hosts: &[(String, &HostEntry)],
        context_lines: u8,
        first_match: bool,
        timeout: Duration,
    ) -> Result<Vec<GrepResult>> {
        if hosts.is_empty() {
            return Ok(Vec::new());
        }

        if first_match {
            self.grep_first_match(pattern, log_path, hosts, context_lines, timeout)
                .await
        } else {
            self.grep_all(pattern, log_path, hosts, context_lines, timeout)
                .await
        }
    }

    /// Construit la commande grep distante.
    /// Échappe les apostrophes dans le pattern pour éviter les injections shell.
    fn build_grep_command(pattern: &str, log_path: &str, context_lines: u8) -> String {
        // Échapper les apostrophes : ' → '\''
        let escaped_pattern = pattern.replace('\'', r"'\''");
        format!(
            "grep -C{context} '{pattern}' {path} 2>/dev/null; echo __EXIT__$?",
            context = context_lines,
            pattern = escaped_pattern,
            path = log_path
        )
    }

    /// Exécuter le grep sur un seul hôte via le pool
    async fn execute_grep_on_host(
        &self,
        host_name: &str,
        host_entry: &HostEntry,
        command: &str,
        timeout: Duration,
    ) -> Result<GrepResult> {
        let (username, host) = Uploader::parse_server_alias(&host_entry.alias)?;
        let host_key = format!("{}@{}", username, host);

        log::debug!("Grep sur {} via pool", host_key);

        let (client_arc, _permit) = self.pool.acquire(&host_key, username, host).await?;
        let mut client = client_arc.lock().await;

        let output = match client.execute_command(command, timeout).await {
            Ok(out) => out,
            Err(e) => {
                drop(client);
                self.pool.invalidate(&host_key);
                log::warn!("⚠️  Erreur grep sur {} : {}", host_key, e);
                return Err(e);
            }
        };

        // Extraire le vrai exit code depuis le marqueur __EXIT__N
        let (lines_str, exit_code) = Self::parse_grep_output(&output.stdout);

        let matches: Vec<String> = lines_str
            .lines()
            .map(|l| l.to_string())
            .collect();

        // Compter les vraies lignes de match (pas les séparateurs "--")
        let match_count = if exit_code == 0 {
            matches.iter().filter(|l| *l != "--").count()
        } else {
            0
        };

        log::debug!(
            "Grep {} : {} matches (exit {})",
            host_name,
            match_count,
            exit_code
        );

        Ok(GrepResult {
            host: host_name.to_string(),
            matches,
            match_count,
            exit_code,
        })
    }

    /// Parse la sortie du grep qui contient __EXIT__N à la fin
    fn parse_grep_output(raw: &str) -> (String, i32) {
        if let Some(idx) = raw.rfind("__EXIT__") {
            let content = &raw[..idx];
            let exit_str = &raw[idx + 8..].trim();
            let exit_code = exit_str.parse::<i32>().unwrap_or(1);
            (content.to_string(), exit_code)
        } else {
            (raw.to_string(), 0)
        }
    }

    /// Grep exhaustif sur tous les hôtes en parallèle
    async fn grep_all(
        &self,
        pattern: &str,
        log_path: &str,
        hosts: &[(String, &HostEntry)],
        context_lines: u8,
        timeout: Duration,
    ) -> Result<Vec<GrepResult>> {
        let command = Self::build_grep_command(pattern, log_path, context_lines);

        let futures = hosts.iter().map(|(host_name, host_entry)| {
            let cmd = command.clone();
            let name = host_name.clone();
            let entry = (*host_entry).clone();
            let pool = self.pool.clone();

            async move {
                let executor = GrepExecutor { pool };
                executor
                    .execute_grep_on_host(&name, &entry, &cmd, timeout)
                    .await
            }
        });

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(10)
            .collect()
            .await;

        let mut grep_results: Vec<GrepResult> = results.into_iter().filter_map(|r| r.ok()).collect();

        // Trier : hôtes avec matches en premier
        grep_results.sort_by(|a, b| b.match_count.cmp(&a.match_count));

        self.pool.close_all().await;
        Ok(grep_results)
    }

    /// Grep avec court-circuit : stoppe dès le premier hôte ayant des résultats.
    ///
    /// Utilise un canal mpsc + watch pour la signalisation inter-tâches :
    ///   1. Chaque tâche lance le grep via `select!` (opération vs signal d'arrêt)
    ///   2. La première tâche avec un match envoie le résultat et active le signal
    ///   3. Les tâches restantes reçoivent le signal et s'arrêtent proprement
    ///   4. `JoinSet::abort_all()` annule les tâches encore en vol
    async fn grep_first_match(
        &self,
        pattern: &str,
        log_path: &str,
        hosts: &[(String, &HostEntry)],
        context_lines: u8,
        timeout: Duration,
    ) -> Result<Vec<GrepResult>> {
        let command = Self::build_grep_command(pattern, log_path, context_lines);

        // Canal de résultats (capacité = nb hôtes pour ne jamais bloquer)
        let (result_tx, mut result_rx) =
            tokio::sync::mpsc::channel::<GrepResult>(hosts.len());

        // Signal d'arrêt via watch (visible par les tâches non encore démarrées)
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
        let stop_tx = Arc::new(stop_tx);

        let mut set = tokio::task::JoinSet::new();

        for (host_name, host_entry) in hosts.iter() {
            let cmd = command.clone();
            let name = host_name.clone();
            let entry = (*host_entry).clone();
            let pool = self.pool.clone();
            let tx = result_tx.clone();
            let mut stop = stop_rx.clone();
            let stop_sender = Arc::clone(&stop_tx);

            set.spawn(async move {
                let executor = GrepExecutor { pool };
                tokio::select! {
                    // Branche 1 : exécuter le grep
                    result = executor.execute_grep_on_host(&name, &entry, &cmd, timeout) => {
                        if let Ok(grep_result) = result {
                            if grep_result.found() {
                                // Envoyer le résultat et signaler l'arrêt aux autres
                                let _ = tx.send(grep_result).await;
                                let _ = stop_sender.send(true);
                            }
                        }
                    }
                    // Branche 2 : un autre hôte a déjà trouvé — on abandonne
                    _ = stop.changed() => {
                        log::debug!("Premier match déjà trouvé, abandon de {}", name);
                    }
                }
            });
        }

        // Fermer l'émetteur : result_rx.recv() retourne None quand toutes les tâches sont finies
        drop(result_tx);

        // Attendre le premier résultat positif
        let mut results = Vec::new();
        if let Some(first) = result_rx.recv().await {
            results.push(first);
        }

        // Annuler toutes les tâches encore en vol
        set.abort_all();
        self.pool.close_all().await;

        Ok(results)
    }
}

impl Default for GrepExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_grep_command_simple() {
        let cmd = GrepExecutor::build_grep_command("jdupont", "/var/log/*.log", 3);
        assert!(cmd.contains("grep -C3"));
        assert!(cmd.contains("'jdupont'"));
        assert!(cmd.contains("/var/log/*.log"));
        assert!(cmd.contains("__EXIT__"));
    }

    #[test]
    fn test_build_grep_command_escapes_apostrophe() {
        let cmd = GrepExecutor::build_grep_command("l'utilisateur", "/var/log/*.log", 0);
        // L'apostrophe doit être échappée pour ne pas casser le shell distant
        assert!(cmd.contains(r"'\''"));
    }

    #[test]
    fn test_parse_grep_output_with_exit() {
        let raw = "ligne1\nligne2\n__EXIT__0\n";
        let (content, code) = GrepExecutor::parse_grep_output(raw);
        assert_eq!(code, 0);
        assert!(content.contains("ligne1"));
        assert!(!content.contains("__EXIT__"));
    }

    #[test]
    fn test_parse_grep_output_no_match() {
        let raw = "__EXIT__1\n";
        let (content, code) = GrepExecutor::parse_grep_output(raw);
        assert_eq!(code, 1);
        assert!(content.is_empty() || content.trim().is_empty());
    }

    #[test]
    fn test_grep_result_found() {
        let result = GrepResult {
            host: "wls-node-01".to_string(),
            matches: vec!["2024-01-01 jdupont login".to_string()],
            match_count: 1,
            exit_code: 0,
        };
        assert!(result.found());
    }

    #[test]
    fn test_grep_result_not_found() {
        let result = GrepResult {
            host: "wls-node-02".to_string(),
            matches: vec![],
            match_count: 0,
            exit_code: 1,
        };
        assert!(!result.found());
    }

    #[test]
    fn test_executor_creation() {
        let executor = GrepExecutor::new();
        assert_eq!(executor.pool.active_connections(), 0);
    }
}
