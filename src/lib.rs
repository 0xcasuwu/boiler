//! YieldVault - Bitcoin Smart Contract
//! An adaptation of ERC-4626 for Bitcoin WebAssembly

// Module declarations
mod storage;
mod security;
mod utils;
mod asset_management;

#[cfg(test)]
pub mod tests;

// Import our trait implementations
use storage::Storage;
use security::Security;
use utils::Conversion;
use asset_management::AssetManagement;

// Import required crates
use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
use alkanes_support::context::Context;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_runtime::storage::StoragePointer;
use metashrew_support::compat::to_arraybuffer_layout;

// Import Bitcoin types directly without conditionals
use bitcoin::Txid;

/// YieldVault implements an ERC-4626 style vault on Bitcoin
/// It manages deposits and withdrawals of an underlying asset
pub struct YieldVault {}

// Default implementation
impl Default for YieldVault {
    fn default() -> Self {
        Self {}
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

// We'll define our opcodes as constants for better type safety and readability
/// Opcode definitions for all operations
pub mod opcodes {
    // Initialization
    pub const INITIALIZE: u32 = 0;
    
    // Asset Management
    pub const DEPOSIT: u32 = 10;
    pub const MINT: u32 = 11;
    pub const WITHDRAW: u32 = 12;
    pub const REDEEM: u32 = 13;
    
    // Metadata View Functions
    pub const GET_NAME: u32 = 100;
    pub const GET_SYMBOL: u32 = 101;
    pub const GET_DECIMALS: u32 = 102;
    pub const GET_ASSET: u32 = 103;
    
    // Accounting View Functions
    pub const GET_TOTAL_ASSETS: u32 = 200;
    pub const CONVERT_TO_SHARES: u32 = 201;
    pub const CONVERT_TO_ASSETS: u32 = 202;
    
    // Limit View Functions
    pub const GET_MAX_DEPOSIT: u32 = 300;
    pub const GET_MAX_MINT: u32 = 301;
    pub const GET_MAX_WITHDRAW: u32 = 302;
    pub const GET_MAX_REDEEM: u32 = 303;
    
    // Preview View Functions
    pub const PREVIEW_DEPOSIT: u32 = 400;
    pub const PREVIEW_MINT: u32 = 401;
    pub const PREVIEW_WITHDRAW: u32 = 402;
    pub const PREVIEW_REDEEM: u32 = 403;
    
    // Custom Data Operations
    pub const SET_DATA: u32 = 500;
    pub const GET_DATA: u32 = 501;
    
    // Balance Management
    pub const GET_BALANCE_OF: u32 = 600;
    pub const GET_TOTAL_SUPPLY: u32 = 601;
    
    // Administrative Operations
    pub const UPDATE_YIELD_RATE: u32 = 900;
}

// Implementation of message handlers for YieldVault
impl YieldVault {
    // Message handlers
    // Initialize the vault with its base parameters
    fn initialize(
        &self,
        name: String, 
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimal_offset: u128
    ) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Use the initialization guard to prevent multiple initializations
        Security::observe_initialization(self)
            .map_err(|e| anyhow!("Initialization error: {}", e))?;
        
        // Store basic token metadata
        self.name_pointer().set(std::sync::Arc::new(name.as_bytes().to_vec()));
        self.symbol_pointer().set(std::sync::Arc::new(symbol.as_bytes().to_vec()));
        self.asset_name_pointer().set(std::sync::Arc::new(asset_name.as_bytes().to_vec()));
        self.asset_symbol_pointer().set(std::sync::Arc::new(asset_symbol.as_bytes().to_vec()));
        
        // Convert u128 to u8 for storage, as the original contract used u8
        let decimal_offset_u8 = if decimal_offset > u8::MAX.into() {
            // Default to 8 if out of range
            8u8
        } else {
            decimal_offset as u8
        };
        self.decimals_pointer().set_value(decimal_offset_u8);
        
        // Store the asset ID (using the first incoming alkane's ID)
        if let Some(incoming) = context.incoming_alkanes.0.first() {
            self.store_asset_id(&incoming.id);
        }
        
        // Initialize accounting state
        self.total_supply_pointer().set_value(0u128);
        self.total_assets_pointer().set_value(0u128);
        
        // Initialize yield rate (basis points, e.g. 500 = 5%)
        self.yield_rate_pointer().set_value(0u128);
        
        // Initialize the last yield block height
        self.last_yield_height_pointer().set_value(self.height());
        
        Ok(response)
    }
    
    // Update the yield rate
    fn update_yield_rate(&self, yield_rate: u128) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Update the yield first with the old rate
        self.update_yield()
            .map_err(|e| anyhow!("Yield update error: {}", e))?;
        
        // Set the new yield rate
        self.yield_rate_pointer().set_value(yield_rate);
        
        Ok(response)
    }
    
