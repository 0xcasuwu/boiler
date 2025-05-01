use crate::YieldVault;
use std::sync::Arc;
use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_runtime::storage::StoragePointer;
use anyhow::Result;
use wasm_bindgen_test::wasm_bindgen_test;

// Reset all storage keys used in tests
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
    let vault = YieldVault::default();

    // First initialization should succeed
    assert!(vault.observe_initialization().is_ok());

    // Second initialization should fail
    assert!(vault.observe_initialization().is_err());
}

#[wasm_bindgen_test]
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
fn test_deposit_functionality() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Initial state should be zero
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    
    // Create a transaction hash and test deposit
    let tx_hash = "abc123".to_string();
    let caller = "alice".to_string();
    let receiver = "alice".to_string();
    let assets = 100u128;
    
    // Call deposit
    vault.deposit(tx_hash.clone(), caller.clone(), receiver.clone(), assets).unwrap();
    
    // Verify state changes
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), assets);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), assets); // 1:1 ratio for first deposit
    
    // Get user balance
    let balance_key = format!("/balances/{}", receiver);
    let balance_pointer = StoragePointer::from_keyword(&balance_key);
    assert_eq!(balance_pointer.get_value::<u128>(), assets);
    
    // Try to use the same tx_hash again (should fail)
    let result = vault.deposit(tx_hash, caller, receiver, assets);
    assert!(result.is_err());
}

#[wasm_bindgen_test]
fn test_mint_functionality() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Initial state should be zero
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    
    // Create a transaction hash and test mint
    let tx_hash = "abc123".to_string();
    let caller = "alice".to_string();
    let receiver = "alice".to_string();
    let shares = 100u128;
    
    // Call mint
    vault.mint(tx_hash, caller, receiver.clone(), shares).unwrap();
    
    // Verify state changes
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), shares); // 1:1 ratio for first mint
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), shares);
    
    // Get user balance
    let balance_key = format!("/balances/{}", receiver);
    let balance_pointer = StoragePointer::from_keyword(&balance_key);
    assert_eq!(balance_pointer.get_value::<u128>(), shares);
}

#[wasm_bindgen_test]
fn test_conversion_functions() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Set up initial state: 100 assets and 50 shares
    // This gives a 2:1 ratio of assets to shares
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(50u128);
    
    // Test asset to share conversion
    assert_eq!(vault.convert_assets_to_shares(10u128).unwrap(), 5u128); // 10 assets should be 5 shares
    assert_eq!(vault.convert_assets_to_shares(20u128).unwrap(), 10u128); // 20 assets should be 10 shares
    
    // Test share to asset conversion
    assert_eq!(vault.convert_shares_to_assets(5u128).unwrap(), 10u128); // 5 shares should be 10 assets
    assert_eq!(vault.convert_shares_to_assets(25u128).unwrap(), 50u128); // 25 shares should be 50 assets
    
    // Edge cases
    assert_eq!(vault.convert_assets_to_shares(0u128).unwrap(), 0u128);
    assert_eq!(vault.convert_shares_to_assets(0u128).unwrap(), 0u128);
    
    // Reset for 1:1 ratio test
    vault.total_assets_pointer().set_value(0u128);
    vault.total_supply_pointer().set_value(0u128);
    
    // With empty vault, ratio should be 1:1
    assert_eq!(vault.convert_assets_to_shares(10u128).unwrap(), 10u128);
    assert_eq!(vault.convert_shares_to_assets(10u128).unwrap(), 10u128);
}

#[wasm_bindgen_test]
fn test_preview_functions() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Set up initial state: 100 assets and 50 shares
    // This gives a 2:1 ratio of assets to shares
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(50u128);
    
    // Test preview deposit (should match convert_assets_to_shares)
    assert_eq!(vault.preview_deposit(10u128).unwrap(), 5u128);
    
    // Test preview mint (should round up)
    // For 10 shares: 10 * 100 / 50 = 20 assets
    assert_eq!(vault.preview_mint(10u128).unwrap(), 20u128);
    // For 5 shares: 5 * 100 / 50 = 10 assets
    assert_eq!(vault.preview_mint(5u128).unwrap(), 10u128);
    
    // Test preview withdraw (should round up)
    // For 11 assets: 11 * 50 / 100 = 5.5, rounded up to 6 shares
    assert_eq!(vault.preview_withdraw(11u128).unwrap(), 6u128);
    // For 10 assets: 10 * 50 / 100 = 5 shares exactly
    assert_eq!(vault.preview_withdraw(10u128).unwrap(), 5u128);
    
    // Test preview redeem (should match convert_shares_to_assets)
    assert_eq!(vault.preview_redeem(10u128).unwrap(), 20u128);
}

#[wasm_bindgen_test]
fn test_yield_accrual() {
    use std::thread::sleep;
    use std::time::Duration;
    
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Set initial state
    let initial_assets = 1000u128;
    vault.total_assets_pointer().set_value(initial_assets);
    vault.total_supply_pointer().set_value(1000u128); // 1:1 ratio initially
    
    // Set yield rate to 5% (500 basis points)
    let yield_rate = 500u128;
    vault.yield_rate_pointer().set_value(yield_rate);
    
    // Get current timestamp
    let initial_timestamp = crate::tests::mock::get_timestamp();
    vault.last_yield_update_pointer().set_value(initial_timestamp);
    
    // Mock time advancement (1 day = 86400 seconds)
    // This will be adjusted in the mock time function
    sleep(Duration::from_millis(100)); // Just a brief delay to ensure the mock time changes
    
    // Update yield
    vault.test_update_yield().unwrap();
    
    // Check that assets increased but supply remained the same
    let new_assets = vault.total_assets_pointer().get_value::<u128>();
    let new_supply = vault.total_supply_pointer().get_value::<u128>();
    
    assert!(new_assets > initial_assets, "Assets should increase due to yield");
    assert_eq!(new_supply, 1000u128, "Supply should remain unchanged");
    
    // Check that timestamp was updated
    let new_timestamp = vault.last_yield_update_pointer().get_value::<u64>();
    assert!(new_timestamp > initial_timestamp, "Timestamp should be updated");
}

#[wasm_bindgen_test]
fn test_transaction_validation() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // First use of a transaction hash should succeed
    let tx_hash = "tx123";
    assert!(vault.validate_and_track_transaction(tx_hash).is_ok());
    
    // Second use of the same hash should fail
    assert!(vault.validate_and_track_transaction(tx_hash).is_err());
    
    // Different hash should succeed
    let new_tx_hash = "tx456";
    assert!(vault.validate_and_track_transaction(new_tx_hash).is_ok());
}

#[wasm_bindgen_test]
fn test_balance_management() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Test set_balance and get_balance
    let account = "alice";
    let amount = 100u128;
    
    // Initial balance should be zero
    assert_eq!(vault.get_balance(account), 0u128);
    
    // Set balance
    vault.set_balance(account, amount);
    
    // Check balance is updated
    assert_eq!(vault.get_balance(account), amount);
    
    // Test mint_shares
    let additional = 50u128;
    vault.mint_shares(account, additional).unwrap();
    
    // Check updated balance and total supply
    assert_eq!(vault.get_balance(account), amount + additional);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), additional);
    
    // Test burn_shares
    vault.burn_shares(account, 25u128).unwrap();
    
    // Check updated balance and total supply
    assert_eq!(vault.get_balance(account), amount + additional - 25u128);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), additional - 25u128);
    
    // Try to burn more than balance (should fail)
    let result = vault.burn_shares(account, 1000u128);
    assert!(result.is_err());
}
