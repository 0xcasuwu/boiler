#!/bin/bash
# deploy-slop.sh
#
# This script automates the deployment of the SLOP (Smart Launchpad with Orbital Bonds)
# contracts to specified environments, with comprehensive testing and verification.
#
# Usage: ./deploy-slop.sh [--env <environment>] [--clean] [--skip-tests] [--skip-audit] [--features <feature_list>]
#
# Options:
#   --env <environment>    Target environment (local, testnet, mainnet). Default: local
#   --clean                Clean build artifacts before building
#   --skip-tests           Skip running tests before deployment
#   --skip-audit           Skip security audit
#   --features <features>  Comma-separated list of Rust features to enable. Default: blockchain

set -e  # Exit on error

# Text colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
ENVIRONMENT="local"
CLEAN_BUILD=false
SKIP_TESTS=false
SKIP_AUDIT=false
FEATURES="blockchain"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
LOG_FILE="deploy-slop-${TIMESTAMP}.log"
CONFIG_FILE="deploy_config.json"

# Contract paths
FACTORY_CONTRACT_PATH="./target/wasm32-unknown-unknown/release/slop_launchpad_factory.wasm"
COLLECTION_CONTRACT_PATH="./target/wasm32-unknown-unknown/release/slop_orbital_bond_collection.wasm"
CURVE_CONTRACT_PATH="./target/wasm32-unknown-unknown/release/slop_bond_curve.wasm"

# Contract constants
FACTORY_CONSTANT="100001"
COLLECTION_CONSTANT="100002"
CURVE_CONSTANT="100003"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --env)
      ENVIRONMENT="$2"
      shift 2
      ;;
    --clean)
      CLEAN_BUILD=true
      shift
      ;;
    --skip-tests)
      SKIP_TESTS=true
      shift
      ;;
    --skip-audit)
      SKIP_AUDIT=true
      shift
      ;;
    --features)
      FEATURES="$2"
      shift 2
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      exit 1
      ;;
  esac
