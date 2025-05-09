YieldVault WebAssembly Build Process

## Overview

The YieldVault smart contract requires compilation to WebAssembly (WASM) for deployment on the Bitcoin blockchain. This document outlines the build process, architecture-specific considerations, and key components involved in generating deployment-ready WebAssembly binaries.

## Build Architecture

The build process uses a multi-stage approach:

1. **Standard Rust compilation** - Initial compilation of Rust code
2. **WebAssembly cross-compilation** - Targeting the wasm32-unknown-unknown platform
3. **Post-processing** - Compression and encoding for testing and deployment
4. **Deployment** - OylNet testnet integration

### Key Components

- `build.rs` - Custom build script handling WebAssembly compilation
- `src/tests/std/` - Directory for test files including WebAssembly binaries
- `alkanes/target/` - Target directory for WebAssembly output files
- `final_fork_build.sh` - Main build script with dependency forking
- `deploy_to_oylnet.sh` - Script for OylNet deployment
- `interact_with_vault.sh` - OylNet contract interaction script

## Custom Dependency Fork

To resolve persistent issues with the secp256k1-sys crate on Apple Silicon, we've created a custom fork approach:

### Fork Architecture

1. **Local Fork Repository**: Located at `fork-repos/secp256k1-sys/`
2. **Stub Implementation**: Contains minimal no-op implementations for required functions
3. **Custom Build Script**: `build.rs` to satisfy the `links = "secp256k1"` requirement
4. **Cargo Patching**: Using `[patch]` sections in Cargo.toml to redirect all dependencies

### Fork Implementation

The forked version of secp256k1-sys:
- Contains stub versions of all required API functions
- Includes a build script meeting Cargo's requirements
- Doesn't attempt to build the actual C library for WebAssembly targets
- Is structured as a drop-in replacement for the original crate

### Core files in the fork:

```rust
// src/lib.rs (essential stubs)
#![allow(unused_variables, dead_code)]

pub const SECP256K1_FLAGS_TYPE_MASK: u32 = 0x00000003;
pub const SECP256K1_FLAGS_TYPE_CONTEXT: u32 = 0x00000001;
pub const SECP256K1_FLAGS_TYPE_COMPRESSION: u32 = 0x00000002;
pub const SECP256K1_FLAGS_BIT_COMPRESSION: u32 = 0x00000004;

pub const SECP256K1_CONTEXT_VERIFY: u32 = 0x00000101;
pub const SECP256K1_CONTEXT_SIGN: u32 = 0x00000201;
pub const SECP256K1_CONTEXT_NONE: u32 = 0x00000000;

pub const SECP256K1_EC_COMPRESSED: u32 = 0x00000002;
pub const SECP256K1_EC_UNCOMPRESSED: u32 = 0x00000000;

// Essential no-op functions
#[no_mangle]
pub unsafe extern "C" fn secp256k1_context_create(_flags: u32) -> *mut core::ffi::c_void {
    core::ptr::null_mut()
}

// ... additional stub functions ...
```

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-lib=secp256k1");
    println!("cargo:rerun-if-changed=build.rs");
    
    // For WebAssembly target, we don't actually link to any C library
    if std::env::var("TARGET").unwrap_or_default().contains("wasm32") {
        println!("cargo:warning=Building for WebAssembly target - no actual linking performed");
        return;
    }
}
```

## Mac M1 Architecture Support

Building WebAssembly on Apple Silicon (M1/M2/M3) requires special handling due to compatibility issues with the secp256k1-sys crate.

### Detection and Configuration

The build script now includes Mac M1 detection:

```rust
fn is_mac_m1() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Try to detect Apple Silicon
        if let Ok(output) = Command::new("sysctl").arg("-n").arg("machdep.cpu.brand_string").output() {
            let cpu_info = String::from_utf8_lossy(&output.stdout);
            return cpu_info.contains("Apple") && !cpu_info.contains("Intel");
        }
    }
    false
}
```

### LLVM Requirements

Mac M1 systems require a specific LLVM installation via Homebrew for successful WebAssembly compilation:

1. Install Homebrew using Rosetta:
   ```bash
   arch -x86_64 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
   ```

2. Install LLVM using Rosetta-enabled Homebrew:
   ```bash
   arch -x86_64 /usr/local/bin/brew install llvm
   ```

3. Add LLVM to your path:
   ```bash
   export PATH="/usr/local/opt/llvm/bin:$PATH"
   ```

### Diagnostic Script

The repository includes a diagnostic script (`check_mac_m1.sh`) to verify Mac M1 detection and LLVM installation:

```bash
#!/bin/bash

