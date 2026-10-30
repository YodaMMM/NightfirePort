//! walkreal probe #8 (level-0c-gate-band-crossing): the full transformed triangle sets of the
//! four band-slab providers (room_turbine02/03, obj_turb2_walkway01, obj_turb3_walkway04) over
//! x 50..72, z −212..−202, y 5..13 — every tri printed — plus the primitive census of each
//! object's raw collision block (the types the port's parser keeps). Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_band_tri_extents() {
    let iso = iso();
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    // The providers by name.
    let want = ["room_turbine02", "room_turbine03", "obj_turb2_walkway01", "obj_turb3_walkway04"];
    for (oi, o) in level.maps[main].objects.iter().enumerate() {
        let name = o.name.trim();
        if !want.iter().any(|w| name.eq_ignore_ascii_case(w)) { continue; }
        let Some(col) = o.collision.as_ref() else { eprintln!("\"{name}\": no collision block"); continue };
        // Which placements use this object (by index)?
        let plcs: Vec<usize> = level.maps[main].placements.iter().enumerate()
            .filter(|(_, p)| !p.by_hash() && p.index as usize == oi)
            .map(|(i, _)| i).collect();
        eprintln!("== obj{oi} \"{name}\" hash {:#010x} raw_tris {} placements {:?} ==", o.hashcode, col.triangles().len(), plcs);
        for pi in plcs {
            let p = &level.maps[main].placements[pi];
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            let mut n = 0usize;
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                // In the region?
                let cx = w.iter().map(|v| v[0]).sum::<f32>() / 3.0;
                let cy = w.iter().map(|v| v[1]).sum::<f32>() / 3.0;
                let cz = w.iter().map(|v| v[2]).sum::<f32>() / 3.0;
                if !(50.0..=72.0).contains(&cx) || !(-212.0..=-202.0).contains(&cz) || !(5.0..=13.0).contains(&cy) { continue; }
                n += 1;
                eprintln!("  plc{pi} tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
                    w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2]);
            }
            eprintln!("  plc{pi}: {n} tris in region");
        }
    }
    eprintln!("ok");
}
