//! mp-drops: the two-window online check, scripted. Two real `nf-view` processes on this PC (a
//! host with `--net-host`, a joiner with `--net-join 127.0.0.1`), LAN-only (NF_NO_REACH, no cloud
//! listing), silent. The host shoots the joiner (NF_MP_DUEL / NF_MP_FACE_OTHER / NF_NET_AUTOFIRE);
//! both windows log the death drop (NF_NET_DROPLOG): created on the same match tick with the dead
//! player's clip, landed, spinning, and gone exactly `FRAME_RATE_INT · 30` = 1800 ticks later on
//! both machines (`Pickup_CreateSimple` 0x18f348's MP lifetime [M disasm 0x18f440-0x18f47c]).
//!
//! Opt-in: opens two game windows for about 80 s. Run with NF_TWO_INSTANCE=1 and NF_ISO set:
//! `cargo test -p nf-view --test mp_two_instance_live -- --ignored --nocapture`.
//! The script twin with more knobs: audits/tools-wip/mp_two_instance.py.

use std::process::{Command, Stdio};

fn drop_lines(log: &str) -> Vec<(u32, String)> {
    log.lines()
        .filter(|l| l.starts_with("[droplog] t ") && l.contains(" drop "))
        .filter_map(|l| Some((l.split_whitespace().nth(2)?.parse().ok()?, l.to_string())))
        .collect()
}

/// (creation tick, rounds, gone tick, the landed headings seen).
fn summary(log: &str) -> (u32, String, Option<u32>, Vec<String>) {
    let d = drop_lines(log);
    let first = d.iter().find(|(_, l)| l.contains("state 0 lifetime 1800")).expect("the death drop was created");
    let rounds = first.1.split("rounds ").nth(1).and_then(|r| r.split_whitespace().next()).unwrap_or("").to_string();
    let gone = d.iter().find(|(_, l)| l.contains("gone true")).map(|(t, _)| *t);
    let heads = d.iter().filter(|(_, l)| l.contains("state 1") && l.contains("gone false"))
        .filter_map(|(_, l)| l.split("heading ").nth(1).map(str::to_string)).collect();
    (first.0, rounds, gone, heads)
}

#[test]
#[ignore = "opens two game windows; NF_TWO_INSTANCE=1"]
fn a_death_drop_lasts_30_seconds_on_both_machines() {
    if std::env::var_os("NF_TWO_INSTANCE").is_none() { eprintln!("NF_TWO_INSTANCE not set — skipping"); return; }
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let dir = std::env::temp_dir().join(format!("nf-2inst-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // A private copy of the exe (as the script does): a rebuild cannot swap it under the windows.
    let exe = dir.join("nf-view.exe");
    std::fs::copy(env!("CARGO_BIN_EXE_nf-view"), &exe).expect("copies the exe");
    let port = "27433";
    let run = |role: &str, args: &[&str], extra: &[(&str, &str)]| {
        let d = dir.join(role);
        std::fs::create_dir_all(d.join("cfg")).unwrap();
        let mut c = Command::new(&exe);
        c.arg(&iso).args(args).current_dir(&d)
            .env("NF_SILENT", "1").env("NF_NULL_MIXER", "1").env("NF_NO_REACH", "1").env("NF_LOBBY_SERVICE", "")
            .env("NF_CONFIG_DIR", d.join("cfg"))
            .env("NF_MP_FACE_OTHER", "1").env("NF_MP_DUEL", "5").env("NF_MP_FACE_TURN", "0:0.1:-0.08")
            .env("NF_NET_DROPLOG", "300")
            .stdout(Stdio::null()).stderr(std::fs::File::create(d.join("out.txt")).unwrap());
        for (k, v) in extra { c.env(k, v); }
        c.spawn().expect("starts the game")
    };
    let mut host = run("host", &["--net-host", port, "0x07000023"], &[("NF_NET_AUTOFIRE", "120-500")]);
    std::thread::sleep(std::time::Duration::from_secs(4));
    let mut join = run("join", &["--net-join", &format!("127.0.0.1:{port}")], &[]);
    // Load (~15 s) + the kill (~4 s) + 30 s + slack.
    let t0 = std::time::Instant::now();
    let done = |p: &std::path::Path| std::fs::read_to_string(p).unwrap_or_default().contains("gone true");
    while t0.elapsed().as_secs() < 120 && !(done(&dir.join("host/out.txt")) && done(&dir.join("join/out.txt"))) {
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    let _ = host.kill();
    let _ = join.kill();
    let _ = host.wait();
    let _ = join.wait();
    let h = summary(&std::fs::read_to_string(dir.join("host/out.txt")).unwrap());
    let j = summary(&std::fs::read_to_string(dir.join("join/out.txt")).unwrap());
    eprintln!("host {h:?}\njoin {j:?}");
    assert_eq!((h.0, &h.1, h.2), (j.0, &j.1, j.2), "both machines hold the same drop on the same ticks");
    assert_eq!(h.2, Some(h.0 + 1800), "the drop is gone exactly 1800 ticks (30 s) after the death");
    assert!(h.3.windows(2).any(|w| w[0] != w[1]), "the landed drop turns");
    let _ = std::fs::remove_dir_all(&dir);
}
