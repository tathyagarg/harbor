use std::{env, path::PathBuf, process::Command};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let zig_dir = root.join("../js");

    let zig_src = zig_dir.join("src/root.zig");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib = out.join("libjs.a");

    println!("cargo:rerun-if-changed={}", zig_src.display());

    let status = Command::new("zig")
        .args([
            "build-lib",
            zig_src.to_str().unwrap(),
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
