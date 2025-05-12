#!/bin/bash

# Source environment variables
source .env

# Set colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Create wallet address from mnemonic
echo -e "${BLUE}Generating wallet address from mnemonic...${NC}"
echo -e "${YELLOW}Mnemonic: $MNEMONIC${NC}"
echo -e "${YELLOW}Derivation path: $DERIVATION_PATH${NC}"

# Capture the result of the mnemonicToAccount command to a file
echo -e "${BLUE}Executing wallet generation command...${NC}"
oyl account mnemonicToAccount --mnemonic="$MNEMONIC" --provider=oylnet > wallet_info.json
cat wallet_info.json

# Extract address - simplified approach since oyl CLI output may vary
ADDRESS=$(oyl account mnemonicToAccount --mnemonic="$MNEMONIC" --provider=oylnet | grep -o '"address": *"[^"]*"' | grep -o '"[^"]*"$' | tr -d '"' || echo "bcrt1q7g0m7sav63h2ms3mjrrxvhugdufexs3t94jjdj")

# If we couldn't extract address, use default
if [ -z "$ADDRESS" ]; then
    ADDRESS="bcrt1q7g0m7sav63h2ms3mjrrxvhugdufexs3t94jjdj"
    echo -e "${YELLOW}Using default address: $ADDRESS${NC}"
else
    echo -e "${GREEN}Generated address: $ADDRESS${NC}"
fi

# Fund the address using regtest faucet
echo -e "${BLUE}Funding address from faucet...${NC}"
AMOUNT=100000000 # 1 BTC
oyl regtest sendFromFaucet -p oylnet -t $ADDRESS -s $AMOUNT

# Generate blocks to confirm transaction
echo -e "${BLUE}Generating blocks to confirm funding...${NC}"
oyl regtest genBlocks -p oylnet -c 10

# Save the funded address for future use
echo "export FUNDED_ADDRESS=$ADDRESS" >> .env

echo -e "${GREEN}Wallet setup complete!${NC}"
echo -e "${BOLD}Funded address: $ADDRESS${NC}"
echo -e "${BOLD}Amount: $AMOUNT satoshis ($(echo "scale=8; $AMOUNT/100000000" | bc) BTC)${NC}"
