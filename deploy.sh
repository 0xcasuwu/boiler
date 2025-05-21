#!/bin/bash

# Check if namespace parameter is provided
if [ -z "$1" ]; then
  echo "Error: Namespace parameter is required"
  echo "Usage: ./deploy.sh <namespace>"
  exit 1
fi

# Store the namespace parameter
NAMESPACE=$1
POSITION_TOKEN_WASM_PATH="/home/e/Downloads/boiler/position-token/target/wasm32-unknown-unknown/release/position_token.wasm"
VAULT_FACTORY_WASM_PATH="/home/e/Downloads/boiler/vault-factory/target/wasm32-unknown-unknown/release/vault_factory.wasm"
OYL_CMD="node bin/oyl.js"
OYL_DIR="./oyl-sdk"

echo "Starting deployment with namespace: $NAMESPACE"
echo "----------------------------------------"

# Step 0: Generate blocks before starting
echo "Step 0: Generating blocks before starting"
INITIAL_BLOCKS=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$INITIAL_BLOCKS"

# Function to execute a command and capture its output
execute_command() {
  echo "Executing: $1"
  echo "----------------------------------------"
  # Use eval with command substitution to capture output
  local output
  output=$(eval "$1" 2>&1)
  local status=$?
  
  # Check if the command was successful
  if [ $status -eq 0 ]; then
    echo "Command completed successfully"
    echo "$output"
    echo "----------------------------------------"
    echo "$output" # Return the output
  else
    echo "Command failed with exit code $status"
    echo "Output: $output"
    exit 1
  fi
}

# Function to extract txId from JSON output
extract_txid() {
  # Debug: Show the input to extract_txid
  echo "DEBUG: Extracting txId from output" >&2
  
  # Use a simpler approach to extract the txId value
  local txid
  txid=$(echo "$1" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
  
  # Debug: Show the extracted txId
  echo "DEBUG: Extracted txId: $txid" >&2
  
  echo "$txid"
}

# Generate blocks before position-token deployment
echo "Generating blocks before position-token deployment"
BLOCKS_PRE_POSITION=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_PRE_POSITION"

# Add a pause before deployment
echo "Pausing for 5 seconds before position-token deployment..."
sleep 5

# Step 1: Deploy position-token contract with opcode 3 (new-contract)
echo "Step 1: Deploying position-token contract with opcode 3"
DEPLOY_OUTPUT_POSITION=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $POSITION_TOKEN_WASM_PATH -data 3,$NAMESPACE,0 -p oylnet)
echo "$DEPLOY_OUTPUT_POSITION"
POSITION_TXID1=$(extract_txid "$DEPLOY_OUTPUT_POSITION")
echo "Position Token Transaction ID: $POSITION_TXID1"

# Check if POSITION_TXID1 is empty
if [ -z "$POSITION_TXID1" ]; then
  echo "ERROR: Failed to extract transaction ID from position-token deployment"
  echo "Raw output:"
  echo "$DEPLOY_OUTPUT_POSITION"
  exit 1
fi

# Generate blocks after position-token deployment
echo "Generating blocks after position-token deployment"
BLOCKS_AFTER_POSITION1=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_AFTER_POSITION1"

# Add a pause before next deployment
echo "Pausing for 5 seconds before position-token token deployment..."
sleep 5

# Generate blocks before position-token token deployment
echo "Generating blocks before position-token token deployment"
BLOCKS_PRE_POSITION_TOKEN=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_PRE_POSITION_TOKEN"

# Step 3: Deploy position-token contract with opcode 6 (new-token)
echo "Step 3: Deploying position-token contract with opcode 6"
DEPLOY_OUTPUT_POSITION2=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $POSITION_TOKEN_WASM_PATH -data 6,$NAMESPACE,0 -p oylnet)
echo "$DEPLOY_OUTPUT_POSITION2"
POSITION_TXID2=$(extract_txid "$DEPLOY_OUTPUT_POSITION2")
echo "Position Token Transaction ID: $POSITION_TXID2"

# Check if POSITION_TXID2 is empty
if [ -z "$POSITION_TXID2" ]; then
  echo "ERROR: Failed to extract transaction ID from position-token token deployment"
  echo "Raw output:"
  echo "$DEPLOY_OUTPUT_POSITION2"
  exit 1
fi

# Generate blocks after position-token token deployment
echo "Generating blocks after position-token token deployment"
BLOCKS_AFTER_POSITION2=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_AFTER_POSITION2"

# Add a pause before trace
echo "Pausing for 5 seconds before running trace command..."
sleep 5

# Generate blocks before trace
echo "Generating blocks before trace"
BLOCKS_PRE_TRACE=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_PRE_TRACE"

# Step 5: Run trace on the position-token transaction
echo "Step 5: Running trace on the position-token transaction $POSITION_TXID2"

# Use the trace.sh script to run the trace command
echo "Using trace.sh to run trace command for position-token"
POSITION_TRACE_OUTPUT=$(./trace.sh "$POSITION_TXID2" 3)
echo "$POSITION_TRACE_OUTPUT"

# Extract position-token template ID from the JSON output
echo "Extracting template ID from trace output..."

