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
//! The shot itself is hip fire through the real input path: with the aim button UP the fire
//! ray is Bond's own look direction 1:1 (`run()`'s `dir = aim`, the cross/cursor halving to
//! zero outside aim mode), so the harness aligns `bond.yaw`/`bond.pitch` onto the volume and
//! taps the semi-auto trigger — no free-aim cursor in the way (the aim-mode ray adds the
//! clamped cursor's ±13° offsets and folds over past the pitch clamp — the measured rails
//! the first harness died on). Bond is placed under Icicle #1d (the volume at
//! (−85.5, 20.0, 37.9)) after `free_bond`, as the merged 07000008 kill-floor harness does;
//! the walk from the archive's start is not driven here. Opt-in via NF_ISO.

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
        // The game's own placement (cel, prev pos, floor snap): a raw pos poke is undone by
        // the step's large-jump recovery.
        let yaw = sim.bond.yaw;
        sim.place_bond([x, y + 2.0, z], yaw);
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
    // Two clocks: `start_loop` is the loop tick the flight was first seen (the place-under
    // timer runs on it — loop ticks equal sim ticks until the place itself), `started_at` the
    // sim frame (the pace assert below — see the profile note).
    let mut start_loop = None;
    // The fall is profiled in SIM frames (`drones.frame` — one per `step_bond`), not loop
    // iterations: the mid-flight `place` steps the sim tens of ticks inside one iteration, and
    // the loop clock would read the 90-tick traversal as ~30.
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
            if start_loop.is_none() { start_loop = Some(t); started_at = Some(sim.drones.frame); }
            let y = h.pose[3][1];
            if fall_profile.last().map(|(_, ly)| (ly - y).abs() > 0.01).unwrap_or(true) {
                fall_profile.push((sim.drones.frame, y));
            }
            if h.fired { break; }
            // The icicle fell on its own while Bond stayed at the shooting spot: move him under
            // the path mid-fall (the harness positioning the merged 07000008 harness also uses)
            // so the contact arm's damage law is exercised on the real flight.
            if t == start_loop.unwrap() + 20 {
                place(&mut scene, -85.5, -6.0, 37.88);
            }
            let lethal = scene.sim.as_ref().unwrap().drones.health.health <= health0 - 105.0;
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            // The first lethal contact tick kills Bond (health0 81.25 < 105): stop there — past
            // the death NIS the world sections pause [M Game_Run 0x1c9790:30-38], Hurt_Update's
            // clock stops with them [M 0x17e5d0], and the game freezes the same way. The retire
            // pin lives in the clear sister below, whose flight runs unmolested to the wrap.
            if lethal { break; }
            continue;
        }
        // Hip fire: with the aim button UP the fire ray is Bond's own look direction —
        // `run()`'s `dir = aim = normalise(look + tx·right + ty·up)`, and the cross/cursor
        // halve every tick outside aim mode, so the ray is 1:1 with `bond.yaw`/`bond.pitch`.
        // (The abandoned aim-mode harness fed the look input into the free-aim cursor first:
        // the cursor clamps at ±0.296/±0.4 and its eased offsets add ±13° to the fire ray —
        // both rails measured in the old [cal] logs — and past the pitch clamp the ray folds
        // over. Hip fire has none of that coupling.)
        let sim = scene.sim.as_mut().unwrap();
        let muzzle = [sim.bond.pos[0], sim.bond.pos[1] + 1.6, sim.bond.pos[2]];
        let dx = target_mid[0] - muzzle[0];
        let dz = target_mid[2] - muzzle[2];
        let dy = target_mid[1] - muzzle[1];
        let want_yaw = dx.atan2(dz);
        let want_pitch = (dy.atan2((dx * dx + dz * dz).sqrt().max(1e-4)) / std::f32::consts::FRAC_PI_2).clamp(-1.0, 1.0);
        let dyaw = (want_yaw - sim.bond.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
        let turn = dyaw.clamp(-0.05, 0.05);
        let look = (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05);
        // The semi-auto's press edge, tapped while the look direction sits on the volume:
        // 0.02 rad ≈ 0.2 m across the ~10 m horizontal — inside the icicle's own extent —
        // and the tap repeats every 6 ticks, so the P2K's spread cone gets fresh tries (the
        // round itself: speed 5.0, range 50.0 [D census rec 2] — the ~33 m slant is inside).
        let press = dyaw.abs() < 0.02 && (want_pitch - sim.bond.pitch).abs() < 0.02 && t % 6 == 0;
        sim.step_bond(Input { turn, look, ..Input::default() }, press, false, false);
    }
    let sim = scene.sim.as_mut().unwrap();
    let start = started_at.expect("the real shot started the flight");
    eprintln!("[flight] start {start} profile {fall_profile:?}");
    // No retire assert on this run: the first contact tick's 105 kills Bond (health0 81.25),
    // the death NIS pauses the world sections [M Game_Run 0x1c9790:30-38] and the wrap
    // [M 0x17e5d0:129-132] lands a tick after the freeze — the game behaves identically. The
    // pace + retire pins live in the clear sister below.
    let health1 = sim.drones.health.health;
    eprintln!("[damage] health {health0} → {health1}");
    // The contact arm's amount (the 105 [D param 1; the 0x46 normal factor 1.0]) is pinned by
    // the unit layer (the delivery of self.damage); live, the 105 exceeds any health pool — the
    // observable is the lethal contact itself (the health block clamps at 0).
    assert!(health1 <= 0.0, "the falling icicle hurt Bond in its way — the contact arm's 105 [D param 1; the 0x46 normal factor 1.0] is lethal from any pool (the health block clamps at 0), got {health0} → {health1}");
}

