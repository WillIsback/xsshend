# 🚀 xsshend

[![Crates.io](https://img.shields.io/crates/v/xsshend.svg)](https://crates.io/crates/xsshend)
[![Documentation](https://docs.rs/xsshend/badge.svg)](https://docs.rs/xsshend)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021%2B-orange.svg)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/github/actions/workflow/status/WillIsback/xsshend/rust.yml?branch=main)](https://github.com/WillIsback/xsshend/actions)

**xsshend** est un outil CLI Rust moderne pour **gérer vos serveurs SSH à grande échelle**. Téléversez des fichiers et exécutez des commandes sur plusieurs serveurs simultanément.

## ✨ Fonctionnalités

- 📤 **Upload** - Transfert parallèle de fichiers vers plusieurs serveurs
- ⚡ **Command** - Exécution de commandes SSH (séquentiel/parallèle)
- 🔐 Authentification SSH sécurisée (Ed25519, ECDSA, RSA)
- 📊 Barres de progression et format JSON pour CI/CD
- 🔍 Filtrage par environnement, région, type de serveur
- 🎨 Mode interactif pour configuration assistée

## 🔒 Note de Sécurité

⚠️ **Vulnérabilité connue**: Dépend de `russh` avec `rsa 0.9.8`, affecté par [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071).

**Recommandations** :
- ✅ Utilisez des clés **Ed25519** (non affectées)
- ✅ Utilisez sur **réseaux de confiance** (VPN, internes)
- ⚠️ Évitez les **réseaux publics non sécurisés**

📄 Voir [SECURITY.md](docs/SECURITY.md) pour plus de détails.

## 🚀 Installation

```bash
cargo install xsshend
```

## ⚡ Démarrage rapide

```bash
# 1. Initialiser la configuration
xsshend init

# 2. Éditer ~/.ssh/hosts.json avec vos serveurs

# 3. Lister les serveurs
xsshend list

# 4. Téléverser un fichier
xsshend upload app.tar.gz --env Production

# 5. Exécuter une commande
xsshend command --inline "systemctl restart nginx" --env Production
```

## 📖 Exemples

### Upload de fichiers

```bash
# Upload simple
xsshend upload myfile.txt --env Production

# Upload avec filtrage
xsshend upload config.json --env Staging --region EU-West --type Web

# Upload avec destination personnalisée
xsshend upload app.tar.gz --env Production --dest /opt/app/

# Mode simulation
xsshend upload deploy.sh --env Production --dry-run
```

### Exécution de commandes

```bash
# Commande simple
xsshend command --inline "uptime" --env Production

# Script bash
xsshend command --script deploy.sh --env Staging

# Mode parallèle
xsshend command --inline "systemctl restart nginx" --env Production --parallel

# Format JSON pour CI/CD
xsshend command --inline "hostname" --env Production --output-format json

# Avec timeout personnalisé
xsshend command --inline "apt update" --env Production --timeout 120
```

## ⚙️ Configuration

Fichier `~/.ssh/hosts.json` :

```json
{
  "Production": {
    "EU-West": {
      "Web": {
        "prod-web-01": {
          "alias": "deploy@prod-web-01.example.com",
          "env": "Production"
        }
      }
    }
  },
  "Staging": {
    "EU-West": {
      "Web": {
        "stage-web-01": {
          "alias": "deploy@stage-web-01.example.com",
          "env": "Staging"
        }
      }
    }
  }
}
```

## 📚 Documentation complète

- 📖 [**Documentation principale**](https://willisback.github.io/xsshend/)
- 📘 [Guide d'utilisation](docs/usage.md)
- ⚙️ [Configuration](docs/configuration.md)
- 🔐 [Gestion des clés SSH](docs/ssh-keys.md)
- 🔧 [Installation](docs/INSTALLATION.md)
- 🔒 [Sécurité](docs/SECURITY.md)
- 🤖 [CI/CD](docs/cicd.md)
- 📝 [Changelog](CHANGELOG.md)

## 🛠️ Développement

```bash
# Cloner le projet
git clone https://github.com/WillIsback/xsshend.git
cd xsshend

# Compiler
cargo build --release

# Tests (118 tests)
cargo test

# Qualité
cargo clippy
cargo fmt
```

## 🤝 Contribution

Les contributions sont bienvenues ! Avant de soumettre une PR :

1. ✅ Formatage : `cargo fmt`
2. ✅ Linting : `cargo clippy`
3. ✅ Tests : `cargo test`
4. ✅ Documentation à jour

## 📄 Licence

MIT License - voir [LICENSE](LICENSE)

## 🔗 Liens

- 📦 [Crate](https://crates.io/crates/xsshend)
- 📚 [Documentation](https://docs.rs/xsshend)
- 🌐 [Site Web](https://willisback.github.io/xsshend/)
- 🐙 [GitHub](https://github.com/WillIsback/xsshend)

## 👤 Auteur

**William Derue** - [@WillIsback](https://github.com/WillIsback)

---

<div align="center">

**⭐ Si ce projet vous est utile, n'hésitez pas à lui donner une étoile !**

Made with ❤️ and 🦀 Rust

</div>
