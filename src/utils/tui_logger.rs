use log::{Metadata, Record}; // Removed unused Level
use std::sync::{Arc, Mutex};

/// Logger personnalisé qui peut rediriger les logs vers le TUI ou vers la sortie standard
pub struct TuiAwareLogger {
    tui_log_sender: Option<Arc<Mutex<dyn Fn(String) + Send + Sync>>>,
    default_logger: env_logger::Logger,
}

impl TuiAwareLogger {
    /// Crée un nouveau logger TUI-aware
    pub fn new() -> Self {
        let default_logger = env_logger::Builder::from_default_env()
            .target(env_logger::Target::Stderr)
            .build();

        Self {
            tui_log_sender: None,
            default_logger,
        }
    }

    // Unused methods - commented out for optimization
    // pub fn set_tui_sender<F>(&mut self, sender: F)
    // where
    //     F: Fn(String) + Send + Sync + 'static,
    // {
    //     self.tui_log_sender = Some(Arc::new(Mutex::new(sender)));
    // }

    // pub fn disable_tui_mode(&mut self) {
    //     self.tui_log_sender = None;
    // }

    // pub fn is_tui_mode(&self) -> bool {
    //     self.tui_log_sender.is_some()
    // }
}

impl log::Log for TuiAwareLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Toujours activer si on a un sender TUI, sinon utiliser le logger par défaut
        if self.tui_log_sender.is_some() {
            true
        } else {
            self.default_logger.enabled(metadata)
        }
    }

    fn log(&self, record: &Record) {
        if let Some(ref sender) = self.tui_log_sender {
            // Mode TUI : envoyer vers le TUI
            let message = format!("{}", record.args());
            if let Ok(sender_fn) = sender.lock() {
                sender_fn(message);
            }
        } else {
            // Mode normal : utiliser le logger par défaut
            self.default_logger.log(record);
        }
    }

    fn flush(&self) {
        self.default_logger.flush();
    }
}

/// Logger global singleton
static mut GLOBAL_LOGGER: Option<Box<TuiAwareLogger>> = None;
static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialise le logger global
pub fn init_tui_aware_logger() {
    LOGGER_INIT.call_once(|| {
        let logger = Box::new(TuiAwareLogger::new());
        unsafe {
            GLOBAL_LOGGER = Some(logger);
            if let Some(ref logger) = GLOBAL_LOGGER {
                log::set_logger(logger.as_ref()).expect("Failed to set logger");
                log::set_max_level(log::LevelFilter::Debug);
            }
        }
    });
}

// Unused functions - commented out for optimization
// pub fn enable_tui_logging<F>(sender: F)
// where
//     F: Fn(String) + Send + Sync + 'static,
// {
//     unsafe {
//         if let Some(ref mut logger) = GLOBAL_LOGGER {
//             logger.set_tui_sender(sender);
//         }
//     }
// }

// pub fn disable_tui_logging() {
//     unsafe {
//         if let Some(ref mut logger) = GLOBAL_LOGGER {
//             logger.disable_tui_mode();
//         }
//     }
// }

// pub fn is_tui_logging_enabled() -> bool {
//     unsafe {
//         if let Some(ref logger) = GLOBAL_LOGGER {
//             logger.is_tui_mode()
//         } else {
//             false
//         }
//     }
// }
