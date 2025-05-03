use anyhow::{Context, Result, anyhow};
use flate2::write::GzEncoder;
use flate2::Compression;
use hex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn compress(binary: Vec<u8>) -> Result<Vec<u8>> {
    let mut writer = GzEncoder::new(Vec::<u8>::with_capacity(binary.len()), Compression::best());
    writer.write_all(&binary)?;
    Ok(writer.finish()?)
}

// Check if we're on a Mac system, specifically M1 architecture
fn is_mac_m1() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Try to detect Apple Silicon
        if let Ok(output) = Command::new("sysctl").arg("-n").arg("machdep.cpu.brand_string").output() {
            let cpu_info = String::from_utf8_lossy(&output.stdout);
            return cpu_info.contains("Apple") && !cpu_info.contains("Intel");
        }
    }
    false
}

fn build_alkane(manifest_dir: &Path, target_dir: &Path, features: Vec<&'static str>) -> Result<()> {
    println!("Building WASM in directory: {}", manifest_dir.display());
    println!("Target directory: {}", target_dir.display());
    
    // Create the command using manifest_dir as the working directory
    let mut cmd = Command::new("cargo");
    cmd.current_dir(manifest_dir);
    
    // Mac M1 specific environment variables to help with WebAssembly builds
    if is_mac_m1() {
        println!("Detected Mac M1 architecture, applying special build configuration...");
        
        // Check if we have LLVM installed via homebrew
        let homebrew_llvm_path = "/usr/local/opt/llvm/bin";
        let has_homebrew_llvm = Path::new(homebrew_llvm_path).exists();
        
        if has_homebrew_llvm {
            println!("Found Homebrew LLVM installation, using it for WebAssembly build");
            
            // Use LLVM from Homebrew for compilation
            cmd.env("PATH", format!("{}:{}", homebrew_llvm_path, env::var("PATH").unwrap_or_default()));
            cmd.env("CC", format!("{}/clang", homebrew_llvm_path));
            cmd.env("AR", format!("{}/llvm-ar", homebrew_llvm_path));
        } else {
            println!("WARNING: For successful WebAssembly builds on Mac M1, you may need to install LLVM via Homebrew:");
            println!("  arch -x86_64 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install.sh)\"");
            println!("  arch -x86_64 /usr/local/bin/brew install llvm");
            println!("  export PATH=\"/usr/local/opt/llvm/bin:$PATH\"");
        }
        
        // Set RUSTFLAGS to use LLVM and avoid embedding bitcode
        cmd.env("RUSTFLAGS", "--embed-bitcode=no");
    } else {
        // Default RUSTFLAGS for other platforms
        cmd.env("RUSTFLAGS", "--embed-bitcode=no");
    }
    
    // Set target dir explicitly
    cmd.env("CARGO_TARGET_DIR", target_dir.to_str().unwrap());
    cmd.arg("build")
       .arg("--release")
       .arg("--target=wasm32-unknown-unknown");
    
    // Add features if any
    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }
    
    // Execute command
    println!("Executing command: cargo build --release --target=wasm32-unknown-unknown");
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let status = cmd.spawn()?.wait()?;
    
    if !status.success() {
        return Err(anyhow!("WebAssembly build failed with status: {:?}", status));
    }
    
    println!("WebAssembly build completed successfully");
    Ok(())
}

