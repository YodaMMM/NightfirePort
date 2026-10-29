//! Probe [D]: the SP placements whose scripts are the lift (060009e9), the launch (0600090a),
//! the cinemas (06000631/32/33) and the exhaust (0600090c/060008bc/060008ba) — full params,
//! plus the world bounds of the lift objects obj[152..157] and the shuttle entities.
//! Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for name in ["07000016.bin"] {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let _ = LevelBin::parse(&data).unwrap();
        let level = Level::parse(&data).unwrap();
        let mi = level.main_map().unwrap();
        let m = &level.maps[mi];
        println!("--- SP/switch/trigger placements carrying the escape scripts (whole level) ---");
        for (i, p) in m.placements.iter().enumerate() {
            let kind = p.flags & 0xff;
            if !matches!(kind, 0xd9 | 0x29 | 0x31 | 0xdc | 0xdd | 0xec | 0xed | 0xee | 0xef) { continue; }
            let ps: Vec<u32> = p.params.iter().map(|&(_, v)| v).collect();
            let hit = ps.iter().any(|&v| matches!(v, 0x060009e9 | 0x0600090a | 0x06000631 | 0x06000632 | 0x06000633 | 0x0600090c | 0x060008bc | 0x060008ba | 80 | 0x50));
            if !hit { continue; }
            let name = m.objects.get(p.index as usize).map_or("", |o| o.name.as_str());
            println!("place[{i}] kind {kind:#04x} idx {} pos ({:.2},{:.2},{:.2}) name {name:?} params {:?} rot {:?}",
                p.index, p.translation[0], p.translation[1], p.translation[2], p.params, p.rotation);
        }
        println!("--- the lift + shuttle objects' own bounds ---");
        for (i, o) in m.objects.iter().enumerate() {
            if matches!(o.hashcode, 0x02000a79 | 0x02000a7a | 0x02000a7b | 0x02000a7c | 0x02000a89 | 0x02000a8a | 0x0200098f | 0x0200098c | 0x02000992 | 0x02000993) {
                println!("obj[{i}] hash {:08x} name {:?} centre ({:.2},{:.2},{:.2}) radius {:.2} min ({:.2},{:.2},{:.2}) max ({:.2},{:.2},{:.2})",
                    o.hashcode, o.name, o.centre[0], o.centre[1], o.centre[2], o.radius, o.min[0], o.min[1], o.min[2], o.max[0], o.max[1], o.max[2]);
            }
        }
    }
}
