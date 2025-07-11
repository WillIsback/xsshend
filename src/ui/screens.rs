use crate::core::parallel::TransferStatus;
use crate::ui::app_state::AppState;
use crate::ui::theme::{ThemeColors, ratatui_theme};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Text},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};

/// Composant pour la sélection de fichiers
pub struct FileSelectionScreen;

impl FileSelectionScreen {
    #[allow(dead_code)]
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        Self::render_with_theme(f, area, state, &theme_colors);
    }

    pub fn render_with_theme(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        theme_colors: &ThemeColors,
    ) {
        // Layout principal
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Titre
                Constraint::Length(3), // Chemin actuel
                Constraint::Min(5),    // Liste des fichiers
                Constraint::Length(5), // Fichiers sélectionnés
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Titre avec couleurs du thème
        let title = Paragraph::new("📁 Sélection des fichiers")
            .style(ratatui_theme::title_primary_style(theme_colors))
            .block(ratatui_theme::primary_block(theme_colors, ""));
        f.render_widget(title, chunks[0]);

        // Chemin actuel avec couleurs du thème
        let current_path = Paragraph::new(format!("📂 {}", state.current_file_path))
            .style(ratatui_theme::text_secondary_style(theme_colors))
            .block(ratatui_theme::secondary_block(
                theme_colors,
                "Répertoire actuel",
            ));
        f.render_widget(current_path, chunks[1]);

        // Liste des fichiers disponibles
        if let Ok(files) = state.get_current_directory_files() {
            let mut items =
                vec![ListItem::new("📁 ..").style(ratatui_theme::text_accent_style(theme_colors))];

            for file in &files {
                let file_name = file.file_name().unwrap_or_default().to_string_lossy();

                let style = if state.selected_files.contains(file) {
                    ratatui_theme::success_style(theme_colors)
                } else {
                    ratatui_theme::unselected_item_style(theme_colors)
                };

                let icon = if state.selected_files.contains(file) {
                    "✅"
                } else {
                    "📄"
                };
                items.push(ListItem::new(format!("{} {}", icon, file_name)).style(style));
            }

            let files_list = List::new(items)
                .block(ratatui_theme::themed_block(
                    theme_colors,
                    "Fichiers (Espace pour sélectionner)",
                ))
                .highlight_style(ratatui_theme::selection_style(theme_colors));

            // État de la liste avec curseur
            let mut list_state = ListState::default();
            list_state.select(Some(state.file_selection_cursor));
            f.render_stateful_widget(files_list, chunks[2], &mut list_state);
        }

        // Fichiers sélectionnés avec taille
        let selected_text = if state.selected_files.is_empty() {
            Text::from(
                "Aucun fichier sélectionné\n\nUtilisez les flèches ↑↓ pour naviguer\nAppuyez sur Espace pour sélectionner",
            )
        } else {
            let mut lines = vec![Line::from(format!(
                "📁 {} fichier(s) sélectionné(s):",
                state.selected_files.len()
            ))];
            let mut total_size = 0u64;

            for file in &state.selected_files {
                let size = std::fs::metadata(file).map(|m| m.len()).unwrap_or(0);
                total_size += size;
                let file_name = file.file_name().unwrap_or_default().to_string_lossy();
                lines.push(Line::from(format!(
                    "  ✅ {} ({})",
                    file_name,
                    crate::ui::app_state::format_bytes(size)
                )));
            }

            lines.push(Line::from(""));
            lines.push(Line::from(format!(
                "📊 Taille totale: {}",
                crate::ui::app_state::format_bytes(total_size)
            )));
            Text::from(lines)
        };

        let selected_files = Paragraph::new(selected_text)
            .style(ratatui_theme::success_style(theme_colors))
            .block(ratatui_theme::secondary_block(theme_colors, "Sélectionnés"))
            .wrap(Wrap { trim: true });
        f.render_widget(selected_files, chunks[3]);

        // Instructions améliorées
        let instructions = Paragraph::new(
            "🗂️ Fichiers: ↑↓ Naviguer | Espace: Sélectionner | Entrée: Dossier parent | h: Home\n📁 Sélection: a: Tout | c: Vider | Tab: Serveurs → | Esc: Reset | q: Quitter"
        )
        .style(ratatui_theme::help_text_style(theme_colors))
        .block(ratatui_theme::themed_block(theme_colors, "Aide"))
        .wrap(Wrap { trim: true });
        f.render_widget(instructions, chunks[4]);
    }
}

