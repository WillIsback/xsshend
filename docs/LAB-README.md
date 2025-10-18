# xsshend Lab - Environnement de Test Docker

## 📋 Vue d'ensemble

Ce lab fournit un environnement de test complet pour xsshend utilisant Docker et Docker Compose. Il simule un environnement réel avec :

- **1 conteneur master** : ArchLinux avec xsshend installé
- **2 conteneurs target** : ArchLinux avec SSH configuré
- **Réseau isolé** : Communication sécurisée entre les conteneurs
- **Clés SSH multiples** : Tests avec RSA et Ed25519

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                Docker Network: xsshend_net                   │
│                                                              │
│  ┌──────────────┐          ┌──────────────┐  ┌──────────────┐│
│  │   master     │          │   target1    │  │   target2    ││
│  │              │   SSH    │              │  │              ││
│  │  xsshend     │─────────▶│  testuser    │  │  testuser    ││
│  │  (Arch)      │          │  (Arch)      │  │  (Arch)      ││
│  │              │          │              │  │              ││
│  │ RSA+ED25519  │          │  RSA only    │  │  RSA only    ││
│  └──────────────┘          └──────────────┘  └──────────────┘│
│       │                         :22               :22         │
└───────┼─────────────────────────┴────────────────┴───────────┘
        │
   localhost:2221          localhost:2222
```

## 🚀 Démarrage Rapide

### Prérequis

- Docker
- Docker Compose
- ~2GB d'espace disque

### Installation

```bash
# 1. Cloner le repository xsshend
git clone https://github.com/WillIsback/xsshend.git
cd xsshend

# 2. Créer le répertoire lab
mkdir -p lab
cd lab

# 3. Copier le script de setup
cp ../scripts/lab-setup.sh .

# 4. Exécuter le setup
chmod +x lab-setup.sh
./lab-setup.sh

# 5. Démarrer l'environnement
docker-compose up -d --build

# 6. Attendre que les conteneurs démarrent (10 secondes)
sleep 10

# 7. Vérifier le statut
docker-compose ps
```

### Accès aux Conteneurs

```bash
# Master (xsshend)
docker exec -it xsshend_master bash

# Target 1
docker exec -it xsshend_target1 bash

# Target 2
docker exec -it xsshend_target2 bash
```

## 🧪 Tests

### Tests Automatisés

```bash
# Depuis le répertoire lab/
chmod +x ../scripts/test-lab.sh
../scripts/test-lab.sh
```

Le script de test automatisé vérifie :
- ✅ Statut des conteneurs
- ✅ Installation de xsshend
- ✅ Configuration des clés SSH
- ✅ Démons SSH actifs
- ✅ Connectivité SSH manuelle
- ✅ Commande `xsshend list`
- ✅ Upload en mode dry-run
- ✅ Upload réel vers les targets
- ✅ Vérification des fichiers uploadés
- ✅ Upload multi-fichiers
- ✅ Logs SSH d'authentification

### Tests Manuels

Consultez le guide complet : [docs/LAB-TESTING-GUIDE.md](../docs/LAB-TESTING-GUIDE.md)

#### Test Rapide

```bash
# Accéder au master
docker exec -it xsshend_master bash

# Vérifier la version
xsshend --version

# Lister les serveurs
xsshend list

# Créer un fichier de test
echo "Hello from xsshend!" > test.txt

# Tester l'upload (dry-run)
xsshend upload test.txt --env Test --dry-run

# Upload réel
xsshend upload test.txt --env Test --server-type RSA-Targets

# Vérifier sur target1
exit
docker exec -it xsshend_target1 cat /tmp/test.txt
```

## 🔑 Configuration des Clés SSH

Le lab crée automatiquement deux paires de clés :

### Clé RSA (SANS passphrase)

- **Fichier** : `ssh_keys/id_rsa`
- **Type** : RSA 4096 bits
- **Passphrase** : Aucune
- **Enregistrée sur** : target1, target2
- **Usage** : Tests de connexion réussie

### Clé Ed25519 (AVEC passphrase)

- **Fichier** : `ssh_keys/id_ed25519`
- **Type** : Ed25519
- **Passphrase** : `testpassphrase`
- **Enregistrée sur** : Aucune cible
- **Usage** : Tests d'échec et mode interactif

## 📁 Structure des Fichiers

```
lab/
├── docker-compose.yml          # Configuration Docker Compose
├── Dockerfile.master           # Image pour master (xsshend)
├── Dockerfile.target           # Image pour targets (SSH)
├── lab-setup.sh               # Script de configuration
├── authorized_keys            # Clé publique RSA pour targets
└── ssh_keys/
    ├── id_rsa                 # Clé privée RSA
    ├── id_rsa.pub             # Clé publique RSA
    ├── id_ed25519             # Clé privée Ed25519 (avec passphrase)
    ├── id_ed25519.pub         # Clé publique Ed25519
    └── hosts.json             # Configuration xsshend
```

## 📝 Configuration hosts.json

```json
{
  "Test": {
    "Lab": {
      "RSA-Targets": {
        "TARGET1": {
          "alias": "testuser@target1",
          "env": "TEST"
        },
        "TARGET2": {
          "alias": "testuser@target2",
          "env": "TEST"
        }
      },
      "ED25519-Targets": {
        "TARGET1_ED25519": {
          "alias": "testuser@target1",
          "env": "TEST"
        }
      }
    }
  }
}
```

## 🔧 Dépannage

### Les conteneurs ne démarrent pas

```bash
# Vérifier les logs
docker-compose logs

# Reconstruire les images
docker-compose down
docker-compose up -d --build --force-recreate
```

### SSH ne fonctionne pas

```bash
# Vérifier que sshd tourne
docker exec xsshend_target1 ps aux | grep sshd

