#!/bin/bash
set -e

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== Direct YieldVault Contract Deployment ===${NC}"

# 1. Generate more blocks
echo -e "${YELLOW}1. Generating blocks to ensure faucet funding${NC}"
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 20 || true

# 2. Create the calldata
echo -e "${YELLOW}2. Preparing contract parameters${NC}"
NAME_HEX=$(echo -n "YieldVault" | xxd -p | tr -d '\n')
SYMBOL_HEX=$(echo -n "YVT" | xxd -p | tr -d '\n')
ASSET_NAME_HEX=$(echo -n "Bitcoin" | xxd -p | tr -d '\n')
ASSET_SYMBOL_HEX=$(echo -n "BTC" | xxd -p | tr -d '\n')
DECIMALS=8

CALLDATA="0,0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${DECIMALS}"
echo -e "${GREEN}Calldata: ${CALLDATA}${NC}"

# 3. Ensure WebAssembly is ready
echo -e "${YELLOW}3. Preparing WebAssembly${NC}"
mkdir -p build
cp target/wasm32-unknown-unknown/release/yield_vault.wasm build/ || {
  echo -e "${RED}Error: WebAssembly file not found${NC}"
  exit 1
}
echo -e "${GREEN}WebAssembly copied to build directory${NC}"

# 4. Deploy with minimal fee rate
echo -e "${YELLOW}4. Deploying contract${NC}"
echo -e "${BLUE}Using CONTRACT_PATH=./build/yield_vault.wasm${NC}"
echo -e "${BLUE}Using CALLDATA=${CALLDATA}${NC}"

# Try to deploy with the lowest possible fee rate
export PROVIDER=oylnet
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract \
  --contract './build/yield_vault.wasm' \
  --provider 'oylnet' \
  --calldata "${CALLDATA}" \
  --feeRate 0.1

DEPLOY_RESULT=$?
if [ $DEPLOY_RESULT -eq 0 ]; then
  echo -e "${GREEN}Contract deployment successful!${NC}"
else
  echo -e "${RED}Contract deployment failed with exit code ${DEPLOY_RESULT}${NC}"
  
  # Try again with even lower fee rate if the first attempt failed
  echo -e "${YELLOW}Retrying with even lower fee rate...${NC}"
  
  # Generate more blocks to ensure faucet is funded
  echo -e "${YELLOW}Generating additional blocks...${NC}"
  NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 50 || true
  
  # Attempt deployment again with minimal fee rate
  NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane new-contract \
    --contract './build/yield_vault.wasm' \
    --provider 'oylnet' \
    --calldata "${CALLDATA}" \
    --feeRate 0.01
    
  RETRY_RESULT=$?
  if [ $RETRY_RESULT -eq 0 ]; then
    echo -e "${GREEN}Contract deployment successful with minimal fee!${NC}"
  else
    echo -e "${RED}All deployment attempts failed${NC}"
    exit 1
  fi
fi

# 5. Generate blocks to confirm deployment
echo -e "${YELLOW}5. Generating blocks to confirm deployment${NC}"
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10 || true

echo -e "${GREEN}Deployment process complete${NC}"
