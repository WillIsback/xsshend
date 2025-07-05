// Types d'erreurs personnalisés pour xsshend
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XsshendError {
    #[error("Erreur de configuration: {0}")]
    Config(String),

    #[error("Erreur de connexion SSH: {0}")]
    SshConnection(String),

    #[error("Erreur de transfert de fichier: {0}")]
    FileTransfer(String),

    #[error("Erreur d'authentification: {0}")]
    Authentication(String),

    #[error("Fichier non trouvé: {0}")]
    FileNotFound(String),

    #[error("Permission refusée: {0}")]
    PermissionDenied(String),

    #[error("Timeout de connexion: {0}")]
    Timeout(String),

    #[error("Erreur réseau: {0}")]
    Network(String),

    #[error("Erreur interne: {0}")]
    Internal(String),
}

pub type XsshendResult<T> = Result<T, XsshendError>;
