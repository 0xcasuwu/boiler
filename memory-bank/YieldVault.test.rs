// YieldVault Tests
// Tests for the Bitcoin implementation of ERC-4626 standard
// Based on the memorybank architecture

use std::collections::HashSet;
use serde_json;

// Mock dependencies for testing
mod mock {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use lazy_static::lazy_static;

    // Mock storage
    lazy_static! {
        pub static ref STORAGE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
        pub static ref TIMESTAMP: Mutex<u64> = Mutex::new(1651388400); // May 1, 2022 12:00:00 PM UTC
    }

    pub mod storage {
        use super::*;
        
        pub fn get_string(path: &str) -> Option<String> {
            let storage = STORAGE.lock().unwrap();
            storage.get(path).map(|bytes| {
                String::from_utf8(bytes.clone()).unwrap_or_default()
            })
        }
        
        pub fn set_string(path: &str, value: &str) {
            let mut storage = STORAGE.lock().unwrap();
            storage.insert(path.to_string(), value.as_bytes().to_vec());
        }
        
        pub fn get_u128(path: &str) -> Option<u128> {
            let storage = STORAGE.lock().unwrap();
            storage.get(path).map(|bytes| {
                let mut buf = [0u8; 16];
                let len = std::cmp::min(bytes.len(), 16);
                buf[..len].copy_from_slice(&bytes[..len]);
                u128::from_le_bytes(buf)
            })
        }
        
        pub fn set_u128(path: &str, value: u128) {
            let mut storage = STORAGE.lock().unwrap();
            storage.insert(path.to_string(), value.to_le_bytes().to_vec());
        }
        
        pub fn get_u64(path: &str) -> Option<u64> {
            let storage = STORAGE.lock().unwrap();
            storage.get(path).map(|bytes| {
                let mut buf = [0u8; 8];
                let len = std::cmp::min(bytes.len(), 8);
                buf[..len].copy_from_slice(&bytes[..len]);
                u64::from_le_bytes(buf)
            })
        }
        
        pub fn set_u64(path: &str, value: u64) {
            let mut storage = STORAGE.lock().unwrap();
            storage.insert(path.to_string(), value.to_le_bytes().to_vec());
        }
        
        pub fn get_u8(path: &str) -> Option<u8> {
            let storage = STORAGE.lock().unwrap();
            storage.get(path).map(|bytes| {
                bytes[0]
            })
        }
        
        pub fn set_u8(path: &str, value: u8) {
            let mut storage = STORAGE.lock().unwrap();
            storage.insert(path.to_string(), vec![value]);
        }
        
        pub fn get_bool(path: &str) -> Option<bool> {
            let storage = STORAGE.lock().unwrap();
            storage.get(path).map(|bytes| {
                !bytes.is_empty() && bytes[0] != 0
            })
        }
        
        pub fn set_bool(path: &str, value: bool) {
            let mut storage = STORAGE.lock().unwrap();
            storage.insert(path.to_string(), vec![value as u8]);
        }
        
        pub fn clear() {
            let mut storage = STORAGE.lock().unwrap();
            storage.clear();
        }
    }
    
    pub fn get_timestamp() -> u64 {
        *TIMESTAMP.lock().unwrap()
    }
    
    pub fn set_timestamp(time: u64) {
        let mut timestamp = TIMESTAMP.lock().unwrap();
        *timestamp = time;
    }
    
    pub fn advance_time(seconds: u64) {
        let mut timestamp = TIMESTAMP.lock().unwrap();
        *timestamp += seconds;
    }
    
    pub fn generate_tx_hash() -> String {
        let timestamp = get_timestamp();
        format!("tx_{}", timestamp)
    }
}

// Import the YieldVault code, adjusting imports to use our mock
mod yield_vault {
    use super::mock::storage;
    use std::collections::HashSet;
    use serde_json;

    // YieldVault struct definition
    pub struct YieldVault {}
    
    impl Default for YieldVault {
        fn default() -> Self {
            Self {}
        }
    }
    
    impl YieldVault {
        // == Initialization ==
        pub fn initialize(
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
        pub fn observe_initialization(&self) -> Result<(), &'static str> {
            if storage::get_bool("/initialized").unwrap_or(false) {
                return Err("Already initialized");
            }
            storage::set_bool("/initialized", true);
            Ok(())
        }
        
        pub fn validate_and_track_transaction(&self, tx_hash: &str) -> Result<(), &'static str> {
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
        
        pub fn get_transaction_hashes(&self) -> HashSet<String> {
            let json = storage::get_string("/tx-hashes").unwrap_or_default();
            if json.is_empty() {
                HashSet::new()
            } else {
                serde_json::from_str(&json).unwrap_or_default()
            }
        }
        
