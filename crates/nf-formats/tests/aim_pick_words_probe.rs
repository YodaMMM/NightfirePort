//! The aim pick's disc-side inputs on every level: the glist flags word's 0x10000000 /
//! 0x20000000 bits (Collide_Pick 0x1eb320's cel walk skips `obj+0xF0 & 0x30000000`; those
//! objects are handed to the ForcedList walk by build_LinkToRoom 0x1c9680:62-65 and
//! Controls_StraddleTest 0x134778, which picks them anyway), and the collision surface bytes
//! carrying 0x80 (Collide_Intersect 0x1ebaa8 ORs them into the pass bit unconditionally: the
//! `movn` at 0x1ebf94 sits in the branch delay slot). Opt-in via NF_ISO.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level};

#[test]
fn aim_pick_words_on_every_level() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    let (files, bin) = disc::files_bin_entries(&iso).unwrap();
    let (mut levels, mut objs, mut cel_only, mut forced, mut b80) = (0, 0, 0, 0, 0usize);
    for f in &files {
        let n = f.name.to_ascii_uppercase();
        if !(n.starts_with("070000") && n.ends_with(".BIN")) { continue; }
        let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64).unwrap();
        let Ok(lvl) = level::Level::parse(&data) else { continue };
        levels += 1;
        for m in &lvl.maps {
            for ob in &m.objects {
                objs += 1;
                // 0x10000000 without 0x20000000: a cel-linked object the cel walk skips and no
                // ForcedList insert covers — the only word the pre-filter could drop from the pick.
                if ob.flags & 0x3000_0000 == 0x1000_0000 { cel_only += 1; eprintln!("{n} cel-only {:#x} '{}'", ob.flags, ob.name); }
                if ob.flags & 0x2000_0000 != 0 { forced += 1; }
                if let Some(c) = &ob.collision { b80 += c.materials.iter().filter(|&&b| b & 0x80 != 0).count(); }
            }
        }
    }
    eprintln!("[aim-words] {levels} levels, {objs} objects: {cel_only} with 0x10000000 alone, {forced} with 0x20000000, {b80} surface bytes with 0x80");
    assert!(levels > 20);
    assert_eq!(cel_only, 0, "no disc word can make the cel walk's pre-filter drop an object from the pick");
}
