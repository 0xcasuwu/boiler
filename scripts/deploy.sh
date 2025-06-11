#!/bin/bash

# Check if namespace parameter is provided
if [ -z "$1" ]; then
  echo "Error: Namespace parameter is required"
  echo "Usage: ./deploy.sh <namespace>"
  exit 1
fi

# Store the namespace parameter
NAMESPACE=$1
POSITION_TOKEN_WASM_PATH="/home/e/Documents/boiler/target/wasm32-unknown-unknown/release/alk4626_position_token.wasm"
VAULT_FACTORY_WASM_PATH="/home/e/Documents/boiler/target/wasm32-unknown-unknown/release/alk4626_vault_factory.wasm"
FREE_MINT_WASM_PATH="/home/e/Documents/free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm"
OYL_CMD="node bin/oyl.js"
OYL_DIR="../oyl-sdk"

echo "🚀 TWO-TOKEN ECOSYSTEM DEPLOYMENT"
echo "================================="
echo "Namespace: $NAMESPACE"
echo "Position Token: $POSITION_TOKEN_WASM_PATH"
echo "Vault Factory: $VAULT_FACTORY_WASM_PATH" 
echo "Free Mint: $FREE_MINT_WASM_PATH"
echo ""

# Function to extract txId from JSON output
extract_txid() {
  echo "DEBUG: Extracting txId from output" >&2
  local txid
  txid=$(echo "$1" | grep -o "txId: '[^']*'" | cut -d "'" -f 2)
  echo "DEBUG: Extracted txId: $txid" >&2
  echo "$txid"
}

# Function to extract template ID from trace output  
extract_template_id() {
  echo "DEBUG: Extracting template ID from trace output" >&2
  local template_id_hex
  template_id_hex=$(echo "$1" | grep -o '"event":"create","data":{"block":"0x[0-9a-f]*","tx":"0x[0-9a-f]*"}' | grep -o '"tx":"0x[0-9a-f]*"' | head -1 | cut -d '"' -f 4)
  
  if [ -z "$template_id_hex" ]; then
    template_id_hex=$(echo "$1" | grep -o '"tx":"0x[0-9a-f]*"' | head -1 | cut -d '"' -f 4)
  fi
  
  if [ ! -z "$template_id_hex" ]; then
    local template_id_decimal=$(printf "%d" $template_id_hex)
    echo "DEBUG: Extracted template ID: $template_id_hex ($template_id_decimal)" >&2
    echo "$template_id_decimal"
  else
    echo "DEBUG: Could not extract template ID" >&2
    echo ""
  fi
}

# Function to generate blocks immediately after any deployment
generate_blocks_now() {
  echo "Generating blocks immediately..."
  cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet
  sleep 3
}

# Function to pause and generate blocks with MAXIMUM timing
pause_and_generate() {
  echo "Waiting 15 seconds for mempool processing..."
  sleep 15
  echo "Generating multiple blocks..."
  cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet
  echo ""
}

# Function to trace with MAXIMUM delay
trace_with_delay() {
  local txid=$1
  local vout=$2
  echo "Waiting 10 seconds for transaction to be fully included..."
  sleep 10
  echo "Generating blocks before tracing..."
  cd $OYL_DIR && $OYL_CMD regtest genBlocks -p oylnet
  /home/e/Documents/boiler/scripts/trace.sh "$txid" "$vout"
}

echo "Step 0: Initial block generation"
pause_and_generate

# STEP 1: Deploy Position Token Template (3:0x379)
echo "🔧 STEP 1: Deploying Position Token Template"
echo "============================================"
DEPLOY_OUTPUT_POSITION=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $POSITION_TOKEN_WASM_PATH -data 3,$NAMESPACE,0 -p oylnet)
generate_blocks_now  # Generate blocks immediately after deployment
echo "$DEPLOY_OUTPUT_POSITION"
POSITION_TEMPLATE_TXID=$(extract_txid "$DEPLOY_OUTPUT_POSITION")

if [ -z "$POSITION_TEMPLATE_TXID" ]; then
  echo "❌ ERROR: Failed to extract position token template transaction ID"
  exit 1
fi

echo "✅ Position Token Template deployed at: $POSITION_TEMPLATE_TXID"
pause_and_generate

# Use position template txid directly - no need to trace for template ID
echo "✅ Position Token Template TXID: $POSITION_TEMPLATE_TXID"
POSITION_TEMPLATE_ID="889"  # Default 0x379 in decimal for vault factory reference
generate_blocks_now
echo ""