/// Composant pour la sélection de serveurs
pub struct ServerSelectionScreen;

impl ServerSelectionScreen {
    #[allow(dead_code)]
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        Self::render_with_theme(f, area, state, &theme_colors);
    }

    pub fn render_with_theme(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        theme_colors: &ThemeColors,
    ) {
        // Si le sélecteur hiérarchique n'est pas initialisé, afficher un message
        if let Some(ref selector) = state.hierarchical_selector {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Titre
                    Constraint::Length(5), // Fichiers sélectionnés
                    Constraint::Min(5),    // Sélecteur hiérarchique
                ])
                .split(area);

            // Titre avec couleurs du thème
            let title = Paragraph::new("🌳 Sélection hiérarchique des serveurs")
                .style(ratatui_theme::title_primary_style(theme_colors))
                .block(ratatui_theme::primary_block(theme_colors, ""));
            f.render_widget(title, chunks[0]);

            // Résumé des fichiers sélectionnés
            let files_summary = format!(
                "📄 {} fichier(s) sélectionné(s)\n{}",
                state.selected_files.len(),
                state
                    .selected_files
                    .iter()
                    .take(3)
                    .map(|f| format!(
                        "  • {}",
                        f.file_name().unwrap_or_default().to_string_lossy()
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            let files_info = Paragraph::new(files_summary)
                .style(ratatui_theme::help_text_style(theme_colors))
                .block(ratatui_theme::secondary_block(
                    theme_colors,
                    "Fichiers à téléverser",
                ))
                .wrap(Wrap { trim: true });
            f.render_widget(files_info, chunks[1]);

            // Rendu du sélecteur hiérarchique avec thème
            selector.render_with_theme(f, chunks[2], theme_colors);
        } else {
            // Affichage de fallback si le sélecteur n'est pas initialisé
            let error_msg = Paragraph::new(
                "❌ Erreur: Sélecteur hiérarchique non initialisé\n\nAppuyez sur 'q' pour quitter",
            )
            .style(ratatui_theme::error_style(theme_colors))
            .block(ratatui_theme::primary_block(theme_colors, "Erreur"))
            .wrap(Wrap { trim: true });
            f.render_widget(error_msg, area);
        }
    }
}

/// Composant pour la saisie de destination
pub struct DestinationInputScreen;

