use std::sync::Arc;

use crate::storage::Storage;
use crate::security::Security;
use metashrew_support::index_pointer::KeyValuePointer;
use alkanes_runtime::storage::StoragePointer;
use anyhow::Result;
use wasm_bindgen_test::wasm_bindgen_test;
use wasm_bindgen_test::wasm_bindgen_test_configure;

// Configure wasm tests to run in browser
wasm_bindgen_test_configure!(run_in_browser);

// A test wrapper for YieldVault that uses a prefix for isolation
struct TestVault {
    prefix: String,
}

impl TestVault {
    fn new(test_name: &str) -> Self {
        Self {
            prefix: format!("/test/{}", test_name),
        }
    }

    fn get_prefixed_path(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    fn name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/name"))
    }

    fn symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/symbol"))
    }

    fn asset_name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/asset-name"))
    }

    fn asset_symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/asset-symbol"))
    }

    fn decimals_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/decimals"))
    }

    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/total-supply"))
    }

    fn total_assets_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/total-assets"))
    }

    fn yield_rate_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/yield-rate"))
    }

    fn last_yield_update_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/last-yield-update"))
    }

    fn initialized_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path("/initialized"))
    }

    fn balance_pointer(&self, account: &str) -> StoragePointer {
        StoragePointer::from_keyword(&self.get_prefixed_path(&format!("/balances/{}", account)))
    }

    fn observe_initialization(&self) -> Result<(), &'static str> {
        if self.initialized_pointer().get_value::<u8>() != 0 {
            return Err("Already initialized");
        }
        
        self.initialized_pointer().set_value(1u8);
        Ok(())
    }

    fn get_balance(&self, account: &str) -> u128 {
        self.balance_pointer(account).get_value()
    }
}

#[test]
#[wasm_bindgen_test]
fn test_initialization() {
    // Create test vault with isolated storage
    let vault = TestVault::new("init_test");

    // Verify initial state (empty/zero values)
    assert_eq!(vault.name_pointer().get().len(), 0);
    assert_eq!(vault.symbol_pointer().get().len(), 0);
}

#[test]
#[wasm_bindgen_test]
fn test_initialization_guard() {
    // Create test vault with isolated storage
    let vault = TestVault::new("guard_test");

    // First initialization should succeed
    assert!(vault.observe_initialization().is_ok());

    // Second initialization should fail
    assert!(vault.observe_initialization().is_err());
}

#[test]
#[wasm_bindgen_test]
fn test_metadata_operations() -> Result<()> {
    // Create test vault with isolated storage
    let vault = TestVault::new("metadata_test");
    
    // Initialize the vault
    vault.observe_initialization().map_err(anyhow::Error::msg)?;
    
    // Set metadata
    vault.name_pointer().set(Arc::new("Test Vault".as_bytes().to_vec()));
    vault.symbol_pointer().set(Arc::new("vTEST".as_bytes().to_vec()));
    vault.asset_name_pointer().set(Arc::new("Test Asset".as_bytes().to_vec()));
    vault.asset_symbol_pointer().set(Arc::new("TEST".as_bytes().to_vec()));
    vault.decimals_pointer().set_value(18u8);
    
    // Verify metadata
    let name = String::from_utf8(vault.name_pointer().get().as_ref().to_vec()).unwrap();
    let symbol = String::from_utf8(vault.symbol_pointer().get().as_ref().to_vec()).unwrap();
    let asset_name = String::from_utf8(vault.asset_name_pointer().get().as_ref().to_vec()).unwrap();
    let asset_symbol = String::from_utf8(vault.asset_symbol_pointer().get().as_ref().to_vec()).unwrap();
    
    assert_eq!(name, "Test Vault");
    assert_eq!(symbol, "vTEST");
    assert_eq!(asset_name, "Test Asset");
    assert_eq!(asset_symbol, "TEST");
    assert_eq!(vault.decimals_pointer().get_value::<u8>(), 18u8);
    
    Ok(())
}

#[test]
#[wasm_bindgen_test]
fn test_balance_operations() -> Result<()> {
    // Create test vault with isolated storage
    let vault = TestVault::new("balance_test");
    
    // Initialize the vault
    vault.observe_initialization().map_err(anyhow::Error::msg)?;
    
    // Test accounts
    let alice = "alice";
    let bob = "bob";
    
    // Initial balances should be zero
    assert_eq!(vault.get_balance(alice), 0u128);
    assert_eq!(vault.get_balance(bob), 0u128);
    
    // Set balances
    vault.balance_pointer(alice).set_value(100u128);
    vault.balance_pointer(bob).set_value(50u128);
    
    // Verify balances
    assert_eq!(vault.get_balance(alice), 100u128);
    assert_eq!(vault.get_balance(bob), 50u128);
    
    Ok(())
}
