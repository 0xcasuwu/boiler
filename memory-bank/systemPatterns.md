# Yield Vault System Design and Patterns

## IRONCLAD RULE FOR DEPENDENCIES

**ALWAYS use direct GitHub repositories from kungfuflex/alkanes-rs, NEVER use local stubs** (except for secp256k1-sys, which needs a stub for cross-platform compatibility). This rule has been established after thorough testing and verification to ensure maximum compatibility and functionality.

## Architecture Overview

The Yield Vault smart contract follows ERC-4626 tokenized vault standards and is designed using a modular architecture with clear separation of concerns. The codebase is organized into components that handle specific functionality, with a core MessageDispatch pattern for operation routing.

## Core Design Patterns

### 1. MessageDispatch Pattern

Operations are dispatched through a central handler based on numeric opcodes:

```rust
pub fn handle_alkane_message(data: &[u8]) -> Result<Vec<u8>> {
    let message_opcode = read_prefix(data)?;
    match message_opcode {
        // Initialization
        0 => handle_initialize(data),
        
        // Asset Management Operations (10-19)
        10 => handle_deposit(data),
        11 => handle_mint(data),
        12 => handle_withdraw(data),
        13 => handle_redeem(data),
        
        // Metadata View Functions (100-199)
        100 => handle_get_name(),
        101 => handle_get_symbol(),
        102 => handle_get_decimals(),
        103 => handle_get_asset(),
        
        // Accounting View Functions (200-299)
        200 => handle_get_total_assets(),
        201 => handle_convert_to_shares(data),
        202 => handle_convert_to_assets(data),
        
        // Limit View Functions (300-399)
        300 => handle_get_max_deposit(data),
        301 => handle_get_max_mint(data),
        302 => handle_get_max_withdraw(data),
        303 => handle_get_max_redeem(data),
        
        // Preview View Functions (400-499)
        400 => handle_preview_deposit(data),
        401 => handle_preview_mint(data),
        402 => handle_preview_withdraw(data),
        403 => handle_preview_redeem(data),
        
        // Custom Data Operations (500-599)
        500 => handle_set_data(data),
        501 => handle_get_data(data),
        
        // Balance Management (600-699)
        600 => handle_get_balance_of(data),
        601 => handle_get_total_supply(),
        
        // Administrative Operations (900-999)
        900 => handle_update_yield_rate(data),
        901 => handle_get_yield_rate(),
        
        _ => Err(anyhow!("Unknown opcode")),
    }
}
```

### 2. Standardized Storage Pattern

The contract uses a consistent storage pointer system with standardized paths:

```rust
// Storage pointer definition
pub fn name_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/name")
}

pub fn symbol_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/symbol")
}

pub fn asset_name_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/asset-name")
}

// ... and so on for other storage items
```

Key storage paths include:
- `/name` - Vault token name
- `/symbol` - Vault token symbol
- `/asset-name` - Underlying asset name
- `/asset-symbol` - Underlying asset symbol
- `/decimals` - Decimal precision
- `/total-supply` - Total share supply tracking
- `/total-assets` - Total assets under management
- `/yield-rate` - Configured yield rate in basis points
- `/last-yield-update` - Timestamp of last yield update
- `/balances/{account}` - Account share balances
- `/data/{key}` - Custom stored data
- `/initialized` - Initialization guard

### 3. Security Patterns

#### 3.1 Initialization Guard

```rust
pub fn observe_initialization() -> Result<()> {
    let initialized = initialized_pointer().get::<bool>().unwrap_or(false);
    if initialized {
        return Err(anyhow!("Contract already initialized"));
    }
    initialized_pointer().set(&true);
    Ok(())
}
```

#### 3.2 Transaction Replay Protection

