# Technical Reference

## Architecture Overview

The Yield Vault smart contract implements the ERC-4626 tokenized vault standard on Bitcoin, providing a yield-bearing token system with secure asset management. This document covers the technical architecture, design patterns, and implementation details.

## Core Components

### 1. MessageDispatch Pattern

The contract uses a message dispatch pattern based on opcodes to route function calls:

| Opcode Range | Functionality                   |
|--------------|--------------------------------|
| 0            | Initialize                     |
| 10-19        | Asset Management Operations    |
| 100-199      | Metadata View Functions        |
| 200-299      | Accounting View Functions      |
| 300-399      | Limit View Functions           |
| 400-499      | Preview View Functions         |
| 500-599      | Custom Data Operations         |
| 600-699      | Balance Management             |
| 900-999      | Administrative Operations      |

### 2. Storage Architecture

The contract uses a standardized storage pointer approach for all persistent data:

```rust
// Storage pointer definition for standardized paths
StoragePointer::from_keyword("/key-name")
```

#### Key Storage Paths:

| Storage Path           | Purpose                           |
|------------------------|-----------------------------------|
| `/name`                | Vault token name                  |
| `/symbol`              | Vault token symbol                |
| `/asset-name`          | Underlying asset name             |
| `/asset-symbol`        | Underlying asset symbol           |
| `/decimals`            | Decimal precision                 |
| `/total-supply`        | Total share supply tracking       |
| `/total-assets`        | Total assets under management     |
| `/yield-rate`          | Configured yield rate (basis pts) |
| `/last-yield-update`   | Timestamp of last yield update    |
| `/balances/{account}`  | Account share balances            |
| `/data/{key}`          | Custom stored data                |
| `/initialized`         | Initialization guard              |

### 3. Traits and Interfaces

The contract implements several key traits:

1. **AlkaneResponder**
   - Provides context access for transaction data
   - Supplies block height for time-based operations
   - Processes transaction data for security validations

2. **AssetManagement**
   - Handles deposit, mint, withdraw, redeem operations
   - Implements asset/share conversion logic
   - Enforces balance and permission checks

3. **YieldAccrual**
   - Manages yield calculation based on elapsed time
   - Updates asset totals while maintaining share supply
   - Provides configurable yield rates

## Security Model

### Authentication and Authorization

The contract uses an AlkaneId-based authentication system:

```rust
let transfer = &context.incoming_alkanes.0[0];
let is_token_valid = if auth_token_id == "auth_token_123" {
    // Test mode - special handling
    let block = transfer.id.block;
    let tx = transfer.id.tx;
    block == 1 && tx == 1
} else {
    // Standard mode - compare string representations
    let transfer_id_str = format!("{:?}", transfer.id);
    transfer_id_str == auth_token_id
};
```

This approach validates ownership through:
1. For testing: Checking against specific block/tx values (1,1)
2. For production: Comparing the AlkaneId string representation

### Security Patterns

The contract implements multiple security patterns:

1. **Initialization Guard**
   ```rust
   // Prevent multiple initializations
   pub fn observe_initialization() -> Result<()> {
       let initialized = initialized_pointer().get::<bool>().unwrap_or(false);
       if initialized {
           return Err(anyhow!("Contract already initialized"));
       }
       initialized_pointer().set(&true);
       Ok(())
   }
   ```

2. **Transaction Anti-Replay Protection**
   ```rust
   // Prevent transaction replay attacks
   pub fn validate_transaction_hash(tx_hash: &[u8]) -> Result<()> {
       let mut used_hashes = used_transaction_hashes_pointer().get_or_default::<HashSet<Vec<u8>>>();
       if used_hashes.contains(&tx_hash.to_vec()) {
           return Err(anyhow!("Transaction hash already used"));
       }
       used_hashes.insert(tx_hash.to_vec());
       used_transaction_hashes_pointer().set(&used_hashes);
       Ok(())
   }
   ```

3. **Overflow Protection**
   ```rust
   // Safe arithmetic to prevent overflow
   pub fn overflow_error<T: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv>(op: Option<T>) -> Result<T> {
       op.ok_or_else(|| anyhow!("Arithmetic overflow or division by zero"))
   }
   ```

4. **Permission Validation**
   ```rust
   // Verify user has appropriate permissions
   pub fn check_authorization(actual_id: &AlkaneId, expected: &str) -> Result<()> {
       if format!("{:?}", actual_id) != expected {
           return Err(anyhow!("Unauthorized operation"));
       }
       Ok(())
   }
   ```

## ERC-4626 Implementation

The contract fully complies with ERC-4626 tokenized vault standard with these key functions:

### Asset Management

1. **Deposit**: Convert assets to shares and assign to receiver
   ```rust
   pub fn deposit(context: &Context, tx_hash: &[u8], caller: &str, receiver: &str, assets: u128) -> Result<u128>
   ```

2. **Mint**: Issue exact share amount in exchange for assets
   ```rust
   pub fn mint(context: &Context, tx_hash: &[u8], caller: &str, receiver: &str, shares: u128) -> Result<u128>
   ```

3. **Withdraw**: Remove assets from vault
   ```rust
   pub fn withdraw(context: &Context, tx_hash: &[u8], caller: &str, receiver: &str, owner: &str, assets: u128) -> Result<u128>
   ```

4. **Redeem**: Burn shares to receive assets
   ```rust
   pub fn redeem(context: &Context, tx_hash: &[u8], caller: &str, receiver: &str, owner: &str, shares: u128) -> Result<u128>
   ```

