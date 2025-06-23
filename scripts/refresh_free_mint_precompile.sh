``````````````````````#!/bin/bash

# Refresh Free-Mint Precompiles Script
# Usage: ./refresh_free_mint_precompile.sh [--force-rebuild] [--alkamist-only] [--dust-only]
#
# This script automates the complete process of refreshing both free-mint precompiles:
# 1. Builds alkanes-build binary (if needed)
# 2. Builds ALKAMIST and DUST WASM modules
# 3. Converts WASM to Rust source code
# 4. Updates the precompiled modules in boiler

set -e

# Configuration
ALKANES_RS_DIR="/home/e/Documents/alkanes-rs"
FREE_MINT_DIR="/home/e/Documents/free-mint"
BOILER_DIR="/home/e/Documents/boiler"
TARGET_ARCH="aarch64-unknown-linux-gnu"

# Derived paths
ALKANES_BUILD_BINARY="$ALKANES_RS_DIR/target/$TARGET_ARCH/release/alkanes-build"
ALKAMIST_WASM="$FREE_MINT_DIR/target/wasm32-unknown-unknown/release/alkamist.wasm"
DUST_WASM="$FREE_MINT_DIR/target/wasm32-unknown-unknown/release/dust.wasm"
ALKAMIST_OUTPUT="$BOILER_DIR/src/precompiled/alkamist_build.rs"
DUST_OUTPUT="$BOILER_DIR/src/precompiled/dust_build.rs"

# Parse arguments
FORCE_REBUILD=false
BUILD_ALKAMIST=true
BUILD_DUST=true

for arg in "$@"; do
    case $arg in
        --force-rebuild)
            FORCE_REBUILD=true
            ;;
        --alkamist-only)
            BUILD_ALKAMIST=true
            BUILD_DUST=false
            ;;
        --dust-only)
            BUILD_ALKAMIST=false
            BUILD_DUST=true
            ;;
        *)
            echo "Unknown argument: $arg"
            echo "Usage: $0 [--force-rebuild] [--alkamist-only] [--dust-only]"
            exit 1
            ;;
    esac
done

echo "🔄 FREE-MINT PRECOMPILES REFRESH"
echo "================================="
echo "Source: $FREE_MINT_DIR"
if [[ "$BUILD_ALKAMIST" == true ]]; then
    echo "ALKAMIST Output: $ALKAMIST_OUTPUT"
fi
if [[ "$BUILD_DUST" == true ]]; then
    echo "DUST Output: $DUST_OUTPUT"
fi
echo "Target: $TARGET_ARCH"
echo ""

# Utility functions
check_directory() {
    local dir="$1"
    local name="$2"
    if [[ ! -d "$dir" ]]; then
        echo "❌ Error: $name directory not found: $dir"
        echo "   Please ensure the directory exists and is accessible"
        exit 1
    fi
}

check_rust_targets() {
    echo "🔍 Checking Rust targets..."
    
    # Check wasm32-unknown-unknown target
    if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
        echo "📦 Installing wasm32-unknown-unknown target..."
        rustup target add wasm32-unknown-unknown
    fi
    
    # Check native target (for building alkanes-build)
    if ! rustup target list --installed | grep -q "$TARGET_ARCH"; then
        echo "📦 Installing $TARGET_ARCH target..."
        rustup target add "$TARGET_ARCH"
    fi
    
    echo "✅ Rust targets verified"
}

build_alkanes_build() {
    echo "🔨 STEP 1: Building alkanes-build binary..."
    
    if [[ -f "$ALKANES_BUILD_BINARY" && "$FORCE_REBUILD" == false ]]; then
        echo "✅ alkanes-build binary already exists: $ALKANES_BUILD_BINARY"
        echo "   Use --force-rebuild to rebuild anyway"
        return 0
    fi
    
    echo "🏗️  Building alkanes-build for $TARGET_ARCH..."
    cd "$ALKANES_RS_DIR"
    
    # Build with progress indication
    cargo build --release --package alkanes-build --target "$TARGET_ARCH" || {
        echo "❌ Failed to build alkanes-build binary"
        echo "   Check that alkanes-rs directory is valid and up to date"
        exit 1
    }
    
    # Verify the binary was created
    if [[ ! -f "$ALKANES_BUILD_BINARY" ]]; then
        echo "❌ alkanes-build binary not found after build"
        echo "   Expected: $ALKANES_BUILD_BINARY"
        exit 1
    fi
    
    # Check if binary is executable
    if [[ ! -x "$ALKANES_BUILD_BINARY" ]]; then
        echo "🔧 Making alkanes-build executable..."
        chmod +x "$ALKANES_BUILD_BINARY"
    fi
    
    echo "✅ alkanes-build binary ready"
}

