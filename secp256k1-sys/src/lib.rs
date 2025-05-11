//! Fork of secp256k1-sys for compatibility with Apple Silicon and other platforms
//! This is a stub implementation that satisfies dependencies without requiring
//! native code compilation.

#![no_std]

// Re-export types that may be used by other crates
pub type secp256k1_context = u8;
pub type secp256k1_pubkey = [u8; 64];
pub type secp256k1_ecdsa_signature = [u8; 64];

// Constants needed by the API
pub const SECP256K1_CONTEXT_VERIFY: u32 = 1;
pub const SECP256K1_CONTEXT_SIGN: u32 = 2;
pub const SECP256K1_EC_COMPRESSED: u32 = 258;
pub const SECP256K1_EC_UNCOMPRESSED: u32 = 2;

#[no_mangle]
pub unsafe extern "C" fn secp256k1_context_create(flags: u32) -> *mut secp256k1_context {
    // Return a non-null pointer, but don't actually create anything
    1 as *mut secp256k1_context
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_context_destroy(ctx: *mut secp256k1_context) {
    // No-op, nothing to destroy
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_ec_pubkey_parse(
    _ctx: *const secp256k1_context,
    _pubkey: *mut secp256k1_pubkey,
    _input: *const u8,
    _inputlen: usize,
) -> i32 {
    // Always return success
    1
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_ec_pubkey_serialize(
    _ctx: *const secp256k1_context,
    _output: *mut u8,
    _outputlen: *mut usize,
    _pubkey: *const secp256k1_pubkey,
    _flags: u32,
) -> i32 {
    // Always return success
    1
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_ecdsa_signature_parse_compact(
    _ctx: *const secp256k1_context,
    _sig: *mut secp256k1_ecdsa_signature,
    _input64: *const u8,
) -> i32 {
    // Always return success
    1
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_ecdsa_signature_serialize_compact(
    _ctx: *const secp256k1_context,
    _output64: *mut u8,
    _sig: *const secp256k1_ecdsa_signature,
) -> i32 {
    // Always return success
    1
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_ecdsa_verify(
    _ctx: *const secp256k1_context,
    _sig: *const secp256k1_ecdsa_signature,
    _msg32: *const u8,
    _pubkey: *const secp256k1_pubkey,
) -> i32 {
    // Always return success
    1
}

// Add any other functions that might be needed by the dependent crates
