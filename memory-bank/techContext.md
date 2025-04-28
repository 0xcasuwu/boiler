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

### Authentication & Authorization

- **Orbital Token Ownership**: The system relies on orbital token possession for authentication ("orbital token IS the bond" model)
- **Transaction Context-Based Verification**: Secure redemption requires cryptographic proof of token ownership
- **No User Addresses**: User addresses are never used as part of the authentication flow
- **Validation Order**: Token ownership verification happens before any other checks to prevent information leakage
- **Generic Error Messaging**: Error messages are designed to not leak state information to unauthorized users
- **Cross-Collection Protection**: Guards against using tokens from one collection to redeem in another
- **Single Redemption Pathway**: Unified secure redemption method provides single source of truth for security checks

### State Protection

- **Active Flag**: Collections have an active flag to prevent unauthorized minting
- **Bond Status**: Bonds track their status to prevent multiple redemptions
- **Immutable Parameters**: Core parameters like interest rates are immutable after creation
- **Mapping Cleanup**: Token-to-bond mappings are removed after redemption to prevent reuse
- **Checks-Effects-Interactions Pattern**: State changes happen before external interactions to prevent re-entrancy attacks
- **Early State Updates**: Bond status updated to "Redeemed" before calculating redemption amounts
- **State Isolation**: State changes are atomic and completed before external interactions

### Mathematical Security

- **Overflow Protection**: All mathematical operations use safeguards against integer overflow and underflow
- **U128 Intermediates**: Financial calculations use u128 for intermediate values to prevent overflow
- **Safe Type Conversions**: Explicit checks when converting between numeric types
- **Boundary Checking**: All inputs have explicit boundary checking for extreme values
- **Saturation Logic**: Values that would overflow are capped at their maximum rather than wrapping
- **Edge Case Handling**: Special handling for division-by-zero and other mathematical edge cases

### Error Handling

- **Result Type**: All operations that could fail return Rust's Result type
- **Early Validation**: Inputs are validated before state changes
- **Clear Error Messages**: Error messages are descriptive for debugging but don't leak sensitive information
- **Exhaustive Error Checking**: All potential error conditions are explicitly checked
- **Information Leakage Prevention**: Error messages carefully crafted to avoid exposing system state

### Comprehensive Security Testing

#### Penetration Testing

- **Advanced Security Testing**: Comprehensive penetration testing to identify vulnerabilities
- **Adversarial Approach**: Tests written from an attacker's perspective to identify exploits
- **Attack Vectors Tested**: Token forgery, double redemption, re-entrancy, mathematical exploits, time manipulation
- **Deactivation Bypass**: Testing to ensure collection deactivation properly restricts operations
- **Cross-Collection Attacks**: Testing against using tokens from one collection in another
- **Re-entrancy Simulation**: Testing against callback exploitation
- **Multi-Collection Exploitation**: Tests for cross-collection confusion attacks

#### Property-Based Testing

- **Systematic Edge Case Discovery**: Property-based tests using randomized inputs
- **Boundary Testing**: Testing with extreme values and edge cases
- **Invariant Verification**: Tests that verify system properties across many inputs
- **Mathematical Correctness**: Testing for mathematical correctness and manipulation resistance
- **Security Properties**: Tests focused on bond redemption security and collection isolation

#### Formal Verification

- **Mathematical Proofs**: Formal verification of financial operation correctness
- **Pre/Post Condition System**: Implementation of preconditions and postconditions for financial calculations
- **Financial Invariants**: Definition of critical system invariants that must be maintained
- **Verification Harness**: Framework for mathematical proof of financial operations

### Security Audit Process

- **Comprehensive Framework**: Structured methodology for security assessment
- **Automated Security Analysis**: Integration of static analysis, testing, and vulnerability scanning
- **Security Audit Script**: Automated security verification with reporting
- **Manual Code Review Process**: Systematic review of security-critical components
- **Bitcoin-specific Security Reviews**: Focus on transaction context verification and block-based vulnerabilities
- **Scheduled Audits**: Regular security review schedule
- **Security Metrics**: Tracking of security issues and fixes over time

### Security Documentation

- **Threat Model**: Documented attack vectors and mitigations
- **Security Architecture**: Security design patterns and controls
- **Risk Assessment**: Identified risks and severity ratings
- **Security Changes Tracking**: Comprehensive documentation of security improvements
- **Function Call Flows**: Sequence diagrams for security-critical operations
- **User Stories with Threat Models**: Description of typical usage scenarios with associated threats

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
5. **Gas Optimization** (for blockchain mode):
   - Minimize storage operations
   - Optimize data structures for gas efficiency
   - Batch operations where possible

## Production Readiness

### Current Status

The codebase is approaching production readiness with some remaining improvements needed:

### Code Cleanup Priorities

The codebase currently has warnings that should be addressed:

- **Unused imports**: Several unused imports across the codebase
- **Unnecessary mutability**: Variables with mut that don't need it
- **Unused variables**: Variables that should be prefixed with underscore or removed
- **Dead code**: Unused fields and methods in mock.rs

