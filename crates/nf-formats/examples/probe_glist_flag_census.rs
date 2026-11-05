//! trigger-glist-extents census probe (prints only): every trigger placement's target-glist
//! flags, per level. The game's raw-refine discriminator is bit 0x40 of the target celglist's
//! flags word (obj+0xf0, copied from celglist+0x44 [M Trigger_Create 0x1b1720:95; Collide_Intersect
//! 0x1ebaa8:93-99 — `*(uint *)(obj+0xf0) & 0x40` → HITDATA+0x51 |= 2]). Bucket the flags so the
//! port's flags==0x645 stand-in (triggers.rs seg_beam) can be measured against the game's bit.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn load(img: &IsoImage, name: &str) -> Level {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).expect("FILES.BIN");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("no {name}"));
    let data = img
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    Level::parse(&data).expect("parse")
}

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    for file in ["07000001.bin", "07000002.bin", "07000003.bin", "07000004.bin", "07000005.bin",
                 "07000006.bin", "07000007.bin", "07000008.bin", "07000009.bin", "0700000a.bin",
                 "0700000b.bin", "0700000c.bin", "0700000d.bin", "07000011.bin", "07000012.bin",
                 "07000013.bin", "07000014.bin", "07000015.bin", "0700001b.bin", "07000041.bin",
                 "0700004a.bin"] {
        let lv = load(&img, file);
        let Some(mm) = lv.main_map() else { continue };
        let mut buckets: std::collections::BTreeMap<u32, usize> = Default::default();
        let mut no_target = 0usize;
        let mut total = 0usize;
        for p in lv.maps[mm].placements.iter() {
            let k = p.flags & 0xff;
            if !matches!(k, 0xea | 0xeb | 0xdc | 0xec | 0xed | 0xee | 0xef) { continue; }
            total += 1;
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            match target.and_then(|(om, oi)| lv.maps[om].objects.get(oi)) {
                Some(o) => { *buckets.entry(o.flags).or_default() += 1; }
                None => no_target += 1,
            }
        }
        let line: String = buckets.iter().map(|(f, n)| format!("0x{f:x}×{n}")).collect::<Vec<_>>().join(" ");
        println!("CENSUS {file}: {total} trigger placements; flags: {line}; no-target {no_target}");
    }
}
