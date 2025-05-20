#!/bin/bash

# Check if namespace parameter is provided
if [ -z "$1" ]; then
  echo "Error: Namespace parameter is required"
  echo "Usage: ./deploy.sh <namespace>"
  exit 1
fi

# Store the namespace parameter
NAMESPACE=$1
WASM_PATH="/home/e/boiler/minimal-contract/target/wasm32-unknown-unknown/release/minimal_yield_vault.wasm"
OYL_CMD="node bin/oyl.js"
OYL_DIR="/home/e/boiler/oyl-sdk"

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

# Step 1: Deploy contract with opcode 3 (new-contract)
echo "Step 1: Deploying contract with opcode 3"
DEPLOY_OUTPUT=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $WASM_PATH -data 3,$NAMESPACE,0 -p oylnet)
echo "$DEPLOY_OUTPUT"
TXID1=$(extract_txid "$DEPLOY_OUTPUT")
echo "Transaction ID: $TXID1"

# Check if TXID1 is empty
if [ -z "$TXID1" ]; then
  echo "ERROR: Failed to extract transaction ID from first deployment"
  echo "Raw output:"
  echo "$DEPLOY_OUTPUT"
  exit 1
fi

# Step 2: Generate blocks
echo "Step 2: Generating blocks"
BLOCKS_OUTPUT=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_OUTPUT"

# Step 3: Deploy contract with opcode 6 (new-token)
echo "Step 3: Deploying contract with opcode 6"
DEPLOY_OUTPUT2=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $WASM_PATH -data 6,$NAMESPACE,0 -p oylnet)
echo "$DEPLOY_OUTPUT2"
TXID2=$(extract_txid "$DEPLOY_OUTPUT2")
echo "Transaction ID: $TXID2"

# Check if TXID2 is empty
if [ -z "$TXID2" ]; then
  echo "ERROR: Failed to extract transaction ID from second deployment"
  echo "Raw output:"
  echo "$DEPLOY_OUTPUT2"
  exit 1
fi

# Step 4: Generate blocks again
echo "Step 4: Generating blocks"
BLOCKS_OUTPUT2=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_OUTPUT2"

# Step 5: Generate additional blocks to ensure transaction is confirmed
echo "Step 5: Generating additional blocks to ensure transaction is confirmed"
BLOCKS_OUTPUT3=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_OUTPUT3"

# Step 6: Run trace only on the last transaction with vout=3
echo "Step 6: Running trace on the last transaction $TXID2 with vout=3"
TRACE_PARAMS="{\"txid\":\"$TXID2\",\"vout\":3}"
echo "Trace parameters: $TRACE_PARAMS"

# Use single quotes around the JSON parameters to prevent shell interpretation
TRACE_CMD="$OYL_CMD alkane trace -params '$TRACE_PARAMS' -p oylnet"
echo "Executing trace command: $TRACE_CMD"
TRACE_OUTPUT=$(cd $OYL_DIR && eval "$TRACE_CMD")

# Check if the trace output is empty or doesn't contain useful data
if [ -z "$TRACE_OUTPUT" ] || ! echo "$TRACE_OUTPUT" | grep -q "myself"; then
  echo "No trace result with vout=3, trying with vout=4..."
  TRACE_PARAMS="{\"txid\":\"$TXID2\",\"vout\":4}"
  echo "Trace parameters: $TRACE_PARAMS"
  TRACE_CMD="$OYL_CMD alkane trace -params '$TRACE_PARAMS' -p oylnet"
  echo "Executing trace command: $TRACE_CMD"
  TRACE_OUTPUT=$(cd $OYL_DIR && eval "$TRACE_CMD")
fi

echo "Trace result for last transaction:"
echo "$TRACE_OUTPUT"

# Extract and convert the namespace number from the trace output
if [ -n "$TRACE_OUTPUT" ]; then
  echo "Parsing trace output to extract namespace information..."
  
  # Use a more specific pattern to extract the "myself" block and tx values
  # The pattern looks for the "context" object that contains the "myself" object
  CONTEXT_PATTERN='"context":{"myself":{"block":"[^"]*","tx":"[^"]*"}'
  CONTEXT=$(echo "$TRACE_OUTPUT" | grep -o "$CONTEXT_PATTERN")
  
  if [ -n "$CONTEXT" ]; then
    BLOCK_HEX=$(echo "$CONTEXT" | grep -o '"block":"[^"]*"' | cut -d'"' -f4)
    TX_HEX=$(echo "$CONTEXT" | grep -o '"tx":"[^"]*"' | cut -d'"' -f4)
    
    if [ -n "$BLOCK_HEX" ] && [ -n "$TX_HEX" ]; then
      # Remove the "0x" prefix if present
      BLOCK_HEX_CLEAN=${BLOCK_HEX#0x}
      TX_HEX_CLEAN=${TX_HEX#0x}
      
      # Convert hexadecimal to decimal
      BLOCK_DEC=$(printf "%d" 0x$BLOCK_HEX_CLEAN 2>/dev/null)
      TX_DEC=$(printf "%d" 0x$TX_HEX_CLEAN 2>/dev/null)
      
      echo "Namespace Information:"
      echo "Block: $BLOCK_HEX (decimal: $BLOCK_DEC)"
      echo "TX: $TX_HEX (decimal: $TX_DEC)"
      echo "Namespace Number: $TX_DEC"
    else
      echo "Could not extract block or tx values from context"
    fi
  else
    echo "Could not find context with myself information in trace output"
    echo "Raw trace output:"
    echo "$TRACE_OUTPUT"
  fi
else
  echo "No trace output available"
fi

# Step 7: Generate blocks after finishing
echo "Step 7: Generating blocks after finishing"
FINAL_BLOCKS=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$FINAL_BLOCKS"

echo "Deployment completed successfully!"
echo "Namespace used: $NAMESPACE"
echo "First transaction ID: $TXID1"
echo "Second transaction ID: $TXID2"
