use std::fs;
use std::path::Path;

fn main() {
    if std::env::var("TARGET")
        .map(|t| t.contains("windows"))
        .unwrap_or(false)
    {
        // Link delay-load helper so wpcap.dll is dynamically resolved at first call
        println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-link-arg=/DELAYLOAD:wpcap.dll");

        // Copy Npcap DLLs to target folder for local execution convenience
        let out_dir = std::env::var("OUT_DIR").unwrap();
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
}
