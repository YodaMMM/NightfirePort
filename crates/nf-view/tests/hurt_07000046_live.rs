//! 07000046-interactables: the seven flying Hurt volumes (placement 0xE4, class 0x27) — the
//! falling Icicles.
//!
//! #1d..#23 are seven "Icicle" objects hanging in the island cave (x −95.4..−80.4, z 37.5..44.3
//! [D placements]), each with the flight triple non-zero: count 6, rate 0x5A (90), path
//! 14..20 [D raw +0x40/+0x42/+0x44]. Params: type 1, damage 105, gate 0 (ungated), sound 0xffff,
//! who 0x2 (the class-5 bit — a player bullet), one-shot 1 [D params; M the who-mask law
//! Hurt_Update 0x17e5d0:57-95 — the bullet contributes class 5's bit 2, the player's class 2
//! brings 0x10 which 0x2 lacks, so a body touch never starts one]. State 1 flies the volume
//! along its 0x19 key path (6 keys, a 28.4 m vertical drop from y ≈ 20.04 to ≈ −8.36 [D
//! path_blocks 14..20]) with `Spline_Interp(t, ext+0x08, 0, count−1, obj+0x90, 0, 2)` [M
//! 0x17e5d0:121-123], `t += (count−1)·(1/60)·(60/rate)` per tick [M 0x17e870 disasm
//! 0x17e968-0x17e998; the constants 1/60 and 60.0 [D ACTION.ELF 0x30d0dc/0x30d0d0]] — a
//! traversal is `rate` = 90 ticks (1.5 s); passing count−1 resets t and the one-shot parks
//! state 2 (retire) [M :129-132]. The damage does not flow through Hurt_Update: the contact
//! arm applies it (`Player_DealWithObjHit` 0x194458 → `Player_HandlePain`), so a falling
//! icicle hurts Bond it passes through and never hurts him clear of it.
//!
//! Bond is placed under Icicle #1d (the volume at (−85.5, 20.0, 37.9)) after `free_bond`, as
//! the merged 07000008 kill-floor harness does; the walk from the archive's start lies outside
//! this sandbox's window. Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn free_bond(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
}

/// Settle Bond onto the ground at (x, z): drop from 2 m above the y given and step until
/// grounded (the port's capsule/ground law does the rest).
fn place(scene: &mut nf_view::scene::Scene, x: f32, y: f32, z: f32) {
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [x, y + 2.0, z];
        sim.bond.pitch = 0.0;
    }
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..240 { sim.step_bond(Input::default(), false, false, false); if sim.bond.grounded { break; } }
}

/// The Icicle volume the tests share: the first 0xE4 placement with a path (#1d).
fn icicle<'a>(sim: &'a nf_view::scene::Sim) -> &'a nf_game::hurt::Hurt {
    sim.hurts.iter().find(|h| h.path.is_some()).expect("the level loads a flying hurt volume")
}

