//! `NightfirePort`: the one program players run. Double-clicked it is the launcher — pick the disc image once, then
//! PLAY (crates/nf-view/src/launcher/mod.rs); PLAY starts this same program as `NightfirePort <iso> --game`, which runs
//! the game (`nf_view::run`, the front end). A GUI-subsystem program on Windows, so neither mode opens a console (the
//! launcher sends the game's output to nf-view.log).
#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    let result = if std::env::args().any(|a| a == "--game") { nf_view::run() } else { nf_view::launcher::run() };
    if let Err(e) = result {
        let msg = format!("{e:#}");
        eprintln!("NightfirePort: {msg}");
        nf_view::launcher::platform::fatal(&msg);
        std::process::exit(1);
    }
}
