# YieldVault: Bitcoin Smart Contract Implementation of ERC-4626

This document details the YieldVault smart contract, which adapts the Ethereum ERC-4626 tokenized vault standard to the Bitcoin blockchain using the memory-bank architecture.

## Overview

YieldVault is a Bitcoin smart contract that implements the core functionality of ERC-4626, providing a standardized API for yield-bearing vaults. It allows users to deposit assets into a yield-generating vault and receive shares that represent their proportional claim on the underlying assets.

## ERC-4626 Background

The ERC-4626 standard was developed for Ethereum to standardize tokenized yield-bearing vaults. Key features include:

- **Deposit and Withdrawal**: Standardized methods for depositing assets and withdrawing them
- **Accounting Functions**: Methods for calculating shares from assets and vice versa
- **Metadata Functions**: Standard interface for retrieving vault information
- **Preview Functions**: Methods to simulate the effects of deposits, withdrawals, etc.

## Adaptation to Bitcoin Memory-Bank Architecture

The YieldVault contract adapts ERC-4626 to Bitcoin's WebAssembly-based smart contract platform using:

### Core Architecture Changes

1. **Opcode-Based Interface**: Instead of Ethereum's function-based ABI, uses numeric opcodes for message dispatch
2. **Transaction Hash Tracking**: Implements Bitcoin-specific replay protection via transaction hash tracking
3. **Storage Patterns**: Uses key-value storage with standardized paths instead of Ethereum's slot-based storage
4. **WebAssembly Compilation**: Compiles to WASM for execution on Bitcoin instead of EVM bytecode

### Key Components

#### Message Dispatch System

The contract uses the `MessageDispatch` derive macro to create an opcode-based interface:

```rust
#[derive(MessageDispatch)]
enum YieldVaultMessage {
    #[opcode(0)]
    Initialize { /* parameters */ },
    
    #[opcode(10)]
    Deposit { /* parameters */ },
    
    // Additional operations...
}
```

#### Security Layer

Security features adapted for Bitcoin's environment:

1. **Initialization Guard**: Prevents multiple initializations
2. **Transaction Hash Validation**: Prevents replay attacks
3. **Authorization Checks**: Ensures only owners can manage their assets
4. **Overflow Protection**: Uses checked arithmetic throughout

#### Storage Structure

Follows consistent storage paths:

- `/name`, `/symbol` - Vault metadata
- `/asset-name`, `/asset-symbol` - Underlying asset information
- `/total-supply` - Total number of shares
- `/total-assets` - Total assets under management
- `/balances/{account}` - Account balances
- `/tx-hashes` - Used transaction hashes
- `/yield-rate` - Current yield rate in basis points
- `/last-yield-update` - Timestamp of last yield application

## Core Functionality

### Asset Management

- **Deposit**: Deposit assets and receive shares in return
- **Mint**: Mint a specific amount of shares by depositing the required assets
- **Withdraw**: Withdraw assets by burning shares
- **Redeem**: Redeem shares for underlying assets

### Yield Management

The contract implements time-based yield accrual:

```rust
fn update_yield(&self) -> Result<(), &'static str> {
    // Calculate time elapsed since last update
    let current_time = self.get_timestamp();
    let last_update = storage::get_u64("/last-yield-update").unwrap_or(current_time);
    let time_elapsed = current_time - last_update;
    
    // Apply yield based on elapsed time
    // yield = assets * rate * timeElapsed / (10000 * seconds_in_year)
    // ...
}
```

### Accounting

The contract provides standard accounting functions:

- **TotalAssets**: Returns the total amount of underlying assets
- **ConvertToShares**: Converts a specified amount of assets to shares
- **ConvertToAssets**: Converts a specified amount of shares to assets

### Share Price Calculation

Share price is determined by the ratio of total assets to total shares:

```
price_per_share = total_assets / total_supply
```

## Opcode Interface

