# Guide de Test et Validation xsshend Lab

## 🧪 Configuration du Lab

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Docker Network: xsshend_net              │
│                                                              │
│  ┌──────────────┐          ┌──────────────┐  ┌──────────────┐│
│  │   master     │          │   target1    │  │   target2    ││
│  │              │   SSH    │              │  │              ││
│  │  xsshend     │─────────▶│  testuser    │  │  testuser    ││
│  │  (ArchLinux) │          │  (ArchLinux) │  │  (ArchLinux) ││
│  │              │          │              │  │              ││
│  │ RSA + ED25519│          │  RSA only    │  │  RSA only    ││
│  └──────────────┘          └──────────────┘  └──────────────┘│
│       │                         :22               :22         │
└───────┼─────────────────────────┴────────────────┴───────────┘
        │
   Host :2221                Host :2222
```

### Clés SSH

| Clé | Type | Passphrase | Enregistrée sur | Usage |
|-----|------|------------|-----------------|-------|
| `id_rsa` | RSA 4096 | ❌ NON | target1, target2 | Test authentification réussie |
| `id_ed25519` | Ed25519 | ✅ OUI (`testpassphrase`) | ❌ AUCUNE | Test échec + mode interactif |

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

## 🚀 Procédure de Test

### 1. Démarrage du Lab

```bash
# Construire et démarrer les conteneurs
docker-compose up -d --build

# Vérifier que tous les conteneurs sont actifs
docker-compose ps

# Expected output:
# NAME                IMAGE                    STATUS
# xsshend_master      xsshend-lab-master      Up
# xsshend_target1     xsshend-lab-target1     Up
# xsshend_target2     xsshend-lab-target2     Up

# Attendre 10 secondes que SSH démarre
sleep 10
```

### 2. Vérification de l'Environnement

#### Sur master (xsshend)

```bash
# Accéder au conteneur master
docker exec -it xsshend_master bash

# Vérifier l'installation de xsshend
xsshend --version

# Vérifier les clés SSH
ls -la ~/.ssh/
# Expected:
# -rw------- id_rsa          (clé privée RSA)
# -rw-r--r-- id_rsa.pub      (clé publique RSA)
# -rw------- id_ed25519      (clé privée Ed25519 AVEC passphrase)
# -rw-r--r-- id_ed25519.pub  (clé publique Ed25519)
# -rw-r--r-- hosts.json      (configuration)

# Vérifier la configuration
cat ~/.ssh/hosts.json
```

#### Sur target1 et target2

```bash
# Dans un autre terminal - Vérifier target1
docker exec -it xsshend_target1 bash

# Vérifier que sshd tourne
ps aux | grep sshd
# Expected: /usr/sbin/sshd -D

# Vérifier la configuration SSH
cat /etc/ssh/sshd_config | grep -E "(PubkeyAuthentication|PasswordAuthentication|PermitRootLogin)"
# Expected:
# PermitRootLogin no
# PubkeyAuthentication yes
# PasswordAuthentication no

# Vérifier les clés autorisées pour testuser
cat /home/testuser/.ssh/authorized_keys
# Expected: contenu de id_rsa.pub (RSA key)

# Vérifier les logs SSH
tail -50 /var/log/auth.log 2>/dev/null || journalctl -u sshd -n 50 2>/dev/null || echo "Logs non disponibles"

# Répéter pour target2
docker exec -it xsshend_target2 bash
# ... mêmes vérifications
```

### 3. Tests de Connectivité Manuelle

Avant de tester xsshend, vérifier que SSH fonctionne manuellement :

```bash
# Sur master
docker exec -it xsshend_master bash

# Test 1: Connexion avec clé RSA vers target1
ssh -i ~/.ssh/id_rsa -o StrictHostKeyChecking=no testuser@target1 "hostname && whoami"
# Expected: target1
#           testuser

# Test 2: Connexion avec clé RSA vers target2
ssh -i ~/.ssh/id_rsa -o StrictHostKeyChecking=no testuser@target2 "hostname && whoami"
# Expected: target2
#           testuser

