//! level-05-every-objective-4: what does Bond stand on at (21.8, −68.1) at y 50.04?
//! The -3 test `the_door_band_exit_chains_to_07000006_climb_parked` places Bond at
//! (22.0, 49.2, −68.0) expecting the y-49 terrace band; he settles at y 50.035 at
//! (21.805, −68.098). This probe maps the terrace collision around that spot: the
//! top surface below y0 55 on a fine grid over x 14..38, z −73..−63, plus full
//! column dumps at the settle point and at door #112's threshold. Scratch probe.

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
        // NO 0x20 skip this run: the port's CollisionWorld::build_scoped keeps every object with
        // collision (its push filter is only "collision present, not the 0x1000 missile flag").
        let skip20 = om.flags & 0x20 != 0;
        let missile = om.flags & 0x1000 != 0;
        for (t, mat) in c.triangles_with_material() {
            world.push(W {
                v: [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])],
                tag: format!("i{} m{} o{} mat{}{}{}", ii, inst.map, inst.object, mat,
                    if skip20 { " SKIP20" } else { "" },
                    if missile { " MISSILE" } else { "" }),
            });
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
    let all_hits = |x: f32, z: f32, y0: f32| -> Vec<(f32, String)> {
        let mut v: Vec<(f32, String)> = Vec::new();
        for w in world.iter() {
            if let Some(h) = ray_hit(w, x, z, y0) { v.push((h, w.tag.clone())); }
        }
        v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        v.dedup_by(|a, b| (a.0 - b.0).abs() < 0.02);
        v
    };

    // ---- MAP A: the terrace top below y0 55, x 14..38 step 1, z −73..−63 step 1 ----
    println!("=== MAP A terrace top below y0 55 (x east, z south->north rows; . = void) ===");
    print!("z\\x");
    for x in 14..=38 { print!("{:>4}", x); }
    println!();
    for zz in (-73..=-63).rev() {
        let z = zz as f32 + 0.5;
        print!("{:>6.1}", z);
        for x in 14..=38 {
            match all_hits(x as f32 + 0.5, z, 55.0).first() {
                Some((h, _)) => print!("{:>4.1}", h),
                None => print!("   ."),
            }
        }
        println!();
    }

    // ---- MAP B: the columns at the settle point and nearby ----
    for (x, z) in [(21.805f32, -68.098f32), (22.0, -68.0), (21.5, -68.0), (22.5, -68.0),
                   (23.0, -68.0), (24.0, -68.0), (26.0, -68.0), (30.0, -68.0), (34.0, -68.0),
                   (22.0, -69.0), (22.0, -67.0), (22.0, -66.0), (22.0, -70.0), (35.4, -69.0),
                   (37.0, -69.0), (104.954, 108.825)] {
        let hits = all_hits(x, z, 58.0);
        println!("=== col ({x}, {z}) below y0 58: {} ===", hits.len());
        for (h, tag) in hits.iter().take(8) { println!("  {:7.3}  {}", h, tag); }
    }
}
