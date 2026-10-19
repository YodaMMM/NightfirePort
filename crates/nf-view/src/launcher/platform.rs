//! OS glue: the open-file dialog, starting the game without a console, and noticing when its window is up.
//!
//! Windows uses the system's own common dialog (`GetOpenFileNameW`, comdlg32) and `EnumWindows` (user32) through
//! hand-written declarations: a few functions don't justify a new dependency. Elsewhere the dialog is `zenity` when
//! installed (drag-and-drop works everywhere) and the launcher closes a moment after the game starts.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The game is the launcher's own program run with `--game` (one `NightfirePort.exe`, the user's choice 2026-10-19:
/// "why are there 2 of them?"); `src/bin/NightfirePort.rs` hands `--game` to the game's `run`.
pub fn game_exe() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| format!("can't locate the launcher: {e}"))
}

/// The separate game exe the v0.1.0 zip shipped; the single-exe launcher deletes it on start.
pub const LEGACY_GAME_EXE: &str = "NightfirePort-game.exe";

/// Starts the game (`<exe> <iso> --game`) on `iso`. Its output goes to `log`; no window but the game's appears.
pub fn launch(exe: &Path, iso: &Path, log: Option<&Path>) -> Result<Child, String> {
    let mut cmd = Command::new(exe);
    cmd.arg(iso).arg("--game").stdin(Stdio::null());
    if let Some(dir) = exe.parent() { cmd.current_dir(dir); }
    match log.and_then(|p| { if let Some(d) = p.parent() { let _ = std::fs::create_dir_all(d); } std::fs::File::create(p).ok() }) {
        Some(f) => {
            let f2 = f.try_clone().map_err(|e| e.to_string())?;
            cmd.stdout(Stdio::from(f)).stderr(Stdio::from(f2));
        }
        None => { cmd.stdout(Stdio::null()).stderr(Stdio::null()); }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().map_err(|e| format!("The game couldn't start: {e}"))
}

/// The last non-empty line of the game's log (shown when it exits right away).
pub fn log_tail(log: &Path) -> Option<String> {
    let t = std::fs::read_to_string(log).ok()?;
    t.lines().rev().map(str::trim).find(|l| !l.is_empty()).map(|l| l.chars().take(140).collect())
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    type Hwnd = *mut c_void;

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct OpenFileNameW {
        pub lStructSize: u32,
        pub hwndOwner: Hwnd,
        pub hInstance: *mut c_void,
        pub lpstrFilter: *const u16,
        pub lpstrCustomFilter: *mut u16,
        pub nMaxCustFilter: u32,
        pub nFilterIndex: u32,
        pub lpstrFile: *mut u16,
        pub nMaxFile: u32,
        pub lpstrFileTitle: *mut u16,
        pub nMaxFileTitle: u32,
        pub lpstrInitialDir: *const u16,
        pub lpstrTitle: *const u16,
        pub Flags: u32,
        pub nFileOffset: u16,
        pub nFileExtension: u16,
        pub lpstrDefExt: *const u16,
        pub lCustData: isize,
        pub lpfnHook: *mut c_void,
        pub lpTemplateName: *const u16,
        pub pvReserved: *mut c_void,
        pub dwReserved: u32,
        pub FlagsEx: u32,
    }

    #[link(name = "comdlg32")]
    extern "system" {
        pub fn GetOpenFileNameW(ofn: *mut OpenFileNameW) -> i32;
    }
    #[link(name = "user32")]
    extern "system" {
        pub fn EnumWindows(cb: extern "system" fn(Hwnd, isize) -> i32, lparam: isize) -> i32;
        pub fn GetWindowThreadProcessId(hwnd: Hwnd, pid: *mut u32) -> u32;
        pub fn IsWindowVisible(hwnd: Hwnd) -> i32;
        pub fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
        pub fn GetClientRect(hwnd: Hwnd, rect: *mut [i32; 4]) -> i32;
        pub fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, kind: u32) -> i32;
    }

    pub const OFN_HIDEREADONLY: u32 = 0x4;
    pub const OFN_NOCHANGEDIR: u32 = 0x8;
    pub const OFN_PATHMUSTEXIST: u32 = 0x800;
    pub const OFN_FILEMUSTEXIST: u32 = 0x1000;
    pub const OFN_EXPLORER: u32 = 0x80000;

    pub fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

    struct Find { pid: u32, found: bool }
    extern "system" fn enum_cb(hwnd: Hwnd, lp: isize) -> i32 {
        // SAFETY: lp is the &mut Find passed to EnumWindows below, alive for the call.
        let f = unsafe { &mut *(lp as *mut Find) };
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        if pid != f.pid || unsafe { IsWindowVisible(hwnd) } == 0 { return 1; }
        // A real window: titled and with a client area (drivers leave visible 0×0 helper windows around).
        let mut r = [0i32; 4];
        let sized = unsafe { GetClientRect(hwnd, &mut r) } != 0 && r[2] > r[0] && r[3] > r[1];
        if sized && unsafe { GetWindowTextLengthW(hwnd) } > 0 { f.found = true; return 0; }
        1
    }
    pub fn has_visible_window(pid: u32) -> bool {
        let mut f = Find { pid, found: false };
        unsafe { EnumWindows(enum_cb, &mut f as *mut Find as isize) };
        f.found
    }
}

