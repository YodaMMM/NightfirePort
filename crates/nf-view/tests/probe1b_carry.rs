//! level-1b-drake-4 probe: the player-carry sweep. Decodes EVERY script in the 0700001b lib and
//! dumps (a) every event-4 (Player_Disable [M Script_EventHandler 0x1d94c0:118-121]) and event-6
//! (the enable-matrix capture [M :125-134]) site with its script/stream/tick, (b) the SUB-script
//! chains, (c) the iris-door placements (kind 0x2031, plc #30/#32) with their full params and any
//! script whose Entity hash matches them, (d) the ch-74/75/30/46 writers with ticks. Assertion-free
//! diagnostic, run with --ignored --nocapture.

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

/// Walk a stream's commands with running tick (End/Wait add), printing the interesting ones.
fn dump_stream(name: &str, si: usize, st: &nf_formats::nis::Stream, interesting: &dyn Fn(&Cmd) -> bool) {
    let mut tick = 0u32;
    let mut hits = Vec::new();
    let subs: Vec<String> = Vec::new();
    for cmd in st.decode() {
        match &cmd {
            Cmd::End(t) => tick += *t as u32,
            Cmd::Wait(t) => tick += *t as u32,
            Cmd::Event { kind, params } => {
                if interesting(&cmd) {
                    hits.push(format!("    t{tick} event {kind:#04x} params {:?} [{} s{si}]", params, name));
                }
            }
            Cmd::SubScript { hash, stream, mode } => {
                hits.push(format!("    t{tick} SUB {:08x} s{} mode {} [{} s{si}]", hash, stream, mode, name));
            }
            _ => {}
        }
    }
    for h in hits { println!("{}", h); }
    let _ = subs;
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn sweep_0700001b_player_carry() {
    let iso = iso();
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");

    // (c) the iris doors and any 0x2031-family placements with full params.
    println!("=== iris-door family placements (kind 0x31/0x2031/0x20/0x21) ===");
    if let Some(mm) = lv.main_map() {
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let k = p.flags & 0xff;
            if matches!(k, 0x20 | 0x21 | 0x30 | 0x31 | 0x32) || (p.flags & 0xffff) == 0x2031 {
                let mut ps = String::new();
                for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
                println!("  plc #{i} kind {:#04x} flags {:#06x} pos ({:.2},{:.2},{:.2}){ps}",
                    k, p.flags, p.translation[0], p.translation[1], p.translation[2]);
            }
        }
    }

    // (a)+(b) the whole lib: event 4 / event 6 / ch 74/75/30/46 writers + SUB chains.
    println!("\n=== the whole-lib event sweep (event 4/6, SUB chains, ch 74/75/30/46/10/0x0a writers) ===");
    let mut hashes: Vec<u32> = Vec::new();
    // The script lib is reachable through the loaded scene (the SPs' scripts + the subs); sweep
    // every hash the level's SPs name, then follow SUBs one level deep.
    let mmidx = lv.main_map().unwrap();
    for p in lv.maps[mmidx].placements.iter().filter(|p| p.flags & 0xff == 0xd9) {
        for (_, h) in p.params.iter().take(2) { if (*h >> 16) == 0x0600 { hashes.push(*h); } }
    }
    hashes.sort_unstable(); hashes.dedup();
    let mut seen: std::collections::BTreeSet<u32> = Default::default();
    let mut queue = hashes.clone();
    let interesting = |c: &Cmd| matches!(c, Cmd::Event { kind: 4 | 6, .. });

    // The scene's parsed lib (nf_view::load) has every script the SPs + SUBs reach.
    let scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_ref().unwrap();
    let lib = &sim.drones.scripts.lib;
    let mut all: Vec<u32> = lib.keys().copied().collect();
    all.extend(queue.drain(..));
    all.sort_unstable(); all.dedup();
    for h in &all {
        let Some(nis) = lib.get(h) else { continue };
        if !seen.insert(*h) { continue; }
        for (si, st) in nis.streams.iter().enumerate() {
            // Event writers with the channel in p0: 0xd (set ch p0), 0x11 (cond set), 0x9 (Cam2Mode).
            let mut tick = 0u32;
            for cmd in st.decode() {
                match &cmd {
                    Cmd::End(t) | Cmd::Wait(t) => tick += *t as u32,
                    Cmd::Event { kind, params } => {
                        let ch = params.first().copied().unwrap_or(0);
                        if matches!(kind, 4 | 6) || ch == 74 || ch == 75 || ch == 30 || ch == 46 || ch == 10 || ch == 0x0a {
                            println!("  {:08x} s{si} t{tick} event {kind:#04x} params {:?}", h, params);
                        }
                    }
                    Cmd::SubScript { hash, stream, mode } => {
                        println!("  {:08x} s{si} t{tick} SUB {:08x} s{} mode {}", h, hash, stream, mode);
                    }
                    _ => {}
                }
            }
        }
    }
    // The interesting-event one-liner recap (event 4/6 across the whole lib).
    println!("\n=== recap: every event-4 / event-6 site ===");
    for h in &all {
        let Some(nis) = lib.get(h) else { continue };
        for (si, st) in nis.streams.iter().enumerate() {
            dump_stream(&format!("{h:08x}"), si, st, &interesting);
        }
    }

    // (e) the carry scripts: which SPs name 0x0600085c/0x0600085f, their gates/outs/modes, and
    // the two scripts' FULL stream dumps (Entity creates with hashes/params, End/Wait keys).
    println!("\n=== the carry scripts 0600085c / 0600085f: owners + full streams ===");
    let mmidx2 = lv.main_map().unwrap();
    for (i, p) in lv.maps[mmidx2].placements.iter().enumerate() {
        if p.flags & 0xff != 0xd9 { continue; }
        let owns = p.params.iter().take(2).any(|(_, h)| *h == 0x0600_085c || *h == 0x0600_085f);
        if !owns { continue; }
        let mut ps = String::new();
        for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
        println!("SP #{i} pos=({:.2},{:.2},{:.2}){ps}", p.translation[0], p.translation[1], p.translation[2]);
    }
    for (i, p) in sim.drones.scripts.players.iter().enumerate() {
        if p.scripts.contains(&0x0600_085c) || p.scripts.contains(&0x0600_085f) {
            println!("live player #{} placement {} scripts [{:08x}, {:08x}] modes [{}, {}] gate {:#x} out {:#x} mask {:#x} stay {}",
                i, p.placement, p.scripts[0], p.scripts[1], p.modes[0], p.modes[1], p.gate, p.out, p.mask, p.stay);
        }
    }
    for h in [0x0600_085cu32, 0x0600_085f] {
        let Some(nis) = lib.get(&h) else { println!("script {h:08x}: NOT IN LIB"); continue };
        println!("script {h:08x}: length {} camera {} loop_start {} streams {}", nis.length, nis.camera, nis.loop_start, nis.streams.len());
        for (si, st) in nis.streams.iter().enumerate() {
            println!("  s{si}: keys {}..{} cmds {}", st.first_key, st.last_key, st.commands.len());
            let mut tick = 0u32;
            for cmd in st.decode() {
                match &cmd {
                    Cmd::End(t) | Cmd::Wait(t) => tick += *t as u32,
                    Cmd::Entity { hash, parent, spline, obj_type, params } => {
                        println!("    t{tick} ENTITY hash {:08x} parent {:?} spline {} type {} params {:?}", hash, parent, spline, obj_type, params);
                    }
                    Cmd::Anim { script, skin, start } => println!("    t{tick} ANIM {:08x} skin {:08x} start {}", script, skin, start),
                    Cmd::Event { kind, params } => println!("    t{tick} EVENT {kind:#04x} {:?}", params),
                    Cmd::Sound { id, .. } => println!("    t{tick} SOUND {:08x}", id),
                    Cmd::Camera { spline, first, last } => println!("    t{tick} CAMERA spline {} {}..{}", spline, first, last),
                    Cmd::SubScript { hash, stream, mode } => println!("    t{tick} SUB {:08x} s{} mode {}", hash, stream, mode),
                    _ => {}
                }
            }
        }
    }

    // (f) the fly-by chain scripts in full: 060007fd, 060008f0, 06000a1d — every command.
    println!("\n=== the fly-by chain in full ===");
    for h in [0x0600_07fdu32, 0x0600_08f0, 0x0600_0a1d] {
        let Some(nis) = lib.get(&h) else { println!("script {h:08x}: NOT IN LIB"); continue };
        println!("script {h:08x}: length {} camera {} loop_start {} streams {}", nis.length, nis.camera, nis.loop_start, nis.streams.len());
        for (si, st) in nis.streams.iter().enumerate() {
            let mut tick = 0u32;
            let mut lines = Vec::new();
            for cmd in st.decode() {
                match &cmd {
                    Cmd::End(t) | Cmd::Wait(t) => tick += *t as u32,
                    Cmd::Entity { hash, parent, spline, obj_type, params } => lines.push(format!("    t{tick} ENTITY {:08x} parent {:?} spline {} type {} params {:?}", hash, parent, spline, obj_type, params)),
                    Cmd::Anim { script, skin, start } => lines.push(format!("    t{tick} ANIM {:08x} skin {:08x} start {}", script, skin, start)),
                    Cmd::Event { kind, params } => lines.push(format!("    t{tick} EVENT {kind:#04x} {:?}", params)),
                    Cmd::Camera { spline, first, last } => lines.push(format!("    t{tick} CAMERA spline {} {}..{}", spline, first, last)),
                    Cmd::SubScript { hash, stream, mode } => lines.push(format!("    t{tick} SUB {:08x} s{} mode {}", hash, stream, mode)),
                    _ => {}
                }
            }
            if !lines.is_empty() {
                println!("  s{si} ({} cmds):", st.commands.len());
                for l in lines { println!("{}", l); }
            }
        }
    }

    // (g) SP#141/#142 + every SP's gate/out/trigger, and 06000818 in full.
    println!("\n=== every SP live player: gate/out/mask (ch-74 gate hunt) ===");
    for p in &sim.drones.scripts.players {
        println!("  placement {} scripts [{:08x}, {:08x}] gate {:#x} out {:#x} mask {:#x} stay {}",
            p.placement, p.scripts[0], p.scripts[1], p.gate, p.out, p.mask, p.stay);
    }
    println!("\n=== SP#141/#142 placements + script 06000818 in full ===");
    let mmidx3 = lv.main_map().unwrap();
    for (i, p) in lv.maps[mmidx3].placements.iter().enumerate() {
        if matches!(i, 141 | 142) || p.flags & 0xff == 0xd9 && p.translation[0].abs() < 7.0 && (p.translation[2].abs() - 8.82).abs() < 1.5 {
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("  plc #{i} kind {:#04x} pos ({:.2},{:.2},{:.2}){ps}", p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2]);
        }
    }
    for h in [0x0600_0818u32] {
        let Some(nis) = lib.get(&h) else { println!("script {h:08x}: NOT IN LIB"); continue };
        println!("script {h:08x}: length {} camera {} loop {} streams {}", nis.length, nis.camera, nis.loop_start, nis.streams.len());
        for (si, st) in nis.streams.iter().enumerate() {
            let mut tick = 0u32;
            let mut lines = Vec::new();
            for cmd in st.decode() {
                match &cmd {
                    Cmd::End(t) | Cmd::Wait(t) => tick += *t as u32,
                    Cmd::Entity { hash, parent, spline, obj_type, params } => lines.push(format!("    t{tick} ENTITY {:08x} parent {:?} spline {} type {} params {:?}", hash, parent, spline, obj_type, params)),
                    Cmd::Anim { script, skin, start } => lines.push(format!("    t{tick} ANIM {:08x} skin {:08x} start {}", script, skin, start)),
                    Cmd::Event { kind, params } => lines.push(format!("    t{tick} EVENT {kind:#04x} {:?}", params)),
                    Cmd::SubScript { hash, stream, mode } => lines.push(format!("    t{tick} SUB {:08x} s{} mode {}", hash, stream, mode)),
                    _ => {}
                }
            }
            if !lines.is_empty() {
                println!("  s{si} ({} cmds):", st.commands.len());
                for l in lines { println!("{}", l); }
            }
        }
    }
}
