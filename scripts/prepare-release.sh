#!/bin/bash

# Script d'automatisation des releases pour xsshend
# Usage: ./scripts/prepare-release.sh <version>

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Fonctions utilitaires
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Fonction d'aide
show_help() {
    cat << EOF
Script de préparation de release pour xsshend

Usage: $0 <version> [options]

Arguments:
  version           Version à créer (ex: 0.2.3)

Options:
  -h, --help       Afficher cette aide
  -d, --dry-run    Simuler sans faire de changements
  -p, --push       Pousser automatiquement le tag
  --no-test        Ignorer les tests
  --no-fmt         Ignorer la vérification du formatage avec cargo fmt
  --no-clippy      Ignorer la vérification avec cargo clippy
  --force          Forcer même si des incohérences sont détectées

Exemples:
  $0 0.2.3                    # Préparer la version 0.2.3
  $0 0.2.3 --dry-run          # Simuler la préparation
  $0 0.2.3 --push             # Préparer et pousser le tag
EOF
}

# Validation des arguments
if [[ $# -eq 0 ]]; then
    log_error "Version requise"
    show_help
    exit 1
fi

# Check for help first
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_help
    exit 0
fi

VERSION="$1"
shift

# Parsing des options
DRY_RUN=false
PUSH_TAG=false
RUN_TESTS=true
RUN_FMT=true
RUN_CLIPPY=true
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -d|--dry-run)
            DRY_RUN=true
            ;;
        -p|--push)
            PUSH_TAG=true
            ;;
        --no-test)
            RUN_TESTS=false
            ;;
        --no-fmt)
            RUN_FMT=false
            ;;
        --no-clippy)
            RUN_CLIPPY=false
            ;;
        --force)
            FORCE=true
            ;;
        *)
            log_error "Option inconnue: $1"
            show_help
            exit 1
            ;;
    esac
    shift
done

# Validation du format de version
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.-]+)?$ ]]; then
    log_error "Format de version invalide: $VERSION (attendu: X.Y.Z ou X.Y.Z-suffixe)"
    exit 1
fi

TAG="v$VERSION"

log_info "Préparation de la release $TAG"
if [[ "$DRY_RUN" == "true" ]]; then
    log_warning "Mode simulation activé - aucun changement ne sera effectué"
fi

# Vérifier que nous sommes dans le bon répertoire
cd "$PROJECT_ROOT"

# Vérifications préliminaires de la qualité du code
if [[ "$RUN_FMT" == "true" ]]; then
    log_info "Vérification du formatage du code avec cargo fmt..."
    if [[ "$DRY_RUN" == "false" ]]; then
        if ! cargo fmt --check; then
            log_error "Le code n'est pas formaté correctement"
            log_info "Exécutez 'cargo fmt' pour corriger le formatage avant de continuer"
            if [[ "$FORCE" == "false" ]]; then
                exit 1
            else
                log_warning "Formatage incorrect détecté mais ignoré avec --force"
            fi
        fi
        log_success "Formatage du code vérifié"
    else
        log_info "Simulation: vérification du formatage avec cargo fmt"
    fi
fi

if [[ "$RUN_CLIPPY" == "true" ]]; then
    log_info "Vérification des bonnes pratiques avec cargo clippy..."
    if [[ "$DRY_RUN" == "false" ]]; then
        if ! cargo clippy --all-targets --all-features -- -D warnings; then
            log_error "cargo clippy a détecté des problèmes"
            log_warning "Veuillez examiner et corriger les avertissements/erreurs de clippy"
            log_info "Utilisez 'cargo clippy --fix' pour corriger automatiquement certains problèmes"
            if [[ "$FORCE" == "false" ]]; then
                exit 1
            else
                log_warning "Problèmes clippy détectés mais ignorés avec --force"
            fi
        fi
        log_success "Vérifications clippy passées"
    else
        log_info "Simulation: vérification avec cargo clippy"
    fi
fi

if [[ "$RUN_FMT" == "true" || "$RUN_CLIPPY" == "true" ]]; then
    log_success "Vérifications de qualité du code passées"
fi

# Vérifier que nous sommes sur la branche main ou master
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" ]]; then
    log_error "Vous devez être sur la branche 'main' ou 'master' pour créer une release (branche actuelle: $CURRENT_BRANCH)"
    exit 1
fi

# Vérifier que le répertoire de travail est propre
if ! git diff-index --quiet HEAD --; then
    log_error "Le répertoire de travail contient des modifications non commitées"
    exit 1
fi

# Récupérer les dernières modifications
log_info "Récupération des dernières modifications..."
if [[ "$DRY_RUN" == "false" ]]; then
    git pull origin $CURRENT_BRANCH
fi

# Vérifier les versions actuelles
log_info "Vérification des versions actuelles..."

CARGO_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
MAIN_VERSION=$(grep '\.version(' src/main.rs | sed 's/.*\.version("\(.*\)").*/\1/')

log_info "Version actuelle dans Cargo.toml: $CARGO_VERSION"
log_info "Version actuelle dans main.rs: $MAIN_VERSION"

