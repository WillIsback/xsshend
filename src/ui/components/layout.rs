use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

/// Gère la disposition principale de l'application TUI
pub struct AppLayout;

impl AppLayout {
    /// Divise l'écran en zones principales
    pub fn split_main(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)  // Réduire la marge pour éviter les chevauchements
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(5),    // Main content (progress/logs) - Min au lieu de 6
                Constraint::Length(3), // Status
                Constraint::Length(1), // Controls
            ])
            .split(area)
            .to_vec()
    }

    /// Divise la zone principale pour afficher les logs à côté des progrès
    pub fn split_main_with_logs(area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Progress
                Constraint::Percentage(40), // Logs
            ])
            .split(area)
            .to_vec()
    }
}
