// Client SSH/SFTP réel avec ssh2
use anyhow::{Context, Result};
use dirs::home_dir;
use ssh2::{Session, Sftp};
use std::io::prelude::*;
use std::net::TcpStream;
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

    /// Se connecte au serveur SSH
    pub fn connect(&mut self) -> Result<()> {
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

        // Connexion TCP
        let tcp = TcpStream::connect((hostname, port))
            .with_context(|| format!("Impossible de se connecter à {}:{}", hostname, port))?;

        // Session SSH
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .with_context(|| format!("Échec du handshake SSH avec {}", hostname))?;

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
                if let Ok(()) = session.userauth_pubkey_file(
                    &self.username,
                    Some(Path::new(&public_key_path)),
                    key_path,
                    None, // Pas de passphrase pour l'instant
                ) {
                    return Ok(());
                }
            }
        }

        // Si aucune clé ne fonctionne, essayer l'agent SSH
        if session.userauth_agent(&self.username).is_ok() {
            return Ok(());
        }

        anyhow::bail!(
            "Échec d'authentification SSH pour {} sur {}. \
            Vérifiez que votre clé publique est installée sur le serveur.",
            self.username,
            self.host
        );
    }

    /// Téléverse un fichier via SFTP
    pub fn upload_file(&mut self, local_path: &Path, remote_path: &str) -> Result<u64> {
        let sftp = self
            .sftp
            .as_ref()
            .context("Client SFTP non initialisé. Appelez connect() d'abord.")?;

        // Ouvrir le fichier local
        let mut local_file = std::fs::File::open(local_path).with_context(|| {
            format!(
                "Impossible d'ouvrir le fichier local: {}",
                local_path.display()
            )
        })?;

        // Créer le fichier distant
        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .with_context(|| format!("Impossible de créer le fichier distant: {}", remote_path))?;

        // Copier les données
        let bytes_copied = std::io::copy(&mut local_file, &mut remote_file)
            .with_context(|| "Erreur lors de la copie des données")?;

        Ok(bytes_copied)
    }

    /// Téléverse un fichier avec callback de progression
    pub fn upload_file_with_progress<F>(
        &mut self,
        local_path: &Path,
        remote_path: &str,
        mut progress_callback: F,
    ) -> Result<u64>
    where
        F: FnMut(u64, u64), // (bytes_written, total_bytes)
    {
        let sftp = self
            .sftp
            .as_ref()
            .context("Client SFTP non initialisé. Appelez connect() d'abord.")?;

        // Obtenir la taille du fichier
        let file_size = std::fs::metadata(local_path)
            .with_context(|| {
                format!(
                    "Impossible de lire les métadonnées: {}",
                    local_path.display()
                )
            })?
            .len();

        // Ouvrir le fichier local
        let mut local_file = std::fs::File::open(local_path).with_context(|| {
            format!(
                "Impossible d'ouvrir le fichier local: {}",
                local_path.display()
            )
        })?;

        // Créer le fichier distant
        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .with_context(|| format!("Impossible de créer le fichier distant: {}", remote_path))?;

        // Copier par chunks avec progression
        let mut buffer = vec![0u8; 64 * 1024]; // Buffer de 64KB
        let mut total_written = 0u64;

        loop {
            let bytes_read = local_file
                .read(&mut buffer)
                .context("Erreur lors de la lecture du fichier local")?;

            if bytes_read == 0 {
                break; // EOF
            }

            remote_file
                .write_all(&buffer[..bytes_read])
                .context("Erreur lors de l'écriture du fichier distant")?;

            total_written += bytes_read as u64;
            progress_callback(total_written, file_size);
        }

        Ok(total_written)
    }

    /// Vérifie si la connexion est active
    pub fn is_connected(&self) -> bool {
        self.session.is_some() && self.sftp.is_some()
    }

    /// Exécute une commande sur le serveur distant et retourne la sortie
    pub fn execute_command(&mut self, command: &str) -> Result<String> {
        let session = self.session.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Pas de session SSH active"))?;

        let mut channel = session.channel_session()
            .with_context(|| "Impossible de créer un canal SSH")?;

        channel.exec(command)
            .with_context(|| format!("Impossible d'exécuter la commande: {}", command))?;

        let mut output = String::new();
        channel.read_to_string(&mut output)
            .with_context(|| "Impossible de lire la sortie de la commande")?;

        channel.wait_close()
            .with_context(|| "Erreur lors de la fermeture du canal")?;

        let exit_status = channel.exit_status()
            .with_context(|| "Impossible de récupérer le code de sortie")?;

        if exit_status != 0 {
            return Err(anyhow::anyhow!(
                "La commande '{}' a échoué avec le code {}", 
                command, 
                exit_status
            ));
        }

        Ok(output)
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
