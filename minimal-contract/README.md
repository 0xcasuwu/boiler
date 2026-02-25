# Minimal YieldVault Contract

A minimal WebAssembly contract for Oylnet using the alkanes-rs framework.

## Overview

This is a simple YieldVault contract that implements the `Token` trait, defining basic functionality for a token with name and symbol. The contract is designed to be deployed to the OylNet blockchain.

## Features

- Token implementation with name and symbol
- Message handling for initialization, name, and symbol retrieval
- Storage management for contract state

## Dependencies

- alkanes-runtime
- alkanes-support
- metashrew-support
- anyhow

## Building

To build the contract:

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled WebAssembly binary will be available at `target/wasm32-unknown-unknown/release/minimal_yield_vault.wasm`.

## Deployment

To deploy the contract to OylNet:

```bash
oyl alkane new-contract -c ./target/wasm32-unknown-unknown/release/minimal_yield_vault.wasm -data 0 -p oylnet
```

## License

MIT
