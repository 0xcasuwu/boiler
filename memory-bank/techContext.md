# Bitcoin Smart Contract Technical Context

## Core Technologies

### Rust and WebAssembly Foundation

Bitcoin smart contracts in this architecture are built using Rust compiled to WebAssembly, enabling execution in compatible blockchain environments:

- **Rust** - Memory-safe systems programming language
- **WebAssembly (WASM)** - Portable binary format for contract execution
- **Alkanes Framework** - Smart contract framework for Bitcoin
- **MessageDispatch** - Macro for opcode-based message handling

### Key Dependencies

```toml
[dependencies]
alkanes-support = "0.1.0"         # Core support library
alkanes-runtime = "0.1.0"         # Runtime for contract execution
metashrew-support = "0.1.0"       # Metashrew protocol compatibility
protorune-support = "0.1.0"       # Protorune protocol support
alkane-factory-support = "0.1.0"  # Factory pattern support
ordinals = "0.1.0"                # Ordinals protocol integration
anyhow = "1.0"                    # Error handling
bitcoin = "0.30"                  # Bitcoin protocol implementation
serde = { version = "1.0", features = ["derive"] }  # Serialization framework
serde_json = "1.0"                # JSON support for complex data
```

## Technical Implementation

### MessageDispatch Implementation

The `MessageDispatch` derive macro is a cornerstone of the architecture. It processes enum definitions to create WebAssembly-compatible message handling:

```rust
// The MessageDispatch derives generates code that:
// 1. Creates handler methods for each enum variant
// 2. Implements a dispatch method for routing opcodes
// 3. Handles serialization/deserialization of parameters and return values
#[derive(MessageDispatch)]
enum MintableAlkaneMessage {
    #[opcode(0)]
    Initialize { 
        units: u8, 
        value_per_mint: u128,
        cap: u128, 
        name: String, 
        symbol: String 
    },
    
    #[opcode(77)]
    Mint { tx_hash: String },
    
    // Additional operations...
    
    #[opcode(99)]
    #[returns(String)]
    GetName {},
}
```

#### Generated Handler Methods

For each enum variant, the macro generates a handler method with the appropriate signature:

```rust
// Generated handler for Initialize variant
fn handle_initialize(
    &mut self, 
    units: u8, 
    value_per_mint: u128,
    cap: u128, 
    name: String, 
    symbol: String
) -> Result<(), &'static str> {
    // Implement initialization logic
}

// Generated handler for GetName variant
fn handle_get_name(&self) -> String {
    // Return the name value
}
```

#### Dispatch Implementation

The macro also generates the dispatch method that routes opcodes to handlers:

```rust
// The generated dispatch method
fn dispatch(&mut self, opcode: i32, bytes: *mut u8, bytes_len: usize) -> *mut u8 {
    match opcode {
        0 => {
            // Deserialize parameters for Initialize
            let params = deserialize_params::<InitializeParams>(bytes, bytes_len);
            let result = self.handle_initialize(
                params.units,
                params.value_per_mint,
                params.cap,
                params.name,
                params.symbol
            );
            // Serialize and return result
            serialize_result(result)
        },
        77 => {
            // Handle Mint operation
        },
        // Other opcodes...
        _ => {
            // Handle unknown opcode
            serialize_result::<(), &str>(Err("Unknown opcode"))
        }
    }
}
```

### Storage Implementation

The storage system uses key-value pairs with consistent path conventions:

```rust
// Storage access helpers
fn get_u128(path: &str) -> u128 {
    storage::get_u128(path).unwrap_or(0)
}

fn set_u128(path: &str, value: u128) {
    storage::set_u128(path, value);
}

fn get_string(path: &str) -> String {
    storage::get_string(path).unwrap_or_default()
}

fn set_string(path: &str, value: &str) {
    storage::set_string(path, value);
}

fn get_bool(path: &str) -> bool {
    storage::get_bool(path).unwrap_or(false)
}

fn set_bool(path: &str, value: bool) {
    storage::set_bool(path, value);
}
```

#### Complex Data Storage Pattern

For complex data structures like HashSets, the contract uses JSON serialization:

```rust
// Retrieve and deserialize a complex data structure
fn get_transaction_hashes() -> HashSet<String> {
    let json = storage::get_string("/tx-hashes").unwrap_or_default();
    if json.is_empty() {
        HashSet::new()
    } else {
        serde_json::from_str(&json).unwrap_or_default()
    }
}

// Update and serialize a complex data structure
fn set_transaction_hashes(hashes: &HashSet<String>) -> Result<(), &'static str> {
    let json = serde_json::to_string(hashes)
        .map_err(|_| "Failed to serialize transaction hashes")?;
    storage::set_string("/tx-hashes", &json);
    Ok(())
}
```

