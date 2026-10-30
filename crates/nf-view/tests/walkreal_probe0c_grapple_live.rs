//! walkreal probe #21 (level-0c-gate-band-crossing): the crossing gadgets pinned. Dumps (1) the
//! sim's grapple_points with their positions, (2) the Zipline placement's full mesh tri list from
//! the disc (the wire's actual run), (3) the icon placements' full world tris.
//! Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_grapple_and_zipline() {
    let isopath = iso();
    {
        let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
        w.wait_enabled(3000, "the intro");
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== grapple points ===");
        for (i, g) in sim.drones.grapple_points.iter().enumerate() {
            eprintln!("gp{i}: pos ({:.3},{:.3},{:.3}) {:?}", g.pos[0], g.pos[1], g.pos[2], g);
        }
    }
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    eprintln!("=== Zipline plc512 + the icons: full tris ===");
    for (pi, p) in level.maps[main].placements.iter().enumerate() {
        if !matches!(pi, 512 | 89 | 91 | 92 | 93 | 511) { continue; }
        let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
        let Some((om, oi)) = target else { continue };
        let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { eprintln!("plc{pi}: no collision"); continue };
        let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
        eprintln!("== plc{pi} \"{}\" class {:#04x} ==", level.maps[om].objects[oi].name.trim(), p.flags & 0xff);
        for t in col.triangles() {
            let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
            eprintln!("  ({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})", w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2]);
        }
    }
    eprintln!("ok");
}
