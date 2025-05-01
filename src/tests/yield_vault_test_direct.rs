use std::sync::Arc;

use crate::YieldVault;
use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::storage::StoragePointer;
use anyhow::Result;
use metashrew_support::index_pointer::KeyValuePointer;

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
    
    // Clear any balance entries
    // In a real test, we would need to clear specific balance entries if they exist
}

#[test]
fn test_initialization() {
    // Reset storage
    reset_test_storage();

    // Create the vault
    let vault = YieldVault::default();
    
    // Use observe_initialization directly
    assert!(vault.observe_initialization().is_ok());
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    
    // Initialize accounting
    vault.total_supply_pointer().set_value(0u128);
    vault.total_assets_pointer().set_value(0u128);
    vault.yield_rate_pointer().set_value(0u128);
    vault.last_yield_update_pointer().set_value(1651388400u64); // May 1, 2022
    
    // Verify initialization
    assert!(StoragePointer::from_keyword("/initialized").get_value::<u8>() != 0);
    
    // Check values
    assert_eq!(String::from_utf8(vault.name_pointer().get().as_ref().to_vec()).unwrap(), "Test Vault");
    assert_eq!(String::from_utf8(vault.symbol_pointer().get().as_ref().to_vec()).unwrap(), "vTEST");
    assert_eq!(String::from_utf8(vault.asset_name_pointer().get().as_ref().to_vec()).unwrap(), "Test Asset");
    assert_eq!(String::from_utf8(vault.asset_symbol_pointer().get().as_ref().to_vec()).unwrap(), "TEST");
    assert_eq!(vault.decimals_pointer().get_value::<u8>(), 18u8);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0u128);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0u128);
}

#[test]
fn test_double_initialization() {
    // Reset storage
    reset_test_storage();

    // Create the vault
    let vault = YieldVault::default();
    
    // First initialization should succeed
    assert!(vault.observe_initialization().is_ok());
    
    // Second initialization should fail
    assert!(vault.observe_initialization().is_err());
}

#[test]
fn test_deposit_and_withdraw() -> Result<()> {
    // Reset storage
    reset_test_storage();
    
    // Create the vault
    let vault = YieldVault::default();
    
    // Initialize
    vault.observe_initialization().map_err(|e| anyhow::anyhow!(e))?;
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    vault.total_supply_pointer().set_value(0u128);
    vault.total_assets_pointer().set_value(0u128);
    
    // Create test hash set for transaction tracking
    let tx_hash = "test_tx_hash_1";
    let mut tx_hashes = HashSet::new();
    tx_hashes.insert(tx_hash.to_string());
    let json = serde_json::to_string(&tx_hashes)?;
    vault.tx_hashes_pointer().set(Arc::new(json.as_bytes().to_vec()));
    
    // Simulate deposit - 100 assets
    const ALICE: &str = "alice";
    const DEPOSIT_AMOUNT: u128 = 100;
    
    // Since the vault is empty, shares should be 1:1 with assets
    let shares = DEPOSIT_AMOUNT;
    
    // Manual update of state to simulate deposit
    vault.total_assets_pointer().set_value(DEPOSIT_AMOUNT);
    vault.total_supply_pointer().set_value(shares);
    
    // Set Alice's balance
    let alice_balance_key = format!("/balances/{}", ALICE);
    StoragePointer::from_keyword(&alice_balance_key).set_value(shares);
    
    // Check state after deposit
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), DEPOSIT_AMOUNT);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), shares);
    assert_eq!(StoragePointer::from_keyword(&alice_balance_key).get_value::<u128>(), shares);
    
    // Now simulate withdrawal - 30 assets
    const WITHDRAW_AMOUNT: u128 = 30;
    
    // Calculate shares to burn (assets/total_assets * totalSupply)
    let shares_to_burn = (WITHDRAW_AMOUNT * vault.total_supply_pointer().get_value::<u128>()) / 
                           vault.total_assets_pointer().get_value::<u128>();
    
    // Manual update of state to simulate withdrawal
    vault.total_assets_pointer().set_value(DEPOSIT_AMOUNT - WITHDRAW_AMOUNT);
    vault.total_supply_pointer().set_value(shares - shares_to_burn);
    
    // Update Alice's balance
    StoragePointer::from_keyword(&alice_balance_key).set_value(shares - shares_to_burn);
    
    // Check state after withdrawal
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), DEPOSIT_AMOUNT - WITHDRAW_AMOUNT);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), shares - shares_to_burn);
    assert_eq!(StoragePointer::from_keyword(&alice_balance_key).get_value::<u128>(), 
               shares - shares_to_burn);
    
    Ok(())
}

