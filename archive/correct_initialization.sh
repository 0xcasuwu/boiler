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

echo -e "${BLUE}=== YieldVault Contract Initialization ===${NC}"

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

# Create hex-encoded parameters
NAME_HEX=$(echo -n "$NAME" | xxd -p | tr -d '\n')
SYMBOL_HEX=$(echo -n "$SYMBOL" | xxd -p | tr -d '\n')
ASSET_NAME_HEX=$(echo -n "$ASSET_NAME" | xxd -p | tr -d '\n')
ASSET_SYMBOL_HEX=$(echo -n "$ASSET_SYMBOL" | xxd -p | tr -d '\n')

# Call format is likely contract:opcode,arg1,arg2...
CALLDATA="${INIT_OPCODE},0x${NAME_HEX},0x${SYMBOL_HEX},0x${ASSET_NAME_HEX},0x${ASSET_SYMBOL_HEX},${DECIMALS}"
echo -e "Using calldata: ${CALLDATA}"

# Try initializing using the PATH_TO_CONTRACT:OPCODE format
echo -e "${CYAN}$ oyl alkane execute --contract '${CONTRACT_ID}' --provider oylnet --calldata '${CALLDATA}'${NC}"

NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute --contract "${CONTRACT_ID}" --provider oylnet --calldata "${CALLDATA}" || {
  echo -e "${YELLOW}First format failed, trying with direct CONTRACT_ID in calldata...${NC}"
  
  # Try with CONTRACT_ID:OPCODE format
  CONTRACT_CALLDATA="${CONTRACT_ID}:${CALLDATA}"
  echo -e "${CYAN}$ oyl alkane execute --provider oylnet --calldata '${CONTRACT_CALLDATA}'${NC}"
  
  NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute --provider oylnet --calldata "${CONTRACT_CALLDATA}" || {
    echo -e "${YELLOW}Second format failed, trying one more approach...${NC}"
    
    # Try with path format for both contract and calldata
    echo -e "${CYAN}$ oyl alkane execute --contract '${CONTRACT_ID}' -data '${CALLDATA}' --provider oylnet${NC}"
    
    NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute --contract "${CONTRACT_ID}" -data "${CALLDATA}" --provider oylnet || {
      echo -e "${RED}All initialization approaches failed${NC}"
      
      # One final attempt with the correct parameter naming from --help
      echo -e "${YELLOW}Final attempt with simplified calldata...${NC}"
      echo -e "${CYAN}$ oyl alkane execute --calldata '${CONTRACT_ID}:${INIT_OPCODE}' --provider oylnet${NC}"
      
      NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute --calldata "${CONTRACT_ID}:${INIT_OPCODE}" --provider oylnet || {
        echo -e "${RED}Failed to initialize contract${NC}"
        exit 1
      }
    }
  }
}

# 4. Generate blocks to confirm initialization
echo -e "\n${YELLOW}4. Generating blocks to confirm initialization${NC}"
NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl regtest genBlocks -p oylnet -c 10 || echo "Block generation failed but proceeding"

# 5. Verify initialization by checking contract state
echo -e "\n${YELLOW}5. Verifying contract initialization${NC}"

# Get vault name - opcode 100
echo -e "${CYAN}Checking vault name (opcode 100)${NC}"
echo -e "${CYAN}$ oyl alkane execute --calldata '${CONTRACT_ID}:100' --provider oylnet${NC}"
NAME_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute --calldata "${CONTRACT_ID}:100" --provider oylnet 2>/dev/null)
echo -e "Returned Name: $NAME_RESULT"

# Get vault symbol - opcode 101
echo -e "${CYAN}Checking vault symbol (opcode 101)${NC}"
echo -e "${CYAN}$ oyl alkane execute --calldata '${CONTRACT_ID}:101' --provider oylnet${NC}"
SYMBOL_RESULT=$(NODE_OPTIONS=--require=./oyl-sdk/lib/shared/load_patch.js oyl alkane execute --calldata "${CONTRACT_ID}:101" --provider oylnet 2>/dev/null)
echo -e "Returned Symbol: $SYMBOL_RESULT"

# Check if initialization worked
if [[ "$NAME_RESULT" == "$NAME" || "$SYMBOL_RESULT" == "$SYMBOL" ]]; then
  echo -e "\n${GREEN}✅ Contract initialization successful!${NC}"
else
  echo -e "\n${YELLOW}⚠️ Contract initialization may not have been successful${NC}"
  echo -e "Expected Name: $NAME, Got: $NAME_RESULT"
  echo -e "Expected Symbol: $SYMBOL, Got: $SYMBOL_RESULT"
fi

echo -e "\n${BLUE}=== Contract Initialization Process Complete ===${NC}"
