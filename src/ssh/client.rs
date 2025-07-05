// Client SSH/SFTP réel avec ssh2
use anyhow::{Context, Result};
use dirs::home_dir;
use ssh2::{Session, Sftp};
use std::path::Path;

pub struct SshClient {
    session: Option<Session>,
    sftp: Option<Sftp>,
    host: String,
    username: String,
}

impl SshClient {
    /// Crée un nouveau client SSH
    pub fn new(host: &str, username: &str) -> Result<Self> {
        Ok(SshClient {
            session: None,
            sftp: None,
            host: host.to_string(),
            username: username.to_string(),
        })
    }

    /// Se connecte au serveur SSH avec timeout personnalisable
    pub fn connect_with_timeout(&mut self, timeout: std::time::Duration) -> Result<()> {
        // Parse host:port si port spécifié
        let (hostname, port) = if self.host.contains(':') {
            let parts: Vec<&str> = self.host.split(':').collect();
            (
                parts[0],
                parts.get(1).unwrap_or(&"22").parse().unwrap_or(22),
            )
        } else {
            (self.host.as_str(), 22)
        };

        log::debug!(
            "Tentative de connexion TCP vers {}:{} avec timeout {:?}",
            hostname,
            port,
            timeout
        );

        // Connexion TCP avec timeout - résolution d'adresse plus robuste
        use std::net::ToSocketAddrs;
        let socket_addr = format!("{}:{}", hostname, port);
        let mut addrs = socket_addr
            .to_socket_addrs()
            .with_context(|| format!("Impossible de résoudre l'adresse: {}", socket_addr))?;

        let addr = addrs
            .next()
            .with_context(|| format!("Aucune adresse trouvée pour: {}", socket_addr))?;

        let tcp = std::net::TcpStream::connect_timeout(&addr, timeout).with_context(|| {
            format!(
                "Timeout de connexion TCP vers {} après {:?}",
                socket_addr, timeout
            )
        })?;

        // Définir des timeouts pour les opérations read/write
        tcp.set_read_timeout(Some(timeout))?;
        tcp.set_write_timeout(Some(timeout))?;

        log::debug!("Connexion TCP établie, début du handshake SSH");

        // Session SSH
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);

        // Le handshake peut aussi bloquer, donc on utilise un timeout pour tout le processus
        session
            .handshake()
            .with_context(|| format!("Échec du handshake SSH avec {} après timeout", hostname))?;

        log::debug!("Handshake SSH réussi, début de l'authentification");

        // Authentification par clé SSH
        self.authenticate_with_key(&mut session)?;

        // Initialiser SFTP
        let sftp = session
            .sftp()
            .with_context(|| "Impossible d'initialiser le canal SFTP")?;

        self.session = Some(session);
        self.sftp = Some(sftp);

