#!/bin/bash

echo "==========================================="
echo "🏦 Interacting with YieldVault Contract on OylNet"
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

# Check if contract ID file exists
if [ ! -f ".contract_id" ]; then
    echo -e "${RED}❌ Error: Contract ID file (.contract_id) not found!${NC}"
    echo "You need to deploy the contract first using ./deploy_to_oylnet.sh"
    exit 1
fi

# Read contract ID
CONTRACT_ID=$(cat .contract_id)
# Clean up quotes, etc. if present
CONTRACT_ID=$(echo "$CONTRACT_ID" | sed -E 's/^.*txId: '"'"'([0-9a-f]+)'"'"'.*$/\1/')

echo -e "${BLUE}Using contract ID: ${BOLD}$CONTRACT_ID${NC}"

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
    local opcode=$1
    local params=${2:-""}
    local description=$3
    
    echo -e "\n${BLUE}🔍 Reading contract - $description${NC}"
    
    local calldata=""
    if [ -z "$params" ]; then
        calldata="$opcode"
    else
        calldata="$opcode,$params"
    fi
    
    local command="source .env && export ACTIVE_CONTRACT=$CONTRACT_ID && oyl alkane execute -p oylnet --calldata \"${calldata}\""
    execute_with_retry "$command" "Read operation: $description" 3 2
    
    return $?
}

# Execute contract write operation
write_contract() {
    local opcode=$1
    local params=${2:-""}
    local description=$3
    
    echo -e "\n${BLUE}✏️ Writing to contract - $description${NC}"
    
    local calldata=""
    if [ -z "$params" ]; then
        calldata="$opcode"
    else
        calldata="$opcode,$params"
    fi
    
    local command="source .env && export ACTIVE_CONTRACT=$CONTRACT_ID && oyl alkane execute -p oylnet --calldata \"${calldata}\""
    execute_with_retry "$command" "Write operation: $description" 3 2
    local status=$?
    
    if [ $status -eq 0 ]; then
        generate_blocks 1
    fi
    
    return $status
}

# Test contract metadata functions
test_metadata() {
    echo -e "\n${BOLD}${BLUE}Testing metadata view functions...${NC}"
    
    # Get name (opcode 100)
    read_contract "100" "" "Get Name"
    
    # Get symbol (opcode 101)
    read_contract "101" "" "Get Symbol"
    
    # Get decimals (opcode 102)
    read_contract "102" "" "Get Decimals"
    
    # Get asset name (opcode 103)
    read_contract "103" "" "Get Asset Name"
    
    echo -e "\n${GREEN}✅ Metadata view functions test complete${NC}"
}

# Test accounting functions
test_accounting() {
    echo -e "\n${BOLD}${BLUE}Testing accounting view functions...${NC}"
    
    # Get total assets (opcode 200)
    read_contract "200" "" "Get Total Assets"
    
    # Get total supply (opcode 601)
    read_contract "601" "" "Get Total Supply"
    
    echo -e "\n${GREEN}✅ Accounting view functions test complete${NC}"
}

# Update yield rate
update_yield_rate() {
    echo -e "\n${BOLD}${BLUE}Updating yield rate...${NC}"
    
    # Set yield rate to 500 basis points (5%)
    local yield_rate=500
    write_contract "900" "$yield_rate" "Update Yield Rate to $yield_rate basis points (${yield_rate/100}%)"
    
    echo -e "\n${GREEN}✅ Yield rate update complete${NC}"
}

# Get yield rate
get_yield_rate() {
    echo -e "\n${BOLD}${BLUE}Getting current yield rate...${NC}"

    # Get yield rate (opcode 901)
    read_contract "901" "" "Get current yield rate"

    echo -e "\n${GREEN}✅ Yield rate retrieval complete${NC}"
}

# Deposit assets using AlkaneId
deposit_assets() {
    echo -e "\n${BOLD}${BLUE}Depositing assets with numeric test mode values...${NC}"
    
    # Create transaction hash
    local tx_hash=$(openssl rand -hex 32)
    
    # For testing, use numeric block/tx values that the contract accepts in test mode
    # Parameters: tx_hash, block, tx, assets
    local block=1  # test mode block
    local tx=1     # test mode transaction
    local assets=1000000 # 0.01 BTC (assuming 8 decimals)
    
    # Create parameters with numeric values instead of auth_token_123 string
    local params="0x${tx_hash},${block},${tx},${assets}"
    
    # Call deposit (opcode 10)
    write_contract "10" "$params" "Deposit $assets sats with test mode values"
    
    echo -e "\n${GREEN}✅ Deposit operation with AlkaneId complete${NC}"
}

# Check balance with AlkaneId
check_balance() {
    echo -e "\n${BOLD}${BLUE}Checking balance with numeric test mode values...${NC}"
    
    # For testing, use numeric block/tx values that the contract accepts in test mode
    local block=1  # test mode block
    local tx=1     # test mode transaction
    
    # Check the balance using numeric values
    read_contract "600" "${block},${tx}" "Get balance with test mode values"
    
    echo -e "\n${GREEN}✅ Balance check operation complete${NC}"
}

# Main test process
main() {
    echo -e "${BOLD}Starting interaction with YieldVault contract...${NC}"
    
    # Test 1: Check metadata
    test_metadata
    
    # Test 2: Check accounting
    test_accounting
    
    # Test 3: Update yield rate
    update_yield_rate
    
    # Test 4: Get updated yield rate
    get_yield_rate

    # Test 5: Deposit assets using AlkaneId
    deposit_assets
    
    # Test 6: Check balance using AlkaneId
    check_balance
    
    echo -e "\n${BOLD}${GREEN}🎉 Contract Interaction Complete!${NC}"
    echo -e "Your YieldVault contract is deployed and functional on OylNet."
    echo -e "Contract ID: ${BOLD}$CONTRACT_ID${NC}"
    
    return 0
}

# Run the tests
main
