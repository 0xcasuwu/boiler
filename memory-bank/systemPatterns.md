# SLOP System Patterns

## Architecture Overview

SLOP follows a factory pattern architecture where the main LaunchpadFactory creates and manages multiple independent OrbitalBondCollection instances. Each collection manages its own set of bonds, with the bonds themselves being represented by orbital tokens.

```
                  ┌─────────────────┐
                  │LaunchpadFactory │
                  └────────┬────────┘
                           │
                           │ creates
                           ▼
     ┌─────────────┬───────────────┬─────────────┐
     │             │               │             │
┌────▼─────┐ ┌─────▼─────┐   ┌─────▼─────┐ ┌─────▼─────┐
│Collection1│ │Collection2│   │Collection3│ │Collection4│
└────┬─────┘ └─────┬─────┘   └─────┬─────┘ └─────┬─────┘
     │             │               │             │
     │ mints       │ mints         │ mints       │ mints
     ▼             ▼               ▼             ▼
┌───────────┐ ┌───────────┐   ┌───────────┐ ┌───────────┐
│Orbital/   │ │Orbital/   │   │Orbital/   │ │Orbital/   │
│Bond       │ │Bond       │   │Bond       │ │Bond       │
└───────────┘ └───────────┘   └───────────┘ └───────────┘
```

## Core Design Patterns

### 1. Factory Pattern

The LaunchpadFactory acts as a factory that creates and manages OrbitalBondCollection instances. This pattern provides:

- **Centralized Creation Logic**: All collections are created through a single factory
- **Standardization**: Collections follow consistent patterns and interfaces
- **Management Capabilities**: The factory maintains references to all created collections

Implementation:
```rust
pub struct LaunchpadFactory {
    collections: HashMap<String, OrbitalBondCollection>,
    next_collection_id: u64,
    // ...
}

impl LaunchpadFactory {
    pub fn create_collection(...) -> String {
        // Generate a unique collection ID
        let collection_id = format!("collection-{}", self.next_collection_id);
        self.next_collection_id += 1;
        
        // Create the collection
        let collection = OrbitalBondCollection::new(...);
        
        // Store and return the reference
        self.collections.insert(collection_id.clone(), collection);
        collection_id
    }
}
```

### 2. Entity-Component Pattern (Modified)

The bonds function as entities, with their properties (amount, maturity, etc.) as components. The key modification is that the orbital token itself IS the entity, not just a reference to it:

- **Entity**: The orbital token
- **Components**: Bond attributes (amount, maturity, interest rate)
- **Behavior**: Redemption logic, maturity checking, value calculation

```rust
pub struct Bond {
    pub id: String,
    pub orbital_token_id: String,  // The orbital IS the bond
    pub amount: u64,
    pub creation_block: u64,
    pub maturity_block: u64,
    pub status: BondStatus,
    pub interest_rate_bps: u16,
    pub metadata: Option<serde_json::Value>,
}
```

### 3. Block-Based Time

The system uses block numbers instead of timestamps for measuring time, which provides:

- **Consistency**: Block-based time is more consistent across validators
- **Determinism**: Everyone agrees on the exact block number
- **Simplicity**: No need for complex time calculations

```rust
pub trait BlockContext {
    fn get_current_block_height(&self) -> u64;
    
    fn is_block_height_reached(&self, target_height: u64) -> bool {
        self.get_current_block_height() >= target_height
    }
}
```

### 4. Context-Based Ownership

Rather than maintaining explicit maps of token owners, the system uses the transaction context to determine ownership. This means:

- **No Address Tracking**: The system never tracks user addresses
- **Possession = Ownership**: Whoever presents the orbital token owns it
- **Simplified Authentication**: No complex auth flows needed

Implementation pattern:
```rust
// Redeem using only the orbital token ID from the context
pub fn redeem_bond<T: BlockContext>(
    &mut self,
    orbital_token_id: &str,
    block_context: &T,
) -> Result<u64, &'static str> {
    // Find the bond by orbital ID
    let bond_id = self.orbital_to_bond.get(orbital_token_id)
        .ok_or("Orbital token has no associated bond")?;
    
    // Get bond and attempt redemption
    let bond = self.bonds.get_mut(bond_id)
        .ok_or("Bond not found")?;
        
    bond.redeem(block_context)
}
```

