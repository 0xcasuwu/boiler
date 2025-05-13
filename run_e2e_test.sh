#!/bin/bash

# ANSI color codes for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BOLD}${BLUE}=== YieldVault End-to-End Test Runner ===${NC}\n"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Node.js is not installed. Please install Node.js to run this test.${NC}"
    exit 1
fi

# Check if the OYL SDK is available
if [ ! -d "/workspaces/boiler/oyl-sdk" ]; then
    echo -e "${RED}OYL SDK not found. Please ensure the SDK is available at /workspaces/boiler/oyl-sdk.${NC}"
    exit 1
fi

# Build the WebAssembly contract if it doesn't exist
if [ ! -f "/workspaces/boiler/target/wasm32-unknown-unknown/release/yield_vault.wasm" ]; then
    echo -e "${YELLOW}Building WebAssembly contract...${NC}"
    cd /workspaces/boiler && cargo build --target wasm32-unknown-unknown --release

    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to build WebAssembly contract.${NC}"
        exit 1
    fi

    echo -e "${GREEN}WebAssembly contract built successfully.${NC}"
fi

# Run the end-to-end test
echo -e "${YELLOW}Running end-to-end test...${NC}"
NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js node /workspaces/boiler/deployment/e2e_test.js

if [ $? -ne 0 ]; then
    echo -e "${RED}End-to-end test failed.${NC}"
    exit 1
fi

echo -e "${GREEN}End-to-end test completed successfully.${NC}"
