//! all placements quick dump
use nf_formats::level::Level;
fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let mm = level.main_map().expect("main map");
    for (i, p) in level.maps[mm].placements.iter().enumerate() {
        let (x, y, z) = (p.translation[0], p.translation[1], p.translation[2]);
        let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < level.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        let name = target.map(|(m, o)| level.maps[m].objects[o].name.trim().to_string()).unwrap_or_default();
        let nl = name.to_lowercase();
        let route = (x as i32) >= 8 && (x as i32) <= 52 && (z as i32) >= -80 && (z as i32) <= -55;
        if nl.contains("wire") || nl.contains("ladder") || nl.contains("icon") || nl.contains("grap") || route {
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("#{i} kind {:#04x} \"{}\" @({:.2},{:.2},{:.2}) rot({:.3},{:.3},{:.3}){ps}", p.flags & 0xff, name, x, y, z, p.rotation[0], p.rotation[1], p.rotation[2]);
        }
    }
}
