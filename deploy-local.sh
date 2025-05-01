#!/bin/bash
# deploy-local.sh
#
# This script deploys the pre-built SLOP contracts to the Docker instance running at port 18888,
# skipping the build step.

set -e  # Exit on error

# Text colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Docker-specific configuration
DOCKER_RPC_URL="http://localhost:18888"
PROVIDER="alkanes"
FEE_RATE=5
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="deploy-slop-local-${TIMESTAMP}.log"
INTEREST_RATE=500
MATURITY_BLOCKS=100
VERSION="1.0.0"

# Contract paths (using pre-built binaries)
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
  
  local dependencies=("oyl" "jq")
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

# Function to verify Docker instance is running
verify_docker_instance() {
  log_step "Verifying Docker instance at $DOCKER_RPC_URL"
  
  # Try curl to see if RPC is responding - accept any response that's not a connection error
  HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$DOCKER_RPC_URL")
  if [[ "$HTTP_CODE" == "000" ]]; then
    log_error "Docker instance at $DOCKER_RPC_URL is not responding. Make sure it's running."
  else
    log_info "Docker instance responded with HTTP code: $HTTP_CODE"
  fi
  
  log_info "Testing connection to Docker instance"
  
  # Test RPC connection using oyl
  local PROVIDER_OUTPUT
  PROVIDER_OUTPUT=$(oyl provider alkanes -method metashrew_height -p $PROVIDER 2>&1)
  
  if echo "$PROVIDER_OUTPUT" | grep -q "error"; then
    log_info "Warning: Provider check returned errors. This might be expected for a fresh instance."
    log_info "Provider output: $PROVIDER_OUTPUT"
  else
    log_info "Provider check succeeded: $PROVIDER_OUTPUT"
  fi
  
  log_success "Docker instance is running and accessible"
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
  
  log_info "Native SegWit Address: $NATIVE_SEGWIT_ADDRESS"
  log_info "Taproot Address: $TAPROOT_ADDRESS"
  log_info "Admin Address: $ADMIN_ADDRESS"
  log_success "Account setup complete"
}

# Function to deploy contracts
deploy_contracts() {
  log_step "Deploying contracts to Docker instance"
  
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
  log_info "Calldata: 3,$COLLECTION_CONSTANT,$INTEREST_RATE,$MATURITY_BLOCKS"
  
  COLLECTION_OUTPUT=$(oyl alkane new-contract -c "$COLLECTION_CONTRACT_PATH" --calldata "3,$COLLECTION_CONSTANT,$INTEREST_RATE,$MATURITY_BLOCKS" --feeRate $FEE_RATE -p $PROVIDER)
  COLLECTION_TXID=$(extract_txid "$COLLECTION_OUTPUT")
  
  if [ -z "$COLLECTION_TXID" ]; then
    log_error "Failed to deploy OrbitalBondCollection. Output: $COLLECTION_OUTPUT"
  fi
  
  log_success "OrbitalBondCollection deployed with txid: $COLLECTION_TXID"
  
  # Deploy bond curve if it exists
  if [ -f "$CURVE_CONTRACT_PATH" ]; then
    log_info "Deploying BondCurve"
    log_info "Contract path: $CURVE_CONTRACT_PATH"
    log_info "Calldata: 3,$CURVE_CONSTANT,1,0"
    
    CURVE_OUTPUT=$(oyl alkane new-contract -c "$CURVE_CONTRACT_PATH" --calldata "3,$CURVE_CONSTANT,1,0" --feeRate $FEE_RATE -p $PROVIDER)
    CURVE_TXID=$(extract_txid "$CURVE_OUTPUT")
    
    if [ -z "$CURVE_TXID" ]; then
      log_info "Warning: Failed to deploy BondCurve. Output: $CURVE_OUTPUT"
      log_info "Continuing without BondCurve contract"
    else
      log_success "BondCurve deployed with txid: $CURVE_TXID"
    fi
  else
    log_info "BondCurve contract not found, skipping deployment"
  fi
}

