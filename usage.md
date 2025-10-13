# 📖 Guide d'utilisation de xsshend

> Guide complet pour maîtriser xsshend - Téléversement SSH simple et efficace via CLI

## 🎯 Vue d'ensemble

xsshend est un outil Rust moderne pour le téléversement de fichiers vers multiples serveurs SSH. Interface en ligne de commande simple avec filtrage avancé et suivi en temps réel des transferts.

## 🚀 Installation et configuration

### Installation via Cargo

```bash
cargo install xsshend
```

### Configuration initiale

```bash
# Initialisation assistée avec création du fichier hosts.json
xsshend init

# Forcer la réinitialisation si nécessaire
xsshend init --force
```

### Première utilisation

```bash
# Lister les serveurs disponibles
xsshend list

# Aide générale
xsshend --help

# Aide sur une commande spécifique
xsshend upload --help
```

## ⚙️ Configuration des serveurs

### Structure hiérarchique

xsshend organise les serveurs selon une structure à 3 niveaux :

```
Environment/
├── Region/
│   ├── Type/
│   │   ├── SERVER_NAME_1
│   │   └── SERVER_NAME_2
│   └── Type/
└── Region/
```

### Fichier hosts.json

Le fichier `~/.ssh/hosts.json` contient la configuration :

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

## 🎮 Utilisation CLI

### Commandes principales

#### Initialisation
```bash
xsshend init           # Configuration assistée
xsshend init --force   # Réinitialisation complète
```

#### Liste des serveurs
```bash
xsshend list           # Affichage hiérarchique avec filtres CLI
xsshend --list         # Forme courte
```

#### Téléversement de fichiers
```bash
xsshend upload <FILES>... [OPTIONS]
```

### Options de filtrage

#### Par environnement
```bash
xsshend upload config.json --env Production
xsshend upload app.jar --env Staging
xsshend upload debug.log --env Development
```

#### Par région
```bash
xsshend upload regional-config.json --region Region-A
xsshend upload backup.tar --region Region-B
```

#### Par type de serveur
```bash
xsshend upload public-config.json --type Public
xsshend upload db-script.sql --type Private
```

#### Filtrage combiné
```bash
# Environnement + Région
xsshend upload config.json --env Production --region Region-A

# Environnement + Type
xsshend upload app.war --env Production --type Public

# Région + Type
xsshend upload local-config.json --region Region-A --type Private

# Tous les filtres
xsshend upload deploy.sh --env Production --region Region-A --type Public
```

### Gestion des destinations

```bash
# Destination par défaut (/tmp/)
xsshend upload file.txt --env Production

# Destination personnalisée
xsshend upload app.war --env Production --dest /opt/tomcat/webapps/
xsshend upload config.json --env Staging --dest /etc/myapp/
xsshend upload logs/ --env Development --dest /var/log/myapp/
```

### Mode simulation (dry-run)

```bash
# Simulation complète sans transfert réel
xsshend upload deploy.sh --env Production --dry-run

# Vérification des serveurs ciblés
xsshend upload app.jar --env Production --region Region-A --dry-run

# Test avec filtrage complexe
xsshend upload config.json --env Production --type Public --dry-run
```

## 📊 Interface de progression

### Affichage en temps réel

Les transferts montrent une progression détaillée :

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

### Gestion des erreurs

En cas d'erreur, xsshend affiche des détails précis :

```
❌ Échec de connexion: WEB_SERVER_02
   Erreur: Connection refused (port 22)
   Conseil: Vérifier que le serveur est accessible

✅ Téléversement partiel terminé
📊 2/3 serveur(s) réussis - 1 échec(s)
```

## 🔑 Gestion des clés SSH

### Découverte automatique

xsshend détecte automatiquement les clés SSH :

- **Ed25519** : `~/.ssh/id_ed25519` (priorité 1)
- **RSA** : `~/.ssh/id_rsa` (priorité 2)
- **ECDSA** : `~/.ssh/id_ecdsa` (priorité 3)
- **DSA** : `~/.ssh/id_dsa` (priorité 4)

### Intégration ssh-agent

Si aucune clé n'est trouvée ou accessible, xsshend utilise ssh-agent :

```bash
# Vérifier les clés chargées
ssh-add -l

# Ajouter une clé
ssh-add ~/.ssh/id_ed25519

# Démarrer ssh-agent si nécessaire
eval $(ssh-agent)
```

