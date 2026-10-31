//! The two draw rules this task pins on 07000011, through the real input path:
//!
//! 1. A hole on a door rides the leaf [M Effect_Bullet 0x1ce8a8:160-200 — a dynamic hit's hole
//!    node goes on the object's list (`node+0x14 = obj`) with its matrix made object-local;
//!    View_DrawObjectGlist 0x1e6780:66-74 draws it at `node+0x20 · obj+0x90` at every draw, so
//!    the hole moves with the door]. The report's floating holes are 07000011's doors #192 and
//!    #213; the harness door is the stairwell door (placement 13, the walkreal E-opened kind-0
//!    leaf) — the same DecalHost::Object path every door takes.
//! 2. A cel-hosted hole draws only while its cel is in the frame's visible set [M
//!    View_DrawCelGlist 0x1e6538:62-96 — a cel's hole nodes draw with that cel's draw]. The port
//!    collapses the palette entries of holes whose cel the frame's room mask does not draw
//!    (Scene::decal_cell_entries / hide_unseen_decals — the same call the window makes).
//!
//! Evidence tags: [M] decompile address, [D] disc bytes, [L] live measurement, [S] port choice.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_formats::anim::Mat4;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn len(a: [f32; 3]) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }

/// Closest point on a triangle (Ericson 5.1.5) — the stairwell_impacts shape.
fn closest_on_tri(p: [f32; 3], t: &[[f32; 3]; 3]) -> [f32; 3] {
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 { return a; }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, ap));
    let _ = d4;
    if d3 >= 0.0 && d4 <= d3 { return b; }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 { let v = d1 / (d1 - d3); return [a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v]; }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, ap), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 { return c; }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 >= 0.0 { let w = d2 / (d2 - d6); return [a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w]; }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 { let w = (d4 - d3) / ((d4 - d3) + (d5 - d6)); let bc = sub(c, b); return [b[0] + bc[0] * w, b[1] + bc[1] * w, b[2] + bc[2] * w]; }
    let den = 1.0 / (va + vb + vc);
    let (v, w) = (vb * den, vc * den);
    [a[0] + ab[0] * v + ac[0] * w, a[1] + ab[1] * v + ac[1] * w, a[2] + ab[2] * v + ac[2] * w]
}

/// The min distance from `p` to the dynamic collision triangles near it (the moved leaf's faces).
fn dyn_tri_dist(w: &nf_game::world::CollisionWorld, p: [f32; 3], pad: f32) -> f32 {
    let lo = [p[0] - pad, p[1] - pad, p[2] - pad];
    let hi = [p[0] + pad, p[1] + pad, p[2] + pad];
    w.dynamic_tris_near(lo, hi).iter()
        .map(|t| len(sub(p, closest_on_tri(p, t))))
        .fold(f32::INFINITY, f32::min)
}

/// One aimed round through the real input path (the stairwell_impacts harness): the M32 drawn,
/// full clips, the trigger HELD until a decal spawns (the first ticks go to the weapon raise),
/// then settled. Returns nothing; the caller diffs the decal list.
fn fire_one(scene: &mut nf_view::scene::Scene, pos: [f32; 3], yaw: f32, pitch: f32) {
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = pos;
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = yaw;
        sim.bond.pitch = pitch;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 12;
        sim.gun.pending = 0;
        for c in sim.drones.inventory.clip.iter_mut() { *c = 12; }
        for c in sim.drones.inventory.carried.iter_mut() { *c = 48; }
    }
    let input = nf_game::player::Input { fire: true, ..Default::default() };
    for t in 0..30u32 {
        let n0 = scene.sim.as_ref().unwrap().drones.effects.decals.len();
        scene.sim.as_mut().unwrap().step_bond(input, true, false, false);
        let n1 = scene.sim.as_ref().unwrap().drones.effects.decals.len();
        if n1 > n0 { break; }
        if t == 29 { eprintln!("[fire] 30 fire ticks, no decal — the shot missed or the gun never fired"); }
    }
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
}

fn mat_delta(a: &Mat4, b: &Mat4) -> f32 {
    (0..4).map(|i| (0..4).map(|j| (a[i][j] - b[i][j]).abs()).fold(0.0f32, f32::max)).fold(0.0f32, f32::max)
}

