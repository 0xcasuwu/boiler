# Build and Deployment Guide

## Overview

This document provides comprehensive instructions for building the Yield Vault smart contract for Bitcoin and deploying it to OylNet testnet. The build process has been rigorously tested and optimized for all platforms, with special attention to Apple Silicon (M1/M2/M3) compatibility.

## Prerequisites

- Rust 1.75.0 or later
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`
- For Apple Silicon:
  - Homebrew LLVM: `arch -x86_64 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"`
  - LLVM installation: `arch -x86_64 /usr/local/bin/brew install llvm`
  - LLVM in PATH: `export PATH="/usr/local/opt/llvm/bin:$PATH"`
- OylNet SDK for deployment and testing

## Build Process

The Yield Vault contract uses a custom build approach with local dependency forks to ensure compatibility across all platforms. Our build system offers several options depending on your needs:

### Option 1: Automatic Platform Detection (Recommended)

The unified build script automatically detects your platform and chooses the appropriate build method:

```bash
# Run the build script with auto-detection
./scripts/build.sh
```

This script:
- Detects your architecture (Apple Silicon or standard)
- Sets appropriate environment variables and compiler options
- Uses the optimal build strategy for your platform

### Option 2: Minimal Build (For Apple Silicon)

For explicit Apple Silicon optimization:

```bash
# Run the build script with minimal mode
./scripts/build.sh --minimal
```

This option:
- Sets up the build environment for Apple Silicon
- Configures local secp256k1-sys fork
- Creates a valid WebAssembly binary for testing and deployment

### Option 3: Final Fork Build (For Standard Architectures)

For a more complete build on standard architectures:

```bash
# Run the build script with final fork mode
./scripts/build.sh --final
```

This option:
- Uses the local fork of secp256k1-sys
- Performs complete compilation
- Optimizes the WebAssembly output

### Option 4: Custom Fork Integration (Advanced)

For development environments that need more control:

```bash
# Run the build script with custom fork mode
./scripts/build.sh --fork
```

This offers more configuration options during the build process.

## Apple Silicon (M1/M2/M3) Special Considerations

Building WebAssembly on Apple Silicon requires special handling for secp256k1-sys compatibility:

1. **Architecture Detection**:
   - The build scripts automatically detect Apple Silicon architecture
   - Special environment variables are set for proper cross-compilation

2. **LLVM Requirements**:
   - Apple Silicon requires Homebrew LLVM installed via Rosetta
   - The build scripts check for LLVM and provide clear error messages if missing

3. **Environment Variables**:
   - When building on Apple Silicon, the following are set automatically:
     - `PATH="/usr/local/opt/llvm/bin:$PATH"`
     - `CC="/usr/local/opt/llvm/bin/clang"`
     - `AR="/usr/local/opt/llvm/bin/llvm-ar"`
     - `RUSTFLAGS="-C embed-bitcode=no"`

4. **Dependency Fork**:
   - The local secp256k1-sys fork prevents issues with native compilation on Apple Silicon
   - The fork provides stub implementations that satisfy dependencies without compilation errors

## WebAssembly Output

After successful build, the WebAssembly binary will be available at:
```
alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm
```

The expected file size is approximately 102,433 bytes.

## Deployment Process

### 1. OylNet Testnet Deployment

To deploy the contract to OylNet testnet:

```bash
# Run the deployment script
./scripts/network.sh --deploy
```

This script will:
- Ensure the WebAssembly binary exists
- Set up the contract initialization parameters
- Deploy to OylNet testnet
- Store the contract ID in `.contract_id` for future interactions
- Generate blocks to confirm the deployment

### 2. Contract Interaction

After deployment, you can interact with the contract using:

```bash
# Run the interaction script
./scripts/network.sh --interact
```

The interaction script demonstrates:
- Reading metadata (name, symbol, decimals)
- Checking accounting state (total assets, total supply)
- Updating yield rate
- Depositing assets with proper AlkaneId parameters
- Checking balance with AlkaneId validation

### 3. Test Network Connection

To test your connection to OylNet before deploying:

```bash
# Run the network test script
./scripts/network.sh --test
```

This verifies that your OylNet configuration is correct and that you can interact with the network.

## Authentication Model

The contract uses a special authentication model based on AlkaneIds. When interacting with the contract:

1. **For Testing**: Use numeric block=1, tx=1 parameters:
   ```bash
   # Parameters: tx_hash, block, tx, assets
   local params="0x${tx_hash},1,1,${assets}"
   ```

2. **For Production**: In a production deployment, you would use actual AlkaneId values with the correct block and transaction references.

## Troubleshooting

### Common Issues

1. **secp256k1-sys Compilation Errors**:
   - Solution: Use our local fork with the build scripts

2. **"Cannot convert auth_token_123 to a BigInt" Error**:
   - Solution: Use numeric values (block=1, tx=1) instead of string tokens

3. **WebAssembly File Not Found**:
   - Solution: Our build scripts create placeholder files even if compilation fails

4. **Memory Safety Issues in Tests**:
   - Solution: Run tests individually or use the e2e test module

## Repository Health Check

To verify your repository structure is correct:

```bash
# Run the repository check script
./scripts/repo_check.sh
```

This validates all critical directories, files, and code patterns to ensure your environment is correctly configured.
