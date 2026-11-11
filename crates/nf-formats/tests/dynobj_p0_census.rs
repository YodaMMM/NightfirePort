//! The [D] census behind dynobj-hidden-skip: every main-map placement kind 0x3b
//! (`DynamicObject_Create` 0x18dbf8) on the disc, its p0 (level_tag+0x2C — the create reads it as
//! a SHORT [M 0x18dbf8:24,30-32]) and the resolved object's glist word (celglist+0x44 =
//! MapObject.flags [M Control_SetGList 0x134960:11]).
//!
//! p0 != 0 makes the create OR 0x30 into obj+0xF0 [M 0x18dbf8:30-32]; 0x20 is in every
//! `Collide_PickObj` skip set [M 0x1eae80:72-82], so the game never picks such an object for any
//! ray query. A short-read disagreement row (p0 nonzero as u32, zero as u16) is a level where the
//! port's u32 condition would diverge from the game's short read.
//!
//! Opt-in via NF_ISO. Prints; asserts nothing.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level};

#[test]
fn census_dynobj_p0() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let mut total = 0usize;
    let mut hidden = 0usize;
    let mut short_disagree = 0usize;
    let mut sub_map_hidden = 0usize;
    for fe in &entries {
        let lower = fe.name.to_ascii_lowercase();
        if !lower.starts_with("07") || !lower.ends_with(".bin") { continue; }
        let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(lvl) = level::Level::parse(&data) else { continue };
        let Some(main) = lvl.main_map() else { continue };
        for (mi, m) in lvl.maps.iter().enumerate() {
            if mi != main && mi != 0 { continue; } // the main map's placements are what the port creates; note sub-map rows separately
            let is_main = mi == main;
            for (pi, p) in m.placements.iter().enumerate() {
                if p.flags & 0xff != 0x3b { continue; }
                let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
                let obj = if p.by_hash() { lvl.by_hash.get(&p.hashcode).map(|&(mm, oo)| &lvl.maps[mm].objects[oo]) }
                    else { m.objects.get(p.index as usize) };
                let (word, oname, ohash) = obj.map(|o| (o.flags, o.name.clone(), o.hashcode)).unwrap_or((0, "<none>".into(), 0));
                let p0 = param(0);
                let hid = p0 != 0;
                if is_main {
                    total += 1;
                    if hid { hidden += 1; }
                    if hid && (p0 as u16) == 0 { short_disagree += 1; }
                } else if hid { sub_map_hidden += 1; }
                eprintln!("{:14} {} pl {:3} at ({:8.2},{:8.2},{:8.2}) p0 {:#010x} (short {:#06x}) p1 {} p2 {} p3 {} cel {} obj {:#x} '{}' word {:#x} w&0x10 {} w&0x20 {} OR0x30 {}",
                    fe.name, if is_main { "main" } else { "sub " }, pi,
                    p.translation[0], p.translation[1], p.translation[2], p0, p0 as u16, param(1), param(2), param(3),
                    p.creates_cel(), ohash, oname, word, word & 0x10 != 0, word & 0x20 != 0, hid);
            }
        }
    }
    eprintln!("TOTAL main-map 0x3b placements {total}, p0 != 0 (u32) {hidden}, u16-short disagreements {short_disagree}, sub-map p0 != 0 {sub_map_hidden}");
}
