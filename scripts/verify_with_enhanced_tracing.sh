#!/bin/bash

# Enhanced Contract Verification Script with Multi-Vout Tracing
# Tests all deployed contracts by iterating through vouts 3-5 to find actual content

# Usage: ./verify_with_enhanced_tracing.sh [txid1] [txid2] [txid3] [txid4] [txid5]
# If no TXIDs provided, prompts for them

# Detect environment and set paths - updated for boiler repo location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOILER_ROOT="$(dirname "$SCRIPT_DIR")"

# Check for oyl-sdk directory (relative to boiler repo)
if [ -d "$BOILER_ROOT/../oyl-sdk" ]; then
  OYL_DIR="$BOILER_ROOT/../oyl-sdk"
elif [ -d "/home/e/Documents/oyl-sdk" ]; then
  OYL_DIR="/home/e/Documents/oyl-sdk"
elif [ -d "/home/e/oyl-sdk" ]; then
  OYL_DIR="/home/e/oyl-sdk"
else
  echo "❌ Error: Could not find oyl-sdk directory"
  echo "Please ensure oyl-sdk is available or update OYL_DIR path"
  exit 1
fi

OYL_CMD="node bin/oyl.js"

echo "🔍 ENHANCED CONTRACT VERIFICATION WITH MULTI-VOUT TRACING"
echo "========================================================="
echo "Boiler Root: $BOILER_ROOT"
echo "OYL SDK: $OYL_DIR"
echo ""

# Enhanced function to trace multiple vouts and find content
verify_contract_content() {
  local txid=$1
  local contract_name=$2
  
  if [ -z "$txid" ]; then
    echo "⚠️  No txid provided for $contract_name"
    return 1
  fi
  
  echo "🔍 Enhanced verification for $contract_name"
  echo "   TX ID: $txid"
  echo "   Testing vouts 3, 4, 5 to find actual content..."
  
  local found_content=false
  local successful_vout=""
  local best_trace_output=""
  
  for vout in 3 4 5; do
    echo "   🔸 Testing vout $vout..."
    
    # Use the fixed OYL SDK RPC client
    local trace_output
    trace_output=$(cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "trace" -params "[{\"txid\": \"$txid\", \"vout\": $vout}]" -p oylnet 2>/dev/null)
    local trace_status=$?
    
    if [ $trace_status -eq 0 ] && [ ! -z "$trace_output" ]; then
      # Check if response contains actual content - look for storage entries or meaningful data
      local has_storage_entries=$(echo "$trace_output" | grep -c '"key":' 2>/dev/null || echo "0")
      local has_meaningful_data=$(echo "$trace_output" | grep -q '"data":"[0-9a-f]\{100,\}' && echo "true" || echo "false")
      
      # Clean up any newlines in the count
      has_storage_entries=$(echo "$has_storage_entries" | tr -d '\n' | head -1)
      
      if [ "$has_storage_entries" -gt 0 ] 2>/dev/null || [ "$has_meaningful_data" = "true" ]; then
        echo "   ✅ Found content in vout $vout!"
        found_content=true
        successful_vout=$vout
        best_trace_output="$trace_output"
        
        # Extract key information
        local has_storage=$(echo "$trace_output" | grep -q '"storage"' && echo "Yes" || echo "No")
        local storage_count=$(echo "$trace_output" | grep -c '"key":' || echo "0")
        local data_length=$(echo "$trace_output" | grep -o '"data":"[^"]*"' | head -1 | cut -d'"' -f4 | wc -c)
        
        echo "   📊 Content summary:"
        echo "      • Vout: $vout"
        echo "      • Has storage entries: $has_storage"
        echo "      • Storage entries count: $storage_count"
        echo "      • Data length: $data_length characters"
        
        # Show storage keys if present
        if [ "$has_storage" = "Yes" ] && [ "$storage_count" -gt 0 ]; then
          echo "      • Key storage entries:"
          echo "$trace_output" | grep -o '"key":"[^"]*"' | cut -d'"' -f4 | head -5 | sed 's/^/        - /'
          if [ "$storage_count" -gt 5 ]; then
            echo "        - ... and $(( storage_count - 5 )) more"
          fi
          
          # Check for specific contract indicators
          if echo "$trace_output" | grep -q "/totalsupply"; then
            local total_supply=$(echo "$trace_output" | grep -A1 '"/totalsupply"' | grep '"value"' | cut -d'"' -f4)
            echo "      • 💰 Total Supply: $total_supply"
          fi
          
          if echo "$trace_output" | grep -q "/cap"; then
            local cap=$(echo "$trace_output" | grep -A1 '"/cap"' | grep '"value"' | cut -d'"' -f4)
            echo "      • 🎯 Cap: $cap"
          fi
          
          if echo "$trace_output" | grep -q "/initialized"; then
            local initialized=$(echo "$trace_output" | grep -A1 '"/initialized"' | grep '"value"' | cut -d'"' -f4)
            echo "      • ✅ Initialized: $initialized"
          fi
          
          if echo "$trace_output" | grep -q "/minted"; then
            local minted=$(echo "$trace_output" | grep -A1 '"/minted"' | grep '"value"' | cut -d'"' -f4)
            echo "      • 🪙 Minted: $minted"
          fi
        fi
        
        break  # Found content, no need to test other vouts
      else
        echo "   ❌ Vout $vout: Empty or minimal content"
      fi
    else
      echo "   ❌ Vout $vout: Trace failed or no response"
    fi
  done
  
  if [ "$found_content" = true ]; then
    echo "   🎉 Verification SUCCESS for $contract_name!"
    echo "   📋 Use vout $successful_vout for future traces of this contract"
    return 0
  else
    echo "   ⚠️  No vout (3-5) contained significant content for $contract_name"
    echo "   🔍 You may need to check if the contract was deployed correctly"
    return 1
  fi
}

