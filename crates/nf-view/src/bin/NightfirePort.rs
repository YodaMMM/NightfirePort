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

/// `--game` started by something other than our launcher (another front end, a shortcut): a GUI-subsystem
/// program then has no stderr, so the game's output — the startup lines a crash report needs — went nowhere.
/// With no stderr handle, open `<config dir>/nf-view.log` (the launcher's own log file) and make it stderr;
/// `eprintln!` writes unbuffered, so the last line names how far a crashing start got. A launcher-started
/// game keeps the launcher's redirect.
#[cfg(windows)]
fn log_to_file_without_stderr() {
    use std::os::windows::io::IntoRawHandle;
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(which: u32) -> *mut core::ffi::c_void;
        fn SetStdHandle(which: u32, h: *mut core::ffi::c_void) -> i32;
    }
    // SAFETY: plain Win32 calls; the handle handed to SetStdHandle is an owned file handle we leak on purpose
    // (it must live for the whole process).
    unsafe {
        let h = GetStdHandle(STD_ERROR_HANDLE);
        if !h.is_null() && h as isize != -1 { return; }
        let Some(p) = nf_view::settings::config_dir().map(|d| d.join("nf-view.log")) else { return };
        let _ = std::fs::create_dir_all(p.parent().unwrap_or(&p));
        let Ok(f) = std::fs::File::create(&p) else { return };
        SetStdHandle(STD_ERROR_HANDLE, f.into_raw_handle() as *mut _);
    }
    eprintln!("NightfirePort {} started without a launcher log; logging here", nf_view::bugreport::version_line());
}

fn main() {
    #[cfg(windows)]
    restrict_dll_search();
    let game = std::env::args().any(|a| a == "--game");
    #[cfg(windows)]
    if game { log_to_file_without_stderr(); }
    let result = if game { nf_view::run() } else { nf_view::launcher::run() };
    if let Err(e) = result {
        let msg = format!("{e:#}");
        eprintln!("NightfirePort: {msg}");
        nf_view::launcher::platform::fatal(&msg);
        std::process::exit(1);
    }
}
