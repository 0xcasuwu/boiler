# Subrail RPC Server Extension

To enable successful deployment of SLOP contracts to the Subfrost environment, we need to extend the subrail implementation with the missing RPC methods. This document outlines the necessary modifications to the `Dockerfile.subrail` file.

## Extending the RPC Server Implementation

Below is a modified version of the RPC server implementation with the required methods added. The key changes include:

1. Adding support for essential contract deployment methods
2. Implementing mock storage for deployed contracts
3. Creating compatibility aliases for existing methods

```go
// Additional global state for contract functionality
var (
    currentHeight int = 840000
    contextAddress string = "0x0000000000000000000000000000000000000000"
    psbtStore = make(map[int][]string)
    // New state for contract deployment
    deployedContracts = make(map[string]string)  // txid -> contract binary
    contractCalldata = make(map[string]string)   // txid -> calldata
    nextTxid = 1000  // Starting point for transaction IDs
)

// Generate a mock transaction ID for contract deployments
func generateTxid() string {
    txid := fmt.Sprintf("%064x", nextTxid)
    nextTxid++
    return txid
}

// Enhanced RPC request handling with additional methods
switch request.Method {
case "subrail_height":
    response.Result = currentHeight

case "metashrew_height":  // Alias for subrail_height for compatibility
    response.Result = currentHeight

case "btc_getblockcount":  // Bitcoin node block count (same as our height)
    response.Result = currentHeight

case "alkane_newContract":
    // Parse params: [contractHex, calldata, feeRate]
    var params []interface{}
    err = json.Unmarshal(request.Params, &params)
    if err != nil || len(params) < 2 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params"}
        break
    }
    
    // Extract contract hex and calldata
    contractHex, ok1 := params[0].(string)
    calldata, ok2 := params[1].(string)
    if !ok1 || !ok2 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params types"}
        break
    }
    
    // Generate transaction ID and store contract
    txid := generateTxid()
    deployedContracts[txid] = contractHex
    contractCalldata[txid] = calldata
    
    // Return transaction ID as result
    response.Result = txid

case "alkane_callContract":
    // Parse params: [txid, opcode, args]
    var params []interface{}
    err = json.Unmarshal(request.Params, &params)
    if err != nil || len(params) < 3 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params"}
        break
    }
    
    // Extract txid, opcode, and args
    txid, ok1 := params[0].(string)
    opcode, ok2 := params[1].(float64)
    args, ok3 := params[2].([]interface{})
    if !ok1 || !ok2 || !ok3 {
        response.Error = &JsonRpcError{Code: -32602, Message: "Invalid params types"}
        break
    }
    
    // Check if contract exists
    _, exists := deployedContracts[txid]
    if !exists {
        response.Error = &JsonRpcError{Code: -32000, Message: "Contract not found"}
        break
    }
    
    // Mock successful call and return a new txid
    callTxid := generateTxid()
    response.Result = callTxid

case "subrail_getBundle":
    // Existing implementation...

case "subrail_getBundleState":
    // Existing implementation...

case "subrail_setContext":
    // Existing implementation...

default:
    response.Error = &JsonRpcError{Code: -32601, Message: "Method not found"}
}
```

## Implementation Steps

To implement these changes:

1. Create a fork of the Subfrost repository
2. Modify the `Dockerfile.subrail` to include the enhanced RPC method implementations
3. Add the necessary global state variables and helper functions
4. Rebuild the Docker containers

## Testing the Modified Environment

After implementing these changes, we can test the environment with:

```bash
# Test metashrew_height method
curl -X POST http://localhost:18888 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"metashrew_height","params":[],"id":1}'

# Test deploying a contract
curl -X POST http://localhost:18888 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"alkane_newContract","params":["contract_hex_here","some_calldata",5],"id":1}'
```

## Integration with Deployment Scripts

Once these RPC methods are implemented, our existing deployment scripts should work without modification, as they rely on these standard method names.

This approach provides a minimal but functional extension to the Subfrost environment that enables contract deployment while maintaining compatibility with our existing tools and scripts.
