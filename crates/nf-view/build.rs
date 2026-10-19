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

    // The DLL-hijack guard (the user, 2026-10-19: NightfirePort.exe run from Downloads loaded a game mod's
    // `dxgi.dll` sitting beside it, which popped "UGameViewportClient::DrawTransition signature failed" boxes).
    // Windows looks in the exe's own folder first for every DLL that isn't a KnownDLL, and these are linked at load
    // time, before `main` can say otherwise. Delay-load them: their first use comes after `main` has restricted the
    // search to System32 (`bin/NightfirePort.rs`, SetDefaultDllDirectories).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        for dll in DELAY_LOADED {
            println!("cargo:rustc-link-arg-bin=NightfirePort=/DELAYLOAD:{dll}");
        }
        println!("cargo:rustc-link-arg-bin=NightfirePort=delayimp.lib");
    }
}

/// The hijack-prone (non-KnownDLL) system DLLs the exe imports — the names mod loaders and overlays ship as proxies.
/// tools/release/check-imports.py verifies the release exe imports none of them directly.
const DELAY_LOADED: [&str; 5] = ["dxgi.dll", "dwmapi.dll", "uxtheme.dll", "opengl32.dll", "mmdevapi.dll"];
