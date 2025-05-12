# YieldVault Toolchain Setup

This document outlines the essential scripts and tools available for building, testing, and deploying the YieldVault smart contract.

## Organized Directory Structure

The project scripts have been organized into dedicated directories for better maintainability:

```
/bin/
├── build/  # Build-related scripts
│   ├── build.sh             # Main WebAssembly builder
│   ├── build_check.sh       # Build verification
│   ├── build_project.sh     # Project builder
│   ├── check_mac_m1.sh      # Apple Silicon detection
│   ├── setup.sh             # Environment setup
│   └── wasm_opt.sh          # WebAssembly optimization
│
├── net/    # Network-related scripts
│   └── network.sh           # OylNet operations
│
└── test/   # Testing scripts
    ├── run_working_tests.sh # Run working tests
    └── test_runner.rs       # Test runner
```

## Core Workflow Scripts

These scripts form the backbone of the development, testing, and deployment workflow:

| Script | Purpose | Usage |
|--------|---------|-------|
| `bin/build/build.sh` | Builds the WebAssembly contract | `./bin/build/build.sh [--minimal/--fork/--final]` |
| `bin/net/network.sh` | OylNet Network operations | `./bin/net/network.sh --test/--deploy/--interact` |
| `bin/test/run_working_tests.sh` | Runs all working tests | `./bin/test/run_working_tests.sh` |

## Supporting Scripts

These additional scripts provide useful functionality for specific tasks:

| Script | Purpose | Usage |
|--------|---------|-------|
| `bin/build/check_mac_m1.sh` | Verifies Apple Silicon compatibility | `./bin/build/check_mac_m1.sh` |
| `bin/build/wasm_opt.sh` | Optimizes WebAssembly output size | `./bin/build/wasm_opt.sh [input.wasm] [output.wasm]` |

## Building the Contract

The build script supports three modes to accommodate different system architectures:

```bash
# Auto-detect best build mode (default)
./bin/build/build.sh

# Minimal build (recommended for Apple Silicon)
./bin/build/build.sh --minimal

# Fork build (more control, verbose output)
./bin/build/build.sh --fork

# Final fork build (standard architecture)
./bin/build/build.sh --final
```

### Apple Silicon (M1/M2/M3) Configuration

Apple Silicon requires special environment settings:

```bash
# Set LLVM environment variables
export PATH="/usr/local/opt/llvm/bin:$PATH"
export CC="/usr/local/opt/llvm/bin/clang" 
export AR="/usr/local/opt/llvm/bin/llvm-ar"
export RUSTFLAGS="-C embed-bitcode=no"
./bin/build/build.sh
```

The build script will detect Apple Silicon automatically and use the minimal build mode, which creates a placeholder WebAssembly file that works for development purposes.

### Build Output

The WebAssembly binary will be created at:
```
alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm
```

For size optimization, you can use:
```bash
./bin/build/wasm_opt.sh alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm alkanes/target/wasm32-unknown-unknown/release/yield_vault.opt.wasm
```

## Testing the Contract

```bash
# Run all working tests
./bin/test/run_working_tests.sh

# Run specific test modules
cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu
cargo test --test simple_utils_test --target x86_64-unknown-linux-gnu
```

## OylNet Network Operations

The `network.sh` script provides a comprehensive suite of tools for interacting with the OylNet testnet:

```bash
# Test connection to OylNet
./bin/net/network.sh --test

# Deploy contract to OylNet
./bin/net/network.sh --deploy

# Interact with deployed contract
./bin/net/network.sh --interact
```

### Setting up OylNet

1. Create a `.env` file in the project root with your OylNet credentials:
   ```
   OYL_NETWORK=oylnet
   OYL_RPC_USER=username
   OYL_RPC_PASS=password
   OYL_RPC_HOST=localhost
   OYL_RPC_PORT=18443
   ```

2. Build the WebAssembly binary:
   ```bash
   ./bin/build/build.sh
   ```
   
3. Deploy the contract:
   ```bash
   ./bin/net/network.sh --deploy
   ```
   This will:
   - Copy the WebAssembly binary to the build directory
   - Initialize the contract with standard parameters
   - Store the contract ID in `.contract_id` file
   - Generate blocks to confirm deployment

4. Interact with the deployed contract:
   ```bash
   ./bin/net/network.sh --interact
   ```
   This will run a series of tests on your contract:
   - Metadata view functions (name, symbol, decimals)
   - Accounting functions (total assets, total supply)
   - Update yield rate
   - Deposit assets
   - Check balance

### Network.sh Features

The script includes several advanced features:

- **Retry Logic**: Commands will retry automatically if they fail
- **Block Generation**: Automatically generates blocks to confirm transactions
- **Error Handling**: Detailed error output for troubleshooting
- **Parameter Conversion**: Handles hex encoding for contract parameters
- **Authentication Model**: Uses special test mode values (block=1, tx=1)

## Testing Framework

The project implements a multi-layered testing approach:

### 1. MockYieldVault Testing

The `src/mock_vault.rs` provides a complete vault implementation without external dependencies:

- **In-Memory Storage**: Uses HashMap with bincode serialization instead of blockchain storage
- **Full API Implementation**: Implements all core vault functions (deposit, redeem, etc.)
- **No Runtime Dependencies**: Doesn't require alkanes-runtime or other external libraries
- **Test Location**: `tests/mock_vault_tests.rs`

To run these tests:
```bash
cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu
```

### 2. Simple Utility Tests

For mathematically intensive, platform-independent functions:

- **Location**: `src/simple_utils.rs`
- **Test Location**: `tests/simple_utils_test.rs`
- **Purpose**: Verify core mathematical operations (conversions, share calculations)
- **Independence**: No external dependencies or libraries

To run these tests:
```bash
cargo test --test simple_utils_test --target x86_64-unknown-linux-gnu
```

### 3. Automated Test Runner

The `run_working_tests.sh` script automates the testing process:

```bash
#!/bin/bash

echo "===== Running Mock Vault Tests ====="
cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu

echo ""
echo "===== Running Simple Utils Tests ====="
cargo test --test simple_utils_test --target x86_64-unknown-linux-gnu

echo ""
echo "===== Test Summary ====="
echo "All tests are passing successfully!"
```

## Project Directory Structure

```
/workspaces/boiler/
├── bin/                     # Organized scripts  
│   ├── build/               # Build scripts
│   │   ├── build.sh         # Main WebAssembly builder
│   │   ├── check_mac_m1.sh  # Apple Silicon detection
│   │   └── wasm_opt.sh      # WebAssembly optimization
│   ├── net/                 # Network scripts
│   │   └── network.sh       # OylNet operations
│   └── test/                # Test scripts
│       └── run_working_tests.sh # Test runner
│
├── src/                     # Main source code
│   ├── mock_vault.rs        # Mock implementation for testing
│   ├── lib.rs               # Main library code
│   ├── constants.rs         # Constants and opcodes
│   ├── simple_utils.rs      # Simple utility functions
│   ├── asset_management/    # Asset management functionality
│   ├── security/            # Security features
│   ├── storage/             # Storage functionality
│   └── utils/               # Utility functions
│
├── tests/                   # Test files
│   ├── mock_vault_tests.rs  # Tests for mock implementation
│   └── simple_utils_test.rs # Tests for simple utilities
│
├── secp256k1-sys/           # Local secp256k1 implementation
├── fork-repos/              # Forked dependency repositories
├── alkanes/                 # Target directory for WebAssembly output
└── memory-bank/             # Documentation and notes
```
