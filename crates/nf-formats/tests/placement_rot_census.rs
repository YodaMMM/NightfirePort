//! Disc census [D]: every placement's rotation angles per class (the low byte of the flags word),
//! over every map of every level `.bin` on the disc. Counts rows with a non-zero rotation-x or
//! rotation-z (pitch/roll) — the rows where the two game matrix builders, `RotMatrix` 0x1e2368
//! (X·Y·Z) and `View_RotTransMatrix` 0x1e5168 (Y·Z·X), disagree — so a per-class builder switch
//! knows its reach. Informational: asserts nothing about the data.
//!
//! Run: `NF_ISO=… cargo test -p nf-formats --test placement_rot_census -- --nocapture`

use std::collections::BTreeMap;

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn nonzero(v: f32) -> bool {
    v.to_bits() & 0x7fff_ffff != 0
}

#[test]
fn census_placement_rotations_per_class() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let im = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    // class -> [total, creates_cel, rx != 0, rz != 0]
    let mut per: BTreeMap<u32, [u32; 4]> = BTreeMap::new();
    let mut levels = 0u32;
    let mut maps = 0u32;
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = im
            .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
            .unwrap();
        let Ok(lv) = Level::parse(&data) else { continue };
        levels += 1;
        for m in &lv.maps {
            maps += 1;
            for p in &m.placements {
                let c = per.entry(p.flags & 0xff).or_default();
                c[0] += 1;
                c[1] += p.creates_cel() as u32;
                c[2] += nonzero(p.rotation[0]) as u32;
                c[3] += nonzero(p.rotation[2]) as u32;
            }
        }
    }
    println!("levels parsed: {levels}, maps: {maps}");
    println!("class\ttotal\tcreates_cel\trx!=0\trz!=0");
    for (class, c) in &per {
        println!("{class:#04x}\t{}\t{}\t{}\t{}", c[0], c[1], c[2], c[3]);
    }
}
