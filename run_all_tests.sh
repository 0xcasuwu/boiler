#!/bin/bash

echo "Running all tests in the project..."

# Run the lib tests
echo "Running lib tests..."
cargo test --lib --target x86_64-unknown-linux-gnu > lib_test_results.txt 2>&1

# Check if lib tests passed
if grep -q "test result: FAILED" lib_test_results.txt; then
    echo "Some lib tests failed. See details below:"
    grep -A 3 "test result: FAILED" lib_test_results.txt
    echo ""
    echo "For more details, check lib_test_results.txt"
else
    echo "All lib tests passed successfully!"
    # Show test summary
    grep "test result:" lib_test_results.txt
fi

# Run the inflation attack protection tests
echo "Running inflation attack protection tests..."
cargo test --test inflation_attack_protection_tests --target x86_64-unknown-linux-gnu > inflation_test_results.txt 2>&1

# Check if inflation tests passed
if grep -q "test result: FAILED" inflation_test_results.txt; then
    echo "Some inflation tests failed. See details below:"
    grep -A 3 "test result: FAILED" inflation_test_results.txt
    echo ""
    echo "For more details, check inflation_test_results.txt"
else
    echo "All inflation tests passed successfully!"
    # Show test summary
    grep "test result:" inflation_test_results.txt
fi
