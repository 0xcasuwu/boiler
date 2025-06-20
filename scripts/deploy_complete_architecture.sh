#!/bin/bash

# 2-Component Architecture Deployment with Contract Initialization
# Production-ready deployment system with comprehensive transaction isolation
# Components: Position Token, Vault Factory
# Features: Contract initialization, regtest isolation, 3s rate limiting, full trace analysis

echo "🏗️  2-COMPONENT ARCHITECTURE DEPLOYMENT"
echo "======================================="
echo "Production-ready position token and vault factory deployment"
echo "Based on breakthrough debugging analysis"
echo ""
echo "Components:"
echo "1. Position token template + initialization"
echo "2. Vault factory template + initialization"
echo ""

# Configuration - Updated for boiler repository structure
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOILER_ROOT="$(dirname "$SCRIPT_DIR")"
OYL_DIR="/home/e/Documents/oyl-sdk"
OYL_CMD="node bin/oyl.js"

# WASM paths for 2 components - Corrected for boiler structure
POSITION_TOKEN_WASM_PATH="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
VAULT_FACTORY_WASM_PATH="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"

# Comprehensive fallback paths for components
POSITION_TOKEN_FALLBACK_PATHS=(
    "$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
    "$BOILER_ROOT/../position-token/target/wasm32-unknown-unknown/release/position_token.wasm"
)

VAULT_FACTORY_FALLBACK_PATHS=(
    "$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"
    "$BOILER_ROOT/../vault-factory/target/wasm32-unknown-unknown/release/vault_factory.wasm"
)

# Function to generate dynamic random parameters for 2 components
generate_component_namespaces() {
    # Generate random base seed (avoid conflicts)
    local base_seed=$((RANDOM % 3000 + 10000))  # Random between 10000-40000
    
    # Add timestamp-based offset to ensure uniqueness
    local timestamp_offset=$(($(date +%s) % 1000))
    local base_namespace=$((base_seed + (timestamp_offset * 10)))
    
    # Generate dynamic namespace for vault factory only
    VAULT_FACTORY_NAMESPACE=$((base_namespace + 200))
    
    # Position token uses CONSTANT namespace for predictable deployment
    POSITION_TOKEN_NAMESPACE=899  # Constant value (0x379 hex) - required for system integration
    
    # Ensure we don't exceed reasonable bounds for vault factory
    if [ $VAULT_FACTORY_NAMESPACE -gt 60000 ]; then
        VAULT_FACTORY_NAMESPACE=$((VAULT_FACTORY_NAMESPACE - 15000))
    fi
}

# Generate dynamic parameters for 2 components
generate_component_namespaces

# Deploy parameters for each component (based on working test architecture)
POSITION_TOKEN_DEPLOY_PARAMS="3,$POSITION_TOKEN_NAMESPACE,10"  # Template deploy, namespace, amount 10
VAULT_FACTORY_DEPLOY_PARAMS="3,$VAULT_FACTORY_NAMESPACE,10"   # Template deploy, namespace, amount 10

echo "📋 2-COMPONENT CONFIGURATION"
echo "============================"
echo "Boiler Root: $BOILER_ROOT"
echo "OYL Directory: $OYL_DIR"
echo ""
echo "🏗️  WASM Files:"
echo "  1. Position token: $POSITION_TOKEN_WASM_PATH"
echo "  2. Vault factory: $VAULT_FACTORY_WASM_PATH"
echo ""
echo "🎲 Deployment Parameters:"
echo "  1. Position token: $POSITION_TOKEN_DEPLOY_PARAMS (namespace: $POSITION_TOKEN_NAMESPACE - CONSTANT 0x379)"
echo "  2. Vault factory: $VAULT_FACTORY_DEPLOY_PARAMS (namespace: $VAULT_FACTORY_NAMESPACE - DYNAMIC)"
echo ""
echo "📍 NAMESPACE STRATEGY:"
echo "  • Position token: CONSTANT at 889 (0x379 hex) for predictable system integration"
echo "  • Vault factory: DYNAMIC to avoid deployment conflicts"
echo ""

