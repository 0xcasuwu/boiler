# Bond Curve and Contract Test Refactoring Plan

## Overview

The test suite needs to be refactored to match the updated API structure. This document outlines the key changes needed in each test module to make them compatible with the current implementation.

## Key API Changes

1. **BondCurve API Changes**
   - `exp_to_level` static method was removed
   - `get_current_block` static method was removed
   - `purchase_bond` method was removed (use `get_amount_out` instead)
   - `redeem_bond` method was removed (use `get_redeem_amount` instead)
   - `update_pricing` method was replaced by `update_parameters`

2. **OrbitalBondCollection API Changes**
   - `get_bond_by_orbital` method was removed (use `get_bond` instead)
   - `calculate_interest` method was removed (interest calculation now happens in `Bond::new`)
   - Bond fields have changed:
     - `is_redeemed` => `status` enum
     - `created_block` => `creation_block`
     - `owner_id` is no longer a field

3. **LaunchpadFactory API Changes**
   - `get_collection_mut` method was removed (use `get_collection` instead)
   - `redeem_bond_by_alkane_secure` method was removed
   - `deactivate_collection` method was removed
   - `reactivate_collection` method was removed
   - `verify_orbital_provenance` method was removed
   - `total_value` method was removed
   - `mint_bond` return type changed from tuple to string

## Test Module Refactoring Guidelines

### 1. curve_security_test.rs

Update tests to work with the current BondCurve API:

```rust
// Replace static methods with instance methods
// Before:
let result = BondCurve::exp_to_level(max_value, 3600, 3600, 5000);

// After:
let curve = BondCurve::new(/* params */);
let decayed_reserves = curve.calculate_decayed_reserves(3600);
// Test decayed_reserves instead

// Replace get_current_block
// Before:
let current_block = BondCurve::get_current_block();

// After:
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs() as u64 / 10; // Simple approximation: 1 block = 10 seconds

// Replace purchase_bond
// Before:
let purchase_result = curve.purchase_bond(100_000, 0);

// After:
let output_amount = curve.get_amount_out(100_000, 0);
// Test output_amount directly

// Replace update_pricing
// Before:
curve.update_pricing(/* params */, true);

// After:
curve.update_parameters(/* params */);
```

### 2. security_fixes_test.rs and penetration_tests.rs

Update tests for the new Bond structure and OrbitalBondCollection API:

```rust
// Replace get_bond_by_orbital
// Before:
let bond = collection.get_bond_by_orbital(&orbital_id).unwrap();

// After:
let bond_id = format!("bond-{}-{}", orbital_id, start_block);
let bond = collection.get_bond(&bond_id).unwrap();

// Check Bond status instead of is_redeemed
// Before:
assert!(!bond.is_redeemed);

// After:
assert_eq!(bond.status, crate::models::bond::BondStatus::Active);

// Fix mint_bond return value
// Before:
let (bond_id, alkane_id, _) = factory.mint_bond(/* params */);

// After:
let bond_id = factory.mint_bond(/* params */);
// No more tuple unpacking
```

### 3. property_tests.rs

This module requires more extensive changes:

```rust
// Replace update_bonds_for_test with a new approach
// Before:
collection.update_bonds_for_test(|bond| { /* ... */ });

// After:
// This method is no longer available - use a different approach
// One option is to create a new collection for each test case

// Replace redeem_bond_for_test
// Before:
let redemption_result = collection.redeem_bond_for_test(/* params */);

// After:
let context = MockBlockContext::new();
let tx_context = MockTransactionContext::new_with_caller("test-caller");
let redemption_result = collection.redeem_bond_secure(&tx_context, redeemer_id, &context);
```

## Implementation Strategy

1. Update tests one at a time, starting with the most critical ones
2. Use the `#[ignore]` attribute on tests that are still being refactored
3. Focus on fixing API changes first, then address logical test changes
4. Update mock implementations as needed to support the new tests

## Estimated Effort

- curve_security_test.rs: 2-3 hours
- security_fixes_test.rs: 2-3 hours
- penetration_tests.rs: 3-4 hours
- property_tests.rs: 4-5 hours
- provenance_tests.rs: 1-2 hours

Total estimated effort: 12-17 hours of development time
