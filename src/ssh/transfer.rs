// Module de transfert SSH avec barres de progression
use crate::config::HostEntry;
use crate::ssh::client::SshClient;
use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;

pub struct FileTransfer {
    multi_progress: Arc<MultiProgress>,
}

impl FileTransfer {
    pub fn new() -> Self {
        FileTransfer {
            multi_progress: Arc::new(MultiProgress::new()),
        }
    }

    /// Téléverse un fichier vers un serveur avec barre de progression
    pub fn upload_with_progress(
        &self,
        local_file: &Path,
        remote_path: &str,
        host_entry: &HostEntry,
    ) -> Result<u64> {
        // Parser l'alias pour extraire username et host
        let (username, host) = self.parse_ssh_alias(&host_entry.alias)?;

        // Créer un client SSH
        let mut client = SshClient::new(&host, &username)?;

        // Obtenir la taille du fichier pour la barre de progression
        let file_size = std::fs::metadata(local_file)
            .with_context(|| {
                format!(
                    "Impossible de lire les métadonnées: {}",
                    local_file.display()
                )
            })?
            .len();

        // Créer la barre de progression
        let pb = self.create_progress_bar(file_size, &host_entry.alias);
        let pb_clone = pb.clone();

        // Se connecter
        pb.set_message("Connexion...");
        client
            .connect()
            .with_context(|| format!("Échec de connexion à {}", host_entry.alias))?;

        pb.set_message("Téléversement...");

        // Téléverser avec callback de progression
        let bytes_uploaded = client.upload_file_with_progress(
            local_file,
            remote_path,
            move |bytes_written, _total_bytes| {
                pb_clone.set_position(bytes_written);
            },
        )?;

        pb.finish_with_message("✅ Terminé");

        Ok(bytes_uploaded)
    }

    /// Téléverse vers plusieurs serveurs en parallèle
    pub fn upload_parallel<'a>(
        &self,
        local_file: &Path,
        remote_path: &str,
        hosts: &'a [(String, &'a HostEntry)],
    ) -> Result<Vec<(String, Result<u64>)>> {
        use rayon::prelude::*;

        // Traitement parallèle
        let results: Vec<(String, Result<u64>)> = hosts
            .par_iter()
            .map(|(name, host_entry)| {
                let result = self.upload_with_progress(local_file, remote_path, host_entry);
                (name.clone(), result)
            })
            .collect();

        Ok(results)
    }

    /// Parse un alias SSH (user@host:port) pour extraire username et host
    fn parse_ssh_alias(&self, alias: &str) -> Result<(String, String)> {
        if let Some(at_pos) = alias.find('@') {
            let username = alias[..at_pos].to_string();
            let host = alias[at_pos + 1..].to_string();
            Ok((username, host))
        } else {
            anyhow::bail!(
                "Format d'alias SSH invalide: '{}'. Format attendu: user@host",
                alias
            );
        }
    }

    /// Crée une barre de progression formatée
    fn create_progress_bar(&self, file_size: u64, host_alias: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(file_size));

        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{} [{{bar:40.cyan/blue}}] {{bytes}}/{{total_bytes}} ({{eta}})",
                    Self::truncate_alias(host_alias, 20)
                ))
                .unwrap()
                .progress_chars("##-"),
        );

        pb.set_message("Préparation...");
        pb
    }

    /// Tronque un alias pour l'affichage
    fn truncate_alias(alias: &str, max_len: usize) -> String {
        if alias.len() <= max_len {
            alias.to_string()
        } else {
            format!("{}...", &alias[..max_len - 3])
        }
    }

    /// Affiche un résumé des résultats
    pub fn display_summary(&self, results: &[(String, Result<u64>)]) {
        log::info!("📊 Résumé du téléversement:");

        let mut success_count = 0;
        let mut error_count = 0;
        let mut total_bytes = 0u64;

        for (host_name, result) in results {
            match result {
                Ok(bytes) => {
                    success_count += 1;
                    total_bytes += bytes;
                    log::info!("  ✅ {} - {} octets", host_name, Self::format_bytes(*bytes));
                }
                Err(e) => {
                    error_count += 1;
                    log::warn!("  ❌ {} - Erreur: {}", host_name, e);
                }
            }
        }

        log::info!("📈 Statistiques:");
        log::info!("  Succès: {}/{}", success_count, results.len());
        log::info!("  Échecs: {}", error_count);
        log::info!("  Total transféré: {}", Self::format_bytes(total_bytes));
    }

    /// Formate une taille en octets de manière lisible
    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_alias() {
        let transfer = FileTransfer::new();

        let (username, host) = transfer.parse_ssh_alias("user@example.com").unwrap();
        assert_eq!(username, "user");
        assert_eq!(host, "example.com");

        let (username, host) = transfer
            .parse_ssh_alias("deploy@server.local:2222")
            .unwrap();
        assert_eq!(username, "deploy");
        assert_eq!(host, "server.local:2222");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(FileTransfer::format_bytes(512), "512 B");
        assert_eq!(FileTransfer::format_bytes(1024), "1.0 KB");
        assert_eq!(FileTransfer::format_bytes(1536), "1.5 KB");
        assert_eq!(FileTransfer::format_bytes(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_truncate_alias() {
        assert_eq!(FileTransfer::truncate_alias("short", 20), "short");
        assert_eq!(
            FileTransfer::truncate_alias("very_long_alias_name_here", 10),
            "very_lo..."
        );
    }
}
