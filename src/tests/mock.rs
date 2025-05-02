// Mock functionality for testing YieldVault

use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_support::context::Context;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransferParcel;
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use metashrew_support::index_pointer::KeyValuePointer;

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

// Create a mock transaction hash as a hex string
pub fn generate_tx_hash() -> String {
    let counter = MOCK_TIMESTAMP.fetch_add(1, Ordering::SeqCst);
    format!("mock_tx_{:016x}", counter)
}

// Helper to set up a clean test environment
pub fn setup_test_environment() {
    reset_timestamp();
    storage::reset();
}

/// Mock implementation of the Context for testing
#[derive(Clone)]
pub struct MockContext {
    pub inputs: Vec<u128>,
    pub myself: AlkaneId,
    pub incoming_alkanes: AlkaneTransferParcel,
    pub caller: AlkaneId,
    pub vout: u32,
}

impl Default for MockContext {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            myself: AlkaneId::default(),
            incoming_alkanes: AlkaneTransferParcel::default(),
            caller: AlkaneId::default(),
            vout: 0,
        }
    }
}

/// Mock implementation of the AlkaneResponder trait for testing
pub struct MockAlkaneResponder {
    pub context: MockContext,
}

impl MockAlkaneResponder {
    pub fn new(context: MockContext) -> Self {
        Self { context }
    }

    pub fn with_inputs(mut self, inputs: Vec<u128>) -> Self {
        self.context.inputs = inputs;
        self
    }
}

impl AlkaneResponder for MockAlkaneResponder {
    fn context(&self) -> Result<Context> {
        Ok(Context {
            inputs: self.context.inputs.clone(),
            myself: self.context.myself.clone(),
            incoming_alkanes: self.context.incoming_alkanes.clone(),
            caller: self.context.caller.clone(),
            vout: self.context.vout,
        })
    }

    fn transaction(&self) -> Vec<u8> {
        Vec::new() // Mock implementation
    }
}

// Global storage implementation
pub mod storage {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use once_cell::sync::Lazy;

    // Global storage for tests
    static GLOBAL_STORAGE: Lazy<Arc<Mutex<HashMap<String, Vec<u8>>>>> = 
        Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

    // Reset all storage
    pub fn reset() {
        let mut storage = GLOBAL_STORAGE.lock().unwrap();
        storage.clear();
    }

    // Set a string value
    pub fn set_string(key: &str, value: &str) {
        let mut storage = GLOBAL_STORAGE.lock().unwrap();
        storage.insert(key.to_string(), value.as_bytes().to_vec());
    }

    // Get a string value
    pub fn get_string(key: &str) -> Option<String> {
        let storage = GLOBAL_STORAGE.lock().unwrap();
        storage.get(key).map(|bytes| String::from_utf8_lossy(bytes).into_owned())
    }

    // Set a u8 value
    pub fn set_u8(key: &str, value: u8) {
        let mut storage = GLOBAL_STORAGE.lock().unwrap();
        storage.insert(key.to_string(), vec![value]);
    }

    // Get a u8 value
    pub fn get_u8(key: &str) -> Option<u8> {
        let storage = GLOBAL_STORAGE.lock().unwrap();
        storage.get(key).and_then(|bytes| bytes.first().copied())
    }

    // Set a u64 value
    pub fn set_u64(key: &str, value: u64) {
        let mut storage = GLOBAL_STORAGE.lock().unwrap();
        storage.insert(key.to_string(), value.to_le_bytes().to_vec());
    }

    // Get a u64 value
    pub fn get_u64(key: &str) -> Option<u64> {
        let storage = GLOBAL_STORAGE.lock().unwrap();
        storage.get(key).and_then(|bytes| {
            if bytes.len() >= 8 {
                let mut array = [0u8; 8];
                array.copy_from_slice(&bytes[0..8]);
                Some(u64::from_le_bytes(array))
            } else {
                None
            }
        })
    }

    // Set a u128 value
    pub fn set_u128(key: &str, value: u128) {
        let mut storage = GLOBAL_STORAGE.lock().unwrap();
        storage.insert(key.to_string(), value.to_le_bytes().to_vec());
    }

    // Get a u128 value
    pub fn get_u128(key: &str) -> Option<u128> {
        let storage = GLOBAL_STORAGE.lock().unwrap();
        storage.get(key).and_then(|bytes| {
            if bytes.len() >= 16 {
                let mut array = [0u8; 16];
                array.copy_from_slice(&bytes[0..16]);
                Some(u128::from_le_bytes(array))
            } else {
                None
            }
        })
    }

    // Set a boolean value
    pub fn set_bool(key: &str, value: bool) {
        let mut storage = GLOBAL_STORAGE.lock().unwrap();
        storage.insert(key.to_string(), vec![if value { 1 } else { 0 }]);
    }

    // Get a boolean value
    pub fn get_bool(key: &str) -> Option<bool> {
        let storage = GLOBAL_STORAGE.lock().unwrap();
        storage.get(key).and_then(|bytes| bytes.first().map(|&b| b != 0))
    }

    // Get raw bytes
    pub fn get_bytes(key: &str) -> Option<Vec<u8>> {
        let storage = GLOBAL_STORAGE.lock().unwrap();
        storage.get(key).cloned()
    }

    // Set raw bytes
    pub fn set_bytes(key: &str, value: &[u8]) {
        let mut storage = GLOBAL_STORAGE.lock().unwrap();
        storage.insert(key.to_string(), value.to_vec());
    }
}

/// Mock implementation of StoragePointer for testing
#[derive(Clone)]
pub struct MockStoragePointer {
    key: String,
}

impl MockStoragePointer {
    pub fn from_keyword(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    pub fn get(&self) -> Arc<Vec<u8>> {
        Arc::new(storage::get_bytes(&self.key).unwrap_or_default())
    }

    pub fn set(&mut self, value: Arc<Vec<u8>>) {
        storage::set_bytes(&self.key, value.as_ref());
    }

    pub fn get_value<T: Default + From<Vec<u8>>>(&self) -> T {
        let bytes = storage::get_bytes(&self.key).unwrap_or_default();
        if bytes.is_empty() {
            T::default()
        } else {
            T::from(bytes)
        }
    }

    pub fn set_value<T: Into<Vec<u8>>>(&mut self, value: T) {
        let bytes: Vec<u8> = value.into();
        storage::set_bytes(&self.key, &bytes);
    }
}

// Implementation of key-value pointer traits for MockStoragePointer
impl KeyValuePointer for MockStoragePointer {
    fn set(&mut self, v: Arc<Vec<u8>>) {
        self.set(v);
    }

    fn get(&self) -> Arc<Vec<u8>> {
        self.get()
    }

    fn append(&self, v: Arc<Vec<u8>>) {
        let mut this = self.clone();
        let mut current = this.get();
        Arc::make_mut(&mut current).extend_from_slice(&v);
        this.set(current);
    }

    fn from_keyword(keyword: &str) -> Self {
        Self::from_keyword(keyword)
    }
    
    fn wrap(bytes: &Vec<u8>) -> Self {
        let mut pointer = Self::from_keyword("/temp");
        pointer.set(Arc::new(bytes.clone()));
        pointer
    }
    
    fn unwrap(&self) -> Arc<Vec<u8>> {
        self.get()
    }
    
    fn inherits(&mut self, other: &Self) {
        // Not needed for mock implementation
    }
}
