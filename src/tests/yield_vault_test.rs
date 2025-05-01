// YieldVault Unit Tests
// Tests for core functionality of the YieldVault contract

use crate::YieldVault;
use crate::tests::mock::*;
use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::parcel::AlkaneTransfer;
use std::collections::HashSet;
use serde_json;

// Set up a clean vault for each test
fn setup_vault() -> YieldVault {
    // Clear storage and reset timestamps
    setup_test_environment();
    
    let vault = YieldVault::default();
    
    vault
}

// Initialize a vault for testing
fn initialize_vault(vault: &mut YieldVault) {
    let ctx = MockContext::default();
    
    let result = vault.initialize(
        "Test Vault".to_string(),
        "vTEST".to_string(),
        "Test Asset".to_string(),
        "TEST".to_string(),
        18
    );
    
    assert!(result.is_ok());
}

#[test]
fn test_initialization() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // Verify that the vault was initialized with the correct values
    assert_eq!(storage::get_string("/name").unwrap(), "Test Vault");
    assert_eq!(storage::get_string("/symbol").unwrap(), "vTEST");
    assert_eq!(storage::get_string("/asset-name").unwrap(), "Test Asset");
    assert_eq!(storage::get_string("/asset-symbol").unwrap(), "TEST");
    assert_eq!(storage::get_u8("/decimals").unwrap(), 18);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 0);
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 0);
    assert_eq!(storage::get_u128("/yield-rate").unwrap(), 0);
    assert!(storage::get_bool("/initialized").unwrap());
    assert_eq!(storage::get_u64("/last-yield-update").unwrap(), get_timestamp());
}

#[test]
fn test_double_initialization() {
    let mut vault = setup_vault();
    
    // First initialization should succeed
    initialize_vault(&mut vault);
    
    // Second initialization should fail
    let result = vault.initialize(
        "Another Vault".to_string(),
        "vANOTHER".to_string(),
        "Another Asset".to_string(),
        "ANOTHER".to_string(),
        18
    );
    
    // Check that the initialization failed
    assert!(result.is_err());
    
    // Check that the original values remain unchanged
    assert_eq!(storage::get_string("/name").unwrap(), "Test Vault");
    assert_eq!(storage::get_string("/symbol").unwrap(), "vTEST");
}

#[test]
fn test_deposit() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // Deposit 100 assets
    let result = vault.deposit(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        100
    );
    
    assert!(result.is_ok());
    
    // Check state after deposit
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 100);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 100); // Initial 1:1 ratio
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 100);
}

#[test]
fn test_mint() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // Mint 100 shares
    let result = vault.mint(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        100
    );
    
    assert!(result.is_ok());
    
    // Check state after mint
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 100);
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 100); // Initial 1:1 ratio
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 100);
}

#[test]
fn test_multiple_deposits() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // First deposit: 100 assets by Alice
    let result1 = vault.deposit(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        100
    );
    assert!(result1.is_ok());
    
    // Second deposit: 150 assets by Bob
    let result2 = vault.deposit(
        generate_tx_hash(),
        "bob".to_string(),
        "bob".to_string(),
        150
    );
    assert!(result2.is_ok());
    
    // Check state after deposits
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 250);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 250);
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 100);
    assert_eq!(storage::get_u128("/balances/bob").unwrap(), 150);
}

#[test]
fn test_withdraw() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // First deposit 100 assets
    vault.deposit(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        100
    ).unwrap();
    
    // Withdraw 30 assets
    let result = vault.withdraw(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        "alice".to_string(),
        30
    );
    
    assert!(result.is_ok());
    
    // Check state after withdrawal
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 70);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 70);
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 70);
}

#[test]
fn test_redeem() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // First mint 100 shares
    vault.mint(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        100
    ).unwrap();
    
    // Redeem 30 shares
    let result = vault.redeem(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        "alice".to_string(),
        30
    );
    
    assert!(result.is_ok());
    
    // Check state after redemption
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 70);
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 70);
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 70);
}

#[test]
fn test_transaction_replay_protection() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    let tx_hash = "repeated_tx_hash".to_string();
    
    // First deposit with this hash should work
    let result1 = vault.deposit(
        tx_hash.clone(),
        "alice".to_string(),
        "alice".to_string(),
        100
    );
    
    assert!(result1.is_ok());
    
    // Second deposit with the same hash should fail
    let result2 = vault.deposit(
        tx_hash,
        "alice".to_string(),
        "alice".to_string(),
        100
    );
    
    assert!(result2.is_err());
    
    // State should reflect only the first deposit
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 100);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 100);
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 100);
}

#[test]
fn test_authorization_check() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // First deposit 100 assets as Alice
    vault.deposit(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        100
    ).unwrap();
    
    // Bob tries to withdraw Alice's assets
    let result = vault.withdraw(
        generate_tx_hash(),
        "bob".to_string(), // Caller is Bob
        "bob".to_string(),
        "alice".to_string(), // Owner is Alice
        30
    );
    
    // Should fail with authorization error
    assert!(result.is_err());
    
    // State should remain unchanged
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 100);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 100);
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 100);
}

#[test]
fn test_yield_accrual() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // Deposit 1000 assets
    vault.deposit(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        1000
    ).unwrap();
    
    // Set yield rate to 5% (500 basis points)
    vault.update_yield_rate(500).unwrap();
    
    // Check initial state
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 1000);
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 1000);
    assert_eq!(storage::get_u128("/yield-rate").unwrap(), 500);
    
    // Advance time by 1 year (in seconds)
    advance_time(365 * 24 * 60 * 60);
    
    // Trigger yield calculation by performing any operation
    vault.update_yield().unwrap();
    
    // Assets should have increased by about 5%
    // Exact calculation: 1000 * 500 * (365*24*60*60) / (10000 * 365*24*60*60) = 50
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 1050);
    
    // Total supply shouldn't have changed
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 1000);
}

#[test]
fn test_yield_after_partial_withdrawal() {
    let mut vault = setup_vault();
    initialize_vault(&mut vault);
    
    // Deposit 1000 assets
    vault.deposit(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        1000
    ).unwrap();
    
    // Set yield rate to 10% (1000 basis points)
    vault.update_yield_rate(1000).unwrap();
    
    // Advance time by 6 months
    advance_time(182 * 24 * 60 * 60);
    
    // Withdraw half of assets
    vault.withdraw(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        "alice".to_string(),
        500
    ).unwrap();
    
    // After the withdrawal, yield should have been applied
    // Approximate 5% for 6 months: 1000 * 0.05 = 50
    assert!(storage::get_u128("/total-assets").unwrap() > 500); // Should be around 550
    assert!(storage::get_u128("/total-assets").unwrap() <= 550); // But not more than 550
    
    // Advance time by another 6 months
    advance_time(183 * 24 * 60 * 60);
    
    // Calculate remaining balance
    let remaining_balance = storage::get_u128("/balances/alice").unwrap();
    
    // Withdraw remaining assets
    vault.redeem(
        generate_tx_hash(),
        "alice".to_string(),
        "alice".to_string(),
        "alice".to_string(),
        remaining_balance
    ).unwrap();
    
    // After the redemption, yield should have been applied to the reduced amount
    // Balance should be 0 after full redemption
    assert_eq!(storage::get_u128("/balances/alice").unwrap(), 0);
    // Total supply should be 0
    assert_eq!(storage::get_u128("/total-supply").unwrap(), 0);
    // Total assets should be 0
    assert_eq!(storage::get_u128("/total-assets").unwrap(), 0);
}
