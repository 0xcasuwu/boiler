#!/bin/bash

# This script runs tests one by one to avoid segmentation faults
# that might happen when running all tests at once

# Make sure the script exits on any error
set -e

echo "Running Yield Vault tests from minimal_test.rs..."
echo "------------------------------------------------"

# Run individual tests from minimal_test.rs
echo "Running test_initialization..."
cargo test tests::minimal_test::test_initialization

echo "Running test_deposit_functionality..."
cargo test tests::minimal_test::test_deposit_functionality

echo "Running test_mint_functionality..."
cargo test tests::minimal_test::test_mint_functionality

echo "Running test_conversion_functions..."
cargo test tests::minimal_test::test_conversion_functions

echo "Running test_preview_functions..."
cargo test tests::minimal_test::test_preview_functions

echo "Running test_yield_accrual..."
cargo test tests::minimal_test::test_yield_accrual

echo "Running test_transaction_validation..."
cargo test tests::minimal_test::test_transaction_validation

echo "Running test_balance_management..."
cargo test tests::minimal_test::test_balance_management

echo "All tests completed successfully!"
