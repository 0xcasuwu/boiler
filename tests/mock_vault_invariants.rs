//! Invariant and sanity checks for the MockYieldVault implementation
//! These tests verify that critical system invariants are maintained under various conditions

use yield_vault::mock_vault::MockYieldVault;

#[cfg(test)]
mod invariant_tests {
    use super::*;
    
    #[test]
    fn test_exchange_rate_invariant() {
        // Setup vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        
        // Function to verify the exchange rate invariant:
        // total_assets / total_issuance = assets_per_token
        let verify_invariant = |v: &MockYieldVault, expected_ratio: Option<f64>| {
            let total_assets = v.get_total_assets();
            let total_issuance = v.get_total_issuance();
            
            if total_issuance == 0 {
                assert_eq!(total_assets, 0, "If no tokens are issued, assets should be 0");
                return;
            }
            
            let exchange_rate = total_assets as f64 / total_issuance as f64;
            
            if let Some(expected) = expected_ratio {
                let tolerance = 0.001; // Allow 0.1% tolerance for rounding errors
                assert!((exchange_rate - expected).abs() < tolerance * expected, 
                        "Exchange rate should be {}, got {}", expected, exchange_rate);
            }
            
            // Verify preview functions match exchange rate
            let test_amount = 1000u128;
            let preview_assets = v.preview_redeem(test_amount).unwrap();
            let calculated_assets = (test_amount as f64 * exchange_rate).round() as u128;
            
            // Allow for small rounding differences
            assert!((preview_assets as i128 - calculated_assets as i128).abs() <= 1,
                    "Preview redemption should match exchange rate calculation");
        };
        
        // Initial state - empty vault
        verify_invariant(&vault, None);
        
        // After first deposit
        vault.deposit("alice", "alice", 1000).unwrap();
        verify_invariant(&vault, Some(1.0)); // 1:1 ratio initially
        
        // After yield accrual
        vault.set_block_height(vault.get_block_height() + 31_536_000); // 1 year
        vault.update_yield().unwrap();
        // Should be around 1.05 with 5% default yield
        let expected_rate = 1000.0 * 1.05 / 1000.0;
        verify_invariant(&vault, Some(expected_rate));
        
        // After second deposit at new exchange rate
        vault.deposit("bob", "bob", 1000).unwrap();
        verify_invariant(&vault, Some(expected_rate));
        
        // After redemption
        vault.redeem("alice", "alice", "alice", 500).unwrap();
        verify_invariant(&vault, Some(expected_rate));
    }
    
    #[test]
    fn test_user_balance_sum_equals_total_issuance() {
        // Setup vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        
        // Function to verify the token balance invariant:
        // sum of all user token balances = total issuance
        let verify_balance_invariant = |v: &MockYieldVault, users: &[&str]| {
            let total_issuance = v.get_total_issuance();
            let sum_of_balances: u128 = users.iter()
                .map(|user| v.get_token_balance(user))
                .sum();
                
            assert_eq!(sum_of_balances, total_issuance, 
                      "Sum of user balances ({}) should equal total issuance ({})", 
                      sum_of_balances, total_issuance);
        };
        
        // Initial state - empty vault
        let users = ["alice", "bob", "charlie"];
        verify_balance_invariant(&vault, &users);
        
        // After deposits
        vault.deposit("alice", "alice", 1000).unwrap();
        vault.deposit("bob", "bob", 2000).unwrap();
        vault.deposit("charlie", "charlie", 3000).unwrap();
        verify_balance_invariant(&vault, &users);
        
        // After yield accrual
        vault.set_block_height(vault.get_block_height() + 31_536_000); // 1 year
        vault.update_yield().unwrap();
        verify_balance_invariant(&vault, &users);
        
        // After partial redemption
        vault.redeem("alice", "alice", "alice", 500).unwrap();
        verify_balance_invariant(&vault, &users);
        
        // After full redemption by one user
        vault.redeem("bob", "bob", "bob", vault.get_token_balance("bob")).unwrap();
        verify_balance_invariant(&vault, &users);
    }
    
    #[test]
    fn test_concurrent_operations() {
        // Setup vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        
        // Initial deposit to establish baseline
        vault.deposit("genesis", "genesis", 1000).unwrap();
        
        // Simulate multiple users performing operations "concurrently"
        
        // Record starting state
        let initial_assets = vault.get_total_assets();
        let initial_issuance = vault.get_total_issuance();
        
        // Track all changes we expect
        let mut expected_asset_delta = 0i128;
        let mut expected_issuance_delta = 0i128;
        
        // User 1 deposits
        let user1_deposit = 500u128;
        let user1_tokens = vault.deposit("user1", "user1", user1_deposit).unwrap();
        expected_asset_delta += user1_deposit as i128;
        expected_issuance_delta += user1_tokens as i128;
        
        // User 2 deposits
        let user2_deposit = 1500u128;
        let user2_tokens = vault.deposit("user2", "user2", user2_deposit).unwrap();
        expected_asset_delta += user2_deposit as i128;
        expected_issuance_delta += user2_tokens as i128;
        
        // Genesis account redeems half
        let genesis_redeem = 500u128;
        let genesis_assets = vault.redeem("genesis", "genesis", "genesis", genesis_redeem).unwrap();
        expected_asset_delta -= genesis_assets as i128;
        expected_issuance_delta -= genesis_redeem as i128;
        
        // User 1 redeems all
        let user1_assets = vault.redeem("user1", "user1", "user1", user1_tokens).unwrap();
        expected_asset_delta -= user1_assets as i128;
        expected_issuance_delta -= user1_tokens as i128;
        
        // User 3 deposits
        let user3_deposit = 2000u128;
        let user3_tokens = vault.deposit("user3", "user3", user3_deposit).unwrap();
        expected_asset_delta += user3_deposit as i128;
        expected_issuance_delta += user3_tokens as i128;
        
        // Verify final state matches expected deltas
        let final_assets = vault.get_total_assets();
        let final_issuance = vault.get_total_issuance();
        
        assert_eq!(final_assets as i128 - initial_assets as i128, expected_asset_delta,
                  "Total assets should change by the expected delta");
                  
        assert_eq!(final_issuance as i128 - initial_issuance as i128, expected_issuance_delta,
                  "Total issuance should change by the expected delta");
    }
    
