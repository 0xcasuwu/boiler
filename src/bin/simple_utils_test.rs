//! Standalone test program for simple_utils
//!
//! This binary runs tests for the simple_utils module
//! without requiring WebAssembly execution

use yield_vault::simple_utils::{add, average, simple_share_calculation, simple_asset_calculation};

fn main() {
    println!("Running simple_utils tests...");
    println!("=============================");
    
    // Test add function
    test_add();
    
    // Test average function
    test_average();
    
    // Test simple_share_calculation function
    test_simple_share_calculation();
    
    // Test simple_asset_calculation function
    test_simple_asset_calculation();
    
    // Test conversion round-trip
    test_conversion_round_trip();

    println!("=============================");
    println!("All tests PASSED! ✓");
}

fn test_add() {
    println!("Testing add function:");
    
    // Test case 1: Small numbers
    let a = 5;
    let b = 7;
    let expected = 12;
    let result = add(a, b);
    println!("  add({}, {}) = {} (expected: {})", a, b, result, expected);
    assert_eq!(result, expected);
    
    // Test case 2: Zero
    let a = 0;
    let b = 10;
    let expected = 10;
    let result = add(a, b);
    println!("  add({}, {}) = {} (expected: {})", a, b, result, expected);
    assert_eq!(result, expected);
    
    // Test case 3: Large numbers
    let a = 1_000_000_000;
    let b = 2_000_000_000;
    let expected = 3_000_000_000;
    let result = add(a, b);
    println!("  add({}, {}) = {} (expected: {})", a, b, result, expected);
    assert_eq!(result, expected);
    
    println!("  ✓ add tests passed");
}

fn test_average() {
    println!("Testing average function:");
    
    // Test case 1: Even average
    let a = 10;
    let b = 20;
    let expected = 15;
    let result = average(a, b);
    println!("  average({}, {}) = {} (expected: {})", a, b, result, expected);
    assert_eq!(result, expected);
    
    // Test case 2: Odd average
    let a = 5;
    let b = 8;
    let expected = 6;  // Integer division rounds down
    let result = average(a, b);
    println!("  average({}, {}) = {} (expected: {})", a, b, result, expected);
    assert_eq!(result, expected);
    
    // Test case 3: Large numbers
    let a = 1_000_000_000_000;
    let b = 3_000_000_000_000;
    let expected = 2_000_000_000_000;
    let result = average(a, b);
    println!("  average({}, {}) = {} (expected: {})", a, b, result, expected);
    assert_eq!(result, expected);
    
    println!("  ✓ average tests passed");
}

fn test_simple_share_calculation() {
    println!("Testing simple_share_calculation function:");
    
    // Test case 1: Empty vault (1:1 ratio)
    let assets = 100;
    let total_assets = 0;
    let total_supply = 0;
    let expected = 100;
    let result = simple_share_calculation(assets, total_assets, total_supply);
    println!("  simple_share_calculation({}, {}, {}) = {} (expected: {})", 
        assets, total_assets, total_supply, result, expected);
    assert_eq!(result, expected);
    
    // Test case 2: Non-empty vault
    // If total_assets = 1000 and total_supply = 500,
    // then 100 assets should be 50 shares
    let assets = 100;
    let total_assets = 1000;
    let total_supply = 500;
    let expected = 50;
    let result = simple_share_calculation(assets, total_assets, total_supply);
    println!("  simple_share_calculation({}, {}, {}) = {} (expected: {})", 
        assets, total_assets, total_supply, result, expected);
    assert_eq!(result, expected);
    
    // Test case 3: Different ratio
    // If total_assets = 800 and total_supply = 200,
    // then 80 assets should be 20 shares
    let assets = 80;
    let total_assets = 800;
    let total_supply = 200;
    let expected = 20;
    let result = simple_share_calculation(assets, total_assets, total_supply);
    println!("  simple_share_calculation({}, {}, {}) = {} (expected: {})", 
        assets, total_assets, total_supply, result, expected);
    assert_eq!(result, expected);
    
    println!("  ✓ simple_share_calculation tests passed");
}

fn test_simple_asset_calculation() {
    println!("Testing simple_asset_calculation function:");
    
    // Test case 1: Empty vault (1:1 ratio)
    let shares = 100;
    let total_assets = 0;
    let total_supply = 0;
    let expected = 100;
    let result = simple_asset_calculation(shares, total_assets, total_supply);
    println!("  simple_asset_calculation({}, {}, {}) = {} (expected: {})", 
        shares, total_assets, total_supply, result, expected);
    assert_eq!(result, expected);
    
    // Test case 2: Non-empty vault
    // If total_assets = 1000 and total_supply = 500,
    // then 50 shares should be 100 assets
    let shares = 50;
    let total_assets = 1000;
    let total_supply = 500;
    let expected = 100;
    let result = simple_asset_calculation(shares, total_assets, total_supply);
    println!("  simple_asset_calculation({}, {}, {}) = {} (expected: {})", 
        shares, total_assets, total_supply, result, expected);
    assert_eq!(result, expected);
    
    // Test case 3: Different ratio
    // If total_assets = 800 and total_supply = 200,
    // then 20 shares should be 80 assets
    let shares = 20;
    let total_assets = 800;
    let total_supply = 200;
    let expected = 80;
    let result = simple_asset_calculation(shares, total_assets, total_supply);
    println!("  simple_asset_calculation({}, {}, {}) = {} (expected: {})", 
        shares, total_assets, total_supply, result, expected);
    assert_eq!(result, expected);
    
    println!("  ✓ simple_asset_calculation tests passed");
}

fn test_conversion_round_trip() {
    println!("Testing round-trip conversions:");
    
    let total_assets = 1000;
    let total_supply = 500;
    
    // Test case 1: Round trip assets -> shares -> assets
    let original_assets = 100;
    let shares = simple_share_calculation(original_assets, total_assets, total_supply);
    let assets_back = simple_asset_calculation(shares, total_assets, total_supply);
    
    println!("  Round-trip: {} assets -> {} shares -> {} assets", 
        original_assets, shares, assets_back);
    assert_eq!(original_assets, assets_back);
    
    // Test case 2: Round trip shares -> assets -> shares
    let original_shares = 50;
    let assets = simple_asset_calculation(original_shares, total_assets, total_supply);
    let shares_back = simple_share_calculation(assets, total_assets, total_supply);
    
    println!("  Round-trip: {} shares -> {} assets -> {} shares", 
        original_shares, assets, shares_back);
    assert_eq!(original_shares, shares_back);
    
    println!("  ✓ Round-trip conversion tests passed");
}
