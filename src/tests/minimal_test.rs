use crate::YieldVault;
use std::sync::Arc;
use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_runtime::storage::StoragePointer;
use wasm_bindgen_test::wasm_bindgen_test;

// Reset all storage keys used in tests
#[test]
fn reset_test_storage() {
    // Clear all storage keys used in tests
    StoragePointer::from_keyword("/initialized").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/name").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/symbol").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/asset-name").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/asset-symbol").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/decimals").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/total-supply").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/total-assets").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/yield-rate").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/last-yield-update").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/tx-hashes").set(Arc::new(Vec::new()));
    
    // Clear example balances
    StoragePointer::from_keyword("/balances/alice").set(Arc::new(Vec::new()));
    StoragePointer::from_keyword("/balances/bob").set(Arc::new(Vec::new()));
    
    // Clear any custom data
    StoragePointer::from_keyword("/data").set(Arc::new(Vec::new()));
}

#[wasm_bindgen_test]
fn test_can_create_vault() {
    // Reset storage before the test
    reset_test_storage();
    
    // Just create a vault and make sure it doesn't crash
    let _vault = YieldVault::default();
    
    // If we get here, the test passes
    assert!(true);
}

#[wasm_bindgen_test]
fn test_storage_pointers() {
    // Reset storage before the test
    reset_test_storage();
    
    // Create a vault
    let vault = YieldVault::default();
    
    // Verify that the storage pointers exist and return empty values
    assert_eq!(vault.name_pointer().get().len(), 0);
    assert_eq!(vault.symbol_pointer().get().len(), 0);
    assert_eq!(vault.asset_name_pointer().get().len(), 0);
    assert_eq!(vault.asset_symbol_pointer().get().len(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.yield_rate_pointer().get_value::<u128>(), 0);
}

#[wasm_bindgen_test]
fn test_constant_values() {
    // Test the constants used in calculations
    assert_eq!(crate::BASIS_POINTS_DENOMINATOR, 10000);
    assert_eq!(crate::SECONDS_PER_YEAR, 365 * 24 * 60 * 60);
    assert_eq!(crate::YIELD_CALCULATION_DENOMINATOR, crate::BASIS_POINTS_DENOMINATOR * crate::SECONDS_PER_YEAR);
}

#[wasm_bindgen_test]
fn test_ceil_div() {
    // Test the ceil_div helper function
    assert_eq!(crate::ceil_div(10, 5).unwrap(), 2);
    assert_eq!(crate::ceil_div(11, 5).unwrap(), 3);  // Ceiling division
    assert_eq!(crate::ceil_div(0, 5).unwrap(), 0);
    assert!(crate::ceil_div(10, 0).is_err());  // Division by zero
}

#[wasm_bindgen_test]
fn test_initialization_guard() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();

    // First initialization should succeed
    assert!(vault.observe_initialization().is_ok());

    // Second initialization should fail
    assert!(vault.observe_initialization().is_err());
}

#[wasm_bindgen_test]
#[test]
fn test_initialization() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize with test values
    let name = "Test Vault".to_string();
    let symbol = "vTEST".to_string();
    let asset_name = "Test Asset".to_string();
    let asset_symbol = "TEST".to_string();
    let decimal_offset = 18u8;
    
    // Call initialize directly for testing
    vault.initialize(name.clone(), symbol.clone(), asset_name.clone(), asset_symbol.clone(), decimal_offset).unwrap();
    
    // Verify the values were set correctly
    let stored_name = String::from_utf8(vault.name_pointer().get().as_ref().to_vec()).unwrap();
    let stored_symbol = String::from_utf8(vault.symbol_pointer().get().as_ref().to_vec()).unwrap();
    let stored_asset_name = String::from_utf8(vault.asset_name_pointer().get().as_ref().to_vec()).unwrap();
    let stored_asset_symbol = String::from_utf8(vault.asset_symbol_pointer().get().as_ref().to_vec()).unwrap();
    
    assert_eq!(stored_name, name);
    assert_eq!(stored_symbol, symbol);
    assert_eq!(stored_asset_name, asset_name);
    assert_eq!(stored_asset_symbol, asset_symbol);
    assert_eq!(vault.decimals_pointer().get_value::<u8>(), decimal_offset);
    
    // Check initialization flag
    assert_eq!(vault.initialized_pointer().get_value::<u8>(), 1);
}

