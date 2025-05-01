// Mock functionality for testing YieldVault

use std::sync::atomic::{AtomicU64, Ordering};

// Static atomic counter for mocking timestamp progress
static MOCK_TIMESTAMP: AtomicU64 = AtomicU64::new(1672527600); // Jan 1, 2023 00:00:00 UTC

/// Get a mock timestamp that increases with each call
pub fn get_timestamp() -> u64 {
    // Advance time by 86400 seconds (1 day) each time this is called
    MOCK_TIMESTAMP.fetch_add(86400, Ordering::SeqCst)
}

/// Reset the mock timestamp to the initial value
pub fn reset_timestamp() {
    MOCK_TIMESTAMP.store(1672527600, Ordering::SeqCst);
}

/// Set the mock timestamp to a specific value
pub fn set_timestamp(timestamp: u64) {
    MOCK_TIMESTAMP.store(timestamp, Ordering::SeqCst);
}

/// Advance the mock timestamp by the specified number of seconds
pub fn advance_time(seconds: u64) {
    MOCK_TIMESTAMP.fetch_add(seconds, Ordering::SeqCst);
}

// Mock Bitcoin hash functionality for tests
#[cfg(test)]
pub fn mock_bitcoin_hash(data: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    // Simple hash algorithm for testing - just copy data or use default values
    if !data.is_empty() {
        let len = std::cmp::min(data.len(), 32);
        hash[0..len].copy_from_slice(&data[0..len]);
    }
    hash
}

// Create a mock transaction hash as a hex string
pub fn mock_tx_hash() -> String {
    let counter = MOCK_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    format!("mock_tx_{:016x}", counter)
}
