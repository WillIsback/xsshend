# Guide de sélection des clés SSH

## Vue d'ensemble

xsshend offre plusieurs méthodes pour gérer et sélectionner les clés SSH pour les connexions, résolvant ainsi le problème de sélection automatique incorrecte des clés.

## Problème résolu

**Avant** : xsshend sélectionnait automatiquement la première clé trouvée (souvent Ed25519) même si ce n'était pas la bonne clé pour les serveurs cibles.

**Maintenant** : Contrôle complet sur la sélection des clés SSH avec plusieurs options.

## Options disponibles

### 1. Sélection interactive (`--ssh-key-interactive`)

```bash
xsshend upload file.txt --ssh-key-interactive
```

**Comportement** :
- Affiche un menu avec toutes les clés SSH disponibles
- Permet de choisir manuellement la clé à utiliser
- Recommandé quand vous avez plusieurs clés pour différents environnements

**Exemple de sortie** :
```
🔑 Sélection de la clé SSH...
🔑 Plusieurs clés SSH disponibles:
? Sélectionnez la clé SSH à utiliser ›
❯ id_rsa (RSA) - william.derue@gmail.com
  id_rsa_ci_cd (RSA) - ci-cd@smartdoc.com
  company_key (OpenSSH) - company-admin
  runpod_ed25519 (Ed25519) - william.dernier@gmail.com
```

### 2. Spécification par nom (`--ssh-key <nom>`)

```bash
xsshend upload file.txt --ssh-key id_rsa
xsshend upload file.txt --ssh-key company_key
```

**Comportement** :
- Utilise directement la clé spécifiée par son nom de fichier (sans extension)
- Erreur si la clé n'existe pas
- Idéal pour les scripts automatisés

### 3. Sélection automatique forcée (`--ssh-key-auto`)

```bash
xsshend upload file.txt --ssh-key-auto
```

**Comportement** :
- Force la sélection de la "meilleure" clé selon la priorité définie
- Priorité : Ed25519 > RSA > ECDSA > Autres
- Pratique quand vous voulez toujours utiliser la clé la plus sécurisée

### 4. Comportement par défaut (aucune option)

```bash
xsshend upload file.txt
```

**Comportement** :
- Détecte toutes les clés disponibles
- Si une seule clé : l'utilise automatiquement
- Si plusieurs clés : sélectionne automatiquement la meilleure et informe l'utilisateur
- Suggère l'utilisation de `--ssh-key-interactive` pour un choix manuel

## Détection des clés

xsshend cherche automatiquement les clés dans `~/.ssh/` :

### Clés standard recherchées
- `id_ed25519` - Clé Ed25519 (recommandée)
- `id_rsa` - Clé RSA standard
- `id_ecdsa` - Clé ECDSA
- `id_dsa` - Clé DSA (obsolète)

### Clés personnalisées
- Tous les fichiers dans `~/.ssh/` contenant "PRIVATE KEY"
- Détection automatique du type de clé
- Support des commentaires dans les clés publiques

## Exemples d'utilisation pratique

### Cas 1 : Environnement de développement

```bash
# Utilisation interactive pour choisir selon l'environnement
xsshend upload app.jar --env Production --ssh-key-interactive
```

### Cas 2 : Automation/CI-CD

```bash
# Spécification directe dans les scripts
xsshend upload deploy.tar.gz --ssh-key ci_cd_key --env Production
```

### Cas 3 : Utilisation quotidienne

```bash
# Laisser xsshend choisir intelligemment
xsshend upload file.txt --env Staging
# Affiche : "🔑 Clé sélectionnée automatiquement: company_key (RSA)"
```

### Cas 4 : Résolution de problèmes

```bash
# Forcer l'utilisation de la meilleure clé technique
xsshend upload file.txt --ssh-key-auto --env Production
```

## Diagnostic et dépannage

### Voir les clés détectées

```bash
# Mode dry-run pour voir la sélection sans transfert
xsshend upload file.txt --ssh-key-interactive --dry-run
```

### Messages d'erreur courants

**Clé non trouvée** :
```
❌ Clé SSH 'inexistante_key' non trouvée
```
Solution : Vérifiez le nom de la clé avec `ls ~/.ssh/`

**Aucune clé disponible** :
```
🔑 Aucune clé SSH trouvée, utilisation de ssh-agent
```
Solution : Configurez ssh-agent ou créez des clés SSH

**Authentification échouée** :
```
❌ Échec de l'authentification SSH pour l'utilisateur 'user'
```
Solution : Vérifiez que la clé publique est déployée sur le serveur cible

## Compatibilité

### ssh-agent
- Utilisé automatiquement en fallback
- Compatible avec toutes les méthodes de sélection
- Gestion transparente des passphrases

### Types de clés supportés
- ✅ Ed25519 (recommandé)
- ✅ RSA (2048+ bits)
- ✅ ECDSA
- ⚠️ DSA (obsolète, support limité)

## Sécurité

### Recommandations
1. **Utilisez Ed25519** pour les nouvelles clés
2. **Spécifiez explicitement** les clés pour les environnements sensibles
3. **Évitez ssh-agent** sur les serveurs partagés
4. **Utilisez des passphrases** pour les clés critiques

### Audit des clés
```bash
# Voir toutes les clés détectées
xsshend upload dummy --ssh-key-interactive --dry-run
```

## Migration depuis l'ancienne version

Si vous utilisiez xsshend avant cette mise à jour :

### Comportement identique
```bash
# Ancienne commande (fonctionne toujours)
xsshend upload file.txt

# Nouveau comportement : sélection intelligente + information
```

### Contrôle explicite
```bash
# Pour reproduire l'ancien comportement de sélection de la première clé
xsshend upload file.txt --ssh-key id_ed25519

# Pour avoir le contrôle complet
xsshend upload file.txt --ssh-key-interactive
```

## Exemples avancés

### Script avec gestion d'erreur
```bash
#!/bin/bash

# Essayer avec une clé spécifique
if xsshend upload app.tar.gz --ssh-key prod_key --env Production; then
    echo "✅ Déploiement réussi avec la clé de production"
else
    echo "❌ Échec avec la clé de production, essai avec sélection interactive"
    xsshend upload app.tar.gz --ssh-key-interactive --env Production
fi
```

### Sélection conditionnelle
```bash
#!/bin/bash

ENV=${1:-Development}

case $ENV in
    Production)
        xsshend upload app.jar --ssh-key prod_rsa --env Production
        ;;
    Staging)
        xsshend upload app.jar --ssh-key staging_key --env Staging
        ;;
    *)
        xsshend upload app.jar --ssh-key-auto --env Development
        ;;
esac
```
