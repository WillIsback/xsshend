use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Représente une clé SSH disponible
#[derive(Debug, Clone, PartialEq)]
pub struct SshKey {
    pub name: String,
    pub private_key_path: PathBuf,
    pub public_key_path: Option<PathBuf>,
    pub key_type: SshKeyType,
    pub comment: Option<String>,
}

/// Types de clés SSH supportées
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SshKeyType {
    Ed25519,
    Rsa,
    Ecdsa,
    Unknown(String),
}

impl std::fmt::Display for SshKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshKeyType::Ed25519 => write!(f, "Ed25519"),
            SshKeyType::Rsa => write!(f, "RSA"),
            SshKeyType::Ecdsa => write!(f, "ECDSA"),
            SshKeyType::Unknown(name) => write!(f, "{}", name),
        }
    }
}

impl SshKey {
    /// Crée une nouvelle instance de SshKey
    pub fn new(name: String, private_key_path: PathBuf) -> Result<Self> {
        let public_key_path = Self::find_public_key(&private_key_path);
        let key_type = Self::detect_key_type(&private_key_path)?;
        let comment = Self::extract_comment(&public_key_path).ok();

        Ok(Self {
            name,
            private_key_path,
            public_key_path,
            key_type,
            comment,
        })
    }

    /// Trouve la clé publique correspondante
    fn find_public_key(private_key_path: &Path) -> Option<PathBuf> {
        let public_key_path = private_key_path.with_extension("pub");
        if public_key_path.exists() {
            Some(public_key_path)
        } else {
            // Essayer avec .pub ajouté au nom complet
            let mut public_key_path = private_key_path.to_path_buf();
            public_key_path.set_extension(format!(
                "{}.pub",
                private_key_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            ));
            if public_key_path.exists() {
                Some(public_key_path)
            } else {
                None
            }
        }
    }

    /// Détecte le type de clé en analysant le fichier
    fn detect_key_type(private_key_path: &Path) -> Result<SshKeyType> {
        if let Ok(content) = fs::read_to_string(private_key_path) {
            if content.contains("BEGIN OPENSSH PRIVATE KEY") {
                // Nouvelle format OpenSSH - analyser plus en détail si nécessaire
                if private_key_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.contains("ed25519"))
                    .unwrap_or(false)
                {
                    return Ok(SshKeyType::Ed25519);
                }
                // Essayer de déterminer depuis le nom du fichier
                if let Some(filename) = private_key_path.file_name().and_then(|s| s.to_str()) {
                    if filename.contains("rsa") {
                        return Ok(SshKeyType::Rsa);
                    }
                    if filename.contains("ecdsa") {
                        return Ok(SshKeyType::Ecdsa);
                    }
                    if filename.contains("ed25519") {
                        return Ok(SshKeyType::Ed25519);
                    }
                }
                return Ok(SshKeyType::Unknown("OpenSSH".to_string()));
            }

            if content.contains("BEGIN RSA PRIVATE KEY") {
                return Ok(SshKeyType::Rsa);
            }
            if content.contains("BEGIN EC PRIVATE KEY") {
                return Ok(SshKeyType::Ecdsa);
            }
            if content.contains("BEGIN DSA PRIVATE KEY") {
                return Ok(SshKeyType::Unknown("DSA".to_string()));
            }
        }

        // Fallback: essayer de deviner depuis le nom du fichier
        if let Some(filename) = private_key_path.file_name().and_then(|s| s.to_str()) {
            if filename.contains("ed25519") {
                return Ok(SshKeyType::Ed25519);
            }
            if filename.contains("rsa") {
                return Ok(SshKeyType::Rsa);
            }
            if filename.contains("ecdsa") {
                return Ok(SshKeyType::Ecdsa);
            }
        }

        Ok(SshKeyType::Unknown("Unknown".to_string()))
    }

    /// Extrait le commentaire de la clé publique
    fn extract_comment(public_key_path: &Option<PathBuf>) -> Result<String> {
        if let Some(path) = public_key_path {
            let content = fs::read_to_string(path)?;
            // Format typique: "ssh-ed25519 AAAAC3... user@hostname"
            if let Some(comment) = content.split_whitespace().nth(2) {
                return Ok(comment.to_string());
            }
        }
        Err(anyhow!("Aucun commentaire trouvé"))
    }

    /// Obtient une description formatée de la clé
    pub fn description(&self) -> String {
        let mut desc = format!("{} ({})", self.name, self.key_type);
        if let Some(comment) = &self.comment {
            desc.push_str(&format!(" - {}", comment));
        }
        desc
    }
}

/// Gestionnaire des clés SSH multiples
pub struct SshKeyManager {
    keys: Vec<SshKey>,
    ssh_dir: PathBuf,
}

