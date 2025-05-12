#!/bin/bash

# =============================================
# 🌐 YieldVault Basic Deployment Example
# =============================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Get repository root directory
ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../.." >/dev/null 2>&1 && pwd )"

echo -e "${BOLD}${BLUE}YieldVault Basic Deployment Example${NC}"
echo ""
echo -e "This script demonstrates a basic deployment and interaction with YieldVault."
echo -e "${YELLOW}This is a simplified example for testing purposes only.${NC}"
echo ""

# Step 1: Build WebAssembly binary
echo -e "${BLUE}Step 1: Building WebAssembly binary...${NC}"
cd "$ROOT_DIR"
cargo build --target wasm32-unknown-unknown --release

# Ensure build directory exists
mkdir -p "$ROOT_DIR/build"

# Copy the WebAssembly binary to the build directory
cp "$ROOT_DIR/target/wasm32-unknown-unknown/release/yield_vault.wasm" "$ROOT_DIR/build/yield_vault.wasm"

echo -e "${GREEN}✅ WebAssembly binary built and copied to build directory.${NC}"

# Determine whether to use Docker or direct deployment
if command -v docker &> /dev/null && [ -f "$ROOT_DIR/docker/docker-compose.yml" ]; then
    DEPLOYMENT_METHOD="docker"
    echo -e "${BLUE}Using Docker-based deployment...${NC}"
elif command -v oyl &> /dev/null; then
    DEPLOYMENT_METHOD="direct"
    echo -e "${BLUE}Using direct OylNet deployment...${NC}"
else
    echo -e "${RED}❌ Neither Docker nor the OylNet client are available.${NC}"
    echo -e "${YELLOW}Install Docker or the OylNet client to continue.${NC}"
    exit 1
fi

# Step 2: Deploy the contract
echo -e "\n${BLUE}Step 2: Deploying YieldVault contract...${NC}"
if [ "$DEPLOYMENT_METHOD" == "docker" ]; then
    "$ROOT_DIR/bin/net/docker-deploy.sh" --deploy
else
    "$ROOT_DIR/bin/net/network.sh" --deploy
fi

# Check if deployment was successful
if [ ! -f "$ROOT_DIR/.contract_id" ]; then
    echo -e "${RED}❌ Deployment failed! No contract ID found.${NC}"
    exit 1
fi

# Extract contract ID
CONTRACT_ID=$(cat "$ROOT_DIR/.contract_id" | sed -E 's/^.*txId: '"'"'([0-9a-f]+)'"'"'.*$/\1/')
echo -e "${GREEN}✅ Contract deployed successfully with ID: ${BOLD}$CONTRACT_ID${NC}"

# Step 3: Basic interaction test
echo -e "\n${BLUE}Step 3: Running basic interaction tests...${NC}"
if [ "$DEPLOYMENT_METHOD" == "docker" ]; then
    # For Docker deployment, we'll create a simple test script
    TEST_SCRIPT="$ROOT_DIR/build/basic_test.sh"
    
    cat > "$TEST_SCRIPT" << EOF
#!/bin/bash
source .env

# Read contract ID
CONTRACT_ID=\$(cat .contract_id | sed -E 's/^.*txId: '"'"'([0-9a-f]+)'"'"'.*$/\1/')
echo "Using contract ID: \$CONTRACT_ID"

# Test metadata view functions
echo -e "\n[TEST] Getting contract metadata..."
oyl alkane execute -p oylnet --calldata "100" --contract \$CONTRACT_ID # Get Name
oyl alkane execute -p oylnet --calldata "101" --contract \$CONTRACT_ID # Get Symbol
oyl alkane execute -p oylnet --calldata "102" --contract \$CONTRACT_ID # Get Decimals

# Test basic function - update yield rate
echo -e "\n[TEST] Setting yield rate to 500 basis points (5%)..."
oyl alkane execute -p oylnet --calldata "900,500" --contract \$CONTRACT_ID

# Read yield rate
echo -e "\n[TEST] Reading current yield rate..."
oyl alkane execute -p oylnet --calldata "901" --contract \$CONTRACT_ID

echo -e "\n[TEST] Basic tests completed successfully!"
EOF
    
    chmod +x "$TEST_SCRIPT"
    
    # Run the test in Docker container
    echo -e "${BLUE}Running basic tests in Docker container...${NC}"
    cd "$ROOT_DIR/docker"
    docker-compose run oylnet /bin/bash -c "cd /app && build/basic_test.sh"
else
    # For direct deployment, use network.sh --interact
    "$ROOT_DIR/bin/net/network.sh" --interact
fi

echo -e "\n${GREEN}${BOLD}YieldVault basic deployment and interaction example completed!${NC}"
echo -e "See ${CYAN}docs/DEPLOYMENT_GUIDE.md${NC} for more deployment options and details."
echo -e "For a more comprehensive deployment with security testing, use ${CYAN}bin/test/security_deploy_test.sh${NC}."
