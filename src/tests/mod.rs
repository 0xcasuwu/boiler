// Tests are being refactored gradually according to the test refactoring plan
// in memory-bank/test-refactoring-plan.md
pub mod mock;

// Phase 1: Enabled core security tests
#[cfg(test)]
mod curve_security_test;

// Tests still being refactored - will be enabled later
// #[cfg(test)]
// mod security_fixes_test;
// #[cfg(test)]
// mod penetration_tests;
// #[cfg(test)]
// mod property_tests;
// #[cfg(test)]
// mod provenance_tests;

// Make mock module available to other modules
pub use mock::*;