/// The stairwell door scene: Bond in the top office before the stairwell door (placement 13,
/// t (13.82, 46.78, 7.09) [L probe_doors]), the M32 drawn.
fn door_scene() -> nf_view::scene::Scene {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::new([12.2, 46.6, 6.5], std::f32::consts::FRAC_PI_2);
    sim.bond.pitch = 0.05;
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(0x12).max(1);
    assert!(sim.drones.inventory.equip_weapon(&wd, 0x12, rounds, true), "the M32 equips");
    sim.drones.inventory.equip_ammo(&wd, 0x12, rounds * 3);
    // The raise/switch runs out before the first shot: idle ticks.
    for _ in 0..30 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    scene
}

/// The main rule: the hole shot into the stairwell door rides the leaf while it E-opens.
#[test]
fn hole_on_the_stairwell_door_rides_the_leaf() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = door_scene();
    // The door's parts before the shot (discovery output for the pinned numbers).
    {
        let sim = scene.sim.as_ref().unwrap();
        let w = sim.data().world;
        for (id, lo, hi) in w.dynamic_boxes() {
            if (lo[0] - 13.8).abs() < 3.0 && (lo[1] - 46.8).abs() < 3.0 && (lo[2] - 7.1).abs() < 3.0 {
                eprintln!("[doorbox] id {id:#x} lo {lo:?} hi {hi:?}");
            }
        }
        let d = &sim.drones.doors.list[0];
        eprintln!("[door] before: state {} pos01 {:.3} t ({:.3},{:.3},{:.3})", d.state, d.pos01, d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]);
    }
    // Aim at the leaf across the doorway (the discovery fan pin): one round.
    let before = scene.sim.as_ref().unwrap().drones.effects.decals.len();
    fire_one(&mut scene, [12.2, 46.6, 6.5], std::f32::consts::FRAC_PI_2, 0.05);
    let sim = scene.sim.as_ref().unwrap();
    let new: Vec<&nf_game::effect::Decal> = sim.drones.effects.decals[before..].iter().collect();
    eprintln!("[shot] {} new decals", new.len());
    for d in &new {
        eprintln!("[shot] hole p ({:.3},{:.3},{:.3}) n ({:.2},{:.2},{:.2}) mesh {:#x} host {:?}", d.pos[0], d.pos[1], d.pos[2], d.normal[0], d.normal[1], d.normal[2], d.mesh, match d.host {
            nf_game::effect::DecalHost::Cel(c) => format!("Cel({c:?})"),
            nf_game::effect::DecalHost::Object { id, .. } => format!("Object({id:#x})"),
        });
    }
    let (p0, host_id, om0, d0, serial) = {
        let hole = new.iter().find(|d| matches!(d.host, nf_game::effect::DecalHost::Object { .. }))
            .expect("the door shot made an object-hosted hole [M Effect_Bullet 0x1ce8a8:160-200]");
        let (p0, host_id) = (hole.pos, match hole.host { nf_game::effect::DecalHost::Object { id, .. } => id, _ => unreachable!() });
        let w = sim.data().world;
        let om0 = w.dynamic_matrix(host_id).expect("the door part is a mover with a live matrix");
        let d0 = dyn_tri_dist(w, p0, 0.6);
        (p0, host_id, om0, d0, hole.serial)
    };
    eprintln!("[follow] host {host_id:#x} p0 {p0:?} leaf dist {d0:.3}");
    assert!(d0 <= 0.03, "the fresh hole sits on the door part's collision face [L], got {d0:.3}");
    drop(sim);
    // E-open the door through the real input path (the walker's Use: face it, hold use).
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.yaw = std::f32::consts::FRAC_PI_2; // face +x, the door
    let pos01_0 = sim.drones.doors.list[0].pos01;
    for t in 0..160u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, true);
        if t % 20 == 19 {
            let d = &sim.drones.doors.list[0];
            eprintln!("[open] t {t} state {} pos01 {:.3}", d.state, d.pos01);
        }
    }
    drop(sim);
    let sim = scene.sim.as_ref().unwrap();
    let d1 = &sim.drones.doors.list[0];
    eprintln!("[door] after: state {} pos01 {:.3}", d1.state, d1.pos01);
    let (om1, p1, d1dist) = {
        let w = sim.data().world;
        let om1 = w.dynamic_matrix(host_id).expect("the door part's matrix after the open");
        let hole1 = sim.drones.effects.decals.iter().find(|d| d.serial == serial).expect("the hole is still live");
        let p1 = hole1.pos;
        (om1, p1, dyn_tri_dist(w, p1, 0.6))
    };
    let md = mat_delta(&om0, &om1);
    eprintln!("[follow] door matrix delta {md:.3}");
    assert!(md > 0.1, "the E-open moved the door part's matrix (delta {md:.3}) — the harness door really opened");
    // The hole followed: its node matrix is object-local, its world position rides obj+0x90.
    let moved = len(sub(p1, p0));
    eprintln!("[follow] p1 {p1:?} moved {moved:.3} leaf dist {d1dist:.3}");
    assert!(moved > 0.15, "the hole moved with the leaf (moved {moved:.3}) — in the pre-fix port it stayed at p0 and floated");
    assert!(d1dist <= 0.03, "the moved hole still sits on the leaf's collision face [L], got {d1dist:.3}");
    // The static sister: the floor hole next to the door never moves.
    let before = sim.drones.effects.decals.len();
    drop(sim);
    fire_one(&mut scene, [12.2, 46.6, 6.5], std::f32::consts::FRAC_PI_2, -1.2); // straight down
    let sim = scene.sim.as_ref().unwrap();
    let floor_hole = sim.drones.effects.decals[before..].iter().find(|d| matches!(d.host, nf_game::effect::DecalHost::Cel(_)))
        .expect("the floor shot made a cel-hosted hole (static world)");
    let q0 = floor_hole.pos;
    let qserial = floor_hole.serial;
    eprintln!("[static] floor hole q0 {q0:?} serial {qserial}");
    drop(sim);
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let q1 = sim.drones.effects.decals.iter().find(|d| d.serial == qserial).expect("the floor hole lives").pos;
    let qd = len(sub(q1, q0));
    eprintln!("[static] q1 {q1:?} moved {qd:.4}");
    assert!(qd < 0.005, "a static-world hole never moves while the door opens (moved {qd:.4}) — the fix must not drag every hole with the door");
    let _ = pos01_0;
}

