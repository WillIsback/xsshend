// Module de validation des fichiers et serveurs
use anyhow::{Result, Context};
use std::path::Path;
use std::fs;

pub struct Validator;

impl Validator {
    /// Valide qu'un fichier existe et est lisible
    pub fn validate_file(file_path: &Path) -> Result<()> {
        if !file_path.exists() {
            anyhow::bail!("Fichier non trouvé: {}", file_path.display());
        }

        if !file_path.is_file() {
            anyhow::bail!("Le chemin ne pointe pas vers un fichier: {}", file_path.display());
        }

        // Vérifier la lisibilité
        fs::File::open(file_path)
            .with_context(|| format!("Impossible de lire le fichier: {}", file_path.display()))?;

        Ok(())
    }

    /// Valide une liste de fichiers
    pub fn validate_files(files: &[&Path]) -> Result<()> {
        for file in files {
            Self::validate_file(file)?;
        }
        Ok(())
    }

    /// Obtient la taille d'un fichier en octets
    pub fn get_file_size(file_path: &Path) -> Result<u64> {
        let metadata = fs::metadata(file_path)
            .with_context(|| format!("Impossible de lire les métadonnées: {}", file_path.display()))?;
        
        Ok(metadata.len())
    }

    /// Formate une taille en octets de manière lisible
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{:.0} {}", size, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_file_validation() {
        // Créer un fichier temporaire
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        // Test validation réussie
        assert!(Validator::validate_file(temp_file.path()).is_ok());

        // Test fichier inexistant
        let non_existent = Path::new("/path/that/does/not/exist");
        assert!(Validator::validate_file(non_existent).is_err());
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(Validator::format_file_size(512), "512 B");
        assert_eq!(Validator::format_file_size(1024), "1.0 KB");
        assert_eq!(Validator::format_file_size(1536), "1.5 KB");
        assert_eq!(Validator::format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(Validator::format_file_size(2 * 1024 * 1024 * 1024), "2.0 GB");
    }
}
