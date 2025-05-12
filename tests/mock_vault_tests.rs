//! Tests using the MockYieldVault implementation instead of the real YieldVault
//! This avoids dependency issues with alkanes-runtime for testing purposes

use yield_vault::mock_vault::MockYieldVault;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_vault_initialization() {
        // Create a new vault instance
        let vault = MockYieldVault::default();
        
        // Initialize the vault with parameters
        let result = vault.initialize(
            "Bitcoin Yield Vault".to_string(),
            "bYV".to_string(),
            "Bitcoin".to_string(),
            "BTC".to_string(),
            8
        );
        
        assert!(result.is_ok());
        
        // Verify basic parameters
        assert_eq!(vault.get_name().unwrap(), "Bitcoin Yield Vault");
        assert_eq!(vault.get_symbol().unwrap(), "bYV");
        assert_eq!(vault.get_decimals().unwrap(), 8);
    }

    #[test]
    fn test_mock_vault_deposit_flow() {
        // Create a new vault instance
        let vault = MockYieldVault::default();
        
        // Initial state should be empty
        assert_eq!(vault.get_total_assets(), 0);
        assert_eq!(vault.get_total_supply(), 0);
        
        // Deposit 1000 assets from alice and verify shares received
        let shares = vault.deposit("alice", "alice", 1000).unwrap();
        assert_eq!(shares, 1000); // 1:1 ratio when empty
        
        // Check vault state
        assert_eq!(vault.get_total_assets(), 1000);
        assert_eq!(vault.get_total_supply(), 1000);
        assert_eq!(vault.get_balance("alice"), 1000);
        
        // Deposit 1000 more assets from bob
        let shares = vault.deposit("bob", "bob", 1000).unwrap();
        assert_eq!(shares, 1000); // Still 1:1 ratio at this point
        
        // Check final state
        assert_eq!(vault.get_total_assets(), 2000);
        assert_eq!(vault.get_total_supply(), 2000);
        assert_eq!(vault.get_balance("alice"), 1000);
        assert_eq!(vault.get_balance("bob"), 1000);
    }

    #[test]
    fn test_mock_vault_yield_effect() {
        // Create a vault with initial state: 
        // 1000 assets backing 500 shares (2:1 ratio)
        let vault = MockYieldVault::new(1000, 500);
        
        // Assign some shares to users
        vault.set_balance("alice", 300); // 60% of shares
        vault.set_balance("bob", 200);   // 40% of shares
        
        // Simulate yield accrual: total assets increased to 1200
        // while supply remains the same
        vault.set_total_assets(1200);
        
        // Now we should have a 1200:500 ratio (2.4:1)
        
        // If alice redeems all her shares, she should get 60% of 1200 = 720 assets
        let assets = vault.redeem("alice", "alice", "alice", 300).unwrap();
        assert_eq!(assets, 720);
        
        // Check vault state
        assert_eq!(vault.get_total_assets(), 480);  // 1200 - 720 = 480
        assert_eq!(vault.get_total_supply(), 200);  // 500 - 300 = 200
        assert_eq!(vault.get_balance("alice"), 0);
        assert_eq!(vault.get_balance("bob"), 200);
        
        // If bob redeems all his shares, he should get all remaining assets
        let assets = vault.redeem("bob", "bob", "bob", 200).unwrap();
        assert_eq!(assets, 480);
        
        // Check final state - vault is empty
        assert_eq!(vault.get_total_assets(), 0);
        assert_eq!(vault.get_total_supply(), 0);
        assert_eq!(vault.get_balance("bob"), 0);
    }

    #[test]
    fn test_mock_vault_preview_methods() {
        // Create a vault with initial state
        let vault = MockYieldVault::new(2000, 1000); // 2:1 ratio
        
        // Preview depositing 100 assets
        let shares = vault.preview_deposit(100).unwrap();
        assert_eq!(shares, 50); // At 2:1 ratio, 100 assets = 50 shares
        
        // Preview redeeming 200 shares
        let assets = vault.preview_redeem(200).unwrap();
        assert_eq!(assets, 400); // At 2:1 ratio, 200 shares = 400 assets
        
        // Verify calling preview doesn't change state
        assert_eq!(vault.get_total_assets(), 2000);
        assert_eq!(vault.get_total_supply(), 1000);
    }

    #[test]
    fn test_mock_vault_complex_scenario() {
        // Start with an empty vault
        let vault = MockYieldVault::default();
        
        // Step 1: Alice deposits 1000 assets
        let alice_shares = vault.deposit("alice", "alice", 1000).unwrap();
        assert_eq!(alice_shares, 1000); // 1:1 ratio when empty
        
        // Step 2: Simulate yield accrual (assets grow to 1100)
        vault.set_total_assets(1100);
        
        // Step 3: Bob deposits 1100 assets
        let bob_shares = vault.deposit("bob", "bob", 1100).unwrap();
        // Should get 1000 shares: (1100 * 1000) / 1100 = 1000
        assert_eq!(bob_shares, 1000);
        
        // State: 2200 assets, 2000 shares, ratio is 2200:2000 = 1.1:1
        assert_eq!(vault.get_total_assets(), 2200);
        assert_eq!(vault.get_total_supply(), 2000);
        
        // Step 4: More yield accrual (assets grow to 2420)
        vault.set_total_assets(2420);
        
        // Step 5: Charlie deposits 1210 assets
        let charlie_shares = vault.deposit("charlie", "charlie", 1210).unwrap();
        // Should get 1000 shares: (1210 * 2000) / 2420 = 1000
        assert_eq!(charlie_shares, 1000);
        
        // Final state: 3630 assets, 3000 shares, ratio is 3630:3000 = 1.21:1
        assert_eq!(vault.get_total_assets(), 3630);
        assert_eq!(vault.get_total_supply(), 3000);
        assert_eq!(vault.get_balance("alice"), 1000);
        assert_eq!(vault.get_balance("bob"), 1000);
        assert_eq!(vault.get_balance("charlie"), 1000);
        
        // All users have equal shares (1000 each), but they'll get different amounts if they redeem
        // due to the yield that accrued at different times
    }
}
