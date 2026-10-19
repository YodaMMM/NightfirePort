// Bakes the third-party licence notices into the exe (a release is just the exe — the user, 2026-10-19). The release
// build (tools/release/make-release.ps1) writes them with notices.py and points NF_NOTICES_FILE at the file; a dev
// build gets a one-line placeholder.
fn main() {
    println!("cargo:rerun-if-env-changed=NF_NOTICES_FILE");
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("notices.txt");
    let text = match std::env::var("NF_NOTICES_FILE") {
        Ok(p) if !p.is_empty() => {
            println!("cargo:rerun-if-changed={p}");
            std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("NF_NOTICES_FILE {p}: {e}"))
        }
        _ => "Third-party licence notices are included in release builds (tools/release/notices.py).\n".to_string(),
    };
    std::fs::write(out, text).unwrap();
}
