use std::{env, path::PathBuf, process::Command};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let zig_dir = root.join("../js");

    let zig_src_dir = zig_dir.join("src");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib = out.join("libjs.a");

    for entry in std::fs::read_dir(&zig_src_dir).expect("Failed to read Zig source directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("zig") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    let status = Command::new("zig")
        .args([
            "build-lib",
            zig_src_dir.join("root.zig").to_str().unwrap(),
            "-O",
            "ReleaseFast",
            "-fPIC",
            "--name",
            "js",
            "-static",
        ])
        .status()
        .expect("Failed to execute zig");

    assert!(status.success(), "Zig build failed");

    println!("Built Zig library successfully, copying to output directory...");

    std::fs::copy(root.join("libjs.a"), &lib).expect("Failed to copy built library");

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=js");
}