```rust
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

#### 3.3 Overflow Protection

```rust
pub fn overflow_error<T: CheckedAdd + CheckedSub + CheckedMul + CheckedDiv>(op: Option<T>) -> Result<T> {
    op.ok_or_else(|| anyhow!("Arithmetic overflow or division by zero"))
}
```

#### 3.4 AlkaneId Verification Pattern

The contract uses direct comparison of AlkaneId structs for verification:

```rust
// Direct comparison of AlkaneId structs
// This works because AlkaneId implements PartialEq
if transfer.id == asset_id {
    received = received.checked_add(transfer.value)
        .ok_or("Asset amount overflow")?;
}
```

This pattern is used in:
- `verify_incoming_assets` method
- `verify_incoming_shares` method
- Asset ID verification in `withdraw` and `redeem` methods

The direct comparison approach is more reliable than string-based methods, as it compares the actual `block` and `tx` fields of the `AlkaneId` struct.

### 4. Yield Calculation Pattern

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

### 5. Asset Management Pattern

#### 5.1 Deposit Function

```rust
pub fn deposit(context: &Context, tx_hash: &[u8], caller: &str, receiver: &str, assets: u128) -> Result<u128> {
    validate_transaction_hash(tx_hash)?;
    check_authorization(context, caller)?;
    update_yield(context)?;

    if assets == 0 {
        return Err(anyhow!("Cannot deposit zero assets"));
    }

    let shares = convert_to_shares(assets)?;
    if shares == 0 {
        return Err(anyhow!("Deposit amount too small"));
    }

    let max_deposit = get_max_deposit(receiver)?;
    if assets > max_deposit {
        return Err(anyhow!("Deposit exceeds maximum"));
    }

    // Update total assets and shares
    let total_assets = total_assets_pointer().get::<u128>().unwrap_or(0);
    let new_total_assets = overflow_error(total_assets.checked_add(assets))?;
    total_assets_pointer().set(&new_total_assets);

    let total_supply = total_supply_pointer().get::<u128>().unwrap_or(0);
    let new_total_supply = overflow_error(total_supply.checked_add(shares))?;
    total_supply_pointer().set(&new_total_supply);

    // Update receiver balance
    let receiver_balance_pointer = balance_pointer(receiver);
    let receiver_balance = receiver_balance_pointer.get::<u128>().unwrap_or(0);
    let new_receiver_balance = overflow_error(receiver_balance.checked_add(shares))?;
    receiver_balance_pointer.set(&new_receiver_balance);

    Ok(shares)
}
```

#### 5.2 Share/Asset Conversion Functions

```rust
pub fn convert_to_shares(assets: u128) -> Result<u128> {
    let total_assets = total_assets_pointer().get::<u128>().unwrap_or(0);
    let total_supply = total_supply_pointer().get::<u128>().unwrap_or(0);
    
    if total_assets == 0 || total_supply == 0 {
        return Ok(assets);
    }
    
    overflow_error(assets.checked_mul(total_supply).and_then(|v| v.checked_div(total_assets)))
}

