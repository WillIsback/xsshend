# Changelog

## [0.4.1] - 2025-10-18 🔒 SECURITY DOCUMENTATION

### 🔒 Sécurité

- **📄 Ajout de SECURITY.md** : Documentation complète de la politique de sécurité
- **⚠️ Documentation de RUSTSEC-2023-0071** : Vulnérabilité connue (Marvin Attack) dans `rsa 0.9.8`
  - Dépendance transitive via `russh 0.54.6`
  - Sévérité moyenne (5.9/10)
  - **Aucun correctif disponible** actuellement
  - **Recommandations** : Utiliser des clés Ed25519 et des réseaux de confiance
- **📋 Ajout de deny.toml** : Configuration cargo-deny avec exemption documentée
- **🔧 Workflow CI/CD** : Ajout de vérifications de sécurité automatiques (`.github/workflows/security.yml`)

### 📚 Documentation

- **README.md** : Ajout d'une section "Note de Sécurité" visible
- **SECURITY.md** : Politique de sécurité complète avec :
  - Description de la vulnérabilité RUSTSEC-2023-0071
  - Recommandations d'utilisation sécurisée
  - Guide de signalement de vulnérabilités
  - Historique et statut des vulnérabilités connues

### 🛡️ Mitigation

**Contexte** : La crate `rsa 0.9.8` (dépendance de `russh`) contient une vulnérabilité de timing sidechannel (Marvin Attack). Bien qu'aucun correctif ne soit disponible, l'impact peut être minimisé :

**✅ Utilisations SÉCURISÉES** :
- Réseaux privés/internes
- Connexions via VPN
- Environnements de développement local
- Utilisation de clés **Ed25519** (recommandé, non affectées)

**⚠️ Utilisations À RISQUE** :
- Serveurs publics sur Internet
- Réseaux WiFi publics
- Utilisation de clés **RSA** (affectées par la vulnérabilité)

### 🔗 Références

- Advisory: https://rustsec.org/advisories/RUSTSEC-2023-0071
- Issue russh: https://github.com/Eugeny/russh/issues/337
- Marvin Attack: https://people.redhat.com/~hkario/marvin/

## [0.4.0] - 2025-10-17 🚀 PURE RUST EDITION

### 🎉 Migration Majeure : OpenSSL → Pure Rust

**BREAKING CHANGE** : Migration complète de `ssh2` (C/OpenSSL) vers `russh` (100% Pure Rust)

#### ✨ Nouveautés

- **⚡ Compilation 4x plus rapide** : De ~60-70s à ~16s (et 5-8s sur recompilations)
- **🦀 100% Pure Rust** : Plus aucune dépendance C ou OpenSSL
- **🚀 Architecture Async** : Utilisation de Tokio pour des performances optimales
- **🔒 RustCrypto** : Cryptographie moderne et audité régulièrement
- **🌐 Cross-platform amélioré** : Compilation uniforme sans dépendances système

#### 🔧 Changements Techniques

**Dépendances :**
- ❌ Supprimé : `ssh2` (wrapper C), `libssh2-sys`, `openssl-sys` (⏰ build C long)
- ✅ Ajouté : `russh` v0.45, `russh-sftp` v2.0, `russh-keys` v0.45
- ✅ Ajouté : `tokio` v1 (async runtime), `async-trait` v0.1

**Architecture :**
- Conversion complète en async/await avec Tokio
- API SSH modernisée avec `russh::client::Handle`
- SFTP asynchrone avec `SftpSession`
- Authentification multi-clés préservée (v0.3.4)

#### 📊 Performances

**Temps de Compilation** :
```
AVANT (v0.3.x - OpenSSL) :  ~60-70 secondes
APRÈS (v0.4.0 - Pure Rust): ~16 secondes
GAIN : 4x plus rapide ! 🚀
```

**Runtime** :
- Connexions SSH asynchrones (non bloquantes)
- Support de milliers de connexions simultanées
- Réduction de la mémoire utilisée (tasks vs threads)

#### 🔄 Compatibilité

**Pour les Utilisateurs** :
- ✅ CLI identique (aucun changement visible)
- ✅ Configuration identique (`~/.ssh/hosts.json`)
- ✅ Comportement identique
- ✅ Clés SSH identiques (Ed25519, RSA, ECDSA)
- ⚠️ Nécessite Rust 1.75+ pour compiler

**Pour les Développeurs** :
- ⚠️ API interne async (méthodes avec `.await`)
- ⚠️ `#[tokio::main]` requis dans main.rs
- ⚠️ Tests avec `#[tokio::test]` au lieu de `#[test]`

#### 🎯 Migration Guide

```bash
# Installation
cargo install xsshend --force

# Plus besoin de libssl-dev système !
# La compilation est maintenant beaucoup plus rapide
```

