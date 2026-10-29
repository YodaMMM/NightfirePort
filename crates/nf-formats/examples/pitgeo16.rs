//! Probe [D]: the launch-pit escape geometry on 07000016.
//! 1) The lift script 060009e9's keys per entity segment (positions over time).
//! 2) Every named object whose WORLD bbox intersects the pit column
//!    (x -12..18, y -25..2, z -196..-160): name, kind, world bbox.
//! 3) The placements carrying hashes 02000a79/a7a/a7b/a7c/a89/a8a (the lift entities).
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin, nis::Nis};

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();

    // --- the lift script keys ---
    for name in ["07000016.bin"] {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let lb = LevelBin::parse(&data).unwrap();
        let e = lb.entries.iter().find(|e| e.kind == 7 && e.name.eq_ignore_ascii_case("060009e9")).expect("lift script");
        let n = Nis::parse(0x060009e9, lb.data(&data, e)).unwrap();
        println!("=== lift script 060009e9: {} keys total; streams {}", n.keys.len(), n.streams.len());
        for (ki, k) in n.keys.iter().enumerate() {
            println!("  key[{ki}] pos ({:.2},{:.2},{:.2},{:.2}) time {:.1} scale {:.2}", k.pos[0], k.pos[1], k.pos[2], k.pos[3], k.time, k.scale);
        }
        // which key indices each stream references is not decoded here; positions above tell the path.

        // --- objects intersecting the pit column ---
        let level = Level::parse(&data).unwrap();
        let mi = level.main_map().unwrap();
        let m = &level.maps[mi];
        println!("--- objects whose world bbox hits the pit column (x -12..18, y -25..2, z -196..-160) ---");
        for (i, o) in m.objects.iter().enumerate() {
            // world bbox = centre +- (max-min)/2-ish: use min/max as LOCAL extents around centre
            let lo = [o.centre[0] + o.min[0], o.centre[1] + o.min[1], o.centre[2] + o.min[2]];
            let hi = [o.centre[0] + o.max[0], o.centre[1] + o.max[1], o.centre[2] + o.max[2]];
            let hit = hi[0] >= -12.0 && lo[0] <= 18.0 && hi[1] >= -25.0 && lo[1] <= 2.0 && hi[2] >= -196.0 && lo[2] <= -160.0;
            if !hit { continue; }
            println!("obj[{i}] hash {:08x} name {:?} centre ({:.2},{:.2},{:.2}) world lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})",
                o.hashcode, o.name, o.centre[0], o.centre[1], o.centre[2], lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        }
        // --- the lift-entity placements ---
        println!("--- placements of the lift entities ---");
        for (i, p) in m.placements.iter().enumerate() {
            let Some(o) = m.objects.get(p.index as usize) else { continue };
            if !matches!(o.hashcode, 0x02000a79 | 0x02000a7a | 0x02000a7b | 0x02000a7c | 0x02000a89 | 0x02000a8a) { continue; }
            println!("place[{i}] kind {:#04x} idx {} pos ({:.2},{:.2},{:.2}) hash {:08x} name {:?} params {:?} rot {:?}",
                p.flags & 0xff, p.index, p.translation[0], p.translation[1], p.translation[2], o.hashcode, o.name, p.params, p.rotation);
        }
    }
}
