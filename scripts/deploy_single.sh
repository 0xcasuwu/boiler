#!/bin/bash

# Single Component Inline Deployment Script
# Usage: ./deploy_single.sh <wasm_file> <deploy_params> [network] [namespace]
# Example: ./deploy_single.sh free_mint.wasm "3,12345,101" signet 12345

set -e

# Configuration
OYL_DIR="/home/e/Documents/oyl-sdk"
OYL_CMD="node bin/oyl.js"

# Parse arguments
WASM_FILE="$1"
DEPLOY_PARAMS="$2"
NETWORK="${3:-oylnet}"
NAMESPACE="${4:-$((RANDOM % 10000 + 10000))}"

# Validate arguments
if [[ -z "$WASM_FILE" || -z "$DEPLOY_PARAMS" ]]; then
    echo "Usage: $0 <wasm_file> <deploy_params> [network] [namespace]"
    echo "Example: $0 /path/to/contract.wasm '3,12345,101' signet 12345"
    exit 1
fi

# Validate network
if [[ "$NETWORK" != "oylnet" && "$NETWORK" != "signet" ]]; then
    echo "Error: Network must be 'oylnet' or 'signet'"
    exit 1
fi

# Check WASM file exists
if [[ ! -f "$WASM_FILE" ]]; then
    echo "Error: WASM file not found: $WASM_FILE"
    exit 1
fi

echo "🚀 SINGLE COMPONENT DEPLOYMENT"
echo "=============================="
echo "WASM: $WASM_FILE"
echo "Params: $DEPLOY_PARAMS"
echo "Network: $NETWORK"
echo "Namespace: $NAMESPACE"
echo ""

# Block generation function
generate_blocks() {
    if [[ "$NETWORK" == "signet" ]]; then
        echo "⏭️ Skipping block generation (signet mode)"
        return 0
    fi
    
    echo "⛏️ Generating blocks..."
    cd "$OYL_DIR" && $OYL_CMD regtest genBlocks -p $NETWORK >/dev/null 2>&1
    echo "✅ Blocks generated"
}

# Rate limiting
rate_limit() {
    echo "⏱️ Rate limiting (3s)..."
    sleep 3
}

# Initial setup
generate_blocks
rate_limit

# Build deployment command
deploy_cmd="$OYL_CMD alkane new-contract -c \"$WASM_FILE\" -data \"$DEPLOY_PARAMS\" -p $NETWORK"
if [[ "$NETWORK" == "signet" ]]; then
    deploy_cmd="$deploy_cmd --feeRate 10"
fi

echo "📤 DEPLOYING..."
echo "Command: $deploy_cmd"

# Execute deployment
deploy_output=$(cd "$OYL_DIR" && eval "$deploy_cmd" 2>&1)
deploy_status=$?

if [[ $deploy_status -eq 0 ]]; then
    # Extract transaction ID
    txid=$(echo "$deploy_output" | grep -o '"txId":"[^"]*"' | cut -d'"' -f4)
    if [[ -z "$txid" ]]; then
        txid=$(echo "$deploy_output" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
    fi
    
    if [[ -n "$txid" ]]; then
        echo "✅ DEPLOYMENT SUCCESS!"
        echo "Transaction ID: $txid"
        echo "Namespace: $NAMESPACE"
        
        # Post-deployment blocks
        rate_limit
        generate_blocks
        
        # Quick trace
        echo ""
        echo "🔍 TRACE:"
        cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params "[{\"txid\": \"$txid\", \"vout\": 3}]" -p $NETWORK 2>/dev/null || echo "Trace not available yet"
        
    else
        echo "❌ Could not extract transaction ID"
        echo "$deploy_output"
        exit 1
    fi
else
    echo "❌ DEPLOYMENT FAILED"
    echo "$deploy_output"
    exit 1
fi