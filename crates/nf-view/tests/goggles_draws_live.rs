//! goggles-draws: the goggle-mode draws that ride on other objects — the guards' breath puffs
//! (`Env_Breath` 0x17be00 from `NDrone2_ControlSTANDARD` 0x1490f0:168-198) with their x-ray-only
//! variant, the guards' weapon-0x18 laser ray (`AnimObjectDraw` 0x1c4490:259-311), and the
//! tripwire triggers' key-list motion (`Trigger_Update` 0x1b1a78:60-70). Evidence map:
//! docs/systems/goggles.md, docs/systems/env-breath-ripple.md. Opt-in via NF_ISO.

use nf_game::player::Input;
use nf_view::scene::{Scene, GUARD_BEAMS, GUARD_BEAM_BASE, GUARD_BEAM_DATUM, XRAY_SLOT_BASE};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const FAR_Y: f32 = -9999.0;

fn step(scene: &mut Scene, n: u32) {
    for _ in 0..n { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
}

fn press_to(scene: &mut Scene, want: u16) {
    for t in 0..900 {
        if scene.sim.as_ref().unwrap().goggle_mode() == want && t > 0 { break; }
        // muzzle-repin-5: hold Bond alive through the presses — the start rooms are fighting and
        // the shifted stream (3 muzzle draws per guard shot) changes who hits Bond during this
        // window; a dead Bond ignores the key (the mode gate needs a live player), which used to
        // fail a_captains_laser_ray's mode-1 press. The tests' subject is the goggle draw, not
        // survivability; every caller here presses inside a live room.
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        let mut i = Input::default();
        i.goggles_action = t % 2 == 0 && scene.sim.as_ref().unwrap().goggle_mode() != want;
        scene.sim.as_mut().unwrap().step_bond(i, false, false, false);
    }
    assert_eq!(scene.sim.as_ref().unwrap().goggle_mode(), want, "the key reached mode {want}");
}

fn loaded(iso: &str, level: &str) -> Scene {
    let mut scene = nf_view::load(iso, level, false).expect("load");
    for _ in 0..3000 {
        step(&mut scene, 1);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
    step(&mut scene, 300);
    scene
}

fn opts() -> nf_view::scene::ViewOpts {
    nf_view::scene::ViewOpts { reinforcements: false, firing: false, right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0] }
}

/// Bond 3.0 in front of a live guard (on the guard's +x side), turned to face it, so the guard is
/// in view (`drawn_grace` stays up). Returns the drone index.
fn face_a_guard(scene: &mut Scene, pick: impl Fn(&nf_game::drone::Drone) -> bool) -> usize {
    let di = {
        let s = scene.sim.as_ref().unwrap();
        s.char_drone.iter().filter_map(|d| *d)
            .find(|&d| { let dr = &s.drones.list[d]; dr.present && dr.enabled && dr.health > 0.0 && pick(dr) })
            .expect("a live guard")
    };
    place(scene, di);
    di
}

/// Bond 3.0 along the guard's facing (yaw → (sin, cos), the port's convention), turned back to it.
fn place(scene: &mut Scene, di: usize) {
    let s = scene.sim.as_mut().unwrap();
    let (g, y) = (s.drones.list[di].pos, s.drones.list[di].yaw);
    let f = [y.sin(), y.cos()];
    s.bond.pos = [g[0] + f[0] * 3.0, g[1], g[2] + f[1] * 3.0];
    s.bond.yaw = (-f[0]).atan2(-f[1]);
}

fn breath_puffs(scene: &Scene) -> Vec<nf_game::gas::Gas> {
    scene.sim.as_ref().unwrap().drones.gas.puffs.iter().filter(|g| g.mesh == nf_game::env::BREATH_MESH).cloned().collect()
}

/// A snow level (07000005: `Env_Create` drop type 0 [D]) — the guards breathe the plain puff in
/// every mode: spawned at the head + dir·0.22 (0.12 by the caller, 0.1 by Env_Breath), alpha
/// 128 fading to 1 while staying white (end 0xffffff01), size 0.05 -> 0.1, no x-ray bit
/// [M 0x17bf00-0x17bf68]; the pool's breath slots draw it in
/// mode 0. Sister: a dead guard stops breathing.
#[test]
fn snow_level_guards_breathe_in_every_mode() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000005.bin");
    assert_eq!(scene.sim.as_ref().unwrap().drones.env.as_ref().map(|e| e.kind), Some(0), "07000005's env is snow [D 0xDF p0 = 0]");
    let di = face_a_guard(&mut scene, |_| true);
    let mut seen = Vec::new();
    for _ in 0..240 {
        place(&mut scene, di);
        step(&mut scene, 1);
        seen.extend(breath_puffs(&scene).into_iter().filter(|g| g.life >= 39.0));
    }
    assert!(!seen.is_empty(), "the guard breathes within 240 ticks of mode 0");
    // The spawn values, read back through the ticks the puff has run (life 40 at spawn).
    let born = |g: &nf_game::gas::Gas| g.rgba[3] - g.rgba_rate[3] * (40.0 - g.life);
    for g in &seen {
        assert!(!g.xray_only, "the snow variant carries no 0x400 bit");
        assert!((born(g) - 128.0).abs() < 0.5, "alpha 0x80 at spawn [M 0x17bf60]: {}", born(g));
        let born_scale = g.scale - g.scale_rate * (40.0 - g.life);
        assert!((born_scale - 0.05).abs() < 1e-5 && (g.scale_rate - 0.05 / 40.0).abs() < 1e-7, "0.05 -> 0.1 [M f12/f13 0x17bf00-0x17bf2c]: {born_scale} {}", g.scale_rate);
        assert!(g.rgba[..3].iter().all(|&c| c == 255.0), "the puff stays white (end 0xffffff01) [M 0x17bf20/0x17bf68]: {:?}", g.rgba);
    }
    let gp = scene.sim.as_ref().unwrap().drones.list[di].pos;
    let near = seen.iter().any(|g| ((g.pos[0] - gp[0]).powi(2) + (g.pos[2] - gp[2]).powi(2)).sqrt() < 1.0 && g.pos[1] > gp[1]);
    assert!(near, "a puff sits at the guard's head");
    // A live puff draws in mode 0 through the gas slots.
    let fs = scene.frame_state(None, 0.0, opts());
    let drawn = scene.attachments.iter().enumerate().filter(|(_, a)| a.gas.is_some_and(|(_, m)| m == nf_game::env::BREATH_MESH)).any(|(k, _)| fs.attach[k][3][1] > FAR_Y);
    assert!(drawn || breath_puffs(&scene).is_empty(), "a live breath puff draws in mode 0");
    // Sister (can fail): with every guard dead no new puff is made.
    for dr in scene.sim.as_mut().unwrap().drones.list.iter_mut() { dr.health = 0.0; }
    step(&mut scene, 60);
    let n0 = breath_puffs(&scene).len();
    step(&mut scene, 120);
    assert!(breath_puffs(&scene).iter().all(|g| g.life < 40.0) && breath_puffs(&scene).len() <= n0, "dead guards make no breath");
}

