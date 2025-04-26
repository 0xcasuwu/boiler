// Block context utilities for handling blockchain and standalone environments
pub mod block_context;
pub mod transaction_context;

// Re-export commonly used utilities
pub use self::block_context::{BlockContext, get_default_context, StandaloneBlockContext};
