//! Probe [D]: the 07000016 water objects' flag words (the 0xC023 swim-cel flag [M
//! parseentity_fixup_entity 0x1d00c0]) and their exact world bboxes; plus every object with the
//! flag anywhere on the level. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let _ = LevelBin::parse(&data).unwrap();
    let level = Level::parse(&data).unwrap();
    let mi = level.main_map().unwrap();
    let m = &level.maps[mi];
    println!("--- every object; name/flags/bbox (flag filter: 0xC023 or name water) ---");
    for (i, o) in m.objects.iter().enumerate() {
        let is_water_name = o.name.to_ascii_lowercase().contains("water");
        if !is_water_name && o.flags != 0xC023 { continue; }
        println!("obj[{i}] hash {:08x} name {:?} flags {:08x} centre ({:.2},{:.2},{:.2}) min ({:.2},{:.2},{:.2}) max ({:.2},{:.2},{:.2})",
            o.hashcode, o.name, o.flags, o.centre[0], o.centre[1], o.centre[2], o.min[0], o.min[1], o.min[2], o.max[0], o.max[1], o.max[2]);
    }
}
