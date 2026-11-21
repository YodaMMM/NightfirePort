//! A native fault (an access violation in a GPU driver, an overlay DLL, the audio stack) ends the process without
//! a Rust panic, so the panic hooks (lib.rs `run`, bugreport.rs) never run and the log just stops. On Windows,
//! [`install`] sets an unhandled-exception filter that writes the exception code, the faulting module and offset,
//! the access kind and target for an access violation, and the module+offset of each stack frame to
//! `<config dir>/nf-crash.log` and to the game log, then lets the fault carry on (the process still ends with
//! the same exit code). The filter allocates nothing: it formats into a stack buffer and writes with plain
//! Win32 calls. Elsewhere it is a no-op.

/// Installs the filter (call first thing in `main`). `mode` names the program's role in the log line.
pub fn install(mode: &'static str) {
    #[cfg(windows)]
    win::install(mode);
    #[cfg(not(windows))]
    let _ = mode;
}

/// Sets the filter again if something replaced it (a driver or overlay may install its own while the GPU
/// device or the audio stream opens); the replaced filter is chained to.
pub fn rearm() {
    #[cfg(windows)]
    win::rearm();
}

/// The game's own log file handle when stderr is not that file (the tee beside a pipe, stdio_guard.rs):
/// the filter writes there too, since the relay thread dies with the fault.
pub fn set_log_handle(h: usize) {
    #[cfg(windows)]
    win::LOG.store(h, std::sync::atomic::Ordering::SeqCst);
    #[cfg(not(windows))]
    let _ = h;
}

/// The crash file's path (`<config dir>/nf-crash.log`).
pub fn crash_log_path() -> Option<std::path::PathBuf> { crate::settings::config_dir().map(|d| d.join("nf-crash.log")) }

/// Formats an exception record the way the filter writes it (the testable half: no OS calls). `module` maps an
/// address to (module file name, offset).
pub fn describe(out: &mut impl core::fmt::Write, header: &str, code: u32, address: usize, info: &[usize], frames: &[usize],
    module: &mut dyn FnMut(usize, &mut dyn core::fmt::Write) -> bool) -> core::fmt::Result {
    writeln!(out, "{header}")?;
    write!(out, "native crash: exception {code:#010x} ({}) at {address:#x} ", code_name(code))?;
    if !module(address, out) { write!(out, "(no module)")?; }
    writeln!(out)?;
    // EXCEPTION_ACCESS_VIOLATION / EXCEPTION_IN_PAGE_ERROR: [0] = 0 read, 1 write, 8 execute (DEP); [1] = target.
    if (code == 0xC000_0005 || code == 0xC000_0006) && info.len() >= 2 {
        let kind = match info[0] { 0 => "read of", 1 => "write to", 8 => "execute at", _ => "access to" };
        writeln!(out, "  {kind} {:#x}", info[1])?;
    }
    for (i, &f) in frames.iter().enumerate() {
        write!(out, "  #{i:02} {f:#x} ")?;
        if !module(f, out) { write!(out, "?")?; }
        writeln!(out)?;
    }
    Ok(())
}

fn code_name(code: u32) -> &'static str {
    match code {
        0xC000_0005 => "access violation",
        0xC000_0006 => "in-page error",
        0xC000_001D => "illegal instruction",
        0xC000_0094 => "integer divide by zero",
        0xC000_00FD => "stack overflow",
        0xC000_0409 => "fail fast",
        0xC000_0374 => "heap corruption",
        0xC06D_007E => "delay-load module not found",
        0xC06D_007F => "delay-load procedure not found",
        0xE06D_7363 => "C++ exception",
        0x8000_0003 => "breakpoint",
        _ => "other",
    }
}

/// A fixed buffer `core::fmt::Write` fills without allocating (output past the end is cut).
pub struct StackBuf<const N: usize> { pub b: [u8; N], pub n: usize }
impl<const N: usize> StackBuf<N> {
    pub fn new() -> Self { StackBuf { b: [0; N], n: 0 } }
    pub fn as_bytes(&self) -> &[u8] { &self.b[..self.n] }
}
impl<const N: usize> Default for StackBuf<N> { fn default() -> Self { Self::new() } }
impl<const N: usize> core::fmt::Write for StackBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let take = s.len().min(N - self.n);
        self.b[self.n..self.n + take].copy_from_slice(&s.as_bytes()[..take]);
        self.n += take;
        Ok(())
    }
}

