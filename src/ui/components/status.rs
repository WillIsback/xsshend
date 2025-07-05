use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::ui::tui::TuiState;

/// Composant pour la barre de statut
pub struct StatusBar;

impl StatusBar {
    pub fn render(f: &mut Frame, area: Rect, state: &TuiState) {
        let total_speed = state.get_total_speed();
        let eta = state.get_overall_eta();
        let (completed, failed, total) = state.get_summary();

        let status_text = vec![
            Line::from(vec![
                Span::styled("Débit: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}/s", Self::format_bytes(total_speed as u64)), 
                    Style::default().fg(Color::Green)
                ),
                Span::styled(" | ", Style::default().fg(Color::Gray)),
                Span::styled("ETA: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    eta.map(|d| format!("{}s", d.as_secs()))
                        .unwrap_or_else(|| "--".to_string()),
                    Style::default().fg(Color::Yellow)
                ),
                Span::styled(" | ", Style::default().fg(Color::Gray)),
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("{}/{} terminés", completed, total), 
                    Style::default().fg(Color::Green)
                ),
                if failed > 0 {
                    Span::styled(
                        format!(" - {} erreurs", failed), 
                        Style::default().fg(Color::Red)
                    )
                } else {
                    Span::styled("", Style::default())
                },
                if state.is_paused {
                    Span::styled(
                        " [PAUSE]", 
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD)
                    )
                } else {
                    Span::styled("", Style::default())
                },
            ]),
        ];

        let status = Paragraph::new(status_text)
            .block(Block::default()); // Supprimer les bordures qui causent des problèmes

        f.render_widget(status, area);
    }

    fn format_bytes(bytes: u64) -> String {
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
}
