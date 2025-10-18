# Guide de Diagnostic - xsshend Lab

## 🔍 Diagnostic Rapide

Ce guide vous aide à identifier et résoudre les problèmes courants dans le lab xsshend.

## 📋 Checklist Rapide

Avant de chercher plus loin, vérifiez :

```bash
# 1. Conteneurs actifs
docker ps | grep xsshend
# Expected: 3 conteneurs (master, target1, target2)

# 2. Réseau Docker
docker network ls | grep xsshend
# Expected: xsshend_net

# 3. Volumes
docker volume ls | grep master_home
# Expected: master_home volume exists

# 4. Version xsshend
docker exec xsshend_master xsshend --version
# Expected: xsshend 0.4.1 ou supérieur

# 5. Clés SSH
docker exec xsshend_master ls -la ~/.ssh/
# Expected: id_rsa, id_ed25519, hosts.json
```

## 🚨 Problèmes Courants et Solutions

### 1. Conteneurs Ne Démarrent Pas

#### Symptôme
```bash
docker-compose ps
# Shows: Exit 1 or Exited
```

#### Diagnostic
```bash
# Voir les logs
docker-compose logs master
docker-compose logs target1
docker-compose logs target2

# Vérifier les erreurs de build
docker-compose build --no-cache master
```

#### Solutions

**A. Erreur de réseau**
```bash
# Recréer le réseau
docker-compose down
docker network prune -f
docker-compose up -d
```

**B. Problème de permissions**
```bash
# Vérifier les permissions du répertoire ssh_keys
ls -la ssh_keys/
# Si incorrectes:
chmod 600 ssh_keys/id_rsa ssh_keys/id_ed25519
chmod 644 ssh_keys/id_rsa.pub ssh_keys/id_ed25519.pub
chmod 644 ssh_keys/hosts.json
```

**C. Image corrompue**
```bash
# Reconstruire complètement
docker-compose down -v
docker rmi xsshend-lab-master xsshend-lab-target1 xsshend-lab-target2
docker-compose up -d --build --force-recreate
```

---

### 2. SSH Ne Fonctionne Pas

#### Symptôme
```bash
docker exec xsshend_master ssh -i ~/.ssh/id_rsa testuser@target1 "hostname"
# Returns: Connection refused or timeout
```

#### Diagnostic Étape par Étape

**Étape 1 : Vérifier que sshd tourne**
```bash
docker exec xsshend_target1 ps aux | grep sshd
# Expected: /usr/sbin/sshd -D (process en cours)

# Si absent, démarrer manuellement
docker exec xsshend_target1 /usr/sbin/sshd -D &
```

**Étape 2 : Vérifier la connectivité réseau**
```bash
# Ping depuis master vers target1
docker exec xsshend_master ping -c 3 target1
# Expected: 3 packets transmitted, 3 received

# Si échec, vérifier le réseau Docker
docker network inspect xsshend_net
```

**Étape 3 : Vérifier la configuration SSH**
```bash
# Voir la config sshd
docker exec xsshend_target1 cat /etc/ssh/sshd_config | grep -E "(PubkeyAuth|PasswordAuth|PermitRoot)"
# Expected:
# PermitRootLogin no
# PubkeyAuthentication yes
# PasswordAuthentication no

# Vérifier les host keys
docker exec xsshend_target1 ls -la /etc/ssh/ssh_host_*
# Expected: plusieurs fichiers ssh_host_*_key
```

**Étape 4 : Vérifier les clés autorisées**
```bash
# Voir les clés autorisées sur target1
docker exec xsshend_target1 cat /home/testuser/.ssh/authorized_keys
# Expected: contenu de id_rsa.pub

# Comparer avec la clé publique master
docker exec xsshend_master cat ~/.ssh/id_rsa.pub

# Vérifier les permissions
docker exec xsshend_target1 ls -ld /home/testuser/.ssh
# Expected: drwx------ (700)

docker exec xsshend_target1 ls -l /home/testuser/.ssh/authorized_keys
# Expected: -rw------- ou -rw-r--r-- (600 ou 644)
```

**Étape 5 : Logs SSH détaillés**
```bash
# Tenter une connexion avec verbose
docker exec xsshend_master ssh -vvv -i ~/.ssh/id_rsa -o StrictHostKeyChecking=no testuser@target1

# Voir les logs sur le serveur (plusieurs méthodes)
docker exec xsshend_target1 journalctl -u sshd -n 50 --no-pager
# ou
docker exec xsshend_target1 tail -50 /var/log/auth.log
# ou
docker exec xsshend_target1 dmesg | grep -i ssh
```

#### Solutions Courantes