| Opcode | Function | Description |
|--------|----------|-------------|
| 0 | Initialize | Initialize the vault with basic parameters |
| 10 | Deposit | Deposit assets and receive shares |
| 11 | Mint | Mint exact shares by depositing assets |
| 12 | Withdraw | Withdraw assets by burning shares |
| 13 | Redeem | Redeem shares for assets |
| 100-103 | Metadata | View functions for name, symbol, decimals, asset |
| 200-202 | Accounting | Functions for asset and share accounting |
| 300-303 | Limits | Functions that return maximum deposit/withdrawal amounts |
| 400-403 | Preview | Preview functions for deposit/withdrawal operations |
| 500-501 | Custom Data | Store and retrieve custom data |
| 600-601 | Balances | Get account balance and total supply |
| 900 | UpdateYield | Update the yield rate |

## Usage Examples

### Initialization

Initialize a new vault:

```javascript
// Example initialization using Bitcoin opcall
opcall(contractId, 0, {
  name: "Bitcoin Yield Vault", 
  symbol: "bYV",
  asset_name: "Bitcoin",
  asset_symbol: "BTC",
  decimal_offset: 8
});
```

### Deposit Assets

Deposit assets into the vault:

```javascript
// Example deposit
opcall(contractId, 10, {
  tx_hash: "0x123...", // Current transaction hash
  caller: "bc1q...",   // Caller's address
  receiver: "bc1q...", // Receiver's address
  assets: 100000000    // 1 BTC (in satoshis)
});
```

### View Share Balance

Query share balance:

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

## Implementation Differences from ERC-4626

### Added Security Features

1. **Transaction Hash Validation**: Required for Bitcoin to prevent replay attacks
2. **Explicit Authorization**: Every operation includes caller and owner parameters

### Parameter Changes

1. **Transaction Context**: Each modifying operation requires a transaction hash
2. **Authorization Parameters**: Operations include explicit caller/receiver/owner parameters

### Yield Management

Added yield management functionality not specified in ERC-4626:

1. **Yield Rate Setting**: Ability to set and update yield rates
2. **Time-based Accrual**: Automatic yield calculation based on time elapsed

## Testing and Deployment

### Build Process

```bash
# Build for WebAssembly target
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release --features "blockchain"

# Optimize the WebAssembly binary
wasm-opt -Oz -o yield_vault_opt.wasm target/wasm32-unknown-unknown/release/yield_vault.wasm
```

### Deployment Process

Deploy the contract through a Bitcoin-specific deployment tool:

```javascript
async function deployVault() {
  const wasmCode = fs.readFileSync('./yield_vault.wasm');
  
  // Create deployment transaction
  const deployTx = await createDeploymentTransaction(wasmCode);
  
  // Sign and broadcast transaction
  const txId = await signAndBroadcast(deployTx);
  
  console.log(`Vault deployed with transaction ID: ${txId}`);
  return txId;
}
```

## Security Considerations

### Specific to Bitcoin Implementation

1. **Transaction Hash Storage Growth**: As more transactions are processed, the storage requirements for hash tracking grow. Consider implementing a pruning mechanism for old transaction hashes.

2. **Authorization Model**: The current implementation uses a simple owner-based authorization. Consider implementing a more flexible delegation system in the future.

3. **Yield Calculation Precision**: The yield calculation uses fixed-point arithmetic, which may lead to rounding errors over time. Consider implementing a more precise accounting system for long-term vaults.

### Common ERC-4626 Considerations

1. **Share Price Manipulation**: The contract is vulnerable to share price manipulation if a large deposit is made and immediately withdrawn, affecting the share price for other users.

2. **Rounding Errors**: Rounding in asset/share conversion can lead to small value leaks over time.

## Future Improvements

1. **Optimized Transaction Hash Storage**: Implement bloom filters or other efficient data structures
2. **Advanced Authorization System**: Support delegated operations
3. **Variable Yield Strategies**: Support pluggable yield strategies
4. **Fee Structure**: Add support for management and performance fees

## Conclusion

The YieldVault implementation successfully adapts the ERC-4626 standard to the Bitcoin blockchain using the memory-bank architecture. It provides a secure, standardized interface for yield-bearing vaults while addressing Bitcoin-specific security considerations.