/// The sister of the ride assert: the measure (hole-to-leaf distance) flags a STUCK hole — the
/// pre-fix port's hole pinned at p0 while the leaf swings away hangs off the moved faces. Fails
/// vacuously if the door never moved (then p0 is still on the faces and nothing is flagged).
#[test]
fn the_follow_measure_flags_a_stuck_hole() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = door_scene();
    let before = scene.sim.as_ref().unwrap().drones.effects.decals.len();
    fire_one(&mut scene, [12.2, 46.6, 6.5], std::f32::consts::FRAC_PI_2, 0.05);
    let sim = scene.sim.as_ref().unwrap();
    let (p0, host_id, om0) = {
        let hole = sim.drones.effects.decals[before..].iter().find(|d| matches!(d.host, nf_game::effect::DecalHost::Object { .. }))
            .expect("the door shot made an object-hosted hole");
        let (p0, host_id) = (hole.pos, match hole.host { nf_game::effect::DecalHost::Object { id, .. } => id, _ => unreachable!() });
        let om0 = sim.data().world.dynamic_matrix(host_id).unwrap();
        (p0, host_id, om0)
    };
    drop(sim);
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.yaw = std::f32::consts::FRAC_PI_2;
    for _ in 0..160u32 { sim.step_bond(nf_game::player::Input::default(), false, false, true); }
    drop(sim);
    let sim = scene.sim.as_ref().unwrap();
    let w = sim.data().world;
    let om1 = w.dynamic_matrix(host_id).unwrap();
    assert!(mat_delta(&om0, &om1) > 0.1, "the door moved (the sister's premise)");
    // The stuck hole: the pre-fix port left the hole at p0.
    let stuck = dyn_tri_dist(w, p0, 0.6);
    eprintln!("[stuck] p0 {p0:?} off the moved leaf by {stuck:.3}");
    assert!(stuck > 0.03, "a hole left at p0 hangs off the moved leaf by {stuck:.3} — the measure flags it");
}

