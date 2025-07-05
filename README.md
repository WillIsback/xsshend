# 🚀 xsshend

**xsshend** est un outil Rust moderne et efficace pour le **téléversement parallèle de fichiers vers multiples serveurs SSH**. Inspiré de projets comme `jless`, `xsv`, et `csvlens`, il offre une interface TUI (Terminal User Interface) élégante avec suivi en temps réel des transferts.

## ✨ Fonctionnalités

- 🔄 **Téléversement parallèle** vers plusieurs serveurs SSH simultanément
- 🎯 **Interface TUI moderne** avec barres de progression en temps réel
- 🔐 **Authentification sécurisée** par clés SSH avec support agent SSH
- 📊 **Configuration hiérarchique** des serveurs (environnements, régions, types)
- ⚡ **Performance optimisée** avec threading natif Rust
- 🛡️ **Gestion d'erreurs robuste** avec rapports détaillés
- 📁 **Support multi-fichiers** avec validation et confirmation

## 🏗️ Architecture

```
Production/
├── Region-A/
│   ├── Public/     # Serveurs publics
│   └── Private/    # Serveurs internes
└── Region-B/
    ├── Public/
    └── Private/

Staging/
├── Region-A/
└── Region-B/

Development/
└── Local/
```

## 🚀 Installation

### Prérequis

- **Rust 2024** ou plus récent
- **OpenSSH** configuré avec clés SSH
- **Fichier de configuration** `~/.ssh/hosts.json`

### Compilation

```bash
# Cloner le projet
git clone https://github.com/username/xsshend.git
cd xsshend

# Compiler en mode release
cargo build --release

# Installer globalement
cargo install --path .
```

## ⚙️ Configuration

### Fichier hosts.json

Créez le fichier `~/.ssh/hosts.json` avec la structure suivante :

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
        "INTERNAL_WEB_01": {
          "alias": "iweb01@prod-internal-01.example.com",
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
  }
}
```

### Clés SSH

Assurez-vous que vos clés SSH publiques sont déployées sur tous les serveurs cibles :

```bash
# Copier votre clé publique vers un serveur
ssh-copy-id user@server.example.com

# Ou ajouter manuellement dans ~/.ssh/authorized_keys sur le serveur distant
```

## 🎮 Utilisation

### Interface en Ligne de Commande

```bash
# Téléverser un fichier vers tous les serveurs de production
xsshend upload ./myfile.tar.gz --env Production

# Téléverser vers une région spécifique
xsshend upload ./app.jar --env Production --region Region-A

# Téléverser vers des serveurs publics uniquement
xsshend upload ./config.json --env Staging --type Public

# Téléverser plusieurs fichiers
xsshend upload ./file1.txt ./file2.json --env Development

# Mode interactif avec sélection de serveurs
xsshend upload ./deploy.sh --interactive

# Spécifier le répertoire de destination
xsshend upload ./app.war --env Production --dest /opt/apps/

# Mode verbeux avec logs détaillés
xsshend upload ./script.sh --env Staging --verbose
```

### Interface TUI

L'interface TUI se lance automatiquement et affiche :

```
┌─ xsshend - Téléversement Multi-SSH ─────────────────────────┐
│                                                             │
│ Fichier: ./myapp.jar (2.3 MB)                              │
│ Destinations: 8 serveurs sélectionnés                      │
│                                                             │
│ ┌─ Progression par serveur ─────────────────────────────┐   │
│ │ [████████████████████████████████] web01@prod... 100%  │   │
│ │ [██████████████████              ] api01@prod...  65%  │   │
│ │ [████████████████████████████     ] db01@stage...  85%  │   │
│ │ [                                 ] cache01@dev...  0%  │   │
│ └─────────────────────────────────────────────────────────┘   │
│                                                             │
│ Status: 3/8 terminés - 1 erreur                            │
│                                                             │
│ [Q] Quitter  [P] Pause  [R] Reprendre  [L] Logs            │
└─────────────────────────────────────────────────────────────┘
```

### Options Avancées

```bash
# Exclure certains serveurs
xsshend upload ./file.txt --env Production --exclude WEB_SERVER_01,API_SERVER_02

# Timeout personnalisé
xsshend upload ./largefile.bin --env Production --timeout 300

# Nombre max de connexions parallèles
xsshend upload ./file.txt --env Production --max-parallel 5

# Mode dry-run (simulation)
xsshend upload ./file.txt --env Production --dry-run

# Forcer l'écrasement de fichiers existants
xsshend upload ./file.txt --env Production --force

# Utiliser SCP au lieu de SFTP
xsshend upload ./file.txt --env Production --protocol scp
```

## 🔧 Stack Technologique

### Crates Principales

- **`ssh2`** - Connectivité SSH/SFTP robuste
- **`indicatif`** - Barres de progression multi-threads
- **`dialoguer`** - Prompts interactifs élégants
- **`clap`** - Parsing d'arguments CLI moderne
- **`rayon`** - Parallélisation efficace
- **`crossterm`** - Contrôle terminal cross-platform
- **`serde`** - Sérialisation JSON
- **`rpassword`** - Saisie sécurisée de passphrase
- **`anyhow`** - Gestion d'erreurs ergonomique

### Architecture du Code

```
src/
├── main.rs              # Point d'entrée et CLI
├── config/
│   ├── mod.rs
│   ├── hosts.rs         # Parsing hosts.json
│   └── ssh.rs           # Configuration SSH
├── ssh/
│   ├── mod.rs
│   ├── client.rs        # Client SSH/SFTP
│   ├── auth.rs          # Authentification
│   └── transfer.rs      # Logique de transfert
├── ui/
│   ├── mod.rs
│   ├── tui.rs           # Interface TUI
│   ├── progress.rs      # Barres de progression
│   └── prompts.rs       # Dialogues interactifs
├── core/
│   ├── mod.rs
│   ├── uploader.rs      # Orchestrateur principal
│   ├── parallel.rs      # Gestion parallélisme
│   └── validator.rs     # Validation fichiers/serveurs
└── utils/
    ├── mod.rs
    ├── errors.rs        # Types d'erreurs
    └── logger.rs        # Système de logs
