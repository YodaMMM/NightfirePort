//! interactables-1b console probe: dump the ScriptPlayers that reference the console scripts
//! 0x060007e2..ea, the scripts' own event streams (0x0D channel writes), and the eight
//! kind-0x39 SpaceMissile placements' params — the console→channel wiring on 0700001b.
//! Diagnostic only — run with --nocapture. [D via nf-formats.]

use nf_formats::iso9660::IsoImage;
use nf_formats::{level, levelbin, nis};

fn read_archive(iso_path: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn p3(v: [f32; 3]) -> String { format!("({:.2},{:.2},{:.2})", v[0], v[1], v[2]) }

#[test]
fn dump_console_wiring() {
    let path = std::env::var("NF_ISO").expect("NF_ISO");
    let data = read_archive(&path, "0700001b.bin");
    let lb = levelbin::LevelBin::parse(&data).unwrap();
    eprintln!("=== 0700001b console wiring ===");
    // The scripts (kind 7 NIS) by hashcode.
    let mut scripts: std::collections::HashMap<u32, nis::Nis> = std::collections::HashMap::new();
    for e in &lb.entries {
        if e.kind != 7 { continue; }
        if let Ok(n) = nis::Nis::parse(e.hash(), lb.data(&data, e)) {
            scripts.insert(n.hashcode, n);
        }
    }
    eprintln!("scripts {} in the archive", scripts.len());
    let level = level::Level::parse(&data).unwrap();

    // ScriptPlayers (0xD9) over every map: which script, mode, gates.
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            if p.flags & 0xff != 0xD9 { continue; }
            let par = |k: u32| p.params.iter().find(|(i, _)| *i == k).map_or(0u32, |&(_, v)| v);
            eprintln!("map{mi} SP#{} at={} scripts=[{:08x},{:08x}] modes=[{},{}] stay={} mask={:#x} out={:#x} hp={} gate={:#x} scale={}",
                p.index, p3(p.translation), par(0), par(1), par(2), par(3), par(4), par(5), par(6), par(7), par(9), par(10));
        }
    }

    // The console scripts' streams + decoded events.
    for h in [0x060007e2u32, 0x060007e3, 0x060007e4, 0x060007e5, 0x060007e6, 0x060007e7, 0x060007e8, 0x060007e9, 0x060007ea] {
        let Some(n) = scripts.get(&h) else { eprintln!("{h:08x}: NOT PRESENT"); continue };
        eprintln!("--- script {:08x}: {} streams, length {} ---", n.hashcode, n.streams.len(), n.length);
        for (si, s) in n.streams.iter().enumerate() {
            eprintln!("  stream[{si}] keys {}..{}", s.first_key, s.last_key);
            for c in s.decode() {
                if let nf_formats::nis::Cmd::Event { kind, params } = c {
                    let ps = params.iter().map(|x| format!("{x:#x}")).collect::<Vec<_>>().join(",");
                    eprintln!("    EVENT kind {:#04x} [{}]", kind, ps);
                }
            }
        }
    }

    // The eight missiles (0x39) + touch triggers (1/2) on every map.
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            let f = p.flags & 0xff;
            let par = |k: u32| p.params.iter().find(|(i, _)| *i == k).map_or(0u32, |&(_, v)| v);
            if f == 0x39 {
                eprintln!("map{mi} MISSILE#{} at={} p0={:08x} p1={:08x} p2={:08x} p3={} p4={} p5={:#x} p6={} p7={:#x}",
                    p.index, p3(p.translation), par(0), par(1), par(2), par(3), par(4), par(5), par(6), par(7));
            }
            if matches!(f, 1 | 2) {
                eprintln!("map{mi} TOUCH#{} kind={} at={} p0={:x} p1={:x} p2={:x} p3={:x} p4={:x} p5={:x} p6={:x} p7={:x}",
                    p.index, f, p3(p.translation), par(0), par(1), par(2), par(3), par(4), par(5), par(6), par(7));
            }
        }
    }
    eprintln!("=== end console wiring ===");
}

#[test]
fn dump_flag_kinds_and_missiles() {
    let path = std::env::var("NF_ISO").expect("NF_ISO");
    let data = read_archive(&path, "0700001b.bin");
    let level = level::Level::parse(&data).unwrap();
    eprintln!("maps {} (main {:?})", level.maps.len(), level.main_map());
    for (mi, m) in level.maps.iter().enumerate() {
        let mut kinds: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
        for p in &m.placements { *kinds.entry(p.flags & 0xff).or_default() += 1; }
        let ks = kinds.iter().map(|(k, n)| format!("{k:#04x}x{n}")).collect::<Vec<_>>().join(" ");
        eprintln!("map{mi} kind={} placements {} kinds: {ks}", m.kind, m.placements.len());
    }
    // The missile placements by index on the main map.
    let mm = level.main_map().unwrap();
    for p in level.maps[mm].placements.iter() {
        let f = p.flags & 0xff;
        if matches!(f, 0x39 | 0xe1 | 0xe4 | 0x47 | 0x4a | 0x38 | 0x3b | 0x2a | 0xec) {
            eprintln!("PL#{} flags={:#x} at={} params {:x?}", p.index, p.flags, p3(p.translation), p.params);
        }
    }
}