/// The room rule: a cel-hosted hole draws only while its cel is in the frame's visible set
/// [M View_DrawCelGlist 0x1e6538:62-96 — a cel's hole nodes draw with that cel]. The harness:
/// the stairwell spot's shot (the discovery run's hole in the landing's room), then two real
/// walks of the scene graph: the landing eye facing the wall (the hole's cel in the set — the
/// entry draws) and the office spawn's eye (the walk's set, discovery pins whether the hole's
/// cel drops — if it does the entry collapses in the palette, exactly what the window shows).
#[test]
fn a_wall_hole_draws_only_while_its_cel_is_in_the_frame() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::new([16.03, 32.17, -2.39], 0.340);
        sim.bond.pitch = -0.101;
        let wd = sim.weapons.clone();
        let rounds = wd.clip_size(0x12).max(1);
        assert!(sim.drones.inventory.equip_weapon(&wd, 0x12, rounds, true), "the M32 equips");
        sim.drones.inventory.equip_ammo(&wd, 0x12, rounds * 3);
        for _ in 0..30 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    }
    let before = scene.sim.as_ref().unwrap().drones.effects.decals.len();
    fire_one(&mut scene, [16.03, 32.17, -2.39], 0.340, -0.101);
    let sim = scene.sim.as_ref().unwrap();
    let hole = sim.drones.effects.decals[before..].iter().find(|d| matches!(d.host, nf_game::effect::DecalHost::Cel(Some(_))))
        .expect("the stairwell shot made a cel-hosted hole [M the static arm 0x1ce8a8:160-170]");
    let (hp, cel) = (hole.pos, match hole.host { nf_game::effect::DecalHost::Cel(Some(c)) => c, _ => unreachable!() });
    drop(sim);
    let (layout, _) = scene.skin_layout();
    let entries = scene.decal_cell_entries(&layout);
    let entry = entries.iter().find(|(e, c)| *c == cel).expect("the hole's palette entry is listed with its cel").0 as usize;
    eprintln!("[room] hole p {hp:?} cel {cel} entry {entry}");
    // Walk A: the landing eye facing the hole.
    let eye = scene.sim.as_ref().unwrap().bond.head();
    let mut st = nf_view::scene::ViewerCel::default();
    let graph = &scene.cel_graph;
    let cell = nf_view::scene::eye_cell(graph, &mut st, eye, None).expect("the eye cel resolves");
    let vp = view_proj(eye, 0.340, -0.101, 16.0 / 9.0);
    let vis_a = nf_view::scene::portal_visible_cels_from_cell_doors(graph, cell, eye, &depth0_game(vp, eye), None).expect("the walk runs");
    eprintln!("[room] eye cell {cell} facing set has the hole's cel: {}", vis_a[cel as usize]);
    let rooms_a: Vec<bool> = graph.room_to_cell.iter().map(|m| m.map(|c| vis_a[c]).unwrap_or(true)).collect();
    {
        let (pal, _) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), Some(&rooms_a));
        assert!(vis_a[cel as usize], "the facing walk sees the hole's cel");
        assert!(!is_far(&pal[entry]), "the hole draws while its cel is in the frame's visible set [M View_DrawCelGlist 0x1e6538:62-96]");
    }
    // Walk B: the office spawn's eye, facing west (away from the stairwell).
    let eye2 = [-0.40f32, 46.97 + 0.70, 25.83];
    let mut st2 = nf_view::scene::ViewerCel::default();
    let cell2 = nf_view::scene::eye_cell(graph, &mut st2, eye2, None).expect("the office eye cel resolves");
    let yaw2 = std::f32::consts::PI; // face -x, away from the stairwell (east)
    let vp2 = view_proj(eye2, yaw2, 0.0, 16.0 / 9.0);
    let vis_b = nf_view::scene::portal_visible_cels_from_cell_doors(graph, cell2, eye2, &depth0_game(vp2, eye2), None).expect("the walk runs");
    eprintln!("[room] office cell {cell2} away set has the hole's cel: {}", vis_b[cel as usize]);
    let rooms_b: Vec<bool> = graph.room_to_cell.iter().map(|m| m.map(|c| vis_b[c]).unwrap_or(true)).collect();
    let (pal2, _) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), Some(&rooms_b));
    if vis_b[cel as usize] {
        eprintln!("[room] the away walk still sees the hole's cel (one open cel spans both spots) — pinning the draw instead");
        assert!(!is_far(&pal2[entry]), "the entry follows the walk's set");
    } else {
        assert!(is_far(&pal2[entry]), "the hole's entry collapses while its cel is out of the frame's visible set (the pre-fix port drew it)");
    }
    // The sister: with every room visible nothing hides, and the shot's hole has an entry on it
    // (the ricochet chain makes several holes in the same cel — take this hole's own entry).
    let all: Vec<bool> = vec![true; rooms_a.len()];
    let (pal3, _) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), Some(&all));
    let mine = entries.iter().filter(|(_, c)| *c == cel)
        .map(|(e, _)| { let e = *e as usize; let pt = [pal3[e][3][0], pal3[e][3][1], pal3[e][3][2]]; (e, len(sub(pt, hp))) })
        .min_by(|a, b| a.1.total_cmp(&b.1)).expect("a cel-128 entry");
    assert!(mine.1 < 0.02, "the shot's hole has its own drawn entry (nearest {:?} at {:.3})", mine.0, mine.1);
    assert!(!is_far(&pal3[mine.0]), "the all-visible mask never hides the hole");
}

