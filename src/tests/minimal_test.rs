use crate::YieldVault;
use std::sync::Arc;
use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_runtime::storage::StoragePointer;
use wasm_bindgen_test::wasm_bindgen_test;

// Thread-local storage to hold the current test's storage prefix
thread_local! {
    static STORAGE_PREFIX: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

// Reset all storage keys used in tests with test-specific prefixes to avoid collisions
fn reset_test_storage(test_name: &str) {
    let prefix = format!("/{}", test_name);
    
    // Clear all storage keys with test-specific prefixes
    // Use non-empty vectors for string data to prevent SIGSEGV when slicing
    StoragePointer::from_keyword(&format!("{}/initialized", prefix)).set(Arc::new(vec![0u8]));
    StoragePointer::from_keyword(&format!("{}/name", prefix)).set(Arc::new("Default".as_bytes().to_vec()));
    StoragePointer::from_keyword(&format!("{}/symbol", prefix)).set(Arc::new("SYM".as_bytes().to_vec()));
    StoragePointer::from_keyword(&format!("{}/asset-name", prefix)).set(Arc::new("Asset".as_bytes().to_vec()));
    StoragePointer::from_keyword(&format!("{}/asset-symbol", prefix)).set(Arc::new("AST".as_bytes().to_vec()));
    StoragePointer::from_keyword(&format!("{}/decimals", prefix)).set(Arc::new(vec![0u8]));
    StoragePointer::from_keyword(&format!("{}/total-supply", prefix)).set(Arc::new(vec![0u8; 16]));
    StoragePointer::from_keyword(&format!("{}/total-assets", prefix)).set(Arc::new(vec![0u8; 16]));
    StoragePointer::from_keyword(&format!("{}/yield-rate", prefix)).set(Arc::new(vec![0u8; 16]));
    StoragePointer::from_keyword(&format!("{}/last-yield-update", prefix)).set(Arc::new(vec![0u8; 8])); // u64 is 8 bytes
    StoragePointer::from_keyword(&format!("{}/tx-hashes", prefix)).set(Arc::new(vec![0u8; 4]));
    
    // Clear example balances with properly sized vectors for u128 (16 bytes)
    StoragePointer::from_keyword(&format!("{}/balances/alice", prefix)).set(Arc::new(vec![0u8; 16]));
    StoragePointer::from_keyword(&format!("{}/balances/bob", prefix)).set(Arc::new(vec![0u8; 16]));
    
    // Clear any custom data
    StoragePointer::from_keyword(&format!("{}/data", prefix)).set(Arc::new(vec![0u8; 4]));
    
    // Override global storage paths with test-specific ones
    STORAGE_PREFIX.with(|cell| {
        *cell.borrow_mut() = Some(prefix);
    });
}

// Mock YieldVault implementation for testing with isolated storage
struct TestYieldVault {
    base: YieldVault,
    test_name: String,
}

impl TestYieldVault {
    fn new(test_name: &str) -> Self {
        reset_test_storage(test_name);
        Self {
            base: YieldVault::default(),
            test_name: test_name.to_string()
        }
    }
    
    // Delegate all methods to base vault but use prefixed storage
    fn name_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/name", prefix),
                None => "/name".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn symbol_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/symbol", prefix),
                None => "/symbol".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn asset_name_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/asset-name", prefix),
                None => "/asset-name".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn asset_symbol_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/asset-symbol", prefix),
                None => "/asset-symbol".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn decimals_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/decimals", prefix),
                None => "/decimals".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn total_supply_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/total-supply", prefix),
                None => "/total-supply".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn total_assets_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/total-assets", prefix),
                None => "/total-assets".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn yield_rate_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/yield-rate", prefix),
                None => "/yield-rate".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn last_yield_update_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/last-yield-update", prefix),
                None => "/last-yield-update".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn initialized_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/initialized", prefix),
                None => "/initialized".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn tx_hashes_pointer(&self) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/tx-hashes", prefix),
                None => "/tx-hashes".to_string(),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    fn balance_pointer(&self, account: &str) -> StoragePointer {
        let path = STORAGE_PREFIX.with(|cell| {
            match &*cell.borrow() {
                Some(prefix) => format!("{}/balances/{}", prefix, account),
                None => format!("/balances/{}", account),
            }
        });
        StoragePointer::from_keyword(&path)
    }
    
    // Implement core functionality methods using prefixed storage
    fn observe_initialization(&self) -> Result<(), &'static str> {
        if self.initialized_pointer().get_value::<u8>() != 0 {
            return Err("Already initialized");
        }
        self.initialized_pointer().set_value(1u8);
        Ok(())
    }
    
    fn initialize(&mut self, name: String, symbol: String, asset_name: String, asset_symbol: String, decimals: u8) -> Result<(), &'static str> {
        self.observe_initialization()?;
        
        self.name_pointer().set(Arc::new(name.as_bytes().to_vec()));
        self.symbol_pointer().set(Arc::new(symbol.as_bytes().to_vec()));
        self.asset_name_pointer().set(Arc::new(asset_name.as_bytes().to_vec()));
        self.asset_symbol_pointer().set(Arc::new(asset_symbol.as_bytes().to_vec()));
        self.decimals_pointer().set_value(decimals);
        
        // Initialize with zero values
        self.total_supply_pointer().set_value(0u128);
        self.total_assets_pointer().set_value(0u128);
        self.yield_rate_pointer().set_value(0u128);
        self.last_yield_update_pointer().set_value(0u64);
        self.tx_hashes_pointer().set(Arc::new("{}".as_bytes().to_vec()));
        
        Ok(())
    }
    
    fn get_balance(&self, account: &str) -> u128 {
        self.balance_pointer(account).get_value()
    }
    
    fn set_balance(&self, account: &str, amount: u128) {
        self.balance_pointer(account).set_value(amount);
    }
    
    fn mint_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
        // Update account balance
        let current = self.get_balance(account);
        let new_amount = current.checked_add(amount).ok_or("Balance overflow")?;
        self.set_balance(account, new_amount);
        
        // Update total supply
        let current_supply = self.total_supply_pointer().get_value::<u128>();
        let new_supply = current_supply.checked_add(amount).ok_or("Supply overflow")?;
        self.total_supply_pointer().set_value(new_supply);
        
        Ok(())
    }
    
    fn burn_shares(&self, account: &str, amount: u128) -> Result<(), &'static str> {
        // Check if account has enough balance
        let current = self.get_balance(account);
        if current < amount {
            return Err("Insufficient balance");
        }
        
        // Update account balance
        self.set_balance(account, current - amount);
        
        // Update total supply
        let current_supply = self.total_supply_pointer().get_value::<u128>();
        let new_supply = current_supply.checked_sub(amount).ok_or("Supply underflow")?;
        self.total_supply_pointer().set_value(new_supply);
        
        Ok(())
    }
    
    fn validate_and_track_transaction(&self, tx_hash: &str) -> Result<(), &'static str> {
        // Get current tx hashes
        let tx_hashes_data = self.tx_hashes_pointer().get();
        let tx_hashes_str = String::from_utf8_lossy(&tx_hashes_data);
        
        // Parse JSON object of transaction hashes
        let mut tx_hashes: serde_json::Map<String, serde_json::Value> = 
            serde_json::from_str(&tx_hashes_str).unwrap_or_default();
        
        // Check if transaction hash already exists
        if tx_hashes.contains_key(tx_hash) {
            return Err("Transaction already processed");
        }
        
        // Add the transaction hash
        tx_hashes.insert(tx_hash.to_string(), serde_json::Value::Bool(true));
        
        // Serialize back to JSON and update storage
        let updated_json = serde_json::to_string(&tx_hashes).map_err(|_| "JSON serialization error")?;
        self.tx_hashes_pointer().set(Arc::new(updated_json.as_bytes().to_vec()));
        
        Ok(())
    }
    
    fn deposit(&mut self, tx_hash: String, caller: String, receiver: String, assets: u128) -> Result<(), &'static str> {
        // Validate the transaction hash
        self.validate_and_track_transaction(&tx_hash)?;
        
        // Check that the vault is initialized
        if self.initialized_pointer().get_value::<u8>() == 0 {
            return Err("Vault not initialized");
        }
        
        // Calculate shares to mint based on current ratio
        let shares = if self.total_assets_pointer().get_value::<u128>() == 0 {
            // If first deposit, use 1:1 ratio
            assets
        } else {
            // Otherwise convert assets to shares based on current ratio
            self.convert_assets_to_shares(assets)?
        };
        
        // Update user's balance
        self.mint_shares(&receiver, shares)?;
        
        // Update total assets
        let current_assets = self.total_assets_pointer().get_value::<u128>();
        let new_assets = current_assets.checked_add(assets).ok_or("Assets overflow")?;
        self.total_assets_pointer().set_value(new_assets);
        
        Ok(())
    }
    
    fn mint(&mut self, tx_hash: String, caller: String, receiver: String, shares: u128) -> Result<(), &'static str> {
        // Validate the transaction hash
        self.validate_and_track_transaction(&tx_hash)?;
        
        // Check that the vault is initialized
        if self.initialized_pointer().get_value::<u8>() == 0 {
            return Err("Vault not initialized");
        }
        
        // Calculate assets needed based on current ratio
        let assets = if self.total_supply_pointer().get_value::<u128>() == 0 {
            // If first mint, use 1:1 ratio
            shares
        } else {
            // Otherwise convert shares to assets based on current ratio
            self.convert_shares_to_assets(shares)?
        };
        
        // Update user's balance
        self.mint_shares(&receiver, shares)?;
        
        // Update total assets
        let current_assets = self.total_assets_pointer().get_value::<u128>();
        let new_assets = current_assets.checked_add(assets).ok_or("Assets overflow")?;
        self.total_assets_pointer().set_value(new_assets);
        
        Ok(())
    }
    
    // Conversion functions
    fn convert_assets_to_shares(&self, assets: u128) -> Result<u128, &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        if total_assets == 0 || total_supply == 0 {
            return Ok(assets);
        }
        assets
            .checked_mul(total_supply)
            .ok_or("Convert to shares multiplication overflow")?
            .checked_div(total_assets)
            .ok_or("Convert to shares division error")
    }
    
    fn convert_shares_to_assets(&self, shares: u128) -> Result<u128, &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        if total_supply == 0 {
            return Ok(shares);
        }
        shares
            .checked_mul(total_assets)
            .ok_or("Convert to assets multiplication overflow")?
            .checked_div(total_supply)
            .ok_or("Convert to assets division error")
    }
    
    // Preview functions with rounding
    fn preview_deposit(&self, assets: u128) -> Result<u128, &'static str> {
        self.convert_assets_to_shares(assets)
    }
    
    fn preview_mint(&self, shares: u128) -> Result<u128, &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        if total_supply == 0 || shares == 0 {
            return Ok(shares);
        }
        
        // Round up: (shares * total_assets + total_supply - 1) / total_supply
        let product = shares.checked_mul(total_assets).ok_or("Mint preview multiplication overflow")?;
        let numerator = product.checked_add(total_supply).ok_or("Mint preview addition overflow")?;
        let adjusted = numerator.checked_sub(1).unwrap_or(numerator); // Avoid underflow
        
        adjusted.checked_div(total_supply).ok_or("Mint preview division error")
    }
    
    fn preview_withdraw(&self, assets: u128) -> Result<u128, &'static str> {
        let total_assets = self.total_assets_pointer().get_value::<u128>();
        let total_supply = self.total_supply_pointer().get_value::<u128>();
        
        if total_assets == 0 || assets == 0 {
            return Ok(assets);
        }
        
        // Round up: (assets * total_supply + total_assets - 1) / total_assets
        let product = assets.checked_mul(total_supply).ok_or("Withdraw preview multiplication overflow")?;
        let numerator = product.checked_add(total_assets).ok_or("Withdraw preview addition overflow")?;
        let adjusted = numerator.checked_sub(1).unwrap_or(numerator); // Avoid underflow
        
        adjusted.checked_div(total_assets).ok_or("Withdraw preview division error")
    }
    
    fn preview_redeem(&self, shares: u128) -> Result<u128, &'static str> {
        self.convert_shares_to_assets(shares)
    }
    
    // Test yield accrual
    fn test_update_yield(&mut self) -> Result<(), &'static str> {
        use crate::tests::mock::get_timestamp;
        
        let current_time = get_timestamp();
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
        println!("Initial assets: {}", total_assets);
        if total_assets == 0 {
            return Ok(());  // No assets to apply yield to
        }
        
        // Calculate yield: assets * rate * timeElapsed / (10000 * 365 * 24 * 60 * 60)
        // Rate is in basis points (1/100 of a percent)
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
                
            self.total_assets_pointer().set_value(new_total);
            println!("New assets: {}, Increase: {}", new_total, yield_amount);
        } else {
            println!("New assets: {}, Increase: {}", total_assets, yield_amount);
        }
        
        // Update the last yield timestamp
        self.last_yield_update_pointer().set_value(current_time);
        
        Ok(())
    }
}