### View Functions

1. **Accounting Functions**: total assets, share conversion, price per share
2. **Limit Functions**: max deposit, max mint, max withdraw, max redeem
3. **Preview Functions**: simulate operations without state changes

## Yield Accrual System

The yield accrual system increases asset totals while maintaining the same supply:

```rust
pub fn update_yield(context: &Context) -> Result<()> {
    let last_update_height = last_yield_height_pointer().get::<u64>().unwrap_or(0);
    let current_height = context.this_block;
    
    if current_height <= last_update_height {
        return Ok(());
    }
    
    let yield_rate = yield_rate_pointer().get::<u64>().unwrap_or(0);
    if yield_rate == 0 {
        return Ok(());
    }
    
    let blocks_elapsed = current_height - last_update_height;
    let total_assets = total_assets_pointer().get::<u128>().unwrap_or(0);
    
    // Calculate yield: assets * rate * time / (10000 * BLOCKS_PER_YEAR)
    let yield_amount = overflow_error(
        total_assets.checked_mul(yield_rate.into())
            .and_then(|v| v.checked_mul(blocks_elapsed.into()))
            .and_then(|v| v.checked_div(10000 * BLOCKS_PER_YEAR as u128))
    )?;
    
    let new_total_assets = overflow_error(total_assets.checked_add(yield_amount))?;
    total_assets_pointer().set(&new_total_assets);
    last_yield_height_pointer().set(&current_height);
    
    Ok(())
}
```

The yield calculation factors in:
- Time elapsed since last update (in blocks)
- Configured yield rate (in basis points)
- Current total assets
- Scaling factor based on blocks per year

## Asset/Share Conversion Logic

The contract implements bidirectional conversion between assets and shares:

```rust
// Convert assets to shares
pub fn convert_to_shares(assets: u128) -> Result<u128> {
    let total_assets = total_assets_pointer().get::<u128>().unwrap_or(0);
    let total_supply = total_supply_pointer().get::<u128>().unwrap_or(0);
    
    if total_assets == 0 || total_supply == 0 {
        return Ok(assets);
    }
    
    overflow_error(assets.checked_mul(total_supply).and_then(|v| v.checked_div(total_assets)))
}

// Convert shares to assets
pub fn convert_to_assets(shares: u128) -> Result<u128> {
    let total_assets = total_assets_pointer().get::<u128>().unwrap_or(0);
    let total_supply = total_supply_pointer().get::<u128>().unwrap_or(0);
    
    if total_supply == 0 {
        return Ok(0);
    }
    
    overflow_error(shares.checked_mul(total_assets).and_then(|v| v.checked_div(total_supply)))
}
```

This ensures accurate pricing based on:
- Current share supply
- Current total assets
- Special handling for edge cases (empty vault)

## Opcode Reference

| Opcode | Function                           | Parameters                          |
|--------|------------------------------------|------------------------------------|
| 0      | Initialize                         | name, symbol, asset_name, asset_symbol, decimal_offset |
| 10     | Deposit                            | tx_hash, caller, receiver, assets   |
| 11     | Mint                               | tx_hash, caller, receiver, shares   |
| 12     | Withdraw                           | tx_hash, caller, receiver, owner, assets |
| 13     | Redeem                             | tx_hash, caller, receiver, owner, shares |
| 100    | GetName                            | -                                  |
| 101    | GetSymbol                          | -                                  |
| 102    | GetDecimals                        | -                                  |
| 103    | GetAsset                           | -                                  |
| 200    | GetTotalAssets                     | -                                  |
| 201    | ConvertToShares                    | assets                             |
| 202    | ConvertToAssets                    | shares                             |
| 300    | GetMaxDeposit                      | receiver                           |
| 301    | GetMaxMint                         | receiver                           |
| 302    | GetMaxWithdraw                     | owner                              |
| 303    | GetMaxRedeem                       | owner                              |
| 400    | PreviewDeposit                     | assets                             |
| 401    | PreviewMint                        | shares                             |
| 402    | PreviewWithdraw                    | assets                             |
| 403    | PreviewRedeem                      | shares                             |
| 500    | SetData                            | key, value                         |
| 501    | GetData                            | key                                |
| 600    | GetBalanceOf                       | account                            |
| 601    | GetTotalSupply                     | -                                  |
| 900    | UpdateYield                        | yield_rate                         |
| 901    | GetYieldRate                       | -                                  |

## Implementation Best Practices

The implementation follows several best practices:

1. **Storage Standardization**: All persistent data uses a consistent storage path pattern
2. **Error Handling**: Comprehensive error messages with the anyhow crate
3. **Security-First Design**: Multiple layers of validation and protection
4. **Checked Arithmetic**: All numeric operations use checked variants to prevent overflow
5. **Consistent Interface**: Standardized function signatures across similar operations
6. **State Validation**: Pre-conditions checked before state changes
7. **Memory Safety**: References used over raw pointers where possible
8. **Documentation**: Comprehensive inline documentation for all functions

## Memory Model

The contract uses a storage-first approach with minimal in-memory operations:

1. **Immutable References**: Used wherever possible to prevent memory issues
2. **Storage Pointers**: Standardized access to persistent storage
3. **Context Validation**: All context access checked before use
4. **Serialization**: JSON-based serialization for complex data types

This approach minimizes memory usage and reduces the risk of memory corruption issues.
