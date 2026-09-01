use std::path::Path;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("windows") {
        let arch_dir = if target.contains("aarch64") {
            "arm64"
        } else if target.contains("x86_64") {
            "x64"
        } else if target.contains("i686") || target.contains("i586") {
            "x86"
        } else {
            "x64"
        };

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let root_dir = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
        let lib_path = root_dir.join("lib").join(arch_dir);

        if lib_path.exists() {
            println!("cargo:rustc-link-search=native={}", lib_path.display());
        }
    }
}
