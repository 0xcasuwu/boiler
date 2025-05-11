//! Native tests for the yield-vault functionality
//! These tests don't use WebAssembly and can be run with regular cargo test

mod tests {
    use yield_vault::YieldVault;
    use yield_vault::utils::Conversion;

    #[test]
    fn test_default_constructor() {
        // This test just verifies that we can create a YieldVault instance
        let _vault = YieldVault::default();
        
        // Simple verification that the instance was created successfully
        assert!(true);
    }

    #[test]
    fn test_conversion_functions() {
        // Create a basic vault instance
        let vault = YieldVault::default();
        
        // Test a simple conversion that doesn't require complex dependencies
        let assets = 100;
        let total_assets = 0;
        let total_supply = 0;
        
        // For an empty vault, shares should equal assets (1:1 ratio)
        let shares = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
        assert_eq!(shares, assets);
        
        // Test with non-zero values
        // If total_assets = 1000 and total_supply = 500, then:
        // 1 asset = 0.5 shares, and 1 share = 2 assets
        
        // 100 assets should convert to 50 shares
        let result = vault.convert_assets_to_shares(100, 1000, 500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
        
        // 50 shares should convert to 100 assets
        let result = vault.convert_shares_to_assets(50, 1000, 500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_preview_deposit_functionality() {
        // This test verifies the basic preview_deposit functionality
        let vault = YieldVault::default();
        
        // For an empty vault, preview_deposit should return the same value
        let result = vault.preview_deposit(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }
}
