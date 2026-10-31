//! level-05-terrace-walk scratch: floor map over the stairs' east continuation
//! (x 33..54, z -80..-54) — highest hit below y0, to find the disc's own route from the
//! covered staircase's top to the y-49 terrace band.
use nf_formats::level::Level;
fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let (instances, _missing) = level.instances(true);
    let tx = |m: &[[f32; 4]; 4], p: [f32; 3]| -> [f32; 3] { nf_formats::placement::transform(m, p) };
    struct W { v: [[f32; 3]; 3], tag: String }
    let mut world: Vec<W> = Vec::new();
    for (ii, inst) in instances.iter().enumerate() {
        let om = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &om.collision else { continue };
        for (t, mat) in c.triangles_with_material() {
            world.push(W { v: [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])],
                tag: format!("i{}.m{}.o{}.mat{}", ii, inst.map, inst.object, mat) });
        }
    }
    eprintln!("tris {}", world.len());
    let ray_hit = |w: &W, x: f32, z: f32, y0: f32| -> Option<f32> {
        let (a, b, c) = (w.v[0], w.v[1], w.v[2]);
        let (e1, e2) = ([b[0]-a[0], b[1]-a[1], b[2]-a[2]], [c[0]-a[0], c[1]-a[1], c[2]-a[2]]);
        let pv = [-e2[2], 0.0, e2[0]];
        let det = e1[0]*pv[0] + e1[1]*pv[1] + e1[2]*pv[2];
        if det.abs() < 1e-9 { return None; }
        let inv = 1.0/det;
        let s = [x-a[0], y0-a[1], z-a[2]];
        let u = (s[0]*pv[0] + s[1]*pv[1] + s[2]*pv[2]) * inv;
        if !(0.0..=1.0).contains(&u) { return None; }
        let qv = [s[1]*e1[2]-s[2]*e1[1], s[2]*e1[0]-s[0]*e1[2], s[0]*e1[1]-s[1]*e1[0]];
        let vv = -qv[1]*inv;
        if vv < 0.0 || u+vv > 1.0 { return None; }
        let t = (e2[0]*qv[0] + e2[1]*qv[1] + e2[2]*qv[2]) * inv;
        let h = y0 - t;
        (t >= 0.0 && h >= 0.0).then_some(h)
    };
    let top = |x: f32, z: f32, y0: f32| -> Option<f32> {
        let mut best: Option<f32> = None;
        for w in world.iter() { if let Some(h) = ray_hit(w, x, z, y0) { if best.map_or(true, |b| h > b) { best = Some(h); } } }
        best
    };
    // F: the walls crossing the y 50.2 plane (body height) in the doorhouse box —
    // print every triangle's cut segment so the doorway gap shows.
    {
        println!("=== F: wall sections at y 50.2, x 34..42, z -65..-72 ===");
        let y = 50.2f32;
        for w in world.iter() {
            let mut pts: Vec<[f32; 2]> = Vec::new();
            let v = &w.v;
            for a in 0..3 {
                let b = (a + 1) % 3;
                let (ya, yb) = (v[a][1], v[b][1]);
                if (ya - y) * (yb - y) < 0.0 {
                    let t = (y - ya) / (yb - ya);
                    pts.push([v[a][0] + t * (v[b][0] - v[a][0]), v[a][2] + t * (v[b][2] - v[a][2])]);
                }
            }
            if pts.len() == 2 {
                let (p1, p2) = (pts[0], pts[1]);
                let in_box = |p: [f32; 2]| p[0] >= 34.0 && p[0] <= 42.0 && p[1] >= -72.0 && p[1] <= -65.0;
                if in_box(p1) && in_box(p2) {
                    println!("  ({:.2},{:.2})-({:.2},{:.2})  {}", p1[0], p1[1], p2[0], p2[1], w.tag);
                }
            }
        }
    }
    // E: the doorhouse wall face at 0.25 — where is the arch opening?
    {
        println!("=== E: fine 0.25 tops, x 35..41 step 0.25, z -66..-71 (y0 55) ===");
        print!("z\\x ");
        let x0 = 35.0f32; let x1 = 41.0f32;
        let mut xx = x0; while xx <= x1 { print!("{:>5.2}", xx); xx += 0.25; }
        println!();
        let mut zz = -66.0f32;
        while zz >= -71.0 {
            print!("{:>6.2}", zz);
            let mut xx = x0;
            while xx <= x1 {
                match top(xx, zz, 55.0) {
                    Some(h) => print!("{:>5.0}", h),
                    None => print!("    ."),
                }
                xx += 0.25;
            }
            println!();
            zz -= 0.25;
        }
    }
    // FINE: the stairs'-top → walkway junction at 0.25
    {
        println!("=== D: fine 0.25 below y0 46.5, x 35..47, z -73..-68 (rows = z) ===");
        print!("z\\x ");
        let x0 = 35.0f32; let x1 = 47.0f32;
        let mut xx = x0; while xx <= x1 { print!("{:>5.2}", xx); xx += 0.5; }
        println!();
        let mut zz = -73.0f32;
        while zz <= -68.0 {
            print!("{:>6.2}", zz);
            let mut xx = x0;
            while xx <= x1 {
                match top(xx, zz, 46.5) {
                    Some(h) => print!("{:>5.1}", h),
                    None => print!("    ."),
                }
                xx += 0.5;
            }
            println!();
            zz += 0.25;
        }
    }
    for (label, y0, xr, zr) in [
        ("A: below y0 52", 52.0f32, 33..=54, -80..=-54),
        ("B: below y0 50 (under the 52 roof)", 50.0, 36..=56, -72..=-54),
        ("C: below y0 46.5", 46.5, 36..=56, -72..=-54),
    ] {
        println!("=== {label}, x {}..{}, z {}..{} (rows = z, . = void) ===", xr.start(), xr.end(), zr.start(), zr.end());
        print!("z\\x");
        for x in xr.clone() { print!("{:>5}", x); }
        println!();
        for zz in zr.clone().rev() {
            let z = zz as f32 + 0.5;
            print!("{:>6.1}", z);
            for x in xr.clone() {
                match top(x as f32 + 0.5, z, y0) {
                    Some(h) => print!("{:>5.1}", h),
                    None => print!("    ."),
                }
            }
            println!();
        }
    }
}
