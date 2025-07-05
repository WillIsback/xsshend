# Gestion Automatique des Clés SSH

## Vue d'ensemble

`xsshend` intègre désormais une gestion automatique des clés SSH pour simplifier l'expérience des nouveaux utilisateurs. L'application vérifie automatiquement l'existence de clés SSH privées lors du démarrage et propose de générer une nouvelle clé si aucune n'est trouvée.

## Fonctionnalités

### Détection Automatique des Clés

Au démarrage, `xsshend` vérifie la présence de clés SSH dans `~/.ssh/` selon l'ordre de préférence suivant :

1. **Ed25519** (`id_ed25519`) - Recommandé pour sa sécurité et performance
2. **RSA** (`id_rsa`) - Support legacy
3. **ECDSA** (`id_ecdsa`) - Alternative moderne

### Génération Automatique

Si aucune clé n'est trouvée, l'application :

1. **Crée automatiquement** le répertoire `~/.ssh/` si nécessaire
2. **Informe l'utilisateur** de l'absence de clés SSH
3. **Propose interactivement** de générer une nouvelle clé Ed25519
4. **Guide l'utilisateur** dans le processus de génération

## Processus de Génération

### Interaction Utilisateur

```bash
❌ Aucune clé SSH trouvée dans ~/.ssh/
🔑 Pour utiliser xsshend, vous avez besoin d'une clé SSH privée.
Voulez-vous générer une nouvelle clé SSH Ed25519 ? (o/N): o
Entrez votre adresse email (optionnel, pour identifier la clé): user@example.com
🔄 Génération de la clé SSH en cours...
✅ Clé SSH Ed25519 générée avec succès: /home/user/.ssh/id_ed25519
📋 Clé publique: /home/user/.ssh/id_ed25519.pub
```

### Paramètres de Génération

- **Type de clé** : Ed25519 (cryptographie moderne et rapide)
- **Nom de fichier** : `id_ed25519` (standard SSH)
- **Passphrase** : Aucune (pour simplifier l'utilisation automatisée)
- **Commentaire** : Email utilisateur (optionnel, pour identification)

### Affichage de la Clé Publique

Après génération, l'application affiche :

```bash
📄 Contenu de votre clé publique:
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIq42UTi1i/xQaRTUbxJeMk0K5lCrR578pKJ+0uBC/TB user@example.com

💡 Copiez cette clé publique sur vos serveurs avec:
   ssh-copy-id -i ~/.ssh/id_ed25519.pub user@hostname
```

## Sécurité

### Meilleures Pratiques

- **Algorithme Ed25519** : Utilise la cryptographie la plus moderne
- **Permissions strictes** : Clé privée avec permissions 600 (lecture seule pour le propriétaire)
- **Pas de passphrase par défaut** : Simpllifie l'usage automatisé (peut être ajoutée manuellement)

### Recommandations

Pour une sécurité renforcée, vous pouvez ajouter une passphrase à votre clé :

```bash
ssh-keygen -p -f ~/.ssh/id_ed25519
```

## Cas d'Usage

### Nouveau Utilisateur

1. **Installation** : `xsshend` installé pour la première fois
2. **Premier lancement** : Aucune clé SSH existante
3. **Auto-configuration** : Création automatique de `hosts.json` et génération de clé SSH
4. **Prêt à l'usage** : Configuration complète en une seule commande

### Utilisateur Existant

1. **Détection** : Clés SSH existantes détectées automatiquement
2. **Message informatif** : Indication des clés trouvées
3. **Pas d'intervention** : Aucune modification des clés existantes

## Intégration

### Avec la Configuration Automatique

Cette fonctionnalité complète la création automatique du fichier `hosts.json` :

1. **Création du répertoire** `~/.ssh/` (si nécessaire)
2. **Génération du fichier** `hosts.json` (si absent)
3. **Vérification des clés SSH** (nouvelle fonctionnalité)
4. **Génération de clé** (si aucune trouvée et acceptée par l'utilisateur)

### Dépendances Système

La génération de clés nécessite :

- **OpenSSH** : Commande `ssh-keygen` disponible dans le PATH
- **Droits d'écriture** : Accès en écriture au répertoire `~/.ssh/`

## Messages d'Erreur

### ssh-keygen non trouvé

```bash
Impossible d'exécuter ssh-keygen. Assurez-vous qu'OpenSSH est installé.
```

**Solution** : Installer OpenSSH client

```bash
# Ubuntu/Debian
sudo apt-get install openssh-client

# CentOS/RHEL
sudo yum install openssh-clients

# openSUSE
sudo zypper install openssh
```

### Erreurs de permissions

```bash
Impossible de créer le répertoire /home/user/.ssh
```

**Solution** : Vérifier les permissions du répertoire home

## Désactivation

Pour désactiver la génération automatique, répondez simplement "N" ou "n" à la question :

```bash
Voulez-vous générer une nouvelle clé SSH Ed25519 ? (o/N): n
⚠️  Génération de clé SSH ignorée.
💡 Vous pouvez générer une clé manuellement avec :
   ssh-keygen -t ed25519 -C "votre_email@example.com"
```

## Limitations

1. **Une seule clé générée** : Seule une clé Ed25519 est proposée
2. **Pas de passphrase** : Génération sans passphrase pour simplifier l'usage
3. **Pas de configuration avancée** : Utilise les paramètres par défaut d'OpenSSH
4. **Pas de sauvegarde** : Ne sauvegarde pas les clés existantes avant modification

## Roadmap

### Améliorations Futures Possibles

- **Support de passphrase** : Option pour générer des clés avec passphrase
- **Choix d'algorithme** : Permettre de choisir entre Ed25519, RSA, ECDSA
- **Configuration avancée** : Personnalisation des paramètres de génération
- **Importation de clés** : Assistant pour importer des clés existantes