### WebAssembly Export Architecture

The contract is compiled to WebAssembly and exports a standardized interface:

```rust
// WebAssembly entry point
#[no_mangle]
pub extern "C" fn call(opcode: i32, bytes: *mut u8, bytes_len: usize) -> *mut u8 {
    // Delegate to the MessageDispatch-generated code
    MintableAlkane::default().dispatch(opcode, bytes, bytes_len)
}
```

#### Memory Management

WebAssembly memory is handled through specific patterns:

```rust
// Deserialize parameters from WebAssembly memory
fn deserialize_params<T: DeserializeOwned>(bytes: *mut u8, bytes_len: usize) -> T {
    let slice = unsafe { std::slice::from_raw_parts(bytes, bytes_len) };
    serde_json::from_slice(slice).unwrap_or_else(|_| panic!("Failed to deserialize parameters"))
}

// Serialize result to WebAssembly memory
fn serialize_result<T: Serialize, E: Display>(result: Result<T, E>) -> *mut u8 {
    let result_json = match result {
        Ok(value) => {
            let value_json = serde_json::to_vec(&value)
                .unwrap_or_else(|_| panic!("Failed to serialize success value"));
            serde_json::to_vec(&Ok::<Vec<u8>, String>(value_json))
                .unwrap_or_else(|_| panic!("Failed to serialize success result"))
        },
        Err(err) => {
            serde_json::to_vec(&Err::<Vec<u8>, String>(err.to_string()))
                .unwrap_or_else(|_| panic!("Failed to serialize error result"))
        }
    };
    
    // Allocate memory for the result
    let result_ptr = allocate(result_json.len());
    
    // Copy the result to WebAssembly memory
    unsafe {
        std::ptr::copy_nonoverlapping(
            result_json.as_ptr(),
            result_ptr as *mut u8,
            result_json.len()
        );
    }
    
    result_ptr as *mut u8
}
```

### Transaction Hash Tracking Implementation

The contract implements transaction hash tracking to enforce one mint per transaction:

```rust
fn validate_and_track_transaction(tx_hash: &str) -> Result<(), &'static str> {
    // Retrieve the current set of transaction hashes
    let mut tx_hashes = get_transaction_hashes();
    
    // Check if this transaction hash has been used
    if tx_hashes.contains(tx_hash) {
        return Err("Transaction hash already used");
    }
    
    // Add the transaction hash to the set
    tx_hashes.insert(tx_hash.to_string());
    
    // Store the updated set
    set_transaction_hashes(&tx_hashes)?;
    
    Ok(())
}
```

### CallResponse Pattern

For blockchain execution, the contract uses the `CallResponse` pattern:

```rust
pub struct CallResponse {
    pub result: Vec<u8>,         // Serialized result data
    pub transfers: Vec<Transfer>, // Token transfers
    pub logs: Vec<String>,        // Event logs
}

// Create a response for the blockchain runtime
fn create_response<T: Serialize>(result: T, transfers: Vec<Transfer>, logs: Vec<String>) -> CallResponse {
    let result_bytes = serde_json::to_vec(&result).unwrap_or_default();
    
    CallResponse {
        result: result_bytes,
        transfers,
        logs,
    }
}
```

## Build and Compilation

### Library Configuration

The contract is configured as both a cdylib (for WebAssembly) and rlib (for testing):

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = []
blockchain = ["alkanes-runtime/blockchain", "alkanes-support/blockchain"]
std = ["serde/std", "serde_json/std"]
```

### Build Script

A build script prepares the WebAssembly binary for deployment:

```rust
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
    
    // Set optimization level for WebAssembly
    println!("cargo:rustc-flag=-Copt-level=3");
    println!("cargo:rustc-flag=-Clto=true");
}
```

### Conditional Compilation

Feature flags control functionality based on the target environment:

```rust
#[cfg(feature = "blockchain")]
fn get_transaction_hash() -> String {
    // Use blockchain-specific API
    alkanes_runtime::get_transaction_hash()
}

