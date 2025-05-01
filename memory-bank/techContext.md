# Technical Context for Boiler Project

## Alkanes Ecosystem Architecture

### MessageDispatch Derive Macro

The core of the alkanes runtime is the `MessageDispatch` derive macro which processes enums to create WebAssembly-compatible message handling code. Key insights:

1. **Attribute-driven Design**: The macro processes two specific attributes:
   - `#[opcode(n)]` - Assigns a numeric opcode to each variant
   - `#[returns(Type)]` - Specifies the return type for a variant

2. **Usage Pattern**:
   ```rust
   #[derive(MessageDispatch)]
   enum Message {
       #[opcode(0)]
       Initialize { /* params */ },
       
       #[opcode(1)]
       #[returns(u128)]
       GetValue { /* params */ },
   }
   ```

3. **Implementation Details**:
   - The attributes are not separate macros but identifiers processed by MessageDispatch
   - The macro generates code to map numeric opcodes to functions and handle serialization
   - Generated code handles WebAssembly exports and imports automatically

### Alkane Transfer Model

A common pattern across alkane contracts is the `AlkaneTransfer` structure, which represents token transfers:

```rust
pub struct AlkaneTransfer {
    pub id: Vec<u8>,
    pub value: u128,
    pub from: Option<String>,
    pub to: Option<String>,
}
```

This structure is used for both native blockchain transfers and in testing environments.

### BlockContext Pattern

The runtime uses a `BlockContext` trait to abstract blockchain state information:

- Provides methods for getting current block height and checking maturity
- Has implementations for both blockchain and standalone environments
- Uses feature flags for conditional compilation

## WebAssembly Compilation Challenges

Building for WebAssembly introduces unique challenges:

1. **C dependencies**: Libraries with native code components (like secp256k1) require special handling
2. **Target-specific configuration**: Need conditional compilation for wasm32-unknown-unknown target
3. **Build script**: The build.rs needs to create proper WebAssembly artifacts with correct exports

## Blockchain Runtime Requirements

For proper operation on a blockchain, contracts need:

1. **Runtime integration**: Must integrate with alkanes-runtime for blockchain state access
2. **Message dispatch**: All external methods should be exposed through the MessageDispatch system
3. **Feature flags**: Must use conditional compilation for blockchain vs. testing environments

## WebAssembly Export Architecture

The Alkanes ecosystem employs a specific pattern for WebAssembly exports:

1. **CallResponse Pattern**: All methods exposed to WebAssembly should return a `CallResponse` type:
   ```rust
   pub struct CallResponse {
       pub result: Vec<u8>,
       pub transfers: Vec<AlkaneTransfer>,
       pub logs: Vec<String>,
   }
   ```

   This standardized response format enables:
   - Consistent return value handling
   - Support for token transfers as side effects
   - Logging for debugging and monitoring

2. **Binary Wrapper Implementation**:
   - Each contract should have a wrapper structure implementing `Default`
   - `MessageDispatch` derives generate the WebAssembly entry points
   - The wrapper initializes contract state and delegates method calls

3. **Metashrew Integration**:
   - `to_arraybuffer_layout` converts Rust types to WebAssembly-compatible formats
   - WebAssembly memory is managed through specific patterns required by Metashrew
   - Runtime errors must be carefully handled to avoid WebAssembly crashes

## Contract Deployment Architecture

Different architectural approaches exist in the ecosystem:

1. **Free-Mint Style**: Single monolithic contract pattern
   - One WebAssembly module contains all functionality
   - Operations are distinguished by opcode values
   - Simpler deployment model with one transaction ID

2. **SLOP Style**: Multi-contract architecture
   - Separate WebAssembly modules for different contract functions
   - Inter-contract communication handled through transaction IDs
   - More complex orchestration but better separation of concerns

3. **Build Script Operations**:
   - Feature flags control which functionality is included
   - Compiled WebAssembly is compressed and serialized for deployment
   - Test files are auto-generated for integration testing