# Print the trace output for debugging
echo "Trace output:"
echo "$POSITION_TRACE_OUTPUT"

# Use a direct approach to extract the template ID from the event.create.data.tx field
# This is looking for the pattern: "event":"create","data":{"block":"0x2","tx":"0x2ee"}
POSITION_TEMPLATE_ID_HEX=$(echo "$POSITION_TRACE_OUTPUT" | grep -o '"event":"create","data":{"block":"0x[0-9a-f]*","tx":"0x[0-9a-f]*"}' | grep -o '"tx":"0x[0-9a-f]*"' | head -1 | cut -d '"' -f 4)

echo "Position Token Template ID (hex): $POSITION_TEMPLATE_ID_HEX"

# If we still don't have a template ID, try a more general approach
if [ -z "$POSITION_TEMPLATE_ID_HEX" ]; then
  echo "Trying alternative extraction method..."
  # Just look for any tx field in the JSON
  POSITION_TEMPLATE_ID_HEX=$(echo "$POSITION_TRACE_OUTPUT" | grep -o '"tx":"0x[0-9a-f]*"' | head -1 | cut -d '"' -f 4)
  echo "Position Token Template ID (hex): $POSITION_TEMPLATE_ID_HEX"
fi

# If we still don't have a template ID, use a hardcoded value as a fallback
if [ -z "$POSITION_TEMPLATE_ID_HEX" ]; then
  echo "WARNING: Could not extract template ID from trace output, using hardcoded value 0x2ee"
  POSITION_TEMPLATE_ID_HEX="0x2ee"
fi

# Convert hexadecimal to decimal
if [ ! -z "$POSITION_TEMPLATE_ID_HEX" ]; then
  POSITION_TEMPLATE_ID=$(printf "%d" $POSITION_TEMPLATE_ID_HEX)
  echo "Position Token Template ID (decimal): $POSITION_TEMPLATE_ID"
else
  POSITION_TEMPLATE_ID=""
  echo "WARNING: Could not extract Position Token Template ID"
fi

# Generate blocks after trace
echo "Generating blocks after trace"
BLOCKS_AFTER_TRACE=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_AFTER_TRACE"

# Add a pause before vault-factory deployment
echo "Pausing for 5 seconds before vault-factory deployment..."
sleep 5

# Generate blocks before vault-factory deployment
echo "Generating blocks before vault-factory deployment"
BLOCKS_PRE_VAULT=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_PRE_VAULT"

# Step 6: Deploy vault-factory contract with opcode 3 (new-contract)
echo "Step 6: Deploying vault-factory contract with opcode 3"

# Pass the position token template ID as an argument to the vault-factory deployment
if [ ! -z "$POSITION_TEMPLATE_ID" ]; then
  echo "Using position token template ID: $POSITION_TEMPLATE_ID"
  DEPLOY_OUTPUT_VAULT=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $VAULT_FACTORY_WASM_PATH -data 3,$NAMESPACE,$POSITION_TEMPLATE_ID -p oylnet)
else
  echo "WARNING: Position token template ID not found, using 0 as default"
  DEPLOY_OUTPUT_VAULT=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $VAULT_FACTORY_WASM_PATH -data 3,$NAMESPACE,0 -p oylnet)
fi

echo "$DEPLOY_OUTPUT_VAULT"
VAULT_TXID1=$(extract_txid "$DEPLOY_OUTPUT_VAULT")
echo "Vault Factory Transaction ID: $VAULT_TXID1"

# Check if VAULT_TXID1 is empty
if [ -z "$VAULT_TXID1" ]; then
  echo "ERROR: Failed to extract transaction ID from vault-factory deployment"
  echo "Raw output:"
  echo "$DEPLOY_OUTPUT_VAULT"
  exit 1
fi

# Generate blocks after vault-factory deployment
echo "Generating blocks after vault-factory deployment"
BLOCKS_AFTER_VAULT1=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_AFTER_VAULT1"

echo "Deployment completed successfully!"
echo "Namespace used: $NAMESPACE"
echo "Position Token Template ID: $POSITION_TEMPLATE_ID"
echo "Position Token Transaction IDs: $POSITION_TXID1, $POSITION_TXID2"
echo "Vault Factory Transaction ID: $VAULT_TXID1"

# Update the POSITION_TOKEN_TEMPLATE_ID constant in the vault-factory code
echo "Updating POSITION_TOKEN_TEMPLATE_ID in vault-factory/src/lib.rs"
if [ ! -z "$POSITION_TEMPLATE_ID" ]; then
  # Convert decimal to hex
  POSITION_TEMPLATE_ID_HEX=$(printf "0x%x" $POSITION_TEMPLATE_ID)
  
  # Use sed to replace the constant value
  sed -i "s/const POSITION_TOKEN_TEMPLATE_ID: u128 = 0x379;/const POSITION_TOKEN_TEMPLATE_ID: u128 = $POSITION_TEMPLATE_ID_HEX;/" vault-factory/src/lib.rs
  echo "Updated POSITION_TOKEN_TEMPLATE_ID to $POSITION_TEMPLATE_ID_HEX"
else
  echo "WARNING: Could not update POSITION_TOKEN_TEMPLATE_ID because the template ID could not be extracted"
fi