#[cfg(not(feature = "blockchain"))]
fn get_transaction_hash() -> String {
    // Use mock implementation for testing
    "test_tx_hash".to_string()
}
```

## Contract Execution Flow

### Initialization Flow

```rust
fn initialize(
    &mut self,
    units: u8, 
    value_per_mint: u128,
    cap: u128, 
    name: String, 
    symbol: String
) -> Result<(), &'static str> {
    // Check if already initialized
    self.observe_initialization()?;
    
    // Store the parameters
    storage::set_u8("/units", units);
    storage::set_u128("/value-per-mint", value_per_mint);
    storage::set_u128("/cap", cap);
    storage::set_string("/name", &name);
    storage::set_string("/symbol", &symbol);
    
    // Initialize other state
    storage::set_u128("/totalsupply", 0);
    storage::set_u128("/minted", 0);
    
    Ok(())
}
```

### Mint Flow

```rust
fn mint(&mut self, tx_hash: &str) -> Result<(), &'static str> {
    // Validate transaction hash
    self.validate_and_track_transaction(tx_hash)?;
    
    // Get current values
    let value_per_mint = self.value_per_mint();
    let minted = self.minted();
    let cap = self.cap();
    
    // Validate cap
    self.validate_cap(minted, cap)?;
    
    // Update state
    let new_minted = minted.checked_add(1).ok_or("Minted overflow")?;
    storage::set_u128("/minted", new_minted);
    
    // Update total supply
    let total = self.total_supply();
    let new_total = total.checked_add(value_per_mint).ok_or("Total supply overflow")?;
    storage::set_u128("/totalsupply", new_total);
    
    Ok(())
}
```

### View Function Flow

```rust
fn name(&self) -> String {
    storage::get_string("/name").unwrap_or_default()
}

fn symbol(&self) -> String {
    storage::get_string("/symbol").unwrap_or_default()
}

fn total_supply(&self) -> u128 {
    storage::get_u128("/totalsupply").unwrap_or(0)
}

fn minted(&self) -> u128 {
    storage::get_u128("/minted").unwrap_or(0)
}

fn cap(&self) -> u128 {
    storage::get_u128("/cap").unwrap_or(0)
}
```

## Technical Constraints

### WebAssembly Size Considerations

WebAssembly modules deployed to Bitcoin have size constraints:

- Minimize dependencies to reduce compiled size
- Avoid large standard library functions
- Use feature flags to exclude unnecessary code
- Optimize for size in the build process

```toml
[profile.release]
opt-level = 's'       # Optimize for size
lto = true            # Link-time optimization
codegen-units = 1     # Maximize optimization
panic = 'abort'       # Smaller panic handler
strip = true          # Strip debug symbols
```

### Memory Management Considerations

WebAssembly memory management requires careful handling:

- Use fixed-size buffers where possible
- Minimize allocations and deallocations
- Implement efficient serialization/deserialization
- Handle memory leaks carefully, especially in error paths
- Use stack allocation for small objects

### Error Handling Constraints

Error handling in WebAssembly contracts has limitations:

- Use static string errors for consistent memory management
- Avoid panics in production code
- Use Result types consistently
- Provide clear, concise error messages
- Propagate errors up the call stack using the `?` operator

## Integration Points

### Factory Integration

The contract implements the MintableToken trait:

```rust
impl MintableToken for MintableAlkane {
    fn name(&self) -> String {
        self.name()
    }
    
    fn symbol(&self) -> String {
        self.symbol()
    }
    
    fn total_supply(&self) -> u128 {
        self.total_supply()
    }
    
    fn get_data(&self, key: &str) -> Option<String> {
        self.get_data(key)
    }
    
    fn observe_initialization(&self) -> Result<(), &'static str> {
        if storage::get_bool("/initialized").unwrap_or(false) {
            return Err("Already initialized");
        }
        storage::set_bool("/initialized", true);
        Ok(())
    }
}
```

### Blockchain Runtime Integration

The contract integrates with the blockchain runtime:

```rust
#[cfg(feature = "blockchain")]
fn execute_mint(&mut self, tx_hash: &str) -> CallResponse {
    match self.mint(tx_hash) {
        Ok(_) => {
            let value_per_mint = self.value_per_mint();
            let receiver = alkanes_runtime::get_transaction_sender();
            
            // Create transfer record
            let transfer = Transfer {
                id: tx_hash.as_bytes().to_vec(),
                value: value_per_mint,
                from: None,
                to: Some(receiver),
            };
            
            // Create success response with transfer
            create_response(
                "success",
                vec![transfer],
                vec!["Mint successful".to_string()]
            )
        },
        Err(e) => {
            // Create error response with no transfers
            create_response(
                e,
                vec![],
                vec![format!("Mint failed: {}", e)]
            )
        }
    }
}
```

## Testing Architecture

### Unit Testing

Tests for individual components use the rlib version:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initialization() {
        let mut token = MintableAlkane::default();
        
        // Initialize the token
        let result = token.initialize(18, 1000, 1000000, "Test Token", "TST");
        assert!(result.is_ok());
        
        // Check the initialized state
        assert_eq!(token.name(), "Test Token");
        assert_eq!(token.symbol(), "TST");
        assert_eq!(token.cap(), 1000000);
        assert_eq!(token.value_per_mint(), 1000);
    }
    
    // Additional tests...
}
```

