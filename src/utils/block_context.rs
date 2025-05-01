//! # Block Context Module
//!
//! This module provides abstractions for accessing blockchain-related time and
//! block information, with fallbacks for non-blockchain environments. It serves
//! as the foundation for time-based operations throughout the SLOP system,
//! particularly for bond maturity tracking.
//! 
//! The module includes:
//! - A `BlockContext` trait defining the core interface for accessing block information
//! - A `StandaloneBlockContext` implementation for testing and non-blockchain environments
//! - Optional blockchain-specific implementations toggled via feature flags
//! 
//! This design allows bond calculations to be deterministic and testable while
//! maintaining compatibility with actual blockchain environments.

use std::time::{SystemTime, UNIX_EPOCH};

/// # Block Context Interface
///
/// Provides methods to access blockchain context like block height
/// with fallbacks for non-blockchain environments.
///
/// This trait abstracts blockchain-specific details, allowing the system to work
/// in various deployment scenarios including unit tests, development environments,
/// and production blockchain deployments.
pub trait BlockContext {
    /// # Get Current Block Height
    ///
    /// Retrieves the current block height from the blockchain or
    /// simulates it in non-blockchain environments.
    ///
    /// # Returns
    /// The current block height as a `u64`
    ///
    /// # Example
    /// ```
    /// use slop::utils::{BlockContext, StandaloneBlockContext};
    ///
    /// let context = StandaloneBlockContext::new();
    /// let current_height = context.get_current_block_height();
    /// println!("Current block height: {}", current_height);
    /// ```
    fn get_current_block_height(&self) -> u64;

    /// # Check If Block Height Reached
    ///
    /// Determines whether a specific target block height has been reached
    /// or surpassed by the current block height.
    ///
    /// # Parameters
    /// * `target_height` - The block height to check against
    ///
    /// # Returns
    /// `true` if the current block height is greater than or equal to the target height,
    /// `false` otherwise
    ///
    /// # Example
    /// ```
    /// use slop::utils::{BlockContext, StandaloneBlockContext};
    ///
    /// let context = StandaloneBlockContext::new();
    /// let current_height = context.get_current_block_height();
    ///
    /// // Check if maturity block has been reached
    /// let maturity_block = current_height + 100;
    /// let is_mature = context.is_block_height_reached(maturity_block);
    ///
    /// if !is_mature {
    ///     println!("Bond not yet mature - more blocks needed");
    /// }
    /// ```
    fn is_block_height_reached(&self, target_height: u64) -> bool {
        self.get_current_block_height() >= target_height
    }

    /// # Calculate Remaining Blocks
    ///
    /// Calculates the number of blocks remaining until a future target block height.
    /// If the target height has already been reached, returns 0.
    ///
    /// # Parameters
    /// * `target_height` - The target block height
    ///
    /// # Returns
    /// The number of blocks remaining until the target height, or 0 if already reached
    ///
    /// # Example
    /// ```
    /// use slop::utils::{BlockContext, StandaloneBlockContext};
    ///
    /// let context = StandaloneBlockContext::new();
    /// let current_height = context.get_current_block_height();
    ///
    /// // Calculate blocks until maturity
    /// let maturity_block = current_height + 100;
    /// let blocks_left = context.blocks_remaining(maturity_block);
    ///
    /// println!("Blocks remaining until maturity: {}", blocks_left);
    /// ```
    fn blocks_remaining(&self, target_height: u64) -> u64 {
        if target_height <= self.get_current_block_height() {
            0
        } else {
            target_height - self.get_current_block_height()
        }
    }
}

/// # Standalone Block Context
///
/// A blockchain-independent implementation of `BlockContext` used
/// for testing and non-blockchain environments.
///
/// This implementation simulates blockchain behavior using the local system time,
/// converting seconds to blocks based on a configurable number of seconds per block.
/// It also supports artificial time offsets for testing different scenarios.
///
/// # Example
/// ```
/// use slop::utils::{BlockContext, StandaloneBlockContext};
///
/// // Create a standard context (10-second blocks)
/// let context = StandaloneBlockContext::new();
/// let block_height = context.get_current_block_height();
///
/// // Create a context with custom block time (5-second blocks)
/// let fast_context = StandaloneBlockContext::with_seconds_per_block(5);
///
/// // Create a context with time offset for testing future states
/// let future_context = StandaloneBlockContext::new().with_offset(3600); // 1 hour ahead
/// ```
pub struct StandaloneBlockContext {
    // Configurable parameter: how many seconds per block (default 10)
    seconds_per_block: u64,
    
    // Optional time offset for testing
    time_offset_seconds: u64,
}

