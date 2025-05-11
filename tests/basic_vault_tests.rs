//! Basic tests for the yield-vault functionality
//! These tests don't depend on WebAssembly features and can be run with regular cargo test

// Make sure this file only runs in non-wasm environments
#![cfg(not(target_arch = "wasm32"))]

use yield_vault::YieldVault;
use yield_vault::utils::Conversion;
use yield_vault::simple_utils::{simple_share_calculation, simple_asset_calculation, add, average};

// BASIC CONSTRUCTOR TESTS

#[test]
fn test_default_constructor() {
    // This test just verifies that we can create a YieldVault instance
    let vault = YieldVault::default();
    println!("Test: YieldVault default constructor");
    // Simple verification that the instance was created successfully
    assert!(true);
}

// CONVERSION TESTS

#[test]
fn test_conversion_functions() {
    // Create a basic vault instance
    let vault = YieldVault::default();
    println!("Test: Basic conversion functions");
    
    // Test a simple conversion that doesn't require complex dependencies
    let assets = 100;
    let total_assets = 0;
    let total_supply = 0;
    
    // For an empty vault, shares should equal assets (1:1 ratio)
    let shares = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
    println!("  Empty vault: {} assets = {} shares", assets, shares);
    assert_eq!(shares, assets);
    
    // Test with non-zero values
    // If total_assets = 1000 and total_supply = 500, then:
    // 1 asset = 0.5 shares, and 1 share = 2 assets
    
    // 100 assets should convert to 50 shares
    let result = vault.convert_assets_to_shares(100, 1000, 500);
    assert!(result.is_ok());
    println!("  Non-empty vault: 100 assets = {} shares", result.unwrap());
    assert_eq!(result.unwrap(), 50);
    
    // 50 shares should convert to 100 assets
    let result = vault.convert_shares_to_assets(50, 1000, 500);
    assert!(result.is_ok());
    println!("  Non-empty vault: 50 shares = {} assets", result.unwrap());
    assert_eq!(result.unwrap(), 100);
}

#[test]
fn test_preview_deposit_functionality() {
    // This test verifies the basic preview_deposit functionality
    let vault = YieldVault::default();
    println!("Test: Preview deposit functionality");
    
    // For an empty vault, preview_deposit should return the same value
    let result = vault.preview_deposit(100);
    assert!(result.is_ok());
    println!("  preview_deposit(100) = {}", result.unwrap());
    assert_eq!(result.unwrap(), 100);
}

// COMPARISON TESTS

#[test]
fn test_compare_with_simple_utils() {
    // Compare results from YieldVault with our simple_utils implementation
    let vault = YieldVault::default();
    println!("Test: Compare vault conversion with simple_utils");
    
    // Test case 1: Empty vault
    let assets = 150;
    let total_assets = 0;
    let total_supply = 0;
    
    let vault_result = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
    let simple_result = simple_share_calculation(assets, total_assets, total_supply);
    println!("  Empty vault: vault={}, simple={}", vault_result, simple_result);
    assert_eq!(vault_result, simple_result);
    
    // Test case 2: Non-empty vault
    let assets = 300;
    let total_assets = 2000;
    let total_supply = 1000;
    
    let vault_result = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
    let simple_result = simple_share_calculation(assets, total_assets, total_supply);
    println!("  Non-empty vault: vault={}, simple={}", vault_result, simple_result);
    assert_eq!(vault_result, simple_result);
    
    // Test case 3: Shares to assets
    let shares = 75;
    let vault_result = vault.convert_shares_to_assets(shares, total_assets, total_supply).unwrap();
    let simple_result = simple_asset_calculation(shares, total_assets, total_supply);
    println!("  Shares to assets: vault={}, simple={}", vault_result, simple_result);
    assert_eq!(vault_result, simple_result);
}

// EDGE CASES AND SPECIAL SCENARIOS

#[test]
fn test_zero_values() {
    let vault = YieldVault::default();
    println!("Test: Zero values");
    
    // Zero assets
    let assets = 0;
    let total_assets = 1000;
    let total_supply = 500;
    
    let result = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
    println!("  0 assets = {} shares", result);
    assert_eq!(result, 0);
    
    // Zero shares
    let shares = 0;
    let result = vault.convert_shares_to_assets(shares, total_assets, total_supply).unwrap();
    println!("  0 shares = {} assets", result);
    assert_eq!(result, 0);
}

#[test]
fn test_large_values() {
    let vault = YieldVault::default();
    println!("Test: Large values");
    
    // Very large values
    let assets = 10_000_000_000_000;
    let total_assets = 100_000_000_000_000;
    let total_supply = 50_000_000_000_000;
    
    let result = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
    println!("  {} assets = {} shares", assets, result);
    assert_eq!(result, 5_000_000_000_000);
    
    // Compare with simple implementation
    let simple_result = simple_share_calculation(assets, total_assets, total_supply);
    assert_eq!(result, simple_result);
}

// UTILITY FUNCTION TESTS

#[test]
fn test_simple_utils_functions() {
    println!("Test: Simple utility functions");
    
    // Test add function
    let a = 5;
    let b = 7;
    let result = add(a, b);
    println!("  add({}, {}) = {}", a, b, result);
    assert_eq!(result, 12);
    
    // Test average function
    let a = 10;
    let b = 20;
    let result = average(a, b);
    println!("  average({}, {}) = {}", a, b, result);
    assert_eq!(result, 15);
    
    // Test with large values
    let a = 1_000_000_000_000;
    let b = 3_000_000_000_000;
    let result = average(a, b);
    println!("  average({}, {}) = {}", a, b, result);
    assert_eq!(result, 2_000_000_000_000);
}

// COMPLEX SCENARIOS

#[test]
fn test_conversion_sequences() {
    let vault = YieldVault::default();
    println!("Test: Conversion sequences");
    
    // Start with 1000 assets and 500 shares
    let total_assets = 1000;
    let total_supply = 500;
    
    // Convert 100 assets to shares
    let shares = vault.convert_assets_to_shares(100, total_assets, total_supply).unwrap();
    println!("  Step 1: 100 assets -> {} shares", shares);
    assert_eq!(shares, 50);
    
    // Then convert those shares back to assets
    let assets_back = vault.convert_shares_to_assets(shares, total_assets, total_supply).unwrap();
    println!("  Step 2: {} shares -> {} assets", shares, assets_back);
    assert_eq!(assets_back, 100);
    
    // Verify no loss in the round-trip conversion
    println!("  Round-trip conversion: 100 -> {} -> {}", shares, assets_back);
    assert_eq!(assets_back, 100);
}
