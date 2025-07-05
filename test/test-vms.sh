#!/bin/bash

# Script de gestion des VMs de test pour xsshend avec Multipass
# Usage: ./test-vms.sh [command] [options]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CLOUD_INIT_FILE="$SCRIPT_DIR/multipass/cloud-init.yaml"
HOSTS_CONFIG_FILE="$SCRIPT_DIR/configs/test-hosts.json"
SSH_KEY_DIR="$SCRIPT_DIR/.ssh"

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration des VMs
declare -A VMS=(
    ["xsshend-dev"]="ubuntu-22.04 --cpus 1 --mem 1G --disk 5G"
    ["xsshend-staging"]="ubuntu-22.04 --cpus 1 --mem 1G --disk 5G" 
    ["xsshend-prod-web"]="ubuntu-22.04 --cpus 2 --mem 2G --disk 10G"
    ["xsshend-prod-api"]="ubuntu-22.04 --cpus 2 --mem 2G --disk 10G"
    ["xsshend-prod-db"]="ubuntu-22.04 --cpus 1 --mem 2G --disk 8G"
)

# Fonctions d'affichage
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Génération des clés SSH de test
generate_ssh_keys() {
    log_info "Génération des clés SSH de test..."
    
    mkdir -p "$SSH_KEY_DIR"
    
    # Clé principale de test
    if [[ ! -f "$SSH_KEY_DIR/test_key" ]]; then
        ssh-keygen -t ed25519 -f "$SSH_KEY_DIR/test_key" -N "" -C "xsshend-test-key"
        log_success "Clé SSH principale générée"
    fi
    
    # Clé de déploiement
    if [[ ! -f "$SSH_KEY_DIR/deploy_key" ]]; then
        ssh-keygen -t ed25519 -f "$SSH_KEY_DIR/deploy_key" -N "" -C "xsshend-deploy-key"
        log_success "Clé SSH de déploiement générée"
    fi
    
    # Clé API
    if [[ ! -f "$SSH_KEY_DIR/api_key" ]]; then
        ssh-keygen -t ed25519 -f "$SSH_KEY_DIR/api_key" -N "" -C "xsshend-api-key"
        log_success "Clé SSH API générée"
    fi
    
    # Afficher les clés publiques pour la configuration cloud-init
    log_info "Clés publiques générées:"
    echo "  Test key: $(cat "$SSH_KEY_DIR/test_key.pub")"
    echo "  Deploy key: $(cat "$SSH_KEY_DIR/deploy_key.pub")"
    echo "  API key: $(cat "$SSH_KEY_DIR/api_key.pub")"
}

# Mise à jour du fichier cloud-init avec les vraies clés
update_cloud_init() {
    log_info "Mise à jour du fichier cloud-init avec les clés SSH..."
    
    if [[ ! -f "$SSH_KEY_DIR/test_key.pub" ]]; then
        log_error "Les clés SSH n'existent pas. Exécutez d'abord 'generate-keys'"
        exit 1
    fi
    
    local test_key=$(cat "$SSH_KEY_DIR/test_key.pub")
    local deploy_key=$(cat "$SSH_KEY_DIR/deploy_key.pub")
    local api_key=$(cat "$SSH_KEY_DIR/api_key.pub")
    
    # Créer une version temporaire du cloud-init avec les vraies clés
    sed -e "s|ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAA... xsshend-test-key|$test_key|g" \
        -e "s|ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAA... deploy-key|$deploy_key|g" \
        -e "s|ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAA... api-key|$api_key|g" \
        "$CLOUD_INIT_FILE" > "$CLOUD_INIT_FILE.tmp"
    
    mv "$CLOUD_INIT_FILE.tmp" "$CLOUD_INIT_FILE.processed"
    log_success "Fichier cloud-init mis à jour"
}

