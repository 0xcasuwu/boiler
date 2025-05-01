use crate::utils::{BlockContext, TransactionContextExt};
use crate::anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;

// Mock Block Context for Testing
pub struct MockBlockContext {
    current_block_height: u64,
    current_timestamp: u64, // Not used in BlockContext trait but kept for backward compatibility
    entropy_hash: String, // Not used in BlockContext trait but kept for backward compatibility
}

impl Default for MockBlockContext {
    fn default() -> Self {
        Self {
            current_block_height: 100,
            current_timestamp: 1620000000,
            entropy_hash: "mock_entropy_hash".to_string(),
        }
    }
}

impl MockBlockContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Legacy constructor - for backward compatibility
    pub fn new_with_params(block_height: u64, timestamp: u64, entropy: &str) -> Self {
        Self {
            current_block_height: block_height,
            current_timestamp: timestamp,
            entropy_hash: entropy.to_string(),
        }
    }

    pub fn with_block_height(mut self, height: u64) -> Self {
        self.current_block_height = height;
        self
    }

    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.current_timestamp = timestamp;
        self
    }

    pub fn with_entropy(mut self, entropy: &str) -> Self {
        self.entropy_hash = entropy.to_string();
        self
    }
}

impl BlockContext for MockBlockContext {
    fn get_current_block_height(&self) -> u64 {
        self.current_block_height
    }
    
    // These methods are implemented by the trait itself using get_current_block_height
    // So we don't need to implement them here
    // fn is_block_height_reached(&self, target_height: u64) -> bool { ... }
    // fn blocks_remaining(&self, target_height: u64) -> u64 { ... }
}

// Mock Transaction Context for Testing
pub struct MockTransactionContext {
    caller_id: String,
    valid_signatures: RefCell<HashMap<String, bool>>,
    orbital_token: String,
}

impl Default for MockTransactionContext {
    fn default() -> Self {
        Self {
            caller_id: "mock_caller".to_string(),
            valid_signatures: RefCell::new(HashMap::new()),
            orbital_token: "mock_token".to_string(),
        }
    }
}

impl MockTransactionContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Legacy constructor - for backward compatibility
    pub fn new_with_caller(caller: &str) -> Self {
        Self {
            caller_id: caller.to_string(),
            valid_signatures: RefCell::new(HashMap::new()),
            orbital_token: "mock_token".to_string(),
        }
    }

    pub fn with_caller(mut self, caller: &str) -> Self {
        self.caller_id = caller.to_string();
        self
    }
    
    pub fn with_orbital_token(mut self, token_id: &str) -> Self {
        self.orbital_token = token_id.to_string();
        self
    }
    
    pub fn with_transaction_id(self, _tx_id: &str) -> Self {
        // For backward compatibility - we're not storing tx_id anymore
        self
    }

    pub fn set_signature_validity(&self, address: &str, is_valid: bool) {
        self.valid_signatures.borrow_mut().insert(address.to_string(), is_valid);
    }
}

impl TransactionContextExt for MockTransactionContext {
    fn get_caller_id(&self) -> &str {
        &self.caller_id
    }

    fn verify_signature(&self, address: &str) -> bool {
        // If we have an explicit setting for this address, use it
        if let Some(is_valid) = self.valid_signatures.borrow().get(address) {
            return *is_valid;
        }
        
        // Default behavior: Only valid for the caller's address
        address == self.caller_id
    }
    
    fn orbital_token_id(&self) -> Result<String> {
        Ok(self.orbital_token.clone())
    }
}

// Secure Transaction Context Stub for Testing
pub struct SecurityTransactionContext {
    underlying: MockTransactionContext,
}

impl SecurityTransactionContext {
    pub fn new(caller: &str) -> Self {
        Self {
            underlying: MockTransactionContext::new().with_caller(caller),
        }
    }
}

impl TransactionContextExt for SecurityTransactionContext {
    fn get_caller_id(&self) -> &str {
        self.underlying.get_caller_id()
    }

    fn verify_signature(&self, address: &str) -> bool {
        self.underlying.verify_signature(address)
    }
    
    fn orbital_token_id(&self) -> Result<String> {
        self.underlying.orbital_token_id()
    }
}
