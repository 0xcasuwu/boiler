#!/bin/bash
set -e  # Exit on any error

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print header
print_header() {
    echo -e "${BLUE}====================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}====================================${NC}"
}

# Clean function
clean() {
    print_header "Cleaning project"
    cargo clean
    echo -e "${GREEN}Clean completed${NC}"
}

# Build function
build() {
    print_header "Building SLOP project"
    
    echo -e "${YELLOW}Building with default features${NC}"
    cargo build --verbose
    
    echo -e "${YELLOW}Building with blockchain feature${NC}"
    cargo build --verbose --features blockchain
    
    echo -e "${GREEN}Builds completed successfully${NC}"
}

# Test function
run_tests() {
    print_header "Running SLOP tests"
    
    echo -e "${YELLOW}Running tests with default features${NC}"
    cargo test --verbose
    
    echo -e "${YELLOW}Running tests with blockchain feature${NC}"
    cargo test --verbose --features blockchain
    
    echo -e "${GREEN}Tests completed successfully${NC}"
}

# Run specific test group
run_specific_tests() {
    if [ -z "$1" ]; then
        echo -e "${RED}No test name provided${NC}"
        echo "Usage: ./build.sh test <test_name>"
        exit 1
    fi
    
    print_header "Running specific test: $1"
    cargo test $1 --verbose
    echo -e "${GREEN}Specific tests completed${NC}"
}

# Format code
format() {
    print_header "Formatting code"
    cargo fmt --all
    echo -e "${GREEN}Format completed${NC}"
}

# Check code
check() {
    print_header "Checking code"
    cargo check --verbose
    echo -e "${GREEN}Check completed${NC}"
}

# Clippy
clippy() {
    print_header "Running clippy"
    cargo clippy -- -D warnings
    echo -e "${GREEN}Clippy completed${NC}"
}

# Help message
show_help() {
    echo "SLOP Build Script"
    echo ""
    echo "Usage: ./build.sh [command]"
    echo ""
    echo "Commands:"
    echo "  build           Build the project (default)"
    echo "  clean           Clean build artifacts"
    echo "  test            Run all tests"
    echo "  test <name>     Run specific test(s)"
    echo "  format          Format code with rustfmt"
    echo "  check           Check code for errors"
    echo "  clippy          Run clippy linter"
    echo "  help            Show this help message"
    echo ""
}

# Script entry point
case "$1" in
    clean)
        clean
        ;;
    test)
        shift
        if [ -n "$1" ]; then
            run_specific_tests "$1"
        else
            run_tests
        fi
        ;;
    format)
        format
        ;;
    check)
        check
        ;;
    clippy)
        clippy
        ;;
    help)
        show_help
        ;;
    build|"")
        build
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        show_help
        exit 1
        ;;
esac

exit 0