### Security Testing Requirements

Current security testing should be expanded with:

#### Fuzzing Tests:
- Implement property-based testing using `proptest` or similar tools
- Focus on extreme values, very long strings, UTF-8 edge cases, and special characters

#### Concurrency Testing:
- Test for race conditions in concurrent operations
- Areas to test: simultaneous bond creation, redemption attempts, state changes

#### Financial Attack Vectors:
- Market manipulation scenarios
- Interest rate manipulation
- Early redemption exploits

### Documentation Requirements

Documentation needs to be enhanced for better understanding and maintainability:

- **Security Guarantees**: Detailed documentation of security guarantees
- **Architecture Documentation**: High-level architecture diagrams and component interactions
- **Error Handling Documentation**: Clear documentation of error handling patterns

### Error Handling Improvements

Error handling needs standardization:

- **Custom Error Types**: Implement custom error types instead of string errors
- **Better Error Messages**: More descriptive errors with actionable information
- **Robust Error Handling**: Proper propagation and recovery mechanisms

### Performance Optimization Requirements

Several performance areas need further optimization:

- **Load Testing**: Test with large numbers of bonds/collections
- **Benchmarking**: Identify and improve performance bottlenecks
- **Batch Optimization**: Optimize batch operations for efficiency

### Input Validation and Authorization

Additional security measures needed:

- **Size Limits**: Implement size limits for string inputs
- **Numerical Constraints**: Verify numerical constraints and boundaries
- **Authorization Mechanisms**: Add stronger transaction authorization
- **Administrative Controls**: Enhance administrative capabilities

## Architectural Considerations

### 1. State Management
- The current HashMap-based approach provides a good balance of simplicity and efficiency
- More sophisticated structures may be needed for large-scale deployments

### 2. Authentication Flow
- The orbital-is-bond model simplifies authentication significantly
- All interfaces must follow this pattern consistently

### 3. Error Recovery
- Current error handling is relatively basic
- Error information should be enhanced to help with debugging and user feedback

### 4. Testing Strategy
- The TestBlockContext approach provides deterministic, time-independent testing
- Current test suite verifies core functionality with high confidence
- Testing guarantees reliable behavior of time-dependent financial operations

## Key Questions for Future Development

1. How should edge cases in bond redemption be handled when the bond has matured but market conditions have changed?

2. Should we support partial redemptions, or are bonds atomic and must be redeemed in full?

3. How can we optimize gas usage for batch operations like redeeming multiple bonds at once?

4. What additional metadata should we store with bonds to improve user experience?

5. How should error handling work in a blockchain context versus standalone context?

## Document Management System

The project implements a structured document management system to maintain consistent and consolidated documentation.

### Cardinal Rule

**"Never create new memory-bank documents, always update existing ones"**

This rule ensures that knowledge is consolidated in standardized locations rather than fragmented across multiple files.

### Document Mapping

Standard files for specific types of content:

| Content Type | Target File |
|-------------|-------------|
| Technical details | `techContext.md` |
| Product information | `productContext.md` |
| Project overview | `projectBrief.md` |
| System design | `systemPatterns.md` |
| Security improvements | `security-enhancements-summary.md` |
| Progress updates | `progress.md` |
| Next actions | `next-steps.md` |

### Documentation Procedure

1. Before creating a new document, check if a related file exists in `memory-bank/`
2. If similar content exists, update or append to the existing file
3. Use consolidated files for general information (like `techContext.md`)
4. Use `progress.md` for ongoing work updates and milestones
5. Group related concepts in existing documents (e.g., security concepts in `security-enhancements-summary.md`)

### Implementation

The document management system is enforced through:

```json
{
  "document_management": {
    "cardinal_rule": "Never create new memory-bank documents, always update existing ones",
    "procedure": [
      "1. Before considering creating any new document, check if a related file already exists in memory-bank/",
      "2. If similar content exists, update or append to the existing file rather than creating a new one",
      "3. Use consolidated files (like techContext.md) for general technical information",
      "4. Use progress.md for ongoing work updates and milestones",
      "5. Group related security concepts in existing security documents"
    ],
    "memory_bank_mapping": {
      "technical_details": "techContext.md",
      "product_information": "productContext.md",
      "project_overview": "projectBrief.md",
      "system_design": "systemPatterns.md",
      "security_improvements": "security-enhancements-summary.md",
      "progress_updates": "progress.md",
      "next_actions": "next-steps.md"
    },
    "check_reminder": "IMPORTANT: Always check document_management.cardinal_rule before creating any new files in memory-bank/"
  }
}
```

### Benefits

- **Knowledge Consolidation**: Prevents information fragmentation
- **Easier Navigation**: Predictable document structure
- **Improved Reference**: Clear mapping between content types and target files
- **Reduced Redundancy**: Minimizes duplicate information
- **Consistent Updates**: Regular updates to standardized files
