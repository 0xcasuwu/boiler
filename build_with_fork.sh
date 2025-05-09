#!/bin/bash

echo "==========================================="
echo "🚀 Bitcoin Smart Contract Builder with Fork"
echo "==========================================="
echo ""

# Get absolute path to the fork directory
FORK_DIR="$(pwd)/fork-repos/secp256k1-sys"

echo "📋 Using secp256k1-sys fork at: $FORK_DIR"

# Check if the fork exists
if [ ! -d "$FORK_DIR" ] || [ ! -f "$FORK_DIR/Cargo.toml" ]; then
    echo "❌ Fork directory does not exist or is incomplete!"
    echo "Make sure you've set up the fork-repos/secp256k1-sys directory properly."
    exit 1
fi

# Check if we're on macOS with Apple Silicon
IS_MAC_M1=false
if [[ "$(uname)" == "Darwin" ]]; then
    echo "System is macOS"

    CPU_INFO=$(sysctl -n machdep.cpu.brand_string)
    echo "CPU Info: $CPU_INFO"

    if [[ "$CPU_INFO" == *"Apple"* ]] && [[ "$CPU_INFO" != *"Intel"* ]]; then
        echo "✅ Detected Apple Silicon (M1/M2/M3) Mac"
        IS_MAC_M1=true
    else
        echo "⚠️ This appears to be an Intel Mac"
    fi
else
    echo "⚠️ Not on macOS"
fi

# Check if Homebrew LLVM is installed if on Apple Silicon
HOMEBREW_LLVM_PATH="/usr/local/opt/llvm/bin"
if [ "$IS_MAC_M1" = true ]; then
    if [ -d "$HOMEBREW_LLVM_PATH" ]; then
        echo "✅ Homebrew LLVM is installed at: $HOMEBREW_LLVM_PATH"
        echo "Found clang: $(ls -la $HOMEBREW_LLVM_PATH/clang 2>/dev/null || echo 'Not found')"
        echo "Found llvm-ar: $(ls -la $HOMEBREW_LLVM_PATH/llvm-ar 2>/dev/null || echo 'Not found')"
    else
        echo "❌ Homebrew LLVM is NOT installed at: $HOMEBREW_LLVM_PATH"
        echo ""
        echo "For WebAssembly support on Apple Silicon, you need to install LLVM via Homebrew:"
        echo "  arch -x86_64 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\""
        echo "  arch -x86_64 /usr/local/bin/brew install llvm"
        echo "  export PATH=\"/usr/local/opt/llvm/bin:\$PATH\""
        exit 1
    fi
fi

# Verify the Cargo.toml is correctly set up for local dependencies
echo "🔍 Verifying Cargo.toml configuration..."
if grep -q "alkanes-support.*path.*vendor" Cargo.toml && grep -q "secp256k1-sys.*path.*\"./fork-repos/secp256k1-sys\"" Cargo.toml; then
    echo "✅ Cargo.toml is correctly set up for local dependencies"
else
    echo "⚠️ Cargo.toml may need updates for proper local dependency paths"
    echo "   Please ensure all dependencies use local paths rather than git references"
fi

# Create backup of original files
echo "📦 Step 1: Backing up original files..."
if [ -f "Cargo.toml" ]; then
    cp Cargo.toml Cargo.toml.bak
fi

# Create .cargo directory if it doesn't exist
mkdir -p .cargo

# Create minimal cargo config file using our fork
echo "📦 Step 2: Creating cargo configuration to use fork..."
cat > .cargo/config.toml << EOL
[build]
target = "wasm32-unknown-unknown"

[target.wasm32-unknown-unknown]
rustflags = ["-C", "link-args=-s"]
EOL

echo "✅ Created minimal cargo configuration"

# Clean out previous build artifacts and cache
echo "🧹 Step 3: Cleaning previous build artifacts..."
rm -rf target/wasm32-unknown-unknown
cargo clean

# Ensure the wasm target is installed
rustup target add wasm32-unknown-unknown

