#!/bin/bash
set -e

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== YieldVault Optimized Deployment ===${NC}"

# Set up environment variables
export PROVIDER="oylnet"
export NODE_OPTIONS="--require=./oyl-sdk/lib/shared/load_patch.js"

# 1. Generate a few more blocks
echo -e "${YELLOW}1. Generating blocks${NC}"
oyl regtest genBlocks -p oylnet -c 10 || true

# 2. Prepare contract parameters
echo -e "${YELLOW}2. Preparing contract parameters${NC}"
NAME_HEX=$(echo -n "YieldVault" | xxd -p | tr -d '\n')
SYMBOL_HEX=$(echo -n "YVT" | xxd -p | tr -d '\n')
ASSET_NAME_HEX=$(echo -n "Bitcoin" | xxd -p | tr -d '\n')
ASSET_SYMBOL_HEX=$(echo -n "BTC" | xxd -p | tr -d '\n')
DECIMALS=8

CALLDATA="0,0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${DECIMALS}"
echo -e "${GREEN}Using calldata: ${CALLDATA}${NC}"

# 3. Deploy with minimal fee rate using SDK test wallet
echo -e "${YELLOW}3. Deploying contract${NC}"
echo -e "${BLUE}Using test wallet address: bcrt1qzr9vhs60g6qlmk7x3dd7g3ja30wyts48sxuemv${NC}"

# Create a .env file with the test wallet address
echo "PROVIDER=oylnet" > .env
echo "FUNDED_ADDRESS=\"bcrt1qzr9vhs60g6qlmk7x3dd7g3ja30wyts48sxuemv\"" >> .env
echo "TEST_MODE=true" >> .env

# Deploy the contract with minimal fee
oyl alkane new-contract \
  --contract './build/yield_vault.wasm' \
  --provider 'oylnet' \
  --calldata "${CALLDATA}" \
  --feeRate 0.01

DEPLOY_STATUS=$?
if [ $DEPLOY_STATUS -eq 0 ]; then
  echo -e "${GREEN}Contract deployment successful!${NC}"
else
  echo -e "${RED}Contract deployment failed with exit code ${DEPLOY_STATUS}${NC}"
  exit 1
fi

# 4. Generate blocks to confirm deployment
echo -e "${YELLOW}4. Generating blocks to confirm deployment${NC}"
oyl regtest genBlocks -p oylnet -c 10 || true

echo -e "${GREEN}Deployment process complete${NC}"