        pub fn set_transaction_hashes(&self, hashes: &HashSet<String>) -> Result<(), &'static str> {
            let json = serde_json::to_string(hashes)
                .map_err(|_| "Failed to serialize transaction hashes")?;
            storage::set_string("/tx-hashes", &json);
            Ok(())
        }
        
        pub fn check_authorization(&self, caller: &str, owner: &str) -> Result<(), &'static str> {
            if caller != owner {
                return Err("Not authorized");
            }
            Ok(())
        }
        
        // == Asset Management Functions ==
        pub fn deposit(
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
            let max_deposit = self.max_deposit(receiver.clone())?;
            if assets > max_deposit {
                return Err("Deposit amount exceeds limit");
            }
            
            // Calculate shares from assets
            let shares = self.preview_deposit(assets)?;
            if shares == 0 {
                return Err("Zero shares");
            }
            
            // Update state
            self.add_total_assets(assets)?;
            self.mint_shares(&receiver, shares)?;
            
            Ok(())
        }
        
        pub fn mint(
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
            let max_mint = self.max_mint(receiver.clone())?;
            if shares > max_mint {
                return Err("Mint amount exceeds limit");
            }
            
            // Calculate assets needed for shares
            let assets = self.preview_mint(shares)?;
            if assets == 0 {
                return Err("Zero assets");
            }
            
            // Update state
            self.add_total_assets(assets)?;
            self.mint_shares(&receiver, shares)?;
            
            Ok(())
        }
        
        pub fn withdraw(
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
            let max_withdraw = self.max_withdraw(owner.clone())?;
            if assets > max_withdraw {
                return Err("Withdrawal amount exceeds limit");
            }
            
            // Calculate shares needed for assets
            let shares = self.preview_withdraw(assets)?;
            if shares == 0 {
                return Err("Zero shares");
            }
            
            // Update state
            self.subtract_total_assets(assets)?;
            self.burn_shares(&owner, shares)?;
            
            Ok(())
        }
        
        pub fn redeem(
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
            let max_redeem = self.max_redeem(owner.clone())?;
            if shares > max_redeem {
                return Err("Redemption amount exceeds limit");
            }
            
            // Calculate assets for shares
            let assets = self.preview_redeem(shares)?;
            
            // Update state
            self.subtract_total_assets(assets)?;
            self.burn_shares(&owner, shares)?;
            
            Ok(())
        }
        
        // == Yield Management ==
        pub fn update_yield(&self) -> Result<(), &'static str> {
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
        
        pub fn update_yield_rate(&mut self, yield_rate: u128) -> Result<(), &'static str> {
            // Update the yield first with the old rate
            self.update_yield()?;
            
            // Set the new yield rate
            storage::set_u128("/yield-rate", yield_rate);
            
            Ok(())
        }
        
        // == View Functions: Metadata ==
        pub fn name(&self) -> String {
            storage::get_string("/name").unwrap_or_default()
        }
        
        pub fn symbol(&self) -> String {
            storage::get_string("/symbol").unwrap_or_default()
        }
        
        pub fn decimals(&self) -> u8 {
            storage::get_u8("/decimals").unwrap_or(18)
        }
        
        pub fn asset(&self) -> String {
            storage::get_string("/asset-symbol").unwrap_or_default()
        }
        
        // == View Functions: Accounting ==
        pub fn total_assets(&self) -> u128 {
            storage::get_u128("/total-assets").unwrap_or(0)
        }
        
        pub fn convert_to_shares(&self, assets: u128) -> Result<u128, &'static str> {
            let total_assets = self.total_assets();
            let total_supply = self.total_supply();
            
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
        
        pub fn convert_to_assets(&self, shares: u128) -> Result<u128, &'static str> {
            let total_assets = self.total_assets();
            let total_supply = self.total_supply();
            
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
        pub fn max_deposit(&self, _receiver: String) -> Result<u128, &'static str> {
            // In this simple implementation, we don't have a cap
            Ok(u128::MAX)
        }
        
        pub fn max_mint(&self, _receiver: String) -> Result<u128, &'static str> {
            // In this simple implementation, we don't have a cap
            Ok(u128::MAX)
        }
        
        pub fn max_withdraw(&self, owner: String) -> Result<u128, &'static str> {
            // Calculate based on owner's balance
            let balance = self.balance_of(&owner);
            self.convert_to_assets(balance)
        }
        
        pub fn max_redeem(&self, owner: String) -> Result<u128, &'static str> {
            // Simply return the owner's balance
            Ok(self.balance_of(&owner))
        }
        
        // == View Functions: Preview ==
        pub fn preview_deposit(&self, assets: u128) -> Result<u128, &'static str> {
            self.convert_to_shares(assets)
        }
        
        pub fn preview_mint(&self, shares: u128) -> Result<u128, &'static str> {
            let total_assets = self.total_assets();
            let total_supply = self.total_supply();
            
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
        
        pub fn preview_withdraw(&self, assets: u128) -> Result<u128, &'static str> {
            let total_assets = self.total_assets();
            let total_supply = self.total_supply();
            
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
        
        pub fn preview_redeem(&self, shares: u128) -> Result<u128, &'static str> {
            self.convert_to_assets(shares)
        }
        
        // == Balance and Supply Management ==
        pub fn balance_of(&self, account: &str) -> u128 {
            self.get_balance(account)
        }
        
        pub fn total_supply(&self) -> u128 {
            storage::get_u128("/total-supply").unwrap_or(0)
        }
        
        pub fn get_balance(&self, account: &str) -> u128 {
            let key = format!("/balances/{}", account);
            storage::get_u128(&key).unwrap_or(0)
        }
        
        pub fn set_balance(&self, account: &str, amount: u128) {
            let key = format!("/balances/{}", account);
            storage::set_u128(&key, amount);
        }
        
        pub fn mint_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
            // Get current balance
            let balance = self.get_balance(account);
            
            // Calculate new balance
            let new_balance = balance
                .checked_add(amount)
                .ok_or("Balance overflow")?;
                
            // Update balance
            self.set_balance(account, new_balance);
            
            // Update total supply
            let supply = self.total_supply();
            let new_supply = supply
                .checked_add(amount)
                .ok_or("Supply overflow")?;
                
            storage::set_u128("/total-supply", new_supply);
            
            Ok(())
        }
        
        pub fn burn_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
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
            let supply = self.total_supply();
            let new_supply = supply - amount;
            storage::set_u128("/total-supply", new_supply);
            
            Ok(())
        }
        
        pub fn add_total_assets(&self, amount: u128) -> Result<(), &'static str> {
            let total = self.total_assets();
            let new_total = total
                .checked_add(amount)
                .ok_or("Total assets overflow")?;
                
            storage::set_u128("/total-assets", new_total);
            Ok(())
        }
        
        pub fn subtract_total_assets(&self, amount: u128) -> Result<(), &'static str> {
            let total = self.total_assets();
            
            // Ensure sufficient assets
            if total < amount {
                return Err("Insufficient assets");
            }
            
            let new_total = total - amount;
            storage::set_u128("/total-assets", new_total);
            Ok(())
        }
        
        pub fn get_timestamp(&self) -> u64 {
            // Use our mock timestamp
            super::mock::get_timestamp()
        }
    }
}

