use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_support::utils::overflow_error;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use alkanes_runtime::runtime::AlkaneResponder;
use anyhow::{anyhow, Result};

use crate::storage::Storage;
use crate::security::Security;
use crate::utils::Conversion;

/// Asset management trait for the YieldVault
pub trait AssetManagement: Storage + Security + Conversion + AlkaneResponder {
    /// Get the current block height
    fn get_block_height(&self) -> u64 {
        self.height()
    }

    /// Helper function to verify incoming assets
    fn verify_incoming_assets(&self, incoming_alkanes: &AlkaneTransferParcel) -> Result<u128, &'static str> {
        // Get the asset ID
        let asset_id = self.get_asset_id();
        
        // Check if there are any incoming assets at all
        if incoming_alkanes.0.is_empty() {
            return Err("No assets provided in transaction");
        }
        
        // Sum all incoming assets with matching ID
        let mut received = 0u128;
        
        for transfer in &incoming_alkanes.0 {
            // Direct comparison of AlkaneId structs
            // This works because AlkaneId implements PartialEq
            if transfer.id == asset_id {
                received = received.checked_add(transfer.value)
                    .ok_or("Asset amount overflow")?;
            }
        }
            
        // If no assets match our asset ID, this is an error
        if received == 0 {
            return Err("Invalid asset ID: received assets do not match expected asset type");
        }
            
