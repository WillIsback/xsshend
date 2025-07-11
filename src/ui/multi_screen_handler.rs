use crate::core::parallel::TransferStatus;
use crate::ui::app_state::{AppScreen, AppState};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

/// Gestionnaire d'événements pour le TUI multi-écrans
pub struct MultiScreenEventHandler;

impl MultiScreenEventHandler {
    pub fn handle_event(state: &mut AppState, event: Event) -> Result<()> {
        if let Event::Key(key_event) = event {
            Self::handle_key_event(state, key_event)?;
        }
        Ok(())
    }

    fn handle_key_event(state: &mut AppState, key_event: KeyEvent) -> Result<()> {
        // Gestion globale sauf pour l'écran de destination (qui a sa propre logique Escape)
        match key_event.code {
            KeyCode::Esc if !matches!(state.current_screen, AppScreen::DestinationInput) => {
                // Permettre le reset uniquement si pas de transferts actifs
                let has_active_transfers = state.transfers.values().any(|t| {
                    t.status == crate::core::parallel::TransferStatus::Transferring
                        || t.status == crate::core::parallel::TransferStatus::Connecting
                });

                if !has_active_transfers {
                    state.reset_to_beginning();
                    return Ok(());
                }
            }
            KeyCode::Char('q') => {
                state.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                state.should_quit = true;
                return Ok(());
            }
            _ => {}
        }

        // Gestion par écran
        match state.current_screen {
            AppScreen::FileSelection => Self::handle_file_selection(state, key_event)?,
            AppScreen::SshKeySelection => Self::handle_ssh_key_selection(state, key_event)?,
            AppScreen::PassphraseInput => Self::handle_passphrase_input(state, key_event)?,
            AppScreen::ServerSelection => Self::handle_server_selection(state, key_event)?,
            AppScreen::DestinationInput => Self::handle_destination_input(state, key_event)?,
            AppScreen::UploadProgress => Self::handle_upload_progress(state, key_event)?,
        }

        Ok(())
    }

