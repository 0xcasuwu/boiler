# secp256k1-sys WebAssembly Fork

This is a WebAssembly-compatible fork of the [secp256k1-sys](https://github.com/rust-bitcoin/rust-secp256k1) library designed specifically to enable building Bitcoin smart contracts on Apple Silicon (M1/M2/M3) Macs.

## Purpose

The official secp256k1-sys crate relies on compiling C code, which can cause challenges when:
- Cross-compiling to WebAssembly target
- Building on Apple Silicon architecture
- Working in environments where a C toolchain is unavailable

This fork provides stub implementations that satisfy the API requirements of secp256k1-sys without requiring native C compilation. This enables successful builds of projects that depend on secp256k1-sys when targeting WebAssembly, particularly on Apple Silicon.

## Important Note

This is **NOT** a cryptographically secure implementation. It provides stub functions that return successful results but do not perform actual cryptographic operations. Use this fork **only** for:

- Building WebAssembly targets for testing/development
- Bypassing compilation issues on Apple Silicon
- Development environments where cryptographic verification is not required

**DO NOT** use this fork for:
- Production systems requiring actual cryptographic operations
- Any environment where security is required
- Validating cryptographic signatures

## Usage

Add this fork to your `Cargo.toml` file:

```toml
[dependencies]
secp256k1-sys = { path = "/path/to/fork-repos/secp256k1-sys" }
```

Or use a patch section to override the dependency for all dependencies:

```toml
[patch.crates-io]
secp256k1-sys = { path = "/path/to/fork-repos/secp256k1-sys" }
```

To use with a specific dependency:

```toml
[patch."https://github.com/alkimake/secp256k1-sys"]
secp256k1-sys = { path = "/path/to/fork-repos/secp256k1-sys" }
```

## Features

This fork supports the same feature flags as the original crate:

- `recovery` - Enable recovery module
- `endomorphism` - Enable endomorphism optimization
- `lowmemory` - Optimize for memory instead of speed
- `static-secp` - Use a statically built version of libsecp256k1 (no-op in this stub version)
- `std` - Enable std library integration

## Building WebAssembly on Apple Silicon

For optimal building on Apple Silicon:

```bash
# Set environment variables for Apple Silicon builds
export PATH="/usr/local/opt/llvm/bin:$PATH"
export CC="/usr/local/opt/llvm/bin/clang"
export AR="/usr/local/opt/llvm/bin/llvm-ar" 
export RUSTFLAGS="-C embed-bitcode=no"

# Build with WebAssembly target
cargo build --target wasm32-unknown-unknown --release
```

## License

This fork is distributed under the same license as the original secp256k1-sys crate (Apache-2.0).
