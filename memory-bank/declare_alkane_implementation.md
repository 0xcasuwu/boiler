# Implementation of the declare_alkane Macro

## Overview

We've successfully implemented a simplified version of the `declare_alkane` macro from the free-mint repository. This macro provides a clean way to connect message opcodes to handler methods in the YieldVault contract.

## Implementation Details

### 1. Custom Macro Implementation

We created a simplified version of the `declare_alkane` macro in `src/macros.rs`:

```rust
#[macro_export]
macro_rules! declare_alkane_simple {
    (impl AlkaneResponder for $type:ty {
        type Message = $message:ty;
    }) => {
        impl $type {
            pub fn dispatch_message(&self, opcode: u32, args: &[u8]) -> Result<CallResponse, anyhow::Error> {
                match opcode {
                    // Opcode handlers for various operations
                    // ...
                }
            }
        }
    };
}
```

This macro generates a `dispatch_message` method for the implementing type that routes opcodes to the appropriate handler methods.

### 2. Custom Attributes

We implemented a custom attribute system in `src/attributes.rs` to replace the `#[opcode(n)]` attributes from the MessageDispatch derive macro:

```rust
pub struct OpcodeAttribute {
    pub code: u32,
}

pub trait GetOpcode {
    fn get_opcode(&self) -> u32;
}

impl GetOpcode for YieldVaultMessage {
    fn get_opcode(&self) -> u32 {
        match self {
            Self::Initialize { .. } => 0,
            Self::Deposit { .. } => 10,
            // ...
        }
    }
}
```

### 3. Message Enum

We simplified the YieldVaultMessage enum by removing the MessageDispatch derive and using our custom attribute system instead:

```rust
pub enum YieldVaultMessage {
    /// Initialize the vault with its base parameters
    #[doc(hidden)]
    Initialize {
        name: String,
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimal_offset: u128,
    },
    // ...
}
```

### 4. Integration with YieldVault

We integrated our macro with the YieldVault struct in `src/lib.rs`:

```rust
mod yield_vault_dispatch {
    use super::*;
    
    declare_alkane_simple! {
        impl AlkaneResponder for YieldVault {
            type Message = YieldVaultMessage;
        }
    }
}
```

And updated the `call` function to use our new dispatch method:

```rust
#[wasm_bindgen]
pub fn call(opcode: u32, args: &[u8]) -> Vec<u8> {
    let vault = YieldVault::default();
    
    match vault.dispatch_message(opcode, args) {
        Ok(response) => response.data.to_vec(),
        Err(e) => format!("Error: {}", e).as_bytes().to_vec(),
    }
}
```

## Testing

We created a simple test in `src/bin/simple_test.rs` that demonstrates the functionality of our implementation:

```rust
fn main() {
    println!("Testing simple dispatch mechanism");
    
    // Create a test instance
    let test = TestStruct::default();
    
    // Test the dispatch
    let args = b"World";
    match test.dispatch_message(0, args) {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => println!("Error: {}", e),
    }
}
```

The test successfully runs and produces the expected output:

```
Testing simple dispatch mechanism
Response: CallResponse { alkanes: AlkaneTransferParcel([]), data: [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33] }
```

## Advantages of Our Implementation

1. **Simplicity**: Our implementation is simpler and more straightforward than the original MessageDispatch derive macro.
2. **Flexibility**: It's easier to customize and extend for specific project needs.
3. **Maintainability**: The code is more explicit and easier to understand.
4. **Compatibility**: It works with the existing codebase without requiring changes to the handler methods.

## Future Improvements

1. **Error Handling**: Add more robust error handling for invalid opcodes and arguments.
2. **Type Safety**: Improve type safety for message parameters.
3. **Documentation**: Add more comprehensive documentation for the macro and its usage.
4. **Testing**: Add more comprehensive tests for edge cases.
