# OYL SDK Alkane ID Extension

## Overview

This document explains the functionality of the OYL SDK with respect to alkane execution and details the extension that has been implemented to allow passing an alkane ID directly within the execute call.

## Current Functionality

The OYL SDK provides a command-line interface for interacting with alkane contracts. The `alkaneExecute` command is used to execute operations on these contracts.

### Before the Extension

Previously, the alkane ID had to be included as part of the calldata, where:
- The first parameter in calldata was the block number
- The second parameter in calldata was the transaction index

For example:
```bash
oyl alkane execute -data 123456,789,1,2,3 -p regtest
```

In this example, `123456` is the block number and `789` is the transaction index, which together form the alkane ID `123456:789`. The remaining parameters (`1,2,3`) are the actual operation code and parameters to be called on the contract.

This approach had several limitations:
1. It was not explicit that the first two parameters were being used as the alkane ID
2. It consumed two parameters from the calldata that could otherwise be used for the operation
3. It made the API less intuitive for users

## The Extension

The extension adds a new option to the `alkaneExecute` command that allows specifying the alkane ID directly:

```bash
oyl alkane execute -id 123456:789 -data 1,2,3 -p regtest
```

### How It Works

1. A new option `-id, --alkane-id <alkaneId>` has been added to the `alkaneExecute` command
2. When this option is provided, the alkane ID is parsed from it in the format `block:tx`
3. If the option is not provided, the command falls back to the previous behavior of extracting the alkane ID from the first two parameters of calldata

### Implementation Details

The implementation modifies the `alkaneExecute` command in `minimal-contract/oyl-sdk/lib/cli/alkane.js`:

```javascript
exports.alkaneExecute = new AlkanesCommand('execute')
    // ... existing options ...
    .option('-id, --alkane-id <alkaneId>', 'Alkane ID in format "block:tx"')
    .action(async (options) => {
        // ... existing code ...

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

        // ... rest of the function ...
    });
```

## Benefits

1. **Explicit API**: The alkane ID is now explicitly specified in the command options
2. **Cleaner Calldata**: The calldata can now be used solely for the operation parameters
3. **Backward Compatibility**: Existing code will continue to work as before
4. **Improved Developer Experience**: Makes the API more intuitive and easier to use

## Usage Examples

### Using the New Option

```bash
# Execute a contract operation with alkane ID specified directly
oyl alkane execute -id 123456:789 -data 1,2,3 -p regtest

# Execute with edicts
oyl alkane execute -id 123456:789 -data 1,2,3 -e 123456:789:1000:0 -p regtest
```

### Using the Legacy Approach (still supported)

```bash
# Execute a contract operation with alkane ID in calldata
oyl alkane execute -data 123456,789,1,2,3 -p regtest

# Execute with edicts
oyl alkane execute -data 123456,789,1,2,3 -e 123456:789:1000:0 -p regtest
```

## Future Considerations

1. Consider adding similar options to other alkane-related commands for consistency
2. Consider deprecating the legacy approach in future versions to encourage use of the more explicit API
3. Add validation for the alkane ID format to ensure it's properly formatted as `block:tx`
