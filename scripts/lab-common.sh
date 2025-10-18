#!/bin/bash
# Common functions and variables for xsshend lab scripts
# Version: 1.0
# Usage: source "$(dirname "${BASH_SOURCE[0]}")/lab-common.sh"

# ============================================================================
# COLORS
# ============================================================================
export RED='\033[0;31m'
export GREEN='\033[0;32m'
export YELLOW='\033[1;33m'
export BLUE='\033[0;34m'
export CYAN='\033[0;36m'
export MAGENTA='\033[0;35m'
export NC='\033[0m' # No Color

# ============================================================================
# CONFIGURATION (centralisée)
# ============================================================================
export LAB_CONTAINER_PREFIX="${LAB_CONTAINER_PREFIX:-xsshend}"
export LAB_MASTER_NAME="${LAB_MASTER_NAME:-${LAB_CONTAINER_PREFIX}_master}"
export LAB_TARGET1_NAME="${LAB_TARGET1_NAME:-${LAB_CONTAINER_PREFIX}_target1}"
export LAB_TARGET2_NAME="${LAB_TARGET2_NAME:-${LAB_CONTAINER_PREFIX}_target2}"
export LAB_NETWORK="${LAB_NETWORK:-${LAB_CONTAINER_PREFIX}_net}"
export LAB_VOLUME="${LAB_VOLUME:-master_home}"

# Paths (dans les conteneurs)
export LAB_MASTER_USER="${LAB_MASTER_USER:-master}"
export LAB_TARGET_USER="${LAB_TARGET_USER:-testuser}"
export LAB_MASTER_HOME="/home/${LAB_MASTER_USER}"
export LAB_TARGET_HOME="/home/${LAB_TARGET_USER}"
export LAB_MASTER_SSH_DIR="${LAB_MASTER_HOME}/.ssh"
export LAB_TARGET_SSH_DIR="${LAB_TARGET_HOME}/.ssh"
export LAB_TMP_DIR="${LAB_TMP_DIR:-/tmp}"

# Paths (sur l'hôte)
export LAB_HOST_SSH_DIR="${LAB_HOST_SSH_DIR:-./ssh_keys}"
export LAB_HOST_AUTHORIZED_KEYS="${LAB_HOST_AUTHORIZED_KEYS:-./authorized_keys}"

# SSH Keys
export LAB_RSA_KEY="id_rsa"
export LAB_ED25519_KEY="id_ed25519"
export LAB_ED25519_PASSPHRASE="${LAB_ED25519_PASSPHRASE:-testpassphrase}"

# Docker Compose
export LAB_COMPOSE_FILE="${LAB_COMPOSE_FILE:-docker-compose.yml}"
export LAB_MASTER_DOCKERFILE="${LAB_MASTER_DOCKERFILE:-Dockerfile.master}"
export LAB_TARGET_DOCKERFILE="${LAB_TARGET_DOCKERFILE:-Dockerfile.target}"

# ============================================================================
# PRINT FUNCTIONS
# ============================================================================

# Print header for major sections
print_header() {
    echo -e "\n${BLUE}════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}\n"
}

# Print section (with box)
print_section() {
    echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
    printf "${BLUE}║ %-61s ║${NC}\n" "$1"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}\n"
}

# Print success message
print_success() {
    echo -e "${GREEN}✓ SUCCESS:${NC} $1"
}

# Print OK message (for checks)
print_ok() {
    echo -e "${GREEN}✓ OK:${NC} $1"
}

# Print error message
print_error() {
    echo -e "${RED}✗ ERROR:${NC} $1" >&2
}

# Print failure message
print_failure() {
    echo -e "${RED}✗ FAILED:${NC} $1" >&2
}

# Print warning message
print_warning() {
    echo -e "${YELLOW}⚠ WARNING:${NC} $1"
}

# Print info message
print_info() {
    echo -e "${BLUE}ℹ INFO:${NC} $1"
}

