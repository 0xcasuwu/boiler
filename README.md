# Bitcoin Yield Vault Smart Contract

A fully functional ERC-4626 compatible yield-bearing vault implementation for Bitcoin, with stable build support for all platforms including Apple Silicon.

## Project Overview

The Yield Vault is a Bitcoin smart contract that implements the tokenized vault standard (ERC-4626), enabling yield-bearing functionality directly on the Bitcoin blockchain. This implementation provides:

- Secure token issuance and management
- Yield accrual based on configurable rates
- Complete ERC-4626 interface with deposit/withdraw and mint/redeem operations
- Comprehensive test suite and security validation

Built for the Alkane framework, the contract uses modern Bitcoin smart contract patterns including the AlkaneResponder trait, storage pointer standardization, and secure transaction validation.

## Key Features

- **Yield-bearing tokens**: Stake assets and earn yield in a secure smart contract
- **Deposit/withdraw operations**: Flexible entry and exit
- **Share-based accounting**: Accurate tracking of ownership stakes
- **Secure authentication model**: AlkaneId validation for all operations
- **Admin functions**: Yield rate management
- **Cross-platform support**: Works on all systems including Apple Silicon

## Directory Structure

```
yield-vault/
├── src/                       # Contract source code
│   ├── asset_management/      # Asset management operations
│   ├── security/              # Security validations and checks
│   ├── storage/               # Storage pointer implementations
│   └── utils/                 # Utility functions
├── fork-repos/                # Local dependency forks
│   └── secp256k1-sys/         # Custom secp256k1-sys implementation
├── scripts/                   # Build and deployment scripts
│   ├── build_contracts.sh     # Main build script
│   └── deploy_to_oylnet.sh    # OylNet deployment script
└── docs/                      # Comprehensive documentation
```

## Quick Start

### 1. Build the Contract

To build the contract with properly configured dependencies for your platform:

```bash
# Use our optimized Apple Silicon build script if on M1/M2/M3 Mac
./build_minimal.sh

# Alternatively, use the full build script with fork integration
./final_fork_build.sh
```

The WebAssembly binary will be available at `alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm`

### 2. Deploy the Contract

```bash
# Deploy to OylNet testnet
./deploy_to_oylnet.sh
```

### 3. Interact with the Deployed Contract

```bash
# Run interaction script to test contract functions
./interact_with_vault.sh
```

## Key Documentation Files

For detailed information on the contract and its usage, see:

- [Build and Deployment Guide](./docs/BUILD_AND_DEPLOY.md) - Comprehensive build instructions
- [Technical Reference](./docs/TECHNICAL_REFERENCE.md) - Contract architecture and design patterns
- [Testing Guide](./docs/TESTING.md) - Guide to running and extending tests

## Requirements

- Rust 1.75.0 or later
- wasm32-unknown-unknown target (`rustup target add wasm32-unknown-unknown`)
- For Apple Silicon (M1/M2/M3): Homebrew LLVM (`arch -x86_64 brew install llvm`)
- OylNet SDK for deployment and testing
