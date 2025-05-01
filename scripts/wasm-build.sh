#!/bin/bash
# WebAssembly build script for YieldVault

# Ensure we exit on any errors
set -e

# Display build start message
echo "Building YieldVault for WebAssembly..."

# Build for WebAssembly target
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release

# Verify the build succeeded
if [ ! -f "target/wasm32-unknown-unknown/release/yield_vault.wasm" ]; then
    echo "Error: WebAssembly build failed!"
    exit 1
fi

# Optimize the WebAssembly binary if wasm-opt is available
if command -v wasm-opt > /dev/null; then
    echo "Optimizing WebAssembly binary with wasm-opt..."
    wasm-opt -Oz -o yield_vault_opt.wasm target/wasm32-unknown-unknown/release/yield_vault.wasm
    cp yield_vault_opt.wasm yield_vault.wasm
else
    echo "wasm-opt not found, skipping optimization step..."
    cp target/wasm32-unknown-unknown/release/yield_vault.wasm yield_vault.wasm
fi

# Display final size information
echo "Final WebAssembly binary size: $(stat -f%z yield_vault.wasm) bytes"
echo "Build complete! Output: yield_vault.wasm"