// Test suite
#[cfg(test)]
mod tests {
    use super::*;
    use super::yield_vault::YieldVault;
    use super::mock::*;
    
    // Helper to set up a clean vault for each test
    fn setup_vault() -> YieldVault {
        // Clear storage
        storage::clear();
        
        // Reset timestamp
        set_timestamp(1651388400); // May 1, 2022 12:00:00 PM UTC
        
        let mut vault = YieldVault::default();
        
        // Initialize the vault
        vault.initialize(
            "Test Vault".to_string(),
            "vTEST".to_string(),
            "Test Asset".to_string(),
            "TEST".to_string(),
            18
        ).unwrap();
        
        vault
    }
    
    #[test]
    fn test_initialization() {
        let vault = setup_vault();
        
        // Check basic metadata
        assert_eq!(vault.name(), "Test Vault");
        assert_eq!(vault.symbol(), "vTEST");
        assert_eq!(vault.asset(), "TEST");
        assert_eq!(vault.decimals(), 18);
        
        // Check initial state
        assert_eq!(vault.total_supply(), 0);
        assert_eq!(vault.total_assets(), 0);
    }
    
    #[test]
    fn test_double_initialization() {
        let mut vault = setup_vault();
        
        // Try to initialize again
        let result = vault.initialize(
            "Another Vault".to_string(),
            "vANOTHER".to_string(),
            "Another Asset".to_string(),
            "ANOTHER".to_string(),
            18
        );
        
        // Should fail with already initialized error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Already initialized");
        
        // Original metadata should remain unchanged
        assert_eq!(vault.name(), "Test Vault");
        assert_eq!(vault.symbol(), "vTEST");
    }
    