**A. sshd ne démarre pas**
```bash
# Régénérer les host keys
docker exec xsshend_target1 ssh-keygen -A

# Redémarrer le conteneur
docker restart xsshend_target1

# Attendre 5 secondes
sleep 5

# Vérifier
docker exec xsshend_target1 pgrep sshd
```

**B. Clé non reconnue**
```bash
# Recréer les clés autorisées
docker cp ssh_keys/id_rsa.pub authorized_keys
chmod 600 authorized_keys

# Redémarrer les targets
docker restart xsshend_target1 xsshend_target2
```

**C. Problème de permissions**
```bash
# Corriger sur target1
docker exec xsshend_target1 chown -R testuser:testuser /home/testuser/.ssh
docker exec xsshend_target1 chmod 700 /home/testuser/.ssh
docker exec xsshend_target1 chmod 600 /home/testuser/.ssh/authorized_keys

# Répéter pour target2
docker exec xsshend_target2 chown -R testuser:testuser /home/testuser/.ssh
docker exec xsshend_target2 chmod 700 /home/testuser/.ssh
docker exec xsshend_target2 chmod 600 /home/testuser/.ssh/authorized_keys
```

---

### 3. xsshend Ne Trouve Pas les Clés

#### Symptôme
```bash
docker exec xsshend_master xsshend list
# Returns: Erreur - Aucune clé SSH trouvée
```

#### Diagnostic
```bash
# Vérifier les clés
docker exec xsshend_master ls -la ~/.ssh/

# Vérifier les permissions
docker exec xsshend_master stat -c '%a %n' ~/.ssh/id_rsa
# Expected: 600 /home/master/.ssh/id_rsa

docker exec xsshend_master stat -c '%a %n' ~/.ssh/id_ed25519
# Expected: 600 /home/master/.ssh/id_ed25519
```

#### Solutions

**A. Permissions incorrectes**
```bash
docker exec xsshend_master chmod 600 ~/.ssh/id_rsa
docker exec xsshend_master chmod 600 ~/.ssh/id_ed25519
docker exec xsshend_master chmod 644 ~/.ssh/id_rsa.pub
docker exec xsshend_master chmod 644 ~/.ssh/id_ed25519.pub
```

**B. Clés absentes (recréer)**
```bash
# Arrêter le master
docker stop xsshend_master

# Recréer les clés localement
cd lab/
ssh-keygen -t rsa -b 4096 -f ssh_keys/id_rsa -N "" -C "xsshend_rsa_key"
ssh-keygen -t ed25519 -f ssh_keys/id_ed25519 -N "testpassphrase" -C "xsshend_ed25519_key"

# Redémarrer master
docker start xsshend_master

# Vérifier
docker exec xsshend_master ls -la ~/.ssh/
```

---

### 4. xsshend list Ne Montre Aucun Serveur

#### Symptôme
```bash
docker exec xsshend_master xsshend list
# Returns: Aucun serveur trouvé
```

#### Diagnostic
```bash
# Vérifier hosts.json
docker exec xsshend_master cat ~/.ssh/hosts.json

# Vérifier le format JSON
docker exec xsshend_master cat ~/.ssh/hosts.json | python -m json.tool
# ou
docker exec xsshend_master cat ~/.ssh/hosts.json | jq .
```

#### Solutions

**A. hosts.json absent**
```bash
# Recréer depuis le template
cat > ssh_keys/hosts.json << 'EOF'
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
EOF

# Redémarrer master
docker restart xsshend_master
```

**B. Format JSON invalide**
```bash
# Valider le JSON
docker exec xsshend_master cat ~/.ssh/hosts.json | python3 -c "import sys,json; json.load(sys.stdin); print('Valid JSON')"

# Si invalide, remplacer par la version correcte ci-dessus
```

---

### 5. Upload Échoue

#### Symptôme
```bash
docker exec xsshend_master xsshend upload test.txt --env Test
# Returns: Échec de téléversement
```

#### Diagnostic Complet

**Étape 1 : Vérifier que le fichier existe**
```bash
docker exec xsshend_master ls -lh test.txt
# ou
docker exec xsshend_master ls -lh /tmp/test.txt
```

**Étape 2 : Vérifier la connectivité SSH (voir section 2)**
```bash
docker exec xsshend_master ssh -i ~/.ssh/id_rsa testuser@target1 "hostname"
```

**Étape 3 : Vérifier les permissions destination**
```bash
# Vérifier /tmp sur targets
docker exec xsshend_target1 ls -ld /tmp
# Expected: drwxrwxrwt (1777)

# Tester l'écriture manuelle
docker exec xsshend_master ssh -i ~/.ssh/id_rsa testuser@target1 "echo 'test' > /tmp/test_manual.txt && cat /tmp/test_manual.txt"
```

