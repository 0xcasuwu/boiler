// Build script for YieldVault
// Handles WebAssembly compilation setup

fn main() {
    // Tells Cargo to re-run this script if any of these files change
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/constants.rs");
    println!("cargo:rerun-if-changed=build.rs");
    
    // Set optimization level for WebAssembly
    println!("cargo:rustc-flag=-Copt-level=3");
    println!("cargo:rustc-flag=-Clto=true");
    println!("cargo:rustc-flag=-Ccodegen-units=1");
    
    // Add additional flags for wasm target
    if cfg!(target_arch = "wasm32") {
        // Ensure proper WebAssembly generation
        println!("cargo:rustc-flag=-Clink-arg=--import-memory");
        println!("cargo:rustc-flag=-Clink-arg=--export-table");
    }
}
