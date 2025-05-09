# Yield Vault: Bitcoin Implementation of ERC-4626

A WebAssembly implementation of the ERC-4626 tokenized vault standard for Bitcoin, optimized for compatibility across all platforms including Apple Silicon.

## 🔍 Overview

This project implements a yield-bearing vault on Bitcoin, following the ERC-4626 standard. The vault allows users to deposit Bitcoin and earn yield over time, with all accounting and yield calculations handled on-chain.

## 🛠 Architecture

The contract follows the ERC-4626 interface with Bitcoin-specific adaptations:

- **Storage Pattern**: Persistent storage using StoragePointer API
- **Security Model**: Transaction replay protection and proper authorization
- **Yield Accrual**: Time-based yield that increases assets while maintaining constant shares supply
- **Opcodes**: Full implementation of asset management, accounting, and administrative operations

## 🚀 Quick Start

### Prerequisites

- Rust 1.75.0 or later
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`
- For Apple Silicon (M1/M2/M3):
  - Homebrew LLVM: `arch -x86_64 /usr/local/bin/brew install llvm`
  - LLVM in PATH: `export PATH="/usr/local/opt/llvm/bin:$PATH"`

### Building the Contract

Using the consolidated build script with auto-detection:

```bash
# Build with automatic platform detection
./scripts/build.sh
```

For specific build modes:

```bash
# For Apple Silicon optimization
./scripts/build.sh --minimal

# For standard architecture with complete build
./scripts/build.sh --final

# For development environments with custom fork
./scripts/build.sh --fork
```

### Deployment and Interaction

Test connection to OylNet:

```bash
./scripts/network.sh --test
```

Deploy the contract:

```bash
./scripts/network.sh --deploy
```

Interact with the deployed contract:

```bash
./scripts/network.sh --interact
```

## 📂 Project Structure

```
yield-vault/
├── .cargo/             # Cargo configuration
├── docs/               # Documentation
│   ├── BUILD_AND_DEPLOY.md
│   ├── TECHNICAL_REFERENCE.md
│   └── TESTING.md
├── fork-repos/         # Local dependency forks
│   └── secp256k1-sys/  # Fork optimized for Apple Silicon
├── memory-bank/        # Documentation and notes
├── scripts/            # Build and deployment scripts
│   ├── build.sh        # Consolidated build script
│   ├── network.sh      # Network operations (test, deploy, interact)
│   ├── build_all.sh    # Complete build process
│   ├── cleanup.sh      # Cleanup script
│   ├── check_mac_m1.sh # Apple Silicon detection
│   └── repo_check.sh   # Repository validation
├── src/                # Contract source code
│   ├── asset_management/
│   ├── security/
│   ├── storage/
│   ├── utils/
│   ├── lib.rs
│   └── constants.rs
├── Cargo.toml          # Project manifest
└── build.rs           # Build script
```

## 🔧 Apple Silicon Special Handling

Building WebAssembly on Apple Silicon (M1/M2/M3) requires special configuration:

1. **LLVM Requirement**: Install LLVM via Rosetta-enabled Homebrew
   ```
   arch -x86_64 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
   arch -x86_64 /usr/local/bin/brew install llvm
   ```

2. **Environment Setup**: Our build scripts automatically set the required environment variables:
   ```
   PATH="/usr/local/opt/llvm/bin:$PATH"
   CC="/usr/local/opt/llvm/bin/clang"
   AR="/usr/local/opt/llvm/bin/llvm-ar"
   RUSTFLAGS="-C embed-bitcode=no"
   ```

3. **Local Fork**: We use a custom fork of secp256k1-sys that's compatible with Apple Silicon.

## 📝 Documentation

- [Build and Deployment Guide](docs/BUILD_AND_DEPLOY.md) - Detailed instructions for building and deploying
- [Technical Reference](docs/TECHNICAL_REFERENCE.md) - Contract architecture and design
- [Testing Guide](docs/TESTING.md) - Testing approach and best practices

## 📋 Opcode Standards

The contract implements the following opcode standards:

| Opcode Range | Function Group             |
|--------------|----------------------------|
| 0            | Initialize                 |
| 10-19        | Asset Management           |
| 100-199      | Metadata View Functions    |
| 200-299      | Accounting View Functions  |
| 300-399      | Limit View Functions       |
| 400-499      | Preview View Functions     |
| 500-599      | Custom Data Operations     |
| 600-699      | Balance Management         |
| 900-999      | Administrative Operations  |

## 🤝 Contributing

Contributions are welcome! Please ensure your changes follow the project's code style and security patterns. Make sure to run the repository check script before submitting changes:

```bash
./scripts/repo_check.sh
```

## 📄 License

This project is licensed under the terms specified in the LICENSE file.
