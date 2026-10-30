//! walkreal probe #15 (level-0c-gate-band-crossing): glist bounds vs collision extents for the
//! band providers. The MapObject's centre/radius/min/max are the glist's bounds [M map.rs]; if a
//! walkway's glist spans the void but its 0x2E collision stops short, the mesh bridges it.
//! Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_glist_vs_collision() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    let want = ["obj_turb2_walkway01", "obj_turb2_walkway02", "obj_turb2_walkway03", "obj_turb2_walkway04",
        "obj_turb3_walkway04", "obj_turb1_walkway01", "room_turbine02", "room_turbine03", "obj_turb2_pulley"];
    for (oi, o) in level.maps[main].objects.iter().enumerate() {
        let name = o.name.trim().to_string();
        if !want.iter().any(|w| name.eq_ignore_ascii_case(w)) { continue; }
        let plcs: Vec<usize> = level.maps[main].placements.iter().enumerate()
            .filter(|(_, p)| !p.by_hash() && p.index as usize == oi)
            .map(|(i, _)| i).collect();
        for pi in plcs {
            let p = &level.maps[main].placements[pi];
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            // glist bounds through the placement matrix (the min/max corners are enough for axis-aligned).
            let gmin = plc::transform(&matrix, o.min);
            let gmax = plc::transform(&matrix, o.max);
            let (colx0, colx1, coly0, coly1, colz0, colz1) = match o.collision.as_ref() {
                Some(c) => {
                    let xs: Vec<f32> = c.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[0])).collect();
                    let ys: Vec<f32> = c.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[1])).collect();
                    let zs: Vec<f32> = c.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[2])).collect();
                    (xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max),
                     ys.iter().cloned().fold(f32::MAX, f32::min), ys.iter().cloned().fold(f32::MIN, f32::max),
                     zs.iter().cloned().fold(f32::MAX, f32::min), zs.iter().cloned().fold(f32::MIN, f32::max))
                }
                None => (f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN, f32::NAN),
            };
            eprintln!("\"{name}\" plc{pi}: glist centre ({:.2},{:.2},{:.2}) r {:.2} bounds ({:.2},{:.2},{:.2})..({:.2},{:.2},{:.2}) | collision x {colx0:.2}..{colx1:.2} y {coly0:.2}..{coly1:.2} z {colz0:.2}..{colz1:.2}",
                o.centre[0], o.centre[1], o.centre[2], o.radius, gmin[0], gmin[1], gmin[2], gmax[0], gmax[1], gmax[2]);
        }
    }
    eprintln!("ok");
}
