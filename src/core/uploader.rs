// Module principal d'orchestration des téléversements (simplifié)
use crate::config::HostEntry;
use crate::core::validator::Validator;
use crate::ssh::client::SshClient;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

pub struct Uploader;

impl Uploader {
    pub fn new() -> Self {
        Uploader
    }

    /// Téléverse plusieurs fichiers vers plusieurs serveurs (simplifié)
    pub fn upload_files(
        &self,
        files: &[&Path],
        hosts: &[(String, &HostEntry)],
        destination: &str,
    ) -> Result<()> {
        // Validation des fichiers
        for file in files {
            Validator::validate_file(file)
                .with_context(|| format!("Validation échouée pour {}", file.display()))?;
        }

        println!(
            "🚀 Début du téléversement: {} fichier(s) vers {} serveur(s)",
            files.len(),
            hosts.len()
        );
        println!("📂 Destination: {}", destination);
        println!("🎯 Serveurs ciblés:");
        for (host_name, host_entry) in hosts {
            println!(
                "   • {} → {} ({})",
                host_name, host_entry.alias, host_entry.env
            );
        }

        let mut failed_files = Vec::new();

        for file in files {
            println!("\n📤 Téléversement de {} en cours...", file.display());

            let progress = ProgressBar::new(hosts.len() as u64);
            progress.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"));

            let mut file_success = true;
            for (host_name, host_entry) in hosts {
                progress.set_message(format!("→ {}", host_name));

                match self.upload_to_single_host(file, host_entry, destination) {
                    Ok(_) => {
                        progress.println(format!("  ✅ {}", host_name));
                    }
                    Err(e) => {
                        progress.println(format!("  ❌ {} : {}", host_name, e));
                        file_success = false;
                    }
                }
                progress.inc(1);
            }
            progress.finish();

            if file_success {
                println!("✅ Fichier {} téléversé avec succès", file.display());
            } else {
                println!("❌ Échec partiel pour {}", file.display());
                failed_files.push(file.display().to_string());
            }
        }

        // Résumé final
        if failed_files.is_empty() {
            println!("\n✅ Téléversement terminé avec succès!");
        } else {
            println!(
                "\n⚠️ Téléversement terminé avec {} fichier(s) en échec: {}",
                failed_files.len(),
                failed_files.join(", ")
            );
            println!(
                "📊 {} fichier(s) sur {} réussi(s)",
                files.len() - failed_files.len(),
                files.len()
            );
        }
        Ok(())
    }

    /// Téléverse un fichier vers un seul serveur
    fn upload_to_single_host(
        &self,
        file: &Path,
        host_entry: &HostEntry,
        destination: &str,
    ) -> Result<()> {
        let (username, host) = Self::parse_server_alias(&host_entry.alias)?;

        let mut client = SshClient::new(&host, &username)?;

        client.connect_with_timeout(std::time::Duration::from_secs(10))?;

        let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("file");
        let full_destination = if destination.ends_with('/') {
            format!("{}{}", destination, file_name)
        } else {
            format!("{}/{}", destination, file_name)
        };

        client.upload_file(file, &full_destination)?;
        client.disconnect()?;

        Ok(())
    }

    /// Parse un alias serveur au format "user@host"
    pub fn parse_server_alias(alias: &str) -> Result<(String, String)> {
        if let Some(at_pos) = alias.find('@') {
            if at_pos == 0 || at_pos == alias.len() - 1 {
                anyhow::bail!(
                    "Alias serveur invalide '{}' - format attendu: user@host",
                    alias
                );
            }
            let username = alias[..at_pos].to_string();
            let host = alias[at_pos + 1..].to_string();
            if username.is_empty() || host.is_empty() {
                anyhow::bail!(
                    "Alias serveur invalide '{}' - format attendu: user@host",
                    alias
                );
            }
            Ok((username, host))
        } else {
            anyhow::bail!(
                "Alias serveur invalide '{}' - format attendu: user@host",
                alias
            );
        }
    }

    /// Mode dry-run : simulation sans transfert réel
    pub fn dry_run(
        &self,
        files: &[&Path],
        hosts: &[(String, &HostEntry)],
        destination: &str,
    ) -> Result<()> {
        println!("🔍 Mode dry-run - Simulation du téléversement");

        println!("📁 Fichiers à téléverser:");
        for file in files {
            Validator::validate_file(file)
                .with_context(|| format!("Validation échouée pour {}", file.display()))?;

            let file_size = Validator::get_file_size(file)?;
            println!(
                "   • {} ({})",
                file.display(),
                Validator::format_file_size(file_size)
            );
        }

        println!("🎯 Serveurs cibles:");
        for (name, host_entry) in hosts {
            println!("   • {} → {} ({})", name, host_entry.alias, host_entry.env);
        }

        println!("📂 Destination: {}", destination);
        println!("✅ Simulation terminée - Aucun fichier réellement transféré");

        Ok(())
    }
}

impl Default for Uploader {
    fn default() -> Self {
        Self::new()
    }
}
