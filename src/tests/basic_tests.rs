use std::sync::Arc;
use crate::YieldVault;
use alkanes_runtime::storage::StoragePointer;
use metashrew_support::index_pointer::KeyValuePointer;

// Reset storage before each test
fn reset_storage() {
    // Clear all storage keys
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
}

#[test]
fn test_basic_initialization_flag() {
    reset_storage();
    
    // Create a vault
    let vault = YieldVault::default();
    
    // Check that it's not initialized yet
    assert_eq!(vault.initialized_pointer().get_value::<u8>(), 0);
    
    // Set initialized
    vault.initialized_pointer().set_value(1u8);
    
    // Verify it's now initialized
    assert_eq!(vault.initialized_pointer().get_value::<u8>(), 1);
}

#[test]
fn test_basic_observe_initialization() {
    reset_storage();
    
    // Create a vault
    let vault = YieldVault::default();
    
    // Check that initialization succeeds
    let result = vault.observe_initialization();
    assert!(result.is_ok());
    
    // Check that a second initialization fails
    let result = vault.observe_initialization();
    assert!(result.is_err());
}

#[test]
fn test_basic_storage_values() {
    reset_storage();
    
    // Create a vault
    let vault = YieldVault::default();
    
    // Set some basic values
    vault.name_pointer().set(Arc::new("Test Name".as_bytes().to_vec()));
    vault.total_supply_pointer().set_value(100u128);
    
    // Check that values are stored correctly
    let name_bytes = vault.name_pointer().get();
    let name = String::from_utf8(name_bytes.as_ref().to_vec()).unwrap();
    assert_eq!(name, "Test Name");
    
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 100);
}