        Ok(received)
    }
    
    /// Deposit assets and mint shares
    fn deposit(
        &self,
        tx_hash: String,
        _caller: String,
        receiver: String,
        assets: u128
    ) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
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
        
        // Update global state only
        self.add_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.add_total_supply(shares)
            .map_err(|e| anyhow!("Total supply update error: {}", e))?;
        
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
        _caller: String,
        receiver: String,
        shares: u128
    ) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
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
        
        // Update global state only
        self.add_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.add_total_supply(shares)
            .map_err(|e| anyhow!("Total supply update error: {}", e))?;
        
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
        _caller: String,
        _receiver: String,
        owner: String,
        assets: u128
    ) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)
            .map_err(|e| anyhow!("Transaction validation error: {}", e))?;
        
        // Update the yield before any operations
        self.update_yield()
            .map_err(|e| anyhow!("Yield update error: {}", e))?;
            
        // Get the asset ID for transferring out the assets
        let asset_id = self.get_asset_id();
        
        // Verify that if there are any incoming assets, at least one has the correct ID
        // Do this check early to prevent burning shares if asset ID is incorrect
        // Note: For redeem operation, incoming assets are NOT required
        if !context.incoming_alkanes.0.is_empty() {
            let valid_asset_found = context.incoming_alkanes.0.iter()
                .any(|transfer| transfer.id == asset_id);
                
            if !valid_asset_found {
                return Err(anyhow!("Invalid asset ID: received assets do not match expected asset type"));
            }
        }
        
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
        
        // Verify incoming shares
        let received_shares = self.verify_incoming_shares(&context.incoming_alkanes)
            .map_err(|e| anyhow!("Share verification error: {}", e))?;
        
        // Check that we received at least the expected shares
        if received_shares < shares {
            return Err(anyhow!("Insufficient shares received: expected {}, got {}", 
                             shares, received_shares));
        }
        
        // Update global state only
        self.subtract_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.subtract_total_supply(shares)
            .map_err(|e| anyhow!("Total supply update error: {}", e))?;
        
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
        _caller: String,
        _receiver: String,
        owner: String,
        shares: u128
    ) -> Result<CallResponse> {
        let context = AlkaneResponder::context(self)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        
        // Validate the transaction
        self.validate_and_track_transaction(&tx_hash)
            .map_err(|e| anyhow!("Transaction validation error: {}", e))?;
        
        // Update the yield before any operations
        self.update_yield()
            .map_err(|e| anyhow!("Yield update error: {}", e))?;
            
        // Get the asset ID for transferring out the assets
        let asset_id = self.get_asset_id();
        
        // Verify that if there are any incoming assets, at least one has the correct ID
        // Do this check early to prevent burning shares if asset ID is incorrect
        if !context.incoming_alkanes.0.is_empty() {
            let valid_asset_found = context.incoming_alkanes.0.iter()
                .any(|transfer| transfer.id == asset_id);
                
            if !valid_asset_found {
                return Err(anyhow!("Invalid asset ID: received assets do not match expected asset type"));
            }
        }
        
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
        
        // Verify incoming shares
        let received_shares = self.verify_incoming_shares(&context.incoming_alkanes)
            .map_err(|e| anyhow!("Share verification error: {}", e))?;
        
        // Check that we received at least the expected shares
        if received_shares < shares {
            return Err(anyhow!("Insufficient shares received: expected {}, got {}", 
                             shares, received_shares));
        }
        
        // Update global state only
        self.subtract_total_assets(assets)
            .map_err(|e| anyhow!("Asset update error: {}", e))?;
        self.subtract_total_supply(shares)
            .map_err(|e| anyhow!("Total supply update error: {}", e))?;
        
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
        let current_height = self.get_block_height();
        let last_update_height = self.last_yield_height_pointer().get_value::<u64>();
        
        // Calculate blocks elapsed
        if current_height <= last_update_height {
            return Ok(());  // No blocks passed or replay protection
        }
        
        let blocks_elapsed = current_height - last_update_height;
        if blocks_elapsed == 0 {
            return Ok(());  // No blocks passed
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
        
        // Calculate yield: assets * rate * blocks_elapsed / YIELD_CALCULATION_DENOMINATOR
        // Rate is in basis points (1/100 of a percent)
        // This gives a per-block compounding
        let yield_multiplier = overflow_error(yield_rate.checked_mul(blocks_elapsed as u128))
            .map_err(|_| "Yield calculation overflow")?;
            
        // Use constant for the yield calculation denominator (BASIS_POINTS_DENOMINATOR * BLOCKS_PER_YEAR)
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
        
        // Update the last yield height
        self.last_yield_height_pointer().set_value(current_height);
        
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

    /// Add to total supply
    fn add_total_supply(&self, amount: u128) -> Result<(), &'static str> {
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        let new_supply = overflow_error(total_supply.checked_add(amount))
            .map_err(|_| "Total supply overflow")?;
        self.total_supply_pointer().set_value(new_supply);
        Ok(())
    }
    
    /// Subtract from total supply
    fn subtract_total_supply(&self, amount: u128) -> Result<(), &'static str> {
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        if total_supply < amount {
            return Err("Insufficient total supply");
        }
        let new_supply = total_supply - amount;
        self.total_supply_pointer().set_value(new_supply);
        Ok(())
    }
    
    /// Verify incoming shares in the transaction
    fn verify_incoming_shares(&self, incoming_alkanes: &AlkaneTransferParcel) -> Result<u128, &'static str> {
        let context = match AlkaneResponder::context(self) {
            Ok(ctx) => ctx,
            Err(_) => return Err("Failed to get context"),
        };
        
        // Get the contract ID (share token ID)
        let contract_id = context.myself.clone();
        
        // Check if there are any incoming alkanes at all
        if incoming_alkanes.0.is_empty() {
            return Err("No shares provided in transaction");
        }
        
        // Log the incoming shares for debugging
        #[cfg(debug_assertions)]
        {
            // In a real implementation, we would log the incoming shares
            // For example: log!("Incoming shares: {:?}", incoming_alkanes);
        }
        
        // Sum all incoming shares with matching ID
        let mut received = 0u128;
        
        for transfer in &incoming_alkanes.0 {
            // Direct comparison of AlkaneId structs
            // This works because AlkaneId implements PartialEq
            if transfer.id == contract_id {
                received = received.checked_add(transfer.value)
                    .ok_or("Share amount overflow")?;
            }
        }
        
        // If no shares match our contract ID, this is an error
        if received == 0 {
            return Err("Invalid share token ID: received shares do not match expected type");
        }
            
        Ok(received)
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
    fn max_withdraw(&self, _owner: &str) -> Result<u128, &'static str> {
        // In the token-based model, the contract doesn't track individual balances
        // The maximum is determined by the tokens presented in the transaction
        // For simplicity, we return the maximum possible value
        Ok(u128::MAX)
    }
    
    /// Calculate maximum redeem amount for an owner
    fn max_redeem(&self, _owner: &str) -> Result<u128, &'static str> {
        // In the token-based model, the contract doesn't track individual balances
        // The maximum is determined by the tokens presented in the transaction
        // For simplicity, we return the maximum possible value
        Ok(u128::MAX)
    }
}
