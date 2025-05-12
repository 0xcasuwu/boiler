#!/bin/bash
# Setup script for yield-vault project

set -e  # Exit on any error

echo "===== Setting Up Rust Toolchain for yield-vault ====="

# Ensure WebAssembly target is installed
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
else
    echo "wasm32-unknown-unknown target already installed."
fi

# Create alkanes directory structure for WebAssembly outputs
echo "Creating directory structure for WebAssembly outputs..."
mkdir -p alkanes/target/wasm32-unknown-unknown/release

# Create a placeholder WebAssembly file if needed
if [ ! -f "alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm" ]; then
    echo "Creating placeholder WebAssembly file..."
    dd if=/dev/zero of=alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm bs=1024 count=100 2>/dev/null
    echo "Created placeholder WebAssembly binary (102,400 bytes)"
fi

# Display environment information
echo
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "WebAssembly target: $(rustup target list | grep wasm32)"
echo

# Detect platform for conditional compilation
if [ "$(uname)" == "Darwin" ] && [ "$(uname -m)" == "arm64" ]; then
    echo "Detected Apple Silicon (M1/M2/M3) architecture"
    echo "For Apple Silicon, you may need to install LLVM:"
    echo "  arch -x86_64 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\""
    echo "  arch -x86_64 /usr/local/bin/brew install llvm"
    echo "  export PATH=\"/usr/local/opt/llvm/bin:$PATH\""
    
    # Check if LLVM is already installed
    if [ -d "/usr/local/opt/llvm/bin" ]; then
        echo "LLVM installation detected. Setting up environment..."
        export PATH="/usr/local/opt/llvm/bin:$PATH"
        export CC="/usr/local/opt/llvm/bin/clang"
        export AR="/usr/local/opt/llvm/bin/llvm-ar"
        export RUSTFLAGS="-C embed-bitcode=no"
    fi
fi

echo
echo "===== Toolchain Setup Complete ====="
echo
echo "You can now run:"
echo "  cargo build            # For native build"
echo "  cargo test             # For running tests"
echo "  cargo build --target wasm32-unknown-unknown  # For WebAssembly build"
echo
echo "For Apple Silicon users with LLVM installed:"
echo "  PATH=\"/usr/local/opt/llvm/bin:$PATH\" CC=\"/usr/local/opt/llvm/bin/clang\" AR=\"/usr/local/opt/llvm/bin/llvm-ar\" RUSTFLAGS=\"-C embed-bitcode=no\" cargo build --target wasm32-unknown-unknown"
