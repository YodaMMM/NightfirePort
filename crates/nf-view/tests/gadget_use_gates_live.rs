//! gadgets: the Micro-Camera's aim gate and the Stunner's first-hit target.
//!
//! - The Micro-Camera (base 0x54 — the pair 0x54/0x55, label 0x05000045 "Micro-Camera" [L USATxt])
//!   photographs only with the aim bit up: on the fire-held branch `base == 0x54 && !(anim+0x60 & 1)`
//!   returns [M 0x1a5460:512-514]. The port photographed on any press.
//! - Each camera shot plays the 2-D `Sound_Play(0xEC, 100.0)` from `Player_WeaponInitBullet`'s
//!   pellet loop [M 0x1a4bf0:261-263] (the port's fire_player already does; pinned here as the
//!   sister: the gate must not cost the aimed shot its shutter).
//! - The Stunner's target is the crosshair ray's first hit [M 0x1a4bf0:152-200]: a wall in front
//!   of the guard makes the tick a free miss. The port's sphere stand-in tased through walls.
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn equip(sim: &mut nf_view::scene::Sim, w: u8) {
    sim.step_bond(Input { switch_to: Some(w), ..Default::default() }, false, false, false);
    for _ in 0..400 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.gun.weapon == w && sim.gun.switch_state == 0 { break; }
    }
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.gun.weapon, w, "the real switch input equips {w:#x}");
}

/// One tick with the fire held; returns (shot spawned, the 2-D/3-D sound ids started).
fn fire_tick(sim: &mut nf_view::scene::Sim, aiming: bool) -> (bool, Vec<u32>) {
    sim.step_bond(Input { aiming, fire: true, ..Default::default() }, true, false, false);
    let shot = sim.shot_tick;
    let (pos, f) = (sim.bond.head(), sim.bond.look());
    let l = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
    sim.drones.audio.update(sim.sfx.as_ref(), (pos, [f[2] / l, 0.0, -f[0] / l], [0.0, 1.0, 0.0], f), &|_| false);
    let cmds = sim.drones.audio.take_commands();
    let ids = cmds.into_iter()
        .filter_map(|c| if let nf_game::sound::Cmd::Start { id, .. } = c { Some(id) } else { None }).collect();
    (shot, ids)
}

/// Press the fire for `n` ticks, then release for 70 (past the camera's +0x40 = 60 refire).
fn press(sim: &mut nf_view::scene::Sim, aiming: bool, n: u32) -> (u32, Vec<u32>) {
    let (mut shots, mut ids) = (0, Vec::new());
    for _ in 0..n {
        let (s, i) = fire_tick(sim, aiming);
        shots += s as u32;
        ids.extend(i);
    }
    for _ in 0..70 { sim.step_bond(Input { aiming, ..Default::default() }, false, false, false); }
    let _ = sim.drones.audio.take_commands();
    (shots, ids)
}

/// 0700000c (the level's own start kit carries the camera [M 0x1a7ef8 case 0x700000c]): an
/// unaimed held fire takes no photograph and makes no shutter sound; the aimed press photographs
/// with the 0xEC shutter.
#[test]
fn the_micro_camera_photographs_only_while_aimed_and_clicks_its_shutter() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    equip(sim, 0x54);
    let _ = sim.drones.audio.take_commands();
    // The press's edge toggles the pair (0x54 ↔ 0x55, `Player_Weapon`'s watch OR [M 0x1a24c8]);
    // the HELD fire after that switch settles still takes no photograph unaimed.
    let (shots, ids) = press(sim, false, 240);
    assert_eq!(shots, 0, "an unaimed held fire takes no photograph [M 0x1a5460:512-514 — base 0x54 without the aim bit returns]");
    assert!(!ids.contains(&0xec), "no shutter without the shot, got {ids:x?}");
    // Raise the camera: the aim bit rises when the raise script stops [M 0x1a2e38:554-561].
    for _ in 0..240 {
        sim.step_bond(Input { aiming: true, ..Default::default() }, false, false, false);
        if sim.aiming { break; }
    }
    assert!(sim.aiming, "the camera's raise completes and the aim bit is up");
    for _ in 0..10 { sim.step_bond(Input { aiming: true, ..Default::default() }, false, false, false); }
    let _ = sim.drones.audio.take_commands();
    let (shots, ids) = press(sim, true, 3);
    assert_eq!(shots, 1, "the aimed press photographs once (the +0x40 = 60 refire)");
    assert_eq!(ids.iter().filter(|&&i| i == 0xec).count(), 1,
        "the shot plays the 0xEC shutter once [M 0x1a4bf0:261-263], got {ids:x?}");
}

