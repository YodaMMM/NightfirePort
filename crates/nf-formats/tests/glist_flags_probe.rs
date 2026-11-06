//! The glist flags word (celglist+0x44) per placement-resolved object, for the aim-object-flags
//! task: the disc evidence that `MapObject.flags` (entity-params payload +4 [M level.rs; the
//! game's parsemap_block_entity_params]) carries the word `Control_SetGList` 0x134960:11 copies
//! into obj+0xF0, and that it disagrees with the placement's own flags word (the stand-in
//! `Door.bullet_pass` read before the task).
//!
//! Live anchors [L the 0700001b audits]: the solar panels read obj+0xF0 = 0x81, the shutter
//! pieces 0xc1, debris 0xa8a0. This probe prints the disc words next to them.
//!
//! Opt-in via NF_ISO. Prints; asserts nothing.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, levelbin, map, placement, level};

fn main_path(iso: &IsoImage<std::fs::File>, name: &str) -> Vec<u8> {
    let (files, bin) = disc::files_bin_entries(iso).unwrap();
    let f = files.iter().find(|f| f.name.to_ascii_uppercase() == name.to_ascii_uppercase())
        .unwrap_or_else(|| panic!("{name} not in FILES.BIN"));
    iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64).unwrap()
}

#[test]
fn probe_glist_flags() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();

    for name in ["07000014.bin", "0700001b.bin", "07000008.bin"] {
        let data = main_path(&iso, name);
        let lvl = level::Level::parse(&data).unwrap();
        let m = lvl.main_map().expect("main map");
        eprintln!("== {name} main map {m}: {} objects", lvl.maps[m].objects.len());
        // Doors: placement kind 0xdb (the Doors::create filter), the placement word vs the object word.
        for (pi, p) in lvl.maps[m].placements.iter().enumerate() {
            if p.flags & 0xff != 0xdb || p.flags & 0xa000 != 0 { continue; }
            let obj = if p.by_hash() { lvl.by_hash.get(&p.hashcode).map(|&(mm, oo)| &lvl.maps[mm].objects[oo]) }
                else { lvl.maps[m].objects.get(p.index as usize) };
            let (hash, ow, oname, oi) = obj.map(|o| (o.hashcode, o.flags, o.name.clone(), p.index as usize)).unwrap_or((0, 0, "<none>".into(), usize::MAX));
            let _ = oi;
            eprintln!("door pl {pi} at ({:.2},{:.2},{:.2}) by_hash {} hash {:#x} idx {} -> {:#x} '{oname}' glist_flags {:#x} f0&0x60 {} (pl {}) (gl {})",
                p.translation[0], p.translation[1], p.translation[2], p.by_hash(), p.hashcode, p.index, hash, ow,
                ow & 0x60 != 0, p.flags & 0x60 != 0, ow & 0x60 != 0);
        }
        // The pass-capable words' names on every probed level.
        for (o, ob) in lvl.maps[m].objects.iter().enumerate() {
            if matches!(ob.flags & 0xffff, 0x30 | 0x50 | 0xc1 | 0x20 | 0x81 | 0x21 | 0x221) && ob.name.to_uppercase().contains("SHUT") || ob.flags & 0x60 != 0 {
                eprintln!("PASS? obj[{o}] hash {:#x} '{}' flags {:#x}", ob.hashcode, ob.name, ob.flags);
            }
        }
        // The 14 door object and neighbours: centre/radius for the sphere arithmetic.
        if name == "07000014.bin" {
            for (o, ob) in lvl.maps[m].objects.iter().enumerate() {
                if o > 250 && o < 290 && (ob.name.contains("Door") || ob.name.contains("door") || ob.name.contains("Wire")) {
                    eprintln!("obj[{o}] hash {:#x} '{}' flags {:#x} centre {:?} r {}", ob.hashcode, ob.name, ob.flags, ob.centre, ob.radius);
                }
            }
        }
        // Named objects of interest.
        for (o, ob) in lvl.maps[m].objects.iter().enumerate() {
            let hit = match name.to_ascii_uppercase().as_str() {
                "0700001B.BIN" => ob.hashcode == 0x2000a42 || ob.flags == 0xc1 || ob.flags == 0xa8a0,
                "07000014.BIN" => ob.hashcode == 0x2000aa1 || ob.hashcode == 0x2000aa4 || ob.hashcode == 0xffffffff && ob.name == "OBJECT_Door",
                "07000008.BIN" => (ob.flags & 0x60) != 0 && ob.flags & 0xff0000 == 0x02000000,
                _ => false,
            };
            if hit {
                eprintln!("obj {o} hash {:#x} name {} glist_flags {:#x} centre {:?} radius {}", ob.hashcode, ob.name, ob.flags, ob.centre, ob.radius);
            }
        }
        // Every distinct glist flags word on the main map (the shape of the word population).
        let mut words: Vec<(u32, usize)> = Vec::new();
        for ob in &lvl.maps[m].objects {
            match words.iter_mut().find(|(w, _)| *w == ob.flags) { Some((_, n)) => *n += 1, None => words.push((ob.flags, 1)) }
        }
        words.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        eprintln!("distinct glist words: {:?}", words.iter().map(|(w, n)| (format!("{w:#x}"), *n)).take(14).collect::<Vec<_>>());
        let _ = placement::parse; // silence unused if the import goes stale
    }
}
