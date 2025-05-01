# SLOP Contracts Deployment to Subfrost

This document provides instructions for deploying the SLOP contracts to the Subfrost environment.

## Overview

The deployment process involves:

1. Building WebAssembly binaries for all three SLOP contracts
2. Transferring these binaries to the Subfrost directory
3. Deploying contracts using a combination of `oyl` CLI and direct RPC methods
4. Running validation tests to ensure contracts are working properly

## Prerequisites

- Node.js and npm installed (for running the JavaScript helper)
- `oyl` CLI tool and `oyl-sdk` available
- `jq` command-line JSON processor installed
- `curl` installed for direct RPC interactions
- Access to a running Subfrost environment (usually at http://localhost:18888)
- WebAssembly builds of the three SLOP contracts

## Deployment Files

Two key files facilitate the deployment process:

1. **slop-bond-helper.js**: JavaScript helper for bond operations (minting, redeeming, purchasing)
2. **deploy-slop-subfrost.sh**: Main deployment script that handles all steps of the process

## Deployment Steps

### 1. Ensure WebAssembly Binaries are Built

If you haven't already built the WebAssembly binaries, run:

```bash
./scripts/wasm-build.sh
```

This will build all three SLOP contract binaries in the `target/wasm32-unknown-unknown/release` directory.

### 2. Run the Deployment Script

```bash
./deploy-slop-subfrost.sh
```

This script will:
- Check for required dependencies
- Verify blockchain synchronization
- Transfer WASM files to the Subfrost directory
- Set up accounts using the default provider mnemonic
- Deploy all three contracts
- Run validation tests
- Generate a deployment report

### 3. Verify Deployment

After the script completes, check the generated deployment report:

```bash
cat slop_deployment_report_subfrost_*.md
```

This report contains:
- Transaction IDs for all deployed contracts
- Configuration parameters used during deployment
- Results of validation tests
- Usage instructions for interacting with the contracts

## Troubleshooting

### RPC Methods Not Found

If you receive errors like:
```
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}
```

This indicates that the Subfrost environment doesn't support the method being used. In this case:

1. Check the Subfrost documentation for supported RPC methods
2. Use the direct RPC methods in the script:
   ```bash
   # Example of using a direct RPC method
   ./deploy-contract-direct.sh "$CURVE_CONTRACT_PATH" "BondCurve" "3,$CURVE_CONSTANT,1,0"
   ```

### Docker Container Issues

If there are issues with the Docker containers:

1. Check container status: `docker ps`
2. Restart containers if needed: `docker restart subfrost-subrail`
3. Check container logs: `docker logs subfrost-subrail`

## Using the Bond Helper Directly

You can use the bond helper script directly for manual operations:

```bash
# Mint a bond
node slop-bond-helper.js --action mint --target-txid <collection_id> --amount 1000000 --recipient <address> --provider alkanes

# Redeem a bond
node slop-bond-helper.js --action redeem --target-txid <collection_id> --bond-id <bond_id> --provider alkanes

# Purchase a bond from curve
node slop-bond-helper.js --action purchase --target-txid <bond_curve_id> --amount 1000000 --provider alkanes
```

## Additional Information

- The script logs all operations to a timestamped log file (`deploy-slop-subfrost-*.log`)
- Contract constants used: Factory (100001), Collection (100002), Curve (100003)
- Default interest rate: 5.00% (500 basis points)
- Default maturity period: 100 blocks
