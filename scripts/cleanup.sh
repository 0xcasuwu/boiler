#!/bin/bash

echo "==========================================="
echo "🧹 Project Cleanup and Script Organization"
echo "==========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Get repository root directory
ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"

# Create legacy-scripts directory if it doesn't exist
if [ ! -d "${ROOT_DIR}/legacy-scripts" ]; then
    echo -e "${BLUE}Creating legacy-scripts directory...${NC}"
    mkdir -p "${ROOT_DIR}/legacy-scripts"
    echo -e "${GREEN}✓ Created legacy-scripts directory${NC}"
fi

# Move redundant build scripts to legacy-scripts
move_to_legacy() {
    local file=$1
    
    if [ -f "${ROOT_DIR}/$file" ]; then
        echo -e "${BLUE}Moving $file to legacy-scripts...${NC}"
        mv "${ROOT_DIR}/$file" "${ROOT_DIR}/legacy-scripts/"
        echo -e "${GREEN}✓ Moved $file to legacy-scripts${NC}"
    else
        echo -e "${YELLOW}⚠️ $file not found, skipping...${NC}"
    fi
}

# Create a symlink for backward compatibility
create_symlink() {
    local original=$1
    local target=$2
    
    if [ -f "${ROOT_DIR}/scripts/$target" ]; then
        echo -e "${BLUE}Creating symlink from $original to scripts/$target...${NC}"
        ln -sf "${ROOT_DIR}/scripts/$target" "${ROOT_DIR}/$original"
        echo -e "${GREEN}✓ Created symlink for $original${NC}"
    else
        echo -e "${RED}❌ Target script scripts/$target not found!${NC}"
    fi
}

# Move build scripts to legacy-scripts
move_to_legacy "build_minimal.sh"
move_to_legacy "build_with_fork.sh"
move_to_legacy "final_fork_build.sh"
move_to_legacy "test_oylnet_connection.sh"
move_to_legacy "deploy_to_oylnet.sh"
move_to_legacy "interact_with_vault.sh"
move_to_legacy "repo_check.sh"

# Create symlinks for backward compatibility
create_symlink "build_minimal.sh" "build.sh"
create_symlink "build_with_fork.sh" "build.sh"
create_symlink "final_fork_build.sh" "build.sh"
create_symlink "test_oylnet_connection.sh" "network.sh"
create_symlink "deploy_to_oylnet.sh" "network.sh"
create_symlink "interact_with_vault.sh" "network.sh"
create_symlink "repo_check.sh" "repo_check.sh"

echo ""
echo -e "${GREEN}${BOLD}Cleanup complete!${NC}"
echo ""
echo -e "${BLUE}New script organization:${NC}"
echo -e "  scripts/build.sh    - Unified build script"
echo -e "  scripts/network.sh  - Unified network operations script"
echo -e "  scripts/repo_check.sh - Repository structure validator"
echo ""
echo -e "${YELLOW}Legacy scripts have been moved to legacy-scripts/ directory.${NC}"
echo -e "${YELLOW}Symlinks have been created for backward compatibility.${NC}"
echo ""
echo -e "${BLUE}To build:${NC} ./scripts/build.sh"
echo -e "${BLUE}To deploy:${NC} ./scripts/network.sh --deploy"
echo -e "${BLUE}To interact:${NC} ./scripts/network.sh --interact"
echo ""
