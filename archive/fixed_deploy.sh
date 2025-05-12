#!/bin/bash
set -e

# Turn off exit-on-error to make the script more resilient
set +e

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== YieldVault Fixed Deployment Script ===${NC}"

# 1. Prepare environment
echo -e "${YELLOW}1. Preparing environment${NC}"
export PROVIDER="oylnet"
# Apply our patch for address format validation
export NODE_OPTIONS="--require=./oyl-sdk/lib/shared/load_patch.js"
# Make sure we have proper values for fee calculation
export MIN_FEE=10000

# Create a .env file with the proper settings
echo "PROVIDER=oylnet" > .env

# 2. Generate blocks to make sure the chain is active
echo -e "${YELLOW}2. Generating blocks to ensure chain is active${NC}"
oyl regtest genBlocks -p oylnet -c 20 || echo "Block generation failed but continuing"

# 3. Prepare contract parameters
echo -e "${YELLOW}3. Preparing contract parameters${NC}"
NAME_HEX=$(echo -n "YieldVault" | xxd -p | tr -d '\n')
SYMBOL_HEX=$(echo -n "YVT" | xxd -p | tr -d '\n')
ASSET_NAME_HEX=$(echo -n "Bitcoin" | xxd -p | tr -d '\n')
ASSET_SYMBOL_HEX=$(echo -n "BTC" | xxd -p | tr -d '\n')
DECIMALS=8

CALLDATA="0,0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${DECIMALS}"
echo -e "${GREEN}Using calldata: ${CALLDATA}${NC}"

# 4. Make sure the build directory exists and copy the WebAssembly file
echo -e "${YELLOW}4. Preparing WebAssembly binary${NC}"
mkdir -p build
WASM_PATH="./target/wasm32-unknown-unknown/release/yield_vault.wasm"

if [ ! -f "$WASM_PATH" ]; then
  echo -e "${RED}WebAssembly file not found at $WASM_PATH${NC}"
  exit 1
fi

cp "$WASM_PATH" "./build/yield_vault.wasm"
echo -e "${GREEN}WebAssembly binary copied to build directory${NC}"

# 5. Deploy with higher fee rate to meet the minimum relay fee
echo -e "${YELLOW}5. Deploying contract with higher fee rate${NC}"
# Use high integer feeRate to meet minimum relay fee requirements
oyl alkane new-contract \
  --contract './build/yield_vault.wasm' \
  --provider 'oylnet' \
  --calldata "${CALLDATA}" \
  --feeRate 10

DEPLOY_STATUS=$?
if [ $DEPLOY_STATUS -eq 0 ]; then
  echo -e "${GREEN}Contract deployment successful!${NC}"

  # 6. Generate blocks to confirm deployment
  echo -e "${YELLOW}6. Generating blocks to confirm deployment${NC}"
  oyl regtest genBlocks -p oylnet -c 10 || echo "Confirmation block generation failed but deployment succeeded"

  echo -e "${GREEN}Deployment process complete${NC}"
  exit 0
else
  echo -e "${RED}Contract deployment failed with exit code ${DEPLOY_STATUS}${NC}"
  
  # Try alternative deployment method
  echo -e "${YELLOW}Trying alternative deployment with higher fee rate...${NC}"
  
  # Generate more blocks to ensure full confirmation of funding
  oyl regtest genBlocks -p oylnet -c 20 || echo "Block generation failed but continuing"
  
  # Try deployment with even higher fee rate
  oyl alkane new-contract \
    --contract './build/yield_vault.wasm' \
    --provider 'oylnet' \
    --calldata "${CALLDATA}" \
    --feeRate 20
  
  ALT_STATUS=$?
  if [ $ALT_STATUS -eq 0 ]; then
    echo -e "${GREEN}Alternative contract deployment successful!${NC}"
    exit 0
  else
    echo -e "${RED}All deployment attempts failed${NC}"
    
    # Try one more time with an extremely high fee rate
    echo -e "${YELLOW}Trying final deployment with extremely high fee rate...${NC}"
    
    # Generate more blocks to ensure full confirmation of funding
    oyl regtest genBlocks -p oylnet -c 20 || echo "Block generation failed but continuing"
    
    # Try deployment with extremely high fee rate
    oyl alkane new-contract \
      --contract './build/yield_vault.wasm' \
      --provider 'oylnet' \
      --calldata "${CALLDATA}" \
      --feeRate 50
    
    FINAL_STATUS=$?
    if [ $FINAL_STATUS -eq 0 ]; then
      echo -e "${GREEN}Final attempt contract deployment successful!${NC}"
      exit 0
    else
      echo -e "${RED}All deployment attempts failed${NC}"
      
      echo -e "${YELLOW}Deployment Summary:${NC}"
      echo "1. The issue appears to be a combination of address format incompatibility"
      echo "   and fee calculation errors in the OylNet SDK."
      echo "2. We patched the address format validation, but the transactions still fail"
      echo "   with 'Insufficient Balance', 'Expected property of type Satoshi',"
      echo "   or 'min relay fee not met' errors."
    echo "3. These issues suggest deeper integration problems between bitcoinjs-lib"
    echo "   and the OylNet system that would require changes to the SDK itself."
    echo ""
    echo "Recommendations:"
    echo "1. Use the mock implementation for development and testing"
    echo "2. Test with other deployment methods or networks"
    echo "3. Further debug the OylNet SDK transaction handling"
    
    exit 1
  fi
fi
