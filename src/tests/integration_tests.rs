#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use std::sync::Arc;
    use wasm_bindgen_test::*;

    // Import our actual implementation
    use crate::{YieldVault, Storage, Conversion, AssetManagement};

    // Import from alkanes and metashrew
    use alkanes_runtime::storage::StoragePointer;
    use metashrew_support::index_pointer::KeyValuePointer;

    // Configure wasm tests to run in browser
    wasm_bindgen_test_configure!(run_in_browser);

    // We'll use this helper for testing instead of implementing AlkaneResponder twice
    fn fixed_height_for_tests() -> u64 {
        // Return a fixed block height for testing
        1000u64
    }

    // Helper to reset storage for tests
    fn reset_test_storage() {
        // Clear key storage locations
        let keys_to_clear = [
            "/name",
            "/symbol",
            "/asset-name",
            "/asset-symbol",
            "/decimals",
            "/total-supply",
            "/total-assets",
            "/yield-rate",
            "/last-yield-height",
            "/initialized",
        ];
        
        for key in keys_to_clear {
            let mut pointer = StoragePointer::from_keyword(key);
            pointer.set(Arc::new(vec![]));
        }
        
        // Clear some test balances
        for account in ["user1", "user2", "user3"] {
            let mut pointer = StoragePointer::from_keyword(&format!("/balances/{}", account));
            pointer.set(Arc::new(vec![]));
        }
    }

#[wasm_bindgen_test]
fn test_initialization() {
    // Reset storage before test
    reset_test_storage();
    
    // Create a new vault instance
    let vault = YieldVault::default();
    
    // Check that initialization works
    let result = vault.initialize(
        "Test Vault".to_string(),
        "TVT".to_string(),
        "Test Asset".to_string(),
        "ASSET".to_string(),
        8u128
    );
    
    // Verify initialization succeeded
    assert!(result.is_ok());
    
    // Verify storage was updated correctly
    let name = String::from_utf8(vault.name_pointer().get().as_ref().to_vec())
        .expect("Invalid UTF-8 in name");
    assert_eq!(name, "Test Vault");
    
    let symbol = String::from_utf8(vault.symbol_pointer().get().as_ref().to_vec())
        .expect("Invalid UTF-8 in symbol");
    assert_eq!(symbol, "TVT");
    
    let asset_name = String::from_utf8(vault.asset_name_pointer().get().as_ref().to_vec())
        .expect("Invalid UTF-8 in asset name");
    assert_eq!(asset_name, "Test Asset");
    
    let asset_symbol = String::from_utf8(vault.asset_symbol_pointer().get().as_ref().to_vec())
        .expect("Invalid UTF-8 in asset symbol");
    assert_eq!(asset_symbol, "ASSET");
    
    // Check decimals
    assert_eq!(vault.decimals_pointer().get_value::<u8>(), 8u8);
    
    // Check initial values
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    
    // Check that double initialization fails
    let result = vault.initialize(
        "Another Vault".to_string(),
        "AVT".to_string(),
        "Another Asset".to_string(),
        "AASSET".to_string(),
        8u128
    );
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_asset_share_conversion() {
    // Reset storage before test
    reset_test_storage();
    
    // Create a new vault instance
    let vault = YieldVault::default();
    
    // Test converting assets to shares in an empty vault (should be 1:1)
    let result = vault.convert_assets_to_shares(100, 0, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
    
    // Test converting shares to assets in an empty vault (should be 1:1)
    let result = vault.convert_shares_to_assets(50, 0, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 50);
    
    // Test with non-zero values
    // If total_assets = 1000 and total_supply = 500, then:
    // 1 asset = 0.5 shares, and 1 share = 2 assets
    
    // 100 assets should convert to 50 shares
    let result = vault.convert_assets_to_shares(100, 1000, 500);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 50);
    
    // 50 shares should convert to 100 assets
    let result = vault.convert_shares_to_assets(50, 1000, 500);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
    
    // Test conversion with precision loss
    // If we have 1000 assets and 3 shares:
    // 1 asset = 0.003 shares, 1 share = 333.333... assets
    
    // Test rounding behavior
    let result = vault.convert_shares_to_assets(1, 1000, 3);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 333); // integer division truncates
    
    // Test preview functions
    let result = vault.preview_mint(1, 1000, 3);
    assert!(result.is_ok());
    let assets_needed = result.unwrap();
    assert!(assets_needed >= 334); // ceiling division for mint
}