    #[test]
    fn test_extreme_values() {
        // Setup vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        
        // Test with very large assets (but not u128::MAX to avoid overflow)
        let large_amount = u128::MAX / 10;
        
        // Should handle large deposits safely
        let result = vault.deposit("whale", "whale", large_amount);
        assert!(result.is_ok(), "Large deposits should be handled safely");
        
        // Should handle large redemptions safely
        let tokens = result.unwrap();
        let redeem_result = vault.redeem("whale", "whale", "whale", tokens);
        assert!(redeem_result.is_ok(), "Large redemptions should be handled safely");
        
        // Verify the amounts match
        let redeemed_amount = redeem_result.unwrap();
        assert_eq!(redeemed_amount, large_amount, 
                  "Large deposit-redeem cycle should preserve value");
                  
        // Empty vault
        vault.set_value("total_assets", 0);
        vault.set_total_issuance(0);
        
        // Test with extremes for yield calculation
        vault.set_yield_rate(10000); // 100% annual rate
        vault.deposit("user", "user", 1000).unwrap();
        
        // Advance by maximum reasonable block height (100 years)
        vault.set_block_height(vault.get_block_height() + 31_536_000 * 100);
        
        // Should not overflow
        let result = vault.update_yield();
        assert!(result.is_ok(), "Extreme yield calculation should not overflow");
        
        // Assets should have grown significantly but finitely
        let assets = vault.get_total_assets();
        assert!(assets > 1000, "Assets should grow with yield");
        assert!(assets < u128::MAX / 2, "Assets should not approach overflow");
    }
    
    #[test]
    fn test_token_uniqueness_and_conservation() {
        // Setup vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        
        // First user deposits
        let alice_tokens = vault.deposit("alice", "alice", 1000).unwrap();
        
        // Cannot redeem more tokens than owned
        let over_redeem = vault.redeem("alice", "alice", "alice", alice_tokens + 1);
        assert!(over_redeem.is_err(), "Cannot redeem more tokens than owned");
        
        // Cannot redeem another user's tokens without holding them
        let result = vault.redeem("bob", "bob", "alice", 10);
        assert!(result.is_err(), 
                "Cannot redeem another user's tokens without holding them");
        
        // After redeeming, can't reuse the same tokens (double-spend)
        let half_tokens = alice_tokens / 2;
        vault.redeem("alice", "alice", "alice", half_tokens).unwrap();
        
        // Try to spend all original tokens (double-spend attempt)
        let double_spend = vault.redeem("alice", "alice", "alice", alice_tokens);
        assert!(double_spend.is_err(), "Cannot double-spend tokens");
        
        // Can only spend remaining tokens
        let remaining = vault.redeem("alice", "alice", "alice", half_tokens);
        assert!(remaining.is_ok(), "Can spend remaining tokens");
    }

    #[test]
    fn test_deposit_redeem_roundtrip_fairness() {
        // Setup vault with an initial deposit to create a non-1:1 ratio
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        
        // Initial ratio of 2:1 (assets:tokens)
        vault.deposit("genesis", "genesis", 2000).unwrap();
        vault.set_total_issuance(1000); // Manually adjust to 2:1 ratio
        
        // Deposit
        let deposit_amount = 100u128;
        let tokens = vault.deposit("user", "user", deposit_amount).unwrap();
        
        // Immediately redeem
        let assets = vault.redeem("user", "user", "user", tokens).unwrap();
        
        // Should get back close to original deposit amount
        // (May be off by 1 due to rounding)
        let difference = if deposit_amount > assets { 
            deposit_amount - assets 
        } else { 
            assets - deposit_amount 
        };
        
        assert!(difference <= 1, 
               "Deposit-redeem roundtrip should preserve value within 1 unit. Got difference of {}", 
               difference);
    }
    
    #[test]
    fn test_zero_yield_behavior() {
        // Setup vault with zero yield rate
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        vault.set_yield_rate(0); // 0% yield
        
        // Make deposit
        vault.deposit("user", "user", 1000).unwrap();
        
        // Advance time significantly
        vault.set_block_height(vault.get_block_height() + 31_536_000 * 10); // 10 years
        
        // Update yield
        vault.update_yield().unwrap();
        
        // Assets should remain unchanged
        assert_eq!(vault.get_total_assets(), 1000, 
                  "With 0% yield, assets should remain unchanged even as time passes");
    }
}
