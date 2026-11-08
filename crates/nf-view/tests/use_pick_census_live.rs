//! use-pick census: every story level's E targets (SimpleScripts, doors, keypad locks, Q-Worm
//! PCs, monitors) can be used from every side the game's pick allows, through the real E path
//! (`Sim::step_bond`'s use key = the fresh press of action 0xE [M Input_Action 0x17f208 →
//! Player_WeaponFiring 0x1a5460 → Player_Activate 0x1a5170]).
//!
//! The game's pick [M, crates/nf-game/src/usepick.rs]: a radius-1.0 sphere at head + view
//! direction keeps an object when (1) it touches the object's world sphere (obj+0x80, the
//! glist's local centre posed through the object matrix — NOT the matrix translation) and (2)
//! it lies within 1.0 IN FRONT of one of the object's collision triangles. So an object is
//! usable from in front of each of its collision faces; the census derives those faces from the
//! disc's collision shapes [D] (the object-space triangles posed by the object matrix): for each
//! face direction (four compass bins and up/down) it takes the largest face, puts the use point
//! 0.5 in front of its middle, aims Bond's view along the face's inward normal (a level look for
//! wall faces; four yaws looking 36° down/up for floor/ceiling faces) and presses E.
//!
//! The only facing rule in the chain is the lock arm's: `Mat_GetDir(lock+0x90) · (player+0xC0 −
//! lock+0xC0) < 0` refuses [M 0x1a5170:158-166] — those approaches are asserted to REFUSE.
//!
//! Sister on every row: the effect is recorded off just before the press (state restored between
//! trials). A stand that physics moves (Bond pushed out of a wall, > 0.2) or that sits within
//! reach of a LOAD_LEVEL volume is not a stand and is skipped; every target must keep at least
//! one stand. Old code measured the SimpleScript pick from the model origin (0700000b's floor
//! grate #98 answered only Bond facing west, towards its origin corner) and the lock pick from
//! an unrotated centre, and the monitor pick was 2.2 from the head.

use nf_formats::{disc, iso9660::IsoImage, level::Level};
use nf_view::scene::Sim;

