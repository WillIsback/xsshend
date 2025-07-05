use log::{Log, Metadata, Record};
use std::sync::{Arc, Mutex};

/// Logger personnalisé qui capture tous les logs pour les rediriger vers le TUI
pub struct TuiLogger {
    log_sink: Arc<Mutex<Vec<String>>>,
    fallback_logger: env_logger::Logger,
}

impl TuiLogger {
    /// Crée un nouveau logger TUI avec un buffer de logs partagé
    pub fn new(log_sink: Arc<Mutex<Vec<String>>>) -> Self {
        let fallback_logger = env_logger::Builder::from_default_env()
            .target(env_logger::Target::Stderr)
            .build();

        Self {
            log_sink,
            fallback_logger,
        }
    }

    /// Initialise le logger TUI comme logger global
    pub fn init(log_sink: Arc<Mutex<Vec<String>>>) -> Result<(), log::SetLoggerError> {
        let logger = Box::new(Self::new(log_sink));
        log::set_boxed_logger(logger)?;
        log::set_max_level(log::LevelFilter::Debug);
        Ok(())
    }

    /// Essaie d'initialiser le logger TUI, mais ne fait rien si un logger est déjà actif
    pub fn try_init(log_sink: Arc<Mutex<Vec<String>>>) -> bool {
        Self::init(log_sink).is_ok()
    }
}

impl Log for TuiLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Accepter tous les logs de niveau Debug et plus élevé
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = chrono::Local::now().format("%H:%M:%S");
            let level_icon = match record.level() {
                log::Level::Error => "❌",
                log::Level::Warn => "⚠️",
                log::Level::Info => "ℹ️",
                log::Level::Debug => "🔍",
                log::Level::Trace => "📝",
            };

            let message = format!("[{}] {} {}", timestamp, level_icon, record.args());

            // Essayer d'ajouter au buffer TUI
            let added_to_tui = if let Ok(mut logs) = self.log_sink.lock() {
                logs.push(message.clone());

                // Garder seulement les 1000 derniers logs pour éviter une consommation excessive de mémoire
                if logs.len() > 1000 {
                    logs.remove(0);
                }
                true
            } else {
                false
            };

            // Utiliser aussi le logger de fallback pour assurer que les logs ne sont pas perdus
            if !added_to_tui {
                self.fallback_logger.log(record);
            }
        }
    }

    fn flush(&self) {
        self.fallback_logger.flush();
    }
}

/// Fonction utilitaire pour créer un buffer de logs partagé
pub fn create_shared_log_buffer() -> Arc<Mutex<Vec<String>>> {
    Arc::new(Mutex::new(Vec::new()))
}
