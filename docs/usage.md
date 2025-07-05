# Guide d'utilisation de xsshend

## Vue d'ensemble

xsshend est un outil de téléversement multi-SSH avec une interface utilisateur hiérarchique moderne. Il permet de sélectionner facilement des fichiers et des serveurs via une interface TUI intuitive et de transférer les fichiers en parallèle.

## Interface de listage avec étiquettes CLI

La commande `xsshend list` (ou `xsshend -l`) affiche maintenant un aperçu hiérarchique enrichi avec des étiquettes CLI pour faciliter l'utilisation en ligne de commande :

```bash
xsshend list
```

**Exemple de sortie :**
```
🔍 Liste des cibles SSH disponibles:

📁 Production (--env Production)
  📂 Region-A (--region Region-A)
    📂 Public (--type Public)
      🖥️  WEB_SERVER_01 → web01@prod-web-01.example.com (PROD)
      🖥️  API_SERVER_01 → api01@prod-api-01.example.com (PROD)
    📂 Private (--type Private)
      🖥️  DATABASE_01 → db01@prod-db-01.example.com (PROD)

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

### Utilisation des étiquettes CLI

Les étiquettes facilitent la construction des commandes de filtrage :

#### Filtrage par environnement
```bash
# Déployer sur tout l'environnement Production
xsshend upload --env Production deploy.sh

# Déployer sur l'environnement Staging
xsshend upload --env Staging config.json
```

#### Filtrage combiné environnement + région
```bash
# Déployer sur Production dans la Region-A uniquement
xsshend upload --env Production --region Region-A app.jar

# Déployer sur Staging dans une région spécifique
xsshend upload --env Staging --region Region-B logs.tar.gz
```

#### Filtrage par environnement + type
```bash
# Déployer sur les serveurs publics de Production
xsshend upload --env Production --type Public web-assets.zip

# Déployer sur les serveurs privés de Staging
xsshend upload --env Staging --type Private database-backup.sql
```

#### Filtrage traditionnel (région ou type seulement)
```bash
# Déployer sur tous les serveurs d'une région
xsshend upload --region Region-A monitoring.sh

# Déployer sur tous les serveurs d'un type
xsshend upload --type Public static-files.tar.gz
```

## Vérification de connectivité

La nouvelle option `--online-only` permet de vérifier la connectivité des serveurs avant de lancer l'interface TUI :

```bash
# Lance le TUI en n'affichant que les serveurs en ligne
xsshend --online-only
```

Cette option :
- Teste la connectivité SSH vers chaque serveur avec un timeout (5 secondes par défaut)
- Filtre automatiquement les serveurs hors ligne
- Affiche seulement les serveurs accessibles dans l'interface TUI
- Améliore les performances en évitant les timeouts pendant les transferts

## Interface utilisateur hiérarchique

### Principe

L'interface organise vos serveurs en arbre hiérarchique :
- **Environnements** (Production, Staging, Development) - Filtrable avec `--env`
- **Régions** (Region-A, Region-B, Local, etc.) - Filtrable avec `--region`
- **Types de serveurs** (Public, Private, Services, etc.) - Filtrable avec `--type`
- **Serveurs** individuels avec leurs alias SSH

### Navigation dans l'interface

#### Écran de sélection des fichiers
- **↑↓** : Naviguer dans la liste des fichiers/dossiers
- **Espace** : Sélectionner/désélectionner un fichier
- **Entrée** : Entrer dans un dossier ou remonter au parent
- **h** : Aller au répertoire home
- **a** : Sélectionner tous les fichiers visibles
- **c** : Vider la sélection
- **Tab** : Passer à l'écran suivant (sélection des serveurs)
- **q** : Quitter l'application

#### Écran de sélection des serveurs
- **↑↓** : Naviguer dans l'arbre hiérarchique
- **→** ou **Entrée** : Déplier un nœud / Sélectionner un serveur
- **←** : Réduire un nœud ou remonter au parent
- **Espace** : Sélectionner/désélectionner un serveur
- **/** : Activer le mode recherche
- **a** : Sélectionner tous les serveurs visibles
- **c** : Vider la sélection de serveurs
- **Tab** : Passer à l'écran suivant (destination)

#### Mode recherche
- **Caractères** : Taper pour filtrer en temps réel
- **Backspace** : Effacer un caractère
- **Entrée** : Valider et sortir du mode recherche
- **Échap** : Annuler la recherche

#### Écran de destination
- **Caractères** : Taper le chemin de destination
- **Entrée** : Valider et passer au téléversement
- **Échap** : Revenir à l'écran précédent

#### Écran de progression
- **q** : Quitter après la fin des transferts
- **p** : Mettre en pause/reprendre (si supporté)

## Modes d'utilisation

### 1. Mode interface complète (recommandé)

Lancez l'application sans arguments pour accéder à l'interface complète :

```bash
xsshend
```

Cette interface vous guide à travers toutes les étapes :
1. Sélection des fichiers
2. Sélection des serveurs (interface hiérarchique)
3. Choix de la destination
4. Téléversement avec barre de progression

### 2. Mode interactif avec fichiers pré-sélectionnés

```bash
xsshend --interactive file1.txt file2.txt
```

ou

```bash
xsshend upload file1.txt file2.txt --interactive
```

Les fichiers sont pré-sélectionnés, vous n'avez qu'à choisir les serveurs et la destination.

### 3. Mode ligne de commande avec filtres

```bash
# Téléverser vers tous les serveurs de production
xsshend upload file.txt --env Production --dest /opt/app/

