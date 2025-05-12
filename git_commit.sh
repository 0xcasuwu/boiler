#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Committing YieldVault OylNet Deployment Changes ===${NC}"

# Add all changes including new files
echo -e "${YELLOW}Adding all changes to git...${NC}"
git add .

# Show status
echo -e "${YELLOW}Current git status:${NC}"
git status

# Commit the changes with a descriptive message
echo -e "${YELLOW}Committing changes...${NC}"
git commit -m "YieldVault OylNet Deployment Success

- Successfully deployed contract to OylNet network
- Contract ID: 7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f
- Initialized contract and verified view functions
- Updated documentation in memory-bank
- Organized deployment scripts and cleaned up repository"

# Try to push changes if remote is configured
echo -e "${YELLOW}Attempting to push changes...${NC}"
git push 2>/dev/null || echo -e "${YELLOW}Could not push automatically. Please push manually when ready.${NC}"

echo -e "${GREEN}Git commit completed successfully!${NC}"
echo -e "${BLUE}=== Done ===${NC}"
