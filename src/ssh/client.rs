// Client SSH/SFTP réel avec ssh2
use anyhow::{Context, Result};
use crossterm::terminal;
use dialoguer::{Password, theme::ColorfulTheme};
use dirs::home_dir;
use ssh2::{Session, Sftp};
use std::io::{Read, Write};
use std::path::Path;

use super::keys::{SshKey, SshKeyManager};

/// Guard pour gérer temporairement la sortie du mode raw
pub struct TerminalModeGuard {
    was_raw: bool,
}

impl TerminalModeGuard {
    pub fn new() -> Result<Self> {
        let was_raw = match terminal::is_raw_mode_enabled() {
            Ok(is_raw) => {
                if is_raw {
                    terminal::disable_raw_mode()?;
                    true
                } else {
                    false
                }
            }
            Err(_) => false, // Assume non-raw if we can't detect
        };

        Ok(TerminalModeGuard { was_raw })
    }
}

impl Drop for TerminalModeGuard {
    fn drop(&mut self) {
        if self.was_raw {
            let _ = terminal::enable_raw_mode();
        }
    }
}

pub struct SshClient {
    session: Option<Session>,
    sftp: Option<Sftp>,
    host: String,
    username: String,
    selected_key: Option<SshKey>,
}

impl SshClient {
    /// Crée un nouveau client SSH
    pub fn new(host: &str, username: &str) -> Result<Self> {
        Ok(SshClient {
            session: None,
            sftp: None,
            host: host.to_string(),
            username: username.to_string(),
            selected_key: None,
        })
    }

