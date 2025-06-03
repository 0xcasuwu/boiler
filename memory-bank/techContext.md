# Technical Context - ALK4626 Vault Factory

## Technology Stack

### **Blockchain Platform**
- **Alkanes Protocol**: WebAssembly-based smart contract platform
- **Consensus**: Bitcoin-secured blockchain with Protorune integration
- **Runtime**: Custom Alkanes runtime with message dispatching

### **Programming Language & Framework**
- **Language**: Rust (stable channel)
- **Runtime**: alkanes-runtime framework
- **Macros**: alkanes_runtime::declare_alkane! for contract declaration
- **Message System**: alkanes-runtime::message::MessageDispatch for opcode routing

### **Key Dependencies**
```rust
// Core Alkanes dependencies
alkanes-runtime = { version = "0.1.0", features = ["full"] }
alkanes-support = { version = "0.1.0" }
alkanes_support = { version = "0.1.0" }

// Protocol integration
protorune = { version = "0.1.0" }
protorune-support = { version = "0.1.0" }

// Utilities
anyhow = "1.0"              // Error handling
hex = "0.4"                 // Hex encoding for storage keys
metashrew-support = "0.1.0" // Core support utilities
```

### **Testing Framework**
```rust
// WebAssembly testing
wasm-bindgen-test = "0.3"

// Bitcoin protocol testing
bitcoin = "0.30"
ordinals = "0.15"

// Standard testing
protobuf = "3.0"  // For message parsing
```

## Development Environment

### **Build System**
- **Target**: `wasm32-unknown-unknown` (WebAssembly)
- **Build Tool**: Cargo with custom build.rs
- **Test Runner**: wasm-bindgen-test-runner
- **Compilation**: Rust → WebAssembly → Alkanes blockchain

### **Project Structure**
```
boiler/
├── alkanes/                          # Contract implementations
│   ├── alk4626-vault-factory/        # Main vault contract
│   └── alk4626-position-token/       # Position token contract
├── src/
│   ├── precompiled/                  # Pre-built contract bytes
│   └── tests/                        # Integration test suite
├── scripts/                          # Deployment scripts
└── memory-bank/                      # Documentation system
```

### **Critical Files**
- `alkanes/alk4626-vault-factory/src/lib.rs` - Main vault implementation
- `alkanes/alk4626-position-token/src/lib.rs` - Position token implementation
- `src/tests/vault_factory.rs` - Comprehensive test suite
- `src/precompiled/free_mint_build.rs` - Test token generator

## Technical Constraints

### **WebAssembly Limitations**
- No standard library features requiring system calls
- Limited memory allocation patterns
- Deterministic execution requirements
- No async/await or threading

### **Alkanes Protocol Constraints**
- Fixed fuel limits for contract execution
- Specific storage key patterns (`/key` format)
- Message dispatching via opcode routing
- Transaction-based execution model

### **Blockchain Constraints**
- Immutable contract code after deployment
- Gas/fuel costs for all operations
- Transaction ordering dependencies
- Block height-based timing

## Tool Usage Patterns

### **Testing Commands**
```bash
# Run specific test
cargo test test_deployment

# Run with trace output
cargo test test_deployment -- --nocapture

# Run all vault factory tests
cargo test vault_factory

# Build contracts
cargo build --target wasm32-unknown-unknown
```

### **Development Workflow**
1. **Contract Development**: Write Rust contract code
2. **Local Testing**: Run wasm-bindgen-test suite
3. **Integration Testing**: Full blockchain simulation tests
4. **Trace Analysis**: Examine transaction trace logs
5. **Deployment**: Deploy to Alkanes testnet/mainnet

### **Key Debugging Tools**
- **Trace Logs**: `alkanes::view::trace()` for execution analysis
- **Balance Sheets**: `protorune::balance_sheet::load_sheet()` for token tracking
- **Storage Inspection**: Direct storage key reading via `self.load()`
- **Console Output**: `metashrew_core::println!()` for debugging

## Architecture-Specific Patterns

### **Message Dispatching**
```rust
#[derive(MessageDispatch)]
enum VaultFactoryMessage {
    #[opcode(0)]
    Initialize { reward_per_block: u128, start_block: u128, /* ... */ },
    
    #[opcode(1)]
    Deposit { assets: u128 },
    
    #[opcode(4)]
    WithdrawFees { auth_token_count: u128 },
    
    #[opcode(14)]
    #[returns(u128)]
    GetFeePercentage,
}
```

### **Storage Patterns**
```rust
// Direct storage access (preferred)
fn fee_percentage(&self) -> u128 {
    self.load_u128("/fee_percentage")
}

fn set_fee_percentage(&self, fee_percentage: u128) {
    self.store("/fee_percentage".as_bytes().to_vec(), 
               fee_percentage.to_le_bytes().to_vec());
}
```

### **Inter-Contract Communication**
```rust
// Factory pattern for position token creation
let cellpack = Cellpack {
    target: AlkaneId { block: 6, tx: POSITION_TOKEN_TEMPLATE_ID },
    inputs: vec![0x0, position_id, assets, shares, current_block],
};

let create_response = self.call(&cellpack, &position_parcel, self.fuel())?;
```

### **Token Transfer Patterns**
```rust
// Alkanes native token transfers
response.alkanes.0.push(AlkaneTransfer {
    id: token_id,
    value: amount,
});

// Protorune/Protobuf integration for Bitcoin protocol
let runestone = Runestone {
    edicts: vec![], // Critical: empty for input-based auth
    protocol: Some(protostones.encipher()?),
    // ... other fields
};
```

## Performance Considerations

### **Gas/Fuel Optimization**
- Minimize storage operations (most expensive)
- Use checked arithmetic for safety without panic overhead
- Batch operations when possible
- Prefer staticcall over call when no state changes needed

### **Memory Management**
- Use `Vec::with_capacity()` for known-size allocations
- Avoid unnecessary string operations
- Prefer primitive types over complex structures
- Cache frequently accessed storage values

### **Testing Optimization**
- Use helper functions to reduce test code duplication
- Mock external dependencies when possible
- Parallelize independent test cases
- Use targeted trace analysis instead of full logging

## Security Patterns

### **Input Validation**
```rust
// Always validate inputs
if assets == 0 {
    return Err(anyhow!("Cannot deposit zero assets"));
}

if fee_percentage > 10000 {
    return Err(anyhow!("Fee percentage cannot exceed 10000 (100%)"));
}
```

### **Overflow Protection**
```rust
// Use checked arithmetic throughout
let new_total = self.total_assets()
    .checked_add(assets)
    .ok_or_else(|| anyhow!("Total assets overflow"))?;
```

### **Authentication Verification**
```rust
// Verify token types and registry membership
if !self.is_position_in_registry(&transfer.id) {
    return Err(anyhow!("Token is not a registered position token"));
}
