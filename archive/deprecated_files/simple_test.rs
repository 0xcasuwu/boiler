//! Standalone test utility that only uses the simple_utils module
//! This file is separate from the main yield-vault codebase to avoid WebAssembly dependencies

pub fn main() {
    // Define test values for conversion calculations
    let test_values = vec![
        (100, 0, 0),     // Empty vault - 1:1 ratio
        (100, 1000, 500), // Standard case: 1 asset = 0.5 shares
        (80, 800, 200),   // Different ratio: 1 asset = 0.25 shares
        (0, 1000, 500),   // Zero assets
        (10000, 10000, 10000), // 1:1 ratio with non-zero values
    ];
    
    println!("===== Simple Share/Asset Conversion Calculator =====");
    println!("This utility demonstrates the core math behind the yield vault");
    println!();
    
    // Test asset-to-share conversions
    println!("ASSETS TO SHARES CONVERSIONS:");
    println!("{:<15} {:<15} {:<15} {:<15}", "Assets", "Total Assets", "Total Supply", "Shares");
    println!("--------------------------------------------------------");
    
    for (assets, total_assets, total_supply) in &test_values {
        let shares = simple_share_calculation(*assets, *total_assets, *total_supply);
        println!("{:<15} {:<15} {:<15} {:<15}", assets, total_assets, total_supply, shares);
    }
    
    println!();
    
    // Test share-to-asset conversions
    println!("SHARES TO ASSETS CONVERSIONS:");
    println!("{:<15} {:<15} {:<15} {:<15}", "Shares", "Total Assets", "Total Supply", "Assets");
    println!("--------------------------------------------------------");
    
    for (shares, total_assets, total_supply) in &test_values {
        let assets = simple_asset_calculation(*shares, *total_assets, *total_supply);
        println!("{:<15} {:<15} {:<15} {:<15}", shares, total_assets, total_supply, assets);
    }
    
    println!();
    
    // Test round-trip conversions
    println!("ROUND-TRIP CONVERSION TESTS:");
    println!("Starting with 100 assets in a vault with 1000 total assets and 500 total shares:");
    let start_assets = 100;
    let total_assets = 1000;
    let total_supply = 500;
    
    let shares = simple_share_calculation(start_assets, total_assets, total_supply);
    let assets_back = simple_asset_calculation(shares, total_assets, total_supply);
    
    println!("  100 assets -> {} shares -> {} assets", shares, assets_back);
    println!("  Result: {}", if start_assets == assets_back { "PASSED ✓" } else { "FAILED ✗" });
    
    println!();
    println!("All calculations executed successfully!");
    println!("=================================================");
}

/// Calculate shares from assets based on vault state
pub fn simple_share_calculation(assets: u128, total_assets: u128, total_supply: u128) -> u128 {
    // Empty vault case - 1:1 ratio
    if total_assets == 0 || total_supply == 0 {
        return assets;
    }
    
    // Calculate shares based on the ratio of assets to total_assets
    assets.saturating_mul(total_supply) / total_assets
}

/// Calculate assets from shares based on vault state
pub fn simple_asset_calculation(shares: u128, total_assets: u128, total_supply: u128) -> u128 {
    // Empty vault case - 1:1 ratio
    if total_assets == 0 || total_supply == 0 {
        return shares;
    }
    
    // Calculate assets based on the ratio of shares to total_supply
    shares.saturating_mul(total_assets) / total_supply
}
