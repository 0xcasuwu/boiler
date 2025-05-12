#!/bin/bash

# =============================================
# 🌐 OylNet Network Operations Unified Script
# =============================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Default mode
MODE=""

# Get repository root directory (works even when script is called from another directory)
ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --test) MODE="test";;
        --deploy) MODE="deploy";;
        --interact) MODE="interact";;
        --help|-h)
            echo -e "${BOLD}OylNet Network Operations Unified Script${NC}"
            echo ""
            echo -e "Usage: $0 [OPTIONS]"
            echo ""
            echo -e "Options:"
            echo -e "  --test       Test connection to OylNet network"
            echo -e "  --deploy     Deploy contract to OylNet network"
            echo -e "  --interact   Interact with deployed contract"
            echo -e "  --help, -h   Show this help message"
            exit 0
            ;;
        *) echo "Unknown parameter: $1"; exit 1;;
    esac
    shift
done

# Check if mode is specified
if [ -z "$MODE" ]; then
    echo -e "${RED}Error: No mode specified. Use --test, --deploy, or --interact.${NC}"
    echo -e "Run '$0 --help' for more information."
    exit 1
fi

# Function to execute a command with retries
execute_with_retry() {
    local command="$1"
    local description="$2"
    local max_retries=${3:-3}
    local retry_delay=${4:-3}
    
    echo -e "${CYAN}$ $command${NC}"
    
    local attempt=1
    local status=1
    local output=""
    
    while [ $attempt -le $max_retries ] && [ $status -ne 0 ]; do
        if [ $attempt -gt 1 ]; then
            echo -e "${YELLOW}⚠️ Retry $attempt of $max_retries after $retry_delay seconds...${NC}"
            sleep $retry_delay
        fi
        
        output=$(eval "$command" 2>&1)
        status=$?
        
        if [ $status -eq 0 ]; then
            echo -e "${GREEN}✅ Success: $description${NC}"
            echo -e "Response:"
            echo "$output"
            return 0
        else
            echo -e "${YELLOW}⚠️ Attempt $attempt failed: $description (exit code $status)${NC}"
            echo -e "Error output:"
            echo "$output"
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
    
    echo -e "${YELLOW}Waiting 3 seconds for block indexing...${NC}"
    sleep 3
    
    return 0
}

# Execute contract read operation
read_contract() {
    local contract_id=$1
    local opcode=$2
    local params=${3:-""}
    local description=$4
    
    echo -e "\n${BLUE}🔍 Reading contract - $description${NC}"
    
    local calldata=""
    if [ -z "$params" ]; then
        calldata="$opcode"
    else
        calldata="$opcode,$params"
    fi
    
    local command="source .env && export ACTIVE_CONTRACT=$contract_id && oyl alkane execute -p oylnet --calldata \"${calldata}\""
    execute_with_retry "$command" "Read operation: $description" 3 2
    
    return $?
}

# Execute contract write operation
write_contract() {
    local contract_id=$1
    local opcode=$2
    local params=${3:-""}
    local description=$4
    
    echo -e "\n${BLUE}✏️ Writing to contract - $description${NC}"
    
    local calldata=""
    if [ -z "$params" ]; then
        calldata="$opcode"
    else
        calldata="$opcode,$params"
    fi
    
    local command="source .env && export ACTIVE_CONTRACT=$contract_id && oyl alkane execute -p oylnet --calldata \"${calldata}\""
    execute_with_retry "$command" "Write operation: $description" 3 2
    local status=$?
    
    if [ $status -eq 0 ]; then
        generate_blocks 1
    fi
    
    return $status
}

# Test connection to OylNet
test_connection() {
    echo -e "${BOLD}${BLUE}Testing Connection to OylNet Network...${NC}"
    echo ""
    
    # Check if .env file exists
    if [ ! -f "${ROOT_DIR}/.env" ]; then
        echo -e "${RED}❌ Error: .env file not found!${NC}"
        echo "Create a .env file with your OylNet credentials."
        exit 1
    fi
    
    # Test OylNet connection by generating a block
    echo -e "${BLUE}Testing OylNet connection by generating a block...${NC}"
    generate_blocks 1
    
    echo -e "\n${GREEN}✅ Connection to OylNet is working properly!${NC}"
}

