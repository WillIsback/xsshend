#!/bin/bash

# xsshend Lab Test Script
# Automated testing suite for xsshend in Docker lab environment
# Version: 1.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Functions
print_header() {
    echo -e "\n${BLUE}════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}════════════════════════════════════════════════════════${NC}\n"
}

print_test() {
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    echo -e "${YELLOW}[TEST $TESTS_TOTAL]${NC} $1"
}

print_success() {
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo -e "${GREEN}✓ PASSED${NC} $1\n"
}

print_failure() {
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo -e "${RED}✗ FAILED${NC} $1\n"
}

print_info() {
    echo -e "${BLUE}ℹ INFO:${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠ WARNING:${NC} $1"
}

run_test() {
    local test_name=$1
    local test_command=$2
    local expected_exit_code=${3:-0}
    
    print_test "$test_name"
    
    if eval "$test_command" > /tmp/test_output.log 2>&1; then
        actual_exit_code=0
    else
        actual_exit_code=$?
    fi
    
    if [ $actual_exit_code -eq $expected_exit_code ]; then
        print_success "$test_name"
        return 0
    else
        print_failure "$test_name (exit code: $actual_exit_code, expected: $expected_exit_code)"
        echo "Command output:"
        cat /tmp/test_output.log
        return 1
    fi
}

