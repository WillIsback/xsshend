// Client SSH/SFTP pour xsshend - Implémentation Pure Rust avec russh
use anyhow::{Context, Result};
use russh::client::{self, Handle};
use russh::keys::*;
use russh_sftp::client::SftpSession;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use super::keys::{SshKey, SshKeyManager};

/// Handler pour les événements du client SSH
struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Pour l'instant, accepter toutes les clés serveur
        // TODO: Vérifier contre known_hosts
        Ok(true)
    }
}

/// Client SSH/SFTP asynchrone
pub struct SshClient {
    handle: Option<Handle<ClientHandler>>,
    sftp: Option<SftpSession>,
    host: String,
    username: String,
    port: u16,
}

impl SshClient {
    /// Créer un nouveau client SSH
    pub fn new(host: &str, username: &str) -> Result<Self> {
        Ok(SshClient {
            handle: None,
            sftp: None,
            host: host.to_string(),
            username: username.to_string(),
            port: 22,
        })
    }

    /// Se connecter au serveur SSH avec timeout
    pub async fn connect_with_timeout(&mut self, timeout: Duration) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);

        log::debug!("Connexion à {}...", addr);

        // Configuration du client SSH
        let config = Arc::new(russh::client::Config::default());
        let handler = ClientHandler;

        // Connexion avec timeout
        let mut session =
            tokio::time::timeout(timeout, russh::client::connect(config, &addr, handler))
                .await
                .context("Timeout de connexion SSH")?
                .context("Impossible de se connecter au serveur SSH")?;

        // Authentification
        self.authenticate(&mut session).await?;

        // Créer le canal SFTP
        let channel = session.channel_open_session().await?;

        // Demander le sous-système SFTP (étape cruciale !)
        channel
            .request_subsystem(true, "sftp")
            .await
            .context("Impossible de demander le sous-système SFTP")?;

        // Créer la session SFTP
        let sftp = SftpSession::new(channel.into_stream())
            .await
            .context("Impossible de créer la session SFTP")?;

        self.handle = Some(session);
        self.sftp = Some(sftp);

        log::debug!(
            "✅ Connexion SSH établie avec {}@{}",
            self.username,
            self.host
        );
        Ok(())
    }

    /// Authentification SSH - essaie toutes les méthodes disponibles
    async fn authenticate(&self, session: &mut Handle<ClientHandler>) -> Result<()> {
        // Essayer ssh-agent en premier si disponible
        if self.try_ssh_agent_auth(session).await? {
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

                match self.authenticate_with_key(session, key).await {
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
    async fn try_ssh_agent_auth(&self, _session: &mut Handle<ClientHandler>) -> Result<bool> {
        // russh ne supporte pas ssh-agent directement de la même manière
        // On va essayer avec les clés disponibles à la place
        log::debug!("Tentative avec les clés locales (ssh-agent non supporté pour l'instant)");
        Ok(false)
    }

    /// Authentification avec une clé spécifique
    async fn authenticate_with_key(
        &self,
        session: &mut Handle<ClientHandler>,
        key: &SshKey,
    ) -> Result<()> {
        // Charger la clé privée avec gestion de passphrase (mode non-interactif par défaut)
        // TODO: Ajouter un paramètre pour activer le mode interactif si nécessaire
        let key_pair = SshKeyManager::load_key_with_passphrase(&key.private_key_path, false)
            .context(format!("Impossible de charger la clé {}", key.name))?;

        // Authentification avec la clé
        let auth_result = session
            .authenticate_publickey(
                &self.username,
                PrivateKeyWithHashAlg::new(
                    Arc::new(key_pair),
                    session.best_supported_rsa_hash().await?.flatten(),
                ),
            )
            .await
            .context(format!("Authentification échouée avec la clé {}", key.name))?;

        if !auth_result.success() {
            anyhow::bail!(
                "Authentification refusée par le serveur pour la clé {}",
                key.name
            );
        }

        log::debug!("Authentification réussie avec la clé {}", key.name);
        Ok(())
    }

    /// Téléverser un fichier
    pub async fn upload_file(&mut self, local_path: &Path, remote_path: &str) -> Result<u64> {
        // Lire le fichier local
        let buffer = tokio::fs::read(local_path)
            .await
            .with_context(|| format!("Impossible de lire le fichier local: {:?}", local_path))?;

        // S'assurer que le répertoire distant existe
        if let Some(parent_dir) = Path::new(remote_path).parent() {
            self.ensure_remote_directory(parent_dir.to_str().unwrap_or("/tmp"))
                .await?;
        }

        // Obtenir la session SFTP
        let sftp = self
            .sftp
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Connexion SFTP non établie"))?;

        // Créer le fichier distant et écrire les données
        let mut remote_file = sftp
            .create(remote_path)
            .await
            .with_context(|| format!("Impossible de créer le fichier distant: {}", remote_path))?;

        remote_file
            .write_all(&buffer)
            .await
            .context("Erreur lors de l'écriture du fichier distant")?;

        remote_file
            .shutdown()
            .await
            .context("Erreur lors de la fermeture du fichier distant")?;

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
    async fn ensure_remote_directory(&mut self, remote_dir: &str) -> Result<()> {
        let sftp = self
            .sftp
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Connexion SFTP non établie"))?;

        // Essayer de créer le répertoire (ignore l'erreur s'il existe déjà)
        let _ = sftp.create_dir(remote_dir).await;
        Ok(())
    }

    /// Fermer la connexion SSH
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            let _ = handle
                .disconnect(russh::Disconnect::ByApplication, "", "")
                .await;
        }
        self.sftp = None;
        log::debug!("Connexion SSH fermée avec {}@{}", self.username, self.host);
        Ok(())
    }
}

impl Drop for SshClient {
    fn drop(&mut self) {
        // Note: Dans un contexte async, on ne peut pas await dans Drop
        // Les ressources seront nettoyées automatiquement
        if self.handle.is_some() {
            log::debug!("Fermeture automatique de la connexion SSH");
        }
    }
}
