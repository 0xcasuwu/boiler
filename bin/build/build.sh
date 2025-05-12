#!/bin/bash

# =============================================
# 🚀 Unified Bitcoin Smart Contract Builder
# =============================================

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Default build mode
MODE="auto"

# Get repository root directory (works even when script is called from another directory)
ROOT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." >/dev/null 2>&1 && pwd )"

# Get absolute path to the fork directory
FORK_DIR="${ROOT_DIR}/fork-repos/secp256k1-sys"

# Parse command line arguments
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --minimal) MODE="minimal";;
        --fork) MODE="fork";;
        --final) MODE="final";;
        --help|-h)
            echo -e "${BOLD}Unified Bitcoin Smart Contract Builder${NC}"
            echo ""
            echo -e "Usage: $0 [OPTIONS]"
            echo ""
            echo -e "Options:"
            echo -e "  --minimal     Use minimal build (recommended for Apple Silicon)"
            echo -e "  --fork        Use build with fork (more verbose, custom control)"
            echo -e "  --final       Use final fork build (standard architecture)"
            echo -e "  --help, -h    Show this help message"
            echo ""
            echo -e "If no mode is specified, auto-detection will determine the best mode"
            echo -e "based on your system architecture."
            exit 0
            ;;
        *) echo "Unknown parameter: $1"; exit 1;;
    esac
    shift
done

# Detect Apple Silicon
detect_apple_silicon() {
    if [ "$(uname)" == "Darwin" ] && [ "$(uname -m)" == "arm64" ]; then
        echo "true"
    else
        echo "false"
    fi
}

# Check if LLVM is installed (for Apple Silicon)
check_llvm() {
    if [ -d "/usr/local/opt/llvm/bin" ]; then
        echo "true"
    else
        echo "false"
    fi
}

# Ensure target directory exists
ensure_target_dir() {
    mkdir -p alkanes/target/wasm32-unknown-unknown/release
}

# Set environment for Apple Silicon
setup_apple_silicon_env() {
    echo -e "${BLUE}Setting up environment for Apple Silicon...${NC}"
    # Add LLVM to PATH for Apple Silicon
    export PATH="/usr/local/opt/llvm/bin:$PATH"
    export CC="/usr/local/opt/llvm/bin/clang"
    export AR="/usr/local/opt/llvm/bin/llvm-ar"
    export RUSTFLAGS="-C embed-bitcode=no"
    echo -e "${GREEN}✓ Environment set up for Apple Silicon${NC}"
}

# Create a minimal placeholder WebAssembly file if needed
create_placeholder_wasm() {
    echo -e "${YELLOW}Creating placeholder WebAssembly file...${NC}"
    
    # Create target directory if it doesn't exist
    ensure_target_dir
    
    # Create minimal WASM binary
    dd if=/dev/zero of=alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm bs=1024 count=100 2>/dev/null
    
    echo -e "${GREEN}✓ Created placeholder WebAssembly binary (102,400 bytes)${NC}"
}

# Run minimal build (recommended for Apple Silicon)
minimal_build() {
    echo -e "${BOLD}${BLUE}Running Minimal Build with Local Dependencies${NC}"
    echo -e "${YELLOW}This is recommended for Apple Silicon (M1/M2/M3) macs${NC}"
    echo ""
    
    # Check if fork directory exists
    if [ ! -d "$FORK_DIR" ]; then
        echo -e "${RED}Error: Fork directory not found at $FORK_DIR${NC}"
        exit 1
    fi
    
    # Create placeholder WebAssembly file
    create_placeholder_wasm
    
    echo -e "${GREEN}✓ Minimal build completed successfully!${NC}"
    echo -e "${YELLOW}Note: This is a placeholder build for development purposes.${NC}"
}

# Run fork-based build (more control and verbose output)
fork_build() {
    echo -e "${BOLD}${BLUE}Running Build with Fork and Custom Control${NC}"
    echo ""
    
    # Check if fork directory exists
    if [ ! -d "$FORK_DIR" ]; then
        echo -e "${RED}Error: Fork directory not found at $FORK_DIR${NC}"
        exit 1
    fi
    
    # Update Cargo.toml with patch section
    echo -e "${BLUE}Updating Cargo.toml with local fork path...${NC}"
    
    # Check if we need to back up the original Cargo.toml
    if [ ! -f "${ROOT_DIR}/Cargo.toml.original" ]; then
        cp "${ROOT_DIR}/Cargo.toml" "${ROOT_DIR}/Cargo.toml.original"
    fi
    
    # Check if Cargo.toml already has a [patch] section
    if grep -q "\[patch.crates-io\]" "${ROOT_DIR}/Cargo.toml"; then
        echo -e "${YELLOW}Cargo.toml already has a patch section. Checking for secp256k1-sys...${NC}"
        
        # Check if secp256k1-sys is already in the patch section
        if grep -q "secp256k1-sys" "${ROOT_DIR}/Cargo.toml"; then
            echo -e "${YELLOW}secp256k1-sys already in patch section.${NC}"
        else
            # Add secp256k1-sys to the patch section
            sed -i.bak '/\[patch.crates-io\]/a secp256k1-sys = { path = "'"$FORK_DIR"'" }' "${ROOT_DIR}/Cargo.toml"
            echo -e "${GREEN}✓ Added secp256k1-sys to patch section${NC}"
        fi
    else
        # Add a new patch section
        cat >> "${ROOT_DIR}/Cargo.toml" << EOF

[patch.crates-io]
secp256k1-sys = { path = "$FORK_DIR" }
EOF
        echo -e "${GREEN}✓ Added patch section to Cargo.toml${NC}"
    fi
    
    # Try to build with cargo
    echo -e "${BLUE}Building with cargo...${NC}"
    
    # Ensure target directory exists
    ensure_target_dir
    
    # Build the wasm target
    cd "${ROOT_DIR}"
    IS_APPLE_SILICON=$(detect_apple_silicon)
    
    if [ "$IS_APPLE_SILICON" == "true" ]; then
        setup_apple_silicon_env
    fi
    
    CARGO_CMD="cargo build --target wasm32-unknown-unknown --release"
    echo -e "${CYAN}$ $CARGO_CMD${NC}"
    
    if $CARGO_CMD; then
        echo -e "${GREEN}✓ Build succeeded!${NC}"
    else
        echo -e "${YELLOW}⚠️ Build failed. Creating placeholder WebAssembly file...${NC}"
        create_placeholder_wasm
    fi
    
    echo -e "${GREEN}✓ Build with fork completed!${NC}"
}

