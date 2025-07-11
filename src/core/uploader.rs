// Module principal d'orchestration des téléversements
use crate::config::HostEntry;
use crate::core::parallel::SshConnectionPool;
use crate::core::validator::Validator;
use crate::ssh::keys::SshKeyWithPassphrase;
use crate::utils::logger::XsshendLogger;
use anyhow::{Context, Result};
use std::path::Path;

pub struct Uploader {
    ssh_pool: SshConnectionPool,
}

impl Uploader {
    pub fn new() -> Self {
        Uploader {
            ssh_pool: SshConnectionPool::new(),
        }
    }

    /// Crée un nouvel uploader avec une clé SSH validée (avec passphrase)
    pub fn new_with_validated_key(validated_key: SshKeyWithPassphrase) -> Self {
        Uploader {
            ssh_pool: SshConnectionPool::new_with_validated_key(validated_key),
        }
    }

    /// Initialise le pool SSH avec les serveurs (à appeler une seule fois)
    pub fn initialize_ssh_pool(&mut self, hosts: &[(String, &HostEntry)]) -> Result<()> {
        self.ssh_pool.initialize_with_hosts(hosts)?;
        Ok(())
    }

    /// Téléverse plusieurs fichiers vers plusieurs serveurs avec pool SSH
    pub fn upload_files(
        &mut self,
        files: &[&Path],
        hosts: &[(String, &HostEntry)],
        destination: &str,
    ) -> Result<()> {
        // Validation des fichiers
        XsshendLogger::log_upload_start(files.len(), hosts.len());

        for file in files {
            Validator::validate_file(file)
                .with_context(|| format!("Validation échouée pour {}", file.display()))?;
        }

        // Initialiser le pool avec tous les serveurs
        self.ssh_pool.initialize_with_hosts(hosts)?;

        log::info!(
            "🚀 Début du téléversement: {} fichier(s) vers {} serveur(s)",
            files.len(),
            hosts.len()
        );
        log::info!("📂 Destination: {}", destination);

        // Téléverser chaque fichier en parallèle avec gestion d'erreur gracieuse
        let mut overall_success = true;
        let mut failed_files = Vec::new();

        for file in files {
            match self.upload_single_file_parallel_with_callback(file, hosts, destination, None) {
                Ok(_) => {
                    log::info!("✅ Fichier {} téléversé avec succès", file.display());
                }
                Err(e) => {
                    log::error!("❌ Échec téléversement de {} : {}", file.display(), e);
                    failed_files.push(file.display().to_string());
                    overall_success = false;
                    // Continue avec les autres fichiers au lieu de s'arrêter
                }
            }
        }

        // Afficher les statistiques du pool
        let (created, reused, active) = self.ssh_pool.get_stats();
        log::info!(
            "📊 Statistiques connexions - Créées: {}, Réutilisées: {}, Actives: {}",
            created,
            reused,
            active
        );

        // Nettoyer les connexions à la fin
        self.ssh_pool.cleanup_connections()?;

        // Résumé final
        if overall_success {
            log::info!("✅ Téléversement terminé avec succès!");
        } else {
            log::warn!(
                "⚠️ Téléversement terminé avec {} fichier(s) échoué(s): {}",
                failed_files.len(),
                failed_files.join(", ")
            );
            if files.len() - failed_files.len() > 0 {
                log::info!(
                    "📊 {} fichier(s) sur {} réussi(s)",
                    files.len() - failed_files.len(),
                    files.len()
                );
            }
        }
        Ok(())
    }

    /// Téléverse un seul fichier vers tous les serveurs en parallèle avec callback
    pub fn upload_single_file_parallel_with_callback(
        &mut self,
        file: &Path,
        hosts: &[(String, &HostEntry)],
        destination: &str,
        progress_callback: Option<crate::core::parallel::ProgressCallback>,
    ) -> Result<()> {
        let file_size = Validator::get_file_size(file)?;

        log::info!(
            "📤 Téléversement de {} vers {} ({})",
            file.display(),
            destination,
            Validator::format_file_size(file_size)
        );

        // IMPORTANT: Initialiser le pool avec tous les serveurs avant le transfert
        self.ssh_pool.initialize_with_hosts(hosts)?;

        // Utiliser le pool SSH pour upload parallèle avec callback
        self.ssh_pool.upload_file_parallel_with_callback(
            file,
            hosts,
            destination,
            progress_callback,
        )?;

        Ok(())
    }

    /// Téléverse un fichier avec un pool SSH déjà initialisé (évite la réinitialisation)
    pub fn upload_single_file_with_initialized_pool(
        &mut self,
        file: &Path,
        hosts: &[(String, &HostEntry)],
        destination: &str,
        progress_callback: Option<crate::core::parallel::ProgressCallback>,
    ) -> Result<()> {
        let file_size = Validator::get_file_size(file)?;

        log::debug!(
            "📤 Téléversement de {} vers {} ({})",
            file.display(),
            destination,
            Validator::format_file_size(file_size)
        );

        // Utiliser le pool SSH déjà initialisé pour upload parallèle avec callback
        self.ssh_pool.upload_file_parallel_with_callback(
            file,
            hosts,
            destination,
            progress_callback,
        )?;

        Ok(())
    }

    /// Mode dry-run : simulation sans transfert réel
    pub fn dry_run(
        &self,
        files: &[&Path],
        hosts: &[(String, &HostEntry)],
        destination: &str,
    ) -> Result<()> {
        log::info!("🔍 Mode dry-run - Simulation du téléversement");

        // Validation des fichiers
        for file in files {
            Validator::validate_file(file)
                .with_context(|| format!("Validation échouée pour {}", file.display()))?;

            let file_size = Validator::get_file_size(file)?;
            log::info!(
                "📁 {} ({})",
                file.display(),
                Validator::format_file_size(file_size)
            );
        }

        log::info!("🎯 Serveurs cibles:");
        for (name, host_entry) in hosts {
            log::info!("   🖥️  {} → {}", name, host_entry.alias);
        }

        log::info!("📂 Destination: {}", destination);
        log::info!("✅ Simulation terminée - Aucun fichier réellement transféré");

        Ok(())
    }

    /// Nettoyer toutes les connexions SSH du pool
    pub fn cleanup_ssh_connections(&mut self) -> Result<()> {
        self.ssh_pool.cleanup_connections()
    }

    // Unused method - commented out for optimization
    // pub fn upload_interactive(
    //     &self,
    //     files: &[&Path],
    //     hosts: &[(String, &HostEntry)],
    //     destination: &str
    // ) -> Result<()> {
    //     use crate::ui::prompts;

    //     // Demander confirmation
    //     if !prompts::confirm_upload(files, hosts, destination)? {
    //         println!("❌ Téléversement annulé par l'utilisateur");
    //         return Ok(());
    //     }

    //     // Demander la passphrase si nécessaire
    //     if let Some(_passphrase) = prompts::prompt_passphrase()? {
    //         println!("🔑 Passphrase SSH configurée");
    //     }

    //     // Procéder au téléversement normal
    //     self.upload_files(files, hosts, destination)
    // }
}

impl Default for Uploader {
    fn default() -> Self {
        Self::new()
    }
}