/// A level without a snow env (07000012: no 0xDF placement [D]) — breath spawns only in x-ray, with the
/// 0x400 bit, alpha 200, growing 0.05 → 0.2 [M 0x17bf7c-0x17c02c], and its slots draw only in
/// mode 2. Sister: mode 0 makes none.
#[test]
fn rain_level_breath_is_x_ray_only() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000012.bin");
    assert!(scene.sim.as_ref().unwrap().drones.env.is_none(), "07000012 has no Env_Create [D]");
    let di = face_a_guard(&mut scene, |_| true);
    for _ in 0..240 { place(&mut scene, di); scene.sim.as_mut().unwrap().drones.health.health = 100.0; step(&mut scene, 1); }
    assert!(breath_puffs(&scene).is_empty(), "mode 0: no breath off the snow levels");
    press_to(&mut scene, 2);
    let mut seen = Vec::new();
    for _ in 0..240 {
        place(&mut scene, di);
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        step(&mut scene, 1);
        if scene.sim.as_ref().unwrap().goggle_mode() != 2 { break; }
        seen.extend(breath_puffs(&scene).into_iter().filter(|g| g.life >= 39.0));
    }
    assert!(!seen.is_empty(), "mode 2: the guard breathes");
    let born = |g: &nf_game::gas::Gas| g.rgba[3] - g.rgba_rate[3] * (40.0 - g.life);
    for g in &seen {
        assert!(g.xray_only, "the x-ray variant carries the 0x400 bit");
        assert!((born(g) - 200.0).abs() < 0.5, "alpha 0xc8 at spawn [M 0x17c018]: {}", born(g));
        assert!(g.scale_rate > 0.0, "grows 0.05 → 0.2");
    }
    let k = scene.attachments.iter().enumerate().filter(|(_, a)| a.gas.is_some_and(|(_, m)| m == nf_game::env::BREATH_MESH)).map(|(k, _)| k).collect::<Vec<_>>();
    let fs = scene.frame_state(None, 0.0, opts());
    let live = !breath_puffs(&scene).is_empty();
    assert!(!live || k.iter().any(|&k| fs.attach[k][3][1] > FAR_Y), "mode 2: the puffs draw");
    // Sister (can fail): back in mode 0 the x-ray puffs still in the pool park.
    press_to(&mut scene, 0);
    let fs = scene.frame_state(None, 0.0, opts());
    let parked = scene.attachments.iter().enumerate().filter(|(_, a)| a.gas.is_some_and(|(i, m)| m == nf_game::env::BREATH_MESH
        && scene.sim.as_ref().unwrap().drones.gas.puffs.get(i as usize).is_some_and(|g| g.xray_only && g.mesh == m)))
        .all(|(k, _)| fs.attach[k][3][1] <= FAR_Y);
    assert!(parked, "mode 0: an x-ray puff does not draw");
}