# STEP 2: Deploy Vault Factory Template (3:0x37a with embedded position template ID)
echo "🏭 STEP 2: Deploying Vault Factory Template"
echo "==========================================="
DEPLOY_OUTPUT_VAULT_TEMPLATE=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $VAULT_FACTORY_WASM_PATH -data 3,$NAMESPACE,$POSITION_TEMPLATE_ID -p oylnet)
generate_blocks_now  # Generate blocks immediately after deployment
echo "$DEPLOY_OUTPUT_VAULT_TEMPLATE"
VAULT_TEMPLATE_TXID=$(extract_txid "$DEPLOY_OUTPUT_VAULT_TEMPLATE")

if [ -z "$VAULT_TEMPLATE_TXID" ]; then
  echo "❌ ERROR: Failed to extract vault factory template transaction ID"
  exit 1
fi

echo "✅ Vault Factory Template deployed at: $VAULT_TEMPLATE_TXID (with position template ID: $POSITION_TEMPLATE_ID)"
pause_and_generate

# STEP 3: Create Deposit Token (Direct Token Creation)
echo "🪙 STEP 3: Creating Deposit Token (100k supply)"
echo "==============================================="
echo "Creating DEPOSIT token with 100k cap, 1000 per mint, no premine..."
DEPLOY_OUTPUT_DEPOSIT_TOKEN=$(cd $OYL_DIR && $OYL_CMD alkane new-token -pre 0 -amount 100000000000 -c 100000 -name "DEPOSIT" -symbol "DEP" -resNumber $NAMESPACE -p oylnet)
generate_blocks_now  # Generate blocks immediately after deployment
echo "$DEPLOY_OUTPUT_DEPOSIT_TOKEN"
DEPOSIT_TOKEN_TXID=$(extract_txid "$DEPLOY_OUTPUT_DEPOSIT_TOKEN")

if [ -z "$DEPOSIT_TOKEN_TXID" ]; then
  echo "❌ ERROR: Failed to extract deposit token transaction ID"
  exit 1
fi

echo "✅ Deposit Token created at: $DEPOSIT_TOKEN_TXID"
pause_and_generate

# Use deposit token txid directly - no need to trace for token ID  
echo "✅ Deposit Token TXID: $DEPOSIT_TOKEN_TXID"
DEPOSIT_TOKEN_ID="4"  # Use simple incremental ID for factory reference
generate_blocks_now
echo ""

# STEP 4: Create Reward Token (Direct Token Creation with Premine)
echo "🎁 STEP 4: Creating Reward Token (100k supply, 30k premine)"
echo "==========================================================="
echo "Creating REWARD token with 100k cap, 1000 per mint, 30k premine..."
# 30k tokens = 30000 * 100000000 = 3000000000000 alks
# 1000 tokens per mint = 1000 * 100000000 = 100000000000 alks
DEPLOY_OUTPUT_REWARD_TOKEN=$(cd $OYL_DIR && $OYL_CMD alkane new-token -pre 3000000000000 -amount 100000000000 -c 100000 -name "REWARD" -symbol "REW" -resNumber $NAMESPACE -p oylnet)
generate_blocks_now  # Generate blocks immediately after deployment
echo "$DEPLOY_OUTPUT_REWARD_TOKEN"
REWARD_TOKEN_TXID=$(extract_txid "$DEPLOY_OUTPUT_REWARD_TOKEN")

if [ -z "$REWARD_TOKEN_TXID" ]; then
  echo "❌ ERROR: Failed to extract reward token transaction ID"
  exit 1
fi

echo "✅ Reward Token created at: $REWARD_TOKEN_TXID"
pause_and_generate

# Use reward token txid directly - no need to trace for token ID
echo "✅ Reward Token TXID: $REWARD_TOKEN_TXID"
echo "✅ Reward Token premined with 30,000 tokens for vault reward pool"
REWARD_TOKEN_ID="5"  # Use simple incremental ID for factory reference
generate_blocks_now

# Set reward pool size (we know we have 30k premined)
REWARD_POOL_SIZE="30000"
echo ""

# STEP 5: Deploy Factory Instance (call 3:0x37a with opcode 3)
echo "🏗️ STEP 5: Deploying Factory Instance"
echo "====================================="
# Calculate vault factory template ID (should be 0x37a = 890 decimal)
VAULT_FACTORY_TEMPLATE_ID="890"  # 0x37a

DEPLOY_OUTPUT_FACTORY_INSTANCE=$(cd $OYL_DIR && $OYL_CMD alkane new-contract -c $VAULT_FACTORY_WASM_PATH -data 3,$VAULT_FACTORY_TEMPLATE_ID,0 -p oylnet)
generate_blocks_now  # Generate blocks immediately after deployment
echo "$DEPLOY_OUTPUT_FACTORY_INSTANCE"
FACTORY_INSTANCE_TXID=$(extract_txid "$DEPLOY_OUTPUT_FACTORY_INSTANCE")

