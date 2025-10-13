// Bibliothèque publique pour xsshend
// Expose les modules pour les tests et l'utilisation en tant que bibliothèque

pub mod config;
pub mod core;
pub mod ssh;
pub mod utils;

// Re-exports pour faciliter l'utilisation
pub use config::{HostEntry, HostsConfig};
pub use core::{uploader::Uploader, validator::Validator};
pub use ssh::{
    client::SshClient,
    keys::{SshKey, SshKeyManager, SshKeyType, SshKeyWithPassphrase},
};
