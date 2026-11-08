//! stairwell-impacts-render-11: measure 07000011's stairwell player-bullet world impacts against
//! the drawn (render) geometry — the 10-25 report: bullet holes hanging in mid-air along the
//! railing level, on a plane in the well when looking up, and invisible walls eating shots
//! (10-26 "maybe progress?" unresolved either way).
//!
//! Method ([L] measurement, no [S] in the assertions): from each reported stairwell spot, fire a
//! real fan through the real input path (step_bond trigger press → gun → fire_player → the
//! flight → flush_world_impacts), collect every `PlayerImpact::World` point, and compare it with
//! the render mesh: the distance from the hole's point to the nearest drawn World-layer triangle
//! (`Scene.vertices/indices/materials`). A hole with no drawn surface within
//! `HOLE_ON_SURFACE_MAX` hangs in mid-air. The impact's collision face is attributed by
//! re-casting a short segment through the point (`CollideWorld::segment_surface_flags` — the
//! impacts_live idiom) and records the surface byte and the node pass flag, so a hole on an
//! invisible pass-flagged plane (glist 0x10 fudge map43#126, surface bytes 0x40 [D 07000011.bin])
//! or on any other undrawn collision face names itself.
//!
//! Evidence tags: [M] decompile address, [D] disc bytes, [L] live measurement, [S] port choice.

use std::collections::HashMap;

use nf_view::scene::{Layer, Scene};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_d3() -> Scene {
    nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3")
}

/// The level with every guard held where it stands (`Drones::hold_npcs`, the `--hold-npcs` dev
/// gate: the AI, movement, sight and firing passes skip, as with `npc_enabled` clear
/// [M Drone_Control 0x139268:39-42]) so the fan measures Bond's own rounds only. Without the
/// hold, guard 20 wakes during the fan and his rounds add holes whose placement follows his sight
/// timing, not the player's bullet path [L the dump below with NF_STAIR_GUARDS=1].
fn load_d3_bond_only() -> Scene {
    let mut scene = load_d3();
    if std::env::var_os("NF_STAIR_GUARDS").is_none() {
        scene.sim.as_mut().unwrap().drones.hold_npcs = true;
    }
    scene
}

/// A hole further than this from every drawn World triangle hangs in mid-air (the report's
/// symptom). Decal quads sit on their face; edge/skirt geometry tolerates 0.35 [S threshold,
/// measurement-only — kept out of the pass/fail pins below where noted].
const HOLE_ON_SURFACE_MAX: f32 = 0.35;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
#[allow(dead_code)]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn len(a: [f32; 3]) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }

/// Closest point on a triangle to `p` (Ericson, Real-Time Collision Detection 5.1.5) — the
/// probe14_ghost.rs `closest_on_tri` shape.
fn closest_on_tri(p: [f32; 3], t: &[[f32; 3]; 3]) -> [f32; 3] {
    let (a, b, c) = (t[0], t[1], t[2]);
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 { return a; }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 { return b; }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v];
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 { return c; }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [b[0] + sub(c, b)[0] * w, b[1] + sub(c, b)[1] * w, b[2] + sub(c, b)[2] * w];
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        a[0] + ab[0] * v + ac[0] * w,
        a[1] + ab[1] * v + ac[1] * w,
        a[2] + ab[2] * v + ac[2] * w,
    ]
}

/// The drawn World-layer triangles in a coarse spatial hash, for hole-vs-render distances.
struct RenderSurfaces {
    tris: Vec<[[f32; 3]; 3]>,
    grid: HashMap<[i32; 3], Vec<u32>>,
}

const CELL: f32 = 2.0;

