# Yield Vault Project Setup Guide

This guide provides step-by-step instructions for setting up the Yield Vault project from a fresh clone.

## Prerequisites

- Node.js and npm
- Rust and Cargo
- Git

## Initial Setup

### 1. Clone the Repository

```bash
git clone <repository-url>
cd yield-vault
```

### 2. Set Up Environment Variables

Create a `.env` file in the root directory with the following content:

```
PROVIDER=oylnet
NETWORK=regtest
API_KEY=lasereyes
```

### 3. Install Node.js Dependencies

```bash
# Install dependencies in the oyl-sdk directory
cd oyl-sdk
npm install
cd ..
```

### 4. Set Up Rust and Cargo

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Add WebAssembly target
rustup target add wasm32-unknown-unknown
```

## Building the Project

```bash
# Build the project
./yield-vault.sh build
```

## Running Tests

```bash
# Run the active tests
./yield-vault.sh test
```

## Deploying the Contract

```bash
# Deploy a fresh contract
./yield-vault.sh deploy
```

## Project Structure

- **src/**: Source code for the Yield Vault contract
- **tests/**: Test files for the project
- **bin/**: Build and test scripts
- **deployment/**: Deployment scripts and configuration
- **memory-bank/**: Project documentation and notes
- **oyl-sdk/**: SDK for interacting with the OylNet network
- **secp256k1-sys/**: Fork of the secp256k1-sys crate for cross-platform compatibility

## Available Commands

The `yield-vault.sh` script provides a unified interface for common operations:

```bash
# Show help information
./yield-vault.sh help

# Build the WebAssembly contract
./yield-vault.sh build

# Run the tests
./yield-vault.sh test

# Prune deprecated test files
./yield-vault.sh prune

# Deploy a fresh contract to OylNet
./yield-vault.sh deploy

# Interact with OylNet network
./yield-vault.sh net --deploy
```

## Troubleshooting

### Building on Apple Silicon (M1/M2/M3)

If you're using an Apple Silicon Mac, you may need to install LLVM:

```bash
arch -x86_64 /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)"
arch -x86_64 /usr/local/bin/brew install llvm
export PATH="/usr/local/opt/llvm/bin:$PATH"
```

Then build with:

```bash
./yield-vault.sh build --minimal
```

### OylNet Connection Issues

If you're having trouble connecting to OylNet, make sure your `.env` file is correctly set up and that you've installed the dependencies in the oyl-sdk directory.
