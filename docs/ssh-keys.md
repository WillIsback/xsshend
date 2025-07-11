# 🔑 Gestion des clés SSH

> Guide complet pour la gestion, sélection et configuration des clés SSH dans xsshend

## 🎯 Vue d'ensemble

xsshend offre une gestion complète des clés SSH avec détection automatique, sélection interactive et génération automatique. Le système simplifie l'authentification SSH en gérant automatiquement les différents types de clés.

## 🔄 Configuration automatique

### Première utilisation

Au **premier lancement**, xsshend :

1. **Détecte** l'absence du répertoire `~/.ssh/`
2. **Crée automatiquement** le répertoire et les fichiers nécessaires
3. **Vérifie** la présence de clés SSH existantes
4. **Propose la génération** d'une clé Ed25519 si aucune n'est trouvée
5. **Guide l'utilisateur** dans le processus

### Génération automatique

```bash
❌ Aucune clé SSH trouvée dans ~/.ssh/
🔑 Pour utiliser xsshend, vous avez besoin d'une clé SSH privée.
Voulez-vous générer une nouvelle clé SSH Ed25519 ? (o/N): o
Entrez votre adresse email (optionnel): user@example.com
🔄 Génération de la clé SSH en cours...
✅ Clé SSH Ed25519 générée avec succès: ~/.ssh/id_ed25519
```

### Structure créée

```
~/.ssh/
├── hosts.json          # Configuration serveurs
├── id_ed25519          # Clé privée SSH
├── id_ed25519.pub      # Clé publique SSH
└── config              # Configuration SSH (optionnel)
```

## 🎯 Sélection des clés

### Options disponibles

#### 1. Sélection interactive (`--ssh-key-interactive`)

```bash
xsshend upload file.txt --ssh-key-interactive
```

**Interface de sélection :**
```
🔑 Sélection de la clé SSH...
🔑 Plusieurs clés SSH disponibles:
? Sélectionnez la clé SSH à utiliser ›
❯ id_rsa (RSA) - william.derue@gmail.com
  id_rsa_ci_cd (RSA) - ci-cd@smartdoc.com
  company_key (OpenSSH) - company-admin
  runpod_ed25519 (Ed25519) - william.dernier@gmail.com
```

#### 2. Spécification par nom (`--ssh-key <nom>`)

```bash
xsshend upload file.txt --ssh-key id_rsa
xsshend upload file.txt --ssh-key company_key
```

#### 3. Sélection automatique forcée (`--ssh-key-auto`)

```bash
xsshend upload file.txt --ssh-key-auto
```

**Priorité :** Ed25519 > RSA > ECDSA > Autres

#### 4. Comportement par défaut

```bash
xsshend upload file.txt
```

Sélection intelligente avec information à l'utilisateur.

## 🔍 Détection des clés

### Clés standard recherchées

- `id_ed25519` - Clé Ed25519 (recommandée)
- `id_rsa` - Clé RSA standard
- `id_ecdsa` - Clé ECDSA
- `id_dsa` - Clé DSA (obsolète)

### Clés personnalisées

- Tous les fichiers dans `~/.ssh/` contenant "PRIVATE KEY"
- Détection automatique du type de clé
- Support des commentaires dans les clés publiques

## 🛡️ Sécurité

### Meilleures pratiques

- ✅ **Algorithme Ed25519** : Cryptographie moderne et rapide
- ✅ **Permissions strictes** : Clé privée avec permissions 600
- ✅ **ssh-agent** : Support transparent des passphrases
- ✅ **Rotation régulière** : Renouvellement des clés

### Recommandations

```bash
# Ajouter une passphrase à une clé existante
ssh-keygen -p -f ~/.ssh/id_ed25519

# Vérifier les permissions
ls -la ~/.ssh/
chmod 600 ~/.ssh/id_*
chmod 644 ~/.ssh/*.pub
```

## 🔧 Exemples pratiques

### Développement

```bash
# Sélection interactive pour différents environnements
xsshend upload app.jar --env Production --ssh-key-interactive
```

### Automation/CI-CD

```bash
# Spécification directe dans les scripts
xsshend upload deploy.tar.gz --ssh-key ci_cd_key --env Production
```

### Utilisation quotidienne

```bash
# Laisser xsshend choisir intelligemment
xsshend upload file.txt --env Staging
# Affiche : "🔑 Clé sélectionnée automatiquement: company_key (RSA)"
```

## 🔍 Diagnostic et dépannage

### Voir les clés détectées

```bash
# Mode dry-run pour voir la sélection sans transfert
xsshend upload file.txt --ssh-key-interactive --dry-run
```

### Messages d'erreur courants

**Clé non trouvée :**
```
❌ Clé SSH 'inexistante_key' non trouvée
```
**Solution :** Vérifiez le nom avec `ls ~/.ssh/`

**Aucune clé disponible :**
```
🔑 Aucune clé SSH trouvée, utilisation de ssh-agent
```
**Solution :** Configurez ssh-agent ou créez des clés SSH

**Authentification échouée :**
```
❌ Échec de l'authentification SSH pour l'utilisateur 'user'
```
**Solution :** Vérifiez que la clé publique est déployée sur le serveur

## 🔄 Migration et compatibilité

### Depuis l'ancienne version

```bash
# Ancienne commande (fonctionne toujours)
xsshend upload file.txt

# Nouveau comportement : sélection intelligente + information

# Pour reproduire l'ancien comportement
xsshend upload file.txt --ssh-key id_ed25519

# Pour avoir le contrôle complet
xsshend upload file.txt --ssh-key-interactive
```

### Types de clés supportés

- ✅ Ed25519 (recommandé)
- ✅ RSA (2048+ bits)
- ✅ ECDSA
- ⚠️ DSA (obsolète, support limité)

## 📚 Références

- [Guide d'utilisation principal](usage.md)
- [Configuration des serveurs](configuration.md)
- [Workflow CI/CD](cicd.md)
