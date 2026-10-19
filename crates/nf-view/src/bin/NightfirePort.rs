//! `NightfirePort`: the one program players run. Double-clicked it is the launcher — pick the disc image once, then
//! PLAY (crates/nf-view/src/launcher/mod.rs); PLAY starts this same program as `NightfirePort <iso> --game`, which runs
//! the game (`nf_view::run`, the front end). A GUI-subsystem program on Windows, so neither mode opens a console (the
//! launcher sends the game's output to nf-view.log).
#![cfg_attr(windows, windows_subsystem = "windows")]

/// The DLL-hijack guard: from here on, DLLs load from System32 (and paths the program names itself), never from the
/// exe's own folder, which is often Downloads, where a game mod's `dxgi.dll` once loaded into us (2026-10-19). The
/// hijack-prone imports are delay-loaded (build.rs), so their first load comes after this.
#[cfg(windows)]
fn restrict_dll_search() {
    const LOAD_LIBRARY_SEARCH_SYSTEM32: u32 = 0x0000_0800;
    const LOAD_LIBRARY_SEARCH_USER_DIRS: u32 = 0x0000_0400;
    #[link(name = "kernel32")]
    extern "system" {
        fn SetDefaultDllDirectories(flags: u32) -> i32;
    }
    // SAFETY: a plain Win32 call with constant flags; no pointers.
    unsafe { SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32 | LOAD_LIBRARY_SEARCH_USER_DIRS) };
}

fn main() {
    #[cfg(windows)]
    restrict_dll_search();
    let result = if std::env::args().any(|a| a == "--game") { nf_view::run() } else { nf_view::launcher::run() };
    if let Err(e) = result {
        let msg = format!("{e:#}");
        eprintln!("NightfirePort: {msg}");
        nf_view::launcher::platform::fatal(&msg);
        std::process::exit(1);
    }
}