**Étape 4 : Tester en verbose**
```bash
# Activer les logs de debug (si xsshend le supporte)
docker exec xsshend_master xsshend upload test.txt --env Test -vvv
```

#### Solutions

**A. Destination non accessible**
```bash
# Utiliser /tmp explicitement
docker exec xsshend_master xsshend upload test.txt --env Test --dest /tmp/

# Ou créer un répertoire accessible
docker exec xsshend_target1 mkdir -p /home/testuser/uploads
docker exec xsshend_target1 chown testuser:testuser /home/testuser/uploads
docker exec xsshend_master xsshend upload test.txt --env Test --dest /home/testuser/uploads/
```

**B. Fichier trop volumineux**
```bash
# Vérifier l'espace disque sur targets
docker exec xsshend_target1 df -h /tmp

# Si plein, nettoyer
docker exec xsshend_target1 rm -f /tmp/*.txt
```

---

### 6. Performances Lentes

#### Symptôme
```bash
time docker exec xsshend_master xsshend upload largefile.bin --env Test
# Returns: temps > 30 secondes pour 10MB
```

#### Diagnostic
```bash
# Vérifier les ressources CPU/Mémoire
docker stats xsshend_master xsshend_target1 xsshend_target2

# Vérifier le réseau Docker
docker network inspect xsshend_net | grep -A 10 "Options"

# Tester la vitesse réseau entre conteneurs
docker exec xsshend_master sh -c "dd if=/dev/zero bs=1M count=100 | ssh -i ~/.ssh/id_rsa testuser@target1 'cat > /dev/null'"
```

#### Solutions

**A. Réseau lent**
```bash
# Utiliser le driver host (si possible)
# Modifier docker-compose.yml:
# network_mode: host

# Ou recréer le réseau
docker-compose down
docker network rm xsshend_net
docker network create xsshend_net --driver bridge --opt com.docker.network.driver.mtu=1500
docker-compose up -d
```

**B. Ressources limitées**
```bash
# Augmenter les limites Docker
# Modifier docker-compose.yml pour chaque service:
# deploy:
#   resources:
#     limits:
#       cpus: '2.0'
#       memory: 2G
```

---

### 7. Logs SSH Absents

#### Symptôme
```bash
docker exec xsshend_target1 journalctl -u sshd
# Returns: No entries
```

#### Solutions

**A. Utiliser dmesg**
```bash
docker exec xsshend_target1 dmesg | grep -i ssh
```

**B. Activer le logging verbose**
```bash
# Modifier sshd_config pour plus de logs
docker exec xsshend_target1 bash -c "echo 'LogLevel DEBUG3' >> /etc/ssh/sshd_config"
docker restart xsshend_target1

# Tenter une connexion
docker exec xsshend_master ssh -i ~/.ssh/id_rsa testuser@target1 "hostname"

# Voir les nouveaux logs
docker exec xsshend_target1 dmesg | tail -50
```

**C. Utiliser Docker logs**
```bash
docker logs xsshend_target1 -f
# Garder ce terminal ouvert et tester SSH dans un autre terminal
```

---

## 🔧 Outils de Diagnostic

### Script de Diagnostic Complet

```bash
#!/bin/bash
# diagnostic.sh - Script de diagnostic complet

echo "=== xsshend Lab Diagnostic ==="
echo ""

echo "1. Conteneurs Docker"
docker ps -a | grep xsshend
echo ""

echo "2. Réseau Docker"
docker network ls | grep xsshend
docker network inspect xsshend_net | grep -E "(Name|Subnet|Gateway)"
echo ""

echo "3. Version xsshend"
docker exec xsshend_master xsshend --version 2>&1 || echo "ERREUR: xsshend non accessible"
echo ""

echo "4. Clés SSH Master"
docker exec xsshend_master ls -la ~/.ssh/ 2>&1 || echo "ERREUR: Clés non accessibles"
echo ""

echo "5. SSH Daemons"
echo "Target1:"
docker exec xsshend_target1 pgrep -a sshd || echo "ERREUR: sshd non actif"
echo "Target2:"
docker exec xsshend_target2 pgrep -a sshd || echo "ERREUR: sshd non actif"
echo ""

echo "6. Connectivité Réseau"
echo "Master -> Target1:"
docker exec xsshend_master ping -c 2 target1 2>&1 | grep "packets"
echo "Master -> Target2:"
docker exec xsshend_master ping -c 2 target2 2>&1 | grep "packets"
echo ""

echo "7. Connectivité SSH"
echo "Master -> Target1 (RSA):"
docker exec xsshend_master ssh -i ~/.ssh/id_rsa -o StrictHostKeyChecking=no -o ConnectTimeout=5 testuser@target1 "hostname" 2>&1 || echo "ÉCHEC"
echo "Master -> Target2 (RSA):"
docker exec xsshend_master ssh -i ~/.ssh/id_rsa -o StrictHostKeyChecking=no -o ConnectTimeout=5 testuser@target2 "hostname" 2>&1 || echo "ÉCHEC"
echo ""

echo "8. Configuration hosts.json"
docker exec xsshend_master cat ~/.ssh/hosts.json 2>&1 | head -10
echo ""

echo "9. Serveurs listés par xsshend"
docker exec xsshend_master xsshend list 2>&1 || echo "ERREUR"
echo ""

echo "10. Espace disque"
docker exec xsshend_master df -h /tmp | tail -1
docker exec xsshend_target1 df -h /tmp | tail -1
docker exec xsshend_target2 df -h /tmp | tail -1
echo ""

echo "=== Diagnostic Terminé ==="
```

