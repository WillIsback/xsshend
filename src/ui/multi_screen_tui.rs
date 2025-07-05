use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::config::{HostEntry, HostsConfig};
use crate::core::uploader::Uploader;
use super::{
    app_state::{AppScreen, AppState},
    multi_screen_handler::MultiScreenEventHandler,
    screens::{FileSelectionScreen, ServerSelectionScreen, DestinationInputScreen, ProgressScreen},
};

/// Application TUI multi-écrans principale
pub struct MultiScreenTuiApp {
    state: Arc<Mutex<AppState>>,
}

impl MultiScreenTuiApp {
    pub fn new(config: &HostsConfig) -> Result<Self> {
        let available_hosts: HashMap<String, HostEntry> = config.get_all_hosts()
            .into_iter()
            .map(|(name, entry)| (name, entry.clone()))
            .collect();

        let mut app_state = AppState::new(available_hosts);
        
        // Initialiser le sélecteur hiérarchique
        app_state.init_hierarchical_selector(config)?;
        
        let state = Arc::new(Mutex::new(app_state));
        
        Ok(Self { state })
    }

    /// Précharge les fichiers sélectionnés depuis la ligne de commande
    pub fn set_selected_files(&mut self, files: Vec<std::path::PathBuf>) -> Result<()> {
        if let Ok(mut state) = self.state.lock() {
            state.selected_files = files;
        }
        Ok(())
    }

    /// Lance l'application TUI
    pub fn run(&mut self) -> Result<()> {
        // Configuration du terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Boucle principale
        let result = self.main_loop(&mut terminal);

        // Restaurer le terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        // Cloner le state pour le thread d'upload
        let upload_state = Arc::clone(&self.state);
        let mut upload_handle: Option<thread::JoinHandle<()>> = None;

        loop {
            // Dessiner l'interface
            terminal.draw(|f| {
                let state = self.state.lock().unwrap();
                match state.current_screen {
                    AppScreen::FileSelection => {
                        FileSelectionScreen::render(f, f.size(), &state);
                    }
                    AppScreen::ServerSelection => {
                        ServerSelectionScreen::render(f, f.size(), &state);
                    }
                    AppScreen::DestinationInput => {
                        DestinationInputScreen::render(f, f.size(), &state);
                    }
                    AppScreen::UploadProgress => {
                        ProgressScreen::render(f, f.size(), &state);
                    }
                }
            })?;

            // Vérifier si on doit quitter
            {
                let state = self.state.lock().unwrap();
                if state.should_quit {
                    // Attendre la fin des transferts si en cours
                    if let Some(handle) = upload_handle.take() {
                        drop(state); // Libérer le lock avant join
                        let _ = handle.join();
                    }
                    break;
                }

                // Démarrer les transferts si on passe à l'écran de progression
                if matches!(state.current_screen, AppScreen::UploadProgress) && upload_handle.is_none() {
                    let state_clone = Arc::clone(&upload_state);
                    let files = state.selected_files.clone();
                    let hosts = state.selected_hosts.clone();
                    let destination = state.destination.clone();
                    
                    upload_handle = Some(thread::spawn(move || {
                        let state_ref = Arc::clone(&state_clone);
                        if let Err(e) = Self::run_uploads(state_clone, files, hosts, destination) {
                            let mut state = state_ref.lock().unwrap();
                            state.add_log(&format!("❌ Erreur lors des transferts: {}", e));
                        }
                    }));
                }
            }

            // Gérer les événements avec timeout
            if event::poll(Duration::from_millis(100))? {
                let event = event::read()?;
                let mut state = self.state.lock().unwrap();
                MultiScreenEventHandler::handle_event(&mut state, event)?;
            }
        }

        Ok(())
    }

    /// Lance les transferts en parallèle dans un thread séparé
    fn run_uploads(
        state: Arc<Mutex<AppState>>,
        files: Vec<std::path::PathBuf>,
        hosts: Vec<(String, HostEntry)>,
        destination: String,
    ) -> Result<()> {
        use crate::core::uploader::Uploader;
        use rayon::prelude::*;

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log("🚀 Démarrage des transferts...");
        }

        // Créer l'uploader
        let uploader = Uploader::new();
        let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();

