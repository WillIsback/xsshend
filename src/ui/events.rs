use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use std::sync::{Arc, Mutex};

use crate::ui::tui::TuiState;

/// Gestionnaire d'événements pour l'interface TUI
pub struct EventHandler {
    state: Arc<Mutex<TuiState>>,
    should_quit: bool,
}

impl EventHandler {
    pub fn new(state: Arc<Mutex<TuiState>>) -> Self {
        Self {
            state,
            should_quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    KeyCode::Char('p') => {
                        self.handle_pause_toggle()?;
                    }
                    KeyCode::Char('l') => {
                        self.handle_logs_toggle()?;
                    }
                    KeyCode::Char('r') => {
                        self.handle_refresh()?;
                    }
                    KeyCode::Char('h') => {
                        self.handle_help()?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_pause_toggle(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.is_paused = !state.is_paused;
        let status = if state.is_paused { "Pause" } else { "Reprise" };
        state.add_log(&format!("🎮 {}", status));
        Ok(())
    }

    fn handle_logs_toggle(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.show_logs = !state.show_logs;
        Ok(())
    }

    fn handle_refresh(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.add_log("🔄 Actualisation");
        Ok(())
    }

    fn handle_help(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.add_log("❓ Aide: Q=Quitter, P=Pause, L=Logs, R=Actualiser, H=Aide");
        Ok(())
    }
}