Sauvegarder ce script :
```bash
# Créer le script
cat > diagnostic.sh << 'EOF'
[coller le contenu ci-dessus]
EOF

# Rendre exécutable
chmod +x diagnostic.sh

# Exécuter
./diagnostic.sh > diagnostic_output.txt 2>&1
cat diagnostic_output.txt
```

### Tests de Connectivité Rapides

```bash
# Test 1-ligne complet
docker exec xsshend_master sh -c "echo 'Quick test' > /tmp/quick.txt && xsshend upload /tmp/quick.txt --env Test --server-type RSA-Targets && echo 'SUCCESS' || echo 'FAILED'"

# Vérification 1-ligne
docker exec xsshend_target1 cat /tmp/quick.txt && docker exec xsshend_target2 cat /tmp/quick.txt
```

---

## 📞 Obtenir de l'Aide

Si les solutions ci-dessus ne fonctionnent pas :

### 1. Collecter les Informations

```bash
# Créer un rapport complet
cat > issue_report.txt << EOF
xsshend Lab Issue Report
========================

Date: $(date)
Docker Version: $(docker --version)
Docker Compose Version: $(docker-compose --version)

Container Status:
$(docker ps -a | grep xsshend)

Network Info:
$(docker network inspect xsshend_net)

Master Logs (last 50 lines):
$(docker logs xsshend_master --tail 50)

Target1 Logs (last 50 lines):
$(docker logs xsshend_target1 --tail 50)

xsshend Version:
$(docker exec xsshend_master xsshend --version 2>&1)

SSH Keys:
$(docker exec xsshend_master ls -la ~/.ssh/)

Problem Description:
[Décrire votre problème ici]

Steps to Reproduce:
1. [Étape 1]
2. [Étape 2]
3. ...

Expected Behavior:
[Ce qui devrait se passer]

Actual Behavior:
[Ce qui se passe réellement]
EOF

cat issue_report.txt
```

### 2. Ouvrir une Issue GitHub

- **URL** : https://github.com/WillIsback/xsshend/issues
- **Joindre** : Le fichier `issue_report.txt`
- **Inclure** : Logs pertinents et commandes exactes

### 3. Vérifier la Documentation

- [LAB-README.md](LAB-README.md)
- [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md)
- [SECURITY.md](../SECURITY.md)
- [README.md](../README.md)

---

## 🧹 Nettoyage et Réinitialisation

### Nettoyage Léger (garder les images)

```bash
docker-compose down
docker volume rm xsshend_master_home 2>/dev/null
docker-compose up -d
```

### Nettoyage Complet (tout supprimer)

```bash
# Arrêter et supprimer
docker-compose down -v

# Supprimer les images
docker rmi $(docker images | grep xsshend-lab | awk '{print $3}')

# Nettoyer les réseaux orphelins
docker network prune -f

# Recréer depuis zéro
./lab-setup.sh
docker-compose up -d --build
```

### Réinitialisation des Clés SSH

```bash
# Supprimer les anciennes clés
rm -rf ssh_keys/ authorized_keys

# Recréer
ssh-keygen -t rsa -b 4096 -f ssh_keys/id_rsa -N "" -C "xsshend_rsa_key"
ssh-keygen -t ed25519 -f ssh_keys/id_ed25519 -N "testpassphrase" -C "xsshend_ed25519_key"
cp ssh_keys/id_rsa.pub authorized_keys
chmod 600 ssh_keys/id_rsa ssh_keys/id_ed25519 authorized_keys
chmod 644 ssh_keys/id_rsa.pub ssh_keys/id_ed25519.pub

# Redémarrer les conteneurs
docker-compose restart
```

---

**Version du guide** : 1.0  
**Dernière mise à jour** : 18 octobre 2025  
**Compatible avec** : xsshend v0.4.1+
