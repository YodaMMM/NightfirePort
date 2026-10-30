//! walkreal probe #9 (level-0c-gate-band-crossing): the mover census. Dumps (1) every object
//! whose name mentions turb/walk/gate/bridge/platform/lift with its placements and raw tri
//! counts, (2) every 0xe2 SS and 0x3b DynamicObject and 0xe1 Destroy placement in the wide box
//! x 30..95 z −225..−185 with key counts and channels, (3) the same for ANY map (not just main),
//! (4) resolves the scripts' Entity glist hashes to object names. Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_band_mover_census() {
    let iso = iso();
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = LevelBin::parse(&data).expect("levelbin");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    // 1. Name census.
    eprintln!("=== objects named *turb*/*walk*/*gate*/*bridge*/*platform*/*lift* ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (oi, o) in m.objects.iter().enumerate() {
            let n = o.name.trim().to_lowercase();
            if !(n.contains("turb") || n.contains("walk") || n.contains("gate") || n.contains("bridge") || n.contains("platform") || n.contains("lift")) { continue; }
            let plcs: Vec<String> = (0..2).map(|_| String::new()).collect();
            let _ = plcs;
            let mut rows = Vec::new();
            for (pi, p) in m.placements.iter().enumerate() {
                let hit = if p.by_hash() { p.hashcode == o.hashcode } else { p.index as usize == oi };
                if hit { rows.push(format!("plc{pi} cel {} t ({:.2},{:.2},{:.2})", p.creates_cel(), p.translation[0], p.translation[1], p.translation[2])); }
            }
            eprintln!("map{mi} obj{oi} \"{n}\" hash {:#010x} tris {} keys? {}", o.hashcode, o.collision.as_ref().map_or(0, |c| c.triangles().len()), rows.join(" | "));
        }
    }

    // 2. SS / DynamicObject / Destroy / Gondola-ish placements in the wide box.
    eprintln!("=== 0xe2/0x3b/0xe1/0xdb placements wide box x 30..95 z -225..-185 ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let class = p.flags & 0xff;
            if !matches!(class, 0xe2 | 0x3b | 0xe1 | 0xdb) { continue; }
            let t = p.translation;
            if !(t[0] >= 30.0 && t[0] <= 95.0 && t[2] >= -225.0 && t[2] <= -185.0) { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(om, oi)| level.maps[om].objects[oi].name.trim().to_string()).unwrap_or("<unresolved>".into());
            eprintln!("map{mi} plc{pi} class {class:#04x} \"{name}\" keys {:?} t ({:.2},{:.2},{:.2}) params {:?}", p.extra.iter().map(|w| w & 0xffff).collect::<Vec<_>>(), t[0], t[1], t[2], p.params);
        }
    }

    // 3. The scripts' Entity glist hashes -> object names (all maps, by hash).
    eprintln!("=== script Entity glists resolved ===");
    let mut glist_names: std::collections::BTreeMap<u32, Vec<String>> = Default::default();
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = nf_formats::nis::Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        let mut spawns: Vec<String> = Vec::new();
        for (si, s) in nis.streams.iter().enumerate() {
            for c in s.decode() {
                if let nf_formats::nis::Cmd::Entity { hash, obj_type, params, .. } = c {
                    let name = level.by_hash.get(&hash).map(|&(m, o)| format!("map{m} obj{o} \"{}\"", level.maps[m].objects[o].name.trim()))
                        .unwrap_or(format!("<unknown {hash:08x}>"));
                    spawns.push(format!("s{si} type {obj_type:#04x} {hash:08x} -> {name} params {params:?}"));
                    glist_names.entry(hash).or_default().push(format!("{:08x}", nis.hashcode));
                }
            }
        }
        if !spawns.is_empty() {
            eprintln!("== script {:08x} ==", nis.hashcode);
            for s in spawns { eprintln!("  {s}"); }
        }
    }
    eprintln!("ok");
}
