# SLOP Project Next Steps

## 1. Subfrost Environment Extension (NEW HIGH PRIORITY)

Based on our thorough investigation of the Subfrost environment, we need to extend its RPC capabilities to enable contract deployment. 

### Current Blocking Issues

The Subfrost environment's RPC server (`subrail`) has a deliberately limited implementation that doesn't support the methods required for contract deployment:

| Required Method | Status | Purpose |
|----------------|--------|---------|
| `alkane_newContract` | Missing | Deploying new contracts |
| `metashrew_height` | Missing | Block synchronization |
| `btc_getblockcount` | Missing | Bitcoin node status |
| `subrail_height` | Available | Getting current blockchain height |

### Step-by-Step Implementation Plan

1. **Fork the Subfrost Repository**
   ```bash
   git clone https://github.com/[organization]/subfrost.git
   cd subfrost
   git checkout -b feature/extend-rpc
   ```

2. **Modify the Subrail Implementation**
   - Edit the `Dockerfile.subrail` to add the missing RPC method implementations
   - Follow the technical specification in `memory-bank/subrail-extension.md`
   - Add global state variables for contract storage
   - Implement helper functions for transaction ID generation

3. **Build and Test the Extended Subrail Server**
   ```bash
   docker build -t subfrost-subrail:extended -f Dockerfile.subrail .
   ```

4. **Update Docker Compose Configuration**
   - Modify `docker-compose.yml` to use the extended subrail image
   - Ensure proper container dependencies and networking

5. **Deploy and Test**
   - Deploy the modified Docker environment
   - Test the newly added RPC methods
   - Verify contract deployment capabilities

6. **Run the SLOP Deployment Script**
   - Use `deploy-slop-subfrost.sh` to deploy SLOP contracts
   - Generate deployment report

### Timeline Estimation
- Forking and modifying: 1 day
- Building and testing: 1 day
- Deployment and verification: 1 day
- Total: ~3 days

## 2. Test Refactoring Implementation Plan

### Priority Order for Test Refactoring

1. **Core Security Tests** (High Priority)
   - curve_security_test.rs - Start with this as it tests the mathematical security of the bond curve
   - security_fixes_test.rs - Critical for ensuring security vulnerabilities are addressed

2. **Functional Tests** (Medium Priority)
   - penetration_tests.rs - Important for security but more complex to update
   - property_tests.rs - Contains complex property-based tests using proptest

3. **Integration Tests** (Lower Priority)
   - provenance_tests.rs - Tests orbital token provenance features

### Step-by-Step Implementation Plan for Test Refactoring

#### Phase 1: Core Security Test Module (curve_security_test.rs)

1. Create a feature branch: `git checkout -b refactor/security-tests`
2. Modify `src/tests/mod.rs` to enable only this test module:
   ```rust
   #[cfg(test)]
   mod mock;
   #[cfg(test)]
   mod curve_security_test;
   // Other tests still disabled
   ```
3. Implement changes documented in test-refactoring-plan.md:
   - Replace static methods with instance methods
   - Replace time calculations with SystemTime
   - Update test assertions to match new API structure

4. Run and debug: `cargo test curve_security`
5. Commit working changes: `git commit -m "refactor: Update curve security tests for new API"`

#### Phase 2: Security Fixes Test Module (security_fixes_test.rs)

1. Re-enable module in `src/tests/mod.rs`
2. Implement changes for OrbitalBondCollection API:
   - Fix `get_bond_by_orbital` references
   - Update Bond status checking
   - Fix mint_bond return type handling

3. Run tests: `cargo test security_fixes`
4. Commit working changes

#### Phase 3: Complex Test Modules (penetration_tests.rs & property_tests.rs)

1. Update penetration_tests.rs:
   - Address collection mutations
   - Fix transaction context interaction
   - Update error handling patterns

2. Update property_tests.rs:
   - Reimagine property tests for the new API
   - Update test generators

3. Run tests: `cargo test -- --include-ignored`
4. Commit working changes

#### Phase 4: Final Test Module (provenance_tests.rs)

1. Update orbital provenance test cases
2. Run all tests: `cargo test`
3. Create PR for test refactoring

## 3. Deployment Pipeline Completion

### Environment Setup

1. Create deployment environment directory:
   ```bash
   mkdir -p deploy/testnet
   cp target/wasm32-unknown-unknown/release/*.wasm.gz deploy/testnet/
   ```

2. Create deployment configuration file:
   ```bash
   touch deploy/testnet/config.json
   ```

3. Populate configuration with initialization parameters for each contract:
   ```json
   {
     "bond_curve": {
       "virtual_input_reserves": 1000000,
       "virtual_output_reserves": 500000,
       "half_life": 3600,
       "level_bips": 5000,
       "term_blocks": 86400
     },
     "orbital_bond_collection": {
       "name": "Orbital Bonds",
       "symbol": "ORB",
       "interest_rate_bps": 500,
       "maturity_blocks": 10000
     },
     "launchpad_factory": {
       "version": "1.0.0",
       "default_maturity_blocks": 10000,
       "default_interest_rate_bps": 500
     }
   }
   ```

### Deployment Script Creation

1. Create deployment script:
   ```bash
   touch deploy/testnet/deploy.sh
   chmod +x deploy/testnet/deploy.sh
   ```

2. Implement deployment script for proper contract sequencing:
   - Deploy bond_curve first
   - Deploy orbital_bond_collection
   - Deploy launchpad_factory with references to the other contracts

3. Add verification steps:
   - Verify contract initialization
   - Test contract interaction

### Testing Deployment Pipeline

1. Create a local mock blockchain environment
2. Test deployment script against local environment
3. Create verification test suite

## 4. Documentation Update

### API Documentation

1. Add JSDoc-style comments to all public methods in:
   - src/contracts/bond_curve.rs
   - src/contracts/orbital_bond_collection.rs
   - src/contracts/launchpad_factory.rs

2. Document parameter types, return values, and error cases

### Usage Examples

1. Create example usage file for each contract:
   ```
   examples/bond_curve_example.rs
   examples/orbital_bond_collection_example.rs
   examples/launchpad_factory_example.rs
   ```

2. Each example should demonstrate:
   - Contract initialization
   - Common operations
   - Error handling

### Integration Guide

1. Create integration documentation for third-party developers:
   ```
   docs/integration-guide.md
   ```

2. Cover topics:
   - Contract deployment sequence
   - Inter-contract communication
   - Required transaction flows
   - Common error scenarios and solutions

## Timeline Estimation

1. **Subfrost Environment Extension**: ~3 days
   - Forking and modifying: 1 day
   - Building and testing: 1 day
   - Deployment and verification: 1 day

2. **Test Refactoring**: ~1-2 weeks
   - Core security tests: 2-3 days
   - Functional tests: 3-4 days
   - Integration tests: 2-3 days

3. **Deployment Pipeline**: ~1 week
   - Environment setup: 1 day
   - Deployment script: 2-3 days
   - Testing pipeline: 2-3 days

4. **Documentation**: ~3-5 days
   - API documentation: 1-2 days
   - Usage examples: 1-2 days
   - Integration guide: 1 day

Total estimated time: ~3-4 weeks
