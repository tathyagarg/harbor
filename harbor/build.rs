use std::{env, path::PathBuf, process::Command};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let c_dir = root.join("../js");

    let include = c_dir.join("include");
    let src = c_dir.join("src");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", src.join("js.c").display());
    println!("cargo:rerun-if-changed={}", include.join("js.h").display());

    let obj = out.join("js.o");
    let lib = out.join("libjsruntime.a");

    let status = Command::new("cc")
        .args([
            "-c",
            src.join("js.c").to_str().unwrap(),
            "-I",
            include.to_str().unwrap(),
            "-o",
            obj.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("ar")
        .args(["crus", lib.to_str().unwrap(), obj.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=jsruntime");
}
