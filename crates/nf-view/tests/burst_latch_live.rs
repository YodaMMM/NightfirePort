//! burst-latch: the fire request `Drone+0x3B` on a state change, and the burst-finished latch
//! `Drone+0x3D`.
//!
//! Game law [M]:
//! - `Drone_SM_RouteMsgDCV` 0x171df8:125-173, on every state change and ahead of the old
//!   state's EXIT: a change INTO Death_Anim 0x44 / SpecialDeath 0x46 / Dead 0x47 /
//!   BulletImpact 0x55 keeps the request and sets `+0x3D = 1` (:155-166); any other change
//!   clears `+0x3B` (:172). (The mode-0x1E arm reads `BOTSTATE_getStateType`, which is 0 for
//!   every state below 0xC3 [M 0x1291f0], so it takes the same path for these states.)
//! - `DroneWeap_HandleFiring` 0x17a180:34-63: with `+0x3D` clear each pass clears `+0x3C`
//!   then runs `DoFiring`; with it set, a pass that finds `+0x3C` already latched clears
//!   `+0x3B` and returns (:37-58), otherwise it fires on and `DoFiring`'s 0 latches `+0x3C`
//!   (:63). `DroneWeap_Fire` 0x17a8c0:15-17 clears both bytes again.
//! - Death_Anim's ENTER does not touch `+0x3B` [M 0x1637f8 case 1]; its UPDATE drops the gun
//!   only once `+0x3B` is clear (or the anim set is 4 / 9) [M 0x1637f8 case 3]. A dying guard
//!   still runs the weapon pass: `NDrone2_ControlSTANDARD` gates it on the AI bit alone
//!   [M 0x1490f0:140-143, :231].
//!
//! So a guard killed with his fire request on finishes the burst first (in practice: `DoFiring`
//! returns 0 once the unseen counter `+0x274` reaches 0x10 [M 0x17a080:24-25] — the dead
//! drone counts up every sight pass [M 0x176a28:191-201]), loses the request on the next pass,
//! and drops his gun on the UPDATE after that. The old port cleared the request on the death
//! ENTER and dropped the gun on the first UPDATE; it also carried a standing request through
//! every other state change.
//!
//! Setup: the corpse_fire_live approach on 07000011 / 07000005 (Bond walks on the real stick until the
//! nearest armed guard opens fire through the real AI). The state change itself is the port's
//! real dispatch (`next_state` + `send`), standing in for the kill or the AI's own choice;
//! every tick after it is the real `Sim::tick`.
//! Opt-in: set `NF_ISO` to the disc image.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::drone::state;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Load 07000011, walk Bond at the nearest armed guard until his fire request is live
/// mid-burst; returns the walker and the guard index.
fn firing_guard(level: &str, gun_set_only: bool) -> (Walker, usize) {
    let mut w = Walker::new(nf_view::load(&iso(), level, false).expect("load"));
    w.wait_enabled(6000, "the intro");
    w.wait(120);
    let g = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.team == 1 && d.started && d.present && d.enabled && !d.corpse && d.health > 0.0
                && (2..=0x46).contains(&d.weapon)
                && !(gun_set_only && matches!(d.anim_set, 4 | 9))
        }).min_by(|&i, &j| walker::dxz(sim.drones.list[i].pos, sim.bond.pos)
            .total_cmp(&walker::dxz(sim.drones.list[j].pos, sim.bond.pos)))
            .expect("an armed guard (outside anim sets 4/9 when asked) alive after the intro")
    };
    if gun_set_only {
        // Bond placed 4 u west of the guard (the drone_weapon_drop_live placement): the walked
        // approach on 05 meets walls, the placed one meets the guard's sight.
        let gp = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        let mut dev = nf_view::dev::Dev { scenario: None, ..Default::default() };
        dev.teleport(&mut w.scene, [gp[0] - 4.0, gp[1], gp[2]], 0.0);
    }
    for _ in 0..6000u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[g];
            assert!(d.present && d.enabled && !d.corpse, "guard {g} left the fight before firing");
            if !d.shots.is_empty() && d.firing && !d.burst_done && d.unseen_ticks == 0 { return (w, g); }
        }
        let (dx, dz) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.list[g].pos[0] - sim.bond.pos[0], sim.drones.list[g].pos[2] - sim.bond.pos[2])
        };
        let err = walker::wrap_pi(dx.atan2(dz) - w.scene.sim.as_ref().unwrap().bond.yaw);
        let dist = (dx * dx + dz * dz).sqrt();
        let input = nf_game::player::Input {
            turn: err.clamp(-0.07, 0.07),
            forward: if dist > 2.0 { 1.0 } else { 0.0 },
            ..Default::default()
        };
        w.tick(input, false);
    }
    panic!("guard {g} never opened fire on the walked approach");
}

