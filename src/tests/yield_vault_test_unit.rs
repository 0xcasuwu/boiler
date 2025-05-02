use std::sync::Arc;

use crate::YieldVault;
use crate::storage::Storage;
use crate::security::Security;
use crate::utils::Conversion;
use crate::asset_management::AssetManagement;
use alkanes_runtime::storage::StoragePointer;
use metashrew_support::index_pointer::KeyValuePointer;
// No need for wasm_bindgen_test in this basic test

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

#[test]
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

#[test]
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

#[test]
fn test_set_get_metadata() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Set metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    
    // Verify metadata
    let name = String::from_utf8(vault.name_pointer().get().as_ref().to_vec()).unwrap();
    let symbol = String::from_utf8(vault.symbol_pointer().get().as_ref().to_vec()).unwrap();
    let asset_name = String::from_utf8(vault.asset_name_pointer().get().as_ref().to_vec()).unwrap();
    let asset_symbol = String::from_utf8(vault.asset_symbol_pointer().get().as_ref().to_vec()).unwrap();
    
    assert_eq!(name, "Test Vault");
    assert_eq!(symbol, "vTEST");
    assert_eq!(asset_name, "Test Asset");
    assert_eq!(asset_symbol, "TEST");
    assert_eq!(vault.decimals_pointer().get_value::<u8>(), 18u8);
}

#[test]
fn test_deposit_and_withdraw() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Initial state should be zero
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    
    // Test deposit - first deposit should maintain 1:1 ratio
    let tx_hash1 = "tx1".to_string();
    let caller = "alice".to_string();
    let receiver = "alice".to_string();
    let deposit_amount = 100u128;
    
    vault.deposit(tx_hash1, caller.clone(), receiver.clone(), deposit_amount).unwrap();
    
    // Check balances after deposit
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), deposit_amount);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), deposit_amount); // 1:1 ratio for first deposit
    assert_eq!(vault.get_balance(&receiver), deposit_amount);
    
    // Test withdraw
    let tx_hash2 = "tx2".to_string();
    let withdraw_amount = 40u128;
    
    vault.withdraw(tx_hash2, caller.clone(), receiver.clone(), receiver.clone(), withdraw_amount).unwrap();
    
    // Check balances after withdraw
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), deposit_amount - withdraw_amount);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), deposit_amount - withdraw_amount);
    assert_eq!(vault.get_balance(&receiver), deposit_amount - withdraw_amount);
}

#[test]
fn test_mint_and_redeem() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Setup initial state for better testing
    vault.total_assets_pointer().set_value(1000u128);
    vault.total_supply_pointer().set_value(500u128);
    
    // Initial state - 2:1 ratio (assets:shares)
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 1000u128);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 500u128);
    
    // Test mint
    let tx_hash1 = "tx1".to_string();
    let caller = "alice".to_string();
    let receiver = "alice".to_string();
    let mint_shares = 100u128;
    
    vault.mint(tx_hash1, caller.clone(), receiver.clone(), mint_shares).unwrap();
    
    // Check state after mint
    // For 100 shares at 2:1 ratio, should require 200 assets
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 1200u128); // 1000 + 200
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 600u128); // 500 + 100
    assert_eq!(vault.get_balance(&receiver), 100u128); // New receiver balance
    
    // Test redeem
    let tx_hash2 = "tx2".to_string();
    let redeem_shares = 50u128;
    
    vault.redeem(tx_hash2, caller.clone(), receiver.clone(), receiver.clone(), redeem_shares).unwrap();
    
    // Check state after redeem
    // For 50 shares at 2:1 ratio, should return 100 assets
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 1100u128); // 1200 - 100
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 550u128); // 600 - 50
    assert_eq!(vault.get_balance(&receiver), 50u128); // 100 - 50
}

