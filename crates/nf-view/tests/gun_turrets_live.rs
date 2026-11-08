//! gun-turrets-live: the 0700000a wall guns deploy and shoot through the real paths.
//!
//! The game's rules, cited:
//! - `GT_Update` 0x180498 top: BOTH scripts `Script_Update` every tick; the ARMING gate (case 1)
//!   waits for both SCRIPTINFO+0x9c delays != 0 — the looping deploy anims park on their event-3
//!   delays (0x06000041 stream 10 / 0x06000042 stream 8 end parked, p0 = 0 → −1) [D the level's
//!   script streams; M 0017ffa0:112-116 plays both looping + one update].
//! - DORMANT (case 0): only the facing-1 gate runs — the enable channel (param 4, ext+0xe8) must
//!   be ON and the disable channel (param 5) OFF, then `GT_Track` must find a target; the tail
//!   clears both +0x9c (the rewind) and → ARMING [M 0x180498:205-230]. Before ch 18 nothing
//!   deploys.
//! - The wall guns (facing 1, param 0 = 1): HP 0xffff [M 0017ffa0:92-96], candidates = Bond only
//!   (drones/copters are facing-0-only [M 0x17f9c8:101,119]), the front-cone dot test against the
//!   create-time ext+0x80 = −up of the placement matrix [M 0017ffa0:144-146, 0x17f9c8:252-257].
//! - COMBAT (case 4): the fire cadence ext+0xe0 > 0.1 s per shot, the ammo (ext+0xf0) decrements
//!   only while > 0, `Bullet_init(-1, owner, barrel, weapon 0x2c55d0 = record 96, muzzle, dir)`
//!   [M 0x180498:319-348].
//!
//! Fixtures [D placements dump]: 0700000a #636/#637 — "GunSamuri-Placement" wall guns, facing 1,
//! enable ch 18, scripts 0x06000041/0x06000042 (the loader's id→hash resolution), at
//! (9.84, 2.28, 17.48) yaw −π/2 and (26.18, 2.28, 4.98).
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::player::Input;

fn settle(scene: &mut nf_view::scene::Scene, cap: u32) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..cap {
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
}

/// The wall gun #636's index in the record list (the first ch-18 facing-1 turret).
fn wall_gun(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.gt_turrets.iter().position(|t| t.param4 == 18 && t.facing == 1)
        .expect("the fixture wall gun (ch 18, facing 1)")
}

#[test]
fn before_the_enable_channel_nothing_deploys() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    settle(&mut scene, 3000);
    let sim = scene.sim.as_mut().unwrap();
    let ti = wall_gun(sim);
    assert_eq!(sim.drones.gt_turrets[ti].state, nf_game::gt::gt_state::DORMANT,
        "the wall gun sits DORMANT at load [M 0017ffa0:117]");
    assert_eq!(sim.drones.gt_turrets[ti].hp, 0xffff, "facing 1 → the 0xffff HP selector [M 0017ffa0:92-96]");
    assert!(sim.drones.gt_turrets[ti].script_player_a != usize::MAX, "the deploy script is mounted");
    // Ten seconds of aimless standing: ch 18 is down, so case 0's gate returns every tick —
    // no ARMING, no COMBAT, no shots, no health change.
    let h0 = sim.drones.health.health;
    for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.gt_turrets[ti].state, nf_game::gt::gt_state::DORMANT,
        "the enable channel gate holds the gun in DORMANT [M 0x180498:206-212]");
    assert_eq!(sim.drones.health.health, h0, "no deploy → no shots → no damage");
    assert!(sim.drones.gt_turrets[ti].scripts_done, "the scripts stay parked at their load beat");
}

