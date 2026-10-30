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
    // Stand the tester OFF the mount column, inside the gun's front cone (the create-time fire
    // direction is −up of the placement matrix — every floor point qualifies, dot = +1.19/dist
    // [M 0017ffa0:144-146, 0x17f9c8:252-257]), within the 50-unit track range, and LOS-clear from
    // the muzzle [M 0x17f9c8:232-242 — the ray starts at t.pos + fire_dir · ext+0xd0, the [L]
    // 1.9865224]. Directly beneath the mount the game's own shot geometry can never wound: the
    // aim matrix looks from obj+0x30 AT the target's obj+0x30 [M 0x180498 case 4 Mat_LookAt
    // 0x180498:262-263], so for a same-column target the muzzle sits 0.29 above the deck and the
    // bullet dies in the deck top face — the game's shots only reach a target the muzzle ray
    // reaches diagonally. Standing height = the floor point + 1.1 [M 0x190d88:0x190ee0-0x190f18]
    // (a teleport to the bare floor point falls out of the world — measured y = −569.8).
    let fd = sim.drones.gt_turrets[ti].fire_dir;
    let muzzle = [gun_pos[0] + fd[0] * sim.drones.gt_turrets[ti].muzzle_offset,
                  gun_pos[1] + fd[1] * sim.drones.gt_turrets[ti].muzzle_offset,
                  gun_pos[2] + fd[2] * sim.drones.gt_turrets[ti].muzzle_offset];
    let mut spot = None;
    'outer: for ox in [12.0f32, -12.0, 8.0, -8.0, 6.0, -6.0, 4.0, -4.0, 16.0, -16.0, 20.0, -20.0, 28.0, -28.0] {
        for oz in [10.0f32, -10.0, 6.0, -6.0, 3.0, -3.0, 0.0] {
            let (x, z) = (gun_pos[0] + ox, gun_pos[2] + oz);
            let a = [x, gun_pos[1] - 0.3, z];
            let b = [x, gun_pos[1] - 40.0, z];
            let Some(y) = sim.world.line_of_sight_hit(a, b, 0).map(|f| a[1] + (b[1] - a[1]) * f + 1.1) else { continue };
            let d = [x - gun_pos[0], y - gun_pos[1], z - gun_pos[2]];
            let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if m < 3.0 || m > 45.0 { continue; }
            let dot = fd[0] * d[0] / m + fd[1] * d[1] / m + fd[2] * d[2] / m;
            if dot < 0.01 { continue; } // the front-cone gate [M 0x17f9c8:252-257]
            let sph = nf_game::player::Bond::sphere_centre_at([x, y, z], sim.bond.yaw);
            if !sim.world.line_of_sight(muzzle, sph, nf_game::world::LOS_TURRET_TRACK) { continue; }
            spot = Some([x, y, z]);
            break 'outer;
        }
    }
    sim.bond.pos = spot.expect("a standing in-cone spot near the wall gun");
    // The real channel path (the game's switch/script write — `switch_channels[18] = 1`).
    let frame = sim.drones.frame;
    sim.drones.channels.set(18, 1, frame);
    // The deploy: the rewind → the looping anims replay → both parks re-set → ARMING completes
    // [M 0x180498:205-238]; the script loop is 180 ticks (0x06000041's length [D]).
    let mut reached_combat = false;
    let mut hp_at_combat = 0u16;
    let mut min_hp = u16::MAX;
    for _ in 0..900 {
        sim.step_bond(Input::default(), false, false, false);
        let t = &sim.drones.gt_turrets[ti];
        if !reached_combat && t.state == nf_game::gt::gt_state::COMBAT {
            reached_combat = true;
            hp_at_combat = t.hp;
        }
        min_hp = min_hp.min(t.hp);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(reached_combat, "the wall gun reached COMBAT within 15 s of ch 18 — state {}",
        sim.drones.gt_turrets[ti].state);
    // The shots: the 0.1 s cadence fires 6/second while the target holds [M 0x180498:317-348].
    // The counter is read as the run's minimum: the cycle does not necessarily end in COMBAT —
    // a dead target (the shots ARE lethal here) drops the candidate, the gun re-holsters and,
    // facing 1, drops its 0x52 pickup and deletes itself [M 0x180498:246-257] — the fixed `ti`
    // then names the next ch-18 gun (#637), whose untouched 0xffff would mask #636's spent shots.
    assert!(min_hp < hp_at_combat,
        "the ammo counter decremented per shot [M 0x180498:321-323] ({} → {})",
        hp_at_combat, min_hp);
    assert!(sim.drones.health.health < 100.0, "the turret's shots reached Bond through the bullet path (health {:.1})",
        sim.drones.health.health);
}