#[test]
fn dump_channel_writers() {
    let path = std::env::var("NF_ISO").expect("NF_ISO");
    let data = read_archive(&path, "0700001b.bin");
    let lb = levelbin::LevelBin::parse(&data).unwrap();
    let mut scripts: std::collections::HashMap<u32, nis::Nis> = std::collections::HashMap::new();
    for e in &lb.entries {
        if e.kind != 7 { continue; }
        if let Ok(n) = nis::Nis::parse(e.hash(), lb.data(&data, e)) { scripts.insert(n.hashcode, n); }
    }
    // Every script's channel writes (event 0x0D [ch, v] / 0x11 [c_on, c_off, v]).
    for (h, n) in &scripts {
        let mut writes: Vec<String> = Vec::new();
        for s in &n.streams {
            for c in s.decode() {
                if let nf_formats::nis::Cmd::Event { kind, params } = c {
                    let g = |k: usize| params.get(k).copied().unwrap_or(0);
                    match kind {
                        0xd if g(0) != 0 => writes.push(format!("w{}", g(0))),
                        0x11 => {
                            let c = if g(0) != 0 { g(1) } else { g(2) };
                            if c != 0 { writes.push(format!("w{c}")); }
                        }
                        _ => {}
                    }
                }
            }
        }
        if !writes.is_empty() {
            eprintln!("WRITER {:08x} len {}: {}", h, n.length, writes.join(" "));
        }
    }
    eprintln!("end channel writers");
}

#[test]
fn dump_console_script_full() {
    let path = std::env::var("NF_ISO").expect("NF_ISO");
    let data = read_archive(&path, "0700001b.bin");
    let lb = levelbin::LevelBin::parse(&data).unwrap();
    for e in &lb.entries {
        if e.kind != 7 { continue; }
        let h = e.hash();
        if !matches!(h, 0x060007e2 | 0x060007e7 | 0x060007e6) { continue; }
        let Ok(n) = nis::Nis::parse(h, lb.data(&data, e)) else { continue };
        eprintln!("=== script {:08x} len {} camera={} ===", h, n.length, n.camera);
        for (si, s) in n.streams.iter().enumerate() {
            let kpos = if s.first_key < n.keys.len() as u16 { let k = &n.keys[s.first_key as usize]; format!(" first ({:.2},{:.2},{:.2})", k.pos[0], k.pos[1], k.pos[2]) } else { String::new() };
            eprintln!(" stream[{si}] keys {}..{}{kpos}", s.first_key, s.last_key);
            for c in s.decode() {
                match c {
                    nf_formats::nis::Cmd::Event { kind, params } => {
                        let ps = params.iter().map(|x| format!("{x:#x}")).collect::<Vec<_>>().join(",");
                        eprintln!("   EVENT {:#04x} [{}]", kind, ps);
                    }
                    nf_formats::nis::Cmd::Entity { hash, parent, spline, obj_type, params } => {
                        eprintln!("   ENTITY hash={:08x} parent={parent:?} class={obj_type:#x} stream{si}", hash);
                    }
                    nf_formats::nis::Cmd::Wait(t) => eprintln!("   WAIT {t}"),
                    nf_formats::nis::Cmd::Marker(t) => eprintln!("   MARKER {t}"),
                    nf_formats::nis::Cmd::SubScript { hash, stream, mode } => eprintln!("   SUB {hash:08x} stream {stream} mode {mode}"),
                    _ => {}
                }
            }
        }
    }
    eprintln!("end console script full");
}

#[test]
fn dump_entity_hash_resolution() {
    let path = std::env::var("NF_ISO").expect("NF_ISO");
    let data = read_archive(&path, "0700001b.bin");
    let level = level::Level::parse(&data).unwrap();
    for h in [0x020007f8u32, 0x02000961, 0x02000962, 0x020007ff, 0x02000989, 0x02000b04] {
        let mut found = None;
        for (mi, m) in level.maps.iter().enumerate() {
            for (oi, o) in m.objects.iter().enumerate() {
                if o.hashcode == h { found = Some((mi, oi, o.centre, o.radius)); }
            }
        }
        match found {
            Some((mi, oi, c, r)) => eprintln!("hash {h:08x}: map{mi} obj{oi} centre ({:.2},{:.2},{:.2}) r {r:.2}", c[0], c[1], c[2]),
            None => eprintln!("hash {h:08x}: NOT in any map's objects"),
        }
    }
    eprintln!("end hash resolution");
}
