//! standins-door-933: the aim ray's door pick is the game's front-face MESH ray
//! (`Collide_RayIntersect` 0x1ec690 → `Collide_Intersect` 0x1ebaa8:157-229 →
//! `Intersect_RayGeom` 0x10a8c8:168-172), not the deleted world-sphere + inside-clamp
//! stand-in. Real doors through `nf_view::load`; the pick is exercised at the getter
//! (`Doors::pick_target`) with a crafted camera pose — the scene's Check_Target block calls
//! this getter with the camera, and the wall-segment gate in front of 07000001's doors makes
//! the hint label unreachable through the full tick path (doors_remainder's probe). Opt-in: NF_ISO.

#[path = "common/walker.rs"]
mod walker;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn norm_h(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[2] * v[2]).sqrt();
    if l < 1e-4 { [0.0, 0.0, 0.0] } else { [v[0] / l, 0.0, v[2] / l] }
}

/// Row-vector pose (translation in row 3) — the game's `RotPreTransVec` [M 0x1ebaa8:161].
fn pose(m: &[[f32; 4]; 4], v: [f32; 3]) -> [f32; 3] {
    let mut o = [m[3][0], m[3][1], m[3][2]];
    for k in 0..3 { for c in 0..3 { o[c] += v[k] * m[k][c]; } }
    o
}

/// The first shape-backed door: a resolved glist (the ray only tests glist-backed objects
/// [M Collide_PickObj 0x1eae80:88-90]) whose collision triangles are in the world's shapes
/// table, with a non-degenerate live sphere.
fn shape_backed_door(w: &Walker) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.doors.list.iter().position(|d| {
        d.glist != 0 && d.sph.1 > 0.5 && sim.world.has_shape(d.glist)
    })
}

/// The through/span axes and the eye parked INSIDE the door's live world sphere but OUTSIDE
/// its collision solid along the through-axis (the leaf's thin direction): `off` clears the
/// posed shape's half-extent and stays under the sphere radius.
fn eye_placement(w: &Walker, di: usize) -> ([f32; 3], [f32; 3], [f32; 3], f32) {
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.doors.list[di];
    let (centre, r) = (d.sph.0, d.sph.1);
    // Through-axis = the horizontal matrix row the leaf does NOT span (the span row carries
    // the placement-origin → sphere-centre offset — doors_remainder's `door_axis` probe).
    let off0 = sub(centre, [d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]]);
    let r0 = [d.matrix[0][0], d.matrix[0][1], d.matrix[0][2]];
    let r2 = [d.matrix[2][0], d.matrix[2][1], d.matrix[2][2]];
    let (through, span) = if dot(off0, r0).abs() >= dot(off0, r2).abs() { (norm_h(r2), norm_h(r0)) } else { (norm_h(r0), norm_h(r2)) };
    let tris = sim.world.shape(d.glist).expect("shape-backed door");
    let half_extent = tris.iter()
        .flat_map(|t| t.iter().map(|&v| dot(sub(pose(&d.matrix, v), centre), through).abs()))
        .fold(0.0f32, f32::max);
    let off = (0.5 * r).max(half_extent + 0.15);
    assert!(off < r, "eye offset {off:.2} clears the solid but must stay inside the sphere r {r:.2}");
    let eye = [centre[0] + through[0] * off, centre[1] + through[1] * off, centre[2] + through[2] * off];
    (eye, through, span, off)
}

/// (stand-in-failing) From inside the door's world sphere, a ray running PARALLEL to the leaf
/// plane (along the leaf's span, in front of the near face) crosses no front face — the game's
/// mesh ray reports nothing [M Intersect_RayGeom 0x10a8c8:168: `n·dir >= 0` skips every
/// triangle it could touch]. The deleted stand-in's closest approach sits in front of the eye
/// and inside the sphere, clamped the entry to 0 and reported the door — a false pick that
/// burns `Door_DoMsg`'s one-shot +0x74 latch for a door the player never aimed at.
/// (Verified against the pre-task tree: it returned `(door, 0.0)` here.)
#[test]
fn aim_ray_parallel_to_the_leaf_from_inside_the_sphere_reports_nothing() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).unwrap());
    w.wait_enabled(3000, "level start");
    w.wait(30);
    let di = shape_backed_door(&w).expect("07000001 has a shape-backed door");
    let (eye, _through, span, off) = eye_placement(&w, di);
    // Nudge the eye a step BEHIND the sphere centre along the ray so the stand-in's closest
    // approach lands robustly in front of it (t = +0.05, no float-sign luck) — still inside
    // the sphere, still off the leaf plane.
    let e: f32 = 0.05;
    let eye = [eye[0] - span[0] * e, eye[1] - span[1] * e, eye[2] - span[2] * e];
    eprintln!("[aim-ray] door #{di} eye {eye:?} ({off:.2} off the leaf plane)");
    let sim = w.scene.sim.as_ref().unwrap();
    let pick = sim.drones.doors.pick_target(&sim.world, eye, span, 1.0);
    assert!(pick.is_none(), "a ray from inside the door's sphere running parallel to the leaf \
        must not report the door (got {pick:?}) — the sphere stand-in's inside-clamp did");
}

/// Sister: the same eye aiming AT the leaf reports that door with the mesh hit's own fraction
/// of the 50-unit ray — the front-face hit the game sorts to first [M 0x1ec690:121-135], not
/// the stand-in's clamped 0 (the pre-task tree returned `(door, 0.0)` here). Guards the
/// neighbour the sphere pick got right: a door in front of the eye still takes the pick.
#[test]
fn aim_ray_inside_the_sphere_at_the_leaf_still_reports_the_door() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).unwrap());
    w.wait_enabled(3000, "level start");
    w.wait(30);
    let di = shape_backed_door(&w).expect("07000001 has a shape-backed door");
    let (eye, through, _span, off) = eye_placement(&w, di);
    let sim = w.scene.sim.as_ref().unwrap();
    let toward = [-through[0], -through[1], -through[2]];
    let (di_hit, tf) = sim.drones.doors.pick_target(&sim.world, eye, toward, 1.0)
        .expect("the mesh ray from inside the sphere aiming at the leaf reports the door");
    assert_eq!(di_hit, di, "the nearest front-face hit is the aimed-at door");
    assert!(tf > 0.0 && tf < 1.0, "the hit fraction is the mesh hit's, not the clamp's 0 (got {tf})");
    eprintln!("[aim-ray] door #{di} hit at t={tf:.4} (eye {off:.2} off the plane)");
}
