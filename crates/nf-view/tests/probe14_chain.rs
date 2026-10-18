//! level-14 probe 6: the vent room's chain — every interactive placement (triggers, muxes, SPs,
//! fuseboxes, switches, breakables) with full params in the vent store room + north duct
//! (z > 50, x 38..48), to map what opens the z=58.91 grate. Prints only.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_vent_chain() {
    let iso_path = iso();
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let mm = level.main_map().unwrap();

    let kinds = [0x0f_u32, 0x20, 0x29, 0x2c, 0x31, 0x3a, 0xd9, 0xdc, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xf0, 0xf1];
    for (pi, p) in level.maps[mm].placements.iter().enumerate() {
        let k = p.flags & 0xff;
        if !kinds.contains(&k) { continue; }
        let t = p.translation;
        if !(t[2] > 50.0 && t[0] > 36.0 && t[0] < 50.0) && !(k == 0xec || k == 0xed || k == 0xee || k == 0xef) { continue; }
        let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < level.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        let name = target.map(|(m, o)| level.maps[m].objects[o].name.trim().to_string()).unwrap_or_default();
        let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
        eprintln!("#{pi} kind {k:#04x} \"{}\" @({:.2},{:.2},{:.2}) params [{}]", name, t[0], t[1], t[2], params.join(" "));
    }
}
