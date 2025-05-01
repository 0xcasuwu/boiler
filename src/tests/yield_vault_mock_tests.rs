use crate::tests::mock::{setup_test_environment, get_timestamp};
use crate::tests::mock_vault::MockYieldVault;

/// Test fixture to create a clean testing environment for each test
fn setup() -> MockYieldVault {
    // Reset the test environment
    setup_test_environment();
    
    // Create a new mock vault
    MockYieldVault::new()
}

#[test]
fn test_initialization() {
    let vault = setup();
    
    // Initialize the vault and verify it succeeded
    assert!(vault.initialize().is_ok());
    
    // Try to initialize again and verify it fails
    assert!(vault.initialize().is_err());
}

#[test]
fn test_deposit() {
    let vault = setup();
    
    // Initialize the vault
    vault.initialize().expect("Failed to initialize vault");
    
    // Deposit 100 assets
    let result = vault.deposit(
        "test_tx_1".to_string(),
        "alice".to_string(), 
        "alice".to_string(), 
        100
    );
    assert!(result.is_ok());
    
    // Check balances after deposit
    assert_eq!(vault.get_balance("alice"), 100);
    assert_eq!(vault.get_total_assets(), 100);
    assert_eq!(vault.get_total_supply(), 100);
}

#[test]
fn test_yield_accrual() {
    let vault = setup();
    
    // Initialize the vault
    vault.initialize().expect("Failed to initialize vault");
    
    // Deposit initial assets
    vault.deposit(
        "test_tx_1".to_string(),
        "alice".to_string(), 
        "alice".to_string(), 
        1000
    ).expect("Deposit failed");
    
    // Set a 5% yield rate
    vault.update_yield_rate(500).expect("Failed to update yield rate");
    
    // Check initial state
    assert_eq!(vault.get_total_assets(), 1000);
    assert_eq!(vault.get_total_supply(), 1000);
    assert_eq!(vault.get_yield_rate(), 500);
    
    // Advance time by 1 year (in seconds)
    let current_time = get_timestamp();
    let one_year_later = current_time + (365 * 24 * 60 * 60);
    crate::tests::mock::set_timestamp(one_year_later);
    
    // Update yield by calling any function that triggers yield update
    vault.update_yield().expect("Failed to update yield");
    
    // Check that assets increased by ~5%
    assert_eq!(vault.get_total_assets(), 1050);
    
    // Supply should remain the same
    assert_eq!(vault.get_total_supply(), 1000);
}