#[test]
fn the_shot_icicle_flies_its_path_at_the_game_pace_and_hurts_bond_in_its_way() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000046.bin", false).expect("load 07000046");
    free_bond(&mut scene);
    // 16.5 m south of Icicle #1d: the hang spot sits at a ~59° elevation, well inside the
    // port's fire-ray reach, so a real bullet can start the flight [M the class-5 starter].
    place(&mut scene, -76.0, -6.0, 35.0);
    eprintln!("[spot] after settle {:?}", scene.sim.as_ref().unwrap().bond.pos);
    let health0 = scene.sim.as_ref().unwrap().drones.health.health;
    let mut started_at = None;
    let mut fall_profile: Vec<(u32, f32)> = Vec::new();
    let target_mid = {
        let sim = scene.sim.as_ref().unwrap();
        let h = icicle(sim);
        [(h.lo[0] + h.hi[0]) * 0.5, (h.lo[1] + h.hi[1]) * 0.5, (h.lo[2] + h.hi[2]) * 0.5]
    };
    for t in 1..=1600u32 {
        let sim = scene.sim.as_mut().unwrap();
        let h = icicle(sim);
        if h.flying {
            if started_at.is_none() { started_at = Some(t); }
            let y = h.pose[3][1];
            if fall_profile.last().map(|(_, ly)| (ly - y).abs() > 0.01).unwrap_or(true) {
                fall_profile.push((t, y));
            }
            if h.fired { break; }
            // The icicle fell on its own while Bond stayed at the shooting spot: move him under
            // the path mid-fall (the harness positioning the merged 07000008 harness also uses)
            // so the contact arm's damage law is exercised on the real flight.
            if t == started_at.unwrap() + 20 {
                place(&mut scene, -85.5, -6.0, 37.88);
            }
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            continue;
        }
        let sim = scene.sim.as_mut().unwrap();
        let h = icicle(sim);
        let hang = h.pose[3];
        let muzzle_y = sim.bond.pos[1] + 1.6;
        let muzzle = [sim.bond.pos[0], muzzle_y, sim.bond.pos[2]];
        let segs = sim.drones.bullet_flight_segments();
        if let Some(&(a, b)) = segs.last() {
            let md = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ml = (md[0] * md[0] + md[1] * md[1] + md[2] * md[2]).sqrt().max(1e-6);
            let wd = [target_mid[0] - muzzle[0], target_mid[1] - muzzle[1], target_mid[2] - muzzle[2]];
            let wl = (wd[0] * wd[0] + wd[1] * wd[1] + wd[2] * wd[2]).sqrt().max(1e-6);
            let (mp, wp) = ((md[1] / ml).asin(), (wd[1] / wl).asin());
            let (myaw, wyaw) = (md[0].atan2(md[2]), wd[0].atan2(wd[2]));
            let dyaw = (wyaw - myaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
            let turn = dyaw.clamp(-0.06, 0.06);
            let look = ((wp - mp) / std::f32::consts::FRAC_PI_2).clamp(-0.06, 0.06);
            let aimed = dyaw.abs() < 0.06 && (wp - mp).abs() < 0.12;
            let press = aimed && t % 6 == 0;
            if t % 100 == 0 { eprintln!("[cal t{t}] mp {mp:.3} wp {wp:.3} myaw {myaw:.3} wyaw {wyaw:.3}"); }
            sim.step_bond(Input { turn, look, aiming: true, ..Input::default() }, press, false, false);
            continue;
        }
        let dx = target_mid[0] - muzzle[0];
        let dz = target_mid[2] - muzzle[2];
        let dy = target_mid[1] - muzzle[1];
        let want_yaw = dx.atan2(dz);
        let want_pitch = (dy.atan2((dx * dx + dz * dz).sqrt().max(1e-4)) / std::f32::consts::FRAC_PI_2).clamp(-1.0, 1.0);
        let dyaw = (want_yaw - sim.bond.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
        let turn = dyaw.clamp(-0.05, 0.05);
        let look = (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05);
        let press = dyaw.abs() < 0.05 && (want_pitch - sim.bond.pitch).abs() < 0.05 && t % 6 == 0;
        sim.step_bond(Input { turn, look, aiming: true, ..Input::default() }, press, false, false);
    }
    let sim = scene.sim.as_mut().unwrap();
    let start = started_at.expect("the real shot started the flight");
    let h = icicle(sim);
    eprintln!("[flight] start {start} profile {fall_profile:?}");
    assert!(h.fired, "the one-shot icicle retired after its traversal [M 0x17e5d0:129-132]");
    let duration = fall_profile.last().map(|(tk, _)| tk - start).unwrap_or(0);
    assert!((85..=95).contains(&duration), "the traversal took ~90 ticks (the game's rate), got {duration}");
    assert!(h.pose[3][1] < -7.0, "the icicle reached the cave floor (the path's last key)");
    let health1 = sim.drones.health.health;
    eprintln!("[damage] health {health0} → {health1}");
    assert!(health0 - health1 >= 105.0, "the falling icicle hurt Bond in its way (the contact arm's 105 [D param 1; the 0x46 normal factor 1.0]), got {}", health0 - health1);
}

#[test]
fn the_shot_icicle_never_hurts_bond_clear_of_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000046.bin", false).expect("load 07000046");
    free_bond(&mut scene);
    place(&mut scene, -76.0, -6.0, 35.0);
    let health0 = scene.sim.as_ref().unwrap().drones.health.health;
    let mut started = false;
    let target_mid = {
        let sim = scene.sim.as_ref().unwrap();
        let h = icicle(sim);
        [(h.lo[0] + h.hi[0]) * 0.5, (h.lo[1] + h.hi[1]) * 0.5, (h.lo[2] + h.hi[2]) * 0.5]
    };
    for t in 1..=1600u32 {
        let sim = scene.sim.as_mut().unwrap();
        let h = icicle(sim);
        if h.fired { break; }
        if h.flying {
            started = true;
            sim.step_bond(Input::default(), false, false, false); // Bond stays at the shooting spot
            continue;
        }
        let sim = scene.sim.as_mut().unwrap();
        let h = icicle(sim);
        let hang = h.pose[3];
        let muzzle_y = sim.bond.pos[1] + 1.6;
        let muzzle = [sim.bond.pos[0], muzzle_y, sim.bond.pos[2]];
        let segs = sim.drones.bullet_flight_segments();
        if let Some(&(a, b)) = segs.last() {
            let md = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ml = (md[0] * md[0] + md[1] * md[1] + md[2] * md[2]).sqrt().max(1e-6);
            let wd = [target_mid[0] - muzzle[0], target_mid[1] - muzzle[1], target_mid[2] - muzzle[2]];
            let wl = (wd[0] * wd[0] + wd[1] * wd[1] + wd[2] * wd[2]).sqrt().max(1e-6);
            let (mp, wp) = ((md[1] / ml).asin(), (wd[1] / wl).asin());
            let (myaw, wyaw) = (md[0].atan2(md[2]), wd[0].atan2(wd[2]));
            let dyaw = (wyaw - myaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
            let turn = dyaw.clamp(-0.06, 0.06);
            let look = ((wp - mp) / std::f32::consts::FRAC_PI_2).clamp(-0.06, 0.06);
            let aimed = dyaw.abs() < 0.06 && (wp - mp).abs() < 0.12;
            let press = aimed && t % 6 == 0;
            sim.step_bond(Input { turn, look, aiming: true, ..Input::default() }, press, false, false);
            continue;
        }
        let dx = target_mid[0] - muzzle[0];
        let dz = target_mid[2] - muzzle[2];
        let dy = target_mid[1] - muzzle[1];
        let want_yaw = dx.atan2(dz);
        let want_pitch = (dy.atan2((dx * dx + dz * dz).sqrt().max(1e-4)) / std::f32::consts::FRAC_PI_2).clamp(-1.0, 1.0);
        let dyaw = (want_yaw - sim.bond.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
        let turn = dyaw.clamp(-0.05, 0.05);
        let look = (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05);
        let press = dyaw.abs() < 0.05 && (want_pitch - sim.bond.pitch).abs() < 0.05 && t % 6 == 0;
        sim.step_bond(Input { turn, look, aiming: true, ..Input::default() }, press, false, false);
    }
    let sim = scene.sim.as_mut().unwrap();
    assert!(started, "the real shot started the flight");
    let h = icicle(sim);
    assert!(h.fired, "the icicle flew and retired");
    let health1 = sim.drones.health.health;
    eprintln!("[clear] health {health0} → {health1}");
    assert_eq!(health1, health0, "clear of the fall, Bond is untouched");
}