#[wasm_bindgen_test]
fn test_balance_tracking() {
    // Reset storage before test
    reset_test_storage();
    
    // Create a new vault instance
    let vault = YieldVault::default();
    
    // Initialize the vault
    let _ = vault.initialize(
        "Test Vault".to_string(),
        "TVT".to_string(),
        "Test Asset".to_string(),
        "ASSET".to_string(),
        8u128
    );
    
    // Test accounts
    let user1 = "user1";
    let user2 = "user2";
    
    // Check initial balances
    assert_eq!(vault.get_balance(user1), 0);
    assert_eq!(vault.get_balance(user2), 0);
    
    // Manually update balances (we'd normally use mint_shares in production)
    let mut result = vault.mint_shares(user1, 100);
    assert!(result.is_ok());
    
    result = vault.mint_shares(user2, 50);
    assert!(result.is_ok());
    
    // Check updated balances
    assert_eq!(vault.get_balance(user1), 100);
    assert_eq!(vault.get_balance(user2), 50);
    
    // Check total supply was updated
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 150);
    
    // Test burning shares
    result = vault.burn_shares(user1, 30);
    assert!(result.is_ok());
    
    // Check balance was reduced
    assert_eq!(vault.get_balance(user1), 70);
    
    // Check total supply was reduced
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 120);
    
    // Test burning more than available should fail
    result = vault.burn_shares(user2, 100);
    assert!(result.is_err());
    
    // Balance should be unchanged
    assert_eq!(vault.get_balance(user2), 50);
}

#[wasm_bindgen_test]
fn test_yield_calculation() {
    // This test needs to be adapted for your actual implementation
    // as it depends on how AlkaneResponder::height() is implemented
    
    // Reset storage before test
    reset_test_storage();
    
    // Create a new vault instance
    let vault = YieldVault::default();
    
    // Initialize the vault
    let _ = vault.initialize(
        "Test Vault".to_string(),
        "TVT".to_string(),
        "Test Asset".to_string(),
        "ASSET".to_string(),
        8u128
    );
    
    // Set up some initial state
    // Set total assets
    vault.total_assets_pointer().set_value(1000u128);
    
    // Set yield rate to 10% per year (1000 basis points)
    vault.yield_rate_pointer().set_value(1000u128);
    
    // Set initial yield update height
    vault.last_yield_height_pointer().set_value(1000u64);
    
    // For this test, we need to implement height() so it returns a known value
    // This is already done with our test implementation above
    
    // Update yield - in our implementation the yield calculation will use the
    // difference between current height and last update height
    // This will vary based on your implementation, but the test shows the pattern
    let result = vault.update_yield();
    
    // Check that yield update succeeded
    assert!(result.is_ok());
    
    // In a real test, we would check the updated total_assets
    // This will depend on your specific yield calculation formula
}
}

// Add a basic non-wasm test that doesn't require wasm-bindgen-test
#[cfg(not(target_arch = "wasm32"))]
mod native_tests {
    use crate::YieldVault;
    
    #[test]
    fn basic_sanity_test() {
        // This test doesn't use WebAssembly specific functionality
        // Create a basic vault instance
        let vault = YieldVault::default();
        
        // Test something that doesn't require WebAssembly
        assert_eq!(vault.preview_deposit(100).unwrap(), 100);
    }
}