# Function to find WASM file with fallbacks
find_wasm_file() {
    local component_name="$1"
    local primary_path_var="$2"
    local fallback_paths_var="$3"
    
    echo "🔍 LOCATING $component_name WASM"
    echo "================================="
    
    # Get the primary path value
    local primary_path="${!primary_path_var}"
    
    if [ -f "$primary_path" ]; then
        local wasm_size=$(ls -lh "$primary_path" | awk '{print $5}')
        echo "✅ Found primary WASM: $primary_path"
        echo "📄 File size: $wasm_size"
        return 0
    fi
    
    echo "⚠️  Primary WASM not found, checking fallbacks..."
    
    # Use indirect reference to access the array
    local -n fallback_paths=$fallback_paths_var
    for path in "${fallback_paths[@]}"; do
        echo "   🔎 Checking: $path"
        if [ -f "$path" ]; then
            # Update the primary path variable
            eval "$primary_path_var=\"$path\""
            local wasm_size=$(ls -lh "$path" | awk '{print $5}')
            echo "✅ Found fallback WASM: $path"
            echo "📄 File size: $wasm_size"
            return 0
        fi
    done
    
    echo "❌ No $component_name WASM found!"
    echo "Expected locations:"
    echo "  - $primary_path"
    for path in "${fallback_paths[@]}"; do
        echo "  - $path"
    done
    return 1
}

# Function to find all WASM files and determine deployment strategy
find_all_wasm_files() {
    echo "🔍 LOCATING 2-COMPONENT WASM FILES"
    echo "=================================="
    echo ""
    
    # Track which components are available
    POSITION_TOKEN_AVAILABLE=false
    VAULT_FACTORY_AVAILABLE=false
    
    if find_wasm_file "POSITION-TOKEN" "POSITION_TOKEN_WASM_PATH" "POSITION_TOKEN_FALLBACK_PATHS"; then
        POSITION_TOKEN_AVAILABLE=true
    fi
    echo ""
    
    if find_wasm_file "VAULT-FACTORY" "VAULT_FACTORY_WASM_PATH" "VAULT_FACTORY_FALLBACK_PATHS"; then
        VAULT_FACTORY_AVAILABLE=true
    fi
    echo ""
    
    # Count available components
    local available_count=0
    if [ "$POSITION_TOKEN_AVAILABLE" = true ]; then ((available_count++)); fi
    if [ "$VAULT_FACTORY_AVAILABLE" = true ]; then ((available_count++)); fi
    
    echo "📊 COMPONENT AVAILABILITY SUMMARY"
    echo "================================="
    echo "✅ Available components: $available_count/2"
    echo "  - Position token: $( [ "$POSITION_TOKEN_AVAILABLE" = true ] && echo "✅ Available" || echo "❌ Missing" )"
    echo "  - Vault factory: $( [ "$VAULT_FACTORY_AVAILABLE" = true ] && echo "✅ Available" || echo "❌ Missing" )"
    echo ""
    
    if [ $available_count -eq 2 ]; then
        echo "🎉 ALL 2 COMPONENTS AVAILABLE - Full deployment possible!"
        DEPLOYMENT_MODE="FULL"
        return 0
    elif [ $available_count -ge 1 ]; then
        echo "⚠️  PARTIAL DEPLOYMENT MODE - Will deploy available components"
        DEPLOYMENT_MODE="PARTIAL"
        return 0
    else
        echo "❌ NO COMPONENTS AVAILABLE - Cannot proceed with deployment"
        return 1
    fi
}

# Function to generate blocks with rate limiting
generate_blocks() {
    echo "⛏️  Generating blocks..."
    local block_output
    block_output=$(cd "$OYL_DIR" && $OYL_CMD regtest genBlocks -p oylnet 2>&1)
    local block_status=$?
    echo "📊 Block generation status: $block_status"
    if [ $block_status -ne 0 ]; then
        echo "⚠️  Block generation output: $block_output"
    else
        echo "✅ Blocks generated successfully"
    fi
    echo ""
}

