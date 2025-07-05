use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::ui::tui::TuiState;

/// Composant pour afficher les logs
pub struct LogsView;

impl LogsView {
    pub fn render(f: &mut Frame, area: Rect, state: &TuiState) {
        let logs: Vec<ListItem> = state.logs
            .iter()
            .rev()
            .take(area.height.saturating_sub(2) as usize)
            .map(|log| {
                ListItem::new(Line::from(Span::styled(
                    log, 
                    Style::default().fg(Color::Gray)
                )))
            })
            .collect();

        let logs_list = List::new(logs)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Logs"));

        f.render_widget(logs_list, area);
    }
}
