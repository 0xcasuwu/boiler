# YieldVault WebAssembly Build Process

## Overview

The YieldVault smart contract requires compilation to WebAssembly (WASM) for deployment on the Bitcoin blockchain. This document outlines the build process, architecture-specific considerations, and key components involved in generating deployment-ready WebAssembly binaries.

## Build Architecture

The build process uses a multi-stage approach:

1. **Standard Rust compilation** - Initial compilation of Rust code
2. **WebAssembly cross-compilation** - Targeting the wasm32-unknown-unknown platform
3. **Post-processing** - Compression and encoding for testing and deployment

### Key Components

- `build.rs` - Custom build script handling WebAssembly compilation
- `src/tests/std/` - Directory for test files including WebAssembly binaries
- `alkanes/target/` - Target directory for WebAssembly output files

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

## Build Command for WebAssembly

To build the WebAssembly target with proper configuration:

```bash
PATH="/usr/local/opt/llvm/bin:$PATH" CC="/usr/local/opt/llvm/bin/clang" AR="/usr/local/opt/llvm/bin/llvm-ar" RUSTFLAGS="-C embed-bitcode=no" cargo build --target wasm32-unknown-unknown --release
```

This command:
1. Sets the PATH to include LLVM binaries
2. Configures the C compiler (CC) to use LLVM's clang
3. Sets the archive tool (AR) to LLVM's llvm-ar
4. Sets Rust flags to avoid embedding bitcode
5. Specifies the WebAssembly target
6. Uses release optimization

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

1. **WebAssembly Binary**: `alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm` (~266KB)
2. **Compressed WebAssembly**: `alkanes/target/wasm32-unknown-unknown/release/yield_vault.wasm.gz` (~140KB)
3. **Test Build File**: `src/tests/std/yield_vault_build.rs` (~532KB)
4. **Test Module File**: `src/tests/std/mod.rs` (Small module definition)

## Common Issues and Solutions

### secp256k1-sys Compilation Failures

**Issue**: The secp256k1-sys crate often fails to compile on Mac M1 systems due to architecture incompatibilities.

**Solutions**:
1. Use Rosetta 2 with Homebrew to install LLVM
2. Set proper environment variables for cross-compilation
3. Use the provided build command with explicit tool paths

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

## Integration with Testing

The generated WebAssembly binary and associated files enable WebAssembly-based testing:

1. Tests can import the WebAssembly bytes: `use crate::tests::std::yield_vault_build::get_bytes;`
2. These bytes can be used to instantiate the WebAssembly module for testing
3. This approach ensures tests run against the actual WebAssembly that will be deployed

## Future Improvements

1. Implement incremental WebAssembly builds to reduce compilation time
2. Add WebAssembly size benchmarking to track binary size over time
3. Explore additional optimization techniques using wasm-opt
4. Consider implementing custom allocators for WebAssembly memory efficiency