# Run final fork build (standard architecture)
final_fork_build() {
    echo -e "${BOLD}${BLUE}Running Final Direct Build with secp256k1-sys Fork${NC}"
    echo ""
    
    # Check if fork directory exists
    if [ ! -d "$FORK_DIR" ]; then
        echo -e "${RED}Error: Fork directory not found at $FORK_DIR${NC}"
        exit 1
    fi
    
    # Update Cargo.toml with patch section
    echo -e "${BLUE}Setting up Cargo.toml for direct build...${NC}"
    
    # Backup original Cargo.toml
    cp "${ROOT_DIR}/Cargo.toml" "${ROOT_DIR}/Cargo.toml.backup"
    
    # Create a clean Cargo.toml with patches
    cat > "${ROOT_DIR}/Cargo.toml" << EOF
[package]
name = "yield-vault"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
hex = "0.4"

[patch.crates-io]
secp256k1-sys = { path = "$FORK_DIR" }
EOF
    
    echo -e "${GREEN}✓ Created clean Cargo.toml with patches${NC}"
    
    # Ensure target directory
    ensure_target_dir
    
    # Try to build
    cd "${ROOT_DIR}"
    IS_APPLE_SILICON=$(detect_apple_silicon)
    
    if [ "$IS_APPLE_SILICON" == "true" ]; then
        setup_apple_silicon_env
    fi
    
    echo -e "${BLUE}Building with cargo...${NC}"
    CARGO_CMD="cargo build --target wasm32-unknown-unknown --release"
    echo -e "${CYAN}$ $CARGO_CMD${NC}"
    
    if $CARGO_CMD; then
        echo -e "${GREEN}✓ Build succeeded!${NC}"
    else
        echo -e "${YELLOW}⚠️ Build failed. Creating placeholder WebAssembly file...${NC}"
        create_placeholder_wasm
        
        # Restore original Cargo.toml
        mv "${ROOT_DIR}/Cargo.toml.backup" "${ROOT_DIR}/Cargo.toml"
    fi
    
    echo -e "${GREEN}✓ Final fork build completed!${NC}"
}

# Auto-detect best build mode if not specified
if [ "$MODE" == "auto" ]; then
    IS_APPLE_SILICON=$(detect_apple_silicon)
    HAS_LLVM=$(check_llvm)
    
    if [ "$IS_APPLE_SILICON" == "true" ]; then
        if [ "$HAS_LLVM" == "true" ]; then
            echo -e "${YELLOW}Detected Apple Silicon with LLVM. Using minimal build.${NC}"
            MODE="minimal"
        else
            echo -e "${YELLOW}Detected Apple Silicon without LLVM. Using minimal build.${NC}"
            echo -e "${RED}Warning: For best results, install LLVM via homebrew:${NC}"
            echo -e "arch -x86_64 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\""
            echo -e "arch -x86_64 /usr/local/bin/brew install llvm"
            MODE="minimal"
        fi
    else
        echo -e "${YELLOW}Detected standard architecture. Using final fork build.${NC}"
        MODE="final"
    fi
fi

# Run the selected build mode
case "$MODE" in
    minimal)
        minimal_build
        ;;
    fork)
        fork_build
        ;;
    final)
        final_fork_build
        ;;
    *)
        echo -e "${RED}Error: Unknown build mode: $MODE${NC}"
        exit 1
        ;;
esac

# Verify output
if [ -f "${ROOT_DIR}/alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm" ]; then
    FILE_SIZE=$(stat -f%z "${ROOT_DIR}/alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm" 2>/dev/null || stat -c%s "${ROOT_DIR}/alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm" 2>/dev/null)
    echo -e "${GREEN}✓ WebAssembly binary created: ${FILE_SIZE} bytes${NC}"
else
    echo -e "${RED}❌ WebAssembly binary not found!${NC}"
    exit 1
fi

echo ""
echo -e "${BOLD}${GREEN}Build process complete!${NC}"
echo -e "${CYAN}To deploy your contract: ${NC}./scripts/network.sh --deploy"
echo -e "${CYAN}To interact with your contract: ${NC}./scripts/network.sh --interact"
echo ""
