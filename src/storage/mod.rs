use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use std::sync::Arc;
use metashrew_support::index_pointer::KeyValuePointer; // Add this import

/// Storage trait for the YieldVault
pub trait Storage {
    /// Returns a StoragePointer for the vault name
    fn name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/name")
    }

    /// Returns a StoragePointer for the vault symbol
    fn symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/symbol")
    }

    /// Returns a StoragePointer for the underlying asset name
    fn asset_name_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/asset-name")
    }

    /// Returns a StoragePointer for the underlying asset symbol
    fn asset_symbol_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/asset-symbol")
    }

    /// Returns a StoragePointer for decimals
    fn decimals_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/decimals")
    }

    /// Returns a StoragePointer for total supply
    fn total_supply_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/total-supply")
    }

    /// Returns a StoragePointer for total assets
    fn total_assets_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/total-assets")
    }

    /// Returns a StoragePointer for transaction hashes (for replay protection)
    fn tx_hashes_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/tx-hashes")
    }

    /// Returns a StoragePointer for initialization status
    fn initialized_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/initialized")
    }

    /// Returns a StoragePointer for yield rate
    fn yield_rate_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/yield-rate")
    }

    /// Returns a StoragePointer for last yield update timestamp
    fn last_yield_update_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/last-yield-update")
    }
    
    /// Returns a StoragePointer for the underlying asset ID
    fn asset_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/asset-id")
    }
    
    /// Get the AlkaneId from storage
    fn get_asset_id(&self) -> AlkaneId {
        // Get raw data from storage
        let data = self.asset_id_pointer().get();
        
        // For simplicity, just use the default AlkaneId constructor
        // In a real implementation, you'd deserialize the ID properly
        AlkaneId::default()
    }
    
    /// Store the AlkaneId
    fn store_asset_id(&self, _asset_id: &AlkaneId) {
        // In a real implementation, you'd serialize the ID properly
        // For now, we'll just store a placeholder
        self.asset_id_pointer().set(Arc::new(vec![0; 32]));
    }

    /// Get balance of an account
    fn get_balance(&self, account: &str) -> u128 {
        let key = format!("/balances/{}", account);
        StoragePointer::from_keyword(&key).get_value::<u128>()
    }
    
    /// Set balance of an account
    fn set_balance(&self, account: &str, amount: u128) {
        let key = format!("/balances/{}", account);
        StoragePointer::from_keyword(&key).set_value(amount);
    }
}
