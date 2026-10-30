//! level-11-every-objective-2: the ch 0x48 writer hunt on 07000011 — every switch_channels
//! writer class in one sweep: the NIS streams' Script_EventHandler 0xD/0x11 events [M
//! 0x1d94c0:162-187], the parsed triggers/SS/doors/monitors, and the raw placement params.
//! Diagnostic only — run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_0x48_writer_07000011() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000011.bin", false).expect("load 07000011");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 1. the scripts' channel-write events (Script_EventHandler 0xD/0x11 [M 0x1d94c0]) ===");
    let mut hits = 0usize;
    for (hash, nis) in sim.drones.scripts.lib.iter() {
        for (si, stream) in nis.streams.iter().enumerate() {
            for cmd in stream.decode() {
                if let nf_formats::nis::Cmd::Event { kind, params } = cmd {
                    if kind == 0xd || kind == 0x11 {
                        let is48 = params.iter().any(|&p| p == 0x48);
                        if is48 { hits += 1; }
                        eprintln!("script {hash:08x} stream {si}: event {kind:#x} params {params:?}{}",
                            if is48 { "   <== writes 0x48!" } else { "" });
                    }
                }
            }
        }
    }
    eprintln!("0x48-writing stream events: {hits}");
    eprintln!("=== 2. the SP placements (0xd9) and their scripts ===");
    {
        use nf_formats::{disc, iso9660::IsoImage, level::Level};
        let img = IsoImage::open(&std::env::var("NF_ISO").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let m = level.main_map().unwrap();
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            if p.flags & 0xff == 0xd9 {
                let par = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
                eprintln!("SP plc[{pi}] at=({:.2},{:.2},{:.2}) script={:08x} param0={} param2={} param3={}",
                    p.translation[0], p.translation[1], p.translation[2], par(0), par(0), par(2), par(3));
            }
        }
        eprintln!("=== 3. the raw placement param/extra sweep for 0x48 (72) ===");
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            let param_hits: Vec<u32> = p.params.iter().filter(|&(_, v)| *v == 0x48).map(|&(i, _)| i).collect();
            let extra_hits: Vec<usize> = p.extra.iter().enumerate().filter(|&(_, v)| *v == 0x48).map(|(i, _)| i).collect();
            if !param_hits.is_empty() || !extra_hits.is_empty() {
                eprintln!("plc[{pi}] flags={:#04x} at=({:.2},{:.2},{:.2}) PARAM hits idx {param_hits:?} EXTRA hits {extra_hits:?}",
                    p.flags, p.translation[0], p.translation[1], p.translation[2]);
            }
        }
        eprintln!("raw sweep done");
    }
    eprintln!("=== 4. the sim's parsed channel objects with 0x48 ===");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        if t.out == 0x48 || t.list.contains(&0x48) {
            let c = t.centre();
            eprintln!("trig[{i}] placement={} kind={:#x} out={:#04x} gate={} key={} mask={:#04x} list={:?} at=({:.2},{:.2},{:.2})",
                t.placement, t.kind, t.out, t.gate, t.key_gate, t.mask, t.list, c[0], c[1], c[2]);
        }
    }
    for (i, s) in sim.drones.sss.list.iter().enumerate() {
        if s.channel == 0x48 {
            eprintln!("ss[{i}] placement={} ch={:#04x} state={:#x} at=({:.2},{:.2},{:.2})",
                s.placement, s.channel, s.state, s.matrix[3][0], s.matrix[3][1], s.matrix[3][2]);
        }
    }
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if d.ch_open == 0x48 || d.ch_lock == 0x48 {
            eprintln!("door[{i}] placement={} ch_open={:#04x} ch_lock={:#04x}", d.placement, d.ch_open, d.ch_lock);
        }
    }
    for (i, m) in sim.drones.monitors.list.iter().enumerate() {
        eprintln!("monitor[{i}] at=({:.2},{:.2},{:.2})", m.pos[0], m.pos[1], m.pos[2]);
    }
    eprintln!("--- the drones' death channels (DroneFunc_SetDeathChannel 0x13e660 writes SC[+0x13c] = param 12) ---");
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.death_channel != 0 {
            eprintln!("drone[{i}] death_channel={:#04x}{}", d.death_channel,
                if d.death_channel == 0x48 { "   <== writes 0x48 on death!" } else { "" });
        }
    }
    eprintln!("=== sweep done ===");
}

// level-11-every-objective-3: the same sweep for 07000012 — the level the 0x48 writer (#679)
// lives on and the level the extended walk covers. Diagnostic only — run with --nocapture.

#[test]
fn probe_0x48_writer_07000012() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000012.bin", false).expect("load 07000012");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 12.1 the scripts' channel-write events (Script_EventHandler 0xD/0x11 [M 0x1d94c0]) ===");
    for (hash, nis) in sim.drones.scripts.lib.iter() {
        for (si, stream) in nis.streams.iter().enumerate() {
            for cmd in stream.decode() {
                if let nf_formats::nis::Cmd::Event { kind, params } = cmd {
                    if kind == 0xd || kind == 0x11 {
                        let is48 = params.iter().any(|&p| p == 0x48);
                        eprintln!("script {hash:08x} stream {si}: event {kind:#x} params {params:?}{}",
                            if is48 { "   <== writes 0x48!" } else { "" });
                    }
                }
            }
        }
    }
    eprintln!("=== 12.2 the parsed triggers with out/gate/list 0x48 ===");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        if t.out == 0x48 || t.list.contains(&0x48) {
            let c = t.centre();
            eprintln!("trig[{i}] placement={} kind={:#x} out={:#04x} gate={} key={} mask={:#04x} list={:?} at=({:.2},{:.2},{:.2})",
                t.placement, t.kind, t.out, t.gate, t.key_gate, t.mask, t.list, c[0], c[1], c[2]);
        }
    }
    eprintln!("=== 12.3 the raw placement param/extra sweep for 0x48 (72) ===");
    {
        use nf_formats::{disc, iso9660::IsoImage, level::Level};
        let img = IsoImage::open(&std::env::var("NF_ISO").unwrap()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000012.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        let m = level.main_map().unwrap();
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            let param_hits: Vec<u32> = p.params.iter().filter(|&(_, v)| *v == 0x48).map(|&(i, _)| i).collect();
            let extra_hits: Vec<usize> = p.extra.iter().enumerate().filter(|&(_, v)| *v == 0x48).map(|(i, _)| i).collect();
            if !param_hits.is_empty() || !extra_hits.is_empty() {
                eprintln!("plc[{pi}] flags={:#04x} at=({:.2},{:.2},{:.2}) PARAM hits idx {param_hits:?} EXTRA hits {extra_hits:?}",
                    p.flags, p.translation[0], p.translation[1], p.translation[2]);
            }
        }
        eprintln!("raw sweep done");
    }
    eprintln!("=== 12.4 the drones' death channels on 07000012 ===");
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.death_channel != 0 {
            eprintln!("drone[{i}] death_channel={:#04x}{}", d.death_channel,
                if d.death_channel == 0x48 { "   <== writes 0x48 on death!" } else { "" });
        }
    }
    eprintln!("=== 12 sweep done ===");
}