### Integration Testing

Integration tests use the compiled WebAssembly:

```rust
#[test]
fn test_wasm_initialization() {
    // Load the WebAssembly module
    let module = load_wasm_module("free_mint.wasm");
    
    // Create parameters for initialization
    let params = json!({
        "units": 18,
        "value_per_mint": 1000,
        "cap": 1000000,
        "name": "Test Token",
        "symbol": "TST"
    });
    
    // Call the initialization function
    let result = call_wasm_function(module, 0, params);
    assert!(result.is_ok());
    
    // Call the view functions to verify state
    let name = call_wasm_function(module, 99, json!({}));
    assert_eq!(name.unwrap(), "Test Token");
    
    let symbol = call_wasm_function(module, 100, json!({}));
    assert_eq!(symbol.unwrap(), "TST");
}
```

### Mock Storage

Testing uses mock storage implementations:

```rust
#[cfg(test)]
mod mock_storage {
    use std::collections::HashMap;
    use std::sync::Mutex;
    
    lazy_static! {
        static ref STORAGE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
    }
    
    pub fn get_u128(path: &str) -> Option<u128> {
        let storage = STORAGE.lock().unwrap();
        storage.get(path).map(|bytes| {
            let mut buf = [0u8; 16];
            buf.copy_from_slice(bytes);
            u128::from_le_bytes(buf)
        })
    }
    
    pub fn set_u128(path: &str, value: u128) {
        let mut storage = STORAGE.lock().unwrap();
        storage.insert(path.to_string(), value.to_le_bytes().to_vec());
    }
    
    // Other storage functions...
    
    pub fn clear() {
        let mut storage = STORAGE.lock().unwrap();
        storage.clear();
    }
}
```

## Deployment Workflow

### Build Process

The contract is built using a WebAssembly-targeted build script:

```bash
#!/bin/bash
# Build for WebAssembly target
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release --features "blockchain"

# Optimize the WebAssembly binary
wasm-opt -Oz -o free_mint_opt.wasm target/wasm32-unknown-unknown/release/free_mint.wasm

# Create the deployment package
echo "Creating deployment package..."
cp free_mint_opt.wasm free_mint.wasm
```

### Deployment Process

The contract is deployed through a blockchain-specific deployment tool:

```javascript
async function deployContract() {
  const wasmCode = fs.readFileSync('./free_mint.wasm');
  
  // Create deployment transaction
  const deployTx = await createDeploymentTransaction(wasmCode);
  
  // Sign and broadcast transaction
  const txId = await signAndBroadcast(deployTx);
  
  console.log(`Contract deployed with transaction ID: ${txId}`);
  return txId;
}

async function initializeContract(contractId, params) {
  // Create initialization transaction
  const initTx = await createCallTransaction(contractId, 0, params);
  
  // Sign and broadcast transaction
  const txId = await signAndBroadcast(initTx);
  
  console.log(`Contract initialized with transaction ID: ${txId}`);
  return txId;
}
```

### Contract Usage

Clients interact with the contract through opcode calls:

```javascript
async function mintToken(contractId, txHash) {
  // Create mint transaction
  const mintTx = await createCallTransaction(contractId, 77, { tx_hash: txHash });
  
  // Sign and broadcast transaction
  const txId = await signAndBroadcast(mintTx);
  
  console.log(`Token minted with transaction ID: ${txId}`);
  return txId;
}

async function getTokenInfo(contractId) {
  // Get token name
  const name = await callViewFunction(contractId, 99, {});
  
  // Get token symbol
  const symbol = await callViewFunction(contractId, 100, {});
  
  // Get token total supply
  const totalSupply = await callViewFunction(contractId, 101, {});
  
  return { name, symbol, totalSupply };
}
```

## Performance Considerations

### Storage Optimization

The contract optimizes storage usage:

- Minimal storage keys for core state
- JSON serialization for complex data structures
- Efficient string encoding/decoding
- Lazy initialization of collections
- Clear data organization with predictable paths

### Computational Efficiency

The contract optimizes computation:

- Uses efficient HashSet for O(1) lookups
- Implements checked arithmetic for safety
- Minimizes string operations
- Uses slice operations where possible
- Employs early returns for validation checks

### Memory Efficiency

The contract optimizes memory usage:

- Minimal state in contract structure
- Efficient serialization/deserialization
- Stack allocation for small objects
- Proper memory management in error paths
- Careful handling of WebAssembly memory constraints
