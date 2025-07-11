# Release Notes - xsshend v0.2.1

## 🔑 Nouvelles Fonctionnalités SSH

### Sélection de Clés SSH en CLI

Cette version résout le problème de sélection forcée de clés SSH en ajoutant des options CLI dédiées :

```bash
# Sélection interactive de la clé SSH
xsshend upload file.txt --ssh-key-interactive

# Spécifier une clé par nom
xsshend upload file.txt --ssh-key id_rsa

# Forcer la sélection automatique optimale
xsshend upload file.txt --ssh-key-auto
```

### Sélection Automatique Intelligente

- **Détection automatique** de toutes les clés SSH disponibles
- **Priorité intelligente** : Ed25519 > RSA > ECDSA > Autres
- **Messages informatifs** sur la clé utilisée
- **Proposition interactive** si plusieurs clés sont détectées

### Intégration Complète

- Support des clés spécifiées dans le **pool de connexions SSH**
- **Messages de débogage** détaillés pour identifier les problèmes d'authentification
- **Fallback gracieux** vers ssh-agent si aucune clé n'est spécifiée

## 🔧 Problèmes Résolus

### Avant (v0.2.0)
```log
🔐 Clé SSH ED25519 trouvée: /home/user/.ssh/id_ed25519
❌ Impossible de se connecter à server@host : Échec de l'authentification SSH
```

### Après (v0.2.1)
```log
🔑 Plusieurs clés SSH détectées. Sélection automatique en cours...
🔑 Clé sélectionnée automatiquement: id_rsa (RSA) - user@hostname
🔌 Tentative de connexion SSH vers user@host (alias: server)
✅ Connexion SSH établie avec server (user@host)
```

## 📚 Documentation Mise à Jour

- **README.md** : Section dédiée aux nouvelles options SSH
- **CHANGELOG.md** : Historique détaillé des modifications  
- **Aide CLI** : Documentation intégrée des nouvelles options

## 🚀 Comment Migrer

### Utilisation Simple
```bash
# Avant : clé forcée
xsshend upload file.txt

# Maintenant : sélection intelligente avec choix utilisateur
xsshend upload file.txt
# Affiche les clés détectées et propose la sélection interactive
```

### Utilisation Avancée
```bash
# Nouveau : contrôle total de la clé SSH
xsshend upload file.txt --ssh-key-interactive
xsshend upload file.txt --ssh-key company_rsa
```

## ⚡ Performance et Qualité

- **0 warning Clippy** après nettoyage du code
- **Intégration native** des clés SSH dans l'architecture
- **Messages utilisateur** améliorés et informatifs
- **Tests complets** de toutes les nouvelles fonctionnalités

---

**xsshend v0.2.1** - *Sélection SSH intelligente et contrôle utilisateur* 🔑
