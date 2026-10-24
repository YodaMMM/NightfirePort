//! level-05-west-lane-6 scratch probe: every collision tri in the band-gap column
//! (x −17..−8.5, z 44..49.5, y 18..36) of 07000005, with node flags, surface byte and
//! winding — the hunt for a thin/one-way surface the down-ray map missed. Also the west
//! pieces' height profile (x −20..−15, z 45..48.5) and a two-sided fine floor profile over
//! x −26..−8. Scratch print, not committed.

use nf_formats::level::Level;
use nf_formats::placement::transform;

fn main() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin in FILES.BIN");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read 07000005.bin");
    let level = Level::parse(&data).expect("parse 07000005");

    // The EXACT instance set the sim's collision world builds from (scene.rs b9: level.instances(true).0).
    let (instances, _missing) = level.instances(true);

    let tx = |m: &[[f32; 4]; 4], p: [f32; 3]| -> [f32; 3] { transform(m, p) };

    // The collected world tris (static set, world space).
    struct W { v: [[f32; 3]; 3], mat: u8, pass: bool, cage: bool, inst: usize, name: String }
    let mut world: Vec<W> = Vec::new();
    for (idx, inst) in instances.iter().enumerate() {
        let om = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &om.collision else { continue };
        if om.flags & 0x20 != 0 { continue; } // shadow cels: the world skips
        let cage = om.flags & 0x1000 != 0;
        let pass = om.flags & 0x40 != 0;
        for (t, m) in c.triangles_with_material() {
            let v = [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])];
            world.push(W { v, mat: m, pass, cage, inst: idx, name: om.name.clone() });
        }
    }
    eprintln!("world tris total {}", world.len());

    // The gap column dump.
    let in_box = |v: &[[f32; 3]; 3], lo: [f32; 3], hi: [f32; 3]| ->
        bool {
        (0..3).all(|k| v.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min) <= hi[k]
            && v.iter().map(|p| p[k]).fold(f32::NEG_INFINITY, f32::max) >= lo[k])
    };
    println!("== gap column x −17..−8.5, z 44..49.5, y 18..36 ==");
    let mut n = 0;
    for w in &world {
        if in_box(&w.v, [-17.0, 18.0, 44.0], [-8.5, 36.0, 49.5]) {
            let nrm = {
                let (a, b, c) = (w.v[0], w.v[1], w.v[2]);
                let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if l > 1e-9 { [n[0] / l, n[1] / l, n[2] / l] } else { [0.0; 3] }
            };
            println!("  inst {:3} {} mat {:#04x} pass {} cage {}  n {:.2} {:.2} {:.2}\n    v0 {:.2} {:.2} {:.2}\n    v1 {:.2} {:.2} {:.2}\n    v2 {:.2} {:.2} {:.2}",
                w.inst, w.name, w.mat, w.pass as u8, w.cage as u8, nrm[0], nrm[1], nrm[2],
                w.v[0][0], w.v[0][1], w.v[0][2], w.v[1][0], w.v[1][1], w.v[1][2], w.v[2][0], w.v[2][1], w.v[2][2]);
            n += 1;
        }
    }
    println!("  ({n} tris in the gap column)");

    // Two-sided fine floor profile over the gap + the west pieces (vertical ray from y 36 down).
    let ray_hit = |w: &W, x: f32, z: f32| -> Option<f32> {
        // Intersect the vertical DOWN ray (x, z) from y 40 with the tri (two-sided, winding-agnostic).
        // d = (0,−1,0): p = d×e2 = (−e2.z, 0, e2.x).
        let (a, b, c) = (w.v[0], w.v[1], w.v[2]);
        let (e1, e2) = ([b[0] - a[0], b[1] - a[1], b[2] - a[2]], [c[0] - a[0], c[1] - a[1], c[2] - a[2]]);
        let pv = [-e2[2], 0.0, e2[0]];
        let det = e1[0] * pv[0] + e1[1] * pv[1] + e1[2] * pv[2];
        if det.abs() < 1e-9 { return None; }
        let inv = 1.0 / det;
        let s = [x - a[0], 40.0 - a[1], z - a[2]];
        let u = (s[0] * pv[0] + s[1] * pv[1] + s[2] * pv[2]) * inv;
        if !(0.0..=1.0).contains(&u) { return None; }
        let qv = [s[1] * e1[2] - s[2] * e1[1], s[2] * e1[0] - s[0] * e1[2], s[0] * e1[1] - s[1] * e1[0]];
        let v = (-qv[1]) * inv; // d = (0,−1,0): v = (d·q)/det = −q.y/det
        if v < 0.0 || u + v > 1.0 { return None; }
        let t = (e2[0] * qv[0] + e2[1] * qv[1] + e2[2] * qv[2]) * inv;
        let h = 40.0 - t;
        (t >= 0.0 && h >= 10.0).then_some(h)
    };
    println!("== two-sided floor profile (top hit below y 40, all tris incl. pass/cage) ==");
    for z in [40.0, 38.0, 36.0, 34.0, 32.0, 30.0, 28.0, 26.0, 24.0, 22.0, 20.0, 18.0, 16.0, 14.0] {
        let mut row = format!("z {z:5.1}: ");
        let mut x = -28.0;
        while x <= -18.0 + 1e-6 {
            let mut best: Option<(f32, &W)> = None;
            for we in &world {
                if let Some(h) = ray_hit(we, x, z) {
                    if best.map_or(true, |(bh, _)| h > bh) { best = Some((h, we)); }
                }
            }
            match best {
                Some((h, w)) => row.push_str(&format!("{h:5.1}{}", if w.pass { "p" } else if w.cage { "c" } else { " " })),
                None => row.push_str("  .  "),
            }
            x += 0.5;
        }
        println!("{row}");
    }
    println!("   (x from −26 to −8 step 0.5; p = node pass flag, c = missile cage; the ray is winding-agnostic)");
}