# Function to run sanity tests
run_sanity_tests() {
  log_step "Running sanity tests on deployed contracts"
  
  # Create a test collection via factory
  log_info "Creating test bond collection"
  COLLECTION_NAME="Test Bonds"
  COLLECTION_SYMBOL="TBND"
  TEST_INTEREST_RATE=500  # 5.00%
  TEST_MATURITY_BLOCKS=10  # Short maturity for testing
  
  CREATE_COLLECTION_OUTPUT=$(oyl alkane call-contract -t "$FACTORY_TXID" -m "create_collection" -a "$COLLECTION_NAME,$COLLECTION_SYMBOL,$TEST_INTEREST_RATE,$TEST_MATURITY_BLOCKS,1" --feeRate $FEE_RATE -p $PROVIDER)
  TEST_COLLECTION_ID=$(extract_txid "$CREATE_COLLECTION_OUTPUT")
  
  if [ -z "$TEST_COLLECTION_ID" ]; then
    log_error "Failed to create test collection. Output: $CREATE_COLLECTION_OUTPUT"
  fi
  
  log_success "Test collection created with ID: $TEST_COLLECTION_ID"
  
  # Wait for Docker instance to process transaction
  log_info "Waiting for transaction confirmation..."
  sleep 5
  
  # Mint a test bond
  log_info "Minting test bond"
  BOND_AMOUNT=1000000  # 0.01 BTC in satoshis
  
  MINT_OUTPUT=$(oyl alkane call-contract -t "$TEST_COLLECTION_ID" -m "mint_bond" -a "$NATIVE_SEGWIT_ADDRESS,$BOND_AMOUNT" --feeRate $FEE_RATE -p $PROVIDER)
  BOND_ID=$(extract_txid "$MINT_OUTPUT")
  
  if [ -z "$BOND_ID" ]; then
    log_error "Failed to mint test bond. Output: $MINT_OUTPUT"
  fi
  
  log_success "Test bond minted with ID: $BOND_ID"
  
  # Try to redeem the bond
  log_info "Testing bond redemption"
  REDEEM_OUTPUT=$(oyl alkane call-contract -t "$TEST_COLLECTION_ID" -m "redeem_bond_secure" -a "$BOND_ID" --feeRate $FEE_RATE -p $PROVIDER)
  REDEEM_TXID=$(extract_txid "$REDEEM_OUTPUT")
  
  if [ -z "$REDEEM_TXID" ]; then
    log_info "Note: Bond redemption may have failed because the bond hasn't matured in Docker instance"
    log_info "Redemption output: $REDEEM_OUTPUT"
    log_info "This is expected if the Docker instance doesn't generate enough blocks for maturity"
  else
    log_success "Test bond redeemed with txid: $REDEEM_TXID"
  fi
  
  log_success "Sanity tests completed"
}

# Function to generate deployment report
generate_report() {
  log_step "Generating deployment report"
  
  REPORT_FILE="slop_deployment_report_local_${TIMESTAMP}.md"
  
  cat > "$REPORT_FILE" << EOF
# SLOP Local Deployment Report

**Date:** $(date)
**Docker RPC URL:** $DOCKER_RPC_URL
**Version:** $VERSION

## Contract Deployment

| Contract | Transaction ID |
|----------|---------------|
| LaunchpadFactory | $FACTORY_TXID |
| OrbitalBondCollection | $COLLECTION_TXID |
EOF

  if [ -n "${CURVE_TXID:-}" ]; then
    cat >> "$REPORT_FILE" << EOF
| BondCurve | $CURVE_TXID |
EOF
  fi

  cat >> "$REPORT_FILE" << EOF

## Configuration

- Interest Rate: $INTEREST_RATE basis points
- Maturity Period: $MATURITY_BLOCKS blocks
- Admin Address: $ADMIN_ADDRESS
- Docker RPC URL: $DOCKER_RPC_URL

## Sanity Test Results

- Test Collection ID: $TEST_COLLECTION_ID
- Test Bond ID: $BOND_ID
EOF

  if [ -n "${REDEEM_TXID:-}" ]; then
    cat >> "$REPORT_FILE" << EOF
- Successful Redemption: $REDEEM_TXID
EOF
  else
    cat >> "$REPORT_FILE" << EOF
- Bond redemption not completed (may require additional blocks for maturity in Docker instance)
EOF
  fi

  cat >> "$REPORT_FILE" << EOF

## Usage Instructions

### Create a New Bond Collection
\`\`\`bash
oyl alkane call-contract -t "$FACTORY_TXID" -m "create_collection" -a "<name>,<symbol>,<interest_rate>,<maturity_blocks>,<active_flag>" --feeRate $FEE_RATE -p $PROVIDER
\`\`\`

### Mint a Bond
\`\`\`bash
oyl alkane call-contract -t "<collection_id>" -m "mint_bond" -a "<recipient_address>,<amount_in_sats>" --feeRate $FEE_RATE -p $PROVIDER
\`\`\`

### Redeem a Bond
\`\`\`bash
oyl alkane call-contract -t "<collection_id>" -m "redeem_bond_secure" -a "<bond_id>" --feeRate $FEE_RATE -p $PROVIDER
\`\`\`
EOF

  log_success "Deployment report created: $REPORT_FILE"
}

# Main script execution
echo "Starting SLOP Local Deployment at $(date)" > "$LOG_FILE"
echo "==========================================" >> "$LOG_FILE"
log_step "SLOP Local Deployment Script"
log_info "Docker RPC URL: $DOCKER_RPC_URL"

# Run all deployment steps
check_dependencies
verify_docker_instance
check_wasm_files
setup_accounts
deploy_contracts
run_sanity_tests
generate_report

log_step "Deployment Complete!"
echo -e "${GREEN}LaunchpadFactory:      $FACTORY_TXID${NC}"
echo -e "${GREEN}OrbitalBondCollection: $COLLECTION_TXID${NC}"
if [ -n "${CURVE_TXID:-}" ]; then
  echo -e "${GREEN}BondCurve:            $CURVE_TXID${NC}"
fi
echo -e "${YELLOW}See deployment report: $REPORT_FILE${NC}"

exit 0
