# Documentation xsshend

## Architecture du Projet

Le projet xsshend suit une architecture modulaire claire :

### Structure des Modules

```
src/
├── main.rs              # Point d'entrée et CLI principal
├── config/
│   ├── mod.rs           # Module de configuration
│   └── hosts.rs         # Parsing et gestion du fichier hosts.json
├── ssh/
│   ├── mod.rs           # Module SSH
│   ├── client.rs        # Client SSH/SFTP
│   ├── auth.rs          # Authentification SSH
│   └── transfer.rs      # Logique de transfert avec progression
├── ui/
│   ├── mod.rs           # Module interface utilisateur
│   └── prompts.rs       # Dialogues interactifs
├── core/
│   ├── mod.rs           # Module logique métier
│   ├── uploader.rs      # Orchestrateur principal des téléversements
│   ├── parallel.rs      # Gestion du parallélisme
│   └── validator.rs     # Validation des fichiers et serveurs
└── utils/
    ├── mod.rs           # Module utilitaires
    ├── errors.rs        # Types d'erreurs personnalisés
    └── logger.rs        # Système de logging
```

### Configuration hosts.json

La configuration suit une hiérarchie à 4 niveaux :

```
Environnement → Région → Type → Serveur
    ↓           ↓        ↓      ↓
Production → Region-A → Public → WEB_SERVER_01
```

### Stack Technologique

- **SSH/SFTP** : `ssh2` (bindings libssh2)
- **Interface** : `indicatif` + `dialoguer` + `crossterm`
- **CLI** : `clap` v4.x avec dérivation
- **Parallélisme** : `rayon` pour threading natif
- **Configuration** : `serde` + `serde_json`
- **Erreurs** : `anyhow` + `thiserror`
- **Logs** : `log` + `env_logger`

## Utilisation

### Commandes de Base

```bash
# Lister tous les serveurs
xsshend list

# Filtrer par environnement
xsshend list --env Production

# Téléversement simple
xsshend upload file.txt --env Production --dry-run

# Mode interactif
xsshend upload file.txt --interactive
```

### Configuration

Fichier `~/.ssh/hosts.json` requis avec structure hiérarchique.

## Tests

```bash
# Tests unitaires
cargo test

# Tests d'intégration
cargo test --test integration

# Benchmarks
cargo bench
```