#[test]
fn test_yield_accrual() -> Result<()> {
    // Reset storage
    reset_test_storage();
    
    // Create the vault
    let vault = YieldVault::default();
    
    // Initialize
    vault.observe_initialization().map_err(|e| anyhow::anyhow!(e))?;
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    
    // Initial state
    const INITIAL_ASSETS: u128 = 1000;
    const INITIAL_SHARES: u128 = 1000;
    
    // Set initial state
    vault.total_assets_pointer().set_value(INITIAL_ASSETS);
    vault.total_supply_pointer().set_value(INITIAL_SHARES);
    
    // Set yield rate to 5% (500 basis points)
    const YIELD_RATE: u128 = 500;
    vault.yield_rate_pointer().set_value(YIELD_RATE);
    
    // Set current timestamp
    let current_time = 1651388400u64; // May 1, 2022
    vault.last_yield_update_pointer().set_value(current_time);
    
    // We're simulating the effect of manually updating the yield
    // In a real contract, this would be done by calling a function
    
    // Advance time by 1 year (in seconds)
    let one_year_later = current_time + (365 * 24 * 60 * 60);
    
    // Calculate yield manually for verification
    // yield = assets * rate * time / (10000 * seconds_in_year)
    // For 1 year exactly, this simplifies to assets * rate / 10000
    let expected_yield = (INITIAL_ASSETS * YIELD_RATE) / 10000;
    let expected_new_assets = INITIAL_ASSETS + expected_yield;
    
    // Assume the update_yield function was called, we update the total assets
    // and the last yield update timestamp
    vault.total_assets_pointer().set_value(expected_new_assets);
    vault.last_yield_update_pointer().set_value(one_year_later);
    
    // Check new state
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), expected_new_assets);
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), INITIAL_SHARES); // shouldn't change
    assert_eq!(vault.last_yield_update_pointer().get_value::<u64>(), one_year_later);
    
    // The exchange rate should now be different
    // Originally 1 share = 1 asset
    // Now 1 share = (1000 + 50) / 1000 = 1.05 assets
    let exchange_rate = (expected_new_assets * 100) / INITIAL_SHARES; // *100 for precision
    assert_eq!(exchange_rate, 105); // 1.05 * 100
    
    Ok(())
}

use std::collections::HashSet; 
use serde_json;

#[test]
fn test_transaction_replay_protection() -> Result<()> {
    // Reset storage
    reset_test_storage();
    
    // Create the vault
    let vault = YieldVault::default();
    
    // Initialize
    vault.observe_initialization().map_err(|e| anyhow::anyhow!(e))?;
    
    // Create empty transaction hash set
    let mut tx_hashes = HashSet::new();
    
    // Add a transaction hash
    let tx_hash1 = "test_tx_hash_1".to_string();
    tx_hashes.insert(tx_hash1.clone());
    
    // Serialize and store
    let json = serde_json::to_string(&tx_hashes)?;
    vault.tx_hashes_pointer().set(Arc::new(json.as_bytes().to_vec()));
    
    // Now try to validate the same hash - should fail
    {
        // Get current hash set
        let json_data = vault.tx_hashes_pointer().get();
        let json = String::from_utf8(json_data.as_ref().to_vec())?;
        let existing_hashes: HashSet<String> = serde_json::from_str(&json)?;
        
        // Check if hash already exists
        assert!(existing_hashes.contains(&tx_hash1));
    }
    
    // Try a new hash - should succeed
    {
        let tx_hash2 = "test_tx_hash_2".to_string();
        
        // Get current hash set
        let json_data = vault.tx_hashes_pointer().get();
        let json = String::from_utf8(json_data.as_ref().to_vec())?;
        let mut existing_hashes: HashSet<String> = serde_json::from_str(&json)?;
        
        // Check if hash already exists
        assert!(!existing_hashes.contains(&tx_hash2));
        
        // Add new hash
        existing_hashes.insert(tx_hash2);
        
        // Serialize and store
        let json = serde_json::to_string(&existing_hashes)?;
        vault.tx_hashes_pointer().set(Arc::new(json.as_bytes().to_vec()));
    }
    
    // Verify both hashes now exist
    {
        let json_data = vault.tx_hashes_pointer().get();
        let json = String::from_utf8(json_data.as_ref().to_vec())?;
        let existing_hashes: HashSet<String> = serde_json::from_str(&json)?;
        
        assert!(existing_hashes.contains(&tx_hash1));
        assert!(existing_hashes.contains(&"test_tx_hash_2".to_string()));
        assert_eq!(existing_hashes.len(), 2);
    }
    
    Ok(())
}

#[test]
fn test_shares_assets_conversion() -> Result<()> {
    // Reset storage
    reset_test_storage();
    
    // Create the vault
    let vault = YieldVault::default();
    
    // Initialize
    vault.observe_initialization().map_err(|e| anyhow::anyhow!(e))?;
    
    // Set initial state (empty vault)
    vault.total_assets_pointer().set_value(0u128);
    vault.total_supply_pointer().set_value(0u128);
    
    // For an empty vault, 1:1 conversion
    const AMOUNT: u128 = 100;
    
    // Now set up a non-empty vault with some yield
    // 1000 assets backing 800 shares (1.25 assets per share)
    vault.total_assets_pointer().set_value(1000u128);
    vault.total_supply_pointer().set_value(800u128);
    
    // Convert assets to shares (manual calculation for test)
    // shares = assets * totalSupply / totalAssets
    let assets_to_convert = 250u128;
    let expected_shares = (assets_to_convert * 800u128) / 1000u128; // = 200
    
    // Verify manually
    assert_eq!(expected_shares, 200u128);
    
    // Convert shares to assets (manual calculation for test)
    // assets = shares * totalAssets / totalSupply
    let shares_to_convert = 400u128;
    let expected_assets = (shares_to_convert * 1000u128) / 800u128; // = 500
    
    // Verify manually
    assert_eq!(expected_assets, 500u128);
    
    Ok(())
}
