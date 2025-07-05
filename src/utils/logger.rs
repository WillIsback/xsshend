// Système de logging pour xsshend
use log::{info, warn, error, debug};

pub struct XsshendLogger;

impl XsshendLogger {
    pub fn init() {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    pub fn log_upload_start(file_count: usize, host_count: usize) {
        info!("🚀 Démarrage téléversement: {} fichier(s) vers {} serveur(s)", file_count, host_count);
    }

    pub fn log_upload_success(file: &str, host: &str) {
        info!("✅ {} téléversé avec succès vers {}", file, host);
    }

    pub fn log_upload_error(file: &str, host: &str, error: &str) {
        error!("❌ Échec téléversement {} vers {}: {}", file, host, error);
    }

    pub fn log_connection_attempt(host: &str) {
        debug!("🔌 Tentative de connexion à {}", host);
    }

    pub fn log_connection_success(host: &str) {
        info!("✅ Connexion établie avec {}", host);
    }

    pub fn log_connection_error(host: &str, error: &str) {
        warn!("⚠️  Échec connexion à {}: {}", host, error);
    }

    pub fn log_progress(file: &str, host: &str, percent: u8) {
        debug!("📊 {} → {}: {}%", file, host, percent);
    }
}
