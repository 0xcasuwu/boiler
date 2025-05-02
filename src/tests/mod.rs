// Test modules export
pub mod mock;
pub mod test_utils;
pub mod basic_tests;

// Re-export mock module for use in the main lib
pub use mock::MockStoragePointer;
