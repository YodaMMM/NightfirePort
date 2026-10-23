//! level-1b-drake-fight probe: the script chain dump. Every kind-0xD9 ScriptPlayer placement
//! with its params, and every script in the loaded lib with its per-stream Event commands
//! (the channel writes: event 0x0D = set ch p0 ← p1; event 0x11 = conditional set). Assertion-
//! free diagnostic, run with --ignored --nocapture.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::nis::Cmd;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_0700001b_script_chain() {
    let iso = iso();
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    println!("=== SP placements (0xd9) ===");
    for (i, p) in lv.maps[mm].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xd9) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
        println!("SP #{i} pos=({:.2},{:.2},{:.2}){ps}", p.translation[0], p.translation[1], p.translation[2]);
    }
    println!("\n=== triggers ===");
    for (i, p) in lv.maps[mm].placements.iter().enumerate().filter(|(_, p)| matches!(p.flags & 0xff, 0x03 | 0x04 | 0x05 | 0x12 | 0x13)) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
        println!("TRIG #{i} kind={:#04x} pos=({:.2},{:.2},{:.2}){ps}", p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2]);
    }
    let scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_ref().unwrap();
    println!("\n=== SP players (live, with gate/trigger/mask) ===");
    for p in &sim.drones.scripts.players {
        println!("player #{} scripts [{:08x}, {:08x}] modes [{}, {}] gate {:#x} out {:#x} mask {:#x} stay {}",
            p.placement, p.scripts[0], p.scripts[1], p.modes[0], p.modes[1], p.gate, p.out, p.mask, p.stay);
    }
    println!("\n=== stream kinds for the fight scripts ===");
    for h in [0x0600_08f0u32, 0x0600_07fd, 0x0600_0a1d, 0x0600_0971, 0x0600_096e, 0x0600_0a20, 0x0600_091c, 0x0600_091a, 0x0600_09f2, 0x0600_08a3] {
        let Some(nis) = sim.drones.scripts.lib.get(&h) else { continue };
        println!("script {h:08x}: length {} camera {} loop_start {}", nis.length, nis.camera, nis.loop_start);
        for (si, st) in nis.streams.iter().enumerate() {
            let ops: Vec<String> = st.commands.iter().take(6).map(|b| format!("{b:02x}")).collect();
            println!("   stream {si} keys {}..{} cmd bytes[{:?}]", st.first_key, st.last_key, ops.join(" "));
        }
    }
    println!("\n=== scripts and their events ===");
    let mut hashes: Vec<_> = sim.drones.scripts.lib.keys().copied().collect();
    hashes.sort();
    for h in hashes {
        let nis = &sim.drones.scripts.lib[&h];
        for (si, s) in nis.streams.iter().enumerate() {
            let cmds = s.decode();
            let evs: Vec<_> = cmds.iter().filter_map(|c| match c {
                Cmd::Event { kind, params } => Some((*kind, params.clone(), 0u32)),
                Cmd::SubScript { hash, stream, mode } => Some((0xffu8, vec![*hash, *stream as u32, *mode as u32], 1u32)),
                _ => None,
            }).collect();
            if evs.is_empty() { continue; }
            let list = evs.iter().map(|(k, p, sub)| {
                let ps = p.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(",");
                if *sub == 1 { format!("SUB[{ps}]") } else { format!("ev{k:#04x}[{ps}]") }
            }).collect::<Vec<_>>().join(" ");
            println!("script {h:08x} stream {si} (keys {}..{}): {list}", s.first_key, s.last_key);
        }
    }
}
