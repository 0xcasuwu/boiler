// YieldVault - Bitcoin Smart Contract
// An adaptation of ERC-4626 for Bitcoin WebAssembly

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
use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::context::Context;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_runtime::storage::StoragePointer;

// Bitcoin stubs for non-bitcoin environments
#[cfg(not(feature = "bitcoin"))]
mod bitcoin_stubs {
    use std::fmt;
    
    #[derive(Clone, Debug)]
    pub struct Txid(pub [u8; 32]);
    
    impl Txid {
        pub fn from_slice(slice: &[u8]) -> Result<Self, &'static str> {
            if slice.len() != 32 {
                return Err("Invalid length for Txid");
            }
            let mut data = [0u8; 32];
            data.copy_from_slice(slice);
            Ok(Txid(data))
        }
    }
    
    impl fmt::Display for Txid {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", hex::encode(self.0))
        }
    }
    
    // Mock Transaction struct
    pub struct Transaction {}
    
    impl Transaction {
        pub fn compute_txid(&self) -> Txid {
            Txid([0; 32])
        }
    }
}

#[cfg(not(feature = "bitcoin"))]
use bitcoin_stubs::Txid;

#[cfg(feature = "bitcoin")]
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

// Implement the remaining required methods for AssetManagement trait
impl YieldVault {
    /// Get the current context
    fn context(&self) -> Result<Context> {
        let mut cursor = Cursor::new(CONTEXT.transaction());
        Context::parse(&mut cursor)
            .map_err(|_| anyhow!("Failed to parse context"))
    }
    
    /// Get the current timestamp
    fn get_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// Now implement the AssetManagement trait
impl AssetManagement for YieldVault {
    fn context(&self) -> Result<Context> {
        let mut cursor = Cursor::new(CONTEXT.transaction());
        Context::parse(&mut cursor)
            .map_err(|_| anyhow!("Failed to parse context"))
    }
    
