# Changelog

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
