use crate::core::parallel::TransferStatus;
use crate::ui::app_state::AppState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// Composant pour la sélection de fichiers
pub struct FileSelectionScreen;

impl FileSelectionScreen {
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
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

        // Titre
        let title = Paragraph::new("📁 Sélection des fichiers")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Chemin actuel
        let current_path = Paragraph::new(format!("📂 {}", state.current_file_path))
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Répertoire actuel"),
            );
        f.render_widget(current_path, chunks[1]);

        // Liste des fichiers disponibles
        if let Ok(files) = state.get_current_directory_files() {
            let mut items = vec![ListItem::new("📁 ..").style(Style::default().fg(Color::Cyan))];

            for file in &files {
                let file_name = file.file_name().unwrap_or_default().to_string_lossy();

                let style = if state.selected_files.contains(file) {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let icon = if state.selected_files.contains(file) {
                    "✅"
                } else {
                    "📄"
                };
                items.push(ListItem::new(format!("{} {}", icon, file_name)).style(style));
            }

            let files_list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Fichiers (Espace pour sélectionner)"),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                );

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
            .style(Style::default().fg(Color::Green))
            .block(Block::default().borders(Borders::ALL).title("Sélectionnés"))
            .wrap(Wrap { trim: true });
        f.render_widget(selected_files, chunks[3]);

        // Instructions améliorées
        let instructions = Paragraph::new(
            "🗂️ Fichiers: ↑↓ Naviguer | Espace: Sélectionner | Entrée: Dossier parent | h: Home\n📁 Sélection: a: Tout | c: Vider | Tab: Serveurs → | Esc: Reset | q: Quitter"
        )
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Aide"))
        .wrap(Wrap { trim: true });
        f.render_widget(instructions, chunks[4]);
    }
}

/// Composant pour la sélection de serveurs
pub struct ServerSelectionScreen;

impl ServerSelectionScreen {
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
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

            // Titre
            let title = Paragraph::new("🌳 Sélection hiérarchique des serveurs")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
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
                .style(Style::default().fg(Color::Yellow))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Fichiers à téléverser"),
                )
                .wrap(Wrap { trim: true });
            f.render_widget(files_info, chunks[1]);

            // Rendu du sélecteur hiérarchique
            selector.render(f, chunks[2]);
        } else {
            // Affichage de fallback si le sélecteur n'est pas initialisé
            let error_msg = Paragraph::new(
                "❌ Erreur: Sélecteur hiérarchique non initialisé\n\nAppuyez sur 'q' pour quitter",
            )
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Erreur"))
            .wrap(Wrap { trim: true });
            f.render_widget(error_msg, area);
        }
    }
}

/// Composant pour la saisie de destination
pub struct DestinationInputScreen;

impl DestinationInputScreen {
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Titre
                Constraint::Length(5), // Résumé fichiers
                Constraint::Length(5), // Résumé serveurs
                Constraint::Length(5), // Saisie destination
                Constraint::Length(8), // Exemples et variables
                Constraint::Length(4), // Instructions
            ])
            .split(area);

        // Titre
        let title = Paragraph::new("📂 Destination des fichiers")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Résumé des fichiers
        let total_size: u64 = state
            .selected_files
            .iter()
            .filter_map(|f| std::fs::metadata(f).ok())
            .map(|m| m.len())
            .sum();

        let files_summary = format!(
            "📄 {} fichier(s) sélectionné(s) - Taille totale: {}",
            state.selected_files.len(),
            crate::ui::app_state::format_bytes(total_size)
        );

        let files_info = Paragraph::new(files_summary)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Fichiers"));
        f.render_widget(files_info, chunks[1]);

        // Résumé des serveurs
        let servers_summary = format!(
            "🖥️ {} serveur(s) sélectionné(s): {}",
            state.selected_hosts.len(),
            state
                .selected_hosts
                .iter()
                .take(3)
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let servers_info = Paragraph::new(servers_summary)
            .style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Serveurs cibles"),
            );
        f.render_widget(servers_info, chunks[2]);

        // Saisie de destination avec curseur visuel
        let destination_display = format!("📂 {}_", state.destination_input);
        let destination_input = Paragraph::new(destination_display)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Répertoire de destination"),
            );
        f.render_widget(destination_input, chunks[3]);

        // Exemples de chemins de destination
        let examples_text = "💡 Exemples de chemins :\n\
            • /tmp/uploads/           (répertoire simple)\n\
            • /opt/apps/              (applications)\n\
            • /var/www/html/          (web)\n\
            • /home/user/deploy/      (utilisateur)\n\
            • /etc/config/            (configuration)"
            .to_string();

        let examples = Paragraph::new(examples_text)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Exemples de chemins de destination"),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(examples, chunks[4]);

        // Instructions de saisie
        let instructions_text = "📝 Saisie: Tapez votre chemin | Backspace: Effacer (ou retour si vide) | Esc: Vider OU Reset\n🚀 Raccourcis: F1=/home | F2=/tmp | F3=/opt | Entrée/Tab: Continuer → | q: Quitter".to_string();

        let instructions = Paragraph::new(instructions_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Contrôles"))
            .wrap(Wrap { trim: true });
        f.render_widget(instructions, chunks[5]);
    }
}

