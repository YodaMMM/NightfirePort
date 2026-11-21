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

/// Where this run's output goes when nobody else keeps it. Our launcher starts the game with its output in
/// `<config dir>/nf-view.log` (a disk file) and a console keeps its own: both stay as they are. Started by another
/// front end, a GUI-subsystem program has either no stderr at all, or a pipe into that front end, where the
/// startup lines a crash report needs are lost to us. No stderr: open `<config dir>/<name>` and make it stderr.
/// A pipe (game only): keep the pipe and copy everything into the file as well (stdio_guard.rs `TEE`; the
/// native-crash filter writes there too). `eprintln!` writes unbuffered, so the last line names how far a
/// crashing start got.
#[cfg(windows)]
fn log_to_file_if_unkept(name: &str, tee_pipe: bool) {
    use std::os::windows::io::IntoRawHandle;
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    const FILE_TYPE_DISK: u32 = 1;
    const FILE_TYPE_CHAR: u32 = 2;
    const FILE_TYPE_PIPE: u32 = 3;
    #[link(name = "kernel32")]
    extern "system" {
        fn GetStdHandle(which: u32) -> *mut core::ffi::c_void;
        fn SetStdHandle(which: u32, h: *mut core::ffi::c_void) -> i32;
        fn GetFileType(h: *mut core::ffi::c_void) -> u32;
        fn GetConsoleMode(h: *mut core::ffi::c_void, mode: *mut u32) -> i32;
    }
    // SAFETY: plain Win32 calls; the handle handed to SetStdHandle / the tee is an owned file handle we leak on
    // purpose (it must live for the whole process).
    unsafe {
        let h = GetStdHandle(STD_ERROR_HANDLE);
        let valid = !h.is_null() && h as isize != -1;
        let kind = if valid { GetFileType(h) } else { 0 };
        let mut mode = 0u32;
        // A disk file (our launcher's redirect) or a real console: kept. The NUL device is a char file too, but
        // no console.
        if kind == FILE_TYPE_DISK || (kind == FILE_TYPE_CHAR && GetConsoleMode(h, &mut mode) != 0) { return; }
        let pipe = kind == FILE_TYPE_PIPE;
        if pipe && !tee_pipe { return; }
        let Some(p) = nf_view::settings::config_dir().map(|d| d.join(name)) else { return };
        let _ = std::fs::create_dir_all(p.parent().unwrap_or(&p));
        let Ok(f) = std::fs::File::create(&p) else { return };
        let raw = f.into_raw_handle() as usize;
        if pipe {
            nf_view::stdio_guard::TEE.store(raw, std::sync::atomic::Ordering::SeqCst);
            nf_view::native_crash::set_log_handle(raw);
            eprintln!("NightfirePort {}: output piped to another program; copying it to {name} as well", nf_view::bugreport::version_line());
        } else {
            SetStdHandle(STD_ERROR_HANDLE, raw as *mut _);
            eprintln!("NightfirePort {} started without a launcher log; logging here", nf_view::bugreport::version_line());
        }
    }
}

/// The first log line of a run: how it was started (another front end can pass odd arguments or folders).
fn start_line(game: bool) {
    let args: Vec<String> = std::env::args_os().skip(1).map(|a| a.to_string_lossy().into_owned()).collect();
    let cwd = std::env::current_dir().map(|d| d.display().to_string()).unwrap_or_else(|e| format!("(unreadable: {e})"));
    let exe = std::env::current_exe().map(|d| d.display().to_string()).unwrap_or_else(|e| format!("(unknown: {e})"));
    eprintln!("start: {} mode, exe {exe}, folder {cwd}, args {args:?}", if game { "game" } else { "launcher" });
}

fn main() {
    #[cfg(windows)]
    restrict_dll_search();
    // `args_os`: an argument that isn't valid Unicode must not panic before the log is open.
    let game = std::env::args_os().any(|a| a == "--game");
    // A native fault (a driver, an overlay, the audio stack) leaves no panic report: this writes its code and
    // module to nf-crash.log and the log (native_crash.rs).
    nf_view::native_crash::install(if game { "game" } else { "launcher" });
    // The game's pipe guard first, so a piped log is copied to the file from its first line on.
    if game { nf_view::stdio_guard::install(); }
    #[cfg(windows)]
    log_to_file_if_unkept(if game { "nf-view.log" } else { "launcher.log" }, game);
    start_line(game);
    nf_view::native_crash::test_fault_if_requested();
    let result = if game { nf_view::run() } else { nf_view::launcher::run() };
    if let Err(e) = result {
        let msg = format!("{e:#}");
        eprintln!("NightfirePort: {msg}");
        nf_view::launcher::platform::fatal(&msg);
        std::process::exit(1);
    }
}
