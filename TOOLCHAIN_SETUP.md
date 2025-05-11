# Yield Vault Project Toolchain Setup

This document describes the configuration of the Rust toolchain for building and testing the yield-vault project.

## IRONCLAD RULE

**ALWAYS use direct GitHub repositories from kungfuflex/alkanes-rs, NEVER use local stubs** (except for secp256k1-sys, which needs a stub for cross-platform compatibility).

## Toolchain Configuration Steps

We have configured the toolchain to handle both native and WebAssembly builds:

1. **Rust Toolchain**:
   - Confirmed Rust 1.86.0 is installed
   - WebAssembly target (wasm32-unknown-unknown) is properly configured

2. **Direct GitHub Dependencies**:
   - **IRONCLAD RULE**: Direct integration with primary repositories:
     - `alkanes-runtime`: From `https://github.com/kungfuflex/alkanes-rs`
     - `alkanes-support`: From `https://github.com/kungfuflex/alkanes-rs` 
     - `metashrew-support`: From `https://github.com/sandshrewmetaprotocols/metashrew`
   - Single exception for compatibility:
     - `secp256k1-sys`: Local stub for cross-platform compatibility with Apple Silicon/WebAssembly

3. **Cargo Configuration**:
   - Modified `Cargo.toml` to use direct GitHub repositories for all core dependencies
   - Used local stub via the `[patch]` section **only** for secp256k1-sys
   - This configuration ensures maximum functionality and consistency with the actual codebase

4. **WebAssembly Support**:
   - Set up proper directory structure in `alkanes/target/wasm32-unknown-unknown/release/`
   - Configured for WebAssembly compilation and testing with wasm-pack

## Usage

To build the project, use the included `build_project.sh` script:

```bash
chmod +x build_project.sh
./build_project.sh
```

This script will:
1. Ensure the wasm32-unknown-unknown target is installed
2. Create necessary directories
3. Update dependencies to the latest versions from GitHub repositories
4. Build the native code

### Native Builds

```bash
cargo build
```

### WebAssembly Builds

```bash
cargo build --target wasm32-unknown-unknown
```

### Running Tests

For native tests:
```bash
cargo test --lib --target x86_64-unknown-linux-gnu
```

For WebAssembly tests (tests tagged with wasm_bindgen_test):
```bash
./build_project.sh --test-wasm
```

This command uses wasm-pack to run tests in a headless Chrome browser, ensuring that all WebAssembly-specific functionality is properly tested.

## Dependencies Overview

### 1. alkanes-rs Repository (https://github.com/kungfuflex/alkanes-rs)
- **alkanes-runtime**: Core runtime functionality
  - Storage pointers implementation
  - Message dispatch system
  - Runtime components

- **alkanes-support**: Support utilities
  - ID management
  - Transfer parcels
  - Overflow checking
  - Response handling

### 2. metashrew Repository (https://github.com/sandshrewmetaprotocols/metashrew)
- **metashrew-support**: Key-value functionality
  - Index pointers for key-value storage

### 3. secp256k1-sys (Local Stub)
- **EXCEPTION TO THE RULE**: This is the only component using a local stub
- Provides cryptographic primitives stub for cross-platform compatibility
- Critical for WebAssembly builds on different architectures including Apple Silicon

## WebAssembly Testing Notes

1. **Test Tags**:
   - Tests intended for WebAssembly execution are tagged with `#[wasm_bindgen_test]`
   - These tests are located in various files throughout the src/tests/ directory

2. **Runtime Environment**:
   - Tests run in a headless Chrome instance through wasm-pack
   - This provides a realistic JavaScript+WebAssembly environment for testing

3. **Test Configuration**:
   - Tests are configured with `wasm_bindgen_test_configure!(run_in_browser);`
   - This ensures they run in a browser environment rather than Node.js

## Platform-specific Considerations

### Apple Silicon (M1/M2/M3) Support

For building on Apple Silicon:

```bash
PATH="/usr/local/opt/llvm/bin:$PATH" CC="/usr/local/opt/llvm/bin/clang" AR="/usr/local/opt/llvm/bin/llvm-ar" RUSTFLAGS="-C embed-bitcode=no" cargo build --target wasm32-unknown-unknown --release
```

See the Apple Silicon build guide in memory-bank/apple-silicon-build-guide.md for full details.
