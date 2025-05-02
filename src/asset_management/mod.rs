use metashrew_support::index_pointer::KeyValuePointer; // Add this import
use alkanes_support::utils::overflow_error;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use alkanes_support::context::Context;
use anyhow::{anyhow, Result};

use crate::storage::Storage;
use crate::security::Security;
use crate::utils::Conversion;

/// Asset management trait for the YieldVault
pub trait AssetManagement: Storage + Security + Conversion {
    /// Get the current context
    fn context(&self) -> Result<Context>;
    
    /// Get the current timestamp
    fn get_timestamp(&self) -> u64;

    /// Helper function to verify incoming assets
    fn verify_incoming_assets(&self, incoming_alkanes: &AlkaneTransferParcel) -> Result<u128, &'static str> {
        // Get the asset ID
        let asset_id = self.get_asset_id();
        
        // Sum all incoming assets with matching ID
        let received = incoming_alkanes.0.iter()
            .filter(|transfer| transfer.id == asset_id)
            .map(|transfer| transfer.value)
            .sum::<u128>();
            
        Ok(received)
    }
    
    /// Deposit assets and mint shares
    fn deposit(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        assets: u128
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Verify we received the correct amount of underlying assets
        let received_assets = self.verify_incoming_assets(&context.incoming_alkanes)
            .map_err(|e| anyhow!("Asset verification error: {}", e))?;
        
        // Check that we received at least the expected assets
        if received_assets < assets {
            return Err(anyhow!("Insufficient assets received: expected {}, got {}", 
                             assets, received_assets));
        }
        
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)
            .map_err(|e| anyhow!("Transaction validation error: {}", e))?;
        
        // Update the yield before any operations
        self.update_yield()
            .map_err(|e| anyhow!("Yield update error: {}", e))?;
        
        // Check deposit limit
        let max_deposit = self.max_deposit(&receiver)
            .map_err(|e| anyhow!("Max deposit check error: {}", e))?;
        if assets > max_deposit {
            return Err(anyhow!("Deposit amount exceeds limit"));
        }
        
        // Get current state
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        // Calculate shares from assets
        let shares = self.convert_assets_to_shares(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview deposit error: {}", e))?;
        if shares == 0 {
            return Err(anyhow!("Zero shares"));
        }
        
        // Update state
        self.add_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.mint_shares(&receiver, shares)
            .map_err(|e| anyhow!("Share mint error: {}", e))?;
        
        // Create share token transfer to receiver
        let share_transfer = AlkaneTransfer {
            id: context.myself.clone(), // Share token ID is this contract
            value: shares,
        };
        
        // Add to response
        response.alkanes.0.push(share_transfer);
        
        Ok(response)
    }
    
    /// Mint exact shares by depositing assets
    fn mint(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        shares: u128
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)
            .map_err(|e| anyhow!("Transaction validation error: {}", e))?;
        
        // Update the yield before any operations
        self.update_yield()
            .map_err(|e| anyhow!("Yield update error: {}", e))?;
        
        // Check mint limit
        let max_mint = self.max_mint(&receiver)
            .map_err(|e| anyhow!("Max mint check error: {}", e))?;
        if shares > max_mint {
            return Err(anyhow!("Mint amount exceeds limit"));
        }
        
        // Get current state
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        // Calculate assets needed for shares
        let assets = self.preview_mint(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview mint error: {}", e))?;
        if assets == 0 {
            return Err(anyhow!("Zero assets"));
        }
        
        // Verify we received the correct amount of underlying assets
        let received_assets = self.verify_incoming_assets(&context.incoming_alkanes)
            .map_err(|e| anyhow!("Asset verification error: {}", e))?;
        
        // Check that we received at least the expected assets
        if received_assets < assets {
            return Err(anyhow!("Insufficient assets received: expected {}, got {}", 
                              assets, received_assets));
        }
        
        // Update state
        self.add_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.mint_shares(&receiver, shares)
            .map_err(|e| anyhow!("Share mint error: {}", e))?;
        
        // Create share token transfer to receiver
        let share_transfer = AlkaneTransfer {
            id: context.myself.clone(), // Share token ID is this contract
            value: shares,
        };
        
        // Add to response
        response.alkanes.0.push(share_transfer);
        
        Ok(response)
    }
    
    /// Withdraw assets by burning shares
    fn withdraw(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        assets: u128
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)
            .map_err(|e| anyhow!("Transaction validation error: {}", e))?;
        
        // Update the yield before any operations
        self.update_yield()
            .map_err(|e| anyhow!("Yield update error: {}", e))?;
        
        // Check authorization
        self.check_authorization(&caller, &owner)
            .map_err(|e| anyhow!("Authorization error: {}", e))?;
        
        // Check withdrawal limit
        let max_withdraw = self.max_withdraw(&owner)
            .map_err(|e| anyhow!("Max withdraw check error: {}", e))?;
        if assets > max_withdraw {
            return Err(anyhow!("Withdrawal amount exceeds limit"));
        }
        
        // Get current state
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        // Calculate shares needed for assets
        let shares = self.preview_withdraw(assets, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview withdraw error: {}", e))?;
        if shares == 0 {
            return Err(anyhow!("Zero shares"));
        }
        
        // Update state
        self.subtract_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.burn_shares(&owner, shares)
            .map_err(|e| anyhow!("Share burn error: {}", e))?;
        
        // Get the asset ID for transferring out the assets
        let asset_id = self.get_asset_id();
        
        // Create asset transfer to receiver
        let asset_transfer = AlkaneTransfer {
            id: asset_id,
            value: assets,
        };
        
        // Add to response
        response.alkanes.0.push(asset_transfer);
        
        Ok(response)
    }
    
    /// Redeem shares for assets
    fn redeem(
        &self,
        tx_hash: String,
        caller: String,
        receiver: String,
        owner: String,
        shares: u128
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)
            .map_err(|e| anyhow!("Transaction validation error: {}", e))?;
        
        // Update the yield before any operations
        self.update_yield()
            .map_err(|e| anyhow!("Yield update error: {}", e))?;
        
        // Check authorization
        self.check_authorization(&caller, &owner)
            .map_err(|e| anyhow!("Authorization error: {}", e))?;
        
        // Check redemption limit
        let max_redeem = self.max_redeem(&owner)
            .map_err(|e| anyhow!("Max redeem check error: {}", e))?;
        if shares > max_redeem {
            return Err(anyhow!("Redemption amount exceeds limit"));
        }
        
        // Get current state
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        // Calculate assets for shares
        let assets = self.convert_shares_to_assets(shares, total_assets, total_supply)
            .map_err(|e| anyhow!("Preview redeem error: {}", e))?;
        
        // Update state
        self.subtract_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.burn_shares(&owner, shares)
            .map_err(|e| anyhow!("Share burn error: {}", e))?;
        
        // Get the asset ID for transferring out the assets
        let asset_id = self.get_asset_id();
        
        // Create asset transfer to receiver
        let asset_transfer = AlkaneTransfer {
            id: asset_id,
            value: assets,
        };
        
        // Add to response
        response.alkanes.0.push(asset_transfer);
        
        Ok(response)
    }
    
    /// Update the accumulated yield
    fn update_yield(&self) -> Result<(), &'static str> {
        let current_time = self.get_timestamp();
        let last_update = self.last_yield_update_pointer().get_value::<u64>();
        
        // Calculate time elapsed in seconds
        if current_time <= last_update {
            return Ok(());  // No time passed or clock issues
        }
        
        let time_elapsed = current_time - last_update;
        if time_elapsed == 0 {
            return Ok(());  // No time passed
        }
        
        // Get current yield rate (in basis points)
        let yield_rate = self.yield_rate_pointer().get_value::<u128>();
        if yield_rate == 0 {
            return Ok(());  // No yield to apply
        }
        
        // Get current total assets
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        if total_assets == 0 {
            return Ok(());  // No assets to apply yield to
        }
        
        // Calculate yield: assets * rate * timeElapsed / YIELD_CALCULATION_DENOMINATOR
        // Rate is in basis points (1/100 of a percent)
        // This gives a per-second compounding
        let yield_multiplier = overflow_error(yield_rate.checked_mul(time_elapsed as u128))
            .map_err(|_| "Yield calculation overflow")?;
            
        // Use constant for the yield calculation denominator (BASIS_POINTS_DENOMINATOR * SECONDS_PER_YEAR)
        let yield_amount = overflow_error(total_assets.checked_mul(yield_multiplier))
            .map_err(|_| "Yield amount overflow")?
            .checked_div(crate::utils::YIELD_CALCULATION_DENOMINATOR)
            .ok_or("Yield division error")?;
            
        // Add yield to total assets
        if yield_amount > 0 {
            let new_total = overflow_error(total_assets.checked_add(yield_amount))
                .map_err(|_| "Total assets overflow")?;
                
            self.total_assets_pointer().set_value(new_total);
        }
        
        // Update the last yield timestamp
        self.last_yield_update_pointer().set_value(current_time);
        
        Ok(())
    }

    /// Add to total assets
    fn add_total_assets(&self, amount: u128) -> Result<(), &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let new_total = overflow_error(total_assets.checked_add(amount))
            .map_err(|_| "Total assets overflow when adding assets")?;
        self.total_assets_pointer().set_value(new_total);
        Ok(())
    }
    
    /// Subtract from total assets
    fn subtract_total_assets(&self, amount: u128) -> Result<(), &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        
        // Ensure sufficient assets
        if total_assets < amount {
            return Err("Insufficient total assets for subtraction");
        }
        
        let new_total = total_assets - amount;
        self.total_assets_pointer().set_value(new_total);
        Ok(())
    }

    /// Mint shares to an account
    fn mint_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
        // Get current balance
        let balance = self.get_balance(account);
        
        // Calculate new balance
        let new_balance = overflow_error(balance.checked_add(amount))
            .map_err(|_| "Balance overflow when minting shares")?;
            
        // Update account balance
        self.set_balance(account, new_balance);
        
        // Update total supply
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        let new_supply = overflow_error(total_supply.checked_add(amount))
            .map_err(|_| "Total supply overflow when minting shares")?;
        self.total_supply_pointer().set_value(new_supply);
        
        Ok(())
    }
    
    /// Burn shares from an account
    fn burn_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
        // Get current balance
        let balance = self.get_balance(account);
        
        // Ensure sufficient balance
        if balance < amount {
            return Err("Insufficient balance for burning shares");
        }
        
        // Calculate new balance
        let new_balance = balance - amount;
        self.set_balance(account, new_balance);
        
        // Update total supply
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        let new_supply = total_supply.checked_sub(amount)
            .ok_or("Total supply underflow when burning shares")?;
        self.total_supply_pointer().set_value(new_supply);
        
        Ok(())
    }

    // == Limit Calculation Functions ==
    
    /// Calculate maximum deposit amount for a receiver
    fn max_deposit(&self, _receiver: &str) -> Result<u128, &'static str> {
        // In this implementation, there's no limit on deposits
        // In real implementations, this might check against a cap or other constraints
        Ok(u128::MAX)
    }
    
    /// Calculate maximum mint amount for a receiver
    fn max_mint(&self, _receiver: &str) -> Result<u128, &'static str> {
        // In this implementation, there's no limit on mints
        Ok(u128::MAX)
    }
    
    /// Calculate maximum withdraw amount for an owner
    fn max_withdraw(&self, owner: &str) -> Result<u128, &'static str> {
        // Can withdraw at most the assets corresponding to owned shares
        let shares = self.get_balance(owner);
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        self.convert_shares_to_assets(shares, total_assets, total_supply)
    }
    
    /// Calculate maximum redeem amount for an owner
    fn max_redeem(&self, owner: &str) -> Result<u128, &'static str> {
        // Can redeem at most the owned shares
        Ok(self.get_balance(owner))
    }
}