    // Get vault name
    fn get_name(&self) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let name = String::from_utf8(self.name_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse name"))?;
        response.data = name.as_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get vault symbol
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let symbol = String::from_utf8(self.symbol_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse symbol"))?;
        response.data = symbol.as_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get decimals
    fn get_decimals(&self) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let decimals = self.decimals_pointer().get_value::<u8>();
        // Convert to u128 for better compatibility
        let decimals_u128: u128 = decimals.into();
        response.data = decimals_u128.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get asset symbol
    fn get_asset(&self) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let asset_symbol = String::from_utf8(self.asset_symbol_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse asset symbol"))?;
        response.data = asset_symbol.as_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get total assets
    fn get_total_assets(&self) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        response.data = total_assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Convert to shares API endpoint
    // Convert to shares API endpoint
    fn convert_to_shares(&self, assets: u128) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let shares = self.convert_assets_to_shares(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Conversion error: {}", e))?;
        response.data = shares.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Convert to assets API endpoint
    // Convert to assets API endpoint
    fn convert_to_assets(&self, shares: u128) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let assets = self.convert_shares_to_assets(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Conversion error: {}", e))?;
        response.data = assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get max deposit API endpoint
    fn get_max_deposit(&self, receiver: String) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_deposit = self.max_deposit(&receiver)
            .map_err(|e| anyhow!("Max deposit error: {}", e))?;
        response.data = max_deposit.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get max mint API endpoint
    fn get_max_mint(&self, receiver: String) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_mint = self.max_mint(&receiver)
            .map_err(|e| anyhow!("Max mint error: {}", e))?;
        response.data = max_mint.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get max withdraw API endpoint
    fn get_max_withdraw(&self, owner: String) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_withdraw = self.max_withdraw(&owner)
            .map_err(|e| anyhow!("Max withdraw error: {}", e))?;
        response.data = max_withdraw.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get max redeem API endpoint
    fn get_max_redeem(&self, owner: String) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_redeem = self.max_redeem(&owner)
            .map_err(|e| anyhow!("Max redeem error: {}", e))?;
        response.data = max_redeem.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Preview deposit API endpoint
    fn preview_deposit_api(&self, assets: u128) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let shares = self.convert_assets_to_shares(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview deposit error: {}", e))?;
        response.data = shares.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Preview mint API endpoint
    fn preview_mint_api(&self, shares: u128) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let assets = self.preview_mint(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview mint error: {}", e))?;
        response.data = assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Preview withdraw API endpoint
    fn preview_withdraw_api(&self, assets: u128) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let shares = self.preview_withdraw(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview withdraw error: {}", e))?;
        response.data = shares.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Preview redeem API endpoint
    fn preview_redeem_api(&self, shares: u128) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let assets = self.convert_shares_to_assets(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview redeem error: {}", e))?;
        response.data = assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Set custom data
    fn set_data(&self, key: String, value: String) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Prefix with /data/ to separate from core storage
        let storage_key = format!("/data/{}", key);
        let mut storage_pointer = StoragePointer::from_keyword(&storage_key);
        storage_pointer.set(std::sync::Arc::new(value.as_bytes().to_vec()));
        
        Ok(response)
    }
    
    // Get custom data
    fn get_data(&self, key: String) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Prefix with /data/ to separate from core storage
        let storage_key = format!("/data/{}", key);
        let storage_pointer = StoragePointer::from_keyword(&storage_key);
        let data_bytes = storage_pointer.get();
        
        let data = String::from_utf8(data_bytes.as_ref().clone())
            .unwrap_or_default();
        response.data = data.as_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get account balance API endpoint
    fn get_balance_of(&self, account: String) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let balance = self.get_balance(&account);
        response.data = balance.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get total supply API endpoint
    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        response.data = total_supply.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    #[cfg(test)]
    pub fn test_deposit(
        &mut self,
        tx_hash: String,
        caller: String,
        receiver: String,
        assets: u128
    ) -> Result<CallResponse> {
        self.deposit(tx_hash, caller, receiver, assets)
    }
    
    #[cfg(test)]
    pub fn test_mint(
        &mut self,
        tx_hash: String,
        caller: String,
        receiver: String,
        shares: u128
    ) -> Result<CallResponse> {
        self.mint(tx_hash, caller, receiver, shares)
    }
}

// Implement AlkaneResponder for YieldVault
impl AlkaneResponder for YieldVault {}

// Manual dispatch implementation with clean structure - inspired by MessageDispatch pattern
impl YieldVault {
    // Main dispatch method that routes opcodes to their handlers
    fn dispatch(&self, opcode: u32, args: &[u8]) -> Result<CallResponse> {
        match opcode {
            // == Initialization ==
            opcodes::INITIALIZE => {
                #[cfg(test)]
                {
                    use crate::tests::test_utils;
                    return test_utils::handle_test_initialize(self, args);
                }
                
                #[cfg(not(test))]
                {
                    let mut args_iter = args.split(|&b| b == 0);
                    let name = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                    let symbol = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                    let asset_name = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                    let asset_symbol = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                    let decimal_offset = 8u128; // Default to 8 decimals for Bitcoin
                    
                    self.initialize(name, symbol, asset_name, asset_symbol, decimal_offset)
                }
            },
            
            // == Asset Management ==
            opcodes::DEPOSIT => {
                // Parse deposit arguments
                let mut args_iter = args.split(|&b| b == 0);
                let tx_hash = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                
                // Parse assets as u128 from bytes
                let assets_bytes = args_iter.next().unwrap_or(&[]);
                let mut assets = 0u128;
                if !assets_bytes.is_empty() {
                    let mut buf = [0u8; 16];
                    let len = std::cmp::min(assets_bytes.len(), 16);
                    buf[..len].copy_from_slice(&assets_bytes[..len]);
                    assets = u128::from_le_bytes(buf);
                }
                
                self.deposit(tx_hash, caller, receiver, assets)
            },
            
            opcodes::MINT => {
                // Parse mint arguments
                let mut args_iter = args.split(|&b| b == 0);
                let tx_hash = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                
                // Parse shares as u128 from bytes
                let shares_bytes = args_iter.next().unwrap_or(&[]);
                let mut shares = 0u128;
                if !shares_bytes.is_empty() {
                    let mut buf = [0u8; 16];
                    let len = std::cmp::min(shares_bytes.len(), 16);
                    buf[..len].copy_from_slice(&shares_bytes[..len]);
                    shares = u128::from_le_bytes(buf);
                }
                
                self.mint(tx_hash, caller, receiver, shares)
            },
            
            opcodes::WITHDRAW => {
                // Parse withdraw arguments
                let mut args_iter = args.split(|&b| b == 0);
                let tx_hash = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let owner = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                
                // Parse assets as u128 from bytes
                let assets_bytes = args_iter.next().unwrap_or(&[]);
                let mut assets = 0u128;
                if !assets_bytes.is_empty() {
                    let mut buf = [0u8; 16];
                    let len = std::cmp::min(assets_bytes.len(), 16);
                    buf[..len].copy_from_slice(&assets_bytes[..len]);
                    assets = u128::from_le_bytes(buf);
                }
                
                self.withdraw(tx_hash, caller, receiver, owner, assets)
            },
            
            opcodes::REDEEM => {
                // Parse redeem arguments
                let mut args_iter = args.split(|&b| b == 0);
                let tx_hash = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let owner = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                
                // Parse shares as u128 from bytes
                let shares_bytes = args_iter.next().unwrap_or(&[]);
                let mut shares = 0u128;
                if !shares_bytes.is_empty() {
                    let mut buf = [0u8; 16];
                    let len = std::cmp::min(shares_bytes.len(), 16);
                    buf[..len].copy_from_slice(&shares_bytes[..len]);
                    shares = u128::from_le_bytes(buf);
                }
                
                self.redeem(tx_hash, caller, receiver, owner, shares)
            },
            
            // == View Functions: Metadata ==
            opcodes::GET_NAME => self.get_name(),
            opcodes::GET_SYMBOL => self.get_symbol(),
            opcodes::GET_DECIMALS => self.get_decimals(),
            opcodes::GET_ASSET => self.get_asset(),
            
            // == View Functions: Accounting ==
            opcodes::GET_TOTAL_ASSETS => self.get_total_assets(),
            opcodes::CONVERT_TO_SHARES => {
                // Parse assets from args
                let assets = if args.len() >= 16 {
                    let mut buf = [0u8; 16];
                    buf.copy_from_slice(&args[..16]);
                    u128::from_le_bytes(buf)
                } else {
                    0u128
                };
                self.convert_to_shares(assets)
            },
            opcodes::CONVERT_TO_ASSETS => {
                // Parse shares from args
                let shares = if args.len() >= 16 {
                    let mut buf = [0u8; 16];
                    buf.copy_from_slice(&args[..16]);
                    u128::from_le_bytes(buf)
                } else {
                    0u128
                };
                self.convert_to_assets(shares)
            },
            
            // == View Functions: Limits ==
            opcodes::GET_MAX_DEPOSIT => {
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_deposit(receiver)
            },
            opcodes::GET_MAX_MINT => {
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_mint(receiver)
            },
            opcodes::GET_MAX_WITHDRAW => {
                let owner = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_withdraw(owner)
            },
            opcodes::GET_MAX_REDEEM => {
                let owner = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_redeem(owner)
            },
            
            // == View Functions: Preview ==
            opcodes::PREVIEW_DEPOSIT => {
                // Parse assets from args
                let assets = if args.len() >= 16 {
                    let mut buf = [0u8; 16];
                    buf.copy_from_slice(&args[..16]);
                    u128::from_le_bytes(buf)
                } else {
                    0u128
                };
                self.preview_deposit_api(assets)
            },
            opcodes::PREVIEW_MINT => {
                // Parse shares from args
                let shares = if args.len() >= 16 {
                    let mut buf = [0u8; 16];
                    buf.copy_from_slice(&args[..16]);
                    u128::from_le_bytes(buf)
                } else {
                    0u128
                };
                self.preview_mint_api(shares)
            },
            opcodes::PREVIEW_WITHDRAW => {
                // Parse assets from args
                let assets = if args.len() >= 16 {
                    let mut buf = [0u8; 16];
                    buf.copy_from_slice(&args[..16]);
                    u128::from_le_bytes(buf)
                } else {
                    0u128
                };
                self.preview_withdraw_api(assets)
            },
            opcodes::PREVIEW_REDEEM => {
                // Parse shares from args
                let shares = if args.len() >= 16 {
                    let mut buf = [0u8; 16];
                    buf.copy_from_slice(&args[..16]);
                    u128::from_le_bytes(buf)
                } else {
                    0u128
                };
                self.preview_redeem_api(shares)
            },
            
            // == Custom Data Management ==
            opcodes::SET_DATA => {
                let mut args_iter = args.split(|&b| b == 0);
                let key = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                let value = String::from_utf8(args_iter.next().unwrap_or(&[]).to_vec()).unwrap_or_default();
                self.set_data(key, value)
            },
            opcodes::GET_DATA => {
                let key = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_data(key)
            },
            
            // == Balance Management ==
            opcodes::GET_BALANCE_OF => {
                let account = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_balance_of(account)
            },
            opcodes::GET_TOTAL_SUPPLY => self.get_total_supply(),
            
            // == Security Operations ==
            opcodes::UPDATE_YIELD_RATE => {
                // Parse yield rate from args
                let yield_rate = if args.len() >= 16 {
                    let mut buf = [0u8; 16];
                    buf.copy_from_slice(&args[..16]);
                    u128::from_le_bytes(buf)
                } else {
                    0u128
                };
                self.update_yield_rate(yield_rate)
            },
            
            // Unknown opcode
            _ => Err(anyhow!("Unknown opcode: {}", opcode))
        }
    }
}

// WebAssembly exports through wasm-bindgen
#[wasm_bindgen]
pub fn call(opcode: u32, args: &[u8]) -> Vec<u8> {
    // Create a default vault instance
    let vault = YieldVault::default();
    
    // Use a structured dispatch pattern
    match vault.dispatch(opcode, args) {
        Ok(response) => response.data,
        Err(e) => format!("Error: {}", e).as_bytes().to_vec(),
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    // Initialize any global state if needed
}

/// ContextHandle implementation for the contract
pub struct ContextHandle(());

impl AlkaneResponder for ContextHandle {}

pub const CONTEXT: ContextHandle = ContextHandle(());