#[cfg(windows)]
mod win {
    use core::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
    use std::sync::OnceLock;

    #[repr(C)]
    struct ExceptionRecord { code: u32, flags: u32, record: *mut ExceptionRecord, address: *mut c_void, nparams: u32, info: [usize; 15] }
    #[repr(C)]
    struct ExceptionPointers { record: *mut ExceptionRecord, context: *mut c_void }
    type Filter = Option<unsafe extern "system" fn(*mut ExceptionPointers) -> i32>;

    const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 2;
    const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 4;
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    const FILE_TYPE_DISK: u32 = 1;
    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;

    #[link(name = "kernel32")]
    extern "system" {
        fn SetUnhandledExceptionFilter(f: Filter) -> Filter;
        fn GetModuleHandleExW(flags: u32, name: *const u16, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameW(module: *mut c_void, buf: *mut u16, len: u32) -> u32;
        fn CreateFileW(name: *const u16, access: u32, share: u32, sa: *mut c_void, disp: u32, flags: u32, template: *mut c_void) -> *mut c_void;
        fn WriteFile(h: *mut c_void, buf: *const c_void, len: u32, written: *mut u32, ov: *mut c_void) -> i32;
        fn FlushFileBuffers(h: *mut c_void) -> i32;
        fn CloseHandle(h: *mut c_void) -> i32;
        fn GetStdHandle(which: u32) -> *mut c_void;
        fn GetFileType(h: *mut c_void) -> u32;
        fn GetCurrentThreadId() -> u32;
        fn RtlCaptureStackBackTrace(skip: u32, n: u32, frames: *mut *mut c_void, hash: *mut u32) -> u16;
    }

    /// The crash file's NUL-terminated wide path and the header line, both built at install (the filter only reads them).
    static PATH: OnceLock<Vec<u16>> = OnceLock::new();
    static HEADER: OnceLock<String> = OnceLock::new();
    /// The filter that was in place before ours (chained to after writing).
    static PREV: AtomicUsize = AtomicUsize::new(0);
    /// See `super::set_log_handle`.
    pub static LOG: AtomicUsize = AtomicUsize::new(0);
    static BUSY: AtomicBool = AtomicBool::new(false);

    pub fn install(mode: &'static str) {
        if let Some(p) = super::crash_log_path() {
            use std::os::windows::ffi::OsStrExt;
            let _ = std::fs::create_dir_all(p.parent().unwrap_or(&p));
            let _ = PATH.set(p.as_os_str().encode_wide().chain([0]).collect());
        }
        let _ = HEADER.set(format!("NightfirePort {} ({mode})", crate::bugreport::version_line()));
        rearm();
    }

    pub fn rearm() {
        // SAFETY: swaps the process's top-level filter for ours; the previous one is kept as a plain address and
        // called only as the filter type it was registered as.
        let prev = unsafe { SetUnhandledExceptionFilter(Some(filter)) };
        if let Some(p) = prev {
            if p as usize != filter as *const () as usize { PREV.store(p as usize, SeqCst); }
        }
    }

    /// Writes the module file name holding `addr` and the offset into it.
    fn module_of(addr: usize, out: &mut dyn core::fmt::Write) -> bool {
        let mut m: *mut c_void = std::ptr::null_mut();
        // SAFETY: FROM_ADDRESS takes any address; UNCHANGED_REFCOUNT means nothing to release.
        if unsafe { GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, addr as *const u16, &mut m) } == 0 || m.is_null() {
            return false;
        }
        let mut name = [0u16; 520];
        // SAFETY: the buffer and its length match.
        let n = unsafe { GetModuleFileNameW(m, name.as_mut_ptr(), name.len() as u32) } as usize;
        let name = &name[..n.min(name.len())];
        // The file name only (the folder can carry the account name).
        let start = name.iter().rposition(|&c| c == b'\\' as u16 || c == b'/' as u16).map_or(0, |i| i + 1);
        for c in char::decode_utf16(name[start..].iter().copied()) {
            let _ = out.write_char(c.unwrap_or('?'));
        }
        let _ = write!(out, "+{:#x}", addr.wrapping_sub(m as usize));
        true
    }