### 5. Feature Flagging

The system supports multiple execution environments through feature flags:

- **`blockchain` feature**: Enables blockchain-specific functionality
- **Standalone mode**: Default when blockchain feature is disabled

```rust
#[cfg(feature = "blockchain")]
pub fn get_default_context() -> impl BlockContext {
    blockchain::BlockchainContext::new()
}

#[cfg(not(feature = "blockchain"))]
pub fn get_default_context() -> impl BlockContext {
    StandaloneBlockContext::new()
}
```

## Data Flow

The primary data flows in the system are:

### 1. Collection Creation Flow

```
User Request → LaunchpadFactory → Create OrbitalBondCollection → Return Collection ID
```

### 2. Bond Minting Flow

```
User Request + Diesel → Collection → Mint Orbital/Bond → Transfer to User
```

### 3. Bond Redemption Flow

```
User + Orbital → Collection → Verify Maturity → Mark Redeemed → Return Principal + Interest
```

### 4. Collection Query Flow

```
User Query → LaunchpadFactory → Find Collection → Retrieve Collection Data
```

### 5. Bond Value Calculation Flow

```
Bond + Block Context → Calculate Time Elapsed → Compute Current Value
```

## State Management

State is managed at two primary levels:

### 1. Factory Level State

- **Collections Map**: Tracks all created collections by ID
- **Next Collection ID**: Counter for generating unique collection IDs

### 2. Collection Level State

- **Bonds Map**: Tracks bonds by ID
- **Orbital-to-Bond Map**: Maps orbital token IDs to bond IDs
- **Collection Parameters**: Interest rate, maturity period, etc.
- **Active Status**: Whether the collection can mint new bonds

### 3. Bond Level State

- **Financial Details**: Amount, interest rate, maturity
- **Status**: Active, Mature, Redeemed, or Canceled
- **Creation Information**: When the bond was created
- **Metadata**: Optional additional bond information

## Error Handling Strategy

The system uses Rust's Result type for error handling, with a focus on:

1. **Early Returns**: Functions exit early when errors are detected
2. **Clear Error Messages**: String-based error messages for debugging
3. **Propagation**: Errors bubble up through the call stack with the `?` operator
4. **Validation**: Input validation before state changes

```rust
pub fn mint_bond<T: BlockContext>(
    &mut self,
    orbital_token_id: String,
    amount: u64,
    block_context: &T,
) -> Result<String, &'static str> {
    // Validate collection state
    if !self.active {
        return Err("Collection is inactive");
    }
    
    // Validate inputs
    if self.orbital_to_bond.contains_key(&orbital_token_id) {
        return Err("Orbital token already has a bond");
    }
    
    // Proceed with operation...
    Ok(bond_id)
}
```

## Testing Patterns

The project employs several testing patterns:

1. **Unit Testing**: Testing individual components in isolation
2. **Block Context Mocking**: Simulating different block heights for testing time-dependent behavior
3. **Scenario Testing**: Testing complete workflows from creation to redemption
4. **Property-Based Testing**: Verifying that properties hold across different inputs
5. **Mock-Free Testing**: Avoiding global mocks in favor of dependency injection

## Interface Patterns

The system exports several key traits and interfaces:

1. **BlockContext**: Abstraction for accessing block height information
2. **LaunchpadFactory**: Interface for creating and managing collections
3. **OrbitalBondCollection**: Interface for managing bonds within a collection

These interfaces are designed to be:

- **Minimal**: Exposing only necessary functionality
- **Consistent**: Following similar patterns across the system
- **Self-Contained**: Not requiring external dependencies
- **Ergonomic**: Providing intuitive, easy-to-use methods

## Security Patterns

1. **Possession-Based Authentication**: The orbital token itself proves ownership
2. **No Direct State Manipulation**: All state changes go through verified interfaces
3. **Active Collection Guard**: Only active collections can mint new bonds
4. **Maturity Verification**: Bonds can only be redeemed after reaching maturity
5. **Single Redemption**: Bonds cannot be redeemed multiple times
6. **Status Tracking**: Bond status transitions are strictly controlled
