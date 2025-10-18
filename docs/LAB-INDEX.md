# Documentation xsshend Lab - Index

## 📚 Vue d'ensemble

Cette documentation fournit tout ce dont vous avez besoin pour configurer, tester et dépanner l'environnement de test Docker pour xsshend.

## 🗂️ Structure de la Documentation

```
docs/
├── LAB-INDEX.md                         # ← Vous êtes ici
├── LAB-README.md                        # Guide principal du lab
├── LAB-TESTING-GUIDE.md                 # Guide de test détaillé
├── LAB-TROUBLESHOOTING.md               # Guide de dépannage
└── LAB-TEST-RESULTS-TEMPLATE.md         # Template de rapport de test

scripts/
├── lab-setup.sh                         # Script de configuration du lab
├── test-lab.sh                          # Suite de tests automatisés
└── lab-diagnostic.sh                    # Script de diagnostic
```

## 📖 Guides par Objectif

### 🚀 Je veux DÉMARRER le lab

**➜ Lire** : [LAB-README.md](LAB-README.md) - Sections :
- Démarrage Rapide (installation en 7 étapes)
- Architecture
- Configuration des Clés SSH

**➜ Exécuter** :
```bash
# 1. Setup
./scripts/lab-setup.sh  # ou copier depuis scripts/

# 2. Démarrer
cd lab/
docker-compose up -d --build

# 3. Vérifier
docker-compose ps
```

---

### 🧪 Je veux TESTER xsshend

**➜ Lire** : [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md) - Sections :
- Configuration du Lab
- Procédure de Test (10 phases)
- Tests de Connectivité Manuelle
- Tests xsshend
- Scénarios de Test Avancés

**➜ Exécuter** :
```bash
# Tests automatisés
./scripts/test-lab.sh

# Tests manuels
docker exec -it xsshend_master bash
xsshend list
xsshend upload test.txt --env Test --dry-run
xsshend upload test.txt --env Test
```

**➜ Documenter** : [LAB-TEST-RESULTS-TEMPLATE.md](LAB-TEST-RESULTS-TEMPLATE.md)
- Copier le template
- Remplir les résultats au fur et à mesure
- Archiver pour référence

---

### 🔧 J'ai un PROBLÈME avec le lab

**➜ Lire** : [LAB-TROUBLESHOOTING.md](LAB-TROUBLESHOOTING.md) - Sections :
- Checklist Rapide
- Problèmes Courants et Solutions (7 problèmes)
- Outils de Diagnostic
- Nettoyage et Réinitialisation

**➜ Exécuter** :
```bash
# Diagnostic rapide
./scripts/lab-diagnostic.sh

# Sauvegarder le rapport
./scripts/lab-diagnostic.sh > diagnostic_report.txt 2>&1

# Nettoyage complet
docker-compose down -v
docker rmi $(docker images | grep xsshend-lab | awk '{print $3}')
./scripts/lab-setup.sh
docker-compose up -d --build
```

---

### 📊 Je veux DOCUMENTER mes tests

**➜ Utiliser** : [LAB-TEST-RESULTS-TEMPLATE.md](LAB-TEST-RESULTS-TEMPLATE.md)

**Process** :
1. Copier le template :
   ```bash
   cp docs/LAB-TEST-RESULTS-TEMPLATE.md my_test_results_$(date +%Y%m%d).md
   ```

2. Exécuter les tests en remplissant le template au fur et à mesure

3. Remplacer ⏳ par ✅ (réussi) ou ❌ (échoué)

4. Documenter les problèmes rencontrés

5. Archiver le rapport

---

## 📋 Checklist Complète : Premier Lancement

### Étape 1 : Préparation (5 min)

- [ ] Docker installé et fonctionnel
- [ ] Docker Compose installé
- [ ] ~2GB d'espace disque disponible
- [ ] Repository xsshend cloné
- [ ] Documentation lue (LAB-README.md)

