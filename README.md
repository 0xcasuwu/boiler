# Orbital Bond Launchpad

A Rust implementation of a launchpad that accepts diesel tokens and mints orbitals which function as both bonds and authentication tokens.

## Overview

The Orbital Bond Launchpad is a factory-based system that:

1. Accepts diesel tokens from users
2. Creates orbital tokens that represent bonds with specific terms
3. Uses the orbital tokens themselves for authentication (token-based authorization)
4. Manages bond redemption based on maturity

Every new bond issuance creates a completely new collection from the factory, eliminating the need to track user addresses since orbital ownership makes that irrelevant.

## Key Features

- **Token-based Authentication**: Ownership and authorization are determined solely by the presence of orbital tokens in the transaction context
- **Factory/Child Pattern**: LaunchpadFactory creates OrbitalBondCollection instances for each bond issuance
- **Bonding Curve Pricing**: Dynamic pricing based on supply, demand, and time decay
- **Block-based Maturity**: Bonds mature based on block numbers, not timestamps
- **No User Addresses**: The architecture completely eliminates user addresses from the security model

## Architecture

```
┌─────────────────────┐
│                     │
│  LaunchpadFactory   │◄────┐
│                     │     │
│                     │     │
└─────────┬───────────┘     │
          │                 │
          │ creates         │ manages
          │                 │
          ▼                 │
┌─────────────────────┐     │
│                     │     │
│  OrbitalBond        ├─────┘
│  Collection         │
│                     │
└─────────┬───────────┘
          │
          │ issues
          │
          ▼
┌─────────────────────┐
│                     │
│  OrbitalToken       │
│  (Bond + Auth)      │
│                     │
└─────────────────────┘
```

## Project Structure

- `/src/models/bond.rs`: Core bond model with maturity and redemption logic
- `/src/contracts/launchpad_factory.rs`: Factory for creating and managing bond collections
- `/src/contracts/orbital_bond_collection.rs`: Collection representing a bond issuance
- `/src/contracts/bond_curve.rs`: Pricing mechanism with time decay
- `/src/utils/mod.rs`: Utility functions for formatting and calculations
- `/examples/launchpad_example.rs`: Example demonstrating the complete flow

## Usage

### Running the Example

```bash
cargo run --example launchpad_example
```

This example demonstrates:
1. Creating a launchpad factory
2. Processing diesel deposits
3. Creating bond collections
4. Minting orbital tokens as bonds
5. Verifying ownership through transaction context
6. Redeeming bonds when mature

### Key Code Example

```rust
// Create a factory
let mut factory = LaunchpadFactory::new(
    factory_id,
    "Diesel Bond Launchpad",
    1_000_000,  // virtual_input_reserves
    500_000,    // virtual_output_reserves
    3600,       // half_life
    5000,       // level_bips
    100         // default term
);

// Create a bond collection
let collection_id = factory.create_orbital_collection("Example Bonds", None)?;

// Mint an orbital bond for a user depositing diesel
let orbital_id = factory.mint_orbital_bond(&collection_id, diesel_amount, None)?;

// Simulate a transaction context with the token for authentication
collection.set_mock_context(vec![(orbital_id, 1)]);

// This bond can now be redeemed if mature, without needing a user address
if collection.verify_token_ownership(&orbital_id) && bond.is_mature() {
    collection.redeem_bond(&orbital_id)?;
}
```

## Blockchain Integration

The project is designed to work with or without an actual blockchain:

- In development/testing, it uses mock contexts for authentication
- In production, it can be configured to use actual blockchain transaction contexts

This is controlled via the `blockchain` feature flag.

## Credits

This project is based on patterns from existing contracts:
- Orbital Collection tokens
- Bonding Contract implementations
- Factory/Child patterns for token issuance

## License

MIT
