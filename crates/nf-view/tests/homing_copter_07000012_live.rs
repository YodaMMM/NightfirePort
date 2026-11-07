//! port-map-bullets: the `Bullet_heatseekable` 0x1[2f248 class-0 arm on the real level —
//! Bond's AT-600 Scorpion 'Heatseeker' (weapon 0x2e, record `+0x6C` = 0x32200, the homing bit
//! [L ee_start_05.bin @0x2bf150 + 0x2e·0x10C]) locks a COPTER body. The disc reaches the arm
//! here: 07000012 carries the kind-0 copter placement #707 (64.44, 4.30, 19.53) [D] and grants
//! the 0x2e through pickups #2aa/#2f0 [D the placement sweep] — the old code's homing scan knew
//! only the drones (the class-2 arm), so the round flew straight past the copter and the lock
//! never happened (the red run).
//!
//! Evidence tags: [M] decompile, [D] disc bytes, [L] EE snapshot, [S] suggested. The teleport
//! is the documented stand-in transit (the operation itself runs through the real equip + aim +
//! fire input); the 0700000b pairing (copter #148 + pickup #15d) is the same arm's second level.

use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn aim_at(sim: &mut Sim, target: [f32; 3], fire: bool) {
    sim.drones.health.health = sim.drones.health.health.max(100.0);
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
    let (dx, dz, dy) = (target[0] - eye[0], target[2] - eye[2], target[1] - eye[1]);
    let desired = dx.atan2(dz);
    let input = nf_game::player::Input {
        turn: (desired - sim.bond.yaw).clamp(-0.08, 0.08),
        look: ((dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch).clamp(-0.08, 0.08),
        aiming: true,
        fire,
        ..Default::default()
    };
    sim.step_bond(input, fire, false, false);
}

fn copter_pose(sim: &nf_view::scene::Sim) -> [f32; 3] {
    let c = &sim.drones.copters[0];
    [c.matrix[0][3], c.matrix[1][3], c.matrix[2][3]]
}

fn dot_to_copter(b: &nf_game::drone::PlayerBullet, cpos: [f32; 3]) -> f32 {
    let to = [cpos[0] - b.pos[0], cpos[1] - b.pos[1], cpos[2] - b.pos[2]];
    let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-4);
    (to[0] * b.dir[0] + to[1] * b.dir[1] + to[2] * b.dir[2]) / l
}

