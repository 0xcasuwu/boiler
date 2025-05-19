# OYL SDK Alkane ID Extension - Technical Specification

## 1. Introduction

This technical specification outlines the extension to the OYL SDK that allows passing an alkane ID directly within the execute call, rather than requiring it to be part of the calldata.

## 2. Background

### 2.1 Alkane Execution in OYL SDK

The OYL SDK provides functionality for executing operations on alkane contracts through the `alkaneExecute` command. This command requires:

- Calldata: The operation code and parameters to be called on the contract
- Optional edicts: Edicts for the protostone
- Provider: The network provider type (regtest, bitcoin)
- Optional mnemonic: Used for signing transactions
- Optional fee rate: The fee rate for the transaction

### 2.2 Alkane ID Usage

An alkane ID is composed of two parts:
- Block number: The block in which the alkane was created
- Transaction index: The index of the transaction within that block

Together, these form an alkane ID in the format `block:tx` (e.g., `123456:789`).

### 2.3 Previous Implementation

In the previous implementation, the alkane ID was extracted from the first two parameters of the calldata:

```javascript
const alkaneBlock = options.calldata[0];
const alkaneTx = options.calldata[1];
```

This approach had several limitations:
1. It was not explicit in the API that the first two parameters were being used as the alkane ID
2. It consumed two parameters from the calldata that could otherwise be used for the operation
3. It made the API less intuitive for users

## 3. Extension Requirements

### 3.1 Functional Requirements

1. Add a new option to the `alkaneExecute` command to specify the alkane ID directly
2. Maintain backward compatibility with the existing approach
3. Ensure the new option takes precedence over the legacy approach when both are provided
4. Provide clear logging to indicate which approach is being used

### 3.2 Non-Functional Requirements

1. Minimal changes to the existing codebase
2. No impact on performance
3. Clear documentation of the new functionality

## 4. Implementation Details

### 4.1 Command Option Addition

A new option has been added to the `alkaneExecute` command:

```javascript
.option('-id, --alkane-id <alkaneId>', 'Alkane ID in format "block:tx"')
```

### 4.2 Alkane ID Extraction Logic

The alkane ID extraction logic has been modified to check for the new option first, falling back to the legacy approach if it's not provided:

```javascript
// Extract alkane ID from options or calldata
let alkaneBlock, alkaneTx;

if (options.alkaneId) {
    // Parse from the new alkane-id option
    const [block, tx] = options.alkaneId.split(':');
    alkaneBlock = block;
    alkaneTx = tx;
    console.log(`Processing alkane ID from option: ${alkaneBlock}:${alkaneTx}`);
} else {
    // Fall back to extracting from calldata (legacy behavior)
    alkaneBlock = options.calldata[0];
    alkaneTx = options.calldata[1];
    console.log(`Processing alkane ID from calldata: ${alkaneBlock}:${alkaneTx}`);
}
```

### 4.3 UTXO Selection

The UTXO selection logic remains unchanged, as it already works with the alkane ID regardless of where it came from:

```javascript
// Find UTXOs that have the rune we want
const runeUtxos = allUtxos.filter(utxo => {
    if (!utxo.runes) return false;
    return Object.keys(utxo.runes).includes(`${alkaneBlock}:${alkaneTx}`);
});

// If we couldn't find UTXOs with the rune, try with selectAlkanesUtxos
const alkanesUtxos = await utxo.selectAlkanesUtxos({
    utxos: accounts.taproot.alkaneUtxos,
    alkaneId: { block: `${alkaneBlock}`, tx: `${alkaneTx}` },
    targetNumberOfAlkanes: 1,
});
```

## 5. Usage Examples

### 5.1 Using the New Option

```bash
# Execute a contract operation with alkane ID specified directly
oyl alkane execute -id 123456:789 -data 1,2,3 -p regtest

# Execute with edicts
oyl alkane execute -id 123456:789 -data 1,2,3 -e 123456:789:1000:0 -p regtest
```

### 5.2 Using the Legacy Approach (still supported)

```bash
# Execute a contract operation with alkane ID in calldata
oyl alkane execute -data 123456,789,1,2,3 -p regtest

# Execute with edicts
oyl alkane execute -data 123456,789,1,2,3 -e 123456:789:1000:0 -p regtest
```

## 6. Future Improvements

### 6.1 Validation

Add validation for the alkane ID format to ensure it's properly formatted as `block:tx`:

```javascript
if (options.alkaneId) {
    const [block, tx] = options.alkaneId.split(':');
    
    // Validate format
    if (!block || !tx || isNaN(Number(block)) || isNaN(Number(tx))) {
        throw new Error('Invalid alkane ID format. Expected format: block:tx (e.g., 123456:789)');
    }
    
    alkaneBlock = block;
    alkaneTx = tx;
    console.log(`Processing alkane ID from option: ${alkaneBlock}:${alkaneTx}`);
}
```

### 6.2 Consistency Across Commands

For consistency, similar options could be added to other alkane-related commands that require an alkane ID:

- `alkaneSwap`
- `alkaneSend`
- `alkaneRemoveLiquidity`
- etc.

### 6.3 Deprecation Strategy

Consider deprecating the legacy approach in future versions to encourage use of the more explicit API:

1. Phase 1: Add deprecation warning when using the legacy approach
   ```javascript
   if (!options.alkaneId) {
       console.warn('WARNING: Extracting alkane ID from calldata is deprecated. Please use the --alkane-id option instead.');
       alkaneBlock = options.calldata[0];
       alkaneTx = options.calldata[1];
   }
   ```

2. Phase 2: Make the `--alkane-id` option required in a future major version

### 6.4 Enhanced Error Handling

Improve error handling for cases where the alkane ID is not found:

```javascript
if (alkanesUtxos.utxos.length === 0) {
    console.error(`No Alkane Utxos Found for ID ${alkaneBlock}:${alkaneTx}`);
    throw new errors_1.OylTransactionError(Error(`Insufficient balance: No Alkane Utxos Found for the specified alkane ID (${alkaneBlock}:${alkaneTx})`));
}
```

### 6.5 Documentation Updates

Update all relevant documentation to reflect the new option, including:

- README files
- API documentation
- Example scripts
- Command help text

## 7. Testing Strategy

### 7.1 Unit Tests

Add unit tests to verify:
- Parsing of the alkane ID from the new option
- Fallback to legacy behavior when the option is not provided
- Error handling for invalid alkane ID formats

### 7.2 Integration Tests

Add integration tests to verify:
- End-to-end execution with the new option
- Backward compatibility with the legacy approach
- Proper UTXO selection based on the alkane ID

## 8. Conclusion

This extension provides a more explicit and intuitive way to specify the alkane ID when executing operations on alkane contracts. It maintains backward compatibility while improving the developer experience and setting the stage for future improvements to the OYL SDK.
