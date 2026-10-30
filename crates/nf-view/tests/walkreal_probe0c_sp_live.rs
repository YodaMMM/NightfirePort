//! walkreal probe #16 (level-0c-gate-band-crossing): every 0xd9 script-player placement in the
//! level (all maps) with its script hash + params, and every placement of any class carrying a
//! 0x06xxxxxx param. The forklift family census. Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_all_script_players() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");

    eprintln!("=== all 0xd9 placements ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if p.flags & 0xff != 0xd9 { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(om, oi)| level.maps[om].objects[oi].name.trim().to_string()).unwrap_or("<unresolved>".into());
            eprintln!("map{mi} plc{pi} \"{name}\" t ({:.2},{:.2},{:.2}) params {:?}", p.translation[0], p.translation[1], p.translation[2], p.params);
        }
    }
    eprintln!("=== placements with any 0x06/0x07/0x02 hash param (class, name, pos) ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let class = p.flags & 0xff;
            if matches!(class, 0xd9 | 0x0f | 0xf1) { continue; } // dumped above / drones separate
            let hits: Vec<u32> = p.params.iter().filter(|&(_, v)| v >> 24 == 0x06 || v >> 24 == 0x07 || v >> 24 == 0x02).map(|&(_, v)| v).collect();
            if hits.is_empty() { continue; }
            let t = p.translation;
            if !(t[0] >= 30.0 && t[0] <= 100.0 && t[2] >= -250.0 && t[2] <= -180.0) { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(om, oi)| level.maps[om].objects[oi].name.trim().to_string()).unwrap_or("<unresolved>".into());
            eprintln!("map{mi} plc{pi} class {class:#04x} \"{name}\" t ({:.2},{:.2},{:.2}) hashes {:#010x?}", t[0], t[1], t[2], hits);
        }
    }
    eprintln!("ok");
}
