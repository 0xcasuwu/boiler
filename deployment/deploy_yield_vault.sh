#!/bin/bash

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Contract ID from successful deployment
CONTRACT_ID="7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f"

echo -e "${BLUE}=== YieldVault OylNet Deployment Tool ===${NC}"
echo -e "Contract ID: ${CONTRACT_ID}\n"

# Menu options
echo "Select an operation:"
echo "1. Deploy contract (requires WebAssembly file)"
echo "2. Initialize contract"
echo "3. Verify contract state"
echo "4. Generate blocks"
echo "5. Exit"

read -p "Enter your choice: " choice

case $choice in
  1)
    echo -e "\n${YELLOW}Deploying contract...${NC}"
    echo "This will use a pre-built WebAssembly file from the build directory."
    
    # Check if the WebAssembly file exists
    if [ ! -f "../build/yield_vault.wasm" ]; then
      echo -e "${RED}WebAssembly file not found${NC}"
      exit 1
    fi
    
    # Deploy the contract
    NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract \
      --contract "../build/yield_vault.wasm" \
      --provider "oylnet" \
      --calldata "0,YieldVault,YVT,Bitcoin,BTC,8" \
      --feeRate 10
    ;;
    
  2)
    echo -e "\n${YELLOW}Initializing contract...${NC}"
    ../init_contract.sh
    ;;
    
  3)
    echo -e "\n${YELLOW}Verifying contract state...${NC}"
    node ../contract_interaction.js
    ;;
    
  4)
    echo -e "\n${YELLOW}Generating blocks...${NC}"
    read -p "Number of blocks to generate: " blocks
    NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c "$blocks"
    ;;
    
  5)
    echo -e "\n${YELLOW}Exiting...${NC}"
    exit 0
    ;;
    
  *)
    echo -e "\n${RED}Invalid option${NC}"
    exit 1
    ;;
esac

echo -e "\n${GREEN}Operation completed${NC}"
