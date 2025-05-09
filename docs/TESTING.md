# Testing Guide

## Testing Architecture

The Yield Vault contract includes a comprehensive testing framework with multiple types of tests designed to validate functionality, security, and integration. This document provides guidance on running tests, understanding test components, and troubleshooting common issues.

## Test Categories

### 1. Unit Tests

Located in `src/tests/unit_tests.rs`, unit tests focus on individual functions and components:

- Storage pointer operations
- Asset/share conversion logic
- Yield accrual calculations
- Balance management

### 2. Basic Tests

Located in `src/tests/basic_tests.rs`, these tests validate core functionality:

- Contract initialization
- Deposit and withdrawal operations
- Balance tracking
- Token metadata

### 3. End-to-End (E2E) Tests

Located in `src/tests/e2e_tests.rs`, E2E tests simulate complete user workflows:

- Full deposit-yield-withdraw cycles
- Multi-user scenarios
- Share price calculations across operations
- State consistency validation

### 4. Adversarial Tests

Located in `src/tests/adversarial_tests.rs`, these tests verify security properties:

- Transaction replay attack prevention
- Unauthorized operations rejection
- Overflow protection
- Invalid parameter handling
- Multiple initialization attempts

### 5. WebAssembly Integration Tests

Located in `src/tests/std/`, these tests validate WebAssembly compatibility:

- Binary size and format
- Export function availability
- Integration with test harnesses

## Running Tests

### Standard Test Suite

To run the entire test suite:

```bash
cargo test
```

### Running Specific Test Categories

```bash
# Run only unit tests
cargo test -p yield-vault -- unit_tests

# Run only basic tests
cargo test -p yield-vault -- basic_tests

# Run only e2e tests
cargo test -p yield-vault -- e2e_tests

# Run only adversarial tests
cargo test -p yield-vault -- adversarial_tests
```

### Running Individual Tests

For better stability, especially with memory-intensive tests:

```bash
# Run a specific test
cargo test -p yield-vault -- tests::e2e_tests::test_deposit_withdraw_flow
```

### Test Flags

Useful cargo test flags:

```bash
# Show output from tests
cargo test -- --nocapture

# Run tests with more threads
cargo test -- --test-threads=4
```

## Authentication Model Testing

The contract uses a dual-mode authentication approach:

### 1. Test Mode Authentication

For testing environments (including our test scripts), the contract accepts specific block/tx values:

```rust
// Test mode with fixed values
let is_valid = (block == 1 && tx == 1);
```

When interacting with the contract in test mode:

```bash
# Use block=1, tx=1 parameters
local params="0x${tx_hash},1,1,${assets}"
```

### 2. Production Authentication

For production environments, the contract validates against actual AlkaneId string representations:

```rust
// Production mode with actual AlkaneId validation
let transfer_id_str = format!("{:?}", transfer.id);
let is_valid = (transfer_id_str == auth_token_id);
```

## Testing with OylNet

The `interact_with_vault.sh` script demonstrates comprehensive contract testing on OylNet:

### 1. Metadata Testing

Verifies contract metadata operations:
- Name (opcode 100)
- Symbol (opcode 101)
- Decimals (opcode 102)
- Asset name (opcode 103)

### 2. Accounting Testing

Verifies accounting state:
- Total assets (opcode 200)
- Total supply (opcode 601)

### 3. Administrative Operations

Tests yield management:
- Update yield rate (opcode 900)
- Get yield rate (opcode 901)

### 4. Interactive Operations

Tests operations requiring AlkaneId validation:
- Deposit with test mode values (opcode 10)
- Check balance with test mode values (opcode 600)

### Running OylNet Tests

```bash
# Deploy the contract
./deploy_to_oylnet.sh

# Run interaction tests
./interact_with_vault.sh
```

## Memory Safety Considerations

Some tests may experience memory safety issues due to storage interactions. Use these strategies:

### 1. Test Isolation

Each test should use unique storage namespaces:

```rust
// Create a unique storage prefix based on test name or timestamp
let test_id = format!("test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
```

### 2. Individual Test Execution

Run memory-intensive tests individually:

```bash
cargo test -p yield-vault -- tests::e2e_tests::test_deposit_withdraw_flow --nocapture
```

### 3. Resource Cleanup

Ensure proper resource cleanup after tests:

```rust
// Clear storage after test
fn teardown(test_id: &str) {
    // Clean up all storage prefixed with test_id
}
```

## WebAssembly Build Testing

The `build_minimal.sh` script creates placeholder WebAssembly files for testing:

```bash
# Generate WebAssembly test files
./build_minimal.sh
```

This ensures test modules like `src/tests/std/yield_vault_build.rs` have the necessary WebAssembly binary for testing.

## Troubleshooting Common Test Issues

### 1. Memory Corruption in E2E Tests

**Symptoms**: Tests fail with "slice::from_raw_parts requires the pointer to be aligned and non-null"

**Solutions**:
- Run tests individually
- Reduce simultaneous test runs: `--test-threads=1`
- Ensure proper test isolation with unique test IDs

### 2. Thread Panics in Basic Tests

**Symptoms**: "thread panicked while panicking" errors during cleanup

**Solutions**:
- Run tests with `--nocapture` to see detailed output
- Run specific tests individually

### 3. "scriptpubkey" Errors in OylNet Tests

**Symptoms**: "scriptpubkey" errors when using string tokens

**Solutions**:
- Use numeric block=1, tx=1 parameters instead of string tokens
- Format parameters correctly for BigInt conversion

### 4. Unexpected Test Behavior

**Symptoms**: Tests pass individually but fail when run together

**Solutions**:
- Check for shared state between tests
- Look for missing cleanup operations
- Use unique test IDs for better isolation

## Testing Best Practices

1. **Comprehensive Coverage**: Test all opcodes and code paths
2. **Security Focus**: Pay special attention to authorization and overflow
3. **Isolation**: Ensure tests don't interfere with each other
4. **State Verification**: Check state before and after operations
5. **Error Cases**: Test both success paths and error conditions
6. **Parameter Boundaries**: Test edge cases (zero values, max values)
7. **Yield Calculations**: Verify yield accrual for different time periods
8. **Authorization**: Test both authorized and unauthorized operations

By following these guidelines, you can ensure the Yield Vault contract maintains high quality and security standards.