    /// Crée un nouveau client SSH avec une clé spécifique
    #[allow(dead_code)]
    pub fn new_with_key(host: &str, username: &str, key: SshKey) -> Result<Self> {
        Ok(SshClient {
            session: None,
            sftp: None,
            host: host.to_string(),
            username: username.to_string(),
            selected_key: Some(key),
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
        // Si une clé spécifique est sélectionnée, l'utiliser en priorité
        if let Some(ref selected_key) = self.selected_key {
            log::info!(
                "🔑 Utilisation de la clé sélectionnée: {}",
                selected_key.description()
            );

            // Essayer d'abord ssh-agent avec cette clé
            if let Ok(()) = session.userauth_agent(&self.username) {
                log::info!(
                    "✅ Authentification SSH-Agent réussie pour {}",
                    self.username
                );
                return Ok(());
            }

            // Sinon utiliser directement le fichier de clé
            return self.authenticate_with_specific_key(session, selected_key);
        }

        // Comportement par défaut: essayer ssh-agent puis les clés communes
        if let Ok(()) = session.userauth_agent(&self.username) {
            log::info!(
                "✅ Authentification SSH-Agent réussie pour {}",
                self.username
            );
            return Ok(());
        }

        log::debug!("🔑 SSH-Agent non disponible ou sans clés, essai des clés locales");

        // Utiliser le gestionnaire de clés pour découvrir et essayer les clés disponibles
        match SshKeyManager::new() {
            Ok(key_manager) => {
                let keys = key_manager.get_keys();

                if keys.is_empty() {
                    return self.authenticate_with_default_keys(session);
                }

                // Essayer chaque clé découverte
                for key in keys {
                    if let Ok(()) = self.authenticate_with_specific_key(session, key) {
                        return Ok(());
                    }
                }

                // Si toutes les clés découvertes ont échoué, essayer les clés par défaut
                self.authenticate_with_default_keys(session)
            }
            Err(_) => {
                // Fallback vers l'ancienne méthode si le gestionnaire de clés échoue
                self.authenticate_with_default_keys(session)
            }
        }
    }

    /// Authentification avec une clé spécifique
    fn authenticate_with_specific_key(&self, session: &mut Session, key: &SshKey) -> Result<()> {
        log::debug!("🔑 Essai d'authentification avec {}", key.description());

        let public_key_path = key
            .public_key_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        // Essayer d'abord sans passphrase (pour les clés non protégées)
        match session.userauth_pubkey_file(
            &self.username,
            public_key_path.as_ref().map(Path::new),
            &key.private_key_path,
            None, // Pas de passphrase dans le premier essai
        ) {
            Ok(()) => {
                log::info!(
                    "✅ Authentification réussie avec la clé {} (sans passphrase)",
                    key.description()
                );
                Ok(())
            }
            Err(e) => {
                log::debug!(
                    "🔓 Première tentative sans passphrase échouée pour {} : {}",
                    key.description(),
                    e
                );

                // Si l'erreur semble indiquer qu'une passphrase est requise, demander la passphrase
                if self.might_need_passphrase(&e) {
                    log::debug!("🔐 Tentative avec passphrase pour {}", key.description());

                    if let Some(passphrase) = self.prompt_for_passphrase(key)? {
                        match session.userauth_pubkey_file(
                            &self.username,
                            public_key_path.as_ref().map(Path::new),
                            &key.private_key_path,
                            Some(&passphrase),
                        ) {
                            Ok(()) => {
                                log::info!(
                                    "✅ Authentification réussie avec la clé {} (avec passphrase)",
                                    key.description()
                                );
                                Ok(())
                            }
                            Err(e2) => {
                                log::debug!(
                                    "❌ Échec authentification avec passphrase pour {} : {}",
                                    key.description(),
                                    e2
                                );
                                Err(anyhow::anyhow!(
                                    "Authentification échouée avec {}: passphrase incorrecte ou erreur SSH",
                                    key.description()
                                ))
                            }
                        }
                    } else {
                        log::debug!(
                            "❌ Passphrase annulée par l'utilisateur pour {}",
                            key.description()
                        );
                        Err(anyhow::anyhow!(
                            "Authentification annulée pour {}",
                            key.description()
                        ))
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "Authentification échouée avec {}: {}",
                        key.description(),
                        e
                    ))
                }
            }
        }
    }

    /// Détermine si l'erreur pourrait indiquer qu'une passphrase est requise
    fn might_need_passphrase(&self, error: &ssh2::Error) -> bool {
        let error_msg = error.to_string().to_lowercase();
        // Patterns d'erreurs qui suggèrent qu'une passphrase est requise
        error_msg.contains("could not read key from file")
            || error_msg.contains("bad decrypt")
            || error_msg.contains("authentication failed")
            || error_msg.contains("private key")
            || error_msg.contains("passphrase")
            || error_msg.contains("encrypted")
    }

    /// Demande la passphrase à l'utilisateur de manière interactive
    fn prompt_for_passphrase(&self, key: &SshKey) -> Result<Option<String>> {
        log::info!(
            "🔐 La clé {} semble protégée par passphrase",
            key.description()
        );

        // Détecter si on est dans un environnement TUI (terminal en mode raw)
        let is_in_tui = self.is_terminal_in_raw_mode();

        if is_in_tui {
            // Utiliser rpassword pour les environnements TUI
            self.prompt_passphrase_with_rpassword(key)
        } else {
            // Utiliser dialoguer pour les environnements CLI normaux
            self.prompt_passphrase_with_dialoguer(key)
        }
    }

    /// Détermine si le terminal est en mode raw (utilisé par TUI)
    fn is_terminal_in_raw_mode(&self) -> bool {
        // Essayer de détecter si crossterm est en mode raw
        // Ceci est une heuristique car il n'y a pas de moyen direct de le savoir

        match terminal::is_raw_mode_enabled() {
            Ok(is_raw) => {
                log::debug!("🔍 Mode raw détecté: {}", is_raw);
                is_raw
            }
            Err(_) => {
                // Fallback : vérifier si on peut écrire normalement sur le terminal
                log::debug!("🔍 Impossible de détecter le mode raw, utilisation d'une heuristique");
                // Si on est dans un TUI, souvent les variables d'environnement spécifiques sont définies
                std::env::var("TERM_PROGRAM").is_ok() && atty::is(atty::Stream::Stdin)
            }
        }
    }

    /// Prompt de passphrase avec dialoguer (pour CLI)
    fn prompt_passphrase_with_dialoguer(&self, key: &SshKey) -> Result<Option<String>> {
        match Password::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Entrez la passphrase pour la clé SSH '{}' (Entrée vide pour annuler)",
                key.name
            ))
            .allow_empty_password(true)
            .interact()
        {
            Ok(passphrase) => {
                if passphrase.is_empty() {
                    log::debug!("❌ Passphrase vide, annulation pour {}", key.description());
                    Ok(None)
                } else {
                    log::debug!("🔐 Passphrase saisie pour {}", key.description());
                    Ok(Some(passphrase))
                }
            }
            Err(e) => {
                log::error!("❌ Erreur lors de la saisie de passphrase : {}", e);
                Err(anyhow::anyhow!(
                    "Erreur lors de la saisie de passphrase : {}",
                    e
                ))
            }
        }
    }

    /// Prompt de passphrase avec rpassword (pour TUI)
    fn prompt_passphrase_with_rpassword(&self, key: &SshKey) -> Result<Option<String>> {
        // Sortir temporairement du mode raw pour permettre l'input
        let _guard = self.temporarily_exit_raw_mode()?;

        print!(
            "🔐 Entrez la passphrase pour la clé SSH '{}' (ou appuyez sur Entrée pour annuler): ",
            key.name
        );
        std::io::stdout().flush()?;

        match rpassword::read_password() {
            Ok(passphrase) => {
                if passphrase.is_empty() {
                    log::debug!("❌ Passphrase vide, annulation pour {}", key.description());
                    Ok(None)
                } else {
                    log::debug!("🔐 Passphrase saisie pour {}", key.description());
                    Ok(Some(passphrase))
                }
            }
            Err(e) => {
                log::error!("❌ Erreur lors de la saisie de passphrase : {}", e);
                Err(anyhow::anyhow!(
                    "Erreur lors de la saisie de passphrase : {}",
                    e
                ))
            }
        }
    }

    /// Sortir temporairement du mode raw et y retourner automatiquement
    fn temporarily_exit_raw_mode(&self) -> Result<TerminalModeGuard> {
        TerminalModeGuard::new()
    }

    /// Méthode de fallback pour l'authentification avec les clés par défaut
    fn authenticate_with_default_keys(&self, session: &mut Session) -> Result<()> {
        let home = home_dir().context("Impossible de déterminer le répertoire home")?;

        // Chemins des clés SSH par défaut (ordre de priorité)
        let private_key_paths = [
            ("id_ed25519", home.join(".ssh/id_ed25519")),
            ("id_rsa", home.join(".ssh/id_rsa")),
            ("id_ecdsa", home.join(".ssh/id_ecdsa")),
        ];

        // Chercher une clé valide
        for (key_name, key_path) in &private_key_paths {
            if key_path.exists() {
                let public_key_path = format!("{}.pub", key_path.display());

                // Essayer d'abord sans passphrase
                match session.userauth_pubkey_file(
                    &self.username,
                    Some(Path::new(&public_key_path)),
                    key_path,
                    None, // Pas de passphrase dans le premier essai
                ) {
                    Ok(()) => {
                        log::info!(
                            "✅ Authentification par clé publique réussie : {} (sans passphrase)",
                            key_path.display()
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        log::debug!(
                            "🔓 Première tentative sans passphrase échouée pour {} : {}",
                            key_path.display(),
                            e
                        );

                        // Si l'erreur semble indiquer qu'une passphrase est requise, demander la passphrase
                        if self.might_need_passphrase(&e) {
                            log::debug!("🔐 Tentative avec passphrase pour {}", key_path.display());

                            // Créer un SshKey temporaire pour le prompt
                            if let Ok(temp_key) = crate::ssh::keys::SshKey::new(
                                key_name.to_string(),
                                key_path.clone(),
                            ) {
                                if let Ok(Some(passphrase)) = self.prompt_for_passphrase(&temp_key)
                                {
                                    match session.userauth_pubkey_file(
                                        &self.username,
                                        Some(Path::new(&public_key_path)),
                                        key_path,
                                        Some(&passphrase),
                                    ) {
                                        Ok(()) => {
                                            log::info!(
                                                "✅ Authentification par clé publique réussie : {} (avec passphrase)",
                                                key_path.display()
                                            );
                                            return Ok(());
                                        }
                                        Err(e2) => {
                                            log::debug!(
                                                "❌ Échec authentification avec passphrase pour {} : {}",
                                                key_path.display(),
                                                e2
                                            );
                                        }
                                    }
                                } else {
                                    log::debug!(
                                        "❌ Passphrase annulée pour {}",
                                        key_path.display()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        anyhow::bail!(
            "Échec de l'authentification SSH pour l'utilisateur '{}'. Essayé: agent SSH et clés privées (avec gestion des passphrases).",
            self.username
        )
    }

    /// Téléverse un fichier via SFTP
    pub fn upload_file(&mut self, local_path: &Path, remote_path: &str) -> Result<u64> {
        log::info!(
            "📤 Début upload: {} -> {}",
            local_path.display(),
            remote_path
        );

        let sftp = self
            .sftp
            .as_ref()
            .context("Client SFTP non initialisé. Appelez connect() d'abord.")?;

        // Expande le chemin distant (gestion de ~/ et $HOME)
        let expanded_remote_path = self.expand_remote_path(remote_path)?;
        if expanded_remote_path != remote_path {
            log::info!(
                "🔍 Chemin expansé: {} -> {}",
                remote_path,
                expanded_remote_path
            );
        }

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

        // Extraire le répertoire de destination et vérifier/adapter les permissions
        let final_remote_path = if let Some(parent_dir) = Path::new(&expanded_remote_path).parent()
        {
            if let Some(parent_str) = parent_dir.to_str() {
                if !parent_str.is_empty() && parent_str != "/" {
                    log::debug!("Vérification des permissions pour: {}", parent_str);

                    // Essayer de trouver un répertoire accessible
                    match self.find_accessible_directory(parent_str) {
                        Ok(accessible_dir) => {
                            // Construire le nouveau chemin complet
                            let filename = Path::new(&expanded_remote_path)
                                .file_name()
                                .and_then(|f| f.to_str())
                                .unwrap_or("uploaded_file");

                            let new_path =
                                format!("{}/{}", accessible_dir.trim_end_matches('/'), filename);

                            if new_path != expanded_remote_path {
                                log::warn!(
                                    "⚠️  Changement du chemin de destination: {} -> {} (permissions)",
                                    expanded_remote_path,
                                    new_path
                                );
                            }
                            new_path
                        }
                        Err(e) => {
                            log::error!(
                                "❌ Impossible de trouver un répertoire accessible pour {}: {}",
                                parent_str,
                                e
                            );
                            return Err(e);
                        }
                    }
                } else {
                    expanded_remote_path.to_string()
                }
            } else {
                expanded_remote_path.to_string()
            }
        } else {
            expanded_remote_path.to_string()
        };

        log::debug!("Création du fichier distant: {}", final_remote_path);

        // Créer le fichier distant avec gestion d'erreur détaillée
        let mut remote_file = sftp
            .create(Path::new(&final_remote_path))
            .with_context(|| {
                format!(
                    "Impossible de créer le fichier distant: {} (vérifiez les permissions d'écriture et que le répertoire parent existe)", 
                    final_remote_path
                )
            })?;

        log::debug!("Fichier distant créé, début du transfert...");

        // Copier les données avec suivi de progression
        let bytes_copied = std::io::copy(&mut local_file, &mut remote_file).with_context(|| {
            format!(
                "Erreur lors de la copie des données ({} -> {})",
                local_path.display(),
                final_remote_path
            )
        })?;

        log::debug!("Transfert terminé: {} octets copiés", bytes_copied);

        // Informer l'utilisateur du chemin final
        if final_remote_path != remote_path {
            log::info!(
                "✅ Fichier uploadé vers: {} (adapté pour permissions)",
                final_remote_path
            );
        } else {
            log::info!("✅ Fichier uploadé vers: {}", final_remote_path);
        }

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

    /// Assure que le répertoire de destination existe sur le serveur distant (récursif)
    pub fn ensure_remote_directory(&self, remote_dir: &str) -> Result<()> {
        let sftp = self.sftp.as_ref().context("Client SFTP non initialisé")?;

        // Normaliser le chemin (retirer les "//" et les "/./" etc.)
        let normalized_path = remote_dir.trim_end_matches('/');
        if normalized_path.is_empty() || normalized_path == "/" {
            return Ok(()); // Répertoire racine existe toujours
        }

        // Vérifier si le répertoire existe déjà
        match sftp.stat(Path::new(normalized_path)) {
            Ok(stat) => {
                // Vérifier que c'est bien un répertoire
                if stat.is_dir() {
                    log::debug!("Répertoire distant {} existe déjà", normalized_path);
                    return Ok(());
                } else {
                    anyhow::bail!(
                        "Le chemin {} existe mais n'est pas un répertoire",
                        normalized_path
                    );
                }
            }
            Err(_) => {
                log::debug!(
                    "Répertoire distant {} n'existe pas, création récursive...",
                    normalized_path
                );
            }
        }

        // Créer récursivement les répertoires parents d'abord
        if let Some(parent) = Path::new(normalized_path).parent() {
            if let Some(parent_str) = parent.to_str() {
                if !parent_str.is_empty() && parent_str != "/" {
                    // Récursion pour créer le parent d'abord
                    self.ensure_remote_directory(parent_str)?;
                }
            }
        }

        // Créer le répertoire lui-même
        match sftp.mkdir(Path::new(normalized_path), 0o755) {
            Ok(()) => {
                log::info!("✅ Répertoire distant créé : {}", normalized_path);
                Ok(())
            }
            Err(e) => {
                // Vérifier si l'erreur est due au fait que le répertoire existe déjà
                match sftp.stat(Path::new(normalized_path)) {
                    Ok(stat) if stat.is_dir() => {
                        log::debug!(
                            "Répertoire {} existe déjà (créé concurremment)",
                            normalized_path
                        );
                        Ok(())
                    }
                    _ => {
                        log::error!(
                            "❌ Impossible de créer le répertoire {} : {}",
                            normalized_path,
                            e
                        );
                        anyhow::bail!(
                            "Échec création répertoire {}: {} (vérifiez les permissions)",
                            normalized_path,
                            e
                        )
                    }
                }
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

    /// Permet de sélectionner une clé SSH spécifique
    #[allow(dead_code)]
    pub fn set_ssh_key(&mut self, key: SshKey) {
        log::info!("🔑 Clé SSH sélectionnée: {}", key.description());
        self.selected_key = Some(key);
    }

    /// Récupère la clé SSH actuellement sélectionnée
    #[allow(dead_code)]
    pub fn get_selected_key(&self) -> Option<&SshKey> {
        self.selected_key.as_ref()
    }

    /// Permet à l'utilisateur de sélectionner une clé interactivement
    #[allow(dead_code)]
    pub fn select_ssh_key_interactive(&mut self) -> Result<()> {
        let key_manager =
            SshKeyManager::new().context("Impossible d'initialiser le gestionnaire de clés SSH")?;

        if let Some(selected_key) = key_manager.select_key_interactive()? {
            self.selected_key = Some(selected_key.clone());
            log::info!("🔑 Clé sélectionnée: {}", selected_key.description());
        }

        Ok(())
    }

    /// Teste les permissions d'écriture dans un répertoire distant
    pub fn test_write_permissions(&self, remote_dir: &str) -> Result<bool> {
        let sftp = self.sftp.as_ref().context("Client SFTP non initialisé")?;

        // Créer un fichier de test temporaire
        let test_file_name = format!(
            "{}/.xsshend_test_{}",
            remote_dir.trim_end_matches('/'),
            std::process::id()
        );
        let test_path = Path::new(&test_file_name);

        log::debug!("🔍 Test permissions d'écriture dans: {}", remote_dir);

        // Tenter de créer un fichier de test
        match sftp.create(test_path) {
            Ok(mut file) => {
                // Écrire quelques octets de test
                match file.write_all(b"test") {
                    Ok(()) => {
                        // Nettoyer le fichier de test
                        let _ = sftp.unlink(test_path);
                        log::debug!("✅ Permissions d'écriture confirmées pour: {}", remote_dir);
                        Ok(true)
                    }
                    Err(e) => {
                        log::debug!("❌ Échec écriture test dans {}: {}", remote_dir, e);
                        let _ = sftp.unlink(test_path);
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                log::debug!(
                    "❌ Impossible de créer fichier test dans {}: {}",
                    remote_dir,
                    e
                );
                Ok(false)
            }
        }
    }

    /// Trouve un répertoire accessible pour l'upload avec fallback
    pub fn find_accessible_directory(&self, preferred_dir: &str) -> Result<String> {
        log::debug!(
            "🔍 Recherche d'un répertoire accessible, préférence: {}",
            preferred_dir
        );

        // Liste des répertoires à tester par ordre de priorité
        let test_dirs = vec![
            preferred_dir.to_string(),
            format!("{}/xsshend", preferred_dir.trim_end_matches('/')),
            "/tmp".to_string(),
            format!("/tmp/{}", self.username),
            format!("/home/{}", self.username),
            format!("/Users/{}", self.username), // macOS
            "/var/tmp".to_string(),
        ];

        for test_dir in test_dirs {
            log::debug!("🔍 Test du répertoire: {}", test_dir);

            // Vérifier si le répertoire existe ou peut être créé
            match self.ensure_remote_directory(&test_dir) {
                Ok(()) => {
                    // Tester les permissions d'écriture
                    match self.test_write_permissions(&test_dir) {
                        Ok(true) => {
                            if test_dir != preferred_dir {
                                log::warn!(
                                    "⚠️  Utilisation du répertoire alternatif: {} (répertoire original {} inaccessible)",
                                    test_dir,
                                    preferred_dir
                                );
                            }
                            return Ok(test_dir);
                        }
                        Ok(false) => {
                            log::debug!("❌ Pas de permissions d'écriture dans: {}", test_dir);
                        }
                        Err(e) => {
                            log::debug!("❌ Erreur test permissions {}: {}", test_dir, e);
                        }
                    }
                }
                Err(e) => {
                    log::debug!(
                        "❌ Impossible de créer/accéder au répertoire {}: {}",
                        test_dir,
                        e
                    );
                }
            }
        }

        anyhow::bail!(
            "Aucun répertoire accessible trouvé pour l'utilisateur {}@{}. Répertoires testés: {:?}",
            self.username,
            self.host,
            vec![preferred_dir, "/tmp", &format!("/home/{}", self.username)]
        )
    }

    /// Expanse les chemins ~ et $HOME côté serveur SSH
    pub fn expand_remote_path(&self, remote_path: &str) -> Result<String> {
        log::debug!("🔍 Expansion du chemin distant: {}", remote_path);

        // Si le chemin ne contient pas de variables à expanser, le retourner tel quel
        if !remote_path.starts_with('~') && !remote_path.contains("$HOME") {
            return Ok(remote_path.to_string());
        }

        // Obtenir le répertoire home de l'utilisateur SSH distant
        let home_dir = self.get_remote_home_directory()?;

        let expanded_path = if remote_path.starts_with("~/") {
            // Remplacer ~/ par le répertoire home
            remote_path.replacen("~/", &format!("{}/", home_dir.trim_end_matches('/')), 1)
        } else if remote_path == "~" {
            // ~ seul = répertoire home
            home_dir
        } else if remote_path.contains("$HOME") {
            // Remplacer $HOME par le répertoire home
            remote_path.replace("$HOME", &home_dir)
        } else {
            remote_path.to_string()
        };

        log::debug!("✅ Chemin expansé: {} -> {}", remote_path, expanded_path);
        Ok(expanded_path)
    }

    /// Obtient le répertoire home de l'utilisateur distant via SSH
    pub fn get_remote_home_directory(&self) -> Result<String> {
        let session = self
            .session
            .as_ref()
            .context("Session SSH non initialisée")?;

        // Exécuter la commande 'echo $HOME' sur le serveur distant
        let mut channel = session.channel_session()?;
        channel.exec("echo $HOME")?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            anyhow::bail!(
                "Échec de la commande 'echo $HOME' sur le serveur distant (code: {})",
                exit_status
            );
        }

        let home_dir = output.trim().to_string();

        if home_dir.is_empty() {
            // Fallback: construire le chemin basé sur l'utilisateur
            let fallback_home = format!("/home/{}", self.username);
            log::warn!(
                "⚠️  $HOME vide sur le serveur distant, utilisation du fallback: {}",
                fallback_home
            );
            Ok(fallback_home)
        } else {
            log::debug!("✅ Répertoire home distant détecté: {}", home_dir);
            Ok(home_dir)
        }
    }
}

impl Drop for SshClient {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
