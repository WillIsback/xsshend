// Module principal d'orchestration des téléversements
use anyhow::{Context, Result};
use std::path::Path;
use crate::config::HostEntry;
use crate::ssh::transfer::FileTransfer;
use crate::core::validator::Validator;
use crate::utils::logger::XsshendLogger;

pub struct Uploader {
    transfer: FileTransfer,
}

impl Uploader {
    pub fn new() -> Self {
        Uploader {
            transfer: FileTransfer::new(),
        }
    }

    /// Téléverse plusieurs fichiers vers plusieurs serveurs
    pub fn upload_files(
        &self,
        files: &[&Path],
        hosts: &[(String, &HostEntry)],
        destination: &str
    ) -> Result<()> {
        // Validation des fichiers
        XsshendLogger::log_upload_start(files.len(), hosts.len());
        
        for file in files {
            Validator::validate_file(file)
                .with_context(|| format!("Validation échouée pour {}", file.display()))?;
        }

        println!("🚀 Début du téléversement:");
        println!("   📁 {} fichier(s)", files.len());
        println!("   🖥️  {} serveur(s)", hosts.len());
        println!("   📂 Destination: {}", destination);
        println!();

        // Téléverser chaque fichier
        for file in files {
            self.upload_single_file(file, hosts, destination)?;
        }

        println!("\n✅ Téléversement terminé avec succès!");
        Ok(())
    }

    /// Téléverse un seul fichier vers tous les serveurs
    fn upload_single_file(
        &self,
        file: &Path,
        hosts: &[(String, &HostEntry)],
        destination: &str
    ) -> Result<()> {
        let file_name = file.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let remote_path = if destination.ends_with('/') {
            format!("{}{}", destination, file_name)
        } else {
            format!("{}/{}", destination, file_name)
        };

        println!("📤 Téléversement de {} vers {}...", file.display(), remote_path);

        // Afficher informations du fichier
        let file_size = Validator::get_file_size(file)?;
        println!("   Taille: {}", Validator::format_file_size(file_size));
        println!();

        // Téléversement parallèle
        let results = self.transfer.upload_parallel(file, &remote_path, hosts)
            .with_context(|| "Échec du téléversement parallèle")?;

        // Afficher le résumé
        self.transfer.display_summary(&results);

        // Vérifier s'il y a eu des erreurs
        let error_count = results.iter()
            .filter(|(_, result)| result.is_err())
            .count();

        if error_count > 0 {
            anyhow::bail!(
                "Téléversement partiellement échoué: {}/{} serveurs en erreur", 
                error_count, 
                results.len()
            );
        }

        Ok(())
    }

    /// Mode dry-run : simulation sans transfert réel
    pub fn dry_run(
        &self,
        files: &[&Path],
        hosts: &[(String, &HostEntry)],
        destination: &str
    ) -> Result<()> {
        println!("🔍 Mode dry-run - Simulation du téléversement");
        println!();

        // Validation des fichiers
        for file in files {
            Validator::validate_file(file)
                .with_context(|| format!("Validation échouée pour {}", file.display()))?;
            
            let file_size = Validator::get_file_size(file)?;
            println!("📁 {}", file.display());
            println!("   Taille: {}", Validator::format_file_size(file_size));
        }

        println!();
        println!("🎯 Serveurs cibles:");
        for (name, host_entry) in hosts {
            println!("   🖥️  {} → {}", name, host_entry.alias);
        }

        println!();
        println!("📂 Destination: {}", destination);
        println!();
        println!("✅ Simulation terminée - Aucun fichier réellement transféré");

        Ok(())
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