        Ok(())
    }

    /// Authentification par clé SSH
    fn authenticate_with_key(&self, session: &mut Session) -> Result<()> {
        // D'abord essayer l'authentification par agent SSH
        if let Ok(()) = session.userauth_agent(&self.username) {
            log::info!("Authentification SSH-Agent réussie pour {}", self.username);
            return Ok(());
        }

        let home = home_dir().context("Impossible de déterminer le répertoire home")?;

        // Chemins des clés SSH par défaut
        let private_key_paths = [
            home.join(".ssh/id_rsa"),
            home.join(".ssh/id_ed25519"),
            home.join(".ssh/id_ecdsa"),
        ];

        // Chercher une clé valide
        for key_path in &private_key_paths {
            if key_path.exists() {
                let public_key_path = format!("{}.pub", key_path.display());

                // Essayer l'authentification
                match session.userauth_pubkey_file(
                    &self.username,
                    Some(Path::new(&public_key_path)),
                    key_path,
                    None, // Pas de passphrase pour l'instant
                ) {
                    Ok(()) => {
                        log::info!(
                            "Authentification par clé publique réussie : {}",
                            key_path.display()
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        log::debug!("Échec authentification avec {} : {}", key_path.display(), e);
                    }
                }
            }
        }

        anyhow::bail!(
            "Échec de l'authentification SSH pour l'utilisateur '{}'. Essayé: agent SSH et clés privées.",
            self.username
        )
    }

    /// Téléverse un fichier via SFTP
    pub fn upload_file(&mut self, local_path: &Path, remote_path: &str) -> Result<u64> {
        log::debug!("Début upload: {} -> {}", local_path.display(), remote_path);

        let sftp = self
            .sftp
            .as_ref()
            .context("Client SFTP non initialisé. Appelez connect() d'abord.")?;

        // Vérifier que le fichier local existe et est lisible
        if !local_path.exists() {
            anyhow::bail!("Fichier local introuvable: {}", local_path.display());
        }

        let file_metadata = std::fs::metadata(local_path).with_context(|| {
            format!(
                "Impossible de lire les métadonnées du fichier: {}",
                local_path.display()
            )
        })?;

        let file_size = file_metadata.len();
        log::debug!("Taille fichier local: {} octets", file_size);

        // Ouvrir le fichier local
        let mut local_file = std::fs::File::open(local_path).with_context(|| {
            format!(
                "Impossible d'ouvrir le fichier local: {}",
                local_path.display()
            )
        })?;

        log::debug!("Fichier local ouvert, vérification du répertoire distant...");

        // Extraire le répertoire de destination et s'assurer qu'il existe
        if let Some(parent_dir) = Path::new(remote_path).parent() {
            if let Some(parent_str) = parent_dir.to_str() {
                if !parent_str.is_empty() && parent_str != "/" {
                    self.ensure_remote_directory(parent_str)?;
                }
            }
        }

        log::debug!("Création du fichier distant...");

        // Créer le fichier distant avec gestion d'erreur détaillée
        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .with_context(|| {
                format!(
                    "Impossible de créer le fichier distant: {} (vérifiez les permissions et le chemin)", 
                    remote_path
                )
            })?;

        log::debug!("Fichier distant créé, début du transfert...");

        // Copier les données avec suivi de progression
        let bytes_copied = std::io::copy(&mut local_file, &mut remote_file).with_context(|| {
            format!(
                "Erreur lors de la copie des données ({} -> {})",
                local_path.display(),
                remote_path
            )
        })?;

        log::debug!("Transfert terminé: {} octets copiés", bytes_copied);

        // Vérifier que tous les octets ont été transférés
        if bytes_copied != file_size {
            anyhow::bail!(
                "Transfert incomplet: {} octets copiés sur {} attendus",
                bytes_copied,
                file_size
            );
        }

        Ok(bytes_copied)
    }

    /// Assure que le répertoire de destination existe sur le serveur distant
    pub fn ensure_remote_directory(&self, remote_dir: &str) -> Result<()> {
        let sftp = self.sftp.as_ref().context("Client SFTP non initialisé")?;

        // Vérifier si le répertoire existe déjà
        match sftp.stat(Path::new(remote_dir)) {
            Ok(_) => {
                log::debug!("Répertoire distant {} existe déjà", remote_dir);
                return Ok(());
            }
            Err(_) => {
                log::debug!(
                    "Répertoire distant {} n'existe pas, tentative de création",
                    remote_dir
                );
            }
        }

        // Créer le répertoire (récursivement si nécessaire)
        match sftp.mkdir(Path::new(remote_dir), 0o755) {
            Ok(()) => {
                log::info!("✅ Répertoire distant créé : {}", remote_dir);
                Ok(())
            }
            Err(e) => {
                // Ce n'est pas forcément une erreur critique si le répertoire existe déjà
                log::warn!(
                    "⚠️ Impossible de créer le répertoire {} : {}",
                    remote_dir,
                    e
                );
                Ok(()) // On continue quand même
            }
        }
    }

    /// Ferme la connexion SSH
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = &self.session {
            session.disconnect(None, "Goodbye", None)?;
        }
        self.session = None;
        self.sftp = None;
        Ok(())
    }
}

impl Drop for SshClient {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
