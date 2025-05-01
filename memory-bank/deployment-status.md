# SLOP Deployment Status Report

## Summary

We have successfully built WebAssembly binaries for all three SLOP contracts, but encountered issues when attempting to deploy to the local Docker-based testnet running on port 18888.

## Accomplished Tasks

1. **Successfully fixed code issues and built WebAssembly binaries**:
   - slop_bond_curve.wasm (37KB)
   - slop_launchpad_factory.wasm (38KB)
   - slop_orbital_bond_collection.wasm (37KB)

2. **Created deployment scripts**:
   - Modified the original `deploy-slop-docker.sh` script to create `deploy-local.sh`
   - Removed dependency on compilation steps, focusing only on deployment
   - Added better error handling and file verification

3. **Verified Docker infrastructure**:
   - Confirmed that all three required Docker containers are running:
     - subfrost-integration
     - subfrost-subrail (exposing port 18888)
     - subfrost-bitcoind

## Deployment Issues

When attempting to deploy the contracts to the local testnet, we encountered the following issues:

1. **RPC Method Not Found Error**:
   ```
   JSON-RPC Error: { code: -32601, message: 'Method not found' }
   ```
   
   This suggests that the Docker container running on port 18888 doesn't support the RPC methods expected by the `oyl` CLI tool.

2. **Connectivity Issues**:
   While the Docker instance responded to HTTP requests (HTTP 405), the RPC methods required for deployment seem to be unavailable or not properly initialized.

## Next Steps

To resolve the deployment issues, we recommend the following actions:

1. **Docker Container Verification**:
   - Ensure the subfrost-subrail container is properly initialized and healthy
   - Check Docker logs for any startup errors: `docker logs subfrost-subrail`
   - Verify that the container supports the expected RPC methods

2. **RPC Method Compatibility Check**:
   - Determine which RPC methods are available in the subfrost-subrail service
   - Compare with those required by the `oyl` CLI tool
   - Consider checking API documentation for the subfrost-subrail service

3. **Alternative Deployment Methods**:
   - If the current Docker setup doesn't support the required RPC methods, consider:
     - Using direct HTTP requests to deploy contracts if an API is available
     - Creating a custom deployment script that works with the available API
     - Setting up a different local testnet environment that's compatible with the `oyl` CLI

4. **Proceed with Test Refactoring**:
   - While resolving deployment issues, continue with the test refactoring plan documented in `memory-bank/test-refactoring-plan.md`
   - This will ensure the codebase is fully tested and ready for deployment once the infrastructure issues are resolved

## Technical Details

### Docker Container Status

```
CONTAINER ID   IMAGE                  COMMAND                  CREATED         STATUS                             PORTS                                                                                   NAMES
1a7208cde20e   subfrost-integration   "docker-entrypoint.s…"   8 minutes ago   Up 1 second                                                                                                                subfrost-integration
b7d847dfc769   subfrost-subrail       "/subrail --port 188…"   8 minutes ago   Up 11 seconds (health: starting)   0.0.0.0:18888->18888/tcp                                                                subfrost-subrail
56d2d773cb85   subfrost-bitcoind      "/entrypoint.sh bitc…"   8 minutes ago   Up 3 seconds (health: starting)    8332-8333/tcp, 18332-18333/tcp, 38332-38333/tcp, 0.0.0.0:18443-18444->18443-18444/tcp   subfrost-bitcoind
```

Note that both `subfrost-subrail` and `subfrost-bitcoind` are showing "(health: starting)", which may indicate they are still initializing.

### Container Logs Analysis

**subfrost-subrail logs:**
```
2025/04/30 14:51:22 Starting server on 0.0.0.0:18888...
2025/04/30 14:59:51 Starting server on 0.0.0.0:18888...
```
- The subrail server appears to have started twice, most recently at 14:59:51
- No error messages are visible in the filtered logs
- The server is listening on port 18888 as expected

**subfrost-bitcoind logs:**
```
2025-04-30T15:03:36Z [net] Stall started peer=10
[...]
2025-04-30T15:04:07Z [net] Stall started peer=10
```
- The bitcoind service is showing multiple "Stall started" messages for peer connections
- These messages could indicate networking or synchronization issues
- No specific RPC errors are reported in the filtered logs

### Network Configuration

