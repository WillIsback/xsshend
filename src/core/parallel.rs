// Module de pool de connexions SSH pour optimiser les transferts parallèles
use crate::ssh::client::SshClient;
use crate::ssh::keys::SshKey;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

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

/// Pool de connexions SSH optimisé pour les transferts parallèles
pub struct SshConnectionPool {
    /// Cache des informations de connexion par alias
    connection_info: HashMap<String, ConnectionInfo>,
    /// Cache des connexions SSH actives (pour réutilisation)
    active_connections: Arc<Mutex<HashMap<String, SshClient>>>,
    /// Statistiques de connexions
    stats: Arc<Mutex<PoolStats>>,
    /// Clé SSH optionnelle à utiliser pour toutes les connexions
    ssh_key: Option<SshKey>,
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
        SshConnectionPool {
            connection_info: HashMap::new(),
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            ssh_key: None,
        }
    }

    /// Créer un nouveau pool de connexions avec une clé SSH spécifique
    pub fn new_with_key(ssh_key: SshKey) -> Self {
        SshConnectionPool {
            connection_info: HashMap::new(),
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            ssh_key: Some(ssh_key),
        }
    }

    /// Ajouter un serveur au pool
    pub fn add_server(&mut self, alias: &str) -> Result<()> {
        let (username, host) = Self::parse_server_alias(alias)?;

        let info = ConnectionInfo { username, host };

        self.connection_info.insert(alias.to_string(), info);
        log::debug!("Serveur ajouté au pool: {}", alias);
        Ok(())
    }

    /// Créer ou réutiliser une connexion SSH pour un serveur
    pub fn get_or_create_connection(&self, server_alias: &str) -> Result<SshClient> {
        let info = self
            .connection_info
            .get(server_alias)
            .with_context(|| format!("Serveur '{}' non trouvé dans le pool", server_alias))?;

        // Dans cette implémentation, on crée toujours une nouvelle connexion pour éviter
        // les problèmes de concurrence et garantir la stabilité

        log::info!(
            "🔌 Tentative de connexion SSH vers {}@{} (alias: {})",
            info.username,
            info.host,
            server_alias
        );

        // Mettre à jour les stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.connections_created += 1;
        }

        let mut client = if let Some(ref ssh_key) = self.ssh_key {
            // Utiliser la clé SSH spécifiée
            log::info!(
                "🔑 Utilisation de la clé spécifiée: {} pour {}@{}",
                ssh_key.description(),
                info.username,
                info.host
            );
            SshClient::new_with_key(&info.host, &info.username, ssh_key.clone())
        } else {
            // Utiliser le comportement par défaut
            SshClient::new(&info.host, &info.username)
        }
        .with_context(|| format!("Impossible de créer le client SSH pour {}", server_alias))?;

        // Tentative de connexion avec retry pour plus de robustesse et timeout réduit
        let mut attempts = 0;
        let max_attempts = 2; // Réduire le nombre de tentatives pour éviter les blocages
        let connection_timeout = std::time::Duration::from_secs(5); // Timeout réduit

        loop {
            attempts += 1;
            log::debug!(
                "Tentative de connexion {}/{} vers {} avec timeout {:?}",
                attempts,
                max_attempts,
                server_alias,
                connection_timeout
            );

            match client.connect_with_timeout(connection_timeout) {
                Ok(()) => {
                    log::info!(
                        "✅ Connexion SSH établie avec {} ({}@{}) - Tentative {}",
                        server_alias,
                        info.username,
                        info.host,
                        attempts
                    );
                    break;
                }
                Err(e) if attempts < max_attempts => {
                    log::warn!(
                        "⚠️ Tentative {} échouée pour {} : {} - Retry dans 1s...",
                        attempts,
                        server_alias,
                        e
                    );
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    continue;
                }
                Err(e) => {
                    log::error!(
                        "❌ Impossible de se connecter à {} après {} tentatives: {}",
                        server_alias,
                        max_attempts,
                        e
                    );
                    return Err(e.context(format!(
                        "Échec connexion SSH vers {} ({}@{}) après {} tentatives",
                        server_alias, info.username, info.host, max_attempts
                    )));
                }
            }
        }

        Ok(client)
    }

    /// Upload parallèle d'un fichier vers plusieurs serveurs avec callback
    pub fn upload_file_parallel_with_callback(
        &self,
        file_path: &Path,
        servers: &[(String, &crate::config::HostEntry)],
        destination: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        use rayon::prelude::*;

        log::info!(
            "Début upload parallèle: {} vers {} serveurs",
            file_path.display(),
            servers.len()
        );

        // Mettre à jour les stats - début des transferts
        if let Ok(mut stats) = self.stats.lock() {
            stats.active_transfers = servers.len();
        }

        // Lancer les uploads en parallèle avec rayon et collecter tous les résultats
        let results: Vec<Result<()>> = servers
            .par_iter()
            .map(|(name, host)| {
                self.upload_to_single_server_with_callback(
                    file_path,
                    &host.alias,
                    name,
                    destination,
                    progress_callback.clone(),
                )
            })
            .collect();

        // Remettre à zéro les transferts actifs
        if let Ok(mut stats) = self.stats.lock() {
            stats.active_transfers = 0;
        }

        // Analyser les résultats et gérer les erreurs gracieusement
        let mut success_count = 0;
        let mut failed_servers = Vec::new();

        for (i, result) in results.iter().enumerate() {
            let (server_name, _) = &servers[i];
            match result {
                Ok(()) => {
                    success_count += 1;
                    log::info!("✅ Upload réussi vers {}", server_name);
                }
                Err(e) => {
                    failed_servers.push(server_name.clone());
                    log::error!("❌ Upload échoué vers {} : {}", server_name, e);
                }
            }
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

    /// Upload vers un serveur unique avec callback de progression
    fn upload_to_single_server_with_callback(
        &self,
        file_path: &Path,
        server_alias: &str,
        server_name: &str,
        destination: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        // Obtenir la taille du fichier en premier
        let _file_size = match std::fs::metadata(file_path) {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                if let Some(ref callback) = progress_callback {
                    callback(
                        server_name,
                        0,
                        TransferStatus::Failed(format!("Impossible de lire le fichier: {}", e)),
                    );
                }
                return Err(anyhow::anyhow!(
                    "Impossible de lire les métadonnées du fichier: {}",
                    e
                ));
            }
        };

        // Notifier le début de la connexion avec la taille du fichier
        if let Some(ref callback) = progress_callback {
            callback(server_name, 0, TransferStatus::Connecting);
        }

        // Construire le chemin de destination complet
        let full_destination = Self::build_full_destination_path(file_path, destination);
        log::debug!("Chemin destination complet: {}", full_destination);

        // Créer une nouvelle connexion pour ce transfert avec retry
        let mut client = match self.get_or_create_connection(server_alias) {
            Ok(client) => client,
            Err(e) => {
                if let Some(ref callback) = progress_callback {
                    callback(
                        server_name,
                        0,
                        TransferStatus::Failed(format!("Connexion échouée: {}", e)),
                    );
                }
                return Err(e.context(format!(
                    "Impossible d'obtenir connexion pour {}",
                    server_name
                )));
            }
        };

        // Notifier le début du transfert avec taille du fichier
        if let Some(ref callback) = progress_callback {
            callback(server_name, 0, TransferStatus::Transferring);
        }

        // Effectuer le transfert avec gestion d'erreur complète
        let upload_result_size = match client.upload_file(file_path, &full_destination) {
            Ok(size) => {
                log::info!("✅ Upload réussi pour {} : {} octets", server_name, size);
                size
            }
            Err(e) => {
                let error_msg = format!("Transfert échoué: {}", e);
                log::error!("❌ {} - {}", server_name, error_msg);

                if let Some(ref callback) = progress_callback {
                    callback(server_name, 0, TransferStatus::Failed(error_msg.clone()));
                }
                // Tenter de fermer proprement la connexion même en cas d'erreur
                let _ = client.disconnect();
                return Err(e.context(format!(
                    "Échec upload vers {} - Détails: {}",
                    server_name, error_msg
                )));
            }
        };

        // Fermer la connexion immédiatement après le transfert
        if let Err(e) = client.disconnect() {
            log::warn!("Avertissement fermeture connexion {}: {}", server_name, e);
            // Ne pas faire échouer le transfert pour un problème de fermeture
        }

        // Notifier la fin du transfert avec la taille réelle uploadée
        if let Some(ref callback) = progress_callback {
            callback(server_name, upload_result_size, TransferStatus::Completed);
        }

        log::info!(
            "✅ {} - {} octets uploadés",
            server_name,
            upload_result_size
        );
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
        if let Ok(mut connections) = self.active_connections.lock() {
            for (alias, mut connection) in connections.drain() {
                if let Err(e) = connection.disconnect() {
                    log::warn!("Erreur fermeture connexion {}: {}", alias, e);
                }
            }
            log::info!(
                "Pool SSH nettoyé - {} connexions fermées",
                connections.len()
            );
        }
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
