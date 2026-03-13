use std::{env, path::PathBuf, process::Command};

fn rerun_if_changed(root: &PathBuf) {
    for entry in std::fs::read_dir(root).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            rerun_if_changed(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

fn main() {
    let is_windows = cfg!(target_os = "windows");

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let zig_dir = root.parent().unwrap().join("js");

    let zig_src_dir = zig_dir.join("src");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let lib_fname = if is_windows { "js.lib" } else { "libjs.a" };
    let lib = out.join(lib_fname);

    rerun_if_changed(&zig_src_dir);

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

    std::fs::copy(root.join(lib_fname), &lib).expect("Failed to copy built library");

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=js");
}