/// The real dispatch: queue `to` and deliver msg 0 (`Drone_SM_RouteMsgDCV`'s change loop).
fn change_state(w: &mut Walker, g: usize, to: u8) {
    let sim = w.scene.sim.as_mut().unwrap();
    let drones = &mut sim.drones;
    let d = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: drones.difficulty,
        ai_points: &sim.ai_points,
    };
    let dr = &mut drones.list[g];
    dr.next_state = Some(to);
    dr.send(&d, &mut drones.rng, drones.frame, 0);
    assert_eq!(dr.state, to, "the change into {to:#x} did not take");
}

/// [M 0x171df8:172]: a change into a state outside the four latch states drops the request.
/// Combat 0x58's ENTER writes no `+0x3B` [M 0x1731b8 case 1], so only the router can clear it.
/// FAILS on the old port: the request rode through the change.
#[test]
fn a_state_change_drops_the_fire_request() {
    let (mut w, g) = firing_guard("07000011.bin", false);
    change_state(&mut w, g, state::COMBAT);
    let d = &w.scene.sim.as_ref().unwrap().drones.list[g];
    assert!(!d.firing, "guard {g} still holds the fire request after the change into Combat 0x58 [M 0x171df8:172]");
}

/// [M 0x171df8:155-166, 0x17a180:34-63, 0x1637f8 cases 1/3]: the guard killed mid-burst keeps
/// the request, runs the burst out, loses the request one pass after `+0x3C` latches, and drops
/// the gun only then. FAILS on the old port: the death ENTER cleared the request and the gun
/// dropped on the first UPDATE.
#[test]
fn the_dying_guard_keeps_the_gun_until_the_burst_latch_drops_the_request() {
    // Anim sets 4 / 9 drop the gun on the first UPDATE regardless of the request
    // [M 0x1637f8 case 3 `+0xDA == 4 || == 9`]; the latch shows on the other sets.
    let (mut w, g) = firing_guard("07000005.bin", true);
    let weapon = w.scene.sim.as_ref().unwrap().drones.list[g].weapon;
    change_state(&mut w, g, state::DEATH_ANIM);
    {
        let d = &w.scene.sim.as_ref().unwrap().drones.list[g];
        assert!(d.firing, "the change into Death_Anim 0x44 cleared the fire request [M 0x171df8:155-166, 0x1637f8 case 1]");
        assert!(d.burst_latch, "the change into Death_Anim 0x44 did not arm the burst latch +0x3D [M 0x171df8:160]");
        assert_eq!(d.weapon, weapon, "the gun left the hand on the death ENTER");
    }
    let (mut done_at, mut clear_at, mut drop_at) = (None, None, None);
    for t in 1..=240u32 {
        let prev_done = w.scene.sim.as_ref().unwrap().drones.list[g].burst_done;
        w.tick(nf_game::player::Input::default(), false);
        let d = &w.scene.sim.as_ref().unwrap().drones.list[g];
        if d.burst_done && done_at.is_none() { done_at = Some((t, d.unseen_ticks, d.burst)); }
        if !d.firing && clear_at.is_none() {
            assert!(prev_done, "tick {t}: the request cleared before the burst latched +0x3C [M 0x17a180:37-38]");
            clear_at = Some(t);
        }
        if d.weapon == 0 && drop_at.is_none() { drop_at = Some((t, d.state)); }
        if drop_at.is_some() { break; }
    }
    let set = w.scene.sim.as_ref().unwrap().drones.list[g].anim_set;
    eprintln!("[burst-latch] guard {g} weapon {weapon:#x} anim set {set}: burst done {done_at:?}, request clear {clear_at:?}, gun drop {drop_at:?}");
    let (td, _, _) = done_at.expect("the dying guard's burst never latched +0x3C");
    let tc = clear_at.expect("the dying guard's request never cleared");
    let (tg, st) = drop_at.expect("the dying guard never dropped the gun");
    assert_eq!(tc, td + 1, "the request clears on the pass after +0x3C latches [M 0x17a180:37-38]");
    assert!(tg >= tc && tg <= tc + 1, "the gun drops on the UPDATE once the request is clear [M 0x1637f8 case 3] (clear {tc}, drop {tg})");
    assert!(tg > 1, "the gun dropped on the first death UPDATE");
    assert!(matches!(st, state::DEATH_ANIM | state::DEAD), "the drop came in state {st:#x}");
}
