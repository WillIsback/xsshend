# Configuration de xsshend

## Fichier de configuration principal

Le fichier de configuration se trouve dans `~/.ssh/hosts.json` et définit votre infrastructure de serveurs.

### Structure hiérarchique

```json
{
  "Environnement": {
    "Région": {
      "Type": {
        "NOM_SERVEUR": {
          "alias": "utilisateur@hostname",
          "env": "TAG_ENVIRONNEMENT"
        }
      }
    }
  }
}
```

### Exemple complet

```json
{
  "Production": {
    "Region-A": {
      "Public": {
        "WEB_SERVER_01": {
          "alias": "web01@prod-web-01.example.com",
          "env": "PROD",
          "description": "Serveur web principal"
        },
        "API_SERVER_01": {
          "alias": "api01@prod-api-01.example.com",
          "env": "PROD",
          "description": "API principale"
        },
        "DATABASE_01": {
          "alias": "db01@prod-db-01.example.com",
          "env": "PROD",
          "description": "Base de données primaire"
        }
      },
      "Private": {
        "INTERNAL_API_01": {
          "alias": "iapi01@prod-internal-01.example.com",
          "env": "PROD",
          "description": "API interne"
        },
        "BACKUP_SERVER_01": {
          "alias": "backup01@prod-backup-01.example.com",
          "env": "PROD",
          "description": "Serveur de sauvegarde"
        }
      }
    },
    "Region-B": {
      "Public": {
        "WEB_SERVER_02": {
          "alias": "web02@prod-web-02.example.com",
          "env": "PROD",
          "description": "Serveur web secondaire"
        },
        "CACHE_SERVER_01": {
          "alias": "cache01@prod-cache-01.example.com",
          "env": "PROD",
          "description": "Serveur de cache Redis"
        }
      }
    }
  },
  "Staging": {
    "Region-A": {
      "Public": {
        "STAGE_WEB_01": {
          "alias": "web01@stage-web-01.example.com",
          "env": "STAGE",
          "description": "Environnement de test web"
        },
        "STAGE_API_01": {
          "alias": "api01@stage-api-01.example.com",
          "env": "STAGE",
          "description": "API de test"
        }
      },
      "Private": {
        "STAGE_DATABASE": {
          "alias": "db01@stage-db-01.example.com",
          "env": "STAGE",
          "description": "Base de données de test"
        }
      }
    }
  },
  "Development": {
    "Local": {
      "Services": {
        "DEV_WEB_01": {
          "alias": "web01@dev.local.example.com",
          "env": "DEV",
          "description": "Serveur de développement local"
        },
        "DEV_DATABASE": {
          "alias": "db@dev.local.example.com",
          "env": "DEV",
          "description": "Base de données de développement"
        }
      }
    },
    "Demo": {
      "Hetzner": {
        "DEMO_SERVER": {
          "alias": "root@demo.hetzner.example.com",
          "env": "DEMO",
          "description": "Serveur de démonstration"
        }
      }
    }
  }
}
```

## Champs de configuration

### Champs obligatoires

- **alias** : Chaîne de connexion SSH (format `utilisateur@hostname` ou `utilisateur@hostname:port`)
- **env** : Tag d'environnement pour identification (ex: "PROD", "STAGE", "DEV")

### Champs optionnels