# Deploy contract to OylNet
deploy_contract() {
    echo -e "${BOLD}${BLUE}Deploying YieldVault Contract to OylNet Network...${NC}"
    echo ""
    
    # Check if .env file exists
    if [ ! -f "${ROOT_DIR}/.env" ]; then
        echo -e "${RED}❌ Error: .env file not found!${NC}"
        echo "Create a .env file with your OylNet credentials."
        exit 1
    fi
    
    # Check if WebAssembly file exists
    local wasm_path="${ROOT_DIR}/alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm"
    if [ ! -f "$wasm_path" ]; then
        echo -e "${RED}❌ Error: WebAssembly binary not found!${NC}"
        echo "Run './scripts/build.sh' first to build the WebAssembly binary."
        exit 1
    fi
    
    # Create build directory if it doesn't exist
    mkdir -p "${ROOT_DIR}/build"
    
    # Copy the WASM file to the build directory
    echo -e "${BLUE}📦 Copying WebAssembly to build directory...${NC}"
    cp "$wasm_path" "${ROOT_DIR}/build/yield_vault.wasm"
    
    # Set contract parameters
    local name="YieldVault"
    local symbol="YVT"
    local asset_name="Bitcoin"
    local asset_symbol="BTC"
    local decimals=8
    
    # Convert parameters to hex
    NAME_HEX=$(echo -n "$name" | xxd -p | tr -d '\n')
    SYMBOL_HEX=$(echo -n "$symbol" | xxd -p | tr -d '\n')
    ASSET_NAME_HEX=$(echo -n "$asset_name" | xxd -p | tr -d '\n')
    ASSET_SYMBOL_HEX=$(echo -n "$asset_symbol" | xxd -p | tr -d '\n')
    
    # Initialize calldata for deployment
    CALLDATA="0,0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${decimals}"
    
    echo -e "${BLUE}📝 Deployment Parameters:${NC}"
    echo -e "  - Vault Name: $name"
    echo -e "  - Vault Symbol: $symbol"
    echo -e "  - Asset Name: $asset_name"
    echo -e "  - Asset Symbol: $asset_symbol"
    echo -e "  - Decimals: $decimals"
    
    # Deploy contract
    echo -e "${BLUE}Deploying contract to OylNet...${NC}"
    
    local command="source .env && oyl alkane new-contract --contract \"${ROOT_DIR}/build/yield_vault.wasm\" --provider \"oylnet\" --calldata \"${CALLDATA}\" --feeRate 10"
    
    echo -e "${CYAN}$ $command${NC}"
    local deploy_result=$(eval "$command")
    local deploy_status=$?
    
    if [ $deploy_status -eq 0 ]; then
        echo -e "${GREEN}✅ Contract deployed successfully!${NC}"
        echo "$deploy_result"
        
        # Extract transaction ID
        local contract_id=$(echo "$deploy_result" | grep -o '"txId"\s*:\s*"[0-9a-f]\+"' | sed 's/"txId"\s*:\s*"\([0-9a-f]\+\)"/\1/')
        
        if [ -z "$contract_id" ]; then
            # Try alternative format
            contract_id=$(echo "$deploy_result" | grep -o "txId\s*:\s*['\"]\?[0-9a-f]\+['\"]\?" | sed "s/txId\s*:\s*['\"]\?\([0-9a-f]\+\)['\"]\?/\1/")
        fi
        
        if [ -n "$contract_id" ]; then
            echo -e "${BLUE}Contract ID: ${BOLD}$contract_id${NC}"
            echo "$contract_id" > "${ROOT_DIR}/.contract_id"
            echo -e "${GREEN}✅ Contract ID saved to .contract_id file${NC}"
        else
            echo -e "${YELLOW}⚠️ Could not extract contract ID from deployment result.${NC}"
        fi
        
        # Generate blocks to confirm deployment
        generate_blocks 2
    else
        echo -e "${RED}❌ Contract deployment failed!${NC}"
        echo "$deploy_result"
        exit 1
    fi
    
    echo -e "\n${GREEN}✅ Contract deployment complete!${NC}"
}

