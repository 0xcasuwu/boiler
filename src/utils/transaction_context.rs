use anyhow::{anyhow, Result};

/// Extension trait for transaction contexts that provides token identification
///
/// This trait enables secure bond redemption by allowing verification that
/// the transaction context contains a specific orbital token, thus proving
/// ownership of that token.
pub trait TransactionContextExt {
    /// Retrieves the orbital token ID from the transaction context
    ///
    /// # Returns
    /// * `Ok(String)` - The orbital token ID if found in the context
    /// * `Err` - If no orbital token exists in the context
    fn orbital_token_id(&self) -> Result<String>;
}

#[cfg(test)]
pub mod test_impl {
    use super::*;
    use std::cell::RefCell;
    
    /// Mock implementation of transaction context for testing
    #[derive(Debug, Default)]
    pub struct MockTransactionContext {
        pub orbital_token_id: Option<String>,
        pub transaction_id: Option<String>,
        pub logs: RefCell<Vec<String>>,
    }
    
    impl MockTransactionContext {
        /// Creates a new empty mock transaction context
        pub fn new() -> Self {
            Self {
                orbital_token_id: None,
                transaction_id: None,
                logs: RefCell::new(Vec::new()),
            }
        }
        
        /// Sets the orbital token ID for this context and returns self for chaining
        pub fn with_orbital_token(mut self, token_id: impl Into<String>) -> Self {
            self.orbital_token_id = Some(token_id.into());
            self
        }
        
        /// Sets the transaction ID for this context and returns self for chaining
        pub fn with_transaction_id(mut self, tx_id: impl Into<String>) -> Self {
            self.transaction_id = Some(tx_id.into());
            self
        }
        
        /// Logs a message for debugging purposes
        pub fn log(&self, message: &str) {
            self.logs.borrow_mut().push(message.to_string());
        }
        
        /// Retrieves all logs recorded by this context
        pub fn get_logs(&self) -> Vec<String> {
            self.logs.borrow().clone()
        }
    }
    
    impl TransactionContextExt for MockTransactionContext {
        fn orbital_token_id(&self) -> Result<String> {
            self.orbital_token_id.clone().ok_or_else(|| anyhow!("No orbital token ID in context"))
        }
    }
}
