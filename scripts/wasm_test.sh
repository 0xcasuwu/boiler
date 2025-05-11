#!/bin/bash
# WebAssembly test script for yield-vault
# This script focuses specifically on running tests in a WebAssembly environment

set -e  # Exit on any error

echo "===== WebAssembly Testing for yield-vault ====="

# Check for wasm-pack installation
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found, installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check for node.js
if ! command -v node &> /dev/null; then
    echo "ERROR: Node.js is required for WebAssembly testing"
    echo "Please install Node.js to continue"
    exit 1
fi

# Display versions
echo "Using wasm-pack version: $(wasm-pack --version)"
echo "Using node version: $(node --version)"
echo

# Clean any previous builds
echo "Cleaning previous WebAssembly builds..."
rm -rf target/wasm32-unknown-unknown/debug/deps/*.wasm || true
rm -rf target/wasm32-unknown-unknown/debug/*.wasm || true

# Build WebAssembly debug version
echo "Building WebAssembly debug version..."
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals" \
    cargo build --target wasm32-unknown-unknown --features "test"

# Run WebAssembly tests using wasm-pack
echo "Running WebAssembly tests with wasm-pack..."
wasm-pack test --node -- --features "test"

# Run our specific wasm tests
echo "Running specific WebAssembly tests from src/tests/wasm_tests.rs..."
RUSTFLAGS="--cfg=test" wasm-pack test --node -- --features "test" wasm_tests

echo
echo "===== WebAssembly Testing Completed ====="
