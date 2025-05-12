use std::env;
use std::path::PathBuf;

fn main() {
    // Check if we're targeting wasm32
    let target = env::var("TARGET").unwrap_or_default();
    let is_wasm = target.contains("wasm32");
    
    if is_wasm {
        println!("cargo:rustc-link-search=native=/");
        return;
    }
    
    // For Apple Silicon, we need special handling
    #[cfg(target_os = "macos")]
    detect_mac_m1();
    
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(target_os = "macos")]
fn detect_mac_m1() {
    use std::process::Command;
    
    // Check if we're on Apple Silicon
    let output = Command::new("uname")
        .arg("-m")
        .output()
        .expect("Failed to execute uname command");
    
    let arch = String::from_utf8_lossy(&output.stdout);
    let is_arm64 = arch.trim() == "arm64";
    
    if is_arm64 {
        println!("cargo:warning=Detected Apple Silicon (M1/M2/M3) architecture");
        println!("cargo:warning=Using stub implementation for secp256k1-sys");
        
        // Set up paths for Apple Silicon
        if let Ok(path) = env::var("PATH") {
            if !path.contains("/usr/local/opt/llvm/bin") {
                println!("cargo:warning=Consider adding LLVM to your PATH for better compatibility:");
                println!("cargo:warning=export PATH=\"/usr/local/opt/llvm/bin:$PATH\"");
            }
        }
    }
}
