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
    // Simulated block height for yield calculations
    current_block_height: RefCell<u64>,
}

impl Default for MockYieldVault {
    fn default() -> Self {
        let vault = Self {
            storage: RefCell::new(HashMap::new()),
            current_block_height: RefCell::new(1000), // Start at block 1000
        };

        // Initialize with default values
        vault.set_value("total_assets", 0u128);
        vault.set_total_issuance(0u128); // Total issuance instead of supply
        vault.set_value("decimals", 8u8);
        vault.set_value("yield_rate", 500u128); // 5% annual yield (500 basis points)
        vault.set_value("last_yield_height", 1000u64); // Starting block height
        vault.set_value("initialized", false); // Not initialized yet
        
        vault
    }
}

impl MockYieldVault {
    /// Create a new vault with initial values
    pub fn new(total_assets: u128, total_issuance: u128) -> Self {
        let vault = Self::default();
        vault.set_value("total_assets", total_assets);
        vault.set_total_issuance(total_issuance);
        vault
    }
    
    /// Check if the vault is initialized
    fn is_initialized(&self) -> bool {
        self.get_value("initialized")
    }
    
    /// Set the current block height (for testing yield accrual)
    /// This can only be called before initialization
    pub fn set_block_height(&self, height: u64) {
        if !self.is_initialized() {
            *self.current_block_height.borrow_mut() = height;
        }
    }
    
    /// Get current block height
    pub fn get_block_height(&self) -> u64 {
        *self.current_block_height.borrow()
    }

    /// Helper to set a value in storage
    pub fn set_value<T: serde::Serialize>(&self, key: &str, value: T) {
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
        // Check if already initialized
        if self.is_initialized() {
            return Err(anyhow::anyhow!("Already initialized"));
        }
        
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

    /// Get total issuance (total supply)
    pub fn get_total_issuance(&self) -> u128 {
        self.get_value("total_issuance")
    }

    /// Set total issuance
    pub fn set_total_issuance(&self, value: u128) {
        self.set_value("total_issuance", value);
    }
    
    /// Get yield rate in basis points (1 bp = 0.01%)
    pub fn get_yield_rate(&self) -> u128 {
        self.get_value("yield_rate")
    }
    
    /// Set yield rate in basis points
    /// This can only be called before initialization
    pub fn set_yield_rate(&self, rate: u128) {
        if !self.is_initialized() {
            self.set_value("yield_rate", rate);
        }
    }
    
    /// Get last yield update height
    pub fn get_last_yield_height(&self) -> u64 {
        self.get_value("last_yield_height")
    }
    
    /// Set last yield update height
    fn set_last_yield_height(&self, height: u64) {
        self.set_value("last_yield_height", height);
    }
    
    /// In token-based model, this is for test convenience only
    /// It simulates the presence of tokens in the transaction context
    pub fn get_token_balance(&self, token_id: &str) -> u128 {
        self.get_value(&format!("token_{}", token_id))
    }

    /// In token-based model, this is for test convenience only
    /// It simulates issuing tokens to accounts for testing
    /// This can be called at any time for testing purposes
    pub fn issue_tokens(&self, token_id: &str, amount: u128) {
        let current = self.get_token_balance(token_id);
        self.set_value(&format!("token_{}", token_id), current + amount);
    }
    
    /// Simulate tokens in a transaction for testing
    pub fn simulate_transaction_tokens(&self, token_id: &str, amount: u128) {
        self.set_value("tx_token_id", token_id.as_bytes().to_vec());
        self.set_value("tx_token_amount", amount);
    }
    
    /// Get tokens in the current simulated transaction
    pub fn get_transaction_tokens(&self) -> (String, u128) {
        let token_id = String::from_utf8(self.get_value::<Vec<u8>>("tx_token_id")).unwrap_or_default();
        let amount = self.get_value::<u128>("tx_token_amount");
        (token_id, amount)
    }

    /// Convert assets to tokens (shares)
    pub fn convert_assets_to_tokens(&self, assets: u128, total_assets: u128, total_issuance: u128) -> Result<u128, &'static str> {
        // Empty vault case - 1:1 ratio
        if total_assets == 0 || total_issuance == 0 {
            return Ok(assets);
        }
        
        // Calculate tokens based on the ratio of assets to total_assets
        // tokens = assets * total_issuance / total_assets
        total_issuance
            .checked_mul(assets)
            .map(|v| v / total_assets)
            .ok_or("Math overflow in assets to tokens conversion")
    }
    
    /// Convert tokens to assets
    pub fn convert_tokens_to_assets(&self, tokens: u128, total_assets: u128, total_issuance: u128) -> Result<u128, &'static str> {
        // Empty vault case - return 0 assets as there are none
        if total_issuance == 0 {
            return Ok(0);
        }
        
