//! Simple utility functions for testing that don't depend on WebAssembly/Alkanes imports

/// A simple function that can be tested on any platform
pub fn add(a: u128, b: u128) -> u128 {
    a + b
}

/// Calculate the average of two numbers
pub fn average(a: u128, b: u128) -> u128 {
    // Using checked_add to prevent overflow
    match a.checked_add(b) {
        Some(sum) => sum / 2,
        None => (a / 2) + (b / 2) + (a % 2 + b % 2) / 2,
    }
}

/// A more complex function that does share-asset calculations similar to the vault
pub fn simple_share_calculation(assets: u128, total_assets: u128, total_supply: u128) -> u128 {
    // Empty vault case - 1:1 ratio
    if total_assets == 0 || total_supply == 0 {
        return assets;
    }
    
    // Calculate shares based on the ratio of assets to total_assets
    assets.saturating_mul(total_supply) / total_assets
}

/// Convert shares back to assets
pub fn simple_asset_calculation(shares: u128, total_assets: u128, total_supply: u128) -> u128 {
    // Empty vault case - 1:1 ratio
    if total_assets == 0 || total_supply == 0 {
        return shares;
    }
    
    // Calculate assets based on the ratio of shares to total_supply
    shares.saturating_mul(total_assets) / total_supply
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
    
    #[test]
    fn test_average() {
        assert_eq!(average(4, 6), 5);
    }
    
    #[test]
    fn test_simple_share_calculation() {
        // Empty vault - 1:1 ratio
        assert_eq!(simple_share_calculation(100, 0, 0), 100);
        
        // Non-empty vault
        // If total_assets = 1000 and total_supply = 500,
        // then 100 assets should be 50 shares
        assert_eq!(simple_share_calculation(100, 1000, 500), 50);
    }
    
    #[test]
    fn test_simple_asset_calculation() {
        // Empty vault - 1:1 ratio
        assert_eq!(simple_asset_calculation(100, 0, 0), 100);
        
        // Non-empty vault
        // If total_assets = 1000 and total_supply = 500,
        // then 50 shares should be 100 assets
        assert_eq!(simple_asset_calculation(50, 1000, 500), 100);
    }
}