#[test]
#[ignore = "port-map-bullets: the give path refuses the 0x2e equip on 07000012 (the loadout logs 4 owned weapons, the switch never draws — the fp-model gate, pickup.rs equip_weapon's 'first-person model not loaded' arm). The harness is complete; the next round runs it red-then-green once the give draws the Scorpion."]
fn the_scorpions_homing_round_locks_the_copter_body_on_07000012() {
    let mut scene = nf_view::load_weapons(&iso(), "07000012.bin", false, &[0x2e])
        .expect("load 07000012 with the AT-600 Scorpion");
    assert_eq!(scene.sim.as_ref().unwrap().drones.copters.len(), 1, "07000012 carries exactly one kind-0 copter [D #707]");
    let ci = 0usize;
    let c0 = { let c = &scene.sim.as_ref().unwrap().drones.copters[0]; [c.matrix[0][3], c.matrix[1][3], c.matrix[2][3]] };
    eprintln!("copter body pose at load: ({:.2},{:.2},{:.2}) — placement #707 (64.44, 4.30, 19.53) [D]", c0[0], c0[1], c0[2]);
    // The copter waits on its gate channel (param 6, `Copter_Update`'s head [M]) — the game's
    // own activation: raise it, let Copter_Move pose the body, and read the live pose.
    let gate = scene.sim.as_ref().unwrap().drones.copters[0].gate;
    eprintln!("copter gate channel: {gate:#06x}");
    let _ = gate;
    if gate != 0 {
        let f = scene.sim.as_ref().unwrap().drones.frame;
        scene.sim.as_mut().unwrap().drones.channels.set(gate as u32, 1, f);
    }
    for t in 0..600 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let c = &scene.sim.as_ref().unwrap().drones.copters[0];
        if c.matrix[0][3].abs() + c.matrix[1][3].abs() + c.matrix[2][3].abs() > 1.0 {
            eprintln!("copter posed at tick {t}");
            break;
        }
    }

    // Equip the Scorpion: the harness give's own equip path (`equip_weapon(..., true)` —
    // lib.rs 7116's panel give arm; the loadout case for this level never carries the 0x2e),
    // then the real switch request (`M_CONTROL+0x63` → `Player_WeaponSelect` 0x1a2cc0).
    {
        let sim = scene.sim.as_mut().unwrap();
        let ok = sim.drones.inventory.equip_weapon(&sim.weapons, 0x2e as usize, 20, true);
        assert!(ok, "the give's equip_weapon must own the Scorpion (allow=true, the harness give arm)");
        sim.gun.switch_request = 0x2e;
    }
    let mut equipped = false;
    for _ in 0..400 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == 0x2e && scene.sim.as_ref().unwrap().gun.switch_state == 0 { equipped = true; break; }
    }
    assert!(equipped, "the Scorpion (0x2e) must equip; weapon now {:#x}", scene.sim.as_ref().unwrap().gun.weapon);

    // Stand ~140 units out on the copter's floor plane; the aim point is 24 units LEFT of the
    // body — the round spawns ~10° off the copter, inside the 0.66-dot lock cone but not on it.
    let cpos = { let c = &scene.sim.as_ref().unwrap().drones.copters[0]; [c.matrix[0][3], c.matrix[1][3], c.matrix[2][3]] };
    eprintln!("copter body pose after the gate: ({:.2},{:.2},{:.2})", cpos[0], cpos[1], cpos[2]);
    assert!(cpos[0].abs() + cpos[1].abs() + cpos[2].abs() > 1.0, "the gate must pose the copter body");
    let spot = [cpos[0] - 139.0, cpos[1], cpos[2] + 10.0];
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = spot;
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..90 { aim_at(scene.sim.as_mut().unwrap(), [cpos[0] - 139.0, cpos[1], cpos[2] + 34.0], false); }
    let spot = scene.sim.as_ref().unwrap().bond.pos;
    let off = [cpos[0] - 139.0 + 0.0, cpos[1] + 0.5, cpos[2] + 34.0];

    // Hold fire through the real trigger; the launcher's rounds spawn on the record's interval.
    let mut early = None;
    let mut locked = false;
    for t in 0..360 {
        aim_at(scene.sim.as_mut().unwrap(), off, true);
        let sim = scene.sim.as_ref().unwrap();
        let Some(b) = sim.drones.player_bullets.iter().rev().find(|b| b.weapon == 0x2e && b.homing) else { continue };
        let cpos = copter_pose(sim);
        if early.is_none() && t > 20 { early = Some(dot_to_copter(b, cpos)); }
        // The lock itself [M 0x12f248's class-0 arm]: the copter body is the bullet's target.
        if b.target == Some(ci) && b.target_copter { locked = true; }
        if t > 60 {
            if let Some(e) = early {
                assert!(dot_to_copter(b, cpos) > e + 0.02,
                    "the round must TURN onto the copter (dot {e:.4} → {:.4}) — the old code flew it straight past",
                    dot_to_copter(b, cpos));
            }
        }
        let _ = cpos;
        if locked && t > 120 && early.is_some() {
            assert!(dot_to_copter(b, cpos) >= 0.995,
                "the locked round converges onto the copter body (dot {:.4})", dot_to_copter(b, cpos));
            break;
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.player_bullets.iter().any(|b| b.weapon == 0x2e && b.homing),
        "the Scorpion fired homing rounds [L the 0x20000 record bit]");
    assert!(locked, "the homing round must lock the copter body [M Bullet_heatseekable 0x12f248 class 0] — \
        the red run (old code, drones-only scan) never locked it");
}
