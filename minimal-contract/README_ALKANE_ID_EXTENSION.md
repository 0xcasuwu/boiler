# OYL SDK Alkane ID Extension

## Overview

This extension to the OYL SDK allows passing an alkane ID directly within the execute call using a new `-id, --alkane-id` option, rather than requiring it to be part of the calldata.

## Problem Solved

Previously, when executing operations on alkane contracts, the alkane ID had to be included as the first two parameters in the calldata. This approach had several limitations:

1. It was not explicit in the API that the first two parameters were being used as the alkane ID
2. It consumed two parameters from the calldata that could otherwise be used for the operation
3. It made the API less intuitive for users

## Solution

The extension adds a new option to the `alkaneExecute` command that allows specifying the alkane ID directly:

```bash
oyl alkane execute -id 123456:789 -data 1,2,3 -p regtest
```

This makes the API more explicit and intuitive, while maintaining backward compatibility with the existing approach.

## Documentation

For more detailed information about the extension, please refer to the following documents:

- [Alkane ID Extension Documentation](./ALKANE_ID_EXTENSION.md) - Explains the functionality and usage of the extension
- [Alkane ID Extension Technical Specification](./ALKANE_ID_EXTENSION_SPEC.md) - Provides a detailed technical specification of the extension
- [Alkane Execute Examples](./examples/alkane-execute-examples.js) - Demonstrates how to use the extension with examples

## Usage

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

## Benefits

1. **Explicit API**: The alkane ID is now explicitly specified in the command options
2. **Cleaner Calldata**: The calldata can now be used solely for the operation parameters
3. **Backward Compatibility**: Existing code will continue to work as before
4. **Improved Developer Experience**: Makes the API more intuitive and easier to use

## Implementation

The implementation modifies the `alkaneExecute` command in `minimal-contract/oyl-sdk/lib/cli/alkane.js` to add the new option and update the alkane ID extraction logic.

## Future Considerations

1. Add similar options to other alkane-related commands for consistency
2. Consider deprecating the legacy approach in future versions to encourage use of the more explicit API
3. Add validation for the alkane ID format to ensure it's properly formatted as `block:tx`