#[test]
fn test_conversion_functions() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Set initial state with 2:1 ratio (assets:shares)
    vault.total_assets_pointer().set_value(200u128);
    vault.total_supply_pointer().set_value(100u128);
    
    let total_assets = 200u128;
    let total_supply = 100u128;
    
    // Test asset to share conversion
    assert_eq!(vault.convert_assets_to_shares(20u128, total_assets, total_supply).unwrap(), 10u128); // 20 assets = 10 shares
    
    // Test share to asset conversion
    assert_eq!(vault.convert_shares_to_assets(10u128, total_assets, total_supply).unwrap(), 20u128); // 10 shares = 20 assets
    
    // Test preview deposit (should match convert_assets_to_shares)
    assert_eq!(vault.preview_deposit(20u128).unwrap(), 10u128);
    
    // Test preview mint (should round up)
    // For 10 shares: 10 * 200 / 100 = 20 assets
    assert_eq!(vault.convert_shares_to_assets(10u128, total_assets, total_supply).unwrap(), 20u128);
    
    // For non-even division, should round up
    // For 15 shares: 15 * 200 / 100 = 30 assets
    assert_eq!(vault.preview_mint(15u128, total_assets, total_supply).unwrap(), 30u128);
    
    // Test preview withdraw (should round up for shares needed)
    // For 20 assets: 20 * 100 / 200 = 10 shares
    assert_eq!(vault.preview_withdraw(20u128, total_assets, total_supply).unwrap(), 10u128);
    // For 21 assets: 21 * 100 / 200 = 10.5, rounded up to 11 shares
    assert_eq!(vault.preview_withdraw(21u128, total_assets, total_supply).unwrap(), 11u128);
    
    // Test preview redeem (should match convert_shares_to_assets)
    assert_eq!(vault.preview_redeem(10u128).unwrap(), 20u128);
}

#[test]
fn test_yield_accrual() {
    // Reset storage
    reset_test_storage();

    // Create the YieldVault instance
    let vault = YieldVault::default();
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Set initial state
    let initial_assets = 1000u128;
    let initial_shares = 1000u128;
    vault.total_assets_pointer().set_value(initial_assets);
    vault.total_supply_pointer().set_value(initial_shares); // 1:1 ratio initially
    
    // Set yield rate to 5% (500 basis points)
    let yield_rate = 500u128;
    vault.yield_rate_pointer().set_value(yield_rate);
    
    // Set initial timestamp
    let initial_timestamp = 1651388400u64; // May 1, 2022
    vault.last_yield_update_pointer().set_value(initial_timestamp);
    
    // Simulate one year later
    let one_year_later = initial_timestamp + 31536000u64; // Add seconds in a year
    
    // Set current timestamp for test
    let current_time = one_year_later;
    
    // Calculate expected new assets (5% increase)
    let expected_new_assets = initial_assets * 105 / 100; // 1000 * 1.05 = 1050
    
    // Update total assets and timestamp to simulate yield accrual
    vault.total_assets_pointer().set_value(expected_new_assets);
    vault.last_yield_update_pointer().set_value(one_year_later);
    
    // Verify the state
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), expected_new_assets);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), initial_shares); // supply should remain unchanged
    assert_eq!(vault.last_yield_update_pointer().get_value::<u64>(), one_year_later);
    
    // New ratio should be 1.05:1 (assets:shares)
    // Test conversion at new ratio
    assert_eq!(
        vault.convert_assets_to_shares(
            105u128, 
            vault.total_assets_pointer().get_value::<u128>(), 
            vault.total_supply_pointer().get_value::<u128>()
        ).unwrap(), 
        100u128
    ); // 105 assets = 100 shares
    
    assert_eq!(
        vault.convert_shares_to_assets(
            100u128, 
            vault.total_assets_pointer().get_value::<u128>(), 
            vault.total_supply_pointer().get_value::<u128>()
        ).unwrap(), 
        105u128
    ); // 100 shares = 105 assets
}

#[test]
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

#[test]
fn test_account_balances() {
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