# Test 3: Échec avec clé Ed25519 (non enregistrée)
ssh -i ~/.ssh/id_ed25519 -o StrictHostKeyChecking=no testuser@target1 "hostname"
# Expected: Permission denied (publickey)
```

### 4. Tests xsshend

#### Test A: Lister les serveurs

```bash
# Sur master
xsshend list

# Expected output:
# Liste des cibles SSH:
# 
# Environment: Test
#   Region: Lab
#     Type: RSA-Targets
#       - TARGET1 (testuser@target1) [TEST]
#       - TARGET2 (testuser@target2) [TEST]
#     Type: ED25519-Targets
#       - TARGET1_ED25519 (testuser@target1) [TEST]
```

#### Test B: Upload avec clé RSA (devrait réussir)

```bash
# Créer un fichier de test
echo "Test xsshend v0.4.1 - RSA key" > test_rsa.txt

# Upload vers tous les serveurs
xsshend upload test_rsa.txt --env Test --dry-run

# Expected output:
# 🚀 xsshend - Téléversement Multi-SSH
# 🔍 Validation des fichiers...
# 🔐 Clé SSH RSA trouvée: /home/master/.ssh/id_rsa
# 
# Mode dry-run activé - Simulation uniquement
# 
# Fichiers à téléverser:
#   - test_rsa.txt (32 octets)
# 
# Cibles:
#   - TARGET1 (testuser@target1)
#   - TARGET2 (testuser@target2)
#   - TARGET1_ED25519 (testuser@target1)
# 
# Destination: /tmp/
# Simulation terminée avec succès

# Upload réel
xsshend upload test_rsa.txt --env Test

# Expected: Succès pour TARGET1 et TARGET2
#           Échec pour TARGET1_ED25519 (clé Ed25519 non enregistrée)
```

#### Test C: Upload avec filtre RSA-Targets

```bash
# Upload uniquement vers les cibles RSA
xsshend upload test_rsa.txt --env Test --server-type RSA-Targets

# Expected: Succès pour TARGET1 et TARGET2 uniquement
```

#### Test D: Vérification des fichiers sur les targets

```bash
# Vérifier sur target1
docker exec -it xsshend_target1 cat /tmp/test_rsa.txt
# Expected: Test xsshend v0.4.1 - RSA key

# Vérifier sur target2
docker exec -it xsshend_target2 cat /tmp/test_rsa.txt
# Expected: Test xsshend v0.4.1 - RSA key
```

#### Test E: Test avec clé Ed25519 (devrait échouer)

```bash
# Supprimer temporairement la clé RSA pour forcer l'utilisation de Ed25519
mv ~/.ssh/id_rsa ~/.ssh/id_rsa.backup
mv ~/.ssh/id_rsa.pub ~/.ssh/id_rsa.pub.backup

echo "Test Ed25519 authentication" > test_ed25519.txt

# Tenter l'upload
xsshend upload test_ed25519.txt --env Test --server-type ED25519-Targets

# Expected: Échec - Permission denied
# (La clé Ed25519 n'est pas enregistrée sur target1)

# Restaurer la clé RSA
mv ~/.ssh/id_rsa.backup ~/.ssh/id_rsa
mv ~/.ssh/id_rsa.pub.backup ~/.ssh/id_rsa.pub
```

### 5. Tests de Logs et Diagnostics

#### Analyser les logs sur les targets

```bash
# Sur target1
docker exec -it xsshend_target1 bash

# Logs d'authentification
tail -50 /var/log/auth.log 2>/dev/null || \
journalctl -u sshd -n 50 2>/dev/null || \
dmesg | grep -i ssh

# Rechercher les tentatives d'authentification
grep "Accepted publickey" /var/log/auth.log 2>/dev/null || \
journalctl -u sshd | grep "Accepted publickey"

# Rechercher les échecs
grep "Failed" /var/log/auth.log 2>/dev/null || \
journalctl -u sshd | grep "Failed"
```

#### Analyser les connexions actives

```bash
# Sur target1
docker exec -it xsshend_target1 bash

# Processus SSH actifs
ps aux | grep sshd | grep -v grep

# Connexions réseau SSH
netstat -tnlp | grep :22 || ss -tnlp | grep :22

# Sessions SSH actives
who
```

### 6. Tests de Performance

```bash
# Sur master
# Créer un fichier de test plus volumineux
dd if=/dev/urandom of=largefile.bin bs=1M count=10

