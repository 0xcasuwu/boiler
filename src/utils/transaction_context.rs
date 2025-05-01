//! Transaction Context for Security
//!
//! This module provides utilities for working with transaction contexts
//! in both blockchain and standalone environments.

use anyhow::Result;

/// Transaction context extension trait
/// 
/// This trait defines methods that transaction context implementations
/// should provide for identity and security verification.
pub trait TransactionContextExt {
    /// Get the caller ID for the current transaction
    fn get_caller_id(&self) -> &str;
    
    /// Verify a signature from the given address
    fn verify_signature(&self, address: &str) -> bool;
    
    /// Get the orbital token ID associated with the transaction
    fn orbital_token_id(&self) -> Result<String>;
}

/// A mock transaction context for testing
pub struct MockTransactionContext {
    caller_id: String,
}

impl MockTransactionContext {
    /// Create a new mock transaction context with default values
    pub fn new() -> Self {
        Self {
            caller_id: "default_caller".to_string(),
        }
    }
    
    /// Create a new mock transaction context with a specific caller ID
    pub fn new_with_caller(caller_id: &str) -> Self {
        Self {
            caller_id: caller_id.to_string(),
        }
    }

    /// Set the caller ID for this transaction context
    pub fn with_caller(mut self, caller_id: &str) -> Self {
        self.caller_id = caller_id.to_string();
        self
    }
}

impl TransactionContextExt for MockTransactionContext {
    fn get_caller_id(&self) -> &str {
        &self.caller_id
    }
    
    fn verify_signature(&self, _address: &str) -> bool {
        // Always return true in test mode
        true
    }
    
    fn orbital_token_id(&self) -> Result<String> {
        // Return a mock token ID
        Ok("mock-token-id".to_string())
    }
}
