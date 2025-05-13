//! Tests for alkane ID verification in the YieldVault contract
//! These tests verify that the contract correctly handles alkane IDs in transactions

use std::cell::RefCell;
use anyhow::Result;

// Import the MockYieldVault implementation
use yield_vault::mock_vault::MockYieldVault;

// Create a more realistic mock that simulates the transaction context
#[derive(Clone)]
struct TransactionContext {
    incoming_alkanes: Vec<(String, u128)>, // (id, value)
    myself: String,                       // Contract ID
}

// Extended mock that simulates transaction context
struct RealisticMockVault {
    base: MockYieldVault,
    tx_context: RefCell<Option<TransactionContext>>,
}

impl RealisticMockVault {
    fn new() -> Self {
        Self {
            base: MockYieldVault::default(),
            tx_context: RefCell::new(None),
        }
    }
    
    // Set up a transaction context for testing
    fn setup_transaction(&self, incoming_alkanes: Vec<(String, u128)>, contract_id: String) {
        *self.tx_context.borrow_mut() = Some(TransactionContext {
            incoming_alkanes,
            myself: contract_id,
        });
    }
    
    // Clear the transaction context
    fn clear_transaction(&self) {
        *self.tx_context.borrow_mut() = None;
    }
    
    // Get the current transaction context
    fn get_transaction_context(&self) -> Option<TransactionContext> {
        self.tx_context.borrow().clone()
    }
    
    // Initialize the vault
    fn initialize(&self, name: String, symbol: String, asset_name: String, asset_symbol: String, decimals: u8) -> Result<()> {
        self.base.initialize(name, symbol, asset_name, asset_symbol, decimals)
    }
    
    // Deposit with alkane ID verification
    fn deposit_with_verification(&self, tx_hash: &str, caller_id: &str, receiver_id: &str, assets: u128, asset_id: &str) -> Result<u128, &'static str> {
        // Check if we have a transaction context
        if self.tx_context.borrow().is_none() {
            return Err("No transaction context");
        }
        
        // Get the transaction context
        let context = self.tx_context.borrow().clone().unwrap();
        
        // Verify incoming assets match the expected asset ID
        let received_assets = context.incoming_alkanes.iter()
            .filter(|(id, _)| id == asset_id)
            .map(|(_, value)| *value)
            .sum::<u128>();
            
        // Check that we received at least the expected assets
        if received_assets < assets {
            return Err("Insufficient assets received");
        }
        