done

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
  
  local dependencies=("cargo" "rustc" "wasm-pack" "oyl" "jq")
  local missing_deps=()
  
  for dep in "${dependencies[@]}"; do
    if ! command -v "$dep" &> /dev/null; then
      missing_deps+=("$dep")
    fi
  done
  
  if [ ${#missing_deps[@]} -gt 0 ]; then
    log_error "Missing dependencies: ${missing_deps[*]}\nPlease install them before continuing."
  fi
  
  # Check Rust wasm32 target
  if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    log_error "wasm32-unknown-unknown target not installed. Run: rustup target add wasm32-unknown-unknown"
  fi
  
  log_success "All dependencies are installed"
}

# Function to extract txid from oyl output
extract_txid() {
  echo "$1" | grep -oP 'txid: \K[a-f0-9]+' || echo "$1" | grep -oP "txId: '\K[a-f0-9]+(?=')" | head -n 1
}

# Function to create or update configuration file
setup_config() {
  log_step "Setting up configuration"
  
  # Create default config if it doesn't exist
  if [ ! -f "$CONFIG_FILE" ]; then
    log_info "Creating default configuration file: $CONFIG_FILE"
    cat > "$CONFIG_FILE" << EOF
{
  "network": {
    "mainnet": {
      "rpc_url": "https://mainnet.example.com/rpc",
      "api_key": "",
      "default_interest_rate": 500,
      "default_maturity_blocks": 10080
    },
    "testnet": {
      "rpc_url": "https://testnet.example.com/rpc",
      "api_key": "",
      "default_interest_rate": 500,
      "default_maturity_blocks": 1008
    },
    "local": {
      "rpc_url": "http://localhost:18888",
      "api_key": "",
      "default_interest_rate": 500,
      "default_maturity_blocks": 100
    }
  },
  "factory_settings": {
    "version": "1.0.0",
    "admin_address": ""
  }
}
EOF
  fi
  
  # Load configuration
  if ! jq -e . "$CONFIG_FILE" > /dev/null 2>&1; then
    log_error "Invalid JSON in $CONFIG_FILE"
  fi
  
  # Extract values for current environment
  RPC_URL=$(jq -r ".network.$ENVIRONMENT.rpc_url" "$CONFIG_FILE")
  API_KEY=$(jq -r ".network.$ENVIRONMENT.api_key" "$CONFIG_FILE")
  INTEREST_RATE=$(jq -r ".network.$ENVIRONMENT.default_interest_rate" "$CONFIG_FILE")
  MATURITY_BLOCKS=$(jq -r ".network.$ENVIRONMENT.default_maturity_blocks" "$CONFIG_FILE")
  VERSION=$(jq -r ".factory_settings.version" "$CONFIG_FILE")
  ADMIN_ADDRESS=$(jq -r ".factory_settings.admin_address" "$CONFIG_FILE")
  
  # Check values
  if [ "$RPC_URL" = "null" ] || [ -z "$RPC_URL" ]; then
    log_error "RPC URL not configured for $ENVIRONMENT in $CONFIG_FILE"
  fi
  
  # Set environment-specific parameters
  case $ENVIRONMENT in
    "local")
      FEE_RATE=5
      BLOCKS_TO_GENERATE=6
      PROVIDER="alkanes"  # Default provider for local development
      ;;
    "testnet")
      FEE_RATE=10
      BLOCKS_TO_GENERATE=0
      PROVIDER="testnet-alkanes"
      ;;
    "mainnet")
      FEE_RATE=20
      BLOCKS_TO_GENERATE=0
      PROVIDER="mainnet-alkanes"
      
      # Extra confirmation for mainnet deployment
      if [ "$SKIP_TESTS" = true ] || [ "$SKIP_AUDIT" = true ]; then
        log_info "WARNING: You're deploying to MAINNET with tests or security audit skipped."
        read -p "Are you ABSOLUTELY sure you want to continue? (yes/no) " confirm
        if [ "$confirm" != "yes" ]; then
          log_error "Deployment aborted by user"
        fi
      else
        read -p "You're deploying to MAINNET. Are you sure? (yes/no) " confirm
        if [ "$confirm" != "yes" ]; then
          log_error "Deployment aborted by user"
        fi
      fi
      ;;
    *)
      log_error "Unknown environment: $ENVIRONMENT"
      ;;
  esac
  
  log_success "Configuration loaded for $ENVIRONMENT"
}

# Function to run security audit
run_security_audit() {
  if [ "$SKIP_AUDIT" = true ]; then
    log_info "Security audit skipped"
    return 0
  fi
  
  log_step "Running security audit"
  
  if [ -f "./security-audit.sh" ]; then
    ./security-audit.sh
    
    if [ $? -ne 0 ]; then
      log_error "Security audit failed. Fix issues before deploying."
    fi
    
    log_success "Security audit passed"
  else
    log_info "security-audit.sh not found, skipping audit"
  fi
}

# Function to run tests
run_tests() {
  if [ "$SKIP_TESTS" = true ]; then
    log_info "Tests skipped"
    return 0
  fi
  
  log_step "Running tests"
  
  # Unit tests
  log_info "Running unit tests"
  if ! cargo test --lib -- --quiet; then
    log_error "Unit tests failed"
  fi
  log_success "Unit tests passed"
  
  # Security-specific tests if they exist
  if grep -q "security_fixes_test" ./src/tests/mod.rs 2>/dev/null; then
    log_info "Running security fix tests"
    if ! cargo test security_fixes_test -- --quiet; then
      log_error "Security fix tests failed"
    fi
    log_success "Security fix tests passed"
  fi
  
  # Property-based tests if they exist
  if grep -q "property_tests" ./src/tests/mod.rs 2>/dev/null; then
    log_info "Running property-based tests"
    if ! cargo test property_tests -- --quiet; then
      log_error "Property-based tests failed"
    fi
    log_success "Property-based tests passed"
  fi
  
  log_success "All tests passed"
}

