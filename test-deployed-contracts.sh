#!/bin/bash
# Simple script to test already deployed contracts
# This script makes method calls to the contracts we already deployed

set -e

# Configuration
RPC_URL="http://localhost:18889"
LOG_FILE="contract-test-$(date +%Y%m%d-%H%M%S).log"

# Contract IDs (these should match what's already deployed)
BOND_CURVE="00000000000000000000000000000000000000000000000000000000000003e9"
ORBITAL_BOND="00000000000000000000000000000000000000000000000000000000000003ea"
LAUNCHPAD_FACTORY="00000000000000000000000000000000000000000000000000000000000003eb"

# Text colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_step() {
  echo -e "${BLUE}==== $1 ====${NC}"
  echo -e "==== $1 ====" >> "$LOG_FILE"
}

log_success() {
  echo -e "${GREEN}✓ $1${NC}"
  echo -e "✓ $1" >> "$LOG_FILE"
}

log_info() {
  echo -e "${YELLOW}$1${NC}"
  echo -e "$1" >> "$LOG_FILE"
}

log_error() {
  echo -e "${RED}ERROR: $1${NC}"
  echo -e "ERROR: $1" >> "$LOG_FILE"
  exit 1
}

# Function to make a JSON-RPC call
make_rpc_call() {
  local method=$1
  local params=$2
  
  curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}"
}

# Function to directly call a contract method
call_contract() {
  local contract_id=$1
  local method=$2
  local args=$3
  local description=$4
  
  log_info "Calling $method on contract $contract_id ($description)"
  
  # Direct curl command for better reliability
  local curl_cmd="curl -s -X POST $RPC_URL -H \"Content-Type: application/json\" -d '{\"jsonrpc\":\"2.0\",\"method\":\"alkane_callContract\",\"params\":[\"$contract_id\",\"$method\",\"$args\"],\"id\":1}'"
  log_info "Executing: $curl_cmd"
  
  # Execute the curl command
  local response=$(eval "$curl_cmd")
  echo "$response" >> "$LOG_FILE"
  log_info "RPC Response: $response"
  
  # Check for errors
  if echo "$response" | grep -q "error"; then
    log_error "Failed to call $method on $description"
    return 1
  fi
  
  # Extract transaction ID
  local txid=$(echo "$response" | sed -n 's/.*"result":"\([0-9a-f]*\)".*/\1/p')
  if [ -z "$txid" ]; then
    log_error "No transaction ID returned from $method call on $description"
    return 1
  fi
  
  log_success "$method call successful with txid: $txid"
  echo "$txid"
}

# Function to get transaction receipt
get_receipt() {
  local txid=$1
  local description=$2
  
  log_info "Getting receipt for transaction $txid ($description)"
  
  # Wait a moment for transaction to be processed
  sleep 1
  
  # Direct curl command for better reliability
  local curl_cmd="curl -s -X POST $RPC_URL -H \"Content-Type: application/json\" -d '{\"jsonrpc\":\"2.0\",\"method\":\"alkane_getTransactionReceipt\",\"params\":[\"$txid\"],\"id\":1}'"
  log_info "Executing: $curl_cmd"
  
  # Execute the curl command
  local response=$(eval "$curl_cmd")
  echo "$response" >> "$LOG_FILE"
  log_info "RPC Response: $response"
  
  # Check for errors
  if echo "$response" | grep -q "error"; then
    log_error "Failed to get receipt for $description (txid: $txid)"
    return 1
  elif echo "$response" | grep -q '"result":null'; then
    log_info "Receipt not yet available, waiting a bit longer..."
    sleep 2
    response=$(eval "$curl_cmd")
    echo "$response" >> "$LOG_FILE"
    
    if echo "$response" | grep -q '"result":null'; then
      log_error "Receipt still not available for $description (txid: $txid)"
      return 1
    fi
  fi
  
  # Extract and display important receipt information
  if echo "$response" | grep -q "success"; then
    local status=$(echo "$response" | sed -n 's/.*"status":"\([^"]*\)".*/\1/p')
    local block=$(echo "$response" | sed -n 's/.*"block_height":\([0-9]*\).*/\1/p')
    local opcode=$(echo "$response" | sed -n 's/.*"opcode":"\([^"]*\)".*/\1/p')
    
    log_info "Receipt details:"
    log_info "  Status: $status"
    log_info "  Block: $block"
    log_info "  Operation: $opcode"
  else
    log_info "Receipt format unexpected, see log file for details"
  fi
  
  log_success "Receipt retrieved for $description (txid: $txid)"
}

# Function to check server health
check_server() {
  log_step "Checking server health"
  
  local response=$(curl -s $RPC_URL/health)
  if [[ "$response" != *"success"* ]]; then
    log_error "Server is not responding at $RPC_URL"
    return 1
  fi
  
  log_success "Server is running at $RPC_URL"
}

# Main execution
echo "Starting contract tests at $(date)" > "$LOG_FILE"
echo "======================================" >> "$LOG_FILE"

# Check if server is running
check_server

# --- BOND CURVE TESTS ---
log_step "Testing Bond Curve Contract"

# Test getPrice function
price_txid=$(call_contract $BOND_CURVE "getPrice" "10000000" "Bond Curve getPrice")
echo "Transaction ID from getPrice: $price_txid"
get_receipt "$price_txid" "Bond Curve getPrice"

# Test getVersion function
version_txid=$(call_contract $BOND_CURVE "getVersion" "" "Bond Curve getVersion")
echo "Transaction ID from getVersion: $version_txid"
get_receipt "$version_txid" "Bond Curve getVersion"

# --- ORBITAL BOND COLLECTION TESTS ---
log_step "Testing Orbital Bond Collection Contract"

# Test getBondCount function
count_txid=$(call_contract $ORBITAL_BOND "getBondCount" "" "Orbital Bond getBondCount")
echo "Transaction ID from getBondCount: $count_txid"
get_receipt "$count_txid" "Orbital Bond getBondCount"

# Test getTokenInfo function
info_txid=$(call_contract $ORBITAL_BOND "getTokenInfo" "" "Orbital Bond getTokenInfo")
echo "Transaction ID from getTokenInfo: $info_txid" 
get_receipt "$info_txid" "Orbital Bond getTokenInfo"

# --- LAUNCHPAD FACTORY TESTS ---
log_step "Testing Launchpad Factory Contract"

# Test getFactoryInfo function
factory_info_txid=$(call_contract $LAUNCHPAD_FACTORY "getFactoryInfo" "" "Launchpad Factory getFactoryInfo")
echo "Transaction ID from getFactoryInfo: $factory_info_txid"
get_receipt "$factory_info_txid" "Launchpad Factory getFactoryInfo"

# Test getVersion function
factory_version_txid=$(call_contract $LAUNCHPAD_FACTORY "getVersion" "" "Launchpad Factory getVersion")
echo "Transaction ID from getVersion: $factory_version_txid"
get_receipt "$factory_version_txid" "Launchpad Factory getVersion"

log_step "All contract tests completed successfully"
log_info "Log file: $LOG_FILE"

exit 0
