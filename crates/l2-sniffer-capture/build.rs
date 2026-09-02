use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LIB");

    let arch_dir = if target.contains("aarch64") {
        "arm64"
    } else if target.contains("x86_64") {
        "x64"
    } else if target.contains("i686") || target.contains("i586") {
        "x86"
    } else {
        "x64"
    };

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_path = manifest_dir.join("../../lib").join(arch_dir);

    if lib_path.exists() {
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }
}