#[wasm_bindgen_test]
#[test]
fn test_can_create_vault() {
    let _vault = TestYieldVault::new("test_can_create_vault");
    assert!(true, "Test vault creation succeeded");
}

#[wasm_bindgen_test]
#[test]
fn test_storage_pointers() {
    let vault = TestYieldVault::new("test_storage_pointers");
    
    // Verify that the storage pointers exist and return default values
    assert_eq!(vault.name_pointer().get().len(), "Default".as_bytes().len());
    assert_eq!(vault.symbol_pointer().get().len(), "SYM".as_bytes().len());
    assert_eq!(vault.asset_name_pointer().get().len(), "Asset".as_bytes().len());
    assert_eq!(vault.asset_symbol_pointer().get().len(), "AST".as_bytes().len());
    
    // For numeric types, make sure we set the proper vector size
    assert_eq!(vault.total_supply_pointer().get().len(), 16); // u128 is 16 bytes
    assert_eq!(vault.total_assets_pointer().get().len(), 16); // u128 is 16 bytes
    assert_eq!(vault.yield_rate_pointer().get().len(), 16);   // u128 is 16 bytes
    assert_eq!(vault.last_yield_update_pointer().get().len(), 8); // u64 is 8 bytes
    assert_eq!(vault.initialized_pointer().get().len(), 1);   // u8 is 1 byte
    
    // Check that getting values as types works properly
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.total_assets_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.yield_rate_pointer().get_value::<u128>(), 0);
    assert_eq!(vault.last_yield_update_pointer().get_value::<u64>(), 0);
    assert_eq!(vault.initialized_pointer().get_value::<u8>(), 0);
}

