//! Probe: every 07000016 main-map placement inside the launch-pit bbox
//! (x −35..40, y −28..3, z −200..−128), with its kind, so the pit's ladders (0x22), creepwalls,
//! doors, lifts and triggers are visible; then the ladder/door/rim objects' world bounds.
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

fn dump_object(level: &Level, place_idx: usize, label: &str) {
    let mi = level.main_map().unwrap();
    let m = &level.maps[mi];
    let Some(p) = m.placements.get(place_idx) else { return };
    println!("== {label} place[{place_idx}] kind {:#04x} index {} pos {:?}", p.flags & 0xff, p.index, p.translation);
    if let Some(o) = m.objects.get(p.index as usize) {
        println!("   obj[{}] hash {:08x} name {:?} centre {:?} radius {:.3} min {:?} max {:?} collision_tris {}",
            p.index, o.hashcode, o.name, o.centre, o.radius, o.min, o.max,
            o.collision.as_ref().map_or(0, |c| c.triangles().len()));
        if let Some(c) = o.collision.as_ref() {
            let tris = c.triangles();
            let mut lo = [f32::MAX; 3]; let mut hi = [-f32::MAX; 3];
            for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
            println!("   collision tris {} lo({:.2},{:.2},{:.2}) hi({:.2},{:.2},{:.2})", tris.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        }
    }
}

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let _ = LevelBin::parse(&data).unwrap();
    let level = Level::parse(&data).expect("level parses");
    let mi = level.main_map().expect("main map");
    let m = &level.maps[mi];
    let mut n = 0;
    for (i, p) in m.placements.iter().enumerate() {
        let pos = p.translation;
        if pos[0] < -35.0 || pos[0] > 40.0 || pos[1] < -28.0 || pos[1] > 3.0 || pos[2] < -200.0 || pos[2] > -128.0 { continue; }
        let kind = p.flags & 0xff;
        println!("place[{i}] kind {kind:#04x} idx {} pos ({:.2},{:.2},{:.2}) hash {:08x} params {:?} rot {:?}",
            p.index, pos[0], pos[1], pos[2], p.hashcode, p.params, p.rotation);
        n += 1;
    }
    println!("--- placements in pit bbox: {n}");
    for (i, o) in m.objects.iter().enumerate() {
        let c = o.centre;
        if c[0] < -35.0 || c[0] > 40.0 || c[1] < -26.0 || c[1] > 2.0 || c[2] < -200.0 || c[2] > -128.0 { continue; }
        println!("obj[{i}] hash {:08x} name {:?} centre ({:.2},{:.2},{:.2}) radius {:.2} min ({:.2},{:.2},{:.2}) max ({:.2},{:.2},{:.2})",
            o.hashcode, o.name, c[0], c[1], c[2], o.radius, o.min[0], o.min[1], o.min[2], o.max[0], o.max[1], o.max[2]);
    }
    for (idx, label) in [(256usize, "pit-ladder"), (267usize, "pit-door-ish-a"), (268usize, "pit-door-ish-b")] {
        dump_object(&level, idx, label);
    }
}