/// The 0x19 carriers and the captains [D]; the breath mesh 0x200021d and both ray sprites resolve.
#[test]
fn the_laser_ray_carriers() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut rows = Vec::new();
    for level in ["0700000b", "07000011", "07000012", "07000013", "07000005"] {
        let scene = nf_view::load(&iso, &format!("{level}.bin"), false).expect("load");
        let s = scene.sim.as_ref().unwrap();
        let w19 = s.drones.list.iter().filter(|d| d.weapon == 0x19).count();
        let captains = s.drones.list.iter().filter(|d| d.weapon == 0x19 && d.captain).count();
        let slots = scene.attachments.iter().filter(|a| a.tracer.is_some_and(|t| (GUARD_BEAM_BASE..GUARD_BEAM_BASE + GUARD_BEAMS * 2).contains(&t))).count();
        let breath_slots = scene.attachments.iter().filter(|a| a.gas.is_some_and(|(_, m)| m == nf_game::env::BREATH_MESH)).count();
        // Every 0x19 skin carries datum 11 (8 + 3) on a bone.
        for d in s.drones.list.iter().filter(|d| d.weapon == 0x19) {
            assert!(s.skins.get(&d.skin).is_some_and(|k| k.datums.iter().any(|x| x.id == GUARD_BEAM_DATUM && x.bone >= 0)), "{level}: skin {:#x} has datum 11", d.skin);
        }
        assert!(breath_slots > 0, "{level}: the breath mesh has draw slots");
        rows.push((level, w19, captains, slots));
    }
    assert_eq!(rows, vec![
        ("0700000b", 10, 2, GUARD_BEAMS * 2), ("07000011", 10, 0, GUARD_BEAMS * 2), ("07000012", 10, 3, GUARD_BEAMS * 2),
        ("07000013", 4, 0, GUARD_BEAMS * 2), ("07000005", 0, 0, GUARD_BEAMS * 2),
    ]);
}

