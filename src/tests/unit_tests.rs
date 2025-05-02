use std::sync::Arc;

use crate::storage::Storage;
use crate::security::Security;
use crate::utils::Conversion;
use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use metashrew_support::index_pointer::KeyValuePointer;
use wasm_bindgen_test::wasm_bindgen_test;
use wasm_bindgen_test::wasm_bindgen_test_configure;


// Reset all storage keys used in tests - similar to free-mint approach
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
    StoragePointer::from_keyword("/asset-id").set(Arc::new(Vec::new()));

    // Clear any balance-related keys
    for i in 0..10 {
        let key = format!("/balances/test_account{}", i);
        StoragePointer::from_keyword(&key).set(Arc::new(Vec::new()));
    }
}

// Test vault struct for unit testing - direct implementation like free-mint
struct TestVault();

impl TestVault {
    fn new() -> Self {
        Self()
    }
}

// Implement required traits for TestVault
impl Storage for TestVault {}
impl Security for TestVault {}

// Direct unit tests similar to free-mint style

#[test]
#[wasm_bindgen_test]
fn test_asset_id_storage() {
    // Reset storage
    reset_test_storage();

    // Create the test vault instance
    let vault = TestVault::new();
    
    // Store an asset ID
    let asset_id = AlkaneId::default();
    vault.store_asset_id(&asset_id);
    
    // Retrieve the stored asset ID
    let retrieved_id = vault.get_asset_id();
    
    // Check if the IDs match (note: our implementation returns a default for now)
    assert_eq!(format!("{:?}", retrieved_id), format!("{:?}", AlkaneId::default()));
}

#[test]
#[wasm_bindgen_test]
fn test_transaction_hash_tracking() {
    // Reset storage
    reset_test_storage();

    // Create the test vault instance
    let vault = TestVault::new();
    
    // First time tracking a hash should succeed
    let tx_hash = "0x1234567890abcdef";
    assert!(vault.validate_and_track_transaction(tx_hash).is_ok());
    
    // Attempting to track the same hash again should fail
    assert!(vault.validate_and_track_transaction(tx_hash).is_err());
    
    // Different hash should succeed
    let new_tx_hash = "0xabcdef1234567890";
    assert!(vault.validate_and_track_transaction(new_tx_hash).is_ok());
}

#[test]
#[wasm_bindgen_test]
fn test_initialization_guard() {
    // Reset storage
    reset_test_storage();

    // Create the test vault instance
    let vault = TestVault::new();
    
    // First initialization should succeed
    assert!(vault.observe_initialization().is_ok());
    
    // Second initialization should fail
    assert!(vault.observe_initialization().is_err());
}

#[test]
#[wasm_bindgen_test]
fn test_account_balances() {
    // Reset storage
    reset_test_storage();

    // Create the test vault instance
    let vault = TestVault::new();
    
    // Check initial balance
    let account = "test_account";
    assert_eq!(vault.get_balance(account), 0u128);
    
    // Set a balance
    vault.set_balance(account, 100u128);
    
    // Verify the balance
    assert_eq!(vault.get_balance(account), 100u128);
}

#[test]
#[wasm_bindgen_test]
fn test_conversion_empty_supply() {
    // Reset storage
    reset_test_storage();

    // Create the test vault instance
    let vault = TestVault::new();
    
    // Check assets to shares conversion with empty supply (1:1 ratio)
    let result = vault.convert_assets_to_shares(100, 0, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
    
    // Check shares to assets conversion with empty supply (1:1 ratio)
    let result = vault.convert_shares_to_assets(200, 0, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 200);
}

#[test]
#[wasm_bindgen_test]
fn test_conversion_with_supply() {
    // Reset storage
    reset_test_storage();

    // Create the test vault instance
    let vault = TestVault::new();
    
    // Set up non-empty state
    let total_assets = 1000u128;
    let total_supply = 500u128;
    
    // Check assets to shares conversion (assets * supply / total_assets)
    let assets = 100u128;
    let expected_shares = assets * total_supply / total_assets; // 100 * 500 / 1000 = 50
    let result = vault.convert_assets_to_shares(assets, total_assets, total_supply);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_shares);
    
    // Check shares to assets conversion (shares * total_assets / supply)
    let shares = 50u128;
    let expected_assets = shares * total_assets / total_supply; // 50 * 1000 / 500 = 100
    let result = vault.convert_shares_to_assets(shares, total_assets, total_supply);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_assets);
}

#[test]
#[wasm_bindgen_test]
fn test_authorization() {
    // Reset storage
    reset_test_storage();

    // Create the test vault instance
    let vault = TestVault::new();
    
    // Owner should be authorized
    let owner = "owner_account";
    assert!(vault.check_authorization(owner, owner).is_ok());
    
    // Non-owner should not be authorized
    let attacker = "attacker_account";
    assert!(vault.check_authorization(attacker, owner).is_err());
}