    unsafe fn write_all(h: *mut c_void, b: &[u8]) {
        let mut done = 0u32;
        WriteFile(h, b.as_ptr().cast(), b.len() as u32, &mut done, std::ptr::null_mut());
        FlushFileBuffers(h);
    }

    unsafe extern "system" fn filter(ep: *mut ExceptionPointers) -> i32 {
        if !BUSY.swap(true, SeqCst) && !ep.is_null() && !(*ep).record.is_null() {
            let r = &*(*ep).record;
            let mut frames = [std::ptr::null_mut::<c_void>(); 48];
            let n = RtlCaptureStackBackTrace(0, frames.len() as u32, frames.as_mut_ptr(), std::ptr::null_mut()) as usize;
            let frames: [usize; 48] = core::array::from_fn(|i| frames[i] as usize);
            let mut buf = super::StackBuf::<16384>::new();
            let header = HEADER.get().map(String::as_str).unwrap_or("NightfirePort");
            let np = (r.nparams as usize).min(15);
            let _ = super::describe(&mut buf, header, r.code, r.address as usize, &r.info[..np], &frames[..n], &mut module_of);
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("  thread {}\n", GetCurrentThreadId()));
            if let Some(p) = PATH.get() {
                let h = CreateFileW(p.as_ptr(), 0x4000_0000, 1, std::ptr::null_mut(), 2, 0x80, std::ptr::null_mut());
                if !h.is_null() && h as isize != -1 { write_all(h, buf.as_bytes()); CloseHandle(h); }
            }
            // The game log: stderr when it is the file itself, else the tee handle beside a pipe.
            let e = GetStdHandle(STD_ERROR_HANDLE);
            if !e.is_null() && e as isize != -1 && GetFileType(e) == FILE_TYPE_DISK { write_all(e, buf.as_bytes()); }
            else if let l @ 1.. = LOG.load(SeqCst) { write_all(l as *mut c_void, buf.as_bytes()); }
        }
        match PREV.load(SeqCst) {
            0 => EXCEPTION_CONTINUE_SEARCH,
            p => {
                // SAFETY: `p` came from SetUnhandledExceptionFilter as a filter of this type.
                let f: unsafe extern "system" fn(*mut ExceptionPointers) -> i32 = std::mem::transmute(p);
                f(ep)
            }
        }
    }


    /// Test aid: runs the filter's formatting on a synthetic access violation at `addr` (no fault, no file).
    pub fn format_for_test(addr: usize, frames: &[usize]) -> String {
        let mut buf = super::StackBuf::<16384>::new();
        let _ = super::describe(&mut buf, "test", 0xC000_0005, addr, &[1, 0x10], frames, &mut module_of);
        String::from_utf8_lossy(buf.as_bytes()).into_owned()
    }
}

/// Test aid (Windows): the filter's text for a synthetic access violation at `addr`.
#[cfg(windows)]
pub fn format_for_test(addr: usize, frames: &[usize]) -> String { win::format_for_test(addr, frames) }

/// Test aid: `NF_TEST_NATIVE_CRASH=1` makes `main` raise an access violation right after the startup log lines
/// (RaiseException, nothing is touched), so a test can check what a real unhandled fault writes.
pub fn test_fault_if_requested() {
    if std::env::var_os("NF_TEST_NATIVE_CRASH").is_none() { return; }
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" { fn RaiseException(code: u32, flags: u32, n: u32, args: *const usize); }
        let args = [1usize, 0x10];
        // SAFETY: raises a non-continuable exception record we built; it ends the process through the filter.
        unsafe { RaiseException(0xC000_0005, 1, 2, args.as_ptr()) };
    }
}
