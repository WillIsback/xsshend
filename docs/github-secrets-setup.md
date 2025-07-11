# Configuration des Secrets GitHub pour CI/CD

Ce document explique comment configurer les secrets nécessaires pour les workflows GitHub Actions de xsshend.

## Secrets Requis

### `CARGO_REGISTRY_TOKEN`

**Description :** Token pour publier sur [crates.io](https://crates.io)

**Comment l'obtenir :**

1. Connectez-vous sur [crates.io](https://crates.io)
2. Allez dans [Account Settings](https://crates.io/me)
3. Cliquez sur "API Tokens"
4. Cliquez sur "New Token"
5. Donnez un nom au token (ex: "xsshend-github-actions")
6. Copiez le token généré

**Comment l'ajouter à GitHub :**

1. Allez dans les Settings du repository
2. Cliquez sur "Secrets and variables" > "Actions"
3. Cliquez sur "New repository secret"
4. Nom : `CARGO_REGISTRY_TOKEN`
5. Valeur : Collez le token de crates.io
6. Cliquez sur "Add secret"

### `GITHUB_TOKEN` (Automatique)

**Description :** Token automatiquement fourni par GitHub Actions

**Configuration :** Aucune - ce token est automatiquement disponible dans tous les workflows

**Permissions utilisées :**
- Lecture du repository
- Création de releases
- Upload d'assets
- Écriture de résumés

## Vérification de la Configuration

### Test des Secrets

Vous pouvez tester que les secrets sont correctement configurés en créant une release :

```bash
# Préparer une version de test
./scripts/prepare-release.sh 0.2.3 --dry-run

# Si tout semble correct, créer une vraie release
./scripts/prepare-release.sh 0.2.3 --push
```

### Workflow de Test

Le workflow CI s'exécute automatiquement sur les push et pull requests. Vérifiez qu'il fonctionne :

1. Faites un push sur une branche
2. Vérifiez l'onglet "Actions" de GitHub
3. Le workflow "CI" doit s'exécuter sans erreur

### Debug des Problèmes

**Erreur "authentication required" lors de `cargo publish` :**
- Vérifiez que `CARGO_REGISTRY_TOKEN` est configuré
- Vérifiez que le token n'a pas expiré
- Recréez un nouveau token si nécessaire

**Erreur "permission denied" lors de la création de release :**
- Le `GITHUB_TOKEN` automatique devrait suffire
- Vérifiez les permissions du repository dans Settings > Actions > General

## Sécurité des Secrets

### Bonnes Pratiques

- ✅ **Ne jamais** committer de secrets dans le code
- ✅ **Utilisez** les secrets GitHub pour les tokens
- ✅ **Limitez** les permissions des tokens au minimum nécessaire
- ✅ **Renouvelez** régulièrement vos tokens
- ✅ **Révoquez** les tokens non utilisés

### Rotation des Tokens

Il est recommandé de faire tourner le `CARGO_REGISTRY_TOKEN` tous les 3-6 mois :

1. Créez un nouveau token sur crates.io
2. Mettez à jour le secret GitHub
3. Révoquez l'ancien token sur crates.io

## Permissions Repository

### Actions

Dans Settings > Actions > General, assurez-vous que :

- **Actions permissions** : "Allow all actions and reusable workflows"
- **Workflow permissions** : "Read and write permissions"
- **Allow GitHub Actions to create and approve pull requests** : Coché (si nécessaire)

### Branches

Pour les branches protégées (main/master), configurez :

- **Require status checks to pass before merging** : Coché
- **Require branches to be up to date before merging** : Coché
- Sélectionnez les workflows requis : "CI"

## Validation de la Configuration

### Checklist Complète

- [ ] `CARGO_REGISTRY_TOKEN` configuré dans GitHub Secrets
- [ ] Permissions Actions configurées
- [ ] Test du workflow CI réussi
- [ ] Test du script de release en dry-run
- [ ] Documentation à jour

### Commandes de Vérification

```bash
# Vérifier que le token crates.io fonctionne localement
export CARGO_REGISTRY_TOKEN="votre_token"
cargo login $CARGO_REGISTRY_TOKEN
cargo search xsshend

# Vérifier les workflows GitHub
gh workflow list
gh run list --workflow=ci.yml --limit=5

# Vérifier les secrets (ne montre pas les valeurs)
gh secret list
```

## Troubleshooting

### Problèmes Courants

**"Error: Resource not accessible by integration"**
```
Solution: Vérifier les permissions du GITHUB_TOKEN dans Settings > Actions
```

**"Error: failed to authenticate to registry"**
```
Solution: 
1. Vérifier que CARGO_REGISTRY_TOKEN existe
2. Créer un nouveau token sur crates.io
3. Mettre à jour le secret GitHub
```

**"Error: tag already exists"**
```
Solution:
1. Supprimer le tag existant: git tag -d vX.Y.Z
2. Pousser la suppression: git push --delete origin vX.Y.Z
3. Recréer la release avec le script
```

### Support

- 📖 [Documentation GitHub Actions](https://docs.github.com/en/actions)
- 📖 [Documentation crates.io API](https://doc.rust-lang.org/cargo/reference/publishing.html)
- 🐛 [Issues du projet](https://github.com/williamdes/xsshend/issues)

## Automatisation Avancée

### Dependabot (Optionnel)

Créez `.github/dependabot.yml` pour les mises à jour automatiques :

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    labels:
      - "dependencies"
      - "rust"
```

### Notifications (Optionnel)

Ajoutez des notifications Slack/Discord en cas d'échec :

```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: failure
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```
