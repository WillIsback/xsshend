// Module d'exécution de commandes SSH sur plusieurs serveurs
use crate::config::HostEntry;
use crate::core::uploader::Uploader;
use crate::ssh::client::SshClient;
use crate::ssh::keys::PassphraseCache;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Exécuteur de commandes SSH
pub struct CommandExecutor {
    passphrase_cache: PassphraseCache,
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

/// Sérialiser Duration en nombre de secondes (float)
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
    /// Créer un nouveau exécuteur de commandes
    pub fn new() -> Self {
        CommandExecutor {
            passphrase_cache: PassphraseCache::new(),
        }
    }

    /// Exécuter une commande sur plusieurs hôtes
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

    /// Exécution séquentielle (un serveur à la fois)
    async fn execute_sequential(
        &self,
        command: &str,
        hosts: &[(String, &HostEntry)],
        timeout: Duration,
    ) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();

        // Créer la barre de progression
        let progress = Arc::new(Mutex::new(ProgressBar::new(hosts.len() as u64)));
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
        drop(pb);

        for (host_name, host_entry) in hosts.iter() {
            let pb = progress.lock().unwrap();
            pb.set_message(format!("Serveur: {}", host_name));
            drop(pb);

            // Exécuter la commande (suspendre la barre de progression)
            let result = {
                let pb = progress.lock().unwrap();
                pb.suspend(|| {
                    // Utiliser un runtime tokio existant
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
                            println!("  ✅ {} ({:.2}s)", host_name, result.duration.as_secs_f64());
                        } else {
                            println!("  ❌ {} - Exit code: {}", host_name, result.exit_code);
                        }
                    });
                    drop(pb);
                    results.push(result);
                }
                Err(e) => {
                    let pb = progress.lock().unwrap();
                    pb.suspend(|| {
                        println!("  ❌ {} - Erreur: {}", host_name, e);
                    });
                    drop(pb);
                }
            }

            let pb = progress.lock().unwrap();
            pb.inc(1);
            drop(pb);
        }

        let pb = progress.lock().unwrap();
        pb.finish_with_message(format!("✅ Terminé ({} serveurs)", hosts.len()));
        drop(pb);

        Ok(results)
    }

    /// Exécution parallèle (plusieurs serveurs simultanément)
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
            let cache = self.passphrase_cache.clone();

            async move {
                let executor = CommandExecutor {
                    passphrase_cache: cache,
                };
                executor.execute_on_host(&cmd, &name, &entry, timeout).await
            }
        });

        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(10) // Max 10 connexions simultanées
            .collect()
            .await;

        // Afficher les résultats au fur et à mesure
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

        Ok(results.into_iter().filter_map(Result::ok).collect())
    }

    /// Exécuter une commande sur un seul hôte
    async fn execute_on_host(
        &self,
        command: &str,
        host_name: &str,
        host_entry: &HostEntry,
        timeout: Duration,
    ) -> Result<CommandResult> {
        log::debug!("Début d'exécution sur {} ({})", host_name, host_entry.alias);
        let start = std::time::Instant::now();
        let (username, host) = Uploader::parse_server_alias(&host_entry.alias)?;

        log::debug!("Connexion SSH à {}@{}", username, host);
        // Créer le client SSH
        let mut client = SshClient::new_with_cache(host, username, self.passphrase_cache.clone())?;
        client.connect_with_timeout(Duration::from_secs(10)).await?;

        log::debug!("Exécution de la commande (timeout: {:?})", timeout);
        // Exécuter la commande
        let output = client.execute_command(command, timeout).await?;
        let duration = start.elapsed();

        log::debug!(
            "Commande terminée - Exit: {}, Durée: {:?}",
            output.exit_code,
            duration
        );

        // Déconnecter
        client.disconnect().await?;

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
        assert!(executor
            .passphrase_cache
            .get(&std::path::PathBuf::from("test"))
            .is_none());
    }

    #[test]
    fn test_executor_default() {
        let executor = CommandExecutor::default();
        assert!(executor
            .passphrase_cache
            .get(&std::path::PathBuf::from("test"))
            .is_none());
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
