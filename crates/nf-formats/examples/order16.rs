//! Probe: what the 07000016 real mission order (arrival → spawn plate → station/panels → silo
//! plate → the ch-0x81 bridge trigger) sets BEFORE the launch chain, and whether any of it selects
//! a branch: SPs gated/triggered on those channels, scripts' conditional channel writes (event
//! 0x11), hand-back poses (event 6), scripts that move Bond (ENTITY streams), hurt volumes whose
//! gate the order feeds. Opt-in via NF_ISO. Evidence: [D] disc bytes via Level::parse + Nis::parse;
//! the param layouts are [M Trigger_Create 0x1b1720:61-91] (out=p1, inputs=p2..p8, mask=p9),
//! [M SP_CreateScriptPlayer 0x1db9a8:18-61] (p0/p1 scripts, p2/p3 modes, p4 stay, p5 mask, p6 out,
//! p9 gate) and [M Script_EventHandler 0x1d94c0] (0xd writes switch_channels, 0x11 branches on
//! switch_channels, 6 hands the player back at the previous stream's entity).
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin, nis::Cmd, nis::Nis};

fn param(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0)
}

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();

    // ---- the level + every level-16 pack bin ----
    let mut packs: Vec<(String, Vec<u8>)> = Vec::new();
    for e in &entries {
        let n = e.name.to_ascii_lowercase();
        if n.ends_with("00016.bin") {
            let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).unwrap();
            packs.push((e.name.clone(), data));
        }
    }
    packs.sort();
    println!("## pack bins: {:?}", packs.iter().map(|(n, d)| (n.as_str(), d.len())).collect::<Vec<_>>());

    let main_data = &packs.iter().find(|(n, _)| n.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin").1;
    let level = Level::parse(main_data).expect("level parses");
    let mi = level.main_map().unwrap();
    let m = &level.maps[mi];

    // ---- Section A: the real-order on-set (the carried channels + the route's writes, closed over
    // the AND/OR/OUT muxes [M Trigger_Create 0x1b1720 + Trigger_Hit 0x1b1a78]) ----
    let carried = [0x47u32, 0x48, 0x4a, 0x54, 0xce, 0x4b];
    // The route's writes, each with its [D] placement (the walkreal test asserts these fire):
    let route = [
        (0x8eu32, "#252 the spawn plate (TouchOnce, the arrival)"),
        (0x24, "#250 the SecurityStation E panel (mask 4)"),
        (0x27, "door[3]'s E panel"),
        (0x28, "door[2]'s E panel"),
        (0xb4, "door[4]'s E panel"),
        (0x8f, "#253 the silo plate (TouchOnce)"),
    ];
    let mut on: std::collections::BTreeSet<u32> = carried.iter().copied().collect();
    for (c, w) in route { on.insert(c); println!("[route] ch {c:#04x} <- {w}"); }
    // The mux closure.
    let mut muxes: Vec<(usize, u8, u32, Vec<u32>)> = Vec::new(); // (place, kind, out, inputs)
    for (i, p) in m.placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        if !matches!(kind, 0xec | 0xed | 0xee | 0xef) { continue; }
        // The mux family: out = p0, inputs = p1..p6 + p8, mask = p9 [M 0x1b2c68 wrapper + Trigger_Create
        // 0x1b1720:80-91 — from_params' `_` arm].
        let inputs: Vec<u32> = (1..=6).map(|k| param(p, k)).chain(std::iter::once(param(p, 8))).filter(|&v| v != 0).collect();
        muxes.push((i, kind as u8, param(p, 0), inputs));
    }
    loop {
        let mut grew = false;
        for &(i, kind, out, ref inputs) in &muxes {
            if on.contains(&out) { continue; }
            let fire = match kind {
                0xec => inputs.iter().all(|c| on.contains(c)),
                0xef => inputs.iter().any(|c| on.contains(c)),
                0xee => inputs.iter().any(|c| on.contains(c)),
                _ => false, // 0xED SEQ_AND needs order/timing — flagged, not closed
            };
            if fire {
                on.insert(out);
                println!("[mux] place[{i}] kind {kind:#04x} inputs{:?} -> ch {out:#04x} ON", inputs);
                grew = true;
            }
        }
        if !grew { break; }
    }
    for &(i, _kind, out, ref inputs) in &muxes {
        if !on.contains(&out) && inputs.iter().any(|c| on.contains(c)) {
            println!("[mux-partial] place[{i}] out {out:#04x}: some inputs on {:?}, inputs cold {:?}",
                inputs.iter().filter(|c| on.contains(c)).collect::<Vec<_>>(),
                inputs.iter().filter(|c| !on.contains(c)).collect::<Vec<_>>());
        }
    }
    let on: Vec<u32> = on.into_iter().collect();
    println!("## real-order on-set before the bridge trigger: {:x?}", on);

    // ---- Section B: every kind-0xD9 script player ----
    println!("\n## script players (kind 0xD9) — p0/p1 script A/B, p2/p3 mode, p4 stay, p5 mask, p6 out, p9 gate");
    for (i, p) in m.placements.iter().enumerate() {
        if p.flags & 0xff != 0xd9 { continue; }
        let (p0, p1) = (param(p, 0), param(p, 1));
        if p0 == 0 && p1 == 0 { continue; }
        let (g, o, msk, stay) = (param(p, 9), param(p, 6), param(p, 5), param(p, 4));
        let watch = if on.contains(&g) { " GATE-ON-in-real-order" } else { "" };
        let watch2 = if msk & 1 != 0 && on.contains(&o) { " OUT-ON-in-real-order(SP_Hit bit1 fires it!)" } else { "" };
        println!("SP#{i} pos ({:.2},{:.2},{:.2}) A {p0:08x} B {p1:08x} modes {}/{} stay {stay} mask {msk:#x} out {o:#04x} gate {g:#04x} p7dmg {:#x} p8filt {:#x} p10scale {}{watch}{watch2}",
            p.translation[0], p.translation[1], p.translation[2], param(p, 2), param(p, 3), param(p, 7), param(p, 8), param(p, 10));
    }

    // ---- Section C: every placement of any kind whose params name a real-order channel ----
    println!("\n## placements (any kind) whose params name a real-order channel");
    for (i, p) in m.placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        let hits: Vec<u32> = p.params.iter().map(|&(_, v)| v).filter(|v| on.contains(v)).collect();
        if hits.is_empty() { continue; }
        println!("place[{i}] kind {kind:#04x} idx {} pos ({:.2},{:.2},{:.2}) params {:?} hits {:x?}",
            p.index, p.translation[0], p.translation[1], p.translation[2], p.params, hits);
    }

    // ---- Section D: hurt volumes (kind 0xE4) — every gate channel ----
    println!("\n## hurt volumes (kind 0xE4) with gate channels");
    for (i, p) in m.placements.iter().enumerate() {
        if p.flags & 0xff != 0xe4 { continue; }
        let gate = param(p, 2);
        let mark = if on.contains(&gate) { " GATE-ON-in-real-order!" } else { "" };
        println!("hurt place[{i}] idx {} pos ({:.2},{:.2},{:.2}) params {:?} gate {gate}{mark}",
            p.index, p.translation[0], p.translation[1], p.translation[2], p.params);
    }

    // ---- Section E: every script in every level-16 pack ----
    // Bond's hash: the ENTITY streams of the chain scripts (0x06000631 the cinema, 0x0600090a the launch).
    println!("\n## scripts");
    let mut script_map: std::collections::BTreeMap<u32, String> = Default::default();
    let mut bond_hash = None;
    for (name, data) in &packs {
        let Ok(lb) = LevelBin::parse(data) else { println!("## {name}: not a level bin"); continue };
        let mut scripts: Vec<_> = lb.entries.iter().filter(|e| e.kind == 7).collect();
        scripts.sort_by_key(|e| u32::from_str_radix(&e.name, 16).unwrap_or(0));
        for e in &scripts {
            let hash = u32::from_str_radix(&e.name, 16).unwrap_or(0);
            script_map.insert(hash, name.clone());
            let Ok(n) = Nis::parse(hash, lb.data(data, e)) else { println!("{name} {hash:08x}: PARSE FAILED"); continue };
            let chain = matches!(hash, 0x06000631 | 0x0600090a | 0x06000632);
            println!("=== {name} {hash:08x} length {} camera {} loop {} streams {}", n.length, n.camera, n.loop_start, n.streams.len());
            for (si, s) in n.streams.iter().enumerate() {
                let kr = if s.last_key >= s.first_key && (s.last_key as usize) < n.keys.len() {
                    let k0 = &n.keys[s.first_key as usize]; let kl = &n.keys[s.last_key as usize];
                    format!("keys{}..{} t{:.1}..{:.1} p0({:.2},{:.2},{:.2}) pend({:.2},{:.2},{:.2})",
                        s.first_key, s.last_key, k0.time, kl.time, k0.pos[0], k0.pos[1], k0.pos[2], kl.pos[0], kl.pos[1], kl.pos[2])
                } else { "nokeys".into() };
                let mut parts = Vec::new();
                let mut clock = 0.0f32;
                for c in s.decode() {
                    match &c {
                        Cmd::Wait(t) => { clock += *t as f32; }
                        Cmd::Event { kind, params } => {
                            let interesting = matches!(kind, 3 | 4 | 5 | 6 | 7 | 8 | 9 | 0xd | 0x11 | 0x12 | 0xf | 0xa | 0xb);
                            if interesting {
                                let note = if *kind == 0x11 && on.contains(&params.first().copied().unwrap_or(0)) { " <== COND-ON-in-real-order" }
                                    else if *kind == 0xd && params.first().is_some_and(|c| on.contains(c)) { " <== WRITES-real-order-ch" }
                                    else if *kind == 6 { " <== handback" }
                                    else if *kind == 4 { " <== player-disable" }
                                    else { "" };
                                parts.push(format!("@{:.0} EV{kind:#04x}{:?}{note}", clock, params));
                            }
                        }
                        Cmd::Entity { hash, .. } => {
                            parts.push(format!("@{:.0} ENTITY {hash:08x}", clock));
                            if chain && bond_hash.is_none() { bond_hash = Some(*hash); }
                        }
                        Cmd::Anim { script, .. } => parts.push(format!("@{:.0} ANIM {script:08x}", clock)),
                        Cmd::SubScript { hash, stream, mode } => parts.push(format!("@{:.0} SUB {hash:08x} s{stream} m{mode}", clock)),
                        Cmd::Text { label, time } => parts.push(format!("@{:.0} TEXT {label:08x} t{}", clock, time)),
                        Cmd::End(_) => parts.push(format!("@{:.0} END", clock)),
                        _ => {}
                    }
                }
                if !parts.is_empty() || chain {
                    println!("  s{si} {kr} :: {}", parts.join(" | "));
                }
            }
        }
    }

    // ---- Section F: Bond's hash → every script stream + placement carrying it ----
    if let Some(bh) = bond_hash {
        println!("\n## Bond entity hash {bh:08x} (the first ENTITY of the chain scripts)");
        for (i, p) in m.placements.iter().enumerate() {
            if p.hashcode == bh {
                println!("  placement[{i}] kind {:#04x} pos {:?}", p.flags & 0xff, p.translation);
            }
        }
        // objects by name
        for (oi, o) in m.objects.iter().enumerate() {
            if o.hashcode == bh { println!("  object[{oi}] name {:?}", o.name); }
        }
    } else {
        println!("\n## no ENTITY in the chain scripts — Bond moved another way");
    }
    println!("\n## script->pack map ({}): {:?}", script_map.len(),
        script_map.iter().map(|(h, n)| (format!("{h:08x}"), n.as_str())).collect::<Vec<_>>());
}
