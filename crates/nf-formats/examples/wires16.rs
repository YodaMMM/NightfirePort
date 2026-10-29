//! Probe [D]: every Wire (0x3E) / Ladder (0x22) / Grapple (0x3A) / Hurt (0xE4) placement on
//! 07000016 — full level — plus every placement inside the launch-pit bbox
//! (x −12..18, y −28..2, z −196..−160) regardless of kind. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

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

    println!("--- ALL wire/ladder/grapple/hurt placements (whole level) ---");
    for (i, p) in m.placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        if !matches!(kind, 0x3e | 0x22 | 0x3a | 0xe4) { continue; }
        let pos = p.translation;
        println!("place[{i}] kind {kind:#04x} idx {} pos ({:.2},{:.2},{:.2}) hash {:08x} params {:?} rot {:?}",
            p.index, pos[0], pos[1], pos[2], p.hashcode, p.params, p.rotation);
        if let Some(o) = m.objects.get(p.index as usize) {
            println!("   obj[{}] hash {:08x} name {:?} centre {:?} radius {:.3} min {:?} max {:?}",
                p.index, o.hashcode, o.name, o.centre, o.radius, o.min, o.max);
        }
    }

    println!("--- EVERY placement in the pit bbox (x -12..18, y -28..2, z -196..-160) ---");
    for (i, p) in m.placements.iter().enumerate() {
        let pos = p.translation;
        if pos[0] < -12.0 || pos[0] > 18.0 || pos[1] < -28.0 || pos[1] > 2.0 || pos[2] < -196.0 || pos[2] > -160.0 { continue; }
        let kind = p.flags & 0xff;
        let name = m.objects.get(p.index as usize).map_or("", |o| o.name.as_str());
        println!("place[{i}] kind {kind:#04x} idx {} pos ({:.2},{:.2},{:.2}) hash {:08x} name {name:?} params {:?} rot {:?}",
            p.index, pos[0], pos[1], pos[2], p.hashcode, p.params, p.rotation);
    }

    println!("--- objects named wire/ladder/cable/rope/grapple (whole level) ---");
    for (i, o) in m.objects.iter().enumerate() {
        let n = o.name.to_ascii_lowercase();
        if n.contains("wire") || n.contains("ladder") || n.contains("cable") || n.contains("rope") || n.contains("grappl") {
            println!("obj[{i}] hash {:08x} name {:?} centre ({:.2},{:.2},{:.2}) radius {:.2}",
                o.hashcode, o.name, o.centre[0], o.centre[1], o.centre[2], o.radius);
        }
    }
}
