//! Simple utility tests that don't depend on external runtime functions

#[cfg(test)]
mod tests {
    use yield_vault::simple_utils::{simple_share_calculation, simple_asset_calculation};
    
    #[test]
    fn test_simple_share_calculation() {
        // For empty vault (total_assets=0, total_supply=0), shares = assets
        let shares = simple_share_calculation(100, 0, 0);
        assert_eq!(shares, 100);
        
        // For non-empty vault with 2:1 ratio (1000 assets, 500 shares)
        // 100 assets should convert to 50 shares
        let shares = simple_share_calculation(100, 1000, 500);
        assert_eq!(shares, 50);
    }
    
    #[test]
    fn test_simple_asset_calculation() {
        // For empty vault, shares are returned as-is (1:1 ratio)
        let assets = simple_asset_calculation(100, 0, 0);
        assert_eq!(assets, 100);
        
        // For non-empty vault with 2:1 ratio (1000 assets, 500 shares)
        // 50 shares should convert to 100 assets
        let assets = simple_asset_calculation(50, 1000, 500);
        assert_eq!(assets, 100);
    }
}
