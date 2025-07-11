# 🚀 xsshend

[![Crates.io](https://img.shields.io/crates/v/xsshend.svg)](https://crates.io/crates/xsshend)
[![Documentation](https://docs.rs/xsshend/badge.svg)](https://docs.rs/xsshend)
[![Release](https://github.com/williamdes/xsshend/workflows/Release/badge.svg)](https://github.com/williamdes/xsshend/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024%2B-orange.svg)](https://www.rust-lang.org)

**xsshend** est un outil Rust moderne et efficace pour le **téléversement parallèle de fichiers vers multiples serveurs SSH**. Il offre une interface TUI (Terminal User Interface) hiérarchique intuitive avec suivi en temps réel des transferts.

## 🔧 Installation

### Via Cargo (recommandé)

```bash
cargo install xsshend
```

### Depuis les sources

```bash
git clone https://github.com/WillIsback/xsshend.git
cd xsshend
cargo install --path .
```

## 📚 Documentation

- [**Documentation complète**](https://willisback.github.io/xsshend/)
- [Guide d'utilisation](docs/usage.md)
- [Sélection des clés SSH](docs/ssh-key-selection.md)
- [Configuration automatique](docs/auto-configuration.md)
- [Gestion des clés SSH](docs/ssh-key-management.md)
- [Optimisation](docs/optimization.md)

## ✨ Fonctionnalités principales

- 🌳 **Interface hiérarchique moderne** pour la sélection de serveurs
- 🔄 **Téléversement parallèle** vers plusieurs serveurs SSH simultanément  
- 🎯 **Barres de progression en temps réel** pour chaque serveur
- 🔍 **Recherche intégrée** pour filtrer rapidement les serveurs
- 🔐 **Authentification sécurisée** par clés SSH avec support agent SSH
- 🔑 **Sélection interactive de clés SSH** avec découverte automatique
- 🎛️ **Sélection CLI de clés SSH** avec options `--ssh-key` et `--ssh-key-interactive`
- 📊 **Configuration hiérarchique** des serveurs (environnements, régions, types)
- ⚡ **Performance optimisée** avec threading natif Rust
- 🛡️ **Gestion d'erreurs robuste** avec rapports détaillés
- 📁 **Support multi-fichiers** avec sélection interactive
- 🎮 **Modes d'utilisation flexibles** : interface complète, interactif, ligne de commande
- 🎨 **Thème adaptatif** : Détection automatique des thèmes clair/sombre du terminal
- 🌈 **Interface optimisée** : Couleurs et contrastes adaptés pour une meilleure lisibilité

## 🎮 Interface utilisateur

### Interface hiérarchique de sélection

L'interface organise vos serveurs en arbre navigable :

```
📂 Production
├── 🌐 Region-A  
│   ├── 📊 Public
│   │   ├── ✅ WEB_SERVER_01 (web01@prod-web-01.example.com)
│   │   └── �️ API_SERVER_01 (api01@prod-api-01.example.com)
│   └── 📋 Private
│       └── 🖥️ DATABASE_01 (db01@prod-db-01.example.com)
└── 🌐 Region-B
    └── 📊 Public
        └── 🖥️ CACHE_SERVER_01 (cache01@prod-cache-01.example.com)
```

### Navigation intuitive

- **↑↓** : Navigation dans l'arbre
- **→ ←** : Déplier/réduire les nœuds  
- **Espace** : Sélectionner des serveurs
- **/** : Recherche en temps réel
- **a** : Sélectionner tout / **c** : Vider la sélection

## 🏗️ Architecture de configuration

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

## 🔑 Gestion des clés SSH

### Détection automatique

xsshend détecte automatiquement les clés SSH disponibles dans `~/.ssh/` :

- Clés supportées : `id_ed25519`, `id_rsa`, `id_ecdsa`, `id_dsa`
- Gestion des clés avec passphrase
- Support complet de ssh-agent

### Sélection de clés en ligne de commande

```bash
# Sélection interactive - affiche un menu pour choisir parmi les clés disponibles
xsshend upload file.txt --ssh-key-interactive

# Spécification directe par nom de fichier (sans extension)
xsshend upload file.txt --ssh-key id_ed25519
xsshend upload file.txt --ssh-key company_key

# Sélection automatique forcée de la meilleure clé (Ed25519 > RSA > ECDSA)
xsshend upload file.txt --ssh-key-auto

# Comportement par défaut : sélection intelligente
xsshend upload file.txt
# Affiche les clés détectées et sélectionne automatiquement la meilleure
# Suggère l'utilisation de --ssh-key-interactive pour un choix manuel
```

### Priorité de sélection automatique

1. **Ed25519** - Recommandé pour la sécurité et les performances
2. **RSA** - Compatibilité étendue
3. **ECDSA** - Alternative moderne
4. **Autres** - Support basique

### Intégration ssh-agent

Si aucune clé n'est sélectionnée ou disponible, xsshend utilise automatiquement ssh-agent pour l'authentification.

## 🎨 Thèmes et accessibilité

### Détection automatique du thème

xsshend s'adapte automatiquement au thème de votre terminal :

- **Thème sombre** : Couleurs optimisées pour les fonds sombres
- **Thème clair** : Contraste amélioré pour les fonds clairs
- **Détection intelligente** : Utilise les APIs du terminal pour la détection

### Lisibilité optimisée

- Contraste automatique pour tous les éléments
- Titres de panneaux bien visibles
- Éléments non sélectionnés lisibles
- Panneau d'aide avec bon contraste

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

### 1. Interface complète (recommandé)

Lancez l'application pour accéder à l'interface hiérarchique complète :

```bash
# Interface TUI complète avec workflow guidé
xsshend

# Ou explicitément en mode interactif  
xsshend --interactive
```

**Workflow guidé :**
1. **Sélection des fichiers** - Navigateur de fichiers intuitif
2. **Sélection des serveurs** - Interface hiérarchique avec recherche
3. **Destination** - Saisie du répertoire cible
4. **Téléversement** - Progression en temps réel

### 2. Mode interactif avec fichiers pré-sélectionnés

```bash
# Avec fichiers spécifiés, interface pour serveurs et destination
xsshend --interactive file1.txt file2.txt directory/

# Ou via sous-commande
xsshend upload file1.txt file2.txt --interactive
```

### 3. Mode ligne de commande

```bash
# Téléversement direct avec filtres
xsshend upload file.txt --region Production --dest /opt/app/

# Filtrer par région
xsshend upload *.log --region Region-A --dest /var/log/

# Filtrage par environnement (nouveau!)
xsshend upload ./config.json --env Production
xsshend upload ./staging-config.json --env Staging

# Filtrage combiné environnement + région
xsshend upload ./regional-config.json --env Production --region Region-A

# Filtrage combiné environnement + type
xsshend upload ./app.war --env Production --type Public

# Gestion des clés SSH - nouvelles options!
# Sélection interactive de la clé SSH
xsshend upload file.txt --ssh-key-interactive

# Spécifier une clé SSH par nom (sans extension)
xsshend upload file.txt --ssh-key id_rsa
xsshend upload file.txt --ssh-key company_key

# Forcer la sélection automatique de la meilleure clé
xsshend upload file.txt --ssh-key-auto

# Par défaut : sélection intelligente avec suggestion
xsshend upload file.txt  # Sélectionne automatiquement la meilleure clé disponible
```

### 4. Lister les serveurs avec étiquettes hiérarchiques

```bash
# Lister les serveurs disponibles avec étiquettes CLI
xsshend list
# ou
xsshend -l
```

**Exemple de sortie enrichie :**
```
🔍 Liste des cibles SSH disponibles:

📁 Production (--env Production)
  📂 Region-A (--region Region-A)
    📂 Public (--type Public)
      🖥️  WEB_SERVER_01 → web01@prod-web-01.example.com (PROD)
      🖥️  API_SERVER_01 → api01@prod-api-01.example.com (PROD)
    📂 Private (--type Private)
      �️  DATABASE_01 → db01@prod-db-01.example.com (PROD)

📁 Staging (--env Staging)
  📂 Region-A (--region Region-A)
    📂 Public (--type Public)
      🖥️  STAGE_WEB_01 → web01@stage-web-01.example.com (STAGE)

📊 Total: 4 cibles disponibles

�💡 Exemples d'utilisation:
   xsshend upload --env Production file.txt
   xsshend upload --env Staging --region Region-A file.txt
   xsshend upload --region Region-A --type Public file.txt
```

### 5. Gestion robuste des serveurs déconnectés

xsshend gère maintenant gracieusement les serveurs inaccessibles :

```bash
# Vérification de connectivité avant l'interface TUI
xsshend --online-only

# Les timeouts de connexion sont configurés pour éviter les blocages
# Les erreurs de connexion sont logguées mais n'interrompent pas les autres transferts
   xsshend upload --region Region-A --type Public file.txt
```

### 5. Modes de filtrage avancés

```bash
# Filtrage par environnement complet
xsshend upload file.txt --env Production --dest /opt/app/

# Filtrage par environnement et région
xsshend upload file.txt --env Staging --region Region-A --dest /var/log/

# Filtrage par environnement et type de serveurs
xsshend upload config.json --env Production --type Public --dest /etc/app/

# Filtrage traditionnel par région ou type uniquement
xsshend upload *.log --region Region-A --dest /var/log/
xsshend upload config.json --type Public --dest /etc/app/

# Vérification de connectivité avant transfert
xsshend --online-only
```

**Workflow interactif en 4 étapes :**

1. **📁 Sélection de fichiers** - Naviguez et sélectionnez vos fichiers
2. **🖥️ Sélection de serveurs** - Choisissez vos serveurs cibles  
3. **📂 Saisie de destination** - Spécifiez le répertoire de destination ⭐
4. **⚡ Transferts parallèles** - Surveillez les transferts en temps réel

> ⭐ **Important** : L'étape de saisie du répertoire de destination est présente et fonctionnelle dans le TUI. 
> Utilisez Tab/Entrée pour naviguer entre les étapes.

### Interface en Ligne de Commande

```bash
# Téléverser un fichier vers tous les serveurs disponibles
xsshend upload ./myfile.tar.gz

# Téléverser vers un environnement spécifique
xsshend upload ./app.jar --env Production

# Téléverser vers une région spécifique
xsshend upload ./app.jar --region Region-A

# Téléverser vers des serveurs publics uniquement
xsshend upload ./config.json --type Public

# Téléverser vers un environnement ET une région
xsshend upload ./config.json --env Staging --region Region-A

# Téléverser vers un environnement ET un type
xsshend upload ./app.war --env Production --type Public

# Téléverser plusieurs fichiers
xsshend upload ./file1.txt ./file2.json

# Mode interactif avec sélection de serveurs
xsshend upload ./deploy.sh --interactive

# Spécifier le répertoire de destination
xsshend upload ./app.war --dest /opt/apps/

# Mode verbeux avec logs détaillés
xsshend upload ./script.sh --verbose

# Vérifier la connectivité avant le TUI (n'affiche que les serveurs en ligne)
xsshend --online-only
```

### Interface de Progression

L'interface de progression se lance automatiquement et affiche des barres de progression en temps réel :

```
🚀 Début du téléversement:
   📁 1 fichier(s)
   🖥️  3 serveur(s)
   📂 Destination: /opt/uploads/

📤 Téléversement de ./myapp.jar vers /opt/uploads/myapp.jar...
   Taille: 2.3 MB

web01@prod-web-01... [████████████████████████████████] 2.3MB/2.3MB (00:02)
api01@prod-api-01... [██████████████████              ] 1.5MB/2.3MB (00:01)
db01@stage-db-01.... [████████████████████████████     ] 2.1MB/2.3MB (00:00)

📊 Résumé du téléversement:
  ✅ WEB_SERVER_01 - 2,359,296 octets
  ✅ API_SERVER_01 - 2,359,296 octets  
  ✅ DATABASE_01 - 2,359,296 octets

✅ Téléversement terminé avec succès!
```

### Options Avancées

```bash
# Exclure certains serveurs
xsshend upload ./file.txt --exclude WEB_SERVER_01,API_SERVER_02

# Timeout personnalisé
xsshend upload ./largefile.bin --timeout 300

# Nombre max de connexions parallèles
xsshend upload ./file.txt --max-parallel 5

# Mode dry-run (simulation)
xsshend upload ./file.txt --dry-run

# Forcer l'écrasement de fichiers existants
xsshend upload ./file.txt --force

# Utiliser SCP au lieu de SFTP
xsshend upload ./file.txt --protocol scp
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

> 📋 **Note :** Une interface TUI complète avec contrôles interactifs (pause, reprise, logs) est prévue pour la version 0.2.0

### Architecture du Code

```
src/
├── main.rs              # Point d'entrée et CLI
├── config/
│   ├── mod.rs
│   └── hosts.rs         # Parsing hosts.json
├── ssh/
│   ├── mod.rs
│   ├── client.rs        # Client SSH/SFTP
│   ├── auth.rs          # Authentification (placeholder)
│   └── transfer.rs      # Transfert avec barres de progression
├── ui/
│   ├── mod.rs
│   └── prompts.rs       # Dialogues interactifs
├── core/
│   ├── mod.rs
│   ├── uploader.rs      # Orchestrateur principal
│   ├── parallel.rs      # Gestion parallélisme (placeholder)
│   └── validator.rs     # Validation fichiers/serveurs
└── utils/
    ├── mod.rs
    ├── env_expansion.rs # Expansion variables d'environnement
    ├── errors.rs        # Types d'erreurs
    └── logger.rs        # Système de logs (placeholder)
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

## 🧪 Tests et Validation

### Tests Unitaires

```bash
cargo test                # Tests des modules
cargo test --verbose     # Tests avec détails
```

### Tests d'Intégration en Conditions Réelles

Un module de test complet avec **Multipass** permet de tester xsshend sur de vraies VMs Ubuntu :

```bash
# Démonstration complète automatisée
cd test/
./demo.sh

# Ou setup manuel
./test-vms.sh launch-all           # Lance 5 VMs Ubuntu configurées
./generate-test-files.sh           # Génère fichiers de test variés
./run-integration-tests.sh         # Exécute 9 tests automatisés
```

#### Environnement de Test

- **5 VMs Ubuntu 22.04** simulant Dev/Staging/Production
- **SSH configuré automatiquement** avec clés de test
- **Tests parallèles** sur plusieurs serveurs simultanément
- **Fichiers variés** : texte, JSON, binaires, gros fichiers (1MB)
- **Isolation complète** : aucun impact sur vos serveurs

#### Suite de Tests Automatisés

✅ Interface CLI et aide  
✅ Configuration hosts.json et filtrage  
✅ Mode dry-run (simulation)  
✅ Transfert simple et multiple  
✅ Barres de progression (gros fichiers)  
✅ Transferts parallèles multi-serveurs  
✅ Gestion d'erreurs robuste  
✅ Tests de performance et stress  

**Résultat attendu :** 9/9 tests passent = prêt pour production !

## 📖 Documentation

Consultez la documentation complète dans le répertoire `docs/` :

- **[Guide d'utilisation](docs/usage.md)** - Utilisation détaillée de l'interface hiérarchique
- **[Configuration](docs/configuration.md)** - Configuration avancée et personnalisation

### Liens rapides

- **Navigation dans l'interface** : [docs/usage.md#navigation-dans-linterface](docs/usage.md#navigation-dans-linterface)
- **Configuration des serveurs** : [docs/configuration.md#fichier-de-configuration-principal](docs/configuration.md#fichier-de-configuration-principal)
- **Raccourcis et alias** : [docs/configuration.md#raccourcis-et-personnalisation](docs/configuration.md#raccourcis-et-personnalisation)

## 🔧 Configuration avancée

Voir le [guide de configuration](docs/configuration.md) pour :

- Organisation optimale de l'infrastructure
- Variables d'environnement et personnalisation  
- Raccourcis shell et scripts de déploiement
- Résolution des problèmes courants

## � Dépannage

### Problèmes courants

#### Erreur: "Permission denied (publickey)"

```bash
# Vérifier la configuration SSH
ssh -v user@server.example.com

# Vérifier l'agent SSH
ssh-add -l

# Ajouter la clé si nécessaire
ssh-add ~/.ssh/id_rsa
```

#### Erreur: "hosts.json not found"

```bash
# Créer le fichier de configuration
mkdir -p ~/.ssh
# Créer et éditer avec vos serveurs
nano ~/.ssh/hosts.json
```

#### Serveurs déconnectés ou inaccessibles

```bash
# Utiliser --online-only pour pré-filtrer les serveurs accessibles
xsshend --online-only

# Les timeouts de connexion sont configurés automatiquement (5 secondes par défaut)
# En cas d'échec de connexion, xsshend continue avec les autres serveurs

# Vérifier la connectivité manuellement
ssh -o ConnectTimeout=5 user@server.example.com

# Logs d'erreur détaillés pour identifier les problèmes
RUST_LOG=debug xsshend upload file.txt --env Production
```

#### Performances lentes

```bash
# Réduire le parallélisme via variable d'environnement
export XSSHEND_MAX_PARALLEL=5
xsshend upload largefile.zip
```

Consultez le [guide de configuration](docs/configuration.md#dépannage) pour plus de solutions.

## 📝 Logs et Debug

```bash
# Mode verbeux
RUST_LOG=debug xsshend upload file.txt --region Production

# Affichage détaillé des transferts
xsshend upload file.txt --region Production --verbose

# Mode trace pour debugging SSH
RUST_LOG=ssh2=trace xsshend upload file.txt --region Production
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
