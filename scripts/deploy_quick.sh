#!/bin/bash

# Quick Deploy Script - Common Deployment Patterns
# Usage: ./deploy_quick.sh <component> [network]
# Components: free-mint, position-token, vault-factory, auth-token

set -e

# Configuration
OYL_DIR="/home/e/Documents/oyl-sdk"
OYL_CMD="node bin/oyl.js"
BOILER_ROOT="/home/e/Documents/boiler"

# Parse arguments
COMPONENT="$1"
NETWORK="${2:-oylnet}"

# Validate arguments
if [[ -z "$COMPONENT" ]]; then
    echo "Usage: $0 <component> [network]"
    echo "Components: free-mint, position-token, vault-factory, auth-token"
    echo "Networks: oylnet (default), signet"
    exit 1
fi

# Generate random namespace
NAMESPACE=$((RANDOM % 10000 + 10000))

# Define component configurations
case "$COMPONENT" in
    "free-mint")
        WASM_FILE="$BOILER_ROOT/../free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm"
        DEPLOY_PARAMS="3,$NAMESPACE,101"
        DESCRIPTION="Free-mint token contract"
        ;;
    "position-token")
        WASM_FILE="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
        DEPLOY_PARAMS="3,$NAMESPACE,10"
        DESCRIPTION="Position token contract"
        ;;
    "vault-factory")
        WASM_FILE="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"
        DEPLOY_PARAMS="3,$NAMESPACE,10"
        DESCRIPTION="Vault factory contract"
        ;;
    "auth-token")
        WASM_FILE="/home/e/Documents/alkanes-rs/target/wasm32-unknown-unknown/release/alkanes_std_auth_token.wasm"
        DEPLOY_PARAMS="3,$NAMESPACE,0,1"
        DESCRIPTION="Auth token factory contract"
        ;;
    *)
        echo "Error: Unknown component '$COMPONENT'"
        echo "Available: free-mint, position-token, vault-factory, auth-token"
        exit 1
        ;;
esac

# Check if WASM file exists
if [[ ! -f "$WASM_FILE" ]]; then
    echo "Error: WASM file not found: $WASM_FILE"
    echo "Make sure the component is built first"
    exit 1
fi

echo "🚀 QUICK DEPLOY: $COMPONENT"
echo "========================="
echo "Component: $DESCRIPTION"
echo "WASM: $WASM_FILE"
echo "Params: $DEPLOY_PARAMS"
echo "Network: $NETWORK"
echo "Namespace: $NAMESPACE"
echo ""

# Utility functions
generate_blocks() {
    if [[ "$NETWORK" == "signet" ]]; then
        echo "⏭️ Skipping block generation (signet mode)"
        return 0
    fi
    echo "⛏️ Generating blocks..."
    cd "$OYL_DIR" && $OYL_CMD regtest genBlocks -p $NETWORK >/dev/null 2>&1
    echo "✅ Blocks generated"
}

rate_limit() {
    echo "⏱️ Rate limiting (3s)..."
    sleep 3
}

# Pre-deployment setup
echo "🔧 PRE-DEPLOYMENT SETUP"
generate_blocks
rate_limit

# Build command
deploy_cmd="$OYL_CMD alkane new-contract -c \"$WASM_FILE\" -data \"$DEPLOY_PARAMS\" -p $NETWORK"
if [[ "$NETWORK" == "signet" ]]; then
    deploy_cmd="$deploy_cmd --feeRate 10"
fi

echo "📤 EXECUTING DEPLOYMENT"
echo "Command: $deploy_cmd"
echo ""

# Deploy
deploy_output=$(cd "$OYL_DIR" && eval "$deploy_cmd" 2>&1)
deploy_status=$?

if [[ $deploy_status -eq 0 ]]; then
    # Extract transaction ID
    txid=$(echo "$deploy_output" | grep -o '"txId":"[^"]*"' | cut -d'"' -f4)
    if [[ -z "$txid" ]]; then
        txid=$(echo "$deploy_output" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
    fi
    
    if [[ -n "$txid" ]]; then
        echo "🎉 DEPLOYMENT SUCCESS!"
        echo "======================================"
        echo "Component: $COMPONENT"
        echo "Transaction ID: $txid"
        echo "Namespace: $NAMESPACE"
        echo "Network: $NETWORK"
        echo ""
        
        # Post-deployment
        rate_limit
        generate_blocks
        rate_limit
        
        # Get trace
        echo "🔍 GETTING TRACE..."
        trace_output=$(cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params "[{\"txid\": \"$txid\", \"vout\": 3}]" -p $NETWORK 2>&1)
        
        if [[ "$trace_output" == "[]" ]]; then
            echo "⏳ Trace not available yet (normal for recent deployments)"
        else
            echo "✅ Trace data available"
            echo "$trace_output" | head -20
        fi
        
        echo ""
        echo "📋 DEPLOYMENT SUMMARY:"
        echo "• Component deployed successfully"
        echo "• Use namespace $NAMESPACE for interactions"
        echo "• Transaction: $txid"
        
    else
        echo "❌ Could not extract transaction ID"
        echo "$deploy_output"
        exit 1
    fi
else
    echo "❌ DEPLOYMENT FAILED"
    echo "Error details:"
    echo "$deploy_output"
    exit 1
fi
