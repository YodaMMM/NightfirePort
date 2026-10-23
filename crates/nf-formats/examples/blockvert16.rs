//! Probe: the blast-room blocking polys' raw quad vertices (the ramp
//! direction), and the hurt #309 planes' solidity check around the re-enable spot.
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let _ = LevelBin::parse(&data).unwrap();
    let level = Level::parse(&data).expect("level parses");
    let mi = level.main_map().unwrap();
    let m = &level.maps[mi];
    for oi in [117usize, 118] {
        let o = &m.objects[oi];
        println!("== obj[{oi}] {} — quad vertices:", o.name);
        if let Some(c) = o.collision.as_ref() {
            for (ti, t) in c.triangles().iter().enumerate() {
                println!("   tri{ti}: {:?}", t);
            }
        }
    }
    // The pit floor under and around the re-enable spot: sample a floor ray through the world.
    // (nf-formats has no BVH walk here; instead print the grates' planes again + Silo_grate tris.)
    for oi in [119usize, 120, 121] {
        let o = &m.objects[oi];
        if let Some(c) = o.collision.as_ref() {
            let t = &c.triangles()[0];
            println!("== obj[{oi}] {} tri0 {:?}", o.name, t);
        }
    }
}