The Docker containers are connected to the same network:
```
"Containers": {
            "1a7208cde20e25b3827fb9fb665dcfec3ece96e7ca62dae7766eba26a07aba5a": {
                "Name": "subfrost-integration",
                "EndpointID": "7e29f3d18c2d5d3572644c2177d81c55a352a62aa9295d07df2dd07b4bd72fac",
                "MacAddress": "02:42:ac:13:00:04",
                "IPv4Address": "172.19.0.4/16",
                "IPv6Address": ""
            },
            "56d2d773cb857b9538b2806517c7dbd251cb5c8f929597eb121df5177080e683": {
                "Name": "subfrost-bitcoind",
                [...]
```
- All three containers are on the same Docker network
- Network connectivity between containers appears to be correctly configured

### RPC API Testing Results

We've conducted additional API tests to understand the RPC compatibility issues:

**Standard JSON-RPC help request:**
```
$ curl -s -X POST -d '{"jsonrpc":"2.0","method":"help","params":[],"id":1}' http://localhost:18888
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}
```

**Specific metashrew_height method:**
```
$ curl -s -X POST -d '{"jsonrpc":"2.0","method":"metashrew_height","params":[],"id":1}' http://localhost:18888
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}
```

**Basic HTTP GET request:**
```
$ curl -s http://localhost:18888
Method not allowed
```

These results confirm that:
1. The server is running and responding to HTTP requests
2. It's a JSON-RPC server (responds with proper JSON-RPC error format)
3. The methods required by the `oyl` CLI tool are not implemented or exposed
4. The server only accepts POST requests, not GET (which is standard for JSON-RPC)

**Original RPC Error from deployment script:**
```
JSON-RPC Error: { code: -32601, message: 'Method not found' }
Request Error: Error: [object Object]
    at SandshrewBitcoinClient._call (/Users/erickdelgado/Documents/GitHub/oyl-sdk/lib/rpclient/sandshrew.js:32:23)
    at process.processTicksAndRejections (node:internal/process/task_queues:105:5)
    at async SandshrewBitcoinClient.multiCall (/Users/erickdelgado/Documents/GitHub/oyl-sdk/lib/rpclient/sandshrew.js:53:16)
    at async addressUtxos (/Users/erickdelgado/Documents/GitHub/oyl-sdk/lib/utxo/utxo.js:52:23)
    at async Object.accountUtxos (/Users/erickdelgado/Documents/GitHub/oyl-sdk/lib/utxo/utxo.js:184:130)
    at async Command.<anonymous> (/Users/erickdelgado/Documents/GitHub/oyl-sdk/lib/cli/alkane.js:62:74)
```

## Current Deployment Status (Updated 4/30/2025, 11:40 AM)

### Deployment Preparation Complete
- Successfully built WebAssembly binaries for all contracts
- Created deployment scripts (deploy-slop-subfrost.sh) and helper utilities (slop-bond-helper.js)
- Transferred binaries to the Subfrost directory
- All code appears ready for deployment

### Deployment Blocking Issues

The Subfrost environment appears to have significant compatibility issues:

1. **Docker Container Health**:
   - Containers are running but marked as "unhealthy"
   - Both subfrost-bitcoind and subfrost-subrail are in an unhealthy state

2. **No RPC Method Support**:
   - Multiple tests confirm that the expected RPC methods are not available
   - Methods like `metashrew_height`, `btc_getblockcount`, and `rpc_methods` all return "Method not found" errors
   - The `oyl` CLI tool fails with similar errors when attempting to deploy contracts

3. **Limited Log Information**:
   - Server logs show it's receiving requests but provide no insight into method support
   - No error logs that would help diagnose the specific issue

### Next Steps for Deployment

1. **Container Health Investigation**:
   - Restart the Docker containers to see if they can reach a healthy state
   - Check for any configuration issues in the Docker setup

2. **RPC Method Discovery**:
   - Attempt common Bitcoin RPC methods to identify what might be supported
   - Try alternate RPC authentication schemes if applicable

3. **Consider Alternate Deployment Approaches**:
   - Investigate whether the Subfrost project has specific deployment documentation
   - Consider if direct file transfer or other methods might be available instead of RPC deployment

4. **Proceed with Testing Improvements**:
   - While deployment issues are being addressed, continue with the planned test refactoring
   - This ensures the code will be fully tested once deployment becomes possible

In conclusion, the deployment is currently blocked by infrastructure compatibility issues between our deployment tools and the Subfrost environment. Further research into the Subfrost API and deployment requirements is needed.
