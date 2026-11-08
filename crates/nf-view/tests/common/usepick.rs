//! A stand for the use key in front of an object's collision face (the game's pick needs the use
//! point — head + view dir — within 1.0 in front of one of the object's collision triangles and
//! inside its world sphere [M Collide_PickObj 0x1eae80, Collide_Intersect 0x1ebaa8 →
//! Intersect_PointGeom 0x1e92d8; nf_game::usepick]). Tests that press E on an object stand Bond
//! here instead of at the object's origin.
#![allow(dead_code)]

use nf_view::scene::Sim;

type V3 = [f32; 3];
type Mat4 = [[f32; 4]; 4];

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn scale(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn len(a: V3) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }

/// The object's collision triangles posed by `matrix` (empty when the shape is unknown).
pub fn posed(sim: &Sim, hash: u32, matrix: &Mat4) -> Vec<[V3; 3]> {
    sim.world.shape(hash).map(|t| t.iter().map(|tri| tri.map(|p| nf_formats::placement::transform(matrix, p))).collect()).unwrap_or_default()
}

/// Stand Bond in front of one of the faces (wall faces from 1.2-1.8 out, floor faces from
/// 0.3-0.7 out, largest faces first), let the collision settle him, look at the face middle and
/// check the game's pick would keep the object from there. Returns true with Bond standing and
/// aiming (the caller presses E); false when no stand passes.
pub fn stand_in_front(sim: &mut Sim, tris: &[[V3; 3]], sphere: (V3, f32)) -> bool {
    for_each_stand(sim, tris, sphere, &mut |_| true)
}

/// Every stand `stand_in_front` would accept, in its order: `f` runs with Bond standing and
/// aiming, and returns true to stop. Returns whether `f` stopped the walk.
pub fn for_each_stand(sim: &mut Sim, tris: &[[V3; 3]], sphere: (V3, f32), f: &mut dyn FnMut(&mut Sim) -> bool) -> bool {
    let mut faces: Vec<(f32, V3, V3)> = tris.iter().filter_map(|t| {
        let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
        let l = len(n);
        if l < 1e-6 || n[1] / l <= -0.7 { return None; }
        Some((l * 0.5, scale(add(add(t[0], t[1]), t[2]), 1.0 / 3.0), scale(n, 1.0 / l)))
    }).collect();
    faces.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (_, c, n) in faces.into_iter().take(24) {
        let dirs: Vec<V3> = if n[1] >= 0.7 {
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0]]
        } else {
            let h = [-n[0], 0.0, -n[2]];
            vec![scale(h, 1.0 / len(h))]
        };
        let backs: &[f32] = if n[1] >= 0.7 { &[0.5, 0.3, 0.7] } else { &[1.5, 1.2, 1.8] };
        for dir in dirs {
            for &back in backs {
                let (hx, hz) = (c[0] - dir[0] * back, c[2] - dir[2] * back);
                let (top, bot) = ([hx, c[1] + 0.3, hz], [hx, c[1] - 3.5, hz]);
                let Some(fr) = sim.world.segment(top, bot) else { continue };
                let ground = top[1] + (bot[1] - top[1]) * fr;
                let pos = [hx, ground + 1.086, hz];
                sim.bond.pos = pos;
                sim.bond.vel = [0.0; 3];
                for _ in 0..2 { sim.step_bond(Default::default(), false, false, false); }
                if len(sub(sim.bond.pos, pos)) > 0.2 { continue; }
                let to = sub(c, sim.bond.head());
                let pitch = (to[1] / len(to)).asin() / std::f32::consts::FRAC_PI_2;
                if pitch.abs() > 0.9 { continue; }
                sim.bond.yaw = to[0].atan2(to[2]);
                sim.bond.pitch = pitch;
                let u = add(sim.bond.head(), sim.bond.look());
                if len(sub(u, sphere.0)) < 1.0 + sphere.1 && nf_game::world::CollisionWorld::sphere_meets_tris(u, 1.0, tris) {
                    if f(sim) { return true; }
                }
            }
        }
    }
    false
}

/// Stand in front of the object (`stand_in_front`) and pulse E (one press edge per 6 ticks)
/// holding the look, until `done` or `ticks` run out. Returns whether `done` turned true.
pub fn press_on(sim: &mut Sim, tris: &[[V3; 3]], sphere: (V3, f32), ticks: u32, done: &dyn Fn(&Sim) -> bool) -> bool {
    if !stand_in_front(sim, tris, sphere) { return false; }
    pulse(sim, ticks, done)
}

/// Pulse E (one press edge per 6 ticks) holding the current look, until `done` or `ticks` run out.
pub fn pulse(sim: &mut Sim, ticks: u32, done: &dyn Fn(&Sim) -> bool) -> bool {
    let (y, p) = (sim.bond.yaw, sim.bond.pitch);
    for k in 0..ticks {
        sim.bond.yaw = y;
        sim.bond.pitch = p;
        sim.step_bond(Default::default(), false, false, k % 6 == 0);
        if done(sim) { return true; }
    }
    false
}

/// A script entity's posed collision triangles and a bounding sphere of them (the first live
/// entity carrying glist `hash`).
pub fn entity(sim: &Sim, hash: u32) -> Option<(Vec<[V3; 3]>, (V3, f32))> {
    let m = sim.drones.scripts.objects.list.iter().find_map(|o| match o {
        Some(nf_game::nis::Obj::Entity { hash: h, matrix, .. }) if *h == hash => Some(*matrix),
        _ => None,
    })?;
    let tris = posed(sim, hash, &m);
    if tris.is_empty() { return None; }
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in tris.iter().flatten() { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
    let c = scale(add(lo, hi), 0.5);
    Some((tris, (c, len(sub(hi, c)))))
}

/// `entity`, for the live instance of glist `hash` whose translation is nearest `near`.
pub fn entity_near(sim: &Sim, hash: u32, near: V3) -> Option<(Vec<[V3; 3]>, (V3, f32))> {
    let m = sim.drones.scripts.objects.list.iter().filter_map(|o| match o {
        Some(nf_game::nis::Obj::Entity { hash: h, matrix, .. }) if *h == hash => Some(*matrix),
        _ => None,
    }).min_by(|a, b| {
        let d = |m: &Mat4| len(sub([m[3][0], m[3][1], m[3][2]], near));
        d(a).partial_cmp(&d(b)).unwrap()
    })?;
    let tris = posed(sim, hash, &m);
    if tris.is_empty() { return None; }
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in tris.iter().flatten() { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
    let c = scale(add(lo, hi), 0.5);
    Some((tris, (c, len(sub(hi, c)))))
}