/// A captain in combat (07000012's start-room captain) swaps 0x19 → 0x18 [M DSTATE_Attack 0x164210:31-52 / AimStand 0x166178:44]
/// and its ray draws from datum 11 along the datum's −X row, clamped to the first wall within
/// 100.0; mode 1 swaps the sprite to 0x2000131 [M 0x1c4490:296-301]. Sister: a 0x19 guard draws none.
#[test]
fn a_captains_laser_ray() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = loaded(&iso, "07000012.bin");
    // The start room is already fighting by now: put every captain back on the 0x19 for the sister.
    for d in scene.sim.as_mut().unwrap().drones.list.iter_mut().filter(|d| d.captain) { d.weapon = 0x19; }
    let fs = scene.frame_state(None, 0.0, opts());
    let beams0 = scene.guard_beams(|c| fs.posed.get(c).and_then(|p| p.as_ref()).filter(|_| !fs.hidden[c]).map(|(_, w, m)| (w.as_slice(), m)));
    assert!(beams0.is_empty(), "sister: no guard holds 0x18 at the level start (all 0x19)");
    let di = face_a_guard(&mut scene, |d| d.captain && d.weapon == 0x19);
    let mut armed = false;
    for t in 0..900 {
        place(&mut scene, di);
        scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        // A few shots into the far wall first (Bond turned away): the noise alerts the room.
        if t < 120 {
            scene.sim.as_mut().unwrap().bond.yaw = 1.0f32.atan2(0.0);
            let mut i = Input::default();
            i.fire = t % 8 == 0;
            scene.sim.as_mut().unwrap().step_bond(i, false, false, false);
        } else {
            step(&mut scene, 1);
        }
        if scene.sim.as_ref().unwrap().drones.list[di].weapon == 0x18 { armed = true; break; }
    }
    assert!(armed, "the captain's combat swaps its byte to 0x18");
    place(&mut scene, di);
    let ci = scene.sim.as_ref().unwrap().char_drone.iter().position(|d| *d == Some(di)).unwrap();
    let fs = scene.frame_state(None, 0.0, opts());
    let beams = scene.guard_beams(|c| fs.posed.get(c).and_then(|p| p.as_ref()).filter(|_| !fs.hidden[c]).map(|(_, w, m)| (w.as_slice(), m)));
    assert_eq!(beams.len(), 1, "one ray");
    let (a, b) = beams[0];
    let (_, world, m) = fs.posed[ci].as_ref().unwrap();
    let dt = scene.characters[ci].skin.datums.iter().find(|d| d.id == GUARD_BEAM_DATUM).unwrap();
    let dm = nf_formats::anim::mul(&nf_formats::anim::mul(&nf_formats::anim::quat_trans_to_mat(dt.q, dt.t), &world[dt.bone as usize]), m);
    assert_eq!(a, [dm[3][0], dm[3][1], dm[3][2]], "the ray starts at datum 11");
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    assert!(len > 0.0 && len <= 100.0 + 1e-3, "clamped inside 100 ({len})");
    let x = [-dm[0][0], -dm[0][1], -dm[0][2]];
    let dot = (d[0] * x[0] + d[1] * x[1] + d[2] * x[2]) / len;
    assert!(dot > 0.999, "along the datum's −X row ({dot})");
    let slot = |k: usize| scene.attachments.iter().position(|a| a.tracer == Some(k)).unwrap();
    let (day, night) = (slot(GUARD_BEAM_BASE), slot(GUARD_BEAM_BASE + 1));
    assert!(fs.attach[day][3][1] > FAR_Y && fs.attach[night][3][1] <= FAR_Y, "mode 0: 0x2000133");
    let p = fs.attach[day][3];
    assert!((p[0] - a[0]).abs() < 1e-4 && (p[1] - a[1]).abs() < 1e-4 && (p[2] - a[2]).abs() < 1e-4, "the sprite sits at the start");
    // muzzle-repin-5: the latch is CLEAR here — the first press requests the goggles WEAPON
    // 0x5d (a weapon request, swallowed by a hit-stagger) and only the SET latch cycles the
    // mode [M player.rs — BL+0x961 / +0x60]. The shifted stream keeps the room's guards firing
    // through this window and the stagger ate the request (mode stuck 0). Step Bond 30 u back
    // for the press (out of the fire line — the stun test's own trick), then place() re-faces
    // the guard for the mode-1 attach assert below.
    {
        let s = scene.sim.as_mut().unwrap();
        let (bp, byaw) = (s.bond.pos, s.bond.yaw);
        s.bond.pos = [bp[0] - byaw.sin() * 30.0, bp[1], bp[2] - byaw.cos() * 30.0];
    }
    press_to(&mut scene, 1);
    place(&mut scene, di);
    if scene.sim.as_ref().unwrap().drones.list[di].weapon == 0x18 {
        let fs = scene.frame_state(None, 0.0, opts());
        assert!(fs.attach[night][3][1] > FAR_Y && fs.attach[day][3][1] <= FAR_Y, "mode 1: 0x2000131");
    }
    // Sister (can fail): the same guard back on 0x19 draws no ray.
    scene.sim.as_mut().unwrap().drones.list[di].weapon = 0x19;
    let fs = scene.frame_state(None, 0.0, opts());
    assert!(fs.attach[day][3][1] <= FAR_Y && fs.attach[night][3][1] <= FAR_Y, "a 0x19 holder draws no ray");
}

