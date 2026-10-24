//! The game survives its output pipe closing (stdio_guard.rs).
//!
//! `eprintln!` panics when the write fails, and a pipe whose reader has gone away fails every write
//! ("failed printing to stderr: The pipe is being closed. (os error 232)"). The
//! test starts the real exe on a headless render with stdout and stderr piped, closes both read
//! ends at once, and lets the level load (which logs to stderr throughout). The run must still exit
//! cleanly and write its screenshot. Opt-in via `NF_ISO`.

use std::process::{Command, Stdio};

// The guard is Windows-only (players run the Windows build; stdio_guard.rs relays through a Win32 pipe).
#[cfg(windows)]
#[test]
fn a_closed_stderr_pipe_does_not_crash_the_game() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let out = std::env::temp_dir().join(format!("nf-stdio-pipe-{}.png", std::process::id()));
    let _ = std::fs::remove_file(&out);
    let mut child = Command::new(env!("CARGO_BIN_EXE_nf-view"))
        .arg(&iso).arg("07000001").arg("--frame").arg("2").arg("--shot").arg(&out)
        .env("NF_SILENT", "1")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().expect("start nf-view");
    // The readers go away before the game writes its load lines.
    drop(child.stdout.take());
    drop(child.stderr.take());
    let status = child.wait().expect("wait for nf-view");
    let wrote = out.exists();
    let _ = std::fs::remove_file(&out);
    // Sister: without the guard the first load line's write fails and the run panics (exit 101) before the shot.
    assert!(status.success(), "the game must not crash when its output pipe closes (exit {status:?})");
    assert!(wrote, "the run must reach its screenshot");
}