# Lancement d'une VM
launch_vm() {
    local vm_name="$1"
    local vm_config="${VMS[$vm_name]}"
    
    log_info "Lancement de la VM: $vm_name"
    
    if multipass list | grep -q "$vm_name"; then
        log_warning "La VM $vm_name existe déjà"
        return
    fi
    
    # Utiliser le cloud-init avec les vraies clés si disponible
    local cloud_init_option=""
    if [[ -f "$CLOUD_INIT_FILE.processed" ]]; then
        cloud_init_option="--cloud-init $CLOUD_INIT_FILE.processed"
    fi
    
    multipass launch $vm_config --name "$vm_name" $cloud_init_option
    
    log_success "VM $vm_name lancée avec succès"
}

# Lancement de toutes les VMs
launch_all() {
    log_info "Lancement de toutes les VMs de test..."
    
    for vm_name in "${!VMS[@]}"; do
        launch_vm "$vm_name"
    done
    
    log_success "Toutes les VMs ont été lancées"
    list_vms
}

# Liste des VMs
list_vms() {
    log_info "État des VMs de test:"
    echo ""
    multipass list | grep -E "(Name|xsshend-)" || log_warning "Aucune VM xsshend trouvée"
}

# Arrêt des VMs
stop_all() {
    log_info "Arrêt de toutes les VMs xsshend..."
    
    for vm_name in "${!VMS[@]}"; do
        if multipass list | grep -q "$vm_name.*Running"; then
            log_info "Arrêt de $vm_name..."
            multipass stop "$vm_name"
        fi
    done
    
    log_success "Toutes les VMs ont été arrêtées"
}

# Démarrage des VMs
start_all() {
    log_info "Démarrage de toutes les VMs xsshend..."
    
    for vm_name in "${!VMS[@]}"; do
        if multipass list | grep -q "$vm_name.*Stopped"; then
            log_info "Démarrage de $vm_name..."
            multipass start "$vm_name"
        fi
    done
    
    log_success "Toutes les VMs ont été démarrées"
}

# Suppression des VMs
destroy_all() {
    log_warning "⚠️  ATTENTION: Cette action va supprimer TOUTES les VMs de test!"
    read -p "Êtes-vous sûr ? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Suppression de toutes les VMs xsshend..."
        
        for vm_name in "${!VMS[@]}"; do
            if multipass list | grep -q "$vm_name"; then
                log_info "Suppression de $vm_name..."
                multipass delete "$vm_name"
            fi
        done
        
        log_info "Purge des VMs supprimées..."
        multipass purge
        
        log_success "Toutes les VMs ont été supprimées"
    else
        log_info "Suppression annulée"
    fi
}

# Génération de la configuration hosts.json pour les tests
generate_hosts_config() {
    log_info "Génération de la configuration hosts.json de test..."
    
    mkdir -p "$(dirname "$HOSTS_CONFIG_FILE")"
    
    # Récupérer les IPs des VMs
    local dev_ip=$(multipass info xsshend-dev | grep IPv4 | awk '{print $2}' 2>/dev/null || echo "127.0.0.1")
    local staging_ip=$(multipass info xsshend-staging | grep IPv4 | awk '{print $2}' 2>/dev/null || echo "127.0.0.1") 
    local web_ip=$(multipass info xsshend-prod-web | grep IPv4 | awk '{print $2}' 2>/dev/null || echo "127.0.0.1")
    local api_ip=$(multipass info xsshend-prod-api | grep IPv4 | awk '{print $2}' 2>/dev/null || echo "127.0.0.1")
    local db_ip=$(multipass info xsshend-prod-db | grep IPv4 | awk '{print $2}' 2>/dev/null || echo "127.0.0.1")
    
    cat > "$HOSTS_CONFIG_FILE" << EOF
{
  "environments": {
    "Development": {
      "regions": {
        "Local": {
          "types": {
            "Test": {
              "DEV_TEST_VM": {
                "alias": "xsshend-test@$dev_ip",
                "description": "VM de développement Multipass"
              }
            }
          }
        }
      }
    },
    "Staging": {
      "regions": {
        "Test": {
          "types": {
            "Validation": {
              "STAGING_VM": {
                "alias": "deploy@$staging_ip", 
                "description": "VM de staging Multipass"
              }
            }
          }
        }
      }
    },
    "Production": {
      "regions": {
        "Test-Region": {
          "types": {
            "Web": {
              "PROD_WEB_VM": {
                "alias": "deploy@$web_ip",
                "description": "VM web de production (test)"
              }
            },
            "API": {
              "PROD_API_VM": {
                "alias": "api@$api_ip",
                "description": "VM API de production (test)"
              }
            },
            "Database": {
              "PROD_DB_VM": {
                "alias": "xsshend-test@$db_ip",
                "description": "VM base de données (test)"
              }
            }
          }
        }
      }
    }
  }
}
EOF
    
    log_success "Configuration hosts.json générée: $HOSTS_CONFIG_FILE"
    log_info "IPs détectées:"
    echo "  Development: $dev_ip"
    echo "  Staging: $staging_ip"
    echo "  Prod Web: $web_ip"
    echo "  Prod API: $api_ip"
    echo "  Prod DB: $db_ip"
}

