# Documentation xsshend

> 🚀 Documentation complèt## 🔒 Sécurité

### Documentation Sécurité
- **[../SECURITY.md](../SECURITY.md)** - Politique de sécurité officielle ⚠️ **LIRE AVANT PRODUCTION**
- **[RUSTSEC-2023-0071-EXPLANATION.md](RUSTSEC-2023-0071-EXPLANATION.md)** - Explication détaillée de la limitation

### ⚠️ Limitation Connue : RUSTSEC-2023-0071

xsshend v0.4.1 a une **limitation de sécurité connue** (Marvin Attack dans `rsa 0.9.8`).

**Mitigation** :
- ✅ Utiliser des clés Ed25519 (recommandé)
- ✅ Déployer sur réseaux de confiance uniquement
- ❌ Éviter WiFi public, réseaux non sécurisés

**Détails** : [RUSTSEC-2023-0071-EXPLANATION.md](RUSTSEC-2023-0071-EXPLANATION.md)Téléversement SSH parallèle

## 📖 Guides utilisateur

### Démarrage rapide
- [Installation et premier lancement](usage.md#installation-et-premier-lancement)
- [Configuration automatique](usage.md#configuration-automatique)
- [Premiers transferts](usage.md#premiers-transferts)

### Utilisation avancée
- [Interface utilisateur complète](usage.md)
- [Configuration des serveurs](configuration.md)
- [Gestion des clés SSH](ssh-keys.md)

## 🧪 Environnement de Test (Lab)

> Guide complet pour tester xsshend dans un environnement Docker isolé

### 📖 Documentation
- **[LAB-GUIDE.md](LAB-GUIDE.md)** - **Guide complet consolidé** (~4800 lignes)
  - Installation rapide (10 minutes)
  - Architecture Docker (3 conteneurs)
  - 40+ tests automatisés
  - Dépannage complet
  - FAQ et bonnes pratiques

### 🚀 Démarrage Rapide
```bash
# 1. Setup
./scripts/lab-setup.sh

# 2. Démarrer (master + 2 targets)
cd lab/
docker-compose up -d --build

# 3. Tests automatisés
../scripts/test-lab.sh

# 4. Tests manuels
docker exec -it xsshend_master bash
xsshend list
xsshend upload test.txt --env Test
```

### Scripts
- `scripts/lab-setup.sh` - Configuration initiale
- `scripts/test-lab.sh` - Suite de tests (40+ tests)
- `scripts/lab-diagnostic.sh` - Diagnostic rapide

## � Sécurité

### Documentation Sécurité
- **[../SECURITY.md](../SECURITY.md)** - Politique de sécurité officielle ⚠️ **LIRE AVANT PRODUCTION**
- **[RUSTSEC-2023-0071-EXPLANATION.md](RUSTSEC-2023-0071-EXPLANATION.md)** - Explication détaillée de la limitation connue

### ⚠️ Limitation Connue : RUSTSEC-2023-0071

xsshend v0.4.1 a une **limitation de sécurité connue** liée à la vulnérabilité Marvin Attack dans `rsa 0.9.8`.

**En bref** :
- ❌ Pas de correction disponible (dépendance transitive via russh → rsa)
- ✅ **Mitigation** : Utiliser des clés Ed25519 (recommandé)
- ✅ **Production** : Déployer uniquement sur réseaux de confiance
- ✅ **Lab de test** : Aucun impact (réseau Docker isolé)

**Voir la documentation complète** : [RUSTSEC-2023-0071-EXPLANATION.md](RUSTSEC-2023-0071-EXPLANATION.md) (350 lignes)

## �🛠️ Guides développeur

### CI/CD et déploiement
- [Workflow complet](cicd.md)
- [Contribution](../README.md#contribution)

### Architecture
- [Structure du code](../README.md#architecture-du-code)
- [Tests et validation](../README.md#tests-et-validation)

## 🌐 Documentation en ligne

- **Site principal :** [willisback.github.io/xsshend](https://willisback.github.io/xsshend)
- **Documentation API :** [docs.rs/xsshend](https://docs.rs/xsshend)
- **Repository :** [github.com/WillIsback/xsshend](https://github.com/WillIsback/xsshend)

## 🔗 Liens rapides

| Action | Lien |
|--------|------|
| 📦 Installation | [crates.io/crates/xsshend](https://crates.io/crates/xsshend) |
| 🐛 Issues | [GitHub Issues](https://github.com/WillIsback/xsshend/issues) |
| 🔄 Releases | [GitHub Releases](https://github.com/WillIsback/xsshend/releases) |
| 📊 CI/CD Status | [GitHub Actions](https://github.com/WillIsback/xsshend/actions) |