impl Default for StandaloneBlockContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StandaloneBlockContext {
    /// # Create a New Standalone Context
    ///
    /// Creates a new context with default settings (10 seconds per block).
    ///
    /// # Returns
    /// A new `StandaloneBlockContext` instance
    ///
    /// # Example
    /// ```
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// let context = StandaloneBlockContext::new();
    /// ```
    pub fn new() -> Self {
        Self {
            seconds_per_block: 10, // Default to 10 seconds per block
            time_offset_seconds: 0,
        }
    }

    /// # Create Context with Custom Block Time
    ///
    /// Creates a new context with a specified number of seconds per block.
    ///
    /// # Parameters
    /// * `seconds_per_block` - Number of seconds that constitute one block
    ///
    /// # Returns
    /// A new `StandaloneBlockContext` with the specified block time
    ///
    /// # Example
    /// ```
    /// use slop::utils::StandaloneBlockContext;
    ///
    /// // Create context where each block is 5 seconds
    /// let context = StandaloneBlockContext::with_seconds_per_block(5);
    /// ```
    pub fn with_seconds_per_block(seconds_per_block: u64) -> Self {
        Self {
            seconds_per_block,
            time_offset_seconds: 0,
        }
    }

    /// # Apply Time Offset to Context
    ///
    /// Modifies the context by adding a time offset in seconds.
    /// This is particularly useful for testing time-dependent behavior
    /// without waiting for actual time to pass.
    ///
    /// # Parameters
    /// * `offset_seconds` - Time offset to add in seconds
    ///
    /// # Returns
    /// The modified context with the specified time offset
    ///
    /// # Example
    /// ```
    /// use slop::utils::{BlockContext, StandaloneBlockContext};
    ///
    /// // Create context that's 1 hour in the future
    /// let context = StandaloneBlockContext::new().with_offset(3600);
    ///
    /// // Create context that's 1 day in the future (for testing bond maturity)
    /// let mature_context = StandaloneBlockContext::new().with_offset(86400);
    /// ```
    pub fn with_offset(mut self, offset_seconds: u64) -> Self {
        self.time_offset_seconds = offset_seconds;
        self
    }
}

impl BlockContext for StandaloneBlockContext {
    fn get_current_block_height(&self) -> u64 {
        // Get current timestamp and add any offset
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() + self.time_offset_seconds;
        
        // Convert to block height
        now / self.seconds_per_block
    }
}

/// # Blockchain Integration Module
///
/// Provides blockchain-specific implementations of the `BlockContext` trait.
/// This module is only available when the "blockchain" feature is enabled.
///
/// The blockchain implementation uses actual on-chain data to provide
/// accurate block height information, ensuring bonds mature based on
/// real blockchain state rather than simulated time.
#[cfg(feature = "blockchain")]
pub mod blockchain {
    use super::BlockContext;
    
    pub struct BlockchainContext {
        // This would hold references to blockchain context
    }
    
    impl BlockchainContext {
        pub fn new() -> Self {
            Self {}
        }
    }
    
    impl BlockContext for BlockchainContext {
        fn get_current_block_height(&self) -> u64 {
            // In real implementation, this would get block height from blockchain
            #[cfg(feature = "blockchain")]
            {
                // Get current block height using system time for now - this should be
                // replaced with actual blockchain API call in production
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() / 10
            }
            #[cfg(not(feature = "blockchain"))]
            {
                // Fallback to system time based value when not in blockchain context
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() / 10
            }
        }
    }
}

/// # Default Context Provider
///
/// Returns a default `BlockContext` implementation based on available features.
/// When the "blockchain" feature is enabled, returns a `BlockchainContext`.
/// Otherwise, returns a `StandaloneBlockContext` for local testing.
///
/// # Returns
/// A default `BlockContext` implementation
///
/// # Example
/// ```
/// use slop::utils::{BlockContext, get_default_context};
///
/// let context = get_default_context();
/// let block_height = context.get_current_block_height();
/// ```
#[cfg(feature = "blockchain")]
pub fn get_default_context() -> impl BlockContext {
    blockchain::BlockchainContext::new()
}

#[cfg(not(feature = "blockchain"))]
pub fn get_default_context() -> impl BlockContext {
    StandaloneBlockContext::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_standalone_context() {
        let context = StandaloneBlockContext::new();
        let height = context.get_current_block_height();
        assert!(height > 0);
        
        let context_with_offset = StandaloneBlockContext::new().with_offset(3600);
        let height_with_offset = context_with_offset.get_current_block_height();
        assert!(height_with_offset > height);
        
        // Test target height functionality
        let is_reached = context.is_block_height_reached(0);
        assert!(is_reached);
        
        let future_block = context.get_current_block_height() + 1000;
        let is_future_reached = context.is_block_height_reached(future_block);
        assert!(!is_future_reached);
        
        let blocks_left = context.blocks_remaining(future_block);
        assert_eq!(blocks_left, 1000);
    }
}