pub fn convert_to_assets(shares: u128) -> Result<u128> {
    let total_assets = total_assets_pointer().get::<u128>().unwrap_or(0);
    let total_supply = total_supply_pointer().get::<u128>().unwrap_or(0);
    
    if total_supply == 0 {
        return Ok(0);
    }
    
    overflow_error(shares.checked_mul(total_assets).and_then(|v| v.checked_div(total_supply)))
}
```

## Token-Based Authorization Model

The contract uses a token-based authorization model where possession of tokens is sufficient proof of ownership:

### 1. Verify Incoming Assets

```rust
fn verify_incoming_assets(&self, incoming_alkanes: &AlkaneTransferParcel) -> Result<u128, &'static str> {
    // Get the asset ID
    let asset_id = self.get_asset_id();
    
    // Check if there are any incoming assets at all
    if incoming_alkanes.0.is_empty() {
        return Err("No assets provided in transaction");
    }
    
    // Sum all incoming assets with matching ID
    let mut received = 0u128;
    
    for transfer in &incoming_alkanes.0 {
        // Direct comparison of AlkaneId structs
        // This works because AlkaneId implements PartialEq
        if transfer.id == asset_id {
            received = received.checked_add(transfer.value)
                .ok_or("Asset amount overflow")?;
        }
    }
        
    // If no assets match our asset ID, this is an error
    if received == 0 {
        return Err("Invalid asset ID: received assets do not match expected asset type");
    }
        
    Ok(received)
}
```

### 2. Verify Incoming Shares

```rust
fn verify_incoming_shares(&self, incoming_alkanes: &AlkaneTransferParcel) -> Result<u128, &'static str> {
    let context = match AlkaneResponder::context(self) {
        Ok(ctx) => ctx,
        Err(_) => return Err("Failed to get context"),
    };
    
    // Get the contract ID (share token ID)
    let contract_id = context.myself.clone();
    
    // Check if there are any incoming alkanes at all
    if incoming_alkanes.0.is_empty() {
        return Err("No shares provided in transaction");
    }
    
    // Sum all incoming shares with matching ID
    let mut received = 0u128;
    
    for transfer in &incoming_alkanes.0 {
        // Direct comparison of AlkaneId structs
        // This works because AlkaneId implements PartialEq
        if transfer.id == contract_id {
            received = received.checked_add(transfer.value)
                .ok_or("Share amount overflow")?;
        }
    }
    
    // If no shares match our contract ID, this is an error
    if received == 0 {
        return Err("Invalid share token ID: received shares do not match expected type");
    }
        
    Ok(received)
}
```

## Implementation Details

### 1. Asset Management Operations

| Operation | Opcode | Description | Parameters |
|-----------|--------|-------------|------------|
| Deposit   | 10     | Convert assets to shares | tx_hash, caller, receiver, assets |
| Mint      | 11     | Issue exact shares      | tx_hash, caller, receiver, shares |
| Withdraw  | 12     | Remove assets from vault | tx_hash, caller, receiver, owner, assets |
| Redeem    | 13     | Burn shares for assets  | tx_hash, caller, receiver, owner, shares |

### 2. View Functions

| Function Type       | Opcode Range | Example Functions                                |
|---------------------|--------------|--------------------------------------------------|
| Metadata            | 100-199      | GetName, GetSymbol, GetDecimals, GetAsset        |
| Accounting          | 200-299      | GetTotalAssets, ConvertToShares, ConvertToAssets |
| Limits              | 300-399      | GetMaxDeposit, GetMaxMint, GetMaxWithdraw        |
| Previews            | 400-499      | PreviewDeposit, PreviewMint, PreviewWithdraw     |
| Custom Data         | 500-599      | SetData, GetData                                 |
| Balance Management  | 600-699      | GetBalanceOf, GetTotalSupply                     |
| Administration      | 900-999      | UpdateYield, GetYieldRate                         |

## WebAssembly Export Architecture

The contract uses a standardized WebAssembly export architecture:

```rust
#[no_mangle]
pub extern "C" fn memory_alloc(size: u32) -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, 1);
        alloc::alloc::alloc(layout)
    }
}

#[no_mangle]
pub extern "C" fn memory_free(ptr: *mut u8, size: u32) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size as usize, 1);
        alloc::alloc::dealloc(ptr, layout);
    }
}

#[no_mangle]
pub extern "C" fn call(ptr: *const u8, len: u32) -> u64 {
    let result = catch_unwind(|| {
        let data = unsafe { Vec::from_raw_parts(ptr as *mut u8, len as usize, len as usize) };
        let response = handle_alkane_message(&data).unwrap_or_else(|e| {
            format!("ERROR: {}", e).into_bytes()
        });
        let response_ptr = response.as_ptr();
        let response_len = response.len();
        std::mem::forget(response);
        ((response_ptr as u64) << 32) | response_len as u64
    });

    match result {
        Ok(result) => result,
        Err(_) => {
            let response = b"PANIC: Unhandled exception in contract";
            let response_ptr = response.as_ptr();
            let response_len = response.len();
            ((response_ptr as u64) << 32) | response_len as u64
        }
    }
}
```

## Testing Strategies

### 1. Mock Vault Pattern for Dependency-Free Testing

The `MockYieldVault` implementation provides a complete vault implementation without external dependencies:

```rust
/// MockYieldVault is a simplified implementation for testing
/// that doesn't rely on external dependencies like alkanes-runtime
pub struct MockYieldVault {
    // In-memory storage
    storage: RefCell<HashMap<String, Vec<u8>>>,
}

