//! lift-indicators-09 scratch probe: 07000009's garage-lift indicators and call button.
//! 1. Every placement whose class (flags & 0xff) is script-ish (0xd9 SP / 0x29 switch / 0x2f copter /
//!    0xe2 SS / 0x1b), with its script hashes from params — near the lift lobby first.
//! 2. Every script reachable from those roots (plus sub-scripts), decoded: Entity/Sprite/Light/Anim
//!    ops with hashes, kinds, params — the indicator/button candidates.
//! 3. The sim after N ticks: scripts.objects.list — every object's kind, hash, position, src.
//! 4. The SS door objects (#2/#3): state and streams.
//! Scratch print, not committed evidence until cited in the report.

use nf_formats::nis::Cmd;

fn main() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000009.bin")).expect("07000009.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = nf_formats::level::Level::parse(&data).expect("parse 07000009");

    // 1. The script-ish placements.
    println!("== script-ish placements (class: index hash translation params) ==");
    for m in 0..level.maps.len() {
        for p in &level.maps[m].placements {
            let c = p.flags & 0xff;
            if matches!(c, 0xd9 | 0x29 | 0x2f | 0xe2 | 0x1b) {
                let t = p.translation;
                let scripts: Vec<String> = p.params.iter()
                    .filter(|(_, v)| *v >= 0x0600_0000)
                    .map(|(k, v)| format!("{k}:{v:#x}"))
                    .collect();
                println!("  m{m} {:02x} #{} {:08x} ({:.2},{:.2},{:.2}) scripts [{}] params {:?}",
                    c, p.index, p.hashcode, t[0], t[1], t[2], scripts.join(","),
                    p.params.iter().take(8).collect::<Vec<_>>());
            }
        }
    }

    // 2. Every script reachable from those roots + the fixed explosion set; decode streams.
    let mut roots: Vec<u32> = Vec::new();
    for m in 0..level.maps.len() {
        for p in &level.maps[m].placements {
            let c = p.flags & 0xff;
            if matches!(c, 0xd9 | 0x29 | 0x2f) {
                for (_, v) in &p.params {
                    if *v >= 0x0600_0000 { roots.push(*v); }
                }
            }
        }
    }
    roots.extend([0x0600_0052u32, 0x0600_0027, 0x0600_004f, 0x0600_07c4]);
    let mut queued: std::collections::HashSet<u32> = roots.iter().copied().collect();
    let mut all: Vec<u32> = roots.clone();
    while let Some(h) = all.pop() {
        let Some(nis) = level.nis.get(&h) else { continue };
        for st in &nis.streams {
            for cmd in st.decode() {
                if let Cmd::SubScript { hash: sub, .. } = cmd {
                    if sub != 0 && queued.insert(sub) { all.push(sub); }
                }
            }
        }
    }
    println!("== {} reachable scripts; their create ops ==", queued.len());
    let mut hashes_sorted: Vec<u32> = queued.iter().copied().collect();
    hashes_sorted.sort_unstable();
    for h in hashes_sorted {
        let Some(nis) = level.nis.get(&h) else { continue };
        for (si, st) in nis.streams.iter().enumerate() {
            for cmd in st.decode() {
                match cmd {
                    Cmd::Entity { hash, obj_type, params, parent, spline } => {
                        println!("  {:08x} s{} ENTITY gfx {hash:#x} kind {obj_type:#x} parent {parent:?} spline {spline} params {:?}", h, si, params);
                    }
                    Cmd::Sprite { rect, flags, hash } => println!("  {:08x} s{} SPRITE {hash:#x} rect {:?} flags {flags:#x}", h, si, rect),
                    Cmd::Light { parent, kind } => println!("  {:08x} s{} LIGHT kind {kind:#x} parent {parent:?}", h, si),
                    Cmd::Anim { script, skin, start } => println!("  {:08x} s{} ANIM script {script:#x} skin {skin:#x} start {start}", h, si),
                    Cmd::SubScript { hash, stream, mode } => println!("  {:08x} s{} SUB {hash:#x} stream {stream} mode {mode}", h, si),
                    _ => {}
                }
            }
        }
    }

    // 3. The sim after ticks: what the scripts created.
    let mut scene = nf_view::load(&iso_path, "07000009.bin", false).expect("load 07000009");
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        println!("== sim objects after 120 ticks ({} players) ==", sim.drones.scripts.players.len());
        for (i, p) in sim.drones.scripts.players.iter().enumerate() {
            println!("  player[{i}] state {} cur {} mask {:x} scripts {:?}", p.state, p.cur, p.mask,
                p.scripts.iter().filter(|&&h| h != 0).map(|h| format!("{h:#x}")).collect::<Vec<_>>());
        }
        for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
            let Some(o) = o else { continue };
            match o {
                nf_game::nis::Obj::Entity { hash, kind, matrix, src, .. } => {
                    let t = [matrix[0][3], matrix[1][3], matrix[2][3]];
                    println!("  obj[{i}] ENTITY gfx {hash:#x} kind {kind:#x} at ({:.2},{:.2},{:.2}) src {:?}", t[0], t[1], t[2], src);
                }
                nf_game::nis::Obj::Anim { skin, script, frame, src, .. } =>
                    println!("  obj[{i}] ANIM skin {skin:#x} script {script:#x} frame {frame} src {:?}", src),
                nf_game::nis::Obj::Sprite { hash, rect, src, .. } =>
                    println!("  obj[{i}] SPRITE {hash:#x} rect {:?} src {:?}", rect, src),
                nf_game::nis::Obj::Light { kind, pos, radius, src, .. } =>
                    println!("  obj[{i}] LIGHT kind {kind:#x} at ({:.2},{:.2},{:.2}) r {radius:.2} src {:?}", pos[0], pos[1], pos[2], src),
                nf_game::nis::Obj::Camera { src, .. } => println!("  obj[{i}] CAMERA src {:?}", src),
            }
        }
        // The SS doors.
        println!("== SS objects ({} total) ==", sim.drones.sss.list.len());
        for (i, s) in sim.drones.sss.list.iter().enumerate() {
            println!("  ss[{i}] state {:#x} pos ({:.2},{:.2},{:.2})", s.state, s.pos()[0], s.pos()[1], s.pos()[2]);
        }
        // The maps: kind + name (main map = kind 1).
        println!("== maps ==");
        for (m, lm) in level.maps.iter().enumerate() {
            println!("  m{} kind {} name {:?}", m, lm.kind, lm.name);
        }
        // The stream KEYS of the lift scripts: the world positions the interp should produce.
        println!("== stream keys (first 3 + last) for the lift scripts' entity streams ==");
        for h in [0x0600_059bu32, 0x0600_007d, 0x0600_007e, 0x0600_064e, 0x0600_064f] {
            let Some(nis) = level.nis.get(&h) else { continue };
            let keys = &nis.keys;
            println!("  {h:#x}: {} keys total, {} streams", keys.len(), nis.streams.len());
            for (si, st) in nis.streams.iter().enumerate() {
                if !st.decode().iter().any(|c| matches!(c, Cmd::Entity { .. } | Cmd::SubScript { .. })) { continue; }
                let (fk, lk) = (st.first_key as usize, st.last_key as usize);
                let idx = [fk.saturating_sub(1), fk, fk + 1, lk.saturating_sub(2), lk.saturating_sub(1)];
                let mut row = format!("    s{si} keys {fk}..{lk}:");
                for (n, &ki) in idx.iter().enumerate() {
                    if let Some(k) = keys.get(ki) {
                        if n > 0 && ki == idx[n - 1] { continue; }
                        row.push_str(&format!(" [{}]{:.2},{:.2},{:.2}", ki, k.pos[0], k.pos[1], k.pos[2]));
                    }
                }
                println!("{row}");
            }
        }
        // The candidate gfx hashes' baked bounds (the world-space panels).
        let cands: [u32; 24] = [0x20005ea, 0x20005eb, 0x2000506, 0x2000507, 0x2000508, 0x2000509, 0x200050a,
            0x20006a8, 0x20006a9, 0x20006ae, 0x20006af, 0x20006b0, 0x20006be, 0x2000a91,
            0x20006b7, 0x20006b8, 0x20006b9, 0x20006ba, 0x20006b2, 0x20006b3,
            0x20005ed, 0x20005ec, 0x2000a29, 0x2000499];
        println!("== gfx bounds ==");
        for h in cands {
            match sim.glist_bounds.get(&h) {
                Some((lo, hi)) => println!("  {h:#x} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]),
                None => println!("  {h:#x} (no bounds in glist_bounds)"),
            }
        }
        // The RENDER's view: draw slots per (script, stream, hash) and what script_entity_k returns.
        println!("== draw slots (attachments with a script tuple) for the lift hashes ==");
        let mut slots: std::collections::BTreeMap<(u32, u8, u32), Vec<u8>> = std::collections::BTreeMap::new();
        for a in &scene.attachments {
            if let Some((h, si, hash, k)) = a.script {
                if matches!(hash, 0x20005ea | 0x20005eb | 0x2000506..=0x200050a | 0x20006a8 | 0x20006a9
                    | 0x20006ae..=0x20006b3 | 0x20006b7..=0x20006ba | 0x20006be | 0x2000a91) {
                    slots.entry((h, si, hash)).or_default().push(k);
                }
            }
        }
        for ((h, si, hash), ks) in &slots {
            println!("  slot script {h:#x} s{si} gfx {hash:#x} copies {ks:?}");
        }
        let hidden = sim.drones.scripts.hidden_stream_objects();
        println!("  hidden stream objects: {hidden:?}");
        println!("== script_entity_k per sim entity (the render's answer) ==");
        for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
            let Some(nf_game::nis::Obj::Entity { hash, matrix, src, .. }) = o else { continue };
            let got = sim.script_entity_k(*src, *hash, 0);
            let t = [matrix[0][3], matrix[1][3], matrix[2][3]];
            let rt = got.as_ref().map(|m| [m[0][3], m[1][3], m[2][3]]);
            println!("  obj[{i}] gfx {hash:#x} src {src:?} sim_t ({:.2},{:.2},{:.2}) render {:?}", t[0], t[1], t[2],
                rt.map(|r| format!("({:.2},{:.2},{:.2})", r[0], r[1], r[2])).unwrap_or("NONE".into()));
        }
    }
}
