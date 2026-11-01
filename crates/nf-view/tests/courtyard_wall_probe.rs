//! level-03-fall-through-floor — the courtyard wall probe (geometry + a real-input walk).
//!
//! The old flank's nav (c118ed46 on s/1b-wall-wedge-2) ran "through the obstacle at
//! (6.35, −15.7..−17.7)", "Bond climbs to y 7.3, airborne"; 00310262 calls it "west of a wall at
//! (6.4, −17.7)". The suite fall reached y −5577. This probe:
//!   A. dumps every collision tri (floor ny>0.7 and near-vertical) of every instance inside the
//!      box x 3..10, y −2..14, z −22..−12 [D 07000003.bin collision block 0x2E], so the floor
//!      seam/hole around the wall is visible;
//!   B. walks Bond (real input, the Walker's tick) from mid courtyard to the wall spot and
//!      samples y/grounded per tick on the last approach, watching for the drop.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn tri_box(w: &[[f32; 3]; 3]) -> ([f32; 3], [f32; 3]) {
    let mut lo = w[0]; let mut hi = w[0];
    for p in w.iter().skip(1) { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
    (lo, hi)
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_the_courtyard_wall_geometry_and_walk() {
    let iso = iso();
    // ---- A. the geometry -------------------------------------------------------
    let data = read_archive(&iso, "07000003.bin");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    let (blo, bhi) = ([3.0f32, -2.0, -22.0], [10.0f32, 14.0, -12.0]);
    let meets = |lo: &[f32; 3], hi: &[f32; 3]| {
        lo[0] <= bhi[0] && hi[0] >= blo[0] && lo[1] <= bhi[1] && hi[1] >= blo[1] && lo[2] <= bhi[2] && hi[2] >= blo[2]
    };
    let mut floors: Vec<(f32, f32, f32, [f32; 3], u8)> = Vec::new(); // cx, cy, cz, n, mat
    let mut verts: Vec<String> = Vec::new();
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        for (t, m) in c.triangles_with_material() {
            let w: [[f32; 3]; 3] = t.map(|p| nf_formats::placement::transform(&inst.matrix, p));
            let (tlo, thi) = tri_box(&w);
            if !meets(&tlo, &thi) { continue; }
            let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
            let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if l < 1e-9 { continue; }
            let n = [n[0] / l, n[1] / l, n[2] / l];
            let cx = (w[0][0] + w[1][0] + w[2][0]) / 3.0;
            let cy = (w[0][1] + w[1][1] + w[2][1]) / 3.0;
            let cz = (w[0][2] + w[1][2] + w[2][2]) / 3.0;
            if n[1] > 0.7 {
                floors.push((cx, cy, cz, n, m));
                println!("floor  i{pi} x {:+.2}..{:+.2} y {:+.2}..{:+.2} z {:+.2}..{:+.2} n ({:+.2},{:+.2},{:+.2}) mat {m:#04x}",
                    tlo[0], thi[0], tlo[1], thi[1], tlo[2], thi[2], n[0], n[1], n[2]);
            } else if n[1].abs() < 0.35 {
                verts.push(format!(
                    "vert   i{pi} x {:+.2}..{:+.2} y {:+.2}..{:+.2} z {:+.2}..{:+.2} n ({:+.2},{:+.2},{:+.2}) mat {m:#04x}",
                    tlo[0], thi[0], tlo[1], thi[1], tlo[2], thi[2], n[0], n[1], n[2]));
            }
        }
    }
    for v in &verts { println!("{v}"); }
    // The floor-height map on a 0.5 grid: the highest floor face under each xz cell (y 0..9),
    // '.' = nothing walkable under the cell.
    println!("--- floor map (max floor y per 0.5 xz cell, '.' = none in y 0..9) ---");
    let mut z = -22.0f32;
    while z <= -12.0 {
        let mut row = format!("z {z:+06.1} ");
        let mut x = 3.0f32;
        while x <= 10.0 {
            let mut best: Option<f32> = None;
            for (cx, cy, cz, _, _) in &floors {
                // coarse: does the tri's xz box contain the cell centre?
                if (cx - x).abs() <= 1.2 && (cz - z).abs() <= 1.2 && *cy > 0.0 && *cy < 9.0 {
                    best = Some(best.map_or(*cy, |b: f32| b.max(*cy)));
                }
            }
            row.push(match best { Some(y) => char::from_digit((y * 10.0) as u32 % 10, 10).unwrap_or('?'), None => '.' });
            x += 0.5;
        }
        println!("{row}");
        z += 0.5;
    }

    // ---- B. the walk -----------------------------------------------------------
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(60);
    for (name, at) in [("mid courtyard", [13.0f32, 5.9, -18.0]), ("west of the wall", [7.5, 5.9, -17.0])] {
        w.walk_nav_to(at, name, 1.6);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("walk probe: bond at {:?}", sim.bond.pos);
    // Real-input approach straight at the wall spot, per-tick samples.
    for (label, target) in [
        ("to the wall spot (6.4,-17.7)", [6.4f32, 5.9, -17.7]),
        ("to (5.2,-16.5)", [5.2, 5.9, -16.5]),
    ] {
        let start = w.total_ticks;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let d = ((target[0] - p[0]).powi(2) + (target[2] - p[2]).powi(2)).sqrt();
            if d < 1.0 || w.total_ticks - start > 600 { break; }
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = desired - sim.bond.yaw;
            let err = err - (err / std::f32::consts::PI).round() * std::f32::consts::PI;
            let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            let y_before = p[1];
            w.tick(input, false);
            let sim = w.scene.sim.as_ref().unwrap();
            let p2 = sim.bond.pos;
            if !sim.bond.grounded || p2[1] < y_before - 0.05 || (w.total_ticks % 30 == 0) {
                eprintln!("  {label} t{} bond ({:.2},{:.2},{:.2}) grounded {} sub {:#x} ny {:.2}",
                    w.total_ticks, p2[0], p2[1], p2[2], sim.bond.grounded, sim.bond.substate, sim.bond.floor_ny);
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("{label}: ended at {:?}", sim.bond.pos);
    }
    eprintln!("walk probe: falls {:?}", w.report.falls);
}
