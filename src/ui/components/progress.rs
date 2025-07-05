use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::ui::tui::TuiState;

/// Composant pour afficher la progression des transferts
pub struct ProgressView;

impl ProgressView {
    pub fn render(f: &mut Frame, area: Rect, state: &TuiState) {
        let mut transfers: Vec<_> = state.transfers.values().collect();
        transfers.sort_by(|a, b| a.host_name.cmp(&b.host_name));

        let items: Vec<ListItem> = transfers
            .iter()
            .map(|transfer| {
                let progress_ratio = if transfer.total_bytes > 0 {
                    transfer.bytes_transferred as f64 / transfer.total_bytes as f64
                } else {
                    0.0
                };

                let progress_bar = Self::create_progress_bar(progress_ratio, 30);
                let percentage = (progress_ratio * 100.0) as u8;
                
                let speed_text = if transfer.speed > 0.0 {
                    format!(" {}/s", Self::format_bytes(transfer.speed as u64))
                } else {
                    "".to_string()
                };

                let eta_text = if let Some(eta) = transfer.eta {
                    format!(" ETA: {}s", eta.as_secs())
                } else {
                    "".to_string()
                };

                let host_display = Self::truncate_string(&transfer.host_alias, 25);

                let content = Line::from(vec![
                    Span::styled(
                        format!("{} ", progress_bar),
                        Style::default().fg(transfer.status.color())
                    ),
                    Span::styled(
                        format!("{} {}%", host_display, percentage),
                        Style::default().fg(Color::White)
                    ),
                    Span::styled(
                        transfer.status.to_string(),
                        Style::default()
                            .fg(transfer.status.color())
                            .add_modifier(Modifier::BOLD)
                    ),
                    Span::styled(speed_text, Style::default().fg(Color::Cyan)),
                    Span::styled(eta_text, Style::default().fg(Color::Yellow)),
                ]);

                if let Some(error) = &transfer.error_message {
                    ListItem::new(vec![
                        content,
                        Line::from(Span::styled(
                            format!("    ❌ {}", Self::truncate_string(error, 60)),
                            Style::default().fg(Color::Red)
                        )),
                    ])
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        let progress_list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Progression par serveur"))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_widget(progress_list, area);
    }

    fn create_progress_bar(progress: f64, width: usize) -> String {
        let filled = (progress * width as f64) as usize;
        let empty = width.saturating_sub(filled);
        format!("[{}{}]", "█".repeat(filled), " ".repeat(empty))
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

    fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}