/// Composant pour afficher le progrès des uploads
pub struct ProgressScreen;

impl ProgressScreen {
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
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
            if total_speed > 0.0 {
                crate::ui::app_state::format_bytes(total_speed as u64)
            } else {
                "0 B".to_string()
            }
        );

        let title_color = if all_complete {
            if failed > 0 { Color::Red } else { Color::Green }
        } else {
            Color::Cyan
        };

        let header = Paragraph::new(stats_text)
            .style(
                Style::default()
                    .fg(title_color)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("État des transferts"),
            );
        f.render_widget(header, chunks[0]);

        // Liste des transferts avec détails améliorés
        let mut transfer_items = Vec::new();
        for (host_name, progress) in &state.transfers {
            let progress_bar =
                Self::create_progress_bar(progress.bytes_transferred, progress.total_bytes);
            let percentage = if progress.total_bytes > 0 {
                (progress.bytes_transferred as f64 / progress.total_bytes as f64 * 100.0) as u32
            } else {
                0
            };

            let speed_text = if progress.speed > 0.0 {
                format!(
                    " - {}/s",
                    crate::ui::app_state::format_bytes(progress.speed as u64)
                )
            } else {
                String::new()
            };

            let eta_text = if let Some(eta) = progress.eta {
                format!(" - ETA: {}s", eta.as_secs())
            } else {
                String::new()
            };

            let status_icon = match &progress.status {
                TransferStatus::Pending => "⏳",
                TransferStatus::Connecting => "🔄",
                TransferStatus::Transferring => "📤",
                TransferStatus::Completed => "✅",
                TransferStatus::Failed(_) => "❌",
            };

            let host_display = format!("{} ({})", host_name, progress.host_alias);
            let progress_display = format!(
                "{} / {} ({}%)",
                crate::ui::app_state::format_bytes(progress.bytes_transferred),
                crate::ui::app_state::format_bytes(progress.total_bytes),
                percentage
            );

            // Affichage amélioré avec nom du fichier et flèche
            let file_display =
                if !progress.file_name.is_empty() && progress.file_name != "En attente..." {
                    format!("{} → ", progress.file_name)
                } else {
                    String::new()
                };

            let line_text = format!(
                "{} {}{} [{}] {}{}{}",
                status_icon,
                file_display,
                host_display,
                progress_bar,
                progress_display,
                speed_text,
                eta_text
            );

            let item_style = Style::default().fg(progress.status.color());
            if let Some(error) = &progress.error_message {
                // Afficher l'erreur sur une ligne séparée
                transfer_items.push(ListItem::new(line_text).style(item_style));
                transfer_items.push(
                    ListItem::new(format!("   ↳ Erreur: {}", error))
                        .style(Style::default().fg(Color::Red)),
                );
            } else {
                transfer_items.push(ListItem::new(line_text).style(item_style));
            }
        }

        let transfers_list = List::new(transfer_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Détails des transferts"),
        );
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
                .block(Block::default().borders(Borders::ALL).title("Logs"))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(logs_list, chunks[2]);
        }

        // Instructions améliorées
        let instructions_text = if state.are_all_transfers_complete() {
            "✅ Transferts terminés! | l: Basculer logs | Esc: Retour au début | q: Quitter | r: Relancer les échecs"
        } else {
            "⏸️ p: Pause/Reprendre | 📝 l: Basculer logs | Esc: Retour au début | ❌ q: Quitter"
        };

        let instructions = Paragraph::new(instructions_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Contrôles"));
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
