#!/bin/bash
set +e

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Contract ID from successful deployment
CONTRACT_ID="7dbd26587f4d058577afa7e180fe1bfbe4941e6b5459e783d14e336c18238f0f"

echo -e "${BLUE}=== Fix YieldVault Contract Initialization ===${NC}"

# 1. Generate blocks to confirm deployment is on chain
echo -e "${YELLOW}1. Generating blocks to confirm deployment${NC}"
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 20 || echo "Block generation failed but proceeding"

# 2. Prepare initialization parameters
echo -e "\n${YELLOW}2. Creating initialization parameters${NC}"
NAME="YieldVault"
SYMBOL="YVT"
ASSET_NAME="Bitcoin"
ASSET_SYMBOL="BTC"
DECIMALS=8

echo -e "Parameters:"
echo "  Name: $NAME"
echo "  Symbol: $SYMBOL"
echo "  Asset Name: $ASSET_NAME"
echo "  Asset Symbol: $ASSET_SYMBOL"
echo "  Decimals: $DECIMALS"

# 3. Execute initialization with properly formatted calldata
echo -e "\n${YELLOW}3. Executing initialization${NC}"
INIT_OPCODE=0

# First, try the raw parameters approach (separating by commas)
echo -e "${CYAN}$ oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "$INIT_OPCODE,$NAME,$SYMBOL,$ASSET_NAME,$ASSET_SYMBOL,$DECIMALS"${NC}"

NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "$INIT_OPCODE,$NAME,$SYMBOL,$ASSET_NAME,$ASSET_SYMBOL,$DECIMALS" || {
  echo -e "${YELLOW}First initialization attempt failed, trying alternative format...${NC}"
  
  # Try alternative format with hex encoding
  NAME_HEX=$(echo -n "$NAME" | xxd -p | tr -d '\n')
  SYMBOL_HEX=$(echo -n "$SYMBOL" | xxd -p | tr -d '\n')
  ASSET_NAME_HEX=$(echo -n "$ASSET_NAME" | xxd -p | tr -d '\n')
  ASSET_SYMBOL_HEX=$(echo -n "$ASSET_SYMBOL" | xxd -p | tr -d '\n')
  
  echo -e "${CYAN}$ oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "$INIT_OPCODE,0x$NAME_HEX,0x$SYMBOL_HEX,0x$ASSET_NAME_HEX,0x$ASSET_SYMBOL_HEX,$DECIMALS"${NC}"
  
  NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "$INIT_OPCODE,0x$NAME_HEX,0x$SYMBOL_HEX,0x$ASSET_NAME_HEX,0x$ASSET_SYMBOL_HEX,$DECIMALS" || {
    echo -e "${RED}Both initialization attempts failed${NC}"
    exit 1
  }
}

# 4. Generate blocks to confirm initialization
echo -e "\n${YELLOW}4. Generating blocks to confirm initialization${NC}"
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10 || echo "Block generation failed but proceeding"

# 5. Verify initialization by checking contract state
echo -e "\n${YELLOW}5. Verifying contract initialization${NC}"

# Get vault name - opcode 100
echo -e "${CYAN}Checking vault name (opcode 100)${NC}"
NAME_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "100" 2>/dev/null)
echo -e "Returned Name: $NAME_RESULT"

# Get vault symbol - opcode 101
echo -e "${CYAN}Checking vault symbol (opcode 101)${NC}"
SYMBOL_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "101" 2>/dev/null)
echo -e "Returned Symbol: $SYMBOL_RESULT"

# Get asset name - opcode 103
echo -e "${CYAN}Checking asset symbol (opcode 103)${NC}"
ASSET_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "103" 2>/dev/null)
echo -e "Returned Asset: $ASSET_RESULT"

# Get decimals - opcode 102
echo -e "${CYAN}Checking decimals (opcode 102)${NC}"
DECIMALS_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "102" 2>/dev/null)
echo -e "Returned Decimals: $DECIMALS_RESULT"

# Get total assets - opcode 200
echo -e "${CYAN}Checking total assets (opcode 200)${NC}"
ASSETS_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "200" 2>/dev/null)
echo -e "Returned Total Assets: $ASSETS_RESULT"

# Get total supply - opcode 601
echo -e "${CYAN}Checking total supply (opcode 601)${NC}"
SUPPLY_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute -p oylnet --contractId "$CONTRACT_ID" --calldata "601" 2>/dev/null)
echo -e "Returned Total Supply: $SUPPLY_RESULT"

# Check if initialization worked
if [[ "$NAME_RESULT" == "$NAME" || "$SYMBOL_RESULT" == "$SYMBOL" ]]; then
  echo -e "\n${GREEN}✅ Contract initialization successful!${NC}"
else
  echo -e "\n${YELLOW}⚠️ Contract initialization may not have been successful${NC}"
  echo -e "Expected Name: $NAME, Got: $NAME_RESULT"
  echo -e "Expected Symbol: $SYMBOL, Got: $SYMBOL_RESULT"
fi

echo -e "\n${BLUE}=== Contract Initialization Process Complete ===${NC}"