build_free_mint_wasms() {
    echo "🔨 STEP 2: Building free-mint WASM modules..."
    
    cd "$FREE_MINT_DIR"
    
    # Build ALKAMIST if requested
    if [[ "$BUILD_ALKAMIST" == true ]]; then
        echo "🏗️  Compiling ALKAMIST to WASM..."
        cargo build --release --target wasm32-unknown-unknown --features alkamist --no-default-features || {
            echo "❌ Failed to build ALKAMIST WASM"
            echo "   Check that alkamist feature compiles successfully"
            exit 1
        }
        
        # Copy the generic WASM to the specific name
        cp "$FREE_MINT_DIR/target/wasm32-unknown-unknown/release/free_mint.wasm" "$ALKAMIST_WASM"
        
        # Verify WASM file was created
        if [[ ! -f "$ALKAMIST_WASM" ]]; then
            echo "❌ ALKAMIST WASM file not found after build"
            echo "   Expected: $ALKAMIST_WASM"
            exit 1
        fi
        
        # Show WASM file info
        local alkamist_size=$(du -h "$ALKAMIST_WASM" | cut -f1)
        echo "✅ ALKAMIST WASM module ready ($alkamist_size)"
    fi
    
    # Build DUST if requested
    if [[ "$BUILD_DUST" == true ]]; then
        echo "🏗️  Compiling DUST to WASM..."
        cargo build --release --target wasm32-unknown-unknown --features dust --no-default-features || {
            echo "❌ Failed to build DUST WASM"
            echo "   Check that dust feature compiles successfully"
            exit 1
        }
        
        # Copy the generic WASM to the specific name
        cp "$FREE_MINT_DIR/target/wasm32-unknown-unknown/release/free_mint.wasm" "$DUST_WASM"
        
        # Verify WASM file was created
        if [[ ! -f "$DUST_WASM" ]]; then
            echo "❌ DUST WASM file not found after build"
            echo "   Expected: $DUST_WASM"
            exit 1
        fi
        
        # Show WASM file info
        local dust_size=$(du -h "$DUST_WASM" | cut -f1)
        echo "✅ DUST WASM module ready ($dust_size)"
    fi
}

