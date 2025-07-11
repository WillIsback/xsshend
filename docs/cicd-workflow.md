# Workflow CI/CD

Ce document décrit le processus d'intégration continue et de déploiement continu (CI/CD) mis en place pour xsshend.

## Vue d'ensemble

Le projet utilise GitHub Actions pour automatiser :
- ✅ Tests et vérifications de qualité de code
- 📦 Build multi-plateforme
- 🚀 Publication automatique sur crates.io
- 📋 Création de releases GitHub avec binaires
- 🔍 Vérification de cohérence des versions

## Workflows

### 1. CI (Intégration Continue)

**Fichier:** `.github/workflows/ci.yml`

**Déclenché sur :**
- Push sur `main` et `develop`
- Pull requests vers `main`

**Jobs :**
- **test**: Exécute les tests, vérifications de format et Clippy
- **check**: Vérification de la syntaxe Cargo
- **security**: Audit de sécurité avec `cargo-audit`
- **docs**: Build de la documentation et tests des exemples

### 2. Development (Développement)

**Fichier:** `.github/workflows/development.yml`

**Déclenché sur :**
- Push sur `develop` et branches `feature/**`
- Pull requests vers `main` et `develop`

**Jobs :**
- **quick-check**: Vérifications rapides (format, clippy, compilation)
- **test**: Tests complets
- **version-check**: Vérification de cohérence des versions pour les PRs de release

### 3. Release (Publication)

**Fichier:** `.github/workflows/release.yml`

**Déclenché sur :**
- Push de tags `v*` (ex: `v0.2.3`)

**Jobs :**
1. **pre-release-checks**: Vérifications de cohérence avant release
2. **test**: Tests avant publication
3. **build**: Build multi-plateforme (Linux, Windows, macOS)
4. **publish**: Publication sur crates.io
5. **release**: Création de la release GitHub avec assets
6. **cleanup**: Nettoyage des artefacts

## Vérifications de Cohérence

Le workflow vérifie automatiquement :

### 📋 Cohérence des versions
- Version dans `Cargo.toml` = version dans `src/main.rs`
- Version dans les fichiers = version du tag Git
- Version n'existe pas déjà sur crates.io
- Release GitHub n'existe pas déjà

### ⚠️ En cas d'incohérence
Le workflow s'arrête et affiche :
- ❌ Les erreurs détectées
- 🔧 Les actions correctives à effectuer
- 📊 Un résumé dans l'interface GitHub

## Processus de Release

### Option 1: Script automatisé (Recommandé)

```bash
# Préparer une nouvelle release
./scripts/prepare-release.sh 0.2.3

# Avec options
./scripts/prepare-release.sh 0.2.3 --dry-run    # Simulation
./scripts/prepare-release.sh 0.2.3 --push       # Push automatique
./scripts/prepare-release.sh 0.2.3 --no-test    # Sans tests
```

### Option 2: Manuel

1. **Préparer la version**
   ```bash
   # Mettre à jour les versions
   sed -i 's/version = ".*"/version = "0.2.3"/' Cargo.toml
   sed -i 's/version = ".*"/version = "0.2.3"/' src/main.rs
   
   # Tester
   cargo test
   cargo build --release
   ```

2. **Créer le commit et tag**
   ```bash
   git add Cargo.toml src/main.rs
   git commit -m "chore: bump version to 0.2.3"
   git tag -a v0.2.3 -m "Release v0.2.3"
   ```

3. **Pousser**
   ```bash
   git push origin main
   git push origin v0.2.3
   ```

4. **Suivre la release**
   - Aller sur [Actions](https://github.com/williamdes/xsshend/actions)
   - Vérifier que le workflow "Release" se déroule correctement

## Plateformes Supportées

Le build automatique génère des binaires pour :

| Plateforme | Target | Binaire |
|------------|--------|---------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `xsshend` |
| Linux x86_64 (musl) | `x86_64-unknown-linux-musl` | `xsshend` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `xsshend.exe` |
| macOS x86_64 | `x86_64-apple-darwin` | `xsshend` |
| macOS ARM64 | `aarch64-apple-darwin` | `xsshend` |

## Variables d'Environnement Requises

### Secrets GitHub

- `CARGO_REGISTRY_TOKEN`: Token pour publier sur crates.io
  - Obtenir sur [crates.io/me](https://crates.io/me)
  - Ajouter dans Settings > Secrets and variables > Actions

### Permissions

Le workflow utilise le token `GITHUB_TOKEN` automatique pour :
- Créer des releases
- Uploader des assets
- Écrire des résumés

## Monitoring et Debug

### Vérifier le status

```bash
# Status sur crates.io
curl -s https://crates.io/api/v1/crates/xsshend | jq '.crate.newest_version'

# Status des releases GitHub
gh release list --limit 10

# Logs des workflows
gh run list --workflow=release.yml
```

### Debug des échecs

1. **Échec des vérifications de cohérence**
   - Vérifier les versions dans `Cargo.toml` et `src/main.rs`
   - S'assurer que le tag correspond à la version

2. **Échec de publication crates.io**
   - Vérifier le token `CARGO_REGISTRY_TOKEN`
   - Vérifier que la version n'existe pas déjà

3. **Échec du build**
   - Vérifier les dépendances système pour le cross-compilation
   - Tester localement avec `cargo build --target <target>`

## Configuration

### Cache Cargo

Les workflows utilisent `actions/cache` pour accélérer les builds en cachant :
- `~/.cargo/bin/`
- `~/.cargo/registry/`
- `target/`

### Timeouts

- Jobs CI: ~10 minutes
- Job de release: ~30 minutes
- Build par plateforme: ~15 minutes

## Bonnes Pratiques

### Commits

Utiliser [Conventional Commits](https://www.conventionalcommits.org/) :
- `feat:` pour les nouvelles fonctionnalités
- `fix:` pour les corrections de bugs
- `chore:` pour les tâches de maintenance
- `docs:` pour la documentation

### Branches

- `main`: Branche stable, prête pour release
- `develop`: Branche de développement
- `feature/**`: Branches de fonctionnalités

### Tests

- Tous les PRs doivent passer les tests
- Les releases nécessitent 100% de tests passants
- Tests unitaires + tests d'intégration

## Troubleshooting

### Problèmes Courants

**"Version already exists on crates.io"**
```bash
# Incrémenter la version
./scripts/prepare-release.sh 0.2.4
```

**"Git tag already exists"**
```bash
# Supprimer le tag localement et remotement
git tag -d v0.2.3
git push --delete origin v0.2.3
```

**"Consistency check failed"**
```bash
# Vérifier et corriger les versions
grep version Cargo.toml src/main.rs
```

### Forcer une Release

En cas de problème avec les vérifications :

```bash
# Forcer la préparation (à utiliser avec précaution)
./scripts/prepare-release.sh 0.2.3 --force
```

## Améliorations Futures

- [ ] Integration avec Dependabot pour les dépendances
- [ ] Tests de performance automatisés
- [ ] Déploiement automatique sur les registres de packages
- [ ] Notifications Slack/Discord
- [ ] Métriques et analytics
- [ ] Tests sur plus d'architectures (ARM, RISC-V)