### Étape 2 : Installation (10 min)

- [ ] Script `lab-setup.sh` exécuté
- [ ] Clés SSH générées (RSA + Ed25519)
- [ ] `hosts.json` créé
- [ ] `docker-compose.yml` créé
- [ ] Dockerfiles créés

**Commandes** :
```bash
cd lab/
./lab-setup.sh
ls -la ssh_keys/  # Vérifier les clés
```

### Étape 3 : Démarrage (5 min)

- [ ] `docker-compose up -d --build` exécuté
- [ ] 3 conteneurs démarrés (master, target1, target2)
- [ ] Réseau `xsshend_net` créé
- [ ] Volume `master_home` créé
- [ ] Attente de 10 secondes pour le démarrage SSH

**Commandes** :
```bash
docker-compose up -d --build
sleep 10
docker-compose ps  # Doit montrer 3 conteneurs "Up"
```

### Étape 4 : Vérification (5 min)

- [ ] Diagnostic exécuté sans erreurs critiques
- [ ] xsshend installé et fonctionnel
- [ ] SSH daemons actifs sur targets
- [ ] Connectivité réseau OK
- [ ] SSH manuel fonctionne

**Commandes** :
```bash
./scripts/lab-diagnostic.sh
docker exec xsshend_master xsshend --version
docker exec xsshend_master ssh -i ~/.ssh/id_rsa testuser@target1 "hostname"
```

### Étape 5 : Tests Basiques (10 min)

- [ ] `xsshend list` affiche 3 serveurs
- [ ] Upload dry-run réussi
- [ ] Upload réel vers RSA-Targets réussi
- [ ] Fichiers vérifiés sur target1 et target2
- [ ] Tests automatisés passent

**Commandes** :
```bash
docker exec -it xsshend_master bash
xsshend list
echo "Test" > test.txt
xsshend upload test.txt --env Test --dry-run
xsshend upload test.txt --env Test --server-type RSA-Targets
exit
docker exec xsshend_target1 cat /tmp/test.txt
./scripts/test-lab.sh
```

### Étape 6 : Tests Avancés (30 min)

- [ ] Multi-fichiers testés
- [ ] Gros fichiers testés (10MB)
- [ ] Gestion d'erreurs testée
- [ ] Filtres testés (env, type, region)
- [ ] Logs SSH vérifiés

**Référence** : [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md)

### Étape 7 : Documentation (15 min)

- [ ] Template de résultats copié
- [ ] Résultats documentés
- [ ] Problèmes notés
- [ ] Recommandations écrites
- [ ] Rapport archivé

**Template** : [LAB-TEST-RESULTS-TEMPLATE.md](LAB-TEST-RESULTS-TEMPLATE.md)

---

## 🎯 Parcours par Profil Utilisateur

### 👨‍💻 Développeur xsshend

**Objectif** : Tester des modifications du code

**Workflow** :
1. Modifier le code localement
2. Compiler : `cargo build --release`
3. Copier dans le conteneur :
   ```bash
   docker cp target/release/xsshend xsshend_master:/home/master/.cargo/bin/
   ```
4. Tester immédiatement :
   ```bash
   docker exec xsshend_master xsshend --version
   docker exec xsshend_master xsshend upload test.txt --env Test
   ```

**Documentation utile** :
- [LAB-README.md](LAB-README.md) - Section "Développement"
- [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md) - Tous les tests

---

### 🧪 Testeur / QA

**Objectif** : Valider une version avant release

**Workflow** :
1. Installer la version à tester
2. Exécuter la suite complète :
   ```bash
   ./scripts/test-lab.sh > test_report.txt 2>&1
   ```
3. Tester les scénarios avancés (LAB-TESTING-GUIDE.md)
4. Documenter les résultats (LAB-TEST-RESULTS-TEMPLATE.md)
5. Rapporter les bugs si nécessaire