```

## 🛠️ Développement

### Prérequis Développement

```bash
# Installer les outils de développement
rustup component add clippy rustfmt

# Installer cargo-watch pour le développement
cargo install cargo-watch
```

### Commandes Utiles

```bash
# Développement avec rechargement automatique
cargo watch -x "run -- upload examples/test.txt --env Development"

# Tests
cargo test
cargo test -- --nocapture  # Avec output des prints

# Linting
cargo clippy -- -D warnings

# Formatage
cargo fmt

# Vérification complète
cargo check --all-targets --all-features

# Benchmark de performance
cargo bench

# Documentation
cargo doc --open
```

### Structure des Tests

```bash
tests/
├── integration/
│   ├── mod.rs
│   ├── ssh_tests.rs     # Tests SSH réels
│   ├── config_tests.rs  # Tests de configuration
│   └── upload_tests.rs  # Tests de téléversement
├── fixtures/
│   ├── test_hosts.json  # Fichier hosts de test
│   └── test_files/      # Fichiers de test
└── mock/
    ├── ssh_mock.rs      # Mock serveur SSH
    └── helpers.rs       # Utilitaires de test
```

## 🚦 Exemples d'Usage

### Scénario 1: Déploiement Application Web

```bash
# Déployer sur tous les serveurs web de production
xsshend upload ./webapp.war --env Production --filter "WEB_SERVER_*" --dest /opt/tomcat/webapps/

# Vérifier le déploiement
xsshend exec "ls -la /opt/tomcat/webapps/" --env Production --filter "WEB_SERVER_*"
```

### Scénario 2: Mise à jour Configuration

```bash
# Déployer configuration sur tous les environnements
xsshend upload ./config.json --env Production,Staging --dest /etc/myapp/

# Redémarrer les services après déploiement
xsshend exec "systemctl restart myapp" --env Production,Staging
```

### Scénario 3: Backup et Synchronisation

```bash
# Synchroniser scripts de backup
xsshend upload ./backup-scripts/ --env Production --type Private --dest /opt/backup/

# Téléverser avec vérification d'intégrité
xsshend upload ./important-data.tar.gz --env Production --verify-checksum
```

## 🐛 Dépannage

### Problèmes Courants

**Erreur: "Permission denied (publickey)"**
```bash
# Vérifier la configuration SSH
ssh -v user@server.example.com

# Vérifier l'agent SSH
ssh-add -l

# Ajouter la clé si nécessaire
ssh-add ~/.ssh/id_rsa
```

**Erreur: "hosts.json not found"**
```bash
# Créer le fichier de configuration
mkdir -p ~/.ssh
cp examples/hosts.json ~/.ssh/hosts.json
# Éditer avec vos serveurs
```

**Performances lentes**
```bash
# Réduire le parallélisme
xsshend upload file.txt --env Production --max-parallel 3

# Utiliser SCP au lieu de SFTP
xsshend upload file.txt --env Production --protocol scp
```

### Logs et Debug

```bash
# Mode verbeux
RUST_LOG=debug xsshend upload file.txt --env Production --verbose

# Logs dans un fichier
xsshend upload file.txt --env Production --log-file /tmp/xsshend.log

# Mode trace pour debugging SSH
RUST_LOG=ssh2=trace xsshend upload file.txt --env Production
```

## 🤝 Contribution

Les contributions sont les bienvenues ! Veuillez suivre ces étapes :

1. **Fork** le projet
2. Créer une **branche feature** (`git checkout -b feature/ma-fonctionnalite`)
3. **Commiter** vos changements (`git commit -am 'Ajoute ma fonctionnalité'`)
4. **Pousser** vers la branche (`git push origin feature/ma-fonctionnalite`)
5. Ouvrir une **Pull Request**

### Standards de Code

- Code formaté avec `cargo fmt`
- Linting sans warnings avec `cargo clippy`
- Tests passants avec `cargo test`
- Documentation mise à jour

## 📄 Licence

Ce projet est sous licence **MIT** - voir le fichier [LICENSE](LICENSE) pour plus de détails.

## 🙏 Remerciements

- **[clap-rs](https://github.com/clap-rs/clap)** pour l'excellent framework CLI
- **[ssh2-rs](https://github.com/alexcrichton/ssh2-rs)** pour les bindings SSH robustes
- **[indicatif](https://github.com/console-rs/indicatif)** pour les barres de progression élégantes
- Communauté **Rust** pour l'écosystème exceptionnel

---

**xsshend** - *Téléversement SSH parallèle, simple et efficace* 🚀