#[test]
fn raising_ch18_deploys_the_wall_gun_and_its_shots_wound_bond() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    settle(&mut scene, 3000);
    let sim = scene.sim.as_mut().unwrap();
    let ti = wall_gun(sim);
    let gun_pos = sim.drones.gt_turrets[ti].pos;
    // Stand the tester in the gun's front cone, inside its 50-unit track range, on the floor
    // below the mount (the create-time fire direction is −up of the placement matrix — the
    // floor level is in the cone [M 0017ffa0:144-146, 0x17f9c8:252-257]).
    let fd = sim.drones.gt_turrets[ti].fire_dir;
    sim.bond.pos = [gun_pos[0] + fd[0] * 40.0, 0.0, gun_pos[2] + fd[2] * 40.0];
    // The real channel path (the game's switch/script write — `switch_channels[18] = 1`).
    let frame = sim.drones.frame;
    sim.drones.channels.set(18, 1, frame);
    // The deploy: the rewind → the looping anims replay → both parks re-set → ARMING completes
    // [M 0x180498:205-238]; the script loop is 180 ticks (0x06000041's length [D]).
    let mut reached_combat = false;
    let mut hp_at_combat = 0u16;
    for _ in 0..900 {
        sim.step_bond(Input::default(), false, false, false);
        let t = &sim.drones.gt_turrets[ti];
        if !reached_combat && t.state == nf_game::gt::gt_state::COMBAT {
            reached_combat = true;
            hp_at_combat = t.hp;
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(reached_combat, "the wall gun reached COMBAT within 15 s of ch 18 — state {}",
        sim.drones.gt_turrets[ti].state);
    // The shots: the 0.1 s cadence fires 6/second while the target holds [M 0x180498:317-348].
    assert!(sim.drones.gt_turrets[ti].hp < hp_at_combat,
        "the ammo counter decremented per shot [M 0x180498:321-323] ({} → {})",
        hp_at_combat, sim.drones.gt_turrets[ti].hp);
    assert!(sim.drones.health.health < 100.0, "the turret's shots reached Bond through the bullet path (health {:.1})",
        sim.drones.health.health);
}

/// THE SPREAD PIN (turret-round-spread): the wall gun's rounds are NOT dead-on. `Bullet_init`
/// 0x12c5a0's spread block is unconditional [M 0x12c7f4-0x12c858]: three shared-stream draws
/// per round (`FRand_MVar2(2s)·0.0014`, `FRand(2π)`, `FRand(π)`) and the spherical offset adds
/// to the direction; the turret's owner obj is class 0x36 [M 0017ffa0:77], so the scale arm
/// stays 1.0 and the bloom never computes — record 96's full +0x24 = 4.0 cone [L
/// ee_start_05.bin @0x2c55d0+0x24]. `GT_Update` adds the fourth draw per shot, the
/// direction-w-pad `Rand_FRand(2π)` [M 0x180498:349]. The pin: the burst's per-round
/// directions SCATTER (the same-seed golden sequence below) and the shared-stream draw count
/// at each shot matches the game-ordered goldens (3 in `Bullet_init` + 1 in `GT_Update`).
/// Sister (dead-on): the 0x400000 arm leaves the direction EXACTLY the aim while still drawing
/// the three — pinned at the unit level (`bullet::tests::cone_draws_three_even_when_dead_on`).
#[test]
fn the_wall_gun_burst_scatters_and_draws_in_the_game_order() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    settle(&mut scene, 3000);
    // The setup + the deploy wait, in one mut scope.
    let (ti, hp_at_combat, health_at_combat) = {
        let sim = scene.sim.as_mut().unwrap();
        let ti = wall_gun(sim);
        let gun_pos = sim.drones.gt_turrets[ti].pos;
        let fd = sim.drones.gt_turrets[ti].fire_dir;
        sim.bond.pos = [gun_pos[0] + fd[0] * 40.0, 0.0, gun_pos[2] + fd[2] * 40.0];
        let frame = sim.drones.frame;
        sim.drones.channels.set(18, 1, frame);
        // The deploy: the rewind + the looping anims → ARMING completes ~180 ticks [M 0x180498:205-238]
        // (the existing test's own 900-tick COMBAT wait).
        let mut reached_combat = false;
        for _ in 0..900 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.drones.gt_turrets[ti].state == nf_game::gt::gt_state::COMBAT { reached_combat = true; break; }
        }
        assert!(reached_combat, "the wall gun reached COMBAT [M 0x180498:205-238]");
        (ti, sim.drones.gt_turrets[ti].hp, sim.drones.health.health)
    };
    // Track the burst: the turret's rounds impact within their spawn tick (speed 15.48/tick,
    // the tester 40 u away), so the pattern is the per-shot HEALTH DELTA sequence (the spread
    // moves each round's impact across Bond's part boxes) plus the shared-stream draw count at
    // each shot. Dead-on rounds at a standing target would repeat one identical number.
    let mut hp_prev = hp_at_combat;
    let mut health_prev = health_at_combat;
    let mut pattern: Vec<(f32, u64)> = Vec::new();
    for _ in 0..300 {
        let (shot, delta, draws, health_now) = {
            let sim = scene.sim.as_mut().unwrap();
            sim.step_bond(Input::default(), false, false, false);
            let hp_now = sim.drones.gt_turrets[ti].hp;
            let shot = hp_now < hp_prev;
            hp_prev = hp_now;
            let delta = health_prev - sim.drones.health.health;
            let draws = sim.drones.rng.draws;
            let health_now = sim.drones.health.health;
            (shot, delta, draws, health_now)
        };
        health_prev = health_now;
        if shot && delta > 0.0 {
            pattern.push((delta, draws));
            eprintln!("[turret-spread] shot {} damage {:.4} draws {}", pattern.len(), delta, draws);
        }
        if pattern.len() >= 12 || health_now < 25.0 { break; }
    }
    assert!(pattern.len() >= 8, "the burst produced {} damaging shots in 5 s", pattern.len());
    // The pattern golden: bit-exact (the fixed-60 Hz deterministic sim, the shared stream).
    // Any change to the cone draws' order or the spread law moves these numbers.
    let measured: Vec<(u32, u64)> = pattern.iter().map(|(d, dr)| ((*d * 10000.0).round() as u32, *dr)).collect();
    eprintln!("[turret-spread] GOLDEN {:?}", measured);
    // The goldens, re-measured 2026-11-06 on the joined stream-batch tree (bit-exact, the
    // fixed-60 Hz sim): the per-shot damage is STILL the record's flat player damage (the
    // cylinder path has no part boxes, so the ±0.0112 rad cone at 40 u keeps every round on one
    // number — the damage law did not move), and the draw counts are the shared-stream positions
    // at each shot. The stream correction (the guards' per-shot muzzle-flash draws, live-confirmed
    // 0x178f68 ×40) moved the burst's start (+12) and put 460-draw gaps between rounds where a
    // level consumer (a guard's own shot) now spends its game draws between turret rounds — the
    // +454 floor is the game's 4 (the cone 3 + the GT pad) + the level's per-tick consumers.
    // stream-batch-2 re-pin: EVERY draw moved +8 as a block (damage identical, the gaps
    // identical) — the joined tree's ambient consumers (gas puffs, guard muzzle flashes, guard
    // radii-driven fights) spent exactly 8 more shared draws upstream of the burst; the turret's
    // own per-shot law did not move.
    assert_eq!(measured, &[
        (24000, 11693), (24000, 12153), (24000, 12607), (24000, 13067),
        (24000, 13521), (24000, 13981), (24000, 14435), (24000, 14895),
        (24000, 15349), (24000, 15809), (24000, 16263), (24000, 16723),
    ], "the burst's damage/draw pattern matches the game-ordered stream");
}
