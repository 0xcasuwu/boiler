//! Invariant tests for the YieldVault protocol
//! These tests verify that the protocol's invariants remain stable under attack

use std::collections::HashMap;
use std::cell::RefCell;

// Import the MockYieldVault implementation
use yield_vault::mock_vault::MockYieldVault;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that the total assets and total supply invariants are maintained
    /// even when a malicious user tries to manipulate the protocol
    #[test]
    fn test_total_assets_supply_invariant() {
        // Create a vault with initial state
        let vault = MockYieldVault::default();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up initial users
        let alice = "alice";
        let bob = "bob";
        let mallory = "mallory"; // The attacker
        
        // Initial deposits
        vault.deposit("tx1", alice, 1000).unwrap();
        vault.deposit("tx2", bob, 2000).unwrap();
        
        // Verify initial state
        assert_eq!(vault.get_total_assets(), 3000);
        assert_eq!(vault.get_total_issuance(), 3000); // 1:1 ratio initially
        assert_eq!(vault.get_token_balance(alice), 1000);
        assert_eq!(vault.get_token_balance(bob), 2000);
        
        // Attack scenario 1: Mallory tries to deposit 0 assets
        // This should fail and not affect the invariants
        let result = vault.deposit("tx3", mallory, 0);
        assert!(result.is_err());
        
        // Verify invariants are maintained
        assert_eq!(vault.get_total_assets(), 3000);
        assert_eq!(vault.get_total_issuance(), 3000);
        
        // Attack scenario 2: Mallory tries to deposit a very small amount and then withdraw a large amount
        // First, deposit a small amount
        vault.deposit("tx4", mallory, 1).unwrap();
        
        // Verify state after small deposit
        assert_eq!(vault.get_total_assets(), 3001);
        assert_eq!(vault.get_total_issuance(), 3001);
        assert_eq!(vault.get_token_balance(mallory), 1);
        
        // Now try to withdraw more than deposited
        let result = vault.redeem("tx5", mallory, mallory, 1000);
        assert!(result.is_err());
        
        // Verify invariants are maintained
        assert_eq!(vault.get_total_assets(), 3001);
        assert_eq!(vault.get_total_issuance(), 3001);
        assert_eq!(vault.get_token_balance(mallory), 1);
        
        // Attack scenario 3: Mallory tries to manipulate the exchange rate by being the first depositor
        // Create a new vault for this test
        let new_vault = MockYieldVault::default();
        new_vault.initialize(
            "Test Vault 2".to_string(),
            "TEST2".to_string(),
            "Test Asset 2".to_string(),
            "ASSET2".to_string(),
            8
        ).unwrap();
        
        // Mallory deposits a very small amount as the first depositor
        new_vault.deposit("tx6", mallory, 1).unwrap();
        
        // Verify state after first deposit
        assert_eq!(new_vault.get_total_assets(), 1);
        assert_eq!(new_vault.get_total_issuance(), 1);
        assert_eq!(new_vault.get_token_balance(mallory), 1);
        
        // Now Alice deposits a large amount
        new_vault.deposit("tx7", alice, 1000000).unwrap();
        
        // Verify state after Alice's deposit
        assert_eq!(new_vault.get_total_assets(), 1000001);
        assert_eq!(new_vault.get_total_issuance(), 1000001);
        assert_eq!(new_vault.get_token_balance(alice), 1000000);
        
        // Mallory tries to redeem her single share - make sure caller and owner are the same
        let assets = new_vault.redeem("tx8", mallory, mallory, 1).unwrap();
        
        // Verify Mallory only gets 1 asset back (not disproportionately more)
        assert_eq!(assets, 1);
        
        // Verify final state
        assert_eq!(new_vault.get_total_assets(), 1000000);
        assert_eq!(new_vault.get_total_issuance(), 1000000);
        assert_eq!(new_vault.get_token_balance(mallory), 0);
        assert_eq!(new_vault.get_token_balance(alice), 1000000);
    }
    
    /// Test that the yield accrual mechanism cannot be exploited
    #[test]
    fn test_yield_accrual_invariant() {
        // Create a vault with initial state
        let vault = MockYieldVault::default();
        
        // Set yield rate to 10% (1000 basis points)
        vault.set_yield_rate(1000);
        
        // Set up users
        let alice = "alice";
        let bob = "bob";
        let mallory = "mallory"; // The attacker
        
        // Issue tokens to mallory for testing - must be done before initialization
        vault.issue_tokens(mallory, 100);
        
        // Initialize the vault
        vault.initialize(
            "Yield Test Vault".to_string(),
            "YIELD".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Initial deposits
        vault.deposit("tx1", alice, 1000).unwrap();
        vault.deposit("tx2", bob, 2000).unwrap();
        
        // Verify initial state
        assert_eq!(vault.get_total_assets(), 3000);
        assert_eq!(vault.get_total_issuance(), 3000);
        
        // Simulate time passing (1 year worth of blocks)
        vault.update_yield_for_blocks(31536000).unwrap();
        
        // Verify yield has accrued (~10% increase)
        let total_assets_after_yield = vault.get_total_assets();
        let expected_min = 3000 + (3000 * 10 / 100) - 5; // Allow for rounding
        let expected_max = 3000 + (3000 * 10 / 100) + 5;
        assert!(total_assets_after_yield >= expected_min && total_assets_after_yield <= expected_max);
        
        // Total supply should remain unchanged
        assert_eq!(vault.get_total_issuance(), 3000);
        
        // Attack scenario: Mallory tries to manipulate the yield by making many small deposits/withdrawals
        for i in 0..10 {
            // Deposit a small amount - use a larger amount to avoid "Zero tokens minted" error
            vault.deposit(format!("tx_deposit_{}", i).as_str(), mallory, 10).unwrap();
            
            // Withdraw it immediately - make sure caller and owner are the same
            vault.redeem(format!("tx_withdraw_{}", i).as_str(), mallory, mallory, 10).unwrap();
            
            // Try to force yield updates
            vault.update_yield().unwrap();
        }
        
        // Verify that Mallory's actions didn't significantly affect the yield
        let total_assets_after_attack = vault.get_total_assets();
        
        // Allow for a wider range due to the multiple yield updates
        let wider_min = expected_min - 10;
        let wider_max = expected_max + 10;
        assert!(total_assets_after_attack >= wider_min && total_assets_after_attack <= wider_max,
                "Expected assets between {} and {}, got {}",
                wider_min, wider_max, total_assets_after_attack);
        
        // Total supply should be close to the original
        let total_issuance_after_attack = vault.get_total_issuance();
        let issuance_diff = if total_issuance_after_attack > 3000 {
            total_issuance_after_attack - 3000
        } else {
            3000 - total_issuance_after_attack
        };
        
        // Allow for a small difference due to rounding in the multiple operations
        assert!(issuance_diff <= 10, 
                "Expected total issuance close to 3000, got {}", 
                total_issuance_after_attack);
        
        // Alice and Bob should still have their original shares
        assert_eq!(vault.get_token_balance(alice), 1000);
        assert_eq!(vault.get_token_balance(bob), 2000);
    }
    
    /// Test that the token-based authorization prevents unauthorized withdrawals
    #[test]
    fn test_authorization_invariant() {
        // Create a vault with initial state
        let vault = MockYieldVault::default();
        
        // Initialize the vault
        vault.initialize(
            "Auth Test Vault".to_string(),
            "AUTH".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up users
        let alice = "alice";
        let bob = "bob";
        let mallory = "mallory"; // The attacker
        
        // Alice deposits assets
        vault.deposit("tx1", alice, 1000).unwrap();
        
        // Verify initial state
        assert_eq!(vault.get_total_assets(), 1000);
        assert_eq!(vault.get_total_issuance(), 1000);
        assert_eq!(vault.get_token_balance(alice), 1000);
        
        // Attack scenario: Mallory tries to withdraw Alice's assets
        // This should fail because Mallory is not authorized
        let result = vault.withdraw("mallory_tx", mallory, alice, 500);
        assert!(result.is_err());
        
        // Verify invariants are maintained
        assert_eq!(vault.get_total_assets(), 1000);
        assert_eq!(vault.get_total_issuance(), 1000);
        assert_eq!(vault.get_token_balance(alice), 1000);
        assert_eq!(vault.get_token_balance(mallory), 0);
        
        // Attack scenario: Mallory tries to redeem Alice's shares
        // This should fail because Mallory is not authorized
        let result = vault.redeem("mallory_tx2", mallory, alice, 500);
        assert!(result.is_err());
        
        // Verify invariants are maintained
        assert_eq!(vault.get_total_assets(), 1000);
        assert_eq!(vault.get_total_issuance(), 1000);
        assert_eq!(vault.get_token_balance(alice), 1000);
        assert_eq!(vault.get_token_balance(mallory), 0);
    }
    
    /// Test that the conversion functions maintain their mathematical properties
    #[test]
    fn test_conversion_invariants() {
        // Create a vault with initial state
        let vault = MockYieldVault::default();
        
        // Set up users
        let alice = "alice";
        
        // Issue tokens to alice for testing - must be done before initialization
        vault.issue_tokens(alice, 500);
        
        // Initialize the vault
        vault.initialize(
            "Conversion Test Vault".to_string(),
            "CONV".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up initial state with non-trivial exchange rate
        // 1000 assets = 500 shares (2:1 ratio)
        vault.set_total_assets(1000);
        vault.set_total_issuance(500);
        
        // Test invariant: Converting assets to shares and back should return the original assets (minus rounding)
        let assets = 100;
        let shares = vault.convert_assets_to_tokens(assets, 1000, 500).unwrap();
        let assets_back = vault.convert_tokens_to_assets(shares, 1000, 500).unwrap();
        
        // Due to integer division, we might lose some precision, but it should be minimal
        let tolerance = 1;
        assert!(if assets >= assets_back { assets - assets_back } else { assets_back - assets } <= tolerance);
        
        // Test invariant: Converting shares to assets and back should return the original shares (minus rounding)
        let shares = 50;
        let assets = vault.convert_tokens_to_assets(shares, 1000, 500).unwrap();
        let shares_back = vault.convert_assets_to_tokens(assets, 1000, 500).unwrap();
        
        // Due to integer division, we might lose some precision, but it should be minimal
        assert!(if shares >= shares_back { shares - shares_back } else { shares_back - shares } <= tolerance);
        
        // Test with various asset amounts to ensure the invariant holds
        for assets in [1, 10, 100, 1000, 10000].iter() {
            let shares = vault.convert_assets_to_tokens(*assets, 1000, 500).unwrap();
            let assets_back = vault.convert_tokens_to_assets(shares, 1000, 500).unwrap();
            
            // Calculate the maximum acceptable error (0.1% of the original amount)
            let max_error = (*assets * 1) / 1000 + 1;
            
            let diff = if *assets >= assets_back { *assets - assets_back } else { assets_back - *assets };
            assert!(diff <= max_error, 
                    "Conversion error too large: {} assets -> {} shares -> {} assets", 
                    assets, shares, assets_back);
        }
    }
    
    /// Test that the preview functions accurately predict the actual operations
    #[test]
    fn test_preview_function_invariants() {
        // Create a vault with initial state
        let vault = MockYieldVault::default();
        
        // Set up users
        let alice = "alice";
        
        // Issue tokens to alice for testing - must be done before initialization
        vault.issue_tokens(alice, 500);
        
        // Initialize the vault
        vault.initialize(
            "Preview Test Vault".to_string(),
            "PREV".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Set up initial state with non-trivial exchange rate
        // 1000 assets = 500 shares (2:1 ratio)
        vault.set_total_assets(1000);
        vault.set_total_issuance(500);
        
        // Test preview_deposit
        let assets_to_deposit = 100;
        let predicted_shares = vault.preview_deposit(assets_to_deposit).unwrap();
        
        // Perform the actual deposit
        let actual_shares = vault.deposit("tx1", alice, assets_to_deposit).unwrap();
        
        // Verify the prediction was accurate
        assert_eq!(predicted_shares, actual_shares);
        
        // Test preview_withdraw
        let assets_to_withdraw = 50;
        let predicted_shares = vault.preview_withdraw(assets_to_withdraw);
        
        // Perform the actual withdrawal - make sure caller and owner are the same
        let actual_shares = vault.withdraw("tx_withdraw_1", alice, alice, assets_to_withdraw).unwrap();
        
        // Verify the prediction was accurate
        assert_eq!(predicted_shares, actual_shares);
        
        // Test preview_mint
        let shares_to_mint = 25;
        let predicted_assets = vault.preview_mint(shares_to_mint);
        println!("Predicted assets: {}", predicted_assets);
        println!("Alice token balance before mint: {}", vault.get_token_balance(alice));
        println!("Total assets before mint: {}", vault.get_total_assets());
        println!("Total issuance before mint: {}", vault.get_total_issuance());
        
        // Perform the actual mint
        match vault.mint("tx_mint_1", alice, shares_to_mint) {
            Ok(assets) => {
                let actual_assets = assets;
                println!("Actual assets: {}", actual_assets);
                println!("Alice token balance after mint: {}", vault.get_token_balance(alice));
                println!("Total assets after mint: {}", vault.get_total_assets());
                println!("Total issuance after mint: {}", vault.get_total_issuance());
                
                // Verify the prediction was accurate
                assert_eq!(predicted_assets, actual_assets);
            },
            Err(e) => {
                println!("Error: {}", e);
                println!("Alice token balance after error: {}", vault.get_token_balance(alice));
                println!("Total assets after error: {}", vault.get_total_assets());
                println!("Total issuance after error: {}", vault.get_total_issuance());
                panic!("Mint failed: {}", e);
            }
        }
        
        // Test preview_redeem
        let shares_to_redeem = 25;
        let predicted_assets = vault.preview_redeem(shares_to_redeem).unwrap();
        
        // Perform the actual redemption
        let actual_assets = vault.redeem("tx_redeem_1", alice, alice, shares_to_redeem).unwrap();
        
        // Verify the prediction was accurate
        assert_eq!(predicted_assets, actual_assets);
    }
}
