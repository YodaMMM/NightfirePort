//! level-05-west-lane-7 scratch probe: the step shelf's z-46.0 kink at (−23.5, 33, 46.5).
//! 1. Every collision tri in the kink box (x −28..−19, z 41..49.5, y 30..36) — the 45° ramp's
//!    triangles and the wall tri that stalls the spine walk [D].
//! 2. A fine two-sided floor profile over x −28..−20, z 48..41 (step 0.25) — floor vs void at
//!    the kink.
//! 3. The walk to the step spine's stall (the round-6 route through the walked gap), then a
//!    per-tick steer toward spine 4 with the capsule's contacts printed — which tri blocks,
//!    what its normal is, and what the capsule push does [D].
//! Scratch print, not merged evidence until cited in the report.

#[path = "../tests/walkreal_07000005_live.rs"]
mod t05;

use nf_formats::level::Level;
use nf_formats::placement::transform;
use nf_game::player::Input;
use t05::walker::Walker;

fn main() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin in FILES.BIN");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read 07000005.bin");
    let level = Level::parse(&data).expect("parse 07000005");
    let (instances, _missing) = level.instances(true);

    let tx = |m: &[[f32; 4]; 4], p: [f32; 3]| -> [f32; 3] { transform(m, p) };

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

    let tri_normal = |v: &[[f32; 3]; 3]| -> [f32; 3] {
        let (a, b, c) = (v[0], v[1], v[2]);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 1e-9 { [n[0] / l, n[1] / l, n[2] / l] } else { [0.0; 3] }
    };
    let in_box = |v: &[[f32; 3]; 3], lo: [f32; 3], hi: [f32; 3]| -> bool {
        (0..3).all(|k| v.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min) <= hi[k]
            && v.iter().map(|p| p[k]).fold(f32::NEG_INFINITY, f32::max) >= lo[k])
    };

    // 1. The kink box tri dump.
    println!("== kink box x −28..−19, z 41..49.5, y 30..36 ==");
    let mut n = 0;
    for w in &world {
        if in_box(&w.v, [-28.0, 30.0, 41.0], [-19.0, 36.0, 49.5]) {
            let nr = tri_normal(&w.v);
            println!("  inst {:3} {} mat {:#04x} pass {} cage {}  n {:.3} {:.3} {:.3}\n    v0 {:.3} {:.3} {:.3}\n    v1 {:.3} {:.3} {:.3}\n    v2 {:.3} {:.3} {:.3}",
                w.inst, w.name, w.mat, w.pass as u8, w.cage as u8, nr[0], nr[1], nr[2],
                w.v[0][0], w.v[0][1], w.v[0][2], w.v[1][0], w.v[1][1], w.v[1][2], w.v[2][0], w.v[2][1], w.v[2][2]);
            n += 1;
        }
    }
    println!("  ({n} tris in the kink box)");

    // 2. The fine floor profile over the kink (top hit below y 40, two-sided).
    let ray_hit = |w: &W, x: f32, z: f32| -> Option<f32> {
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
        let v = (-qv[1]) * inv;
        if v < 0.0 || u + v > 1.0 { return None; }
        let t = (e2[0] * qv[0] + e2[1] * qv[1] + e2[2] * qv[2]) * inv;
        let h = 40.0 - t;
        (t >= 0.0 && h >= 10.0).then_some(h)
    };
    println!("== fine floor profile x −28..−20 (step 0.25), z 48..41 (step 0.5), top hit below y 40 ==");
    for z in (84..=96).rev().step_by(1).map(|q| q as f32 * 0.5) {
        let mut row = format!("z {z:5.1}: ");
        let mut x = -28.0f32;
        while x <= -19.75 + 1e-6 {
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
            x += 0.25;
        }
        println!("{row}");
    }
    println!("   (x from −28.00 step 0.25; p = node pass flag, c = missile cage; two-sided ray)");

    // 3. The walk to the stall, then the per-tick contact probe.
    let mut w = Walker::new(nf_view::load(&iso_path, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    t05::walk_to_the_slab_east_end(&mut w);
    t05::walk_the_gap_to_the_notch(&mut w);
    eprintln!("[kink] the walked route done at tick {} bond {:?}", w.total_ticks, w.scene.sim.as_ref().unwrap().bond.pos);

    // Steer toward spine 4 (−23.9, 33.6, 45.8) and past it, per-tick dump.
    let targets: [([f32; 3], &str); 10] = [
        ([-23.75, 33.5, 45.95], "stall approach"),
        ([-23.9, 33.6, 45.8], "spine 4"),
        ([-24.3, 33.6, 45.4], "spine 5"),
        ([-24.7, 33.6, 45.0], "spine 6"),
        ([-25.1, 33.5, 44.6], "spine 7"),
        ([-25.5, 33.4, 44.2], "spine 8"),
        ([-25.8, 33.4, 43.8], "spine 9"),
        ([-26.0, 33.4, 43.2], "ledge bend"),
        ([-24.9, 33.4, 36.5], "ledge 36"),
        ([-23.5, 34.5, 18.5], "ledge 18"),
    ];
    for (at, name) in targets {
        let start = w.total_ticks;
        let ok = w.try_walk_to(t05::walker::Wp { at, act: t05::walker::Act::None, arrive: 0.4 }, name);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[kink] {name}: ok {ok} +{} ticks bond {:?}", w.total_ticks - start, sim.bond.pos);
        if !ok { break; }
    }

    // The per-tick contact scan around the CURRENT position (the last reached point): the
    // capsule (A = pos+0.275, B = pos+(0.55−cc), r 0.55) vs every tri within reach.
    let pos = w.scene.sim.as_ref().unwrap().bond.pos;
    let cc = w.scene.sim.as_ref().unwrap().bond.cc;
    let a = [pos[0], pos[1] + 0.275, pos[2]];
    let b = [pos[0], pos[1] + (0.55 - cc), pos[2]];
    let r = 0.55f32;
    println!("== capsule contacts at bond {:?} (cc {cc:.3}) ==", pos);
    let seg_closest = |p: [f32; 3], a: [f32; 3], b: [f32; 3]| -> [f32; 3] {
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let t = dot3([p[0] - a[0], p[1] - a[1], p[2] - a[2]], ab) / dot3(ab, ab).max(1e-9);
        let t = t.clamp(0.0, 1.0);
        [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
    };
    for w3 in &world {
        // Coarse reject.
        let tl = [0, 1, 2].map(|k| w3.v.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min));
        let th = [0, 1, 2].map(|k| w3.v.iter().map(|p| p[k]).fold(f32::NEG_INFINITY, f32::max));
        if (0..3).any(|k| th[k] < pos[k] - 2.5 || tl[k] > pos[k] + 2.5) { continue; }
        // Closest point on the tri to the capsule segment (vertex/edge sampling).
        let mut best = f32::INFINITY;
        let mut best_q = [0.0f32; 3];
        for vv in &w3.v {
            let q = seg_closest(*vv, a, b);
            let d = dist3(q, *vv);
            if d < best { best = d; best_q = *vv; }
        }
        for ei in 0..3 {
            let (p1, p2) = (w3.v[ei], w3.v[(ei + 1) % 3]);
            for s in 1..8 {
                let t = s as f32 / 8.0;
                let pv = [p1[0] + (p2[0] - p1[0]) * t, p1[1] + (p2[1] - p1[1]) * t, p1[2] + (p2[2] - p1[2]) * t];
                let q = seg_closest(pv, a, b);
                let d = dist3(q, pv);
                if d < best { best = d; best_q = pv; }
            }
        }
        if best < r + 0.02 {
            let nr = tri_normal(&w3.v);
            let dir = [a[0] - best_q[0], a[1] - best_q[1], a[2] - best_q[2]];
            let dl = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-9);
            println!("  dist {best:.3} {} mat {:#04x} pass {} n {:.3} {:.3} {:.3} away-from-tri {:.3} {:.3} {:.3}",
                w3.name, w3.mat, w3.pass as u8, nr[0], nr[1], nr[2], dir[0] / dl, dir[1] / dl, dir[2] / dl);
        }
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }
