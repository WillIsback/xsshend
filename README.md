# 🚀 xsshend

[![Crates.io](https://img.shields.io/crates/v/xsshend.svg)](https://crates.io/crates/xsshend)
[![Documentation](https://docs.rs/xsshend/badge.svg)](https://docs.rs/xsshend)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021%2B-orange.svg)](https://www.rust-lang.org)
[![Security: RUSTSEC-2023-0071](https://img.shields.io/badge/security-known%20limitation-yellow)](SECURITY.md)

**xsshend** est un outil Rust moderne et efficace pour le **téléversement de fichiers vers multiples serveurs SSH**. Interface en ligne de commande simple et intuitive avec suivi en temps réel des transferts.

## 🔧 Installation

### Pré-requis

- Rust 1.70+ (recommandé: version stable la plus récente)
- OpenSSL (optionnel, pour accélérer la compilation)

### Via Cargo (recommandé)

```bash
cargo install xsshend
```

#### 🚀 Accélérer l'installation

L'installation peut être longue car elle compile OpenSSL depuis les sources par défaut. Pour accélérer :

**Option 1 : Utiliser OpenSSL du système (Linux/macOS)**
```bash
# Installer OpenSSL si nécessaire
# Ubuntu/Debian:
sudo apt-get install libssl-dev pkg-config

# Fedora/RHEL:
sudo dnf install openssl-devel

# macOS:
brew install openssl pkg-config

# Installer xsshend (utilise OpenSSL système automatiquement)
cargo install xsshend
```

**Option 2 : Compilation parallèle**
```bash
# Utiliser tous les cœurs CPU disponibles
cargo install xsshend -j $(nproc)
```

### Depuis les sources

```bash
git clone https://github.com/willisback/xsshend.git
cd xsshend
cargo install --path .
```

## 📚 Documentation

- [**Documentation complète**](https://willisback.github.io/xsshend/)
- [Guide d'utilisation](#-utilisation)
- [Configuration](#%EF%B8%8F-configuration)
- [**Politique de sécurité**](SECURITY.md)

## 🔒 Note de Sécurité

⚠️ **Vulnérabilité connue**: xsshend dépend de `russh` qui utilise `rsa 0.9.8`, affecté par [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071) (Marvin Attack - timing sidechannel).

**Recommandations** :
- ✅ Utilisez des **clés Ed25519** plutôt que RSA (non affectées)
- ✅ Utilisez xsshend sur des **réseaux de confiance** (internes, VPN)
- ⚠️ Évitez l'utilisation sur des **réseaux publics non sécurisés**

**Contexte** : Aucun correctif n'est disponible actuellement. Cette vulnérabilité permet potentiellement la récupération de clés RSA via l'observation du timing réseau. Les clés Ed25519 et ECDSA ne sont pas affectées.

**Détails complets** : Consultez [SECURITY.md](SECURITY.md) pour plus d'informations et les recommandations détaillées.

## ✨ Fonctionnalités principales

- 🎯 **Téléversement simple** vers plusieurs serveurs SSH
- 📊 **Barres de progression** en temps réel pour chaque transfert
- 🔐 **Authentification sécurisée** par clés SSH avec découverte automatique
- 🔍 **Filtrage avancé** par environnement, région et type de serveur
- 🏗️ **Configuration hiérarchique** des serveurs (environnements, régions, types)
- 🛡️ **Gestion d'erreurs robuste** avec rapports détaillés
- 📁 **Support multi-fichiers** avec validation automatique
- ⚡ **Mode dry-run** pour simulation sans transfert réel
- 🧪 **Suite de tests complète** avec 93 tests automatisés

## 🚀 Démarrage rapide

### 1. Initialisation

```bash
# Initialiser la configuration avec assistant interactif
xsshend init

# Forcer la réinitialisation
xsshend init --force
```

### 2. Configuration des serveurs

Créez le fichier `~/.ssh/hosts.json` avec la structure hiérarchique :

```json
{
  "Production": {
    "Region-A": {
      "Public": {
        "WEB_SERVER_01": {
          "alias": "web01@prod-web-01.example.com",
          "env": "PROD"
        },
        "API_SERVER_01": {
          "alias": "api01@prod-api-01.example.com",
          "env": "PROD"
        }
      },
      "Private": {
        "DATABASE_01": {
          "alias": "db01@prod-db-01.example.com",
          "env": "PROD"
        }
      }
    }
  },
  "Staging": {
    "Region-A": {
      "Public": {
        "STAGE_WEB_01": {
          "alias": "web01@stage-web-01.example.com",
          "env": "STAGE"
        }
      }
    }
  },
  "Development": {
    "Local": {
      "Services": {
        "DEV_DATABASE": {
          "alias": "dev@dev-db.local.example.com",
          "env": "DEV"
        }
      }
    }
  }
}
```

### 3. Utilisation

```bash
# Lister les serveurs disponibles
xsshend list

# Téléverser un fichier vers tous les serveurs
xsshend upload myfile.txt

# Filtrer par environnement
xsshend upload config.json --env Production

# Mode dry-run (simulation)
xsshend upload deploy.sh --env Production --dry-run
```

## 🎮 Utilisation

### Commandes principales

```bash
# Aide générale
xsshend --help

# Initialiser la configuration
xsshend init [--force]

# Lister les serveurs disponibles
xsshend list
# ou
xsshend --list

# Téléverser des fichiers
xsshend upload <FILES>... [OPTIONS]
```

### Options de téléversement

```bash
# Filtrage par environnement
xsshend upload file.txt --env Production
xsshend upload file.txt --env Staging
xsshend upload file.txt --env Development

# Filtrage par région
xsshend upload file.txt --region Region-A
xsshend upload file.txt --region Region-B

# Filtrage par type de serveur
xsshend upload file.txt --type Public
xsshend upload file.txt --type Private

# Filtrage combiné
xsshend upload config.json --env Production --region Region-A
xsshend upload deploy.sh --env Production --type Public
xsshend upload app.war --env Staging --region Region-A --type Public

# Spécifier la destination
xsshend upload file.txt --dest /opt/app/
xsshend upload file.txt --dest /var/www/html/

# Mode simulation (dry-run)
xsshend upload file.txt --env Production --dry-run

# Téléverser plusieurs fichiers
xsshend upload file1.txt file2.json directory/
```

### Exemples d'utilisation

```bash
# Déploiement en production
xsshend upload app.war --env Production --dest /opt/tomcat/webapps/

# Mise à jour de configuration de staging
xsshend upload config.json --env Staging --dest /etc/myapp/

# Déploiement sur serveurs publics uniquement
xsshend upload static-files/ --env Production --type Public --dest /var/www/

# Simulation d'un déploiement
xsshend upload deploy.sh --env Production --dry-run

# Upload vers une région spécifique
xsshend upload regional-config.json --env Production --region Region-A
```

### Interface de progression

Les transferts affichent une progression en temps réel :

```
🚀 Début du téléversement: 1 fichier(s) vers 3 serveur(s)
📂 Destination: /opt/uploads/
🎯 Serveurs ciblés:
   • Production:Region-A:Public:WEB_SERVER_01 → web01@prod-web-01.example.com (PROD)
   • Production:Region-A:Public:API_SERVER_01 → api01@prod-api-01.example.com (PROD)
   • Production:Region-A:Private:DATABASE_01 → db01@prod-db-01.example.com (PROD)

📤 Téléversement de ./myapp.jar en cours...

web01@prod-web-01... [████████████████████████████████] 2.3MB/2.3MB ✅
api01@prod-api-01... [██████████████████              ] 1.5MB/2.3MB ⏳
db01@prod-db-01..... [████████████████████████████     ] 2.1MB/2.3MB ⏳

✅ Téléversement terminé avec succès!
📊 3 serveur(s) - 0 échec(s)
```

## 🔑 Gestion des clés SSH

### Découverte automatique

xsshend détecte automatiquement les clés SSH disponibles dans `~/.ssh/` :

- **Types supportés** : Ed25519, RSA, ECDSA, DSA
- **Gestion des passphrases** avec ssh-agent
- **Sélection automatique** de la meilleure clé disponible

### Priorité de sélection

1. **Ed25519** - Recommandé pour la sécurité et performance
2. **RSA** - Compatibilité étendue
3. **ECDSA** - Alternative moderne
4. **DSA** - Support basique

### Configuration SSH

```bash
# Vérifier les clés disponibles
ssh-add -l

# Ajouter une clé à l'agent SSH
ssh-add ~/.ssh/id_ed25519

# Copier une clé publique vers un serveur
ssh-copy-id user@server.example.com
```

## ⚙️ Configuration

### Structure hiérarchique

La configuration suit une structure à trois niveaux :

```
Environment/
├── Region/
│   ├── Type/
│   │   ├── SERVER_NAME_1
│   │   └── SERVER_NAME_2
│   └── Type/
└── Region/
```

**Exemple d'organisation :**

```
Production/
├── Region-A/
│   ├── Public/     # Serveurs web publics
│   └── Private/    # Bases de données internes
└── Region-B/
    ├── Public/
    └── Private/

Staging/
├── Region-A/
│   └── Public/
└── Region-B/

Development/
└── Local/
    └── Services/
```

### Commande init

La commande `xsshend init` vous guide dans la configuration :

1. **Détection des clés SSH** existantes
2. **Création du fichier hosts.json** avec template
3. **Configuration des permissions** sécurisées (.ssh/ en 700)
4. **Conseils d'utilisation** personnalisés

```bash
# Configuration initiale interactive
xsshend init

# Réinitialisation complète
xsshend init --force
```

### Format des entrées serveur

Chaque serveur est défini avec :

```json
{
  "SERVER_NAME": {
    "alias": "username@hostname.example.com",  // Obligatoire
    "env": "ENVIRONMENT_TAG"                   // Optionnel
  }
}
```

## 🛠️ Développement

### Prérequis

- **Rust 2021 Edition** ou plus récent
- **OpenSSH** pour les clés SSH
- **Git** pour le développement

### Installation développement

```bash
# Cloner le projet
git clone https://github.com/willisback/xsshend.git
cd xsshend

# Installer les outils de développement
rustup component add clippy rustfmt

# Compiler en mode debug
cargo build

# Compiler en mode release
cargo build --release
```

### Tests

Le projet inclut une suite de tests complète avec **93 tests** :

```bash
# Tests unitaires et d'intégration
cargo test

# Tests avec sortie détaillée
cargo test -- --nocapture

# Tests de performance (benchmarks)
cargo bench
```

### Qualité de code

```bash
# Formatage automatique
cargo fmt

# Linting avec Clippy
cargo clippy -- -D warnings

# Vérification complète
cargo check --all-targets --all-features

# Documentation
cargo doc --open
```

## 🧪 Architecture de tests

### Couverture de tests

- **Unit tests** : 6 tests (modules core)
- **Config tests** : 14 tests (configuration JSON)
- **SSH keys tests** : 10 tests (gestion des clés)
- **Uploader tests** : 14 tests (téléversements)
- **CLI tests** : 21 tests (interface ligne de commande)
- **Integration tests** : 12 tests (workflows complets)
- **Validator tests** : 16 tests (validation fichiers)

### Tests d'intégration

Les tests d'intégration valident :

- ✅ Initialisation et configuration
- ✅ Détection et sélection des clés SSH
- ✅ Parsing et filtrage des configurations
- ✅ Workflows complets (init → list → upload)
- ✅ Gestion des erreurs et cas limites
- ✅ Performance avec grandes configurations
- ✅ Interface CLI et validation des arguments

## 🔧 Stack technologique

### Dépendances principales

- **`ssh2`** - Client SSH/SFTP robuste
- **`clap`** - Interface ligne de commande moderne
- **`serde/serde_json`** - Sérialisation JSON
- **`indicatif`** - Barres de progression
- **`anyhow`** - Gestion d'erreurs ergonomique
- **`tempfile`** - Fichiers temporaires (tests)

### Architecture modulaire

```
src/
├── main.rs           # Point d'entrée CLI
├── lib.rs            # Interface bibliothèque
├── config/           # Configuration hosts.json
│   ├── mod.rs
│   └── hosts.rs
├── ssh/              # Client SSH et clés
│   ├── mod.rs
│   ├── client.rs
│   └── keys.rs
├── core/             # Logique métier
│   ├── mod.rs
│   ├── uploader.rs
│   └── validator.rs
└── utils/            # Utilitaires
    ├── mod.rs
    └── logger.rs

tests/               # Tests d'intégration
├── cli_test.rs
├── config_test.rs
├── integration_test.rs
├── ssh_keys_test.rs
├── uploader_test.rs
├── validator_test.rs
└── common/
    └── mod.rs       # Utilitaires de test

benches/             # Tests de performance
└── performance_bench.rs
```

## 🛡️ Dépannage

### Problèmes courants

#### Erreur: "Permission denied (publickey)"

```bash
# Vérifier la configuration SSH
ssh -v user@server.example.com

# Vérifier l'agent SSH
ssh-add -l

# Ajouter une clé si nécessaire
ssh-add ~/.ssh/id_ed25519
```

#### Erreur: "hosts.json not found"

```bash
# Utiliser la commande d'initialisation
xsshend init

# Ou créer manuellement
mkdir -p ~/.ssh
nano ~/.ssh/hosts.json
```

#### Tests en échec

```bash
# Exécuter les tests avec détails
cargo test -- --nocapture

# Vérifier un test specific
cargo test test_name

# Tests d'intégration uniquement
cargo test --test integration_test
```

### Mode debug

```bash
# Logs détaillés
RUST_LOG=debug xsshend upload file.txt --env Production

# Mode très verbeux
RUST_LOG=trace xsshend upload file.txt --env Production
```

## 🤝 Contribution

Les contributions sont les bienvenues !

### Processus de contribution

1. **Fork** le projet
2. Créer une **branche feature** (`git checkout -b feature/ma-fonctionnalite`)
3. **Commiter** les changements (`git commit -am 'Ajoute ma fonctionnalité'`)
4. **Pousser** vers la branche (`git push origin feature/ma-fonctionnalite`)
5. Ouvrir une **Pull Request**

### Standards de qualité

Avant de soumettre une PR, assurez-vous que :

- ✅ Code formaté : `cargo fmt`
- ✅ Linting OK : `cargo clippy -- -D warnings`
- ✅ Tests passants : `cargo test`
- ✅ Documentation à jour

Le workflow GitHub Actions vérifie automatiquement ces critères.

## 📄 Licence

Ce projet est sous licence **MIT** - voir le fichier [LICENSE](LICENSE) pour les détails.

## 🧪 Environnement de Test (Lab)

Un environnement de test Docker complet est disponible pour valider xsshend en conditions réelles :

**Documentation du Lab :**
- **[Index du Lab](docs/LAB-INDEX.md)** - Point d'entrée principal
- **[Guide de Démarrage](docs/LAB-README.md)** - Installation et configuration
- **[Guide de Test](docs/LAB-TESTING-GUIDE.md)** - Tests détaillés (40+ tests)
- **[Dépannage](docs/LAB-TROUBLESHOOTING.md)** - Résolution de problèmes

**Démarrage Rapide :**
```bash
# 1. Setup
git clone https://github.com/WillIsback/xsshend.git
cd xsshend
./scripts/lab-setup.sh

# 2. Démarrer le lab (3 conteneurs : master + 2 targets)
cd lab/
docker-compose up -d --build

# 3. Tests automatisés
../scripts/test-lab.sh

# 4. Tests manuels
docker exec -it xsshend_master bash
xsshend list
xsshend upload test.txt --env Test
```

Le lab fournit :
- ✅ Environnement ArchLinux isolé (Docker)
- ✅ 3 conteneurs configurés (master + 2 targets SSH)
- ✅ Clés SSH pré-configurées (RSA + Ed25519)
- ✅ Suite de tests automatisés (40+ tests)
- ✅ Scripts de diagnostic et dépannage
- ✅ Documentation complète

Voir **[docs/LAB-INDEX.md](docs/LAB-INDEX.md)** pour tous les détails.

## 🙏 Remerciements

- **[clap-rs](https://github.com/clap-rs/clap)** pour l'excellent framework CLI
- **[russh](https://github.com/Eugeny/russh)** pour l'implémentation SSH pure Rust
- **[indicatif](https://github.com/console-rs/indicatif)** pour les barres de progression
- **[serde](https://github.com/serde-rs/serde)** pour la sérialisation JSON
- Communauté **Rust** pour l'écosystème exceptionnel

---

**xsshend** - *Téléversement SSH simple et efficace* 🚀