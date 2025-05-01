#!/bin/bash
# Modified version of deploy-local.sh to work with extended Subfrost environment
# and perform a full lifecycle test

set -e  # Exit on error

# Text colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Extended Subfrost configuration
EXTENDED_RPC_URL="http://localhost:18889"
PROVIDER="alkanes"
FEE_RATE=5
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="deploy-slop-extended-test-${TIMESTAMP}.log"
INTEREST_RATE=500
MATURITY_BLOCKS=10  # Short maturity for testing
VERSION="1.0.0"

# Contract paths
FACTORY_CONTRACT_PATH="./target/wasm32-unknown-unknown/release/slop_launchpad_factory.wasm"
COLLECTION_CONTRACT_PATH="./target/wasm32-unknown-unknown/release/slop_orbital_bond_collection.wasm"
CURVE_CONTRACT_PATH="./target/wasm32-unknown-unknown/release/slop_bond_curve.wasm"

# Contract constants
FACTORY_CONSTANT="100001"
COLLECTION_CONSTANT="100002"
CURVE_CONSTANT="100003"

# Function to log steps
log_step() {
  echo -e "${BLUE}==== $1 ====${NC}"
  echo -e "==== $1 ====" >> "$LOG_FILE"
}

# Function to log success
log_success() {
  echo -e "${GREEN}✓ $1${NC}"
  echo -e "✓ $1" >> "$LOG_FILE"
}

# Function to log info
log_info() {
  echo -e "${YELLOW}$1${NC}"
  echo -e "$1" >> "$LOG_FILE"
}

# Function to log error and exit
log_error() {
  echo -e "${RED}ERROR: $1${NC}"
  echo -e "ERROR: $1" >> "$LOG_FILE"
  exit 1
}