impl RenderSurfaces {
    fn build(scene: &Scene) -> RenderSurfaces {
        let mut tris: Vec<[[f32; 3]; 3]> = Vec::new();
        for m in &scene.materials {
            if !matches!(m.layer, Layer::World) { continue; }
            let end = m.first_index + m.index_count;
            let mut i = m.first_index;
            while i + 2 < end {
                let (ia, ib, ic) = (
                    scene.indices[i as usize], scene.indices[i as usize + 1], scene.indices[i as usize + 2],
                );
                let t = [
                    scene.vertices[ia as usize].pos,
                    scene.vertices[ib as usize].pos,
                    scene.vertices[ic as usize].pos,
                ];
                tris.push(t);
                i += 3;
            }
        }
        let mut grid: HashMap<[i32; 3], Vec<u32>> = HashMap::new();
        for (id, t) in tris.iter().enumerate() {
            let lo = [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])];
            let hi = [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])];
            let clo = lo.map(|v| (v / CELL).floor() as i32);
            let chi = hi.map(|v| (v / CELL).floor() as i32);
            let mut n = 0u32;
            for x in clo[0]..=chi[0] {
                for y in clo[1]..=chi[1] {
                    for z in clo[2]..=chi[2] {
                        grid.entry([x, y, z]).or_default().push(id as u32);
                        n += 1;
                        if n > 4096 { break; } // a monster tri: cap the insertion, the 3x3x3 query
                    }                     // below still sees its cells near any real point
                    if n > 4096 { break; }
                }
                if n > 4096 { break; }
            }
        }
        RenderSurfaces { tris, grid }
    }

    /// Distance from `p` to the nearest drawn triangle, scanning the point's cell neighbourhood.
    fn dist(&self, p: [f32; 3]) -> f32 {
        let c = p.map(|v| (v / CELL).floor() as i32);
        let mut best = f32::INFINITY;
        for x in c[0] - 1..=c[0] + 1 {
            for y in c[1] - 1..=c[1] + 1 {
                for z in c[2] - 1..=c[2] + 1 {
                    let Some(ids) = self.grid.get(&[x, y, z]) else { continue };
                    for &id in ids {
                        let d = len(sub(p, closest_on_tri(p, &self.tris[id as usize])));
                        if d < best { best = d; }
                    }
                }
            }
        }
        best
    }
}

/// The reported spots [D the 10-25 report and the pinned tests]: the repro spot, the second
/// spot's frame-2944 placement, the wake-plate spot, the upper landing of the frame-1258 view.
const SPOTS: [(&str, [f32; 3]); 4] = [
    ("repro_15.33_26.69_1.33", [15.33, 26.69, 1.33]),
    ("frame2944_22.40_30.09_1.42", [22.40, 30.09, 1.42]),
    ("plate_22.34_29.37_3.52", [22.34, 29.37, 3.52]),
    ("upper_15.35_35.42_1.61", [15.35, 35.42, 1.61]),
];

/// One fired round's world impacts with their face + render attribution.
struct ImpactRow {
    spot: &'static str,
    yaw: f32,
    pitch: f32,
    p: [f32; 3],
    mat: u8,
    node_pass: bool,
    normal: [f32; 3],
    face_t: f32,
    render_dist: f32,
}

fn aim_dir(yaw: f32, pitch: f32) -> [f32; 3] {
    let p = pitch * std::f32::consts::FRAC_PI_2;
    let (sp, cp) = (p.sin(), p.cos());
    let (sy, cy) = (yaw.sin(), yaw.cos());
    [cp * sy, sp, cp * cy]
}

