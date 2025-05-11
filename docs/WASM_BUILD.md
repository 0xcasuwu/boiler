# WebAssembly Build Guide for yield-vault

This document explains the WebAssembly build process for the yield-vault project, focusing on the technical aspects without any HTML or frontend dependencies.

## Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target
- wasm-pack (for testing and packaging)
- wasm-opt from Binaryen (optional, for size optimization)

## Installing Prerequisites

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install Binaryen for wasm-opt (optional)
npm install -g binaryen
```

## Building for WebAssembly

The project is configured to build both a standard Rust library (`rlib`) and a WebAssembly module (`cdylib`). The WebAssembly target includes special optimizations for size and load time.

### Standard Build

```bash
# Debug build
cargo build --target wasm32-unknown-unknown

# Release build
cargo build --target wasm32-unknown-unknown --release
```

### Optimized Build

For the smallest possible WebAssembly binary, use the wasm-opt script:

```bash
./scripts/wasm_opt.sh
```

This script:
1. Builds the project with the `wasm-release` profile
2. Applies `wasm-opt -Oz` optimization if available
3. Outputs to `target/wasm-opt/yield_vault.wasm`

## Running Tests

To run tests in a WebAssembly environment:

```bash
./scripts/wasm_test.sh
```

For native testing of components that don't rely on WebAssembly specifics:

```bash
cargo test --lib
```

## WebAssembly Binary Integration

The yield-vault WebAssembly binary exposes the following key functions:

### Main Call Function

```rust
#[wasm_bindgen]
pub fn call(opcode: u32, args: &[u8]) -> Vec<u8>
```

This function takes an operation code and binary arguments, dispatches to the appropriate handler, and returns binary data.

### Key Opcodes

- `0`: Initialize(name, symbol, asset_name, asset_symbol, decimal_offset)
- `10-19`: Asset Management Operations (deposit, mint, withdraw, redeem)
- `100-199`: Metadata View Functions (name, symbol, decimals, asset)
- `200-299`: Accounting View Functions (total assets, conversion)
- `300-399`: Limit View Functions (max deposit/mint/withdraw/redeem)
- `400-499`: Preview View Functions (preview operations)

## Bitcoin Integration Notes

For integration with Bitcoin:

1. The WebAssembly binary is designed to be loaded by a Bitcoin-compatible WebAssembly VM
2. All state management happens through the Alkanes runtime
3. Arguments are serialized as byte arrays with simple binary encoding
4. Results are returned as byte arrays with simple binary encoding

## Size Optimization Tips

- Keep feature flags minimal
- Use the `wasm-release` profile which sets `opt-level = 'z'`
- Apply wasm-opt with `-Oz` flag for maximum size savings
- Use conditional compilation with `#[cfg(target_arch = "wasm32")]` to exclude code not needed in WebAssembly

## Known WebAssembly Constraints

- WebAssembly has 32-bit memory addressing
- Linear memory model requires careful memory management
- No direct access to the filesystem or network
- All external interactions must be handled through imports