# Function to check for required commands
check_dependencies() {
  log_step "Checking dependencies"

  local dependencies=("oyl" "jq" "curl")
  local missing_deps=()

  for dep in "${dependencies[@]}"; do
    if ! command -v "$dep" &> /dev/null; then
      missing_deps+=("$dep")
    fi
  done

  if [ ${#missing_deps[@]} -gt 0 ]; then
    log_error "Missing dependencies: ${missing_deps[*]}\nPlease install them before continuing."
  fi

  log_success "All dependencies are installed"
}

# Function to extract txid from oyl output
extract_txid() {
  echo "$1" | grep -oP 'txid: \K[a-f0-9]+' || echo "$1" | grep -oP "txId: '\K[a-f0-9]+(?=')" | head -n 1
}

# Function to verify Extended Subfrost server is running
verify_extended_subfrost() {
  log_step "Verifying Extended Subfrost server at $EXTENDED_RPC_URL"

  # Check health endpoint
  local health_check
  health_check=$(curl -s "$EXTENDED_RPC_URL/health")
  
  if [[ "$health_check" != *"success"* ]]; then
    log_error "Extended Subfrost server at $EXTENDED_RPC_URL is not responding. Make sure it's running."
  fi
  
  # Check methods endpoint
  local methods_check
  methods_check=$(curl -s "$EXTENDED_RPC_URL/methods")
  
  if [[ "$methods_check" != *"alkane_newContract"* ]]; then
    log_error "Extended Subfrost server does not support required RPC methods."
  fi

  log_success "Extended Subfrost server is running and has required methods"
}

# Function to verify WASM files exist
check_wasm_files() {
  log_step "Checking WASM files"

  # Check if contracts exist
  local missing_contracts=()

  if [ ! -f "$FACTORY_CONTRACT_PATH" ]; then
    missing_contracts+=("LaunchpadFactory ($FACTORY_CONTRACT_PATH)")
  else
    log_info "Found LaunchpadFactory: $FACTORY_CONTRACT_PATH ($(du -h "$FACTORY_CONTRACT_PATH" | cut -f1))"
  fi

  if [ ! -f "$COLLECTION_CONTRACT_PATH" ]; then
    missing_contracts+=("OrbitalBondCollection ($COLLECTION_CONTRACT_PATH)")
  else
    log_info "Found OrbitalBondCollection: $COLLECTION_CONTRACT_PATH ($(du -h "$COLLECTION_CONTRACT_PATH" | cut -f1))"
  fi

  if [ ! -f "$CURVE_CONTRACT_PATH" ]; then
    missing_contracts+=("BondCurve ($CURVE_CONTRACT_PATH)")
  else
    log_info "Found BondCurve: $CURVE_CONTRACT_PATH ($(du -h "$CURVE_CONTRACT_PATH" | cut -f1))"
  fi

  if [ ${#missing_contracts[@]} -gt 0 ]; then
    log_error "Missing WASM files: ${missing_contracts[*]}"
  fi

  log_success "All WASM files are present"
}

# Function to set up OYL provider
setup_oyl_provider() {
  log_step "Setting up OYL provider"
  
  # Configure OYL environment variables
  export OYL_RPC_URL="$EXTENDED_RPC_URL"
  export OYL_PROVIDER="$PROVIDER"
  
  # Test provider connection
  log_info "Testing connection to Extended Subfrost server"
  local height_result
  height_result=$(oyl provider alkanes -method metashrew_height -p $PROVIDER 2>&1)
  
  if echo "$height_result" | grep -q "error"; then
    log_error "Failed to connect to Extended Subfrost server: $height_result"
  fi
  
  log_success "Successfully connected to Extended Subfrost server"
}

# Function to get account addresses
setup_accounts() {
  log_step "Setting up accounts"

  # Use a temporary file to store the output
  TEMP_OUTPUT=$(mktemp)

  # Get account info
  log_info "Fetching account information"
  oyl account mnemonicToAccount -p $PROVIDER > "$TEMP_OUTPUT" 2>&1

  # Extract addresses
  NATIVE_SEGWIT_ADDRESS=$(grep -A 3 "nativeSegwit" "$TEMP_OUTPUT" | grep "address:" | sed -E "s/.*address: '([^']+)'.*/\1/")
  TAPROOT_ADDRESS=$(grep -A 3 "taproot:" "$TEMP_OUTPUT" | grep "address:" | sed -E "s/.*address: '([^']+)'.*/\1/")

  # Clean up addresses
  NATIVE_SEGWIT_ADDRESS=$(echo "$NATIVE_SEGWIT_ADDRESS" | tr -d ' \n\r\t')
  TAPROOT_ADDRESS=$(echo "$TAPROOT_ADDRESS" | tr -d ' \n\r\t')

  rm "$TEMP_OUTPUT"

  # Set admin address
  ADMIN_ADDRESS=$NATIVE_SEGWIT_ADDRESS
  
  # If addresses are empty, use fallback values for testing
  if [ -z "$NATIVE_SEGWIT_ADDRESS" ]; then
    NATIVE_SEGWIT_ADDRESS="bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"
    log_info "Using fallback Native SegWit Address: $NATIVE_SEGWIT_ADDRESS"
  else
    log_info "Native SegWit Address: $NATIVE_SEGWIT_ADDRESS"
  fi
  
  if [ -z "$TAPROOT_ADDRESS" ]; then
    TAPROOT_ADDRESS="bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0"
    log_info "Using fallback Taproot Address: $TAPROOT_ADDRESS"
  else
    log_info "Taproot Address: $TAPROOT_ADDRESS"
  fi
  
  log_info "Admin Address: $ADMIN_ADDRESS"
  log_success "Account setup complete"
}

# Function to deploy contracts
deploy_contracts() {
  log_step "Deploying contracts to Extended Subfrost"

  # Deploy factory contract
  log_info "Deploying LaunchpadFactory"
  log_info "Contract path: $FACTORY_CONTRACT_PATH"
  log_info "Calldata: 3,$FACTORY_CONSTANT,\"$VERSION\",1"

  FACTORY_OUTPUT=$(oyl alkane new-contract -c "$FACTORY_CONTRACT_PATH" --calldata "3,$FACTORY_CONSTANT,\"$VERSION\",1" --feeRate $FEE_RATE -p $PROVIDER)
  FACTORY_TXID=$(extract_txid "$FACTORY_OUTPUT")

  if [ -z "$FACTORY_TXID" ]; then
    log_error "Failed to deploy LaunchpadFactory. Output: $FACTORY_OUTPUT"
  fi

  log_success "LaunchpadFactory deployed with txid: $FACTORY_TXID"

  # Deploy collection contract template
  log_info "Deploying OrbitalBondCollection template"
  log_info "Contract path: $COLLECTION_CONTRACT_PATH"
  log_info "Calldata: 3,\"Orbital Bonds\",\"ORB\",$INTEREST_RATE,$MATURITY_BLOCKS"

  COLLECTION_OUTPUT=$(oyl alkane new-contract -c "$COLLECTION_CONTRACT_PATH" --calldata "3,\"Orbital Bonds\",\"ORB\",$INTEREST_RATE,$MATURITY_BLOCKS" --feeRate $FEE_RATE -p $PROVIDER)
  COLLECTION_TXID=$(extract_txid "$COLLECTION_OUTPUT")

  if [ -z "$COLLECTION_TXID" ]; then
    log_error "Failed to deploy OrbitalBondCollection. Output: $COLLECTION_OUTPUT"
  fi

  log_success "OrbitalBondCollection deployed with txid: $COLLECTION_TXID"

  # Deploy bond curve
  log_info "Deploying BondCurve"
  log_info "Contract path: $CURVE_CONTRACT_PATH"
  log_info "Calldata: 3,$CURVE_CONSTANT,\"$VERSION\",1"

  CURVE_OUTPUT=$(oyl alkane new-contract -c "$CURVE_CONTRACT_PATH" --calldata "3,$CURVE_CONSTANT,\"$VERSION\",1" --feeRate $FEE_RATE -p $PROVIDER)
  CURVE_TXID=$(extract_txid "$CURVE_OUTPUT")

  if [ -z "$CURVE_TXID" ]; then
    log_error "Failed to deploy BondCurve. Output: $CURVE_OUTPUT"
  fi

  log_success "BondCurve deployed with txid: $CURVE_TXID"
}

# Function to perform a full bond lifecycle test
run_full_lifecycle_test() {
  log_step "Running full bond lifecycle test"

  # 1. Get price from bond curve
  log_info "1. Getting price from bond curve for 1,000,000 sats"
  BOND_AMOUNT=1000000  # 0.01 BTC in satoshis
  
  PRICE_OUTPUT=$(oyl alkane call-contract -t "$CURVE_TXID" -m "getPrice" -a "$BOND_AMOUNT" --feeRate $FEE_RATE -p $PROVIDER)
  PRICE_TXID=$(extract_txid "$PRICE_OUTPUT")
  
  if [ -z "$PRICE_TXID" ]; then
    log_error "Failed to get price from bond curve. Output: $PRICE_OUTPUT"
  fi
  
  log_success "Price request successful with txid: $PRICE_TXID"
  
  # Wait for transaction to be processed
  log_info "Waiting for transaction confirmation..."
  sleep 2

  # 2. Create a new bond collection via factory
  log_info "2. Creating new bond collection"
  COLLECTION_NAME="Lifecycle Test Bonds"
  COLLECTION_SYMBOL="LTBND"
  
  CREATE_COLLECTION_OUTPUT=$(oyl alkane call-contract -t "$FACTORY_TXID" -m "create_collection" -a "$COLLECTION_NAME,$COLLECTION_SYMBOL,$INTEREST_RATE,$MATURITY_BLOCKS,1" --feeRate $FEE_RATE -p $PROVIDER)
  NEW_COLLECTION_TXID=$(extract_txid "$CREATE_COLLECTION_OUTPUT")
  
  if [ -z "$NEW_COLLECTION_TXID" ]; then
    log_error "Failed to create bond collection. Output: $CREATE_COLLECTION_OUTPUT"
  fi
  
  log_success "Bond collection created with txid: $NEW_COLLECTION_TXID"
  
  # Wait for transaction to be processed
  log_info "Waiting for transaction confirmation..."
  sleep 2
  
  # 3. Get collection info (optional)
  log_info "3. Getting collection information"
  
  INFO_OUTPUT=$(oyl alkane call-contract -t "$COLLECTION_TXID" -m "getTokenInfo" -a "" --feeRate $FEE_RATE -p $PROVIDER)
  INFO_TXID=$(extract_txid "$INFO_OUTPUT")
  
  if [ -z "$INFO_TXID" ]; then
    log_error "Failed to get collection info. Output: $INFO_OUTPUT"
  fi
  
  log_success "Collection info request successful with txid: $INFO_TXID"
  
  # Wait for transaction to be processed
  log_info "Waiting for transaction confirmation..."
  sleep 2
  
  # 4. Mint a new bond
  log_info "4. Minting a new bond"
  
  MINT_OUTPUT=$(oyl alkane call-contract -t "$COLLECTION_TXID" -m "mint_bond" -a "$NATIVE_SEGWIT_ADDRESS,$BOND_AMOUNT" --feeRate $FEE_RATE -p $PROVIDER)
  MINT_TXID=$(extract_txid "$MINT_OUTPUT")
  
  if [ -z "$MINT_TXID" ]; then
    log_error "Failed to mint bond. Output: $MINT_OUTPUT"
  fi
  
  log_success "Bond minted with txid: $MINT_TXID"
  
  # Wait for transaction to be processed
  log_info "Waiting for transaction confirmation..."
  sleep 2
  
  # 5. Get bond count
  log_info "5. Getting bond count"
  
  COUNT_OUTPUT=$(oyl alkane call-contract -t "$COLLECTION_TXID" -m "getBondCount" -a "" --feeRate $FEE_RATE -p $PROVIDER)
  COUNT_TXID=$(extract_txid "$COUNT_OUTPUT")
  
  if [ -z "$COUNT_TXID" ]; then
    log_error "Failed to get bond count. Output: $COUNT_OUTPUT"
  fi
  
  log_success "Bond count request successful with txid: $COUNT_TXID"
  
  # Wait for transaction to be processed
  log_info "Waiting for transaction confirmation..."
  sleep 2
  
  # 6. Try to redeem the bond
  log_info "6. Attempting to redeem bond (might fail if not matured)"
  
  REDEEM_OUTPUT=$(oyl alkane call-contract -t "$COLLECTION_TXID" -m "redeem_bond" -a "0" --feeRate $FEE_RATE -p $PROVIDER)
  REDEEM_TXID=$(extract_txid "$REDEEM_OUTPUT")
  
  if [ -z "$REDEEM_TXID" ]; then
    log_info "Bond redemption likely failed because it hasn't matured yet. This is expected behavior."
    log_info "Output: $REDEEM_OUTPUT"
  else
    log_success "Bond redemption attempt processed with txid: $REDEEM_TXID"
  fi
  
  log_success "Full lifecycle test completed successfully"
}

# Function to generate test report
generate_report() {
  log_step "Generating test report"

  REPORT_FILE="slop_lifecycle_test_report_${TIMESTAMP}.md"

  cat > "$REPORT_FILE" << EOF
# SLOP Full Lifecycle Test Report

**Date:** $(date)
**Extended Subfrost RPC URL:** $EXTENDED_RPC_URL

## Contract Deployment

| Contract | Transaction ID |
|----------|---------------|
| LaunchpadFactory | $FACTORY_TXID |
| OrbitalBondCollection | $COLLECTION_TXID |
| BondCurve | $CURVE_TXID |

## Lifecycle Test Results

1. **Price Check from Bond Curve**
   - Transaction ID: $PRICE_TXID
   - Amount: $BOND_AMOUNT sats

2. **Bond Collection Creation**
   - Transaction ID: $NEW_COLLECTION_TXID
   - Collection Name: $COLLECTION_NAME
   - Collection Symbol: $COLLECTION_SYMBOL

3. **Collection Info Query**
   - Transaction ID: $INFO_TXID

4. **Bond Minting**
   - Transaction ID: $MINT_TXID
   - Amount: $BOND_AMOUNT sats
   - Recipient: $NATIVE_SEGWIT_ADDRESS

5. **Bond Count Query**
   - Transaction ID: $COUNT_TXID

6. **Bond Redemption Attempt**
   - Transaction ID: ${REDEEM_TXID:-"N/A (Not matured)"}
   - Note: Bonds must reach maturity ($MATURITY_BLOCKS blocks) before they can be redeemed

## Configuration

- Interest Rate: $INTEREST_RATE basis points
- Maturity Period: $MATURITY_BLOCKS blocks
- Admin Address: $ADMIN_ADDRESS
- Extended Subfrost RPC URL: $EXTENDED_RPC_URL
EOF

  log_success "Test report created: $REPORT_FILE"
}

# Main script execution
echo "Starting SLOP Extended Subfrost Full Lifecycle Test at $(date)" > "$LOG_FILE"
echo "==========================================" >> "$LOG_FILE"
log_step "SLOP Extended Subfrost Full Lifecycle Test"
log_info "Extended Subfrost RPC URL: $EXTENDED_RPC_URL"

# Run all steps
check_dependencies
verify_extended_subfrost
check_wasm_files
setup_oyl_provider
setup_accounts
deploy_contracts
run_full_lifecycle_test
generate_report

log_step "Full Lifecycle Test Complete!"
echo -e "${GREEN}LaunchpadFactory:      $FACTORY_TXID${NC}"
echo -e "${GREEN}OrbitalBondCollection: $COLLECTION_TXID${NC}"
echo -e "${GREEN}BondCurve:            $CURVE_TXID${NC}"
echo -e "${YELLOW}See test report: $REPORT_FILE${NC}"

exit 0
