# Changelog

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
