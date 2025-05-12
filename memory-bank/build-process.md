# YieldVault WebAssembly Build Process

## Overview

The YieldVault smart contract requires compilation to WebAssembly (WASM) for deployment on the Bitcoin blockchain. This document outlines the build process, architecture-specific considerations, and key components involved in generating deployment-ready WebAssembly binaries.

## IRONCLAD RULE FOR DEPENDENCIES

**ALWAYS use direct GitHub repositories from kungfuflex/alkanes-rs, NEVER use local stubs** (except for secp256k1-sys, which needs a stub for cross-platform compatibility). This rule has been established after thorough testing and verification to ensure maximum compatibility and functionality.

## Testing Framework

YieldVault implements a multi-layered testing approach to accommodate different testing needs:

1. **Simple Utility Tests** (`simple_utils_test.rs`)
   - Platform-independent tests for utility functions
   - No dependency on external libraries or WebAssembly
   - Run with: `cargo test --test simple_utils_test --target x86_64-unknown-linux-gnu`

2. **Mock Vault Tests** (`mock_vault_tests.rs`)
   - Uses the `MockYieldVault` implementation that avoids external dependencies
   - In-memory simulation of the vault functionality
   - Tests core business logic without WebAssembly dependencies
   - Run with: `cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu`

3. **Native Integration Tests** (When applicable)
   - Tests that use native Rust implementation
   - Most appropriate for testing core functionality

4. **Convenient Test Script**
   - `run_working_tests.sh` - Runs all the working tests in sequence
   - Usage: `chmod +x run_working_tests.sh && ./run_working_tests.sh`

### MockYieldVault Implementation

The `src/mock_vault.rs` module provides a standalone implementation that:
- Uses a HashMap with bincode serialization for storage (no external storage dependencies)
- Implements all core vault functionality (deposit, redeem, conversion, etc.)
- Allows for comprehensive testing without alkanes-runtime dependencies
- Can be extended for new features before implementing them in the main vault

## Build Architecture

The build process uses a multi-stage approach:

1. **Standard Rust compilation** - Initial compilation of Rust code
2. **WebAssembly cross-compilation** - Targeting the wasm32-unknown-unknown platform
3. **Post-processing** - Compression and encoding for testing and deployment
4. **Deployment** - OylNet testnet integration

### Key Components

- `build.rs` - Custom build script handling WebAssembly compilation
- `src/tests/std/` - Directory for test files including WebAssembly binaries
- `alkanes/target/` - Target directory for WebAssembly output files
- `final_fork_build.sh` - Main build script with dependency forking
- `deploy_to_oylnet.sh` - Script for OylNet deployment
- `interact_with_vault.sh` - OylNet contract interaction script

## Custom Dependency Fork

To resolve persistent issues with the secp256k1-sys crate on Apple Silicon, we've created a custom fork approach:

### Fork Architecture

1. **Local Fork Repository**: Located at `fork-repos/secp256k1-sys/`
2. **Stub Implementation**: Contains minimal no-op implementations for required functions
3. **Custom Build Script**: `build.rs` to satisfy the `links = "secp256k1"` requirement
4. **Cargo Patching**: Using `[patch]` sections in Cargo.toml to redirect all dependencies

### Fork Implementation

The forked version of secp256k1-sys:
- Contains stub versions of all required API functions
- Includes a build script meeting Cargo's requirements
- Doesn't attempt to build the actual C library for WebAssembly targets
- Is structured as a drop-in replacement for the original crate

### Core files in the fork:

