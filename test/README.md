# Module de Test xsshend avec Multipass

Ce module permet de tester xsshend dans un environnement contrôlé avec des VMs Ubuntu créées via Multipass.

## 🎯 Objectifs

- **Tests en conditions réelles** : VMs Ubuntu avec SSH configuré
- **Environnement isolé** : Aucun impact sur les serveurs de production
- **Tests automatisés** : Suite complète de tests d'intégration
- **Reproductibilité** : Environnement de test identique pour tous

## 📁 Structure du Module

```
test/
├── README.md                    # Ce fichier
├── test-vms.sh                 # Gestionnaire de VMs Multipass
├── generate-test-files.sh      # Générateur de fichiers de test
├── run-integration-tests.sh    # Suite de tests d'intégration
├── multipass/
│   └── cloud-init.yaml         # Configuration cloud-init des VMs
├── configs/
│   └── test-hosts.json         # Configuration hosts.json de test
├── data/                       # Fichiers de test (généré)
│   ├── simple.txt
│   ├── config.json
│   ├── deploy.sh
│   ├── index.html
│   ├── small-1kb.txt
│   ├── medium-100kb.txt
│   ├── large-1mb.txt
│   ├── test-data.csv
│   └── archive-test.tar.gz
└── .ssh/                       # Clés SSH de test (généré)
    ├── test_key
    ├── test_key.pub
    ├── deploy_key
    ├── deploy_key.pub
    ├── api_key
    └── api_key.pub
```

## 🚀 Installation et Setup

### 1. Installer Multipass

```bash
# Ubuntu/Debian
sudo snap install multipass --classic

# macOS  
brew install multipass

# Windows
# Télécharger depuis https://multipass.run/
```

### 2. Vérifier l'installation

```bash
multipass version
multipass list
```

### 3. Setup du module de test

```bash
cd test/

# 1. Génération des clés SSH de test
./test-vms.sh generate-keys

# 2. Mise à jour cloud-init avec les vraies clés
./test-vms.sh update-cloud-init

# 3. Lancement des VMs de test
./test-vms.sh launch-all

# 4. Génération de la configuration hosts.json
./test-vms.sh generate-config

# 5. Test des connexions SSH
./test-vms.sh test-ssh
```

## 🖥️ VMs de Test

Le module crée 5 VMs Ubuntu 22.04 :

| VM | Rôle | Resources | Utilisateurs |
|----|------|-----------|--------------|
| `xsshend-dev` | Développement | 1 CPU, 1GB RAM, 5GB | xsshend-test |
| `xsshend-staging` | Staging | 1 CPU, 1GB RAM, 5GB | deploy |
| `xsshend-prod-web` | Production Web | 2 CPU, 2GB RAM, 10GB | deploy |
| `xsshend-prod-api` | Production API | 2 CPU, 2GB RAM, 10GB | api |
| `xsshend-prod-db` | Production DB | 1 CPU, 2GB RAM, 8GB | xsshend-test |

### Répertoires de destination dans les VMs

- `/opt/uploads` - Téléversements généraux
- `/var/www/uploads` - Fichiers web
- `/home/deploy/files` - Fichiers de déploiement  
- `/home/api/uploads` - Fichiers API
- `/tmp/xsshend-test` - Tests temporaires

## 🧪 Tests Automatisés

### Exécution complète

```bash
# Génération des fichiers de test
./generate-test-files.sh

# Exécution de tous les tests
./run-integration-tests.sh
```

### Tests inclus

1. **CLI et aide** - Vérification interface ligne de commande
2. **Configuration** - Chargement hosts.json et filtrage
3. **Dry-run** - Mode simulation sans transfert
4. **Upload simple** - Transfert d'un fichier unique
5. **Upload multiple** - Transfert de plusieurs fichiers
6. **Gros fichier** - Test des barres de progression
7. **Parallèle** - Transfert vers plusieurs serveurs
8. **Gestion d'erreurs** - Tests de robustesse
9. **Performance** - Tests de stress

### Exemple de sortie

