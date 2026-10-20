//! The system clipboard for the online pages' join codes (net-p2p §4) [S port design]: copy the
//! host's code, paste a friend's. Windows only, through user32/kernel32 directly (no crate); other
//! platforms report "unavailable" and the pages fall back to their own internal clipboard.

#[cfg(windows)]
mod imp {
    use std::ffi::c_void;
    use std::ptr::null_mut;

    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hwnd: *mut c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn GetClipboardData(format: u32) -> *mut c_void;
        fn SetClipboardData(format: u32, mem: *mut c_void) -> *mut c_void;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(flags: u32, bytes: usize) -> *mut c_void;
        fn GlobalLock(mem: *mut c_void) -> *mut c_void;
        fn GlobalUnlock(mem: *mut c_void) -> i32;
        fn GlobalFree(mem: *mut c_void) -> *mut c_void;
    }
    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 2;
    /// Longest text read back (a join code is 19 chars; anything this long is not one).
    const MAX_CHARS: usize = 4096;

    pub fn get() -> Option<String> {
        // SAFETY: the standard open/get/lock/unlock/close sequence; the locked buffer is read only
        // up to its NUL (capped), and every path closes the clipboard.
        unsafe {
            if OpenClipboard(null_mut()) == 0 {
                return None;
            }
            let h = GetClipboardData(CF_UNICODETEXT);
            let mut out = None;
            if !h.is_null() {
                let p = GlobalLock(h) as *const u16;
                if !p.is_null() {
                    let mut n = 0;
                    while n < MAX_CHARS && *p.add(n) != 0 {
                        n += 1;
                    }
                    out = Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, n)));
                    GlobalUnlock(h);
                }
            }
            CloseClipboard();
            out
        }
    }

    pub fn set(text: &str) -> bool {
        let w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: a moveable global block sized for the UTF-16 text + NUL, filled under its lock;
        // on success the clipboard owns it, on failure we free it; the clipboard is always closed.
        unsafe {
            if OpenClipboard(null_mut()) == 0 {
                return false;
            }
            EmptyClipboard();
            let h = GlobalAlloc(GMEM_MOVEABLE, w.len() * 2);
            if h.is_null() {
                CloseClipboard();
                return false;
            }
            let p = GlobalLock(h) as *mut u16;
            if p.is_null() {
                GlobalFree(h);
                CloseClipboard();
                return false;
            }
            std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
            GlobalUnlock(h);
            let ok = !SetClipboardData(CF_UNICODETEXT, h).is_null();
            if !ok {
                GlobalFree(h);
            }
            CloseClipboard();
            ok
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn get() -> Option<String> {
        None
    }
    pub fn set(_text: &str) -> bool {
        false
    }
}

/// The clipboard's text, if any (None: empty, not text, or unavailable).
pub fn get() -> Option<String> {
    imp::get()
}

/// Put `text` on the clipboard; false when it could not.
pub fn set(text: &str) -> bool {
    imp::set(text)
}
