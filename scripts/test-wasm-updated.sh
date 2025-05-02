#!/bin/bash
set -e

# Install wasm-bindgen-cli with the correct version
echo "Installing wasm-bindgen-cli version 0.2.100..."
cargo install -f wasm-bindgen-cli --version 0.2.100

# Build the project for wasm32-unknown-unknown target
echo "Building for WebAssembly..."
cargo build --target wasm32-unknown-unknown

# Run wasm-bindgen to generate JavaScript bindings
echo "Generating JavaScript bindings..."
wasm-bindgen --out-dir pkg --target web target/wasm32-unknown-unknown/debug/yield_vault.wasm

# Run the tests
echo "Running tests with wasm-pack..."
wasm-pack test --node

# If you need to run tests individually to prevent segmentation faults, uncomment these:
# echo "Running basic tests individually..."
# echo "-----------------------------------"
#
# echo "Running test_can_create_vault..."
# RUST_BACKTRACE=1 wasm-pack test --node -- --test test_can_create_vault
#
# echo "Running test_initialization..."
# RUST_BACKTRACE=1 wasm-pack test --node -- --test test_initialization
#
# echo "Running test_initialization_guard..."
# RUST_BACKTRACE=1 wasm-pack test --node -- --test test_initialization_guard
#
# echo "Running test_balance_operations..."
# RUST_BACKTRACE=1 wasm-pack test --node -- --test test_balance_operations

echo "WebAssembly tests completed successfully!"
