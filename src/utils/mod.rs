//! Utility modules for the SLOP system
//!
//! This module provides common utilities used across the SLOP system.

// Block context implementations for both blockchain and standalone
pub mod block_context;
pub use block_context::{BlockContext, StandaloneBlockContext}; 
pub use block_context::blockchain::BlockchainContext;

// Transaction context for security
pub mod transaction_context;
pub use transaction_context::{TransactionContextExt, MockTransactionContext};

// Formal verification
pub mod formal_verification;
