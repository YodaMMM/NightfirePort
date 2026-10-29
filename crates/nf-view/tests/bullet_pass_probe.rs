//! level-11-stairwell-bullet-wall-2 probe: the gondola window panes' [D] collision material
//! bytes (the surface class the game's pass rule reads) and the 07000001 corridor objects the
//! two red walks shoot at. Opt-in via NF_ISO, prints only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect(archive);
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_gondola_pane_materials() {
    let Some(iso) = std::env::var_os("NF_ISO").map(|s| s.into_string().unwrap()) else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000008.bin");
    let lb = LevelBin::parse(&data).expect("parse");
    let e = lb.entries.iter().find(|e| e.kind == 7 && e.hash() == 0x0600_004c).expect("the ride script");
    let n = nis::Nis::parse(e.hash(), lb.data(&data, e)).expect("parse nis");
    let mut hashes = std::collections::BTreeSet::new();
    for s in &n.streams {
        for cmd in s.decode() {
            if let nis::Cmd::Entity { hash, obj_type: 0x20, .. } = cmd { hashes.insert(hash); }
        }
    }
    eprintln!("pane glists: {hashes:?}");
    // The objects: kind-2/3 entries whose NAME hash matches; the collision block 0x2E sits
    // before the object in the archive stream (the Level parse pairs them) — use Level::parse.
    let lv = Level::parse(&data).expect("parse level");
    for h in &hashes {
        let mut found = None;
        for (mi, m) in lv.maps.iter().enumerate() {
            for (oi, o) in m.objects.iter().enumerate() {
                let name_hash = {
                    let last = o.name.rsplit(['\\', '/']).next().unwrap_or("");
                    last.chars().take(8).map_while(|c| c.to_digit(16)).fold(0u32, |acc, d| acc << 4 | d)
                };
                if name_hash == *h { found = Some((mi, oi)); break; }
            }
            if found.is_some() { break; }
        }
        match found {
            Some((mi, oi)) => {
                let o = &lv.maps[mi].objects[oi];
                let mut mats = o.collision.as_ref().map(|c| c.materials.clone()).unwrap_or_default();
                mats.sort_unstable();
                mats.dedup();
                let mut classes: Vec<u8> = mats.iter().map(|m| m & 0x3f).collect();
                classes.sort_unstable(); classes.dedup();
                eprintln!("pane {h:08x} \"{}\": collision mats {mats:?} (classes &0x3f {classes:?}) pass40 {}",
                    o.name.trim(), mats.iter().any(|m| m & 0x40 != 0));
            }
            None => eprintln!("pane {h:08x}: NO OBJECT FOUND"),
        }
    }
    // The carriage too (0x0200038a) for the record.
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_07000001_shoot_targets() {
    let Some(iso) = std::env::var_os("NF_ISO").map(|s| s.into_string().unwrap()) else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000001.bin");
    let lv = Level::parse(&data).expect("parse level");
    // The objects whose collision materials carry 0x40 or class 0x10 near the spawn corridor —
    // and every breakable/glass-family object on the level, with their material class census.
    for (mi, m) in lv.maps.iter().enumerate() {
        for (oi, o) in m.objects.iter().enumerate() {
            let Some(c) = &o.collision else { continue };
            let mut classes: Vec<u8> = c.materials.iter().map(|mm| mm & 0x3f).collect();
            classes.sort_unstable(); classes.dedup();
            let pass40 = c.materials.iter().any(|mm| mm & 0x40 != 0);
            if pass40 || classes.contains(&0x10) {
                eprintln!("m{mi} obj {oi} \"{}\" gflags {:#x} classes {classes:?} pass40 {pass40}",
                    o.name.trim(), o.flags);
            }
        }
    }
}
