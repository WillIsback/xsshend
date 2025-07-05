#!/bin/bash

# Suite de tests d'intégration pour xsshend
# Tests automatisés sur les VMs Multipass

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
XSSHEND_BIN="$PROJECT_ROOT/target/debug/xsshend"
TEST_CONFIG="$SCRIPT_DIR/configs/test-hosts.json"
TEST_DATA_DIR="$SCRIPT_DIR/data"

# Couleurs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Compteurs de tests
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Fonctions d'affichage
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

test_start() {
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo ""
    log_info "Test $TESTS_TOTAL: $1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

test_success() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    log_success "$1"
}

test_failure() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    log_error "$1"
}

# Vérifications préalables
check_prerequisites() {
    log_info "Vérification des prérequis..."
    
    # Vérifier que Multipass est accessible
    if ! command -v multipass &> /dev/null; then
        if [[ -f "/snap/bin/multipass" ]]; then
            export PATH="/snap/bin:$PATH"
            log_info "Multipass trouvé dans /snap/bin, ajout au PATH"
        else
            log_error "Multipass n'est pas installé ou introuvable!"
            exit 1
        fi
    fi
    
    # Vérifier que xsshend est compilé
    if [[ ! -f "$XSSHEND_BIN" ]]; then
        log_error "xsshend n'est pas compilé. Exécutez 'cargo build' d'abord."
        exit 1
    fi
    
    # Vérifier la configuration de test
    if [[ ! -f "$TEST_CONFIG" ]]; then
        log_error "Configuration de test manquante: $TEST_CONFIG"
        log_info "Exécutez './test-vms.sh generate-config' d'abord."
        exit 1
    fi
    
    # Vérifier les fichiers de test
    if [[ ! -d "$TEST_DATA_DIR" ]] || [[ ! -f "$TEST_DATA_DIR/simple.txt" ]]; then
        log_error "Fichiers de test manquants."
        log_info "Exécutez './generate-test-files.sh' d'abord."
        exit 1
    fi
    
    # Vérifier que les VMs sont en cours d'exécution
    if ! multipass list | grep -q "xsshend.*Running"; then
        log_error "Aucune VM xsshend en cours d'exécution."
        log_info "Exécutez './test-vms.sh launch-all' d'abord."
        exit 1
    fi
    
    log_success "Tous les prérequis sont satisfaits"
}

# Test 1: CLI et help
test_cli_help() {
    test_start "Interface CLI et aide"
    
    if "$XSSHEND_BIN" --help >/dev/null 2>&1; then
        test_success "Commande --help fonctionne"
    else
        test_failure "Commande --help échoue"
        return
    fi
    
    if "$XSSHEND_BIN" --version >/dev/null 2>&1; then
        test_success "Commande --version fonctionne"
    else
        test_failure "Commande --version échoue"
    fi
}

# Test 2: Configuration hosts.json
test_hosts_config() {
    test_start "Chargement configuration hosts.json"
    
    # Utiliser la configuration de test
    export HOME="$SCRIPT_DIR/configs"
    cp "$TEST_CONFIG" "$SCRIPT_DIR/configs/.ssh/hosts.json" 2>/dev/null || {
        mkdir -p "$SCRIPT_DIR/configs/.ssh"
        cp "$TEST_CONFIG" "$SCRIPT_DIR/configs/.ssh/hosts.json"
    }
    
    if "$XSSHEND_BIN" list >/dev/null 2>&1; then
        test_success "Configuration hosts.json chargée correctement"
    else
        test_failure "Échec du chargement de la configuration"
        return
    fi
    
    # Test filtrage par environnement
    if "$XSSHEND_BIN" list --env Development | grep -q "Development"; then
        test_success "Filtrage par environnement fonctionne"
    else
        test_failure "Filtrage par environnement échoue"
    fi
}

# Test 3: Dry-run
test_dry_run() {
    test_start "Mode dry-run (simulation)"
    
    local test_file="$TEST_DATA_DIR/simple.txt"
    
    if "$XSSHEND_BIN" upload "$test_file" --env Development --dry-run >/dev/null 2>&1; then
        test_success "Dry-run fonctionne sans erreur"
    else
        test_failure "Dry-run échoue"
        return
    fi
    
    # Vérifier que le fichier n'a pas été réellement transféré
    local dev_ip=$(multipass info xsshend-dev | grep IPv4 | awk '{print $2}' 2>/dev/null)
    if [[ -n "$dev_ip" ]]; then
        if ! ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no \
            -i "$SCRIPT_DIR/.ssh/test_key" "xsshend-test@$dev_ip" \
            "test -f /tmp/simple.txt" 2>/dev/null; then
            test_success "Dry-run n'a pas transféré de fichier (correct)"
        else
            test_failure "Dry-run a transféré un fichier (incorrect)"
        fi
    fi
}

# Test 4: Transfert réel simple
test_real_upload_single() {
    test_start "Transfert réel d'un fichier simple"
    
    local test_file="$TEST_DATA_DIR/simple.txt"
    
    if "$XSSHEND_BIN" upload "$test_file" --env Development --dest "/tmp/" 2>&1; then
        test_success "Upload simple réussi"
        
        # Vérifier que le fichier est présent sur la VM
        local dev_ip=$(multipass info xsshend-dev | grep IPv4 | awk '{print $2}' 2>/dev/null)
        if [[ -n "$dev_ip" ]]; then
            if ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no \
                -i "$SCRIPT_DIR/.ssh/test_key" "xsshend-test@$dev_ip" \
                "test -f /tmp/simple.txt" 2>/dev/null; then
                test_success "Fichier présent sur la VM de destination"
            else
                test_failure "Fichier absent sur la VM de destination"
            fi
        fi
    else
        test_failure "Upload simple échoué"
    fi
}

