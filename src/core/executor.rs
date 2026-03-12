use crate::config::HostEntry;
use crate::core::uploader::Uploader;
use crate::ssh::keys::PassphraseCache;
use crate::ssh::pool::ConnectionPool;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct CommandExecutor {
    pool: ConnectionPool,
}

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
            let pb = progress.lock().await;
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_message("Exécution en cours...");
        }

        for (host_name, host_entry) in hosts.iter() {
            {
                let pb = progress.lock().await;
                pb.set_message(format!("Serveur: {}", host_name));
            }

            match self.execute_on_host(command, host_name, host_entry, timeout).await {
                Ok(result) => {
                    let pb = progress.lock().await;
                    if result.success {
                        pb.println(format!("  ✅ {} ({:.2}s)", host_name, result.duration.as_secs_f64()));
                    } else {
                        pb.println(format!("  ❌ {} - Exit code: {}", host_name, result.exit_code));
                    }
                    pb.inc(1);
                    results.push(result);
                }
                Err(e) => {
                    let pb = progress.lock().await;
                    pb.println(format!("  ❌ {} - Erreur: {}", host_name, e));
                    pb.inc(1);
                }
            }
        }

        {
            let pb = progress.lock().await;
            pb.finish_with_message(format!("✅ Terminé ({} serveurs)", hosts.len()));
        }

        self.pool.close_all().await;
        Ok(results)
    }

    /// Exécution parallèle avec pool partagé (Arc clone — pas de copie).
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
            let pool = self.pool.clone();

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
                println!("  ✅ {} ({:.2}s)", result.host, result.duration.as_secs_f64());
            } else {
                println!("  ❌ {} - Exit code: {}", result.host, result.exit_code);
            }
        }

        self.pool.close_all().await;
        Ok(results.into_iter().filter_map(Result::ok).collect())
    }

    /// Exécuter via le pool — ne déconnecte PAS (connexion réutilisée).
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

        Ok(CommandResult {
            host: host_name.to_string(),
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            duration: start.elapsed(),
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
    fn test_command_result() {
        let result = CommandResult {
            host: "test-host".to_string(),
            exit_code: 0,
            stdout: "ok".to_string(),
            stderr: "".to_string(),
            duration: Duration::from_secs(1),
            success: true,
        };
        assert!(result.success);
    }
}