/// Fire one round from `spot_name`'s position at (yaw, pitch) through the real input path and
/// return the world impacts it produced (with collision-face attribution) plus the raw decal
/// points — the visible holes — for the caller's render measurement.
fn fire_and_collect(
    scene: &mut Scene, spot_name: &'static str, pos: [f32; 3], yaw: f32, pitch: f32,
) -> (Vec<ImpactRow>, Vec<[f32; 3]>) {
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = pos;
        sim.bond.vel = [0.0; 3];
        // Aim through the real fields (the stairwell_bullets encoding).
        let dir = aim_dir(yaw, pitch);
        sim.bond.yaw = yaw;
        sim.bond.pitch = pitch;
        let _ = dir;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 12;
        sim.gun.pending = 0;
        // The clip/spare live in Bond's inventory and re-sync into gun.clip every tick
        // [M the BLData +0x1B8/+0x170 slots, scene.rs's per-tick sync] — refill every slot so
        // the 160-shot fan never dries the magazine (harness ammo, not a sim rule).
        for c in sim.drones.inventory.clip.iter_mut() { *c = 12; }
        for c in sim.drones.inventory.carried.iter_mut() { *c = 48; }
    }
    let mut input = nf_game::player::Input::default();
    input.fire = true;
    let bullets_at_fire = scene.sim.as_ref().unwrap().drones.player_bullets.len();
    let decals_at_fire = scene.sim.as_ref().unwrap().drones.effects.decals.len();
    // The new holes by creation serial, not by Vec index: once the 200-node pool is full every
    // new hole evicts the oldest [M Effect_Bullet 0x1cea20..0x1cea58], the Vec length stays 200
    // and an index slice from `decals_at_fire` would be empty for the rest of the fan.
    let made_at_fire = scene.sim.as_ref().unwrap().drones.effects.made;
    scene.sim.as_mut().unwrap().step_bond(input, true, false, false);
    let spawned = scene.sim.as_ref().unwrap().drones.player_bullets.len() > bullets_at_fire;
    let mut rows = Vec::new();
    let mut max_bullets = 0usize;
    for _ in 0..60u32 {
        max_bullets = max_bullets.max(scene.sim.as_ref().unwrap().drones.player_bullets.len());
        let impacts: Vec<[f32; 3]> = {
            let sim = scene.sim.as_ref().unwrap();
            sim.drones
                .player_impacts
                .iter()
                .filter_map(|im| match im {
                    nf_game::drone::PlayerImpact::World(p) => Some(*p),
                    _ => None,
                })
                .collect()
        };
        if !impacts.is_empty() {
            let (head, dir) = {
                let sim = scene.sim.as_ref().unwrap();
                let head = sim.bond.head();
                let d = sub(impacts[0], head);
                let l = len(d).max(1e-6);
                (head, [d[0] / l, d[1] / l, d[2] / l])
            };
            for p in impacts {
                // Attribute the face: a short segment through the hole along the shot ray.
                let front = [p[0] - dir[0] * 0.3, p[1] - dir[1] * 0.3, p[2] - dir[2] * 0.3];
                let back = [p[0] + dir[0] * 0.3, p[1] + dir[1] * 0.3, p[2] + dir[2] * 0.3];
                let (t, n, mat, node_pass) = scene
                    .sim
                    .as_ref()
                    .unwrap()
                    .data()
                    .world
                    .segment_surface_flags(front, back, false)
                    .unwrap_or((0.5, [0.0; 3], 0, false));
                rows.push(ImpactRow {
                    spot: spot_name,
                    yaw,
                    pitch,
                    p,
                    mat,
                    node_pass,
                    normal: n,
                    face_t: t,
                    render_dist: f32::NAN, // filled by the caller (needs &Scene while sim borrowed)
                });
            }
            let _ = head;
            break;
        }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    // Settle: clear the gun's fire interval so the NEXT press isn't eaten by it.
    for _ in 0..25 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!(
        "[fire] {} yaw {:+.2} pitch {:+.2} spawned {} maxb {} clip {} pending {} reload {} mode {} mission {:?} impacts {} decals {}",
        spot_name, yaw, pitch, spawned, max_bullets, sim.gun.clip, sim.gun.pending,
        sim.gun.reload_state, sim.bond.mode, sim.drones.mission.status(), rows.len(),
        sim.drones.effects.made.wrapping_sub(made_at_fire)
    );
    let _ = decals_at_fire;
    // The DECAL is the visible hole: report the new ones with their positions so the caller
    // can measure them against the render mesh too.
    let fresh: Vec<&nf_game::effect::Decal> =
        sim.drones.effects.decals.iter().filter(|d| d.serial.wrapping_sub(made_at_fire) < 0x8000_0000).collect();
    let new_decals: Vec<[f32; 3]> = fresh.iter().map(|d| d.pos).collect();
    for d in &fresh {
        let host = match &d.host {
            nf_game::effect::DecalHost::Cel(c) => format!("cel {c:?}"),
            nf_game::effect::DecalHost::Object { id, .. } => format!("object {id:#x}"),
        };
        eprintln!("[decal] {} yaw {:+.2} pitch {:+.2} p ({:.3},{:.3},{:.3}) host {host}", spot_name, yaw, pitch, d.pos[0], d.pos[1], d.pos[2]);
    }
    (rows, new_decals)
}

