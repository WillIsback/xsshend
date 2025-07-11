# Release Notes - xsshend v0.2.2

## 🔧 Résolution Majeure des Problèmes de Permissions SSH

### 🎯 Problème Résolu

**Avant v0.2.2** : Les transferts multi-cibles échouaient souvent avec des erreurs de permissions comme "Impossible de créer le fichier distant" car :
- Les chemins `~/` et `$HOME` n'étaient pas expansés côté serveur
- Aucune vérification préalable des permissions d'écriture
- Pas de fallback vers des répertoires accessibles
- Chaque utilisateur SSH tentait d'écrire dans le même chemin literal

**Maintenant v0.2.2** : Transferts multi-cibles robustes avec gestion intelligente des permissions.

## 🚀 Nouvelles Fonctionnalités

### Expansion Automatique des Chemins SSH
```bash
# Ces chemins sont maintenant correctement expansés côté serveur :
xsshend upload file.txt --dest "~/uploads/file.txt"     # → /home/user1/uploads/file.txt
xsshend upload file.txt --dest "$HOME/data/file.txt"    # → /home/user2/data/file.txt
```

### Détection et Fallback des Permissions
- ✅ **Test préalable** des permissions d'écriture dans le répertoire cible
- ✅ **Fallback automatique** vers des répertoires accessibles (`/tmp/`, `/home/user/`, etc.)
- ✅ **Messages informatifs** sur les changements de destination
- ✅ **Création récursive** des répertoires parents manquants

### Nouvelles Méthodes SSH
```rust
// Expansion des chemins distants
pub fn expand_remote_path(&self, remote_path: &str) -> Result<String>
pub fn get_remote_home_directory(&self) -> Result<String>

// Test et recherche de répertoires accessibles
pub fn test_write_permissions(&self, remote_dir: &str) -> Result<bool>
pub fn find_accessible_directory(&self, preferred_dir: &str) -> Result<String>
```

## 🔍 Logs Améliorés

### Exemple de Logs Informatifs
```log
🔍 Chemin expansé: ~/upload/test.txt -> /home/user1/upload/test.txt
🔍 Recherche d'un répertoire accessible, préférence: /home/user1/upload
🔍 Test du répertoire: /home/user1/upload
✅ Permissions d'écriture confirmées pour: /home/user1/upload
📤 Début upload: test.txt -> /home/user1/upload/test.txt
✅ Fichier uploadé vers: /home/user1/upload/test.txt
```

### En Cas de Permissions Insuffisantes
```log
❌ Pas de permissions d'écriture dans: /home/user1/restricted
🔍 Test du répertoire: /tmp
✅ Permissions d'écriture confirmées pour: /tmp
⚠️  Utilisation du répertoire alternatif: /tmp (répertoire original /home/user1/restricted inaccessible)
⚠️  Changement du chemin de destination: /home/user1/restricted/test.txt -> /tmp/test.txt (permissions)
✅ Fichier uploadé vers: /tmp/test.txt (adapté pour permissions)
```

## 🎭 Scénarios de Fallback

### Ordre de Priorité des Répertoires
1. **Répertoire demandé** (ex: `/home/user1/upload/`)
2. **Sous-répertoire xsshend** (ex: `/home/user1/upload/xsshend/`)
3. **Répertoire temporaire** (`/tmp/`)
4. **Répertoire temporaire utilisateur** (`/tmp/{username}`)
5. **Répertoire home utilisateur** (`/home/{username}`)
6. **Répertoire home macOS** (`/Users/{username}`)
7. **Répertoire var** (`/var/tmp`)

## 🔧 Améliorations Techniques

### Gestion Robuste des Erreurs SSH
- ✅ **Timeouts configurables** pour les opérations SSH
- ✅ **Gestion des erreurs de concurrence** (répertoires créés simultanément)
- ✅ **Validation préalable** des fichiers locaux et distants
- ✅ **Nettoyage automatique** des fichiers de test temporaires

### Compatibilité Multi-Plateforme
- ✅ **Linux** : `/home/{user}`, `/tmp/`
- ✅ **macOS** : `/Users/{user}`, `/tmp/`
- ✅ **Serveurs Unix** : Détection automatique du `$HOME`

## 📊 Impact sur les Transferts Multi-Cibles

### Avant v0.2.2
```log
❌ Transfert 1/4 : Échec (permissions)
❌ Transfert 2/4 : Échec (permissions) 
✅ Transfert 3/4 : Succès
❌ Transfert 4/4 : Échec (permissions)
```

### Après v0.2.2
```log
✅ Transfert 1/4 : Succès (/home/user1/upload/file.txt)
✅ Transfert 2/4 : Succès (/tmp/file.txt - fallback)
✅ Transfert 3/4 : Succès (/home/user3/upload/file.txt)
✅ Transfert 4/4 : Succès (/home/user4/file.txt - fallback)
```

## 🛠️ Installation et Mise à Jour

```bash
# Installation depuis crates.io
cargo install xsshend

# Mise à jour vers v0.2.2
cargo install xsshend --force

# Vérification de la version
xsshend --version  # doit afficher "xsshend 0.2.2"
```

## 📚 Documentation

- **Expansion de chemins** : Support natif de `~/` et `$HOME`
- **Gestion des permissions** : Fallback automatique et informatif
- **Logs détaillés** : Mode `--verbose` pour le débogage
- **Compatibilité** : Fonctionne avec tous les serveurs SSH/SFTP standards

---

**Release v0.2.2** résout définitivement les problèmes de permissions multi-cibles et introduit une gestion robuste des chemins SSH distants.

*Les utilisateurs ayant des configurations multi-serveurs avec différents utilisateurs SSH verront une amélioration drastique du taux de succès des transferts.*
