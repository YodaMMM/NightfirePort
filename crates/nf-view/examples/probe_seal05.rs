//! The band's north seal, fine: the region between the approach (z −14..−20) and the wall-hug
//! band (z −21..−26) at x −36..−20 — floors (down-ray from y 42), the ceiling above each floor
//! (up-ray headroom), and three vertical cut lanes at 0.25 resolution. [D the port's collision
//! of the disc's own data — the disc's tri set, no port collision code].

use nf_formats::level::Level;

fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let (instances, _missing) = level.instances(true);
    let tx = |m: &[[f32; 4]; 4], p: [f32; 3]| -> [f32; 3] { nf_formats::placement::transform(m, p) };

    struct W { v: [[f32; 3]; 3] }
    let mut world: Vec<W> = Vec::new();
    for inst in instances.iter() {
        let om = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &om.collision else { continue };
        if om.flags & 0x20 != 0 { continue; }
        for (t, _) in c.triangles_with_material() {
            world.push(W { v: [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])] });
        }
    }
    eprintln!("tris {}", world.len());

    // A vertical ray (dir = −1 or +1 on y): returns every hit height, sorted.
    let ray_hits = |w: &W, x: f32, z: f32, y0: f32, down: bool| -> Vec<f32> {
        let (a, b, c) = (w.v[0], w.v[1], w.v[2]);
        let (e1, e2) = ([b[0]-a[0], b[1]-a[1], b[2]-a[2]], [c[0]-a[0], c[1]-a[1], c[2]-a[2]]);
        // dir = (0, ∓1, 0): det = e1·(e2×dir)
        let pv = if down { [-e2[2], 0.0, e2[0]] } else { [e2[2], 0.0, -e2[0]] };
        let det = e1[0]*pv[0] + e1[1]*pv[1] + e1[2]*pv[2];
        if det.abs() < 1e-9 { return vec![]; }
        let inv = 1.0/det;
        let s = [x-a[0], y0-a[1], z-a[2]];
        let u = (s[0]*pv[0] + s[1]*pv[1] + s[2]*pv[2]) * inv;
        if !(0.0..=1.0).contains(&u) { return vec![]; }
        let qv = [s[1]*e1[2]-s[2]*e1[1], s[2]*e1[0]-s[0]*e1[2], s[0]*e1[1]-s[1]*e1[0]];
        let vv = if down { -qv[1]*inv } else { qv[1]*inv };
        if vv < 0.0 || u+vv > 1.0 { return vec![]; }
        let t = (e2[0]*qv[0] + e2[1]*qv[1] + e2[2]*qv[2]) * inv;
        if t < 0.0 { return vec![]; }
        vec![if down { y0 - t } else { y0 + t }]
    };

    // ---- 1: the floor + headroom map, 0.5 units ----
    println!("=== floor (down from 42) + ceiling (up from floor+0.05, cap 46) — x −36..−20 step 0.5, z −14..−27 step 0.5 ===");
    let mut z = -14.0f32;
    while z >= -26.75 {
        let mut row = format!("z {z:6.2}: ");
        let mut x = -36.0f32;
        while x <= -19.75 {
            let mut floor: Option<f32> = None;
            for w in &world {
                for hh in ray_hits(w, x, z, 42.0, true) {
                    if floor.map_or(true, |f| hh > f) { floor = Some(hh); }
                }
            }
            match floor {
                Some(f) => {
                    // the lowest ceiling above the floor
                    let mut ceil: Option<f32> = None;
                    for w in &world {
                        for hh in ray_hits(w, x, z, f + 0.05, false) {
                            if hh <= 46.0 && ceil.map_or(true, |c| hh < c) { ceil = Some(hh); }
                        }
                    }
                    match ceil {
                        Some(c) => row.push_str(&format!("{f:4.1}/{:.1} ", c - f)), // floor/headroom
                        None => row.push_str(&format!("{f:4.1}/  .  ")),
                    }
                }
                None => row.push_str("   .     "),
            }
            x += 0.5;
        }
        println!("{row}");
        z -= 0.5;
    }
    println!("(each cell: floor/headroom; . = no floor under 42)");

    // ---- 2: vertical cut lanes at 0.25 z-resolution: every surface between y 30 and 42 ----
    for &xl in &[-26.0f32, -24.3, -23.7, -31.5, -33.0, -34.5] {
        println!("=== lane x {xl}: all hits (down from 42), z −14..−27 step 0.25 ===");
        let mut z = -14.0f32;
        let mut row = String::from("   z: ");
        while z >= -26.75 { row.push_str(&format!("{:5.1}", z)); z -= 1.0; }
        println!("{row}");
        for &y0 in &[42.0f32] {
            let mut z = -14.0f32;
            while z >= -26.75 {
                let mut hits: Vec<f32> = Vec::new();
                for w in &world { hits.extend(ray_hits(w, xl, z, y0, true)); }
                hits.sort_by(|a, b| b.partial_cmp(a).unwrap());
                hits.dedup_by(|a, b| (*a - *b).abs() < 0.02);
                let s: String = hits.iter().take(6).map(|h| format!("{h:5.1}")).collect();
                println!("z {z:6.2}: {s}");
                z -= 0.25;
            }
        }
    }
}