impl DestinationInputScreen {
    #[allow(dead_code)]
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        Self::render_with_theme(f, area, state, &theme_colors);
    }

    pub fn render_with_theme(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        theme_colors: &ThemeColors,
    ) {
        // Layout principal avec instructions agrandies
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Titre
                Constraint::Length(4), // Fichiers
                Constraint::Length(4), // Serveurs sélectionnés
                Constraint::Length(3), // Saisie de destination
                Constraint::Length(5), // Exemples
                Constraint::Length(3), // Instructions
            ])
            .split(area);

        // Titre avec couleurs du thème
        let title = Paragraph::new("📝 Saisie du répertoire de destination")
            .style(ratatui_theme::title_primary_style(theme_colors))
            .block(ratatui_theme::primary_block(theme_colors, ""));
        f.render_widget(title, chunks[0]);

        // Résumé des fichiers avec couleurs du thème
        let files_text = format!(
            "📄 {} fichier(s) sélectionné(s): {}",
            state.selected_files.len(),
            state
                .selected_files
                .iter()
                .take(2)
                .map(|f| f.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let files_info = Paragraph::new(files_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::secondary_block(theme_colors, "Fichiers"))
            .wrap(Wrap { trim: true });
        f.render_widget(files_info, chunks[1]);

        // Résumé des serveurs sélectionnés avec couleurs du thème
        let servers_text = format!(
            "🌐 {} serveur(s) sélectionné(s): {}",
            state.selected_hosts.len(),
            state
                .selected_hosts
                .iter()
                .take(3)
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let servers_info = Paragraph::new(servers_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::secondary_block(theme_colors, "Serveurs"));
        f.render_widget(servers_info, chunks[2]);

        // Saisie de destination avec curseur visuel et couleurs du thème
        let destination_display = format!("📂 {}_", state.destination_input);
        let destination_input = Paragraph::new(destination_display)
            .style(ratatui_theme::text_style(theme_colors).add_modifier(Modifier::BOLD))
            .block(ratatui_theme::themed_block(
                theme_colors,
                "Répertoire de destination",
            ));
        f.render_widget(destination_input, chunks[3]);

        // Exemples de chemins de destination avec couleurs du thème
        let examples_text = "💡 Exemples de chemins :\n\
            • /tmp/uploads/           (répertoire simple)\n\
            • /opt/apps/              (applications)\n\
            • /var/www/html/          (web)\n\
            • /home/user/deploy/      (utilisateur)\n\
            • /etc/config/            (configuration)"
            .to_string();

        let examples = Paragraph::new(examples_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::themed_block(
                theme_colors,
                "Exemples de chemins de destination",
            ))
            .wrap(Wrap { trim: true });
        f.render_widget(examples, chunks[4]);

        // Instructions de saisie avec couleurs du thème
        let instructions_text = "📝 Saisie: Tapez votre chemin | Backspace: Effacer (ou retour si vide) | Esc: Vider OU Reset\n🚀 Raccourcis: F1=/home | F2=/tmp | F3=/opt | Entrée/Tab: Continuer → | q: Quitter".to_string();

        let instructions = Paragraph::new(instructions_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::themed_block(theme_colors, "Contrôles"))
            .wrap(Wrap { trim: true });
        f.render_widget(instructions, chunks[5]);
    }
}

/// Composant pour afficher le progrès des uploads
pub struct ProgressScreen;

impl ProgressScreen {
    #[allow(dead_code)]
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        Self::render_with_theme(f, area, state, &theme_colors);
    }

    pub fn render_with_theme(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        theme_colors: &ThemeColors,
    ) {
        let chunks = if state.show_logs {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5), // Titre et résumé avec stats
                    Constraint::Min(8),    // Liste des transferts
                    Constraint::Length(8), // Logs
                    Constraint::Length(3), // Instructions
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5), // Titre et résumé avec stats
                    Constraint::Min(8),    // Liste des transferts
                    Constraint::Length(3), // Instructions
                ])
                .split(area)
        };

        // En-tête avec statistiques détaillées
        let (completed, failed, total) = state.get_summary();
        let all_complete = state.are_all_transfers_complete();

        let status_text = if all_complete {
            if failed > 0 {
                format!("❌ Transferts terminés avec {} échec(s)", failed)
            } else {
                "✅ Tous les transferts terminés avec succès!".to_string()
            }
        } else {
            format!("⚡ Transferts en cours - {}/{} terminés", completed, total)
        };

        // Calculer les statistiques
        let total_transferred: u64 = state.transfers.values().map(|t| t.bytes_transferred).sum();
        let total_size: u64 = state.transfers.values().map(|t| t.total_bytes).sum();
        let total_speed = state.get_total_speed();

        let stats_text = format!(
            "{}\n� Fichiers: {} | �📊 Progrès: {} / {} ({:.1}%)\n🚀 Vitesse: {}/s",
            status_text,
            state.get_files_progress_string(),
            crate::ui::app_state::format_bytes(total_transferred),
            crate::ui::app_state::format_bytes(total_size),
            if total_size > 0 {
                (total_transferred as f64 / total_size as f64) * 100.0
            } else {
                0.0
            },
            crate::ui::app_state::format_bytes(total_speed as u64)
        );

        let header_style = if all_complete {
            if failed > 0 {
                ratatui_theme::error_style(theme_colors)
            } else {
                ratatui_theme::success_style(theme_colors)
            }
        } else {
            ratatui_theme::text_accent_style(theme_colors)
        };

        let header = Paragraph::new(stats_text)
            .style(header_style)
            .block(ratatui_theme::primary_block(
                theme_colors,
                "📊 Progression générale",
            ))
            .wrap(Wrap { trim: true });
        f.render_widget(header, chunks[0]);

        // Liste des transferts avec couleurs du thème
        let mut transfer_items = Vec::new();
        for (key, transfer) in &state.transfers {
            let (server, file) = match key.split_once("::") {
                Some((s, f)) => (s, f),
                None => ("", key.as_str()),
            };

            let status_icon = match transfer.status {
                TransferStatus::Pending => "⏳",
                TransferStatus::Connecting => "�",
                TransferStatus::Transferring => "🚀",
                TransferStatus::Completed => "✅",
                TransferStatus::Failed(_) => "❌",
            };

            let progress_bar =
                Self::create_progress_bar(transfer.bytes_transferred, transfer.total_bytes);
            let percentage = if transfer.total_bytes > 0 {
                (transfer.bytes_transferred as f64 / transfer.total_bytes as f64) * 100.0
            } else {
                0.0
            };

            let line_text = format!(
                "{} {} → {} [{:.1}%] {} {}/{}",
                status_icon,
                file,
                server,
                percentage,
                progress_bar,
                crate::ui::app_state::format_bytes(transfer.bytes_transferred),
                crate::ui::app_state::format_bytes(transfer.total_bytes)
            );

            let item_style = match transfer.status {
                TransferStatus::Completed => ratatui_theme::success_style(theme_colors),
                TransferStatus::Failed(_) => ratatui_theme::error_style(theme_colors),
                TransferStatus::Transferring => ratatui_theme::text_accent_style(theme_colors),
                TransferStatus::Connecting => ratatui_theme::warning_style(theme_colors),
                TransferStatus::Pending => ratatui_theme::unselected_item_style(theme_colors),
            };

            transfer_items.push(ListItem::new(line_text).style(item_style));
        }

        let transfers_list = List::new(transfer_items).block(ratatui_theme::themed_block(
            theme_colors,
            "Détails des transferts",
        ));
        f.render_widget(transfers_list, chunks[1]);

        // Logs si activés
        if state.show_logs && chunks.len() > 3 {
            let log_items: Vec<ListItem> = state
                .logs
                .iter()
                .rev()
                .take(6)
                .rev()
                .map(|log| ListItem::new(log.as_str()))
                .collect();

            let logs_list = List::new(log_items)
                .block(ratatui_theme::themed_block(theme_colors, "Logs"))
                .style(ratatui_theme::help_text_style(theme_colors));
            f.render_widget(logs_list, chunks[2]);
        }

        // Instructions améliorées
        let instructions_text = if state.are_all_transfers_complete() {
            "✅ Transferts terminés! | l: Basculer logs | Esc: Retour au début | q: Quitter | r: Relancer les échecs"
        } else {
            "⏸️ p: Pause/Reprendre | 📝 l: Basculer logs | Esc: Retour au début | ❌ q: Quitter"
        };

        let instructions = Paragraph::new(instructions_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::themed_block(theme_colors, "Contrôles"));
        f.render_widget(instructions, chunks[chunks.len() - 1]);
    }

    fn create_progress_bar(current: u64, total: u64) -> String {
        if total == 0 {
            return "██████████".to_string();
        }

        let progress = (current as f64 / total as f64).min(1.0);
        let filled = (progress * 20.0) as usize; // Barre plus longue pour plus de précision
        let empty = 20 - filled;

        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }
}

