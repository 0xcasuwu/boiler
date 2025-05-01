use std::sync::Arc;

use crate::YieldVault;
use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::storage::StoragePointer;
use anyhow::Result;
use metashrew_support::index_pointer::KeyValuePointer;
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
fn test_initialization() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();

    // Verify initial state (empty/zero values)
    assert_eq!(vault.name_pointer().get().len(), 0);
    assert_eq!(vault.symbol_pointer().get().len(), 0);
    assert_eq!(vault.asset_name_pointer().get().len(), 0);
    assert_eq!(vault.asset_symbol_pointer().get().len(), 0);
    assert_eq!(vault.decimals_pointer().get_value::<u8>(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.yield_rate_pointer().get_value::<u128>(), 0);
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
fn test_deposit_and_mint_functionality() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Initial state should be zero
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    
    // Test deposit
    let tx_hash1 = "tx1".to_string();
    let caller = "alice".to_string();
    let receiver = "alice".to_string();
    let deposit_amount = 100u128;
    
    vault.deposit(tx_hash1, caller.clone(), receiver.clone(), deposit_amount).unwrap();
    
    // Verify deposit state changes
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), deposit_amount);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), deposit_amount); // 1:1 ratio for first deposit
    
    let balance_key = format!("/balances/{}", receiver);
    let alice_balance = StoragePointer::from_keyword(&balance_key).get_value::<u128>();
    assert_eq!(alice_balance, deposit_amount);
    
    // Test mint with a new transaction hash
    let tx_hash2 = "tx2".to_string();
    let mint_amount = 50u128;
    
    vault.mint(tx_hash2, caller, receiver.clone(), mint_amount).unwrap();
    
    // Calculate expected changes
    // Assets should increase based on the current ratio (preview_mint calculation)
    let total_assets = vault.total_assets_pointer().get_value::<u128>();
    let expected_new_assets = deposit_amount + mint_amount; // Initial assets are 1:1, so mint adds same amount
    
    // Verify mint state changes
    assert_eq!(total_assets, expected_new_assets);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), deposit_amount + mint_amount);
    
    // Check updated balance
    let updated_balance = StoragePointer::from_keyword(&balance_key).get_value::<u128>();
    assert_eq!(updated_balance, deposit_amount + mint_amount);
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
    
    // Mock time advancement by waiting briefly
    // This will trigger an increment in the mock timestamp function
    sleep(Duration::from_millis(100));
    
    // Update yield
    vault.test_update_yield().unwrap();
    
    // Assets should increase due to yield while supply remains constant
    let new_assets = vault.total_assets_pointer().get_value::<u128>();
    let new_supply = vault.total_supply_pointer().get_value::<u128>();
    
    assert!(new_assets > initial_assets, "Assets should increase due to yield");
    assert_eq!(new_supply, 1000u128, "Supply should remain unchanged");
    
    // Timestamp should be updated
    let new_timestamp = vault.last_yield_update_pointer().get_value::<u64>();
    assert!(new_timestamp > initial_timestamp, "Timestamp should be updated");
}

#[wasm_bindgen_test]
fn test_conversion_and_preview_functions() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let mut vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Set initial state with 2:1 ratio (assets:shares)
    vault.total_assets_pointer().set_value(200u128);
    vault.total_supply_pointer().set_value(100u128);
    
    // Test asset to share conversion
    assert_eq!(vault.convert_assets_to_shares(20u128).unwrap(), 10u128); // 20 assets = 10 shares
    
    // Test share to asset conversion
    assert_eq!(vault.convert_shares_to_assets(10u128).unwrap(), 20u128); // 10 shares = 20 assets
    
    // Test preview deposit (should match convert_assets_to_shares)
    assert_eq!(vault.preview_deposit(20u128).unwrap(), 10u128);
    
    // Test preview mint (should round up)
    // For 10 shares: 10 * 200 / 100 = 20 assets
    assert_eq!(vault.preview_mint(10u128).unwrap(), 20u128);
    
    // For non-even division, should round up
    // For 15 shares: 15 * 200 / 100 = 30 assets
    assert_eq!(vault.preview_mint(15u128).unwrap(), 30u128);
    
    // Test preview withdraw (should round up for shares needed)
    assert_eq!(vault.preview_withdraw(20u128).unwrap(), 10u128);
    // For 21 assets: 21 * 100 / 200 = 10.5, rounded up to 11 shares
    assert_eq!(vault.preview_withdraw(21u128).unwrap(), 11u128);
    
    // Test preview redeem (should match convert_shares_to_assets)
    assert_eq!(vault.preview_redeem(10u128).unwrap(), 20u128);
}

#[wasm_bindgen_test]
fn test_transaction_replay_protection() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // First use of a transaction hash should succeed
    let tx_hash = "tx123";
    assert!(vault.validate_and_track_transaction(tx_hash).is_ok());
    
    // Second use of the same hash should fail (replay protection)
    assert!(vault.validate_and_track_transaction(tx_hash).is_err());
    
    // Different hash should succeed
    let new_tx_hash = "tx456";
    assert!(vault.validate_and_track_transaction(new_tx_hash).is_ok());
}

#[wasm_bindgen_test]
fn test_vault_metadata() {
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
    
    // Initialize vault
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
}

#[wasm_bindgen_test]
fn test_account_balance_management() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Test balance management functions
    let alice = "alice";
    let bob = "bob";
    
    // Initial balances should be zero
    assert_eq!(vault.get_balance(alice), 0u128);
    assert_eq!(vault.get_balance(bob), 0u128);
    
    // Set balances
    vault.set_balance(alice, 100u128);
    vault.set_balance(bob, 50u128);
    
    // Verify balances
    assert_eq!(vault.get_balance(alice), 100u128);
    assert_eq!(vault.get_balance(bob), 50u128);
    
    // Add shares using mint_shares
    vault.mint_shares(alice, 50u128).unwrap();
    vault.mint_shares(bob, 25u128).unwrap();
    
    // Verify updated balances
    assert_eq!(vault.get_balance(alice), 150u128);
    assert_eq!(vault.get_balance(bob), 75u128);
    
    // Total supply should be the sum of all shares minted
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 75u128);
    
    // Burn shares
    vault.burn_shares(alice, 30u128).unwrap();
    vault.burn_shares(bob, 15u128).unwrap();
    
    // Verify updated balances after burning
    assert_eq!(vault.get_balance(alice), 120u128);
    assert_eq!(vault.get_balance(bob), 60u128);
    
    // Updated total supply
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 30u128);
    
    // Try to burn more than available (should fail)
    assert!(vault.burn_shares(alice, 1000u128).is_err());
}