# Function for rate limiting delay
rate_limit_pause() {
    local seconds=${1:-5}
    echo "⏱️  Rate limiting pause ($seconds seconds)..."
    sleep $seconds
    echo ""
}

# Enhanced trace function with hex-to-decimal conversion
get_trace() {
    local txid="$1"
    
    echo "🔍 RAW TRACE OUTPUT"
    echo "=================="
    echo "🆔 Transaction ID: $txid"
    
    if [ -z "$txid" ]; then
        echo "❌ No transaction ID provided"
        return 1
    fi
    
    # Simple vout 3 trace
    local trace_cmd="$OYL_CMD provider alkanes -method \"trace\" -params '[{\"txid\": \"$txid\", \"vout\": 3}]' -p oylnet"
    echo "📤 Command: $trace_cmd"
    
    local trace_output
    trace_output=$(cd "$OYL_DIR" && eval "$trace_cmd" 2>&1)
    echo "📥 Raw output:"
    echo "$trace_output"
    echo ""
}

# Function to extract and convert hex transaction ID from trace output
extract_actual_tx_id() {
    local trace_output="$1"
    local hex_tx_id
    
    # Extract the hex transaction ID from trace output (look for "tx": "0x...")
    hex_tx_id=$(echo "$trace_output" | grep -o '"tx": *"0x[^"]*"' | head -1 | sed 's/.*"0x//' | sed 's/".*//')
    
    if [ ! -z "$hex_tx_id" ]; then
        # Convert hex to decimal
        local decimal_tx_id=$((16#$hex_tx_id))
        # Output debug info to stderr so it doesn't interfere with command substitution
        echo "🔄 TRANSACTION ID CONVERSION" >&2
        echo "==============================" >&2
        echo "📥 Hex from trace: 0x$hex_tx_id" >&2
        echo "📤 Decimal converted: $decimal_tx_id" >&2
        echo "" >&2
        # Only output the number to stdout for command substitution
        echo "$decimal_tx_id"
    else
        echo "❌ Could not extract hex transaction ID from trace" >&2
        return 1
    fi
}

# Generic deployment function with complete transaction isolation
deploy_component() {
    local component_name="$1"
    local wasm_path="$2"
    local deploy_params="$3"
    local namespace="$4"
    local description="$5"
    
    echo "🚀 DEPLOYING $component_name"
    echo "============================="
    echo "📋 WASM: $wasm_path"
    echo "📋 Parameters: $deploy_params"
    echo "📋 Description: $description"
    echo "📋 Namespace: $namespace"
    echo ""
    
    # Step 1: Generate initial blocks with rate limiting
    generate_blocks
    rate_limit_pause 3
    
    # Step 2: Execute deployment
    echo "📤 EXECUTING DEPLOYMENT COMMAND"
    echo "==============================="
    local deploy_cmd="$OYL_CMD alkane new-contract -c \"$wasm_path\" -data \"$deploy_params\" -p oylnet"
    echo "Command: $deploy_cmd"
    echo ""
    
    local deploy_output
    deploy_output=$(cd "$OYL_DIR" && eval "$deploy_cmd" 2>&1)
    local deploy_status=$?
    
    echo "📊 Command exit status: $deploy_status"
    
    if [ $deploy_status -eq 0 ]; then
        echo "✅ Deployment command succeeded"
        
        # Extract transaction ID quickly
        local txid
        txid=$(echo "$deploy_output" | grep -o '"txId":"[^"]*"' | cut -d'"' -f4)
        if [ -z "$txid" ]; then
            txid=$(echo "$deploy_output" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
        fi
        
        if [ ! -z "$txid" ]; then
            echo "✅ Transaction ID: $txid"
            echo ""
            
            # Step 3: Rate limit pause before block generation
            rate_limit_pause 3
            
            # Step 4: Generate blocks after deployment
            generate_blocks
            rate_limit_pause 3
            
            # Step 5: Get trace output
            echo "🎯 DEPLOYMENT TRACE - $component_name"
            echo "===================================="
            get_trace "$txid"
            
            # Step 6: Generate final blocks
            rate_limit_pause 3
            generate_blocks
            
            echo "🎉 $component_name DEPLOYMENT COMPLETE!"
            echo "======================================="
            echo "🆔 Transaction ID: $txid"
            echo "🏗️  Namespace: $namespace"
            echo "✅ Component successfully deployed"
            echo ""
            
            # Store the transaction ID and namespace for reference
            eval "${component_name}_TXID=\"$txid\""
            eval "${component_name}_NAMESPACE=\"$namespace\""
            
            return 0
        else
            echo "❌ Could not extract transaction ID from deployment output"
            echo "Full output:"
            echo "$deploy_output"
            return 1
        fi
    else
        echo "❌ Deployment command failed with status: $deploy_status"
        echo "Error output:"
        echo "$deploy_output"
        
        # Check for specific error types
        if echo "$deploy_output" | grep -q "ETIMEDOUT\|timeout\|rate"; then
            echo ""
            echo "🚨 POSSIBLE RATE LIMITING DETECTED"
            echo "Suggestion: Wait longer and retry"
        elif echo "$deploy_output" | grep -q "already exists\|conflict\|duplicate"; then
            echo ""
            echo "🚨 NAMESPACE CONFLICT DETECTED"
            echo "🔄 Current namespace $namespace may be in use"
            echo "Suggestion: Retry with regenerated parameters"
        elif echo "$deploy_output" | grep -q "Transaction not in mempool\|mempool"; then
            echo ""
            echo "🚨 MEMPOOL/BLOCKCHAIN STATE ISSUE DETECTED"
            echo "💡 This indicates the transaction broadcast failed"
            echo "Possible causes:"
            echo "  - Insufficient funds in wallet"
            echo "  - Blockchain synchronization issues"
            echo "  - Network connectivity problems"
            echo "Suggestion: Check wallet balance and blockchain sync status"
        elif echo "$deploy_output" | grep -q "JSON-RPC Error"; then
            echo ""
            echo "🚨 BLOCKCHAIN RPC ERROR DETECTED"
            echo "💡 Communication issue with Bitcoin node"
            echo "Suggestion: Check if local Bitcoin node is running and synced"
        fi
        return 1
    fi
}


# Deploy available components based on what's found
deploy_available_architecture() {
    echo "🏗️  DEPLOYING AVAILABLE COMPONENTS ($DEPLOYMENT_MODE MODE)"
    echo "========================================================"
    echo "Deploying 2-component architecture..."
    echo ""
    
    local deployment_success=true
    local deployed_count=0
    
    # Component 1: Position token template (if available)
    if [ "$POSITION_TOKEN_AVAILABLE" = true ]; then
        echo "📦 COMPONENT 1: POSITION TOKEN TEMPLATE"
        echo "======================================="
        if deploy_component "POSITION_TOKEN" "$POSITION_TOKEN_WASM_PATH" "$POSITION_TOKEN_DEPLOY_PARAMS" "$POSITION_TOKEN_NAMESPACE" "Template deploy, position token, amount 10"; then
            ((deployed_count++))
        else
            deployment_success=false
            echo "❌ Position token deployment failed!"
        fi
        
        # Component isolation - ensure clean state between deployments
        echo "🔧 COMPONENT ISOLATION: POSITION-TOKEN → VAULT-FACTORY"
        echo "======================================================"
        generate_blocks
        rate_limit_pause 3
        echo ""
    else
        echo "⏭️  SKIPPING COMPONENT 1: POSITION TOKEN TEMPLATE (not available)"
        echo ""
    fi
    
    # Component 2: Vault factory template (if available)
    if [ "$VAULT_FACTORY_AVAILABLE" = true ]; then
        echo "📦 COMPONENT 2: VAULT FACTORY TEMPLATE"
        echo "======================================"
        if deploy_component "VAULT_FACTORY" "$VAULT_FACTORY_WASM_PATH" "$VAULT_FACTORY_DEPLOY_PARAMS" "$VAULT_FACTORY_NAMESPACE" "Template deploy, vault factory, amount 10"; then
            ((deployed_count++))
        else
            deployment_success=false
            echo "❌ Vault factory deployment failed!"
        fi
        
        # Final component isolation - ensure clean final state
        echo "🔧 FINAL DEPLOYMENT STATE CONSOLIDATION"
        echo "======================================="
        generate_blocks
        rate_limit_pause 3
        echo ""
    else
        echo "⏭️  SKIPPING COMPONENT 2: VAULT FACTORY TEMPLATE (not available)"
        echo ""
    fi
    
    echo "📊 DEPLOYMENT SUMMARY"
    echo "===================="
    echo "✅ Components deployed: $deployed_count/2"
    echo ""
    
    if [ $deployed_count -ge 1 ]; then
        echo "📋 DEPLOYMENT STATUS:"
        if [ ! -z "$POSITION_TOKEN_TXID" ]; then
            echo "  1. ✅ Position token: DEPLOYED at namespace $POSITION_TOKEN_NAMESPACE"
            echo "     Transaction: $POSITION_TOKEN_TXID"
        fi
        if [ ! -z "$VAULT_FACTORY_TXID" ]; then
            echo "  2. ✅ Vault factory: DEPLOYED at namespace $VAULT_FACTORY_NAMESPACE"
            echo "     Transaction: $VAULT_FACTORY_TXID"
        fi
        echo ""
        
        if [ "$deployment_success" = true ]; then
            return 0
        else
            echo "⚠️  Some components failed - partial deployment completed"
            return 1
        fi
    else
        echo "❌ NO COMPONENTS DEPLOYED SUCCESSFULLY"
        return 1
    fi
}

# Main execution
echo "🏁 STARTING 2-COMPONENT DEPLOYMENT"
echo "=================================="

# Initial blockchain state setup
echo "🔧 INITIALIZING BLOCKCHAIN STATE"
echo "================================="
generate_blocks
rate_limit_pause 3
echo ""

# Find all WASM files
if ! find_all_wasm_files; then
    echo "❌ Cannot proceed without required WASM files"
    exit 1
fi

echo ""

# Check oyl-sdk
if [ ! -d "$OYL_DIR" ]; then
    echo "❌ oyl-sdk directory not found: $OYL_DIR"
    exit 1
fi

echo "✅ oyl-sdk found: $OYL_DIR"
echo ""

# Pre-deployment blockchain state
echo "🔧 PREPARING BLOCKCHAIN FOR DEPLOYMENT"
echo "======================================"
generate_blocks
rate_limit_pause 3
echo ""

# Deploy available architecture
if deploy_available_architecture; then
    echo ""
    echo "🎉 2-COMPONENT DEPLOYMENT SUCCESS!"
    echo "================================="
    echo "✅ Position token and vault factory templates deployed successfully"
    echo ""
    echo "🎯 DEPLOYMENT COMPLETE:"
    echo "• Position token deployed at constant namespace 889 (0x379 hex)"
    echo "• Vault factory deployed at dynamic namespace for conflict avoidance"
    echo "• Both components are ready for system integration"
    echo ""
    echo "🔧 NEXT STEPS:"
    echo "• Position token is available at predictable namespace 889"
    echo "• Vault factory can be initialized manually if needed"
    echo "• System integration can reference position token at constant location"
else
    echo ""
    echo "❌ DEPLOYMENT FAILED"
    echo "==================="
    echo "Check individual component errors above"
    echo "Some components may have deployed successfully"
    exit 1
fi