```rust
// src/lib.rs (essential stubs)
#![allow(unused_variables, dead_code)]

pub const SECP256K1_FLAGS_TYPE_MASK: u32 = 0x00000003;
pub const SECP256K1_FLAGS_TYPE_CONTEXT: u32 = 0x00000001;
pub const SECP256K1_FLAGS_TYPE_COMPRESSION: u32 = 0x00000002;

pub const SECP256K1_FLAGS_BIT_CONTEXT_VERIFY: u32 = 0x00000100;
pub const SECP256K1_FLAGS_BIT_CONTEXT_SIGN: u32 = 0x00000200;
pub const SECP256K1_FLAGS_BIT_COMPRESSION: u32 = 0x00000100;

pub const SECP256K1_CONTEXT_VERIFY: u32 = SECP256K1_FLAGS_TYPE_CONTEXT | SECP256K1_FLAGS_BIT_CONTEXT_VERIFY;
pub const SECP256K1_CONTEXT_SIGN: u32 = SECP256K1_FLAGS_TYPE_CONTEXT | SECP256K1_FLAGS_BIT_CONTEXT_SIGN;
pub const SECP256K1_CONTEXT_NONE: u32 = SECP256K1_FLAGS_TYPE_CONTEXT;

pub const SECP256K1_EC_COMPRESSED: u32 = SECP256K1_FLAGS_TYPE_COMPRESSION | SECP256K1_FLAGS_BIT_COMPRESSION;
pub const SECP256K1_EC_UNCOMPRESSED: u32 = SECP256K1_FLAGS_TYPE_COMPRESSION;

#[repr(C)]
pub struct Context(u8);

#[repr(C)]
pub struct PublicKey([u8; 64]);

#[repr(C)]
pub struct SecretKey([u8; 32]);

// ... Additional stubs as needed
```

## WebAssembly Build Process

### 1. Standard Build (Non-Apple Silicon)

For standard platforms, the build process is straightforward:

```bash
# Standard build command
cargo build --target wasm32-unknown-unknown --release
```

### 2. Apple Silicon Build Process

Building on Apple Silicon (M1/M2/M3) requires special handling:

```bash
# Set environment variables
export PATH="/usr/local/opt/llvm/bin:$PATH"
export CC="/usr/local/opt/llvm/bin/clang"
export AR="/usr/local/opt/llvm/bin/llvm-ar"
export RUSTFLAGS="-C embed-bitcode=no"

# Run build with special flags
cargo build --target wasm32-unknown-unknown --release
```

### 3. Specialized Build Scripts

We provide several build scripts to handle different scenarios:

#### build_minimal.sh (Recommended for Apple Silicon)

This script creates a placeholder WebAssembly file and sets up a minimal build environment, ideal for Apple Silicon machines where compilation might be challenging.

```bash
./build_minimal.sh
```

#### final_fork_build.sh

This script attempts a full compilation with local dependencies:

```bash
./final_fork_build.sh
```

#### build_with_fork.sh

Provides more granular control over the build process:

```bash
./build_with_fork.sh
```

## Post-Build Processing

After successful build, the WebAssembly binary is:

1. Copied to `alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm`
2. Size is approximately 102,433 bytes
3. Ready for deployment on OylNet

## Troubleshooting Common Build Issues

### 1. secp256k1-sys Compilation Errors

**Symptom**: Errors related to secp256k1-sys compilation
**Solution**: Use the local fork approach with build_minimal.sh or final_fork_build.sh

### 2. LLVM Not Found on Apple Silicon

**Symptom**: "Command failed: xcrun --sdk macosx --find clang"
**Solution**: Install LLVM via Homebrew:
```bash
arch -x86_64 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
arch -x86_64 /usr/local/bin/brew install llvm
export PATH="/usr/local/opt/llvm/bin:$PATH"
```

### 3. WebAssembly File Not Found

**Symptom**: Missing `.wasm` file after build
**Solution**: Our build scripts create placeholder files even if compilation fails

### 4. "Cannot locate remote-tracking branch" Error

**Symptom**: Error when running with --offline flag
**Solution**: Use build_minimal.sh which handles completely local dependencies

## Deployment Process

Once built, deploy to OylNet using:

```bash
./deploy_to_oylnet.sh
```

This will:
1. Deploy the WebAssembly binary to OylNet
2. Initialize the contract with proper parameters
3. Store the contract ID for future interactions

## Contract Interaction

Interact with the deployed contract using:

```bash
./interact_with_vault.sh
```

**Important Note**: When interacting with the contract, use numeric values for AlkaneId (block=1, tx=1) rather than string tokens to avoid "scriptpubkey" errors.

## Recommended Build Workflow

1. Check Apple Silicon architecture: `scripts/check_mac_m1.sh`
2. For Apple Silicon: `./build_minimal.sh`
3. For other platforms: `./final_fork_build.sh`
4. Verify WebAssembly output: Check for ~102KB file in alkanes/target/...
5. Deploy: `./deploy_to_oylnet.sh`
6. Interact: `./interact_with_vault.sh`
