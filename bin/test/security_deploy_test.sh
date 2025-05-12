#!/bin/bash

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

echo -e "${BOLD}${BLUE}YieldVault Security Test & Deployment Suite${NC}"
echo ""

# Step 1: Run the basic security tests
echo -e "${BLUE}Running basic security tests on mock implementation...${NC}"
cd "$ROOT_DIR"
cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu
test_exit=$?

if [ $test_exit -ne 0 ]; then
    echo -e "${RED}❌ Basic security tests failed! Fix issues before deploying.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Basic security tests passed!${NC}"

# Step 2: Run the combined security tests
echo -e "${BLUE}Running combined security tests...${NC}"
cargo test --test mock_vault_combined_tests --target x86_64-unknown-linux-gnu
test_exit=$?

if [ $test_exit -ne 0 ]; then
    echo -e "${RED}❌ Combined security tests failed! Fix issues before deploying.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Combined security tests passed!${NC}"

# Step 3: Run token model tests
echo -e "${BLUE}Running token model tests...${NC}"
cargo test --test mock_vault_token_tests --target x86_64-unknown-linux-gnu
test_exit=$?

if [ $test_exit -ne 0 ]; then
    echo -e "${YELLOW}⚠️ Token model tests failed! Consider fixing before deploying.${NC}"
    read -p "Continue with deployment anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Step 4: Build the WebAssembly binary
echo -e "${BLUE}Building WebAssembly binary...${NC}"
cargo build --target wasm32-unknown-unknown --release
build_exit=$?

if [ $build_exit -ne 0 ]; then
    echo -e "${RED}❌ WebAssembly build failed!${NC}"
    exit 1
fi

echo -e "${GREEN}✅ WebAssembly binary built successfully!${NC}"

# Copy WebAssembly to build directory
mkdir -p "$ROOT_DIR/build"
cp "$ROOT_DIR/target/wasm32-unknown-unknown/release/yield_vault.wasm" "$ROOT_DIR/build/yield_vault.wasm"

# Step 5: Ask if user wants to deploy
echo ""
echo -e "${YELLOW}Security tests passed and WebAssembly binary built.${NC}"
read -p "Do you want to deploy to OylNet now? (y/n) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Check if Docker is available
    if command -v docker &> /dev/null; then
        echo -e "${BLUE}Using Docker-based deployment...${NC}"
        "$ROOT_DIR/bin/net/docker-deploy.sh" --deploy
    else
        # Check if oyl client is available
        if command -v oyl &> /dev/null; then
            echo -e "${BLUE}Using direct OylNet deployment...${NC}"
            "$ROOT_DIR/bin/net/network.sh" --deploy
        else
            echo -e "${RED}❌ Neither Docker nor the OylNet client are available.${NC}"
            echo -e "${YELLOW}Install Docker or the OylNet client to deploy.${NC}"
            exit 1
        fi
    fi
    
    # Step 6: Ask if user wants to interact with the contract
    echo ""
    read -p "Do you want to interact with the deployed contract now? (y/n) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Check if Docker is available
        if command -v docker &> /dev/null; then
            echo -e "${BLUE}Using Docker-based interaction...${NC}"
            "$ROOT_DIR/bin/net/docker-deploy.sh" --interact
        else
            # Check if oyl client is available
            if command -v oyl &> /dev/null; then
                echo -e "${BLUE}Using direct OylNet interaction...${NC}"
                "$ROOT_DIR/bin/net/network.sh" --interact
            else
                echo -e "${RED}❌ Neither Docker nor the OylNet client are available.${NC}"
                exit 1
            fi
        fi
    fi
fi

echo -e "${GREEN}${BOLD}YieldVault security testing and deployment process complete!${NC}"
echo -e "For more information on deployment options, see docs/DEPLOYMENT_GUIDE.md"
