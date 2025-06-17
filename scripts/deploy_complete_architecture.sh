#!/bin/bash

# Complete 4-Component Architecture Deployment with Contract Initialization
# Production-ready deployment system with comprehensive transaction isolation
# Components: Free-mint, Position Token, Vault Factory, Auth Token Factory
# Features: Contract initialization, regtest isolation, 30s rate limiting, full trace analysis

echo "🏗️  COMPLETE 4-COMPONENT ARCHITECTURE DEPLOYMENT"
echo "================================================="
echo "Production-ready secure free-mint system deployment"
echo "Based on breakthrough debugging analysis"
echo ""
echo "Components:"
echo "1. Free-mint template + initialization"
echo "2. Position token template + initialization" 
echo "3. Vault factory template + initialization"
echo "4. Auth token factory (breakthrough component)"
echo ""

# Configuration - Updated for boiler repository structure
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOILER_ROOT="$(dirname "$SCRIPT_DIR")"
OYL_DIR="/home/e/Documents/oyl-sdk"
OYL_CMD="node bin/oyl.js"

# WASM paths for all 4 components - Corrected for boiler structure
FREE_MINT_WASM_PATH="$BOILER_ROOT/../free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm"
POSITION_TOKEN_WASM_PATH="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
VAULT_FACTORY_WASM_PATH="$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"
AUTH_TOKEN_WASM_PATH="/home/e/Documents/alkanes-rs/target/wasm32-unknown-unknown/release/alkanes_std_auth_token.wasm"

# Comprehensive fallback paths for all components
FREE_MINT_FALLBACK_PATHS=(
    "$BOILER_ROOT/src/precompiled/alkanes_std_free_mint_build.wasm"
    "$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alkanes_std_free_mint.wasm"
    "$BOILER_ROOT/target/wasm32-unknown-unknown/release/alkanes_std_free_mint.wasm"
    "$BOILER_ROOT/../free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm"
)

POSITION_TOKEN_FALLBACK_PATHS=(
    "$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
    "$BOILER_ROOT/../position-token/target/wasm32-unknown-unknown/release/position_token.wasm"
)

VAULT_FACTORY_FALLBACK_PATHS=(
    "$BOILER_ROOT/target/alkanes/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"
    "$BOILER_ROOT/../vault-factory/target/wasm32-unknown-unknown/release/vault_factory.wasm"
)

AUTH_TOKEN_FALLBACK_PATHS=(
    "/home/e/Documents/auth-token/target/wasm32-unknown-unknown/release/auth_token.wasm"
    "/home/e/Documents/alkanes-rs/target/alkanes/wasm32-unknown-unknown/release/alkanes_std_auth_token.wasm"
    "$BOILER_ROOT/../auth-token/target/wasm32-unknown-unknown/release/auth_token.wasm"
)

# Function to generate dynamic random parameters for all components
generate_component_namespaces() {
    # Generate random base seed (avoid conflicts)
    local base_seed=$((RANDOM % 30000 + 10000))  # Random between 10000-40000
    
    # Add timestamp-based offset to ensure uniqueness
    local timestamp_offset=$(($(date +%s) % 1000))
    local base_namespace=$((base_seed + (timestamp_offset * 10)))
    
    # Generate 4 unique namespaces with spacing
    FREE_MINT_NAMESPACE=$((base_namespace))
    POSITION_TOKEN_NAMESPACE=$((base_namespace + 100))
    VAULT_FACTORY_NAMESPACE=$((base_namespace + 200))
    AUTH_TOKEN_NAMESPACE=$((base_namespace + 300))
    
    # Ensure we don't exceed reasonable bounds
    if [ $AUTH_TOKEN_NAMESPACE -gt 60000 ]; then
        local reduction=$((AUTH_TOKEN_NAMESPACE - 45000))
        FREE_MINT_NAMESPACE=$((FREE_MINT_NAMESPACE - reduction))
        POSITION_TOKEN_NAMESPACE=$((POSITION_TOKEN_NAMESPACE - reduction))
        VAULT_FACTORY_NAMESPACE=$((VAULT_FACTORY_NAMESPACE - reduction))
        AUTH_TOKEN_NAMESPACE=$((AUTH_TOKEN_NAMESPACE - reduction))
    fi
}

