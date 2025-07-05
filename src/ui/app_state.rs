use super::hierarchical_selector::HierarchicalServerSelector;
use crate::config::HostEntry;
use crate::core::parallel::TransferStatus;
use anyhow::Result;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Structure pour suivre le progrès d'un transfert vers un serveur
#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub status: TransferStatus,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub speed: f64, // octets/seconde
    pub host_alias: String,
    pub file_name: String, // Nom du fichier en cours de transfert
    pub eta: Option<Duration>,
    pub error_message: Option<String>,
}

/// États du workflow TUI multi-étapes
#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    FileSelection,
    ServerSelection,
    DestinationInput,
    UploadProgress,
}

/// État global de l'application TUI
pub struct AppState {
    pub current_screen: AppScreen,
    pub selected_files: Vec<PathBuf>,
    // pub available_hosts: HashMap<String, HostEntry>, // Unused field
    pub selected_hosts: Vec<(String, HostEntry)>,
    pub hierarchical_selector: Option<HierarchicalServerSelector>,
    pub destination: String,
    pub transfers: HashMap<String, TransferProgress>,
    pub start_time: Option<Instant>,
    pub is_paused: bool,
    pub show_logs: bool,
    pub logs: Vec<String>,
    pub log_buffer: Arc<Mutex<Vec<String>>>, // Buffer de logs partagé avec le logger
    pub selected_transfer: Option<String>,
    pub file_selection_cursor: usize,
    // pub server_selection_cursor: usize, // Unused field
    pub current_file_path: String,
    pub destination_input: String,
    pub should_quit: bool,
    // Compteur de fichiers transférés
    pub completed_files_count: usize,
    pub total_files_count: usize,
}

impl Default for AppState {
    fn default() -> Self {
        let log_buffer = Arc::new(Mutex::new(Vec::new()));
        Self {
            current_screen: AppScreen::FileSelection,
            selected_files: Vec::new(),
            selected_hosts: Vec::new(),
            hierarchical_selector: None,
            destination: "/tmp/".to_string(),
            transfers: HashMap::new(),
            start_time: None,
            is_paused: false,
            show_logs: false,
            logs: Vec::new(),
            log_buffer,
            selected_transfer: None,
            file_selection_cursor: 0,
            current_file_path: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .to_string_lossy()
                .to_string(),
            destination_input: "/tmp/".to_string(),
            should_quit: false,
            completed_files_count: 0,
            total_files_count: 0,
        }
    }
}

