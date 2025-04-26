# SLOP: Smart Contract Launchpad for Orbital Payments

SLOP is a robust bond management system built on the innovative concept that **orbital tokens themselves ARE bonds**. This approach ties bond ownership directly to token possession, creating a secure, verifiable path for bond issuance and redemption.

## Core Application Flow

SLOP follows a hierarchical structure with a factory pattern for efficient management. Below is the complete function call flow during typical usage scenarios.

### 1. Factory Initialization

```rust
// Create a block context for time/maturity tracking
let block_context = StandaloneBlockContext::new();

// Create the LaunchpadFactory - the central coordinator
let mut factory = LaunchpadFactory::new(
    "1.0.0".to_string(),    // Factory version
    50,                     // Default maturity (blocks)
    500,                    // Default interest rate (5% = 500 basis points)
    &block_context,         // Block context for time tracking
);
```

**Test Evidence**: In `test_factory_creation()` from `bond_collection_test.rs`, we verify the factory is properly initialized with default parameters.

### 2. Collection Creation

```rust
// Create a bond collection through the factory
let collection_id = factory.create_collection(
    "Premium Bonds".to_string(),           // Collection name
    "PBND".to_string(),                    // Symbol/ticker
    Some("High-yield bonds".to_string()),  // Optional description
    Some(800),                             // Custom interest rate (8%)
    Some(43200),                           // Custom maturity period
    &block_context,
);
```

**Test Evidence**: In `test_collection_creation()`, we verify collections are created with correct parameters and can include optional descriptions.

### 3. Bond Minting

```rust
// Create a new bond tied to an orbital token
let orbital_id = "orbital-123".to_string();
let (bond_id, alkane_token_id, alkane_transfer) = factory.mint_bond(
    &collection_id,         // Collection to mint in
    orbital_id.clone(),     // Orbital token ID (THE BOND)
    1000,                   // Principal amount
    "owner-1".to_string(),  // Owner ID
    &block_context,
);
```

**Test Evidence**: In `test_bond_minting()`, we verify bonds are correctly created with expected parameters and that duplicates are prevented.

### 4. Bond Status Verification

```rust
// Check which bonds are mature
let collection = factory.get_collection(&collection_id).unwrap();

// Get all mature bonds
let mature_bonds = collection.get_mature_bonds(&block_context);

// Check if a specific bond is mature
let bond = collection.get_bond_by_orbital(&orbital_id).unwrap();
let is_mature = bond.is_mature(&block_context);
```

**Test Evidence**: In `test_mature_bonds_query()`, we verify the system correctly tracks maturity status as blocks advance.

### 5. Bond Redemption

```rust
// After maturity is reached, redeem the bond
let redemption_amount = factory.redeem_bond(
    &collection_id,       // Collection containing the bond
    &orbital_id,          // Orbital token ID
    "redeemer-id",        // ID of entity redeeming
    &mature_context,      // Block context showing maturity
);

// Redemption amount includes principal + interest
assert_eq!(redemption_amount, 1050); // 1000 principal + 5% interest
```

**Test Evidence**: In `test_bond_redemption()`, we verify bonds can only be redeemed once and only after reaching maturity, with correct interest calculation.

### 6. Collection Management

```rust
// Deactivate a collection (prevent new bonds)
factory.deactivate_collection(&collection_id);

// Reactivate a collection
factory.reactivate_collection(&collection_id);

// Calculate total value across all collections
let total_value = factory.total_value(&block_context);
```

**Test Evidence**: In `test_collection_deactivation()`, we verify collections can be deactivated to prevent new minting while still allowing existing bonds to mature.

## User Stories

### 1. Collection Creator

**Story**: As a bond issuer, I want to create collections with different parameters to offer various bond products.