# Main test suite
main() {
    print_header "xsshend Lab Test Suite - v0.4.1"
    
    # Check if Docker is available
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}ERROR: Docker is not installed or not in PATH${NC}"
        exit 1
    fi
    
    # Check if docker-compose is available
    if ! command -v docker-compose &> /dev/null; then
        echo -e "${RED}ERROR: docker-compose is not installed or not in PATH${NC}"
        exit 1
    fi
    
    print_info "Starting Docker environment checks..."
    
    # Test 1: Check if containers are running
    print_test "Verify master container is running"
    if docker ps | grep -q xsshend_master; then
        print_success "Master container is running"
    else
        print_failure "Master container is not running"
        print_info "Run: docker-compose up -d --build"
        exit 1
    fi
    
    print_test "Verify target1 container is running"
    if docker ps | grep -q xsshend_target1; then
        print_success "Target1 container is running"
    else
        print_failure "Target1 container is not running"
        exit 1
    fi
    
    print_test "Verify target2 container is running"
    if docker ps | grep -q xsshend_target2; then
        print_success "Target2 container is running"
    else
        print_failure "Target2 container is not running"
        exit 1
    fi
    
    # Test 2: Check xsshend installation
    print_header "Testing xsshend Installation"
    
    print_test "Check xsshend is installed in master"
    if docker exec xsshend_master which xsshend > /dev/null 2>&1; then
        print_success "xsshend is installed"
        version=$(docker exec xsshend_master xsshend --version 2>&1 | head -1)
        print_info "Version: $version"
    else
        print_failure "xsshend is not installed"
        exit 1
    fi
    
    # Test 3: Check SSH keys
    print_header "Testing SSH Keys Configuration"
    
    print_test "Check RSA private key exists"
    if docker exec xsshend_master test -f /home/master/.ssh/id_rsa; then
        print_success "RSA private key found"
    else
        print_failure "RSA private key not found"
    fi
    
    print_test "Check RSA public key exists"
    if docker exec xsshend_master test -f /home/master/.ssh/id_rsa.pub; then
        print_success "RSA public key found"
    else
        print_failure "RSA public key not found"
    fi
    
    print_test "Check Ed25519 private key exists"
    if docker exec xsshend_master test -f /home/master/.ssh/id_ed25519; then
        print_success "Ed25519 private key found"
    else
        print_failure "Ed25519 private key not found"
    fi
    
    print_test "Check hosts.json exists"
    if docker exec xsshend_master test -f /home/master/.ssh/hosts.json; then
        print_success "hosts.json configuration found"
    else
        print_failure "hosts.json not found"
    fi
    
    # Test 4: Check SSH daemon on targets
    print_header "Testing SSH Daemons"
    
    print_test "Check sshd is running on target1"
    if docker exec xsshend_target1 pgrep -x sshd > /dev/null; then
        print_success "sshd is running on target1"
    else
        print_failure "sshd is not running on target1"
    fi
    
    print_test "Check sshd is running on target2"
    if docker exec xsshend_target2 pgrep -x sshd > /dev/null; then
        print_success "sshd is running on target2"
    else
        print_failure "sshd is not running on target2"
    fi
    
    # Test 5: Manual SSH connectivity
    print_header "Testing Manual SSH Connectivity"
    
    print_test "SSH to target1 with RSA key"
    if docker exec xsshend_master ssh -i /home/master/.ssh/id_rsa -o StrictHostKeyChecking=no -o ConnectTimeout=5 testuser@target1 "hostname" 2>/dev/null | grep -q target1; then
        print_success "Manual SSH to target1 works"
    else
        print_failure "Manual SSH to target1 failed"
        print_info "Checking target1 logs..."
        docker exec xsshend_target1 journalctl -u sshd -n 20 2>/dev/null || echo "Logs not available"
    fi
    
    print_test "SSH to target2 with RSA key"
    if docker exec xsshend_master ssh -i /home/master/.ssh/id_rsa -o StrictHostKeyChecking=no -o ConnectTimeout=5 testuser@target2 "hostname" 2>/dev/null | grep -q target2; then
        print_success "Manual SSH to target2 works"
    else
        print_failure "Manual SSH to target2 failed"
        print_info "Checking target2 logs..."
        docker exec xsshend_target2 journalctl -u sshd -n 20 2>/dev/null || echo "Logs not available"
    fi
    
    # Test 6: xsshend list command
    print_header "Testing xsshend list Command"
    
    print_test "List all servers"
    if docker exec xsshend_master xsshend list 2>&1 | grep -q "TARGET1"; then
        print_success "xsshend list shows servers"
    else
        print_failure "xsshend list failed or no servers found"
    fi
    
    # Test 7: Create test file for uploads
    print_header "Preparing Test Files"
    
    print_test "Create test file"
    if docker exec xsshend_master bash -c "echo 'xsshend test v0.4.1' > /tmp/test_upload.txt"; then
        print_success "Test file created"
    else
        print_failure "Failed to create test file"
    fi
    
    # Test 8: xsshend upload dry-run
    print_header "Testing xsshend Upload (Dry-Run)"
    
    print_test "Upload dry-run to Test environment"
    if docker exec xsshend_master xsshend upload /tmp/test_upload.txt --env Test --dry-run 2>&1 | grep -q "Simulation"; then
        print_success "Dry-run completed successfully"
    else
        print_failure "Dry-run failed"
    fi
    
    # Test 9: xsshend upload real
    print_header "Testing xsshend Upload (Real)"
    
    print_test "Upload to RSA-Targets only"
    docker exec xsshend_master xsshend upload /tmp/test_upload.txt --env Test --server-type RSA-Targets > /tmp/upload_output.log 2>&1
    upload_exit=$?
    
    if [ $upload_exit -eq 0 ]; then
        print_success "Upload command completed"
    else
        print_warning "Upload command returned non-zero exit code: $upload_exit"
        cat /tmp/upload_output.log
    fi
    
    # Test 10: Verify files on targets
    print_header "Verifying Files on Targets"
    
    print_test "Check file exists on target1"
    if docker exec xsshend_target1 test -f /tmp/test_upload.txt; then
        content=$(docker exec xsshend_target1 cat /tmp/test_upload.txt)
        if echo "$content" | grep -q "xsshend test v0.4.1"; then
            print_success "File correctly uploaded to target1 with correct content"
        else
            print_failure "File on target1 has incorrect content: $content"
        fi
    else
        print_failure "File not found on target1"
    fi
    
    print_test "Check file exists on target2"
    if docker exec xsshend_target2 test -f /tmp/test_upload.txt; then
        content=$(docker exec xsshend_target2 cat /tmp/test_upload.txt)
        if echo "$content" | grep -q "xsshend test v0.4.1"; then
            print_success "File correctly uploaded to target2 with correct content"
        else
            print_failure "File on target2 has incorrect content: $content"
        fi
    else
        print_failure "File not found on target2"
    fi
    
    # Test 11: Multi-file upload
    print_header "Testing Multi-File Upload"
    
    print_test "Create multiple test files"
    docker exec xsshend_master bash -c "for i in {1..3}; do echo 'File \$i' > /tmp/file\$i.txt; done"
    if [ $? -eq 0 ]; then
        print_success "Multiple test files created"
    else
        print_failure "Failed to create multiple test files"
    fi
    
    print_test "Upload multiple files"
    docker exec xsshend_master xsshend upload /tmp/file1.txt /tmp/file2.txt /tmp/file3.txt --env Test --server-type RSA-Targets > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        print_success "Multi-file upload completed"
    else
        print_warning "Multi-file upload returned non-zero exit code"
    fi
    
    print_test "Verify all files on target1"
    files_ok=true
    for i in {1..3}; do
        if ! docker exec xsshend_target1 test -f /tmp/file$i.txt; then
            files_ok=false
        fi
    done
    
    if [ "$files_ok" = true ]; then
        print_success "All files found on target1"
    else
        print_failure "Some files missing on target1"
    fi
    
    # Test 12: Check SSH logs
    print_header "Checking SSH Logs on Targets"
    
    print_test "Check target1 SSH logs for successful authentications"
    docker exec xsshend_target1 journalctl -u sshd -n 50 2>/dev/null | grep "Accepted publickey" > /tmp/target1_auth.log || true
    auth_count=$(wc -l < /tmp/target1_auth.log || echo 0)
    
    if [ "$auth_count" -gt 0 ]; then
        print_success "Found $auth_count successful authentications on target1"
        print_info "Last 3 successful authentications:"
        tail -3 /tmp/target1_auth.log
    else
        print_warning "No successful authentication logs found on target1 (logs might not be available)"
    fi
    
    # Final summary
    print_header "Test Summary"
    
    echo -e "Total tests: ${BLUE}$TESTS_TOTAL${NC}"
    echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "\n${GREEN}╔════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║  ✓ ALL TESTS PASSED SUCCESSFULLY ✓   ║${NC}"
        echo -e "${GREEN}╚════════════════════════════════════════╝${NC}\n"
        exit 0
    else
        echo -e "\n${RED}╔════════════════════════════════════════╗${NC}"
        echo -e "${RED}║  ✗ SOME TESTS FAILED ✗                ║${NC}"
        echo -e "${RED}╚════════════════════════════════════════╝${NC}\n"
        exit 1
    fi
}

# Cleanup function
cleanup() {
    print_info "Cleaning up test files..."
    docker exec xsshend_master rm -f /tmp/test_upload.txt /tmp/file*.txt 2>/dev/null || true
    docker exec xsshend_target1 rm -f /tmp/test_upload.txt /tmp/file*.txt 2>/dev/null || true
    docker exec xsshend_target2 rm -f /tmp/test_upload.txt /tmp/file*.txt 2>/dev/null || true
    rm -f /tmp/test_output.log /tmp/upload_output.log /tmp/target1_auth.log
}

# Trap for cleanup on exit
trap cleanup EXIT

# Run main test suite
main "$@"
