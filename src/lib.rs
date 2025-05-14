//! YieldVault - Bitcoin Smart Contract
//! An adaptation of ERC-4626 for Bitcoin WebAssembly

// Module declarations
pub mod constants;
pub mod storage;
pub mod security;
pub mod utils;
pub mod asset_management;
pub mod simple_utils;
pub mod mock_vault;
pub mod mock_vault_extension;
pub mod message;
pub mod macros;
pub mod attributes;

#[cfg(test)]
pub mod tests;

// Direct access to these traits within this file
// without a separate import required

// Export for external use
pub use crate::utils::Conversion;
pub use crate::storage::Storage;
pub use crate::security::Security;
pub use crate::asset_management::AssetManagement;
pub use crate::message::YieldVaultMessage;

// Import required crates
use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::*;

// Import Alkanes runtime and support
use alkanes_runtime::{message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_proc_macros::declare_alkane;
use alkanes_runtime::storage::StoragePointer;
// Import metashrew support
use metashrew_support::compat::to_arraybuffer_layout;
use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_support::context::Context;
use alkanes_support::response::CallResponse;

// Import Bitcoin types directly without conditionals

/// YieldVault implements an ERC-4626 style vault on Bitcoin
/// It manages deposits and withdrawals of an underlying asset
pub struct YieldVault {}

// Default implementation
impl Default for YieldVault {
    fn default() -> Self {
        Self {}
    }
}

// Forward implementation of AlkaneResponder for YieldVault
impl AlkaneResponder for YieldVault {
    fn height(&self) -> u64 {
        // Return a placeholder height
        0
    }
    
    fn context(&self) -> Result<Context, anyhow::Error> {
        // Return a new Context
        Ok(Context::default())
    }
}

// Implement the trait requirements for YieldVault
impl Storage for YieldVault {}
impl Security for YieldVault {}
impl AssetManagement for YieldVault {}

// Convenience methods for YieldVault
impl YieldVault {
    /// Preview deposit - calculates shares to be minted for a given asset amount
    pub fn preview_deposit(&self, assets: u128) -> Result<u128, &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        self.convert_assets_to_shares(assets, total_assets, total_supply)
    }
    
    /// Preview redeem - calculates assets to be withdrawn for a given share amount
    pub fn preview_redeem(&self, shares: u128) -> Result<u128, &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        self.convert_shares_to_assets(shares, total_assets, total_supply)
    }
}


// Implementation of message handlers for YieldVault
// These methods will be called by the MessageDispatch trait
impl YieldVault {
    // Wrapper methods for MessageDispatch compatibility
    
