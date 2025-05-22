#!/bin/bash

# Check if transaction ID parameter is provided
if [ -z "$1" ]; then
  echo "Error: Transaction ID parameter is required"
  echo "Usage: ./trace.sh <txid> [vout]"
  exit 1
fi

# Store the transaction ID parameter
TXID=$1

# Store the vout parameter (default to 3 if not provided)
VOUT=${2:-3}

OYL_CMD="node bin/oyl.js"
OYL_DIR="./oyl-sdk"

echo "Running trace on transaction: $TXID with vout=$VOUT"
echo "----------------------------------------"

# Generate blocks before trace
echo "Generating blocks before trace"
BLOCKS_BEFORE=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_BEFORE"

# Add a pause before trace
echo "Pausing for 5 seconds before running trace command..."
sleep 5

# Run the trace command
TRACE_OUTPUT=$(cd $OYL_DIR && $OYL_CMD alkane trace -params '{"txid":"'$TXID'","vout":'$VOUT'}' -p oylnet)

# Output the trace result
echo "$TRACE_OUTPUT"

# Generate blocks after trace
echo "Generating blocks after trace"
BLOCKS_AFTER=$(cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet)
echo "$BLOCKS_AFTER"
