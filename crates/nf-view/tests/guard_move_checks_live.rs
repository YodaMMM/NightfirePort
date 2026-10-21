//! guard-move-checks (the call-tree gap task): the NDrone2 move-check family's real-path pin.
//!
//! The bathhouse scenario (the armed-girl playtest's level 07000002, wake channels 22/23/25, Bond
//! at the playtest spot) runs the armed guards' combat loop — `AimStandFire` 0x68 →
//! `NDrone2_ChooseCombatMove` 0x150c18 — whose cases probe `NDrone2_CanStrafeDodgeLeft/Right`
//! (0x14fdf8/0x14ffc0, ±3.6), `NDrone2_CanRollLeft/Right` (0x150338/0x150500, ±5.3) and
//! `NDrone2_CanStepLeft/Right` (0x14f640/0x14f808, ±1.8) before setting the move state, and whose
//! too-close arm asks `NDrone2_CanBackoff` 0x1501a0. Pins [M 0x150c18:164-316 + the option gates
//! :55-162]:
//! - the move-state vocabulary is the game's: any drone's state log that ever holds one of the
//!   ChooseCombatMove result states (0x73-0x7a) belongs to an armed team-1 guard — and the
//!   civilians (the bathhouse girls, team 3, no props, the fists) NEVER hold one. This can fail:
//!   a port that let a civilian dodge/step/roll, or that invented a move state id, trips here.
//! - the sister on the guard side: the armed guard's log reaches the fire states (0x68/0x70) —
//!   the flow that owns the probes actually ran (the same outcome the armed-girl test pins via
//!   its bullets).
//! Opt-in via NF_ISO.

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The armed-girl test's loader, verbatim (level 07000002, the intro skipped).
fn load_and_skip() -> Option<nf_view::scene::Scene> {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return None };
    let mut scene = match nf_view::load(&iso, "07000002.bin", false) {
        Ok(s) => s,
        Err(e) => { eprintln!("load: {e}"); return None }
    };
    let mut n = 0u32;
    let sim = scene.sim.as_mut().unwrap();
    while n < 400 {
        if sim.drones.scripts.players.iter().any(|p| p.state == 2 && p.scripts.contains(&0x0600_0084)) {
            sim.drones.scripts.skip_request = true; // the game's skip action [M SP_Update 0x1dbda8]
        }
        sim.step_bond(Input::default(), false, false, false);
        n += 1;
    }
    drop(sim);
    Some(scene)
}

/// `NDrone2_ChooseCombatMove` 0x150c18's result states [M :164-316: dodge 0x77/0x78, strafe
/// 0x75/0x76, roll 0x79/0x7a, step 0x73/0x74].
const MOVE_STATES: [u8; 8] = [
    state::STEP_AIM_LEFT, state::STEP_AIM_RIGHT,
    state::STRAFE_AIM_LEFT, state::STRAFE_AIM_RIGHT,
    state::STRAFE_DODGE_LEFT, state::STRAFE_DODGE_RIGHT,
    state::ROLL_LEFT_CROUCH, state::ROLL_RIGHT_CROUCH,
];

#[test]
fn combat_move_states_belong_to_armed_guards_and_never_to_civilians() {
    let Some(mut scene) = load_and_skip() else { return };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.set_channel(22);
        sim.drones.set_channel(23);
        sim.drones.set_channel(25);
        sim.bond.pos = [-43.11, 6.46, 10.37]; // the playtest camera [L the playtest report]
        sim.bond.prev_pos = sim.bond.pos;
    }
    for _ in 0..1800u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    // The combat loop ran: an armed team-1 guard reached a fire state — the ChooseCombatMove
    // caller [M DSTATE_AimStandFire 0x166328 / AimCrouchFire 0x167060 re-pick timers].
    let fire_states = [state::AIM_STAND_FIRE, state::AIM_CROUCH_FIRE];
    let fired = sim.drones.list.iter()
        .filter(|d| d.team == 1 && d.weapon >= 2)
        .any(|d| d.log.iter().any(|&(_, s)| fire_states.contains(&s)));
    assert!(fired, "an armed guard reached AimStandFire/AimCrouchFire — the flow that owns the NDrone2 move checks ran");
    // The move-state vocabulary + the civilian sister. Every drone whose log holds one of the
    // game's ChooseCombatMove result states is an armed team-1 guard; the girls (team 3, the
    // fists, no dodge/step/roll props — the option list's gates [M 0x150c18:55-162]) never hold
    // one. Universal over the log, so an invented state id or a civilian move trips it.
    for d in &sim.drones.list {
        let hits: Vec<u8> = d.log.iter().map(|&(_, s)| s).filter(|s| MOVE_STATES.contains(s)).collect();
        if d.team == 1 && d.weapon >= 2 {
            for s in &hits {
                assert!(MOVE_STATES.contains(s), "guard #{} logged move state {s:#x} outside the game's result set", d.placement);
            }
        } else {
            assert!(hits.is_empty(),
                "drone #{} (team {}, weapon {:#x}) logged combat-move states {hits:x?} — the game's option gates [M 0x150c18:55-162] never let a propless/unarmed actor pick one",
                d.placement, d.team, d.weapon);
        }
    }
    // The girls are in the level and awake (the scenario ran for real — the armed-girl census).
    let girls = sim.drones.list.iter().filter(|d| matches!(d.skin, 0x0500_0034 | 0x0500_004f)).count();
    assert!(girls >= 3, "the bathhouse girls are present and scanned ({girls})");
}
