use crate::YieldVault;
use crate::storage::Storage;
use crate::security::Security;
use alkanes_support::context::Context;
use alkanes_support::response::CallResponse;
use anyhow::Result;
use metashrew_support::index_pointer::KeyValuePointer;

/// Handle test initialization during unit tests
/// 
/// This function now accepts an immutable reference to match our MessageDispatch implementation
pub fn handle_test_initialize(vault: &YieldVault, args: &[u8]) -> Result<CallResponse> {
    // Create a dummy response
    let response = CallResponse::default();
    
    // Parse the initialization parameters
    let mut args_iter = args.split(|&b| b == 0);
    let name = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
    let symbol = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
    let asset_name = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
    let asset_symbol = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
    let decimal_offset = 8u8; // Default to 8 decimals for Bitcoin
    
    // Initialize the vault
    vault.observe_initialization().unwrap();
    
    // Store basic token metadata
    vault.name_pointer().set(std::sync::Arc::new(name.as_bytes().to_vec()));
    vault.symbol_pointer().set(std::sync::Arc::new(symbol.as_bytes().to_vec()));
    vault.asset_name_pointer().set(std::sync::Arc::new(asset_name.as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(std::sync::Arc::new(asset_symbol.as_bytes().to_vec()));
    vault.decimals_pointer().set_value(decimal_offset);
    
    // Initialize accounting state
    vault.total_supply_pointer().set_value(0u128);
    vault.total_assets_pointer().set_value(0u128);
    
    // Initialize yield rate (basis points, e.g. 500 = 5%)
    vault.yield_rate_pointer().set_value(0u128);
    
    // Initialize the last yield timestamp
    vault.last_yield_height_pointer().set_value(0u64);
    
    Ok(response)
}
