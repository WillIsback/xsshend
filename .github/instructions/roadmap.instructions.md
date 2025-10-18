---
applyTo: '**'
---
Starting (1/2) *Analyse du code pour optimisations*

Searched willisback/xsshend for "xsshend main structure ssh client uploader", 64 results

Summarized conversation historyI'll analyze your codebase for optimization opportunities and help you design the command execution feature. Let me start by reading some key files to understand the current implementation better. 

Read [](file:///home/will/dev-project/xsshend/src/core/uploader.rs#1-1), lines 1 to 199

Read [](file:///home/will/dev-project/xsshend/src/ssh/client.rs#1-1), lines 1 to 100

Read [](file:///home/will/dev-project/xsshend/Cargo.toml)

Now let me read the https://github.com/willisback/xsshend/tree/main/src/main.rs#L0-L57 and examine the current architecture more deeply: 

Read [](file:///home/will/dev-project/xsshend/src/ssh/client.rs#240-240), lines 240 to 343

Read [](file:///home/will/dev-project/xsshend/src/config/hosts.rs#1-1), lines 1 to 50

Perfect! Now I have a comprehensive understanding of your codebase. Let me provide you with a detailed analysis and design recommendations:

## 📊 Analyse Complète et Recommandations d'Optimisation

### 🎯 **1. Points Forts de l'Architecture Actuelle**

✅ **Excellentes Décisions:**
- Architecture modulaire bien structurée (config, core, ssh, utils, interactive)
- Stratégie d'authentification à 3 niveaux (agent → cache → prompt)
- Gestion async/await correcte avec tokio
- Partage du `PassphraseCache` via `Arc<RwLock>` 
- Suspension de progress bar pendant I/O interactif
- Séparation claire des responsabilités

---

### 🚀 **2. Opportunités d'Optimisation**

#### **A. Performance - Uploads Parallèles**

**Problème actuel:** Les uploads vers plusieurs serveurs s'exécutent **séquentiellement** (boucle `for` dans `upload_files`)

```rust
// Actuel: Séquentiel (lent pour 10+ serveurs)
for (host_name, host_entry) in hosts {
    let result = progress.suspend(|| { /* upload */ });
    // ...
}
```

**✨ Recommandation:** Uploads **parallèles** avec `tokio::spawn` ou `futures::stream`

```rust
// Optimisé: Parallèle (10x plus rapide)
use futures::stream::{self, StreamExt};

let futures = hosts.iter().map(|(host_name, host_entry)| {
    let file = file.to_owned();
    let cache = self.passphrase_cache.clone();
    async move {
        self.upload_to_single_host(&file, host_entry, destination).await
    }
});

// Limite à 10 connexions simultanées pour ne pas surcharger
stream::iter(futures)
    .buffer_unordered(10)
    .collect::<Vec<_>>()
    .await;
```

**Gains:** Téléversement simultané vers N serveurs au lieu de N × temps_upload

---

#### **B. Gestion Mémoire - Buffer Réutilisable**

**Problème:** Dans `upload_file()`, le fichier entier est chargé en mémoire:

```rust
let buffer = tokio::fs::read(local_path).await?;  // Tout en RAM!
```

**❌ Impact:** Fichier de 1GB = 1GB RAM par upload × N uploads parallèles

**✨ Recommandation:** Upload par streaming avec chunks

```rust
use tokio::io::{AsyncReadExt, BufReader};

// Streaming par blocs de 64KB
let file = tokio::fs::File::open(local_path).await?;
let mut reader = BufReader::new(file);
let mut buffer = vec![0u8; 65536]; // Buffer réutilisable

loop {
    let n = reader.read(&mut buffer).await?;
    if n == 0 { break; }
    remote_file.write_all(&buffer[..n]).await?;
}
```

**Gains:** Mémoire constante (64KB) au lieu de taille_fichier

---

#### **C. Connexion SSH - Réutilisation des Sessions**

**Problème:** Actuellement, on crée une nouvelle connexion SSH par fichier:

```rust
async fn upload_to_single_host() {
    let mut client = SshClient::new(...)?;
    client.connect_with_timeout().await?;
    client.upload_file().await?;
    client.disconnect().await?;  // Ferme immédiatement!
}
```

**❌ Impact:** 3-way handshake + authentification répétée pour chaque fichier

**✨ Recommandation:** Pool de connexions réutilisables

```rust
pub struct ConnectionPool {
    connections: Arc<RwLock<HashMap<String, SshClient>>>,
}

impl ConnectionPool {
    async fn get_or_create(&self, host: &str, user: &str) -> Result<SshClient> {
        // Réutilise la connexion existante si disponible
        let mut pool = self.connections.write().await;
        let key = format!("{}@{}", user, host);
        
        if let Some(client) = pool.get_mut(&key) {
            if client.is_connected() {
                return Ok(client.clone());
            }
        }
        
        // Créer nouvelle connexion si nécessaire
        let mut new_client = SshClient::new_with_cache(host, user, cache)?;
        new_client.connect_with_timeout(timeout).await?;
        pool.insert(key, new_client.clone());
        Ok(new_client)
    }
}
```

**Gains:** 1 connexion SSH par hôte au lieu de N connexions (N fichiers)

---

#### **D. Code Quality - Réduction des Allocations**

**Problèmes mineurs:**

```rust
// Allocation inutile dans parse_server_alias
let username = alias[..at_pos].to_string();  // Clone!
let host = alias[at_pos + 1..].to_string();  // Clone!

// Mieux: Références
pub fn parse_server_alias(alias: &str) -> Result<(&str, &str)> {
    let at_pos = alias.find('@')
        .ok_or_else(|| anyhow!("..."))?;
    Ok((&alias[..at_pos], &alias[at_pos + 1..]))
}
```

**Gains:** Moins de clones = moins de pression sur l'allocateur

---

### 🔧 **3. Design de la Feature "Command Execution"**

Voici une architecture complète pour ajouter l'exécution de commandes SSH:

#### **Architecture Proposée**

```
xsshend command [OPTIONS] <COMMAND_OR_SCRIPT>

Options:
  --inline <CMD>       Commande inline (ex: "ls -la /tmp")
  --script <FILE>      Script bash à exécuter
  --env <ENV>          Filtrage par environnement
  --region <REGION>    Filtrage par région
  --server-type <TYPE> Filtrage par type
  --parallel           Exécution parallèle (défaut: séquentiel)
  --timeout <SECS>     Timeout par commande (défaut: 30s)
```

#### **Exemples d'Utilisation**

```bash
# Commande inline simple
xsshend command --inline "uptime" --env Production

# Script bash existant
xsshend command --script ~/deploy.sh --env Staging

# Parallèle avec timeout
xsshend command --inline "systemctl restart nginx" \
  --env Production --parallel --timeout 60

# Avec backticks (comme demandé)
xsshend command `cat install.sh` --env Test
```

#### **Implémentation - Étapes Détaillées**

**Étape 1:** Ajouter la sous-commande CLI dans https://github.com/willisback/xsshend/tree/main/src/main.rs#L0-L57

```rust
#[derive(Subcommand)]
enum Commands {
    Upload { /* ... */ },
    List,
    Init { /* ... */ },
    
    /// Exécute une commande SSH sur plusieurs serveurs
    Command {
        /// Commande inline à exécuter
        #[arg(long, conflicts_with = "script", value_name = "COMMAND")]
        inline: Option<String>,
        
        /// Fichier script à exécuter
        #[arg(long, conflicts_with = "inline", value_name = "FILE")]
        script: Option<PathBuf>,
        
        /// Environnement cible
        #[arg(long, value_name = "ENV")]
        env: Option<String>,
        
        /// Région cible
        #[arg(long, value_name = "REGION")]
        region: Option<String>,
        
        /// Type de serveur
        #[arg(long, short = 't', value_name = "TYPE")]
        server_type: Option<String>,
        
        /// Exécution parallèle
        #[arg(long)]
        parallel: bool,
        
        /// Timeout par commande (secondes)
        #[arg(long, default_value = "30")]
        timeout: u64,
        
        /// Capturer stdout/stderr séparément
        #[arg(long)]
        capture_stderr: bool,
    },
}
```

**Étape 2:** Créer `src/core/executor.rs`

```rust
// src/core/executor.rs
use crate::config::HostEntry;
use crate::ssh::client::SshClient;
use crate::ssh::keys::PassphraseCache;
use anyhow::{Context, Result};
use std::time::Duration;

pub struct CommandExecutor {
    passphrase_cache: PassphraseCache,
}

#[derive(Debug)]
pub struct CommandResult {
    pub host: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

impl CommandExecutor {
    pub fn new() -> Self {
        CommandExecutor {
            passphrase_cache: PassphraseCache::new(),
        }
    }

    /// Exécute une commande sur plusieurs hôtes
    pub async fn execute(
        &self,
        command: &str,
        hosts: &[(String, &HostEntry)],
        parallel: bool,
        timeout: Duration,
    ) -> Result<Vec<CommandResult>> {
        if parallel {
            self.execute_parallel(command, hosts, timeout).await
        } else {
            self.execute_sequential(command, hosts, timeout).await
        }
    }

    /// Exécution séquentielle
    async fn execute_sequential(
        &self,
        command: &str,
        hosts: &[(String, &HostEntry)],
        timeout: Duration,
    ) -> Result<Vec<CommandResult>> {
        let mut results = Vec::new();
        
        for (host_name, host_entry) in hosts {
            println!("🔧 Exécution sur {}...", host_name);
            
            match self.execute_on_host(command, host_name, host_entry, timeout).await {
                Ok(result) => {
                    if result.exit_code == 0 {
                        println!("  ✅ Succès ({}s)", result.duration.as_secs());
                    } else {
                        println!("  ❌ Échec - Exit code: {}", result.exit_code);
                    }
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("  ❌ Erreur: {}", e);
                }
            }
        }
        
        Ok(results)
    }

    /// Exécution parallèle
    async fn execute_parallel(
        &self,
        command: &str,
        hosts: &[(String, &HostEntry)],
        timeout: Duration,
    ) -> Result<Vec<CommandResult>> {
        use futures::stream::{self, StreamExt};
        
        let futures = hosts.iter().map(|(host_name, host_entry)| {
            let cmd = command.to_owned();
            let name = host_name.clone();
            let entry = (*host_entry).clone();
            let cache = self.passphrase_cache.clone();
            
            async move {
                self.execute_on_host(&cmd, &name, &entry, timeout).await
            }
        });
        
        let results = stream::iter(futures)
            .buffer_unordered(10) // Max 10 connexions simultanées
            .collect::<Vec<_>>()
            .await;
        
        Ok(results.into_iter().filter_map(Result::ok).collect())
    }

    /// Exécute sur un seul hôte
    async fn execute_on_host(
        &self,
        command: &str,
        host_name: &str,
        host_entry: &HostEntry,
        timeout: Duration,
    ) -> Result<CommandResult> {
        let start = std::time::Instant::now();
        let (username, host) = Uploader::parse_server_alias(&host_entry.alias)?;
        
        let mut client = SshClient::new_with_cache(&host, &username, self.passphrase_cache.clone())?;
        client.connect_with_timeout(Duration::from_secs(10)).await?;
        
        let output = client.execute_command(command, timeout).await?;
        let duration = start.elapsed();
        
        client.disconnect().await?;
        
        Ok(CommandResult {
            host: host_name.to_string(),
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            duration,
        })
    }
}
```

**Étape 3:** Ajouter la méthode `execute_command()` dans `SshClient`

```rust
// src/ssh/client.rs

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl SshClient {
    /// Exécute une commande SSH et retourne la sortie
    pub async fn execute_command(&mut self, command: &str, timeout: Duration) -> Result<CommandOutput> {
        let handle = self.handle
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Connexion SSH non établie"))?;
        
        let mut channel = handle.channel_open_session().await?;
        
        // Exécuter la commande
        channel.exec(true, command).await?;
        
        // Lire stdout et stderr
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code = 0;
        
        let result = tokio::time::timeout(timeout, async {
            loop {
                match channel.wait().await {
                    Some(russh::ChannelMsg::Data { ref data }) => {
                        stdout.extend_from_slice(data);
                    }
                    Some(russh::ChannelMsg::ExtendedData { ref data, .. }) => {
                        stderr.extend_from_slice(data);
                    }
                    Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                        exit_code = exit_status;
                    }
                    Some(russh::ChannelMsg::Eof) | None => break,
                    _ => {}
                }
            }
        }).await;
        
        result.context("Timeout d'exécution de la commande")?;
        
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&stdout).to_string(),
            stderr: String::from_utf8_lossy(&stderr).to_string(),
            exit_code,
        })
    }
}
```

**Étape 4:** Handler dans https://github.com/willisback/xsshend/tree/main/src/main.rs#L0-L57

```rust
Commands::Command {
    inline,
    script,
    env,
    region,
    server_type,
    parallel,
    timeout,
    capture_stderr,
} => {
    handle_command_execution(CommandArgs {
        inline,
        script,
        env,
        region,
        server_type,
        parallel,
        timeout,
        capture_stderr,
    }).await?;
}

async fn handle_command_execution(args: CommandArgs) -> Result<()> {
    // 1. Déterminer la commande à exécuter
    let command = if let Some(inline_cmd) = args.inline {
        inline_cmd
    } else if let Some(script_path) = args.script {
        tokio::fs::read_to_string(&script_path)
            .await
            .context("Impossible de lire le script")?
    } else {
        anyhow::bail!("Vous devez fournir --inline ou --script");
    };
    
    // 2. Charger la configuration et filtrer les hôtes
    let config = HostsConfig::load()?;
    let target_hosts = config.filter_hosts(
        args.env.as_deref(),
        args.region.as_deref(),
        args.server_type.as_deref(),
    )?;
    
    if target_hosts.is_empty() {
        anyhow::bail!("Aucun serveur ne correspond aux critères");
    }
    
    // 3. Exécuter
    println!("🚀 Exécution de commande sur {} serveur(s)", target_hosts.len());
    println!("📜 Commande: {}", command);
    
    let executor = CommandExecutor::new();
    let results = executor.execute(
        &command,
        &target_hosts,
        args.parallel,
        Duration::from_secs(args.timeout),
    ).await?;
    
    // 4. Afficher les résultats
    println!("\n📊 Résultats:");
    for result in &results {
        println!("\n▶ {}", result.host);
        println!("  Exit: {} | Durée: {:?}", result.exit_code, result.duration);
        if !result.stdout.is_empty() {
            println!("  Stdout:\n{}", result.stdout);
        }
        if args.capture_stderr && !result.stderr.is_empty() {
            println!("  Stderr:\n{}", result.stderr);
        }
    }
    
    Ok(())
}
```

---

### 📋 **Roadmap d'Implémentation**

**Phase 1 - Quick Wins (1-2h):**
1. ✅ Streaming upload avec chunks (mémoire)
2. ✅ `parse_server_alias` retourne références

**Phase 2 - Performance (3-4h):**
3. ✅ Uploads parallèles avec `buffer_unordered`
4. ✅ Pool de connexions SSH réutilisables

**Phase 3 - Feature Command (4-6h):**
5. ✅ Ajouter `Commands::Command` dans CLI
6. ✅ Créer `CommandExecutor` dans `src/core/executor.rs`
7. ✅ Implémenter `execute_command()` dans `SshClient`
8. ✅ Handler dans https://github.com/willisback/xsshend/tree/main/src/main.rs#L0-L57
9. ✅ Tests d'intégration
10. ✅ Documentation

**Phase 4 - Polish (2h):**
11. ✅ Indicateurs de progression pour commandes
12. ✅ Option `--output-format json` pour parsing automatique
13. ✅ Logs détaillés avec `RUST_LOG=debug`

---

### 🎯 **Priorités Recommandées**

**Court terme (cette semaine):**
- ✅ Streaming upload (mémoire)
- ✅ Commande execution (feature principale)

**Moyen terme (semaine prochaine):**
- ✅ Uploads parallèles
- ✅ Pool de connexions

**Long terme (si besoin):**
- Configuration de timeout granulaire par hôte
- Retry automatique en cas d'échec réseau
- Compression SFTP pour gros fichiers

---