# Function to get current blockchain height
get_current_height() {
  echo "🔍 Getting current blockchain height..."
  local height_output
  height_output=$(cd "$OYL_DIR" && $OYL_CMD provider alkanes -method "metashrewHeight" -params '[]' -p oylnet 2>/dev/null)
  local height=$(echo "$height_output" | grep -o '[0-9]*' | tail -1)
  if [ ! -z "$height" ]; then
    echo "📊 Current blockchain height: $height"
  else
    echo "⚠️  Could not determine blockchain height"
  fi
}

# Function to verify a list of contracts
verify_contract_list() {
  local contracts=("$@")
  local contract_names=("Free-mint Template" "Position-token Template" "Vault-factory Template" "Free-mint Instance" "Position-token Instance")
  
  echo "🔍 VERIFYING ${#contracts[@]} CONTRACTS"
  echo "======================================"
  
  local success_count=0
  local total_count=${#contracts[@]}
  
  for i in "${!contracts[@]}"; do
    local txid="${contracts[$i]}"
    local name="${contract_names[$i]}"
    
    if [ -z "$txid" ] || [ "$txid" = "skip" ]; then
      echo "⏭️  Skipping ${name} (no TXID provided)"
      continue
    fi
    
    echo ""
    verify_contract_content "$txid" "$name"
    if [ $? -eq 0 ]; then
      ((success_count++))
    fi
    echo ""
  done
  
  echo "📊 VERIFICATION SUMMARY:"
  echo "======================="
  echo "✅ Successful verifications: $success_count"
  echo "❌ Failed verifications: $(( total_count - success_count ))"
  echo "📈 Success rate: $(( success_count * 100 / total_count ))%"
}

# Get current blockchain status first
get_current_height

echo ""

# Check if TXIDs were provided as arguments
if [ $# -eq 0 ]; then
  echo "📝 INTERACTIVE MODE: Please provide contract TXIDs"
  echo "================================================="
  echo ""
  echo "Enter the TXIDs for your deployed contracts (press Enter to skip):"
  echo ""
  
  read -p "Free-mint Template TXID: " FREE_MINT_TEMPLATE_TXID
  read -p "Position-token Template TXID: " POSITION_TEMPLATE_TXID
  read -p "Vault-factory Template TXID: " VAULT_TEMPLATE_TXID
  read -p "Free-mint Instance TXID: " FREE_MINT_INSTANCE_TXID
  read -p "Position-token Instance TXID: " POSITION_INSTANCE_TXID
  
  contracts=("$FREE_MINT_TEMPLATE_TXID" "$POSITION_TEMPLATE_TXID" "$VAULT_TEMPLATE_TXID" "$FREE_MINT_INSTANCE_TXID" "$POSITION_INSTANCE_TXID")
  
elif [ $# -eq 1 ] && [ "$1" = "test" ]; then
  # Test mode with known working TXIDs
  echo "🧪 TEST MODE: Using example TXIDs"
  echo "================================="
  contracts=("56b0beb401569808c6a74a9b9be3c6a17fe2b3a34fff16f5c5ebf46730ed3b50" "15d09a9bf46239492ea4812d3f5733ad59bee7023568a5bda1244c992b367358" "22ac960647f94594ded57920f12a21e922b475c46aa668f097c154b848ee0de4" "114c3f2a024d421278e6132982397de258c1e770bd32ce0f75108a5c514eaef6" "skip")
  
else
  # Command line arguments provided
  echo "📋 COMMAND LINE MODE: Using provided TXIDs"
  echo "=========================================="
  contracts=("$1" "$2" "$3" "$4" "$5")
fi

# Verify the contracts
verify_contract_list "${contracts[@]}"

echo ""
echo "💡 USAGE TIPS:"
echo "=============="
echo ""
echo "1. To verify specific contracts:"
echo "   ./verify_with_enhanced_tracing.sh [txid1] [txid2] [txid3] [txid4] [txid5]"
echo ""
echo "2. To run in test mode with example TXIDs:"
echo "   ./verify_with_enhanced_tracing.sh test"
echo ""
echo "3. To run in interactive mode:"
echo "   ./verify_with_enhanced_tracing.sh"
echo ""
echo "4. Example manual trace command:"
echo "   cd $OYL_DIR && $OYL_CMD provider alkanes -method \"trace\" -params '[{\"txid\": \"YOUR_TXID\", \"vout\": 3}]' -p oylnet"
echo ""
echo "🔍 VERIFICATION COMPLETED!"
echo "========================="
echo ""
echo "If any verifications failed, check:"
echo "• The TXID is correct and complete"
echo "• The contract was deployed successfully"
echo "• The blockchain is synchronized"
echo "• Try different vout values (0-5) manually if needed"
