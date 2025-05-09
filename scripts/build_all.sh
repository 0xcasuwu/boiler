#!/bin/bash
# Master build and verification script for Yield Vault

echo "======================================================"
echo "🚀 Yield Vault Complete Build and Verification Process"
echo "======================================================"
echo ""

# Set executable permissions for all scripts
chmod +x scripts/build.sh
chmod +x scripts/network.sh
chmod +x scripts/repo_check.sh
chmod +x scripts/check_mac_m1.sh
chmod +x scripts/cleanup.sh

# Print system info
echo "📊 System Information:"
echo "---------------------"
uname -a
echo ""

# Check for Apple Silicon and LLVM
scripts/check_mac_m1.sh
MAC_CHECK=$?

# Repository structure validation
echo "🔍 Validating Repository Structure..."
./scripts/repo_check.sh

# Build process
echo "🏗️ Starting Build Process..."

if [ "$MAC_CHECK" -eq 0 ]; then
    echo "✅ On Apple Silicon - Using optimized build script"
    ./scripts/build.sh --minimal
else
    echo "✅ On standard architecture - Using normal build script"
    ./scripts/build.sh --final
fi

# Verify the WebAssembly file exists
if [ -f "alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm" ]; then
    WASM_SIZE=$(stat -f %z alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm 2>/dev/null || stat -c %s alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm 2>/dev/null)
    echo "✅ WebAssembly binary created successfully: $WASM_SIZE bytes"
else
    echo "❌ WebAssembly binary not found!"
    exit 1
fi

echo ""
echo "✅ Build process completed successfully!"
echo ""
echo "📝 Documentation available in the docs/ directory:"
echo "  - BUILD_AND_DEPLOY.md: Complete build instructions"
echo "  - TECHNICAL_REFERENCE.md: Contract architecture and design"
echo "  - TESTING.md: Testing approach and best practices"
echo ""
echo "🧪 To deploy to OylNet: ./scripts/network.sh --deploy"
echo "🔍 To interact with deployed contract: ./scripts/network.sh --interact"
echo ""
echo "======================================================"
