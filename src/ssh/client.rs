// Client SSH/SFTP pour xsshend - Implémentation Pure Rust avec russh
use anyhow::{Context, Result};
use russh::client::{self, Handle};
use russh::keys::*;
use russh_sftp::client::SftpSession;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use super::agent::SshAgentManager;
use super::keys::{PassphraseCache, SshKey, SshKeyManager};

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

/// Client SSH/SFTP asynchrone avec support ssh-agent et cache de passphrases
pub struct SshClient {
    handle: Option<Handle<ClientHandler>>,
    sftp: Option<SftpSession>,
    host: String,
    username: String,
    port: u16,
    passphrase_cache: PassphraseCache,
    /// Répertoire HOME réel récupéré du serveur distant
    remote_home: Option<String>,
}

impl SshClient {
    /// Créer un nouveau client SSH avec un cache de passphrases partagé
    pub fn new_with_cache(host: &str, username: &str, cache: PassphraseCache) -> Result<Self> {
        Ok(SshClient {
            handle: None,
            sftp: None,
            host: host.to_string(),
            username: username.to_string(),
            port: 22,
            passphrase_cache: cache,
            remote_home: None,
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

        // Récupérer le répertoire HOME réel du serveur distant
        self.fetch_remote_home().await?;

        log::debug!(
            "✅ Connexion SSH établie avec {}@{} (HOME: {})",
            self.username,
            self.host,
            self.remote_home.as_deref().unwrap_or("unknown")
        );
        Ok(())
    }

    /// Authentification SSH - Stratégie multi-niveaux
    /// 1. ssh-agent (si disponible)
    /// 2. Clés locales avec cache de passphrases
    /// 3. Demande interactive de passphrase
    async fn authenticate(&mut self, session: &mut Handle<ClientHandler>) -> Result<()> {
        // Niveau 1: Essayer ssh-agent en premier
        log::debug!("🔐 Tentative d'authentification avec ssh-agent...");
        if self.try_ssh_agent_auth(session).await? {
            log::info!("✅ Authentification réussie via ssh-agent");
            return Ok(());
        }

        // Niveau 2 & 3: Clés locales avec cache de passphrases
        log::debug!("🔑 ssh-agent non disponible, essai avec les clés locales");

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
    async fn try_ssh_agent_auth(&self, session: &mut Handle<ClientHandler>) -> Result<bool> {
        // Essayer de se connecter à ssh-agent
        let agent = match SshAgentManager::try_connect().await {
            Some(agent) => agent,
            None => {
                log::debug!("ℹ️  ssh-agent non disponible");
                return Ok(false);
            }
        };

        // Récupérer les identités de l'agent
        let identities = match agent.list_identities().await {
            Ok(ids) => ids,
            Err(e) => {
                log::warn!("⚠️  Impossible de lister les identités ssh-agent: {}", e);
                return Ok(false);
            }
        };

        if identities.is_empty() {
            log::debug!("ℹ️  ssh-agent ne contient aucune clé");
            return Ok(false);
        }

        log::debug!("🔑 {} clé(s) trouvée(s) dans ssh-agent", identities.len());

        // Obtenir le client agent pour l'authentification
        let agent_client = match agent.get_client() {
            Some(client) => client,
            None => {
                log::warn!("⚠️  Impossible d'obtenir le client ssh-agent");
                return Ok(false);
            }
        };

        // Essayer chaque identité de l'agent
        for public_key in identities {
            log::debug!(
                "🔑 Tentative avec clé ssh-agent: {}",
                public_key.algorithm()
            );

            // Utiliser authenticate_publickey_with avec le signer AgentClient
            let mut agent_lock = agent_client.lock().await;

            match session
                .authenticate_publickey_with(
                    &self.username,
                    public_key.clone(),
                    None, // hash_alg - None pour auto
                    &mut *agent_lock,
                )
                .await
            {
                Ok(auth_result) if auth_result.success() => {
                    log::debug!("✅ Authentification réussie avec clé ssh-agent");
                    return Ok(true);
                }
                Ok(_) => {
                    log::debug!("❌ Authentification refusée pour cette clé ssh-agent");
                    continue;
                }
                Err(e) => {
                    log::debug!("❌ Erreur d'authentification ssh-agent: {}", e);
                    continue;
                }
            }
        }

        log::debug!("ℹ️  Aucune clé ssh-agent n'a fonctionné");
        Ok(false)
    }

    /// Authentification avec une clé spécifique (utilise le cache de passphrases)
    async fn authenticate_with_key(
        &mut self,
        session: &mut Handle<ClientHandler>,
        key: &SshKey,
    ) -> Result<()> {
        // Charger la clé privée avec gestion de passphrase et cache
        let key_pair = SshKeyManager::load_key_with_passphrase(
            &key.private_key_path,
            true,
            Some(&self.passphrase_cache),
        )
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

    /// Téléverser un fichier par streaming (optimisé mémoire)
    pub async fn upload_file(&mut self, local_path: &Path, remote_path: &str) -> Result<u64> {
        use tokio::io::{AsyncReadExt, BufReader};

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

        // Créer le fichier distant
        let mut remote_file = sftp
            .create(remote_path)
            .await
            .with_context(|| format!("Impossible de créer le fichier distant: {}", remote_path))?;

        // Ouvrir le fichier local et créer un BufReader pour lecture par chunks
        let file = tokio::fs::File::open(local_path)
            .await
            .with_context(|| format!("Impossible de lire le fichier local: {:?}", local_path))?;

        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; 65536]; // Buffer réutilisable de 64KB
        let mut total_bytes = 0u64;

        // Lire et transférer par chunks
        loop {
            let n = reader
                .read(&mut buffer)
                .await
                .context("Erreur lors de la lecture du fichier local")?;

            if n == 0 {
                break; // EOF atteint
            }

            remote_file
                .write_all(&buffer[..n])
                .await
                .context("Erreur lors de l'écriture du fichier distant")?;

            total_bytes += n as u64;
        }

        remote_file
            .shutdown()
            .await
            .context("Erreur lors de la fermeture du fichier distant")?;

        log::debug!(
            "Fichier téléversé: {} -> {} ({} octets)",
            local_path.display(),
            remote_path,
            total_bytes
        );

        Ok(total_bytes)
    }

    /// Récupérer le répertoire HOME réel du serveur distant
    async fn fetch_remote_home(&mut self) -> Result<()> {
        // Essayer d'abord avec 'pwd' (répertoire de connexion = HOME)
        if let Ok(output) = self.execute_command("pwd", Duration::from_secs(5)).await {
            if output.exit_code == 0 {
                let home = output.stdout.trim();
                if !home.is_empty() {
                    self.remote_home = Some(home.to_string());
                    log::debug!("📂 HOME détecté via pwd: {}", home);
                    return Ok(());
                }
            }
        }

        // Fallback avec 'echo $HOME'
        if let Ok(output) = self
            .execute_command("echo $HOME", Duration::from_secs(5))
            .await
        {
            if output.exit_code == 0 {
                let home = output.stdout.trim();
                if !home.is_empty() && home != "$HOME" {
                    self.remote_home = Some(home.to_string());
                    log::debug!("📂 HOME détecté via $HOME: {}", home);
                    return Ok(());
                }
            }
        }

        // Dernier recours : supposer /home/username
        let fallback_home = format!("/home/{}", self.username);
        self.remote_home = Some(fallback_home.clone());
        log::warn!(
            "⚠️  Impossible de détecter HOME, utilisation de fallback: {}",
            fallback_home
        );

        Ok(())
    }

    /// Obtenir le répertoire HOME réel du serveur distant
    pub fn get_remote_home(&self) -> Option<&str> {
        self.remote_home.as_deref()
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

    /// Exécuter une commande SSH et capturer la sortie
    pub async fn execute_command(
        &mut self,
        command: &str,
        timeout: Duration,
    ) -> Result<CommandOutput> {
        log::debug!("execute_command: '{}'", command);

        let handle = self
            .handle
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Connexion SSH non établie"))?;

        log::debug!("Ouverture d'un canal SSH");
        let mut channel = handle.channel_open_session().await?;

        // Exécuter la commande
        log::debug!("Envoi de la commande au serveur");
        channel.exec(true, command).await?;

        // Lire stdout et stderr
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code: i32 = 0;

        log::debug!("Lecture de la sortie (timeout: {:?})", timeout);
        let result = tokio::time::timeout(timeout, async {
            loop {
                match channel.wait().await {
                    Some(russh::ChannelMsg::Data { ref data }) => {
                        log::trace!("Reçu {} octets sur stdout", data.len());
                        stdout.extend_from_slice(data);
                    }
                    Some(russh::ChannelMsg::ExtendedData { ref data, .. }) => {
                        log::trace!("Reçu {} octets sur stderr", data.len());
                        stderr.extend_from_slice(data);
                    }
                    Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                        log::debug!("Code de sortie: {}", exit_status);
                        exit_code = exit_status as i32;
                    }
                    Some(russh::ChannelMsg::Eof) | None => {
                        log::debug!("Fin de la sortie (EOF)");
                        break;
                    }
                    _ => {}
                }
            }
        })
        .await;

        match result {
            Ok(_) => {
                log::debug!(
                    "Commande terminée - stdout: {} octets, stderr: {} octets, exit: {}",
                    stdout.len(),
                    stderr.len(),
                    exit_code
                );
            }
            Err(_) => {
                log::warn!("Timeout lors de l'exécution de la commande");
            }
        }

        result.context("Timeout d'exécution de la commande")?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&stdout).to_string(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
            exit_code,
        })
    }
}

/// Sortie d'une commande SSH exécutée
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
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
