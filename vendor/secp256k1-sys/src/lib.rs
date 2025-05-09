#![no_std]

// This is a stub implementation of secp256k1-sys for WebAssembly compatibility
// It provides just enough symbols to satisfy the Rust compiler for WebAssembly builds

#[cfg(feature = "std")]
extern crate std;

#[repr(C)]
pub struct Context(u8);
pub type secp256k1_context = Context;

#[repr(C)]
pub struct PublicKey([u8; 64]);
pub type secp256k1_pubkey = PublicKey;

#[repr(C)]
pub struct Signature([u8; 64]);
pub type secp256k1_ecdsa_signature = Signature;

#[repr(C)]
pub struct RecoveryId(u8);
pub type secp256k1_ecdsa_recoverable_signature = RecoveryId;

pub const SECP256K1_CONTEXT_NONE: u32 = 1;
pub const SECP256K1_CONTEXT_SIGN: u32 = 2;
pub const SECP256K1_CONTEXT_VERIFY: u32 = 4;

#[no_mangle]
pub extern "C" fn secp256k1_context_create(flags: u32) -> *mut secp256k1_context {
    // This is just a stub - it returns a non-null pointer to satisfy FFI requirements
    // but doesn't actually create a context
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn secp256k1_context_destroy(_ctx: *mut secp256k1_context) {
    // This is just a stub
}

#[no_mangle]
pub extern "C" fn secp256k1_context_randomize(_ctx: *mut secp256k1_context, _seed32: *const u8) -> i32 {
    // Return success
    1
}

// Add other stub functions as needed