    #[test]
    fn test_deposit() {
        let mut vault = setup_vault();
        let tx_hash = generate_tx_hash();
        
        // Deposit 100 assets
        let result = vault.deposit(
            tx_hash,
            "alice".to_string(),
            "alice".to_string(),
            100
        );
        
        assert!(result.is_ok());
        
        // Check state after deposit
        assert_eq!(vault.total_assets(), 100);
        assert_eq!(vault.total_supply(), 100); // Initial 1:1 ratio
        assert_eq!(vault.balance_of("alice"), 100);
    }
    
    #[test]
    fn test_mint() {
        let mut vault = setup_vault();
        let tx_hash = generate_tx_hash();
        
        // Mint 100 shares
        let result = vault.mint(
            tx_hash,
            "alice".to_string(),
            "alice".to_string(),
            100
        );
        
        assert!(result.is_ok());
        
        // Check state after mint
        assert_eq!(vault.total_supply(), 100);
        assert_eq!(vault.total_assets(), 100); // Initial 1:1 ratio
        assert_eq!(vault.balance_of("alice"), 100);
    }
    
    #[test]
    fn test_multiple_deposits() {
        let mut vault = setup_vault();
        
        // First deposit: 100 assets by Alice
        vault.deposit(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            100
        ).unwrap();
        
        // Second deposit: 150 assets by Bob
        vault.deposit(
            generate_tx_hash(),
            "bob".to_string(),
            "bob".to_string(),
            150
        ).unwrap();
        
        // Check state after deposits
        assert_eq!(vault.total_assets(), 250);
        assert_eq!(vault.total_supply(), 250);
        assert_eq!(vault.balance_of("alice"), 100);
        assert_eq!(vault.balance_of("bob"), 150);
    }
    
    #[test]
    fn test_withdraw() {
        let mut vault = setup_vault();
        
        // First deposit 100 assets
        vault.deposit(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            100
        ).unwrap();
        
        // Withdraw 30 assets
        let result = vault.withdraw(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            "alice".to_string(),
            30
        );
        
        assert!(result.is_ok());
        
        // Check state after withdrawal
        assert_eq!(vault.total_assets(), 70);
        assert_eq!(vault.total_supply(), 70);
        assert_eq!(vault.balance_of("alice"), 70);
    }
    
    #[test]
    fn test_redeem() {
        let mut vault = setup_vault();
        
        // First mint 100 shares
        vault.mint(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            100
        ).unwrap();
        
        // Redeem 30 shares
        let result = vault.redeem(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            "alice".to_string(),
            30
        );
        
        assert!(result.is_ok());
        
        // Check state after redemption
        assert_eq!(vault.total_supply(), 70);
        assert_eq!(vault.total_assets(), 70);
        assert_eq!(vault.balance_of("alice"), 70);
    }
    
