//! walkreal probe #10 (level-0c-gate-band-crossing): the walkway02/03/04 + pulley world extents.
//! Where do the parked turbine pieces actually sit? Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_walkway_extents() {
    let iso = iso();
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    let want = ["obj_turb2_walkway02", "obj_turb2_walkway03", "obj_turb2_walkway04", "obj_turb2_pulley", "obj_turb1_walkway05", "obj_turb1_walkway06", "obj_turb1_walkway07", "obj_turb1_walkway08"];
    for (oi, o) in level.maps[main].objects.iter().enumerate() {
        let name = o.name.trim().to_string();
        if !want.iter().any(|w| name.eq_ignore_ascii_case(w)) { continue; }
        let Some(col) = o.collision.as_ref() else { eprintln!("\"{name}\": no collision"); continue };
        for (pi, p) in level.maps[main].placements.iter().enumerate() {
            let hit = if p.by_hash() { p.hashcode == o.hashcode } else { p.index as usize == oi };
            if !hit { continue; }
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            let xs: Vec<f32> = col.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[0])).collect();
            let ys: Vec<f32> = col.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[1])).collect();
            let zs: Vec<f32> = col.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[2])).collect();
            let (x0, x1) = (xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max));
            let (y0, y1) = (ys.iter().cloned().fold(f32::MAX, f32::min), ys.iter().cloned().fold(f32::MIN, f32::max));
            let (z0, z1) = (zs.iter().cloned().fold(f32::MAX, f32::min), zs.iter().cloned().fold(f32::MIN, f32::max));
            eprintln!("\"{name}\" plc{pi} cel {} flags {:#010x}: {} tris, world x {x0:.2}..{x1:.2} y {y0:.2}..{y1:.2} z {z0:.2}..{z1:.2}",
                p.creates_cel(), p.flags, col.triangles().len());
        }
    }
    eprintln!("ok");
}
