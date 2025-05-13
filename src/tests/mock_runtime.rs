//! Mock implementations of runtime functions for testing

#[no_mangle]
pub extern "C" fn __request_storage(_key_ptr: *const u8, _key_len: usize) -> i32 {
    // Mock implementation for testing
    0
}

#[no_mangle]
pub extern "C" fn __load_storage(_dest_ptr: *mut u8, _max_len: usize) -> i32 {
    // Mock implementation for testing
    0
}

#[no_mangle]
pub extern "C" fn __store_storage(_key_ptr: *const u8, _key_len: usize, _value_ptr: *const u8, _value_len: usize) -> i32 {
    // Mock implementation for testing
    0
}

#[no_mangle]
pub extern "C" fn __get_storage_len(_key_ptr: *const u8, _key_len: usize) -> i32 {
    // Mock implementation for testing
    0
}

#[no_mangle]
pub extern "C" fn __get_block_height() -> u64 {
    // Mock implementation for testing
    0
}

#[no_mangle]
pub extern "C" fn __get_tx_hash(_dest_ptr: *mut u8) -> i32 {
    // Mock implementation for testing
    0
}

#[no_mangle]
pub extern "C" fn __get_tx_index() -> u32 {
    // Mock implementation for testing
    0
}

#[no_mangle]
pub extern "C" fn __get_block_hash(_dest_ptr: *mut u8) -> i32 {
    // Mock implementation for testing
    0
}