    #[test]
    fn test_transaction_replay_protection() {
        let mut vault = setup_vault();
        let tx_hash = "repeated_tx_hash".to_string();
        
        // First deposit with this hash should work
        let result1 = vault.deposit(
            tx_hash.clone(),
            "alice".to_string(),
            "alice".to_string(),
            100
        );
        
        assert!(result1.is_ok());
        
        // Second deposit with the same hash should fail
        let result2 = vault.deposit(
            tx_hash,
            "alice".to_string(),
            "alice".to_string(),
            100
        );
        
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), "Transaction hash already used");
        
        // State should reflect only the first deposit
        assert_eq!(vault.total_assets(), 100);
        assert_eq!(vault.total_supply(), 100);
        assert_eq!(vault.balance_of("alice"), 100);
    }
    
    #[test]
    fn test_authorization_check() {
        let mut vault = setup_vault();
        
        // First deposit 100 assets as Alice
        vault.deposit(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            100
        ).unwrap();
        
        // Bob tries to withdraw Alice's assets
        let result = vault.withdraw(
            generate_tx_hash(),
            "bob".to_string(), // Caller is Bob
            "bob".to_string(),
            "alice".to_string(), // Owner is Alice
            30
        );
        
        // Should fail with authorization error
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not authorized");
        
        // State should remain unchanged
        assert_eq!(vault.total_assets(), 100);
        assert_eq!(vault.total_supply(), 100);
        assert_eq!(vault.balance_of("alice"), 100);
    }
    
    #[test]
    fn test_yield_accrual() {
        let mut vault = setup_vault();
        
        // Deposit 1000 assets
        vault.deposit(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            1000
        ).unwrap();
        
        // Set yield rate to 5% (500 basis points)
        vault.update_yield_rate(500).unwrap();
        
        // Check initial state
        assert_eq!(vault.total_assets(), 1000);
        assert_eq!(vault.total_supply(), 1000);
        
        // Advance time by 1 year (in seconds)
        advance_time(365 * 24 * 60 * 60);
        
        // Trigger yield calculation by performing any operation
        vault.update_yield().unwrap();
        
        // Assets should have increased by about 5%
        // Exact calculation: 1000 * 500 * (365*24*60*60) / (10000 * 365*24*60*60) = 50
        assert_eq!(vault.total_assets(), 1050);
        
        // Total supply shouldn't have changed
        assert_eq!(vault.total_supply(), 1000);
        
        // Share price should have increased
        let assets_per_share = vault.convert_to_assets(1000).unwrap();
        assert_eq!(assets_per_share, 1050);
    }
    
    #[test]
    fn test_yield_after_partial_withdrawal() {
        let mut vault = setup_vault();
        
        // Deposit 1000 assets
        vault.deposit(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            1000
        ).unwrap();
        
        // Set yield rate to 10% (1000 basis points)
        vault.update_yield_rate(1000).unwrap();
        
        // Advance time by 6 months
        advance_time(182 * 24 * 60 * 60);
        
        // Withdraw half of assets
        vault.withdraw(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            "alice".to_string(),
            500
        ).unwrap();
        
        // After the withdrawal, yield should have been applied
        // Approximate 5% for 6 months: 1000 * 0.05 = 50
        assert!(vault.total_assets() > 500); // Should be around 550
        assert!(vault.total_assets() <= 550); // But not more than 550
        
        // Advance time by another 6 months
        advance_time(183 * 24 * 60 * 60);
        
        // Withdraw remaining assets
        let remaining_balance = vault.balance_of("alice");
        vault.redeem(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            "alice".to_string(),
            remaining_balance
        ).unwrap();
        
        // After the redemption, yield should have been applied to the reduced amount
        // Balance should be 0 after full redemption
        assert_eq!(vault.balance_of("alice"), 0);
        // Total supply should be 0
        assert_eq!(vault.total_supply(), 0);
        // Total assets should be 0
        assert_eq!(vault.total_assets(), 0);
    }
    
    #[test]
    fn test_convert_to_shares_with_yield() {
        let mut vault = setup_vault();
        
        // Deposit 1000 assets initially
        vault.deposit(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            1000
        ).unwrap();
        
        // Set yield rate to 10% (1000 basis points)
        vault.update_yield_rate(1000).unwrap();
        
        // Advance time by 1 year
        advance_time(365 * 24 * 60 * 60);
        
        // Update yield
        vault.update_yield().unwrap();
        
        // Check total assets after a year (should be 1100)
        assert_eq!(vault.total_assets(), 1100);
        
        // Now deposit 1000 more assets
        // This should give fewer shares than the initial deposit
        // Since each share is now worth more
        let shares = vault.preview_deposit(1000).unwrap();
        
        // New shares should be approximately: 1000 * 1000 / 1100 = 909
        assert!(shares < 1000); // Should be less than original deposit
        assert!(shares >= 909 && shares <= 910); // Should be around 909 shares
        
        // Make the deposit
        vault.deposit(
            generate_tx_hash(),
            "bob".to_string(),
            "bob".to_string(),
            1000
        ).unwrap();
        
        // Check bob's balance
        assert!(vault.balance_of("bob") < 1000); // Should be less than 1000
        assert!(vault.balance_of("bob") >= 909 && vault.balance_of("bob") <= 910); // Around 909
        
        // Total supplies and assets
        assert_eq!(vault.total_assets(), 2100); // 1100 + 1000
        assert!(vault.total_supply() < 2000); // Less than 2000 due to yield effect
    }
    
    #[test]
    fn test_share_price_calculation() {
        let mut vault = setup_vault();
        
        // Deposit 1000 assets
        vault.deposit(
            generate_tx_hash(),
            "alice".to_string(),
            "alice".to_string(),
            1000
        ).unwrap();
        
        // Initial share price should be 1:1
        let initial_price = vault.convert_to_assets(100).unwrap();
        assert_eq!(initial_price, 100);
        
        // Set yield rate to 20% (2000 basis points)
        vault.update_yield_rate(2000).unwrap();
        
        // Advance time by 1 year
        advance_time(365 * 24 * 60 * 60);
        
        // Update yield
        vault.update_yield().unwrap();
        
        // After yield, share price should increase
        let new_price = vault.convert_to_assets(100).unwrap();
        assert_eq!(new_price, 120); // 20% increase
        
        // Converting back should give original shares
        let back_to_shares = vault.convert_to_shares(120).unwrap();
        assert_eq!(back_to_shares, 100);
    }
}
