//! level-05-west-lane-8 scratch probe: the step shelf's z-46.0 kink stall, per tick.
//! Walks the lane-6 route to the step spine 3 end, then RAW-steers toward spine 4 with NO
//! stuck ladder: every tick prints pos / yaw / grounded / capsule contacts (dist, tri, normal,
//! the game-shaped push unit(cp−centre)·(r−dist) [M Intersect_CylGeom 0x109da0]) — which tri
//! stalls the walk and whether the push carries +y (the game rides it up the ramp).
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

    struct W { v: [[f32; 3]; 3], mat: u8, inst: usize, name: String }
    let mut world: Vec<W> = Vec::new();
    for (idx, inst) in instances.iter().enumerate() {
        let om = &level.maps[inst.map].objects[inst.object];
        let Some(c) = &om.collision else { continue };
        if om.flags & 0x20 != 0 { continue; }
        for (t, m) in c.triangles_with_material() {
            let v = [tx(&inst.matrix, t[0]), tx(&inst.matrix, t[1]), tx(&inst.matrix, t[2])];
            world.push(W { v, mat: m, inst: idx, name: om.name.clone() });
        }
    }

    let tri_normal = |v: &[[f32; 3]; 3]| -> [f32; 3] {
        let (a, b, c) = (v[0], v[1], v[2]);
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 1e-9 { [n[0] / l, n[1] / l, n[2] / l] } else { [0.0; 3] }
    };
    let dot3 = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let seg_closest = |p: [f32; 3], a: [f32; 3], b: [f32; 3]| -> [f32; 3] {
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let t = (dot3([p[0] - a[0], p[1] - a[1], p[2] - a[2]], ab) / dot3(ab, ab).max(1e-9)).clamp(0.0, 1.0);
        [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
    };
    let dist3 = |a: [f32; 3], b: [f32; 3]| (dot3([a[0] - b[0], a[1] - b[1], a[2] - b[2]], [a[0] - b[0], a[1] - b[1], a[2] - b[2]])).sqrt();
    // Closest point on the tri SOLID to p (vertex/edge/face sampling like the game's cyl pick).
    let tri_closest = |w: &W, p: [f32; 3]| -> ([f32; 3], f32) {
        let mut best = f32::INFINITY;
        let mut best_q = [0.0f32; 3];
        for vv in &w.v {
            let q = seg_closest(p, *vv, *vv);
            let d = dist3(q, p);
            if d < best { best = d; best_q = *vv; }
        }
        for ei in 0..3 {
            let (p1, p2) = (w.v[ei], w.v[(ei + 1) % 3]);
            for s in 1..16 {
                let t = s as f32 / 16.0;
                let pv = [p1[0] + (p2[0] - p1[0]) * t, p1[1] + (p2[1] - p1[1]) * t, p1[2] + (p2[2] - p1[2]) * t];
                let d = dist3(pv, p);
                if d < best { best = d; best_q = pv; }
            }
        }
        (best_q, best)
    };
    // Down-ray floor at (x,z) from y 40 (two-sided, any tri).
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

    // 1. The contacts at the spine-3 END position, before any steering.
    let mut w = Walker::new(nf_view::load(&iso_path, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    t05::walk_to_the_slab_east_end(&mut w);
    t05::walk_the_gap_to_the_notch(&mut w);
    let p0 = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[k2] the walked end: tick {} bond {:?}", w.total_ticks, p0);
    let cc = w.scene.sim.as_ref().unwrap().bond.cc;
    let capsule = |pos: [f32; 3]| -> ([f32; 3], [f32; 3], f32) {
        ([pos[0], pos[1] + 0.275, pos[2]], [pos[0], pos[1] + (0.55 - cc), pos[2]], 0.55)
    };
    let contacts = |pos: [f32; 3]| -> Vec<(f32, String, u8, [f32; 3], [f32; 3], f32)> {
        let (a, b, r) = capsule(pos);
        let centre = [pos[0], (a[1] + b[1]) / 2.0, pos[2]];
        let mut out = Vec::new();
        for w3 in &world {
            let tl = [0, 1, 2].map(|k| w3.v.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min));
            let th = [0, 1, 2].map(|k| w3.v.iter().map(|p| p[k]).fold(f32::NEG_INFINITY, f32::max));
            if (0..3).any(|k| th[k] < pos[k] - 2.0 || tl[k] > pos[k] + 2.0) { continue; }
            let (q, d) = tri_closest(w3, [a[0], (a[1] + b[1]) / 2.0, a[2]]);
            // exact: min over the two sphere centres
            let (q2, d2) = tri_closest(w3, b);
            let (q3, d3) = tri_closest(w3, a);
            let (q, d) = if d2 < d { (q2, d2) } else { (q, d) };
            let (q, d) = if d3 < d { (q3, d3) } else { (q, d) };
            if d < r + 0.02 {
                let nr = tri_normal(&w3.v);
                let push = [q[0] - centre[0], q[1] - centre[1], q[2] - centre[2]];
                out.push((d, w3.name.clone(), w3.mat, nr, push, r - d));
            }
        }
        out.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        out
    };
    println!("== contacts at the spine-3 end {:?} ==", p0);
    for (d, name, mat, nr, push, pen) in contacts(p0) {
        let l = (dot3(push, push)).sqrt().max(1e-9);
        println!("  d {d:.3} pen {pen:.3} {} mat {:#04x} n {:.3} {:.3} {:.3} pushdir {:.3} {:.3} {:.3}",
            name, mat, nr[0], nr[1], nr[2], push[0] / l, push[1] / l, push[2] / l);
    }

    // 2. RAW steer through the spine + ledge at FULL forward, per tick, no stuck ladder.
    // The drift [L probe_kink2 run 1]: ungrounded on the ramp, the pushes drift NW ~0.024/tick;
    // at forward 0.45 the drift beats the walk and Bond crosses the base edge. Full forward +
    // a SE (down-slope) aim bias to compensate.
    let targets: [([f32; 3], &str); 16] = [
        ([-23.9, 33.6, 45.8], "spine 4"),
        ([-24.3, 33.6, 45.4], "spine 5"),
        ([-24.7, 33.6, 45.0], "spine 6"),
        ([-25.1, 33.5, 44.6], "spine 7"),
        ([-25.5, 33.4, 44.2], "spine 8"),
        ([-25.8, 33.4, 43.8], "spine 9"),
        ([-26.0, 33.4, 43.2], "ledge bend"),
        ([-26.2, 33.4, 42.6], "ledge neck"),
        ([-26.5, 33.5, 41.6], "ledge nw"),
        ([-26.3, 33.5, 40.6], "ledge 40"),
        ([-25.9, 33.5, 38.6], "ledge 38"),
        ([-24.9, 33.4, 36.5], "ledge 36"),
        ([-24.2, 33.2, 32.5], "ledge 32"),
        ([-23.6, 33.4, 28.5], "ledge 28"),
        ([-23.5, 33.5, 22.5], "ledge 22"),
        ([-23.5, 34.5, 18.5], "ledge 18"),
    ];
    let se_bias: f32 = 0.35; // aim this far SE (down-slope) of each target
    let mut leg = 0;
    let mut ticks_on_leg = 0u32;
    for step in 0..6000u32 {
        if leg >= targets.len() { break; }
        let (at, name) = targets[leg];
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let d = ((p[0] - at[0]).powi(2) + (p[2] - at[2]).powi(2)).sqrt();
        if d < 0.42 && step > 0 {
            eprintln!("[k2] {name} ARRIVED +{ticks_on_leg} ticks bond {:?}", p);
            leg += 1; ticks_on_leg = 0;
            continue;
        }
        // aim SE of the target: (0.709, -0.705) unit * se_bias
        let aim = [at[0] + 0.709 * se_bias, at[1], at[2] - 0.705 * se_bias];
        let desired = (aim[0] - p[0]).atan2(aim[2] - p[2]);
        let err = wrap_pi_k(desired - sim.bond.yaw);
        let input = Input {
            turn: err.clamp(-0.07, 0.07),
            forward: 1.0,
            ..Default::default()
        };
        let fall_y = p[1];
        w.tick(input, false);
        ticks_on_leg += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        let p1 = sim.bond.pos;
        let near_kink = dxz2(p1, [-23.5, 46.5]) < 1.5;
        if near_kink || step % 60 == 0 || ticks_on_leg > 240 {
            let floor = {
                let mut best: Option<f32> = None;
                for w3 in &world {
                    if let Some(h) = ray_hit(w3, p1[0], p1[2]) {
                        if best.map_or(true, |bh| h > bh) { best = Some(h); }
                    }
                }
                best
            };
            let cs = contacts(p1);
            let mut line = format!("[k2 t{}] leg {} p ({:.3},{:.3},{:.3}) yaw {:.2} g {} dy {:+.3}",
                step, targets[leg].1, p1[0], p1[1], p1[2], sim.bond.yaw, sim.bond.grounded as u8, p1[1] - fall_y);
            if let Some(f) = floor { line.push_str(&format!(" floor {:.2}", f)); }
            for (d, name, mat, nr, _push, pen) in cs.iter().take(3) {
                line.push_str(&format!("\n      c d {:.3} pen {:.3} {} mat {:#04x} n {:.2} {:.2} {:.2}", d, pen, name, mat, nr[0], nr[1], nr[2]));
            }
            eprintln!("{}", line);
        }
        if p1[1] < 33.2 {
            eprintln!("[k2] FELL off the shelf at leg {} tick {} bond {:?}", targets[leg].1, step, p1);
            break;
        }
        if ticks_on_leg > 900 {
            eprintln!("[k2] leg {} WEDGED (900 ticks) at {:?}", targets[leg].1, p1);
            break;
        }
    }
    eprintln!("[k2] done at tick {} bond {:?}", w.total_ticks, w.scene.sim.as_ref().unwrap().bond.pos);
}

fn wrap_pi_k(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}
fn dxz2(a: [f32; 3], b: [f32; 2]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[1]).powi(2)).sqrt() }
