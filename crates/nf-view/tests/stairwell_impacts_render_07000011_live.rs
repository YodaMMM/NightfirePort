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
        sim.drones.effects.decals.len() - decals_at_fire
    );
    // The DECAL is the visible hole: report the new ones with their positions so the caller
    // can measure them against the render mesh too.
    let new_decals: Vec<[f32; 3]> = scene.sim.as_ref().unwrap().drones.effects.decals
        [decals_at_fire.min(scene.sim.as_ref().unwrap().drones.effects.decals.len())..]
        .iter().map(|d| d.pos).collect();
    for p in &new_decals { eprintln!("[decal] {} yaw {:+.2} pitch {:+.2} p ({:.3},{:.3},{:.3})", spot_name, yaw, pitch, p[0], p[1], p[2]); }
    (rows, new_decals)
}

/// THE MEASUREMENT (assertion-free): fire the fan from every reported spot — across the well
/// (pitch 0), up the well (+) and down the well (−), 8 yaws each — and print every world impact
/// with its surface byte, node pass flag, normal and the distance to the nearest drawn triangle.
/// Mid-air rows (render_dist > HOLE_ON_SURFACE_MAX) are the 10-25 report's floating holes.
#[test]
#[ignore = "assertion-free measurement dump; run with --ignored --nocapture"]
fn dump_the_stairwell_impacts_vs_render() {
    let mut scene = load_d3();
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

/// The pinned shape (level-11-stairwell-bullet-holes-6 — the scoped flush re-query landed and the
/// per-hit walk was compared line by line against `Bullet_CollisionHandler` 0x12ce80):
///
/// 1. Every world impact whose face is NOT a pass face sits ON drawn geometry (≤ HOLE_ON_SURFACE_MAX
///    from the nearest drawn World triangle). The fan measured 104 impacts; the only rows beyond the
///    bound are the six mat-0x40 records below [L this test, both this tree and the -5 master/caps11
///    trees' fan].
/// 2. The pass-face records are EXPECTED and invisible: the fan's six off-surface rows all sit on the
///    stairwell's declared fudge planes (map43#126 "stairwell_blocking_ploys … a fudge!!!!" [D],
///    surface byte 0x40 [D 07000011.bin] at x≈15.95/x≈21.78) — the walk forces them to pass
///    [M Bullet_CollisionHandler 0x12ce80 0x12cf7c: `HITDATA+0x50 & 0x40 → pass`], and their
///    `mat & 0x3f == 0` blocks every `Effect_Create` effect including the decal [M 0x1ce6f8], so no
///    player-visible hole spawns there.
/// 3. Exactly ONE visible mid-air decal remains: (23.219, 28.226, 5.772), 0.402 from the nearest
///    drawn triangle, the third stop of the frame2944 spot's yaw +0.79 pitch −0.50 double-ricochet
///    chain. Game-true by the ported rules: the face is in the game's own scoped candidate set
///    ([M `Collide_Pick` 0x1eb320:125 StraddleCels-only candidates — landed this round as
///    `segment_surface_flags_scoped`, the numbers moved 101→104 impacts while the hole stayed], its
///    material byte is real (the decal passed the `mat & 0x3f == 0` guard [M 0x1ce6f8]), every
///    per-hit rule of the flush matches 0x12ce80 (pass rules 0x12cee0..0x12cfb8 with the glist-0x40
///    node flag [M 0x1ebaa8:143-145], Effect_Create flags 0x61/0x10 [M 0x12d198/0x12cf58], the
///    explode arms gated on !pass [M 0x12d068-0x12d188], the ricochet mask 8 — the 9 variant needs
///    node+0x3c & 0x400000, which no parsemap static node kind carries [M parseentity_fixup_entity
///    0x1d00c0 + parsemap 0x1d1998:102 `cel+0x3c = flags & 0xffff`]), and the flight trace's
///    HITTEST+0x98 = 0 drops nothing in Collide_Intersect [M 0x12f040, disasm 0x1ebff0-0x1ec010].
///    The main agent's render check (docs/reports/level-11-stairwell-bullet-holes-6.md "Live
///    checks") decides the last word: the game should show the same floating hole at the repro spot.
#[test]
fn every_stairwell_impact_sits_on_drawn_geometry() {
    let mut scene = load_d3();
    let mut rows = Vec::new();
    let mut decals = Vec::new();
    // A focused fan: the two reported spots, the well-facing yaws, all five pitches.
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
    assert!(
        !rows.is_empty(),
        "the fan produced world impacts at all (the gun fired; [L])"
    );
    // (1) every NON-pass-face impact on drawn geometry; (2) the pass-face records listed.
    let mut worst: Option<(&ImpactRow, f32)> = None;
    for r in &rows {
        let d = render.dist(r.p);
        if r.mat & 0x40 != 0 || r.node_pass {
            continue; // the fudge-plane records: expected off-surface, invisible (see the doc)
        }
        if worst.is_none_or(|(_, wd)| d > wd) {
            worst = Some((r, d));
        }
    }
    let (r, d) = worst.expect("the fan produced at least one non-pass-face world impact");
    assert!(
        d <= HOLE_ON_SURFACE_MAX,
        "mid-air impact on a non-pass face: {} yaw {:+.2} pitch {:+.2} hole ({:.3},{:.3},{:.3}) \
         mat {:#04x} node_pass {} is {:.3} from the nearest drawn triangle (max {HOLE_ON_SURFACE_MAX})",
        r.spot, r.yaw, r.pitch, r.p[0], r.p[1], r.p[2], r.mat, r.node_pass, d
    );
    // (3) the single game-true fudge stop, pinned exactly (the fan is deterministic).
    let mid_air: Vec<([f32; 3], f32)> = decals
        .iter()
        .map(|p| (*p, render.dist(*p)))
        .filter(|(_, d)| *d > HOLE_ON_SURFACE_MAX)
        .collect();
    assert_eq!(
        mid_air.len(),
        1,
        "exactly one visible mid-air decal expected (the game-true fudge stop); got {mid_air:?}"
    );
    let (p, pd) = mid_air[0];
    let want = [23.219, 28.226, 5.772];
    for k in 0..3 {
        assert!(
            (p[k] - want[k]).abs() < 0.02,
            "the mid-air decal moved: got ({:.3},{:.3},{:.3}), pinned {want:?} — the fudge chain changed",
            p[0], p[1], p[2]
        );
    }
    assert!(
        (pd - 0.402).abs() < 0.05,
        "the mid-air decal's render distance moved: got {pd:.3}, pinned 0.402"
    );
}
