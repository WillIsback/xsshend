# Documentation xsshend

> 🚀 Documentation complète pour xsshend - Téléversement SSH parallèle

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

> Documentation complète pour tester xsshend dans un environnement Docker isolé

### 🎯 Point d'Entrée
- **[LAB-INDEX.md](LAB-INDEX.md)** - **COMMENCER ICI** - Navigation et guides par objectif

### Documentation Lab
- **[LAB-README.md](LAB-README.md)** - Installation et configuration du lab Docker
- **[LAB-TESTING-GUIDE.md](LAB-TESTING-GUIDE.md)** - Guide de test complet (40+ tests)
- **[LAB-TROUBLESHOOTING.md](LAB-TROUBLESHOOTING.md)** - Dépannage et résolution de problèmes
- **[LAB-TEST-RESULTS-TEMPLATE.md](LAB-TEST-RESULTS-TEMPLATE.md)** - Template de rapport de test
- **[LAB-DOCUMENTATION-SUMMARY.md](LAB-DOCUMENTATION-SUMMARY.md)** - Résumé de la documentation lab

### Scripts Lab
- `../scripts/lab-setup.sh` - Configuration initiale du lab
- `../scripts/test-lab.sh` - Suite de tests automatisés (40+ tests)
- `../scripts/lab-diagnostic.sh` - Diagnostic rapide de l'environnement

### Démarrage Rapide Lab
```bash
# 1. Setup
./scripts/lab-setup.sh

# 2. Démarrer (3 conteneurs : master + 2 targets)
cd lab/
docker-compose up -d --build

# 3. Tests automatisés
../scripts/test-lab.sh

# 4. Tests manuels
docker exec -it xsshend_master bash
xsshend list
xsshend upload test.txt --env Test
```

**Total documentation lab** : 6 fichiers (~2850 lignes) + 3 scripts (~790 lignes)

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