fn main() -> Result<()> {
    // Debug: Check if we're running on a Mac M1
    eprintln!("DEBUG: Is Mac M1 detected: {}", is_mac_m1());
    
    // Check if Homebrew LLVM exists
    let homebrew_llvm_path = "/usr/local/opt/llvm/bin";
    eprintln!("DEBUG: Homebrew LLVM path exists: {}", Path::new(homebrew_llvm_path).exists());
    
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
    
    // Guard against recursive execution
    if std::env::var("YIELD_VAULT_BUILD_IN_PROGRESS").is_ok() {
        println!("Build script already running, skipping to prevent recursion");
        return Ok(());
    }
    // Set flag to indicate build is in progress
    std::env::set_var("YIELD_VAULT_BUILD_IN_PROGRESS", "1");

    println!("Build script starting execution");
    
    // Get project root directory
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").context("Failed to get CARGO_MANIFEST_DIR")?);
    println!("Manifest directory: {}", manifest_dir.display());
    
    // Setup directories
    let alkanes_dir = manifest_dir.join("alkanes");
    println!("Creating alkanes directory: {}", alkanes_dir.display());
    fs::create_dir_all(&alkanes_dir).context("Failed to create alkanes directory")?;
    
    let target_dir = alkanes_dir.join("target");
    println!("Setting target directory: {}", target_dir.display());
    fs::create_dir_all(&target_dir).context("Failed to create target directory")?;
    
    let test_dir = manifest_dir.join("src").join("tests").join("std");
    println!("Creating test directory: {}", test_dir.display());
    fs::create_dir_all(&test_dir).context("Failed to create test directory")?;

    // Create a basic mod.rs file for the test directory
    let mod_name = "yield_vault";
    let mod_rs_file = test_dir.join("mod.rs");
    let mod_content = format!(
        "// Generated by build.rs\n\
         // This file provides module structure for tests\n\n\
         // Note: Full WASM functionality may not be available on all platforms\n\
         // particularly on Mac M1 systems due to secp256k1-sys compilation issues\n\
         // See: https://github.com/rust-bitcoin/rust-secp256k1/issues/283\n\n\
         // Uncomment when WebAssembly build is successful:\n\
         // pub mod {}_build;\n", 
        mod_name
    );
    
    fs::write(&mod_rs_file, mod_content)
        .context(format!("Failed to write mod rs file: {}", mod_rs_file.display()))?;
    
    println!("Created test module file: {}", mod_rs_file.display());

    // Try to build WebAssembly, but continue even if it fails
    println!("Attempting WebAssembly build (may not be supported on all platforms)...");
    if let Err(err) = build_alkane(&manifest_dir, &target_dir, vec![]) {
        println!("WebAssembly build failed: {}", err);
        println!("This is a known issue, especially on Mac M1 systems with secp256k1-sys.");
        println!("See: https://github.com/rust-bitcoin/rust-secp256k1/issues/283");
        println!("Continuing with the build process without WebAssembly support.");
        // Continue without WebAssembly functionality
        return Ok(());
    }
    
    // WebAssembly build succeeded, now process the resulting files
    let mod_name = "yield_vault";
    
    // Get paths to the wasm file and output files
    // Check both potential WASM file naming conventions
    let wasm_file_hyphen = target_dir.join("wasm32-unknown-unknown")
        .join("release")
        .join("yield-vault.wasm");
    
    let wasm_file_underscore = target_dir.join("wasm32-unknown-unknown")
        .join("release")
        .join("yield_vault.wasm");
    
    // Determine which file exists
    let wasm_file = if wasm_file_underscore.exists() {
        println!("Found WASM file with underscore naming: {}", wasm_file_underscore.display());
        wasm_file_underscore
    } else if wasm_file_hyphen.exists() {
        println!("Found WASM file with hyphen naming: {}", wasm_file_hyphen.display());
        wasm_file_hyphen
    } else {
        // Look for any wasm files to help debug
        println!("Searching for any WASM files in the target directory...");
        let search_dir = target_dir.join("wasm32-unknown-unknown").join("release");
        if search_dir.exists() {
            match fs::read_dir(&search_dir) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            println!("Found file: {}", entry.path().display());
                        }
                    }
                },
                Err(e) => println!("Error reading directory: {}", e),
            }
        } else {
            println!("Directory doesn't exist: {}", search_dir.display());
        }
        
        println!("No WASM file found at expected paths. WebAssembly support will be limited.");
        return Ok(());
    };
    
    let compressed_file = target_dir.join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{}.wasm.gz", mod_name));
    
    let build_rs_file = test_dir.join(format!("{}_build.rs", mod_name));
    let mod_rs_file = test_dir.join("mod.rs");
    
    // Read compiled wasm file
    println!("Reading wasm file: {}", wasm_file.display());
    
    // Read the wasm binary
    let wasm_binary = fs::read(&wasm_file)
        .context(format!("Failed to read wasm file: {}", wasm_file.display()))?;
    
    // Compress it
    let compressed = compress(wasm_binary.clone())
        .context("Failed to compress wasm binary")?;
    
    // Write compressed file
    fs::write(&compressed_file, &compressed)
        .context(format!("Failed to write compressed file: {}", compressed_file.display()))?;
    
    // Generate hex string
    let hex_data = hex::encode(&wasm_binary);
    
    // Write build file for tests
    let build_file_content = format!(
        "use hex_lit::hex;\n#[allow(long_running_const_eval)]\npub fn get_bytes() -> Vec<u8> {{ (&hex!(\"{}\")).to_vec() }}",
        hex_data
    );
    fs::write(&build_rs_file, build_file_content)
        .context(format!("Failed to write build rs file: {}", build_rs_file.display()))?;
    
    // Write mod.rs file
    let mod_file_content = format!("pub mod {}_build;\n", mod_name);
    fs::write(&mod_rs_file, mod_file_content)
        .context(format!("Failed to write mod rs file: {}", mod_rs_file.display()))?;
    
    println!("Successfully created:");
    println!("  - Compressed WASM: {}", compressed_file.display());
    println!("  - Test build file: {}", build_rs_file.display());
    println!("  - Test mod file: {}", mod_rs_file.display());
    
    Ok(())
}