# Print debug message (only if DEBUG=true)
print_debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        echo -e "${CYAN}🔍 DEBUG:${NC} $1"
    fi
}

# Print test name
print_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

# ============================================================================
# PREREQUISITE CHECKS
# ============================================================================

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        print_info "Install Docker: https://docs.docker.com/get-docker/"
        return 1
    fi
    print_debug "Docker found: $(docker --version)"
    return 0
}

# Check if Docker Compose is installed
check_docker_compose() {
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed or not in PATH"
        print_info "Install Docker Compose: https://docs.docker.com/compose/install/"
        return 1
    fi
    print_debug "Docker Compose found: $(docker-compose --version)"
    return 0
}

# Check if ssh-keygen is installed
check_ssh_keygen() {
    if ! command -v ssh-keygen &> /dev/null; then
        print_error "ssh-keygen is not installed or not in PATH"
        print_info "Install OpenSSH client package"
        return 1
    fi
    print_debug "ssh-keygen found: $(ssh-keygen -V 2>&1 | head -1)"
    return 0
}

# Check all prerequisites
check_prerequisites() {
    local all_ok=true
    
    print_section "Checking Prerequisites"
    
    if ! check_docker; then
        all_ok=false
    else
        print_ok "Docker installed"
    fi
    
    if ! check_docker_compose; then
        all_ok=false
    else
        print_ok "Docker Compose installed"
    fi
    
    if ! check_ssh_keygen; then
        all_ok=false
    else
        print_ok "ssh-keygen installed"
    fi
    
    if [[ "$all_ok" == "false" ]]; then
        print_error "Some prerequisites are missing"
        return 1
    fi
    
    print_success "All prerequisites found"
    return 0
}

# ============================================================================
# CONTAINER CHECKS
# ============================================================================

# Check if a container exists
container_exists() {
    local container_name=$1
    docker ps -a --filter "name=^${container_name}$" --format "{{.Names}}" 2>/dev/null | grep -q "^${container_name}$"
}

# Check if a container is running
is_container_running() {
    local container_name=$1
    docker ps --filter "name=^${container_name}$" --format "{{.Names}}" 2>/dev/null | grep -q "^${container_name}$"
}

# Get container status
get_container_status() {
    local container_name=$1
    docker ps -a --filter "name=^${container_name}$" --format "{{.Status}}" 2>/dev/null
}

# Check if all lab containers are running
check_lab_containers() {
    local all_running=true
    
    print_section "Checking Lab Containers"
    
    if is_container_running "$LAB_MASTER_NAME"; then
        print_ok "Master container is running: $(get_container_status "$LAB_MASTER_NAME")"
    else
        print_error "Master container is not running"
        all_running=false
    fi
    
    if is_container_running "$LAB_TARGET1_NAME"; then
        print_ok "Target1 container is running: $(get_container_status "$LAB_TARGET1_NAME")"
    else
        print_error "Target1 container is not running"
        all_running=false
    fi
    
    if is_container_running "$LAB_TARGET2_NAME"; then
        print_ok "Target2 container is running: $(get_container_status "$LAB_TARGET2_NAME")"
    else
        print_error "Target2 container is not running"
        all_running=false
    fi
    
    if [[ "$all_running" == "false" ]]; then
        print_warning "Start containers with: docker-compose up -d --build"
        return 1
    fi
    
    print_success "All lab containers are running"
    return 0
}

# ============================================================================
# DOCKER EXEC HELPERS
# ============================================================================

# Execute command in master container
exec_master() {
    if ! is_container_running "$LAB_MASTER_NAME"; then
        print_error "Master container is not running"
        return 1
    fi
    docker exec "$LAB_MASTER_NAME" "$@"
}

# Execute command in target1 container
exec_target1() {
    if ! is_container_running "$LAB_TARGET1_NAME"; then
        print_error "Target1 container is not running"
        return 1
    fi
    docker exec "$LAB_TARGET1_NAME" "$@"
}