# Upload avec timing
time xsshend upload largefile.bin --env Test --server-type RSA-Targets

# Vérifier la taille sur les targets
docker exec -it xsshend_target1 ls -lh /tmp/largefile.bin
docker exec -it xsshend_target2 ls -lh /tmp/largefile.bin

# Expected: 10M sur chaque target
```

## 🔍 Scénarios de Test Avancés

### Test 1: Multi-fichiers

```bash
# Créer plusieurs fichiers
for i in {1..5}; do
  echo "File $i content" > file$i.txt
done

# Upload de tous les fichiers
xsshend upload file*.txt --env Test --server-type RSA-Targets

# Vérification
docker exec -it xsshend_target1 ls -la /tmp/file*.txt
```

### Test 2: Destination personnalisée

```bash
# Créer un répertoire de test sur les targets
docker exec -it xsshend_target1 mkdir -p /home/testuser/uploads
docker exec -it xsshend_target2 mkdir -p /home/testuser/uploads

echo "Custom destination test" > custom_dest.txt

xsshend upload custom_dest.txt \
  --env Test \
  --server-type RSA-Targets \
  --dest /home/testuser/uploads/

# Vérification
docker exec -it xsshend_target1 cat /home/testuser/uploads/custom_dest.txt
```

### Test 3: Gestion d'erreurs

```bash
# Test avec fichier inexistant
xsshend upload nonexistent.txt --env Test
# Expected: Erreur claire

# Test avec destination interdite
xsshend upload test.txt --env Test --dest /root/
# Expected: Permission denied

# Test avec serveur down
docker stop xsshend_target2
xsshend upload test.txt --env Test --server-type RSA-Targets
# Expected: Succès pour target1, échec pour target2 avec message clair

# Redémarrer target2
docker start xsshend_target2
sleep 5
```

## 📊 Résultats Attendus

### ✅ Tests Réussis

| Test | Description | Résultat Attendu |
|------|-------------|------------------|
| A | Lister serveurs | 3 serveurs affichés |
| B | Upload RSA dry-run | Simulation réussie |
| B | Upload RSA réel | Succès TARGET1/2, échec TARGET1_ED25519 |
| C | Upload filtré | Succès TARGET1/2 uniquement |
| D | Vérification fichiers | Fichiers présents sur targets |
| 1 | Multi-fichiers | 5 fichiers transférés |
| 2 | Destination custom | Fichiers dans /home/testuser/uploads/ |
| 3 | Gestion erreurs | Messages d'erreur clairs |

### ⚠️ Comportements Attendus

1. **Clé Ed25519 avec passphrase** : Devrait demander la passphrase en mode interactif
2. **Serveur down** : Échec gracieux avec message clair
3. **Permission denied** : Message explicite sur la cause
4. **Multi-clés** : Sélection automatique de la clé RSA

## 🧹 Nettoyage

```bash
# Arrêter et supprimer les conteneurs
docker-compose down -v

# Supprimer les images
docker rmi xsshend-lab-master xsshend-lab-target1 xsshend-lab-target2

# Nettoyer les fichiers générés
rm -rf ssh_keys authorized_keys
rm -f test*.txt file*.txt largefile.bin custom_dest.txt
```

## 📝 Rapport de Test

Après avoir exécuté tous les tests, documenter :

```markdown
### Test Report - xsshend v0.4.1

**Date**: 18 octobre 2025
**Environnement**: Docker Lab (ArchLinux)
**Testeur**: [Votre nom]

#### Résultats

- [x] Installation réussie
- [x] Clés SSH détectées correctement
- [x] Connexions SSH fonctionnelles
- [x] Upload avec clé RSA réussi
- [x] Filtrage par environnement/type fonctionne
- [x] Gestion d'erreurs robuste
- [ ] Mode interactif avec passphrase (à tester)

#### Problèmes Rencontrés

1. [Décrire tout problème]

#### Notes

- RUSTSEC-2023-0071: Limitation connue, documentée dans SECURITY.md
- Recommandation: Utiliser clés Ed25519 en production
```

---

**Version du guide**: 1.0  
**Dernière mise à jour**: 18 octobre 2025
