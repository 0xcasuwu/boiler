// YieldVault - Bitcoin Smart Contract
// An adaptation of ERC-4626 for Bitcoin WebAssembly
// Based on the memorybank architecture

use alkanes_support::*;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use serde_json;

/// YieldVaultMessage defines the opcode interface for the YieldVault contract
/// It adapts ERC-4626 functionality to Bitcoin's WASM execution environment
#[derive(MessageDispatch)]
enum YieldVaultMessage {
    // == Initialization ==
    #[opcode(0)]
    Initialize { 
        name: String, 
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimal_offset: u8 
    },
    
    // == Asset Management ==
    #[opcode(10)]
    Deposit { 
        tx_hash: String,
        caller: String, 
        receiver: String, 
        assets: u128 
    },
    
    #[opcode(11)]
    Mint { 
        tx_hash: String,
        caller: String,
        receiver: String, 
        shares: u128 
    },
    
    #[opcode(12)]
    Withdraw { 
        tx_hash: String,
        caller: String,
        receiver: String, 
        owner: String,
        assets: u128 
    },
    
    #[opcode(13)]
    Redeem { 
        tx_hash: String,
        caller: String,
        receiver: String, 
        owner: String,
        shares: u128 
    },

    // == View Functions: Metadata ==
    #[opcode(100)]
    #[returns(String)]
    Name {},
    
    #[opcode(101)]
    #[returns(String)]
    Symbol {},
    
    #[opcode(102)]
    #[returns(u8)]
    Decimals {},

    #[opcode(103)]
    #[returns(String)]
    Asset {},
    
    // == View Functions: Accounting ==
    #[opcode(200)]
    #[returns(u128)]
    TotalAssets {},
    
    #[opcode(201)]
    #[returns(u128)]
    ConvertToShares { assets: u128 },
    
    #[opcode(202)]
    #[returns(u128)]
    ConvertToAssets { shares: u128 },
    
    // == View Functions: Limits ==
    #[opcode(300)]
    #[returns(u128)]
    MaxDeposit { receiver: String },
    
    #[opcode(301)]
    #[returns(u128)]
    MaxMint { receiver: String },
    
    #[opcode(302)]
    #[returns(u128)]
    MaxWithdraw { owner: String },
    
    #[opcode(303)]
    #[returns(u128)]
    MaxRedeem { owner: String },
    
    // == View Functions: Preview ==
    #[opcode(400)]
    #[returns(u128)]
    PreviewDeposit { assets: u128 },
    
    #[opcode(401)]
    #[returns(u128)]
    PreviewMint { shares: u128 },
    
    #[opcode(402)]
    #[returns(u128)]
    PreviewWithdraw { assets: u128 },
    
    #[opcode(403)]
    #[returns(u128)]
    PreviewRedeem { shares: u128 },

    // == Custom Data Management ==
    #[opcode(500)]
    SetData { key: String, value: String },
    
    #[opcode(501)]
    #[returns(String)]
    GetData { key: String },
    
    // == Balance Management ==
    #[opcode(600)]
    #[returns(u128)]
    BalanceOf { account: String },
    
    #[opcode(601)]
    #[returns(u128)]
    TotalSupply {},

    // == Security Operations ==
    #[opcode(900)]
    UpdateYield { yield_rate: u128 }
}

/// YieldVault implements an ERC-4626 style vault on Bitcoin
/// It manages deposits and withdrawals of an underlying asset
pub struct YieldVault {}

impl Default for YieldVault {
    fn default() -> Self {
        Self {}
    }
}

impl YieldVault {
    // == Initialization ==
    
    /// Initialize the vault with its base parameters
    fn handle_initialize(
        &mut self, 
        name: String, 
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimal_offset: u8
    ) -> Result<(), &'static str> {
        // Use the initialization guard to prevent multiple initializations
        self.observe_initialization()?;
        
        // Store basic token metadata
        storage::set_string("/name", &name);
        storage::set_string("/symbol", &symbol);
        storage::set_string("/asset-name", &asset_name);
        storage::set_string("/asset-symbol", &asset_symbol);
        storage::set_u8("/decimals", decimal_offset);
        
        // Initialize accounting state
        storage::set_u128("/total-supply", 0);
        storage::set_u128("/total-assets", 0);
        
