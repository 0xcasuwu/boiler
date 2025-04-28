// Block context utilities for handling blockchain and standalone environments
pub mod block_context;
pub mod transaction_context;
pub mod formal_verification;

// Re-export commonly used utilities
pub use self::block_context::{BlockContext, get_default_context, StandaloneBlockContext};
pub use self::formal_verification::{FinancialInvariant, PreCondition, PostCondition, FinancialVerificationHarness, VerificationError};
