//! Mock implementation of YieldVault for testing
//! This implementation doesn't rely on external dependencies

use std::collections::HashMap;
use std::cell::RefCell;
use anyhow::Result;

/// MockYieldVault is a simplified implementation for testing
/// that doesn't rely on external dependencies like alkanes-runtime
pub struct MockYieldVault {
    // In-memory storage
    storage: RefCell<HashMap<String, Vec<u8>>>,
}

impl Default for MockYieldVault {
    fn default() -> Self {
        let mut vault = Self {
            storage: RefCell::new(HashMap::new()),
        };

        // Initialize with default values
        vault.set_value("total_assets", 0u128);
        vault.set_value("total_supply", 0u128);
        vault.set_value("decimals", 8u8);
        vault.set_value("yield_rate", 0u128);
        
        vault
    }
}

impl MockYieldVault {
    /// Create a new vault with initial values
    pub fn new(total_assets: u128, total_supply: u128) -> Self {
        let mut vault = Self::default();
        vault.set_value("total_assets", total_assets);
        vault.set_value("total_supply", total_supply);
        vault
    }

    /// Helper to set a value in storage
    fn set_value<T: serde::Serialize>(&self, key: &str, value: T) {
        let value_bytes = bincode::serialize(&value).unwrap_or_default();
        self.storage.borrow_mut().insert(key.to_string(), value_bytes);
    }

    /// Helper to get a value from storage
    fn get_value<T: serde::de::DeserializeOwned>(&self, key: &str) -> T 
    where T: Default {
        if let Some(value_bytes) = self.storage.borrow().get(key) {
            bincode::deserialize(value_bytes).unwrap_or_default()
        } else {
            T::default()
        }
    }

    /// Initialize the vault with the given parameters
    pub fn initialize(
        &self,
        name: String, 
        symbol: String,
        asset_name: String,
        asset_symbol: String,
        decimals: u8
    ) -> Result<()> {
        self.set_value("name", name);
        self.set_value("symbol", symbol);
        self.set_value("asset_name", asset_name);
        self.set_value("asset_symbol", asset_symbol);
        self.set_value("decimals", decimals);
        self.set_value("initialized", true);
        
        Ok(())
    }

    /// Get vault name
    pub fn get_name(&self) -> Result<String> {
        Ok(self.get_value("name"))
    }
    
    /// Get vault symbol
    pub fn get_symbol(&self) -> Result<String> {
        Ok(self.get_value("symbol"))
    }
    
    /// Get decimals
    pub fn get_decimals(&self) -> Result<u8> {
        Ok(self.get_value("decimals"))
    }

    /// Get total assets
    pub fn get_total_assets(&self) -> u128 {
        self.get_value("total_assets")
    }

    /// Get total supply
    pub fn get_total_supply(&self) -> u128 {
        self.get_value("total_supply")
    }

    /// Set total assets
    pub fn set_total_assets(&self, value: u128) {
        self.set_value("total_assets", value);
    }

    /// Set total supply
    pub fn set_total_supply(&self, value: u128) {
        self.set_value("total_supply", value);
    }

    /// Get balance for an account
    pub fn get_balance(&self, account: &str) -> u128 {
        self.get_value(&format!("balance_{}", account))
    }

    /// Set balance for an account
    pub fn set_balance(&self, account: &str, value: u128) {
        self.set_value(&format!("balance_{}", account), value);
    }

    /// Convert assets to shares
    pub fn convert_assets_to_shares(&self, assets: u128, total_assets: u128, total_supply: u128) -> Result<u128, &'static str> {
        // Empty vault case - 1:1 ratio
        if total_assets == 0 || total_supply == 0 {
            return Ok(assets);
        }
        