#[wasm_bindgen_test]
#[test]
fn test_deposit_functionality() {
    use crate::tests::mock;
    
    // Reset storage
    reset_test_storage();
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Initial state should be zero
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    
    // Set up yield rate and timestamp
    vault.yield_rate_pointer().set_value(500u128);  // 5% yield rate
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    
    // Advance mock timestamp for next operation
    mock::set_timestamp(1100);
    
    // Create test data for deposit
    let assets = 100u128;
    
    // Initialize transaction hash storage for replay protection
    vault.tx_hashes_pointer().set(Arc::new("{}".as_bytes().to_vec()));
    
    // Use the public deposit method to test deposit functionality
    #[cfg(test)]
    {
        let tx_hash = "abc123".to_string();
        let caller = "alice".to_string();
        let receiver = "alice".to_string();
        
        let result = vault.deposit(tx_hash, caller, receiver.clone(), assets);
        assert!(result.is_ok());
        
        // Check balances after deposit
        let balance_key = format!("/balances/{}", receiver);
        let balance = StoragePointer::from_keyword(&balance_key).get_value::<u128>();
        assert_eq!(balance, assets);
        
        // Check total supply and total assets
        assert_eq!(vault.total_supply_pointer().get_value::<u128>(), assets);
        assert_eq!(vault.total_assets_pointer().get_value::<u128>(), assets);
    }
}

#[wasm_bindgen_test]
#[test]
fn test_mint_functionality() {
    use crate::tests::mock;

    // Reset storage
    reset_test_storage();
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Initial state should be zero
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    
    // Set up yield rate and timestamp
    vault.yield_rate_pointer().set_value(500u128);  // 5% yield rate
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    
    // Advance mock timestamp for next operation
    mock::set_timestamp(1100);
    
    // Create test data for mint
    let shares = 100u128;
    
    // Initialize transaction hash storage for replay protection
    vault.tx_hashes_pointer().set(Arc::new("{}".as_bytes().to_vec()));
    
    // Use the public mint method to test mint functionality
    #[cfg(test)]
    {
        let tx_hash = "abc123".to_string();
        let caller = "alice".to_string();
        let receiver = "alice".to_string();
        
        let result = vault.mint(tx_hash, caller, receiver.clone(), shares);
        assert!(result.is_ok());
        
        // Check balances after mint
        let balance_key = format!("/balances/{}", receiver);
        let balance = StoragePointer::from_keyword(&balance_key).get_value::<u128>();
        assert_eq!(balance, shares);
        
        // Check total supply and total assets
        assert_eq!(vault.total_supply_pointer().get_value::<u128>(), shares);
        assert_eq!(vault.total_assets_pointer().get_value::<u128>(), shares); // 1:1 ratio for first mint
    }
}

#[wasm_bindgen_test]
#[test]
fn test_conversion_functions() {
    use crate::tests::mock;
    
    // Reset storage
    reset_test_storage();
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    vault.yield_rate_pointer().set_value(500u128);  // 5% yield rate
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Set up initial state: 100 assets and 50 shares
    // This gives a 2:1 ratio of assets to shares
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(50u128);
    
    // Test asset to share conversion
    let shares_from_10_assets = vault.convert_assets_to_shares(10u128).unwrap();
    assert_eq!(shares_from_10_assets, 5u128); // 10 assets should be 5 shares
    
    let shares_from_20_assets = vault.convert_assets_to_shares(20u128).unwrap();
    assert_eq!(shares_from_20_assets, 10u128); // 20 assets should be 10 shares
    
    // Test share to asset conversion
    let assets_from_5_shares = vault.convert_shares_to_assets(5u128).unwrap();
    assert_eq!(assets_from_5_shares, 10u128); // 5 shares should be 10 assets
    
    let assets_from_25_shares = vault.convert_shares_to_assets(25u128).unwrap();
    assert_eq!(assets_from_25_shares, 50u128); // 25 shares should be 50 assets
    
    // Edge cases
    assert_eq!(vault.convert_assets_to_shares(0u128).unwrap(), 0u128);
    assert_eq!(vault.convert_shares_to_assets(0u128).unwrap(), 0u128);
    
    // Test with different ratio (set to 1:1)
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(100u128);
    
    // With 1:1 ratio
    assert_eq!(vault.convert_assets_to_shares(10u128).unwrap(), 10u128);
    assert_eq!(vault.convert_shares_to_assets(10u128).unwrap(), 10u128);
}

