#!/bin/bash

# YieldVault Main Entry Point Script
# This script serves as a unified entry point for all YieldVault operations

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Display help information
function show_help {
    echo -e "${BOLD}YieldVault Smart Contract Toolkit${NC}"
    echo ""
    echo -e "${BOLD}Usage:${NC} ./yield-vault.sh [COMMAND] [OPTIONS]"
    echo ""
    echo -e "${BOLD}Commands:${NC}"
    echo "  build      Build the WebAssembly contract"
    echo "  test       Run the tests"
    echo "  net        Interact with OylNet network"
    echo "  help       Display this help message"
    echo ""
    echo -e "${BOLD}Examples:${NC}"
    echo "  ./yield-vault.sh build            # Build with auto-detection"
    echo "  ./yield-vault.sh build --minimal  # Use minimal build mode"
    echo "  ./yield-vault.sh test             # Run working tests"
    echo "  ./yield-vault.sh net --deploy     # Deploy to OylNet"
    echo ""
    echo -e "${BOLD}For more details:${NC}"
    echo "  See TOOLCHAIN_SETUP.md for comprehensive documentation"
}

# Ensure the bin directory exists
if [ ! -d "bin" ]; then
    echo -e "${RED}Error: The 'bin' directory doesn't exist.${NC}"
    echo "This script requires the reorganized project structure."
    exit 1
fi

# Parse command-line arguments
COMMAND=$1
shift # Remove the command from the arguments list

case $COMMAND in
    build)
        echo -e "${BLUE}Running build with arguments: $@${NC}"
        ./bin/build/build.sh "$@"
        ;;
    test)
        echo -e "${BLUE}Running tests...${NC}"
        ./bin/test/run_working_tests.sh
        ;;
    net)
        if [ "$1" = "--help" ] || [ "$1" = "-h" ] || [ -z "$1" ]; then
            echo -e "${BOLD}Network operations:${NC}"
            echo "  --test     Test connection to OylNet"
            echo "  --deploy   Deploy contract to OylNet"
            echo "  --interact Interact with deployed contract"
            exit 0
        fi
        echo -e "${BLUE}Running network operations with arguments: $@${NC}"
        ./bin/net/network.sh "$@"
        ;;
    help|--help|-h|"")
        show_help
        ;;
    *)
        echo -e "${RED}Unknown command: $COMMAND${NC}"
        show_help
        exit 1
        ;;
esac
