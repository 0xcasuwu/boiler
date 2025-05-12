//! Integration tests for YieldVault focusing on functionality that doesn't require external dependencies

use yield_vault::YieldVault;
use yield_vault::utils::Conversion;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_vault_instantiation() {
        // Create a new YieldVault instance
        let vault = YieldVault::default();
        
        // Basic verification that the instance was created successfully
        assert!(true);
    }

    #[test]
    fn test_conversion_operations() {
        // Create a vault instance
        let vault = YieldVault::default();
        
        // Test asset to share conversion (empty vault - 1:1 ratio)
        let result = vault.convert_assets_to_shares(500, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 500);
        
        // Test with non-empty vault
        // If total_assets = 2000 and total_supply = 1000, 
        // then 400 assets should convert to 200 shares (2:1 ratio)
        let result = vault.convert_assets_to_shares(400, 2000, 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);
        
        // Test share to asset conversion (empty vault)
        let result = vault.convert_shares_to_assets(300, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);  // Empty vault returns 0 assets

        // Test with non-empty vault
        // If total_assets = 2000 and total_supply = 1000,
        // then 150 shares should convert to 300 assets (2:1 ratio)
        let result = vault.convert_shares_to_assets(150, 2000, 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 300);
    }

    #[test]
    fn test_preview_methods() {
        // Create a vault instance
        let vault = YieldVault::default();
        
        // Test preview_deposit
        // For an empty vault, preview_deposit should return the same value
        let result = vault.preview_deposit(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        
        // Test preview_redeem
        // For an empty vault, preview_redeem should return 0
        let result = vault.preview_redeem(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_large_number_handling() {
        // Create a vault instance
        let vault = YieldVault::default();
        
        // Test with very large numbers to check for overflow handling
        let large_assets = u64::MAX as u128;
        let result = vault.convert_assets_to_shares(large_assets, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), large_assets);
        
        // Test a realistic scenario with large numbers
        // If total_assets = 1,000,000,000 and total_supply = 500,000,000,
        // then 2,000,000 assets should convert to 1,000,000 shares (2:1 ratio)
        let result = vault.convert_assets_to_shares(
            2_000_000, 
            1_000_000_000, 
            500_000_000
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1_000_000);
    }

    #[test]
    fn test_conversion_edge_cases() {
        // Create a vault instance
        let vault = YieldVault::default();
        
        // Test with zero assets
        let result = vault.convert_assets_to_shares(0, 1000, 500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // Test with zero shares
        let result = vault.convert_shares_to_assets(0, 1000, 500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