/// Composant pour la saisie de passphrase SSH
pub struct PassphraseInputScreen;

impl PassphraseInputScreen {
    #[allow(dead_code)]
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        Self::render_with_theme(f, area, state, &theme_colors);
    }

    pub fn render_with_theme(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        theme_colors: &ThemeColors,
    ) {
        // Layout principal
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Titre
                Constraint::Length(6), // Info sur la clé sélectionnée
                Constraint::Length(6), // Champ de saisie de passphrase
                Constraint::Min(4),    // Messages d'erreur / status
                Constraint::Length(4), // Instructions
            ])
            .split(area);

        // Titre avec couleurs du thème
        let title = Paragraph::new("🔐 Saisie de la passphrase SSH")
            .style(ratatui_theme::title_primary_style(theme_colors))
            .block(ratatui_theme::primary_block(theme_colors, ""));
        f.render_widget(title, chunks[0]);

        // Informations sur la clé sélectionnée
        let key_info = if let Some(ref key) = state.pending_key_for_passphrase {
            format!(
                "🔑 Clé sélectionnée: {}\n📁 Fichier: {}\n🔧 Type: {}",
                key.name,
                key.private_key_path.display(),
                key.key_type
            )
        } else if let Some(ref key) = state.selected_ssh_key {
            format!(
                "🔑 Clé sélectionnée: {}\n📁 Fichier: {}\n🔧 Type: {}",
                key.name,
                key.private_key_path.display(),
                key.key_type
            )
        } else {
            "❌ Aucune clé sélectionnée".to_string()
        };

        let key_info_paragraph = Paragraph::new(key_info)
            .style(ratatui_theme::text_style(theme_colors))
            .block(ratatui_theme::secondary_block(theme_colors, "Clé SSH"))
            .wrap(Wrap { trim: true });
        f.render_widget(key_info_paragraph, chunks[1]);

        // Champ de saisie de passphrase
        let passphrase_display = if state.passphrase_input_visible {
            state.passphrase_input.clone()
        } else {
            "*".repeat(state.passphrase_input.len())
        };

        let passphrase_style = ratatui_theme::selection_style(theme_colors);
        let passphrase_input = Paragraph::new(format!("🔐 {}", passphrase_display))
            .style(passphrase_style)
            .block(ratatui_theme::themed_block(theme_colors, "Passphrase (Tab: Afficher/Masquer)"));
        f.render_widget(passphrase_input, chunks[2]);

        // Messages de status
        let status_text = if state.pending_key_for_passphrase.is_some() {
            "💡 Entrez la passphrase de votre clé SSH.\n   Laissez vide si la clé n'a pas de passphrase."
        } else {
            "✅ Passphrase validée avec succès !"
        };

        let status = Paragraph::new(status_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::secondary_block(theme_colors, "Status"));
        f.render_widget(status, chunks[3]);

        // Instructions
        let instructions_text = if state.pending_key_for_passphrase.is_some() {
            "🔐 Passphrase: Tapez votre passphrase | Tab: Afficher/Masquer | Entrée: Valider | Esc: Retour | s: Passer sans passphrase"
        } else {
            "✅ Passphrase validée | Entrée/Tab: Continuer → | Esc: Retour | q: Quitter"
        };

        let instructions = Paragraph::new(instructions_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::themed_block(theme_colors, "Contrôles"));
        f.render_widget(instructions, chunks[4]);
    }
}

