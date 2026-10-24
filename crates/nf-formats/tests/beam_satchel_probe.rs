//! SCRATCH probe (camera-beam-satchel-draw): dumps the draw-side facts for the three items.
//! Opt-in via NF_ISO. Prints; asserts nothing.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn load(iso: &IsoImage, name: &str) -> Level {
    let (entries, bin) = disc::files_bin_entries(iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap()
}

#[test]
fn probe() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&path).unwrap();
    let mut hashes: Vec<u32> = vec![0x0200_0039, 0x0200_07b3];
    hashes.extend(0x0200_0791..=0x0200_079a);
    for name in ["0700000a.bin", "07000014.bin", "0700004a.bin"] {
        let level = load(&iso, name);
        let m = level.main_map().unwrap();
        println!("=== {name} ===");
        for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x28) {
            let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v);
            println!(
                "sensor #{}, hash {:?} flags {:?} pos {:?} params {:?}",
                p.index, p.hashcode, p.flags, p.translation, p.params
            );
            let lens = param(0);
            let mount = param(7);
            if let Some(h) = lens { if !hashes.contains(&h) { hashes.push(h); } }
            if let Some(h) = mount { if !hashes.contains(&h) { hashes.push(h); } }
        }
        for h in &hashes {
            match level.by_hash.get(h).copied() {
                Some((om, oi)) => {
                    let o = &level.maps[om].objects[oi];
                    let has_gfx = o.gfx.leaves.iter().any(|l| !l.batches.is_empty());
                    println!(
                        "  hash {h:#010x}: map {om} obj {oi} name {:?} flags {:#010x} centre {:?} r {} min {:?} max {:?} gfx {}",
                        o.name, o.flags, o.centre, o.radius, o.min, o.max, has_gfx
                    );
                }
                None => println!("  hash {h:#010x}: UNRESOLVED"),
            }
        }
    }
}