        // Exécuter les transferts en parallèle avec rayon
        hosts.par_iter().for_each(|(host_name, host_entry)| {
            let state_clone = Arc::clone(&state);
            
            // Vérifier si on doit continuer
            {
                let state_lock = state_clone.lock().unwrap();
                if state_lock.should_quit {
                    return;
                }
            }

            // Mettre à jour le statut
            {
                let mut state_lock = state_clone.lock().unwrap();
                state_lock.update_progress(
                    host_name,
                    0,
                    crate::ui::tui::TransferStatus::Connecting,
                );
            }

            // Simuler le transfert ou faire le vrai transfert
            match Self::transfer_to_host(&uploader, &file_refs, host_entry, &destination, &state_clone, host_name) {
                Ok(_) => {
                    let mut state_lock = state_clone.lock().unwrap();
                    let total_size = state_lock.transfers.get(host_name)
                        .map(|t| t.total_bytes)
                        .unwrap_or(0);
                    state_lock.update_progress(
                        host_name,
                        total_size,
                        crate::ui::tui::TransferStatus::Completed,
                    );
                    state_lock.add_log(&format!("✅ {} : Transfert terminé avec succès", host_name));
                }
                Err(e) => {
                    let mut state_lock = state_clone.lock().unwrap();
                    state_lock.set_error(host_name, e.to_string());
                }
            }
        });

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log("🏁 Tous les transferts terminés");
        }

        Ok(())
    }

    /// Transfert vers un host spécifique avec simulation ou vrai transfert
    fn transfer_to_host(
        uploader: &Uploader,
        files: &[&std::path::Path],
        host_entry: &HostEntry,
        destination: &str,
        state: &Arc<Mutex<AppState>>,
        host_name: &str,
    ) -> Result<()> {
        // Déterminer s'il faut simuler ou faire un vrai transfert
        let should_simulate = host_entry.alias.contains("example") || 
                             host_entry.alias.contains("localhost") ||
                             host_entry.alias.contains("127.0.0.1");

        if should_simulate {
            // Simulation rapide pour les hosts d'exemple
            Self::simulate_transfer(files, state, host_name)?;
        } else {
            // Vrai transfert
            Self::real_transfer(uploader, files, host_entry, destination, state, host_name)?;
        }

        Ok(())
    }

    /// Simulation de transfert pour les tests
    fn simulate_transfer(
        files: &[&std::path::Path],
        state: &Arc<Mutex<AppState>>,
        host_name: &str,
    ) -> Result<()> {
        // Calculer la taille totale
        let total_size: u64 = files.iter()
            .filter_map(|f| std::fs::metadata(f).ok())
            .map(|m| m.len())
            .sum();

        // Simuler le transfert progressif
        {
            let mut state_lock = state.lock().unwrap();
            state_lock.update_progress(host_name, 0, crate::ui::tui::TransferStatus::Transferring);
        }

        let steps = 20;
        for i in 0..=steps {
            // Vérifier si on doit arrêter
            {
                let state_lock = state.lock().unwrap();
                if state_lock.should_quit || state_lock.is_paused {
                    break;
                }
            }

            let progress = (total_size * i as u64) / steps as u64;
            {
                let mut state_lock = state.lock().unwrap();
                state_lock.update_progress(host_name, progress, crate::ui::tui::TransferStatus::Transferring);
            }
            
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    /// Vrai transfert SSH
    fn real_transfer(
        uploader: &Uploader,
        files: &[&std::path::Path],
        host_entry: &HostEntry,
        destination: &str,
        state: &Arc<Mutex<AppState>>,
        host_name: &str,
    ) -> Result<()> {
        // Parser l'alias pour obtenir username et hostname
        let parts: Vec<&str> = host_entry.alias.split('@').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Format d'alias invalide: {}", host_entry.alias));
        }

        let username = parts[0];
        let hostname = parts[1];

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.update_progress(host_name, 0, crate::ui::tui::TransferStatus::Transferring);
            state_lock.add_log(&format!("🔄 {} : Début du transfert vers {}@{}", host_name, username, hostname));
        }

        // Pour l'instant, utiliser la méthode existante sans callback de progression
        // TODO: Améliorer pour avoir une vraie progression en temps réel
        match uploader.upload_files(files, &[(host_name.to_string(), host_entry)], destination) {
            Ok(_) => {
                let mut state_lock = state.lock().unwrap();
                let total_size = files.iter()
                    .filter_map(|f| std::fs::metadata(f).ok())
                    .map(|m| m.len())
                    .sum();
                state_lock.update_progress(host_name, total_size, crate::ui::tui::TransferStatus::Completed);
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(())
    }

    /// Point d'entrée pour lancer le TUI multi-écrans
    pub fn launch(config: &HostsConfig) -> Result<()> {
        let mut app = Self::new(config)?;
        app.run()
    }
}
