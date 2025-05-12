#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}===== Pruning Deprecated Test Files =====${NC}"

# List of active test files that should be kept
ACTIVE_TESTS=(
    "mock_vault_tests.rs"
    "simple_utils_test.rs"
)

# List of deprecated test files that should be moved to archive
DEPRECATED_TESTS=(
    "basic_vault_tests.rs"
    "mock_vault_advanced_penetration_tests.rs"
    "mock_vault_combined_tests.rs"
    "mock_vault_erc4626_specific_tests.rs"
    "mock_vault_invariants.rs"
    "mock_vault_penetration_tests.rs"
    "mock_vault_token_tests.rs"
    "native_tests.rs"
    "non_wasm_tests.rs"
)

# Create archive directory if it doesn't exist
ARCHIVE_DIR="archive/deprecated_tests"
mkdir -p "$ARCHIVE_DIR"

# Move deprecated test files to archive
for test in "${DEPRECATED_TESTS[@]}"; do
    if [ -f "tests/$test" ]; then
        echo -e "${YELLOW}Moving $test to archive...${NC}"
        mv "tests/$test" "$ARCHIVE_DIR/"
        echo -e "${GREEN}✓ Moved $test to $ARCHIVE_DIR/${NC}"
    else
        echo -e "${RED}File tests/$test not found${NC}"
    fi
done

echo -e "\n${BLUE}${BOLD}===== Summary =====${NC}"
echo -e "${GREEN}Deprecated test files have been moved to $ARCHIVE_DIR/${NC}"
echo -e "${YELLOW}Active test files remain in the tests/ directory:${NC}"
