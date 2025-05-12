#!/bin/bash

# =============================================
# 🌐 YieldVault Deployment with Proper Funding
# =============================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Get repository root directory
ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../.." >/dev/null 2>&1 && pwd )"

# Step 1: Build WASM if needed
if [ ! -f "${ROOT_DIR}/build/yield_vault.wasm" ]; then
    echo -e "${YELLOW}Building WebAssembly binary...${NC}"
    cargo build --target wasm32-unknown-unknown --release
    
    # Create build directory if it doesn't exist
    mkdir -p "${ROOT_DIR}/build"
    
    # Copy the WASM file to the build directory
    cp "${ROOT_DIR}/target/wasm32-unknown-unknown/release/yield_vault.wasm" "${ROOT_DIR}/build/yield_vault.wasm"
    
    echo -e "${GREEN}WebAssembly binary built and copied to build directory${NC}"
fi

# Step 2: Fund wallet using the proper SDK approach
echo -e "${BLUE}Funding wallet with proper SDK integration...${NC}"

# Run the funding script with proper error handling
node "${ROOT_DIR}/fix_funding.js"
FUND_STATUS=$?

if [ $FUND_STATUS -ne 0 ]; then
    echo -e "${RED}Failed to fund wallet. Check the error messages above.${NC}"
    exit 1
fi

echo -e "${GREEN}Wallet funded successfully!${NC}"

# Step 3: Load the funded address from .env
source "${ROOT_DIR}/.env"
if [ -z "$FUNDED_ADDRESS" ]; then
    echo -e "${RED}No funded address found in .env file.${NC}"
    exit 1
fi

echo -e "${BLUE}Using funded address: ${FUNDED_ADDRESS}${NC}"

# Step 4: Deploy the contract
echo -e "${BOLD}${BLUE}Deploying YieldVault Contract to OylNet Network...${NC}"
echo ""

# Set contract parameters
NAME="YieldVault"
SYMBOL="YVT"
ASSET_NAME="Bitcoin"
ASSET_SYMBOL="BTC"
DECIMALS=8

# Convert parameters to hex
NAME_HEX=$(echo -n "$NAME" | xxd -p | tr -d '\n')
SYMBOL_HEX=$(echo -n "$SYMBOL" | xxd -p | tr -d '\n')
ASSET_NAME_HEX=$(echo -n "$ASSET_NAME" | xxd -p | tr -d '\n')
ASSET_SYMBOL_HEX=$(echo -n "$ASSET_SYMBOL" | xxd -p | tr -d '\n')

# Initialize calldata for deployment
CALLDATA="0,0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${DECIMALS}"

echo -e "${BLUE}📝 Deployment Parameters:${NC}"
echo -e "  - Vault Name: $NAME"
echo -e "  - Vault Symbol: $SYMBOL"
echo -e "  - Asset Name: $ASSET_NAME"
echo -e "  - Asset Symbol: $ASSET_SYMBOL"
echo -e "  - Decimals: $DECIMALS"

# Deploy the contract using the funded address
echo -e "${BLUE}Deploying contract to OylNet using address ${FUNDED_ADDRESS}...${NC}"

# Generate some blocks to ensure everything is prepared
echo -e "${BLUE}Generating blocks to prepare network...${NC}"
source "${ROOT_DIR}/.env" && oyl regtest genBlocks -p oylnet -c 3

# Execute the deployment command
DEPLOY_CMD="source ${ROOT_DIR}/.env && oyl alkane new-contract --contract \"${ROOT_DIR}/build/yield_vault.wasm\" --provider \"oylnet\" --calldata \"${CALLDATA}\" --feeRate 1 --address \"${FUNDED_ADDRESS}\""
echo -e "${CYAN}$ $DEPLOY_CMD${NC}"

eval "$DEPLOY_CMD" | tee deploy_output.log

# Check if the deployment was successful by looking for a transaction ID
if grep -q "txId" deploy_output.log; then
    echo -e "${GREEN}✅ Contract deployed successfully!${NC}"
    
    # Extract transaction ID
    CONTRACT_ID=$(grep -o '"txId"\s*:\s*"[0-9a-f]\+"' deploy_output.log | sed 's/"txId"\s*:\s*"\([0-9a-f]\+\)"/\1/')
    
    if [ -z "$CONTRACT_ID" ]; then
        # Try alternative format
        CONTRACT_ID=$(grep -o "txId\s*:\s*['\"]\?[0-9a-f]\+['\"]\?" deploy_output.log | sed "s/txId\s*:\s*['\"]\?\([0-9a-f]\+\)['\"]\?/\1/")
    fi
    
    if [ -n "$CONTRACT_ID" ]; then
        echo -e "${BLUE}Contract ID: ${BOLD}$CONTRACT_ID${NC}"
        echo "$CONTRACT_ID" > "${ROOT_DIR}/.contract_id"
        echo -e "${GREEN}✅ Contract ID saved to .contract_id file${NC}"
    else
        echo -e "${YELLOW}⚠️ Could not extract contract ID from deployment result.${NC}"
    fi
    
    # Generate blocks to confirm deployment
    echo -e "${BLUE}Generating blocks to confirm transaction...${NC}"
    source "${ROOT_DIR}/.env" && oyl regtest genBlocks -p oylnet -c 6
    
    echo -e "\n${GREEN}✅ Contract deployment complete!${NC}"
    
    echo -e "\n${BLUE}You can now interact with the contract using:${NC}"
    echo -e "  ./bin/net/network.sh --interact"
    
    exit 0
else
    echo -e "${RED}❌ Contract deployment failed!${NC}"
    echo -e "See deploy_output.log for details."
    exit 1
fi