convert_wasms_to_rust() {
    echo "🔨 STEP 3: Converting WASM files to Rust source..."
    
    # Ensure output directory exists
    mkdir -p "$(dirname "$ALKAMIST_OUTPUT")"
    mkdir -p "$(dirname "$DUST_OUTPUT")"
    
    # Convert ALKAMIST if requested
    if [[ "$BUILD_ALKAMIST" == true ]]; then
        echo "🔄 Converting ALKAMIST WASM to Rust..."
        
        # Backup existing file if it exists
        if [[ -f "$ALKAMIST_OUTPUT" ]]; then
            local backup_file="${ALKAMIST_OUTPUT}.backup.$(date +%Y%m%d_%H%M%S)"
            echo "💾 Backing up existing ALKAMIST file to: $backup_file"
            cp "$ALKAMIST_OUTPUT" "$backup_file"
        fi
        
        "$ALKANES_BUILD_BINARY" \
            --input "$ALKAMIST_WASM" \
            --output "$ALKAMIST_OUTPUT" || {
            echo "❌ Failed to convert ALKAMIST WASM to Rust source"
            echo "   Check alkanes-build binary and input WASM file"
            exit 1
        }
        
        # Verify output file was created
        if [[ ! -f "$ALKAMIST_OUTPUT" ]]; then
            echo "❌ ALKAMIST output file not found after conversion"
            echo "   Expected: $ALKAMIST_OUTPUT"
            exit 1
        fi
        
        # Show output file info
        local alkamist_rust_size=$(du -h "$ALKAMIST_OUTPUT" | cut -f1)
        local alkamist_rust_lines=$(wc -l < "$ALKAMIST_OUTPUT")
        echo "✅ ALKAMIST Rust source generated ($alkamist_rust_size, $alkamist_rust_lines lines)"
    fi
    
    # Convert DUST if requested
    if [[ "$BUILD_DUST" == true ]]; then
        echo "🔄 Converting DUST WASM to Rust..."
        
        # Backup existing file if it exists
        if [[ -f "$DUST_OUTPUT" ]]; then
            local backup_file="${DUST_OUTPUT}.backup.$(date +%Y%m%d_%H%M%S)"
            echo "💾 Backing up existing DUST file to: $backup_file"
            cp "$DUST_OUTPUT" "$backup_file"
        fi
        
        "$ALKANES_BUILD_BINARY" \
            --input "$DUST_WASM" \
            --output "$DUST_OUTPUT" || {
            echo "❌ Failed to convert DUST WASM to Rust source"
            echo "   Check alkanes-build binary and input WASM file"
            exit 1
        }
        
        # Verify output file was created
        if [[ ! -f "$DUST_OUTPUT" ]]; then
            echo "❌ DUST output file not found after conversion"
            echo "   Expected: $DUST_OUTPUT"
            exit 1
        fi
        
        # Show output file info
        local dust_rust_size=$(du -h "$DUST_OUTPUT" | cut -f1)
        local dust_rust_lines=$(wc -l < "$DUST_OUTPUT")
        echo "✅ DUST Rust source generated ($dust_rust_size, $dust_rust_lines lines)"
    fi
}

verify_integration() {
    echo "🔍 STEP 4: Verifying integration..."
    
    # Check that the file is valid Rust syntax by attempting to compile
    cd "$BOILER_DIR"
    echo "🧪 Testing compilation..."
    
    if cargo check --quiet 2>/dev/null; then
        echo "✅ Integration verified - project compiles successfully"
    else
        echo "⚠️  Warning: Project compilation issues detected"
        echo "   You may need to update imports or integration code"
        echo "   Run 'cargo check' for detailed error information"
    fi
}

# Main execution
echo "🔍 Pre-flight checks..."

# Check directories exist
check_directory "$ALKANES_RS_DIR" "alkanes-rs"
check_directory "$FREE_MINT_DIR" "free-mint"
check_directory "$BOILER_DIR" "boiler"

# Check and install required Rust targets
check_rust_targets

echo ""

# Execute main steps
build_alkanes_build
echo ""

build_free_mint_wasms
echo ""

convert_wasms_to_rust
echo ""

verify_integration
echo ""

# Final summary
echo "🎉 PRECOMPILES REFRESH COMPLETE!"
echo "================================="
echo "✅ alkanes-build binary: $(ls -la "$ALKANES_BUILD_BINARY" | awk '{print $5}') bytes"

if [[ "$BUILD_ALKAMIST" == true && -f "$ALKAMIST_WASM" ]]; then
    echo "✅ ALKAMIST WASM: $(ls -la "$ALKAMIST_WASM" | awk '{print $5}') bytes"
    echo "✅ ALKAMIST Rust code: $(ls -la "$ALKAMIST_OUTPUT" | awk '{print $5}') bytes"
    echo "📄 Updated ALKAMIST file: $ALKAMIST_OUTPUT"
fi

if [[ "$BUILD_DUST" == true && -f "$DUST_WASM" ]]; then
    echo "✅ DUST WASM: $(ls -la "$DUST_WASM" | awk '{print $5}') bytes"
    echo "✅ DUST Rust code: $(ls -la "$DUST_OUTPUT" | awk '{print $5}') bytes"
    echo "📄 Updated DUST file: $DUST_OUTPUT"
fi

echo ""
echo "🔄 The precompiles have been refreshed and are ready to use"
echo ""
echo "💡 Next steps:"
echo "   • Run 'cargo build' in boiler directory to verify everything compiles"
echo "   • Test your application with the updated precompiles"
echo "   • Update your deployment scripts to use both WASM files"
echo "   • Commit the updated precompiles if everything works correctly"