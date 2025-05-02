// Test modules export
pub mod mock;
pub mod test_utils;
pub mod basic_tests;
pub mod adversarial_tests;
pub mod unit_tests;

// Re-export mock module for use in the main lib
pub use mock::MockStoragePointer;