# Test de connexion SSH
test_ssh() {
    log_info "Test des connexions SSH vers les VMs..."
    
    local success=0
    local total=0
    
    for vm_name in "${!VMS[@]}"; do
        if multipass list | grep -q "$vm_name.*Running"; then
            total=$((total + 1))
            local vm_ip=$(multipass info "$vm_name" | grep IPv4 | awk '{print $2}')
            
            log_info "Test SSH vers $vm_name ($vm_ip)..."
            
            if ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=no -i "$SSH_KEY_DIR/test_key" "xsshend-test@$vm_ip" "echo 'SSH OK'" 2>/dev/null; then
                log_success "SSH OK vers $vm_name"
                success=$((success + 1))
            else
                log_error "SSH échoué vers $vm_name"
            fi
        fi
    done
    
    log_info "Résultats: $success/$total connexions SSH réussies"
}

# Affichage de l'aide
show_help() {
    cat << EOF
🚀 Gestionnaire de VMs de test pour xsshend

Usage: $0 [COMMAND]

COMMANDS:
    generate-keys     Génère les clés SSH de test
    update-cloud-init Met à jour cloud-init avec les vraies clés SSH
    launch-all        Lance toutes les VMs de test
    start-all         Démarre toutes les VMs
    stop-all          Arrête toutes les VMs  
    destroy-all       Supprime toutes les VMs (avec confirmation)
    list              Affiche l'état des VMs
    generate-config   Génère hosts.json avec les IPs des VMs
    test-ssh          Teste les connexions SSH vers les VMs
    help              Affiche cette aide

WORKFLOW RECOMMANDÉ:
    1. $0 generate-keys        # Génération des clés SSH
    2. $0 update-cloud-init    # Mise à jour cloud-init
    3. $0 launch-all           # Lancement des VMs
    4. $0 generate-config      # Génération de la config
    5. $0 test-ssh             # Test des connexions

VMs CONFIGURÉES:
$(for vm in "${!VMS[@]}"; do echo "    - $vm: ${VMS[$vm]}"; done)

FICHIERS:
    - Cloud-init: $CLOUD_INIT_FILE
    - Config test: $HOSTS_CONFIG_FILE
    - Clés SSH: $SSH_KEY_DIR/
EOF
}

# Main
main() {
    case "${1:-help}" in
        "generate-keys")
            generate_ssh_keys
            ;;
        "update-cloud-init")
            update_cloud_init
            ;;
        "launch-all")
            update_cloud_init
            launch_all
            ;;
        "start-all")
            start_all
            ;;
        "stop-all")
            stop_all
            ;;
        "destroy-all")
            destroy_all
            ;;
        "list")
            list_vms
            ;;
        "generate-config")
            generate_hosts_config
            ;;
        "test-ssh")
            test_ssh
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# Vérification que Multipass est installé
if ! command -v multipass &> /dev/null; then
    log_error "Multipass n'est pas installé. Installez-le avec:"
    echo "  sudo snap install multipass --classic"
    exit 1
fi

main "$@"
