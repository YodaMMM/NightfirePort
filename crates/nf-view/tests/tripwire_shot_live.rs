//! The tripwire objects on 0700000a / 0700000b / 07000014 are not the 0xFE mine class.
//!
//! The 0xFE placement kind is `Mine_Create` 0x17ce28's only creator [M
//! `parsemap_create_dynamic_objects` 0x1d04f0 case 0xfe, the only call site of Mine_Create in
//! ACTION.ELF]. The census below walks every map of the three levels: none carries a 0xFE
//! placement [D], so the laser tripbombs there are the class-5 stuck grenade the 0x3a weapon
//! places (07000014's tripbombs are 0xF0 pickups), whose shot-from-range rule is
//! `Bullet_handle_object_destruction` 0x12cd98 [M], not `Mine_Update` 0x17c930. The 0xFE leg
//! (a round from range detonates the mine) is pinned on 07000015 by walkreal_07000015_live.
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Every map's 0xFE placement count, per level archive.
fn mine_placements(iso: &IsoImage, lvl: &str) -> (usize, usize) {
    let (entries, bin) = disc::files_bin_entries(iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(lvl)).unwrap_or_else(|| panic!("{lvl} in FILES.BIN"));
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let n = level.maps.iter().map(|m| m.placements.iter().filter(|p| p.flags & 0xff == 0xfe).count()).sum();
    (level.maps.len(), n)
}

#[test]
fn the_tripwire_levels_carry_no_0xfe_mine() {
    let Some(iso_path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    for (lvl, maps) in [("0700000a.bin", 38), ("0700000b.bin", 51), ("07000014.bin", 55)] {
        let (n_maps, mines) = mine_placements(&iso, lvl);
        assert_eq!(n_maps, maps, "{lvl}: the census walks every map [D]");
        assert_eq!(mines, 0, "{lvl}: no 0xFE placement on any map [D]");
    }
    // Sister: the same count finds the 0xFE mines where they exist (the nine Tripbomb_3rd cels).
    let (_, mines15) = mine_placements(&iso, "07000015.bin");
    assert_eq!(mines15, 9, "sister: 07000015 carries the nine 0xFE mines [D census_07000015]");
}
