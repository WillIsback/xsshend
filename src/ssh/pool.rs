use crate::ssh::client::SshClient;
use crate::ssh::keys::PassphraseCache;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_MAX_CONCURRENT: usize = 10;

/// Pool de connexions SSH partagé entre les tâches parallèles.
/// Clone cheap — toutes les copies partagent les mêmes Arc internes.
pub struct ConnectionPool {
    connections: Arc<DashMap<String, Arc<Mutex<SshClient>>>>,
    semaphore: Arc<Semaphore>,
    passphrase_cache: PassphraseCache,
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        ConnectionPool {
            connections: Arc::clone(&self.connections),
            semaphore: Arc::clone(&self.semaphore),
            passphrase_cache: self.passphrase_cache.clone(),
        }
    }
}

impl ConnectionPool {
    pub fn new(passphrase_cache: PassphraseCache) -> Self {
        Self::with_concurrency(DEFAULT_MAX_CONCURRENT, passphrase_cache)
    }

    pub fn with_concurrency(max_concurrent: usize, passphrase_cache: PassphraseCache) -> Self {
        ConnectionPool {
            connections: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            passphrase_cache,
        }
    }

    /// Acquérir (ou réutiliser) une connexion SSH.
    /// Retourne un OwnedSemaphorePermit à conserver pendant toute la durée d'utilisation.
    pub async fn acquire(
        &self,
        host_key: &str,
        username: &str,
        host: &str,
    ) -> Result<(Arc<Mutex<SshClient>>, OwnedSemaphorePermit)> {
        let permit = Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|e| anyhow::anyhow!("Pool semaphore fermé : {}", e))?;

        if let Some(existing) = self.connections.get(host_key) {
            log::debug!("♻️  Réutilisation connexion SSH : {}", host_key);
            return Ok((Arc::clone(existing.value()), permit));
        }

        log::debug!("🔌 Nouvelle connexion SSH : {}@{}", username, host);
        let mut client =
            SshClient::new_with_cache(host, username, self.passphrase_cache.clone())?;
        client.connect_with_timeout(CONNECT_TIMEOUT).await?;

        let arc = Arc::new(Mutex::new(client));
        self.connections
            .insert(host_key.to_string(), Arc::clone(&arc));

        Ok((arc, permit))
    }

    /// Invalider une connexion morte — sera recréée au prochain appel.
    pub fn invalidate(&self, host_key: &str) {
        log::debug!("🗑️  Invalidation connexion SSH : {}", host_key);
        self.connections.remove(host_key);
    }

    /// Fermer proprement toutes les connexions du pool.
    pub async fn close_all(&self) {
        let keys: Vec<String> = self.connections.iter().map(|e| e.key().clone()).collect();
        for key in keys {
            if let Some((_, client_arc)) = self.connections.remove(&key) {
                if let Ok(mut client) = client_arc.try_lock() {
                    let _ = client.disconnect().await;
                }
            }
        }
    }

    pub fn active_connections(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = ConnectionPool::new(PassphraseCache::new());
        assert_eq!(pool.active_connections(), 0);
    }

    #[test]
    fn test_pool_clone_shares_state() {
        let pool = ConnectionPool::new(PassphraseCache::new());
        let pool2 = pool.clone();
        assert!(Arc::ptr_eq(&pool.connections, &pool2.connections));
    }
}