        // Initialize yield rate (basis points, e.g. 500 = 5%)
        storage::set_u128("/yield-rate", 0);
        
        // Initialize the last yield timestamp
        storage::set_u64("/last-yield-update", self.get_timestamp());
        
        Ok(())
    }
    
    // == Security Functions ==
    
    /// Validation function to ensure the contract is only initialized once
    fn observe_initialization(&self) -> Result<(), &'static str> {
        if storage::get_bool("/initialized").unwrap_or(false) {
            return Err("Already initialized");
        }
        storage::set_bool("/initialized", true);
        Ok(())
    }
    
    /// Validate and track a transaction hash to prevent replay attacks
    fn validate_and_track_transaction(&self, tx_hash: &str) -> Result<(), &'static str> {
        // Get the current set of transaction hashes
        let mut tx_hashes = self.get_transaction_hashes();
        
        // Check if this transaction hash has been used
        if tx_hashes.contains(tx_hash) {
            return Err("Transaction hash already used");
        }
        
        // Add the transaction hash to the set
        tx_hashes.insert(tx_hash.to_string());
        
        // Store the updated set
        self.set_transaction_hashes(&tx_hashes)
    }
    
    /// Get the stored transaction hashes
    fn get_transaction_hashes(&self) -> HashSet<String> {
        let json = storage::get_string("/tx-hashes").unwrap_or_default();
        if json.is_empty() {
            HashSet::new()
        } else {
            serde_json::from_str(&json).unwrap_or_default()
        }
    }
    
    /// Store the transaction hashes
    fn set_transaction_hashes(&self, hashes: &HashSet<String>) -> Result<(), &'static str> {
        let json = serde_json::to_string(hashes)
            .map_err(|_| "Failed to serialize transaction hashes")?;
        storage::set_string("/tx-hashes", &json);
        Ok(())
    }
    
    /// Check if caller has sufficient authorization
    fn check_authorization(&self, caller: &str, owner: &str) -> Result<(), &'static str> {
        // In this simple implementation, only the owner can operate on their assets
        // In a more complex implementation, this would check approved operators
        if caller != owner {
            return Err("Not authorized");
        }
        Ok(())
    }

    // == Asset Management Functions ==
    
    /// Deposit assets and mint shares
    fn handle_deposit(
        &mut self,
        tx_hash: String,
        caller: String,
        receiver: String,
        assets: u128
    ) -> Result<(), &'static str> {
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)?;
        
        // Update the yield before any operations
        self.update_yield()?;
        
        // Check deposit limit
        let max_deposit = self.handle_max_deposit(receiver.clone())?;
        if assets > max_deposit {
            return Err("Deposit amount exceeds limit");
        }
        
        // Calculate shares from assets
        let shares = self.handle_preview_deposit(assets)?;
        if shares == 0 {
            return Err("Zero shares");
        }
        
        // Update state
        self.add_total_assets(assets)?;
        self.mint_shares(&receiver, shares)?;
        
        Ok(())
    }
    
    /// Mint exact shares by depositing assets
    fn handle_mint(
        &mut self,
        tx_hash: String,
        caller: String,
        receiver: String,
        shares: u128
    ) -> Result<(), &'static str> {
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)?;
        
        // Update the yield before any operations
        self.update_yield()?;
        
        // Check mint limit
        let max_mint = self.handle_max_mint(receiver.clone())?;
        if shares > max_mint {
            return Err("Mint amount exceeds limit");
        }
        
        // Calculate assets needed for shares
        let assets = self.handle_preview_mint(shares)?;
        if assets == 0 {
            return Err("Zero assets");
        }
        
        // Update state
        self.add_total_assets(assets)?;
        self.mint_shares(&receiver, shares)?;
        
        Ok(())
    }
    
    /// Withdraw assets by burning shares
    fn handle_withdraw(
        &mut self,
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        assets: u128
    ) -> Result<(), &'static str> {
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)?;
        
        // Update the yield before any operations
        self.update_yield()?;
        
        // Check authorization
        self.check_authorization(&caller, &owner)?;
        
        // Check withdrawal limit
        let max_withdraw = self.handle_max_withdraw(owner.clone())?;
        if assets > max_withdraw {
            return Err("Withdrawal amount exceeds limit");
        }
        
        // Calculate shares needed for assets
        let shares = self.handle_preview_withdraw(assets)?;
        if shares == 0 {
            return Err("Zero shares");
        }
        
        // Update state
        self.subtract_total_assets(assets)?;
        self.burn_shares(&owner, shares)?;
        
        Ok(())
    }
    
    /// Redeem shares for assets
    fn handle_redeem(
        &mut self,
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        shares: u128
    ) -> Result<(), &'static str> {
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)?;
        
        // Update the yield before any operations
        self.update_yield()?;
        
        // Check authorization
        self.check_authorization(&caller, &owner)?;
        
        // Check redemption limit
        let max_redeem = self.handle_max_redeem(owner.clone())?;
        if shares > max_redeem {
            return Err("Redemption amount exceeds limit");
        }
        
        // Calculate assets for shares
        let assets = self.handle_preview_redeem(shares)?;
        
        // Update state
        self.subtract_total_assets(assets)?;
        self.burn_shares(&owner, shares)?;
        
        Ok(())
    }

    // == Yield Management ==
    
    /// Update the accumulated yield
    fn update_yield(&self) -> Result<(), &'static str> {
        let current_time = self.get_timestamp();
        let last_update = storage::get_u64("/last-yield-update").unwrap_or(current_time);
        
        // Calculate time elapsed in seconds
        if current_time <= last_update {
            return Ok(());  // No time passed or clock issues
        }
        
        let time_elapsed = current_time - last_update;
        if time_elapsed == 0 {
            return Ok(());  // No time passed
        }
        
        // Get current yield rate (in basis points)
        let yield_rate = storage::get_u128("/yield-rate").unwrap_or(0);
        if yield_rate == 0 {
            return Ok(());  // No yield to apply
        }
        
        // Get current total assets
        let total_assets = storage::get_u128("/total-assets").unwrap_or(0);
        if total_assets == 0 {
            return Ok(());  // No assets to apply yield to
        }
        
        // Calculate yield: assets * rate * timeElapsed / (10000 * 365 * 24 * 60 * 60)
        // Rate is in basis points (1/100 of a percent)
        // This gives a per-second compounding
        let yield_multiplier = yield_rate
            .checked_mul(time_elapsed as u128)
            .ok_or("Yield calculation overflow")?;
            
        // 10000 * seconds in a year
        let divisor: u128 = 10000 * 365 * 24 * 60 * 60;
        
        let yield_amount = total_assets
            .checked_mul(yield_multiplier)
            .ok_or("Yield amount overflow")?
            .checked_div(divisor)
            .ok_or("Yield division error")?;
            
        // Add yield to total assets
        if yield_amount > 0 {
            let new_total = total_assets
                .checked_add(yield_amount)
                .ok_or("Total assets overflow")?;
                
            storage::set_u128("/total-assets", new_total);
        }
        
        // Update the last yield timestamp
        storage::set_u64("/last-yield-update", current_time);
        
        Ok(())
    }
    
    /// Set the yield rate (only callable by authorized entities)
    fn handle_update_yield(&mut self, yield_rate: u128) -> Result<(), &'static str> {
        // Authorization would be checked here in a real contract
        // For this example, we'll allow anyone to update the yield rate
        
        // Update the yield first with the old rate
        self.update_yield()?;
        
        // Set the new yield rate
        storage::set_u128("/yield-rate", yield_rate);
        
        Ok(())
    }
    
    // == View Functions: Metadata ==
    
    /// Returns the name of the vault token
    fn handle_name(&self) -> String {
        storage::get_string("/name").unwrap_or_default()
    }
    
    /// Returns the symbol of the vault token
    fn handle_symbol(&self) -> String {
        storage::get_string("/symbol").unwrap_or_default()
    }
    
    /// Returns the decimals of the vault token
    fn handle_decimals(&self) -> u8 {
        storage::get_u8("/decimals").unwrap_or(18)
    }
    
    /// Returns the underlying asset name
    fn handle_asset(&self) -> String {
        storage::get_string("/asset-symbol").unwrap_or_default()
    }
    
    // == View Functions: Accounting ==
    
    /// Returns the total assets managed by the vault
    fn handle_total_assets(&self) -> u128 {
        // For a real contract, this would first update yield
        // But since this is a view function, we'll just return the current value
        storage::get_u128("/total-assets").unwrap_or(0)
    }
    
    /// Converts a given amount of assets to shares
    fn handle_convert_to_shares(&self, assets: u128) -> Result<u128, &'static str> {
        let total_assets = self.handle_total_assets();
        let total_supply = self.handle_total_supply();
        
        // If the vault is empty, use 1:1 ratio
        if total_assets == 0 || total_supply == 0 {
            return Ok(assets);
        }
        
        // shares = assets * totalSupply / totalAssets
        assets
            .checked_mul(total_supply)
            .ok_or("Convert to shares multiplication overflow")?
            .checked_div(total_assets)
            .ok_or("Convert to shares division error")
    }
    
    /// Converts a given amount of shares to assets
    fn handle_convert_to_assets(&self, shares: u128) -> Result<u128, &'static str> {
        let total_assets = self.handle_total_assets();
        let total_supply = self.handle_total_supply();
        
        // If the vault is empty, use 1:1 ratio
        if total_supply == 0 {
            return Ok(shares);
        }
        
        // assets = shares * totalAssets / totalSupply
        shares
            .checked_mul(total_assets)
            .ok_or("Convert to assets multiplication overflow")?
            .checked_div(total_supply)
            .ok_or("Convert to assets division error")
    }
    
    // == View Functions: Limits ==
    
    /// Returns the maximum amount of assets that can be deposited
    fn handle_max_deposit(&self, receiver: String) -> Result<u128, &'static str> {
        // In this simple implementation, we don't have a cap
        // In a real contract, this would be limited by various factors
        Ok(u128::MAX)
    }
    
    /// Returns the maximum amount of shares that can be minted
    fn handle_max_mint(&self, receiver: String) -> Result<u128, &'static str> {
        // In this simple implementation, we don't have a cap
        // In a real contract, this would be limited by various factors
        Ok(u128::MAX)
    }
    
    /// Returns the maximum amount of assets that can be withdrawn
    fn handle_max_withdraw(&self, owner: String) -> Result<u128, &'static str> {
        // Calculate based on owner's balance
        let balance = self.get_balance(&owner);
        self.handle_convert_to_assets(balance)
    }
    
    /// Returns the maximum amount of shares that can be redeemed
    fn handle_max_redeem(&self, owner: String) -> Result<u128, &'static str> {
        // Simply return the owner's balance
        Ok(self.get_balance(&owner))
    }
    
    // == View Functions: Preview ==
    
    /// Simulates the amount of shares that would be minted for a given deposit
    fn handle_preview_deposit(&self, assets: u128) -> Result<u128, &'static str> {
        self.handle_convert_to_shares(assets)
    }
    
    /// Simulates the amount of assets that would be required for a given mint
    fn handle_preview_mint(&self, shares: u128) -> Result<u128, &'static str> {
        let total_assets = self.handle_total_assets();
        let total_supply = self.handle_total_supply();
        
        // If the vault is empty, use 1:1 ratio
        if total_assets == 0 || total_supply == 0 {
            return Ok(shares);
        }
        
        // assets = shares * totalAssets / totalSupply
        // Round up by adding totalSupply - 1
        let numerator = shares
            .checked_mul(total_assets)
            .ok_or("Preview mint multiplication overflow")?;
            
        // Division with ceiling
        let add_amount = total_supply.checked_sub(1).unwrap_or(0);
        let with_ceiling = numerator.checked_add(add_amount).ok_or("Preview mint addition overflow")?;
        
        with_ceiling
            .checked_div(total_supply)
            .ok_or("Preview mint division error")
    }
    
    /// Simulates the amount of shares that would be burned for a given withdrawal
    fn handle_preview_withdraw(&self, assets: u128) -> Result<u128, &'static str> {
        let total_assets = self.handle_total_assets();
        let total_supply = self.handle_total_supply();
        
        // If the vault is empty, use 1:1 ratio
        if total_assets == 0 {
            return Ok(assets);
        }
        
        // shares = assets * totalSupply / totalAssets
        // Round up by adding totalAssets - 1
        let numerator = assets
            .checked_mul(total_supply)
            .ok_or("Preview withdraw multiplication overflow")?;
            
        // Division with ceiling
        let add_amount = total_assets.checked_sub(1).unwrap_or(0);
        let with_ceiling = numerator.checked_add(add_amount).ok_or("Preview withdraw addition overflow")?;
        
        with_ceiling
            .checked_div(total_assets)
            .ok_or("Preview withdraw division error")
    }
    
    /// Simulates the amount of assets that would be returned for a given redemption
    fn handle_preview_redeem(&self, shares: u128) -> Result<u128, &'static str> {
        self.handle_convert_to_assets(shares)
    }
    
    // == Custom Data Management ==
    
    /// Store custom data in the contract
    fn handle_set_data(&mut self, key: String, value: String) -> Result<(), &'static str> {
        // Prefix with /data/ to separate from core storage
        let storage_key = format!("/data/{}", key);
        storage::set_string(&storage_key, &value);
        Ok(())
    }
    
    /// Retrieve custom data from the contract
    fn handle_get_data(&self, key: String) -> String {
        // Prefix with /data/ to separate from core storage
        let storage_key = format!("/data/{}", key);
        storage::get_string(&storage_key).unwrap_or_default()
    }
    
    // == Balance and Supply Management ==
    
    /// Returns the share balance of an account
    fn handle_balance_of(&self, account: String) -> u128 {
        self.get_balance(&account)
    }
    
    /// Returns the total supply of shares
    fn handle_total_supply(&self) -> u128 {
        storage::get_u128("/total-supply").unwrap_or(0)
    }
    
    /// Helper to get account balance
    fn get_balance(&self, account: &str) -> u128 {
        let key = format!("/balances/{}", account);
        storage::get_u128(&key).unwrap_or(0)
    }
    
    /// Helper to set account balance
    fn set_balance(&self, account: &str, amount: u128) {
        let key = format!("/balances/{}", account);
        storage::set_u128(&key, amount);
    }
    
    /// Mint new shares to an account
    fn mint_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
        // Get current balance
        let balance = self.get_balance(account);
        
        // Calculate new balance
        let new_balance = balance
            .checked_add(amount)
            .ok_or("Balance overflow")?;
            
        // Update balance
        self.set_balance(account, new_balance);
        
        // Update total supply
        let supply = self.handle_total_supply();
        let new_supply = supply
            .checked_add(amount)
            .ok_or("Supply overflow")?;
            
        storage::set_u128("/total-supply", new_supply);
        
        Ok(())
    }
    
    /// Burn shares from an account
    fn burn_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
        // Get current balance
        let balance = self.get_balance(account);
        
        // Ensure sufficient balance
        if balance < amount {
            return Err("Insufficient balance");
        }
        
        // Calculate new balance
        let new_balance = balance - amount;
        
        // Update balance
        self.set_balance(account, new_balance);
        
        // Update total supply
        let supply = self.handle_total_supply();
        let new_supply = supply - amount;
        storage::set_u128("/total-supply", new_supply);
        
        Ok(())
    }
    
    /// Add to total assets
    fn add_total_assets(&self, amount: u128) -> Result<(), &'static str> {
        let total = self.handle_total_assets();
        let new_total = total
            .checked_add(amount)
            .ok_or("Total assets overflow")?;
            
        storage::set_u128("/total-assets", new_total);
        Ok(())
    }
    
    /// Subtract from total assets
    fn subtract_total_assets(&self, amount: u128) -> Result<(), &'static str> {
        let total = self.handle_total_assets();
        
        // Ensure sufficient assets
        if total < amount {
            return Err("Insufficient assets");
        }
        
        let new_total = total - amount;
        storage::set_u128("/total-assets", new_total);
        Ok(())
    }
    
    /// Get current timestamp (seconds since epoch)
    fn get_timestamp(&self) -> u64 {
        // In a real contract, this would use the blockchain timestamp
        // For simplicity, we'll mock it
        #[cfg(feature = "blockchain")]
        {
            alkanes_runtime::get_timestamp()
        }
        
        #[cfg(not(feature = "blockchain"))]
        {
            // Mock timestamp for testing
            1651388400 // May 1, 2022 12:00:00 PM UTC
        }
    }
}

// == WebAssembly Entry Point ==

#[no_mangle]
pub extern "C" fn call(opcode: i32, bytes: *mut u8, bytes_len: usize) -> *mut u8 {
    // Delegate to the MessageDispatch-generated code
    YieldVault::default().dispatch(opcode, bytes, bytes_len)
}
