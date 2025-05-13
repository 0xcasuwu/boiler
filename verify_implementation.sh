#!/bin/bash

# ANSI color codes for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

echo -e "${BOLD}${BLUE}=== YieldVault Implementation Verification ===${NC}\n"

# Build the WebAssembly contract
echo -e "${YELLOW}Building WebAssembly contract...${NC}"
cargo build --target wasm32-unknown-unknown --release --lib

if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to build WebAssembly contract.${NC}"
    exit 1
fi

echo -e "${GREEN}WebAssembly contract built successfully.${NC}"

# Run the simple test
echo -e "\n${YELLOW}Running simple test...${NC}"
cargo run --bin simple_test --target=x86_64-unknown-linux-gnu

if [ $? -ne 0 ]; then
    echo -e "${RED}Simple test failed.${NC}"
    exit 1
fi

echo -e "${GREEN}Simple test passed successfully.${NC}"

# Check the size of the WebAssembly file
echo -e "\n${YELLOW}Checking WebAssembly file size...${NC}"
WASM_SIZE=$(du -h target/wasm32-unknown-unknown/release/yield_vault.wasm | cut -f1)
echo -e "WebAssembly file size: ${CYAN}${WASM_SIZE}${NC}"

echo -e "\n${GREEN}${BOLD}Implementation verification completed successfully.${NC}"
echo -e "${CYAN}The declare_alkane macro has been successfully implemented and tested.${NC}"