    fn get_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// Add direct convenience methods for tests (without _api suffix)
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

// Manual implementation of message dispatching
impl YieldVault {
    fn dispatch(&mut self, opcode: u32, args: &[u8]) -> Result<CallResponse> {
        match opcode {
            // == Initialization ==
            0 => {
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
                    let decimal_offset = 8u8; // Default to 8 decimals for Bitcoin
                    
                    self.initialize(name, symbol, asset_name, asset_symbol, decimal_offset)
                }
            },
            
            // == Asset Management ==
            10 => {
                // Deposit
                let tx_hash = String::from_utf8(args.to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args.to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                let assets = 0u128; // Parse from args in real implementation
                
                #[cfg(test)]
                {
                    return self.test_deposit(tx_hash, caller, receiver, assets);
                }
                
                #[cfg(not(test))]
                self.deposit(tx_hash, caller, receiver, assets)
            },
            
            11 => {
                // Mint
                let tx_hash = String::from_utf8(args.to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args.to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                let shares = 0u128; // Parse from args in real implementation
                
                self.mint(tx_hash, caller, receiver, shares)
            },
            
            12 => {
                // Withdraw
                let tx_hash = String::from_utf8(args.to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args.to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                let owner = String::from_utf8(args.to_vec()).unwrap_or_default();
                let assets = 0u128; // Parse from args in real implementation
                
                self.withdraw(tx_hash, caller, receiver, owner, assets)
            },
            
            13 => {
                // Redeem
                let tx_hash = String::from_utf8(args.to_vec()).unwrap_or_default();
                let caller = String::from_utf8(args.to_vec()).unwrap_or_default();
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                let owner = String::from_utf8(args.to_vec()).unwrap_or_default();
                let shares = 0u128; // Parse from args in real implementation
                
                self.redeem(tx_hash, caller, receiver, owner, shares)
            },
            
            // == View Functions: Metadata ==
            100 => self.get_name(),
            101 => self.get_symbol(),
            102 => self.get_decimals(),
            103 => self.get_asset(),
            
            // == View Functions: Accounting ==
            200 => self.get_total_assets(),
            201 => {
                let assets = 0u128; // Parse from args in real implementation
                self.convert_to_shares(assets)
            },
            202 => {
                let shares = 0u128; // Parse from args in real implementation
                self.convert_to_assets(shares)
            },
            
            // == View Functions: Limits ==
            300 => {
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_deposit(receiver)
            },
            301 => {
                let receiver = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_mint(receiver)
            },
            302 => {
                let owner = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_withdraw(owner)
            },
            303 => {
                let owner = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_max_redeem(owner)
            },
            
            // == View Functions: Preview ==
            400 => {
                let assets = 0u128; // Parse from args in real implementation
                self.preview_deposit_api(assets)
            },
            401 => {
                let shares = 0u128; // Parse from args in real implementation
                self.preview_mint_api(shares)
            },
            402 => {
                let assets = 0u128; // Parse from args in real implementation
                self.preview_withdraw_api(assets)
            },
            403 => {
                let shares = 0u128; // Parse from args in real implementation
                self.preview_redeem_api(shares)
            },
            
            // == Custom Data Management ==
            500 => {
                let key = String::from_utf8(args.to_vec()).unwrap_or_default();
                let value = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.set_data(key, value)
            },
            501 => {
                let key = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_data(key)
            },
            
            // == Balance Management ==
            600 => {
                let account = String::from_utf8(args.to_vec()).unwrap_or_default();
                self.get_balance_of(account)
            },
            601 => self.get_total_supply(),
            
            // == Security Operations ==
            900 => {
                let yield_rate = 0u128; // Parse from args in real implementation
                self.update_yield_rate(yield_rate)
            },
            
            // Unknown opcode
            _ => Err(anyhow!("Unknown opcode: {}", opcode))
        }
    }
}

// Additional methods needed for the dispatch function
impl YieldVault {
    // Initialize the vault with its base parameters
    fn initialize(
        &mut self,
        name: String, 
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimal_offset: u8
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Use the initialization guard to prevent multiple initializations
        self.observe_initialization()
            .map_err(|e| anyhow!("Initialization error: {}", e))?;
        
        // Store basic token metadata
        self.name_pointer().set(std::sync::Arc::new(name.as_bytes().to_vec()));
        self.symbol_pointer().set(std::sync::Arc::new(symbol.as_bytes().to_vec()));
        self.asset_name_pointer().set(std::sync::Arc::new(asset_name.as_bytes().to_vec()));
        self.asset_symbol_pointer().set(std::sync::Arc::new(asset_symbol.as_bytes().to_vec()));
        self.decimals_pointer().set_value(decimal_offset);
        
        // Store the asset ID (using the first incoming alkane's ID)
        if let Some(incoming) = context.incoming_alkanes.0.first() {
            self.store_asset_id(&incoming.id);
        }
        
        // Initialize accounting state
        self.total_supply_pointer().set_value(0u128);
        self.total_assets_pointer().set_value(0u128);
        
        // Initialize yield rate (basis points, e.g. 500 = 5%)
        self.yield_rate_pointer().set_value(0u128);
        
        // Initialize the last yield timestamp
        self.last_yield_update_pointer().set_value(self.get_timestamp());
        
        Ok(response)
    }
    
    // Update the yield rate
    fn update_yield_rate(&mut self, yield_rate: u128) -> Result<CallResponse> {
        let context = self.context()?;
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
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let name = String::from_utf8(self.name_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse name"))?;
        response.data = name.as_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get vault symbol
    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let symbol = String::from_utf8(self.symbol_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse symbol"))?;
        response.data = symbol.as_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get decimals
    fn get_decimals(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let decimals = self.decimals_pointer().get_value::<u8>();
        response.data = vec![decimals];
        
        Ok(response)
    }
    
    // Get asset symbol
    fn get_asset(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let asset_symbol = String::from_utf8(self.asset_symbol_pointer().get().as_ref().clone())
            .map_err(|_| anyhow!("Failed to parse asset symbol"))?;
        response.data = asset_symbol.as_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get total assets
    fn get_total_assets(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        response.data = total_assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Convert to shares API endpoint
    fn convert_to_shares(&self, assets: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let shares = self.convert_assets_to_shares(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Conversion error: {}", e))?;
        response.data = shares.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Convert to assets API endpoint
    fn convert_to_assets(&self, shares: u128) -> Result<CallResponse> {
        let context = self.context()?;
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
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_deposit = self.max_deposit(&receiver)
            .map_err(|e| anyhow!("Max deposit error: {}", e))?;
        response.data = max_deposit.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get max mint API endpoint
    fn get_max_mint(&self, receiver: String) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_mint = self.max_mint(&receiver)
            .map_err(|e| anyhow!("Max mint error: {}", e))?;
        response.data = max_mint.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get max withdraw API endpoint
    fn get_max_withdraw(&self, owner: String) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_withdraw = self.max_withdraw(&owner)
            .map_err(|e| anyhow!("Max withdraw error: {}", e))?;
        response.data = max_withdraw.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get max redeem API endpoint
    fn get_max_redeem(&self, owner: String) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let max_redeem = self.max_redeem(&owner)
            .map_err(|e| anyhow!("Max redeem error: {}", e))?;
        response.data = max_redeem.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Preview deposit API endpoint
    fn preview_deposit_api(&self, assets: u128) -> Result<CallResponse> {
        let context = self.context()?;
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
        let context = self.context()?;
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
        let context = self.context()?;
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
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        let assets = self.convert_shares_to_assets(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview redeem error: {}", e))?;
        response.data = assets.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Set custom data
    fn set_data(&mut self, key: String, value: String) -> Result<CallResponse> {
        let context = self.context()?;
        let response = CallResponse::forward(&context.incoming_alkanes);
        
        // Prefix with /data/ to separate from core storage
        let storage_key = format!("/data/{}", key);
        let mut storage_pointer = StoragePointer::from_keyword(&storage_key);
        storage_pointer.set(std::sync::Arc::new(value.as_bytes().to_vec()));
        
        Ok(response)
    }
    
    // Get custom data
    fn get_data(&self, key: String) -> Result<CallResponse> {
        let context = self.context()?;
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
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        let balance = self.get_balance(&account);
        response.data = balance.to_le_bytes().to_vec();
        
        Ok(response)
    }
    
    // Get total supply API endpoint
    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
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

// WebAssembly exports through wasm-bindgen
#[wasm_bindgen]
pub fn call(opcode: u32, args: &[u8]) -> Vec<u8> {
    let mut vault = YieldVault::default();
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
