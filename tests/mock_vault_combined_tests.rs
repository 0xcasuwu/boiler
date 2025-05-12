//! Combined penetration tests for the MockYieldVault implementation
//!
//! This file consolidates all penetration tests to ensure they can be run together.
//! It imports and re-exports tests from specialized test modules.

// Import the MockYieldVault implementation
use yield_vault::mock_vault::MockYieldVault;

// Basic penetration tests 
mod basic_penetration {
    use yield_vault::mock_vault::MockYieldVault;
    
    // Helper function to set up a vault with initial state for testing
    fn setup_vault() -> MockYieldVault {
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault", "TVT", "Test Asset", "ASSET", 8).unwrap();
        vault
    }
    
    #[test]
    fn test_initialization_correctness() {
        let vault = setup_vault();
        
        // Basic checks
        assert_eq!(vault.get_name().unwrap(), "Test Vault");
        assert_eq!(vault.get_symbol().unwrap(), "TVT");
        assert_eq!(vault.get_decimals().unwrap(), 8);
        
        // Security check - ensure vault is initialized
        let initialized_value: bool = vault.get_value("initialized");
        assert!(initialized_value, "Vault should be marked as initialized");
    }
    
    #[test]
    fn test_basic_deposit_and_withdraw_flow() {
        let vault = setup_vault();
        
        // User makes a deposit 
        let deposit_amount = 1000;
        let shares = vault.deposit("user", "user", deposit_amount).unwrap();
        
        // Check that assets and shares are recorded properly
        assert_eq!(vault.get_total_assets(), deposit_amount);
        assert_eq!(vault.get_total_issuance(), shares);
        assert_eq!(vault.get_token_balance("user"), shares);
        
        // Withdraw half
        let withdraw_amount = deposit_amount / 2;
        let shares_burned = vault.withdraw("user", "user", "user", withdraw_amount).unwrap();
        
        // Verify withdrawal worked correctly
        assert_eq!(vault.get_total_assets(), deposit_amount - withdraw_amount);
        assert_eq!(vault.get_total_issuance(), shares - shares_burned);
    }
    
    #[test]
    fn test_yield_accrual() {
        let vault = setup_vault();
        
        // Make initial deposit
        let deposit_amount = 10_000;
        vault.deposit("user", "user", deposit_amount).unwrap();
        
        // Set yield rate (5%)
        vault.update_yield(500).unwrap();
        
        // Fast forward time (simulate blocks passing)
        let current_height = vault.get_block_height();
        vault.set_block_height(current_height + 100_000); // Significant time passing
        
        // Update yield 
        vault.update_yield().unwrap();
        
        // Check assets have increased but shares remain the same
        assert!(vault.get_total_assets() > deposit_amount, 
                "Assets should increase due to yield accrual");
        assert_eq!(vault.get_total_issuance(), deposit_amount, 
                  "Share supply should remain constant");
    }
}

// Main entry point for running all tests
#[cfg(test)]
mod all_penetration_tests {
    // Reexport the individual test modules so they all run together
    pub use super::basic_penetration;
    
    // Import the individual test modules to consolidate reporting
    #[path = "mock_vault_penetration_tests.rs"]
    mod penetration_tests;
    
    #[path = "mock_vault_advanced_penetration_tests.rs"]
    mod advanced_penetration_tests;
    
    #[path = "mock_vault_erc4626_specific_tests.rs"]
    mod erc4626_specific_tests;
}
