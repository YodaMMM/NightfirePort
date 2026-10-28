//! level-4a-rappel-vanish live tests: the 0700004a lift-ride rappel guards — a shot rope guard
//! must do what the game's does (free fall down the shaft, deleted only when the drawn-grace
//! expires), not vanish the tick he is hit.
//! Run with: `NF_ISO=… cargo test -p nf-view --test rappel_vanish_0700004a_live -- --test-threads=1`
//!
//! Disc reach [D] (nf-formats parse of 0700004a.bin): exactly three 0x06000892 placements among
//! the level's 38 drones — #282 (−1.605, −7.518, −4.490) min-difficulty 1, switch ch 0x3d;
//! #288 (−1.537, −50.149, −4.861) min-difficulty 2, ch 0x19; #283 (−1.226, −95.565, −4.645)
//! min-difficulty 1, ch 0x4f. Every other 4a drone carries 0x06000000. At difficulty 1 exactly
//! #282 and #283 spawn [M Drone_Create 0x136be0:49]. The channels are the shaft plates' outs
//! [D docs/levels/0700004a.md §3]: 0x3d at y −31.1, 0x19 at −81.5, 0x4f at −125.2 — each
//! abseiler wakes as the lift passes well below him.
//!
//! The game's shot-rope-guard chain [M]: a killing bullet routes by the mode byte +0xC5 —
//! 0x1b/0x1c → AbseilDeath 0x92 [M NDrone2_BulletImpact 0x1461c8:44-52]; the ENTER on
//! 0x700004a sets `+0x4F8 = (+0x4F8 & ~0x20) | 4` and clears the rope flag +0x23 [M
//! 0x169b28:18-21, 28]. With bit 0x20 clear `NDrone2_DoCollision` 0x149ec8:12-14 returns 0 —
//! NO collision — while `NDrone2_DoGravity` 0x141e58:12-15 returns 1 on bit 4 BEFORE the
//! 0x20/0x200 folds, and `Drone_CollisionHandler` 0x140968:152-192's DoGravity arm integrates
//! the corpse collision-free: velocity += WldGravity·REC_FRAME_RATE (the feet flag anim+0x60 & 8
//! stays clear — no probe runs in this branch), pos += velocity·REC_FRAME_RATE, no damping (the
//! ×0.98 tail is the rope arm's [M 0x140968:139-141]) and no settle [M 0x140968:292-295]. The
//! UPDATE keeps the corpse while it has been drawn recently — `obj+0xFC != 0` early-returns
//! [M 0x169b28:41-44] — and once the drawn-grace is out (obj+0xFC: 2 per drawn tick
//! [M 0x1e62d8:71], −1 per control pass [M 0x1336f8:242-246]) calls `NDrone2_Enable(0)` +
//! `obj+0xFE |= 1` (the delete-me flag) [M 0x169b28:45-46]: the corpse is removed on the 3rd
//! undrawn control pass (docs/systems/corpse-persistence.md). The old port deleted the corpse
//! on the per-tick cull's DRAWN flag — the guard vanished the tick he was hit.
//!
//! The shot cadence is the semi-auto one-shot-per-press-edge (3 pressed ticks, 7 released), the
//! aim re-set at the top of each volley (abseil_damage_live's shape).

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The lift platform's keyed entity y (the Entity stream's object 0x020005b4 [D 0x06000571]),
/// the fix_finale_live re-seat helper.
fn platform_y(sim: &nf_view::scene::Sim) -> Option<f32> {
    sim.drones.scripts.objects.list.iter().flatten().find_map(|o| match o {
        nf_game::nis::Obj::Entity { hash: 0x0200_05b4, matrix, .. } => Some(matrix[3][1]),
        _ => None,
    })
}

/// Aims Bond's eye at `target` (civilian_no_shoot_live's aim_at).
fn aim_at(sim: &mut nf_view::scene::Sim, target: [f32; 3]) {
    let head = sim.bond.head();
    let d = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
    let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1].atan2(horiz)) / (std::f32::consts::FRAC_PI_2);
}

