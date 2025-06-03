# Alkanes Build Process Guide

This guide documents the complete process for building and compiling alkane contracts using the alkanes-build tool.

## Overview

The alkanes-build process converts WASM modules into Rust source code that can be embedded in alkane projects. This guide specifically covers building the free-mint alkane contract.

## Prerequisites

1. **Rust toolchain** with support for:
   - `wasm32-unknown-unknown` target
   - `aarch64-unknown-linux-gnu` target (or appropriate native target)
2. **Repository structure**:
   - `/home/e/Documents/alkanes-rs` - Main alkanes repository
   - `/home/e/Documents/free-mint` - Free-mint alkane contract
   - `/home/e/Documents/boiler` - Target project for compiled output

## Step-by-Step Process

### Step 1: Build the alkanes-build Binary

The alkanes-build tool needs to be compiled first before it can be used.

```bash
# Navigate to the alkanes-rs repository
cd /home/e/Documents/alkanes-rs

# Build the alkanes-build binary for the native target
cargo build --release --package alkanes-build --target aarch64-unknown-linux-gnu
```

**Expected Output**: 
- Binary created at: `/home/e/Documents/alkanes-rs/target/aarch64-unknown-linux-gnu/release/alkanes-build`
- File size: ~1MB executable

**Note**: Replace `aarch64-unknown-linux-gnu` with your system's native target if different (e.g., `x86_64-unknown-linux-gnu` for x86_64 systems).

### Step 2: Build the Free-Mint WASM Module

The alkane contract needs to be compiled to WASM before it can be processed by alkanes-build.

```bash
# Navigate to the free-mint repository
cd /home/e/Documents/free-mint

# Build the WASM module in release mode
cargo build --release --target wasm32-unknown-unknown
```

**Expected Output**:
- WASM file created at: `/home/e/Documents/free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm`
- File size: ~300KB
- May show warnings (these are typically safe to ignore)

### Step 3: Convert WASM to Rust Source

Use the alkanes-build tool to convert the WASM module into embeddable Rust code.

```bash
# Run alkanes-build with full paths
/home/e/Documents/alkanes-rs/target/aarch64-unknown-linux-gnu/release/alkanes-build \
  --input /home/e/Documents/free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm \
  --output /home/e/Documents/boiler/src/precompiled/free_mint_build.rs
```

**Expected Output**:
- Success message: "Successfully converted [input] to [output]"
- Generated file: `/home/e/Documents/boiler/src/precompiled/free_mint_build.rs`
- File size: ~600KB of Rust source code

## Project Structure

After completion, your project structure should look like:

```
/home/e/Documents/
├── alkanes-rs/
│   ├── crates/alkanes-build/          # alkanes-build source
│   └── target/aarch64-unknown-linux-gnu/release/
│       └── alkanes-build              # Built binary
├── free-mint/
│   ├── src/lib.rs                     # Contract source
│   ├── Cargo.toml                     # Project config
│   └── target/wasm32-unknown-unknown/release/
│       └── free_mint.wasm             # Compiled WASM
└── boiler/
    └── src/precompiled/
        └── free_mint_build.rs         # Generated Rust code
```

## Troubleshooting

### Common Issues

1. **"alkanes-build not found"**
   - Ensure the binary was built successfully in Step 1
   - Check the target architecture matches your system
   - Verify the file exists and has execute permissions

2. **"No such file or directory" when running alkanes-build**
   - Ensure the input WASM file exists (complete Step 2 first)
   - Check all paths are absolute and correct
   - Verify the output directory exists

3. **WASM build fails**
   - Ensure `wasm32-unknown-unknown` target is installed: `rustup target add wasm32-unknown-unknown`
   - Check that all dependencies in Cargo.toml are accessible
   - Verify the project compiles for native target first

4. **Target architecture issues** 
   - Check your system architecture: `uname -m`
   - Use appropriate target (aarch64 for ARM64, x86_64 for Intel/AMD64)
   - Install missing targets: `rustup target add <target-name>`

### Verification Commands

```bash
# Verify alkanes-build binary exists and is executable
ls -la /home/e/Documents/alkanes-rs/target/aarch64-unknown-linux-gnu/release/alkanes-build

# Verify WASM file was created
ls -la /home/e/Documents/free-mint/target/wasm32-unknown-unknown/release/free_mint.wasm

# Verify final output file
ls -la /home/e/Documents/boiler/src/precompiled/free_mint_build.rs
```

## What the Generated Code Contains

The `free_mint_build.rs` file contains:
- Embedded WASM bytecode as a Rust byte array
- Helper functions for loading and executing the contract
- Metadata about the original WASM module
- Integration points for the alkanes runtime

This allows the free-mint contract to be compiled directly into your Rust project without runtime WASM loading.

## Next Steps

After generating the `free_mint_build.rs` file:
1. Ensure it's properly included in your project's module system
2. Add any necessary imports or integration code
3. Test the embedded contract functionality
4. Build your main project to verify everything compiles correctly

## Notes

- The process may take several minutes depending on system performance
- Generated files are quite large (~600KB) due to embedded bytecode
- This process needs to be repeated whenever the source alkane contract changes
- Consider version control for the generated files if they don't change frequently
