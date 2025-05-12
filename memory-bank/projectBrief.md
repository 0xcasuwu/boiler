# Yield Vault: Bitcoin Smart Contract

## Project Overview

The Yield Vault project implements a tokenized vault standard (ERC-4626 equivalent) on Bitcoin, providing a secure platform for yield-bearing assets. This contract leverages the Bitcoin blockchain's security and the Alkane smart contract framework to create a capital-efficient yield-generating vault system.

## Core Features

### 1. Tokenized Vault Standard (ERC-4626)
- Full ERC-4626 compatibility
- Asset deposit and withdrawal
- Share minting and redemption
- Exchange rate between assets and shares
- Total assets and total supply tracking

### 2. Yield Generation
- Configurable yield rate (in basis points)
- Automatic yield accrual based on block height
- Yield applied to total assets without diluting shares
- Admin functions for yield rate management

### 3. Security Features
- Initialization guard pattern
- Transaction replay protection
- Overflow protection for numeric operations
- Permission validation via AlkaneId
- Comprehensive error handling

### 4. API Functions

#### Asset Management
- `deposit`: Convert assets to shares
- `mint`: Issue exact share amount
- `withdraw`: Remove assets from vault
- `redeem`: Burn shares to receive assets

#### View Functions
- Metadata: name, symbol, decimals, asset
- Accounting: totalAssets, convertToShares, convertToAssets
- Limits: maxDeposit, maxMint, maxWithdraw, maxRedeem
- Previews: previewDeposit, previewMint, previewWithdraw, previewRedeem

#### Balance Management
- `balanceOf`: Get account share balance
- `totalSupply`: Get total shares outstanding

#### Administrative Operations
- `updateYieldRate`: Set new yield rate
- `getYieldRate`: View current yield rate

### 5. Cross-Platform Compatibility
- WebAssembly compilation
- Apple Silicon (M1/M2/M3) support
- Custom dependency handling for platform compatibility
- Standardized build process

## Technical Architecture

### Modular Design
- **Asset Management**: Handles deposit, mint, withdraw, and redeem operations
- **Security**: Implements authorization and validation checks
- **Storage**: Manages persistent contract state
- **Utils**: Provides helper functions for numeric operations and error handling

### Storage System
- Standardized storage paths
- Prefix-based namespace organization
- Type-safe access to stored values
- Balance tracking with account-based storage

### Security Model
- Dual-mode authentication system
  - Test mode: Validates numeric block=1, tx=1 parameters
  - Production mode: Validates actual AlkaneId string representations
- Transaction hash validation to prevent replay attacks
- Initialization guard to prevent multiple contract initializations
- Checked arithmetic operations to prevent overflows

### Development Environment
- Comprehensive WebAssembly build support
- Platform-specific optimizations for Apple Silicon
- Local dependency forks to ensure cross-platform compatibility
- OylNet testnet deployment and testing

## Build and Deployment

### Build System
- Custom build scripts for different environments
- Local dependency forks to resolve platform-specific issues
- WebAssembly optimization
- LLVM integration for Apple Silicon support

### Deployment Pipeline
- Automated deployment script for OylNet testnet
- Contract initialization with configurable parameters
- Transaction confirmation with block generation
- Contract interaction framework for validation

### Testing Framework
- Unit tests for individual components
- Basic tests for core functionality
- End-to-end tests for complete workflows
- Adversarial tests for security validation
- Test isolation for concurrent test execution
- Mock context for blockchain simulation

## Current Status

The Yield Vault contract is fully functional with a stable build system that works across all platforms including Apple Silicon. The contract has been successfully deployed to OylNet testnet and all core operations have been verified:

- ✅ Metadata view functions
- ✅ Accounting state tracking
- ✅ Administrative operations
- ✅ Deposit operations (with numeric authentication)
- ✅ Balance queries (with numeric authentication)

The authentication model has been refined to properly validate operations using either test mode numeric parameters (block=1, tx=1) or production-mode AlkaneId string comparisons.

## Next Steps

### 1. Production Readiness
- Further WebAssembly size optimization
- Comprehensive security audit
- Gas optimization for all operations
- Complete withdrawal operation testing

### 2. Enhanced Features
- Multi-asset vault extension
- Strategy-based yield generation
- Governance integration
- Fee mechanism implementation

### 3. Testing Improvements
- Memory safety enhancements in e2e tests
- Test coverage expansion
- Performance benchmarking
- Stress testing under high load conditions

## Implementation Decisions

### Storage Structure
The contract uses a standardized storage structure with clear naming conventions:
- `/name` - Vault token name
- `/symbol` - Vault token symbol
- `/asset-name` - Underlying asset name
- `/asset-symbol` - Underlying asset symbol
- `/decimals` - Decimal precision
- `/total-supply` - Total share supply tracking
- `/total-assets` - Total assets under management
- `/yield-rate` - Configured yield rate (basis points)
- `/last-yield-update` - Timestamp of last yield update
- `/balances/{account}` - Account share balances
- `/data/{key}` - Custom stored data
- `/initialized` - Initialization guard

### Yield Calculation
Yield is calculated based on the elapsed time since the last update, the configured yield rate, and the total assets under management:

```
yield_amount = total_assets * yield_rate * time_elapsed / (10000 * BLOCKS_PER_YEAR)
```

This provides a smooth yield accrual that adjusts based on actual time elapsed rather than fixed intervals.

### Authentication Model
The contract implements a dual-mode authentication system:

1. **Test Mode**: Validates operations from AlkaneId where block=1 and tx=1
2. **Production Mode**: Validates operations by comparing AlkaneId string representations

This approach allows for easy testing while maintaining security in production.

## Development Guidelines

1. **Code Style**
   - Follow standard Rust naming conventions
   - Use descriptive function and variable names
   - Document all public functions
   - Use anyhow for error handling

2. **Security Practices**
   - Always use checked arithmetic operations
   - Validate all inputs
   - Verify permissions before state changes
   - Update yield before any state changes

3. **Testing Approach**
   - Write tests for all new functions
   - Ensure test isolation with unique namespaces
   - Include both success and failure cases
   - Test edge conditions (zero values, maximum values)
   - Verify contract state before and after operations

4. **Build and Deployment**
   - Use provided build scripts for WebAssembly generation
   - Verify WebAssembly output before deployment
   - Follow standardized deployment process

5. Initializing Environment
Set .env to '
PROVIDER=oylnet
NETWORK=regtest
API_KEY=lasereyes
'
- cd into oyl-sdk and npm