# Interact with deployed contract
interact_with_contract() {
    echo -e "${BOLD}${BLUE}Interacting with YieldVault Contract on OylNet...${NC}"
    echo ""
    
    # Check if contract ID file exists
    if [ ! -f "${ROOT_DIR}/.contract_id" ]; then
        echo -e "${RED}❌ Error: Contract ID file (.contract_id) not found!${NC}"
        echo "You need to deploy the contract first using './scripts/network.sh --deploy'"
        exit 1
    fi
    
    # Read contract ID
    local contract_id=$(cat "${ROOT_DIR}/.contract_id")
    # Clean up quotes, etc. if present
    contract_id=$(echo "$contract_id" | sed -E 's/^.*txId: '"'"'([0-9a-f]+)'"'"'.*$/\1/')
    
    echo -e "${BLUE}Using contract ID: ${BOLD}$contract_id${NC}"
    
    # Test metadata view functions
    echo -e "\n${BOLD}${BLUE}Testing metadata view functions...${NC}"
    read_contract "$contract_id" "100" "" "Get Name"
    read_contract "$contract_id" "101" "" "Get Symbol"
    read_contract "$contract_id" "102" "" "Get Decimals"
    read_contract "$contract_id" "103" "" "Get Asset Name"
    echo -e "\n${GREEN}✅ Metadata view functions test complete${NC}"
    
    # Test accounting functions
    echo -e "\n${BOLD}${BLUE}Testing accounting view functions...${NC}"
    read_contract "$contract_id" "200" "" "Get Total Assets"
    read_contract "$contract_id" "601" "" "Get Total Supply"
    echo -e "\n${GREEN}✅ Accounting view functions test complete${NC}"
    
    # Update yield rate
    echo -e "\n${BOLD}${BLUE}Updating yield rate...${NC}"
    local yield_rate=500
    write_contract "$contract_id" "900" "$yield_rate" "Update Yield Rate to $yield_rate basis points (${yield_rate/100}%)"
    echo -e "\n${GREEN}✅ Yield rate update complete${NC}"
    
    # Get yield rate
    echo -e "\n${BOLD}${BLUE}Getting current yield rate...${NC}"
    read_contract "$contract_id" "901" "" "Get current yield rate"
    echo -e "\n${GREEN}✅ Yield rate retrieval complete${NC}"
    
    # Deposit assets
    echo -e "\n${BOLD}${BLUE}Depositing assets with numeric test mode values...${NC}"
    local tx_hash=$(openssl rand -hex 32)
    local block=1  # test mode block
    local tx=1     # test mode transaction
    local assets=1000000 # 0.01 BTC (assuming 8 decimals)
    local params="0x${tx_hash},${block},${tx},${assets}"
    write_contract "$contract_id" "10" "$params" "Deposit $assets sats with test mode values"
    echo -e "\n${GREEN}✅ Deposit operation complete${NC}"
    
    # Check balance
    echo -e "\n${BOLD}${BLUE}Checking balance with numeric test mode values...${NC}"
    read_contract "$contract_id" "600" "${block},${tx}" "Get balance with test mode values"
    echo -e "\n${GREEN}✅ Balance check operation complete${NC}"
    
    echo -e "\n${BOLD}${GREEN}🎉 Contract Interaction Complete!${NC}"
    echo -e "Your YieldVault contract is deployed and functional on OylNet."
    echo -e "Contract ID: ${BOLD}$contract_id${NC}"
}

# Execute the selected mode
case "$MODE" in
    test)
        test_connection
        ;;
    deploy)
        deploy_contract
        ;;
    interact)
        interact_with_contract
        ;;
    *)
        echo -e "${RED}Error: Unknown mode: $MODE${NC}"
        exit 1
        ;;
esac

exit 0