- **description** : Description du serveur (affiché dans l'interface)
- **port** : Port SSH spécifique (si différent de 22)
- **key_file** : Chemin vers la clé SSH privée spécifique

## Bonnes pratiques d'organisation

### Noms des environnements

Utilisez des noms clairs et cohérents :

- **Production** : Environnement de production
- **Staging** : Environnement de pré-production
- **Development** : Environnement de développement
- **Testing** : Environnement de tests automatisés

### Noms des régions

Organisez par localisation géographique ou logique :

- **Region-A**, **Region-B** : Régions géographiques
- **Local** : Infrastructure locale
- **Cloud-AWS**, **Cloud-GCP** : Par fournisseur cloud
- **Datacenter-1**, **Datacenter-2** : Par centre de données

### Types de serveurs

Catégorisez par fonction ou visibilité :

- **Public** : Serveurs accessibles depuis Internet
- **Private** : Serveurs du réseau interne
- **Services** : Services spécifiques (base de données, cache, etc.)
- **Workers** : Serveurs de traitement
- **Storage** : Serveurs de stockage

### Noms des serveurs

Utilisez une convention cohérente :

- **Descriptif + Numéro** : `WEB_SERVER_01`, `API_SERVER_02`
- **Fonction + Environnement + Numéro** : `PROD_WEB_01`, `STAGE_API_01`
- **Service + Instance** : `NGINX_MAIN`, `POSTGRES_PRIMARY`

## Configuration SSH

### Authentification par clé

xsshend utilise l'authentification SSH par clé. Assurez-vous que :

1. Votre clé publique est déployée sur tous les serveurs cibles
2. Votre agent SSH est actif : `ssh-add ~/.ssh/id_rsa`
3. Les permissions sont correctes sur vos clés privées : `chmod 600 ~/.ssh/id_rsa`

### Fichier ~/.ssh/config

Vous pouvez combiner xsshend avec votre configuration SSH existante :

```
Host prod-web-*
    User web01
    IdentityFile ~/.ssh/prod_key
    StrictHostKeyChecking no

Host stage-*
    User deploy
    IdentityFile ~/.ssh/stage_key
    Port 2222

Host dev.local.*
    User developer
    IdentityFile ~/.ssh/dev_key
```

## Variables d'environnement

### Configuration xsshend

- **XSSHEND_CONFIG** : Chemin alternatif vers le fichier hosts.json
- **XSSHEND_LOG_LEVEL** : Niveau de log (debug, info, warn, error)
- **XSSHEND_MAX_PARALLEL** : Nombre maximum de transferts simultanés

Exemples :

```bash
export XSSHEND_CONFIG="/path/to/custom/hosts.json"
export XSSHEND_LOG_LEVEL="debug"
export XSSHEND_MAX_PARALLEL="10"
```

## Raccourcis et personnalisation

### Alias shell utiles

Ajoutez ces alias dans votre `.bashrc` ou `.zshrc` :

```bash
# Lancement rapide en mode interactif
alias xs='xsshend --interactive'

# Téléversement vers production avec confirmation
alias xsprod='xsshend upload --env Production --interactive'

# Téléversement vers staging
alias xsstage='xsshend upload --env Staging --interactive'

# Liste des serveurs de production
alias xslist='xsshend list --env Production'

# Mode simulation
alias xsdry='xsshend upload --dry-run'
```

### Scripts de déploiement

Créez des scripts pour vos déploiements récurrents :

```bash
#!/bin/bash
# deploy-frontend.sh
xsshend upload dist/* --env Production --type Public --dest /var/www/html/
```

```bash
#!/bin/bash
# deploy-config.sh
xsshend upload config/prod/*.conf --env Production --dest /etc/myapp/ --interactive
```

## Dépannage

### Problèmes de connexion

1. Vérifiez la connectivité : `ssh user@hostname`
2. Vérifiez l'agent SSH : `ssh-add -l`
3. Testez avec verbosité : `XSSHEND_LOG_LEVEL=debug xsshend ...`

### Fichier de configuration

1. Validez le JSON : `python -m json.tool ~/.ssh/hosts.json`
2. Vérifiez les permissions : `ls -la ~/.ssh/hosts.json`
3. Testez la configuration : `xsshend list`

### Performance

1. Réduisez le parallélisme : `XSSHEND_MAX_PARALLEL=5`
2. Utilisez la compression SSH : configurez `Compression yes` dans `~/.ssh/config`
3. Optimisez la taille des fichiers transférés
