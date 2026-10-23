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
    sim.bond.pos = [gun_pos[0] + fd[0] * 12.0, 0.0, gun_pos[2] + fd[2] * 12.0];
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
