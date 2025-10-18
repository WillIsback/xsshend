#!/bin/bash

# xsshend Lab Diagnostic Script
# Quick diagnostic tool for troubleshooting lab issues
# Version: 1.0

set +e  # Don't exit on errors - we want to collect all info

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_section() {
    echo -e "\n${BLUE}╔═══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║ $1${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════════╝${NC}\n"
}

print_ok() {
    echo -e "${GREEN}✓ OK:${NC} $1"
}

print_error() {
    echo -e "${RED}✗ ERROR:${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠ WARNING:${NC} $1"
}

print_info() {
    echo -e "${BLUE}ℹ INFO:${NC} $1"
}

# Main diagnostic
main() {
    print_section "xsshend Lab Diagnostic Tool - v1.0"
    
    echo "Date: $(date)"
    echo "Directory: $(pwd)"
    echo ""
    
    # 1. Docker Environment
    print_section "1. Docker Environment"
    
    if command -v docker &> /dev/null; then
        print_ok "Docker installed: $(docker --version)"
    else
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    if command -v docker-compose &> /dev/null; then
        print_ok "Docker Compose installed: $(docker-compose --version)"
    else
        print_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi
    
    # 2. Container Status
    print_section "2. Container Status"
    
    master_running=$(docker ps --filter "name=xsshend_master" --format "{{.Status}}" 2>/dev/null)
    if [ -n "$master_running" ]; then
        print_ok "Master container: $master_running"
    else
        print_error "Master container is not running"
        docker ps -a | grep xsshend_master || print_error "Master container not found"
    fi
    
    target1_running=$(docker ps --filter "name=xsshend_target1" --format "{{.Status}}" 2>/dev/null)
    if [ -n "$target1_running" ]; then
        print_ok "Target1 container: $target1_running"
    else
        print_error "Target1 container is not running"
        docker ps -a | grep xsshend_target1 || print_error "Target1 container not found"
    fi
    
    target2_running=$(docker ps --filter "name=xsshend_target2" --format "{{.Status}}" 2>/dev/null)
    if [ -n "$target2_running" ]; then
        print_ok "Target2 container: $target2_running"
    else
        print_error "Target2 container is not running"
        docker ps -a | grep xsshend_target2 || print_error "Target2 container not found"
    fi
    
    # 3. Network
    print_section "3. Docker Network"
    
    if docker network ls | grep -q xsshend_net; then
        print_ok "Network xsshend_net exists"
        network_info=$(docker network inspect xsshend_net -f '{{.IPAM.Config}}' 2>/dev/null)
        print_info "Network config: $network_info"
    else
        print_error "Network xsshend_net does not exist"
    fi
    
    # 4. xsshend Installation
    print_section "4. xsshend Installation"
    
    if [ -n "$master_running" ]; then
        xsshend_version=$(docker exec xsshend_master xsshend --version 2>&1)
        if [ $? -eq 0 ]; then
            print_ok "xsshend installed: $xsshend_version"
        else
            print_error "xsshend not found or not working"
            echo "Error output: $xsshend_version"
        fi
        
        xsshend_path=$(docker exec xsshend_master which xsshend 2>/dev/null)
        if [ -n "$xsshend_path" ]; then
            print_info "xsshend path: $xsshend_path"
        fi
    else
        print_warning "Cannot check xsshend - master container not running"
    fi
    
    # 5. SSH Keys
    print_section "5. SSH Keys Configuration"
    
    if [ -n "$master_running" ]; then
        echo "Master SSH directory contents:"
        docker exec xsshend_master ls -la /home/master/.ssh/ 2>/dev/null
        
        # Check individual keys
        if docker exec xsshend_master test -f /home/master/.ssh/id_rsa; then
            perms=$(docker exec xsshend_master stat -c '%a' /home/master/.ssh/id_rsa)
            if [ "$perms" = "600" ]; then
                print_ok "id_rsa exists with correct permissions (600)"
            else
                print_warning "id_rsa exists but has permissions $perms (should be 600)"
            fi
        else
            print_error "id_rsa not found"
        fi
        
        if docker exec xsshend_master test -f /home/master/.ssh/id_ed25519; then
            perms=$(docker exec xsshend_master stat -c '%a' /home/master/.ssh/id_ed25519)
            if [ "$perms" = "600" ]; then
                print_ok "id_ed25519 exists with correct permissions (600)"
            else
                print_warning "id_ed25519 exists but has permissions $perms (should be 600)"
            fi
        else
            print_error "id_ed25519 not found"
        fi
        
        if docker exec xsshend_master test -f /home/master/.ssh/hosts.json; then
            print_ok "hosts.json exists"
            
            # Validate JSON
            if docker exec xsshend_master cat /home/master/.ssh/hosts.json | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
                print_ok "hosts.json is valid JSON"
            else
                print_error "hosts.json is invalid JSON"
            fi
        else
            print_error "hosts.json not found"
        fi
    else
        print_warning "Cannot check SSH keys - master container not running"
    fi
    
    # 6. SSH Daemons
    print_section "6. SSH Daemons Status"
    
    if [ -n "$target1_running" ]; then
        if docker exec xsshend_target1 pgrep -x sshd > /dev/null 2>&1; then
            sshd_count=$(docker exec xsshend_target1 pgrep -x sshd | wc -l)
            print_ok "sshd running on target1 ($sshd_count processes)"
        else
            print_error "sshd NOT running on target1"
        fi
    else
        print_warning "Cannot check sshd - target1 not running"
    fi
    
    if [ -n "$target2_running" ]; then
        if docker exec xsshend_target2 pgrep -x sshd > /dev/null 2>&1; then
            sshd_count=$(docker exec xsshend_target2 pgrep -x sshd | wc -l)
            print_ok "sshd running on target2 ($sshd_count processes)"
        else
            print_error "sshd NOT running on target2"
        fi
    else
        print_warning "Cannot check sshd - target2 not running"
    fi
    
    # 7. Network Connectivity
    print_section "7. Network Connectivity"
    
    if [ -n "$master_running" ] && [ -n "$target1_running" ]; then
        if docker exec xsshend_master ping -c 2 -W 2 target1 > /dev/null 2>&1; then
            print_ok "Master can ping target1"
        else
            print_error "Master CANNOT ping target1"
        fi
    else
        print_warning "Cannot test connectivity - containers not running"
    fi
    
    if [ -n "$master_running" ] && [ -n "$target2_running" ]; then
        if docker exec xsshend_master ping -c 2 -W 2 target2 > /dev/null 2>&1; then
            print_ok "Master can ping target2"
        else
            print_error "Master CANNOT ping target2"
        fi
    else
        print_warning "Cannot test connectivity - containers not running"
    fi
    
    # 8. SSH Connectivity
    print_section "8. SSH Connectivity"
    
    if [ -n "$master_running" ] && [ -n "$target1_running" ]; then
        ssh_result=$(docker exec xsshend_master ssh -i /home/master/.ssh/id_rsa -o StrictHostKeyChecking=no -o ConnectTimeout=5 -o BatchMode=yes testuser@target1 "hostname" 2>&1)
        if echo "$ssh_result" | grep -q "target1"; then
            print_ok "SSH to target1 works (RSA key)"
        else
            print_error "SSH to target1 FAILED"
            print_info "SSH error: $ssh_result"
        fi
    else
        print_warning "Cannot test SSH - containers not running"
    fi
    
    if [ -n "$master_running" ] && [ -n "$target2_running" ]; then
        ssh_result=$(docker exec xsshend_master ssh -i /home/master/.ssh/id_rsa -o StrictHostKeyChecking=no -o ConnectTimeout=5 -o BatchMode=yes testuser@target2 "hostname" 2>&1)
        if echo "$ssh_result" | grep -q "target2"; then
            print_ok "SSH to target2 works (RSA key)"
        else
            print_error "SSH to target2 FAILED"
            print_info "SSH error: $ssh_result"
        fi
    else
        print_warning "Cannot test SSH - containers not running"
    fi
    
    # 9. Authorized Keys on Targets
    print_section "9. Authorized Keys on Targets"
    
    if [ -n "$target1_running" ]; then
        if docker exec xsshend_target1 test -f /home/testuser/.ssh/authorized_keys; then
            key_count=$(docker exec xsshend_target1 wc -l < /home/testuser/.ssh/authorized_keys)
            print_ok "authorized_keys exists on target1 ($key_count keys)"
            
            perms=$(docker exec xsshend_target1 stat -c '%a' /home/testuser/.ssh/authorized_keys)
            if [ "$perms" = "600" ] || [ "$perms" = "644" ]; then
                print_ok "Permissions on target1 authorized_keys: $perms"
            else
                print_warning "Permissions on target1 authorized_keys: $perms (should be 600 or 644)"
            fi
        else
            print_error "authorized_keys NOT found on target1"
        fi
    fi
    
    if [ -n "$target2_running" ]; then
        if docker exec xsshend_target2 test -f /home/testuser/.ssh/authorized_keys; then
            key_count=$(docker exec xsshend_target2 wc -l < /home/testuser/.ssh/authorized_keys)
            print_ok "authorized_keys exists on target2 ($key_count keys)"
            
            perms=$(docker exec xsshend_target2 stat -c '%a' /home/testuser/.ssh/authorized_keys)
            if [ "$perms" = "600" ] || [ "$perms" = "644" ]; then
                print_ok "Permissions on target2 authorized_keys: $perms"
            else
                print_warning "Permissions on target2 authorized_keys: $perms (should be 600 or 644)"
            fi
        else
            print_error "authorized_keys NOT found on target2"
        fi
    fi
    
    # 10. xsshend list Command
    print_section "10. xsshend list Command"
    
    if [ -n "$master_running" ]; then
        list_output=$(docker exec xsshend_master xsshend list 2>&1)
        if echo "$list_output" | grep -q "TARGET1"; then
            server_count=$(echo "$list_output" | grep -c "TARGET")
            print_ok "xsshend list works ($server_count servers found)"
            echo ""
            echo "$list_output"
        else
            print_error "xsshend list FAILED or no servers found"
            echo "Output:"
            echo "$list_output"
        fi
    else
        print_warning "Cannot test xsshend list - master not running"
    fi
    
    # 11. Disk Space
    print_section "11. Disk Space"
    
    if [ -n "$master_running" ]; then
        master_disk=$(docker exec xsshend_master df -h /tmp | tail -1 | awk '{print $4}')
        print_info "Master /tmp available: $master_disk"
    fi
    
    if [ -n "$target1_running" ]; then
        target1_disk=$(docker exec xsshend_target1 df -h /tmp | tail -1 | awk '{print $4}')
        print_info "Target1 /tmp available: $target1_disk"
    fi
    
    if [ -n "$target2_running" ]; then
        target2_disk=$(docker exec xsshend_target2 df -h /tmp | tail -1 | awk '{print $4}')
        print_info "Target2 /tmp available: $target2_disk"
    fi
    
    # 12. Recent Container Logs (last 10 lines)
    print_section "12. Recent Container Logs (Last 10 Lines)"
    
    echo -e "${YELLOW}Master logs:${NC}"
    docker logs xsshend_master --tail 10 2>&1 || echo "No logs available"
    
    echo ""
    echo -e "${YELLOW}Target1 logs:${NC}"
    docker logs xsshend_target1 --tail 10 2>&1 || echo "No logs available"
    
    echo ""
    echo -e "${YELLOW}Target2 logs:${NC}"
    docker logs xsshend_target2 --tail 10 2>&1 || echo "No logs available"
    
    # Summary
    print_section "Diagnostic Summary"
    
    echo "For more detailed troubleshooting, see:"
    echo "  - docs/LAB-TROUBLESHOOTING.md"
    echo "  - docs/LAB-TESTING-GUIDE.md"
    echo "  - docs/LAB-README.md"
    echo ""
    echo "To save this diagnostic output:"
    echo "  $0 > diagnostic_report_\$(date +%Y%m%d_%H%M%S).txt 2>&1"
    echo ""
}

# Run diagnostic
main "$@"
