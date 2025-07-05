#!/bin/bash

# Démonstration rapide de xsshend avec Multipass
# Script tout-en-un pour une démo complète

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Couleurs
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

demo_step() {
    echo ""
    echo -e "${BLUE}🎬 $1${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    sleep 2
}

demo_success() {
    echo -e "${GREEN}✅ $1${NC}"
    sleep 1
}

demo_info() {
    echo -e "${YELLOW}ℹ️  $1${NC}"
}

show_banner() {
    cat << 'EOF'
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║                    🚀 DÉMONSTRATION xsshend                     ║
║                                                                  ║
║              Téléversement Multi-SSH avec Multipass             ║
║                                                                  ║
║                        Version 0.1.0                           ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
EOF
    echo ""
}

check_multipass() {
    if ! command -v multipass &> /dev/null; then
        echo "❌ Multipass n'est pas installé!"
        echo ""
        echo "Installation :"
        echo "  Ubuntu/Debian: sudo snap install multipass --classic"
        echo "  macOS:         brew install multipass"
        echo "  Windows:       https://multipass.run/"
        exit 1
    fi
}

demo_main() {
    show_banner
    
    demo_step "Vérification des prérequis"
    check_multipass
    demo_success "Multipass installé et fonctionnel"
    
    demo_step "Compilation de xsshend"
    cd "$PROJECT_ROOT"
    cargo build --quiet
    demo_success "xsshend compilé avec succès"
    
    demo_step "Génération des clés SSH de test"
    cd "$SCRIPT_DIR"
    ./test-vms.sh generate-keys
    demo_success "Clés SSH générées"
    
    demo_step "Mise à jour de la configuration cloud-init"
    ./test-vms.sh update-cloud-init
    demo_success "Configuration cloud-init prête"
    
    demo_step "Lancement des VMs de test (cela peut prendre quelques minutes...)"
    demo_info "Création de 5 VMs Ubuntu 22.04 avec SSH configuré"
    ./test-vms.sh launch-all
    demo_success "VMs lancées et configurées"
    
    demo_step "Génération de la configuration hosts.json"
    ./test-vms.sh generate-config
    demo_success "Configuration hosts.json générée"
    
    demo_step "Test des connexions SSH"
    ./test-vms.sh test-ssh
    demo_success "Connexions SSH validées"
    
    demo_step "Génération des fichiers de test"
    ./generate-test-files.sh > /dev/null
    demo_success "Fichiers de test créés"
    
    demo_step "Démonstration de xsshend en action"
    
    echo ""
    demo_info "1. Test dry-run (simulation)"
    cd "$PROJECT_ROOT"
    ./target/debug/xsshend upload test/data/simple.txt --env Development --dry-run
    
    echo ""
    demo_info "2. Upload réel d'un fichier simple"
    HOME="$SCRIPT_DIR/configs" ./target/debug/xsshend upload test/data/simple.txt --env Development
    
    echo ""
    demo_info "3. Upload de plusieurs fichiers vers Production"
    HOME="$SCRIPT_DIR/configs" ./target/debug/xsshend upload test/data/config.json test/data/deploy.sh --env Production --dest "/opt/uploads/"
    
    echo ""
    demo_info "4. Test avec un gros fichier (barres de progression)"
    HOME="$SCRIPT_DIR/configs" ./target/debug/xsshend upload test/data/large-1mb.txt --env Staging
    
    demo_step "Exécution de la suite de tests d'intégration"
    HOME="$SCRIPT_DIR/configs" ./run-integration-tests.sh
    
    demo_step "Nettoyage (optionnel)"
    read -p "Voulez-vous supprimer les VMs de test ? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        ./test-vms.sh destroy-all
        demo_success "VMs supprimées"
    else
        demo_info "VMs conservées pour exploration manuelle"
        echo ""
        echo "Commandes utiles :"
        echo "  multipass list                    # Lister les VMs"
        echo "  multipass shell xsshend-dev       # Connexion à une VM"
        echo "  ./test-vms.sh stop-all           # Arrêter les VMs"
        echo "  ./test-vms.sh destroy-all        # Supprimer les VMs"
    fi
    
    cat << 'EOF'

╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║                      🎉 DÉMONSTRATION TERMINÉE !                ║
║                                                                  ║
║  xsshend v0.1.0 a été testé avec succès dans un environnement   ║
║  multi-VM Multipass. Toutes les fonctionnalités principales     ║
║  ont été validées :                                              ║
║                                                                  ║
║  ✅ Configuration hiérarchique hosts.json                       ║
║  ✅ Interface CLI intuitive                                     ║
║  ✅ Transferts SSH/SFTP parallèles                              ║
║  ✅ Barres de progression en temps réel                         ║
║  ✅ Gestion d'erreurs robuste                                   ║
║  ✅ Mode dry-run pour simulation                                ║
║                                                                  ║
║              Prêt pour la production ! 🚀                       ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝

EOF
}

# Gestion des interruptions
trap 'echo -e "\n${YELLOW}⚠️  Démonstration interrompue${NC}"; exit 1' INT TERM

demo_main "$@"
