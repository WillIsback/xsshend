# Guide Complet du Lab xsshend

**Version** : v0.4.1  
**Date** : 18 octobre 2025

---

## 📋 Table des Matières

1. [Vue d'ensemble](#vue-densemble)
2. [Installation Rapide](#installation-rapide)
3. [Architecture](#architecture)
4. [Tests](#tests)
5. [Dépannage](#dépannage)
6. [Sécurité](#sécurité)
7. [FAQ](#faq)

---

## 🎯 Vue d'ensemble

Le lab xsshend est un **environnement de test Docker complet** pour valider xsshend en conditions réelles avant déploiement production.

### Caractéristiques

- ✅ **3 conteneurs** ArchLinux (1 master + 2 targets)
- ✅ **Réseau isolé** Docker bridge
- ✅ **Clés SSH pré-configurées** (RSA 4096 + Ed25519)
- ✅ **40+ tests automatisés** avec scripts de diagnostic
- ✅ **Documentation complète** intégrée

### Prérequis

- Docker 20.10+
- Docker Compose 1.29+
- ~2GB d'espace disque
- 10 minutes de setup

---

## 🚀 Installation Rapide

### Étape 1 : Cloner et Configurer

```bash
# Cloner le repository
git clone https://github.com/WillIsback/xsshend.git
cd xsshend

# Exécuter le setup
./scripts/lab-setup.sh
```

**Le script crée** :
- Clés SSH (RSA + Ed25519)
- Configuration `hosts.json`
- Dockerfiles pour master et targets
- `docker-compose.yml`

### Étape 2 : Démarrer l'Environnement

```bash
cd lab/
docker-compose up -d --build

# Attendre le démarrage
sleep 10

# Vérifier
docker-compose ps
# Devrait afficher 3 conteneurs "Up"
```

### Étape 3 : Vérifier l'Installation

```bash
cd ..

# Diagnostic rapide
./scripts/lab-diagnostic.sh

# Tests automatisés
./scripts/test-lab.sh
```

**Si tous les tests passent** : ✅ Environnement prêt !

### Étape 4 : Premier Test Manuel

```bash
# Accéder au conteneur master
docker exec -it xsshend_master bash

# Lister les serveurs
xsshend list

# Créer un fichier de test
echo "Hello xsshend!" > test.txt

# Upload (dry-run)
xsshend upload test.txt --env Test --dry-run

# Upload réel
xsshend upload test.txt --env Test --server-type RSA-Targets

# Vérifier
exit
docker exec xsshend_target1 cat /tmp/test.txt
docker exec xsshend_target2 cat /tmp/test.txt
```

---

## 🏗️ Architecture

### Schéma

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

### Composants

#### Master (xsshend_master)
- **OS** : ArchLinux
- **User** : master
- **Rôle** : Exécute xsshend
- **Clés SSH** : RSA + Ed25519
- **Config** : hosts.json avec 3 serveurs

#### Target1 & Target2 (xsshend_target1/2)
- **OS** : ArchLinux
- **User** : testuser
- **Rôle** : Serveurs SSH cibles
- **Clés autorisées** : RSA uniquement
- **SSH** : PubkeyAuth=yes, PasswordAuth=no
- **Ports hôte** : 2221 (target1), 2222 (target2)

### Clés SSH

| Clé | Type | Bits | Passphrase | Enregistrée | Usage |
|-----|------|------|------------|-------------|-------|
| `id_rsa` | RSA | 4096 | ❌ Non | target1, target2 | Tests réussis |
| `id_ed25519` | Ed25519 | 256 | ✅ Oui (`testpassphrase`) | ❌ Aucune | Tests d'échec |

### Configuration hosts.json

```json
{
  "Test": {
    "Lab": {
      "RSA-Targets": {
        "TARGET1": { "alias": "testuser@target1", "env": "TEST" },
        "TARGET2": { "alias": "testuser@target2", "env": "TEST" }
      },
      "ED25519-Targets": {
        "TARGET1_ED25519": { "alias": "testuser@target1", "env": "TEST" }
      }
    }
  }
}
```

---

## 🧪 Tests

### Tests Automatisés (test-lab.sh)

**Suite complète** : 40+ tests en 12 phases

```bash
./scripts/test-lab.sh
```

**Phases testées** :
1. ✅ Conteneurs Docker actifs
2. ✅ Installation xsshend
3. ✅ Clés SSH et permissions
4. ✅ Démons SSH actifs
5. ✅ Connectivité réseau (ping)
6. ✅ Connectivité SSH manuelle
7. ✅ Commande `xsshend list`
8. ✅ Upload dry-run
9. ✅ Upload réel (RSA-Targets)
10. ✅ Vérification fichiers uploadés
11. ✅ Upload multi-fichiers
12. ✅ Logs SSH

**Sortie** : Rapport coloré avec compteurs (✓ PASSED / ✗ FAILED)

### Tests Manuels Essentiels

#### Test 1 : Liste des Serveurs
```bash
docker exec xsshend_master xsshend list
# Attendu : 3 serveurs (TARGET1, TARGET2, TARGET1_ED25519)
```

#### Test 2 : Upload Simple
```bash
docker exec xsshend_master bash -c "echo 'Test' > /tmp/test.txt"
docker exec xsshend_master xsshend upload /tmp/test.txt --env Test --server-type RSA-Targets
# Attendu : Upload réussi vers target1 et target2
```

#### Test 3 : Vérification
```bash
docker exec xsshend_target1 cat /tmp/test.txt
docker exec xsshend_target2 cat /tmp/test.txt
# Attendu : "Test" affiché
```

#### Test 4 : Multi-fichiers
```bash
docker exec xsshend_master bash -c "for i in {1..3}; do echo 'File \$i' > /tmp/file\$i.txt; done"
docker exec xsshend_master xsshend upload /tmp/file{1..3}.txt --server-type RSA-Targets
docker exec xsshend_target1 ls /tmp/file*.txt
# Attendu : 3 fichiers listés
```

#### Test 5 : Filtres
```bash
# Par environnement
docker exec xsshend_master xsshend upload test.txt --env Test --dry-run

# Par type de serveur
docker exec xsshend_master xsshend upload test.txt --server-type RSA-Targets --dry-run

# Par région
docker exec xsshend_master xsshend upload test.txt --region Lab --dry-run
```

#### Test 6 : Gestion d'Erreurs
```bash
# Fichier inexistant
docker exec xsshend_master xsshend upload /tmp/nonexistent.txt --env Test
# Attendu : Erreur claire

# Clé non autorisée (Ed25519)
docker exec xsshend_master bash -c "mv ~/.ssh/id_rsa ~/.ssh/id_rsa.bak"
docker exec xsshend_master xsshend upload test.txt --server-type ED25519-Targets
docker exec xsshend_master bash -c "mv ~/.ssh/id_rsa.bak ~/.ssh/id_rsa"
# Attendu : Échec d'authentification

# Serveur down
docker stop xsshend_target2
docker exec xsshend_master xsshend upload test.txt --server-type RSA-Targets
docker start xsshend_target2
# Attendu : Succès target1, échec target2
```

### Scénarios Avancés

#### Performance (fichier 10MB)
```bash
docker exec xsshend_master dd if=/dev/urandom of=/tmp/large.bin bs=1M count=10
time docker exec xsshend_master xsshend upload /tmp/large.bin --server-type RSA-Targets
docker exec xsshend_target1 ls -lh /tmp/large.bin
# Vérifier : 10M uploadé
```

#### Destination Personnalisée
```bash
docker exec xsshend_target1 mkdir -p /home/testuser/custom
docker exec xsshend_master xsshend upload test.txt --server-type RSA-Targets --dest /home/testuser/custom/
docker exec xsshend_target1 ls /home/testuser/custom/
```

---

## 🔧 Dépannage

### Diagnostic Rapide

```bash
# Script de diagnostic complet
./scripts/lab-diagnostic.sh

# Sauvegarder le rapport
./scripts/lab-diagnostic.sh > diagnostic_$(date +%Y%m%d_%H%M%S).txt
```

### Problèmes Courants

#### 1. Conteneurs ne démarrent pas

**Symptôme** :
```bash
docker-compose ps
# Montre : Exit 1 ou Exited
```

**Solution** :
```bash
# Voir les logs
docker-compose logs master
docker-compose logs target1

# Reconstruire
docker-compose down -v
docker-compose up -d --build --force-recreate
```

#### 2. SSH ne fonctionne pas

**Diagnostic** :
```bash
# Vérifier sshd
docker exec xsshend_target1 pgrep sshd
# Si vide, redémarrer :
docker restart xsshend_target1
sleep 5

# Tester connectivité
docker exec xsshend_master ping -c 2 target1

# Tester SSH avec verbose
docker exec xsshend_master ssh -vvv -i ~/.ssh/id_rsa testuser@target1
```

**Solutions** :
```bash
# Régénérer host keys
docker exec xsshend_target1 ssh-keygen -A
docker restart xsshend_target1

# Corriger permissions
docker exec xsshend_target1 chown -R testuser:testuser /home/testuser/.ssh
docker exec xsshend_target1 chmod 700 /home/testuser/.ssh
docker exec xsshend_target1 chmod 600 /home/testuser/.ssh/authorized_keys
```

#### 3. xsshend ne trouve pas les clés

**Solution** :
```bash
# Vérifier permissions
docker exec xsshend_master ls -la ~/.ssh/
docker exec xsshend_master stat -c '%a' ~/.ssh/id_rsa

# Corriger
docker exec xsshend_master chmod 600 ~/.ssh/id_rsa ~/.ssh/id_ed25519
docker exec xsshend_master chmod 644 ~/.ssh/*.pub ~/.ssh/hosts.json
```

#### 4. Upload échoue

**Diagnostic** :
```bash
# Vérifier destination
docker exec xsshend_target1 ls -ld /tmp
# Attendu : drwxrwxrwt

# Tester écriture manuelle
docker exec xsshend_master ssh -i ~/.ssh/id_rsa testuser@target1 "echo 'test' > /tmp/manual.txt"
```

#### 5. Logs SSH absents

**Solution** :
```bash
# Utiliser journalctl
docker exec xsshend_target1 journalctl -u sshd -n 50 --no-pager

# Ou dmesg
docker exec xsshend_target1 dmesg | grep -i ssh

# Ou logs Docker
docker logs xsshend_target1 -f
```

### Nettoyage Complet

```bash
# Arrêter et supprimer tout
docker-compose down -v
docker rmi $(docker images | grep xsshend-lab | awk '{print $3}')
docker network prune -f

# Recommencer depuis zéro
./scripts/lab-setup.sh
cd lab/
docker-compose up -d --build
```

### Commandes Utiles

```bash
# Statut des conteneurs
docker-compose ps
docker stats xsshend_master xsshend_target1 xsshend_target2

# Logs en temps réel
docker-compose logs -f
docker-compose logs -f master

# Accès shell
docker exec -it xsshend_master bash
docker exec -it xsshend_target1 bash

# Nettoyer fichiers de test
docker exec xsshend_master rm -f /tmp/test*.txt /tmp/file*.txt
docker exec xsshend_target1 rm -f /tmp/test*.txt /tmp/file*.txt
docker exec xsshend_target2 rm -f /tmp/test*.txt /tmp/file*.txt

# Redémarrage rapide
docker-compose restart
```

---

## 🔒 Sécurité

### ⚠️ Environnement de TEST Uniquement

**NE PAS utiliser en production** :
- ❌ Clés SSH générées localement (non sécurisées)
- ❌ Mots de passe en clair dans Dockerfiles
- ❌ Configuration SSH permissive
- ❌ Réseau partagé entre conteneurs

### Limitation RUSTSEC-2023-0071

xsshend v0.4.1 a une **limitation de sécurité connue** :
- **Vulnérabilité** : Marvin Attack dans `rsa 0.9.8`
- **Origine** : Dépendance transitive (xsshend → russh → rsa)
- **Correction** : ❌ Pas disponible (RustCrypto travaille sur v0.10)

**Dans le lab** :
- ✅ **Aucun impact** (réseau Docker isolé, localhost)
- ✅ Pas d'attaquant potentiel
- ✅ Données de test uniquement

**En production** :
- ⚠️ **Utiliser clés Ed25519** (recommandé, non affectées)
- ⚠️ Déployer sur réseaux de confiance uniquement
- ⚠️ Éviter WiFi public, réseaux non sécurisés

**Documentation complète** : [SECURITY.md](../SECURITY.md)

### Bonnes Pratiques

#### Dans le Lab
```bash
# ✅ OK : Tests avec RSA
xsshend upload test.txt --env Test

# ✅ OK : Générer Ed25519 pour production
ssh-keygen -t ed25519 -C "prod@example.com"
```

#### Pour Production
```bash
# ✅ Utiliser Ed25519
ssh-keygen -t ed25519 -C "user@production"

# ✅ Réseaux de confiance
# - VPN d'entreprise
# - Réseau interne
# - Connexions chiffrées point-à-point

# ❌ Éviter
# - WiFi public
# - Réseaux non sécurisés
# - Environnements compromis
```

---

## ❓ FAQ

### Questions Générales

**Q: Combien de temps prend le setup ?**  
A: ~10 minutes (5 min build Docker + 5 min installation xsshend)

**Q: Peut-on réutiliser les clés existantes ?**  
A: Le script écrase les clés existantes. Sauvegardez-les avant si nécessaire.

**Q: Fonctionne sur Windows/Mac ?**  
A: Oui, avec Docker Desktop. Scripts testés sur Linux/macOS.

**Q: Peut-on tester une version locale de xsshend ?**  
A: Oui :
```bash
# Compiler localement
cargo build --release

# Copier dans le conteneur
docker cp target/release/xsshend xsshend_master:/home/master/.cargo/bin/

# Tester
docker exec xsshend_master xsshend --version
```

### Problèmes Fréquents

**Q: "Permission denied (publickey)"**  
A: Vérifier :
```bash
# Clés présentes ?
docker exec xsshend_master ls -la ~/.ssh/

# Permissions correctes ?
docker exec xsshend_master stat -c '%a' ~/.ssh/id_rsa
# Attendu : 600

# authorized_keys sur target ?
docker exec xsshend_target1 cat /home/testuser/.ssh/authorized_keys
```

**Q: "xsshend: command not found"**  
A: Vérifier :
```bash
# Installé ?
docker exec xsshend_master which xsshend

# Si absent, réinstaller
docker exec -u master xsshend_master cargo install xsshend
```

**Q: "Cannot connect to Docker daemon"**  
A: Démarrer Docker :
```bash
# Linux
sudo systemctl start docker

# macOS/Windows
# Ouvrir Docker Desktop
```

**Q: Les tests échouent après redémarrage**  
A: Attendre que SSH démarre :
```bash
docker-compose restart
sleep 10
./scripts/test-lab.sh
```

### Personnalisation

**Q: Changer les noms de conteneurs ?**  
A: Modifier `scripts/lab-setup.sh` et `docker-compose.yml`

**Q: Ajouter un 3ème target ?**  
A: Dans `docker-compose.yml` :
```yaml
target3:
  build:
    context: .
    dockerfile: Dockerfile.target
  container_name: xsshend_target3
  hostname: target3
  networks:
    - xsshend_net
  volumes:
    - ./authorized_keys:/home/testuser/.ssh/authorized_keys:ro
  ports:
    - "2223:22"
```

**Q: Utiliser une autre distribution ?**  
A: Modifier `Dockerfile.master` et `Dockerfile.target` (Ubuntu, Debian, etc.)

### Performance

**Q: Upload est lent**  
A: Normal pour gros fichiers. Vérifier :
```bash
# Ressources Docker
docker stats

# Vitesse réseau conteneurs
docker exec xsshend_master sh -c "dd if=/dev/zero bs=1M count=100 | ssh -i ~/.ssh/id_rsa testuser@target1 'cat > /dev/null'"
```

**Q: Build Docker est long**  
A: Normal (compilation Rust). Accélérer :
```bash
# Utiliser cache
docker-compose build

# Build parallèle
docker-compose build --parallel
```

---

## 📚 Ressources

### Documentation

- **README Principal** : [../README.md](../README.md)
- **Sécurité** : [SECURITY.md](../SECURITY.md)
- **Configuration** : [configuration.md](configuration.md)
- **Clés SSH** : [ssh-keys.md](ssh-keys.md)

### Scripts

- **Setup** : `scripts/lab-setup.sh` - Configuration initiale
- **Tests** : `scripts/test-lab.sh` - Suite automatisée (40+ tests)
- **Diagnostic** : `scripts/lab-diagnostic.sh` - Vérification environnement
- **Fonctions communes** : `scripts/lab-common.sh` - Utilitaires réutilisables

### Support

- **Issues** : https://github.com/WillIsback/xsshend/issues
- **Discussions** : https://github.com/WillIsback/xsshend/discussions
- **Site web** : https://willisback.github.io/xsshend/

---

## ✅ Checklist Avant Production

Avant d'utiliser xsshend en production, valider :

- [ ] **Tous les tests lab passent** (40/40)
- [ ] **SECURITY.md lu et compris**
- [ ] **Décision sur type de clés** (Ed25519 recommandé)
- [ ] **Clés production générées** (différentes du lab)
- [ ] **hosts.json production préparé**
- [ ] **Réseau de confiance identifié** (VPN, interne)
- [ ] **Tests sur échantillon de serveurs** (non prod)
- [ ] **Plan de rollback défini**
- [ ] **Équipe formée**
- [ ] **Monitoring en place**

---

**Version du guide** : 1.0  
**Dernière mise à jour** : 18 octobre 2025  
**Compatible avec** : xsshend v0.4.1+

---

**Documentation créée par** : @WillIsback  
**Support** : [GitHub Issues](https://github.com/WillIsback/xsshend/issues)
