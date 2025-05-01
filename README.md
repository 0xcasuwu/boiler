# YieldVault: ERC-4626 for Bitcoin Smart Contracts

A Bitcoin smart contract implementation of the ERC-4626 tokenized vault standard, built using the memory-bank architecture.

## Overview

YieldVault adapts the Ethereum ERC-4626 standard to Bitcoin's WebAssembly-based smart contract platform. It provides a standardized API for yield-bearing vaults on Bitcoin, allowing users to deposit assets into a yield-generating vault and receive shares that represent their proportional claim on the underlying assets.

## Features

- **Vault Operations**: Deposit assets, mint shares, withdraw assets, redeem shares
- **Yield Generation**: Time-based yield accrual system with configurable rates
- **Security Features**: Initialization guard, transaction hash validation, overflow protection
- **View Functions**: Comprehensive query methods for vault state and calculations
- **Preview Functions**: Simulate operations without executing them
- **Limit Functions**: Get maximum deposit, mint, withdraw, and redeem amounts

## Architecture

YieldVault is built following the memory-bank architecture patterns:

- **MessageDispatch Pattern**: Opcode-based message handling
- **Storage Pattern**: Consistent storage paths and serialization formats
- **Security Pattern**: Multiple security layers including initialization guards and transaction validation
- **Testing Patterns**: Comprehensive unit and integration testing

## Repository Structure

```
boiler/
├── src/
│   ├── lib.rs                       # Main contract implementation
│   ├── constants.rs                 # Storage pointer definitions and constants
│   └── tests/
│       ├── mod.rs                   # Test module organization
│       ├── mock.rs                  # Mock implementations for testing
│       ├── minimal_test.rs          # Basic verification tests
│       ├── basic_tests.rs           # Standard unit tests
│       ├── yield_vault_test.rs      # Advanced unit tests
│       ├── yield_vault_test_direct.rs # Direct method call tests
│       ├── yield_vault_mock_tests.rs  # Tests with mocked dependencies
│       ├── yield_vault_test_unit_wasm.rs # WASM-specific unit tests
│       └── yield_vault_test_integration.rs # Integration tests
├── scripts/
│   ├── wasm-build.sh               # Build script for WASM
│   └── test-wasm.sh                # Test script for WASM tests
├── Cargo.toml                      # Project dependencies
├── README.md                       # This file
└── memory-bank/                    # Documentation and context
    ├── YieldVault.md               # Contract documentation
    ├── YieldVault.rs               # Original implementation (for reference)
    └── implementation-summary.md   # Summary of implementation
```

## Testing

The project features a comprehensive test suite:

- **Unit Tests**: Verify individual components and functions in isolation
- **WebAssembly Tests**: Specifically test the WASM target functionality
  - Use `scripts/test-wasm.sh` to run WASM tests
- **Integration Tests**: Test the contract as a whole system
- **Mock-based Tests**: Use mock implementations for deterministic testing

Run standard tests:
```bash
cargo test
```

Run WebAssembly tests:
```bash
./scripts/test-wasm.sh
```

## Core Concepts

### Asset/Share Mechanics

YieldVault implements the core ERC-4626 concept of assets (deposited tokens) and shares (representation of ownership):

- When users deposit assets, they receive shares proportional to their contribution
- Share price (assets per share) increases as yield accrues
- Later deposits receive fewer shares per asset as the share price increases
- Withdrawals and redemptions convert between assets and shares at the current exchange rate

### Yield Accrual

The contract features a time-based yield accrual system:

- Yield rate is specified in basis points (1/100th of a percent)
- Yield is calculated based on time elapsed since last update
- Yield increases total assets while keeping total shares constant
- This mechanism increases the asset value of each share over time

## Security Features

- **Initialization Guard**: Prevents multiple initializations
- **Transaction Hash Tracking**: Prevents transaction replay attacks
- **Overflow Protection**: Checked arithmetic throughout to prevent numeric overflows
- **Authorization Checks**: Enforces that only asset owners can withdraw or redeem

## Usage

### Initialization

```javascript
// Example initialization
opcall(contractId, 0, {
  name: "Bitcoin Yield Vault", 
  symbol: "bYV",
  asset_name: "Bitcoin",
  asset_symbol: "BTC",
  decimal_offset: 8
});
```

### Deposit Assets

```javascript
// Example deposit
opcall(contractId, 10, {
  tx_hash: "0x123...", // Current transaction hash
  caller: "bc1q...",   // Caller's address
  receiver: "bc1q...", // Receiver's address
  assets: 100000000    // 1 BTC (in satoshis)
});
```

### Check Balance

```javascript
// Get balance
const balance = opcall(contractId, 600, {
  account: "bc1q..."
});

// Get share value in assets
const assetValue = opcall(contractId, 202, {
  shares: balance
});
```

## Implementation Notes

This implementation adapts the ERC-4626 standard to fit Bitcoin's environment:

- Added transaction hash validation for replay protection
- Explicit caller/receiver/owner parameters for authorization
- Uses opcode-based interface instead of function-based ABI
- Employs storage patterns optimized for Bitcoin's unique constraints

## Acknowledgements

Based on the [memory-bank architecture](link) and inspired by the [ERC-4626 standard](https://eips.ethereum.org/EIPS/eip-4626).