    /// Gestion des événements pour l'écran de sélection de fichiers
    fn handle_file_selection(state: &mut AppState, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Up => {
                if state.file_selection_cursor > 0 {
                    state.file_selection_cursor -= 1;
                }
            }
            KeyCode::Down => {
                if let Ok(files) = state.get_current_directory_files() {
                    // +1 pour inclure l'entrée ".."
                    if state.file_selection_cursor < files.len() {
                        state.file_selection_cursor += 1;
                    }
                }
            }
            KeyCode::Char(' ') => {
                // Sélectionner/désélectionner le fichier courant
                if let Ok(files) = state.get_current_directory_files() {
                    if state.file_selection_cursor > 0 {
                        let file_index = state.file_selection_cursor - 1;
                        if let Some(file) = files.get(file_index) {
                            state.toggle_file(file.clone());
                        }
                    }
                }
            }
            KeyCode::Enter => {
                // Naviguer vers le répertoire parent si on est sur ".."
                if state.file_selection_cursor == 0 {
                    let current_path = PathBuf::from(&state.current_file_path);
                    if let Some(parent) = current_path.parent() {
                        state.change_directory(parent.to_path_buf())?;
                    }
                } else {
                    // Ou sélectionner un fichier
                    if let Ok(files) = state.get_current_directory_files() {
                        let file_index = state.file_selection_cursor - 1;
                        if let Some(file) = files.get(file_index) {
                            if file.is_dir() {
                                state.change_directory(file.clone())?;
                            } else {
                                state.toggle_file(file.clone());
                            }
                        }
                    }
                }
            }
            KeyCode::Tab => {
                // Passer à l'écran suivant si des fichiers sont sélectionnés
                if !state.selected_files.is_empty() {
                    state.next_screen()?;
                }
            }
            KeyCode::Char('a') => {
                // Sélectionner tous les fichiers
                if let Ok(files) = state.get_current_directory_files() {
                    for file in files {
                        if !state.selected_files.contains(&file) {
                            state.selected_files.push(file);
                        }
                    }
                }
            }
            KeyCode::Char('c') if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Vider la sélection
                state.selected_files.clear();
            }
            KeyCode::Char('h') => {
                // Aller au répertoire home
                if let Some(home_dir) = dirs::home_dir() {
                    state.change_directory(home_dir)?;
                }
            }
            KeyCode::Char('r') => {
                // Rafraîchir le répertoire courant
                state.file_selection_cursor = 0;
            }
            _ => {}
        }
        Ok(())
    }

    /// Gestion des événements pour l'écran de sélection de serveurs
    fn handle_server_selection(state: &mut AppState, key_event: KeyEvent) -> Result<()> {
        // Déléguer la gestion au sélecteur hiérarchique
        if let Some(ref mut selector) = state.hierarchical_selector {
            let handled = selector.handle_key_event(key_event)?;

            if handled {
                // Synchroniser les serveurs sélectionnés
                state.sync_selected_hosts_from_hierarchical();
                return Ok(());
            }
        }

        // Gestion des touches spéciales non gérées par le sélecteur
        match key_event.code {
            KeyCode::Tab | KeyCode::Enter => {
                // Passer à l'écran de destination si des serveurs sont sélectionnés
                if !state.selected_hosts.is_empty() {
                    state.next_screen()?;
                }
            }
            KeyCode::Backspace => {
                // Retourner à l'écran de sélection de fichiers
                state.previous_screen();
            }
            _ => {}
        }
        Ok(())
    }

    /// Gestion des événements pour l'écran de saisie de destination
    fn handle_destination_input(state: &mut AppState, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char(c) => {
                // Ajouter le caractère à la destination
                state.destination_input.push(c);
            }
            KeyCode::Backspace => {
                // Si le champ est vide, retourner à l'écran précédent
                // Sinon, effacer le dernier caractère
                if state.destination_input.is_empty() {
                    state.previous_screen();
                } else {
                    state.destination_input.pop();
                }
            }
            KeyCode::Enter | KeyCode::Tab => {
                // Passer à l'écran suivant si le chemin n'est pas vide
                if !state.destination_input.trim().is_empty() {
                    state.next_screen()?;
                }
            }
            KeyCode::Esc => {
                // Si le champ est vide, faire un reset complet
                // Sinon, juste vider le champ
                if state.destination_input.is_empty() {
                    state.reset_to_beginning();
                } else {
                    state.destination_input.clear();
                }
            }
            KeyCode::F(1) => {
                // Raccourci : répertoire home
                state.destination_input = "$HOME/".to_string();
            }
            KeyCode::F(2) => {
                // Raccourci : /tmp/
                state.destination_input = "/tmp/".to_string();
            }
            KeyCode::F(3) => {
                // Raccourci : /opt/
                state.destination_input = "/opt/".to_string();
            }
            _ => {}
        }
        Ok(())
    }

    /// Gestion des événements pour l'écran de progression des uploads
    fn handle_upload_progress(state: &mut AppState, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('p') => {
                // Basculer pause/reprendre
                state.is_paused = !state.is_paused;
                if state.is_paused {
                    state.add_log("⏸️  Transferts mis en pause");
                } else {
                    state.add_log("▶️  Transferts repris");
                }
            }
            KeyCode::Char('l') => {
                // Basculer l'affichage des logs
                state.show_logs = !state.show_logs;
            }
            KeyCode::Backspace => {
                // Retourner à l'écran de sélection de serveurs (uniquement si aucun transfert en cours)
                let has_active_transfers = state.transfers.values().any(|t| {
                    t.status == TransferStatus::Transferring
                        || t.status == TransferStatus::Connecting
                });

                if !has_active_transfers {
                    state.previous_screen();
                }
            }
            KeyCode::Char('r') => {
                // Retry les transferts échoués
                for transfer in state.transfers.values_mut() {
                    if matches!(transfer.status, TransferStatus::Failed(_)) {
                        transfer.status = TransferStatus::Pending;
                        transfer.error_message = None;
                        transfer.bytes_transferred = 0;
                    }
                }
                state.add_log("🔄 Reprise des transferts échoués");
            }
            KeyCode::Up => {
                // Navigation dans les transferts (pour sélection future)
                let transfer_names: Vec<_> = state.transfers.keys().cloned().collect();
                if let Some(selected) = &state.selected_transfer {
                    if let Some(index) = transfer_names.iter().position(|name| name == selected) {
                        if index > 0 {
                            state.selected_transfer = Some(transfer_names[index - 1].clone());
                        }
                    }
                } else if !transfer_names.is_empty() {
                    state.selected_transfer = Some(transfer_names[0].clone());
                }
            }
            KeyCode::Down => {
                // Navigation dans les transferts (pour sélection future)
                let transfer_names: Vec<_> = state.transfers.keys().cloned().collect();
                if let Some(selected) = &state.selected_transfer {
                    if let Some(index) = transfer_names.iter().position(|name| name == selected) {
                        if index < transfer_names.len() - 1 {
                            state.selected_transfer = Some(transfer_names[index + 1].clone());
                        }
                    }
                } else if !transfer_names.is_empty() {
                    state.selected_transfer = Some(transfer_names[0].clone());
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Gestion des événements pour l'écran de saisie de passphrase
    fn handle_passphrase_input(state: &mut AppState, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('s') if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Passer sans passphrase (laisser vide et valider)
                state.passphrase_input.clear();
                match state.validate_passphrase() {
                    Ok(()) => {
                        state.next_screen()?;
                    }
                    Err(_) => {
                        // L'erreur est déjà loggée
                    }
                }
            }
            KeyCode::Char('v') if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // 'v' pour basculer la visibilité de la passphrase
                state.toggle_passphrase_visibility();
                state.add_log("🔍 Visibilité de la passphrase basculée");
            }
            KeyCode::Char(c) => {
                // Ajouter le caractère à la passphrase
                state.passphrase_input.push(c);
            }
            KeyCode::Backspace => {
                // Effacer le dernier caractère
                state.passphrase_input.pop();
            }
            KeyCode::Tab => {
                // Passer à l'écran suivant après validation
                match state.validate_passphrase() {
                    Ok(()) => {
                        state.add_log("✅ Passphrase validée, passage au serveur");
                        state.next_screen()?;
                    }
                    Err(e) => {
                        state.add_log(&format!("❌ Erreur validation passphrase: {}", e));
                    }
                }
            }
            KeyCode::Enter => {
                // Valider la passphrase et passer à l'écran suivant
                match state.validate_passphrase() {
                    Ok(()) => {
                        state.add_log("✅ Passphrase validée, passage au serveur");
                        state.next_screen()?;
                    }
                    Err(e) => {
                        state.add_log(&format!("❌ Validation échouée: {}. Réessayez.", e));
                    }
                }
            }
            KeyCode::Esc => {
                // Retour à l'écran précédent
                state.previous_screen();
            }
            _ => {}
        }
        Ok(())
    }

    /// Gestion des événements pour l'écran de sélection de clé SSH
    fn handle_ssh_key_selection(state: &mut AppState, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Up => {
                state.ssh_key_cursor_up();
            }
            KeyCode::Down => {
                state.ssh_key_cursor_down();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Sélectionner la clé courante et valider la passphrase
                if let Err(e) = state.select_current_ssh_key() {
                    state.add_log(&format!("❌ Sélection de clé échouée: {}", e));
                } else {
                    // Passer à l'écran suivant (passphrase input ou server selection)
                    state.next_screen()?;
                }
            }
            KeyCode::Tab => {
                // Passer à l'écran suivant
                state.next_screen()?;
            }
            KeyCode::Char('s') => {
                // Passer la sélection de clé SSH
                if let Err(e) = state.skip_ssh_key_selection() {
                    state.add_log(&format!("❌ Auto-sélection de clé échouée: {}", e));
                } else {
                    state.next_screen()?;
                }
            }
            KeyCode::Esc => {
                // Retour à l'écran précédent
                state.previous_screen();
            }
            _ => {}
        }
        Ok(())
    }
}
