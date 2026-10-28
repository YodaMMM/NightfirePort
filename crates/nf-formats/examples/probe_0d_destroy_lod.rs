//! level-0d-c5-door-laser probe: 0700000d's Destroy placements (0xe1) — params, resolved object,
//! the block-0x27 LOD word (the destroyed-glist source [M Destroy_Create 0x12c318:23-26]) and the
//! target's drawable state — plus the weapon category words at weapon_data+0x10 for the mounted
//! guns and a player gun (the Collide_FilterBullets 0x1ece30 test word) and the 0x67 flight floats.
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("0700000d.bin"))
        .expect("0700000d.bin in FILES.BIN");
    let data = image
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    let lvl = Level::parse(&data).expect("parse");
    let m = lvl.main_map().expect("main map");
    for (pi, p) in lvl.maps[m].placements.iter().enumerate() {
        if p.flags & 0xff != 0xe1 {
            continue;
        }
        let ps: Vec<String> = p.params.iter().map(|(k, v)| format!("{k}:{v:#x}")).collect();
        let target = if p.by_hash() { lvl.by_hash.get(&p.hashcode).copied() } else { Some((m, p.index as usize)) };
        let (objdesc, lodw, lodtarget) = match target {
            Some((mm, oo)) => {
                let o = &lvl.maps[mm].objects[oo];
                let lt = lvl.maps[mm].lod_target(oo);
                let drawable = lt
                    .map(|t| {
                        let to = &lvl.maps[mm].objects[t];
                        format!("obj {t} batches {}", to.gfx.leaves.iter().map(|l| l.batches.len()).sum::<usize>())
                    })
                    .unwrap_or_else(|| "none".into());
                (format!("obj {oo} hash {:08x} flags {:#x}", o.hashcode, o.flags), lvl.maps[mm].lod.get(oo).copied(), format!("{drawable}"))
            }
            None => ("unresolved".into(), None, "n/a".into()),
        };
        println!(
            "placement #{pi} params [{}] {objdesc} lod_word {lodw:?} lod_target {lodtarget}",
            ps.join(" ")
        );
    }
    // The Collide_FilterBullets 0x1ece30 test word: weapon_data[w]+0x10 (u16) — the mounted guns
    // and a spread of handheld guns [M 0x1ece30:26; the record layout nf-formats::weapon].
    let elf = std::fs::read("work/elf/ACTION.ELF").expect("work/elf/ACTION.ELF");
    let wd = nf_formats::weapon::WeaponData::from_action_elf(&elf).expect("weapon_data");
    for w in [0x01u8, 0x08, 0x1d, 0x2e, 0x33, 0x54, 0x61, 0x67, 0x70] {
        let r = &wd.records[w as usize];
        let word = u16::from_le_bytes([r[0x10], r[0x11]]);
        let dmg = f32::from_le_bytes(r[0x0c..0x10].try_into().unwrap());
        println!("weapon 0x{w:02x} category {word:#06x} damage {dmg}");
    }
    let r = &wd.records[0x67];
    let g32 = |o: usize| f32::from_le_bytes(r[o..o + 4].try_into().unwrap());
    println!("w0x67 range(+1c) {} speed(+20) {} spread(+24) {} dmg(+0c) {} word(+10) {:#06x}",
        g32(0x1c), g32(0x20), g32(0x24), g32(0x0c), u16::from_le_bytes([r[0x10], r[0x11]]));
}