/// The shot: Bond rides the lift from the spawn with real input; abseiler #282 wakes on the
/// shaft plate's ch 0x3d [D], ropes, and Bond's real trigger rounds kill him mid-slide. The
/// death must be the game's 0700004a chain: the corpse stays present the tick he is hit (the
/// old port deleted him there — the reported vanish), free-falls with the game's exact
/// gravity integration and no collision, persists while drawn, and is soft-deleted on the 3rd
/// undrawn control pass.
#[test]
fn a_shot_rope_guard_falls_instead_of_vanishing() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
        let abseilers: Vec<usize> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.mode == 0x1c).map(|(i, _)| i).collect();
        assert_eq!(abseilers.len(), 2, "the two difficulty-1 0x06000892 placements #282/#283 [D + M 0x136be0:49], got {abseilers:?}");
        assert_eq!(sim.drones.list[abseilers[0]].pos[1], -7.517_948, "the #282 shaft perch [D]");
        assert_eq!(sim.drones.list[abseilers[0]].switch_channel, 0x3d, "the #282 wake channel = the −31.1 plate's out [D]");
        assert!(sim.drones.list.iter().all(|d| d.mode != 0x1b), "no placement uses 0x0600089c [D]");
    }
    // Ride down with real input (the fix_finale_live shape: re-seat onto the platform's keyed
    // matrix so the plates fire as the car passes) and fire the semi-auto cadence at the first
    // live abseiler's chest from the wake on.
    let mut kill: Option<(u32, f32)> = None; // (tick, death_y)
    let mut pre_log: Vec<String> = Vec::new();
    for t in 0..4000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0;
        if t % 30 == 0 {
            if let Some(y) = platform_y(sim) { sim.bond.pos = [3.0, y + 1.2, -3.4]; }
        }
        let target = sim.drones.list.iter().position(|d| d.present && d.mode == 0x1c && d.enabled)
            .map(|i| sim.drones.list[i].pos);
        if let Some(p) = target {
            aim_at(sim, [p[0], p[1] + 1.0, p[2]]);
            let press = t % 10 < 3;
            sim.step_bond(Input::default(), press, false, false);
        } else {
            sim.step_bond(Input::default(), false, false, false);
        }
        let sim = scene.sim.as_ref().unwrap();
        if let Some(i) = sim.drones.list.iter().position(|d| d.mode == 0x1c) {
            let d = &sim.drones.list[i];
            if d.state == state::ABSEIL_DEATH && d.health <= 0.0 {
                kill = Some((t, d.pos[1]));
                break;
            }
            pre_log.push(format!("t{t}: state {:#04x} pos ({:.2},{:.2},{:.2}) hp {:.0}", d.state, d.pos[0], d.pos[1], d.pos[2], d.health));
        }
    }
    let (kt, ky) = kill.expect("the wake → rope chain ran and Bond's rounds killed the roper");
    assert!(pre_log.iter().any(|l| l.contains("state 0x8f")), "the guard was ON the rope (Slide 0x8f) when the kill landed [M 0x169730], last:\n{}", pre_log.last().unwrap_or(&"".into()));
    {
        let sim = scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[sim.drones.list.iter().position(|d| d.mode == 0x1c).unwrap()];
        assert_eq!(d.state, state::ABSEIL_DEATH, "mode 0x1c kill → AbseilDeath 0x92 [M 0x1461c8:44-52]");
        assert!(!d.air, "+0x23 cleared at the ENTER — off the rope [M 0x169b28:28]");
        assert!(!d.bounded && d.keeps_gravity, "the port's +0x4F8 stand-ins for (& ~0x20) | 4 [M 0x169b28:18-21]");
        assert!(d.present && d.drawn && d.drawn_grace > 0,
            "the corpse does NOT vanish the kill tick: drawn recently (obj+0xFC != 0) → the UPDATE early-returns [M 0x169b28:41-44] — the old port deleted him here");
    }
    // The fall, per tick: the game's DoGravity arm integrates velocity += WldGravity·(1/60) then
    // pos += velocity·(1/60), with no damping and no collision — pin the integration exactly for
    // the first 12 ticks and the no-landing pass through the shaft's garrison/plate lines.
    let r = 1.0f32 / 60.0;
    let mut v = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.list[sim.drones.list.iter().position(|d| d.mode == 0x1c).unwrap()].velocity
    };
    let mut undrawn_run = 0u32; // consecutive undrawn control passes (the flicker resets — a drawn tick refreshes obj+0xFC to 2 [M 0x1e62d8:71])
    let mut deleted_at: Option<u32> = None;
    let mut lowest = ky;
    for t in (kt + 1)..=(kt + 400) {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0;
        if t % 30 == 0 {
            if let Some(y) = platform_y(sim) { sim.bond.pos = [3.0, y + 1.2, -3.4]; }
        }
        // The player keeps the crosshair on the falling guard (his chest) — the drawn gate
        // refreshes while he is in view, exactly the game's View_AddObjects refresh.
        let i = sim.drones.list.iter().position(|d| d.mode == 0x1c);
        let aim = i.filter(|&i| sim.drones.list[i].present).map(|i| sim.drones.list[i].pos);
        if let Some(p) = aim {
            aim_at(sim, [p[0], p[1] + 1.0, p[2]]);
        }
        sim.step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let Some(i) = sim.drones.list.iter().position(|d| d.mode == 0x1c) else { continue };
        let d = &sim.drones.list[i];
        eprintln!("t{t}: state {:#04x} danim {} pos ({:.2},{:.2},{:.2}) vel {:.2} drawn {} grace {} present {} on_surface {}",
            d.state, d.anim.danim, d.pos[0], d.pos[1], d.pos[2], d.velocity[1], d.drawn, d.drawn_grace, d.present, d.on_surface);
        lowest = lowest.min(d.pos[1]);
        if d.present {
            if d.drawn { undrawn_run = 0; } else { undrawn_run += 1; }
            assert!(undrawn_run <= 2, "the corpse is deleted on the 3rd undrawn control pass, never earlier [M 0x169b28:41-46], run {undrawn_run} at t{t}");
            assert!(!d.on_surface && d.in_air, "the corpse never lands — DoCollision 0 with bit 0x20 clear [M 0x149ec8:12-14]");
            assert!(d.state == state::ABSEIL_DEATH, "the corpse stays in AbseilDeath until the soft delete [M 0x169b28 case 3]");
            if t <= kt + 12 {
                v[1] += -9.8 * r;
                assert!((d.velocity[1] - v[1]).abs() < 1e-3,
                    "the exact gravity integration velocity += WldGravity·r, no damping [M 0x140968:173-191]: {} vs {}", d.velocity[1], v[1]);
                assert!(d.pos[1] < ky, "falling below the kill point [M 0x140968:185-191]");
            }
        } else {
            assert!(undrawn_run >= 2, "the delete follows the drawn-grace expiry, not a drawn tick [M 0x169b28:41-46]");
            deleted_at = Some(t);
        }
    }
    let deleted_at = deleted_at.expect("the corpse is soft-deleted once the drawn-grace is out [M 0x169b28:45-46]");
    assert!(deleted_at - kt > 60, "the corpse fell for {}/60 s while drawn before the delete — not the kill-tick vanish", deleted_at - kt);
    // The fall's LENGTH is the drawn-grace's expiry point — obj+0xFC is refreshed by the game's
    // View draw pass, which the port stands in for with the frustum + five bullet-ray probe
    // (`visible_to`, drone.rs — the documented [S] stand-in for the cel/portal cull, no
    // decompilable shape on the AI side). How far the corpse falls before three undrawn control
    // passes therefore rides the stand-in's occlusion (the shaft's ledge/rope lines), not a
    // decompile fact: the game-cited mechanics above (the exact integration, no landing, the
    // 3rd-undrawn delete, the kill-tick presence) are the hard pins. [S] unverified: the
    // distance observation only.
    eprintln!("[4a] the corpse fell {:.1} units below the kill point before the drawn-grace delete ([S] the drawn window rides the port's ray stand-in)", ky - lowest);
    assert!(lowest < ky, "the corpse fell below the kill point [M 0x140968:185-191]");
}

/// Sister: the difficulty-1 spawn census [D + M 0x136be0:49] — exactly the two min-difficulty-1
/// abseiler placements exist in the sim (the third, #288 at −50.1 ch 0x19, is gated to
/// difficulty ≥ 2 by its param 2 and is absent here; the count proves the gate).
#[test]
fn the_disc_abseil_census_at_difficulty_1() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
    let sim = scene.sim.as_mut().unwrap();
    let modes: Vec<(usize, f32, u8)> = sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.mode == 0x1c)
        .map(|(i, d)| (i, d.pos[1], d.switch_channel)).collect();
    assert_eq!(modes.len(), 2, "difficulty 1: #282 (ch 0x3d) + #283 (ch 0x4f) [D + M 0x136be0:49], got {modes:?}");
    assert!(modes.iter().all(|&(_, _, ch)| matches!(ch, 0x3d | 0x4f)), "the difficulty-1 wake channels [D], got {modes:?}");
    assert!(modes.iter().all(|&(_, y, _)| (y - -7.518).abs() < 0.05 || (y - -95.565).abs() < 0.05),
        "the #282/#283 perch heights [D], got {modes:?}");
}