        // Calculate shares based on the ratio of assets to total_assets
        // shares = assets * total_supply / total_assets
        total_supply
            .checked_mul(assets)
            .map(|v| v / total_assets)
            .ok_or("Math overflow in assets to shares conversion")
    }
    
    /// Convert shares to assets
    pub fn convert_shares_to_assets(&self, shares: u128, total_assets: u128, total_supply: u128) -> Result<u128, &'static str> {
        // Empty vault case - return 0 assets as there are none
        if total_supply == 0 {
            return Ok(0);
        }
        
        // Calculate assets based on the ratio of shares to total_supply
        // assets = shares * total_assets / total_supply
        total_assets
            .checked_mul(shares)
            .map(|v| v / total_supply)
            .ok_or("Math overflow in shares to assets conversion")
    }

    /// Preview deposit - calculates shares to be minted for a given asset amount
    pub fn preview_deposit(&self, assets: u128) -> Result<u128, &'static str> {
        let total_assets = self.get_total_assets();
        let total_supply = self.get_total_supply();
        self.convert_assets_to_shares(assets, total_assets, total_supply)
    }
    
    /// Preview redeem - calculates assets to be withdrawn for a given share amount
    pub fn preview_redeem(&self, shares: u128) -> Result<u128, &'static str> {
        let total_assets = self.get_total_assets();
        let total_supply = self.get_total_supply();
        self.convert_shares_to_assets(shares, total_assets, total_supply)
    }

    /// Simplified deposit implementation
    pub fn deposit(&self, caller: &str, receiver: &str, assets: u128) -> Result<u128, &'static str> {
        // Get current state
        let total_assets = self.get_total_assets();
        let total_supply = self.get_total_supply();
        
        // Calculate shares
        let shares = self.convert_assets_to_shares(assets, total_assets, total_supply)?;
        
        // Update state
        self.set_total_assets(total_assets + assets);
        self.set_total_supply(total_supply + shares);
        
        // Update receiver balance
        let receiver_balance = self.get_balance(receiver);
        self.set_balance(receiver, receiver_balance + shares);
        
        Ok(shares)
    }

    /// Simplified redeem implementation
    pub fn redeem(&self, caller: &str, receiver: &str, owner: &str, shares: u128) -> Result<u128, &'static str> {
        // Get current state
        let total_assets = self.get_total_assets();
        let total_supply = self.get_total_supply();
        
        // Check owner balance
        let owner_balance = self.get_balance(owner);
        if owner_balance < shares {
            return Err("Insufficient balance");
        }
        
        // Calculate assets
        let assets = self.convert_shares_to_assets(shares, total_assets, total_supply)?;
        
        // Update state
        self.set_total_assets(total_assets - assets);
        self.set_total_supply(total_supply - shares);
        
        // Update owner balance
        self.set_balance(owner, owner_balance - shares);
        
        Ok(assets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_vault_initialization() {
        let vault = MockYieldVault::default();
        assert_eq!(vault.get_total_assets(), 0);
        assert_eq!(vault.get_total_supply(), 0);
    }

    #[test]
    fn test_mock_vault_conversion() {
        // Create vault with 1000 assets and 500 shares (2:1 ratio)
        let vault = MockYieldVault::new(1000, 500);
        
        // 100 assets should convert to 50 shares
        let shares = vault.convert_assets_to_shares(100, 1000, 500).unwrap();
        assert_eq!(shares, 50);
        
        // 50 shares should convert to 100 assets
        let assets = vault.convert_shares_to_assets(50, 1000, 500).unwrap();
        assert_eq!(assets, 100);
    }

    #[test]
    fn test_mock_vault_deposit() {
        let vault = MockYieldVault::default();
        
        // Deposit 100 assets to user1
        let shares = vault.deposit("caller", "user1", 100).unwrap();
        assert_eq!(shares, 100); // 1:1 ratio when empty
        
        // Check state was updated
        assert_eq!(vault.get_total_assets(), 100);
        assert_eq!(vault.get_total_supply(), 100);
        assert_eq!(vault.get_balance("user1"), 100);
        
        // Deposit more with non-empty vault
        // Now we have 100 assets and 100 shares (1:1 ratio)
        let shares = vault.deposit("caller", "user2", 100).unwrap();
        assert_eq!(shares, 100);
        
        // Check final state
        assert_eq!(vault.get_total_assets(), 200);
        assert_eq!(vault.get_total_supply(), 200);
        assert_eq!(vault.get_balance("user1"), 100);
        assert_eq!(vault.get_balance("user2"), 100);
    }

    #[test]
    fn test_mock_vault_redeem() {
        // Create vault with initial state
        let vault = MockYieldVault::new(200, 100); // 2:1 ratio
        vault.set_balance("user1", 50);
        vault.set_balance("user2", 50);
        
        // Redeem 20 shares from user1 
        let assets = vault.redeem("caller", "receiver", "user1", 20).unwrap();
        assert_eq!(assets, 40); // 2:1 ratio
        
        // Check state was updated
        assert_eq!(vault.get_total_assets(), 160);
        assert_eq!(vault.get_total_supply(), 80);
        assert_eq!(vault.get_balance("user1"), 30);
        assert_eq!(vault.get_balance("user2"), 50);
    }
}
