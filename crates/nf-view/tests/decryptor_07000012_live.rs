//! decryptor-12: the Q-Decryptor's full chain on the real path, on 07000012 (HT Tower2 B — the
//! security centre). The level carries two kind-0 (decryptor) locks, placements #656/#694
//! [D the disc census: param 0 = 0, code index 5, out channel 10, operative tag 1], so the test
//! drives: select the Decryptor (0x56 granted like the game's own loadouts grant gadgets), aim at
//! the lock through the real aim loop, HOLD FIRE through the use gate
//! [M Player_WeaponFiring 0x1a5460:551-578] — and the lock decodes
//! [M Lock_Update 0x1ae3d8 case 5] and writes its out switch channel 10
//! [M Lock_Unlock 0x1b0650 → case 1 :193-196].
//!
//! Sisters (each can fail): the gate REFUSES a fire that is not aimed at a decryptor-kind lock
//! (no scan, no channel — the pre-fix symptom: the decryptor did nothing at all); a mid-scan
//! release decays the scan −4/tick to a full reset [M :327-344]; a laser-kind (record 0 == 1)
//! lock ignores the decryptor entirely [M the gate's `== 0 || == 2`, :558].
//!
//! Evidence tags: [M] decompile, [D] disc bytes, [L] EE snapshot. The teleport to the lock is the
//! walkreal's documented stand-in (the operation itself runs through the real aim + fire input).

use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> (nf_view::scene::Scene, usize, [f32; 3]) {
    // Difficulty 3: the operative tag (param 6 = 1) must NOT auto-unlock these locks
    // [M Lock_Create 0x1ae1f8:67-69 — the unlock is Operative (1) only].
    let mut scene = nf_view::load_with_carry_weapons_difficulty(&iso(), "07000012.bin", false, None, &[0x56], 3)
        .expect("load 07000012 with the Decryptor");
    let li = scene.sim.as_ref().unwrap().drones.locks.iter().position(|l| l.kind == 0 && l.out == 10)
        .expect("07000012 carries the kind-0 lock with out channel 10 [D census]");
    let pos = scene.sim.as_ref().unwrap().drones.locks[li].pos;
    // Select the decryptor through the real switch request (the game's BL+0x63 path).
    scene.sim.as_mut().unwrap().gun.switch_request = 0x56;
    (scene, li, pos)
}

/// Aim the crosshair at the target (the walkreal aim loop) and fire held or not. The caller has
/// placed Bond once; the helper never teleports — the aim chases the stable grounded eye.
fn aim_and_fire(sim: &mut Sim, target: [f32; 3], _spot: [f32; 3], fire: bool) -> f32 {
    // The walker's allowance: this pins the decryptor, not the firefight — the guards nearby may
    // engage Bond (Attack's re-entry fallback, 913, has them fire from AimStand), so keep him up.
    sim.drones.health.health = sim.drones.health.health.max(100.0);
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
    let (dx, dz, dy) = (target[0] - eye[0], target[2] - eye[2], target[1] - eye[1]);
    let desired = dx.atan2(dz);
    let err = desired - sim.bond.yaw;
    let want_pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
    let input = nf_game::player::Input {
        turn: err.clamp(-0.08, 0.08),
        look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08),
        aiming: true,
        fire,
        ..Default::default()
    };
    sim.step_bond(input, fire, false, false);
    err
}

