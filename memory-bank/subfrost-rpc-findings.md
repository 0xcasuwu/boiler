# Subfrost RPC Environment Analysis

## Summary of Findings

After thorough investigation of the Subfrost environment, we've identified why our deployment attempts are failing. The Subfrost environment **does not support the RPC methods required for contract deployment**.

## Implemented RPC Methods

The subrail server in Subfrost only implements these specific RPC methods:

1. `subrail_height` - Returns the current blockchain height (confirmed working)
2. `subrail_getBundle` - Gets a bundle at a specific height
3. `subrail_getBundleState` - Gets the state of a bundle at a specific height (confirmed working)
4. `subrail_setContext` - Sets the context address

## Missing RPC Methods

Notably absent are all the methods required for contract deployment and interaction:

1. `alkane_newContract` - Required for deploying new contracts
2. `metashrew_height` - Used by oyl CLI for block synchronization
3. `btc_getblockcount` - Used to check Bitcoin node status

## Server Implementation Analysis

Examining the Dockerfile.subrail reveals that the RPC server is a mock implementation with deliberately limited functionality:

```go
// Process the request based on method
var response JsonRpcResponse
response.Jsonrpc = "2.0"
response.Id = request.Id

switch request.Method {
case "subrail_height":
    response.Result = currentHeight

case "subrail_getBundle":
    // Implementation...

case "subrail_getBundleState":
    // Implementation...

case "subrail_setContext":
    // Implementation...

default:
    response.Error = &JsonRpcError{Code: -32601, Message: "Method not found"}
}
```

The server explicitly returns "Method not found" for any method not in its limited supported set.

## Container Health Status

The Docker containers are running but remain in an "unhealthy" state:

```
CONTAINER ID   IMAGE                  STATUS                                 
1a7208cde20e   subfrost-integration   Up About a minute                      
b7d847dfc769   subfrost-subrail       Up About a minute (health: starting)   
56d2d773cb85   subfrost-bitcoind      Up About a minute (health: starting)   
```

The healthcheck for the subrail container is configured to check `subrail_height`, which is working, but the containers remain in "starting" health status.

## Deployment Options

Given these findings, our deployment options are:

1. **Modify the Subfrost Environment**:
   - Extend the subrail implementation to add the missing RPC methods
   - This requires modifying the Dockerfile.subrail and rebuilding the container

2. **Create a Direct Deployment Method**:
   - Develop a custom deployment process that doesn't rely on the missing RPC methods
   - Potentially use the existing bundle-related methods to store contract data

3. **Use a Different Deployment Environment**:
   - Consider deploying to an environment that fully supports the required RPC methods
   - This might mean using a different blockchain test environment

## Recommendation

We recommend proceeding with option 1: **Modify the Subfrost Environment** to implement the necessary RPC methods.

Specifically, we should:

1. Fork the subfrost repository
2. Add the missing RPC methods to the subrail service
3. Rebuild the Docker containers
4. Deploy our contracts using the enhanced environment

This approach ensures we maintain compatibility with the existing deployment scripts while adapting to the Subfrost environment's limitations.
