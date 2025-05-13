use metashrew_support::index_pointer::KeyValuePointer;
use crate::storage::Storage;
// No longer needed: use std::collections::HashSet;
// No longer needed: use serde_json;

/// Security trait for the YieldVault
pub trait Security: Storage {
    /// Observe initialization to prevent multiple initializations
    fn observe_initialization(&self) -> Result<(), &'static str> {
        // Use u8 instead of bool for storage compatibility (0 = false, 1 = true)
        if self.initialized_pointer().get_value::<u8>() != 0 {
            return Err("Already initialized");
        }
        self.initialized_pointer().set_value(1u8);
        Ok(())
    }
    
    /// For testing: Reset the initialized flag
    #[cfg(test)]
    fn reset_initialization(&self) {
        self.initialized_pointer().set_value(0u8);
    }
    
    /// Validate and track a transaction hash to prevent replay attacks
    /// This implementation is aligned with free-mint's approach
    fn validate_and_track_transaction(&self, tx_hash: &str) -> Result<(), &'static str> {
        // Convert the string hash to bytes for storage
        // In a real implementation, this would be a proper Txid object
        let tx_bytes = tx_hash.as_bytes().to_vec();
        
        // Check if this transaction hash has been used
        // We're using the same pattern as free-mint: select the exact key and check its value
        if self.tx_hashes_pointer()
            .select(&tx_bytes)
            .get_value::<u8>() == 1 {
            return Err("Transaction hash already used");
        }
        
        // Mark this transaction hash as used with a binary flag (1)
        // This directly aligns with free-mint's implementation:
        // StoragePointer::from_keyword("/tx-hashes/")
        //     .select(&txid.as_byte_array().to_vec())
        //     .set_value::<u8>(0x01);
        self.tx_hashes_pointer()
            .select(&tx_bytes)
            .set_value(1u8);
        
        Ok(())
    }
    
    // Authorization is now handled through token possession
    // The contract doesn't need to check who owns what shares
}
