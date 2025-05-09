// Stub build script for secp256k1-sys WebAssembly compatibility
// This doesn't actually build the C library, but satisfies Cargo's requirement
// for a build script when the `links` attribute is specified

fn main() {
    println!("cargo:rustc-link-lib=secp256k1");
    println!("cargo:rerun-if-changed=build.rs");
    
    // For WebAssembly target, we don't actually link to any C library
    // We just need this build script to exist
    if std::env::var("TARGET").unwrap_or_default().contains("wasm32") {
        println!("cargo:warning=Building for WebAssembly target - no actual linking performed");
        return;
    }
}