# Execute command in target2 container
exec_target2() {
    if ! is_container_running "$LAB_TARGET2_NAME"; then
        print_error "Target2 container is not running"
        return 1
    fi
    docker exec "$LAB_TARGET2_NAME" "$@"
}

# Execute command in all target containers
exec_all_targets() {
    local exit_codes=()
    
    if is_container_running "$LAB_TARGET1_NAME"; then
        docker exec "$LAB_TARGET1_NAME" "$@"
        exit_codes+=($?)
    fi
    
    if is_container_running "$LAB_TARGET2_NAME"; then
        docker exec "$LAB_TARGET2_NAME" "$@"
        exit_codes+=($?)
    fi
    
    # Return 0 if all succeeded
    for code in "${exit_codes[@]}"; do
        if [[ $code -ne 0 ]]; then
            return $code
        fi
    done
    return 0
}

# ============================================================================
# FILE CHECKS
# ============================================================================

# Check if file exists in master container
master_file_exists() {
    local filepath=$1
    exec_master test -f "$filepath" 2>/dev/null
}

# Check if file exists in target container
target_file_exists() {
    local target=$1
    local filepath=$2
    
    case $target in
        1|target1)
            exec_target1 test -f "$filepath" 2>/dev/null
            ;;
        2|target2)
            exec_target2 test -f "$filepath" 2>/dev/null
            ;;
        *)
            print_error "Invalid target: $target (use 1, 2, target1, or target2)"
            return 1
            ;;
    esac
}

# ============================================================================
# NETWORK CHECKS
# ============================================================================

# Check if Docker network exists
network_exists() {
    local network_name=${1:-$LAB_NETWORK}
    docker network ls --filter "name=^${network_name}$" --format "{{.Name}}" 2>/dev/null | grep -q "^${network_name}$"
}

# Check network connectivity between containers
check_network_connectivity() {
    local all_ok=true
    
    print_section "Checking Network Connectivity"
    
    if ! network_exists; then
        print_error "Network $LAB_NETWORK does not exist"
        return 1
    fi
    print_ok "Network $LAB_NETWORK exists"
    
    # Ping target1 from master
    if exec_master ping -c 2 -W 2 target1 > /dev/null 2>&1; then
        print_ok "Master can ping target1"
    else
        print_error "Master CANNOT ping target1"
        all_ok=false
    fi
    
    # Ping target2 from master
    if exec_master ping -c 2 -W 2 target2 > /dev/null 2>&1; then
        print_ok "Master can ping target2"
    else
        print_error "Master CANNOT ping target2"
        all_ok=false
    fi
    
    if [[ "$all_ok" == "false" ]]; then
        return 1
    fi
    
    print_success "Network connectivity OK"
    return 0
}

# ============================================================================
# SSH CHECKS
# ============================================================================

# Check SSH connectivity
check_ssh_connectivity() {
    local target=$1
    local key=${2:-$LAB_MASTER_SSH_DIR/$LAB_RSA_KEY}
    
    exec_master ssh -i "$key" \
        -o StrictHostKeyChecking=no \
        -o ConnectTimeout=5 \
        -o BatchMode=yes \
        "${LAB_TARGET_USER}@${target}" \
        "hostname" > /dev/null 2>&1
}

# ============================================================================
# CLEANUP FUNCTIONS
# ============================================================================

# Cleanup test files from master
cleanup_master_files() {
    local pattern=${1:-'test_*.txt file*.txt'}
    print_info "Cleaning up test files from master..."
    exec_master bash -c "rm -f ${LAB_TMP_DIR}/${pattern}" 2>/dev/null || true
}

# Cleanup test files from targets
cleanup_target_files() {
    local pattern=${1:-'test_*.txt file*.txt'}
    print_info "Cleaning up test files from targets..."
    exec_all_targets bash -c "rm -f ${LAB_TMP_DIR}/${pattern}" 2>/dev/null || true
}

