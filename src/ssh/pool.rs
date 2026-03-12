// Pool de connexions SSH réutilisables — évite un handshake TLS par opération
//
// Architecture :
//   DashMap<host_key, Arc<Mutex<SshClient>>>  → partage thread-safe par hôte
//   Semaphore(max_concurrent)                 → borne la concurrence globale
//
// Gain typique : -60 à -80% de latence sur runs répétés vers les mêmes hôtes
// (un handshake SSH coûte 100–300 ms sur un réseau LAN/VPN d'entreprise).

use crate::ssh::client::SshClient;
use crate::ssh::keys::PassphraseCache;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

/// Timeout de connexion initiale
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Concurrence max par défaut : 10 connexions simultanées
const DEFAULT_MAX_CONCURRENT: usize = 10;

/// Pool de connexions SSH partagé entre les tâches parallèles.
///
/// Clone est cheap — toutes les copies partagent les mêmes structures Arc internes.
pub struct ConnectionPool {
    /// Connexions indexées par "user@host"
    connections: Arc<DashMap<String, Arc<Mutex<SshClient>>>>,
    /// Borne la concurrence globale pour ne pas saturer les serveurs cibles
    semaphore: Arc<Semaphore>,
    /// Cache de passphrases partagé (évite de demander N fois la même passphrase)
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
    /// Créer un pool avec la concurrence par défaut (10)
    pub fn new(passphrase_cache: PassphraseCache) -> Self {
        Self::with_concurrency(DEFAULT_MAX_CONCURRENT, passphrase_cache)
    }

    /// Créer un pool avec une limite de concurrence personnalisée
    pub fn with_concurrency(max_concurrent: usize, passphrase_cache: PassphraseCache) -> Self {
        ConnectionPool {
            connections: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            passphrase_cache,
        }
    }

    /// Acquérir une connexion SSH pour l'hôte donné.
    ///
    /// - Retourne une connexion existante si elle est disponible (réutilisation)
    /// - Crée une nouvelle connexion sinon
    /// - Bloque si la limite de concurrence est atteinte (backpressure)
    ///
    /// Le `OwnedSemaphorePermit` retourné doit être conservé tant que la connexion
    /// est utilisée — il libère automatiquement un slot à sa destruction.
    pub async fn acquire(
        &self,
        host_key: &str,
        username: &str,
        host: &str,
    ) -> Result<(Arc<Mutex<SshClient>>, OwnedSemaphorePermit)> {
        // Acquérir un slot de concurrence avant toute opération réseau
        let permit = Arc::clone(&self.semaphore)
            .acquire_owned()
            .await
            .map_err(|e| anyhow::anyhow!("Pool semaphore fermé : {}", e))?;

        // Connexion existante ?
        if let Some(existing) = self.connections.get(host_key) {
            log::debug!("♻️  Réutilisation connexion SSH : {}", host_key);
            return Ok((Arc::clone(existing.value()), permit));
        }

        // Nouvelle connexion
        log::debug!("🔌 Nouvelle connexion SSH : {}@{}", username, host);
        let mut client =
            SshClient::new_with_cache(host, username, self.passphrase_cache.clone())?;
        client.connect_with_timeout(CONNECT_TIMEOUT).await?;

        let arc = Arc::new(Mutex::new(client));
        self.connections
            .insert(host_key.to_string(), Arc::clone(&arc));

        Ok((arc, permit))
    }

    /// Invalider une connexion (en cas d'erreur réseau — elle sera recréée au prochain appel)
    pub fn invalidate(&self, host_key: &str) {
        log::debug!("🗑️  Invalidation connexion SSH : {}", host_key);
        self.connections.remove(host_key);
    }

    /// Fermer proprement toutes les connexions
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

    /// Nombre de connexions actives dans le pool
    pub fn active_connections(&self) -> usize {
        self.connections.len()
    }
}

impl Drop for ConnectionPool {
    fn drop(&mut self) {
        // Si on est le dernier propriétaire des connexions, on tente un nettoyage
        if Arc::strong_count(&self.connections) == 1 && !self.connections.is_empty() {
            let count = self.connections.len();
            log::debug!(
                "ConnectionPool dropped avec {} connexion(s) active(s)",
                count
            );
            // Nettoyage best-effort via spawn (Drop ne peut pas await)
            let connections = Arc::clone(&self.connections);
            tokio::spawn(async move {
                let keys: Vec<String> =
                    connections.iter().map(|e| e.key().clone()).collect();
                for key in keys {
                    if let Some((_, arc)) = connections.remove(&key) {
                        if let Ok(mut client) = arc.try_lock() {
                            let _ = client.disconnect().await;
                        }
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let cache = PassphraseCache::new();
        let pool = ConnectionPool::new(cache);
        assert_eq!(pool.active_connections(), 0);
    }

    #[test]
    fn test_pool_clone_shares_state() {
        let cache = PassphraseCache::new();
        let pool = ConnectionPool::new(cache);
        let pool2 = pool.clone();
        // Les deux pools partagent le même DashMap
        assert!(Arc::ptr_eq(&pool.connections, &pool2.connections));
    }

    #[test]
    fn test_pool_with_concurrency() {
        let cache = PassphraseCache::new();
        let pool = ConnectionPool::with_concurrency(5, cache);
        assert_eq!(pool.active_connections(), 0);
    }
}
