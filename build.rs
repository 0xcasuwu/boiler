use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use hex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

fn compress(binary: Vec<u8>) -> Result<Vec<u8>> {
    let mut writer = GzEncoder::new(Vec::<u8>::with_capacity(binary.len()), Compression::best());
    writer.write_all(&binary)?;
    Ok(writer.finish()?)
}

fn build_alkane(wasm_str: &str, features: Vec<&'static str>) -> Result<()> {
    if features.len() != 0 {
        let _ = Command::new("cargo")
            .env("CARGO_TARGET_DIR", wasm_str)
            .arg("build")
            .arg("--release")
            .arg("--features")
            .arg(features.join(","))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    } else {
        Command::new("cargo")
            .env("CARGO_TARGET_DIR", wasm_str)
            .arg("build")
            .arg("--release")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

fn main() {
    // Guard against recursive execution
    if std::env::var("SLOP_BUILD_IN_PROGRESS").is_ok() {
        println!("Build script already running, skipping to prevent recursion");
        return;
    }
    // Set flag to indicate build is in progress
    std::env::set_var("SLOP_BUILD_IN_PROGRESS", "1");
    let env_var = env::var_os("OUT_DIR").unwrap();
    let base_dir = Path::new(&env_var)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let out_dir = base_dir.join("release");
    let wasm_dir = base_dir.parent().unwrap().join("alkanes");
    fs::create_dir_all(&wasm_dir).unwrap();
    let wasm_str = wasm_dir.to_str().unwrap();
    let write_dir = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src")
        .join("tests");

    fs::create_dir_all(&write_dir.join("std")).unwrap();
    let crates_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    std::env::set_current_dir(&crates_dir).unwrap();

    // Use a separate target directory specifically for the WASM build
    // Align with Free-mint: don't use feature flags during build
    build_alkane(wasm_str, vec![]).unwrap();

    // Generate builds for each of our contracts - follows free-mint's flat structure pattern
    let contract_files = [
        "slop_orbital_bond_collection",
        "slop_launchpad_factory", 
        "slop_bond_curve"
    ];
    
    let mut mod_rs_content = String::new();

    // Process each contract file
    for mod_name in &contract_files {
        let wasm_path = Path::new(&wasm_str)
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(format!("{}.wasm", mod_name));

        // Skip if WASM file doesn't exist for this contract
        if !wasm_path.exists() {
            eprintln!("Warning: WASM file not found for {}", mod_name);
            continue;
        }

        eprintln!(
            "Processing {}.wasm...",
            mod_name
        );

        let f: Vec<u8> = fs::read(&wasm_path).unwrap();
        let compressed: Vec<u8> = compress(f.clone()).unwrap();

        // Save the compressed version for deployment
        fs::write(
            &Path::new(&wasm_str)
                .join("wasm32-unknown-unknown")
                .join("release")
                .join(format!("{}.wasm.gz", mod_name)),
            &compressed,
        ).unwrap();

        // Create test file to load the binary data
        let data: String = hex::encode(&f);
        fs::write(
            &write_dir.join("std").join(format!("{}_build.rs", mod_name)),
            format!(
                "use hex_lit::hex;\n#[allow(long_running_const_eval)]\npub fn get_bytes() -> Vec<u8> {{ (&hex!(\"{}\")).to_vec() }}",
                data
            ),
        ).unwrap();

        // Add to mod.rs content
        mod_rs_content.push_str(&format!("pub mod {}_build;\n", mod_name));
    }

    // Write the mod.rs file with all our modules
    eprintln!(
        "Writing test builds module to: {}",
        write_dir
            .join("std")
            .join("mod.rs")
            .into_os_string()
            .to_str()
            .unwrap()
    );

    fs::write(&write_dir.join("std").join("mod.rs"), mod_rs_content).unwrap();
    eprintln!("Build script completed successfully");
}
