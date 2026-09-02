use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if !target.contains("windows") {
        return;
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LIB");

    // Link delay-load helper so wpcap.dll is dynamically resolved at first call
    println!("cargo:rustc-link-lib=delayimp");
    println!("cargo:rustc-link-arg=/DELAYLOAD:wpcap.dll");

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

    // Copy Npcap DLLs to target folder for local execution convenience
    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(3) // target/{debug|release}
        .unwrap();

    let npcap_sys = Path::new("C:\\Windows\\System32\\Npcap");
    for dll in &["wpcap.dll", "Packet.dll"] {
        let src = npcap_sys.join(dll);
        let dst = target_dir.join(dll);
        if src.exists() && !dst.exists() {
            let _ = fs::copy(&src, &dst);
        }
    }
}
