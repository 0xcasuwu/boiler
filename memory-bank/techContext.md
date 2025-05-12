# Technical Context for Bitcoin Smart Contracts

## Bitcoin Smart Contract Environment

Bitcoin smart contracts operate in a constrained environment compared to other blockchains. These constraints inform our design decisions and implementation patterns.

### Core Technologies

1. **Bitcoin Script**
   - Stack-based execution model
   - Limited opcodes
   - No loops or complex control flow
   - Hard size limit on scripts

2. **Taproot Upgrade**
   - Schnorr signatures
   - MAST (Merkelized Abstract Syntax Tree)
   - Pay-to-Taproot outputs
   - Improved script execution

3. **Alkane Framework**
   - Smart contract abstraction layer
   - Context access for blockchain data
   - Storage management system
   - Message dispatch pattern

4. **WebAssembly Integration**
   - Contract compilation to WASM
   - Binary size optimization
   - Export function architecture
   - Memory management

### Constraints and Requirements

1. **Storage Limitations**
   - No native key-value storage
   - Limited total script size
   - Need for efficient serialization

2. **Execution Constraints**
   - Deterministic execution
   - No floating-point operations
   - Limited execution steps
   - All operations must have predictable gas cost

3. **Authentication**
   - Asymmetric cryptography for ownership
   - No accounts system like Ethereum
   - Ownership verification through AlkaneIds

4. **Cross-Platform Compatibility**
   - WebAssembly compilation
   - Architecture-specific dependencies
   - Platform-dependent build processes

### Yield-bearing Vault Technical Requirements

1. **Asset Tracking**
   - Account balance management
   - Total supply and assets tracking
   - Share price calculation

2. **Yield Accrual**
   - Time-based calculation
   - Share price adjustment
   - Non-dilutive yield

3. **Security Requirements**
   - Overflow protection
   - Replay attack prevention
   - Authorization validation
   - Initialization guards

## Implementation Stack

### Programming Language

**Rust**
- Memory safety and ownership model
- No runtime errors
- Zero-cost abstractions
- Excellent WebAssembly support

**Key Rust Features Used:**
- Result type for error handling
- Traits for behavior abstraction
- Pattern matching for control flow
- No-std compatibility
- Checked arithmetic operations

### Build System

**Cargo**
- Package management
- Dependency resolution
- Custom build scripts
- Conditional compilation
- WebAssembly target support

**Custom Build Requirements:**
- WebAssembly cross-compilation
- Local dependency forks
- Platform-specific optimizations
- LLVM integration for Apple Silicon

### WebAssembly

**Compilation Target**
- wasm32-unknown-unknown
- No standard library
- Predictable memory model
- Binary size constraints

**Export Architecture**
- Memory allocation/deallocation
- Message handler functions
- Error handling and panic recovery
- Binary data serialization

### Testing Framework

**Test Categories**
- Unit tests for isolated functionality
- Basic tests for core operations
- End-to-end tests for workflows
- Adversarial tests for security properties

**Test Infrastructure**
- Mock context generation
- Storage isolation
- Timestamp-based unique test IDs
- Error simulation
- Parameterized tests for edge cases

### Deployment Infrastructure

**OylNet Testnet**
- Contract deployment
- Transaction execution
- Block generation
- State verification

**Toolchain**
- Custom deployment scripts
- Contract initialization
- Interaction framework
- State management

## Key Dependencies

### secp256k1-sys Crate

This crate provides Rust bindings to the libsecp256k1 C library, essential for cryptographic operations. We've had to create a custom fork due to compatibility issues with Apple Silicon.

**Issues Identified:**
- Compilation errors on Apple Silicon when targeting WebAssembly
- Missing target configuration for arm64-apple-darwin
- Dependencies on native C libraries that fail cross-compilation

**Our Solution:**
- Create local fork with stub implementations
- Use [patch] section in Cargo.toml
- Implement minimal no-op versions of required functions
- Optimize fork for WebAssembly size

### Other Core Dependencies

1. **anyhow**
   - Error handling and propagation
   - Contextual error information
   - Simplified error conversion

2. **hex**
   - Encoding/decoding for byte arrays
   - Human-readable debug output
   - Parameter parsing

3. **serde**
   - Serialization for complex data structures
   - JSON encoding/decoding
   - Type-safe data conversion

## Authentication Model

Our contract uses a dual-mode authentication system based on AlkaneIds:

### 1. Test Mode Authentication

For testing environments, the contract accepts specific block/tx values:

```rust
// Test mode with fixed values
let is_token_valid = (block == 1 && tx == 1);
```

When interacting with the contract in test mode:
```bash
# Parameters: tx_hash, block, tx, assets
local params="0x${tx_hash},1,1,${assets}"
```

### 2. Production Authentication

For production environments, the contract validates AlkaneId string representations:

```rust
// Production mode with actual AlkaneId validation
let transfer_id_str = format!("{:?}", transfer.id);
let is_token_valid = (transfer_id_str == auth_token_id);
```

## Platform-Specific Considerations

### Apple Silicon (M1/M2/M3)

Building WebAssembly on Apple Silicon requires special handling:

1. **LLVM Installation**
   ```bash
   arch -x86_64 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
   arch -x86_64 /usr/local/bin/brew install llvm
   ```

2. **Environment Configuration**
   ```bash
   export PATH="/usr/local/opt/llvm/bin:$PATH"
   export CC="/usr/local/opt/llvm/bin/clang"
   export AR="/usr/local/opt/llvm/bin/llvm-ar"
   export RUSTFLAGS="-C embed-bitcode=no"
   ```

3. **Special Build Scripts**
   - `build_minimal.sh`: Creates placeholder WebAssembly
   - `final_fork_build.sh`: Uses local fork with platform detection
   - `build_with_fork.sh`: Manual control over build process

### Cross-Platform Compatibility

We ensure compatibility across platforms through:

1. **Abstraction Layers**
   - Platform-agnostic code with conditional compilation
   - Architecture-specific build configurations
   - Dependency isolation

2. **Minimal Dependencies**
   - Local forks of problematic dependencies
   - Reduced external dependencies
   - Platform-neutral libraries where possible

3. **Standardized Build Process**
   - Unified build scripts
   - Platform detection
   - Proper error handling and fallbacks

## WebAssembly Best Practices

### Size Optimization

- Remove debugging information
- Optimize for size (`-Oz`)
- Eliminate unused functions
- Minimize string data
- Reduce dependency footprint

### Memory Management

- Proper deallocation of resources
- Controlled buffer sizes
- Clear ownership semantics
- Careful pointer manipulation
- Safe conversion between Rust and WebAssembly types

### Export Architecture

- Consistent function signatures
- Error handling for all exports
- Panic recovery
- Proper memory handling
- Clear input/output formats

## Storage Patterns

### Path-based Storage

All contract state is stored using standardized paths:

```rust
pub fn balance_pointer(account: &str) -> StoragePointer {
    StoragePointer::from_keyword(&format!("/balances/{}", account))
}

pub fn yield_rate_pointer() -> StoragePointer {
    StoragePointer::from_keyword("/yield-rate")
}
```

### Storage Serialization

Storage operations use serialization for type safety:

```rust
// Reading with type conversion
let total_supply = total_supply_pointer().get::<u128>().unwrap_or(0);

// Writing with type preservation
total_supply_pointer().set(&new_total_supply);
```

## Testing Considerations

### Test Isolation

Tests require proper isolation to prevent interference:

```rust
// Create unique test ID
let test_id = format!("test_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());

// Use prefixed storage
let test_name_pointer = StoragePointer::from_keyword(&format!("/test/{}/name", test_id));
```

### Mock Context

Tests use mock context to simulate blockchain environment:

```rust
let mut mock_context = Context {
    incoming_alkanes: Alkanes(vec![
        Alkane {
            id: AlkaneId { block: 1, tx: 1 },
            amount: 1000,
            scriptpubkey: vec![],
        }
    ]),
    outgoing_alkanes: Alkanes(vec![]),
    this_block: 1000,
};
```

### AlkaneResponder Implementation

Test fixtures implement the AlkaneResponder trait:

```rust
impl AlkaneResponder for TestVault {
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

## Deployment and Network Integration

### OylNet Deployment

Contracts are deployed to OylNet using:

```bash
# Deploy contract to OylNet
./deploy_to_oylnet.sh

# Interact with deployed contract
./interact_with_vault.sh
```

### Contract Interaction

Interaction with the contract is done through opcode-based messages:

```bash
# Execute a view function (GetName - opcode 100)
oyl alkane execute -p oylnet --calldata "100"

# Execute a write function (Deposit - opcode 10)
oyl alkane execute -p oylnet --calldata "10,0x${tx_hash},1,1,1000000"
```

### Numeric Parameter Format

When interacting with contracts, use numeric parameters for AlkaneId:

```bash
# Before (causing errors):
local auth_token="auth_token_123"
local params="0x${tx_hash},\"${auth_token}\",${assets}"

# After (working correctly):
local block=1
local tx=1
local params="0x${tx_hash},${block},${tx},${assets}"
```

## Error Handling Strategy

### Result-based Error Handling

All operations return Result types:

```rust
pub fn deposit(context: &Context, tx_hash: &[u8], caller: &str, receiver: &str, assets: u128) -> Result<u128> {
    validate_transaction_hash(tx_hash)?;
    check_authorization(context, caller)?;
    // ...
}
```

### RUN TESTS

eg cargo test --test mock_vault_test --target x84_64-unknown-linux-gnu

### Contextual Errors

Errors include context testfor better diagnostics:

```rust
Err(anyhow!("Deposit exceeds maximum of {}", max_deposit))
```

### Error Recovery

WebAssembly exports include error recovery:

```rust
let response = handle_alkane_message(&data).unwrap_or_else(|e| {
    format!("ERROR: {}", e).into_bytes()
});
```

### Panic Handling

Exports catch panics to prevent WebAssembly crashes:

```rust
let result = catch_unwind(|| {
    // Function logic
});

match result {
    Ok(result) => result,
    Err(_) => {
        let response = b"PANIC: Unhandled exception in contract";
        // Return error response
    }
}
