//! level-07-party-door-lock census: the party level's doors (kind 0xDB with every param), the
//! triggers/SPs/exits, and every script stream's channel-write events (event 0x0D = set
//! ch p0 ← p1 [M Script_EventHandler 0x1d94c0 case 0xd]; event 0x11 = conditional set
//! ch (p0 on ? p1 : p2) ← p3 [M case 0x11]). Assertion-free diagnostic, run with
//! `cargo test -p nf-view --test census_07000007 -- --ignored --nocapture`.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

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
#[ignore = "assertion-free census dump; run with --ignored --nocapture"]
fn dump_07000007_doors_and_channel_writers() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000007.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    let placements = &lv.maps[mm].placements;
    println!("main-map placements: {}", placements.len());

    let name_of = |p: &nf_formats::placement::Placement| -> String {
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        target.map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
            .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode))
    };

    println!("\n=== DOORS (kind 0xdb) ===");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xdb) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v}")); }
        println!("DOOR #{i} \"{}\" @({:.2},{:.2},{:.2}){ps} flags {:#x} hash {:08x}",
            name_of(p), p.translation[0], p.translation[1], p.translation[2], p.flags, p.hashcode);
    }

    println!("\n=== TRIGGERS / EXITS / MUXES / SWITCHES ===");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| matches!(p.flags & 0xff, 0x03 | 0x04 | 0x05 | 0x12 | 0x13 | 0x29 | 0xe8 | 0xea | 0xeb | 0xec | 0xef | 0xe2)) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
        println!("#{i} kind {:#04x} \"{}\" @({:.2},{:.2},{:.2}){ps}",
            p.flags & 0xff, name_of(p), p.translation[0], p.translation[1], p.translation[2]);
    }

    println!("\n=== SCRIPT PLAYERS (kind 0xd9) ===");
    for (i, p) in placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xd9) {
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
        println!("SP #{i} \"{}\" @({:.2},{:.2},{:.2}){ps}",
            name_of(p), p.translation[0], p.translation[1], p.translation[2]);
    }

    // The live sim's script lib (the auto-loaded scripts; the SP_LoadScript ones stream later).
    let scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07");
    let sim = scene.sim.as_ref().unwrap();
    println!("\n=== SP players (live) ===");
    for p in &sim.drones.scripts.players {
        println!("player #{} scripts [{:08x}, {:08x}] modes [{}, {}] gate {:#x} out {:#x} mask {:#x} stay {}",
            p.placement, p.scripts[0], p.scripts[1], p.modes[0], p.modes[1], p.gate, p.out, p.mask, p.stay);
    }

    println!("\n=== script-lib channel-write events (0x0d set / 0x11 conditional) ===");
    let mut hashes: Vec<_> = sim.drones.scripts.lib.keys().copied().collect();
    hashes.sort();
    for h in hashes {
        let nis = &sim.drones.scripts.lib[&h];
        for (si, s) in nis.streams.iter().enumerate() {
            let cmds = s.decode();
            let evs: Vec<_> = cmds.iter().filter_map(|c| match c {
                Cmd::Event { kind, params } if *kind == 0xd || *kind == 0x11 =>
                    Some((*kind, params.clone())),
                Cmd::SubScript { hash, stream, mode } => Some((0xffu8, vec![*hash, *stream as u32, *mode as u32])),
                _ => None,
            }).collect();
            if evs.is_empty() { continue; }
            let list = evs.iter().map(|(k, p)| {
                let ps = p.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(",");
                if *k == 0xff { format!("SUB[{ps}]") } else { format!("ev{k:#04x}[{ps}]") }
            }).collect::<Vec<_>>().join(" ");
            println!("script {h:08x} stream {si} (keys {}..{}): {list}", s.first_key, s.last_key);
        }
    }

    // The pack's streamed scripts are already in the lib (nf_view::load decodes the level's type-0xC
    // records → 07800007.bin/07900007.bin kind-7 entries into level.nis — lib.rs's loadable-pack pass).
    println!("\n=== script lib size: {} scripts ===", sim.drones.scripts.lib.len());
}

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn dump_06000073_stream8() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07");
    let sim = scene.sim.as_ref().unwrap();
    let nis = &sim.drones.scripts.lib[&0x0600_0073];
    println!("script 06000073: length {} camera {} loop_start {} streams {}", nis.length, nis.camera, nis.loop_start, nis.streams.len());
    for (si, s) in nis.streams.iter().enumerate() {
        let ops: Vec<String> = s.commands.iter().map(|b| format!("{b:02x}")).collect();
        println!("stream {si} keys {}..{} cmds [{}]", s.first_key, s.last_key, ops.join(" "));
    }
}
