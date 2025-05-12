#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BLUE}${BOLD}===== Committing and Pushing Changes to Git =====${NC}"

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo -e "${RED}Error: git is not installed${NC}"
    exit 1
fi

# Check if the current directory is a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}Error: Not a git repository${NC}"
    exit 1
fi

# Add all changes
echo -e "${YELLOW}Adding all changes...${NC}"
git add .

# Commit changes
echo -e "${YELLOW}Committing changes...${NC}"
git commit -m "Update deployment process and test organization

- Updated deployment scripts to deploy fresh contracts
- Pruned deprecated test files
- Added setup guide and updated documentation
- Enhanced yield-vault.sh script with new commands"

# Push changes
echo -e "${YELLOW}Pushing changes to remote repository...${NC}"
git push

# Check if push was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}${BOLD}Changes successfully pushed to remote repository${NC}"
else
    echo -e "${RED}${BOLD}Failed to push changes to remote repository${NC}"
    echo -e "${YELLOW}You may need to set up your git credentials or resolve merge conflicts${NC}"
fi
