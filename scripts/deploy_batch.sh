#!/bin/bash

# Batch Deployment Script - Deploy multiple components in sequence
# Usage: ./deploy_batch.sh [network] [components...]
# Example: ./deploy_batch.sh signet free-mint auth-token vault-factory

set -e

# Configuration
OYL_DIR="/home/e/Documents/oyl-sdk"
OYL_CMD="node bin/oyl.js"
BOILER_ROOT="/home/e/Documents/boiler"

# Parse arguments
NETWORK="${1:-oylnet}"
shift
COMPONENTS=("$@")

# If no components specified, deploy all
if [[ ${#COMPONENTS[@]} -eq 0 ]]; then
    COMPONENTS=("free-mint" "position-token" "vault-factory" "auth-token")
fi

echo "🚀 BATCH DEPLOYMENT"
echo "==================="
echo "Network: $NETWORK"
echo "Components: ${COMPONENTS[*]}"
echo ""

# Track deployments
declare -A DEPLOYMENTS
DEPLOYED_COUNT=0
FAILED_COUNT=0

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

deploy_component() {
    local component="$1"
    local namespace=$((RANDOM % 10000 + 10000))
    
    echo ""
    echo "📦 DEPLOYING: $component"
    echo "========================"
    
    # Component configuration
    case "$component" in
        "free-mint")
            local wasm_file="$BOILER_ROOT/../free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm"
            local deploy_params="3,$namespace,101"
            ;;
        "position-token")
            local wasm_file="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
            local deploy_params="3,$namespace,10"
            ;;
        "vault-factory")
            local wasm_file="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"
            local deploy_params="3,$namespace,10"
            ;;
        "auth-token")
            local wasm_file="/home/e/Documents/alkanes-rs/target/wasm32-unknown-unknown/release/alkanes_std_auth_token.wasm"
            local deploy_params="3,$namespace,0,1"
            ;;
        *)
            echo "❌ Unknown component: $component"
            return 1
            ;;
    esac
    
    # Check WASM file
    if [[ ! -f "$wasm_file" ]]; then
        echo "❌ WASM file not found: $wasm_file"
        return 1
    fi
    
    echo "WASM: $wasm_file"
    echo "Params: $deploy_params"
    echo "Namespace: $namespace"
    
    # Build command
    local deploy_cmd="$OYL_CMD alkane new-contract -c \"$wasm_file\" -data \"$deploy_params\" -p $NETWORK"
    if [[ "$NETWORK" == "signet" ]]; then
        deploy_cmd="$deploy_cmd --feeRate 10"
    fi
    
    # Pre-deployment
    generate_blocks
    rate_limit
    
    # Execute
    echo "📤 Deploying..."
    local deploy_output
    deploy_output=$(cd "$OYL_DIR" && eval "$deploy_cmd" 2>&1)
    local deploy_status=$?
    
    if [[ $deploy_status -eq 0 ]]; then
        # Extract transaction ID
        local txid
        txid=$(echo "$deploy_output" | grep -o '"txId":"[^"]*"' | cut -d'"' -f4)
        if [[ -z "$txid" ]]; then
            txid=$(echo "$deploy_output" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
        fi
        
        if [[ -n "$txid" ]]; then
            echo "✅ SUCCESS: $component"
            echo "   TX: $txid"
            echo "   Namespace: $namespace"
            
            DEPLOYMENTS["$component"]="$txid:$namespace"
            ((DEPLOYED_COUNT++))
            
            # Post-deployment
            rate_limit
            generate_blocks
            
        else
            echo "❌ FAILED: $component (no txid)"
            ((FAILED_COUNT++))
            return 1
        fi
    else
        echo "❌ FAILED: $component"
        echo "Error: $deploy_output"
        ((FAILED_COUNT++))
        return 1
    fi
    
    # Component isolation
    rate_limit
}

# Initial setup
echo "🔧 INITIAL SETUP"
generate_blocks
rate_limit

# Deploy each component
for component in "${COMPONENTS[@]}"; do
    deploy_component "$component"
done

echo ""
echo "📊 BATCH DEPLOYMENT SUMMARY"
echo "=========================="
echo "Total requested: ${#COMPONENTS[@]}"
echo "Successfully deployed: $DEPLOYED_COUNT"
echo "Failed: $FAILED_COUNT"
echo "Network: $NETWORK"
echo ""

if [[ $DEPLOYED_COUNT -gt 0 ]]; then
    echo "✅ SUCCESSFUL DEPLOYMENTS:"
    for component in "${!DEPLOYMENTS[@]}"; do
        IFS=':' read -r txid namespace <<< "${DEPLOYMENTS[$component]}"
        echo "  • $component: $txid (namespace: $namespace)"
    done
fi

if [[ $FAILED_COUNT -gt 0 ]]; then
    echo ""
    echo "❌ Some deployments failed. Check errors above."
    exit 1
else
    echo ""
    echo "🎉 ALL DEPLOYMENTS SUCCESSFUL!"
fi
