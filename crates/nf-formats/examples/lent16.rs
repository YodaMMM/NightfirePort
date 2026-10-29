//! Probe [D]: objects 02000ab7/aba/ab8/994-997 (the launch script's entities) and any placement
//! carrying them, with bounds. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};
fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let _ = LevelBin::parse(&data).unwrap();
    let level = Level::parse(&data).unwrap();
    let m = &level.maps[level.main_map().unwrap()];
    let want = [0x02000ab7u32, 0x02000ab8, 0x02000aba, 0x02000994, 0x02000995, 0x02000996, 0x02000997, 0x020008b5];
    for (i, o) in m.objects.iter().enumerate() {
        if want.contains(&o.hashcode) {
            println!("obj[{i}] hash {:08x} name {:?} centre ({:.2},{:.2},{:.2}) radius {:.2} min ({:.2},{:.2},{:.2}) max ({:.2},{:.2},{:.2}) flags {:08x}",
                o.hashcode, o.name, o.centre[0], o.centre[1], o.centre[2], o.radius, o.min[0], o.min[1], o.min[2], o.max[0], o.max[1], o.max[2], o.flags);
        }
    }
    for (i, p) in m.placements.iter().enumerate() {
        if let Some(o) = m.objects.get(p.index as usize) {
            if want.contains(&o.hashcode) {
                println!("place[{i}] kind {:#04x} idx {} pos ({:.2},{:.2},{:.2}) hash {:08x} name {:?} params {:?}",
                    p.flags & 0xff, p.index, p.translation[0], p.translation[1], p.translation[2], o.hashcode, o.name, p.params);
            }
        }
    }
}
