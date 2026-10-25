//! level-07-party-door-lock probe (moved out of the tests at level-07-party-door-lock-2 — a
//! dump, not a test): script 0x06000073 (the control-taking cut-scene) raw stream bytes — the
//! stream 8 that carries the level's only script channel write, event 0x0D [0x82, 0] = ch 130 ← 0
//! [M Script_EventHandler 0x1d94c0 case 0xd].
//!
//! Run with:
//!   NF_ISO=<iso> cargo run -p nf-view --release --example dump_06000073_stream8

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07");
    let sim = scene.sim.as_ref().unwrap();
    let nis = &sim.drones.scripts.lib[&0x0600_0073];
    println!("script 06000073: length {} camera {} loop_start {} streams {}", nis.length, nis.camera, nis.loop_start, nis.streams.len());
    for (si, s) in nis.streams.iter().enumerate() {
        let ops: Vec<String> = s.commands.iter().map(|b| format!("{b:02x}")).collect();
        println!("stream {si} keys {}..{} cmds [{}]", s.first_key, s.last_key, ops.join(" "));
    }
}