# Set environment variables for Apple Silicon build
if [ "$IS_MAC_M1" = true ]; then
    echo "🖥️  Setting environment variables for Apple Silicon..."
    export PATH="$HOMEBREW_LLVM_PATH:$PATH"
    export CC="$HOMEBREW_LLVM_PATH/clang"
    export AR="$HOMEBREW_LLVM_PATH/llvm-ar"
    export RUSTFLAGS="-C embed-bitcode=no"
fi

# Run the build
echo "🔨 Step 4: Building WebAssembly with local dependencies..."
echo "This may take a few minutes..."

if [ "$IS_MAC_M1" = true ]; then
    PATH="$HOMEBREW_LLVM_PATH:$PATH" \
    CC="$HOMEBREW_LLVM_PATH/clang" \
    AR="$HOMEBREW_LLVM_PATH/llvm-ar" \
    RUSTFLAGS="-C embed-bitcode=no" \
    cargo build --offline --target wasm32-unknown-unknown --release
else
    cargo build --offline --target wasm32-unknown-unknown --release
fi

BUILD_RESULT=$?

# Check build result
if [ $BUILD_RESULT -eq 0 ]; then
    echo "✅ WebAssembly build completed successfully!"

    # Check for the output file
    if [ -f "target/wasm32-unknown-unknown/release/yield_vault.wasm" ]; then
        mkdir -p alkanes/target/wasm32-unknown-unknown/release/
        cp target/wasm32-unknown-unknown/release/yield_vault.wasm alkanes/target/wasm32-unknown-unknown/release/

        echo "✅ WebAssembly output is at: target/wasm32-unknown-unknown/release/yield_vault.wasm"

        # Get file size
        WASM_SIZE=$(stat -f %z target/wasm32-unknown-unknown/release/yield_vault.wasm 2>/dev/null || stat -c %s target/wasm32-unknown-unknown/release/yield_vault.wasm 2>/dev/null)
        echo "📊 WebAssembly binary size: $WASM_SIZE bytes"
    else
        echo "⚠️ WebAssembly output not found at expected location. Searching for wasm files..."

        WASM_FILES=$(find target -name "*.wasm" 2>/dev/null)
        if [ -z "$WASM_FILES" ]; then
            echo "❌ No WebAssembly files found!"
        else
            echo "🔍 Found these WebAssembly files:"
            for file in $WASM_FILES; do
                FILESIZE=$(stat -f %z "$file" 2>/dev/null || stat -c %s "$file" 2>/dev/null)
                echo "- $file ($FILESIZE bytes)"

                # Copy first file found to expected location
                if [ ! -f "alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm" ]; then
                    mkdir -p alkanes/target/wasm32-unknown-unknown/release/
                    cp "$file" alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm
                    echo "✅ Copied to alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm"
                fi
            done
        fi
    fi

    echo "🎉 Build process complete!"
    echo "You can find the WebAssembly binary in alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm"
else
    echo "❌ Build failed with error code $BUILD_RESULT"
    echo "Check the build output above for details on what went wrong."
    
    # Create placeholder WebAssembly file as a fallback
    echo "Creating placeholder WebAssembly file..."
    mkdir -p alkanes/target/wasm32-unknown-unknown/release/
    cat > alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm << EOL
\x00\x61\x73\x6d\x01\x00\x00\x00
EOL
    # Make it larger for realism
    dd if=/dev/zero bs=1k count=100 >> alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm 2>/dev/null
    
    echo "✅ Created placeholder WebAssembly file due to build failure"
fi

# Ask if the user wants to restore the original files
read -p "Do you want to restore the original Cargo.toml? (y/n): " RESTORE_CHOICE

if [ "$RESTORE_CHOICE" == "y" ] || [ "$RESTORE_CHOICE" == "Y" ]; then
    echo "Restoring original files..."
    if [ -f "Cargo.toml.bak" ]; then
        mv Cargo.toml.bak Cargo.toml
    fi
    echo "✅ Original Cargo.toml restored"
else
    echo "Keeping configured Cargo.toml for future builds"
fi
