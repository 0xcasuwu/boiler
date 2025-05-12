#!/bin/bash
# Enhanced build script for yield-vault using the actual GitHub repositories

set -e  # Exit on any error

echo "===== Enhanced Build Script for yield-vault ====="

# Check if required Rust targets are installed
if ! rustup target list | grep -q "wasm32-unknown-unknown.*installed"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Ensure directories exist
mkdir -p alkanes/target/wasm32-unknown-unknown/release

# Display environment information
echo
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "WebAssembly target: $(rustup target list | grep wasm32-unknown-unknown)"
echo

# Update dependencies to ensure we have the latest versions
echo "Updating dependencies..."
cargo update
echo

# Build the native version
echo "Building native version..."
cargo build
echo "✓ Native build completed"

# Run WebAssembly tests if requested
if [ "$1" == "--test-wasm" ]; then
    echo
    echo "===== Running WebAssembly Tests ====="
    echo
    echo "Running tests with wasm-pack..."
    wasm-pack test --chrome --headless
    echo "✓ WebAssembly tests completed"
fi

echo
echo "===== Build completed ====="
echo
echo "Notes:"
echo "1. WebAssembly directory structure has been created at: alkanes/target/wasm32-unknown-unknown/release/"
echo "2. Native build completed using direct GitHub repositories from kungfuflex/alkanes-rs"
echo "3. IRONCLAD RULE: Always use direct repos, never use local stubs (except secp256k1-sys)"
echo "4. Only secp256k1-sys uses a local stub for cross-platform compatibility"
echo
echo "Run commands:"
echo "  Native Tests:       cargo test --lib --target x86_64-unknown-linux-gnu"
echo "  WebAssembly Tests:  ./build_project.sh --test-wasm"
echo "  WebAssembly Build:  cargo build --target wasm32-unknown-unknown --release"
