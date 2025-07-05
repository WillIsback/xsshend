use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::tui::TuiState;

/// Composant pour l'en-tête de l'application
pub struct Header;

impl Header {
    pub fn render(f: &mut Frame, area: Rect, state: &TuiState) {
        let (completed, failed, total) = state.get_summary();
        
        let header_text = vec![
            Line::from(vec![
                Span::styled("📁 Fichier: ", Style::default().fg(Color::Cyan)),
                Span::styled(&state.file_name, Style::default().fg(Color::White)),
                Span::styled(
                    format!(" ({}) ", Self::format_bytes(state.file_size)), 
                    Style::default().fg(Color::Gray)
                ),
                Span::styled("| ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} serveurs ", total), 
                    Style::default().fg(Color::Yellow)
                ),
                Span::styled("| ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} terminés ", completed), 
                    Style::default().fg(Color::Green)
                ),
                if failed > 0 {
                    Span::styled(
                        format!("- {} erreurs", failed), 
                        Style::default().fg(Color::Red)
                    )
                } else {
                    Span::styled("", Style::default())
                },
            ]),
        ];

        let header = Paragraph::new(header_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("xsshend - Téléversement Multi-SSH")
                .title_alignment(Alignment::Center))
            .wrap(Wrap { trim: true });

        f.render_widget(header, area);
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
