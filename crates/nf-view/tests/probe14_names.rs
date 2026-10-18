//! level-14 probe 4b: name the z≈59.3 duct blocker — every placement whose object carries
//! collision AND whose placement transform lands geometry near the wedge, via the port's own
//! resolved world: walk a 0.55 sphere along the duct at rest height and print the static triangle
//! nearest each wedge contact (world tris keep no names, so instead: every placement within 8
//! units of the wedge with its name/kind/collision, all maps). Prints only.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_wedge_placements() {
    let iso_path = iso();
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();

    // Every placement of ANY map within 8 units (xz) of the wedge centre (40.8, 59.0), with the
    // resolved object name, kind, collision presence, and the object's collision tri count.
    for (mi, map) in level.maps.iter().enumerate() {
        for (pi, p) in map.placements.iter().enumerate() {
            let d2 = (p.translation[0] - 40.8).powi(2) + (p.translation[2] - 59.0).powi(2);
            if d2 > 64.0 { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < map.objects.len() { Some((mi, p.index as usize)) } else { None };
            let (name, tris) = target
                .map(|(m, o)| (level.maps[m].objects[o].name.trim().to_string(),
                    level.maps[m].objects[o].collision.as_ref().map_or(0, |c| c.tris.len())))
                .unwrap_or_else(|| (format!("UNRESOLVED {:08x}", p.hashcode), 0));
            eprintln!("m{mi} plc {pi} \"{}\" kind {:#04x} @({:.2},{:.2},{:.2}) hash {:08x} coll_tris {tris}",
                name, p.flags & 0xff, p.translation[0], p.translation[1], p.translation[2], p.hashcode);
        }
    }
}
