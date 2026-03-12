// Module principal d'orchestration des téléversements (avec uploads parallèles)
//
// Changements v0.6.0 :
//   - ConnectionPool : les connexions SSH sont réutilisées entre les fichiers
//     5 fichiers → 3 serveurs = 3 connexions au lieu de 15
//   - Buffer SFTP 256KB (était 64KB) : meilleur débit sur connexions à haute latence

use crate::config::HostEntry;
use crate::core::validator::Validator;
use crate::ssh::keys::PassphraseCache;
use crate::ssh::pool::ConnectionPool;
use crate::utils::path_expansion;
use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Uploader {
    pool: ConnectionPool,
}

impl Uploader {
    pub fn new() -> Self {
        Uploader {
            pool: ConnectionPool::new(PassphraseCache::new()),
        }
    }

    /// Téléverse plusieurs fichiers vers plusieurs serveurs (connexions poolées)
    pub async fn upload_files(
        &self,
        files: &[&Path],
        hosts: &[(String, &HostEntry)],
        destination: &str,
    ) -> Result<()> {
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

            let progress_arc = Arc::new(Mutex::new(progress));

            let upload_futures = hosts.iter().map(|(host_name, host_entry)| {
                let file = file.to_owned();
                let host_name = host_name.clone();
                let host_entry = (*host_entry).clone();
                let destination = destination.to_owned();
                let pool = self.pool.clone(); // Arc clone — pool partagé
                let progress_clone = Arc::clone(&progress_arc);

                async move {
                    {
                        let progress = progress_clone.lock().await;
                        progress.set_message(format!("→ {}", host_name));
                    }

                    let result =
                        Self::upload_to_single_host_pooled(pool, file, &host_entry, &destination)
                            .await;

                    {
                        let progress = progress_clone.lock().await;
                        match &result {
                            Ok(_) => progress.println(format!("  ✅ {}", host_name)),
                            Err(e) => progress.println(format!("  ❌ {} : {}", host_name, e)),
                        }
                        progress.inc(1);
                    }

                    (host_name, result)
                }
            });

            let results: Vec<_> = stream::iter(upload_futures)
                .buffer_unordered(10)
                .collect()
                .await;

            let file_success = results.iter().all(|(_, result)| result.is_ok());

            {
                let progress = progress_arc.lock().await;
                progress.finish();
            }

            if file_success {
                println!("✅ Fichier {} téléversé avec succès", file.display());
            } else {
                println!("❌ Échec partiel pour {}", file.display());
                failed_files.push(file.display().to_string());
            }
        }

        // Fermer proprement les connexions poolées après tous les transferts
        self.pool.close_all().await;
        log::debug!(
            "Pool uploader fermé ({} connexion(s) fermée(s))",
            self.pool.active_connections()
        );

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

    /// Upload d'un fichier vers un hôte via le pool de connexions.
    /// Réutilise la connexion SSH si elle existe déjà pour cet hôte.
    async fn upload_to_single_host_pooled(
        pool: ConnectionPool,
        file: &Path,
        host_entry: &HostEntry,
        destination: &str,
    ) -> Result<()> {
        let (username, host) = Self::parse_server_alias(&host_entry.alias)?;
        let host_key = format!("{}@{}", username, host);

        let (client_arc, _permit) = pool.acquire(&host_key, username, host).await?;
        let mut client = client_arc.lock().await;

        let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("file");

        let expanded_destination =
            path_expansion::expand_path(destination, username, client.get_remote_home())
                .context("Erreur lors de l'expansion du chemin de destination")?;

        let full_destination = if expanded_destination.ends_with('/') {
            format!("{}{}", expanded_destination, file_name)
        } else {
            format!("{}/{}", expanded_destination, file_name)
        };

        match client.upload_file(file, &full_destination).await {
            Ok(_) => Ok(()),
            Err(e) => {
                drop(client);
                pool.invalidate(&host_key);
                Err(e)
            }
        }
    }

    /// Parse un alias serveur au format "user@host"
    pub fn parse_server_alias(alias: &str) -> Result<(&str, &str)> {
        if let Some(at_pos) = alias.find('@') {
            if at_pos == 0 || at_pos == alias.len() - 1 {
                anyhow::bail!(
                    "Alias serveur invalide '{}' - format attendu: user@host",
                    alias
                );
            }
            let username = &alias[..at_pos];
            let host = &alias[at_pos + 1..];
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
    pub async fn dry_run(
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
