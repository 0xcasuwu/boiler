//! WebAssembly-specific tests
//! These tests are designed to run with wasm-pack test

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use crate::YieldVault;
use crate::utils::Conversion;
use crate::simple_utils::{simple_share_calculation, simple_asset_calculation};

// Configure wasm_bindgen_test to use browser or node.js
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_default_constructor() {
    // Verify that we can create a vault instance in WASM
    let vault = YieldVault::default();
    assert!(true, "YieldVault instance created successfully in WebAssembly");
}

#[wasm_bindgen_test]
fn test_wasm_conversion_functions() {
    // Test conversion functions in WebAssembly
    let vault = YieldVault::default();
    
    // Test case 1: Empty vault
    let assets = 100;
    let total_assets = 0;
    let total_supply = 0;
    
    let shares = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
    assert_eq!(shares, assets, "Empty vault should convert 1:1");
    
    // Test case 2: Non-empty vault
    let assets = 100;
    let total_assets = 1000;
    let total_supply = 500;
    
    let shares = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
    assert_eq!(shares, 50, "100 assets should convert to 50 shares");
    
    let back_assets = vault.convert_shares_to_assets(shares, total_assets, total_supply).unwrap();
    assert_eq!(back_assets, assets, "Round trip conversion should preserve value");
}

#[wasm_bindgen_test]
fn test_wasm_simple_utils() {
    // Test simple_utils pure functions in WebAssembly
    
    // Empty vault case
    let assets = 100;
    let shares = simple_share_calculation(assets, 0, 0);
    assert_eq!(shares, assets, "Empty vault should convert 1:1");
    
    // Non-empty vault
    let assets = 100;
    let total_assets = 1000;
    let total_supply = 500;
    
    let shares = simple_share_calculation(assets, total_assets, total_supply);
    assert_eq!(shares, 50, "100 assets should convert to 50 shares");
    
    let back_assets = simple_asset_calculation(shares, total_assets, total_supply);
    assert_eq!(back_assets, assets, "Round trip conversion should preserve value");
}

#[wasm_bindgen_test]
fn test_wasm_preview_deposit() {
    // Test preview_deposit function in WebAssembly
    let vault = YieldVault::default();
    
    let result = vault.preview_deposit(100);
    assert!(result.is_ok(), "preview_deposit should succeed");
    assert_eq!(result.unwrap(), 100, "Empty vault should convert 1:1");
}