**Documentation utile** :
- [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md) - Guide complet
- [LAB-TEST-RESULTS-TEMPLATE.md](LAB-TEST-RESULTS-TEMPLATE.md) - Rapport
- [LAB-TROUBLESHOOTING.md](LAB-TROUBLESHOOTING.md) - Si problèmes

---

### 📚 Utilisateur Final

**Objectif** : Apprendre à utiliser xsshend

**Workflow** :
1. Démarrer le lab
2. Suivre le guide de test étape par étape
3. Expérimenter avec différentes commandes
4. Comprendre le comportement avant utilisation en production

**Documentation utile** :
- [LAB-README.md](LAB-README.md) - Démarrage rapide
- [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md) - Exemples d'utilisation
- [../README.md](../README.md) - Documentation principale xsshend

---

### 🔧 Administrateur Système

**Objectif** : Valider xsshend pour déploiement production

**Workflow** :
1. Configurer le lab selon l'architecture de production
2. Tester avec des volumes de fichiers réalistes
3. Vérifier la sécurité (SECURITY.md)
4. Valider les performances
5. Tester les scénarios d'erreur et de récupération

**Documentation utile** :
- [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md) - Tests avancés
- [../SECURITY.md](../SECURITY.md) - Sécurité
- [../docs/RUSTSEC-2023-0071-EXPLANATION.md](RUSTSEC-2023-0071-EXPLANATION.md) - Limitation
- [LAB-TROUBLESHOOTING.md](LAB-TROUBLESHOOTING.md) - Dépannage

---

## 🛠️ Scripts Disponibles

### `scripts/lab-setup.sh`
**Usage** : Configuration initiale du lab
```bash
./scripts/lab-setup.sh
```
**Fait** :
- Génère les clés SSH (RSA + Ed25519)
- Crée `hosts.json`
- Crée les Dockerfiles
- Crée `docker-compose.yml`
- Configure les permissions

---

### `scripts/test-lab.sh`
**Usage** : Suite de tests automatisés
```bash
./scripts/test-lab.sh
```
**Teste** :
- Statut des conteneurs
- Installation xsshend
- Configuration SSH
- Connectivité réseau et SSH
- Commandes xsshend (list, upload)
- Multi-fichiers
- Logs

**Sortie** : Rapport avec compteurs de tests réussis/échoués

---

### `scripts/lab-diagnostic.sh`
**Usage** : Diagnostic rapide de l'environnement
```bash
./scripts/lab-diagnostic.sh
./scripts/lab-diagnostic.sh > diagnostic_report.txt  # Sauvegarder
```
**Vérifie** :
- Docker et Docker Compose
- Statut des conteneurs
- Réseau Docker
- Installation xsshend
- Clés SSH et permissions
- SSH daemons
- Connectivité (réseau et SSH)
- Clés autorisées
- Espace disque
- Logs récents

**Sortie** : Rapport coloré avec ✓/✗/⚠

---

## 📞 Support et Ressources

### Documentation Projet Principal

- **README** : [../README.md](../README.md) - Guide principal xsshend
- **SECURITY** : [../SECURITY.md](../SECURITY.md) - Politique de sécurité
- **CHANGELOG** : [../CHANGELOG.md](../CHANGELOG.md) - Historique des versions
- **Vulnérabilité** : [RUSTSEC-2023-0071-EXPLANATION.md](RUSTSEC-2023-0071-EXPLANATION.md)

### Documentation en Ligne

- **Site web** : https://willisback.github.io/xsshend/
- **Repository** : https://github.com/WillIsback/xsshend
- **Issues** : https://github.com/WillIsback/xsshend/issues
- **Discussions** : https://github.com/WillIsback/xsshend/discussions

### Obtenir de l'Aide

1. **Consulter** : [LAB-TROUBLESHOOTING.md](LAB-TROUBLESHOOTING.md)
2. **Exécuter** : `./scripts/lab-diagnostic.sh > diagnostic.txt`
3. **Chercher** : Issues GitHub existantes
4. **Créer** : Nouvelle issue avec `diagnostic.txt` attaché

