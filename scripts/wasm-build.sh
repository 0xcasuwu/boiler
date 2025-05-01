#!/bin/bash
set -e

# Install wasm-bindgen-cli with the correct version if needed
echo "Installing wasm-bindgen-cli version 0.2.100..."
cargo install -f wasm-bindgen-cli --version 0.2.100

# Clean target directory to avoid any potential issues
echo "Cleaning target directory..."
cargo clean --target wasm32-unknown-unknown

# Build the project for wasm32-unknown-unknown target WITHOUT bitcoin feature
echo "Building yield-vault for WebAssembly..."
RUSTFLAGS="-C link-arg=-s" cargo build \
    --target wasm32-unknown-unknown \
    --no-default-features \
    --release

# Run wasm-bindgen to generate JavaScript bindings
echo "Generating JavaScript bindings..."
wasm-bindgen --out-dir pkg --target web \
    target/wasm32-unknown-unknown/release/yield_vault.wasm

# Optimize the WebAssembly binary if wasm-opt is available
if command -v wasm-opt > /dev/null; then
    echo "Optimizing WebAssembly binary with wasm-opt..."
    wasm-opt -Oz -o pkg/yield_vault_opt.wasm pkg/yield_vault_bg.wasm
    cp pkg/yield_vault_opt.wasm pkg/yield_vault_bg.wasm
fi

# Display final size information
echo "Final WebAssembly binary size: $(stat -f%z pkg/yield_vault_bg.wasm) bytes"
echo "Build complete! Output in pkg directory"
