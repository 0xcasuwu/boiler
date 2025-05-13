#!/bin/bash

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

# Path to the WebAssembly file
WASM_PATH="$(dirname "$0")/../target/wasm32-unknown-unknown/release/yield_vault.wasm"
CONTRACT_DETAILS_FILE="contract_details.json"
CONTRACT_ID=""

# Function to deploy a fresh contract
deploy_contract() {
  echo -e "\n${YELLOW}Deploying fresh contract...${NC}"
  
  # Check if the WebAssembly file exists
  if [ ! -f "$WASM_PATH" ]; then
    echo -e "${RED}WebAssembly file not found at $WASM_PATH${NC}"
    echo -e "${YELLOW}Building WebAssembly file...${NC}"
    cd .. && cargo build --target wasm32-unknown-unknown --release
    cd - > /dev/null
    
    if [ ! -f "$WASM_PATH" ]; then
      echo -e "${RED}Failed to build WebAssembly file${NC}"
      exit 1
    fi
  fi
  
  echo -e "${CYAN}Using WebAssembly file: $WASM_PATH${NC}"
  
  # Deploy the contract and capture the output
  echo -e "${YELLOW}Deploying contract to OylNet...${NC}"
  DEPLOY_OUTPUT=$(NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract \
    --contract "$WASM_PATH" \
    --provider "oylnet" \
    --calldata "0" \
    --feeRate 10)
  
  # Extract contract ID from the output
  CONTRACT_ID=$(echo "$DEPLOY_OUTPUT" | grep -o -E '[a-f0-9]{64}' | head -1)
  
  if [ -z "$CONTRACT_ID" ]; then
    echo -e "${RED}Failed to extract contract ID from deployment output${NC}"
    echo -e "${YELLOW}Deployment output:${NC}"
    echo "$DEPLOY_OUTPUT"
    exit 1
  fi
  
  echo -e "${GREEN}Contract deployed successfully!${NC}"
  echo -e "${CYAN}Contract ID: $CONTRACT_ID${NC}"
  
  # Update contract_details.json with the new contract ID
  echo -e "${YELLOW}Updating contract details file...${NC}"
  if [ -f "$CONTRACT_DETAILS_FILE" ]; then
    # Update existing file
    sed -i "s/\"contractId\": \"[a-f0-9]*\"/\"contractId\": \"$CONTRACT_ID\"/" "$CONTRACT_DETAILS_FILE"
  else
    # Create new file
    echo "{\"contractId\": \"$CONTRACT_ID\", \"status\": \"DEPLOYED\"}" > "$CONTRACT_DETAILS_FILE"
  fi
  
  echo -e "${GREEN}Contract details updated${NC}"
  
  # Generate blocks to confirm deployment
  echo -e "${YELLOW}Generating blocks to confirm deployment...${NC}"
  NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10
  
  return 0
}

# Function to initialize the contract
initialize_contract() {
  if [ -z "$CONTRACT_ID" ]; then
    # Try to read contract ID from file
    if [ -f "$CONTRACT_DETAILS_FILE" ]; then
      CONTRACT_ID=$(grep -o -E '"contractId": "[a-f0-9]+"' "$CONTRACT_DETAILS_FILE" | grep -o -E '[a-f0-9]{64}')
    fi
    
    if [ -z "$CONTRACT_ID" ]; then
      echo -e "${RED}No contract ID found. Please deploy a contract first.${NC}"
      return 1
    fi
  fi
  
  echo -e "\n${YELLOW}Initializing contract: $CONTRACT_ID${NC}"
  
  # Generate blocks to ensure chain activity
  echo -e "${YELLOW}Generating blocks...${NC}"
  NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10
  
  # Try to execute initialization (opcode 0)
  echo -e "${YELLOW}Executing initialization...${NC}"
  INIT_OUTPUT=$(NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0" --provider oylnet)
  
  echo "$INIT_OUTPUT"
  
  # Generate confirmation blocks
  echo -e "${YELLOW}Generating confirmation blocks...${NC}"
  NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10
  
  # Check contract state - get name (opcode 100)
  echo -e "${YELLOW}Checking contract name (opcode 100)...${NC}"
  NAME_OUTPUT=$(NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "100" --provider oylnet)
  
  echo "$NAME_OUTPUT"
  
  # Update contract status in details file
  if [ -f "$CONTRACT_DETAILS_FILE" ]; then
    sed -i 's/"status": "DEPLOYED"/"status": "INITIALIZED"/' "$CONTRACT_DETAILS_FILE"
  fi
  
  echo -e "${GREEN}Contract initialized successfully${NC}"
  return 0
}

# Function to verify contract state
verify_contract() {
  if [ -z "$CONTRACT_ID" ]; then
    # Try to read contract ID from file
    if [ -f "$CONTRACT_DETAILS_FILE" ]; then
      CONTRACT_ID=$(grep -o -E '"contractId": "[a-f0-9]+"' "$CONTRACT_DETAILS_FILE" | grep -o -E '[a-f0-9]{64}')
    fi
    
    if [ -z "$CONTRACT_ID" ]; then
      echo -e "${RED}No contract ID found. Please deploy a contract first.${NC}"
      return 1
    fi
  fi
  
  echo -e "\n${YELLOW}Verifying contract state: $CONTRACT_ID${NC}"
  
  # Check if contract_interaction.js exists
  if [ ! -f "$(dirname "$0")/contract_interaction.js" ]; then
    echo -e "${RED}contract_interaction.js not found${NC}"
    return 1
  fi
  
  # Update contract ID in contract_interaction.js
  sed -i "s/const CONTRACT_ID = '[a-f0-9]*'/const CONTRACT_ID = '$CONTRACT_ID'/" "$(dirname "$0")/contract_interaction.js"
  
  # Run verification
  node "$(dirname "$0")/contract_interaction.js"
  
  return 0
}

# Function to generate blocks
generate_blocks() {
  echo -e "\n${YELLOW}Generating blocks...${NC}"
  read -p "Number of blocks to generate: " blocks
  NODE_OPTIONS=--require=/workspaces/boiler/oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c "$blocks"
  return 0
}

# Main menu
echo -e "${BLUE}=== YieldVault OylNet Deployment Tool ===${NC}"

# Try to read existing contract ID
if [ -f "$CONTRACT_DETAILS_FILE" ]; then
  CONTRACT_ID=$(grep -o -E '"contractId": "[a-f0-9]+"' "$CONTRACT_DETAILS_FILE" | grep -o -E '[a-f0-9]{64}')
  if [ ! -z "$CONTRACT_ID" ]; then
    echo -e "Current Contract ID: ${CYAN}$CONTRACT_ID${NC}"
  fi
fi

echo -e "\nSelect an operation:"
echo "1. Deploy fresh contract"
echo "2. Initialize contract"
echo "3. Verify contract state"
echo "4. Generate blocks"
echo "5. Full deployment process (deploy + initialize + verify)"
echo "6. Exit"

read -p "Enter your choice: " choice

case $choice in
  1)
    deploy_contract
    ;;
  2)
    initialize_contract
    ;;
  3)
    verify_contract
    ;;
  4)
    generate_blocks
    ;;
  5)
    echo -e "\n${BLUE}=== Full Deployment Process ===${NC}"
    deploy_contract && 
    initialize_contract && 
    verify_contract
    ;;
  6)
    echo -e "\n${YELLOW}Exiting...${NC}"
    exit 0
    ;;
  *)
    echo -e "\n${RED}Invalid option${NC}"
    exit 1
    ;;
esac

echo -e "\n${GREEN}Operation completed${NC}"
