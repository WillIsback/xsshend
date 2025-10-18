// Module SSH Agent pour xsshend - Interface avec ssh-agent système
use anyhow::{Context, Result};
use russh::keys::agent::client::AgentClient;
use russh::keys::ssh_key::PublicKey;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Gestionnaire de l'agent SSH
pub struct SshAgentManager {
    client: Option<Arc<Mutex<AgentClient<tokio::net::UnixStream>>>>,
}

impl SshAgentManager {
    /// Créer un nouveau gestionnaire d'agent SSH
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Se connecter à l'agent SSH via SSH_AUTH_SOCK
    pub async fn connect(&mut self) -> Result<()> {
        log::debug!("🔑 Tentative de connexion à ssh-agent...");

        // Connexion via SSH_AUTH_SOCK
        let agent_client = AgentClient::connect_env()
            .await
            .context("Impossible de se connecter à ssh-agent (SSH_AUTH_SOCK)")?;

        self.client = Some(Arc::new(Mutex::new(agent_client)));

        log::info!("✅ Connexion à ssh-agent réussie");
        Ok(())
    }

    /// Obtenir la liste des clés disponibles dans l'agent
    pub async fn list_identities(&self) -> Result<Vec<PublicKey>> {
        if let Some(ref client) = self.client {
            let mut client_guard = client.lock().await;
            let identities = client_guard
                .request_identities()
                .await
                .context("Impossible de récupérer les identités de ssh-agent")?;

            log::debug!("🔑 {} clé(s) trouvée(s) dans ssh-agent", identities.len());
            Ok(identities)
        } else {
            anyhow::bail!("ssh-agent n'est pas connecté");
        }
    }

    /// Obtenir un clone du client pour l'authentification
    pub fn get_client(&self) -> Option<Arc<Mutex<AgentClient<tokio::net::UnixStream>>>> {
        self.client.clone()
    }

    /// Essayer de se connecter à ssh-agent sans erreur fatale
    pub async fn try_connect() -> Option<Self> {
        let mut manager = Self::new();
        match manager.connect().await {
            Ok(()) => {
                log::info!("🔐 ssh-agent disponible et connecté");
                Some(manager)
            }
            Err(e) => {
                log::debug!("ℹ️  ssh-agent non disponible: {}", e);
                None
            }
        }
    }
}

impl Default for SshAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_connection_no_panic() {
        // Test que la connexion à l'agent ne provoque pas de panic
        // même si l'agent n'est pas disponible
        let result = SshAgentManager::try_connect().await;
        // Le résultat dépend de l'environnement, mais ne doit pas paniquer
        if result.is_some() {
            println!("ssh-agent est disponible dans l'environnement de test");
        } else {
            println!("ssh-agent n'est pas disponible dans l'environnement de test");
        }
    }
}
