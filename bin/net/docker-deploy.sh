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

# Parse command line arguments
MODE=""
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --test) MODE="test";;
        --deploy) MODE="deploy";;
        --interact) MODE="interact";;
        --help|-h)
            echo -e "${BOLD}YieldVault Docker Deployment Script${NC}"
            echo ""
            echo -e "Usage: $0 [OPTIONS]"
            echo ""
            echo -e "Options:"
            echo -e "  --test       Test connection to OylNet network"
            echo -e "  --deploy     Deploy contract to OylNet network"
            echo -e "  --interact   Interact with deployed contract"
            echo -e "  --help, -h   Show this help message"
            exit 0
            ;;
        *) echo "Unknown parameter: $1"; exit 1;;
    esac
    shift
done

# Check if mode is specified
if [ -z "$MODE" ]; then
    echo -e "${RED}Error: No mode specified. Use --test, --deploy, or --interact.${NC}"
    echo -e "Run '$0 --help' for more information."
    exit 1
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed. Please install Docker to continue.${NC}"
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}Error: docker-compose is not installed. Please install docker-compose to continue.${NC}"
    exit 1
fi

# Check that WebAssembly binary exists
if [ ! -f "${ROOT_DIR}/build/yield_vault.wasm" ]; then
    echo -e "${YELLOW}WebAssembly binary not found in build directory. Building...${NC}"
    
    # Build the WebAssembly binary
    cd "$ROOT_DIR"
    cargo build --target wasm32-unknown-unknown --release
    
    # Ensure build directory exists
    mkdir -p "$ROOT_DIR/build"
    
    # Copy the WebAssembly binary to the build directory
    cp "$ROOT_DIR/target/wasm32-unknown-unknown/release/yield_vault.wasm" "$ROOT_DIR/build/yield_vault.wasm"
    
    echo -e "${GREEN}WebAssembly binary built and copied to build directory.${NC}"
fi

# Start the docker container
echo -e "${BLUE}Starting Docker container for OylNet...${NC}"
cd "$ROOT_DIR/docker"

# Build the image if it doesn't exist
docker-compose build

# Run the container and execute the network.sh script
echo -e "${BLUE}Running $MODE operation in container...${NC}"
docker-compose run oylnet /bin/bash -c "cd /app && bin/net/network.sh --$MODE"

echo -e "${GREEN}Docker deployment completed.${NC}"