#[wasm_bindgen_test]
#[test]
fn test_preview_functions() {
    use crate::tests::mock;
    
    // Reset storage
    reset_test_storage();
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    vault.yield_rate_pointer().set_value(500u128);  // 5% yield rate
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Set up initial state: 100 assets and 50 shares
    // This gives a 2:1 ratio of assets to shares
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(50u128);
    
    // Test preview deposit (should match convert_assets_to_shares)
    let shares_from_deposit = vault.preview_deposit(10u128).unwrap();
    assert_eq!(shares_from_deposit, 5u128);
    
    // Test preview mint (should round up)
    // For 10 shares: 10 * 100 / 50 = 20 assets
    let assets_for_mint_10 = vault.preview_mint(10u128).unwrap();
    assert_eq!(assets_for_mint_10, 20u128);
    
    // For 5 shares: 5 * 100 / 50 = 10 assets
    let assets_for_mint_5 = vault.preview_mint(5u128).unwrap();
    assert_eq!(assets_for_mint_5, 10u128);
    
    // Test preview withdraw (should round up)
    // For 11 assets: 11 * 50 / 100 = 5.5, rounded up to 6 shares
    let shares_for_withdraw_11 = vault.preview_withdraw(11u128).unwrap();
    assert_eq!(shares_for_withdraw_11, 6u128);
    
    // For 10 assets: 10 * 50 / 100 = 5 shares exactly
    let shares_for_withdraw_10 = vault.preview_withdraw(10u128).unwrap();
    assert_eq!(shares_for_withdraw_10, 5u128);
    
    // Test preview redeem (should match convert_shares_to_assets)
    let assets_for_redeem = vault.preview_redeem(10u128).unwrap();
    assert_eq!(assets_for_redeem, 20u128);
    
    // Test edge cases
    assert_eq!(vault.preview_deposit(0u128).unwrap(), 0u128);
    assert_eq!(vault.preview_redeem(0u128).unwrap(), 0u128);
}

#[wasm_bindgen_test]
#[test]
fn test_yield_accrual() {
    use crate::tests::mock;
    
    // Reset storage
    reset_test_storage();
    
    // Initialize mock timestamp with a specific value
    mock::reset_timestamp();
    let start_time = 1000000u64;
    mock::set_timestamp(start_time);

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);

    // Set initial state
    let initial_assets = 1_000_000_000u128;  // Use a larger number to see larger changes
    vault.total_assets_pointer().set_value(initial_assets);
    vault.total_supply_pointer().set_value(1_000_000_000u128); // 1:1 ratio initially
    
    // Set yield rate to 10% (1000 basis points) - higher rate for more noticeable changes
    let yield_rate = 1000u128;
    vault.yield_rate_pointer().set_value(yield_rate);
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Record the initial timestamp explicitly
    let initial_timestamp = start_time;
    
    // Advance mock time by 30 days for more noticeable yield
    let time_advance = 30 * 86400;
    mock::set_timestamp(initial_timestamp + time_advance);
    
    // Update yield with the advanced time
    vault.test_update_yield().unwrap();
    
    // Check that assets increased but supply remained the same
    let new_assets = vault.total_assets_pointer().get_value::<u128>();
    let new_supply = vault.total_supply_pointer().get_value::<u128>();
    
    // With a 10% annual rate over 30 days, we should see approximately (10% * 30/365) increase
    // which is about 0.82% increase
    println!("Initial assets: {}, New assets: {}, Increase: {}", 
             initial_assets, new_assets, new_assets - initial_assets);
             
    assert!(new_assets > initial_assets, "Assets should increase due to yield");
    
    // Calculate expected yield (approximate)
    let expected_increase = (initial_assets as f64 * yield_rate as f64 * time_advance as f64) 
                             / (10000f64 * 365f64 * 86400f64);
    let min_expected = initial_assets + expected_increase as u128 / 2;  // Allow some tolerance
    
    println!("Expected minimum increase: {}", min_expected - initial_assets);
    assert!(new_assets >= min_expected, 
            "Yield increase too small: got {} but expected at least {}", 
            new_assets - initial_assets, min_expected - initial_assets);
             
    // Supply should remain unchanged
    assert_eq!(new_supply, 1_000_000_000u128, "Supply should remain unchanged");
    
    // Check that timestamp was updated
    let new_timestamp = vault.last_yield_update_pointer().get_value::<u64>();
    assert_eq!(new_timestamp, initial_timestamp + time_advance, "Timestamp should be updated");
    
    // Test multiple yield updates
    // Advance time again by another 30 days
    mock::set_timestamp(new_timestamp + time_advance);
    
    // Update yield again
    vault.test_update_yield().unwrap();
    
    // Assets should increase further
    let final_assets = vault.total_assets_pointer().get_value::<u128>();
    println!("After second update - assets: {}, increase: {}", 
             final_assets, final_assets - new_assets);
    
    assert!(final_assets > new_assets, "Assets should increase after second yield update");
    
    // Supply still unchanged
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 1_000_000_000u128);
}

