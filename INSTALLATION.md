# 🚀 Guide d'Installation Optimisée - xsshend

## Pré-requis

- **Rust** : version 1.70 ou supérieure (recommandé : dernière version stable)
- **OpenSSL** (optionnel mais recommandé pour accélérer la compilation)

Vérifier votre version de Rust :
```bash
rustc --version
```

Si besoin, mettre à jour Rust :
```bash
rustup update stable
```

## Installation Standard

### Via Cargo (crates.io)

```bash
cargo install xsshend
```

⏱️ **Temps estimé** : 5-10 minutes (selon votre machine)

## ⚡ Installation Rapide (Recommandé)

Le temps de compilation peut être considérablement réduit en utilisant OpenSSL du système au lieu de le compiler depuis les sources.

### Option 1 : Utiliser OpenSSL du Système

#### Sur Ubuntu/Debian

```bash
# Installer les dépendances OpenSSL
sudo apt-get update
sudo apt-get install -y libssl-dev pkg-config

# Installer xsshend
cargo install xsshend
```

⏱️ **Temps estimé** : 2-3 minutes

#### Sur Fedora/RHEL/CentOS

```bash
# Installer les dépendances OpenSSL
sudo dnf install -y openssl-devel pkg-config

# Installer xsshend
cargo install xsshend
```

#### Sur Arch Linux

```bash
# Installer les dépendances OpenSSL
sudo pacman -S openssl pkg-config

# Installer xsshend
cargo install xsshend
```

#### Sur macOS

```bash
# Installer OpenSSL via Homebrew
brew install openssl pkg-config

# Installer xsshend
cargo install xsshend
```

### Option 2 : Compilation Parallèle

Utiliser tous les cœurs CPU pour accélérer la compilation :

```bash
# Linux/macOS
cargo install xsshend -j $(nproc)

# macOS (alternative)
cargo install xsshend -j $(sysctl -n hw.ncpu)
```

### Option 3 : Combinaison (Plus Rapide)

```bash
# Ubuntu/Debian
sudo apt-get install -y libssl-dev pkg-config
cargo install xsshend -j $(nproc)
```

⏱️ **Temps estimé** : 1-2 minutes

## Installation depuis les Sources

### Cloner et Installer

```bash
# Cloner le dépôt
git clone https://github.com/willisback/xsshend.git
cd xsshend

# Installation standard
cargo install --path .

# Ou installation rapide avec jobs parallèles
cargo install --path . -j $(nproc)
```

### Mode Développement

```bash
# Build en mode debug (plus rapide)
cargo build

# Exécuter directement
./target/debug/xsshend --help

# Build en mode release (optimisé)
cargo build --release
./target/release/xsshend --help
```

## Résolution de Problèmes

### Erreur : "could not find `libssl`"

**Solution** : Installer OpenSSL développement
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# Fedora/RHEL
sudo dnf install openssl-devel pkg-config
```

### Erreur : "unstable feature 'let_chains'"

**Solution** : Mettre à jour Rust ou utiliser la version 0.3.1+
```bash
rustup update stable
cargo install xsshend --force
```

### Compilation Trop Lente

**Solutions** :
1. Installer OpenSSL du système (voir Option 1 ci-dessus)
2. Utiliser la compilation parallèle avec `-j`
3. Vérifier l'espace disque disponible (minimum 2 Go recommandé)
4. Utiliser `cargo install --locked xsshend` pour éviter les mises à jour de dépendances

### Erreur de Version Rust

**Solution** : Mettre à jour Rust vers une version >= 1.70
```bash
rustup update stable
rustup default stable
```

## Vérification de l'Installation

```bash
# Vérifier la version installée
xsshend --version

# Afficher l'aide
xsshend --help

# Lancer les tests (si installé depuis les sources)
cargo test
```

## Désinstallation

```bash
cargo uninstall xsshend
```

## Optimisations Avancées

### Cache de Compilation

Pour les développeurs qui compilent fréquemment :

```bash
# Installer sccache pour mettre en cache les compilations
cargo install sccache

# Configurer Rust pour utiliser sccache
export RUSTC_WRAPPER=sccache

# Puis compiler normalement
cargo install xsshend
```

### Build Minimal (Pas Recommandé)

Si vous avez vraiment besoin de réduire la taille et le temps de compilation :

```bash
# Clone le dépôt d'abord
git clone https://github.com/willisback/xsshend.git
cd xsshend

# Build avec optimisations minimales
cargo build --release --no-default-features
```

⚠️ **Attention** : Certaines fonctionnalités peuvent être désactivées

## Support

- 📖 [Documentation complète](https://willisback.github.io/xsshend/)
- 🐛 [Signaler un bug](https://github.com/willisback/xsshend/issues)
- 💬 [Discussions](https://github.com/willisback/xsshend/discussions)

## Comparaison des Temps de Compilation

| Méthode | Temps Estimé | Recommandé |
|---------|--------------|------------|
| Installation standard (vendored OpenSSL) | 5-10 min | ❌ |
| OpenSSL système | 2-3 min | ✅ |
| OpenSSL système + jobs parallèles | 1-2 min | ✅✅ |
| Build depuis sources (debug) | 1-2 min | 🔧 |

*Temps basés sur une machine avec CPU 4 cœurs, 8 Go RAM, SSD*
