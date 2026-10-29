//! level-1b census: dump every 0700001b main-map placement by kind, with params. Opt-in NF_ISO; prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_0700001b_placements_by_kind() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap();
    println!("maps {} main {mm} objects[mm] {}", lv.maps.len(), lv.maps[mm].objects.len());
    let placements = &lv.maps[mm].placements;
    println!("main-map placements: {}", placements.len());
    let mut by_kind: Vec<(u32, Vec<usize>)> = Vec::new();
    for (i, p) in placements.iter().enumerate() {
        let k = p.flags & 0xff;
        match by_kind.iter_mut().find(|(kk, _)| *kk == k) {
            Some((_, v)) => v.push(i),
            None => by_kind.push((k, vec![i])),
        }
    }
    by_kind.sort();
    for (k, idxs) in &by_kind {
        println!("== KIND {k:#04x}: {} placements ==", idxs.len());
        for &i in idxs {
            let p = &placements[i];
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let name = target.map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
            let t = p.translation;
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            let cel = if p.creates_cel() { " CEL" } else { "" };
            println!("#{i} kind {k:#04x}{cel} \"{}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
                name, t[0], t[1], t[2], params.join(" "), p.hashcode, p.flags);
        }
    }
}

/// The level-bin script streams: decode every kind-7 script entry of 0700001b (the level bin +
/// the pack bins) with ops per stream — entity/anims, event writes, camera ops. Opt-in NF_ISO; prints only.
#[test]
#[ignore = "assertion-free script dump; run with --ignored"]
fn dump_0700001b_scripts() {
    use nf_formats::levelbin::LevelBin;
    use nf_formats::nis::{Cmd, Nis};
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    for archive in ["0700001b.bin", "0780001b.bin", "0790001b.bin"] {
        let data = match std::panic::catch_unwind(|| read_archive(&iso, archive)) {
            Ok(d) => d,
            Err(_) => { println!("== {archive}: not present =="); continue; }
        };
        let lb = LevelBin::parse(&data).expect("levelbin");
        let script_count = lb.entries.iter().filter(|e| e.kind == 7).count();
        println!("== {archive}: {} entries, {script_count} kind-7 scripts ==", lb.entries.len());
        for se in lb.entries.iter().filter(|e| e.kind == 7) {
            let nis = match Nis::parse(se.hash(), lb.data(&data, se)) {
                Ok(n) => n,
                Err(e) => { println!("script {:08x}: parse failed {e}", se.hash()); continue; }
            };
            println!("script {:08x} len {} streams {}", nis.hashcode, nis.length, nis.streams.len());
            for (si, s) in nis.streams.iter().enumerate() {
                println!(" stream {si}: keys {}..{}", s.first_key, s.last_key);
                for c in s.decode() {
                    match c {
                        Cmd::Entity { hash, parent, spline, obj_type, params } =>
                            println!("   Entity glist {hash:08x} parent {parent:?} spline {spline} type {obj_type:02x} params {params:?}"),
                        Cmd::Anim { skin, script, start } => println!("   Anim skin {skin:08x} script {script:08x} start {start}"),
                        Cmd::Event { kind, params } => println!("   Event kind {kind:02x} params {params:?}"),
                        other => println!("   {other:?}"),
                    }
                }
            }
        }
    }
}
