//! The floor map between the wire landing (−23, 30, z 0) and the battlement chain's west end
//! (13.7, 37.4, −72.2): down-ray top hits below y 55 over the south region. Scratch, not committed.

use nf_formats::level::Level;

fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let (instances, _missing) = level.instances(true);
    let tx = |m: &[[f32; 4]; 4], p: [f32; 3]| -> [f32; 3] { nf_formats::placement::transform(m, p) };

    struct W { v: [[f32; 3]; 3], pass: bool }
    let mut world: Vec<W> = Vec::new();
    for inst in instances.iter() {
        let om = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &om.collision else { continue };
        if om.flags & 0x20 != 0 { continue; }
        let pass = om.flags & 0x40 != 0;
        for (t, _) in c.triangles_with_material() {
            world.push(W { v: [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])], pass });
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
    // Rows z from 4 down to −76; columns x −26..50 step 2. Top hit under y 55.
    for zi in (0..=20).rev() {
        let z = (4 - 4 * zi) as f32;
        let mut row = format!("z {z:6.1}: ");
        let mut x = -26.0f32;
        while x <= 50.0 {
            let mut best: Option<(f32, bool)> = None;
            for w in &world {
                if let Some(h) = ray_hit(w, x, z, 55.0) {
                    if best.map_or(true, |(bh, _)| h > bh) { best = Some((h, w.pass)); }
                }
            }
            match best {
                Some((h, pass)) => row.push_str(&format!("{h:5.1}{}", if pass {"p"} else {" "})),
                None => row.push_str("   .  "),
            }
            x += 2.0;
        }
        println!("{row}");
    }
    println!("  (x −26..50 step 2; height = top hit below y 55; p = pass flag)");
}
