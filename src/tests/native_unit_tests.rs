//! Native unit tests that don't require WebAssembly
//! These tests will run with standard cargo test

#[cfg(test)]
mod tests {
    use crate::YieldVault;
    use crate::utils::Conversion;

    #[test]
    fn test_conversion_functions() {
        // Create a basic vault instance
        let vault = YieldVault::default();
        
        // Test converting assets to shares in an empty vault (should be 1:1)
        let result = vault.convert_assets_to_shares(100, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
        
        // Test converting shares to assets in an empty vault (should be 1:1)
        let result = vault.convert_shares_to_assets(50, 0, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
        
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
    fn test_preview_deposit() {
        let vault = YieldVault::default();
        
        // Test that preview_deposit works (this should work with no dependencies)
        let result = vault.preview_deposit(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn test_default_implementation() {
        // Test that the default implementation works
        let vault = YieldVault::default();
        
        // This is just testing that the default constructor works
        // and doesn't panic or return an invalid object
        assert!(vault.preview_deposit(100).is_ok());
    }
}
