//! corpse-fire: a guard the game's own cleanup has removed never fires again.
//!
//! Game law [M]:
//! - `NDrone2_ControlSTANDARD` 0x1490f0:136-140 returns at `(+0x4F8 & 0x100) == 0` —
//!   `NDrone2_DoTracking(0)` and no control tail — so a guard whose AI-active bit is off
//!   never reaches `DroneWeap_HandleFiring` 0x17a180 (called unconditionally at
//!   0x1490f0:227 for guards that pass the gate) and never spawns a round through
//!   `DroneWeap_DoFiring` 0x17a080 → `DroneWeap_Ready2Fire` 0x178c48 →
//!   `DroneWeap_FireWeapon` 0x178f68.
//! - The port's `enabled` is that bit [M crates/nf-game/src/drone.rs:2601], and the port's
//!   own removal folds clear it together with `present`: the corpse fade
//!   (DroneFunc_DeadDrone 0x13a5a8's port fold, drone.rs:5124/:5145) and the spawner
//!   cleanup (`DroneSpawner_Control` 0x137e68:235-288's port fold, drone.rs:14833 —
//!   `DroneSpawner_SpawnDrone` 0x137cb0-built guards are removed with `obj+0xFE |= 1`).
//!   A removed guard therefore has present=false, enabled=false and — when the cleanup
//!   caught him alive — corpse=false.
//! - The bug this pins: the port's weapon pass ran `handle_firing` for every `!corpse`
//!   drone, so a removed-but-alive guard kept his burst alive in place: the level-0b-roof-
//!   sprint-5 merge check measured Bond dying to shots from removed stairwell guards whose
//!   health read −72/−75 at the hit (weapon 0x19, d 4.1, ~270 ticks of fire from a slot the
//!   spawner had cleared) [L the runs on 0a9d39435 + that branch's world.rs]. The fix gates
//!   the weapon pass and the aim block on the game's own bit.
//!
//! Real input only: Bond walks on the real stick until the guard opens fire through the
//! real AI (`handle_firing`'s own shot latch), the removal is the port's documented fold of
//! the game's cleanup state, and every tick after it runs the real `Sim::tick`.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn a_removed_guard_never_shots_again() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000011.bin", false).expect("load 07000011"));
    w.wait_enabled(6000, "the 11 intro");
    w.wait(120);

    // The nearest started, armed, live hostile — the stairwell-mouth guards stand in the
    // open office floor's sight lines.
    let pick = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.team == 1 && d.started && d.present && d.enabled && !d.corpse && d.health > 0.0
                && (2..=0x46).contains(&d.weapon)

        }).min_by(|&i, &j| walker::dxz(sim.drones.list[i].pos, sim.bond.pos).total_cmp(&walker::dxz(sim.drones.list[j].pos, sim.bond.pos)))
    };
    let Some(g) = pick(&w) else { panic!("no armed guard alive after the 11 intro") };

    // Walk Bond at him until he opens fire through the real AI (the shots latch is
    // `handle_firing`'s own; the walk's return fire is on, so the fight is live). The burst
    // is short, so the final approach is a per-tick steer: the removal must catch him with
    // the request byte live (`firing`, the burst counter and the next-shot deadline standing).
    let mut caught = false;
    'approach: for _ in 0..6000u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[g];
            if !d.present || !d.enabled || d.corpse { break; }
            if d.shots.len() > 0 && d.firing { caught = true; break 'approach; }
        }
        let (dx, dz) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.list[g].pos[0] - sim.bond.pos[0], sim.drones.list[g].pos[2] - sim.bond.pos[2])
        };
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let dist = (dx * dx + dz * dz).sqrt();
        let input = nf_game::player::Input {
            turn: err.clamp(-0.07, 0.07),
            forward: if dist > 2.0 { 1.0 } else { 0.0 },
            ..Default::default()
        };
        w.tick(input, false);
    }
    assert!(caught, "the guard {g} never opened fire with the request byte live on the walked approach — the setup needs a fight");

    // The game's cleanup state (the port's own fold, see the header): removed with
    // `obj+0xFE |= 1` — present and the AI bit cleared, everything else untouched. A guard
    // caught mid-burst keeps `firing`, the burst counter and his next shot deadline.
    let before = {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[g];
        (d.shots.len(), sim.drones.bullets.len())
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let d = &mut sim.drones.list[g];
        assert!(!d.corpse, "the setup caught the guard already dead — the freeze must catch him alive");
        // `Drone_EnableAll(0)`'s state: the AI-active bit cleared, the object standing. The
        // game's ControlSTANDARD returns at the bit [M 0x1490f0:136-140] — the guard neither
        // aims nor fires from here on. The port's script freeze folds to the same fields.
        d.enabled = false;
    }
    // 4 seconds of the real tick — longer than a burst refire interval.
    for _ in 0..240u32 { w.tick(nf_game::player::Input::default(), false); }
    let (after_shots, after_bullets) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[g];
        (d.shots.len(), sim.drones.bullets.iter().filter(|b| b.owner == g).count())
    };
    assert_eq!(after_shots, before.0,
        "the disabled guard {g} kept firing: shots {} → {} (the AI-bit gate [M 0x1490f0:136-140])",
        before.0, after_shots);
    assert_eq!(after_bullets, 0,
        "the disabled guard {g}'s rounds are in flight after the freeze");
    // The aiming half of the same law: the head tracking eased off (DoTracking(0)).
    let (tracking, firing) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[g];
        (d.tracking, d.firing)
    };
    assert!(!tracking, "the disabled guard {g} still head-tracks (DoTracking(0) [M 0x1490f0:136-140])");
    let _ = firing; // the request byte may stand; the gate above is what stops the rounds
}
