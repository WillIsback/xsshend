use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

/// Composant pour afficher les contrôles
pub struct Controls;

impl Controls {
    pub fn render(f: &mut Frame, area: Rect) {
        let controls = Paragraph::new("[Q] Quitter  [P] Pause  [R] Actualiser  [L] Logs  [H] Aide")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        f.render_widget(controls, area);
    }
}