```
🧪 Suite de tests d'intégration xsshend
Date: 2025-07-05 15:30:00
Version: v0.1.0

ℹ️  Vérification des prérequis...
✅ Tous les prérequis sont satisfaits

ℹ️  Test 1: Interface CLI et aide
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Commande --help fonctionne
✅ Commande --version fonctionne

[... autres tests ...]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 RÉSUMÉ DES TESTS xsshend
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Total:   9 tests
  ✅ Réussis: 9
  ❌ Échecs:  0

✅ 🎉 TOUS LES TESTS SONT PASSÉS!
  xsshend v0.1.0 est prêt pour la production!
```

## 🛠️ Gestion des VMs

### Commandes principales

```bash
# Lister les VMs
./test-vms.sh list

# Arrêter toutes les VMs
./test-vms.sh stop-all

# Démarrer toutes les VMs
./test-vms.sh start-all

# Détruire toutes les VMs (avec confirmation)
./test-vms.sh destroy-all

# Aide complète
./test-vms.sh help
```

### Connexion manuelle aux VMs

```bash
# Via Multipass
multipass shell xsshend-dev

# Via SSH avec clés de test
ssh -i test/.ssh/test_key xsshend-test@<VM-IP>
```

### Récupération des IPs

```bash
# Toutes les VMs
multipass list

# VM spécifique
multipass info xsshend-dev
```

## 📋 Tests Manuels

### Test simple

```bash
# Compilation
cargo build

# Test dry-run
./target/debug/xsshend upload test/data/simple.txt --env Development --dry-run

# Test réel
./target/debug/xsshend upload test/data/simple.txt --env Development
```

### Test avec configuration personnalisée

```bash
# Utiliser la config de test
export HOME=/path/to/xsshend/test/configs
./target/debug/xsshend list --env Production
```

## 🔧 Personnalisation

### Modifier les VMs

Éditez les variables dans `test-vms.sh` :

```bash
declare -A VMS=(
    ["mon-vm"]="ubuntu-22.04 --cpus 4 --mem 4G --disk 20G"
)
```

### Ajouter des utilisateurs

Modifiez `multipass/cloud-init.yaml` :

```yaml
users:
  - name: nouveau-user
    groups: sudo
    ssh_authorized_keys:
      - ssh-ed25519 AAAAC3... votre-clé
```

### Créer des fichiers de test personnalisés

Modifiez `generate-test-files.sh` ou créez vos propres fichiers dans `test/data/`.

## 🧹 Nettoyage

### Supprimer tous les tests

```bash
./test-vms.sh destroy-all
rm -rf test/data test/.ssh test/configs
```

### Nettoyage partiel

```bash
# Nettoyer uniquement les données
rm -rf test/data/*

# Régénérer les fichiers de test
./generate-test-files.sh
```

## ⚠️ Limitations et Notes

1. **Ressources** : Les VMs consomment RAM et CPU
2. **Espace disque** : ~50GB requis pour toutes les VMs
3. **Réseau** : Les VMs utilisent le réseau bridge de Multipass
4. **Clés SSH** : Générées automatiquement, pas pour production
5. **Données** : Dossier `/test` exclu du git (`.gitignore`)

## 🐛 Dépannage

### VM ne démarre pas

```bash
# Vérifier les logs
multipass info xsshend-dev

# Relancer avec cloud-init verbeux
multipass launch ubuntu-22.04 --name debug-vm --cloud-init multipass/cloud-init.yaml
```

### SSH échoue

```bash
# Vérifier les clés
ls -la test/.ssh/

# Tester manuellement
ssh -v -i test/.ssh/test_key xsshend-test@<IP>
```

### Tests échouent

```bash
# Mode verbeux
RUST_LOG=debug ./run-integration-tests.sh

# Test spécifique 
./target/debug/xsshend upload test/data/simple.txt --env Development -v
```

## 🎯 Cas d'Usage

1. **Développement** : Tester les nouvelles fonctionnalités
2. **CI/CD** : Intégration dans les pipelines 
3. **Démonstration** : Montrer xsshend en action
4. **Benchmarking** : Mesurer les performances
5. **Formation** : Apprendre SSH/SFTP et transferts parallèles

---

**🚀 Prêt à tester xsshend dans un environnement sûr et contrôlé !**
