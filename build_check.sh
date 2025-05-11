#!/bin/bash
$HOME/.cargo/bin/cargo build
echo "Build exit code: $?"

# Check for WebAssembly output
echo "Looking for WebAssembly files:"
find ./target -name "*.wasm" | grep -v target/debug || echo "No WebAssembly files found"
