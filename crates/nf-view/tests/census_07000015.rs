//! level-15 census: dump every 07000015 placement by kind, with params, across all maps.
//! Opt-in NF_ISO; prints only. Docs: docs/levels/07000015.md.
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
fn dump_07000015_placements_by_kind() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000015.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap_or(0);
    println!("maps {} main {mm}", lv.maps.len());
    let mut placements: Vec<(usize, &nf_formats::placement::Placement)> = Vec::new();
    for (mi, m) in lv.maps.iter().enumerate() {
        for p in &m.placements { placements.push((mi, p)); }
    }
    println!("all-map placements: {}", placements.len());
    let mut by_kind: Vec<(u32, Vec<usize>)> = Vec::new();
    for (i, (_, p)) in placements.iter().enumerate() {
        let k = p.flags & 0xff;
        match by_kind.iter_mut().find(|(kk, _)| *kk == k) {
            Some((_, v)) => v.push(i),
            None => by_kind.push((k, vec![i])),
        }
    }
    by_kind.sort();
    for (k, idxs) in &by_kind {
        println!("== KIND {k:#04x}: {} placements ==", idxs.len());
        if *k == 0x47 {
            // The room placements: the map each references (the port's sub-map loader reads
            // p.index for kind 0x47 [S] — print the raw index to map the placed rooms).
            for &i in idxs {
                let (mi, p) = placements[i];
                println!("m{mi} #{i} room-ref index {} flags {:#x} t {:?}", p.index, p.flags, p.translation);
            }
            continue;
        }
        for &i in idxs {
            let (mi, p) = placements[i];
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mi].objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
            let t = p.translation;
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            let cel = if p.creates_cel() { " CEL" } else { "" };
            println!("m{mi} #{i} kind {k:#04x}{cel} \"{}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
                name, t[0], t[1], t[2], params.join(" "), p.hashcode, p.flags);
        }
    }
}

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_07000015_mine_objects() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000015.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().unwrap_or(0);
    for p in &lv.maps[mm].placements {
        if p.flags & 0xff != 0xfe { continue; }
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        if let Some((m, o)) = target {
            let obj = &lv.maps[m].objects[o];
            match &obj.collision {
                Some(c) => {
                    let mut lo = [f32::MAX; 3]; let mut hi = [-f32::MAX; 3];
                    let tris = c.triangles();
                    let n = tris.len();
                    for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
                    println!("mine m{mm}#{o} \"{}\" tris {n} lo {lo:.2?} hi {hi:.2?} centre {:.2?} r {:.2}", obj.name.trim(), obj.centre, obj.radius);
                }
                None => println!("mine m{mm}#{o} \"{}\" NO collision, centre {:.2?} r {:.2}", obj.name.trim(), obj.centre, obj.radius),
            }
        } else { println!("mine UNRESOLVED hash {:08x}", p.hashcode); }
    }
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_07000015_scripts() {
    // The level's kind-7 scripts through the game's own loader filter (Level::parse keeps
    // what Script_Load accepts) plus the raw LevelBin kind-7 census [D].
    use nf_formats::levelbin::LevelBin;
    use nf_formats::nis::{Cmd, Nis};
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000015.bin");
    let lb = LevelBin::parse(&data).expect("levelbin");
    let kind7: Vec<_> = lb.entries.iter().filter(|e| e.kind == 7).collect();
    println!("== 07000015.bin: {} entries, {} kind-7 scripts ==", lb.entries.len(), kind7.len());
    for se in &kind7 {
        match Nis::parse(se.hash(), lb.data(&data, se)) {
            Ok(nis) => {
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
            Err(e) => println!("script {:08x}: parse failed {e}", se.hash()),
        }
    }
}