/// Asks for a disc image with the system's open dialog. `owner` is the launcher's HWND (Windows), `start` a folder
/// to open in. None = cancelled (or no dialog available).
pub fn pick_iso(owner: Option<*mut std::ffi::c_void>, start: Option<&Path>) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let filter: Vec<u16> = "PS2 disc image (*.iso, *.bin)\0*.iso;*.bin\0All files (*.*)\0*.*\0\0".encode_utf16().collect();
        let title = win::wide("Select your 007: Nightfire disc image");
        let init = start.map(|p| win::wide(&p.to_string_lossy()));
        let mut buf = vec![0u16; 4096];
        let mut ofn = win::OpenFileNameW {
            lStructSize: std::mem::size_of::<win::OpenFileNameW>() as u32,
            hwndOwner: owner.unwrap_or(std::ptr::null_mut()),
            hInstance: std::ptr::null_mut(),
            lpstrFilter: filter.as_ptr(),
            lpstrCustomFilter: std::ptr::null_mut(),
            nMaxCustFilter: 0,
            nFilterIndex: 1,
            lpstrFile: buf.as_mut_ptr(),
            nMaxFile: buf.len() as u32,
            lpstrFileTitle: std::ptr::null_mut(),
            nMaxFileTitle: 0,
            lpstrInitialDir: init.as_ref().map_or(std::ptr::null(), |w| w.as_ptr()),
            lpstrTitle: title.as_ptr(),
            Flags: win::OFN_EXPLORER | win::OFN_FILEMUSTEXIST | win::OFN_PATHMUSTEXIST | win::OFN_HIDEREADONLY | win::OFN_NOCHANGEDIR,
            nFileOffset: 0,
            nFileExtension: 0,
            lpstrDefExt: std::ptr::null(),
            lCustData: 0,
            lpfnHook: std::ptr::null_mut(),
            lpTemplateName: std::ptr::null(),
            pvReserved: std::ptr::null_mut(),
            dwReserved: 0,
            FlagsEx: 0,
        };
        // SAFETY: every pointer in `ofn` outlives the modal call; `buf` is NUL-initialised and its length is nMaxFile.
        let ok = unsafe { win::GetOpenFileNameW(&mut ofn) };
        if ok == 0 { return None; }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(PathBuf::from(std::ffi::OsString::from_wide(&buf[..n])))
    }
    #[cfg(not(windows))]
    {
        let _ = owner;
        let mut cmd = Command::new("zenity");
        cmd.args(["--file-selection", "--title=Select your 007: Nightfire disc image", "--file-filter=PS2 disc image | *.iso *.bin"]);
        if let Some(s) = start { cmd.arg(format!("--filename={}/", s.display())); }
        let out = cmd.output().ok()?;
        if !out.status.success() { return None; }
        let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!p.is_empty()).then(|| PathBuf::from(p))
    }
}

/// True once the process `pid` shows a window (Windows); elsewhere None (unknown).
pub fn game_window_up(pid: u32) -> Option<bool> {
    #[cfg(windows)]
    { Some(win::has_visible_window(pid)) }
    #[cfg(not(windows))]
    { let _ = pid; None }
}

/// Shows a fatal launcher error where a GUI user can see it (a message box on Windows; stderr elsewhere).
pub fn fatal(msg: &str) {
    #[cfg(windows)]
    {
        let (text, cap) = (win::wide(msg), win::wide("NightfirePort"));
        // MB_OK | MB_ICONERROR
        unsafe { win::MessageBoxW(std::ptr::null_mut(), text.as_ptr(), cap.as_ptr(), 0x10) };
    }
    #[cfg(not(windows))]
    let _ = msg;
}

#[cfg(test)]
mod tests {
    #[cfg(all(windows, target_pointer_width = "64"))]
    #[test]
    fn open_file_name_matches_the_win32_layout() {
        // sizeof(OPENFILENAMEW) on x64 (commdlg.h, _WIN32_WINNT >= 0x0500) — the dialog refuses a wrong lStructSize.
        assert_eq!(std::mem::size_of::<super::win::OpenFileNameW>(), 152);
    }

    #[test]
    fn log_tail_is_the_last_line() {
        let p = std::env::temp_dir().join(format!("nf-launcher-log-{}.txt", std::process::id()));
        std::fs::write(&p, "one\ntwo: the error\n\n  \n").unwrap();
        assert_eq!(super::log_tail(&p).as_deref(), Some("two: the error"));
        assert_ne!(super::log_tail(&p).as_deref(), Some("one"));
        let _ = std::fs::remove_file(&p);
    }
}
