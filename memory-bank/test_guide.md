# YieldVault Testing Guide

This guide provides comprehensive information about the testing infrastructure for the YieldVault project, including how to run tests, what tests are available, and how to interpret the results.

## Testing Infrastructure

The YieldVault project uses a multi-layered testing approach to ensure the contract functions correctly:

1. **Unit Tests**: Test individual functions and components in isolation
2. **Integration Tests**: Test interactions between components
3. **Mock Implementation Tests**: Test the contract logic using a mock implementation
4. **WebAssembly Tests**: Test the actual WebAssembly contract
5. **OylNet Deployment Tests**: Test the contract on the OylNet network

## Running Tests

### Quick Start

The easiest way to run the active tests is using the provided script:

```bash
./bin/test/run_working_tests.sh
```

This script will run all the active tests in the `tests/` directory.

### Running Specific Tests

To run a specific test file:

```bash
cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu
```

To run a specific test:

```bash
cargo test --test mock_vault_tests --target x86_64-unknown-linux-gnu -- test_deposit_withdraw
```

> **Important**: Always specify the `--target x86_64-unknown-linux-gnu` flag when running tests. Without this flag, the tests will be compiled for WebAssembly (wasm32-unknown-unknown) and will fail with an "Exec format error".

### Test Categories

#### Active Tests

These are the currently maintained and passing tests:

- `tests/mock_vault_tests.rs`: Core functionality tests for the MockYieldVault implementation
- `tests/simple_utils_test.rs`: Tests for utility functions
- `tests/invariant_tests.rs`: Tests for system invariants
- `tests/alkane_id_verification_tests.rs`: Tests for alkane ID verification in transactions

#### Archived Tests

These tests have been moved to the `archive/deprecated_tests/` directory:

- `mock_vault_penetration_tests.rs`: Basic security tests
- `mock_vault_advanced_penetration_tests.rs`: Advanced security tests
- `mock_vault_erc4626_specific_tests.rs`: Tests specific to the ERC-4626 standard
- `mock_vault_token_tests.rs`: Tests for the token model
- `mock_vault_invariants.rs`: Tests for system invariants
- `mock_vault_combined_tests.rs`: Combined tests for multiple aspects

## Test Structure

### MockVault Tests

The `mock_vault_tests.rs` file contains tests for the core functionality of the YieldVault contract using the MockYieldVault implementation. This implementation uses in-memory storage with HashMap and bincode serialization, allowing for testing without the full WebAssembly environment.

Key test cases include:

1. **test_initialization**: Tests the initialization process and verifies that the contract can only be initialized once
2. **test_deposit_withdraw**: Tests the deposit and withdraw functions
3. **test_mint_redeem**: Tests the mint and redeem functions
4. **test_yield_accrual**: Tests the yield accrual mechanism
5. **test_preview_functions**: Tests the preview functions that simulate operations

### Invariant Tests

The `invariant_tests.rs` file contains tests that verify the invariants of the YieldVault contract. These tests ensure that the contract's mathematical and economic properties hold under various conditions, including attack scenarios.

Key test cases include:

1. **test_total_assets_supply_invariant**: Tests that the total assets and total supply invariants are maintained even when a malicious user tries to manipulate the protocol
2. **test_yield_accrual_invariant**: Tests that the yield accrual mechanism cannot be exploited
3. **test_authorization_invariant**: Tests that the token-based authorization prevents unauthorized withdrawals
4. **test_conversion_invariants**: Tests that the conversion functions maintain their mathematical properties
5. **test_preview_function_invariants**: Tests that the preview functions accurately predict the actual operations

### Token-Based Architecture Testing

The YieldVault contract now uses a token-based architecture, which means:

1. The contract doesn't track individual account balances internally
2. Authorization is handled through token possession
3. The contract only tracks global state (total supply, total assets)

This architecture is tested in several ways:

1. **Token Verification**: Tests verify that the contract correctly checks for the presence of tokens in the transaction
2. **Global State Management**: Tests verify that the contract correctly updates the global state
3. **Authorization**: Tests verify that only users with the appropriate tokens can perform certain operations

### Simple Utils Tests

The `simple_utils_test.rs` file contains tests for utility functions that are used throughout the contract. These tests are completely independent of the runtime environment.

Key test cases include:

1. **test_basis_point_math**: Tests the basis point calculations
2. **test_overflow_protection**: Tests the overflow protection mechanisms
3. **test_conversion_functions**: Tests the conversion between assets and shares

## Test Fixtures

The tests use several fixtures to set up the testing environment:

1. **MockYieldVault**: A mock implementation of the YieldVault contract
2. **MockStorage**: A mock implementation of the storage layer
3. **TestContext**: A context object that simulates the blockchain environment

## Interpreting Test Results

When running tests, you'll see output similar to:

```
running 5 tests
test test_initialization ... ok
test test_deposit_withdraw ... ok
test test_mint_redeem ... ok
test test_yield_accrual ... ok
test test_preview_functions ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

This indicates that all tests passed. If a test fails, you'll see detailed information about the failure, including the line number and the expected vs. actual values.

## Adding New Tests

To add a new test:

1. Create a new test function in the appropriate test file
2. Use the `#[test]` attribute to mark it as a test
3. Use assertions to verify the expected behavior

Example:

```rust
#[test]
fn test_new_feature() {
    // Set up the test environment
    let mut vault = MockYieldVault::new();
    vault.initialize("YieldVault", "YVT", "Bitcoin", "BTC", 8);
    
    // Test the feature
    let result = vault.some_feature();
    
    // Verify the result
    assert_eq!(result, expected_value);
}
```

## Troubleshooting Common Test Issues

### 1. "cannot find function/method/struct/etc in this scope"

This usually means you're missing an import or the item is not public. Check that:
- The item is properly exported from its module
- You've imported it correctly in your test file
- The item is marked as `pub` if it needs to be accessed from outside its module

### 2. "thread 'test_name' panicked at..."

This indicates that an assertion failed. The error message will show:
- The file and line number where the assertion failed
- The expected and actual values
- Any custom message provided to the assertion

### 3. Tests Hang or Timeout

This could indicate an infinite loop or a deadlock. Check for:
- Recursive functions without proper termination conditions
- Waiting for events that never occur
- Resource contention issues

### 4. "linking with `cc` failed"

This usually indicates a problem with native dependencies. Make sure you have:
- The required system libraries installed
- The correct toolchain for your platform
- Set any necessary environment variables

## WebAssembly Testing

Testing the WebAssembly contract requires additional setup:

1. Build the WebAssembly contract:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Run the WebAssembly tests:
   ```bash
   cargo test --test wasm_tests
   ```

Note that WebAssembly tests are currently not active and may require additional setup.

## OylNet Deployment Testing

To test the contract on the OylNet network:

1. Deploy the contract:
   ```bash
   ./deployment/deploy_yield_vault.sh
   ```

2. Select option 5 for the full deployment process

3. Verify the contract state:
   ```bash
   cd deployment && node contract_interaction.js
   ```

This will deploy the contract, initialize it, and verify that it's working correctly on the OylNet network.

## Continuous Integration

The project does not currently have continuous integration set up, but it would be a valuable addition to automatically run tests on every commit.

## Test Coverage

Test coverage is not currently measured, but it would be useful to add a coverage tool to identify areas of the code that are not well-tested.

## Conclusion

The YieldVault project has a robust testing infrastructure that ensures the contract functions correctly. By running the tests regularly and adding new tests as features are added, we can maintain high quality and reliability.