impl MockYieldVault {
    /// Helper to set a value in storage
    fn set_value<T: serde::Serialize>(&self, key: &str, value: T) {
        let value_bytes = bincode::serialize(&value).unwrap_or_default();
        self.storage.borrow_mut().insert(key.to_string(), value_bytes);
    }

    /// Helper to get a value from storage
    fn get_value<T: serde::de::DeserializeOwned>(&self, key: &str) -> T 
    where T: Default {
        if let Some(value_bytes) = self.storage.borrow().get(key) {
            bincode::deserialize(value_bytes).unwrap_or_default()
        } else {
            T::default()
        }
    }
    
    // Vault operations implemented for testing
    pub fn deposit(&self, caller: &str, receiver: &str, assets: u128) -> Result<u128, &'static str> {
        // Implementation details...
    }
    
    pub fn redeem(&self, caller: &str, receiver: &str, owner: &str, shares: u128) -> Result<u128, &'static str> {
        // Implementation details...
    }
}
```

This approach allows testing core business logic without WebAssembly dependencies and is valuable for:
- Testing complex scenarios with multiple users and transactions
- Simulating yield accrual over time
- Ensuring mathematical operations are correct
- Developing new features with TDD before implementing in the main contract

### 2. Test Isolation Pattern

Tests use unique storage namespaces to avoid conflicts:

```rust
// Create a unique test ID
let test_id = format!("test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

// Use prefixed storage paths
let prefixed_path = format!("/test/{}/name", test_id);
let test_pointer = StoragePointer::from_keyword(&prefixed_path);
```

### 3. Test Fixtures with AlkaneResponder

Test fixtures implement the AlkaneResponder trait for mocking blockchain context:

```rust
impl AlkaneResponder for PenTestVault {
    fn context(&self) -> Result<Context> {
        if let Some(ref context) = self.mock_context {
            Ok(context.clone())
        } else {
            Err(anyhow!("No mock context provided"))
        }
    }

    fn transaction(&self) -> Vec<u8> {
        Vec::new() // Mock implementation
    }

    fn height(&self) -> u64 {
        self.mock_timestamp.unwrap_or(1000) // Default for testing
    }
}
```

### 4. Simple Utility Tests

For core mathematical functions and platform-independent utilities:

```rust
#[test]
fn test_simple_share_calculation() {
    // Empty vault - 1:1 ratio
    assert_eq!(simple_share_calculation(100, 0, 0), 100);
    
    // Non-empty vault
    // If total_assets = 1000 and total_supply = 500,
    // then 100 assets should be 50 shares
    assert_eq!(simple_share_calculation(100, 1000, 500), 50);
}
```

### 5. Test Categories

Tests are organized by category:
- Simple utility tests (platform-independent functions)
- Mock vault tests (core business logic without external dependencies)  
- Unit tests (specific functions)
- Basic tests (core functionality)
- E2E tests (complete workflows)
- Adversarial tests (security properties)

### 6. Test Script Automation

The `run_working_tests.sh` script automates test execution:

```bash
#!/bin/bash

echo "===== Running Mock Vault Tests ====="
cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu

echo ""
echo "===== Running Simple Utils Tests ====="
cargo test --test simple_utils_test --target x86_64-unknown-linux-gnu
```

## Hardware-Specific Adaptations

### Apple Silicon Support

The build system includes special handling for Apple Silicon:

1. **Architecture Detection**:
   ```bash
   if [ "$(uname -m)" = "arm64" ]; then
       echo "Detected Apple Silicon architecture"
       # Special Apple Silicon handling
   fi
   ```

2. **LLVM Integration**:
   ```bash
   export PATH="/usr/local/opt/llvm/bin:$PATH"
   export CC="/usr/local/opt/llvm/bin/clang"
   export AR="/usr/local/opt/llvm/bin/llvm-ar"
   export RUSTFLAGS="-C embed-bitcode=no"
   ```

3. **Custom Fork for Platform Stability**:
   - Local fork of secp256k1-sys
   - Stub implementations for critical functions
   - Platform-independent build scripts