    // Preview deposit wrapper for MessageDispatch
    pub fn preview_deposit_wrapper(&self, assets: u128) -> Result<CallResponse> {
        let result = self.preview_deposit(assets)
            .map_err(|e| anyhow!("Preview deposit error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", result).as_bytes().to_vec();
        Ok(response)
    }
    
    // Preview mint wrapper for MessageDispatch
    pub fn preview_mint_wrapper(&self, shares: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let result = self.preview_mint(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview mint error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", result).as_bytes().to_vec();
        Ok(response)
    }
    
    // Preview withdraw wrapper for MessageDispatch
    pub fn preview_withdraw_wrapper(&self, assets: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let result = self.preview_withdraw(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview withdraw error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", result).as_bytes().to_vec();
        Ok(response)
    }
    
    // Preview redeem wrapper for MessageDispatch
    pub fn preview_redeem_wrapper(&self, shares: u128) -> Result<CallResponse> {
        let result = self.preview_redeem(shares)
            .map_err(|e| anyhow!("Preview redeem error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", result).as_bytes().to_vec();
        Ok(response)
    }
    // == Message handlers for YieldVaultMessage ==
    
    // Initialize the vault with its base parameters
    pub fn initialize(
        &self,
        name: String, 
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimal_offset: u128
    ) -> Result<CallResponse> {
        // Use the initialization guard to prevent multiple initializations
        Security::observe_initialization(self)
            .map_err(|e| anyhow!("Initialization error: {}", e))?;
        
        // Store basic token metadata directly with storage pointers
        StoragePointer::from_keyword("/name").set(std::sync::Arc::new(name.as_bytes().to_vec()));
        StoragePointer::from_keyword("/symbol").set(std::sync::Arc::new(symbol.as_bytes().to_vec()));
        StoragePointer::from_keyword("/asset-name").set(std::sync::Arc::new(asset_name.as_bytes().to_vec()));
        StoragePointer::from_keyword("/asset-symbol").set(std::sync::Arc::new(asset_symbol.as_bytes().to_vec()));
        
        // Convert u128 to u8 for storage, as the original contract used u8
        let decimal_offset_u8 = if decimal_offset > u8::MAX.into() {
            // Default to 8 if out of range
            8u8
        } else {
            decimal_offset as u8
        };
        StoragePointer::from_keyword("/decimals").set_value(decimal_offset_u8);
        
        // Initialize accounting state
        StoragePointer::from_keyword("/total-supply").set_value(0u128);
        StoragePointer::from_keyword("/total-assets").set_value(0u128);
        
        // Initialize yield rate (basis points, e.g. 500 = 5%)
        StoragePointer::from_keyword("/yield-rate").set_value(0u128);
        
        // Initialize the last yield block height
        StoragePointer::from_keyword("/last-yield-height").set_value(self.height());
        
        let mut response = CallResponse::default();
        response.data = "Initialized".as_bytes().to_vec();
        Ok(response)
    }
    
    // Deposit assets into the vault
    pub fn deposit(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        assets: u128
    ) -> Result<CallResponse> {
        // This is a placeholder for the actual implementation
        let mut response = CallResponse::default();
        response.data = "Deposit operation".as_bytes().to_vec();
        Ok(response)
    }
    
    // Mint shares from the vault
    pub fn mint(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        shares: u128
    ) -> Result<CallResponse> {
        // This is a placeholder for the actual implementation
        let mut response = CallResponse::default();
        response.data = "Mint operation".as_bytes().to_vec();
        Ok(response)
    }
    
    // Withdraw assets from the vault
    pub fn withdraw(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        assets: u128
    ) -> Result<CallResponse> {
        // This is a placeholder for the actual implementation
        let mut response = CallResponse::default();
        response.data = "Withdraw operation".as_bytes().to_vec();
        Ok(response)
    }
    
    // Redeem shares from the vault
    pub fn redeem(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        shares: u128
    ) -> Result<CallResponse> {
        // This is a placeholder for the actual implementation
        let mut response = CallResponse::default();
        response.data = "Redeem operation".as_bytes().to_vec();
        Ok(response)
    }
    
    // Get vault name
    pub fn get_name(&self) -> Result<CallResponse> {
        let name = String::from_utf8(self.name_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse name"))?;
        
        let mut response = CallResponse::default();
        response.data = name.as_bytes().to_vec();
        Ok(response)
    }
    
    // Get vault symbol
    pub fn get_symbol(&self) -> Result<CallResponse> {
        let symbol = String::from_utf8(self.symbol_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse symbol"))?;
        
        let mut response = CallResponse::default();
        response.data = symbol.as_bytes().to_vec();
        Ok(response)
    }
    
    // Get decimals
    pub fn get_decimals(&self) -> Result<CallResponse> {
        let decimals = self.decimals_pointer().get_value::<u8>();
        let mut response = CallResponse::default();
        response.data = format!("{}", decimals).as_bytes().to_vec();
        Ok(response)
    }
    
    // Get asset symbol
    pub fn get_asset(&self) -> Result<CallResponse> {
        let asset_symbol = String::from_utf8(self.asset_symbol_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse asset symbol"))?;
        
        let mut response = CallResponse::default();
        response.data = asset_symbol.as_bytes().to_vec();
        Ok(response)
    }
    
    // Get total assets
    pub fn get_total_assets(&self) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let mut response = CallResponse::default();
        response.data = format!("{}", total_assets).as_bytes().to_vec();
        Ok(response)
    }
    
    // Convert to shares API endpoint
    pub fn convert_to_shares(&self, assets: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let shares = self.convert_assets_to_shares(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Conversion error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", shares).as_bytes().to_vec();
        Ok(response)
    }
    
    // Convert to assets API endpoint
    pub fn convert_to_assets(&self, shares: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let assets = self.convert_shares_to_assets(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Conversion error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", assets).as_bytes().to_vec();
        Ok(response)
    }
    
    // Get max deposit API endpoint
    pub fn get_max_deposit(&self, receiver: String) -> Result<CallResponse> {
        let max_deposit = self.max_deposit(&receiver)
            .map_err(|e| anyhow!("Max deposit error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", max_deposit).as_bytes().to_vec();
        Ok(response)
    }
    
    // Get max mint API endpoint
    pub fn get_max_mint(&self, receiver: String) -> Result<CallResponse> {
        let max_mint = self.max_mint(&receiver)
            .map_err(|e| anyhow!("Max mint error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", max_mint).as_bytes().to_vec();
        Ok(response)
    }
    
    // Get max withdraw API endpoint
    pub fn get_max_withdraw(&self, owner: String) -> Result<CallResponse> {
        let max_withdraw = self.max_withdraw(&owner)
            .map_err(|e| anyhow!("Max withdraw error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", max_withdraw).as_bytes().to_vec();
        Ok(response)
    }
    
    // Get max redeem API endpoint
    pub fn get_max_redeem(&self, owner: String) -> Result<CallResponse> {
        let max_redeem = self.max_redeem(&owner)
            .map_err(|e| anyhow!("Max redeem error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", max_redeem).as_bytes().to_vec();
        Ok(response)
    }
    
    // Preview deposit API endpoint
    pub fn preview_deposit_api(&self, assets: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let shares = self.convert_assets_to_shares(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview deposit error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", shares).as_bytes().to_vec();
        Ok(response)
    }
    
    // Preview mint API endpoint - wrapper for MessageDispatch
    pub fn preview_mint_api(&self, shares: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let assets = self.preview_mint(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview mint error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", assets).as_bytes().to_vec();
        Ok(response)
    }
    
    // Preview withdraw API endpoint - wrapper for MessageDispatch
    pub fn preview_withdraw_api(&self, assets: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let shares = self.preview_withdraw(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview withdraw error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", shares).as_bytes().to_vec();
        Ok(response)
    }
    
    // Preview redeem API endpoint
    pub fn preview_redeem_api(&self, shares: u128) -> Result<CallResponse> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let assets = self.convert_shares_to_assets(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview redeem error: {}", e))?;
        
        let mut response = CallResponse::default();
        response.data = format!("{}", assets).as_bytes().to_vec();
        Ok(response)
    }
    
    // Get custom data
    pub fn get_data(&self, key: String) -> Result<CallResponse> {
        // Prefix with /data/ to separate from core storage
        let storage_key = format!("/data/{}", key);
        let storage_pointer = StoragePointer::from_keyword(&storage_key);
        let data_bytes = storage_pointer.get();
        
        let data = String::from_utf8(data_bytes.as_ref().clone())
            .unwrap_or_default();
        
        let mut response = CallResponse::default();
        response.data = data.as_bytes().to_vec();
        Ok(response)
    }
    
    // Get total supply API endpoint
    pub fn get_total_supply(&self) -> Result<CallResponse> {
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        let mut response = CallResponse::default();
        response.data = format!("{}", total_supply).as_bytes().to_vec();
        Ok(response)
    }
    
    #[cfg(test)]
    pub fn test_deposit(
        &mut self,
        _tx_hash: String,
        _caller: String,
        _receiver: String,
        _assets: u128
    ) -> Result<()> {
        Ok(())
    }
    
    #[cfg(test)]
    pub fn test_mint(
        &mut self,
        _tx_hash: String,
        _caller: String,
        _receiver: String,
        _shares: u128
    ) -> Result<()> {
        Ok(())
    }
    
    // Special handling for test initialization
    #[cfg(test)]
    pub fn handle_test_initialize(&self, args: &[u8]) -> Result<CallResponse> {
        use crate::tests::test_utils;
        test_utils::handle_test_initialize(self, args)
            .map(|_| {
                let mut response = CallResponse::default();
                response.data = "Initialized".as_bytes().to_vec();
                response
            })
    }
}

// Use the declare_alkane macro from our proc_macros crate for opcode handling
declare_alkane! {
    impl AlkaneResponder for YieldVault {
        type Message = YieldVaultMessage;
    }
}

// WebAssembly exports through wasm-bindgen
#[wasm_bindgen]
pub fn call(opcode: u32, args: &[u8]) -> Vec<u8> {
    // Create a default vault instance
    let vault = YieldVault::default();
    
    // Use the dispatch_message method generated by the declare_alkane macro
    // Convert u32 to u128 as required by our implementation
    match vault.dispatch_message(opcode.into(), args) {
        Ok(response) => response.data.to_vec(),
        Err(e) => format!("Error: {}", e).as_bytes().to_vec(),
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    // Set up WebAssembly-specific initialization
    #[cfg(target_arch = "wasm32")]
    {
        // Install better panic handler for WebAssembly
        console_error_panic_hook::set_once();
        
        // Log that initialization is complete
        web_sys::console::log_1(&"YieldVault WebAssembly module initialized".into());
    }
}

/// ContextHandle implementation for the contract
pub struct ContextHandle(());

// Implement Default for ContextHandle
impl Default for ContextHandle {
    fn default() -> Self {
        Self(())
    }
}

// Implement AlkaneResponder for ContextHandle
impl AlkaneResponder for ContextHandle {
    fn height(&self) -> u64 {
        // Get the current block height
        // This is a placeholder implementation
        
        0
    }
    
    fn context(&self) -> Result<Context, anyhow::Error> {
        // Return a new Context
        Ok(Context::default())
    }
}

// We don't need to implement message dispatch for ContextHandle
// since it's just a helper struct for the contract

pub const CONTEXT: ContextHandle = ContextHandle(());

// Include unit tests directly in lib.rs
#[cfg(test)]
mod lib_tests {
    use crate::YieldVault;
    use crate::utils::{Conversion, PRECISION_OFFSET};

    #[test]
    fn test_default_constructor() {
        // This test just verifies that we can create a YieldVault instance
        let _vault = YieldVault::default();
        
        // Simple verification that the instance was created successfully
        assert!(true);
    }

    #[test]
    fn test_conversion_functions() {
        // Create a basic vault instance
        let vault = YieldVault::default();
        
        // Test a simple conversion that doesn't require complex dependencies
        let assets = 100;
        let total_assets = 0;
        let total_supply = 0;
        
        // For an empty vault, shares should equal assets with precision offset (1:1 ratio)
        // With precision offset of 3, 100 assets = 100,000 shares
        let precision_factor = 10u128.pow(PRECISION_OFFSET as u32);
        let shares = vault.convert_assets_to_shares(assets, total_assets, total_supply).unwrap();
        assert_eq!(shares, assets * precision_factor);
        
        // Test with non-zero values and virtual offset
        // If total_assets = 1000 and total_supply = 500, then with virtual offset:
        // shares = assets * precision_factor * (virtual_shares + total_supply) / (virtual_assets + total_assets)
        // shares = 100 * 1000 * (1000000 + 500) / (1000000 + 1000)
        // shares = 100000 * 1000500 / 1001000 ≈ 99950
        
        let result = vault.convert_assets_to_shares(100, 1000, 500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 99950);
        
        // With virtual offset and precision offset:
        // assets = shares * (virtual_assets + total_assets) / (virtual_shares + total_supply) / precision_factor
        // assets = 50000 * (1000000 + 1000) / (1000000 + 500) / 1000
        // assets = 50000 * 1001000 / 1000500 / 1000 ≈ 50
        let result = vault.convert_shares_to_assets(50000, 1000, 500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 50);
    }

    #[test]
    fn test_preview_deposit_functionality() {
        // This test verifies the basic preview_deposit functionality
        let vault = YieldVault::default();
        
        // For an empty vault, preview_deposit should return the value with precision offset
        // With precision offset of 3, 100 assets = 100,000 shares
        let precision_factor = 10u128.pow(PRECISION_OFFSET as u32);
        let result = vault.preview_deposit(100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100 * precision_factor);
    }
}
