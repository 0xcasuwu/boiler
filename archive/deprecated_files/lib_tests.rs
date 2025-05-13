//! Basic tests that don't require WebAssembly features
//! These tests can run with regular cargo test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_conversion() {
        // Create a basic vault instance
        let vault = YieldVault::default();
        
        // Test a simple conversion that doesn't require complex dependencies
        let assets = 100;
        let total_assets = 0;
        let total_supply = 0;
        
        // For an empty vault, shares should equal assets (1:1 ratio)
        let shares = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
        assert_eq!(shares, assets);
    }

    #[test]
    fn test_default_constructor() {
        // This test just verifies that we can create a YieldVault instance
        let vault = YieldVault::default();
        
        // Simple verification that the instance was created successfully
        assert!(true);
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
