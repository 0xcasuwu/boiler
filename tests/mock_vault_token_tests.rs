//! Token-based MockYieldVault implementation tests
//! These tests focus on the token-based authorization model and yield accrual over time

use yield_vault::mock_vault::MockYieldVault;

#[cfg(test)]
mod token_based_tests {
    use super::*;
    
    #[test]
    fn test_token_based_authorization() {
        // Setup vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        
        // User deposits assets and receives tokens
        let alice_tokens = vault.deposit("tx_context", "alice", 1000).unwrap();
        assert_eq!(alice_tokens, 1000);
        
        // Verify token issuance
        assert_eq!(vault.get_token_balance("alice"), 1000);
        
        // Anyone with alice's token ID in the transaction context can redeem
        // This simulates a transaction where the caller possesses alice's tokens
        let result = vault.redeem("mallory", "mallory", "alice", 500).unwrap();
        
        // Redemption succeeds because the tokens are in the context
        assert_eq!(result, 500);
        
        // Alice's token balance is reduced
        assert_eq!(vault.get_token_balance("alice"), 500);
    }
    
    #[test]
    fn test_yield_accrual_over_time() {
        // Setup vault with initial deposits
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        vault.set_yield_rate(1000); // 10% annual yield
        
        // Two users deposit equal amounts
        vault.deposit("tx1", "alice", 1000).unwrap();
        vault.deposit("tx2", "bob", 1000).unwrap();
        
        // Initial state: 2000 assets, 2000 tokens
        assert_eq!(vault.get_total_assets(), 2000);
        assert_eq!(vault.get_total_issuance(), 2000);
        
        // Advance block height by approximately 3 months
        // (31536000 seconds in a year / 4 ≈ 7,884,000 seconds)
        vault.set_block_height(vault.get_block_height() + 7_884_000);
        
        // Alice redeems half her tokens
        let alice_assets = vault.redeem("tx3", "alice", "alice", 500).unwrap();
        
        // Alice should receive half her tokens plus ~2.5% yield (≈ 512.5)
        // Integer division might result in exactly 512
        assert!(alice_assets >= 512 && alice_assets < 515, 
                "Expected ~512.5 assets after 3 months yield, got {}", alice_assets);
        
        // Advance block height by another 3 months
        vault.set_block_height(vault.get_block_height() + 7_884_000);
        
        // Bob redeems all his tokens
        let bob_assets = vault.redeem("tx4", "bob", "bob", 1000).unwrap();
        
        // Bob should receive his tokens plus ~5% yield (≈ 1050)
        assert!(bob_assets > 1049 && bob_assets < 1051, 
                "Expected ~1050 assets after 6 months yield, got {}", bob_assets);
        
        // Total assets should only include Alice's remaining share plus accrued yield
        let total_assets = vault.get_total_assets();
        let total_issuance = vault.get_total_issuance();
        
        assert_eq!(total_issuance, 500); // Alice's remaining tokens
        assert!(total_assets > 525 && total_assets < 530,
                "Expected ~525-530 total assets remaining, got {}", total_assets);
    }
    
    #[test]
    fn test_yield_impact_on_share_price() {
        // Setup vault with initial deposit
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        vault.set_yield_rate(1000); // 10% annual yield
        
        // Alice makes initial deposit
        vault.deposit("tx1", "alice", 1000).unwrap();
        
        // Initial state: 1000 assets, 1000 tokens (1:1 ratio)
        
        // Advance block height by one year
        vault.set_block_height(vault.get_block_height() + 31_536_000);
        
        // Force yield update
        vault.update_yield().unwrap();
        
        // Total assets should be ~1100 (initial + 10% yield)
        let total_assets = vault.get_total_assets();
        assert!(total_assets >= 1099 && total_assets <= 1101,
                "Expected ~1100 assets after a year, got {}", total_assets);
        
        // Total issuance unchanged at 1000 tokens
        assert_eq!(vault.get_total_issuance(), 1000);
        
        // Bob now deposits the same amount as Alice initially did
        let bob_tokens = vault.deposit("tx2", "bob", 1000).unwrap();
        
        // Bob should get fewer tokens due to the increased share price
        // With 1:1.1 ratio, 1000 assets should get ~909 tokens
        assert!(bob_tokens >= 908 && bob_tokens <= 910,
                "Expected ~909 tokens for 1000 assets at 1:1.1 ratio, got {}", bob_tokens);
        
        // Final state check
        // Total assets: ~2100 (Alice's 1000 + yield 100 + Bob's 1000)
        // Total tokens: ~1909 (Alice's 1000 + Bob's ~909)
        assert!(vault.get_total_assets() >= 2099 && vault.get_total_assets() <= 2101);
        assert!(vault.get_total_issuance() >= 1908 && vault.get_total_issuance() <= 1910);
    }
    
    #[test]
    fn test_yield_manipulation_attack() {
        // Setup vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        vault.set_yield_rate(1000); // 10% annual yield
        
        // Alice deposits
        vault.deposit("tx1", "alice", 1000).unwrap();
        
        // Attacker tries to manipulate time/blocks to extract yield unfairly
        
        // Try to set block height to the past
        let initial_height = vault.get_block_height();
        vault.set_block_height(initial_height - 1000);
        
        // Update yield - should be safe with no additional yield
        vault.update_yield().unwrap();
        
        // Total assets should be unchanged
        assert_eq!(vault.get_total_assets(), 1000);
        
        // Now try extreme future block height
        vault.set_block_height(initial_height + 31_536_000 * 100); // 100 years
        
        // Update yield - should accrue yield safely without overflow
        vault.update_yield().unwrap();
        
        // Total assets should have increased significantly but reasonably
        let total_assets = vault.get_total_assets();
        assert!(total_assets > 1000, "Yield should increase assets over time");
        assert!(total_assets < u128::MAX / 2, "Yield shouldn't cause overflow");
        
        // Yield calculation should be sensible (roughly expected maximum 100x initial for 100% per year over 100 years)
        assert!(total_assets <= 1000 * 100,
                "Yield over 100 years shouldn't exceed 100x principal");
    }
    
    #[test]
    fn test_first_deposit_after_time() {
        // Setup empty vault
        let vault = MockYieldVault::default();
        vault.initialize("Test Vault".to_string(), "TVT".to_string(), "Test Asset".to_string(), "ASSET".to_string(), 8).unwrap();
        vault.set_yield_rate(1000); // 10% annual yield
        
        // Advance block height significantly
        vault.set_block_height(vault.get_block_height() + 31_536_000 * 10); // 10 years
        
        // First deposit after time has passed
        let tokens = vault.deposit("tx1", "alice", 1000).unwrap();
        
        // Should get 1:1 ratio for first deposit, regardless of time passed
        assert_eq!(tokens, 1000);
        
        // Yield rate should still apply going forward
        vault.set_block_height(vault.get_block_height() + 31_536_000); // 1 more year
        
        // Update yield
        vault.update_yield().unwrap();
        
        // Should see ~10% increase
        assert!(vault.get_total_assets() >= 1099 && vault.get_total_assets() <= 1101);
    }
}