# Téléverser vers une région spécifique
xsshend upload *.log --env Staging --region Region-A --dest /var/log/

# Téléverser vers un type de serveurs
xsshend upload config.json --env Production --type Public --dest /etc/app/
```

### 4. Mode simulation (dry-run)

```bash
xsshend upload file.txt --env Production --dry-run
```

Simule le transfert sans effectuer de connexions réelles.

## Configuration des serveurs

Votre fichier de configuration se trouve dans `~/.ssh/hosts.json`. Voici un exemple de structure :

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
  }
}
```

## Liste des serveurs disponibles

```bash
# Lister tous les serveurs
xsshend list

# Filtrer par environnement
xsshend list --env Production
```

## Conseils d'utilisation

### Efficacité
- Utilisez la recherche (`/`) pour filtrer rapidement les serveurs dans les grandes infrastructures
- Utilisez `a` pour sélectionner tous les serveurs d'un environnement après filtrage
- Organisez vos serveurs par environnement → région → type pour une navigation optimale

### Sécurité
- Le fichier `hosts.json` peut contenir des informations sensibles, gardez-le sécurisé
- Testez toujours sur l'environnement de staging avant la production
- Utilisez le mode `--dry-run` pour vérifier vos sélections

### Productivité
- Créez des alias dans votre shell pour les commandes fréquentes
- Pré-sélectionnez les fichiers depuis la ligne de commande quand vous les connaissez
- Utilisez l'interface hiérarchique pour explorer et découvrir votre infrastructure

## Gestion robuste des erreurs et timeouts

### Serveurs déconnectés

xsshend gère gracieusement les serveurs inaccessibles ou déconnectés :

```bash
# Vérification préalable de connectivité (recommandé pour les grandes infrastructures)
xsshend --online-only

# Cette option :
# - Teste la connectivité SSH vers chaque serveur (timeout: 5s)
# - Filtre automatiquement les serveurs inaccessibles 
# - Affiche seulement les serveurs en ligne dans l'interface TUI
# - Évite les blocages pendant les transferts
```

### Timeouts et retry automatique

Les connexions SSH utilisent des timeouts configurés pour éviter les blocages :

- **Timeout de connexion TCP :** 5 secondes par défaut
- **Timeout du handshake SSH :** 5 secondes par défaut  
- **Nombre de tentatives :** 2 tentatives maximum par serveur
- **Délai entre tentatives :** 1 seconde

### Comportement en cas d'erreur

Quand un serveur devient inaccessible pendant les transferts :

1. **Erreur loggée** : L'erreur est enregistrée avec détails
2. **Continuation** : Les transferts vers les autres serveurs continuent
3. **Résumé final** : Affichage des serveurs réussis vs échoués
4. **Code de sortie** : Succès si au moins un serveur a réussi

Exemple de sortie d'erreur gracieuse :
```
❌ Upload échoué vers SERVER_DOWN : Timeout de connexion TCP
✅ Upload réussi vers SERVER_01 : 1,234,567 octets
✅ Upload réussi vers SERVER_02 : 1,234,567 octets

📊 Upload parallèle terminé : 2/3 serveurs réussis
⚠️ Serveurs échoués : SERVER_DOWN
```

### Debug et diagnostic

Pour diagnostiquer les problèmes de connexion :

```bash
# Mode debug complet
RUST_LOG=debug xsshend upload --env Production file.txt

# Test manuel de connectivité SSH
ssh -o ConnectTimeout=5 -o BatchMode=yes user@server.example.com exit

# Vérifier la configuration SSH locale
ssh -v user@server.example.com
```
