// Module de pool de connexions SSH pour optimiser les transferts parallèles
use crate::ssh::client::SshClient;
use crate::ssh::keys::SshKeyWithPassphrase;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};

/// Callback pour le feedback de progression en temps réel
pub type ProgressCallback = Arc<dyn Fn(&str, u64, TransferStatus) + Send + Sync>;

/// État d'un transfert pour feedback TUI
#[derive(Debug, Clone, PartialEq)]
pub enum TransferStatus {
    Pending,
    Connecting,
    Transferring,
    Completed,
    Failed(String),
}

impl TransferStatus {
    // Les couleurs sont maintenant gérées par le système de thème
}

/// Informations de connexion pour un serveur
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub username: String,
    pub host: String,
}

/// Message pour communication avec les threads de transfert
#[derive(Debug)]
pub enum TransferMessage {
    /// Lancer un transfert (file_path, destination, server_name)
    StartTransfer(String, String, String),
    /// Arrêter le thread
    Stop,
}

/// Résultat d'un transfert depuis un thread
#[derive(Debug)]
pub struct TransferResult {
    pub server_name: String,
    pub result: Result<u64>,
}

/// Thread de transfert dédié pour une cible
pub struct TransferThread {
    /// Handle du thread
    handle: JoinHandle<()>,
    /// Sender pour envoyer des commandes au thread
    sender: mpsc::Sender<TransferMessage>,
}

/// Pool de connexions SSH optimisé pour les transferts parallèles avec threads dédiés
pub struct SshConnectionPool {
    /// Cache des informations de connexion par alias
    connection_info: HashMap<String, ConnectionInfo>,
    /// Threads de transfert dédiés par serveur
    transfer_threads: HashMap<String, TransferThread>,
    /// Statistiques de connexions
    stats: Arc<Mutex<PoolStats>>,
    /// Clé SSH validée optionnelle à utiliser pour toutes les connexions
    validated_key: Option<SshKeyWithPassphrase>,
    /// Receiver pour collecter les résultats
    result_receiver: Option<mpsc::Receiver<TransferResult>>,
    /// Sender pour envoyer les résultats
    result_sender: mpsc::Sender<TransferResult>,
}

#[derive(Debug, Default)]
struct PoolStats {
    connections_created: usize,
    connections_reused: usize,
    active_transfers: usize,
}

impl SshConnectionPool {
    /// Créer un nouveau pool de connexions
    pub fn new() -> Self {
        let (result_sender, result_receiver) = mpsc::channel();
        SshConnectionPool {
            connection_info: HashMap::new(),
            transfer_threads: HashMap::new(),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            validated_key: None,
            result_receiver: Some(result_receiver),
            result_sender,
        }
    }

    /// Créer un nouveau pool de connexions avec une clé SSH validée
    pub fn new_with_validated_key(validated_key: SshKeyWithPassphrase) -> Self {
        let (result_sender, result_receiver) = mpsc::channel();
        SshConnectionPool {
            connection_info: HashMap::new(),
            transfer_threads: HashMap::new(),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            validated_key: Some(validated_key),
            result_receiver: Some(result_receiver),
            result_sender,
        }
    }

    /// Ajouter un serveur au pool et créer son thread dédié
    pub fn add_server(&mut self, alias: &str) -> Result<()> {
        let (username, host) = Self::parse_server_alias(alias)?;

        let info = ConnectionInfo { username, host };

        // Créer le thread dédié pour cette cible
        let transfer_thread = self.create_transfer_thread(alias, &info)?;

        self.connection_info.insert(alias.to_string(), info);
        self.transfer_threads
            .insert(alias.to_string(), transfer_thread);

        log::debug!("Serveur ajouté au pool avec thread dédié: {}", alias);
        Ok(())
    }

    /// Créer un thread de transfert dédié pour un serveur
    fn create_transfer_thread(
        &self,
        server_alias: &str,
        info: &ConnectionInfo,
    ) -> Result<TransferThread> {
        let (sender, receiver) = mpsc::channel::<TransferMessage>();
        let result_sender = self.result_sender.clone();
        let validated_key = self.validated_key.clone();
        let server_alias = server_alias.to_string();
        let connection_info = info.clone();

        let handle = thread::spawn(move || {
            let mut ssh_client: Option<SshClient> = None;

            // Boucle principale du thread
            while let Ok(message) = receiver.recv() {
                match message {
                    TransferMessage::StartTransfer(file_path, destination, server_name) => {
                        // Créer une nouvelle connexion SSH si nécessaire
                        if ssh_client.is_none() {
                            match Self::create_ssh_client(&connection_info, &validated_key) {
                                Ok(client) => {
                                    log::info!(
                                        "🔌 Connexion SSH créée pour thread {}",
                                        server_name
                                    );
                                    ssh_client = Some(client);
                                }
                                Err(e) => {
                                    log::error!(
                                        "❌ Impossible de créer la connexion SSH pour {}: {}",
                                        server_name,
                                        e
                                    );
                                    let _ = result_sender.send(TransferResult {
                                        server_name: server_name.clone(),
                                        result: Err(e),
                                    });
                                    continue;
                                }
                            }
                        }

                        // Effectuer le transfert
                        if let Some(ref mut client) = ssh_client {
                            let result = Self::perform_transfer(
                                client,
                                &file_path,
                                &destination,
                                &server_name,
                            );
                            let _ = result_sender.send(TransferResult {
                                server_name: server_name.clone(),
                                result,
                            });
                        }
                    }
                    TransferMessage::Stop => {
                        // Fermer la connexion SSH si elle existe
                        if let Some(mut client) = ssh_client.take() {
                            let _ = client.disconnect();
                            log::debug!("🔌 Connexion SSH fermée pour thread {}", server_alias);
                        }
                        break;
                    }
                }
            }

            log::debug!("Thread de transfert terminé pour {}", server_alias);
        });

        Ok(TransferThread { handle, sender })
    }

