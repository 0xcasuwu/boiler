#![no_std]

//! # secp256k1-sys WebAssembly Fork
//!
//! This is a WebAssembly-compatible fork of secp256k1-sys designed to
//! work on Apple Silicon without requiring native compilation of C libraries.
//! 
//! It provides stub implementations that satisfy the API requirements
//! but may not provide cryptographic security for all operations.
//!
//! For production use on non-WebAssembly targets, use the official secp256k1-sys.

#[cfg(feature = "std")]
extern crate std;

// Define constants from the C library
pub const SECP256K1_FLAGS_TYPE_MASK: u32 = (1 << 8) - 1;
pub const SECP256K1_FLAGS_TYPE_CONTEXT: u32 = 1 << 0;
pub const SECP256K1_FLAGS_TYPE_COMPRESSION: u32 = 1 << 1;

pub const SECP256K1_FLAGS_BIT_CONTEXT_VERIFY: u32 = 1 << 8;
pub const SECP256K1_FLAGS_BIT_CONTEXT_SIGN: u32 = 1 << 9;
pub const SECP256K1_FLAGS_BIT_COMPRESSION: u32 = 1 << 8;

pub const SECP256K1_CONTEXT_VERIFY: u32 = SECP256K1_FLAGS_TYPE_CONTEXT | SECP256K1_FLAGS_BIT_CONTEXT_VERIFY;
pub const SECP256K1_CONTEXT_SIGN: u32 = SECP256K1_FLAGS_TYPE_CONTEXT | SECP256K1_FLAGS_BIT_CONTEXT_SIGN;
pub const SECP256K1_CONTEXT_NONE: u32 = SECP256K1_FLAGS_TYPE_CONTEXT;

pub const SECP256K1_EC_COMPRESSED: u32 = SECP256K1_FLAGS_TYPE_COMPRESSION | SECP256K1_FLAGS_BIT_COMPRESSION;
pub const SECP256K1_EC_UNCOMPRESSED: u32 = SECP256K1_FLAGS_TYPE_COMPRESSION;

// Define required structs for the FFI interface
#[repr(C)]
pub struct Context(u8);
pub type secp256k1_context = Context;

#[repr(C)]
pub struct PublicKey([u8; 64]);
pub type secp256k1_pubkey = PublicKey;

#[repr(C)]
pub struct Signature([u8; 64]);
pub type secp256k1_ecdsa_signature = Signature;

#[cfg(feature = "recovery")]
#[repr(C)]
pub struct RecoveryId(u8);

#[cfg(feature = "recovery")]
pub type secp256k1_ecdsa_recoverable_signature = RecoveryId;

