#!/bin/bash

echo "==========================================="
echo "🚀 Deploying yield-vault to OylNet Network"
echo "==========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Check if WASM file exists
WASM_PATH="alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm"

if [ ! -f "$WASM_PATH" ]; then
    echo -e "${RED}Error: WebAssembly file not found at $WASM_PATH${NC}"
    echo "Run ./final_fork_build.sh first to create the WASM file"
    exit 1
fi

echo -e "${GREEN}✅ Found WebAssembly file: $WASM_PATH${NC}"

# Get file size
WASM_SIZE=$(stat -f %z "$WASM_PATH" 2>/dev/null || stat -c %s "$WASM_PATH" 2>/dev/null)
echo -e "📊 WebAssembly binary size: $WASM_SIZE bytes"

# Create build directory if it doesn't exist
mkdir -p build

# Copy our WASM file to the expected location for the script
echo -e "${BLUE}📦 Copying WebAssembly to build directory...${NC}"
cp "$WASM_PATH" ./build/yield_vault.wasm

# Also copy to the name expected by the oylnet-currency-test.js script
cp "$WASM_PATH" ./build/sale_alkane.wasm

echo -e "${GREEN}✅ WebAssembly file copied to build directory${NC}"

# Define deployment parameters
VAULT_NAME="YieldVault"
VAULT_SYMBOL="YVT"
ASSET_NAME="Bitcoin"
ASSET_SYMBOL="BTC"
DECIMALS="8"

# Convert parameters to hex
NAME_HEX=$(echo -n "$VAULT_NAME" | xxd -p | tr -d '\n')
SYMBOL_HEX=$(echo -n "$VAULT_SYMBOL" | xxd -p | tr -d '\n')
ASSET_NAME_HEX=$(echo -n "$ASSET_NAME" | xxd -p | tr -d '\n')
ASSET_SYMBOL_HEX=$(echo -n "$ASSET_SYMBOL" | xxd -p | tr -d '\n')

# Initialize calldata for deployment
# Opcode 0 = Initialize with name, symbol, asset_name, asset_symbol, decimals
CALLDATA="0,0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${DECIMALS}"

echo -e "${BLUE}📝 Deployment Parameters:${NC}"
echo -e "  - Vault Name: $VAULT_NAME"
echo -e "  - Vault Symbol: $VAULT_SYMBOL"
echo -e "  - Asset Name: $ASSET_NAME"
echo -e "  - Asset Symbol: $ASSET_SYMBOL"
echo -e "  - Decimals: $DECIMALS"

# Function to execute a command with retries
execute_with_retry() {
    local command="$1"
    local description="$2"
    local max_retries=${3:-3}
    local retry_delay=${4:-3}
    
    echo -e "${CYAN}$ $command${NC}"
    
    local attempt=1
    local status=1
    
    while [ $attempt -le $max_retries ] && [ $status -ne 0 ]; do
        if [ $attempt -gt 1 ]; then
            echo -e "${YELLOW}⚠️ Retry $attempt of $max_retries after $retry_delay seconds...${NC}"
            sleep $retry_delay
        fi
        
        eval "$command"
        status=$?
        
        if [ $status -eq 0 ]; then
            echo -e "${GREEN}✅ Success: $description${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️ Attempt $attempt failed: $description (exit code $status)${NC}"
            attempt=$((attempt+1))
        fi
    done
    
    echo -e "${RED}❌ Failed after $max_retries attempts: $description${NC}"
    return $status
}

# Generate blocks to confirm transactions
generate_blocks() {
    local count=${1:-1}
    
    echo -e "${YELLOW}Generating $count blocks...${NC}"
    execute_with_retry "source .env && oyl regtest genBlocks -p oylnet -c $count" "Generate blocks" 3 2
    
    echo -e "${YELLOW}Waiting 5 seconds for block indexing...${NC}"
    sleep 5
    
    return 0
}

# Fund account from faucet
fund_account() {
    echo -e "${BLUE}Funding account from faucet...${NC}"
    
    # Try with smaller amount if the large amount fails
    if ! execute_with_retry "source .env && oyl regtest sendFromFaucet -p oylnet -t \"bcrt1q2dy7lfhttxsqklkds8sjngdvnupd72t648t5cu\" -s 10000000" "Fund account with 10,000,000 sats" 1 1; then
        echo -e "${YELLOW}⚠️ Trying with smaller amount...${NC}"
        execute_with_retry "source .env && oyl regtest sendFromFaucet -p oylnet -t \"bcrt1q2dy7lfhttxsqklkds8sjngdvnupd72t648t5cu\" -s 1000000" "Fund account with 1,000,000 sats" 2 2
    fi
    
    local status=$?
    if [ $status -eq 0 ]; then
        generate_blocks 1
        return 0
    else
        echo -e "${YELLOW}⚠️ Faucet funding failed, but continuing anyway...${NC}"
        echo -e "${YELLOW}⚠️ Note: Account might already have funds${NC}"
        generate_blocks 1
        return 0  # Continue even if funding fails
    fi
}

# Deploy contract
deploy_contract() {
    echo -e "${BLUE}Deploying contract to OylNet...${NC}"
    
    local command="source .env && oyl alkane new-contract --contract \"./build/yield_vault.wasm\" --provider \"oylnet\" --calldata \"${CALLDATA}\" --feeRate 10"
    
    echo -e "${CYAN}$ $command${NC}"
    local output=$(eval "$command")
    local status=$?
    
    if [ $status -ne 0 ]; then
        echo -e "${RED}❌ Failed to deploy contract${NC}"
        return 1
    fi
    
    # Extract transaction ID
    local tx_id=$(echo "$output" | grep -o '"txId"\s*:\s*"[0-9a-f]\+"' | sed 's/"txId"\s*:\s*"\([0-9a-f]\+\)"/\1/')
    
    if [ -z "$tx_id" ]; then
        # Try alternative format
        tx_id=$(echo "$output" | grep -o "txId\s*:\s*['\"]\?[0-9a-f]\+['\"]\?" | sed "s/txId\s*:\s*['\"]\?\([0-9a-f]\+\)['\"]\?/\1/")
    fi
    
    if [ -z "$tx_id" ]; then
        echo -e "${RED}❌ Failed to extract transaction ID${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Contract deployed with transaction ID: $tx_id${NC}"
    
    # Generate blocks to confirm deployment
    generate_blocks 2
    
    # Save contract ID for later use
    echo "$tx_id" > .contract_id
    echo -e "${GREEN}✅ Contract ID saved to .contract_id${NC}"
    
    return 0
}

# Main deployment process
main() {
    echo -e "${BOLD}Starting yield-vault deployment to OylNet...${NC}"
    
    # Step 1: Fund account
    fund_account
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Failed to fund account, aborting deployment${NC}"
        exit 1
    fi
    
    # Step 2: Deploy contract
    deploy_contract
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Failed to deploy contract, aborting deployment${NC}"
        exit 1
    fi
    
    local contract_id=$(cat .contract_id)
    echo -e "${BOLD}${GREEN}🎉 Deployment Complete!${NC}"
    echo -e "${BOLD}Contract ID: $contract_id${NC}"
    echo -e "${BOLD}You can now interact with your yield-vault contract on OylNet${NC}"
    
    return 0
}

# Run the deployment
main
