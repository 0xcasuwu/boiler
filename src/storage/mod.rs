use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use std::sync::Arc;
use metashrew_support::index_pointer::KeyValuePointer;

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

    /// Returns a StoragePointer for last yield update block height
    fn last_yield_height_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/last-yield-height")
    }
    
    /// Returns a StoragePointer for the underlying asset ID
    fn asset_id_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/asset-id")
    }
    
    /// Get the AlkaneId from storage
    fn get_asset_id(&self) -> AlkaneId {
        // Get raw data from storage
        let data = self.asset_id_pointer().get();
        
        // If no data is stored yet, return default
        if data.len() == 0 {
            return AlkaneId::default();
        }
        
        // Check if we have the expected data size (32 bytes for two u128 values)
        if data.len() == 32 {
            // Extract block and tx fields
            let mut block_bytes = [0u8; 16];
            let mut tx_bytes = [0u8; 16];
            
            block_bytes.copy_from_slice(&data.as_ref()[0..16]);
            tx_bytes.copy_from_slice(&data.as_ref()[16..32]);
            
            let block = u128::from_le_bytes(block_bytes);
            let tx = u128::from_le_bytes(tx_bytes);
            
            return AlkaneId { block, tx };
        }
        
        // Fallback to default if data format is unexpected
        AlkaneId::default()
    }
    
    /// Store the AlkaneId
    fn store_asset_id(&self, asset_id: &AlkaneId) {
        // Store block and tx fields directly as serialized bytes
        // Using 16 bytes for each u128 field (little-endian encoding)
        let mut buffer = Vec::with_capacity(32);
        buffer.extend_from_slice(&asset_id.block.to_le_bytes());
        buffer.extend_from_slice(&asset_id.tx.to_le_bytes());
        
        // Store the serialized bytes
        self.asset_id_pointer().set(Arc::new(buffer));
    }

    // Account balances are no longer tracked by the contract
    // Ownership of shares is managed through native token transfers
}