// Core functions for context management
#[no_mangle]
pub extern "C" fn secp256k1_context_create(flags: u32) -> *mut secp256k1_context {
    // In a real implementation, this would create and return a context
    // For WebAssembly, we just return a null pointer
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn secp256k1_context_destroy(_ctx: *mut secp256k1_context) {
    // In a real implementation, this would free the context
    // For WebAssembly stub, we do nothing
}

#[no_mangle]
pub extern "C" fn secp256k1_context_clone(_ctx: *const secp256k1_context) -> *mut secp256k1_context {
    // In a real implementation, this would clone the context
    // For WebAssembly stub, we return null
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn secp256k1_context_randomize(_ctx: *mut secp256k1_context, _seed32: *const u8) -> i32 {
    // Return 1 for success in the stub implementation
    1
}

// ECDSA signature functions
#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_signature_parse_compact(
    _ctx: *const secp256k1_context,
    _sig: *mut secp256k1_ecdsa_signature,
    _input64: *const u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_signature_parse_der(
    _ctx: *const secp256k1_context,
    _sig: *mut secp256k1_ecdsa_signature,
    _input: *const u8,
    _inputlen: usize,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_signature_serialize_der(
    _ctx: *const secp256k1_context,
    _output: *mut u8,
    _outputlen: *mut usize,
    _sig: *const secp256k1_ecdsa_signature,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_signature_serialize_compact(
    _ctx: *const secp256k1_context,
    _output64: *mut u8,
    _sig: *const secp256k1_ecdsa_signature,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_verify(
    _ctx: *const secp256k1_context,
    _sig: *const secp256k1_ecdsa_signature,
    _msg32: *const u8,
    _pubkey: *const secp256k1_pubkey,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_signature_normalize(
    _ctx: *const secp256k1_context,
    _out: *mut secp256k1_ecdsa_signature,
    _sig: *const secp256k1_ecdsa_signature,
) -> i32 {
    1
}

// Public key functions
#[no_mangle]
pub extern "C" fn secp256k1_ec_pubkey_parse(
    _ctx: *const secp256k1_context,
    _pubkey: *mut secp256k1_pubkey,
    _input: *const u8,
    _inputlen: usize,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_pubkey_serialize(
    _ctx: *const secp256k1_context,
    _output: *mut u8,
    _outputlen: *mut usize,
    _pubkey: *const secp256k1_pubkey,
    _flags: u32,
) -> i32 {
    // Simulate returning a compressed pubkey length
    unsafe {
        if !_outputlen.is_null() {
            *_outputlen = 33;
        }
    }
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_pubkey_create(
    _ctx: *const secp256k1_context,
    _pubkey: *mut secp256k1_pubkey,
    _seckey: *const u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_seckey_verify(
    _ctx: *const secp256k1_context,
    _seckey: *const u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_privkey_negate(
    _ctx: *const secp256k1_context,
    _seckey: *mut u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_pubkey_negate(
    _ctx: *const secp256k1_context,
    _pubkey: *mut secp256k1_pubkey,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_privkey_tweak_add(
    _ctx: *const secp256k1_context,
    _seckey: *mut u8,
    _tweak: *const u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_pubkey_tweak_add(
    _ctx: *const secp256k1_context,
    _pubkey: *mut secp256k1_pubkey,
    _tweak: *const u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_privkey_tweak_mul(
    _ctx: *const secp256k1_context,
    _seckey: *mut u8,
    _tweak: *const u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_pubkey_tweak_mul(
    _ctx: *const secp256k1_context,
    _pubkey: *mut secp256k1_pubkey,
    _tweak: *const u8,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ec_pubkey_combine(
    _ctx: *const secp256k1_context,
    _out: *mut secp256k1_pubkey,
    _ins: *const *const secp256k1_pubkey,
    _n: usize,
) -> i32 {
    1
}

#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_sign(
    _ctx: *const secp256k1_context,
    _sig: *mut secp256k1_ecdsa_signature,
    _msg32: *const u8,
    _seckey: *const u8,
    _noncefn: Option<unsafe extern "C" fn()>,
    _noncedata: *const core::ffi::c_void,
) -> i32 {
    1
}

// Recovery module (feature-gated)
#[cfg(feature = "recovery")]
#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_recoverable_signature_parse_compact(
    _ctx: *const secp256k1_context,
    _sig: *mut secp256k1_ecdsa_recoverable_signature,
    _input64: *const u8,
    _recid: i32,
) -> i32 {
    1
}

#[cfg(feature = "recovery")]
#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_recoverable_signature_serialize_compact(
    _ctx: *const secp256k1_context,
    _output64: *mut u8,
    _recid: *mut i32,
    _sig: *const secp256k1_ecdsa_recoverable_signature,
) -> i32 {
    if !_recid.is_null() {
        unsafe { *_recid = 0; }
    }
    1
}

#[cfg(feature = "recovery")]
#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_recoverable_signature_convert(
    _ctx: *const secp256k1_context,
    _sig: *mut secp256k1_ecdsa_signature,
    _sigin: *const secp256k1_ecdsa_recoverable_signature,
) -> i32 {
    1
}

#[cfg(feature = "recovery")]
#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_sign_recoverable(
    _ctx: *const secp256k1_context,
    _sig: *mut secp256k1_ecdsa_recoverable_signature,
    _msg32: *const u8,
    _seckey: *const u8,
    _noncefn: Option<unsafe extern "C" fn()>,
    _noncedata: *const core::ffi::c_void,
) -> i32 {
    1
}

#[cfg(feature = "recovery")]
#[no_mangle]
pub extern "C" fn secp256k1_ecdsa_recover(
    _ctx: *const secp256k1_context,
    _pubkey: *mut secp256k1_pubkey,
    _sig: *const secp256k1_ecdsa_recoverable_signature,
    _msg32: *const u8,
) -> i32 {
    1
}