/// The keyed tripwires move [D: 0700000a 3 of 10, 0700000b 8 of 8, 07000014 8 of 8 carry 3-key
/// lists]: the playhead steps last/rate per update and wraps after `rate` ticks; key 0 is the
/// placement pose; the volume and the x-ray draw follow the pose. Sister: an unkeyed beam stays.
#[test]
fn keyed_tripwires_follow_their_keys() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let count = |scene: &Scene| scene.sim.as_ref().unwrap().drones.triggers.iter().filter(|t| t.motion.is_some()).count();
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load");
    assert_eq!(count(&scene), 8, "0700000b: 8 keyed laserbeam-large triggers");
    let ti = scene.sim.as_ref().unwrap().drones.triggers.iter().position(|t| t.placement == 270).expect("#270");
    {
        let t = &scene.sim.as_ref().unwrap().drones.triggers[ti];
        let m = t.motion.as_ref().unwrap();
        assert_eq!((m.last, m.keys.len()), (2, 3));
        assert!((m.step - 2.0 / 300.0).abs() < 1e-7, "step = last / rate (300) [M 0x1b1720:84-85]");
        let k0 = nf_game::door::interp(&m.keys, 0, 2, 0.0, true).unwrap();
        assert!((0..3).all(|c| (k0[3][c] - m.matrix[3][c]).abs() < 0.05), "key 0 = the placement pose {:?} vs {:?}", k0[3], m.matrix[3]);
    }
    let lo0 = scene.sim.as_ref().unwrap().drones.triggers[ti].lo;
    loaded_steps(&mut scene, 75);
    let (t_now, lo1) = { let t = &scene.sim.as_ref().unwrap().drones.triggers[ti]; (t.motion.as_ref().unwrap().t, t.lo) };
    assert!(t_now > 0.0, "the playhead advanced");
    assert_ne!(lo0, lo1, "the volume moved with the pose");
    press_to(&mut scene, 2);
    let k = scene.attachments.iter().position(|a| a.tracer == Some(XRAY_SLOT_BASE + 270)).expect("#270's x-ray slot");
    let fs = scene.frame_state(None, 0.0, opts());
    let m = scene.sim.as_ref().unwrap().drones.triggers[ti].motion.as_ref().unwrap().matrix;
    if scene.sim.as_ref().unwrap().drones.triggers[ti].gone { return; }
    assert_eq!(fs.attach[k], m, "the x-ray draw sits at the spline pose");
    // The wrap: (float)last < t → 0 — one pass is `rate` updates.
    let mut mo = scene.sim.as_ref().unwrap().drones.triggers[ti].motion.as_ref().unwrap().as_ref().clone();
    mo.t = 0.0;
    let mut n = 0;
    loop { mo.advance(); n += 1; if mo.t == 0.0 || n > 1000 { break; } }
    assert!((300..=301).contains(&n), "a 300-tick pass, wrapping once (float)last < t [M 0x1b1a78:65-69]: {n}");
    // Sister (can fail): 0700000a's unkeyed beams keep no motion; its keyed three do.
    let scene_a = nf_view::load(&iso, "0700000a.bin", false).expect("load");
    assert_eq!(count(&scene_a), 3, "0700000a: #606/#614/#625 keyed, the other 7 static");
    let scene_14 = nf_view::load(&iso, "07000014.bin", false).expect("load");
    assert_eq!(count(&scene_14), 8, "07000014: 8 keyed TouchOnce beams");
}

fn loaded_steps(scene: &mut Scene, n: u32) {
    for _ in 0..3000 {
        step(scene, 1);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
    step(scene, n);
}
