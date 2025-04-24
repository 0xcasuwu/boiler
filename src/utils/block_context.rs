use std::time::{SystemTime, UNIX_EPOCH};

/// BlockContext provides methods to access blockchain context like block height
/// with fallbacks for non-blockchain environments
pub trait BlockContext {
    /// Get the current block height
    fn get_current_block_height(&self) -> u64;

    /// Check if a specific block height has been reached
    fn is_block_height_reached(&self, target_height: u64) -> bool {
        self.get_current_block_height() >= target_height
    }

    /// Calculate blocks until target height
    fn blocks_remaining(&self, target_height: u64) -> u64 {
        if target_height <= self.get_current_block_height() {
            0
        } else {
            target_height - self.get_current_block_height()
        }
    }
}

/// StandaloneBlockContext is used when blockchain dependencies are not enabled
/// It simulates blockchain behavior using local system time
pub struct StandaloneBlockContext {
    // Configurable parameter: how many seconds per block (default 10)
    seconds_per_block: u64,
    
    // Optional time offset for testing
    time_offset_seconds: u64,
}

impl StandaloneBlockContext {
    pub fn new() -> Self {
        Self {
            seconds_per_block: 10, // Default to 10 seconds per block
            time_offset_seconds: 0,
        }
    }

    pub fn with_seconds_per_block(seconds_per_block: u64) -> Self {
        Self {
            seconds_per_block,
            time_offset_seconds: 0,
        }
    }

    // For testing: allows setting a time offset
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

/// When blockchain features are enabled, this will use actual blockchain data
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
            // Using the runtime context from alkanes_runtime
            crate::blockchain::alkanes_runtime::get_block_height()
        }
    }
}

// Default context provider - will use blockchain version when feature enabled
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
