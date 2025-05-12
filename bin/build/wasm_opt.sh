#!/bin/bash
# WebAssembly optimization script for yield-vault
# This script builds and optimizes the WebAssembly output

set -e  # Exit on any error

echo "===== WebAssembly Build & Optimization for yield-vault ====="

# Check for wasm-pack installation
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found, installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check for wasm-opt (from binaryen)
if ! command -v wasm-opt &> /dev/null; then
    echo "WARNING: wasm-opt not found. For best size optimization, install binaryen:"
    echo "npm install -g binaryen"
    HAS_WASM_OPT=0
else
    HAS_WASM_OPT=1
fi

# Display versions
echo "Using wasm-pack version: $(wasm-pack --version)"
if [ $HAS_WASM_OPT -eq 1 ]; then
    echo "Using wasm-opt from binaryen"
fi
echo

# Clean any previous builds
echo "Cleaning previous WebAssembly builds..."
rm -rf target/wasm32-unknown-unknown/release || true
rm -rf target/wasm-opt || true

# Create output directory
mkdir -p target/wasm-opt

# Build WebAssembly in release mode
echo "Building WebAssembly in release mode..."
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
    cargo build --target wasm32-unknown-unknown --release

# Get the output file path
WASM_FILE="target/wasm32-unknown-unknown/release/yield_vault.wasm"

# Check file size before optimization
BEFORE_SIZE=$(ls -la "$WASM_FILE" | awk '{print $5}')
echo "WebAssembly size before optimization: $BEFORE_SIZE bytes"

# Run wasm-opt if available
if [ $HAS_WASM_OPT -eq 1 ]; then
    echo "Running wasm-opt for size optimization..."
    wasm-opt -Oz "$WASM_FILE" -o "target/wasm-opt/yield_vault.wasm"
    
    # Check file size after optimization
    AFTER_SIZE=$(ls -la "target/wasm-opt/yield_vault.wasm" | awk '{print $5}')
    echo "WebAssembly size after optimization: $AFTER_SIZE bytes"
    echo "Size reduction: $(( BEFORE_SIZE - AFTER_SIZE )) bytes ($(( (BEFORE_SIZE - AFTER_SIZE) * 100 / BEFORE_SIZE ))%)"
else
    # Just copy the file if wasm-opt is not available
    cp "$WASM_FILE" "target/wasm-opt/yield_vault.wasm"
    echo "WebAssembly optimization skipped (wasm-opt not installed)"
fi

echo
echo "Final WebAssembly file: target/wasm-opt/yield_vault.wasm"
echo
echo "===== WebAssembly Build & Optimization Completed ====="