# Vérifier les logs SSH
docker exec xsshend_target1 journalctl -u sshd -n 50

# Tester manuellement depuis le master
docker exec xsshend_master ssh -vvv -i ~/.ssh/id_rsa testuser@target1
```

### xsshend ne trouve pas les clés

```bash
# Vérifier les permissions
docker exec xsshend_master ls -la ~/.ssh/

# Expected:
# -rw------- id_rsa
# -rw-r--r-- id_rsa.pub
# -rw------- id_ed25519
# -rw-r--r-- id_ed25519.pub
# -rw-r--r-- hosts.json

# Corriger les permissions si nécessaire
docker exec xsshend_master chmod 600 ~/.ssh/id_rsa ~/.ssh/id_ed25519
docker exec xsshend_master chmod 644 ~/.ssh/*.pub ~/.ssh/hosts.json
```

### Les fichiers ne s'uploadent pas

```bash
# Vérifier la connectivité réseau
docker exec xsshend_master ping -c 3 target1
docker exec xsshend_master ping -c 3 target2

# Tester SSH manuellement
docker exec xsshend_master ssh -i ~/.ssh/id_rsa testuser@target1 "hostname"

# Vérifier les permissions sur la destination
docker exec xsshend_target1 ls -ld /tmp
```

### Accès depuis l'hôte

Vous pouvez aussi SSH depuis votre machine hôte :

```bash
# Vers target1 (port 2221)
ssh -i lab/ssh_keys/id_rsa -p 2221 testuser@localhost

# Vers target2 (port 2222)
ssh -i lab/ssh_keys/id_rsa -p 2222 testuser@localhost
```

## 🧹 Nettoyage

### Nettoyage partiel (conserver les images)

```bash
docker-compose down
```

### Nettoyage complet (supprimer tout)

```bash
# Arrêter et supprimer les conteneurs + volumes
docker-compose down -v

# Supprimer les images
docker rmi xsshend-lab-master xsshend-lab-target xsshend-lab-target1 xsshend-lab-target2 2>/dev/null || true

# Supprimer les fichiers générés
rm -rf ssh_keys/ authorized_keys
```

## 📚 Documentation Complète

- **Guide de test détaillé** : [docs/LAB-TESTING-GUIDE.md](../docs/LAB-TESTING-GUIDE.md)
- **Documentation xsshend** : [README.md](../README.md)
- **Documentation sécurité** : [SECURITY.md](../SECURITY.md)
- **RUSTSEC-2023-0071** : [docs/RUSTSEC-2023-0071-EXPLANATION.md](../docs/RUSTSEC-2023-0071-EXPLANATION.md)

## 🔒 Notes de Sécurité

### Environnement de Test Uniquement

⚠️ **IMPORTANT** : Cet environnement est conçu UNIQUEMENT pour les tests et le développement.

**NE PAS utiliser en production car :**
- Les clés SSH sont générées localement (non sécurisées)
- Les mots de passe sont en clair dans les Dockerfiles
- SSH est configuré pour accepter les connexions sans vérification d'hôte
- Les conteneurs partagent le même réseau

### Limitation RUSTSEC-2023-0071

xsshend v0.4.1 a une **limitation de sécurité connue** liée à RUSTSEC-2023-0071 (Marvin Attack dans `rsa 0.9.8`).

**Dans ce lab de test**, cette limitation n'a **AUCUN impact** car :
- Réseau isolé Docker (pas d'accès externe)
- Environnement contrôlé localhost
- Pas de données sensibles
- Aucun attaquant potentiel avec accès timing

**Pour la production** :
- Utiliser des clés Ed25519 (recommandé)
- Déployer uniquement sur réseaux de confiance
- Consulter [SECURITY.md](../SECURITY.md) pour les détails

## 🎯 Cas d'Usage

### Développement

```bash
# Modifier le code xsshend localement
cd /chemin/vers/xsshend
cargo build --release

# Copier le binaire dans le conteneur master
docker cp target/release/xsshend xsshend_master:/home/master/.cargo/bin/

# Tester immédiatement
docker exec -it xsshend_master xsshend --version
```

### Tests de Régression

```bash
# Lancer la suite de tests automatisés
./scripts/test-lab.sh

# Vérifier que tous les tests passent
# Si des tests échouent, consulter les logs
```

### Démo

```bash
# Démarrer l'environnement
docker-compose up -d

# Faire une démo live
docker exec -it xsshend_master bash
xsshend list
xsshend upload demo.txt --env Test

# Montrer les résultats
docker exec xsshend_target1 cat /tmp/demo.txt
```

## 📊 Métriques et Logs

### Voir les logs en temps réel

```bash
# Tous les conteneurs
docker-compose logs -f

# Master uniquement
docker-compose logs -f master

# Targets uniquement
docker-compose logs -f target1 target2
```

### Statistiques des conteneurs

```bash
# Utilisation CPU/Mémoire
docker stats xsshend_master xsshend_target1 xsshend_target2

# Taille des images
docker images | grep xsshend
```

## 🤝 Contribution

Pour contribuer à l'amélioration de ce lab :

1. Fork le repository
2. Créer une branche pour vos modifications
3. Tester vos changements avec le lab
4. Soumettre une Pull Request

## 📞 Support

- **Issues GitHub** : https://github.com/WillIsback/xsshend/issues
- **Discussions** : https://github.com/WillIsback/xsshend/discussions
- **Documentation** : https://willisback.github.io/xsshend/

---

**Version du lab** : 1.0  
**Compatible avec** : xsshend v0.4.1+  
**Dernière mise à jour** : 18 octobre 2025