type V3 = [f32; 3];
type Mat4 = [[f32; 4]; 4];

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn level_data(name: &str) -> Level {
    let img = IsoImage::open(&iso()).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    Level::parse(&data).expect("parse")
}

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn scale(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn len(a: V3) -> f32 { dot(a, a).sqrt() }

/// One approach: the face middle `c` aimed at, from `dir` (the unit horizontal direction Bond
/// looks along), standing `back` behind it on the ground.
#[derive(Clone, Copy, Debug)]
struct Approach { c: V3, dir: V3, back: f32, bin: &'static str, area: f32 }

/// The approaches the disc's collision shape allows: per face-direction bin the largest face.
/// Wall faces are approached head-on from 1.5 out; floor faces from 0.5 out on four facings
/// (standing on or beside the face, looking down at it).
/// Ceiling faces are not stands and are left out.
fn approaches(world_tris: &[[V3; 3]]) -> Vec<Approach> {
    const BINS: [&str; 5] = ["+z", "+x", "-z", "-x", "up"];
    let mut best: [Option<(f32, V3, V3)>; 5] = [None; 5];
    for t in world_tris {
        let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
        let area = len(n) * 0.5;
        if area < 1e-4 { continue; }
        let n = scale(n, 1.0 / len(n));
        let bin = if n[1] >= 0.7 { 4 } else if n[1] <= -0.7 { continue } else {
            // compass bin of the horizontal normal: 0 = +z, 1 = +x, 2 = -z, 3 = -x
            let a = n[0].atan2(n[2]);
            (((a / std::f32::consts::FRAC_PI_2).round() as i32).rem_euclid(4)) as usize
        };
        let c = scale(add(add(t[0], t[1]), t[2]), 1.0 / 3.0);
        if best[bin].is_none_or(|(a, _, _)| area > a) { best[bin] = Some((area, c, n)); }
    }
    let mut out = Vec::new();
    for (bi, b) in best.iter().enumerate() {
        let Some((area, c, n)) = *b else { continue };
        if bi < 4 {
            let h = [-n[0], 0.0, -n[2]];
            out.push(Approach { c, dir: scale(h, 1.0 / len(h)), back: 1.5, bin: BINS[bi], area });
        } else {
            for d in [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [-1.0, 0.0, 0.0]] {
                out.push(Approach { c, dir: d, back: 0.5, bin: BINS[bi], area });
            }
        }
    }
    out
}

fn posed(world: &nf_game::world::CollisionWorld, hash: u32, m: &Mat4) -> Vec<[V3; 3]> {
    world.shape(hash).map(|t| t.iter().map(|tri| tri.map(|p| nf_formats::placement::transform(m, p))).collect()).unwrap_or_default()
}

/// A placement-created object's matrix and collision-shape key (`View_RotTransMatrix`; the
/// collision world's synthetic map/index key for objects without a hashcode).
fn placed(lv: &Level, m: usize, pi: usize) -> (Mat4, u32) {
    let p = &lv.maps[m].placements[pi];
    let mx = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
    let obj = if p.by_hash() { lv.by_hash.get(&p.hashcode).map(|&(mi, oi)| (mi, oi)) } else { Some((m, p.index as usize)) };
    let hash = obj.and_then(|(mi, oi)| lv.maps[mi].objects.get(oi).map(|o| if o.hashcode == u32::MAX { 0x8000_0000 | ((mi as u32) << 16) | oi as u32 } else { o.hashcode }))
        .unwrap_or(0);
    (mx, hash)
}

fn near_load_trigger(sim: &Sim, p: V3, r: f32) -> bool {
    sim.drones.triggers.iter().any(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL
        && !(0..3).any(|k| p[k] + r < t.lo[k] || p[k] - r > t.hi[k]))
}

/// One E press from the approach: Bond standing on the ground `back` out from the face middle
/// (1.5 for a wall face, 0.5 for a floor face), looking at it. Returns None when it is not a stand
/// (no ground, a look past ±0.9, or physics moved Bond > 0.2), else whether the press reached the
/// target (`hit` reads the effect).
fn press(sim: &mut Sim, a: &Approach, hit: &dyn Fn(&Sim) -> bool) -> Option<bool> {
    let eye = sim.bond.eye - sim.bond.crouch_drop;
    let hx = a.c[0] - a.dir[0] * a.back;
    let hz = a.c[2] - a.dir[2] * a.back;
    let top = [hx, a.c[1] + 0.3, hz];
    let bot = [hx, a.c[1] - 3.5, hz];
    let f = sim.world.segment(top, bot)?;
    let ground = top[1] + (bot[1] - top[1]) * f;
    // Standing: the body sits ROOT_HEIGHT + the rest offset above the feet (1.086 [L], player.rs).
    let rest = 1.086;
    let head = [hx, ground + rest + eye, hz];
    let to = sub(a.c, head);
    let pitch = (to[1] / len(to)).asin() / std::f32::consts::FRAC_PI_2;
    let (pos, yaw) = ([hx, ground + rest, hz], to[0].atan2(to[2]));
    if pitch.abs() > 0.9 { return None; }
    let place = |sim: &mut Sim| {
        sim.bond.yaw = yaw;
        sim.bond.pitch = pitch;
        sim.bond.pos = pos;
        sim.bond.vel = [0.0; 3];
    };
    place(sim);
    sim.step_bond(Default::default(), false, false, false);
    place(sim);
    sim.step_bond(Default::default(), false, false, true);
    let moved = len(sub(sim.bond.pos, pos));
    let turned = (sim.bond.yaw - yaw).abs() > 0.02 || (sim.bond.pitch - pitch).abs() > 0.02;
    if moved > 0.2 || turned { return None; }
    // Read the effect on the press tick and the next four (Bond held in place, no press): a
    // class whose update applies the activation a tick later still counts.
    let mut got = hit(sim);
    for _ in 0..4 {
        if got { break; }
        place(sim);
        sim.step_bond(Default::default(), false, false, false);
        got = hit(sim);
    }
    Some(got)
}

/// The object's world sphere: the placement object's local bounds centre through `matrix`, its
/// radius (`Control_BuildWorldSph` 0x133070 [M]).
fn world_sphere(lv: &Level, m: usize, pi: usize, matrix: &Mat4) -> (V3, f32) {
    let p = &lv.maps[m].placements[pi];
    let obj = if p.by_hash() { lv.by_hash.get(&p.hashcode).map(|&(mi, oi)| &lv.maps[mi].objects[oi]) } else { lv.maps[m].objects.get(p.index as usize) };
    obj.map(|o| (nf_formats::placement::transform(matrix, o.centre), o.radius)).unwrap_or((p.translation, 0.1))
}

#[derive(Default)]
struct Tally { out_of_reach: usize, targets: usize, presses: usize, refusals: usize, skipped: usize, fails: Vec<String>, nostand: Vec<String>, dead: Vec<String> }

fn census(archive: &str) -> Tally {
    let lv = level_data(archive);
    let m = lv.main_map().expect("main map");
    let mut scene = nf_view::load_with_carry_weapons_difficulty(&iso(), archive, false, None, &[], 3)
        .unwrap_or_else(|e| panic!("load {archive}: {e}"));
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let home = sim.bond.pos;
    let mut t = Tally::default();
    let ch0 = sim.drones.channels.clone();
    // Each press starts from the same mission: a press that fails it (07000007's door #243 stand
    // is past the cover line) would leave the mission hold on and drop every later press.
    let mission0 = (sim.drones.mission.internal, sim.drones.mission.state, sim.drones.mission.fail_label);

    // Generic per-target driver: `arm` restores the target's state, `hit` reads the effect,
    // `allowed` says whether the game's chain accepts this approach (the lock facing rule).
    let mut run = |sim: &mut Sim, t: &mut Tally, name: String, tris: Vec<[V3; 3]>, sph: (V3, f32), door: bool,
                   arm: &dyn Fn(&mut Sim), hit: &dyn Fn(&Sim) -> bool, allowed: &dyn Fn(&Sim, &Approach) -> bool| {
        if tris.is_empty() { return; }
        t.targets += 1;
        let mut stands = 0;
        let mut list = approaches(&tris);
        if door {
            // A door is approached from its two broad sides only (the leaf's front and back): its
            // edge and top faces sit in the jamb and the lintel.
            let mut walls: Vec<Approach> = list.iter().copied().filter(|a| a.back > 1.0).collect();
            walls.sort_by(|a, b| b.area.partial_cmp(&a.area).unwrap());
            list = walls.into_iter().take(2).collect();
        }
        let (mut fails, mut any_effect, mut stands_used) = (Vec::new(), false, 0);
        for a in list {
            if near_load_trigger(sim, a.c, 2.5) { t.skipped += 1; continue; }
            arm(sim);
            sim.drones.channels = ch0.clone();
            (sim.drones.mission.internal, sim.drones.mission.state, sim.drones.mission.fail_label) = mission0;
            if hit(sim) { t.fails.push(format!("{name}: effect already on before the press (sister)")); continue; }
            // The stand: the face's own distance first, then nearer and farther ones.
            let tries = if a.back > 1.0 { [1.5, 1.2, 1.8] } else { [0.5, 0.3, 0.7] };
            let mut res = None;
            let mut a = a;
            for b in tries {
                a.back = b;
                arm(sim);
                sim.drones.channels = ch0.clone();
                res = press(sim, &a, hit);
                if res.is_some() { break; }
            }
            let Some(got) = res else { t.skipped += 1; continue };
            stands += 1;
            // The game's pick at the pressed use point [M usepick.rs: the world-sphere candidate,
            // then a collision face within 1.0 in front], then the class's own rule.
            let u = add(sim.bond.head(), sim.bond.look());
            let picked = len(sub(u, sph.0)) < 1.0 + sph.1 && nf_game::world::CollisionWorld::sphere_meets_tris(u, 1.0, &tris);
            if !picked { t.out_of_reach += 1; }
            let want = picked && allowed(sim, &a);
            if want { stands_used += 1; }
            any_effect |= got;
            if want { t.presses += 1 } else { t.refusals += 1 }
            if got != want {
                let meets = nf_game::world::CollisionWorld::sphere_meets_tris(u, 1.0, &tris);
                let cand = len(sub(u, sph.0)) < 1.0 + sph.1;
                fails.push(format!("{name}: face {} at ({:.2},{:.2},{:.2}) looking ({:.2},{:.2}): {} (game: {}) [use point: in the world sphere {cand}, in front of a face {meets}]",
                    a.bin, a.c[0], a.c[1], a.c[2], a.dir[0], a.dir[2], if got { "used" } else { "no effect" }, if want { "used" } else { "refused" }));
            }
            sim.drones.level_exit = None;
        }
        // A door the E never moves from any side, though the pick keeps it, is a door-class
        // matter (`Door_Activate` / the door update), not the pick's: listed, not failed.
        if door && !any_effect && !fails.is_empty() { t.dead.push(name.clone()); } else { t.fails.extend(fails); }
        arm(sim);
        sim.drones.channels = ch0.clone();
        sim.bond.pos = home;
        if stands == 0 || stands_used == 0 { t.nostand.push(name); }
    };

    // SimpleScripts the E arms: state ∉ {0, 1} (an armed rest state) and mask bit 4 [M SS_Activate 0x1a9ea8].
    for i in 0..sim.drones.sss.list.len() {
        let s0 = sim.drones.sss.list[i].clone();
        if s0.state == 0 || s0.state == 1 || s0.mask & 4 == 0 || s0.hash == 0 { continue; }
        let tris = posed(&sim.world, s0.hash, &s0.matrix);
        let keep = s0.clone();
        let sph = world_sphere(&lv, m, s0.placement, &s0.matrix);
        run(sim, &mut t, format!("{archive} SS #{}", s0.placement), tris, sph, false,
            &|s: &mut Sim| s.drones.sss.list[i] = keep.clone(),
            &|s: &Sim| { let x = &s.drones.sss.list[i]; x.state != keep.state || x.t != keep.t || (keep.channel != 0 && s.drones.channels.on(keep.channel as u32)) },
            &|_, _| true);
    }
    // Closed, unlocked doors with keys [M Door_Activate 0x135be0].
    for i in 0..sim.drones.doors.list.len() {
        let d0 = sim.drones.doors.list[i].clone();
        if d0.glist == 0 || d0.ch_lock != 0 || d0.locked_flag != 0 || d0.keys.is_none() || d0.state & 0xc != 0 { continue; }
        let tris = posed(&sim.world, d0.glist, &d0.matrix);
        let keep = d0.clone();
        let sph = d0.live_sph();
        run(sim, &mut t, format!("{archive} door #{}", d0.placement), tris, sph, true,
            &|s: &mut Sim| s.drones.doors.list[i] = keep.clone(),
            &|s: &Sim| { let x = &s.drones.doors.list[i]; x.state != keep.state || (x.pos01 - keep.pos01).abs() > 1e-4 },
            // `Door_Activate`'s own gates (the side test, the busy/lock arms) [M 0x135be0]: the
            // census asks the pick only, so the expectation is that a direct activation with
            // Bond's head as the activator moves the door.
            &|s: &Sim, _| {
                let mut dd = s.drones.doors.clone();
                dd.list[i] = keep.clone();
                let mut ch = ch0.clone();
                dd.activate(i, &[(s.bond.head(), 0.0, false)], true, &mut ch, s.drones.frame, &mut Vec::new())
            });
    }
    // Keypad locks (kind 0, locked, not scanning): E arms the keypad [M Lock_Activate 0x1aed00],
    // refused from behind [M 0x1a5170:158-166].
    for i in 0..sim.drones.locks.len() {
        let l = &sim.drones.locks[i];
        if l.kind != 0 || l.unlocked || l.burned || l.scanning { continue; }
        let (mx, hash) = placed(&lv, m, l.placement);
        let tris = posed(&sim.world, hash, &mx);
        let (facing, origin) = (l.facing, [mx[3][0], mx[3][1], mx[3][2]]);
        let sph = world_sphere(&lv, m, l.placement, &mx);
        run(sim, &mut t, format!("{archive} lock #{}", l.placement), tris, sph, false,
            &|s: &mut Sim| s.drones.keypad = None,
            &move |s: &Sim| s.drones.keypad.as_ref().is_some_and(|(k, _)| *k == i),
            &move |s: &Sim, _| dot(facing, sub(s.bond.pos, origin)) >= 0.0);
    }
    // Q-Worm PCs with the E arm (param 2 ≠ 0) [M PCQWorm_Activate 0x1b0840].
    let pc_placements: Vec<usize> = lv.maps[m].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x35).map(|(i, _)| i).collect();
    for i in 0..sim.drones.useobjs.pcs.len() {
        if sim.drones.useobjs.pcs[i].e_usable == 0 || sim.drones.useobjs.pcs[i].state != 0 { continue; }
        let Some(&pi) = pc_placements.get(i) else { continue };
        let (mx, hash) = placed(&lv, m, pi);
        let tris = posed(&sim.world, hash, &mx);
        let sph = world_sphere(&lv, m, pi, &mx);
        run(sim, &mut t, format!("{archive} PC #{pi}"), tris, sph, false,
            &|s: &mut Sim| s.drones.useobjs.pcs[i].state = 0,
            &|s: &Sim| s.drones.useobjs.pcs[i].state != 0,
            &|_, _| true);
    }
    // Monitors whose activation has a camera (a direct `Monitor_Activate` on a copy succeeds).
    for i in 0..sim.drones.monitors.list.len() {
        let mut probe = sim.drones.monitors.clone();
        if !probe.activate(i, &sim.drones.sensors) { continue; }
        let pi = sim.drones.monitors.list[i].placement;
        let (mx, hash) = placed(&lv, m, pi);
        let tris = posed(&sim.world, hash, &mx);
        let keep = sim.drones.monitors.list[i].clone();
        let sph = world_sphere(&lv, m, pi, &mx);
        run(sim, &mut t, format!("{archive} monitor #{pi}"), tris, sph, false,
            &|s: &mut Sim| { s.drones.monitors.list[i] = keep.clone(); s.disabled = false; },
            &|s: &Sim| s.drones.monitors.list[i].state != keep.state,
            &|_, _| true);
    }
    eprintln!("[census] {archive}: {} targets, {} presses used, {} refusals ({} out of the pick's reach), {} skipped, {} failures, no stand: {:?}, doors E never moves: {:?}",
        t.targets, t.presses, t.refusals, t.out_of_reach, t.skipped, t.fails.len(), t.nostand, t.dead);
    for f in &t.fails { eprintln!("  FAIL {f}"); }
    t
}

fn check(archive: &str) {
    let t = census(archive);
    assert!(t.fails.is_empty(), "{archive}: {} approaches disagree with the game's pick:\n{}", t.fails.len(), t.fails.join("\n"));
}

macro_rules! level_rows { ($($name:ident = $file:expr;)*) => { $( #[test] fn $name() { check($file); } )* } }

level_rows! {
    census_07000001 = "07000001.bin";
    census_07000002 = "07000002.bin";
    census_07000003 = "07000003.bin";
    census_07000004 = "07000004.bin";
    census_07000005 = "07000005.bin";
    census_07000006 = "07000006.bin";
    census_07000007 = "07000007.bin";
    census_07000008 = "07000008.bin";
    census_07000009 = "07000009.bin";
    census_0700000a = "0700000a.bin";
    census_0700000b = "0700000b.bin";
    census_0700000c = "0700000c.bin";
    census_0700000d = "0700000d.bin";
    census_07000011 = "07000011.bin";
    census_07000012 = "07000012.bin";
    census_07000013 = "07000013.bin";
    census_07000014 = "07000014.bin";
    census_07000015 = "07000015.bin";
    census_07000016 = "07000016.bin";
    census_0700001b = "0700001b.bin";
}

/// 0700000b's interior floor grate #98 (vent_grill1, state 0x40, mask 4 [D]): standing on it,
/// E takes from every facing when Bond looks down at it, and from none when he looks level —
/// the use point then sits 1.8 above the grate, past the refine's 1.0 [M Intersect_PointGeom
/// 0x1e92d8:200 via Collide_Intersect 0x1ebaa8:186-189]. The old pick measured from the grate's
/// origin corner and kept no refine: looking level it took facing west only (red there).
#[test]
fn the_0b_floor_grate_takes_e_from_every_facing_looking_down() {
    let lv = level_data("0700000b.bin");
    let m = lv.main_map().unwrap();
    let mut scene = nf_view::load_with_carry_weapons_difficulty(&iso(), "0700000b.bin", false, None, &[], 3).expect("load 0b");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    let i = sim.drones.sss.list.iter().position(|s| s.placement == 98).expect("SS #98");
    let keep = sim.drones.sss.list[i].clone();
    let (c, _) = world_sphere(&lv, m, 98, &keep.matrix);
    // The grate's top at its sphere centre [D the posed collision shape].
    let f = sim.world.segment([c[0], c[1] + 1.0, c[2]], [c[0], c[1] - 1.0, c[2]]).expect("the grate under its centre");
    let top = c[1] + 1.0 - 2.0 * f;
    assert!((top - 49.75).abs() < 0.1, "standing on the grate, top {top}");
    let ch0 = sim.drones.channels.clone();
    let mut press_at = |sim: &mut Sim, yaw: f32, pitch: f32| -> bool {
        sim.drones.sss.list[i] = keep.clone();
        sim.drones.channels = ch0.clone();
        let place = |sim: &mut Sim| {
            sim.bond.pos = [c[0], top + 1.086, c[2]];
            sim.bond.vel = [0.0; 3];
            sim.bond.yaw = yaw;
            sim.bond.pitch = pitch;
        };
        place(sim);
        sim.step_bond(Default::default(), false, false, false);
        assert!(sim.drones.sss.list[i].state == keep.state, "sister: the grate is still armed before the press");
        place(sim);
        sim.step_bond(Default::default(), false, false, true);
        let moved = len(sub(sim.bond.pos, [c[0], top + 1.086, c[2]]));
        assert!(moved < 0.2, "Bond stands on the grate (moved {moved})");
        sim.drones.sss.list[i].state != keep.state
    };
    let yaws = [0.0, std::f32::consts::FRAC_PI_2, std::f32::consts::PI, -std::f32::consts::FRAC_PI_2];
    let down: Vec<bool> = yaws.iter().map(|&y| press_at(sim, y, -0.7)).collect();
    let level: Vec<bool> = yaws.iter().map(|&y| press_at(sim, y, 0.0)).collect();
    eprintln!("[0b grate] looking down (yaw 0, 90, 180, 270): {down:?}; looking level: {level:?}");
    assert!(down.iter().all(|&g| g), "E on the grate looking down from every facing: {down:?}");
    assert!(level.iter().all(|&g| !g), "looking level the use point is past the refine's reach: {level:?}");
}