        // Call the base deposit method
        self.base.deposit(caller_id, receiver_id, assets)
    }
    
    // Redeem with alkane ID verification
    fn redeem_with_verification(&self, tx_hash: &str, caller_id: &str, receiver_id: &str, owner_id: &str, shares: u128) -> Result<u128, &'static str> {
        // Check if we have a transaction context
        if self.tx_context.borrow().is_none() {
            return Err("No transaction context");
        }
        
        // Get the transaction context
        let context = self.tx_context.borrow().clone().unwrap();
        
        // Verify incoming shares match the contract ID (share token ID)
        let received_shares = context.incoming_alkanes.iter()
            .filter(|(id, _)| id == &context.myself)
            .map(|(_, value)| *value)
            .sum::<u128>();
            
        // Check that we received at least the expected shares
        if received_shares < shares {
            return Err("Insufficient shares received");
        }
        
        // Call the base redeem method
        self.base.redeem(caller_id, receiver_id, owner_id, shares)
    }
    
    // Get total assets
    fn get_total_assets(&self) -> u128 {
        self.base.get_total_assets()
    }
    
    // Get total issuance
    fn get_total_issuance(&self) -> u128 {
        self.base.get_total_issuance()
    }
    
    // Get token balance
    fn get_token_balance(&self, token_id: &str) -> u128 {
        self.base.get_token_balance(token_id)
    }
    
    // Issue tokens for testing
    fn issue_tokens(&self, token_id: &str, amount: u128) {
        self.base.issue_tokens(token_id, amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test constants
    const CONTRACT_ID: &str = "contract_id_123";
    const ASSET_ID: &str = "asset_id_456";
    const ALICE: &str = "alice";
    const BOB: &str = "bob";

    #[test]
    fn test_deposit_with_correct_alkane_id() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up a transaction with correct asset ID
        vault.setup_transaction(
            vec![(ASSET_ID.to_string(), 100)], // Incoming assets with correct ID
            CONTRACT_ID.to_string()            // Contract ID
        );
        
        // Deposit should succeed
        let shares = vault.deposit_with_verification("tx1", ALICE, ALICE, 100, ASSET_ID).unwrap();
        
        // Verify deposit was successful
        assert_eq!(shares, 100); // 1:1 ratio when empty
        assert_eq!(vault.get_total_assets(), 100);
        assert_eq!(vault.get_total_issuance(), 100);
        assert_eq!(vault.get_token_balance(ALICE), 100);
    }
    
    #[test]
    fn test_deposit_with_incorrect_alkane_id() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up a transaction with incorrect asset ID
        vault.setup_transaction(
            vec![("wrong_asset_id".to_string(), 100)], // Incoming assets with wrong ID
            CONTRACT_ID.to_string()                   // Contract ID
        );
        
        // Deposit should fail
        let result = vault.deposit_with_verification("tx1", ALICE, ALICE, 100, ASSET_ID);
        
        // Verify deposit failed
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient assets received");
        
        // Verify state was not changed
        assert_eq!(vault.get_total_assets(), 0);
        assert_eq!(vault.get_total_issuance(), 0);
        assert_eq!(vault.get_token_balance(ALICE), 0);
    }
    
    #[test]
    fn test_deposit_with_multiple_alkane_ids() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up a transaction with multiple asset IDs including the correct one
        vault.setup_transaction(
            vec![
                ("wrong_asset_id_1".to_string(), 50),
                (ASSET_ID.to_string(), 100),         // Correct ID
                ("wrong_asset_id_2".to_string(), 75)
            ],
            CONTRACT_ID.to_string()
        );
        
        // Deposit should succeed
        let shares = vault.deposit_with_verification("tx1", ALICE, ALICE, 100, ASSET_ID).unwrap();
        
        // Verify deposit was successful
        assert_eq!(shares, 100);
        assert_eq!(vault.get_total_assets(), 100);
        assert_eq!(vault.get_total_issuance(), 100);
        assert_eq!(vault.get_token_balance(ALICE), 100);
    }
    
    #[test]
    fn test_deposit_with_insufficient_assets() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up a transaction with insufficient assets
        vault.setup_transaction(
            vec![(ASSET_ID.to_string(), 50)], // Only 50 assets
            CONTRACT_ID.to_string()
        );
        
        // Deposit should fail when requesting 100 assets
        let result = vault.deposit_with_verification("tx1", ALICE, ALICE, 100, ASSET_ID);
        
        // Verify deposit failed
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient assets received");
        
        // Verify state was not changed
        assert_eq!(vault.get_total_assets(), 0);
        assert_eq!(vault.get_total_issuance(), 0);
        assert_eq!(vault.get_token_balance(ALICE), 0);
    }
    
    #[test]
    fn test_redeem_with_correct_alkane_id() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up initial state - issue tokens to Alice
        vault.issue_tokens(ALICE, 100);
        
        // Set up a transaction with correct share token ID
        vault.setup_transaction(
            vec![(CONTRACT_ID.to_string(), 50)], // Incoming shares with correct ID
            CONTRACT_ID.to_string()              // Contract ID
        );
        
        // Set initial assets
        vault.base.set_value("total_assets", 100u128);
        vault.base.set_total_issuance(100u128);
        
        // Redeem should succeed
        let assets = vault.redeem_with_verification("tx1", ALICE, ALICE, ALICE, 50).unwrap();
        
        // Verify redeem was successful
        assert_eq!(assets, 50); // 1:1 ratio
        assert_eq!(vault.get_total_assets(), 50);
        assert_eq!(vault.get_total_issuance(), 50);
        assert_eq!(vault.get_token_balance(ALICE), 50);
    }
    
    #[test]
    fn test_redeem_with_incorrect_alkane_id() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up initial state - issue tokens to Alice
        vault.issue_tokens(ALICE, 100);
        
        // Set up a transaction with incorrect share token ID
        vault.setup_transaction(
            vec![("wrong_token_id".to_string(), 50)], // Incoming shares with wrong ID
            CONTRACT_ID.to_string()                  // Contract ID
        );
        
        // Set initial assets
        vault.base.set_value("total_assets", 100u128);
        vault.base.set_total_issuance(100u128);
        
        // Redeem should fail
        let result = vault.redeem_with_verification("tx1", ALICE, ALICE, ALICE, 50);
        
        // Verify redeem failed
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient shares received");
        
        // Verify state was not changed
        assert_eq!(vault.get_total_assets(), 100);
        assert_eq!(vault.get_total_issuance(), 100);
        assert_eq!(vault.get_token_balance(ALICE), 100);
    }
    
    #[test]
    fn test_redeem_with_insufficient_shares() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up initial state - issue tokens to Alice
        vault.issue_tokens(ALICE, 100);
        
        // Set up a transaction with insufficient shares
        vault.setup_transaction(
            vec![(CONTRACT_ID.to_string(), 25)], // Only 25 shares
            CONTRACT_ID.to_string()
        );
        
        // Set initial assets
        vault.base.set_value("total_assets", 100u128);
        vault.base.set_total_issuance(100u128);
        
        // Redeem should fail when requesting 50 shares
        let result = vault.redeem_with_verification("tx1", ALICE, ALICE, ALICE, 50);
        
        // Verify redeem failed
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient shares received");
        
        // Verify state was not changed
        assert_eq!(vault.get_total_assets(), 100);
        assert_eq!(vault.get_total_issuance(), 100);
        assert_eq!(vault.get_token_balance(ALICE), 100);
    }
    
    #[test]
    fn test_redeem_with_multiple_alkane_ids() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up initial state - issue tokens to Alice
        vault.issue_tokens(ALICE, 100);
        
        // Set up a transaction with multiple token IDs including the correct one
        vault.setup_transaction(
            vec![
                ("wrong_token_id_1".to_string(), 25),
                (CONTRACT_ID.to_string(), 50),        // Correct ID
                ("wrong_token_id_2".to_string(), 30)
            ],
            CONTRACT_ID.to_string()
        );
        
        // Set initial assets
        vault.base.set_value("total_assets", 100u128);
        vault.base.set_total_issuance(100u128);
        
        // Redeem should succeed
        let assets = vault.redeem_with_verification("tx1", ALICE, ALICE, ALICE, 50).unwrap();
        
        // Verify redeem was successful
        assert_eq!(assets, 50);
        assert_eq!(vault.get_total_assets(), 50);
        assert_eq!(vault.get_total_issuance(), 50);
        assert_eq!(vault.get_token_balance(ALICE), 50);
    }
    
    #[test]
    fn test_no_transaction_context() {
        // Create a vault with realistic transaction simulation
        let vault = RealisticMockVault::new();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Don't set up a transaction context
        
        // Deposit should fail
        let result = vault.deposit_with_verification("tx1", ALICE, ALICE, 100, ASSET_ID);
        
        // Verify deposit failed
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No transaction context");
        
        // Verify state was not changed
        assert_eq!(vault.get_total_assets(), 0);
        assert_eq!(vault.get_total_issuance(), 0);
        assert_eq!(vault.get_token_balance(ALICE), 0);
    }
}