/// THE MEASUREMENT (assertion-free): fire the fan from every reported spot — across the well
/// (pitch 0), up the well (+) and down the well (−), 8 yaws each — and print every world impact
/// with its surface byte, node pass flag, normal and the distance to the nearest drawn triangle.
/// Mid-air rows (render_dist > HOLE_ON_SURFACE_MAX) are the 10-25 report's floating holes.
#[test]
#[ignore = "assertion-free measurement dump; run with --ignored --nocapture"]
fn dump_the_stairwell_impacts_vs_render() {
    let mut scene = load_d3_bond_only();
    let render = RenderSurfaces::build(&scene);
    eprintln!(
        "[impacts] render World triangles {} grid cells {}",
        render.tris.len(),
        render.grid.len()
    );
    let mut all: Vec<ImpactRow> = Vec::new();
    let mut all_decals: Vec<[f32; 3]> = Vec::new();
    let pitches: [f32; 5] = [0.9, 0.5, 0.0, -0.5, -0.9];
    for (name, pos) in SPOTS {
        for k in 0..8u32 {
            let yaw = k as f32 * std::f32::consts::FRAC_PI_4;
            for &pitch in &pitches {
                let (rs, ds) = fire_and_collect(&mut scene, name, pos, yaw, pitch);
                all.extend(rs);
                all_decals.extend(ds);
            }
        }
    }
    let render = RenderSurfaces::build(&scene);
    let mut mid_air = 0usize;
    for r in &mut all {
        r.render_dist = render.dist(r.p);
        if r.render_dist > HOLE_ON_SURFACE_MAX { mid_air += 1; }
    }
    eprintln!("[impacts] {} world impacts, {} mid-air (> {HOLE_ON_SURFACE_MAX} from any drawn triangle)", all.len(), mid_air);
    let mut mid_air_decals = 0usize;
    for p in &all_decals {
        let d = render.dist(*p);
        if d > HOLE_ON_SURFACE_MAX {
            mid_air_decals += 1;
            eprintln!("[decals] MID-AIR hole p ({:.3},{:.3},{:.3}) render_dist {:.3}", p[0], p[1], p[2], d);
        }
    }
    eprintln!("[decals] {} decals total, {} mid-air", all_decals.len(), mid_air_decals);
    for r in &all {
        eprintln!(
            "[impacts] {} yaw {:+.2} pitch {:+.2} p ({:.3},{:.3},{:.3}) mat {:#04x} node_pass {} n ({:.2},{:.2},{:.2}) face_t {:.2} render_dist {:.3}{}",
            r.spot, r.yaw, r.pitch, r.p[0], r.p[1], r.p[2], r.mat, r.node_pass,
            r.normal[0], r.normal[1], r.normal[2], r.face_t, r.render_dist,
            if r.render_dist > HOLE_ON_SURFACE_MAX { "  <-- MID-AIR" } else { "" }
        );
    }
}

