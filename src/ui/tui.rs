use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    style::Color,
    Frame, Terminal,
};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::config::HostEntry;
use super::components::{AppLayout, Header, ProgressView, LogsView, StatusBar, Controls};
use super::events::EventHandler;

#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub host_name: String,
    pub host_alias: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub status: TransferStatus,
    pub speed: f64, // bytes per second
    pub eta: Option<Duration>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferStatus {
    Pending,
    Connecting,
    Transferring,
    Completed,
    Failed,
    Paused,
}

impl TransferStatus {
    pub fn to_string(&self) -> &'static str {
        match self {
            TransferStatus::Pending => "En attente",
            TransferStatus::Connecting => "Connexion...",
            TransferStatus::Transferring => "Transfert...",
            TransferStatus::Completed => "Terminé ✅",
            TransferStatus::Failed => "Erreur ❌",
            TransferStatus::Paused => "Pause ⏸️",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TransferStatus::Pending => Color::Gray,
            TransferStatus::Connecting => Color::Yellow,
            TransferStatus::Transferring => Color::Blue,
            TransferStatus::Completed => Color::Green,
            TransferStatus::Failed => Color::Red,
            TransferStatus::Paused => Color::Magenta,
        }
    }
}

pub struct TuiState {
    pub file_name: String,
    pub file_size: u64,
    pub destination: String,
    pub transfers: HashMap<String, TransferProgress>,
    pub start_time: Instant,
    pub is_paused: bool,
    pub show_logs: bool,
    pub logs: Vec<String>,
    pub selected_transfer: Option<String>,
}

impl TuiState {
    pub fn new(file_name: String, file_size: u64, destination: String, hosts: &[(String, &HostEntry)]) -> Self {
        let mut transfers = HashMap::new();
        
        for (name, entry) in hosts {
            transfers.insert(name.clone(), TransferProgress {
                host_name: name.clone(),
                host_alias: entry.alias.clone(),
                bytes_transferred: 0,
                total_bytes: file_size,
                status: TransferStatus::Pending,
                speed: 0.0,
                eta: None,
                error_message: None,
            });
        }

        Self {
            file_name,
            file_size,
            destination,
            transfers,
            start_time: Instant::now(),
            is_paused: false,
            show_logs: false,
            logs: Vec::new(),
            selected_transfer: None,
        }
    }

    pub fn update_progress(&mut self, host_name: &str, bytes_transferred: u64, status: TransferStatus) {
        // Créer les variables pour le log avant de faire l'emprunt mutable
        let host_name_str = host_name.to_string();
        let mut should_log = false;
        let mut log_message = String::new();

        if let Some(transfer) = self.transfers.get_mut(host_name) {
            let old_bytes = transfer.bytes_transferred;
            transfer.bytes_transferred = bytes_transferred;
            transfer.status = status;

            // Calculer la vitesse
            let elapsed = self.start_time.elapsed();
            if elapsed.as_secs() > 0 {
                transfer.speed = bytes_transferred as f64 / elapsed.as_secs_f64();
            }

            // Calculer l'ETA
            if transfer.speed > 0.0 && bytes_transferred < transfer.total_bytes {
                let remaining_bytes = transfer.total_bytes - bytes_transferred;
                let eta_seconds = remaining_bytes as f64 / transfer.speed;
                transfer.eta = Some(Duration::from_secs_f64(eta_seconds));
            }

            // Préparer le message de log
            if bytes_transferred > old_bytes {
                should_log = true;
                log_message = format!(
                    "{}: {} / {} ({}%)",
                    host_name_str,
                    format_bytes(bytes_transferred),
                    format_bytes(transfer.total_bytes),
                    (bytes_transferred * 100 / transfer.total_bytes.max(1))
                );
            }
        }

        // Ajouter au log après avoir libéré l'emprunt
        if should_log {
            self.add_log(&log_message);
        }
    }

    pub fn set_error(&mut self, host_name: &str, error: String) {
        if let Some(transfer) = self.transfers.get_mut(host_name) {
            transfer.status = TransferStatus::Failed;
            transfer.error_message = Some(error.clone());
            self.add_log(&format!("❌ {}: {}", host_name, error));
        }
    }

    pub fn add_log(&mut self, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        self.logs.push(format!("[{}] {}", timestamp, message));
        
        // Garder seulement les 1000 derniers logs
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }

    pub fn get_summary(&self) -> (usize, usize, usize) {
        let completed = self.transfers.values().filter(|t| t.status == TransferStatus::Completed).count();
        let failed = self.transfers.values().filter(|t| t.status == TransferStatus::Failed).count();
        let total = self.transfers.len();
        (completed, failed, total)
    }

    pub fn get_total_speed(&self) -> f64 {
        self.transfers.values()
            .filter(|t| t.status == TransferStatus::Transferring)
            .map(|t| t.speed)
            .sum()
    }

    pub fn get_overall_eta(&self) -> Option<Duration> {
        let total_remaining: u64 = self.transfers.values()
            .filter(|t| t.status != TransferStatus::Completed && t.status != TransferStatus::Failed)
            .map(|t| t.total_bytes - t.bytes_transferred)
            .sum();

        let total_speed = self.get_total_speed();
        
        if total_speed > 0.0 && total_remaining > 0 {
            Some(Duration::from_secs_f64(total_remaining as f64 / total_speed))
        } else {
            None
        }
    }
}

/// Application TUI principale avec architecture modulaire
pub struct TuiApp {
    state: Arc<Mutex<TuiState>>,
    event_handler: EventHandler,
}

impl TuiApp {
    pub fn new(state: Arc<Mutex<TuiState>>) -> Self {
        let event_handler = EventHandler::new(Arc::clone(&state));
        Self {
            state,
            event_handler,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            // Dessiner l'interface
            terminal.draw(|f| self.render(f))?;

            // Gérer les événements
            if event::poll(Duration::from_millis(100))? {
                let event = event::read()?;
                self.event_handler.handle_event(event)?;
            }

            if self.event_handler.should_quit() {
                break;
            }
        }
        Ok(())
    }

    fn render(&self, f: &mut Frame) {
        let state = self.state.lock().unwrap();
        
        // Découper l'écran en zones
        let chunks = AppLayout::split_main(f.size());

        // Rendu de l'en-tête
        Header::render(f, chunks[0], &state);

        // Zone principale avec ou sans logs
        if state.show_logs {
            let main_chunks = AppLayout::split_main_with_logs(chunks[1]);
            ProgressView::render(f, main_chunks[0], &state);
            LogsView::render(f, main_chunks[1], &state);
        } else {
            ProgressView::render(f, chunks[1], &state);
        }

        // Barre de statut
        StatusBar::render(f, chunks[2], &state);

        // Contrôles
        Controls::render(f, chunks[3]);
    }
}

/// Point d'entrée principal pour l'interface TUI
pub fn run_tui(state: Arc<Mutex<TuiState>>) -> Result<()> {
    // Configuration du terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Lancer l'application TUI
    let mut app = TuiApp::new(state);
    let res = app.run(&mut terminal);

    // Restaurer le terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

// Fonction utilitaire pour formater les bytes
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