---

## 🔄 Workflow Recommandé

### Workflow Standard de Test

```
1. Setup Initial
   ├── Lire LAB-README.md
   ├── Exécuter lab-setup.sh
   ├── docker-compose up -d --build
   └── Vérifier avec lab-diagnostic.sh

2. Tests Basiques
   ├── Exécuter test-lab.sh
   ├── Tests manuels (LAB-TESTING-GUIDE.md)
   └── Documenter (LAB-TEST-RESULTS-TEMPLATE.md)

3. Tests Avancés
   ├── Scénarios personnalisés
   ├── Tests de performance
   └── Tests d'erreurs

4. Validation
   ├── Revue des résultats
   ├── Documentation des problèmes
   └── Décision go/no-go production

5. Nettoyage
   └── docker-compose down -v
```

---

## ✅ Checklist Avant Production

Si vous utilisez le lab pour valider xsshend avant déploiement production :

- [ ] Tous les tests du lab passent (40/40)
- [ ] Tests de performance acceptables
- [ ] Gestion d'erreurs robuste validée
- [ ] SECURITY.md lu et compris
- [ ] RUSTSEC-2023-0071-EXPLANATION.md lu
- [ ] Décision sur type de clés SSH (Ed25519 recommandé)
- [ ] Configuration production préparée (hosts.json)
- [ ] Plan de rollback défini
- [ ] Équipe formée à l'utilisation
- [ ] Documentation accessible

---

## 📊 Métriques de Validation

Le lab est considéré comme **fonctionnel** si :

- ✅ 3 conteneurs démarrent et restent actifs
- ✅ xsshend --version fonctionne
- ✅ xsshend list retourne 3 serveurs
- ✅ SSH manuel fonctionne (RSA vers target1/2)
- ✅ xsshend upload réussit vers RSA-Targets
- ✅ Fichiers uploadés vérifiables sur targets
- ✅ Gestion d'erreurs gracieuse (clé non enregistrée, serveur down)
- ✅ `./scripts/test-lab.sh` retourne exit code 0

xsshend est considéré comme **prêt pour production** si :

- ✅ Lab fonctionnel (ci-dessus)
- ✅ 40/40 tests passent dans LAB-TESTING-GUIDE.md
- ✅ Tests de performance acceptables
- ✅ Aucun problème critique non résolu
- ✅ Documentation complète et à jour
- ✅ Équipe formée et confiante

---

## 🎓 Formation Recommandée

### Niveau Débutant (2 heures)

1. **Lecture** (45 min)
   - LAB-README.md (sections principales)
   - README.md xsshend (utilisation basique)

2. **Pratique** (1h15)
   - Démarrer le lab
   - Tests basiques (list, upload)
   - Vérifications manuelles

### Niveau Intermédiaire (4 heures)

1. **Lecture** (1h)
   - LAB-TESTING-GUIDE.md complet
   - SECURITY.md

2. **Pratique** (3h)
   - Tous les tests du guide
   - Scénarios avancés
   - Gestion d'erreurs

### Niveau Avancé (8 heures)

1. **Lecture** (2h)
   - Toute la documentation lab
   - RUSTSEC-2023-0071-EXPLANATION.md
   - Code source xsshend

2. **Pratique** (6h)
   - Tests complets
   - Développement/modifications
   - Résolution de problèmes
   - Documentation de nouveaux scénarios

---

**Version de l'index** : 1.0  
**Dernière mise à jour** : 18 octobre 2025  
**Compatible avec** : xsshend v0.4.1+

---

**Navigation** :
- ← Retour : [Documentation principale](../README.md)
- → Démarrer : [LAB-README.md](LAB-README.md)
- → Tester : [LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md)
- → Dépanner : [LAB-TROUBLESHOOTING.md](LAB-TROUBLESHOOTING.md)
