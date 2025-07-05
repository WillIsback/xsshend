// Système de logging pour xsshend
use log::info;

pub struct XsshendLogger;

impl XsshendLogger {
    pub fn log_upload_start(file_count: usize, host_count: usize) {
        info!(
            "🚀 Démarrage téléversement: {} fichier(s) vers {} serveur(s)",
            file_count, host_count
        );
    }
}