/// The pinned shape — Bond's own rounds only (`load_d3_bond_only`: every guard held), so the fan
/// measures the player's bullet path and nothing a guard's sight timing moves:
///
/// 1. Every world impact whose face is NOT a pass face sits ON drawn geometry (≤ HOLE_ON_SURFACE_MAX
///    from the nearest drawn World triangle) [L this test].
/// 2. The off-surface COLLISION is the pass faces, invisible: the stairwell's declared fudge
///    planes (map43#126 "stairwell_blocking_ploys … a fudge!!!!" [D], surface byte 0x40
///    [D 07000011.bin] at x≈15.95/x≈21.78) — the walk forces them to pass
///    [M Bullet_CollisionHandler 0x12ce80 0x12cf7c: `HITDATA+0x50 & 0x40 → pass`], and their
///    `mat & 0x3f == 0` makes `Effect_Create` return before any effect, the decal included
///    [M Effect_Create 0x1ce688: the `(HITDATA+0x50 & 0x3f) != 0` gate]. The fan still stops on
///    them (6 pass-face records measured [L]) — but under the game's convergence law
///    [M Player_WeaponInitBullet 0x1a4bf0:154-164 — the aiming point snapped to what the
///    crosshair ray hits, the round fired from the gun converging on it] every round's stop is
///    aimed at a point ON drawn geometry, so the fan's 0x40 records hug the mesh: the
///    pre-convergence fan (0e0cfdd01 [L]) measured 8 of 158 pass records off-surface (worst
///    2.244, the x≈21.78 plane at y≈34.8); the converged fan measures all 6 in-band (worst
///    0.200) — records moved onto the mesh, NEVER a hole off it (assertion 3 unchanged: 272
///    decals, 0 mid-air, identical count both sides [L]). The sister check that the distance
///    test can fail therefore probes the declared fudge plane itself: a point the collision
///    query answers with the bare 0x40 face at (21.782, 34.802, 1.415) — 2.139 from the drawn
///    mesh [L 0e0cfdd01] — must still measure > HOLE_ON_SURFACE_MAX, so a mesh that draws over
///    the bare plane or a measure collapsed to always-0 names itself.
/// 3. NO hole lies off the drawn mesh. The game puts a bullet hole exactly on the collision face the
///    round hit: node position = `HITDATA+0x40 + HITDATA+0x10 · 0.01` [M Effect_Bullet 0x1ce8a8,
///    the 0x3c23d70a scale], linked to the hit object when `HITDATA+0x54 == 0` (a door leaf moves
///    with its holes). So a floating hole needs a decal-making (`mat & 0x3f != 0`) collision face
///    with nothing drawn on it; Bond's fan from the four reported spots hits none: 272 holes, every
///    one ≤ HOLE_ON_SURFACE_MAX from a drawn static World triangle, none on a door [L this test].
///    The earlier pin's single off-mesh hole (23.219, 28.226, 5.772), later (23.169, 28.299,
///    5.772), was not Bond's: with the guards held no hole lands there, and with them live the fan's
///    only door-hosted hole is a guard round on door #196's leaf at z = 5.772 (DOOR_Stairwell_1,
///    dynamic id door 7) whose spot moves with the guard's sight timing — a drawn door, so not
///    floating either [L the dump with NF_STAIR_GUARDS=1].
#[test]
fn every_stairwell_impact_sits_on_drawn_geometry() {
    let mut scene = load_d3_bond_only();
    let mut rows = Vec::new();
    let mut decals = Vec::new();
    let pitches: [f32; 5] = [0.9, 0.5, 0.0, -0.5, -0.9];
    for (name, pos) in SPOTS {
        for k in 0..8u32 {
            let yaw = k as f32 * std::f32::consts::FRAC_PI_4;
            for &pitch in &pitches {
                let (rs, ds) = fire_and_collect(&mut scene, name, pos, yaw, pitch);
                rows.extend(rs);
                decals.extend(ds);
            }
        }
    }
    let render = RenderSurfaces::build(&scene);
    assert!(!rows.is_empty(), "the fan produced world impacts at all (the gun fired; [L])");
    assert!(decals.len() >= 200, "the fan left its holes (272 measured [L]); got {}", decals.len());
    // (1) every NON-pass-face impact on drawn geometry; the pass-face records counted (they are
    // invisible, not faults — see the doc's (2)).
    let mut worst: Option<(&ImpactRow, f32)> = None;
    let mut pass_rows = 0usize;
    for r in &rows {
        let d = render.dist(r.p);
        if r.mat & 0x40 != 0 || r.node_pass {
            pass_rows += 1;
            continue; // the fudge-plane records: invisible (no decal, `mat & 0x3f == 0`)
        }
        if worst.is_none_or(|(_, wd)| d > wd) {
            worst = Some((r, d));
        }
    }
    let (r, d) = worst.expect("the fan produced at least one non-pass-face world impact");
    assert!(
        d <= HOLE_ON_SURFACE_MAX,
        "mid-air impact on a non-pass face: {} yaw {:+.2} pitch {:+.2} hole ({:.3},{:.3},{:.3})          mat {:#04x} node_pass {} is {:.3} from the nearest drawn triangle (max {HOLE_ON_SURFACE_MAX})",
        r.spot, r.yaw, r.pitch, r.p[0], r.p[1], r.p[2], r.mat, r.node_pass, d
    );
    // (2) the fan still exercises the pass faces, and the render measure CAN fail — probed on
    // the DECLARED fudge plane [D map43#126, surface byte 0x40, x≈21.78], not on the fan's own
    // shape: the game's convergence law [M 0x1a4bf0:154-164] aims every round at the point the
    // crosshair ray hits — ON drawn geometry — so the fan's 0x40 stops hug the mesh (8
    // off-surface pass records pre-convergence [L 0e0cfdd01], 6 in-band after) and the old
    // fan-shaped sister premise died with it. The probe point answers the collision query with
    // the bare 0x40 face and sits 2.139 from the drawn mesh [L 0e0cfdd01] — if the mesh ever
    // draws over the bare plane, or the measure collapses to always-0, this names it.
    assert!(pass_rows >= 1, "the fan no longer stops on a 0x40 pass face (6 measured [L]); got {pass_rows}");
    let probe = [21.782_f32, 34.802, 1.415];
    let (_t, _n, pmat, ppass) = scene
        .sim
        .as_ref()
        .unwrap()
        .data()
        .world
        .segment_surface_flags(
            // the face's normal is (+1,0,0) [L the fan's rows] and the query culls back-faces —
            // cast east→west, the shooter's side.
            [probe[0] + 0.3, probe[1], probe[2]],
            [probe[0] - 0.3, probe[1], probe[2]],
            false,
        )
        .expect("the declared x≈21.78 fudge plane answers the probe segment [D map43#126]");
    assert!(
        pmat & 0x40 != 0 || ppass,
        "the probe point lost its fudge face: mat {pmat:#04x} node_pass {ppass}"
    );
    let pd = render.dist(probe);
    assert!(
        pd > HOLE_ON_SURFACE_MAX,
        "the render measure can no longer flag an off-surface point: the declared fudge plane at {probe:?} measures {pd:.3} from the drawn mesh (2.139 [L 0e0cfdd01])"
    );
    // (3) no hole off the drawn mesh.
    let mid_air: Vec<([f32; 3], f32)> = decals
        .iter()
        .map(|p| (*p, render.dist(*p)))
        .filter(|(_, d)| *d > HOLE_ON_SURFACE_MAX)
        .collect();
    assert!(mid_air.is_empty(), "Bond's rounds left a hole off the drawn mesh: {mid_air:?}");
}
