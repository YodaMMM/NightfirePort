//! Full disc census [D] for level 07000011 (level-11-audit task 2-b): every placement of the
//! main map with every param (grouped by identical param-block when >300, every id kept), the
//! kind histogram, the unresolved count, and every kind-7 script (NIS) in the archive —
//! streams, ops with ticks, camera streams, hand-backs (event 6 = the in, op 0xF CameraEnd).
//! Sister probes: census_07000007.rs (placements), dump_sp_script.rs (one script).
//! Scratch print, not committed. Run: NF_ISO=<iso> cargo run -p nf-view --release --example census_07000011

use nf_formats::level::Level;
use nf_formats::nis::Cmd;
use nf_formats::placement::Placement;

fn main() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).expect("07000011.bin in FILES.BIN");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read 07000011.bin");
    let level = Level::parse(&data).expect("parse 07000011");
    let m = level.main_map().expect("main map");
    let main = &level.maps[m];
    println!(
        "maps: {} (main map #{m} name {} objects: {}, placements: {})",
        level.maps.len(), main.name, main.objects.len(), main.placements.len()
    );
    let mut total_all_maps = 0usize;
    for (mi, map) in level.maps.iter().enumerate() {
        total_all_maps += map.placements.len();
        if !map.placements.is_empty() {
            println!("  map #{mi} {} kind {} objects {} placements {}", map.name, map.kind, map.objects.len(), map.placements.len());
        }
    }
    println!("total placements over all maps: {total_all_maps}");

    let by_hash = |h: u32| level.by_hash.get(&h).copied();

    // ---- placement census (main map) ----
    println!("\n########## PLACEMENT CENSUS (main map) ##########");
    let resolve = |p: &Placement| -> (String, String) {
        if p.by_hash() {
            match by_hash(p.hashcode) {
                Some((map, object)) => (level.maps[map].objects[object].name.trim().to_string(), format!("hash {:08x} (map {map} obj {object})", p.hashcode)),
                None => (format!("<unresolved {:08x}>", p.hashcode), format!("hash {:08x}", p.hashcode)),
            }
        } else if (p.index as usize) < main.objects.len() {
            (main.objects[p.index as usize].name.trim().to_string(), format!("obj#{}", p.index))
        } else {
            (format!("<obj-oob #{}>", p.index), format!("obj#{} OOB", p.index))
        }
    };
    let param_str = |p: &Placement| -> String {
        if p.params.is_empty() { return "-".into(); }
        p.params.iter().map(|(k, v)| format!("p{k}={v:#x}")).collect::<Vec<_>>().join(" ")
    };
    let extra_str = |p: &Placement| -> String {
        if p.extra.iter().all(|&w| w == 0) { return String::new(); }
        let s = p.scale();
        let words: Vec<String> = p.extra.iter().enumerate().filter(|(_, &w)| w != 0).map(|(k, &w)| format!("e{k}={w:#x}")).collect();
        format!(" extra[{}] scale({:.2},{:.2},{:.2})", words.join(" "), s[0], s[1], s[2])
    };
    let rot_str = |p: &Placement| -> String {
        let r = p.rotation;
        if r.iter().all(|&a| a.abs() < 1e-6) { String::new() } else { format!(" rot({:.4},{:.4},{:.4})", r[0], r[1], r[2]) }
    };

    // unresolved count first
    let unresolved: Vec<usize> = main.placements.iter().enumerate()
        .filter(|(_, p)| resolve(p).0.starts_with('<')).map(|(i, _)| i).collect();
    println!("placements: {} total, {} unresolved", main.placements.len(), unresolved.len());
    if !unresolved.is_empty() {
        println!("UNRESOLVED ids: {:?}", unresolved);
    }

    // kind histogram: kind -> count -> distinct names
    println!("\n=== kind histogram (kind -> count -> names) ===");
    let mut by_kind: std::collections::BTreeMap<u8, Vec<usize>> = std::collections::BTreeMap::new();
    for (i, p) in main.placements.iter().enumerate() { by_kind.entry((p.flags & 0xff) as u8).or_default().push(i); }
    for (t, idxs) in &by_kind {
        let mut names: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        for &i in idxs { *names.entry(resolve(&main.placements[i]).0).or_default() += 1; }
        let list: Vec<String> = names.iter().map(|(n, c)| if *c > 1 { format!("{n}×{c}") } else { n.clone() }).collect();
        println!("  {t:#04x}: {} — {}", idxs.len(), list.join(", "));
    }

    // full table: group identical param-blocks (kind + name + rotation-bits + params, position/extra excluded)
    println!("\n=== full per-placement table (grouped by identical param-block; every id kept) ===");
    let grouped = main.placements.len() > 300;
    let key_of = |p: &Placement, name: &str| -> String {
        format!(
            "{}|{:x}|{}|{:x?}|{}",
            (p.flags & 0xff) as u8, p.flags >> 8, name,
            p.rotation.iter().map(|a| a.to_bits()).collect::<Vec<_>>(),
            param_str(p)
        )
    };
    if !grouped {
        for (i, p) in main.placements.iter().enumerate() {
            let (name, src) = resolve(p);
            println!("#{i} kind {:#04x} \"{name}\" {src} @({:.2},{:.2},{:.2}){} {}{}", (p.flags & 0xff) as u8,
                p.translation[0], p.translation[1], p.translation[2], rot_str(p), param_str(p), extra_str(p));
        }
    } else {
        // preserve first-seen order (kind-major, then first id)
        let mut order: Vec<String> = Vec::new();
        let mut groups: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        for (i, p) in main.placements.iter().enumerate() {
            let (name, _) = resolve(p);
            let k = key_of(p, &name);
            if !groups.contains_key(&k) { order.push(k.clone()); }
            groups.entry(k).or_default().push(i);
        }
        for k in &order {
            let ids = &groups[k];
            let p = &main.placements[ids[0]];
            let (name, _) = resolve(p);
            let ids_str = ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",");
            let head = format!("#{ids_str} kind {:#04x} \"{name}\" @({:.2},{:.2},{:.2}){} {}{}", (p.flags & 0xff) as u8,
                p.translation[0], p.translation[1], p.translation[2], rot_str(p), param_str(p), extra_str(p));
            if ids.len() == 1 {
                println!("{head}");
            } else {
                println!("{head}");
                // every member's own position (and extra when it varies within the group)
                let ats: Vec<String> = ids.iter().map(|&i| {
                    let q = &main.placements[i];
                    format!("#{}({:.2},{:.2},{:.2})", i, q.translation[0], q.translation[1], q.translation[2])
                }).collect();
                for chunk in ats.chunks(6) { println!("    at {}", chunk.join(" ")); }
                let mut extras: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
                for &i in ids { extras.insert(extra_str(&main.placements[i])); }
                if extras.len() > 1 {
                    let list: Vec<String> = extras.iter().take(6).cloned().collect();
                    println!("    extra varies ({} distinct: {}{})", extras.len(), list.join(" | "), if extras.len() > 6 { " …" } else { "" });
                }
            }
        }
    }

    // per-kind full detail for kinds the docs care about (every placement ungrouped, with position)
    println!("\n=== interactive kinds ungrouped (every placement with its own position) ===");
    for (t, idxs) in &by_kind {
        if matches!(t, 0x19 | 0x47 | 0x2A) { continue; } // decor cels / room-light volumes / skyline plates
        for &i in idxs {
            let p = &main.placements[i];
            let (name, src) = resolve(p);
            println!("#{i} kind {:#04x} \"{name}\" {src} @({:.2},{:.2},{:.2}){} {}{}", (p.flags & 0xff) as u8,
                p.translation[0], p.translation[1], p.translation[2], rot_str(p), param_str(p), extra_str(p));
        }
    }

    // ---- script census ----
    println!("\n########## SCRIPT CENSUS (all kind-7 entries, archive order) ##########");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("levelbin");
    let sevens: Vec<_> = lb.entries.iter().filter(|e| e.kind == 7).collect();
    println!("kind-7 entries: {}", sevens.len());
    let mut bad = 0usize;
    for se in &sevens {
        let nis = match nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) {
            Ok(n) => n,
            Err(err) => { println!("script {} ({:08x}): PARSE FAILED: {err}", se.name, se.hash()); bad += 1; continue; }
        };
        println!(
            "\nscript {} ({:08x}) camera {} loop_start {} length {} streams {} keys {}",
            se.name, nis.hashcode, nis.camera, nis.loop_start, nis.length, nis.streams.len(), nis.keys.len()
        );
        for (si, s) in nis.streams.iter().enumerate() {
            let cmds = s.decode();
            // tick simulation: ops execute when the stream clock reaches `wait`
            let mut wait = 0u32;
            let mut parts: Vec<String> = Vec::new();
            let mut has_cam = false;
            let mut handbacks: Vec<String> = Vec::new();
            for c in &cmds {
                let tick = wait;
                let (txt, new_wait, is_end) = match c {
                    Cmd::End(t) => (format!("End({t:#x})"), *t as u32, true),
                    Cmd::Wait(t) => (format!("Wait({t:#x})"), *t as u32, false),
                    Cmd::Marker(t) => (format!("Marker({t:#x})"), wait, false),
                    Cmd::Entity { hash, parent, spline, obj_type, params } => (
                        format!("Entity {:08x} parent {parent:?} spline {spline} type {obj_type:#04x} params {:?}", hash, params),
                        wait, false),
                    Cmd::EntityEnd => ("EntityEnd".into(), wait, false),
                    Cmd::Anim { script, skin, start } => (format!("Anim skin {skin:08x} script {script:08x} start {start}"), wait, false),
                    Cmd::AnimEnd => ("AnimEnd".into(), wait, false),
                    Cmd::Event { kind, params } => {
                        let ps = params.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(",");
                        (format!("Event {kind:#04x}[{ps}]"), wait, false)
                    }
                    Cmd::Sound { id, flat, has_obj, stream } => (format!("Sound {id} flat {flat} obj {has_obj} stream {stream}"), wait, false),
                    Cmd::SoundEnd => ("SoundEnd".into(), wait, false),
                    Cmd::Camera { spline, first, last } => { has_cam = true; (format!("Camera spline {spline} keys {first}..{last}"), wait, false) }
                    Cmd::CameraEnd => { has_cam = true; ("CameraEnd".into(), wait, false) }
                    Cmd::Unread(op) => (format!("Unread({op:#04x})"), wait, false),
                    Cmd::Skip { op } => (format!("Skip({op:#04x})"), wait, false),
                    Cmd::SubScript { hash, stream, mode } => (format!("SubScript {hash:08x} stream {stream} mode {mode}"), wait, false),
                    Cmd::Fade(f) => (format!("Fade({f})"), wait, false),
                    Cmd::Sprite { rect, flags, hash } => (format!("Sprite {:?} flags {flags:#x} hash {hash:08x}", rect), wait, false),
                    Cmd::Light { parent, kind } => (format!("Light parent {parent:?} kind {kind:#04x}"), wait, false),
                    Cmd::Text { label, time } => (format!("Text {label:08x} time {time}"), wait, false),
                    Cmd::Other(op) => (format!("Other({op:#04x})"), wait, false),
                };
                if let Cmd::Event { kind, params } = c {
                    if *kind == 6 { handbacks.push(format!("in ev6[{}] @{tick}", params.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(","))); }
                    if *kind == 4 { handbacks.push(format!("player-disable ev4 @{tick}")); }
                }
                if matches!(c, Cmd::CameraEnd) { handbacks.push(format!("CameraEnd @{tick}")); }
                parts.push(if new_wait != wait || matches!(c, Cmd::End(_) | Cmd::Wait(_)) { format!("{txt}@{tick}") } else { txt });
                wait = new_wait;
                if is_end { /* ops after End run only once clock passes the end time */ }
            }
            let cam_tag = if has_cam { " CAMERA-STREAM" } else { "" };
            let hb = if handbacks.is_empty() { String::new() } else { format!(" handbacks [{}]", handbacks.join("; ")) };
            println!("  s{si} keys {}..{} nops {}{}: {}{}", s.first_key, s.last_key, cmds.len(), cam_tag, parts.join(" ; "), hb);
        }
    }
    println!("\nscripts that failed to parse: {bad}");

    // ---- mission record (verified from the ELF snapshot the walk-test doc cites) ----
    println!("\n########## MISSION RECORD ##########");
    let snap = std::fs::read("/home/z/nf/work/ee_start_05.bin");
    match snap {
        Ok(snap) => {
            // MissionData (ACTION.ELF .data @0x2a4350): 24 records × 0x28; find level 0x07000011.
            let rd = |o: usize| u32::from_le_bytes(snap[o..o + 4].try_into().unwrap());
            for i in 0..24u32 {
                let r = 0x2a4350 + (i as usize) * 0x28;
                if rd(r) != 0x0700_0011 { continue; }
                let (mission, part, ptr, count) = (rd(r + 4), rd(r + 8), rd(r + 0xc) as usize, rd(r + 0x10) as usize);
                println!("MissionData record #{i}: level {:#x} mission {:#x} part {part} objectives @{:#x} count {count}", rd(r), mission, rd(r + 0xc));
                for k in 0..count {
                    let o = ptr + k * 0x18;
                    let (text, fail, hint) = (rd(o), rd(o + 4), rd(o + 8));
                    let b = |x: usize| snap[x];
                    println!(
                        "  objective {k}: text {text:#x} fail {fail:#x} hint {hint:#x} channel {:#x} init {} enable {:#x} flags {:#x}",
                        b(o + 0xc), b(o + 0xd), b(o + 0xf), b(o + 0x10)
                    );
                }
            }
        }
        Err(e) => println!("ELF snapshot not readable ({e}) — see crates/nf-game/src/mission/c07000011.rs"),
    }
}
