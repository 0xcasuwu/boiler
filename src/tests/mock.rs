// YieldVault Mock Module
// Mock implementations for testing

use std::collections::HashMap;
use std::sync::Mutex;
use std::collections::HashSet;
use std::sync::Arc;
use lazy_static::lazy_static;
use serde_json;

// Mock storage implementation
lazy_static! {
    pub static ref STORAGE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
    pub static ref TIMESTAMP: Mutex<u64> = Mutex::new(1651388400); // May 1, 2022 12:00:00 PM UTC
}

// Mock implementation of storage for testing
pub mod storage {
    use super::*;
    use alkanes_runtime::storage::StoragePointer;
    
    // Mock implementation of StoragePointer for testing
    pub struct MockStoragePointer {
        pub path: String
    }
    
    impl MockStoragePointer {
        pub fn from_keyword(path: &str) -> Self {
            Self { path: path.to_string() }
        }
        
        pub fn get(&self) -> Arc<Vec<u8>> {
            let storage = STORAGE.lock().unwrap();
            let data = storage.get(&self.path).cloned().unwrap_or_default();
            Arc::new(data)
        }
        
        pub fn set(&self, value: Arc<Vec<u8>>) {
            let mut storage = STORAGE.lock().unwrap();
            storage.insert(self.path.clone(), value.as_ref().clone());
        }
        
        pub fn get_value<T: Copy + Default>(&self) -> T {
            let storage = STORAGE.lock().unwrap();
            
            if let Some(bytes) = storage.get(&self.path) {
                if std::mem::size_of::<T>() <= bytes.len() {
                    unsafe {
                        let mut value = std::mem::MaybeUninit::<T>::uninit();
                        std::ptr::copy_nonoverlapping(
                            bytes.as_ptr(),
                            value.as_mut_ptr() as *mut u8,
                            std::mem::size_of::<T>(),
                        );
                        return value.assume_init();
                    }
                }
            }
            
            T::default()
        }
        
        pub fn set_value<T: Copy>(&self, value: T) {
            let mut storage = STORAGE.lock().unwrap();
            
            let mut bytes = vec![0u8; std::mem::size_of::<T>()];
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &value as *const T as *const u8,
                    bytes.as_mut_ptr(),
                    std::mem::size_of::<T>(),
                );
            }
            
            storage.insert(self.path.clone(), bytes);
        }
        
        pub fn select(&self, key: &[u8]) -> Self {
            let new_path = format!("{}/{}", self.path, hex::encode(key));
            Self { path: new_path }
        }
    }
    
    // Override the StoragePointer from alkanes_runtime
    impl From<MockStoragePointer> for StoragePointer {
        fn from(mock: MockStoragePointer) -> Self {
            StoragePointer::from_keyword(&mock.path)
        }
    }
    
    pub fn clear() {
        let mut storage = STORAGE.lock().unwrap();
        storage.clear();
    }
    
    // Testing helpers
    pub fn get_string(path: &str) -> Option<String> {
        let storage = STORAGE.lock().unwrap();
        storage.get(path).map(|bytes| {
            String::from_utf8(bytes.clone()).unwrap_or_default()
        })
    }
    
    pub fn set_string(path: &str, value: &str) {
        let mut storage = STORAGE.lock().unwrap();
        storage.insert(path.to_string(), value.as_bytes().to_vec());
    }
    
    pub fn get_u128(path: &str) -> Option<u128> {
        let storage = STORAGE.lock().unwrap();
        storage.get(path).map(|bytes| {
            let mut buf = [0u8; 16];
            let len = std::cmp::min(bytes.len(), 16);
            buf[..len].copy_from_slice(&bytes[..len]);
            u128::from_le_bytes(buf)
        })
    }
    
    pub fn set_u128(path: &str, value: u128) {
        let mut storage = STORAGE.lock().unwrap();
        storage.insert(path.to_string(), value.to_le_bytes().to_vec());
    }
    
    pub fn get_u64(path: &str) -> Option<u64> {
        let storage = STORAGE.lock().unwrap();
        storage.get(path).map(|bytes| {
            let mut buf = [0u8; 8];
            let len = std::cmp::min(bytes.len(), 8);
            buf[..len].copy_from_slice(&bytes[..len]);
            u64::from_le_bytes(buf)
        })
    }
    
    pub fn set_u64(path: &str, value: u64) {
        let mut storage = STORAGE.lock().unwrap();
        storage.insert(path.to_string(), value.to_le_bytes().to_vec());
    }
    
    pub fn get_u8(path: &str) -> Option<u8> {
        let storage = STORAGE.lock().unwrap();
        storage.get(path).map(|bytes| {
            bytes[0]
        })
    }
    
    pub fn set_u8(path: &str, value: u8) {
        let mut storage = STORAGE.lock().unwrap();
        storage.insert(path.to_string(), vec![value]);
    }
    
    pub fn get_bool(path: &str) -> Option<bool> {
        let storage = STORAGE.lock().unwrap();
        storage.get(path).map(|bytes| {
            !bytes.is_empty() && bytes[0] != 0
        })
    }
    
    pub fn set_bool(path: &str, value: bool) {
        let mut storage = STORAGE.lock().unwrap();
        storage.insert(path.to_string(), vec![value as u8]);
    }
}

// Timestamp utilities
pub fn get_timestamp() -> u64 {
    *TIMESTAMP.lock().unwrap()
}

pub fn set_timestamp(time: u64) {
    let mut timestamp = TIMESTAMP.lock().unwrap();
    *timestamp = time;
}

pub fn advance_time(seconds: u64) {
    let mut timestamp = TIMESTAMP.lock().unwrap();
    *timestamp += seconds;
}

// Transaction hash generation
pub fn generate_tx_hash() -> String {
    let timestamp = get_timestamp();
    format!("tx_{}", timestamp)
}

// Mock Context for testing
pub struct MockContext {
    pub incoming_alkanes: Vec<alkanes_support::parcel::AlkaneTransfer>,
    pub myself: Vec<u8>,
}

impl Default for MockContext {
    fn default() -> Self {
        Self {
            incoming_alkanes: Vec::new(),
            myself: vec![0; 32],
        }
    }
}

// Helper to set up a clean environment for each test
pub fn setup_test_environment() {
    storage::clear();
    set_timestamp(1651388400); // May 1, 2022 12:00:00 PM UTC
}
