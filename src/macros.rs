//! Macros for the yield vault implementation
//!
//! This module contains macros that are used throughout the yield vault implementation.

/// Re-export the declare_alkane macro from our proc_macros crate
pub use alkanes_proc_macros::declare_alkane;

/// Macro to create a storage pointer from a keyword
#[macro_export]
macro_rules! storage_pointer {
    ($keyword:expr) => {
        StoragePointer::from_keyword($keyword)
    };
}

/// Macro to create a storage pointer from a keyword and select a key
#[macro_export]
macro_rules! storage_pointer_select {
    ($keyword:expr, $key:expr) => {
        StoragePointer::from_keyword($keyword).select($key)
    };
}

/// Macro to get a value from a storage pointer
#[macro_export]
macro_rules! get_value {
    ($pointer:expr, $type:ty) => {
        $pointer.get_value::<$type>()
    };
}

/// Macro to set a value in a storage pointer
#[macro_export]
macro_rules! set_value {
    ($pointer:expr, $value:expr, $type:ty) => {
        $pointer.set_value::<$type>($value)
    };
}