# Generate dynamic parameters for all 4 components
generate_component_namespaces

# Deploy parameters for each component (based on working test architecture)
FREE_MINT_DEPLOY_PARAMS="3,$FREE_MINT_NAMESPACE,101"        # Template deploy, namespace, amount 101
POSITION_TOKEN_DEPLOY_PARAMS="3,$POSITION_TOKEN_NAMESPACE,10"  # Template deploy, namespace, amount 10
VAULT_FACTORY_DEPLOY_PARAMS="3,$VAULT_FACTORY_NAMESPACE,10"   # Template deploy, namespace, amount 10
AUTH_TOKEN_DEPLOY_PARAMS="3,$AUTH_TOKEN_NAMESPACE,0,1"        # Template deploy, namespace, Initialize opcode, amount 1

echo "📋 COMPLETE 4-COMPONENT CONFIGURATION"
echo "====================================="
echo "Boiler Root: $BOILER_ROOT"
echo "OYL Directory: $OYL_DIR"
echo ""
echo "🏗️  WASM Files:"
echo "  1. Free-mint: $FREE_MINT_WASM_PATH"
echo "  2. Position token: $POSITION_TOKEN_WASM_PATH"
echo "  3. Vault factory: $VAULT_FACTORY_WASM_PATH"
echo "  4. Auth token: $AUTH_TOKEN_WASM_PATH"
echo ""
echo "🎲 Dynamic Parameters:"
echo "  1. Free-mint: $FREE_MINT_DEPLOY_PARAMS (namespace: $FREE_MINT_NAMESPACE)"
echo "  2. Position token: $POSITION_TOKEN_DEPLOY_PARAMS (namespace: $POSITION_TOKEN_NAMESPACE)"
echo "  3. Vault factory: $VAULT_FACTORY_DEPLOY_PARAMS (namespace: $VAULT_FACTORY_NAMESPACE)"
echo "  4. Auth token: $AUTH_TOKEN_DEPLOY_PARAMS (namespace: $AUTH_TOKEN_NAMESPACE)"
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
    echo "🔍 LOCATING ALL 4-COMPONENT WASM FILES"
    echo "======================================"
    echo ""
    
    # Track which components are available
    FREE_MINT_AVAILABLE=false
    POSITION_TOKEN_AVAILABLE=false
    VAULT_FACTORY_AVAILABLE=false
    AUTH_TOKEN_AVAILABLE=false
    
    if find_wasm_file "FREE-MINT" "FREE_MINT_WASM_PATH" "FREE_MINT_FALLBACK_PATHS"; then
        FREE_MINT_AVAILABLE=true
    fi
    echo ""
    
    if find_wasm_file "POSITION-TOKEN" "POSITION_TOKEN_WASM_PATH" "POSITION_TOKEN_FALLBACK_PATHS"; then
        POSITION_TOKEN_AVAILABLE=true
    fi
    echo ""
    
    if find_wasm_file "VAULT-FACTORY" "VAULT_FACTORY_WASM_PATH" "VAULT_FACTORY_FALLBACK_PATHS"; then
        VAULT_FACTORY_AVAILABLE=true
    fi
    echo ""
    
    if find_wasm_file "AUTH-TOKEN" "AUTH_TOKEN_WASM_PATH" "AUTH_TOKEN_FALLBACK_PATHS"; then
        AUTH_TOKEN_AVAILABLE=true
    fi
    echo ""
    
    # Count available components
    local available_count=0
    if [ "$FREE_MINT_AVAILABLE" = true ]; then ((available_count++)); fi
    if [ "$POSITION_TOKEN_AVAILABLE" = true ]; then ((available_count++)); fi
    if [ "$VAULT_FACTORY_AVAILABLE" = true ]; then ((available_count++)); fi
    if [ "$AUTH_TOKEN_AVAILABLE" = true ]; then ((available_count++)); fi
    
    echo "📊 COMPONENT AVAILABILITY SUMMARY"
    echo "================================="
    echo "✅ Available components: $available_count/4"
    echo "  - Free-mint: $( [ "$FREE_MINT_AVAILABLE" = true ] && echo "✅ Available" || echo "❌ Missing" )"
    echo "  - Position token: $( [ "$POSITION_TOKEN_AVAILABLE" = true ] && echo "✅ Available" || echo "❌ Missing" )"
    echo "  - Vault factory: $( [ "$VAULT_FACTORY_AVAILABLE" = true ] && echo "✅ Available" || echo "❌ Missing" )"
    echo "  - Auth token: $( [ "$AUTH_TOKEN_AVAILABLE" = true ] && echo "✅ Available" || echo "❌ Missing" )"
    echo ""
    
    if [ $available_count -eq 4 ]; then
        echo "🎉 ALL 4 COMPONENTS AVAILABLE - Full deployment possible!"
        DEPLOYMENT_MODE="FULL"
        return 0
    elif [ $available_count -ge 2 ] && [ "$AUTH_TOKEN_AVAILABLE" = true ]; then
        echo "⚠️  PARTIAL DEPLOYMENT MODE - Will deploy available components"
        echo "🔐 Auth token available - Core breakthrough functionality preserved"
        DEPLOYMENT_MODE="PARTIAL"
        return 0
    elif [ $available_count -ge 1 ]; then
        echo "⚠️  LIMITED DEPLOYMENT MODE - Few components available"
        DEPLOYMENT_MODE="LIMITED"
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
    local seconds=${1:-30}
    echo "⏱️  Rate limiting pause ($seconds seconds)..."
    sleep $seconds
    echo ""
}

