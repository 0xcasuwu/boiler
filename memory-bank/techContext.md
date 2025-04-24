# SLOP Technical Context

## Technology Stack

### Core Technologies

- **Rust**: The primary programming language used for all contract development
- **Serde**: Used for serialization and deserialization of data structures
- **HashMap**: Used for efficient key-value storage in the collections
- **Feature Flags**: Used to conditionally compile blockchain-specific functionality

### Blockchain Integration

The project is designed to work both standalone and integrated with blockchain systems:

- **Alkanes Runtime**: Integration with the blockchain runtime when `blockchain` feature is enabled
- **Block-Based Time**: Integration with block system for time management
- **Context-Based Auth**: Transaction context integration for authentication

## Development Environment

### Build System

- **Cargo**: Rust's package manager and build system
- **Feature Flags**: 
  - `blockchain`: Enables blockchain integration features
  - Default mode: Standalone functionality

### Dependencies

Core dependencies include:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Optional dependencies based on features
[dependencies.alkanes_runtime]
version = "0.1"
optional = true

[dependencies.protorune_support]
version = "0.1"
optional = true

[features]
default = []
blockchain = ["alkanes_runtime", "protorune_support"]
```

## Development Constraints

### Memory Usage

The system is designed to be efficient with memory usage:

- HashMap-based storage for collections and bonds
- No unnecessary data duplication
- Clean mappings between orbitals and bonds

### Processing Complexity

All operations are designed with computational efficiency in mind:

- O(1) bond lookup by ID or orbital token ID
- O(n) operations limited to aggregate queries like total value calculation
- No complex calculations in critical paths

### Blockchain Constraints

When operating in blockchain mode:

- Gas optimization for key operations
- Minimized storage operations
- Avoiding complex loops that might hit block gas limits

## Testing Infrastructure

### Test Environments

1. **Unit Tests**: Standard Rust tests for isolated component verification
2. **Integration Tests**: Testing interactions between components
3. **Block Context Tests**: Tests using simulated block heights for time-based behavior

### Testing Tools

- **StandaloneBlockContext**: For simulating block-based timing in tests
- **Mock Bond Collections**: For testing factory operations
- **Test Fixtures**: For setting up common test scenarios

## Code Organization

```
slop/
├── src/
│   ├── models/
│   │   ├── mod.rs          # Models module exports
│   │   └── bond.rs         # Bond data model
│   ├── contracts/
│   │   ├── mod.rs          # Contracts module exports
│   │   ├── bond_curve.rs   # (Optional) Bond curve implementation
│   │   ├── launchpad_factory.rs  # Factory implementation 
│   │   └── orbital_bond_collection.rs  # Collection implementation
│   ├── utils/
│   │   ├── mod.rs          # Utils module exports
│   │   └── block_context.rs  # Block time abstraction
│   └── lib.rs              # Main library entry point
├── examples/
│   └── launchpad_example.rs # Example usage of the system
├── tests/                  # Integration tests
└── Cargo.toml              # Project configuration
```

## Technical Interfaces

### External Interfaces

- **Orbital Token System**: The system interacts with orbital tokens as bonds
- **Block Height Provider**: Provides the current block height for maturity calculations

### Internal Interfaces

1. **BlockContext Trait**:
   ```rust
   pub trait BlockContext {
       fn get_current_block_height(&self) -> u64;
       fn is_block_height_reached(&self, target_height: u64) -> bool;
       fn blocks_remaining(&self, target_height: u64) -> u64;
   }
   ```

2. **LaunchpadFactory Interface**:
   ```rust
   pub struct LaunchpadFactory {
       // ...
   }
   
   impl LaunchpadFactory {
       pub fn new<T: BlockContext>(version: String, ...) -> Self;
       pub fn create_collection<T: BlockContext>(...) -> String;
       pub fn get_collection(&self, collection_id: &str) -> Option<&OrbitalBondCollection>;
       // ...
   }
   ```

3. **OrbitalBondCollection Interface**:
   ```rust
   pub struct OrbitalBondCollection {
       // ...
   }
   
   impl OrbitalBondCollection {
       pub fn new<T: BlockContext>(...) -> Self;
       pub fn mint_bond<T: BlockContext>(...) -> Result<String, &'static str>;
       pub fn redeem_bond<T: BlockContext>(...) -> Result<u64, &'static str>;
       // ...
   }
   ```

## Technical Decisions

### 1. Block-Based Time vs Timestamps

**Decision**: Use block numbers instead of timestamps for determining bond maturity.

**Rationale**:
- More deterministic than timestamps
- Simpler to test and reason about
- Consistent across all validators
- Maps well to blockchain execution model

### 2. Orbital Tokens as Bonds

**Decision**: Make the orbital token itself BE the bond, rather than just a reference.

**Rationale**:
- Simplifies ownership model
- Eliminates complex mappings
- Fits naturally with the possession-based ownership model
- Reduces state complexity

### 3. Feature Flag for Blockchain Integration

**Decision**: Use feature flags to conditionally compile blockchain integration.

**Rationale**:
- Allows for standalone testing without blockchain dependencies
- Simplifies development workflow
- Enables deployment on multiple platforms
- Keeps core logic blockchain-agnostic

### 4. HashMap-Based State Storage

**Decision**: Use standard Rust HashMap for state storage.

**Rationale**:
- Efficient key-value lookups
- Well-understood behavior
- Serializable via serde
- Flexible for both standalone and blockchain use

### 5. Separate Bond Collection Management

**Decision**: Split bond management into collections with a factory pattern.

**Rationale**:
- Better isolation between bond issuances
- Flexible parameters per collection
- Simplifies administration
- More modular codebase

## Security Considerations

### Authentication

- **Orbital Token Ownership**: The system relies on orbital token possession for authentication
- **No User Addresses**: User addresses are never used as part of the authentication flow

### State Protection

- **Active Flag**: Collections have an active flag to prevent unauthorized minting
- **Bond Status**: Bonds track their status to prevent multiple redemptions
- **Immutable Parameters**: Core parameters like interest rates are immutable after creation

### Error Handling

- **Result Type**: All operations that could fail return Rust's Result type
- **Early Validation**: Inputs are validated before state changes
- **Clear Error Messages**: Error messages are descriptive for debugging

## Compatibility

The system is designed to be compatible with:

1. **Standalone Mode**: For testing and development
2. **Blockchain Integration**: For production deployment
3. **Different Interest Rate Models**: Collections can have different parameters
4. **Various Bond Maturity Periods**: Configurable per collection

## Performance Considerations

1. **Optimized State Access**: Minimized state reads and writes
2. **Efficient Data Structures**: HashMaps for O(1) lookups
3. **Minimal Computational Overhead**: Simple calculations where possible
4. **Batch Operations**: Support for batch redemptions (future)
