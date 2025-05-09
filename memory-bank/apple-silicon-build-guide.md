# Building WebAssembly Contracts on Apple Silicon

This guide documents the process of building the yield-vault WebAssembly contract for Bitcoin smart contracts on Apple Silicon (M1/M2/M3) Macs, including the challenges and solutions.

## Overview

Building WebAssembly contracts for Bitcoin on Apple Silicon presents unique challenges due to:

1. Architecture-specific compilation requirements
2. Dependency compatibility issues, especially with `secp256k1-sys`
3. Cross-compilation complexities for WebAssembly target

This guide provides solutions to these challenges.

## Prerequisites

Before attempting to build, ensure you have:

- Rust and Cargo installed
- WebAssembly target added: `rustup target add wasm32-unknown-unknown`
- LLVM installed via Homebrew (for Apple Silicon compatibility):
  ```bash
  arch -x86_64 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
  arch -x86_64 /usr/local/bin/brew install llvm
  ```

## Build Environment Setup

The Apple Silicon build environment requires specific configurations:

```bash
# Environment variables for Apple Silicon builds
export PATH="/usr/local/opt/llvm/bin:$PATH"
export CC="/usr/local/opt/llvm/bin/clang"
export AR="/usr/local/opt/llvm/bin/llvm-ar" 
export RUSTFLAGS="-C embed-bitcode=no"
```

## Common Build Issues

### 1. secp256k1-sys Dependency Issue

The primary challenge is with the `secp256k1-sys` dependency, which has multiple issues:

- Dependency on a non-existent repository (`https://github.com/alkimake/secp256k1-sys`)
- Conflicting dependency specifications in the dependency tree
- Compilation issues specific to Apple Silicon

### 2. Lock File Ambiguity

The Cargo.lock file may contain ambiguous references to `secp256k1-sys` with both git and path specifications, causing errors like:

```
dependency (secp256k1-sys) specification is ambiguous. Only one of `git` or `path` is allowed.
```

### 3. WebAssembly Target Compatibility

The WebAssembly target requires specific compiler flags and configurations to build correctly on Apple Silicon.

## Current Workaround

After multiple attempts to resolve the dependency issues, a placeholder approach has been implemented:

1. Create a placeholder WebAssembly file in the expected location
2. Generate stub test module files to support development

This placeholder enables development to continue while the underlying dependency issues are addressed at a later time.

## Build Scripts

Several build scripts have been created to address these challenges:

### 1. check_mac_m1.sh

Detects Apple Silicon architecture and verifies LLVM installation.

### 2. build_apple_silicon.sh

Attempts to build using Apple Silicon-specific settings with LLVM.

### 3. analyze_and_fix_dependencies.sh

Creates a stub implementation of `secp256k1-sys` and modifies cargo configuration to use it.

### 4. create_wasm_placeholder.sh

Creates a placeholder WebAssembly binary when full compilation isn't possible.

## Building with Placeholder

To use the placeholder approach:

```bash
# Run the placeholder creation script
chmod +x create_wasm_placeholder.sh
./create_wasm_placeholder.sh
```

This creates:

1. A minimal valid WebAssembly module at `alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm`
2. Test module files in `src/tests/std/`

## Future Improvements

For future development, consider:

1. Forking and maintaining a compatible version of `secp256k1-sys` specifically for WebAssembly on Apple Silicon
2. Creating a custom build script that handles architecture-specific compilation
3. Implementing a conditional compilation approach that avoids problematic dependencies when targeting WebAssembly

## Resources

- [Rust WebAssembly Documentation](https://rustwasm.github.io/docs/book/)
- [secp256k1-sys Repository](https://github.com/rust-bitcoin/rust-secp256k1)
- [Apple Silicon Rust Guide](https://github.com/messense/homebrew-macos-cross-toolchains)
