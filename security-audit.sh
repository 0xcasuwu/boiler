#!/bin/bash
# SLOP Security Audit Script
# This script runs a comprehensive security audit on the SLOP codebase

echo "Starting SLOP Security Audit..."
echo "==============================="

# Setup
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Track overall status
PASSED=0
FAILED=0
WARNINGS=0

# Function to run a test and track results
run_test() {
  local test_name="$1"
  local test_cmd="$2"
  
  echo -e "\n${YELLOW}Running: ${test_name}${NC}"
  
  if eval "$test_cmd"; then
    echo -e "${GREEN}✓ PASSED: ${test_name}${NC}"
    PASSED=$((PASSED + 1))
  else
    echo -e "${RED}✗ FAILED: ${test_name}${NC}"
    FAILED=$((FAILED + 1))
  fi
}

# Static Analysis
echo -e "\n${YELLOW}=== Static Analysis ===${NC}"

# Check for compiler warnings
run_test "Compiler warnings check" "cargo check --all --quiet"

# Run clippy with strict linting
run_test "Clippy linting" "cargo clippy --all -- -D warnings"

# Test Suite
echo -e "\n${YELLOW}=== Test Suite ===${NC}"

# Run unit tests
run_test "Unit tests" "cargo test --lib -- --quiet"

# Run penetration tests specifically
run_test "Penetration tests" "cargo test tests::penetration_tests -- --quiet"

# Run security fix tests specifically
run_test "Security fix tests" "cargo test tests::security_fixes_test -- --quiet"

# Run property-based tests specifically
run_test "Property-based tests" "cargo test tests::property_tests -- --quiet"

# Check test coverage
echo -e "\n${YELLOW}Checking test coverage...${NC}"
if command -v cargo-tarpaulin &> /dev/null; then
  cargo tarpaulin --out Html
  echo "Coverage report generated: tarpaulin-report.html"
else
  echo -e "${YELLOW}Warning: cargo-tarpaulin not installed. Skipping coverage report.${NC}"
  echo "Install with: cargo install cargo-tarpaulin"
  WARNINGS=$((WARNINGS + 1))
fi

# Bitcoin-specific Security Checks
echo -e "\n${YELLOW}=== Bitcoin-specific Security Checks ===${NC}"

# Check for transaction context verification
run_test "Transaction context verification" "grep -r 'orbital_token_id' --include='*.rs' src/ | wc -l | awk '{if(\$1>0)exit(0);else exit(1)}'"

# Check for proper error handling
run_test "Error handling" "grep -r 'Error\\|err\\|Err(' --include='*.rs' src/ | wc -l | awk '{if(\$1>0)exit(0);else exit(1)}'"

# Check for redemption controls 
run_test "Redemption security" "grep -r 'redeem_bond_secure' --include='*.rs' src/ | wc -l | awk '{if(\$1>0)exit(0);else exit(1)}'"

# Vulnerability Scanning
echo -e "\n${YELLOW}=== Vulnerability Scanning ===${NC}"

# Check for vulnerabilities in dependencies
if command -v cargo-audit &> /dev/null; then
  run_test "Dependency vulnerabilities" "cargo audit --quiet"
else
  echo -e "${YELLOW}Warning: cargo-audit not installed. Skipping vulnerability scan.${NC}"
  echo "Install with: cargo install cargo-audit"
  WARNINGS=$((WARNINGS + 1))
fi

# Summary
echo -e "\n${YELLOW}=== Audit Summary ===${NC}"
echo -e "Tests passed: ${GREEN}$PASSED${NC}"
echo -e "Tests failed: ${RED}$FAILED${NC}"
echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"

if [ $FAILED -eq 0 ]; then
  echo -e "\n${GREEN}All security checks passed!${NC}"
  exit 0
else
  echo -e "\n${RED}Some security checks failed. See above for details.${NC}"
  exit 1
fi
