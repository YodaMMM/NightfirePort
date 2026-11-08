//! The 07000008 shutter leaves' collision surface bytes (HITDATA+0x50's source): does the
//! wood/glass carry the node pass bit 0x40? [D] evidence for aim-node-pass.
//! Opt-in via NF_ISO. Prints; asserts nothing.
use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, levelbin, map, level};

#[test]
fn probe_shutter_bytes() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let (files, bin) = disc::files_bin_entries(&iso).unwrap();
    let f = files.iter().find(|f| f.name.to_ascii_uppercase() == "07000008.BIN").unwrap();
    let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64).unwrap();
    let lvl = level::Level::parse(&data).unwrap();
    let m = lvl.main_map().unwrap();
    for o in [45usize, 46, 47] {
        let ob = &lvl.maps[m].objects[o];
        let Some(c) = &ob.collision else { eprintln!("obj[{o}] '{}' flags {:#x} NO collision", ob.name, ob.flags); continue };
        let mut hist: std::collections::BTreeMap<u8, usize> = Default::default();
        for &b in &c.materials { *hist.entry(b).or_default() += 1; }
        eprintln!("obj[{o}] '{}' flags {:#x} tris {} mats {:?}", ob.name, ob.flags, c.tris.len(), hist);
    }
}