## 📝 Exemples d'utilisation

### Déploiement d'application

```bash
# Déploiement complet en production
xsshend upload myapp.war --env Production --dest /opt/tomcat/webapps/

# Mise à jour de configuration
xsshend upload application.properties --env Production --dest /etc/myapp/

# Déploiement sur serveurs publics seulement
xsshend upload static-files/ --env Production --type Public --dest /var/www/
```

### Gestion par environnement

```bash
# Configuration de développement
xsshend upload dev-config.json --env Development

# Test en staging
xsshend upload app-v2.jar --env Staging --dry-run

# Déploiement production avec validation
xsshend upload production-app.war --env Production --dry-run
xsshend upload production-app.war --env Production  # Si OK
```

### Maintenance et logs

```bash
# Copie de logs pour analyse
xsshend upload analyze-script.py --env Production --type Private

# Sauvegarde de configuration
xsshend upload backup-script.sh --env Production --region Region-A

# Mise à jour de sécurité
xsshend upload security-patch.sh --env Production --dry-run  # Test
xsshend upload security-patch.sh --env Production          # Application
```

### Multi-fichiers et répertoires

```bash
# Plusieurs fichiers
xsshend upload config.json app.jar deploy.sh --env Production

# Contenu d'un répertoire
xsshend upload static-files/ --env Production --type Public

# Mix fichiers et répertoires
xsshend upload app.war config/ scripts/ --env Staging
```

## 🔍 Commandes de diagnostic

### Liste détaillée des serveurs

```bash
# Vue complète avec filtres CLI
xsshend list
```

Sortie exemple :
```
🔍 Liste des cibles SSH disponibles:

📁 Production (--env Production)
  📂 Region-A (--region Region-A)
    📂 Public (--type Public)
      🖥️  WEB_SERVER_01 → web01@prod-web-01.example.com (PROD)
      🖥️  API_SERVER_01 → api01@prod-api-01.example.com (PROD)
    📂 Private (--type Private)
      🗄️  DATABASE_01 → db01@prod-db-01.example.com (PROD)

📁 Staging (--env Staging)
  📂 Region-A (--region Region-A)
    📂 Public (--type Public)
      🖥️  STAGE_WEB_01 → web01@stage-web-01.example.com (STAGE)

📊 Total: 4 cibles disponibles

💡 Exemples d'utilisation:
   xsshend upload --env Production file.txt
   xsshend upload --env Staging --region Region-A file.txt
   xsshend upload --region Region-A --type Public file.txt
```

### Validation de configuration

```bash
# Test avec dry-run pour valider la configuration
xsshend upload test.txt --env Production --dry-run

# Test sur un serveur spécifique avec SSH manuel
ssh web01@prod-web-01.example.com "echo 'Test connection OK'"
```

## 🛠️ Conseils et bonnes pratiques

### Organisation des serveurs

1. **Environnements** : Production, Staging, Development, Testing
2. **Régions** : Region-A, Region-B, US-East, EU-West...
3. **Types** : Public, Private, Database, Cache, Load-Balancer

### Workflow recommandé

1. **Test** : Toujours utiliser `--dry-run` d'abord
2. **Staging** : Tester sur environnement de staging
3. **Production** : Déployer par étapes (région par région)

### Sécurité

- Utiliser des clés SSH Ed25519 de préférence
- Éviter les mots de passe, privilégier ssh-agent
- Valider les permissions des fichiers de configuration

### Performance

- Grouper les fichiers pour réduire les connexions SSH
- Utiliser le mode dry-run pour valider avant transfert
- Organiser la configuration pour un filtrage efficace

## 🚫 Dépannage

### Problèmes de connexion SSH

```bash
# Test manuel de connexion
ssh -v web01@prod-web-01.example.com

# Vérification des clés
ssh-add -l

# Ajout de clé si nécessaire
ssh-add ~/.ssh/id_ed25519
```

### Configuration

```bash
# Réinitialiser la configuration
xsshend init --force

# Vérifier les permissions .ssh
ls -la ~/.ssh/
chmod 700 ~/.ssh
chmod 600 ~/.ssh/id_*
```

### Logs et debug

```bash
# Mode verbeux
RUST_LOG=debug xsshend upload file.txt --env Production

# Logs très détaillés
RUST_LOG=trace xsshend upload file.txt --env Production
```