impl SshKeyManager {
    /// Crée un nouveau gestionnaire de clés SSH
    pub fn new() -> Result<Self> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow!("Impossible de trouver le répertoire home"))?;
        let ssh_dir = home_dir.join(".ssh");

        let mut manager = Self {
            keys: Vec::new(),
            ssh_dir,
        };

        manager.discover_keys()?;
        Ok(manager)
    }

    /// Découvre automatiquement les clés SSH disponibles
    pub fn discover_keys(&mut self) -> Result<()> {
        log::debug!("🔑 Découverte des clés SSH dans {:?}", self.ssh_dir);

        if !self.ssh_dir.exists() {
            return Err(anyhow!("Le répertoire .ssh n'existe pas"));
        }

        let mut discovered_keys = Vec::new();

        // Clés communes à chercher
        let common_key_names = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];

        // Chercher les clés communes
        for key_name in &common_key_names {
            let key_path = self.ssh_dir.join(key_name);
            if key_path.exists() && key_path.is_file() {
                match SshKey::new(key_name.to_string(), key_path) {
                    Ok(key) => {
                        log::debug!("🔑 Clé trouvée: {}", key.description());
                        discovered_keys.push(key);
                    }
                    Err(e) => {
                        log::warn!("⚠️ Erreur lors de l'analyse de la clé {}: {}", key_name, e);
                    }
                }
            }
        }

        // Chercher d'autres clés privées (fichiers sans extension .pub)
        if let Ok(entries) = fs::read_dir(&self.ssh_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                    // Ignorer les fichiers connus et les clés publiques
                    if filename.ends_with(".pub")
                        || filename == "config"
                        || filename == "known_hosts"
                        || filename == "authorized_keys"
                        || common_key_names.contains(&filename)
                    {
                        continue;
                    }

                    // Essayer de lire le fichier pour voir si c'est une clé privée
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("PRIVATE KEY") {
                            match SshKey::new(filename.to_string(), path.clone()) {
                                Ok(key) => {
                                    log::debug!(
                                        "🔑 Clé additionnelle trouvée: {}",
                                        key.description()
                                    );
                                    discovered_keys.push(key);
                                }
                                Err(e) => {
                                    log::warn!(
                                        "⚠️ Erreur lors de l'analyse de la clé {}: {}",
                                        filename,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        self.keys = discovered_keys;
        log::info!("🔑 {} clé(s) SSH découverte(s)", self.keys.len());
        Ok(())
    }

    /// Sélectionne automatiquement la meilleure clé disponible (non-interactive)
    pub fn select_key_auto(&self) -> Option<&SshKey> {
        if self.keys.is_empty() {
            return None;
        }

        if self.keys.len() == 1 {
            log::info!(
                "🔑 Une seule clé disponible: {}",
                self.keys[0].description()
            );
            return Some(&self.keys[0]);
        }

        self.select_best_key()
    }

    /// Sélectionne automatiquement la "meilleure" clé disponible
    pub fn select_best_key(&self) -> Option<&SshKey> {
        if self.keys.is_empty() {
            return None;
        }

        // Priorité: Ed25519 > RSA > ECDSA > Autres
        let mut best_key = &self.keys[0];

        for key in &self.keys {
            match (&key.key_type, &best_key.key_type) {
                (SshKeyType::Ed25519, _) => best_key = key,
                (SshKeyType::Rsa, SshKeyType::Ecdsa)
                | (SshKeyType::Rsa, SshKeyType::Unknown(_)) => best_key = key,
                (SshKeyType::Ecdsa, SshKeyType::Unknown(_)) => best_key = key,
                _ => {}
            }
        }

        log::info!(
            "🔑 Clé sélectionnée automatiquement: {}",
            best_key.description()
        );
        Some(best_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ssh_key_creation() {
        let temp_dir = TempDir::new().unwrap();
        let key_path = temp_dir.path().join("test_key");

        // Créer un faux fichier de clé
        fs::write(&key_path, "-----BEGIN OPENSSH PRIVATE KEY-----").unwrap();

        let key = SshKey::new("test_key".to_string(), key_path).unwrap();
        assert_eq!(key.name, "test_key");
    }

    #[test]
    fn test_key_type_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Test clé Ed25519
        let ed25519_path = temp_dir.path().join("id_ed25519");
        fs::write(&ed25519_path, "-----BEGIN OPENSSH PRIVATE KEY-----").unwrap();
        let key = SshKey::new("id_ed25519".to_string(), ed25519_path).unwrap();
        assert_eq!(key.key_type, SshKeyType::Ed25519);

        // Test clé RSA
        let rsa_path = temp_dir.path().join("test_rsa");
        fs::write(&rsa_path, "-----BEGIN RSA PRIVATE KEY-----").unwrap();
        let key = SshKey::new("test_rsa".to_string(), rsa_path).unwrap();
        assert_eq!(key.key_type, SshKeyType::Rsa);
    }
}