/// Composant pour la sélection de clé SSH
pub struct SshKeySelectionScreen;

impl SshKeySelectionScreen {
    #[allow(dead_code)]
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        Self::render_with_theme(f, area, state, &theme_colors);
    }

    pub fn render_with_theme(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        theme_colors: &ThemeColors,
    ) {
        // Layout principal
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Titre
                Constraint::Length(4), // Fichiers sélectionnés
                Constraint::Min(8),    // Liste des clés SSH
                Constraint::Length(5), // Description de la clé sélectionnée
                Constraint::Length(4), // Instructions
            ])
            .split(area);

        // Titre avec couleurs du thème
        let title = Paragraph::new("🔑 Sélection de la clé SSH")
            .style(ratatui_theme::title_primary_style(theme_colors))
            .block(ratatui_theme::primary_block(theme_colors, ""));
        f.render_widget(title, chunks[0]);

        // Résumé des fichiers sélectionnés
        let files_summary = format!(
            "📄 {} fichier(s) sélectionné(s): {}",
            state.selected_files.len(),
            state
                .selected_files
                .iter()
                .take(2)
                .map(|f| f.file_name().unwrap_or_default().to_string_lossy())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let files_info = Paragraph::new(files_summary)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::secondary_block(theme_colors, "Fichiers"))
            .wrap(Wrap { trim: true });
        f.render_widget(files_info, chunks[1]);

        // Liste des clés SSH disponibles
        let ssh_keys_items: Vec<ListItem> = state
            .available_ssh_keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let icon = if state.selected_ssh_key.as_ref() == Some(key) {
                    "✅"
                } else if i == state.ssh_key_selection_cursor {
                    "👉"
                } else {
                    "🔑"
                };

                let text = format!("{} {}", icon, key.description());

                let style = if state.selected_ssh_key.as_ref() == Some(key) {
                    ratatui_theme::success_style(theme_colors)
                } else if i == state.ssh_key_selection_cursor {
                    ratatui_theme::selection_style(theme_colors)
                } else {
                    ratatui_theme::text_style(theme_colors)
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let no_keys_msg = if state.available_ssh_keys.is_empty() {
            vec![
                ListItem::new("❌ Aucune clé SSH trouvée dans ~/.ssh")
                    .style(ratatui_theme::error_style(theme_colors)),
            ]
        } else {
            ssh_keys_items
        };

        let ssh_keys_list = List::new(no_keys_msg)
            .block(ratatui_theme::themed_block(
                theme_colors,
                "Clés SSH disponibles",
            ))
            .highlight_style(ratatui_theme::selection_style(theme_colors));

        // Afficher avec curseur
        let mut list_state = ListState::default();
        if !state.available_ssh_keys.is_empty() {
            list_state.select(Some(state.ssh_key_selection_cursor));
        }
        f.render_stateful_widget(ssh_keys_list, chunks[2], &mut list_state);

        // Description de la clé sélectionnée ou en cours de survol
        let description_text = if let Some(current_key) =
            state.available_ssh_keys.get(state.ssh_key_selection_cursor)
        {
            let selected_indicator = if state.selected_ssh_key.as_ref() == Some(current_key) {
                " ✅ SÉLECTIONNÉE"
            } else {
                ""
            };

            format!(
                "🔑 Nom: {}\n📁 Chemin: {}\n🔧 Type: {}\n💬 Commentaire: {}{}",
                current_key.name,
                current_key.private_key_path.display(),
                current_key.key_type,
                current_key.comment.as_deref().unwrap_or("Aucun"),
                selected_indicator
            )
        } else if state.available_ssh_keys.is_empty() {
            "❌ Aucune clé SSH disponible.\n\nAssurez-vous d'avoir des clés SSH dans ~/.ssh/\n(ex: id_ed25519, id_rsa, etc.)".to_string()
        } else {
            "Sélectionnez une clé SSH avec les flèches ↑↓".to_string()
        };

        let description = Paragraph::new(description_text)
            .style(ratatui_theme::text_style(theme_colors))
            .block(ratatui_theme::secondary_block(
                theme_colors,
                "Détails de la clé",
            ))
            .wrap(Wrap { trim: true });
        f.render_widget(description, chunks[3]);

        // Instructions
        let instructions_text = if state.available_ssh_keys.is_empty() {
            "❌ Aucune clé disponible | Tab: Continuer (utilisation ssh-agent) | Esc: Retour | q: Quitter"
        } else {
            "🔑 Clés: ↑↓ Naviguer | Espace/Entrée: Sélectionner | Tab: Continuer → | s: Passer | Esc: Retour | q: Quitter"
        };

        let instructions = Paragraph::new(instructions_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::themed_block(theme_colors, "Contrôles"));
        f.render_widget(instructions, chunks[4]);
    }
}