# Test 5: Transfert multiple
test_real_upload_multiple() {
    test_start "Transfert de plusieurs fichiers"
    
    local files=(
        "$TEST_DATA_DIR/simple.txt"
        "$TEST_DATA_DIR/config.json"
        "$TEST_DATA_DIR/small-1kb.txt"
    )
    
    if "$XSSHEND_BIN" upload "${files[@]}" --env Staging --dest "/tmp/" 2>&1; then
        test_success "Upload multiple réussi"
    else
        test_failure "Upload multiple échoué"
    fi
}

# Test 6: Gros fichier avec progression
test_large_file_upload() {
    test_start "Transfert d'un gros fichier (test progression)"
    
    local large_file="$TEST_DATA_DIR/large-1mb.txt"
    
    if "$XSSHEND_BIN" upload "$large_file" --env Development --dest "/tmp/" 2>&1 | grep -q "B/"; then
        test_success "Transfert gros fichier avec progression réussi"
    else
        test_failure "Transfert gros fichier échoué"
    fi
}

# Test 7: Transfert parallèle multi-serveurs
test_parallel_upload() {
    test_start "Transfert parallèle vers plusieurs serveurs"
    
    local test_file="$TEST_DATA_DIR/config.json"
    
    # Upload vers Production (plusieurs VMs)
    if "$XSSHEND_BIN" upload "$test_file" --env Production --dest "/tmp/" 2>&1; then
        test_success "Transfert parallèle réussi"
    else
        test_failure "Transfert parallèle échoué"
    fi
}

# Test 8: Gestion d'erreurs
test_error_handling() {
    test_start "Gestion d'erreurs"
    
    # Test avec fichier inexistant
    if ! "$XSSHEND_BIN" upload "/fichier/inexistant.txt" --env Development 2>/dev/null; then
        test_success "Erreur fichier inexistant détectée correctement"
    else
        test_failure "Erreur fichier inexistant non détectée"
    fi
    
    # Test avec environnement inexistant
    if ! "$XSSHEND_BIN" upload "$TEST_DATA_DIR/simple.txt" --env EnvironnementInexistant 2>/dev/null; then
        test_success "Erreur environnement inexistant détectée correctement"
    else
        test_failure "Erreur environnement inexistant non détectée"
    fi
}

# Test 9: Performance et stress
test_performance() {
    test_start "Test de performance (stress test)"
    
    log_info "Transfert de 5 fichiers simultanément..."
    
    local files=(
        "$TEST_DATA_DIR/simple.txt"
        "$TEST_DATA_DIR/config.json"
        "$TEST_DATA_DIR/deploy.sh"
        "$TEST_DATA_DIR/index.html"
        "$TEST_DATA_DIR/medium-100kb.txt"
    )
    
    local start_time=$(date +%s)
    
    if "$XSSHEND_BIN" upload "${files[@]}" --env Development --dest "/opt/uploads/" 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        test_success "Test de performance réussi (${duration}s)"
    else
        test_failure "Test de performance échoué"
    fi
}

# Nettoyage après tests
cleanup_test_files() {
    log_info "Nettoyage des fichiers de test sur les VMs..."
    
    for vm_name in $(multipass list | grep "xsshend.*Running" | awk '{print $1}'); do
        local vm_ip=$(multipass info "$vm_name" | grep IPv4 | awk '{print $2}' 2>/dev/null)
        if [[ -n "$vm_ip" ]]; then
            ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no \
                -i "$SCRIPT_DIR/.ssh/test_key" "xsshend-test@$vm_ip" \
                "rm -f /tmp/*.txt /tmp/*.json /tmp/*.html /tmp/*.sh /opt/uploads/*" 2>/dev/null || true
        fi
    done
}

# Affichage du résumé final
show_test_summary() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📊 RÉSUMÉ DES TESTS xsshend"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  Total:   $TESTS_TOTAL tests"
    echo "  ✅ Réussis: $TESTS_PASSED"
    echo "  ❌ Échecs:  $TESTS_FAILED"
    echo ""
    
    if [[ $TESTS_FAILED -eq 0 ]]; then
        log_success "🎉 TOUS LES TESTS SONT PASSÉS!"
        echo "  xsshend v0.1.0 est prêt pour la production!"
    else
        log_error "⚠️  CERTAINS TESTS ONT ÉCHOUÉ"
        echo "  Ratio de réussite: $(( (TESTS_PASSED * 100) / TESTS_TOTAL ))%"
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# Main - Exécution des tests
main() {
    echo "🧪 Suite de tests d'intégration xsshend"
    echo "Date: $(date)"
    echo "Version: $(cd "$PROJECT_ROOT" && git describe --tags 2>/dev/null || echo "dev")"
    echo ""
    
    check_prerequisites
    
    # Génération des fichiers de test si nécessaire
    if [[ ! -f "$TEST_DATA_DIR/simple.txt" ]]; then
        log_info "Génération des fichiers de test..."
        "$SCRIPT_DIR/generate-test-files.sh"
    fi
    
    # Exécution des tests
    test_cli_help
    test_hosts_config  
    test_dry_run
    test_real_upload_single
    test_real_upload_multiple
    test_large_file_upload
    test_parallel_upload
    test_error_handling
    test_performance
    
    cleanup_test_files
    show_test_summary
    
    # Code de sortie basé sur les résultats
    exit $TESTS_FAILED
}

main "$@"
