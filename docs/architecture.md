# Documentation xsshend

## Architecture du Projet

Le projet xsshend suit une architecture modulaire claire :

### Structure des Modules

```rust
src/
├── main.rs              # Point d'entrée et CLI principal
├── config/
│   ├── mod.rs           # Module de configuration
│   └── hosts.rs         # Parsing et gestion du fichier hosts.json
├── ssh/
│   ├── mod.rs           # Module SSH
│   ├── client.rs        # Client SSH/SFTP réel avec ssh2-rs
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

### Module de Test Intégré

```
test/                           # Module de test en conditions réelles
├── README.md                   # Guide d'utilisation du module
├── demo.sh                     # Démonstration complète automatisée
├── test-vms.sh                 # Gestionnaire de VMs Multipass
├── generate-test-files.sh      # Générateur de fichiers de test
├── run-integration-tests.sh    # Suite de tests d'intégration (9 tests)
├── multipass/
│   └── cloud-init.yaml         # Configuration automatisée des VMs
├── configs/
│   └── test-hosts.json         # Configuration hosts.json de test
├── data/                       # Fichiers de test (généré automatiquement)
└── .ssh/                       # Clés SSH de test (généré automatiquement)
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

## Tests et Validation

### Tests Unitaires

```bash
# Tests des modules individuels
cargo test

# Tests avec couverture
cargo test --verbose
```

### Tests d'Intégration avec Multipass

Le module `/test` fournit un environnement de test complet :

```bash
# Démonstration complète (recommandé)
cd test/
./demo.sh

# Setup manuel étape par étape
./test-vms.sh generate-keys        # Génération clés SSH
./test-vms.sh launch-all           # Lancement 5 VMs Ubuntu
./test-vms.sh generate-config      # Configuration hosts.json
./run-integration-tests.sh         # Suite de 9 tests automatisés
```

#### VMs de Test Multipass

| VM | Environnement | Resources | Utilisateurs |
|----|---------------|-----------|--------------|
| xsshend-dev | Development | 1 CPU, 1GB | xsshend-test |
| xsshend-staging | Staging | 1 CPU, 1GB | deploy |
| xsshend-prod-web | Production | 2 CPU, 2GB | deploy |
| xsshend-prod-api | Production | 2 CPU, 2GB | api |
| xsshend-prod-db | Production | 1 CPU, 2GB | xsshend-test |

#### Tests Automatisés

1. **CLI et aide** - Interface ligne de commande
2. **Configuration** - Chargement hosts.json et filtrage  
3. **Dry-run** - Mode simulation sans transfert
4. **Upload simple** - Transfert fichier unique
5. **Upload multiple** - Transfert plusieurs fichiers
6. **Gros fichier** - Test barres de progression
7. **Parallèle** - Transfert multi-serveurs simultané
8. **Gestion d'erreurs** - Tests de robustesse
9. **Performance** - Tests de stress et timing

### Exemple de Résultat

```
📊 RÉSUMÉ DES TESTS xsshend
  Total:   9 tests
  ✅ Réussis: 9
  ❌ Échecs:  0
  
🎉 TOUS LES TESTS SONT PASSÉS!
xsshend v0.1.0 est prêt pour la production!
```