    /// Créer un client SSH avec les paramètres donnés
    fn create_ssh_client(
        info: &ConnectionInfo,
        validated_key: &Option<SshKeyWithPassphrase>,
    ) -> Result<SshClient> {
        let mut client = if let Some(key) = validated_key {
            log::info!(
                "🔑 Utilisation de la clé validée: {} pour {}@{}",
                key.key.description(),
                info.username,
                info.host
            );
            SshClient::new_with_validated_key(&info.host, &info.username, key.clone())
        } else {
            log::debug!(
                "🔑 Utilisation du comportement par défaut pour {}@{}",
                info.username,
                info.host
            );
            SshClient::new(&info.host, &info.username)
        }?;

        // Connexion avec retry
        let mut attempts = 0;
        let max_attempts = 2;
        let connection_timeout = std::time::Duration::from_secs(10);

        loop {
            attempts += 1;
            log::debug!(
                "Tentative de connexion {}/{} vers {}@{}",
                attempts,
                max_attempts,
                info.username,
                info.host
            );

            match client.connect_with_timeout(connection_timeout) {
                Ok(()) => {
                    log::info!(
                        "✅ Connexion SSH établie avec {}@{} - Tentative {}",
                        info.username,
                        info.host,
                        attempts
                    );
                    break;
                }
                Err(e) if attempts < max_attempts => {
                    log::warn!(
                        "⚠️ Tentative {} échouée pour {}@{}: {} - Retry...",
                        attempts,
                        info.username,
                        info.host,
                        e
                    );
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    continue;
                }
                Err(e) => {
                    return Err(e.context(format!(
                        "Échec connexion SSH vers {}@{} après {} tentatives",
                        info.username, info.host, max_attempts
                    )));
                }
            }
        }

        Ok(client)
    }

    /// Effectuer un transfert de fichier
    fn perform_transfer(
        client: &mut SshClient,
        file_path: &str,
        destination: &str,
        server_name: &str,
    ) -> Result<u64> {
        let path = Path::new(file_path);
        let full_destination = Self::build_full_destination_path(path, destination);

        log::info!(
            "🚀 Début transfert {} vers {} ({})",
            file_path,
            server_name,
            full_destination
        );

        match client.upload_file(path, &full_destination) {
            Ok(size) => {
                log::info!("✅ Transfert terminé pour {}: {} octets", server_name, size);
                Ok(size)
            }
            Err(e) => {
                log::error!("❌ Échec transfert vers {}: {}", server_name, e);
                Err(e)
            }
        }
    }

    /// Lancer un transfert vers un serveur spécifique via son thread dédié
    pub fn start_transfer(
        &self,
        server_alias: &str,
        file_path: &str,
        destination: &str,
        server_name: &str,
    ) -> Result<()> {
        let transfer_thread = self
            .transfer_threads
            .get(server_alias)
            .with_context(|| format!("Thread de transfert non trouvé pour {}", server_alias))?;

        transfer_thread
            .sender
            .send(TransferMessage::StartTransfer(
                file_path.to_string(),
                destination.to_string(),
                server_name.to_string(),
            ))
            .context("Impossible d'envoyer le message au thread de transfert")?;

        Ok(())
    }