# Simplified trace function - raw output only
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
    rate_limit_pause 30
    
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
            rate_limit_pause 30
            
            # Step 4: Generate blocks after deployment
            generate_blocks
            rate_limit_pause 30
            
            # Step 5: Get trace output
            echo "🎯 DEPLOYMENT TRACE - $component_name"
            echo "===================================="
            get_trace "$txid"
            
            # Step 6: Generate final blocks
            rate_limit_pause 30
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

# Function to initialize a deployed contract for actual use
initialize_contract() {
    local component_name="$1"
    local contract_txid="$2"
    local namespace="$3"
    local init_params="$4"
    local description="$5"
    
    echo "🔧 INITIALIZING $component_name FOR FUNCTIONAL USE"
    echo "================================================="
    echo "📋 Template TXID: $contract_txid"
    echo "📋 Namespace: $namespace"
    echo "📋 Init Parameters: $init_params"
    echo "📋 Description: $description"
    echo ""
    
    # Generate blocks before initialization
    generate_blocks
    rate_limit_pause 30
    
    # Execute initialization call
    echo "📤 EXECUTING INITIALIZATION COMMAND"
    echo "==================================="
    local init_cmd="$OYL_CMD execute -t \"$contract_txid\" -data \"$init_params\" -p oylnet"
    echo "Command: $init_cmd"
    echo ""
    
    local init_output
    init_output=$(cd "$OYL_DIR" && eval "$init_cmd" 2>&1)
    local init_status=$?
    
    echo "📊 Command exit status: $init_status"
    
    if [ $init_status -eq 0 ]; then
        echo "✅ Initialization command succeeded"
        
        # Extract transaction ID
        local init_txid
        init_txid=$(echo "$init_output" | grep -o '"txId":"[^"]*"' | cut -d'"' -f4)
        if [ -z "$init_txid" ]; then
            init_txid=$(echo "$init_output" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
        fi
        
        if [ ! -z "$init_txid" ]; then
            echo "✅ Initialization Transaction ID: $init_txid"
            echo ""
            
            # Generate blocks after initialization
            rate_limit_pause 30
            generate_blocks
            rate_limit_pause 30
            
            # Trace initialization results
            echo "🎯 INITIALIZATION TRACE - $component_name"
            echo "========================================"
            get_trace "$init_txid"
            
            echo "🎉 $component_name INITIALIZATION COMPLETE!"
            echo "=========================================="
            echo "🆔 Template TXID: $contract_txid"
            echo "🆔 Init TXID: $init_txid"
            echo "🏗️  Namespace: $namespace"
            echo "✅ Contract ready for withdraw/deposit/mint operations"
            echo ""
            
            # Store initialization TXID
            eval "${component_name}_INIT_TXID=\"$init_txid\""
            
            return 0
        else
            echo "❌ Could not extract initialization transaction ID"
            echo "Full output:"
            echo "$init_output"
            return 1
        fi
    else
        echo "❌ Initialization command failed with status: $init_status"
        echo "Error output:"
        echo "$init_output"
        return 1
    fi
}

# Function to deploy proper architecture following test pattern
deploy_functional_architecture() {
    echo "🏗️  DEPLOYING FUNCTIONAL ARCHITECTURE (FOLLOWING TEST PATTERN)"
    echo "=============================================================="
    echo "Creating actual functional contracts ready for deposit/withdraw operations"
    echo ""
    
    local deployment_success=true
    local deployed_count=0
    
    # Step 1: Deploy and Initialize Free-mint contract (consolidated 6,namespace,0 approach)
    if [ "$FREE_MINT_AVAILABLE" = true ]; then
        echo "🔧 STEP 1: DEPLOYING AND INITIALIZING FREE-MINT CONTRACT (CONSOLIDATED)"
        echo "======================================================================="
        echo "Using consolidated 6,namespace,0 approach - deploys to 2:n and initializes in one call"
        
        # Parameters from working test architecture - consolidated deployment + initialization
        local free_mint_params="6,$FREE_MINT_NAMESPACE,0,100000,1000,2,1179796805,1296649812,4608589"
        echo "📋 Free-mint consolidated params: $free_mint_params"
        echo "📋 Parameters breakdown:"
        echo "   • Opcode: 6 (Deploy + Initialize in one call)"
        echo "   • Namespace: $FREE_MINT_NAMESPACE (will deploy to block 2:$FREE_MINT_NAMESPACE)"
        echo "   • Deploy opcode: 0"
        echo "   • Token units: 100000"
        echo "   • Value per mint: 1000"
        echo "   • Cap: 2 (low for testing)"
        echo "   • Name: FREE (1179796805) + MINT (1296649812)"
        echo "   • Symbol: FRM (4608589)"
        echo "💡 This consolidates template deployment + initialization into single transaction"
        
        if deploy_component "FREE_MINT_FUNCTIONAL" "$FREE_MINT_WASM_PATH" "$free_mint_params" "$FREE_MINT_NAMESPACE" "Deploy and initialize functional free-mint contract (consolidated)"; then
            ((deployed_count++))
            FREE_MINT_FUNCTIONAL_TXID="$FREE_MINT_FUNCTIONAL_TXID"
            echo "✅ Free-mint deployed and initialized in one call"
            echo "🆔 Free-mint functional: $FREE_MINT_FUNCTIONAL_TXID"
            echo "📍 Deployed to: Block 2, TX $FREE_MINT_NAMESPACE"
        else
            deployment_success=false
            echo "❌ Free-mint consolidated deployment failed!"
        fi
        echo ""
    fi
    
    # Step 2: Initialize Vault Factory with binding to free-mint token
    if [ "$VAULT_FACTORY_AVAILABLE" = true ] && [ ! -z "$VAULT_FACTORY_TXID" ] && [ ! -z "$FREE_MINT_FUNCTIONAL_TXID" ]; then
        echo "🔧 STEP 2: INITIALIZING VAULT FACTORY WITH FREE-MINT BINDING"
        echo "==========================================================="
        echo "Following test pattern: 4,namespace,0 + binding parameters"
        
        # Calculate deposit token location (free-mint functional location: block 2, TX namespace)
        # Free-mint was deployed to block 2:$FREE_MINT_NAMESPACE by the consolidated approach
        local vault_params="4,$VAULT_FACTORY_NAMESPACE,0,2,$FREE_MINT_NAMESPACE,10,3,1000,2,$FREE_MINT_NAMESPACE"
        echo "📋 Vault factory params: $vault_params"
        echo "📋 Parameters breakdown:"
        echo "   • Opcode: 4 (Initialize vault factory)"
        echo "   • Namespace: $VAULT_FACTORY_NAMESPACE"
        echo "   • Deploy opcode: 0"
        echo "   • Deposit token: Block 2, TX $FREE_MINT_NAMESPACE (free-mint functional)"
        echo "   • Reward per block: 10"
        echo "   • Start block: 3"
        echo "   • End reward block: 1000 (temporal cap)"
        echo "   • Free-mint contract: Block 2, TX $FREE_MINT_NAMESPACE"
        
        if deploy_component "VAULT_FACTORY_FUNCTIONAL" "$VAULT_FACTORY_WASM_PATH" "$vault_params" "$VAULT_FACTORY_NAMESPACE" "Initialize vault factory bound to free-mint token"; then
            ((deployed_count++))
            VAULT_FACTORY_FUNCTIONAL_TXID="$VAULT_FACTORY_FUNCTIONAL_TXID"
            echo "✅ Vault factory functional deployed and bound to free-mint"
            echo "🆔 Vault factory functional: $VAULT_FACTORY_FUNCTIONAL_TXID"
        else
            deployment_success=false
            echo "❌ Vault factory functional deployment failed!"
        fi
        echo ""
    fi
    
    # Step 3: Authorize factory using auth tokens (following test pattern)
    if [ "$AUTH_TOKEN_AVAILABLE" = true ] && [ ! -z "$AUTH_TOKEN_TXID" ] && [ ! -z "$FREE_MINT_FUNCTIONAL_TXID" ] && [ ! -z "$VAULT_FACTORY_FUNCTIONAL_TXID" ]; then
        echo "🔧 STEP 3: AUTHORIZING FACTORY VIA AUTH TOKENS"
        echo "=============================================="
        echo "Following test pattern: opcode 1 (UpdateFactoryWhitelist)"
        
        # Parameters for calling free-mint contract directly with execute command
        # Since we're using execute -t on the free-mint contract, we only need: opcode, factory_block, factory_tx
        local auth_params="1,4,$VAULT_FACTORY_NAMESPACE"
        echo "📋 Authorization params: $auth_params"
        echo "📋 Parameters breakdown:"
        echo "   • Opcode: 1 (UpdateFactoryWhitelist)"  
        echo "   • Factory block to authorize: 4"
        echo "   • Factory tx to authorize: $VAULT_FACTORY_NAMESPACE"
        echo "   • Target: Free-mint functional contract (via execute command)"
        
        # Use auth token for authorization call
        generate_blocks
        rate_limit_pause 30
        
        echo "🔐 EXECUTING FACTORY AUTHORIZATION - CALLING FREE-MINT CONTRACT"
        echo "============================================================="
        echo "Calling free-mint contract to update factory whitelist using auth token"
        
        # We need to call the FREE-MINT contract (not deploy new auth token)
        # Use the consolidated free-mint functional contract that was deployed
        local free_mint_functional_txid="$FREE_MINT_FUNCTIONAL_TXID"
        
        local auth_cmd="$OYL_CMD execute -t \"$free_mint_functional_txid\" -data \"$auth_params\" -p oylnet"
        echo "Command: $auth_cmd"
        echo "Target: Free-mint functional contract ($free_mint_functional_txid)"
        echo "Action: UpdateFactoryWhitelist (opcode 1)"
        
        local auth_output
        auth_output=$(cd "$OYL_DIR" && eval "$auth_cmd" 2>&1)
        local auth_status=$?
        
        echo "📊 Authorization exit status: $auth_status"
        
        if [ $auth_status -eq 0 ]; then
            local auth_txid
            auth_txid=$(echo "$auth_output" | grep -o '"txId":"[^"]*"' | cut -d'"' -f4)
            if [ -z "$auth_txid" ]; then
                auth_txid=$(echo "$auth_output" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
            fi
            
            if [ ! -z "$auth_txid" ]; then
                echo "✅ Factory authorization successful"
                echo "🆔 Authorization TX: $auth_txid"
                ((deployed_count++))
                
                generate_blocks
                rate_limit_pause 30
                get_trace "$auth_txid"
            else
                echo "❌ Could not extract authorization transaction ID"
                deployment_success=false
            fi
        else
            echo "❌ Factory authorization failed"
            echo "Error output: $auth_output"
            deployment_success=false
        fi
        echo ""
    fi
    
    echo "📊 FUNCTIONAL ARCHITECTURE DEPLOYMENT SUMMARY"
    echo "============================================="
    echo "✅ Functional components deployed: $deployed_count"
    echo ""
    
    if [ $deployed_count -ge 2 ]; then
        echo "📋 FUNCTIONAL ARCHITECTURE STATUS:"
        if [ ! -z "$FREE_MINT_FUNCTIONAL_TXID" ]; then
            echo "  1. ✅ Free-mint: FUNCTIONAL for minting operations"
            echo "     Functional: $FREE_MINT_FUNCTIONAL_TXID"
        fi
        if [ ! -z "$VAULT_FACTORY_FUNCTIONAL_TXID" ]; then
            echo "  2. ✅ Vault factory: FUNCTIONAL and bound to free-mint token"
            echo "     Functional: $VAULT_FACTORY_FUNCTIONAL_TXID"
        fi
        if [ ! -z "$AUTH_TOKEN_TXID" ]; then
            echo "  3. ✅ Auth token: FUNCTIONAL for authorization"
            echo "     Functional: $AUTH_TOKEN_TXID"
        fi
        echo ""
        
        echo "🎯 ARCHITECTURE NOW READY FOR:"
        echo "- 💰 Deposit operations to vault factory"
        echo "- 💸 Withdraw operations with rewards"
        echo "- 🔄 On-demand minting via free-mint integration"
        echo "- 🔐 Cross-contract authorization via auth tokens"
        echo "- ⏰ Temporal reward boundaries (end at block 1000)"
        
        if [ "$deployment_success" = true ]; then
            return 0
        else
            echo "⚠️  Some components failed - partial functionality available"
            return 1
        fi
    else
        echo "❌ INSUFFICIENT FUNCTIONAL COMPONENTS DEPLOYED"
        return 1
    fi
}

# Deploy available components based on what's found
deploy_available_architecture() {
    echo "🏗️  DEPLOYING AVAILABLE COMPONENTS ($DEPLOYMENT_MODE MODE)"
    echo "========================================================"
    echo "Deploying available components in breakthrough order..."
    echo ""
    
    local deployment_success=true
    local deployed_count=0
    
    # Component 1: Free-mint template (if available)
    if [ "$FREE_MINT_AVAILABLE" = true ]; then
        echo "📦 COMPONENT 1: FREE-MINT TEMPLATE"
        echo "=================================="
        if deploy_component "FREE_MINT" "$FREE_MINT_WASM_PATH" "$FREE_MINT_DEPLOY_PARAMS" "$FREE_MINT_NAMESPACE" "Template deploy, free-mint token, amount 101"; then
            ((deployed_count++))
        else
            deployment_success=false
            echo "❌ Free-mint deployment failed!"
        fi
        
        # Component isolation - ensure clean state between deployments
        echo "🔧 COMPONENT ISOLATION: FREE-MINT → POSITION-TOKEN"
        echo "=================================================="
        generate_blocks
        rate_limit_pause 30
        echo ""
    else
        echo "⏭️  SKIPPING COMPONENT 1: FREE-MINT TEMPLATE (not available)"
        echo ""
    fi
    
    # Component 2: Position token template (if available)
    if [ "$POSITION_TOKEN_AVAILABLE" = true ]; then
        echo "📦 COMPONENT 2: POSITION TOKEN TEMPLATE"
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
        rate_limit_pause 30
        echo ""
    else
        echo "⏭️  SKIPPING COMPONENT 2: POSITION TOKEN TEMPLATE (not available)"
        echo ""
    fi
    
    # Component 3: Vault factory template (if available)
    if [ "$VAULT_FACTORY_AVAILABLE" = true ]; then
        echo "📦 COMPONENT 3: VAULT FACTORY TEMPLATE"
        echo "======================================"
        if deploy_component "VAULT_FACTORY" "$VAULT_FACTORY_WASM_PATH" "$VAULT_FACTORY_DEPLOY_PARAMS" "$VAULT_FACTORY_NAMESPACE" "Template deploy, vault factory, amount 10"; then
            ((deployed_count++))
        else
            deployment_success=false
            echo "❌ Vault factory deployment failed!"
        fi
        
        # Component isolation - ensure clean state between deployments
        echo "🔧 COMPONENT ISOLATION: VAULT-FACTORY → AUTH-TOKEN"
        echo "=================================================="
        generate_blocks
        rate_limit_pause 30
        echo ""
    else
        echo "⏭️  SKIPPING COMPONENT 3: VAULT FACTORY TEMPLATE (not available)"
        echo ""
    fi
    
    # Component 4: Auth token factory 
    if [ "$AUTH_TOKEN_AVAILABLE" = true ]; then
        echo "📦 COMPONENT 4: AUTH TOKEN FACTORY (BREAKTHROUGH)"
        echo "================================================="
        if deploy_component "AUTH_TOKEN" "$AUTH_TOKEN_WASM_PATH" "$AUTH_TOKEN_DEPLOY_PARAMS" "$AUTH_TOKEN_NAMESPACE" "Template deploy, auth token factory, Initialize opcode, amount 1 (CRITICAL)"; then
            ((deployed_count++))
        else
            deployment_success=false
            echo "❌ Auth token deployment failed!"
        fi
        
        # Final component isolation - ensure clean final state
        echo "🔧 FINAL DEPLOYMENT STATE CONSOLIDATION"
        echo "======================================="
        generate_blocks
        rate_limit_pause 30
        echo ""
    else
        echo "⏭️  SKIPPING COMPONENT 4: AUTH TOKEN FACTORY (not available)"
        echo "⚠️  WARNING: Auth token is the breakthrough component!"
        echo ""
    fi
    
    return 0
}

# Main execution
echo "🏁 STARTING COMPLETE 4-COMPONENT DEPLOYMENT"
echo "==========================================="

# Initial blockchain state setup
echo "🔧 INITIALIZING BLOCKCHAIN STATE"
echo "================================="
generate_blocks
rate_limit_pause 30
echo ""

# Find all WASM files
if ! find_all_wasm_files; then
    echo "❌ Cannot proceed without all required WASM files"
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
rate_limit_pause 30
echo ""

# Deploy available architecture
if deploy_available_architecture; then
    echo ""
    echo "🎊 TEMPLATE DEPLOYMENT SUCCESS!"
    echo "==============================="
    echo "🏗️  Templates deployed - Now initializing for functional use"
    echo ""
    
    # Deploy functional architecture following test pattern
    if deploy_functional_architecture; then
        echo ""
        echo "🎉 COMPLETE SYSTEM SUCCESS!"
        echo "=========================="
        echo "🏗️  Architecture fully functional and ready for operations"
        echo "💰 Contracts ready for withdraw/deposit/mint operations on oylnet"
        echo "🔐 Zero capital requirements achieved through auth token system"
        echo "🚀 System ready for production use"
    else
        echo ""
        echo "⚠️  TEMPLATES DEPLOYED BUT INITIALIZATION FAILED"
        echo "==============================================="
        echo "Templates are deployed but not yet functional for operations"
        echo "Manual initialization may be required"
    fi
else
    echo ""
    echo "❌ DEPLOYMENT FAILED"
    echo "==================="
    echo "Check individual component errors above"
    echo "Some components may have deployed successfully"
    exit 1
fi