Voir `MIGRATION-RUSSH-0.4.0.md` pour les détails techniques complets.

---

## [0.3.4] - 2025-10-17

### Corrigé

- **🔐 Authentification SSH multi-clés** : Correction majeure du mécanisme d'authentification
  - Le programme essaie maintenant **TOUTES** les clés SSH disponibles, comme SSH natif
  - Auparavant, une seule clé était essayée (la "meilleure" selon priorité Ed25519 > RSA > ECDSA)
  - Maintenant, si une clé échoue, le programme tente automatiquement les autres clés disponibles
  - Compatible avec les serveurs qui n'acceptent que certains types de clés (RSA uniquement, etc.)
  - Logs détaillés indiquant quelle clé a réussi l'authentification
  - Fallback automatique en cas d'échec d'une clé

### Détails techniques

- Remplacement de `select_key_auto()` par `get_all_keys()` dans la logique d'authentification
- Boucle d'essai sur toutes les clés disponibles jusqu'à succès
- Conservation de la priorité ssh-agent (toutes les clés sont déjà gérées par l'agent)
- Meilleure compatibilité avec les infrastructures SSH variées

## [0.3.3] - 2025-10-17

### Amélioré

- **📖 Aide CLI enrichie** : Ajout d'exemples d'utilisation détaillés dans toutes les commandes
  - `xsshend --help` : Exemples complets avec tous les cas d'usage courants
  - `xsshend upload --help` : Guide détaillé des filtres et combinaisons possibles
  - `xsshend list --help` : Exemples d'utilisation de la commande list
  - `xsshend init --help` : Documentation du processus d'initialisation
  - Documentation des filtres disponibles : `--env`, `--region`, `--type`, `--dest`, `--dry-run`
  - Exemples de filtrage combiné pour cibler précisément les serveurs
  - Cas d'usage multi-fichiers et wildcards

## [0.3.2] - 2025-10-17

### Corrigé

- **🐛 Résolution DNS** : Correction du crash lors de la connexion SSH avec des hostnames au lieu d'adresses IP
  - Erreur `AddrParseError(Socket)` corrigée en utilisant `ToSocketAddrs` pour résoudre les hostnames
  - Meilleure gestion des erreurs avec messages explicites lors de la résolution d'adresse
  - Support complet des FQDN (Fully Qualified Domain Names)

## [0.3.1] - 2025-10-17

### Corrigé

- **🐛 Compatibilité Rust** : Correction des expressions `let` chains pour supporter Rust 1.70+ (au lieu de 1.80+)
- **📦 Édition Rust** : Changement de l'édition invalide "2024" vers "2021" (standard actuel)

### Optimisé

- **⚡ Dépendances** : Suppression de dépendances inutilisées (`chrono`, `thiserror`, `ssh2-config`)
- **🚀 Compilation** : Réduction significative du temps de compilation (~30% plus rapide)
- **📖 Documentation** : Ajout de conseils pour accélérer l'installation

### Détails techniques

- Remplacement des `let` chains (Rust 1.80+) par des conditions `if let` imbriquées classiques
- Suppression de 3 dépendances non utilisées, réduisant le graphe de compilation
- Documentation améliorée pour l'installation avec OpenSSL système

## [0.3.0] - 2025-07-15

### Ajouté

- **🔑 Sélection CLI de clés SSH** : Options `--ssh-key` et `--ssh-key-interactive` pour choisir une clé SSH spécifique en ligne de commande
- **🤖 Sélection automatique intelligente** : Détection et proposition automatique de la meilleure clé SSH disponible
- **🎯 Intégration complète clés SSH** : Support des clés spécifiées dans le pool de connexions SSH
- **💬 Messages informatifs** : Affichage clair de la clé SSH utilisée pour chaque connexion

### Amélioré

- **🔧 Gestionnaire de clés SSH** : Amélioration de la découverte et sélection des clés
- **📋 Aide CLI** : Documentation des nouvelles options de clés SSH
- **🔗 Intégration uploader** : Support des clés SSH spécifiées dans l'orchestrateur de téléversement
- **⚡ Pool de connexions** : Prise en compte des clés SSH spécifiées pour les connexions

### Corrigé

- **🐛 Problème clé par défaut** : Les clés spécifiées en CLI sont maintenant correctement utilisées
- **🔑 Sélection Ed25519** : La clé Ed25519 n'est plus forcée si une autre clé est spécifiée
- **❌ Échecs d'authentification** : Meilleure gestion des erreurs d'authentification avec clés spécifiques

### Technique

- **📦 Nouvelle dépendance** : `atty` pour la détection de terminal interactif
- **🏗️ Architecture uploader** : Support des clés SSH dans `SshConnectionPool`
- **🔧 API étendue** : Nouvelles méthodes `new_with_key()` pour uploader et pool

## [0.2.0] - 2025-07-09

### Ajouté

- **🎨 Gestion automatique du thème** : Détection automatique des thèmes clair/sombre du terminal
- **🔑 Sélection interactive de clés SSH** : Interface dédiée pour choisir la clé SSH à utiliser
- **🌈 Système de couleurs avancé** : Couleurs adaptatives pour une meilleure lisibilité sur tous les terminaux
- **📱 Interface TUI améliorée** : Titres de panneaux plus visibles et contraste optimisé
- **🎯 Support multi-clés SSH** : Découverte automatique et sélection des clés SSH disponibles
- **🔧 Intégration ssh-agent** : Support complet pour l'agent SSH système
- **✨ Styles dynamiques** : Adaptation automatique des couleurs selon le thème du terminal

### Amélioré

- **🎨 Lisibilité en thème clair** : Correction des problèmes de contraste pour les éléments non sélectionnés
- **📋 Panneau d'aide** : Amélioration de la visibilité du texte d'aide en bas d'écran
- **🔍 Sélection hiérarchique** : Application des couleurs du thème au sélecteur de serveurs
- **⚡ Performance** : Optimisation du rendu et réduction des warnings Clippy
- **🏗️ Architecture** : Refactoring pour une meilleure séparation des responsabilités

### Corrigé

- **🐛 Contraste insuffisant** : Éléments non sélectionnés quasi invisibles en thème clair
- **🔧 Warnings Clippy** : Correction de tous les warnings et suggestions du linter
- **📚 Code mort** : Suppression du code inutilisé et ajout d'attributs appropriés
- **🎯 Imports** : Nettoyage des imports inutilisés

### Technique

- **📦 Nouvelles dépendances** : `termbg`, `terminal-colorsaurus`, `ssh2-config`
- **🏗️ Nouveaux modules** : `src/ui/theme.rs`, `src/ssh/keys.rs`
- **🔧 Refactoring** : Séparation de la logique de thème et de sélection de clés
- **✅ Tests** : Amélioration de la couverture de tests pour les nouveaux modules

## [0.1.3] - 2025-07-08

### Ajouté

- **Interface hiérarchique avancée** : Sélection de serveurs par arborescence
- **Recherche en temps réel** : Filtrage rapide des serveurs
- **Gestion multi-fichiers** : Sélection et upload de plusieurs fichiers
- **Workflow multi-étapes** : Interface guidée pour l'utilisateur

## [0.1.0] - 2025-07-05

### Ajouté

- **Vraie implémentation SSH/SFTP** avec ssh2-rs
- **Client SSH réel** avec authentification par clés et agent SSH
- **Transfert avec progression** (barres de progression individuelles)
- **Téléversement parallèle** vers plusieurs serveurs simultanément
- **Gestion d'erreurs robuste** avec résumés détaillés
- **Validation automatique** des fichiers avant transfert
- **Intégration complète** CLI → Config → SSH → UI

### Modifié

- Remplacé tous les placeholders par des vraies implémentations
- Amélioré l'affichage des résultats avec statistiques détaillées
- Intégré la vraie logique de transfert dans le main.rs

### Corrigé

- Imports manquants pour std::path::Path
- Signatures de fonctions pour la compatibilité avec les nouveaux types

### Performance

- Transferts parallèles avec rayon
- Buffers optimisés pour le transfert SFTP (64KB chunks)

## [0.0.1] - 2025-01-05 (Kickoff Version)

### Ajouté

- 🎉 Version initiale de xsshend - Proof of Concept
- ⚙️ Configuration via ~/.ssh/hosts.json
- 📋 Commande `list` pour afficher les serveurs disponibles
- 🔍 Mode `dry-run` pour simulation de téléversement
- 🎯 Filtrage par environnement, région et type de serveur
- 🏗️ Architecture modulaire avec placeholders pour implémentation complète

### Fonctionnalités CLI

- CLI moderne avec `clap` 4.x
- Interface intuitive avec sous-commandes
- Gestion d'erreurs robuste avec `anyhow`
- Support de la configuration hiérarchique JSON

### Architecture

- Structure modulaire complète (config, ssh, ui, core, utils)
- Tests unitaires fonctionnels
- Documentation technique
- Scripts de build automatisés

### À venir (v0.1.0)

- 🚀 Implémentation complète du téléversement SSH/SFTP
- 📊 Barres de progression en temps réel avec `indicatif`
- 🔄 Parallélisation avec `rayon`
- 🔐 Authentification SSH par clés
- 🎮 Mode interactif avec `dialoguer`
- 📈 Interface TUI avec `ratatui`
