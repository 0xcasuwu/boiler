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
git commit -m "Implement alkane ID verification tests and update documentation

- Created alkane_id_verification_tests.rs with 9 test cases
- Implemented RealisticMockVault to simulate transaction context
- Verified token-based architecture security model
- Updated documentation:
  * progress.md: Added alkane ID verification testing details
  * test_guide.md: Updated with new test file
  * asset_management_analysis.md: Added alkane ID verification section
  * oylnet_test_plan.md: Created detailed plan for OylNet testing
- All tests pass successfully with x86_64-unknown-linux-gnu target"

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
