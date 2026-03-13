use crate::config::HostEntry;
use crate::core::uploader::Uploader;
use crate::ssh::keys::PassphraseCache;
use crate::ssh::pool::ConnectionPool;
use anyhow::Result;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, serde::Serialize)]
pub struct GrepResult {
    pub host: String,
    pub matches: Vec<String>,
    pub match_count: usize,
    pub exit_code: i32,
}

impl GrepResult {
    pub fn found(&self) -> bool {
        self.match_count > 0
    }
}

pub struct GrepExecutor {
    pool: ConnectionPool,
}

impl GrepExecutor {
    pub fn new() -> Self {
        GrepExecutor {
            pool: ConnectionPool::new(PassphraseCache::new()),
        }
    }

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

    fn build_grep_command(pattern: &str, log_path: &str, context_lines: u8) -> String {
        let escaped_pattern = pattern.replace('\'', r"'\''");
        format!(
            "grep -C{} '{}' {} 2>/dev/null; echo __EXIT__$?",
            context_lines, escaped_pattern, log_path
        )
    }

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

        let (lines_str, exit_code) = Self::parse_grep_output(&output.stdout);
        let matches: Vec<String> = lines_str.lines().map(|l| l.to_string()).collect();
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

    fn parse_grep_output(raw: &str) -> (String, i32) {
        if let Some(idx) = raw.rfind("__EXIT__") {
            let content = raw[..idx].to_string();
            let exit_code = raw[idx + 8..].trim().parse::<i32>().unwrap_or(1);
            (content, exit_code)
        } else {
            (raw.to_string(), 0)
        }
    }

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

        let results: Vec<_> = stream::iter(futures).buffer_unordered(10).collect().await;

        let mut grep_results: Vec<GrepResult> =
            results.into_iter().filter_map(|r| r.ok()).collect();
        grep_results.sort_by(|a, b| b.match_count.cmp(&a.match_count));

        self.pool.close_all().await;
        Ok(grep_results)
    }

    /// Grep avec court-circuit — stoppe dès le premier hôte ayant des résultats.
    /// Utilise tokio::sync::watch pour signalisation + JoinSet::abort_all().
    async fn grep_first_match(
        &self,
        pattern: &str,
        log_path: &str,
        hosts: &[(String, &HostEntry)],
        context_lines: u8,
        timeout: Duration,
    ) -> Result<Vec<GrepResult>> {
        let command = Self::build_grep_command(pattern, log_path, context_lines);
        let (result_tx, mut result_rx) = tokio::sync::mpsc::channel::<GrepResult>(hosts.len());
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
                    result = executor.execute_grep_on_host(&name, &entry, &cmd, timeout) => {
                        if let Ok(grep_result) = result {
                            if grep_result.found() {
                                let _ = tx.send(grep_result).await;
                                let _ = stop_sender.send(true);
                            }
                        }
                    }
                    _ = stop.changed() => {
                        log::debug!("Premier match trouvé, abandon de {}", name);
                    }
                }
            });
        }

        drop(result_tx);

        let mut results = Vec::new();
        if let Some(first) = result_rx.recv().await {
            results.push(first);
        }

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
    fn test_build_grep_command() {
        let cmd = GrepExecutor::build_grep_command("jdupont", "/var/log/*.log", 3);
        assert!(cmd.contains("grep -C3"));
        assert!(cmd.contains("'jdupont'"));
        assert!(cmd.contains("__EXIT__"));
    }

    #[test]
    fn test_build_grep_command_escapes_apostrophe() {
        let cmd = GrepExecutor::build_grep_command("l'utilisateur", "/var/log/*.log", 0);
        assert!(cmd.contains(r"'\''"));
    }

    #[test]
    fn test_parse_grep_output() {
        let (content, code) = GrepExecutor::parse_grep_output("line1\n__EXIT__0\n");
        assert_eq!(code, 0);
        assert!(content.contains("line1"));
    }

    #[test]
    fn test_parse_grep_no_match() {
        let (_, code) = GrepExecutor::parse_grep_output("__EXIT__1\n");
        assert_eq!(code, 1);
    }

    #[test]
    fn test_grep_result_found() {
        let r = GrepResult {
            host: "h".into(),
            matches: vec!["x".into()],
            match_count: 1,
            exit_code: 0,
        };
        assert!(r.found());
    }

    #[test]
    fn test_grep_result_not_found() {
        let r = GrepResult {
            host: "h".into(),
            matches: vec![],
            match_count: 0,
            exit_code: 1,
        };
        assert!(!r.found());
    }
}