        // Calculate assets based on the ratio of tokens to total_issuance
        // assets = tokens * total_assets / total_issuance
        total_assets
            .checked_mul(tokens)
            .map(|v| v / total_issuance)
            .ok_or("Math overflow in tokens to assets conversion")
    }
    
    /// Update yield based on elapsed block height
    pub fn update_yield(&self) -> Result<(), &'static str> {
        let current_height = self.get_block_height();
        let last_height = self.get_last_yield_height();
        
        if current_height <= last_height {
            // No yield if no blocks have passed
            return Ok(());
        }
        
        let blocks_elapsed = current_height - last_height;
        let yield_rate = self.get_yield_rate();
        
        // Very simplified yield calculation: 
        // 1 block = ~1 second, 31536000 seconds in a year
        // annual_yield_bps / 10000 = annual_yield_percent
        // yield_per_block = annual_yield_percent / 31536000
        
        let total_assets = self.get_total_assets();
        
        // Calculate yield for elapsed blocks (simplified)
        let yield_amount = total_assets
            .checked_mul(yield_rate)
            .ok_or("Overflow in yield calculation")?
            .checked_mul(blocks_elapsed as u128)
            .ok_or("Overflow in yield calculation")?
            / 10000 // Convert from basis points
            / 31536000; // Annualized to per-block
        
        // Update total assets with accrued yield
        self.set_value("total_assets", total_assets + yield_amount);
        
        // Update last yield height
        self.set_last_yield_height(current_height);
        
        Ok(())
    }

    /// Preview deposit - calculates tokens to be minted for a given asset amount
    pub fn preview_deposit(&self, assets: u128) -> Result<u128, &'static str> {
        // Update yield first
        self.update_yield()?;
        
        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        self.convert_assets_to_tokens(assets, total_assets, total_issuance)
    }
    
    /// Preview redeem - calculates assets to be withdrawn for a given token amount
    pub fn preview_redeem(&self, tokens: u128) -> Result<u128, &'static str> {
        // Update yield first
        self.update_yield()?;
        
        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        self.convert_tokens_to_assets(tokens, total_assets, total_issuance)
    }

    /// Simplified deposit implementation
    pub fn deposit(&self, _caller_token_id: &str, receiver_token_id: &str, assets: u128) -> Result<u128, &'static str> {
        // Validate non-zero assets
        if assets == 0 {
            return Err("Cannot deposit zero assets");
        }
        
        // Update yield accrual first
        self.update_yield()?;
        
        // Get current state
        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        
        // Calculate tokens to mint
        let tokens = self.convert_assets_to_tokens(assets, total_assets, total_issuance)?;
        if tokens == 0 {
            return Err("Zero tokens minted");
        }
        
        // Update state
        self.set_value("total_assets", total_assets + assets);
        self.set_total_issuance(total_issuance + tokens);
        
        // Issue tokens to receiver (simulating token transfer)
        let current = self.get_token_balance(receiver_token_id);
        self.set_value(&format!("token_{}", receiver_token_id), current + tokens);
        
        Ok(tokens)
    }

    /// Simplified redeem implementation
    pub fn redeem(&self, caller_token_id: &str, _receiver_token_id: &str, owner_token_id: &str, tokens: u128) -> Result<u128, &'static str> {
        // For testing purposes, we'll check if caller == owner
        // But only if the caller is not a special test transaction ID
        if caller_token_id != owner_token_id && 
           !caller_token_id.starts_with("tx") {
            return Err("Caller not authorized");
        }
        
        // Update yield accrual first
        self.update_yield()?;
        
        // Get current state
        let total_assets = self.get_total_assets();
        let total_issuance = self.get_total_issuance();
        
        // Check token ownership - in real implementation, this would be verified by the transaction context
        // Here we simulate by checking our token balance tracking
        let owner_tokens = self.get_token_balance(owner_token_id);
        if owner_tokens < tokens {
            return Err("Insufficient tokens");
        }
        
        // Calculate assets
        let assets = self.convert_tokens_to_assets(tokens, total_assets, total_issuance)?;
        
        // Update state
        self.set_value("total_assets", total_assets - assets);
        self.set_total_issuance(total_issuance - tokens);
        
        // Update token balances (burn owner's tokens)
        self.set_value(&format!("token_{}", owner_token_id), owner_tokens - tokens);
        
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
        assert_eq!(vault.get_total_issuance(), 0);
        
        // Set yield rate before initialization
        vault.set_yield_rate(1000);
        assert_eq!(vault.get_yield_rate(), 1000);
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Try to change yield rate after initialization - should have no effect
        vault.set_yield_rate(2000);
        assert_eq!(vault.get_yield_rate(), 1000); // Still 1000, not changed
    }

    #[test]
    fn test_mock_vault_conversion() {
        // Create vault with 1000 assets and 500 tokens (2:1 ratio)
        let vault = MockYieldVault::new(1000, 500);
        
        // 100 assets should convert to 50 tokens
        let tokens = vault.convert_assets_to_tokens(100, 1000, 500).unwrap();
        assert_eq!(tokens, 50);
        
        // 50 tokens should convert to 100 assets
        let assets = vault.convert_tokens_to_assets(50, 1000, 500).unwrap();
        assert_eq!(assets, 100);
    }

    #[test]
    fn test_mock_vault_deposit() {
        let vault = MockYieldVault::default();
        
        // Deposit 100 assets to user1
        let tokens = vault.deposit("caller", "user1", 100).unwrap();
        assert_eq!(tokens, 100); // 1:1 ratio when empty
        
        // Check state was updated
        assert_eq!(vault.get_total_assets(), 100);
        assert_eq!(vault.get_total_issuance(), 100);
        assert_eq!(vault.get_token_balance("user1"), 100);
        
        // Deposit more with non-empty vault
        // Now we have 100 assets and 100 tokens (1:1 ratio)
        let tokens = vault.deposit("caller", "user2", 100).unwrap();
        assert_eq!(tokens, 100);
        
        // Check final state
        assert_eq!(vault.get_total_assets(), 200);
        assert_eq!(vault.get_total_issuance(), 200);
        assert_eq!(vault.get_token_balance("user1"), 100);
        assert_eq!(vault.get_token_balance("user2"), 100);
    }

    #[test]
    fn test_mock_vault_redeem() {
        // Create vault with initial state
        let vault = MockYieldVault::new(200, 100); // 2:1 ratio
        
        // Setup token balances for testing - must be done before initialization
        vault.issue_tokens("user1", 50);
        vault.issue_tokens("user2", 50);
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Redeem 20 tokens from user1 
        let assets = vault.redeem("caller", "receiver", "user1", 20).unwrap();
        assert_eq!(assets, 40); // 2:1 ratio
        
        // Check state was updated
        assert_eq!(vault.get_total_assets(), 160);
        assert_eq!(vault.get_total_issuance(), 80);
        assert_eq!(vault.get_token_balance("user1"), 30);
        assert_eq!(vault.get_token_balance("user2"), 50);
    }
    
    #[test]
    fn test_yield_accrual() {
        // Create vault with initial state
        let vault = MockYieldVault::new(10000, 10000); // 1:1 ratio
        vault.set_yield_rate(1000); // 10% annual yield (1000 basis points)
        vault.set_block_height(1000);
        vault.set_value("last_yield_height", 1000u64);
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Manually update the block height for testing
        // In a real implementation, this would come from the blockchain
        *vault.current_block_height.borrow_mut() = 1000 + 8766;
        
        // Update yield
        vault.update_yield().unwrap();
        
        // Expected yield: 10000 * 10% * (8766/31536000) ≈ 2.78 units
        // With integer division and rounding, it should be around 2-3 units
        let total_assets = vault.get_total_assets();
        assert!(total_assets > 10000 && total_assets <= 10003, 
                "Expected yield accrual of 2-3 units, got: {}", total_assets - 10000);
        
        // Manually update the block height again
        *vault.current_block_height.borrow_mut() = 1000 + 8766 + 31_536_000;
        
        // Update yield
        vault.update_yield().unwrap();
        
        // Now should have accrued approximately 10% more on the principal + previous yield
        let new_total_assets = vault.get_total_assets();
        let expected_min = (total_assets * 110 / 100) - 5; // Allow some rounding error
        let expected_max = (total_assets * 110 / 100) + 5;
        
        assert!(new_total_assets >= expected_min && new_total_assets <= expected_max,
                "Expected ~10% yield, got: {}%", 
                (new_total_assets - total_assets) * 100 / total_assets);
    }
    
    #[test]
    fn test_immutability_after_initialization() {
        let vault = MockYieldVault::default();
        
        // Set values before initialization
        vault.set_yield_rate(1000);
        vault.set_block_height(2000);
        vault.issue_tokens("user1", 100);
        
        // Verify values were set
        assert_eq!(vault.get_yield_rate(), 1000);
        assert_eq!(vault.get_block_height(), 2000);
        assert_eq!(vault.get_token_balance("user1"), 100);
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "TEST".to_string(),
            "Test Asset".to_string(),
            "ASSET".to_string(),
            8
        ).unwrap();
        
        // Try to change values after initialization
        vault.set_yield_rate(2000);
        vault.set_block_height(3000);
        vault.issue_tokens("user1", 100); // This should work now
        
        // Verify values
        assert_eq!(vault.get_yield_rate(), 1000); // Still 1000, not changed
        assert_eq!(vault.get_block_height(), 2000); // Still 2000, not changed
        assert_eq!(vault.get_token_balance("user1"), 200); // Now 200, tokens can be issued anytime
    }
}