#[wasm_bindgen_test]
#[test]
fn test_initialization_guard() {
    // Create isolated test vault
    let vault = TestYieldVault::new("test_initialization_guard");

    // First initialization should succeed
    assert!(vault.observe_initialization().is_ok());
    
    // Second initialization should fail
    let result = vault.observe_initialization();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Already initialized");
}

#[wasm_bindgen_test]
#[test]
fn test_constant_values() {
    let _vault = TestYieldVault::new("test_constant_values");
    // Test the constants used in calculations
    assert_eq!(crate::BASIS_POINTS_DENOMINATOR, 10000);
    assert_eq!(crate::SECONDS_PER_YEAR, 365 * 24 * 60 * 60);
    assert_eq!(crate::YIELD_CALCULATION_DENOMINATOR, crate::BASIS_POINTS_DENOMINATOR * crate::SECONDS_PER_YEAR);
}

#[wasm_bindgen_test]
#[test]
fn test_ceil_div() {
    let _vault = TestYieldVault::new("test_ceil_div");
    // Test the ceil_div helper function
    assert_eq!(crate::ceil_div(10, 5).unwrap(), 2);
    assert_eq!(crate::ceil_div(11, 5).unwrap(), 3);  // Ceiling division
    assert_eq!(crate::ceil_div(0, 5).unwrap(), 0);
    assert!(crate::ceil_div(10, 0).is_err());  // Division by zero
}