#[test]
fn decryptor_opens_the_kind0_lock_through_the_real_path_and_the_sisters_hold() {
    let (mut scene, li, pos) = load();
    let sim = scene.sim.as_mut().unwrap();
    // Settle the switch: the holster + draw at the game's own rates (gadget_select's wait).
    let mut equipped = false;
    for _ in 0..400 {
        sim.step_bond(Default::default(), false, false, false);
        if sim.gun.weapon == 0x56 && sim.gun.switch_state == 0 { equipped = true; break; }
    }
    assert!(equipped, "the Decryptor (0x56) must equip; weapon now {:#x}", sim.gun.weapon);
    assert!(!sim.drones.locks[li].unlocked, "the lock starts locked on difficulty 3 [M 0x1ae1f8:67-69]");
    assert!(!sim.drones.channels.on(10), "the out channel starts off");

    // The lock object settles after load (its world pos drifts); place Bond once beside it and
    // let him ground — the aim loop then chases a stable eye from a stable floor spot.
    let lp0 = sim.drones.locks[li].pos;
    let base_spot = [lp0[0] + 1.2, lp0[1], lp0[2]];
    sim.bond.pos = base_spot;
    for _ in 0..90 { aim_and_fire(sim, lp0, base_spot, false); }
    let spot = sim.bond.pos;
    let lp = sim.drones.locks[li].pos;
    let root_dist = ((lp[0] - spot[0]).powi(2) + (lp[1] - spot[1]).powi(2) + (lp[2] - spot[2]).powi(2)).sqrt();
    assert!(root_dist <= 2.0, "Bond stands inside the 0x56's 2.0 range (dist {root_dist:.2})");

    // --- Sister 1: the gate REFUSES an off-target fire (the pre-fix symptom: nothing happens).
    // Stand beside the lock but aim at a wall point 30 units the OTHER way; hold fire 120 ticks.
    let away = [pos[0] - 30.0, pos[1] + 1.0, pos[2]];
    for _ in 0..120 { aim_and_fire(sim, away, spot, true); }
    assert!(!sim.drones.channels.on(10), "an off-target fire never opens the lock");
    assert!(!sim.drones.locks[li].scanning && sim.drones.locks[li].scan_solved == 0,
        "an off-target fire never starts the scan");
    assert_eq!(sim.decryptor_target, None, "the gate pick stays empty off-target");

    // --- The chain: aim at the lock, hold fire. The 0x56 needs 425 contact ticks
    // (161+116+76+36+36 = 425: Lock_Update unlocks only after the 5th crossing — the last digit solves twice [M 0x1ae3d8:183,348-350,364-372]; the aim settle eats some) — 900 ticks is headroom.
    let mut unlocked_tick = None;
    for t in 0..900 {
        // Aim at the lock's CURRENT pos (it settles early); the pin is x/z at Bond's floor y.
        let lpos = sim.drones.locks[li].pos;
        let err = aim_and_fire(sim, lpos, spot, true);
        if t == 60 {
            assert!(sim.decryptor_target == Some(li) || sim.drones.locks[li].scanning,
                "the gate must pick the aimed lock (target {:?}, scanning {})",
                sim.decryptor_target, sim.drones.locks[li].scanning);
        }
        if sim.drones.locks[li].unlocked { unlocked_tick = Some(t); break; }
    }
    let ut = unlocked_tick.expect("the held decryptor scan must unlock the lock inside 900 ticks");
    assert!(sim.drones.channels.on(10), "the unlock writes the out switch channel 10 [M :193-196]");
    assert!(ut < 850, "the 0x56's scan lands inside its 389-tick + settle budget (took {ut})");

    // --- Sister 2: a mid-scan release decays −4/tick to a full reset [M 0x1ae3d8:327-344].
    // Use the SECOND kind-0 lock (#694, same out channel — still locked after #656's unlock).
    let li2 = sim.drones.locks.iter().position(|l| l.kind == 0 && l.out == 10 && !l.unlocked)
        .expect("the second kind-0 lock (#694) is still locked");
    // Settle beside it first (the same place-once as the first spot) — on ITS side of the door:
    // #656 (x 7.01) and #694 (x 6.74) are the two faces of one door, and the gate's target is the
    // nearest lock the ray meets [M 0x1a5460:557-566 on +0x880], so from #656's side the open #656
    // blocks the aim at #694 (the far keypad is scanned from the far side, −x).
    let lp2_0 = sim.drones.locks[li2].pos;
    let base2 = [lp2_0[0] - 1.2, lp2_0[1], lp2_0[2]];
    sim.bond.pos = base2;
    for _ in 0..90 { aim_and_fire(sim, lp2_0, base2, false); }
    let spot2 = sim.bond.pos;
    // Aim at THIS lock from its own side (the pick must name #694).
    for _ in 0..100 { aim_and_fire(sim, sim.drones.locks[li2].pos, spot2, true); }
    let (prog, solved) = (sim.drones.locks[li2].scan_progress, sim.drones.locks[li2].scan_solved);
    assert!(sim.drones.locks[li2].scanning && (prog > 0 || solved > 0),
        "the second lock scans while aimed at (prog {prog}, solved {solved})");
    // Release 40 ticks: 40·4 = 160 progress lost; from ≤160 the scan resets fully.
    for _ in 0..40 { aim_and_fire(sim, sim.drones.locks[li2].pos, spot2, false); }
    assert!(!sim.drones.locks[li2].scanning && sim.drones.locks[li2].scan_solved == 0
        && sim.drones.locks[li2].scan_progress == 0,
        "40 released ticks reset the scan completely (prog {}, solved {})",
        sim.drones.locks[li2].scan_progress, sim.drones.locks[li2].scan_solved);

    // --- Sister 4: the contact view lags the shot TWO ticks [M the frame order — the update pass
    // (the fire, the bolt's birth) 0x1336f8:54-69 runs before Collide_Update 0x1ea398 (:238)
    // writes the contact lists, and the fresh bolt's pick segment exists only from its first
    // Bullet_update 0x12e7b0:98-110, so a shot at N lands in the (N+1) collide and Lock_Update
    // reads it at N+2]. After the last shot the scan therefore RISES for two released ticks (the
    // two residual contacts) and only decays on the third [M 0x1ae3d8:327-344]. The old stand-in
    // banked the scan on the shot tick — progress decayed one tick early here.
    for _ in 0..40 { aim_and_fire(sim, sim.drones.locks[li2].pos, spot2, true); }
    let p0 = sim.drones.locks[li2].scan_progress;
    assert!(p0 > 0 && p0 < 150, "the re-scan is mid-flight below the digit-0 threshold (prog {p0})");
    for _ in 0..2 { aim_and_fire(sim, sim.drones.locks[li2].pos, spot2, false); }
    assert_eq!(sim.drones.locks[li2].scan_progress, p0 + 2,
        "the two residual contacts keep the scan rising two released ticks (the collide-after-update order)");
    aim_and_fire(sim, sim.drones.locks[li2].pos, spot2, false);
    assert_eq!(sim.drones.locks[li2].scan_progress, p0 - 2,
        "the first −4 decay lands on the third released tick [M 0x1ae3d8:327-344]");

    // --- Sister 3: the laser-kind (record 0 == 1) lock ignores the decryptor [M :562].
    // A kind-1 lock beside Bond with its own out channel (200 — nothing else writes it here).
    let mut laser_lock = sim.drones.locks[li].clone();
    laser_lock.kind = 1;
    laser_lock.out = 200;
    laser_lock.unlocked = false;
    laser_lock.scan_progress = 0;
    laser_lock.scan_solved = 0;
    laser_lock.scanning = false;
    laser_lock.pos = [spot2[0] + 0.0, spot2[1], spot2[2] - 0.8]; // 0.8 from Bond, off the crosshair
    sim.drones.locks.push(laser_lock);
    let li3 = sim.drones.locks.len() - 1;
    // Fire 150 ticks at the wall point past it: even a stray sphere touch must not scan kind 1.
    for _ in 0..150 { aim_and_fire(sim, away, spot2, true); }    assert!(!sim.drones.locks[li3].scanning && !sim.drones.locks[li3].unlocked,
        "the laser-kind lock never scans under the decryptor");
    assert!(!sim.drones.channels.on(200), "no channel from a non-target");
    // (The stray beam may legitimately re-pick the kind-0 #694 sitting on the ray — the game's
    // own gate does that. Release: the scan decays away before the getter check.)
    for _ in 0..60 { aim_and_fire(sim, away, spot2, false); }
    // And the scan getter mirrors the state machine for the draw code: nothing scans now
    // (the first lock unlocked, the second decayed, the laser-kind can never scan).
    assert!(sim.decryptor_scan().is_none(), "no scan runs after the sisters (the draw getter)");
}
