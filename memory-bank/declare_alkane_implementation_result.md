# declare_alkane Macro Implementation

## Overview

The `declare_alkane` macro has been successfully implemented in the YieldVault project. This macro is a key component of the ALKANES SDK, enabling WebAssembly contracts to interact with the Bitcoin blockchain through the ALKANES runtime.

## Implementation Details

### Key Components

1. **MessageDispatch Trait**: This trait is implemented for the `YieldVaultMessage` enum, providing methods to:
   - Convert opcodes and parameters to message variants (`from_opcode`)
   - Dispatch messages to the appropriate handler methods (`dispatch`)
   - Export the contract's ABI (`export_abi`)

2. **declare_alkane Macro**: This macro generates the `dispatch_message` method for the `YieldVault` struct, which:
   - Converts binary input data to a vector of u128 values
   - Creates a message from the opcode and inputs
   - Dispatches the message to the appropriate handler

3. **__execute Function**: This function is the entry point for the WebAssembly contract, which:
   - Takes an opcode and binary arguments
   - Uses the `dispatch_message` method to process the request
   - Returns the response as a raw pointer

### Parameter Handling

The macro handles different parameter types:
- **String**: Extracted from the input until a null terminator is found
- **u128**: Extracted as a single u128 value
- **AlkaneId**: Extracted as two u128 values (block and tx)
- **Vec<T>**: Extracted as a length followed by elements of type T

### Deployment

When deploying the contract, the calldata must include all required parameters for the Initialize opcode:
```
--calldata "0,Test Vault,TEST,Test Asset,ASSET,8"
```

Where:
- `0` is the opcode for Initialize
- `Test Vault` is the name
- `TEST` is the symbol
- `Test Asset` is the asset name
- `ASSET` is the asset symbol
- `8` is the decimal offset

## Testing

The implementation has been tested by:
1. Building the WebAssembly contract
2. Verifying that the contract can be deployed with the correct parameters
3. Ensuring that the `__execute` function correctly handles the input parameters

## Challenges and Solutions

1. **Parameter Format**: The initial deployment failed because the calldata was missing required parameters. This was fixed by updating the deployment script to include all necessary parameters.

2. **Type Conversion**: The `__execute` function needed to convert between Vec<u8> and *mut u8. This was addressed by implementing a helper function `vec_to_raw_ptr`.

3. **Module Organization**: The message handling code was initially split across multiple files. This was simplified by merging the message.rs file into lib.rs.

## Conclusion

The `declare_alkane` macro is now working correctly, allowing the YieldVault contract to be deployed and executed on the Bitcoin blockchain through the ALKANES runtime. The implementation follows the ERC-4626 standard while adapting it to the Bitcoin environment.
