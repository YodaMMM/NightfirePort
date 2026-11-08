//! Probe: which 07000001 guards gain line of sight on Bond near (11.43, 6.47, -3.51), and whether
//! the game's cel walk (`AINetwork_TestRayCels` 0x15ab78) would let that ray through. Prints only.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;

#[test]
#[ignore = "probe; run with --ignored"]
fn probe_01_sight_through_walls() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut w = walker::Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load"));
    w.wait_enabled(6000, "level start");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::new([11.43, 6.47, -3.51], 0.0);
    }
    let mut reported = std::collections::BTreeSet::new();
    for t in 0..900u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = sim.data();
        let world = d.world;
        let bones = sim.drones.bond_view_bones;
        let bpos = sim.bond.pos;
        for (i, g) in sim.drones.list.iter().enumerate() {
            if !g.los || !reported.insert(i) { continue; }
            let eye = g.head();
            let pts: Vec<[f32; 3]> = match bones { Some(b) => b.iter().map(|&p| nf_game::drone::clamp_to_cylinder(bpos, p)).collect(), None => vec![[bpos[0], bpos[1] + 0.5, bpos[2]]] };
            eprintln!("t{t} guard {i} skin {:08x} state {:#x} pos {:?} eye {:?} dist {:.2} bone cursor {}", g.skin, g.state, g.pos, eye, g.dist, g.view_bone);
            for (k, p) in pts.iter().enumerate() {
                let fwd = world.segment_clear(*p, eye);
                let back = world.segment_clear(eye, *p);
                let (ce, cb) = (world.find_cel(eye), world.find_cel(*p));
                let reach = match (ce, cb) { (Some(a), Some(b)) => a == b || world.debug_ray_cels(b, *p, eye).contains(&a), _ => false };
                eprintln!("   bone {k} at {:?}: clear bone->eye {fwd} eye->bone {back}; cel eye {ce:?} bone {cb:?}; cel walk reaches {reach}", p);
            }
            for (k, p) in pts.iter().enumerate() {
                let lo = [p[0].min(eye[0]) - 0.1, p[1].min(eye[1]) - 0.1, p[2].min(eye[2]) - 0.1];
                let hi = [p[0].max(eye[0]) + 0.1, p[1].max(eye[1]) + 0.1, p[2].max(eye[2]) + 0.1];
                let all: Vec<_> = world.tris_near(lo, hi).into_iter().filter(|t| hits(*p, eye, t)).collect();
                for t in &all { eprintln!("   bone {k} line crosses static tri {:?}", t); }
                if all.is_empty() { eprintln!("   bone {k}: the whole line crosses no static triangle, either face"); }
            }
            let p = pts[0];
            for (di, dr) in sim.drones.doors.list.iter().enumerate() {
                if seg_d2(p, eye, dr.pos) > 3.0f32.powi(2) { continue; }
                let lo = [dr.pos[0] - 3.0, dr.pos[1] - 3.0, dr.pos[2] - 3.0];
                let hi = [dr.pos[0] + 3.0, dr.pos[1] + 3.0, dr.pos[2] + 3.0];
                let tris = world.dynamic_tris_near(lo, hi);
                let stat = world.tris_near(lo, hi);
                let cross_dyn = tris.iter().filter(|t| hits(p, eye, t)).count();
                let cross_stat = stat.iter().filter(|t| hits(p, eye, t)).count();
                eprintln!("   door {di} at {:?} kind {} state {:#x} pos01 {:.2}: dynamic tris {} crossed (either face) {}; static tris crossed {}", dr.pos, dr.kind, dr.state, dr.pos01, tris.len(), cross_dyn, cross_stat);
            }
        }
    }
    eprintln!("bond ended at {:?}, cel {:?}", w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.cel);
}

fn seg_d2(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[2] - a[2]];
    let ap = [p[0] - a[0], p[2] - a[2]];
    let l2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if l2 > 0.0 { ((ap[0] * ab[0] + ap[1] * ab[1]) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let q = [a[0] + ab[0] * t - p[0], a[2] + ab[1] * t - p[2]];
    q[0] * q[0] + q[1] * q[1]
}

/// Segment against a triangle, either face (Moller-Trumbore).
fn hits(a: [f32; 3], b: [f32; 3], t: &[[f32; 3]; 3]) -> bool {
    let sub = |x: [f32; 3], y: [f32; 3]| [x[0] - y[0], x[1] - y[1], x[2] - y[2]];
    let cross = |x: [f32; 3], y: [f32; 3]| [x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0]];
    let dot = |x: [f32; 3], y: [f32; 3]| x[0] * y[0] + x[1] * y[1] + x[2] * y[2];
    let d = sub(b, a);
    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
    let pv = cross(d, e2);
    let det = dot(e1, pv);
    if det.abs() < 1e-12 { return false; }
    let inv = 1.0 / det;
    let s = sub(a, t[0]);
    let u = dot(s, pv) * inv;
    if !(0.0..=1.0).contains(&u) { return false; }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 { return false; }
    let tt = dot(e2, q) * inv;
    (0.0..=1.0).contains(&tt)
}
