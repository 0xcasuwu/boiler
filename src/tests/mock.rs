use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::utils::BlockContext;
// Re-export for backward compatibility with tests
pub use crate::utils::transaction_context::TransactionContextExt;

/// Mock implementation of BlockContext trait for testing
pub struct MockBlockContext {
    current_block_height: u64,
    seconds_per_block: u64,
}

impl Default for MockBlockContext {
    fn default() -> Self {
        Self {
            current_block_height: 100, // Start at a reasonable default
            seconds_per_block: 1,      // Default 1 second per block
        }
    }
}

impl MockBlockContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_block_height(mut self, height: u64) -> Self {
        self.current_block_height = height;
        self
    }

    pub fn with_seconds_per_block(mut self, seconds: u64) -> Self {
        self.seconds_per_block = seconds;
        self
    }

    pub fn advance_blocks(&mut self, blocks: u64) {
        self.current_block_height += blocks;
    }
}

impl BlockContext for MockBlockContext {
    fn get_current_block_height(&self) -> u64 {
        self.current_block_height
    }
    
    // Explicitly implement this method to ensure it works in tests
    fn is_block_height_reached(&self, target_height: u64) -> bool {
        self.current_block_height >= target_height
    }
    
    // Explicitly implement this method to ensure it works in tests
    fn blocks_remaining(&self, target_height: u64) -> u64 {
        if target_height <= self.current_block_height {
            0
        } else {
            target_height - self.current_block_height
        }
    }
}

/// Mock Storage for testing
pub struct MockStorage {
    storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl Default for MockStorage {
    fn default() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_value(self, key: &str, value: &[u8]) -> Self {
        self.storage.lock().unwrap().insert(key.to_string(), value.to_vec());
        self
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.storage.lock().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: &str, value: &[u8]) {
        self.storage.lock().unwrap().insert(key.to_string(), value.to_vec());
    }
}

/// Mock Context that simulates transaction context
pub struct MockTransactionContext {
    pub orbital_token_id: Option<String>,
    pub transaction_id: String,
}

impl Default for MockTransactionContext {
    fn default() -> Self {
        Self {
            orbital_token_id: None,
            transaction_id: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        }
    }
}

impl MockTransactionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_orbital_token(mut self, orbital_token_id: &str) -> Self {
        self.orbital_token_id = Some(orbital_token_id.to_string());
        self
    }

    pub fn with_transaction_id(mut self, tx_id: &str) -> Self {
        self.transaction_id = tx_id.to_string();
        self
    }
}

/// Implementation of TransactionContextExt for MockTransactionContext
impl TransactionContextExt for MockTransactionContext {
    fn orbital_token_id(&self) -> anyhow::Result<String> {
        self.orbital_token_id.clone()
            .ok_or_else(|| anyhow::anyhow!("No orbital token ID in context"))
    }
}