/// The room measure's other edge: hiding is scoped to mapped, unseen cels — a hole whose cel no
/// room maps to (the `Cel(None)` fallback's entries never enter the list, and a mapped cel the
/// mask marks visible is never hidden). Pins hide_unseen_decals' own contract.
#[test]
fn hiding_never_touches_visible_or_unmapped_cels() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3");
    let (layout, _) = scene.skin_layout();
    let mut pal = vec![[[0.0; 4]; 4]; layout.n_mats as usize];
    pal[10] = [[0.0; 4], [0.0; 4], [0.0; 4], [1.0, 2.0, 3.0, 1.0]];
    // entries: (palette entry, cel): an unseen mapped cel hides; a visible mapped cel and an
    // unmapped cel index never do. The mapped cel is room 0's own cel.
    let mapped = scene.cel_graph.room_to_cell.iter().flatten().next().copied().expect("a mapped cel") as u32;
    let entries = [(10u32, mapped), (11, mapped), (12, 9999)];
    let mut vis = vec![false; scene.cel_graph.room_to_cell.len()];
    let hidden = scene.hide_unseen_decals(&mut pal, &entries, &vis);
    assert_eq!(hidden, vec![10, 11], "the unseen mapped cel's entries hid");
    vis[0] = true;
    let mut pal2 = pal.clone();
    let hidden2 = scene.hide_unseen_decals(&mut pal2, &entries, &vis);
    assert!(hidden2.is_empty(), "the visible cel's entry stayed; the unmapped cel's entry stayed");
}

fn is_far(m: &Mat4) -> bool { m[3][1] < -1000.0 }

fn view_proj(pos: [f32; 3], yaw: f32, pitch: f32, aspect: f32) -> [[f32; 4]; 4] {
    use glam::{Mat4, Vec3};
    let (near, far, fov) = (0.05f32, 4000.0f32, 1.0471976f32); // 60°, the render path's GAME_FOV
    let f = 1.0 / (fov * 0.5).tan();
    let proj = Mat4::from_cols_array_2d(&[
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, near * far / (near - far), 0.0],
    ]);
    let cp = pitch.cos();
    let fw = Vec3::new(cp * yaw.sin(), pitch.sin(), cp * yaw.cos()).normalize();
    let s = fw.cross(Vec3::Y).normalize();
    let u = s.cross(fw);
    let view = Mat4::from_cols_array_2d(&[
        [s.x, u.x, -fw.x, 0.0],
        [s.y, u.y, -fw.y, 0.0],
        [s.z, u.z, -fw.z, 0.0],
        [-s.dot(Vec3::from(pos)), -u.dot(Vec3::from(pos)), fw.dot(Vec3::from(pos)), 1.0],
    ]);
    (proj * view).to_cols_array_2d()
}

fn depth0_game(vp: [[f32; 4]; 4], eye: [f32; 3]) -> [[f32; 4]; 5] {
    let planes = nf_view::gpu::extract_frustum(vp);
    let fw = [planes[4][0], planes[4][1], planes[4][2]];
    let fe = fw[0] * eye[0] + fw[1] * eye[1] + fw[2] * eye[2];
    [planes[0], planes[1], planes[2], planes[3], [-fw[0], -fw[1], -fw[2], fe + 200.0]]
}
