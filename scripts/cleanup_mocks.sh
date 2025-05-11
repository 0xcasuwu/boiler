#!/bin/bash
# cleanup_mocks.sh
# Script to clean up mock files and fix some common issues

echo "Starting cleanup process..."

# Remove mock files
echo "Removing mock implementation files..."
if [ -f src/mock_deps.rs ]; then
    rm src/mock_deps.rs
    echo "Removed src/mock_deps.rs"
fi

if [ -f src/mock_impl.rs ]; then
    rm src/mock_impl.rs
    echo "Removed src/mock_impl.rs"
fi

if [ -f src/mock_metashrew.rs ]; then
    rm src/mock_metashrew.rs
    echo "Removed src/mock_metashrew.rs"
fi

# Run cargo fix to automatically fix some issues
echo "Running cargo fix to auto-fix common issues..."
cargo fix --allow-dirty

# Run cargo clippy to find further improvements
echo "Running cargo clippy for additional suggestions..."
cargo clippy

echo "Cleanup complete!"
echo "Note: You may still need to manually address some warnings and errors."
echo "Check the output of cargo clippy for additional suggestions."