    /// Upload parallèle d'un fichier vers plusieurs serveurs avec callback (nouvelle implémentation thread-based)
    pub fn upload_file_parallel_with_callback(
        &mut self,
        file_path: &Path,
        servers: &[(String, &crate::config::HostEntry)],
        destination: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        log::info!(
            "Début upload parallèle avec threads dédiés: {} vers {} serveurs",
            file_path.display(),
            servers.len()
        );

        // Mettre à jour les stats - début des transferts
        if let Ok(mut stats) = self.stats.lock() {
            stats.active_transfers = servers.len();
        }

        let file_path_str = file_path.to_string_lossy().to_string();

        // Lancer les transferts sur tous les threads dédiés
        for (server_name, host_entry) in servers {
            if let Some(ref callback) = progress_callback {
                callback(server_name, 0, TransferStatus::Pending);
            }

            if let Err(e) =
                self.start_transfer(&host_entry.alias, &file_path_str, destination, server_name)
            {
                log::error!(
                    "❌ Impossible de lancer le transfert vers {}: {}",
                    server_name,
                    e
                );
                if let Some(ref callback) = progress_callback {
                    callback(server_name, 0, TransferStatus::Failed(e.to_string()));
                }
            }
        }

        // Collecter les résultats depuis tous les threads
        let mut results = Vec::new();
        let receiver = self
            .result_receiver
            .take()
            .ok_or_else(|| anyhow::anyhow!("Receiver déjà utilisé"))?;

        for _ in 0..servers.len() {
            match receiver.recv() {
                Ok(result) => {
                    if let Some(ref callback) = progress_callback {
                        match &result.result {
                            Ok(size) => {
                                callback(&result.server_name, *size, TransferStatus::Completed);
                            }
                            Err(e) => {
                                callback(
                                    &result.server_name,
                                    0,
                                    TransferStatus::Failed(e.to_string()),
                                );
                            }
                        }
                    }
                    results.push(result);
                }
                Err(e) => {
                    log::error!("❌ Erreur lors de la réception des résultats: {}", e);
                    break;
                }
            }
        }

        // Restaurer le receiver pour les prochains appels
        self.result_receiver = Some(receiver);

        // Analyser les résultats
        let mut success_count = 0;
        let mut failed_servers = Vec::new();

        for result in results {
            match result.result {
                Ok(size) => {
                    success_count += 1;
                    log::info!(
                        "✅ Upload réussi vers {} ({} octets)",
                        result.server_name,
                        size
                    );
                }
                Err(e) => {
                    failed_servers.push(result.server_name.clone());
                    log::error!("❌ Upload échoué vers {} : {}", result.server_name, e);
                }
            }
        }

        // Remettre à zéro les transferts actifs
        if let Ok(mut stats) = self.stats.lock() {
            stats.active_transfers = 0;
        }

        if success_count > 0 {
            log::info!(
                "Upload parallèle terminé : {}/{} serveurs réussis",
                success_count,
                servers.len()
            );
            if !failed_servers.is_empty() {
                log::warn!("Serveurs échoués : {}", failed_servers.join(", "));
            }
            Ok(()) // Considérer comme succès si au moins un serveur a réussi
        } else {
            let error_msg = format!("Tous les uploads ont échoué ({} serveurs)", servers.len());
            log::error!("{}", error_msg);
            Err(anyhow::anyhow!(error_msg))
        }
    }

    /// Arrêter tous les threads de transfert
    pub fn stop_all_threads(&mut self) -> Result<()> {
        log::info!("Arrêt de tous les threads de transfert...");

        // Envoyer le signal d'arrêt à tous les threads
        for (server_alias, transfer_thread) in self.transfer_threads.drain() {
            if let Err(e) = transfer_thread.sender.send(TransferMessage::Stop) {
                log::warn!(
                    "Impossible d'envoyer le signal d'arrêt à {}: {}",
                    server_alias,
                    e
                );
            }

            // Attendre que le thread se termine
            if let Err(e) = transfer_thread.handle.join() {
                log::warn!(
                    "Erreur lors de l'attente du thread {}: {:?}",
                    server_alias,
                    e
                );
            } else {
                log::debug!("Thread {} terminé avec succès", server_alias);
            }
        }

        log::info!("Tous les threads de transfert ont été arrêtés");
        Ok(())
    }

    /// Parse un alias serveur au format "user@host" ou "user@host:port"
    fn parse_server_alias(alias: &str) -> Result<(String, String)> {
        if let Some(at_pos) = alias.find('@') {
            let username = alias[..at_pos].to_string();
            let host = alias[at_pos + 1..].to_string();
            Ok((username, host))
        } else {
            anyhow::bail!(
                "Alias serveur invalide '{}' - format attendu: user@host",
                alias
            );
        }
    }

    /// Obtenir les statistiques du pool
    pub fn get_stats(&self) -> (usize, usize, usize) {
        if let Ok(stats) = self.stats.lock() {
            (
                stats.connections_created,
                stats.connections_reused,
                stats.active_transfers,
            )
        } else {
            (0, 0, 0)
        }
    }

    /// Initialiser le pool avec tous les serveurs de la configuration
    pub fn initialize_with_hosts(
        &mut self,
        hosts: &[(String, &crate::config::HostEntry)],
    ) -> Result<()> {
        for (_, host_entry) in hosts {
            self.add_server(&host_entry.alias)?;
        }
        log::info!("Pool initialisé avec {} serveurs", hosts.len());
        Ok(())
    }

    /// Nettoyer toutes les connexions actives du pool
    pub fn cleanup_connections(&mut self) -> Result<()> {
        self.stop_all_threads()?;
        log::info!("Pool SSH nettoyé - tous les threads arrêtés");
        Ok(())
    }

    /// Construit le chemin de destination complet pour un fichier
    fn build_full_destination_path(file_path: &Path, destination: &str) -> String {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");

        if destination.ends_with('/') {
            format!("{}{}", destination, file_name)
        } else {
            format!("{}/{}", destination, file_name)
        }
    }
}

impl Drop for SshConnectionPool {
    fn drop(&mut self) {
        log::debug!("Nettoyage automatique du pool SSH lors de la destruction");
        let _ = self.stop_all_threads();
    }
}
