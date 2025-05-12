// Simple standalone test runner for core vault conversion functions
// This file doesn't depend on any WebAssembly or external libraries

fn main() {
    println!("===== Simple Share/Asset Conversion Calculator =====");
    println!("Running basic vault conversion tests...");
    println!();
    
    // Test simple share calculation
    test_simple_share_calculation();
    
    // Test simple asset calculation
    test_simple_asset_calculation();
    
    // Test round trip conversions
    test_round_trip_conversions();
    
    // Additional math utility tests
    test_add();
    test_average();
    
    println!();
    println!("All tests completed successfully! ✓");
    println!("=================================================");
}

fn test_simple_share_calculation() {
    println!("Testing simple_share_calculation:");
    
    // Test cases: (assets, total_assets, total_supply) -> expected shares
    let test_cases = [
        // Empty vault - 1:1 ratio
        (100, 0, 0, 100),
        // Standard case: 100 assets in a vault with 1000 assets and 500 shares
        (100, 1000, 500, 50),
        // Different ratio: 80 assets in a vault with 800 assets and 200 shares
        (80, 800, 200, 20),
        // Zero assets
        (0, 1000, 500, 0),
        // 1:1 ratio with non-zero values
        (100, 100, 100, 100),
    ];
    
    for (assets, total_assets, total_supply, expected) in test_cases {
        let shares = simple_share_calculation(assets, total_assets, total_supply);
        println!("  {} assets -> {} shares (expected: {})", assets, shares, expected);
        assert_eq!(shares, expected);
    }
    println!("  ✓ All share calculations passed");
}

fn test_simple_asset_calculation() {
    println!("Testing simple_asset_calculation:");
    
    // Test cases: (shares, total_assets, total_supply) -> expected assets
    let test_cases = [
        // Empty vault - 1:1 ratio
        (100, 0, 0, 100),
        // Standard case: 50 shares in a vault with 1000 assets and 500 shares
        (50, 1000, 500, 100),
        // Different ratio: 20 shares in a vault with 800 assets and 200 shares
        (20, 800, 200, 80),
        // Zero shares
        (0, 1000, 500, 0),
        // 1:1 ratio with non-zero values
        (100, 100, 100, 100),
    ];
    
    for (shares, total_assets, total_supply, expected) in test_cases {
        let assets = simple_asset_calculation(shares, total_assets, total_supply);
        println!("  {} shares -> {} assets (expected: {})", shares, assets, expected);
        assert_eq!(assets, expected);
    }
    println!("  ✓ All asset calculations passed");
}

fn test_round_trip_conversions() {
    println!("Testing round-trip conversions:");
    
    // Test cases: different vault states (total_assets, total_supply)
    let vault_states = [
        (1000, 500),   // 2:1 ratio
        (800, 200),    // 4:1 ratio
        (100, 100),    // 1:1 ratio
    ];
    
    for (total_assets, total_supply) in vault_states {
        // Test assets -> shares -> assets
        let original_assets = 100;
        let shares = simple_share_calculation(original_assets, total_assets, total_supply);
        let assets_back = simple_asset_calculation(shares, total_assets, total_supply);
        
        println!("  Vault [{}/{}]: {} assets -> {} shares -> {} assets", 
            total_assets, total_supply, original_assets, shares, assets_back);
        assert_eq!(original_assets, assets_back);
    }
    println!("  ✓ All round-trip conversions passed");
}

fn test_add() {
    println!("Testing add function:");
    
    // Test cases
    let test_cases = [
        (5, 7, 12),
        (0, 10, 10),
        (1000000000, 2000000000, 3000000000),
    ];
    
    for (a, b, expected) in test_cases {
        let result = add(a, b);
        println!("  add({}, {}) = {} (expected: {})", a, b, result, expected);
        assert_eq!(result, expected);
    }
    println!("  ✓ All addition tests passed");
}

fn test_average() {
    println!("Testing average function:");
    
    // Test cases
    let test_cases = [
        (10, 20, 15),
        (5, 7, 6),  // Integer division rounds down
        (1000000000000, 3000000000000, 2000000000000),
    ];
    
    for (a, b, expected) in test_cases {
        let result = average(a, b);
        println!("  average({}, {}) = {} (expected: {})", a, b, result, expected);
        assert_eq!(result, expected);
    }
    println!("  ✓ All average tests passed");
}

// Function implementations from simple_utils.rs

/// Calculate shares from assets based on vault state
fn simple_share_calculation(assets: u128, total_assets: u128, total_supply: u128) -> u128 {
    // Empty vault case - 1:1 ratio
    if total_assets == 0 || total_supply == 0 {
        return assets;
    }
    
    // Calculate shares based on the ratio of assets to total_assets
    assets.saturating_mul(total_supply) / total_assets
}

/// Calculate assets from shares based on vault state
fn simple_asset_calculation(shares: u128, total_assets: u128, total_supply: u128) -> u128 {
    // Empty vault case - 1:1 ratio
    if total_assets == 0 || total_supply == 0 {
        return shares;
    }
    
    // Calculate assets based on the ratio of shares to total_supply
    shares.saturating_mul(total_assets) / total_supply
}

/// Add two numbers with saturation
fn add(a: u128, b: u128) -> u128 {
    a.saturating_add(b)
}

/// Calculate the average of two numbers
fn average(a: u128, b: u128) -> u128 {
    // This is a simple average function that prevents overflow
    // by using checked operations
    a.saturating_add(b) / 2
}
