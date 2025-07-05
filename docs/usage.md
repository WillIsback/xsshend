# Guide d'utilisation de xsshend

## Vue d'ensemble

xsshend est un outil de téléversement multi-SSH avec une interface utilisateur hiérarchique moderne. Il permet de sélectionner facilement des fichiers et des serveurs via une interface TUI intuitive et de transférer les fichiers en parallèle.

## Interface utilisateur hiérarchique

### Principe

L'interface organise vos serveurs en arbre hiérarchique :
- **Environnements** (Production, Staging, Development)
- **Régions** (Region-A, Region-B, Local, etc.)
- **Types de serveurs** (Public, Private, Services, etc.)
- **Serveurs** individuels avec leurs alias

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