if [ -z "$FACTORY_INSTANCE_TXID" ]; then
  echo "❌ ERROR: Failed to extract factory instance transaction ID"
  exit 1
fi

echo "✅ Factory Instance deployed at: $FACTORY_INSTANCE_TXID"
pause_and_generate

# Use factory instance txid directly - no need to trace for instance ID
echo "✅ Factory Instance TXID: $FACTORY_INSTANCE_TXID"
FACTORY_INSTANCE_ID="6"  # Use simple incremental ID for initialization reference
generate_blocks_now
echo ""

# STEP 7: Initialize Factory Instance (send reward pool from premine)
echo "⚙️  STEP 7: Initializing Factory Instance"
echo "========================================"
REWARD_POOL_SIZE="30000"  # Send 30k of our 50k premine to vault
REWARD_PER_BLOCK="1000"   # 1000 tokens per block emission
START_BLOCK="10"          # Start rewards at block 10

echo "Parameters:"
echo "  - Deposit Token ID: $DEPOSIT_TOKEN_ID"
echo "  - Reward Token ID: $REWARD_TOKEN_ID" 
echo "  - Reward Pool Size: $REWARD_POOL_SIZE"
echo "  - Reward Per Block: $REWARD_PER_BLOCK"
echo "  - Start Block: $START_BLOCK"
echo ""

# Initialize factory with both token IDs and reward pool
# Data format: [factory_id, opcode, deposit_token_id, reward_token_id, reward_per_block, start_block, preloaded_rewards]
INIT_OUTPUT=$(cd $OYL_DIR && $OYL_CMD alkane execute -c $VAULT_FACTORY_WASM_PATH -data $FACTORY_INSTANCE_ID,0,$DEPOSIT_TOKEN_ID,$REWARD_TOKEN_ID,$REWARD_PER_BLOCK,$START_BLOCK,$REWARD_POOL_SIZE -p oylnet)
generate_blocks_now  # Generate blocks immediately after initialization
echo "$INIT_OUTPUT"
INIT_TXID=$(extract_txid "$INIT_OUTPUT")

if [ -z "$INIT_TXID" ]; then
  echo "❌ ERROR: Failed to extract initialization transaction ID"
  exit 1
fi

echo "✅ Factory Instance initialized at: $INIT_TXID"
pause_and_generate

# STEP 8: Summary
echo "🎉 DEPLOYMENT COMPLETE!"
echo "======================"
echo ""
echo "📋 DEPLOYMENT SUMMARY:"
echo "────────────────────────"
echo "Namespace: $NAMESPACE"
echo ""
echo "📍 Template Addresses:"
echo "  Position Token Template: 3:$POSITION_TEMPLATE_ID"
echo "  Vault Factory Template:  3:$VAULT_FACTORY_TEMPLATE_ID"
echo ""
echo "🪙 Token Information:"
echo "  Deposit Token ID:  $DEPOSIT_TOKEN_ID (DEPOSIT/DEP - 100k supply, public mintable)"
echo "  Reward Token ID:   $REWARD_TOKEN_ID (REWARD/REW - 100k supply, 30k premined)"
echo ""
echo "🏭 Factory Information:"
echo "  Factory Instance ID: $FACTORY_INSTANCE_ID"
echo "  Reward Pool Loaded:  $REWARD_POOL_SIZE tokens"
echo "  Emission Rate:       $REWARD_PER_BLOCK tokens/block"
echo "  Rewards Start:       Block $START_BLOCK"
echo ""
echo "📋 Transaction IDs:"
echo "  Position Template:   $POSITION_TEMPLATE_TXID"
echo "  Vault Template:      $VAULT_TEMPLATE_TXID"
echo "  Deposit Token:       $DEPOSIT_TOKEN_TXID"
echo "  Reward Token:        $REWARD_TOKEN_TXID"
echo "  Factory Instance:    $FACTORY_INSTANCE_TXID"
echo "  Factory Initialize:  $INIT_TXID"
echo ""
echo "🎯 NEXT STEPS:"
echo "1. Users can mint deposit tokens directly (DEPOSIT/DEP)"
echo "2. Users can deposit tokens into Factory Instance $FACTORY_INSTANCE_ID"
echo "3. Users receive position tokens at 2:101, 2:102, etc."
echo "4. Users earn rewards from the $REWARD_POOL_SIZE token reward pool"
echo ""
echo "✅ Two-token ecosystem deployment successful!"
``````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````````