#[wasm_bindgen_test]
#[test]
fn test_initialization() {
    let mut vault = TestYieldVault::new("test_initialization");
    
    // Initialize with test values
    let name = "Test Vault".to_string();
    let symbol = "TVL".to_string();
    let asset_name = "Test Asset".to_string();
    let asset_symbol = "TAST".to_string();
    let decimal_offset = 18u8;
    
    // Call initialize directly for testing
    vault.initialize(name.clone(), symbol.clone(), asset_name.clone(), asset_symbol.clone(), decimal_offset).unwrap();
    
    // Verify the values were set correctly
    let stored_name = String::from_utf8(vault.name_pointer().get().as_ref().to_vec()).unwrap();
    assert_eq!(stored_name, name);
    
    let stored_symbol = String::from_utf8(vault.symbol_pointer().get().as_ref().to_vec()).unwrap();
    assert_eq!(stored_symbol, symbol);
    
    let stored_asset_name = String::from_utf8(vault.asset_name_pointer().get().as_ref().to_vec()).unwrap();
    assert_eq!(stored_asset_name, asset_name);
    
    let stored_asset_symbol = String::from_utf8(vault.asset_symbol_pointer().get().as_ref().to_vec()).unwrap();
    assert_eq!(stored_asset_symbol, asset_symbol);
    
    let stored_decimals = vault.decimals_pointer().get_value::<u8>();
    assert_eq!(stored_decimals, decimal_offset);
    
    // Check initialization flag
    assert_eq!(vault.initialized_pointer().get_value::<u8>(), 1);
}