/// Aim the view at `target` from Bond's head (a harness pose: the yaw/pitch the turn input would
/// reach).
fn aim_at(sim: &mut nf_view::scene::Sim, target: [f32; 3]) {
    for _ in 0..3 {
        let h = sim.bond.head();
        let (dx, dy, dz) = (target[0] - h[0], target[1] - h[1], target[2] - h[2]);
        sim.bond.yaw = dx.atan2(dz);
        sim.bond.pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.05)).atan() / std::f32::consts::FRAC_PI_2;
        sim.step_bond(Input::default(), false, false, false);
    }
}

/// 07000005: the Stunner's target is the crosshair ray's FIRST hit [M Player_WeaponInitBullet
/// 0x1a4bf0:152-200 — `Collide_RayIntersect` head → aiming point; the head entry's object must be
/// class 2/3 within the record's +0x1C (5.0 [L])]. Guard #2 (the roof guard at (61.3, 27.1, 99.5)
/// [D the level's placements]) seen from 3 units WEST stands behind the wall the head→chest
/// segment meets 1.2 out [D the port's collision world]: no target, no beam, the spend refunded
/// (the clip holds). Sister: the same guard from 3 units EAST (a clear line) is a target and the
/// clip drains. The guards are held (the dev hold — the guard's AI is not under test).
#[test]
fn the_stunner_does_not_tase_through_a_wall() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.hold_npcs = true;
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    equip(sim, 0x4a);
    let g = sim.drones.list[2].pos;
    assert!((g[0] - 61.34).abs() < 0.5 && (g[2] - 99.50).abs() < 0.5, "guard #2 holds its post, {g:?}");
    let chest = [g[0], g[1] + 1.0, g[2]];
    let mut run = |sim: &mut nf_view::scene::Sim, at: [f32; 3]| {
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, at, 0.0);
        for _ in 0..12 { sim.step_bond(Input::default(), false, false, false); }
        aim_at(sim, chest);
        // Full charge first (the regen: +1 per 4 frames while idle [M 0x193600:380-388]).
        for _ in 0..500 { sim.step_bond(Input::default(), false, false, false); }
        aim_at(sim, chest);
        let blocked = sim.world.segment(sim.bond.head(), chest).is_some();
        let clip0 = sim.gun.clip;
        let (mut targets, mut beams) = (0, 0);
        for _ in 0..20 {
            sim.step_bond(Input { fire: true, ..Default::default() }, true, false, false);
            targets += (sim.stunner_target == Some(true)) as u32;
            beams += sim.laser_beam.is_some() as u32;
        }
        let spent = clip0 as i32 - sim.gun.clip as i32;
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
        (blocked, targets, beams, spent)
    };
    let (blocked, targets, beams, spent) = run(sim, [58.34, 27.10, 99.50]);
    assert!(blocked, "the west stand's head→chest line meets the wall [D]");
    assert_eq!(targets, 0, "through the wall the guard is no target [M 0x1a4bf0:176-186]");
    assert_eq!(beams, 0, "no beam without a target (the +0x95d gate [M 0x1a4298:134])");
    assert!(spent <= 0, "the blocked ticks are free (the miss arm refunds [M 0x1a4bf0:187-199]), spent {spent}");
    let (blocked, targets, _, spent) = run(sim, [64.34, 26.97, 99.50]);
    assert!(!blocked, "the east stand sees the guard's chest");
    assert!(targets > 0, "sister: the clear line is a target");
    assert!(spent > 0, "sister: the aimed ticks spend the charge, spent {spent}");
}