impl AppState {
    /// Crée un nouvel AppState avec un buffer de logs spécifique
    pub fn new_with_log_buffer(log_buffer: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            log_buffer,
            ..Default::default()
        }
    }

    /// Navigation entre les écrans
    pub fn next_screen(&mut self) -> Result<()> {
        match self.current_screen {
            AppScreen::FileSelection => {
                if !self.selected_files.is_empty() {
                    self.current_screen = AppScreen::ServerSelection;
                }
            }
            AppScreen::ServerSelection => {
                if !self.selected_hosts.is_empty() {
                    self.current_screen = AppScreen::DestinationInput;
                }
            }
            AppScreen::DestinationInput => {
                // Finaliser la destination et passer aux transferts
                self.destination = self.destination_input.clone();
                self.current_screen = AppScreen::UploadProgress;
                self.initialize_transfers()?;
            }
            AppScreen::UploadProgress => {
                // Déjà au dernier écran
            }
        }
        Ok(())
    }

    pub fn previous_screen(&mut self) {
        match self.current_screen {
            AppScreen::FileSelection => {
                // Déjà au premier écran
            }
            AppScreen::ServerSelection => {
                self.current_screen = AppScreen::FileSelection;
            }
            AppScreen::DestinationInput => {
                self.current_screen = AppScreen::ServerSelection;
            }
            AppScreen::UploadProgress => {
                self.current_screen = AppScreen::DestinationInput;
            }
        }
    }

    // Unused methods - commented out for optimization
    // pub fn add_file(&mut self, file: PathBuf) {
    //     if !self.selected_files.contains(&file) {
    //         self.selected_files.push(file);
    //     }
    // }

    // pub fn remove_file(&mut self, index: usize) {
    //     if index < self.selected_files.len() {
    //         self.selected_files.remove(index);
    //     }
    // }

    pub fn toggle_file(&mut self, file: PathBuf) {
        if let Some(pos) = self.selected_files.iter().position(|f| f == &file) {
            self.selected_files.remove(pos);
        } else {
            self.selected_files.push(file);
        }
    }

    // Unused methods - commented out for optimization
    // pub fn toggle_host(&mut self, host_name: String) {
    //     if let Some(pos) = self.selected_hosts.iter().position(|(name, _)| name == &host_name) {
    //         self.selected_hosts.remove(pos);
    //     } else if let Some(host_entry) = self.available_hosts.get(&host_name) {
    //         self.selected_hosts.push((host_name, host_entry.clone()));
    //     }
    // }

    // pub fn is_host_selected(&self, host_name: &str) -> bool {
    //     self.selected_hosts.iter().any(|(name, _)| name == host_name)
    // }

    /// Initialise les transferts pour l'écran de progression
    fn initialize_transfers(&mut self) -> Result<()> {
        self.transfers.clear();
        self.start_time = Some(Instant::now());

        // Initialiser le compteur de fichiers
        self.completed_files_count = 0;
        self.total_files_count = self.selected_files.len();

        // Pour l'instant, initialiser avec des tailles par fichier
        // La taille totale sera mise à jour lors du transfert de chaque fichier
        for (host_name, host_entry) in &self.selected_hosts {
            self.transfers.insert(
                host_name.clone(),
                TransferProgress {
                    host_alias: host_entry.alias.clone(),
                    file_name: "En attente...".to_string(), // Sera mis à jour lors du transfert
                    bytes_transferred: 0,
                    total_bytes: 0, // Sera mis à jour lors du transfert du fichier
                    status: TransferStatus::Pending,
                    speed: 0.0,
                    eta: None,
                    error_message: None,
                },
            );
        }

        self.add_log("🚀 Initialisation des transferts...");
        Ok(())
    }

    /// Mise à jour du progrès d'un transfert avec informations complètes
    pub fn update_progress_with_file(
        &mut self,
        host_name: &str,
        bytes_transferred: u64,
        status: TransferStatus,
        file_name: Option<&str>,
    ) {
        // Préparer les variables pour le log
        let mut should_log = false;
        let mut log_message = String::new();

        if let Some(transfer) = self.transfers.get_mut(host_name) {
            let old_status = transfer.status.clone();
            let old_bytes = transfer.bytes_transferred;
            let old_file_name = transfer.file_name.clone();

            transfer.bytes_transferred = bytes_transferred;
            transfer.status = status.clone();

            // Mettre à jour le nom du fichier si fourni
            if let Some(file) = file_name {
                if file != old_file_name {
                    // Nouveau fichier détecté - calculer la taille de ce fichier spécifique
                    if let Some(current_file) = self
                        .selected_files
                        .iter()
                        .find(|f| f.file_name().and_then(|n| n.to_str()).unwrap_or("") == file)
                    {
                        if let Ok(metadata) = std::fs::metadata(current_file) {
                            transfer.total_bytes = metadata.len();
                            transfer.bytes_transferred = 0; // Nouveau fichier commence à 0
                            log::debug!(
                                "Nouveau fichier détecté: {} ({} octets)",
                                file,
                                transfer.total_bytes
                            );
                        }
                    }
                }
                transfer.file_name = file.to_string();
            }

            // Forcer la synchronisation complète pour les transferts terminés
            match &status {
                TransferStatus::Completed => {
                    transfer.bytes_transferred = transfer.total_bytes;
                    transfer.eta = None;
                    should_log = true;
                    log_message = format!(
                        "✅ {} : Transfert terminé ({})",
                        host_name,
                        format_bytes(transfer.total_bytes)
                    );
                }
                TransferStatus::Failed(err) => {
                    transfer.error_message = Some(err.clone());
                    transfer.eta = None;
                    should_log = true;
                    log_message = format!("❌ {} : Erreur - {}", host_name, err);
                }
                _ => {
                    // Calculer la vitesse si le transfert est en cours
                    if let Some(start_time) = self.start_time {
                        let elapsed = start_time.elapsed();
                        if elapsed.as_secs() > 0 && bytes_transferred > 0 {
                            transfer.speed = bytes_transferred as f64 / elapsed.as_secs_f64();

                            // Calculer l'ETA
                            if transfer.speed > 0.0 && bytes_transferred < transfer.total_bytes {
                                let remaining_bytes = transfer.total_bytes - bytes_transferred;
                                let eta_seconds = remaining_bytes as f64 / transfer.speed;
                                transfer.eta = Some(Duration::from_secs_f64(eta_seconds));
                            }
                        }
                    }

                    // Log de progression uniquement si changement significatif
                    if bytes_transferred > old_bytes || old_status != status {
                        let progress_pct = if transfer.total_bytes > 0 {
                            (bytes_transferred * 100) / transfer.total_bytes
                        } else {
                            0
                        };

                        should_log = true;
                        log_message = format!(
                            "{}: {}% ({} / {}) - {}",
                            host_name,
                            progress_pct,
                            format_bytes(bytes_transferred),
                            format_bytes(transfer.total_bytes),
                            transfer.file_name
                        );
                    }
                }
            }
        }

        // Ajouter au log après avoir libéré l'emprunt mutable
        if should_log {
            self.add_log(&log_message);
        }
    }

    /// Ajouter un message au log
    pub fn add_log(&mut self, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let formatted_message = format!("[{}] {}", timestamp, message);

        // Ajouter au buffer local
        self.logs.push(formatted_message.clone());

        // Ajouter aussi au buffer partagé pour cohérence
        if let Ok(mut shared_logs) = self.log_buffer.lock() {
            shared_logs.push(formatted_message);

            // Garder seulement les 1000 derniers logs
            if shared_logs.len() > 1000 {
                shared_logs.remove(0);
            }
        }

        // Garder seulement les 1000 derniers logs locaux aussi
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    /// Synchronise les logs depuis le buffer partagé
    pub fn sync_logs_from_shared_buffer(&mut self) {
        if let Ok(shared_logs) = self.log_buffer.lock() {
            self.logs = shared_logs.clone();
        }
    }

    /// Obtenir le résumé des transferts
    pub fn get_summary(&self) -> (usize, usize, usize) {
        let completed = self
            .transfers
            .values()
            .filter(|t| t.status == TransferStatus::Completed)
            .count();
        let failed = self
            .transfers
            .values()
            .filter(|t| matches!(t.status, TransferStatus::Failed(_)))
            .count();
        let total = self.transfers.len();
        (completed, failed, total)
    }

    /// Obtenir la vitesse totale
    pub fn get_total_speed(&self) -> f64 {
        self.transfers
            .values()
            .filter(|t| t.status == TransferStatus::Transferring)
            .map(|t| t.speed)
            .sum()
    }

    // Unused method - commented out for optimization
    // pub fn get_overall_eta(&self) -> Option<Duration> {
    //     let total_remaining: u64 = self.transfers.values()
    //         .filter(|t| t.status != TransferStatus::Completed && t.status != TransferStatus::Failed)
    //         .map(|t| t.total_bytes - t.bytes_transferred)
    //         .sum();

    //     let total_speed = self.get_total_speed();

    //     if total_speed > 0.0 && total_remaining > 0 {
    //         Some(Duration::from_secs_f64(total_remaining as f64 / total_speed))
    //     } else {
    //         None
    //     }
    // }

    /// Vérifier si tous les transferts sont terminés
    pub fn are_all_transfers_complete(&self) -> bool {
        self.transfers.values().all(|t| {
            t.status == TransferStatus::Completed || matches!(t.status, TransferStatus::Failed(_))
        })
    }

    /// Obtenir les fichiers dans le répertoire courant pour la sélection
    pub fn get_current_directory_files(&self) -> Result<Vec<PathBuf>> {
        let path = PathBuf::from(&self.current_file_path);
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.is_file() {
                    files.push(file_path);
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Changer le répertoire courant
    pub fn change_directory(&mut self, new_path: PathBuf) -> Result<()> {
        if new_path.is_dir() {
            self.current_file_path = new_path.to_string_lossy().to_string();
            self.file_selection_cursor = 0;
        }
        Ok(())
    }

    /// Initialise le sélecteur hiérarchique avec la configuration des hosts
    pub fn init_hierarchical_selector(
        &mut self,
        hosts_config: &crate::config::HostsConfig,
    ) -> Result<()> {
        use super::hierarchical_selector::HierarchicalServerSelector;
        self.hierarchical_selector = Some(HierarchicalServerSelector::new(hosts_config)?);
        Ok(())
    }

    /// Initialise le sélecteur hiérarchique avec filtrage par connectivité
    pub fn init_hierarchical_selector_filtered(
        &mut self,
        hosts_config: &crate::config::HostsConfig,
        timeout_secs: u64,
    ) -> Result<()> {
        use super::hierarchical_selector::HierarchicalServerSelector;
        self.hierarchical_selector = Some(HierarchicalServerSelector::new_with_connectivity(
            hosts_config,
            timeout_secs,
        )?);
        Ok(())
    }

    /// Met à jour les serveurs sélectionnés depuis le sélecteur hiérarchique
    pub fn sync_selected_hosts_from_hierarchical(&mut self) {
        if let Some(ref selector) = self.hierarchical_selector {
            self.selected_hosts = selector.get_selected_hosts();
        }
    }

    /// Réinitialise complètement l'état de l'application (pour retour au début)
    pub fn reset_to_beginning(&mut self) {
        // Sauvegarder le sélecteur hiérarchique pour éviter de le recréer
        let hierarchical_selector = self.hierarchical_selector.take();

        // Réinitialiser vers l'état par défaut
        *self = Self::default();

        // Restaurer le sélecteur hiérarchique
        self.hierarchical_selector = hierarchical_selector;

        // Réinitialiser le sélecteur s'il existe
        if let Some(ref mut selector) = self.hierarchical_selector {
            selector.reset_selection();
        }

        self.add_log("🔄 Application réinitialisée - retour au début du workflow");
    }

    /// Incrémente le compteur de fichiers complétés
    pub fn increment_completed_files(&mut self) {
        if self.completed_files_count < self.total_files_count {
            self.completed_files_count += 1;
        }
    }

    /// Obtient le compteur de fichiers sous forme de chaîne (ex: "2/10")
    pub fn get_files_progress_string(&self) -> String {
        format!("{}/{}", self.completed_files_count, self.total_files_count)
    }
}

/// Fonction utilitaire pour formater les bytes
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{:.0}{}", size, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}
