// Client SSH/SFTP simplifié pour xsshend
use anyhow::{Context, Result};
use ssh2::{Session, Sftp};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use super::keys::{SshKey, SshKeyManager};

pub struct SshClient {
    session: Option<Session>,
    sftp: Option<Sftp>,
    host: String,
    username: String,
}

impl SshClient {
    /// Créer un nouveau client SSH basique
    pub fn new(host: &str, username: &str) -> Result<Self> {
        Ok(SshClient {
            session: None,
            sftp: None,
            host: host.to_string(),
            username: username.to_string(),
        })
    }

    /// Se connecter au serveur SSH avec timeout
    pub fn connect_with_timeout(&mut self, timeout: Duration) -> Result<()> {
        use std::net::ToSocketAddrs;

        // Résoudre le hostname et établir la connexion TCP
        let addr = format!("{}:22", self.host)
            .to_socket_addrs()
            .with_context(|| format!("Impossible de résoudre l'adresse {}", self.host))?
            .next()
            .ok_or_else(|| anyhow::anyhow!("Aucune adresse IP trouvée pour {}", self.host))?;

        let tcp = TcpStream::connect_timeout(&addr, timeout)
            .with_context(|| format!("Impossible de se connecter à {}:22 ({})", self.host, addr))?;

        tcp.set_read_timeout(Some(timeout))?;
        tcp.set_write_timeout(Some(timeout))?;

        // Créer la session SSH
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        // Authentification
        self.authenticate(&mut session)?;

        // Créer le canal SFTP
        let sftp = session.sftp()?;

        self.session = Some(session);
        self.sftp = Some(sftp);

        log::debug!(
            "✅ Connexion SSH établie avec {}@{}",
            self.username,
            self.host
        );
        Ok(())
    }

    /// Authentification SSH - essaie toutes les méthodes disponibles
    fn authenticate(&self, session: &mut Session) -> Result<()> {
        // Essayer d'abord ssh-agent (idéal car il gère toutes les clés)
        if self.try_ssh_agent_auth(session)? {
            return Ok(());
        }

        // Si ssh-agent échoue, essayer toutes les clés disponibles
        log::debug!("ssh-agent non disponible, essai avec les clés locales");

        if let Ok(key_manager) = SshKeyManager::new() {
            let keys = key_manager.get_all_keys();

            if keys.is_empty() {
                anyhow::bail!("Aucune clé SSH trouvée et ssh-agent non disponible");
            }

            // Essayer chaque clé jusqu'à ce qu'une fonctionne
            let mut last_error = None;
            for key in keys {
                log::debug!("Tentative d'authentification avec la clé: {}", key.name);

                match self.authenticate_with_key(session, key, None) {
                    Ok(()) => {
                        log::info!("✅ Authentification réussie avec la clé: {}", key.name);
                        return Ok(());
                    }
                    Err(e) => {
                        log::debug!("❌ Échec avec la clé {}: {}", key.name, e);
                        last_error = Some(e);
                        // Continuer avec la clé suivante
                    }
                }
            }

            // Aucune clé n'a fonctionné
            if let Some(err) = last_error {
                anyhow::bail!(
                    "Authentification échouée avec toutes les clés disponibles. Dernière erreur: {}",
                    err
                );
            } else {
                anyhow::bail!("Aucune clé SSH n'a fonctionné");
            }
        } else {
            anyhow::bail!("Impossible d'accéder aux clés SSH et ssh-agent non disponible");
        }
    }

    /// Essayer l'authentification via ssh-agent
    fn try_ssh_agent_auth(&self, session: &mut Session) -> Result<bool> {
        match session.userauth_agent(&self.username) {
            Ok(()) => {
                log::debug!("Authentification réussie via ssh-agent");
                Ok(true)
            }
            Err(_) => {
                log::debug!("Authentification via ssh-agent échouée, essai avec clés locales");
                Ok(false)
            }
        }
    }

    /// Authentification avec une clé spécifique
    fn authenticate_with_key(
        &self,
        session: &mut Session,
        key: &SshKey,
        passphrase: Option<&str>,
    ) -> Result<()> {
        let private_key_content =
            std::fs::read_to_string(&key.private_key_path).with_context(|| {
                format!(
                    "Impossible de lire la clé privée: {:?}",
                    key.private_key_path
                )
            })?;

        let public_key_content = if let Some(pub_path) = &key.public_key_path {
            Some(std::fs::read_to_string(pub_path)?)
        } else {
            None
        };

        session
            .userauth_pubkey_memory(
                &self.username,
                public_key_content.as_deref(),
                &private_key_content,
                passphrase,
            )
            .with_context(|| format!("Authentification échouée avec la clé {}", key.name))?;

        log::debug!("Authentification réussie avec la clé {}", key.name);
        Ok(())
    }

    /// Téléverser un fichier
    pub fn upload_file(&mut self, local_path: &Path, remote_path: &str) -> Result<u64> {
        let sftp = self
            .sftp
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Connexion SFTP non établie"))?;

        // Lire le fichier local
        let mut local_file = std::fs::File::open(local_path)
            .with_context(|| format!("Impossible d'ouvrir le fichier local: {:?}", local_path))?;

        let mut buffer = Vec::new();
        local_file.read_to_end(&mut buffer)?;

        // S'assurer que le répertoire distant existe
        if let Some(parent_dir) = Path::new(remote_path).parent() {
            self.ensure_remote_directory(parent_dir.to_str().unwrap_or("/tmp"))?;
        }

        // Créer le fichier distant
        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .with_context(|| format!("Impossible de créer le fichier distant: {}", remote_path))?;

        // Écrire les données
        remote_file.write_all(&buffer)?;

        let size = buffer.len() as u64;
        log::debug!(
            "Fichier téléversé: {} -> {} ({} octets)",
            local_path.display(),
            remote_path,
            size
        );

        Ok(size)
    }

    /// S'assurer que le répertoire distant existe
    fn ensure_remote_directory(&self, remote_dir: &str) -> Result<()> {
        let sftp = self
            .sftp
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Connexion SFTP non établie"))?;

        // Essayer de créer le répertoire (ignore l'erreur s'il existe déjà)
        let _ = sftp.mkdir(Path::new(remote_dir), 0o755);
        Ok(())
    }

    /// Fermer la connexion SSH
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            let _ = session.disconnect(None, "Client disconnect", None);
        }
        self.sftp = None;
        log::debug!("Connexion SSH fermée avec {}@{}", self.username, self.host);
        Ok(())
    }
}

impl Drop for SshClient {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
