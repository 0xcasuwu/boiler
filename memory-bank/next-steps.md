# SLOP Next Steps

This document outlines the immediate next steps for the SLOP project based on recent developments and current priorities.

## Immediate Tasks

### 1. Fix API Compatibility Issues

The API signatures in `orbital_bond_collection.rs` have been updated to be more robust, and all test files and dependent code have been updated:

- [x] Update `mint_bond()` calls in test files to include the new `owner_id` parameter 
- [x] Update code that processes the return value from `mint_bond()` to handle the new `(bond_id, alkane_token_id)` tuple
- [x] Update `redeem_bond()` calls to use the new signature with `alkane_token_id` and `redeemer_id`
- [x] Decide whether to use `redeem_bond()` or `redeem_bond_by_orbital()` in various places based on context
- [x] Update `launchpad_factory.rs` to maintain compatibility with the updated API

### 2. Complete Remaining API Documentation

- [x] Enhance documentation for OrbitalBondCollection API with detailed examples
- [ ] Review and enhance documentation for remaining files:
  - [x] `launchpad_factory.rs`
  - [x] `bond_curve.rs`
  - [x] Helper modules in `utils`

### 3. Implement Unified Redemption Security Enhancement

- [x] Replace dual redemption pathways with a single unified method as outlined in `memory-bank/security-unified-redemption.md`
- [x] Enhance security validation to require both alkane and orbital tokens for redemption
- [x] Update all tests to use the new unified redemption method
- [x] Remove the backward compatibility `redeem_bond_by_orbital` method
- [x] Add security tests specifically for redemption validation

### 4. Implement Token Ownership Security Enhancement

- [x] Add secure redemption methods that verify token ownership using transaction context
- [x] Implement `redeem_bond_secure` in OrbitalBondCollection that validates token ownership
- [x] Update LaunchpadFactory with corresponding secure redemption methods
- [x] Add comprehensive tests for token ownership verification in `token_ownership_security_test.rs`
- [x] Document the security enhancement in `memory-bank/token-ownership-security.md`
- [x] Mark legacy methods as deprecated with clear security warnings

### 5. Integrate Standalone Fuzzing Tests

- [ ] Update the standalone fuzzing test runner to use the new API signatures
- [ ] Consider integrating the standalone fuzzing approach into the main test suite
- [ ] Ensure the fuzzing tests cover the new API parameters and return types

### 6. Enhanced Collection Management

- [ ] Develop richer administration tools for bond collections
- [ ] Add ability to update collection parameters
- [ ] Implement collection statistics and reporting

### 7. Batch Operations Support

- [ ] Design and implement a formal API for batch redemptions
- [ ] Add support for batch minting operations 
- [ ] Optimize batch operations for efficiency

### 8. Error Handling Improvements

- [ ] Add more descriptive error messages throughout the codebase
- [ ] Implement better error recovery mechanisms
- [ ] Create documentation for error handling best practices

## Long-term Enhancements

### Advanced Features

- [ ] Implement advanced interest models beyond simple interest
- [ ] Add support for partial redemptions
- [ ] Design and implement a formal bond cancellation API
- [ ] Add extended metadata support for bonds and collections

### Security & Performance

- [ ] Conduct comprehensive security review and auditing
- [ ] Implement formal verification of critical components
- [ ] Optimize gas usage for blockchain operations
- [ ] Enhance protection against common attack vectors