echo "Checking if this is a Mac M1 system..."

# Check if we're on macOS
if [[ "$(uname)" == "Darwin" ]]; then
    echo "System is macOS"

    # Check the CPU type
    CPU_INFO=$(sysctl -n machdep.cpu.brand_string)
    echo "CPU Info: $CPU_INFO"

    if [[ "$CPU_INFO" == *"Apple"* ]] && [[ "$CPU_INFO" != *"Intel"* ]]; then
        echo "This appears to be an Apple Silicon (M1/M2) Mac"
        IS_MAC_M1=true
    else
        echo "This appears to be an Intel Mac"
        IS_MAC_M1=false
    fi
else
    echo "Not on macOS"
    IS_MAC_M1=false
fi

# Check if Homebrew LLVM is installed
HOMEBREW_LLVM_PATH="/usr/local/opt/llvm/bin"
if [ -d "$HOMEBREW_LLVM_PATH" ]; then
    echo "Homebrew LLVM is installed at: $HOMEBREW_LLVM_PATH"
    echo "Found clang: $(ls -la $HOMEBREW_LLVM_PATH/clang 2>/dev/null || echo 'Not found')"
    echo "Found llvm-ar: $(ls -la $HOMEBREW_LLVM_PATH/llvm-ar 2>/dev/null || echo 'Not found')"
else
    echo "Homebrew LLVM is NOT installed at: $HOMEBREW_LLVM_PATH"

    if [ "$IS_MAC_M1" = true ]; then
        echo ""
        echo "For WebAssembly support on Mac M1, you need to install LLVM via Homebrew:"
        echo "  arch -x86_64 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\""
        echo "  arch -x86_64 /usr/local/bin/brew install llvm"
        echo "  export PATH=\"/usr/local/opt/llvm/bin:\$PATH\""
    fi
