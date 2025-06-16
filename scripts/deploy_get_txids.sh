#!/bin/bash

# Deploy contracts, output txids and trace data
# Usage: ./deploy_get_txids.sh
# Generates random namespace > 1000 and auto-increments for each deployment

# Generate random namespace greater than 1000
BASE_NAMESPACE=$((RANDOM + 1000))
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOILER_ROOT="$(dirname "$SCRIPT_DIR")"

# Set paths
FREE_MINT_WASM_PATH="$BOILER_ROOT/../free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm"
POSITION_TOKEN_WASM_PATH="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
VAULT_FACTORY_WASM_PATH="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"

# Use absolute path to oyl-sdk
OYL_DIR="/home/e/Documents/oyl-sdk"
OYL_CMD="node bin/oyl.js"

# Function to extract txId from output
extract_txid() {
  local output="$1"
  local txid
  txid=$(echo "$output" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
  if [ -z "$txid" ]; then
    txid=$(echo "$output" | grep -o '"txId":"[^"]*"' | cut -d'"' -f4)
  fi
  if [ -z "$txid" ]; then
    txid=$(echo "$output" | grep -o 'txId [a-f0-9]\{64\}' | cut -d' ' -f2)
  fi
  echo "$txid"
}

# Function to generate blocks silently
generate_blocks() {
  cd "$OYL_DIR" && $OYL_CMD regtest genBlocks -p oylnet > /dev/null 2>&1
  sleep 1
}

# Function to check if trace output has valid data
has_valid_trace() {
  local output="$1"
  echo "$output" | grep -q '"event": "create"' && echo "$output" | grep -q '"status": "success"'
}

# Function to get trace for txid
get_trace() {
  local txid="$1"
  
  if [ -z "$txid" ]; then
    echo "[]"
    return 1
  fi
  
  # Try vout 3 first - use exact same format as working manual command
  local trace_output
  trace_output=$(cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params '[{"txid": "'$txid'", "vout": 3}]' -p oylnet)
  
  if has_valid_trace "$trace_output"; then
    echo "$trace_output"
    return 0
  fi
  
  # Try vout 4 - use exact same format as working manual command
  trace_output=$(cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params '[{"txid": "'$txid'", "vout": 4}]' -p oylnet)
  
  if has_valid_trace "$trace_output"; then
    echo "$trace_output"
    return 0
  fi
  
  echo "[]"
  return 1
}

# Function to deploy contract, get txid, and return trace data
deploy_and_trace() {
  local wasm_path="$1"
  local data_params="$2"
  local contract_name="$3"
  local max_retries=3
  local retry=0
  
  while [ $retry -lt $max_retries ]; do
    generate_blocks
    
    local output
    output=$(cd "$OYL_DIR" && $OYL_CMD alkane new-contract -c "$wasm_path" -data "$data_params" -p oylnet 2>&1)
    
    if [ $? -eq 0 ]; then
      local txid
      txid=$(extract_txid "$output")
      if [ ! -z "$txid" ]; then
        generate_blocks
        echo "txid: $txid"
        get_trace "$txid"
        echo ""
        return 0
      fi
    fi
    
    retry=$((retry + 1))
    if [ $retry -lt $max_retries ]; then
      sleep 2
    fi
  done
  
  echo "ERROR_DEPLOY_$contract_name"
  return 1
}

# Initial setup
generate_blocks

# Deploy template contracts with auto-incrementing namespaces
NAMESPACE=$BASE_NAMESPACE
deploy_and_trace "$FREE_MINT_WASM_PATH" "3,$NAMESPACE,101" "free-mint-template"

NAMESPACE=$((BASE_NAMESPACE + 1))
deploy_and_trace "$POSITION_TOKEN_WASM_PATH" "3,$NAMESPACE,10" "position-token-template"

NAMESPACE=$((BASE_NAMESPACE + 2))
deploy_and_trace "$VAULT_FACTORY_WASM_PATH" "3,$NAMESPACE,10" "vault-factory-template"
