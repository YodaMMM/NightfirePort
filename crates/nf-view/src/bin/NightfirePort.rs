//! `NightfirePort`: the launcher — pick the disc image once, then PLAY (crates/nf-view/src/launcher/mod.rs).
//! A GUI-subsystem program on Windows, so double-clicking it opens no console.
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    if let Err(e) = nf_view::launcher::run() {
        let msg = format!("{e:#}");
        eprintln!("NightfirePort: {msg}");
        nf_view::launcher::platform::fatal(&msg);
        std::process::exit(1);
    }
}
