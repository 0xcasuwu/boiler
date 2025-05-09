#!/bin/bash

echo "==========================================="
echo "🔍 Repository Structure Validation Check"
echo "==========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check directory structure
check_directory() {
    local dir=$1
    local description=$2
    
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✅ $description found${NC}"
    else
        echo -e "${RED}❌ Missing $description${NC}"
    fi
}

# Check file existence
check_file() {
    local file=$1
    local description=$2
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✅ $description found${NC}"
    else
        echo -e "${RED}❌ Missing $description${NC}"
    fi
}

# Check if a file contains specific content
check_file_content() {
    local file=$1
    local pattern=$2
    local description=$3
    
    if [ -f "$file" ] && grep -q "$pattern" "$file"; then
        echo -e "${GREEN}✅ $description validated${NC}"
    else
        echo -e "${YELLOW}⚠️ $description validation failed${NC}"
    fi
}

# Main validation
echo "Checking critical directory structure..."
check_directory "src" "Source code directory"
check_directory "src/asset_management" "Asset management module" 
check_directory "src/security" "Security module"
check_directory "src/storage" "Storage module"
check_directory "src/utils" "Utils module"
check_directory "src/tests" "Tests directory"
check_directory "docs" "Documentation directory"
check_directory "fork-repos" "Fork repository directory"
check_directory "fork-repos/secp256k1-sys" "secp256k1-sys fork"

echo -e "\nChecking critical files..."
check_file "Cargo.toml" "Cargo manifest"
check_file "build.rs" "Build script"
check_file "src/lib.rs" "Library entry point"
check_file "src/constants.rs" "Constants module"
check_file ".cargo/config.toml" "Cargo configuration"
check_file "fork-repos/secp256k1-sys/Cargo.toml" "Fork manifest"
check_file "fork-repos/secp256k1-sys/build.rs" "Fork build script"

echo -e "\nChecking build and deployment scripts..."
check_file "final_fork_build.sh" "Main build script"
check_file "build_with_fork.sh" "Support build script"
check_file "deploy_to_oylnet.sh" "Deployment script"
check_file "test_oylnet_connection.sh" "Network test script"
check_file "interact_with_vault.sh" "Contract interaction script"

echo -e "\nChecking documentation files..."
check_file "docs/BUILD_AND_DEPLOY.md" "Build process documentation"
check_file "docs/TESTING.md" "Testing documentation"
check_file "docs/TECHNICAL_REFERENCE.md" "Technical documentation"
check_file "README.md" "Project overview"

echo -e "\nValidating key content patterns..."
check_file_content "src/asset_management/mod.rs" "YieldVault" "Asset management module implementation"
check_file_content "src/lib.rs" "AlkaneResponder" "AlkaneResponder trait usage"
check_file_content "src/storage/mod.rs" "StoragePointer" "Storage pointer implementation"
check_file_content "fork-repos/secp256k1-sys/Cargo.toml" "name = \"secp256k1-sys\"" "secp256k1-sys crate name"

echo -e "\n${CYAN}================ Repository Structure Validation Summary =================${NC}"
echo -e "${CYAN}This repository contains a yield-vault implementation with:${NC}"
echo -e "  - Modular architecture with separated concerns (asset management, security, storage)"
echo -e "  - Test suite organized by test types (unit, e2e, adversarial)"
echo -e "  - Custom secp256k1-sys fork for Apple Silicon compatibility"
echo -e "  - Deployment and interaction scripts for OylNet"
echo -e "  - Comprehensive documentation in docs directory"
echo -e "\n${CYAN}==================================================================${NC}"

# Count total files
TOTAL_FILES=$(find . -type f | wc -l)
RUST_FILES=$(find . -name "*.rs" | wc -l)
MARKDOWN_FILES=$(find . -name "*.md" | wc -l)
SH_FILES=$(find . -name "*.sh" | wc -l)

echo -e "\n${CYAN}Repository Statistics:${NC}"
echo -e "  - Total files: $TOTAL_FILES"
echo -e "  - Rust source files: $RUST_FILES"
echo -e "  - Documentation (markdown) files: $MARKDOWN_FILES"
echo -e "  - Shell scripts: $SH_FILES"

echo -e "\n${GREEN}Repository structure validation complete!${NC}"
