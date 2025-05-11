#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use wasm_bindgen_test::*;
    use std::sync::Arc;

    // Configure wasm tests to run in browser
    wasm_bindgen_test_configure!(run_in_browser);

    // Import dependencies for direct testing
    use alkanes_runtime::storage::StoragePointer;
    use metashrew_support::index_pointer::KeyValuePointer;

    #[wasm_bindgen_test]
    fn test_key_value_pointer_implementation() {
        // Test that StoragePointer correctly implements KeyValuePointer trait
        
        // Create a test pointer
        let test_key = "/test/key_value_test";
        let mut pointer = StoragePointer::from_keyword(test_key);
        
        // Test setting a value
        let test_data = Arc::new(vec![1, 2, 3, 4, 5]);
        pointer.set(test_data.clone());
        
        // Test getting a value
        let retrieved_data = pointer.get();
        assert_eq!(retrieved_data.as_ref(), &[1, 2, 3, 4, 5]);
        
        // Test appending data using the KeyValuePointer trait method
        let append_data = Arc::new(vec![6, 7, 8]);
        pointer.append(append_data);
        
        // Verify the data was appended
        let final_data = pointer.get();
        assert_eq!(final_data.as_ref(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        
        // Clear the test data for future test runs
        pointer.set(Arc::new(vec![]));
    }

    #[wasm_bindgen_test]
    fn test_storage_pointer_with_typed_values() {
        // Test typed value storage and retrieval
        
        // Create a test pointer
        let test_key = "/test/typed_values";
        let mut pointer = StoragePointer::from_keyword(test_key);
        
        // Test u8 value
        pointer.set_value(42u8);
        let value_u8 = pointer.get_value::<u8>();
        assert_eq!(value_u8, 42u8);
        
        // Test u64 value
        pointer.set_value(1234567890u64);
        let value_u64 = pointer.get_value::<u64>();
        assert_eq!(value_u64, 1234567890u64);
        
        // Test u128 value
        pointer.set_value(123456789012345678901234567890u128);
        let value_u128 = pointer.get_value::<u128>();
        assert_eq!(value_u128, 123456789012345678901234567890u128);
        
        // Clear the test data
        pointer.set(Arc::new(vec![]));
    }

    #[wasm_bindgen_test]
    fn test_storage_pointer_select() {
        // Test the select operation for hierarchical keys
        let base_key = "/test/select";
        let base_pointer = StoragePointer::from_keyword(base_key);
        
        // Create keys for different accounts
        let account1 = "account1".as_bytes().to_vec();
        let account2 = "account2".as_bytes().to_vec();
        
        // Store values for different accounts
        let mut pointer1 = base_pointer.select(&account1);
        pointer1.set_value(100u128);
        
        let mut pointer2 = base_pointer.select(&account2);
        pointer2.set_value(200u128);
        
        // Verify we can retrieve the distinct values
        let value1 = base_pointer.select(&account1).get_value::<u128>();
        assert_eq!(value1, 100u128);
        
        let value2 = base_pointer.select(&account2).get_value::<u128>();
        assert_eq!(value2, 200u128);
        
        // Modify one value
        let mut pointer1 = base_pointer.select(&account1);
        pointer1.set_value(150u128);
        
        // Check the updated value
        let updated_value1 = base_pointer.select(&account1).get_value::<u128>();
        assert_eq!(updated_value1, 150u128);
        
        // Ensure the other value is unchanged
        let unchanged_value2 = base_pointer.select(&account2).get_value::<u128>();
        assert_eq!(unchanged_value2, 200u128);
        
        // Clean up
        let mut pointer1 = base_pointer.select(&account1);
        pointer1.set(Arc::new(vec![]));
        
        let mut pointer2 = base_pointer.select(&account2);
        pointer2.set(Arc::new(vec![]));
    }

    #[wasm_bindgen_test]
    fn test_opcode_dispatch() {
        use crate::YieldVault;
        use crate::constants::opcodes;
        
        // Create a new vault instance
        let vault = YieldVault::default();
        
        // Clear initialized flag from previous tests
        let mut init_pointer = StoragePointer::from_keyword("/initialized");
        init_pointer.set(Arc::new(vec![]));
        
        // Call the dispatch method with initialization opcode
        let args = [
            "Test Vault".as_bytes(), &[0], // Name
            "TVT".as_bytes(), &[0],         // Symbol
            "Test Asset".as_bytes(), &[0],  // Asset Name
            "ASSET".as_bytes()              // Asset Symbol
        ].concat();
        
        let result = vault.dispatch(opcodes::INITIALIZE, &args);
        
        // Verify initialization succeeded
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response, "Initialized");
        
        // Get name using opcode dispatch
        let result = vault.dispatch(opcodes::GET_NAME, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Test Vault");
        
        // Get symbol using opcode dispatch
        let result = vault.dispatch(opcodes::GET_SYMBOL, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "TVT");
        
        // Get decimals using opcode dispatch
        let result = vault.dispatch(opcodes::GET_DECIMALS, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "8");  // Default value set in initialize
        
        // Test SET_DATA/GET_DATA opcodes
        let args = [
            "test_key".as_bytes(), &[0],
            "test_value".as_bytes()
        ].concat();
        
        let result = vault.dispatch(opcodes::SET_DATA, &args);
        assert!(result.is_ok());
        
        let result = vault.dispatch(opcodes::GET_DATA, "test_key".as_bytes());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_value");
    }
}

// Add a basic non-wasm test for StoragePointer functionality
#[cfg(not(target_arch = "wasm32"))]
mod native_tests {
    use alkanes_runtime::storage::StoragePointer;
    use std::sync::Arc;
    
    #[test]
    fn test_storage_pointer_basics() {
        // Basic test that doesn't require WebAssembly
        let key = "/test/native";
        let mut pointer = StoragePointer::keyword(key);
        
        // Store and retrieve a simple value
        let data = Arc::new(vec![1, 2, 3]);
        pointer.set(data.clone());
        
        let retrieved = pointer.get();
        assert_eq!(retrieved.as_ref(), &[1, 2, 3]);
    }
}