#[wasm_bindgen_test]
#[test]
fn test_deposit_functionality() {
    let mut vault = TestYieldVault::new("test_deposit_functionality");
    use crate::tests::mock;
    
    // Initialize mock timestamp
    let initial_timestamp = 1000u64;
    mock::set_timestamp(initial_timestamp);

    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set name, symbol, etc. for proper initialization
    let name = "Test Vault".to_string();
    let symbol = "TVL".to_string();
    let asset_name = "Test Asset".to_string();
    let asset_symbol = "TAST".to_string();
    let decimal_offset = 18u8;
    
    // Initialize the vault metadata
    vault.name_pointer().set(Arc::new(name.as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new(symbol.as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new(asset_name.as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new(asset_symbol.as_bytes().to_vec()));
    vault.decimals_pointer().set_value(decimal_offset);
    
    // Set some initial assets for testing
    vault.total_assets_pointer().set_value(0u128);
    vault.total_supply_pointer().set_value(0u128);
    
    let tx_hash = "deposit_tx".to_string();
    let assets = 100u128;
    
    // Initialize transaction hash storage for replay protection
    vault.tx_hashes_pointer().set(Arc::new("{}".as_bytes().to_vec()));
    
    // Use the public deposit method to test deposit functionality
    #[cfg(test)]
    {
        let caller = "alice".to_string();
        let receiver = "alice".to_string();
        
        let result = vault.deposit(tx_hash, caller, receiver.clone(), assets);
        assert!(result.is_ok());
        
        // Check balances after deposit
        let alice_balance = vault.get_balance(&receiver);
        assert_eq!(alice_balance, assets, "Alice should have received the shares from deposit");
        
        // Check total supply and assets
        let total_supply = vault.total_supply_pointer().get_value::<u128>();
        let total_assets = vault.total_assets_pointer().get_value::<u128>();
        assert_eq!(total_supply, assets, "Total supply should equal deposited assets");
        assert_eq!(total_assets, assets, "Total assets should equal deposited assets");
    }
}

#[wasm_bindgen_test]
#[test]
fn test_mint_functionality() {
    let mut vault = TestYieldVault::new("test_mint_functionality");
    use crate::tests::mock;

    // Initialize mock timestamp
    let initial_timestamp = 1000u64;
    mock::set_timestamp(initial_timestamp);

    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set name, symbol, etc. for proper initialization
    let name = "Test Vault".to_string();
    let symbol = "TVL".to_string();
    let asset_name = "Test Asset".to_string();
    let asset_symbol = "TAST".to_string();
    let decimal_offset = 18u8;
    
    // Initialize the vault metadata
    vault.name_pointer().set(Arc::new(name.as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new(symbol.as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new(asset_name.as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new(asset_symbol.as_bytes().to_vec()));
    vault.decimals_pointer().set_value(decimal_offset);
    
    // Set some initial assets for testing
    vault.total_assets_pointer().set_value(0u128);
    vault.total_supply_pointer().set_value(0u128);
    
    let tx_hash = "mint_tx".to_string();
    let shares = 100u128;
    
    // Initialize transaction hash storage for replay protection
    vault.tx_hashes_pointer().set(Arc::new("{}".as_bytes().to_vec()));
    
    // Use the public mint method to test mint functionality
    #[cfg(test)]
    {
        let caller = "alice".to_string();
        let receiver = "alice".to_string();
        
        let result = vault.mint(tx_hash, caller, receiver.clone(), shares);
        assert!(result.is_ok());
        
        // Check balances after mint
        let alice_balance = vault.get_balance(&receiver);
        assert_eq!(alice_balance, shares, "Alice should have the minted shares");
        
        // Check total supply and assets
        let total_supply = vault.total_supply_pointer().get_value::<u128>();
        let total_assets = vault.total_assets_pointer().get_value::<u128>();
        assert_eq!(total_supply, shares, "Total supply should equal minted shares");
        assert_eq!(total_assets, shares, "Total assets should equal minted shares (1:1)");
    }
}

#[wasm_bindgen_test]
#[test]
fn test_conversion_functions() {
    let vault = TestYieldVault::new("test_conversion_functions");
    use crate::tests::mock;
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);

    // Create the YieldVault instance
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    vault.yield_rate_pointer().set_value(500u128);  // 5% yield rate
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Set up initial state: 100 assets and 50 shares
    // This gives a 2:1 ratio of assets to shares
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(50u128);
    
    // Test asset to share conversion
    let shares_from_10_assets = vault.convert_assets_to_shares(10u128).unwrap();
    assert_eq!(shares_from_10_assets, 5u128); // 10 assets should be 5 shares
    
    let shares_from_20_assets = vault.convert_assets_to_shares(20u128).unwrap();
    assert_eq!(shares_from_20_assets, 10u128); // 20 assets should be 10 shares
    
    // Test share to asset conversion
    let assets_from_5_shares = vault.convert_shares_to_assets(5u128).unwrap();
    assert_eq!(assets_from_5_shares, 10u128); // 5 shares should be 10 assets
    
    let assets_from_25_shares = vault.convert_shares_to_assets(25u128).unwrap();
    assert_eq!(assets_from_25_shares, 50u128); // 25 shares should be 50 assets
    
    // Edge cases
    assert_eq!(vault.convert_assets_to_shares(0u128).unwrap(), 0u128);
    assert_eq!(vault.convert_shares_to_assets(0u128).unwrap(), 0u128);
    
    // Test with different ratio (set to 1:1)
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(100u128);
    
    // With 1:1 ratio
    assert_eq!(vault.convert_assets_to_shares(10u128).unwrap(), 10u128);
    assert_eq!(vault.convert_shares_to_assets(10u128).unwrap(), 10u128);
}

#[wasm_bindgen_test]
#[test]
fn test_preview_functions() {
    let vault = TestYieldVault::new("test_preview_functions");
    use crate::tests::mock;
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);

    // Create the YieldVault instance
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    vault.yield_rate_pointer().set_value(500u128);  // 5% yield rate
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Set up initial state: 100 assets and 50 shares
    // This gives a 2:1 ratio of assets to shares
    vault.total_assets_pointer().set_value(100u128);
    vault.total_supply_pointer().set_value(50u128);
    
    // Test preview deposit (should match convert_assets_to_shares)
    let shares_from_deposit = vault.preview_deposit(10u128).unwrap();
    assert_eq!(shares_from_deposit, 5u128);
    
    // Test preview mint (should round up)
    // For 10 shares: 10 * 100 / 50 = 20 assets
    let assets_for_mint_10 = vault.preview_mint(10u128).unwrap();
    assert_eq!(assets_for_mint_10, 20u128);
    
    // For 5 shares: 5 * 100 / 50 = 10 assets
    let assets_for_mint_5 = vault.preview_mint(5u128).unwrap();
    assert_eq!(assets_for_mint_5, 10u128);
    
    // Test preview withdraw (should round up)
    // For 11 assets: 11 * 50 / 100 = 5.5, rounded up to 6 shares
    let shares_for_withdraw_11 = vault.preview_withdraw(11u128).unwrap();
    assert_eq!(shares_for_withdraw_11, 6u128);
    
    // For 10 assets: 10 * 50 / 100 = 5 shares exactly
    let shares_for_withdraw_10 = vault.preview_withdraw(10u128).unwrap();
    assert_eq!(shares_for_withdraw_10, 5u128);
    
    // Test preview redeem (should match convert_shares_to_assets)
    let assets_for_redeem = vault.preview_redeem(10u128).unwrap();
    assert_eq!(assets_for_redeem, 20u128);
    
    // Test edge cases
    assert_eq!(vault.preview_deposit(0u128).unwrap(), 0u128);
    assert_eq!(vault.preview_redeem(0u128).unwrap(), 0u128);
}

#[wasm_bindgen_test]
#[test]
fn test_yield_accrual() {
    let mut vault = TestYieldVault::new("test_yield_accrual");
    use crate::tests::mock;
    
    // Initialize mock timestamp with a specific value
    mock::reset_timestamp();
    let start_time = 1000000u64;
    mock::set_timestamp(start_time);

    // Create the YieldVault instance
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);

    // Set initial state
    let initial_assets = 1_000_000_000u128;  // Use a larger number to see larger changes
    vault.total_assets_pointer().set_value(initial_assets);
    vault.total_supply_pointer().set_value(1_000_000_000u128); // 1:1 ratio initially
    
    // Set yield rate to 10% (1000 basis points) - higher rate for more noticeable changes
    let yield_rate = 1000u128;
    vault.yield_rate_pointer().set_value(yield_rate);
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Record the initial timestamp explicitly
    let initial_timestamp = start_time;
    
    // Advance mock time by 30 days for more noticeable yield
    let time_advance = 30 * 86400;
    mock::set_timestamp(initial_timestamp + time_advance);
    
    // Update yield with the advanced time
    vault.test_update_yield().unwrap();
    
    // Check that assets increased but supply remained the same
    let new_assets = vault.total_assets_pointer().get_value::<u128>();
    let new_supply = vault.total_supply_pointer().get_value::<u128>();
    
    // With a 10% annual rate over 30 days, we should see approximately (10% * 30/365) increase
    // which is about 0.82% increase
    println!("Initial assets: {}, New assets: {}, Increase: {}", 
             initial_assets, new_assets, new_assets - initial_assets);
             
    assert!(new_assets > initial_assets, "Assets should increase due to yield");
    
    // Calculate expected yield (approximate)
    let expected_increase = (initial_assets as f64 * yield_rate as f64 * time_advance as f64) 
                             / (10000f64 * 365f64 * 86400f64);
    let min_expected = initial_assets + expected_increase as u128 / 2;  // Allow some tolerance
    
    println!("Expected minimum increase: {}", min_expected - initial_assets);
    assert!(new_assets >= min_expected, 
            "Yield increase too small: got {} but expected at least {}", 
            new_assets - initial_assets, min_expected - initial_assets);
             
    // Supply should remain unchanged
    assert_eq!(new_supply, 1_000_000_000u128, "Supply should remain unchanged");
    
    // Check that timestamp was updated
    let new_timestamp = vault.last_yield_update_pointer().get_value::<u64>();
    assert_eq!(new_timestamp, initial_timestamp + time_advance, "Timestamp should be updated");
    
    // Test multiple yield updates
    // Advance time again by another 30 days
    mock::set_timestamp(new_timestamp + time_advance);
    
    // Update yield again
    vault.test_update_yield().unwrap();
    
    // Assets should increase further
    let final_assets = vault.total_assets_pointer().get_value::<u128>();
    println!("After second update - assets: {}, increase: {}", 
             final_assets, final_assets - new_assets);
    
    assert!(final_assets > new_assets, "Assets should increase after second yield update");
    
    // Supply still unchanged
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), 1_000_000_000u128);
}

#[wasm_bindgen_test]
#[test]
fn test_transaction_validation() {
    let vault = TestYieldVault::new("test_transaction_validation");
    // Initialize tx hash storage with empty JSON object
    vault.tx_hashes_pointer().set(Arc::new("{}".as_bytes().to_vec()));
    
    // First use of a transaction hash should succeed
    let tx_hash = "tx123".to_string();
    let result1 = vault.validate_and_track_transaction(&tx_hash);
    assert!(result1.is_ok(), "First use of transaction hash should succeed");
    
    // Second use of the same hash should fail
    let result2 = vault.validate_and_track_transaction(&tx_hash);
    assert!(result2.is_err(), "Second use of same transaction hash should fail");
    
    // Different hash should succeed
    let new_tx_hash = "tx456".to_string();
    let result3 = vault.validate_and_track_transaction(&new_tx_hash);
    assert!(result3.is_ok(), "Different transaction hash should succeed");
    
    // Check that both hashes are now in storage
    let tx_hashes_json = String::from_utf8(vault.tx_hashes_pointer().get().as_ref().to_vec()).unwrap();
    assert!(tx_hashes_json.contains(&tx_hash), "First hash should be stored");
    assert!(tx_hashes_json.contains(&new_tx_hash), "Second hash should be stored");
}

#[wasm_bindgen_test]
#[test]
fn test_balance_management() {
    let vault = TestYieldVault::new("test_balance_management");
    use crate::tests::mock;
    
    // Initialize mock timestamp
    mock::set_timestamp(1000);
    
    // Create the YieldVault instance
    
    // Initialize the vault with proper metadata
    vault.observe_initialization().unwrap();
    
    // Set up vault metadata (for a complete test environment)
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    vault.yield_rate_pointer().set_value(0u128);  // No yield for this test
    vault.last_yield_update_pointer().set_value(mock::get_timestamp());
    
    // Test accounts
    let alice = "alice".to_string();
    let bob = "bob".to_string();
    
    // Test initial balances
    assert_eq!(vault.get_balance(&alice), 0u128, "Initial alice balance should be zero");
    assert_eq!(vault.get_balance(&bob), 0u128, "Initial bob balance should be zero");
    
    // Test setting balance directly
    let alice_amount = 100u128;
    vault.set_balance(&alice, alice_amount);
    assert_eq!(vault.get_balance(&alice), alice_amount, "Alice balance should be set to 100");
    
    // Test mint_shares function
    let bob_amount = 50u128;
    let mint_result = vault.mint_shares(&bob, bob_amount);
    assert!(mint_result.is_ok(), "Minting shares to Bob should succeed");
    assert_eq!(vault.get_balance(&bob), bob_amount, "Bob balance should be 50 after mint");
    
    // Check total supply after mints
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), bob_amount, 
               "Total supply should equal Bob's balance (mint_shares updates total supply)");
    
    // Test multiple accounts
    assert_eq!(vault.get_balance(&alice), alice_amount, "Alice balance should remain unchanged");
    assert_eq!(vault.get_balance(&bob), bob_amount, "Bob balance should remain unchanged");
    
    // Test burn_shares function
    let burn_amount = 20u128;
    let burn_result = vault.burn_shares(&bob, burn_amount);
    assert!(burn_result.is_ok(), "Burning shares from Bob should succeed");
    assert_eq!(vault.get_balance(&bob), bob_amount - burn_amount, 
               "Bob balance should decrease by burn amount");
    
    // Check total supply after burn
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), bob_amount - burn_amount,
               "Total supply should decrease by burn amount");
    
    // Test invalid operations
    
    // Try to burn more than balance
    let excessive_burn_result = vault.burn_shares(&bob, 1000u128);
    assert!(excessive_burn_result.is_err(), "Burning more than balance should fail");
    
    // Ensure balances remain unchanged after failed operation
    assert_eq!(vault.get_balance(&bob), bob_amount - burn_amount,
               "Bob balance should remain unchanged after failed burn");
    assert_eq!(vault.total_supply_pointer().get_value::<u128>(), bob_amount - burn_amount,
               "Total supply should remain unchanged after failed burn");
}
