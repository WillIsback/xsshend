use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use super::{
    app_state::{AppScreen, AppState},
    multi_screen_handler::MultiScreenEventHandler,
    screens::{
        DestinationInputScreen, FileSelectionScreen, PassphraseInputScreen, ProgressScreen,
        ServerSelectionScreen, SshKeySelectionScreen,
    },
    theme::get_theme_colors,
};
use crate::{
    config::{HostEntry, HostsConfig},
    utils::tui_logger::{TuiLogger, create_shared_log_buffer},
};

/// Application TUI multi-écrans principale
pub struct MultiScreenTuiApp {
    state: Arc<Mutex<AppState>>,
}

impl MultiScreenTuiApp {
    pub fn new(config: &HostsConfig) -> Result<Self> {
        let _available_hosts: HashMap<String, HostEntry> = config
            .get_all_hosts()
            .into_iter()
            .map(|(name, entry)| (name, entry.clone()))
            .collect();

        // Créer un buffer de logs partagé pour capturer tous les logs du système
        let log_buffer = create_shared_log_buffer();

        // Initialiser le logger TUI pour capturer tous les logs (si possible)
        if !TuiLogger::try_init(Arc::clone(&log_buffer)) {
            // Logger déjà initialisé, pas de problème
        }

        let mut app_state = AppState::new_with_log_buffer(log_buffer);

        // Initialiser le sélecteur hiérarchique
        app_state.init_hierarchical_selector(config)?;

        let state = Arc::new(Mutex::new(app_state));

        Ok(Self { state })
    }