# Vérifier la cohérence des versions actuelles
INCONSISTENT=false
if [[ "$CARGO_VERSION" != "$MAIN_VERSION" ]]; then
    log_error "Versions incohérentes: Cargo.toml ($CARGO_VERSION) != main.rs ($MAIN_VERSION)"
    INCONSISTENT=true
fi

# Vérifier que le tag n'existe pas déjà
if git tag -l | grep -q "^$TAG$"; then
    log_error "Le tag $TAG existe déjà"
    INCONSISTENT=true
fi

# Vérifier sur crates.io
log_info "Vérification sur crates.io..."
if curl -sf "https://crates.io/api/v1/crates/xsshend/$VERSION" > /dev/null 2>&1; then
    log_error "La version $VERSION existe déjà sur crates.io"
    INCONSISTENT=true
else
    log_success "Version $VERSION non trouvée sur crates.io - OK"
fi

# Arrêter si des incohérences sont détectées et --force n'est pas utilisé
if [[ "$INCONSISTENT" == "true" && "$FORCE" == "false" ]]; then
    log_error "Des incohérences ont été détectées. Utilisez --force pour ignorer."
    exit 1
fi

# Mettre à jour les versions
log_info "Mise à jour des versions vers $VERSION..."

if [[ "$DRY_RUN" == "false" ]]; then
    # Mettre à jour Cargo.toml
    sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
    
    # Mettre à jour main.rs
    sed -i "s/\.version(\".*\")/\.version(\"$VERSION\")/" src/main.rs
    
    # Forcer la mise à jour du Cargo.lock avec la nouvelle version
    log_info "Mise à jour du Cargo.lock..."
    cargo check > /dev/null 2>&1
    
    log_success "Versions mises à jour"
else
    log_info "Simulation: mise à jour des versions vers $VERSION"
fi

# Exécuter les tests
if [[ "$RUN_TESTS" == "true" ]]; then
    log_info "Exécution des tests..."
    if [[ "$DRY_RUN" == "false" ]]; then
        cargo test
        log_success "Tests passés"
    else
        log_info "Simulation: exécution des tests"
    fi
fi

# Vérifier que le projet compile
log_info "Vérification de la compilation..."
if [[ "$DRY_RUN" == "false" ]]; then
    cargo check
    cargo build --release
    log_success "Compilation réussie"
else
    log_info "Simulation: vérification de la compilation"
fi

# Créer le commit
log_info "Création du commit pour la version $VERSION..."
if [[ "$DRY_RUN" == "false" ]]; then
    git add Cargo.toml src/main.rs Cargo.lock
    git commit -m "chore: bump version to $VERSION"
    log_success "Commit créé"
else
    log_info "Simulation: création du commit"
fi

# Pousser le commit AVANT de créer le tag si --push est activé
if [[ "$PUSH_TAG" == "true" && "$DRY_RUN" == "false" ]]; then
    log_info "Push du commit de version..."
    if ! git push origin "$CURRENT_BRANCH"; then
        log_error "Échec du push du commit"
        exit 1
    fi
    log_success "Commit de version poussé avec succès"
fi

# Créer le tag
log_info "Création du tag $TAG..."
if [[ "$DRY_RUN" == "false" ]]; then
    git tag -a "$TAG" -m "Release $TAG"
    log_success "Tag $TAG créé"
else
    log_info "Simulation: création du tag $TAG"
fi

# Pousser le tag si demandé
if [[ "$PUSH_TAG" == "true" ]]; then
    log_info "Push du tag..."
    if [[ "$DRY_RUN" == "false" ]]; then
        # Pousser le tag pour déclencher le workflow
        if ! git push origin "$TAG"; then
            log_error "Échec du push du tag $TAG"
            exit 1
        fi
        log_success "Tag $TAG poussé avec succès"
        
        # Attendre un peu puis vérifier que le tag est bien sur le remote
        sleep 2
        if git ls-remote --tags origin | grep -q "$TAG"; then
            log_success "Tag $TAG confirmé sur le remote"
            log_info "🚀 Le workflow GitHub Actions va maintenant prendre le relais"
            log_info "🔗 Surveillez le workflow sur : https://github.com/${GITHUB_REPO:-$(git config --get remote.origin.url | sed 's|.*github.com[:/]\([^/]*\)/\([^/]*\)\.git|\1/\2|')}/actions"
        else
            log_warning "Tag $TAG non trouvé sur le remote, le workflow pourrait ne pas se déclencher"
        fi
    else
        log_info "Simulation: push du commit et du tag"
    fi
else
    log_warning "Tag créé localement. Pour déclencher la release:"
    echo "  git push origin $CURRENT_BRANCH"
    echo "  git push origin $TAG"
fi

# Résumé
echo
log_success "Préparation de la release terminée !"
echo "Version: $VERSION"
echo "Tag: $TAG"
if [[ "$DRY_RUN" == "false" ]]; then
    echo "Status: Prêt à être poussé"
else
    echo "Status: Simulation uniquement"
fi

if [[ "$PUSH_TAG" == "false" ]]; then
    echo
    log_info "Prochaines étapes:"
    echo "1. Vérifiez les changements avec 'git log --oneline -5'"
    echo "2. Poussez avec 'git push origin $CURRENT_BRANCH && git push origin $TAG'"
    echo "3. Le workflow GitHub Actions créera automatiquement la release"
fi
