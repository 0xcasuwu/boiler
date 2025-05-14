#!/bin/bash

echo "Running inflation attack protection tests..."

# Run each test individually
for test_name in test_virtual_offset_protection test_precision_offset test_direct_donation_prevention test_zero_amount_prevention; do
    echo "Running test: $test_name"
    output=$(cargo test --test inflation_attack_protection_tests --target x86_64-unknown-linux-gnu -- $test_name --exact 2>&1)
    if [ $? -eq 0 ]; then
        echo "  ✓ PASSED"
    else
        echo "  ✗ FAILED"
        echo "Error details:"
        echo "$output" | grep -A 10 "panicked at" | head -n 10
        echo ""
    fi
done
