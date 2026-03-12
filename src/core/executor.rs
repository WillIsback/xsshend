// Module d'exécution de commandes SSH sur plusieurs serveurs
//
// Changements v0.6.0 :
//   - ConnectionPool : les connexions SSH sont réutilisées entre opérations parallèles
//     → économise N-1 handshakes SSH par groupe d'hôtes
//   - Invalidation automatique en cas d'erreur réseau (reconnexion transparente)

use crate::config::HostEntry;
use crate::core::uploader::Uploader;
use crate::ssh::keys::PassphraseCache;
use crate::ssh::pool::ConnectionPool;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Exécuteur de commandes SSH avec pool de connexions
pub struct CommandExecutor {
    pool: ConnectionPool,
}

/// Résultat de l'exécution d'une commande sur un serveur
#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandResult {
    pub host: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    #[serde(serialize_with = "serialize_duration")]
    pub duration: Duration,
    pub success: bool,
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

/// Résumé de l'exécution pour la sortie JSON
#[derive(Debug, serde::Serialize)]
pub struct ExecutionSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub total_duration_secs: f64,
}

impl CommandExecutor {
    pub fn new() -> Self {
        CommandExecutor {
            pool: ConnectionPool::new(PassphraseCache::new()),
        }
    }

    pub async fn execute(
        &self,
        command: &str,
        hosts: &[(String, &HostEntry)],
        parallel: bool,
        timeout: Duration,
    ) -> Result<Vec<CommandResult>> {
        if parallel {
            self.execute_parallel(command, hosts, timeout).await
        } else {
            self.execute_sequential(command, hosts, timeout).await
        }
    }

    async fn execute_sequential(
        &self,
        command: &str,
        hosts: &[(String, &HostEntry)],
        timeout: Duration,
    ) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();

        let progress = Arc::new(Mutex::new(ProgressBar::new(hosts.len() as u64)));
        {
            let pb = progress.lock().unwrap();
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                    )
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_message("Exécution en cours...");
        }

        for (host_name, host_entry) in hosts.iter() {
            {
                let pb = progress.lock().unwrap();
                pb.set_message(format!("Serveur: {}", host_name));
            }

            let result = {
                let pb = progress.lock().unwrap();
                pb.suspend(|| {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            self.execute_on_host(command, host_name, host_entry, timeout)
                                .await
                        })
                    })
                })
            };

            match result {
                Ok(result) => {
                    let pb = progress.lock().unwrap();
                    pb.suspend(|| {
                        if result.success {
                            println!(
                                "  ✅ {} ({:.2}s)",
                                host_name,
                                result.duration.as_secs_f64()
                            );
                        } else {
                            println!("  ❌ {} - Exit code: {}", host_name, result.exit_code);
                        }
                    });
                    results.push(result);
                }
                Err(e) => {
                    let pb = progress.lock().unwrap();
                    pb.suspend(|| {
                        println!("  ❌ {} - Erreur: {}", host_name, e);
                    });
                }
            }

            {
                let pb = progress.lock().unwrap();
                pb.inc(1);
            }
        }

        {
            let pb = progress.lock().unwrap();
            pb.finish_with_message(format!("✅ Terminé ({} serveurs)", hosts.len()));
        }

        self.pool.close_all().await;
        Ok(results)
    }

    /// Exécution parallèle avec pool de connexions partagé.
    /// Le clone du pool est cheap (Arc interne) — toutes les tâches partagent le même état.
    async fn execute_parallel(
        &self,
        command: &str,
        hosts: &[(String, &HostEntry)],
        timeout: Duration,
    ) -> Result<Vec<CommandResult>> {
        println!("🚀 Exécution parallèle sur {} serveur(s)...\n", hosts.len());

        let futures = hosts.iter().map(|(host_name, host_entry)| {
            let cmd = command.to_owned();
            let name = host_name.clone();
            let entry = (*host_entry).clone();
            let pool = self.pool.clone(); // Arc clone — partagé, pas copié

            async move {
                let executor = CommandExecutor { pool };
                executor.execute_on_host(&cmd, &name, &entry, timeout).await
            }
        });

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(10)
            .collect()
            .await;

        for result in results.iter().flatten() {
            if result.success {
                println!(
                    "  ✅ {} ({:.2}s)",
                    result.host,
                    result.duration.as_secs_f64()
                );
            } else {
                println!("  ❌ {} - Exit code: {}", result.host, result.exit_code);
            }
        }

        self.pool.close_all().await;

        Ok(results.into_iter().filter_map(Result::ok).collect())
    }

    /// Exécuter une commande sur un hôte via le pool.
    /// Ne déconnecte PAS — la connexion reste dans le pool pour réutilisation.
    async fn execute_on_host(
        &self,
        command: &str,
        host_name: &str,
        host_entry: &HostEntry,
        timeout: Duration,
    ) -> Result<CommandResult> {
        let (username, host) = Uploader::parse_server_alias(&host_entry.alias)?;
        let host_key = format!("{}@{}", username, host);

        log::debug!("Exécution sur {} via pool", host_key);
        let start = std::time::Instant::now();

        let (client_arc, _permit) = self.pool.acquire(&host_key, username, host).await?;
        let mut client = client_arc.lock().await;

        let output = match client.execute_command(command, timeout).await {
            Ok(out) => out,
            Err(e) => {
                drop(client);
                self.pool.invalidate(&host_key);
                log::warn!("⚠️  Erreur sur {} (connexion invalidée) : {}", host_key, e);
                return Err(e);
            }
        };

        let duration = start.elapsed();

        Ok(CommandResult {
            host: host_name.to_string(),
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            duration,
            success: output.exit_code == 0,
        })
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor::new();
        assert_eq!(executor.pool.active_connections(), 0);
    }

    #[test]
    fn test_executor_default() {
        let executor = CommandExecutor::default();
        assert_eq!(executor.pool.active_connections(), 0);
    }

    #[test]
    fn test_command_result_creation() {
        let result = CommandResult {
            host: "test-host".to_string(),
            exit_code: 0,
            stdout: "test output".to_string(),
            stderr: "".to_string(),
            duration: Duration::from_secs(1),
            success: true,
        };
        assert_eq!(result.host, "test-host");
        assert_eq!(result.exit_code, 0);
        assert!(result.success);
    }
}