#[test]
fn the_shot_icicle_never_hurts_bond_clear_of_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000046.bin", false).expect("load 07000046");
    free_bond(&mut scene);
    place(&mut scene, -76.0, -6.0, 35.0);
    let health0 = scene.sim.as_ref().unwrap().drones.health.health;
    let mut started = false;
    // Bond stays clear, so the flight runs unmolested to the wrap: this run carries the pace
    // and retire pins (the under-the-fall sister's world freezes on Bond's death before its
    // wrap tick). The profile is in SIM frames — see the flight pin's note.
    let mut start_frame = None;
    let mut fall_profile: Vec<(u32, f32)> = Vec::new();
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
            if start_frame.is_none() { start_frame = Some(sim.drones.frame); }
            let y = h.pose[3][1];
            if fall_profile.last().map(|(_, ly)| (ly - y).abs() > 0.01).unwrap_or(true) {
                fall_profile.push((sim.drones.frame, y));
            }
            sim.step_bond(Input::default(), false, false, false); // Bond stays at the shooting spot
            continue;
        }
        // The same hip-fire aim as the flight pin above: the fire ray is Bond's look
        // direction 1:1 with the aim button up.
        let sim = scene.sim.as_mut().unwrap();
        let muzzle = [sim.bond.pos[0], sim.bond.pos[1] + 1.6, sim.bond.pos[2]];
        let dx = target_mid[0] - muzzle[0];
        let dz = target_mid[2] - muzzle[2];
        let dy = target_mid[1] - muzzle[1];
        let want_yaw = dx.atan2(dz);
        let want_pitch = (dy.atan2((dx * dx + dz * dz).sqrt().max(1e-4)) / std::f32::consts::FRAC_PI_2).clamp(-1.0, 1.0);
        let dyaw = (want_yaw - sim.bond.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
        let turn = dyaw.clamp(-0.05, 0.05);
        let look = (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05);
        let press = dyaw.abs() < 0.02 && (want_pitch - sim.bond.pitch).abs() < 0.02 && t % 6 == 0;
        sim.step_bond(Input { turn, look, ..Input::default() }, press, false, false);
    }
    let sim = scene.sim.as_mut().unwrap();
    assert!(started, "the real shot started the flight");
    let h = icicle(sim);
    assert!(h.fired, "the one-shot icicle retired on the wrap [M 0x17e5d0:129-132]");
    let duration = fall_profile.last().map(|(tk, _)| tk - start_frame.unwrap_or(0)).unwrap_or(0);
    assert!((85..=95).contains(&duration), "the traversal took ~90 ticks (the game's rate: ext+0x0C = count−1 [M Hurt_Create 0x17e870:28-32] over ext[1] = (count−1)·(1/60)·(60/rate) [M 0x17e870 disasm 0x17e968-0x17e998]), got {duration}");
    assert!(h.pose[3][1] < -7.0, "the icicle reached the cave floor (the path's last key)");
    let health1 = sim.drones.health.health;
    eprintln!("[clear] health {health0} → {health1} (profile {}..{:?})", start_frame.unwrap_or(0), fall_profile.last());
    assert_eq!(health1, health0, "clear of the fall, Bond is untouched");
}