    /// Crée une nouvelle instance avec filtrage par connectivité
    pub fn new_with_connectivity_check(config: &HostsConfig, timeout_secs: u64) -> Result<Self> {
        log::info!("🔍 Vérification de la connectivité des serveurs...");

        let online_hosts = config.get_online_hosts_sync(timeout_secs);

        if online_hosts.is_empty() {
            log::warn!("⚠️ Aucun serveur en ligne détecté");
        } else {
            log::info!("✅ {} serveurs en ligne détectés", online_hosts.len());
        }

        // Créer un buffer de logs partagé pour capturer tous les logs du système
        let log_buffer = create_shared_log_buffer();

        // Initialiser le logger TUI pour capturer tous les logs (si possible)
        if !TuiLogger::try_init(Arc::clone(&log_buffer)) {
            // Logger déjà initialisé, pas de problème
        }

        let _available_hosts: HashMap<String, HostEntry> = online_hosts.into_iter().collect();

        let mut app_state = AppState::new_with_log_buffer(log_buffer);

        // Initialiser le sélecteur hiérarchique avec seulement les hosts en ligne
        app_state.init_hierarchical_selector_filtered(config, timeout_secs)?;

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

        // Détecter le thème du terminal une fois au début
        let theme_colors = get_theme_colors();

        loop {
            // Dessiner l'interface avec le thème adapté
            terminal.draw(|f| {
                let state = self.state.lock().unwrap();
                match state.current_screen {
                    AppScreen::FileSelection => {
                        FileSelectionScreen::render_with_theme(f, f.size(), &state, &theme_colors);
                    }
                    AppScreen::SshKeySelection => {
                        SshKeySelectionScreen::render_with_theme(
                            f,
                            f.size(),
                            &state,
                            &theme_colors,
                        );
                    }
                    AppScreen::PassphraseInput => {
                        PassphraseInputScreen::render_with_theme(
                            f,
                            f.size(),
                            &state,
                            &theme_colors,
                        );
                    }
                    AppScreen::ServerSelection => {
                        ServerSelectionScreen::render_with_theme(
                            f,
                            f.size(),
                            &state,
                            &theme_colors,
                        );
                    }
                    AppScreen::DestinationInput => {
                        DestinationInputScreen::render_with_theme(
                            f,
                            f.size(),
                            &state,
                            &theme_colors,
                        );
                    }
                    AppScreen::UploadProgress => {
                        ProgressScreen::render_with_theme(f, f.size(), &state, &theme_colors);
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
                if matches!(state.current_screen, AppScreen::UploadProgress)
                    && upload_handle.is_none()
                {
                    let state_clone = Arc::clone(&upload_state);
                    let files = state.selected_files.clone();
                    let hosts = state.selected_hosts.clone();
                    let destination = state.destination.clone();
                    let validated_key = state.validated_ssh_key.clone();

                    upload_handle = Some(thread::spawn(move || {
                        let state_ref = Arc::clone(&state_clone);
                        if let Err(e) =
                            Self::run_uploads(state_clone, files, hosts, destination, validated_key)
                        {
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

            // Synchroniser régulièrement les logs depuis le buffer partagé
            {
                let mut state = self.state.lock().unwrap();
                state.sync_logs_from_shared_buffer();
            }
        }

        Ok(())
    }

    /// Lance les transferts en parallèle dans un thread séparé avec pool SSH moderne
    fn run_uploads(
        state: Arc<Mutex<AppState>>,
        files: Vec<std::path::PathBuf>,
        hosts: Vec<(String, HostEntry)>,
        destination: String,
        validated_key: Option<crate::ssh::keys::SshKeyWithPassphrase>,
    ) -> Result<()> {
        use crate::core::parallel::{ProgressCallback, TransferStatus};
        use crate::core::uploader::Uploader;
        use std::sync::Arc;

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log("🚀 Démarrage des transferts avec pool SSH...");
        }

        // Créer l'uploader avec la clé validée si disponible
        let mut uploader = if let Some(validated_key) = validated_key {
            {
                let mut state_lock = state.lock().unwrap();
                state_lock.add_log(&format!(
                    "🔑 Utilisation de la clé validée: {}",
                    validated_key.key.description()
                ));
            }
            Uploader::new_with_validated_key(validated_key)
        } else {
            Uploader::new()
        };

        // IMPORTANT: Initialiser le pool SSH UNE SEULE FOIS pour tous les fichiers
        let host_tuples: Vec<(String, &HostEntry)> = hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry))
            .collect();

        // Initialiser le pool avec tous les serveurs une seule fois
        if let Err(e) = uploader.initialize_ssh_pool(&host_tuples) {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log(&format!("❌ Erreur initialisation pool SSH: {}", e));
            return Err(e);
        }

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log(&format!(
                "✅ Pool SSH initialisé pour {} serveur(s)",
                hosts.len()
            ));
        }

        // Lancer les transferts fichier par fichier avec callback
        for (file_index, file) in files.iter().enumerate() {
            // Arrêter si demandé
            {
                let state_lock = state.lock().unwrap();
                if state_lock.should_quit {
                    break;
                }
            }

            let file_name = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("fichier");

            // Créer un callback spécifique pour ce fichier
            let file_progress_callback: ProgressCallback = {
                let state_clone = Arc::clone(&state);
                let file_name = file_name.to_string();
                Arc::new(
                    move |server_name: &str, bytes_transferred: u64, status: TransferStatus| {
                        if let Ok(mut app_state) = state_clone.lock() {
                            // Utiliser la méthode avec informations de fichier
                            app_state.update_progress_with_file(
                                server_name,
                                bytes_transferred,
                                status.clone(),
                                Some(&file_name),
                            );

                            // Log détaillé selon le statut
                            match &status {
                                TransferStatus::Connecting => {
                                    app_state.add_log(&format!(
                                        "🔗 {} ← {} : Connexion SSH...",
                                        server_name, file_name
                                    ));
                                }
                                TransferStatus::Transferring => {
                                    app_state.add_log(&format!(
                                        "📤 {} ← {} : Transfert en cours...",
                                        server_name, file_name
                                    ));
                                }
                                TransferStatus::Completed => {
                                    app_state.add_log(&format!(
                                        "✅ {} ← {} : Transfert terminé ({} octets)",
                                        server_name, file_name, bytes_transferred
                                    ));
                                }
                                TransferStatus::Failed(err) => {
                                    app_state.add_log(&format!(
                                        "❌ {} ← {} : Erreur - {}",
                                        server_name, file_name, err
                                    ));
                                }
                                TransferStatus::Pending => {
                                    app_state.add_log(&format!(
                                        "⏳ {} ← {} : En attente...",
                                        server_name, file_name
                                    ));
                                }
                            }
                        }
                    },
                )
            };

            {
                let mut state_lock = state.lock().unwrap();
                state_lock.add_log(&format!(
                    "📁 Traitement fichier {}/{}: {}",
                    file_index + 1,
                    files.len(),
                    file_name
                ));
            }

            // Upload avec callback spécifique au fichier
            match uploader.upload_single_file_with_initialized_pool(
                file.as_path(),
                &host_tuples,
                &destination,
                Some(file_progress_callback),
            ) {
                Ok(_) => {
                    let mut state_lock = state.lock().unwrap();
                    state_lock.add_log(&format!(
                        "🎯 Fichier {} traité sur tous les serveurs",
                        file_name
                    ));
                    // Incrémenter le compteur de fichiers complétés
                    state_lock.increment_completed_files();
                }
                Err(e) => {
                    let mut state_lock = state.lock().unwrap();
                    state_lock.add_log(&format!("❌ Erreur fichier {}: {}", file_name, e));
                    // Même en cas d'erreur, considérer le fichier comme traité pour le compteur
                    state_lock.increment_completed_files();
                }
            }
        }

        {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log("🏁 Tous les transferts terminés");
        }

        // Nettoyer les connexions SSH à la fin de tous les transferts
        if let Err(e) = uploader.cleanup_ssh_connections() {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log(&format!("⚠️ Avertissement nettoyage connexions: {}", e));
        } else {
            let mut state_lock = state.lock().unwrap();
            state_lock.add_log("🧹 Connexions SSH nettoyées");
        }

        Ok(())
    }
}
