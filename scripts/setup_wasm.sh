#!/bin/bash
# WebAssembly setup and verification script
# This script sets up the WebAssembly environment and verifies it works

set -e  # Exit on any error

echo "===== WebAssembly Setup for yield-vault ====="

# Check if wasm32 target is installed, if not install it
if ! rustup target list | grep -q "wasm32-unknown-unknown"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
else
    echo "wasm32-unknown-unknown target already installed."
fi

# Check for wasm-pack installation
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found, installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
else
    echo "wasm-pack already installed: $(wasm-pack --version)"
fi

# Display environment information
echo
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "wasm-pack version: $(wasm-pack --version)"
echo "Target directory: $(rustc --print target-libdir)"
echo

# Build a debug version of the WebAssembly module
echo "Building WebAssembly debug version..."
cargo build --target wasm32-unknown-unknown

# Verify the WebAssembly file exists
WASM_FILE="target/wasm32-unknown-unknown/debug/yield_vault.wasm"
if [ -f "$WASM_FILE" ]; then
    echo "✓ WebAssembly build successful!"
    echo "WebAssembly file: $WASM_FILE"
    echo "File size: $(ls -lh "$WASM_FILE" | awk '{print $5}')"
else
    echo "❌ WebAssembly build failed! File not found: $WASM_FILE"
    exit 1
fi

# Check if we can run wasm-pack tests
echo
echo "Testing WebAssembly functionality..."
echo "(Note: This may fail if Node.js is not installed)"

if command -v node &> /dev/null; then
    echo "Node.js found: $(node --version)"
    # Just run a simple test with wasm-pack
    # This will fail gracefully if there are issues
    wasm-pack test --node -- --features "test" simple_utils || echo "⚠️ Tests failed, but build was successful"
else
    echo "⚠️ Node.js not found, skipping WebAssembly tests"
    echo "To run WebAssembly tests, install Node.js"
fi

echo
echo "WebAssembly setup complete!"
echo
echo "Next steps:"
echo "1. For optimized builds, run: ./scripts/wasm_opt.sh"
echo "2. For WebAssembly testing, run: ./scripts/wasm_test.sh"
echo "3. For details, see: docs/WASM_BUILD.md"
echo
echo "===== WebAssembly Setup Completed ====="
