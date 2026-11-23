//! A native fault at start leaves a trace (native_crash.rs, bin/NightfirePort.rs).
//!
//! An access violation ends the process without a Rust panic, so before this only the exit code was left, and
//! a game started by another front end with its output piped to that front end (or sent to NUL) wrote no log
//! file at all. These start the real `NightfirePort.exe --game` with `NF_TEST_NATIVE_CRASH=1` (it raises an
//! access violation right after its start lines) and check the crash file and the game log. Opt-in via
//! `NF_ISO`: the arguments are a headless one-frame render, so a build without the fault aid exits by itself
//! (and fails the checks) instead of waiting on an error box.

#![cfg(windows)]

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// No Windows Error Reporting box for the faulting child (the error mode is inherited).
fn quiet_faults() {
    #[link(name = "kernel32")]
    extern "system" { fn SetErrorMode(mode: u32) -> u32; }
    // SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX | SEM_NOOPENFILEERRORBOX
    unsafe { SetErrorMode(0x0001 | 0x0002 | 0x8000) };
}

fn run(tag: &str, piped: bool) -> Option<(i32, PathBuf, String)> {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return None };
    quiet_faults();
    let cfg = std::env::temp_dir().join(format!("nf-native-crash-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cfg);
    std::fs::create_dir_all(&cfg).unwrap();
    let shot = cfg.join("shot.png");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_NightfirePort"));
    cmd.arg(&iso).arg("07000048").arg("--frame").arg("2").arg("--shot").arg(&shot).arg("--game")
        .env("NF_SILENT", "1").env("NF_CONFIG_DIR", &cfg).env("NF_TEST_NATIVE_CRASH", "1")
        .stdin(Stdio::null());
    let (code, piped_err) = if piped {
        let o = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output().expect("start NightfirePort");
        (o.status.code().unwrap_or(0), String::from_utf8_lossy(&o.stderr).into_owned())
    } else {
        let s = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status().expect("start NightfirePort");
        (s.code().unwrap_or(0), String::new())
    };
    Some((code, cfg, piped_err))
}

fn read(p: PathBuf) -> String { std::fs::read_to_string(&p).unwrap_or_default() }

#[test]
fn an_access_violation_at_start_writes_the_crash_file_and_the_log() {
    // Output sent to NUL (a char device, not a console): the game keeps its own log file.
    let Some((code, cfg, _)) = run("nul", false) else { return };
    let crash = read(cfg.join("nf-crash.log"));
    let log = read(cfg.join("nf-view.log"));
    let _ = std::fs::remove_dir_all(&cfg);
    // The process still ends with the fault's own code (the filter lets it carry on).
    assert_eq!(code as u32, 0xC000_0005, "exit code; crash file:\n{crash}");
    assert!(crash.contains("native crash: exception 0xc0000005 (access violation)"), "crash file:\n{crash}");
    assert!(crash.contains("write to 0x10"), "the access kind and target: {crash}");
    // The stack names our own frames by module + offset.
    assert!(crash.contains("NightfirePort.exe+0x"), "stack frames: {crash}");
    assert!(log.contains("start: game mode"), "the log's start line:\n{log}");
    assert!(log.contains("native crash: exception 0xc0000005"), "the crash in the log:\n{log}");
}

#[test]
fn a_piped_start_is_copied_to_the_log_file() {
    // Output piped to another program: the pipe keeps getting it, and the log file holds it too.
    let Some((code, cfg, piped)) = run("pipe", true) else { return };
    let log = read(cfg.join("nf-view.log"));
    let _ = std::fs::remove_dir_all(&cfg);
    assert_eq!(code as u32, 0xC000_0005);
    assert!(piped.contains("start: game mode"), "the pipe still gets the log:\n{piped}");
    assert!(log.contains("start: game mode"), "the copy in the log file:\n{log}");
    assert!(log.contains("native crash: exception 0xc0000005"), "the crash in the log file:\n{log}");
}

/// The filter's text without a fault: a code address inside this test exe maps to its module.
#[test]
fn the_crash_text_names_the_module_and_offset() {
    let here = the_crash_text_names_the_module_and_offset as *const () as usize;
    let t = nf_view::native_crash::format_for_test(here, &[here, 0x10]);
    assert!(t.contains("native crash: exception 0xc0000005 (access violation)"), "{t}");
    assert!(t.contains("write to 0x10"), "{t}");
    let exe = std::env::current_exe().unwrap();
    let name = exe.file_name().unwrap().to_string_lossy().to_string();
    assert!(t.contains(&format!("{name}+0x")), "module name {name} in:\n{t}");
    // An address no module holds.
    assert!(t.contains("#01 0x10 ?"), "{t}");
}
