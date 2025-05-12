#!/bin/bash

# =============================================
# 🧹 YieldVault Deployment Cleanup Script
# =============================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Get repository root directory
ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../.." >/dev/null 2>&1 && pwd )"

echo -e "${BOLD}${BLUE}YieldVault Deployment Cleanup Script${NC}"
echo ""
echo -e "${YELLOW}This script will clean up deployment artifacts and Docker resources.${NC}"
echo -e "This is useful for starting fresh or before committing changes."
echo ""

# Ask for confirmation
read -p "Do you want to proceed with cleanup? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Cleanup canceled.${NC}"
    exit 0
fi

# Step 1: Clean up contract ID
echo -e "\n${BLUE}Removing contract ID file...${NC}"
if [ -f "$ROOT_DIR/.contract_id" ]; then
    rm "$ROOT_DIR/.contract_id"
    echo -e "${GREEN}✅ Contract ID file removed.${NC}"
else
    echo -e "${YELLOW}Contract ID file not found, skipping.${NC}"
fi

# Step 2: Clean up build directory
echo -e "\n${BLUE}Cleaning build directory...${NC}"
if [ -d "$ROOT_DIR/build" ]; then
    rm -rf "$ROOT_DIR/build"
    mkdir -p "$ROOT_DIR/build"
    echo -e "${GREEN}✅ Build directory cleaned.${NC}"
else
    echo -e "${YELLOW}Build directory not found, creating it...${NC}"
    mkdir -p "$ROOT_DIR/build"
fi

# Step 3: Clean up Docker resources
echo -e "\n${BLUE}Cleaning Docker resources...${NC}"
if command -v docker &> /dev/null; then
    if [ -d "$ROOT_DIR/docker" ]; then
        cd "$ROOT_DIR/docker"
        if command -v docker-compose &> /dev/null; then
            echo -e "${BLUE}Stopping and removing Docker containers...${NC}"
            docker-compose down -v --remove-orphans
            echo -e "${GREEN}✅ Docker containers removed.${NC}"
        else
            echo -e "${YELLOW}docker-compose not found, skipping container cleanup.${NC}"
        fi
    else
        echo -e "${YELLOW}Docker directory not found, skipping container cleanup.${NC}"
    fi
else
    echo -e "${YELLOW}Docker not found, skipping container cleanup.${NC}"
fi

# Step 4: Clean target/wasm directory (optional)
echo -e "\n${BLUE}Do you want to clean WebAssembly build artifacts?${NC}"
echo -e "${YELLOW}This will remove all target/wasm32* directories.${NC}"
read -p "Clean WebAssembly artifacts? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${BLUE}Cleaning WebAssembly build artifacts...${NC}"
    rm -rf "$ROOT_DIR/target/wasm32-unknown-unknown"
    echo -e "${GREEN}✅ WebAssembly build artifacts removed.${NC}"
fi

echo -e "\n${GREEN}${BOLD}Cleanup completed successfully!${NC}"
echo -e "The deployment environment has been reset. You can now start fresh."