#[wasm_bindgen_test]
#[test]
fn test_transaction_validation() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Initialize tx hash storage with empty JSON object
    vault.tx_hashes_pointer().set(Arc::new("{}".as_bytes().to_vec()));
    
    // First use of a transaction hash should succeed
    let tx_hash = "tx123";
    let result1 = vault.validate_and_track_transaction(tx_hash);
    assert!(result1.is_ok(), "First use of transaction hash should succeed");
    
    // Second use of the same hash should fail
    let result2 = vault.validate_and_track_transaction(tx_hash);
    assert!(result2.is_err(), "Second use of same transaction hash should fail");
    
    // Different hash should succeed
    let new_tx_hash = "tx456";
    let result3 = vault.validate_and_track_transaction(new_tx_hash);
    assert!(result3.is_ok(), "Different transaction hash should succeed");
    
    // Check that both hashes are now in storage
    let tx_hashes_json = String::from_utf8(vault.tx_hashes_pointer().get().as_ref().to_vec()).unwrap();
    assert!(tx_hashes_json.contains(tx_hash), "First hash should be stored");
    assert!(tx_hashes_json.contains(new_tx_hash), "Second hash should be stored");
}

#[wasm_bindgen_test]
#[test]
fn test_balance_management() {
    use crate::tests::mock;
    
    // Reset storage
    reset_test_storage();
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);
    
    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata (for a complete test environment)
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    vault.yield_rate_pointer().set_value(0u128);  // No yield for this test
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Test accounts
    let alice = "alice";
    let bob = "bob";
    
    // Test initial balances
    assert_eq!(vault.get_balance(alice), 0u128, "Initial alice balance should be zero");
    assert_eq!(vault.get_balance(bob), 0u128, "Initial bob balance should be zero");
    
    // Test setting balance directly
    let alice_amount = 100u128;
    vault.set_balance(alice, alice_amount);
    assert_eq!(vault.get_balance(alice), alice_amount, "Alice balance should be set to 100");
    
    // Test mint_shares function
    let bob_amount = 50u128;
    let mint_result = vault.mint_shares(bob, bob_amount);
    assert!(mint_result.is_ok(), "Minting shares to Bob should succeed");
    assert_eq!(vault.get_balance(bob), bob_amount, "Bob balance should be 50 after mint");
    
    // Check total supply after mints
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), bob_amount, 
               "Total supply should equal Bob's balance (mint_shares updates total supply)");
    
    // Test multiple accounts
    assert_eq!(vault.get_balance(alice), alice_amount, "Alice balance should remain unchanged");
    assert_eq!(vault.get_balance(bob), bob_amount, "Bob balance should remain unchanged");
    
    // Test burn_shares function
    let burn_amount = 20u128;
    let burn_result = vault.burn_shares(bob, burn_amount);
    assert!(burn_result.is_ok(), "Burning shares from Bob should succeed");
    assert_eq!(vault.get_balance(bob), bob_amount - burn_amount, 
               "Bob balance should decrease by burn amount");
    
    // Check total supply after burn
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), bob_amount - burn_amount,
               "Total supply should decrease by burn amount");
    
    // Test invalid operations
    
    // Try to burn more than balance
    let excessive_burn_result = vault.burn_shares(bob, 1000u128);
    assert!(excessive_burn_result.is_err(), "Burning more than balance should fail");
    
    // Ensure balances remain unchanged after failed operation
    assert_eq!(vault.get_balance(bob), bob_amount - burn_amount,
               "Bob balance should remain unchanged after failed burn");
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), bob_amount - burn_amount,
               "Total supply should remain unchanged after failed burn");
}