**Implementation**:
```rust
// Create different collections with varying parameters
let high_yield = factory.create_collection(
    "Premium Bonds".to_string(), "PREM".to_string(),
    None, Some(1000), None, &context // 10% interest
);

let secure = factory.create_collection(
    "Stable Bonds".to_string(), "STBL".to_string(),
    None, Some(300), Some(200), &context // 3% interest, longer maturity
);
```

**Test Evidence**: `test_collection_search()` validates that collections can be created with different parameters and later filtered or searched based on their properties.

### 2. Bond Issuer

**Story**: As a bond administrator, I want to issue bonds tied to orbital tokens to guarantee ownership authenticity.

**Implementation**:
```rust
// Mint bonds with different parameters
factory.mint_bond(&collection_id, "orbital-1", 1000, "owner-1", &context);
factory.mint_bond(&collection_id, "orbital-2", 2000, "owner-2", &context);
```

**Test Evidence**: `test_total_value()` shows multiple bonds can be minted with different values, and their total can be accurately calculated.

### 3. Bond Holder

**Story**: As a bond holder, I want to redeem my mature bonds for principal plus interest.

**Implementation**:
```rust
// After maturity
let redemption_result = factory.redeem_bond(
    &collection_id, &orbital_id, "holder-id", &mature_context
);
```

**Test Evidence**: `test_redemption_path_consistency()` confirms that redemption works correctly only for the legitimate token holder, and prevents double redemptions.

### 4. Security Administrator

**Story**: As a security admin, I want to ensure bonds can't be forged or redeemed fraudulently.

**Implementation**:
```rust
// System automatically verifies:
// 1. The orbital token matches a valid bond
// 2. The bond belongs to the specified collection
// 3. The bond hasn't been redeemed before
// 4. The bond has reached maturity
```

**Test Evidence**: `test_token_based_redemption_security()` and `test_forgery_resilience()` verify that manipulated tokens or IDs are rejected, and `test_state_manipulation_attack()` confirms that even with direct access to collection state, security constraints remain enforced.

## Security Features

The "orbital token IS the bond" architecture provides several security benefits:

1. **Token-Based Authentication**: The orbital token's presence is required for redemption, providing a natural proof-of-ownership mechanism.

2. **Unified Redemption Path**: A single consistent redemption method prevents exploitation of differences between pathways.

3. **Maturity Verification**: Bonds can only be redeemed after their maturity period has elapsed.

4. **Anti-Forgery Measures**: Token IDs have validation ensuring they belong to the correct collection and represent valid bonds.

5. **State Protection**: Even with direct access to the bond data, the system enforces consistent validation rules.

**Test Evidence**: `src/tests/alkane_security_test.rs` contains comprehensive security tests, including:
- Double redemption prevention (`test_token_based_redemption_security`)
- Cross-collection redemption prevention
- Token structure attack resilience (`test_token_structure_attack`)
- State manipulation attack resilience (`test_state_manipulation_attack`)

## Technical Architecture

```
┌──────────────────────────┐
│ LaunchpadFactory         │
│                          │
│ - Creates collections    │
│ - Manages overall state  │
│ - Coordinates redemption │
└───────────┬──────────────┘
            │
            │ creates
            ▼
┌──────────────────────────┐
│ OrbitalBondCollection    │
│                          │
│ - Manages specific bonds │
│ - Handles minting        │
│ - Verifies redemption    │
└───────────┬──────────────┘
            │
            │ creates
            ▼
┌──────────────────────────┐
│ Bond                     │
│                          │
│ - Stores bond parameters │
│ - Tracks status/maturity │
│ - Links to orbital token │
└──────────────────────────┘
```

## Conclusion

SLOP's innovative approach of using orbital tokens directly as bonds creates a secure, efficient bond management system. The factory pattern facilitates easy management of multiple bond collections, while the block-based time tracking ensures deterministic maturity calculations even in testing environments.

The extensive test coverage, including security tests, demonstrates the system's resilience against common attack vectors while maintaining a clean, intuitive API.
