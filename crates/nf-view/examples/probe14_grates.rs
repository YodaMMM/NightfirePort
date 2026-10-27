// level-14-vent-grates scratch: identify the vent grate placements in 07000014 — every
// placement whose object name matches the grate/vent/duct family, plus every placement within
// 6 units of the two mouth view points' sight lines. Assertion-free.
use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn dist2(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let level = Level::parse(&data).expect("parse 07000014");

    // The two mouth view points from the driver.
    let views: [[f32; 3]; 2] = [[32.61, 2.80, 45.27], [56.03, 0.65, 46.89]];

    println!("== A. placements whose object name matches the grate/vent/duct family ==");
    for (m, map) in level.maps.iter().enumerate() {
        for (p, pl) in map.placements.iter().enumerate() {
            let Some(&(om, oo)) = level.by_hash.get(&pl.hashcode) else { continue };
            let obj = &level.maps[om].objects[oo];
            let n = obj.name.to_ascii_uppercase();
            if n.contains("GRATE") || n.contains("VENT") || n.contains("DUCT")
                || n.contains("GRILL") || n.contains("MOUTH") || n.contains("FAN") {
                println!(
                    "map {m} pl {p}: hash {:08x} name {:?} flags {:#x} trans ({:.2}, {:.2}, {:.2}) rot ({:.3}, {:.3}, {:.3})",
                    pl.hashcode, obj.name, pl.flags, pl.translation[0], pl.translation[1], pl.translation[2],
                    pl.rotation[0], pl.rotation[1], pl.rotation[2]
                );
            }
        }
    }

    println!("== B. every placement within 6.0 of a view point ==");
    for (vi, v) in views.iter().enumerate() {
        println!("-- view {vi} {v:?} --");
        for (m, map) in level.maps.iter().enumerate() {
            for (p, pl) in map.placements.iter().enumerate() {
                if dist2(pl.translation, *v) > 36.0 { continue; }
                let name = level.by_hash.get(&pl.hashcode)
                    .map(|&(om, oo)| level.maps[om].objects[oo].name.clone())
                    .unwrap_or_else(|| "?".into());
                println!(
                    "map {m} pl {p}: hash {:08x} name {name:?} trans ({:.2}, {:.2}, {:.2}) rot ({:.3}, {:.3}, {:.3})",
                    pl.hashcode, pl.translation[0], pl.translation[1], pl.translation[2],
                    pl.rotation[0], pl.rotation[1], pl.rotation[2]
                );
            }
        }
    }

    println!("== C. distinct object names containing GRATE/VENT anywhere in the archive ==");
    let mut seen = std::collections::BTreeSet::new();
    for map in level.maps.iter() {
        for obj in map.objects.iter() {
            let n = obj.name.to_ascii_uppercase();
            if (n.contains("GRATE") || n.contains("VENT")) && seen.insert(obj.name.clone()) {
                println!("  {:?} (hash {:08x}, collision {})", obj.name, obj.hashcode,
                    obj.collision.as_ref().map_or(0, |c| c.triangles().len()));
            }
        }
    }
}
