#!/bin/bash

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

# Contract details file
CONTRACT_DETAILS_FILE="$(dirname "$0")/contract_details.json"
CONTRACT_ID=""

# Function to read contract ID from file
read_contract_id() {
  if [ -f "$CONTRACT_DETAILS_FILE" ]; then
    CONTRACT_ID=$(grep -o -E '"contractId": "[a-f0-9]+"' "$CONTRACT_DETAILS_FILE" | grep -o -E '[a-f0-9]{64}')
    if [ ! -z "$CONTRACT_ID" ]; then
      echo -e "${CYAN}Using contract ID from $CONTRACT_DETAILS_FILE: $CONTRACT_ID${NC}"
      return 0
    fi
  fi
  
  echo -e "${RED}No contract ID found in $CONTRACT_DETAILS_FILE${NC}"
  return 1
}

# Read contract ID from file or command line argument
if [ "$1" != "" ]; then
  CONTRACT_ID="$1"
  echo -e "${CYAN}Using contract ID from command line: $CONTRACT_ID${NC}"
else
  read_contract_id || {
    echo -e "${RED}No contract ID provided. Please deploy a contract first or provide a contract ID as an argument.${NC}"
    exit 1
  }
fi

echo -e "${BLUE}=== YieldVault Contract Initialization ===${NC}"
echo -e "Contract ID: ${CYAN}$CONTRACT_ID${NC}\n"

# Generate blocks to ensure chain activity
echo -e "${YELLOW}Generating blocks to ensure chain activity...${NC}"
NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10

# Try to execute initialization (opcode 0) with parameters
echo -e "${YELLOW}Trying initialization with basic format...${NC}"
INIT_OUTPUT=$(NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0" --provider oylnet)
echo "$INIT_OUTPUT"

# Try to execute initialization with string parameters
echo -e "${YELLOW}Trying initialization with full parameters...${NC}"
INIT_FULL_OUTPUT=$(NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "0,YieldVault,YVT,Bitcoin,BTC,8" --provider oylnet)
echo "$INIT_FULL_OUTPUT"

# Generate confirmation blocks
echo -e "${YELLOW}Generating confirmation blocks...${NC}"
NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10

# Check contract state - get name (opcode 100)
echo -e "${YELLOW}Checking contract name (opcode 100)...${NC}"
NAME_OUTPUT=$(NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "100" --provider oylnet)
echo "$NAME_OUTPUT"

# Check contract symbol (opcode 101)
echo -e "${YELLOW}Checking contract symbol (opcode 101)...${NC}"
SYMBOL_OUTPUT=$(NODE_OPTIONS=--require=../oyl-sdk/lib/shared/load_patch.js oyl alkane execute -data "101" --provider oylnet)
echo "$SYMBOL_OUTPUT"

# Update contract status in details file
if [ -f "$CONTRACT_DETAILS_FILE" ]; then
  sed -i 's/"status": "DEPLOYED"/"status": "INITIALIZED"/' "$CONTRACT_DETAILS_FILE"
  echo -e "${GREEN}Updated contract status to INITIALIZED in $CONTRACT_DETAILS_FILE${NC}"
fi

echo -e "${GREEN}=== Initialization Process Complete ===${NC}"
