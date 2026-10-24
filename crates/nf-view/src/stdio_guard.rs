//! A closed output pipe must not crash the game.
//!
//! `println!`/`eprintln!` panic when the write fails ("failed printing to stderr: The pipe is being
//! closed. (os error 232)"). That happens when the game's stdout/stderr is a pipe whose reader goes
//! away while the game keeps running — a script that started the game and ended, or a `timeout` that
//! killed the reader. The game's log lines are diagnostics only, so losing them is fine; crashing is not.
//!
//! On Windows, when stdout or stderr is a pipe, `install` points that standard handle at a pipe of our
//! own and starts a relay thread that copies everything to the original pipe. Once a write to the
//! original fails, the relay drops the rest and keeps draining, so the game's own writes always
//! succeed. (Rust's std looks up `GetStdHandle` on every write, so `SetStdHandle` redirects
//! `println!`/`eprintln!` from then on.) Consoles and files are left untouched: they don't close
//! under the game, and a console needs std's own wide-char path.

#[cfg(windows)]
mod win {
    use std::ffi::c_void;
    type Handle = *mut c_void;
    pub const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    pub const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    pub const FILE_TYPE_PIPE: u32 = 3;
    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetStdHandle(id: u32) -> Handle;
        pub fn SetStdHandle(id: u32, h: Handle) -> i32;
        pub fn GetFileType(h: Handle) -> u32;
        pub fn CreatePipe(read: *mut Handle, write: *mut Handle, attrs: *mut c_void, size: u32) -> i32;
        pub fn ReadFile(h: Handle, buf: *mut c_void, len: u32, read: *mut u32, ov: *mut c_void) -> i32;
        pub fn WriteFile(h: Handle, buf: *const c_void, len: u32, written: *mut u32, ov: *mut c_void) -> i32;
        pub fn CloseHandle(h: Handle) -> i32;
    }

    /// Relays standard handle `id` through our own pipe when it is a pipe. Returns whether it did.
    pub fn guard(id: u32) -> bool {
        // SAFETY: plain Win32 calls on handles this process owns; the relay thread only reads the
        // pipe it was given and writes to the original handle, which stays open for the process's
        // life (we never close a standard handle).
        unsafe {
            let orig = GetStdHandle(id);
            if orig.is_null() || orig as isize == -1 || GetFileType(orig) != FILE_TYPE_PIPE { return false; }
            let (mut r, mut w): (Handle, Handle) = (std::ptr::null_mut(), std::ptr::null_mut());
            if CreatePipe(&mut r, &mut w, std::ptr::null_mut(), 1 << 16) == 0 { return false; }
            if SetStdHandle(id, w) == 0 { CloseHandle(r); CloseHandle(w); return false; }
            let (r, orig) = (r as usize, orig as usize);
            let relay = std::thread::Builder::new().name("stdio-relay".into()).spawn(move || {
                let mut buf = [0u8; 4096];
                let mut alive = true;
                loop {
                    let mut n = 0u32;
                    if ReadFile(r as Handle, buf.as_mut_ptr().cast(), buf.len() as u32, &mut n, std::ptr::null_mut()) == 0 || n == 0 { break; }
                    if alive {
                        let mut done = 0u32;
                        if WriteFile(orig as Handle, buf.as_ptr().cast(), n, &mut done, std::ptr::null_mut()) == 0 { alive = false; }
                    }
                }
            });
            if relay.is_err() { SetStdHandle(id, orig as Handle); return false; }
            true
        }
    }
}

/// Makes the game's stdout/stderr survive their reader going away (see the module doc). Call once,
/// first thing in the game's `run`.
pub fn install() {
    #[cfg(windows)]
    {
        win::guard(win::STD_OUTPUT_HANDLE);
        win::guard(win::STD_ERROR_HANDLE);
    }
}
