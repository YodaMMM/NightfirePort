//! pc-screen probe: every PCQWorm placement (type 0x35) on 0a/0b/11/12, in every archive map —
//! params, the resolved object, its glist flags, the block-0x27 LOD word (`celglist+0x40`, the
//! `Control_NextLOD` 0x1349b0 target, written only by `parsemap_block_LOD` 0x1d1f10) and the
//! per-map LOD link count.
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    for name in ["0700000a.bin", "0700000b.bin", "07000011.bin", "07000012.bin"] {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).expect("level");
        let data = image.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lvl = Level::parse(&data).expect("parse");
        let main = lvl.main_map().expect("main map");
        for (m, map) in lvl.maps.iter().enumerate() {
            let links = map.lod.iter().filter(|&&w| w >= 0).count();
            let pcs = map.placements.iter().filter(|p| p.flags & 0xff == 0x35).count();
            if links == 0 && pcs == 0 { continue; }
            println!("{name} map {m}{} '{}': {} objects, {links} lod links, {pcs} PCQWorm placements",
                if m == main { " (main)" } else { "" }, map.name, map.objects.len());
            for (k, &w) in map.lod.iter().enumerate() {
                if w >= 0 { println!("   link {k} {:?} -> {w} {:?}", map.objects[k].name, map.objects.get(w as usize).map(|o| o.name.clone())); }
            }
            for (pi, p) in map.placements.iter().enumerate() {
                if p.flags & 0xff != 0x35 { continue; }
                let ps: Vec<String> = p.params.iter().map(|(k, v)| format!("{k}:{v:#x}")).collect();
                let target = if p.by_hash() { lvl.by_hash.get(&p.hashcode).copied() } else { Some((m, p.index as usize)) };
                let desc = |mm: usize, oo: usize| {
                    let o = &lvl.maps[mm].objects[oo];
                    let b: usize = o.gfx.leaves.iter().map(|l| l.batches.len()).sum();
                    format!("map {mm} obj {oo} '{}' hash {:08x} flags {:#x} batches {b} centre {:?} r {}", o.name, o.hashcode, o.flags, o.centre, o.radius)
                };
                let (d0, d1) = match target {
                    Some((mm, oo)) => (desc(mm, oo), lvl.maps[mm].lod_target(oo).map(|t| desc(mm, t))
                        .unwrap_or_else(|| format!("none (word {:?})", lvl.maps[mm].lod.get(oo)))),
                    None => ("unresolved".into(), "n/a".into()),
                };
                println!("  #{pi} pos {:?} rot {:?} [{}]\n    {d0}\n    lod -> {d1}", p.translation, p.rotation, ps.join(" "));
            }
        }
    }
}