# Cleanup all test files
cleanup_all_test_files() {
    local pattern=${1:-'test_*.txt file*.txt'}
    cleanup_master_files "$pattern"
    cleanup_target_files "$pattern"
}

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

# Get xsshend version from master
get_xsshend_version() {
    exec_master xsshend --version 2>&1 | head -1
}

# Validate JSON file
validate_json() {
    local json_file=$1
    
    if command -v jq &> /dev/null; then
        jq empty "$json_file" 2>/dev/null
        return $?
    elif command -v python3 &> /dev/null; then
        python3 -c "import json; json.load(open('$json_file'))" 2>/dev/null
        return $?
    else
        print_warning "Cannot validate JSON (jq or python3 not found)"
        return 0  # Assume valid if no validator
    fi
}

# Wait for container to be ready
wait_for_container() {
    local container_name=$1
    local max_wait=${2:-30}
    local counter=0
    
    print_info "Waiting for $container_name to be ready..."
    
    while [[ $counter -lt $max_wait ]]; do
        if is_container_running "$container_name"; then
            print_ok "$container_name is ready"
            return 0
        fi
        sleep 1
        ((counter++))
    done
    
    print_error "$container_name did not start within ${max_wait}s"
    return 1
}

# Wait for SSH daemon
wait_for_sshd() {
    local target=$1
    local max_wait=${2:-30}
    local counter=0
    
    print_info "Waiting for sshd on $target..."
    
    while [[ $counter -lt $max_wait ]]; do
        case $target in
            target1|1)
                if exec_target1 pgrep -x sshd > /dev/null 2>&1; then
                    print_ok "sshd is ready on target1"
                    return 0
                fi
                ;;
            target2|2)
                if exec_target2 pgrep -x sshd > /dev/null 2>&1; then
                    print_ok "sshd is ready on target2"
                    return 0
                fi
                ;;
        esac
        sleep 1
        ((counter++))
    done
    
    print_error "sshd did not start on $target within ${max_wait}s"
    return 1
}

# ============================================================================
# CONFIRMATION FUNCTIONS
# ============================================================================

# Ask for confirmation
confirm() {
    local message=${1:-"Continue?"}
    local default=${2:-"N"}
    
    if [[ "$default" == "Y" ]]; then
        local prompt="[Y/n]"
        local default_reply="Y"
    else
        local prompt="[y/N]"
        local default_reply="N"
    fi
    
    read -p "$message $prompt " -n 1 -r
    echo
    
    REPLY=${REPLY:-$default_reply}
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        return 0
    else
        return 1
    fi
}

# ============================================================================
# INITIALIZATION
# ============================================================================

# Print debug info if DEBUG is enabled
if [[ "${DEBUG:-false}" == "true" ]]; then
    print_debug "lab-common.sh loaded"
    print_debug "LAB_MASTER_NAME=$LAB_MASTER_NAME"
    print_debug "LAB_TARGET1_NAME=$LAB_TARGET1_NAME"
    print_debug "LAB_TARGET2_NAME=$LAB_TARGET2_NAME"
    print_debug "LAB_NETWORK=$LAB_NETWORK"
fi

# Export functions so they're available in subshells
export -f print_header print_section print_success print_ok print_error print_failure
export -f print_warning print_info print_debug print_test
export -f check_docker check_docker_compose check_ssh_keygen check_prerequisites
export -f container_exists is_container_running get_container_status check_lab_containers
export -f exec_master exec_target1 exec_target2 exec_all_targets
export -f master_file_exists target_file_exists
export -f network_exists check_network_connectivity check_ssh_connectivity
export -f cleanup_master_files cleanup_target_files cleanup_all_test_files
export -f get_xsshend_version validate_json wait_for_container wait_for_sshd confirm