# Function to clean build artifacts
clean_build() {
  if [ "$CLEAN_BUILD" = true ]; then
    log_step "Cleaning build artifacts"
    cargo clean
    log_success "Build artifacts cleaned"
  fi
}

# Function to build contracts
build_contracts() {
  log_step "Building WASM contracts"
  
  # Create feature flag argument
  local features_arg=""
  if [ -n "$FEATURES" ]; then
    features_arg="--features $FEATURES"
  fi
  
  # Build contracts
  log_info "Building with features: $FEATURES"
  cargo build --release --target wasm32-unknown-unknown $features_arg
  
  # Check if contracts exist
  local missing_contracts=()
  
  # Get actual contract paths from target directory
  for contract_path in $(find ./target/wasm32-unknown-unknown/release/ -name "*.wasm"); do
    log_info "Found contract: $contract_path"
    
    # Store paths for later use
    if [[ "$contract_path" == *launchpad*factory* ]]; then
      FACTORY_CONTRACT_PATH=$contract_path
    elif [[ "$contract_path" == *orbital*bond*collection* ]]; then
      COLLECTION_CONTRACT_PATH=$contract_path
    elif [[ "$contract_path" == *bond*curve* ]]; then
      CURVE_CONTRACT_PATH=$contract_path
    fi
  done
  
  # Check if main contracts exist
  if [ ! -f "$FACTORY_CONTRACT_PATH" ]; then
    missing_contracts+=("LaunchpadFactory ($FACTORY_CONTRACT_PATH)")
  fi
  
  if [ ! -f "$COLLECTION_CONTRACT_PATH" ]; then
    missing_contracts+=("OrbitalBondCollection ($COLLECTION_CONTRACT_PATH)")
  fi
  
  if [ ${#missing_contracts[@]} -gt 0 ]; then
    log_error "Missing contract artifacts: ${missing_contracts[*]}"
  fi
  
  log_success "WASM contracts built successfully"
}

# Function to initialize environment
initialize_environment() {
  log_step "Initializing $ENVIRONMENT environment"
  
  if [ "$ENVIRONMENT" = "local" ]; then
    log_info "Initializing regtest environment"
    oyl regtest init -p $PROVIDER
    
    log_info "Generating initial blocks"
    oyl regtest genBlocks -p $PROVIDER
    
    log_success "Local environment initialized"
  else
    log_info "Using existing $ENVIRONMENT environment at $RPC_URL"
    # Additional environment-specific initialization can go here
  fi
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
  
  # Set admin address if not specified in config
  if [ -z "$ADMIN_ADDRESS" ] || [ "$ADMIN_ADDRESS" = "null" ]; then
    ADMIN_ADDRESS=$NATIVE_SEGWIT_ADDRESS
    log_info "Admin address not specified in config, using $ADMIN_ADDRESS"
  fi
  
  log_info "Native SegWit Address: $NATIVE_SEGWIT_ADDRESS"
  log_info "Taproot Address: $TAPROOT_ADDRESS"
  log_info "Admin Address: $ADMIN_ADDRESS"
  
  # Fund accounts in local environment
  if [ "$ENVIRONMENT" = "local" ]; then
    log_info "Funding addresses with test coins"
    oyl regtest sendFromFaucet -t "$NATIVE_SEGWIT_ADDRESS" -p $PROVIDER -s 100000000
    
    log_info "Generating blocks to confirm funding"
    for i in {1..6}; do
      oyl regtest genBlocks -p $PROVIDER
      sleep 1
    done
    
    log_success "Accounts funded and ready"
  fi
}

# Function to deploy contracts
deploy_contracts() {
  log_step "Deploying contracts"
  
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
  
  # Generate blocks in local environment
  if [ "$ENVIRONMENT" = "local" ] && [ $BLOCKS_TO_GENERATE -gt 0 ]; then
    log_info "Generating blocks to confirm transaction"
    for i in {1..3}; do
      oyl regtest genBlocks -p $PROVIDER
      sleep 1
    done
  fi
  
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
  
  # Generate blocks in local environment
  if [ "$ENVIRONMENT" = "local" ] && [ $BLOCKS_TO_GENERATE -gt 0 ]; then
    log_info "Generating blocks to confirm transaction"
    for i in {1..3}; do
      oyl regtest genBlocks -p $PROVIDER
      sleep 1
    done
  fi
  
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
  
  # Only run sanity tests in local environment
  if [ "$ENVIRONMENT" != "local" ]; then
    log_info "Skipping sanity tests in $ENVIRONMENT environment"
    return 0
  fi
  
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
  
  # Generate blocks
  log_info "Generating blocks to confirm transaction"
  for i in {1..3}; do
    oyl regtest genBlocks -p $PROVIDER
    sleep 1
  done
  
  # Mint a test bond
  log_info "Minting test bond"
  BOND_AMOUNT=1000000  # 0.01 BTC in satoshis
  
  MINT_OUTPUT=$(oyl alkane call-contract -t "$TEST_COLLECTION_ID" -m "mint_bond" -a "$NATIVE_SEGWIT_ADDRESS,$BOND_AMOUNT" --feeRate $FEE_RATE -p $PROVIDER)
  BOND_ID=$(extract_txid "$MINT_OUTPUT")
  
  if [ -z "$BOND_ID" ]; then
    log_error "Failed to mint test bond. Output: $MINT_OUTPUT"
  fi
  
  log_success "Test bond minted with ID: $BOND_ID"
  
  # Generate blocks to simulate maturity
  log_info "Simulating bond maturity"
  for i in {1..12}; do
    oyl regtest genBlocks -p $PROVIDER
    sleep 1
  done
  
  # Test bond redemption
  log_info "Testing bond redemption"
  REDEEM_OUTPUT=$(oyl alkane call-contract -t "$TEST_COLLECTION_ID" -m "redeem_bond_secure" -a "$BOND_ID" --feeRate $FEE_RATE -p $PROVIDER)
  REDEEM_TXID=$(extract_txid "$REDEEM_OUTPUT")
  
  if [ -z "$REDEEM_TXID" ]; then
    log_error "Failed to redeem test bond. Output: $REDEEM_OUTPUT"
  fi
  
  log_success "Test bond redeemed with txid: $REDEEM_TXID"
  
  log_success "All sanity tests passed"
}

# Function to generate deployment report
generate_report() {
  log_step "Generating deployment report"
  
  REPORT_FILE="slop_deployment_report_${TIMESTAMP}.md"
  
  cat > "$REPORT_FILE" << EOF
# SLOP Deployment Report

**Date:** $(date)
**Environment:** $ENVIRONMENT
**Version:** $VERSION
**Features:** $FEATURES

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
- RPC URL: $RPC_URL

## Sanity Test Results
EOF

  if [ "$ENVIRONMENT" = "local" ]; then
    cat >> "$REPORT_FILE" << EOF
- Test Collection ID: $TEST_COLLECTION_ID
- Test Bond ID: $BOND_ID
- Successful Redemption: $REDEEM_TXID

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
  else
    cat >> "$REPORT_FILE" << EOF
- Sanity tests skipped in $ENVIRONMENT environment.

## Contract Verification

Before production use, please verify contracts with the following instructions:
1. Check transaction details at block explorer
2. Verify WASM bytecode matches local build
3. Test interface endpoints with dry-run calls
EOF
  fi

  log_success "Deployment report created: $REPORT_FILE"
}

# Main script execution
echo "Starting SLOP Deployment at $(date)" > "$LOG_FILE"
echo "==========================================" >> "$LOG_FILE"
log_step "SLOP Deployment Script"
log_info "Environment: $ENVIRONMENT"
log_info "Features: $FEATURES"

# Run all deployment steps
check_dependencies
setup_config
run_security_audit
run_tests
clean_build
build_contracts
initialize_environment
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