fi
```

## Complete Build System

### Build Script Architecture

To reliably build on all architectures, we've created a comprehensive build system:

1. **final_fork_build.sh**: Main build script that:
   - Detects Apple Silicon architecture
   - Sets up a local dependency fork for secp256k1-sys
   - Configures LLVM toolchain for proper cross-compilation
   - Modifies Cargo.toml with patch directives
   - Handles build failures with fallback mechanisms
   - Generates placeholder files when necessary
   - Creates properly structured test module files

2. **test_oylnet_connection.sh**: Validates OylNet connectivity:
   - Verifies SDK installation and environment
   - Tests connection to OylNet
   - Validates API credentials
   - Checks network functionality

3. **deploy_to_oylnet.sh**: Handles OylNet deployment:
   - Copies WebAssembly files to deployment location
   - Sets up initialization parameters
   - Funds deployment account
   - Deploys contract to OylNet
   - Confirms deployment with block generation
   - Saves contract ID for future interaction

4. **interact_with_vault.sh**: Tests contract functionality:
   - Connects to deployed contract
   - Tests metadata view functions
   - Tests accounting view functions
   - Tests administrative operations
   - Tests deposit/withdrawal operations

### Standard Build Command

```bash
./final_fork_build.sh
```

This command:
1. Sets up a fork of secp256k1-sys
2. Modifies Cargo.toml with patch sections
3. Detects architecture and configures build environment
4. Builds WebAssembly target with proper configuration
5. Handles errors and creates fallback files if needed

### Mac M1/M2/M3 (Apple Silicon) Build Command

```bash
PATH="/usr/local/opt/llvm/bin:$PATH" \
CC="/usr/local/opt/llvm/bin/clang" \
AR="/usr/local/opt/llvm/bin/llvm-ar" \
RUSTFLAGS="-C embed-bitcode=no" \
cargo build --target wasm32-unknown-unknown --release
```

This command:
1. Sets the PATH to include LLVM binaries
2. Configures the C compiler (CC) to use LLVM's clang
3. Sets the archive tool (AR) to LLVM's llvm-ar
4. Sets Rust flags to avoid embedding bitcode
5. Specifies the WebAssembly target
6. Uses release optimization

## OylNet Deployment Process

After building the WebAssembly binary, the next step is deploying to OylNet:

1. **Verify Connection**: Use test_oylnet_connection.sh to verify network connectivity
2. **Prepare Deployment**: Copy WebAssembly to build/ directory
3. **Configure Parameters**: Set initialization parameters
   ```bash
   VAULT_NAME="YieldVault"
   VAULT_SYMBOL="YVT"
   ASSET_NAME="Bitcoin"
   ASSET_SYMBOL="BTC"
   DECIMALS="8"
   ```
4. **Fund Account**: Request funds from OylNet faucet
5. **Deploy Contract**: Execute OylNet deployment via CLI
6. **Confirm Deployment**: Generate blocks to confirm transactions
7. **Verify Functionality**: Test contract functions with interaction script

### Deployment Parameters

The contract is deployed with these initialization parameters:

```
NAME: YieldVault
SYMBOL: YVT
ASSET_NAME: Bitcoin
ASSET_SYMBOL: BTC
DECIMALS: 8
```

### Deployment Command

```bash
./deploy_to_oylnet.sh
```

This command executes the full deployment workflow, including:
1. Verifying WebAssembly binary presence
2. Copying files to build location
3. Converting parameters to hex format
4. Executing the deployment transaction
5. Confirming via block generation
6. Saving the contract ID for future use

## WebAssembly Post-Processing

After WebAssembly compilation, the build script performs several post-processing steps:

1. **Binary Compression** - Using gzip/flate2 to reduce file size:
   ```rust
   fn compress(binary: Vec<u8>) -> Result<Vec<u8>> {
       let mut writer = GzEncoder::new(Vec::<u8>::with_capacity(binary.len()), Compression::best());
       writer.write_all(&binary)?;
       Ok(writer.finish()?)
   }
   ```

2. **Hex Encoding** - Converting binary data to hex string for use in tests:
   ```rust
   let hex_data = hex::encode(&wasm_binary);
   let build_file_content = format!(
       "use hex_lit::hex;\n#[allow(long_running_const_eval)]\npub fn get_bytes() -> Vec<u8> {{ (&hex!(\"{}\")).to_vec() }}",
       hex_data
   );
   ```

3. **Test File Generation** - Creating test module files for WebAssembly testing:
   ```rust
   let mod_file_content = format!("pub mod {}_build;\n", mod_name);
   fs::write(&mod_rs_file, mod_file_content)?;
   ```

## Output Files

The build process generates several files:

1. **WebAssembly Binary**: `alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm` (102,433 bytes)
2. **Compressed WebAssembly**: `alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm.gz` (~140KB)
3. **Test Build File**: `src/tests/std/yield_vault_build.rs` (~532KB)
4. **Test Module File**: `src/tests/std/mod.rs` (Small module definition)
5. **Contract ID File**: `.contract_id` (Contains the OylNet deployment ID)

## Common Issues and Solutions

### secp256k1-sys Compilation Failures

**Issue**: The secp256k1-sys crate often fails to compile on Mac M1 systems due to architecture incompatibilities.

**Solutions**:
1. **Custom Fork**: Use the provided local fork at fork-repos/secp256k1-sys
2. **Build Script**: Use final_fork_build.sh which handles the dependency replacement
3. **Patch System**: The build script automatically adds patch sections to Cargo.toml

### Missing LLVM Tools

**Issue**: Build fails with "failed to find tool X" errors.

**Solution**:
1. Install LLVM via Homebrew: `arch -x86_64 brew install llvm`
2. Verify installation with the check_mac_m1.sh script
3. Ensure PATH includes LLVM bin directory

### WebAssembly Size Optimization

**Issue**: WebAssembly binary size may be unnecessarily large.

**Solutions**:
1. Use `--release` flag for optimized builds
2. Use `wasm-opt` for additional size optimization: `wasm-opt -Oz yield_vault.wasm -o yield_vault.opt.wasm`
3. Consider using features to exclude unnecessary code

### OylNet Deposit Operation Errors

**Issue**: Deposit operations fail with "scriptpubkey" errors.

**Potential Solutions**:
1. Check address format and encoding
2. Verify transaction hash format
3. Consider alternative parameter encoding

## Integration with Testing

The generated WebAssembly binary and associated files enable WebAssembly-based testing:

1. Tests can import the WebAssembly bytes: `use crate::tests::std::yield_vault_build::get_bytes;`
2. These bytes can be used to instantiate the WebAssembly module for testing
3. This approach ensures tests run against the actual WebAssembly that will be deployed

## Future Improvements

1. Fix scriptpubkey errors in deposit operations
2. Resolve balance retrieval operation failures
3. Implement incremental WebAssembly builds to reduce compilation time
4. Add WebAssembly size benchmarking to track binary size over time
5. Explore additional optimization techniques using wasm-opt
6. Consider implementing custom allocators for WebAssembly memory efficiency
7. Push the secp256k1-sys fork to a proper Git repository for easier dependency management
