# RPC Method Limitations Analysis

## Critical Issue: Missing Return Value Method

We've identified a critical limitation in the current RPC server implementation that prevents retrieving actual return values from contract method calls. This significantly impacts contract testing and interaction.

## Detailed Analysis

When calling `alkane_getCallResult` to retrieve the return value from a contract call, we consistently receive:
```json
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}
```

The error code -32601 indicates that this method is not implemented in the current server. This means:

1. We can successfully make contract calls using `alkane_callContract`
2. We can verify the transaction was processed using `alkane_getTransactionReceipt`
3. BUT we cannot retrieve the actual returned value from the contract method

## Impact

This limitation has several implications:

1. **Limited Testing Capability**: We cannot verify that contract methods return the expected values
2. **Reduced Contract Interaction**: Interactive applications cannot read state or computed values
3. **Debugging Difficulty**: Makes it harder to diagnose issues with contract logic

## Available Methods

We have confirmed the following RPC methods are working:
- `metashrew_height` - Returns current blockchain height
- `alkane_callContract` - Makes a call to a contract method
- `alkane_getTransactionReceipt` - Gets the receipt of a transaction

## Missing Methods

The following critical methods are not implemented:
- `alkane_getCallResult` - Gets the return value from a contract call
- Several UTXO-related methods required by the `oyl` CLI tool

## Solutions Within Subfrost Ecosystem

We should focus exclusively on solutions that work within the Subfrost ecosystem without attempting to modify the server infrastructure:

1. **Contract Event Pattern**:
   - Modify contracts to emit events containing the return values
   - Parse transaction receipts to extract these events
   - This pattern is well-established in blockchain development

2. **State Storage Pattern**:
   - Create contract methods that store important return values in contract state
   - Add getter methods to retrieve these stored values in separate calls
   - This allows for retrieval of computed values across multiple transactions

3. **Client-Side Computation**:
   - Implement algorithms to derive values on the client side
   - Use only the available RPC methods (callContract, getTransactionReceipt)
   - Move computation logic to client when feasible

## Next Steps

1. **Adapt Contract Design**:
   - Update our contracts to emit events containing return values
   - Implement state storage pattern for key computed values
   - Document event structure for client applications

2. **Enhance Client Tooling**:
   - Create receipt parsers that extract events from transaction receipts
   - Build client-side computation libraries to derive values
   - Update testing scripts to use these new patterns

3. **Documentation & Integration**:
   - Create documentation on working with Subfrost's available RPC methods
   - Update integration guides to reflect the event-based approach
   - Ensure all teams understand the Subfrost-specific workflows
