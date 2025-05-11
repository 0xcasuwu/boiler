# Implementation Summary for Yield Vault

## Architecture Overview

The Yield-Vault WASM contract implements the ERC-4626 tokenized vault standard for Bitcoin. It provides functionality for:

1. Depositing assets and minting shares
2. Withdrawing assets by burning shares
3. Redeeming shares for assets
4. Account and asset management
5. Computing yields over time

## Core Dependencies

The contract relies on three key external dependencies:

1. **alkanes-runtime** - Provides the runtime environment for WASM contracts on Bitcoin
2. **alkanes-support** - Provides support utilities and common functions
3. **metashrew-support** - Provides storage utilities and index pointer functionality

All three dependencies are pulled directly from their GitHub repositories:
```toml
alkanes-runtime = { git = "https://github.com/kungfuflex/alkanes-rs" }
alkanes-support = { git = "https://github.com/kungfuflex/alkanes-rs" }
metashrew-support = { git = "https://github.com/sandshrewmetaprotocols/metashrew" }
```

## Key Components

### Storage

- Uses `StoragePointer::from_keyword()` for all persistent storage as specified in requirements
- Follows the established naming convention for storage keys (e.g., `/name`, `/symbol`, etc.)
- Provides helper functions to access storage pointers for various data types

### Security

- Implements `observe_initialization()` in Initialize operation to prevent multiple initializations
- Uses transaction hash tracking to prevent replay attacks
- Implements authorization checks to verify transaction permissions

### Asset Management

- Follows ERC-4626 interface for yield-bearing vault functionality
- Updates yield before any state-changing operation
- Implements deposit, mint, withdraw, and redeem operations
- Handles asset/share conversions with proper error checking

### Utils

- Provides utilities for converting between assets and shares
- Implements ceil division for precise financial calculations
- Defines constants for yield calculations

## Opcode Implementation

The contract implements the required opcodes as specified in the requirements:
- 0: Initialize - Sets up the vault with initial parameters
- 10-19: Asset Management Operations (deposit, mint, withdraw, redeem)
- 100-199: Metadata View Functions (name, symbol, decimals, etc.)
- 200-299: Accounting View Functions (total assets, conversion methods)
- 300-399: Limit View Functions (max deposit, max mint, etc.)
- 400-499: Preview View Functions (preview deposit, preview mint, etc.)
- 500-599: Custom Data Operations (set/get arbitrary data)
- 600-699: Balance Management (balance, total supply)
- 900-999: Administrative Operations (update yield rate)

## Initialization Process

1. Checks if already initialized to prevent multiple initializations
2. Sets token metadata (name, symbol, asset name, asset symbol, decimals)
3. Initializes accounting state (total supply, total assets)
4. Initializes yield tracking (yield rate, last update timestamp)

## Yield Calculation

Yield is calculated based on:
- The time (blocks) elapsed since the last yield update
- The current yield rate (in basis points)
- The total assets under management

The formula used is: `yield = total_assets * yield_rate * blocks_elapsed / YIELD_CALCULATION_DENOMINATOR`

## Asset/Share Conversion

The contract implements the required conversion functions:
- `convert_assets_to_shares`: Calculates shares for a given asset amount
- `convert_shares_to_assets`: Calculates assets for a given share amount
- With special handling for empty vaults (1:1 ratio)
