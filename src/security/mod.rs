use alkanes_runtime::storage::StoragePointer;
use metashrew_support::index_pointer::KeyValuePointer; // Add this import
use crate::storage::Storage;
use std::collections::HashSet;
use std::sync::Arc;
use serde_json;

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
    fn validate_and_track_transaction(&self, tx_hash: &str) -> Result<(), &'static str> {
        // Get the current set of transaction hashes
        let json_data = self.tx_hashes_pointer().get();
        
        let tx_hashes: HashSet<String> = if json_data.len() == 0 {
            HashSet::new()
        } else {
            // Convert bytes to string, return empty set if conversion fails
            let json = match String::from_utf8(json_data.as_ref().to_vec()) {
                Ok(s) => s,
                Err(_) => return Err("Failed to parse transaction hashes"),
            };
            
            // Parse JSON, return empty set if parsing fails
            serde_json::from_str(&json).unwrap_or_else(|_| HashSet::new())
        };
        
        // Check if this transaction hash has been used
        if tx_hashes.contains(tx_hash) {
            return Err("Transaction hash already used");
        }
        
        // Add the transaction hash to the set
        let mut new_tx_hashes = tx_hashes;
        new_tx_hashes.insert(tx_hash.to_string());
        
        // Serialize to JSON, handle failures
        let json = match serde_json::to_string(&new_tx_hashes) {
            Ok(j) => j,
            Err(_) => return Err("Failed to serialize transaction hashes"),
        };
        
        // Store the serialized data
        let bytes = json.as_bytes().to_vec();
        self.tx_hashes_pointer().set(Arc::new(bytes));
        
        Ok(())
    }
    
    /// Check if caller has sufficient authorization
    fn check_authorization(&self, caller: &str, owner: &str) -> Result<(), &'static str> {
        // In this simple implementation, only the owner can operate on their assets
        // In a more complex implementation, this would check approved operators
        if caller != owner {
            return Err("Not authorized");
        }
        Ok(())
    }
}
