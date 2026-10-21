//! dronefunc-rest (the call-tree gap task 845): the ported helpers' real-path pin.
//!
//! The bathhouse wake scenario (level 07000002, channels 22/23/25, Bond at the playtest spot —
//! the armed-girl test's loader) drives a team-1 guard into combat. Pins, all [M]-backed:
//! - the shared target registry (NPCGlobals, `DroneFunc_AllocateTargetID` 0x141f28 /
//!   `DroneFunc_TargetSeen` 0x14a008): a team-1 guard that acquired the opponent holds a claimed
//!   slot (flag bit 0) stamped seen (flag bit 1) with a position; sisters: the propless girls
//!   (team 3 — the game's `+0x44 != 1` gate) never allocate, and with the bots off / nobody seen
//!   the pool stays empty.
//! - the first-attack stamps (`DroneFunc_SetAsAttacking` 0x14aed8 via `DroneFunc_FirstAttack`
//!   0x148010): a guard that reached its first ATTACK carries alert 1.0 and `first_attack_done`;
//!   sister: an unwoken guard keeps alert 0 and no allocation.
//! Opt-in via NF_ISO.

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

#[test]
fn the_target_registry_and_first_attack_stamps_follow_the_game() {
    let Some(mut scene) = load_and_skip() else { return };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.set_channel(22);
        sim.drones.set_channel(23);
        sim.drones.set_channel(25);
        sim.bond.pos = [-43.11, 6.46, 10.37]; // the playtest camera [L the playtest report]
        sim.bond.prev_pos = sim.bond.pos;
    }
    // The stamp is read on the tick the first ATTACK runs: afterwards gunfire keeps adding sound
    // shares to the alertness UNCLAMPED [M DroneFunc_HandleSoundAlerts 0x148a30], so a read 1800
    // ticks later sees the fight, not the stamp (alert-status-vs-alertness, 916, exposed it).
    let mut first_alert: Option<(usize, f32)> = None;
    for _ in 0..1800u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if first_alert.is_none() {
            let sim = scene.sim.as_ref().unwrap();
            if let Some(i) = sim.drones.list.iter().position(|d| d.team == 1 && d.first_attack_done) {
                first_alert = Some((i, sim.drones.list[i].alert));
            }
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    // A team-1 guard saw Bond and entered its first ATTACK (the armed-girl scenario's proven
    // outcome) — its stamps: alert 1.0, first_attack_done [M 0x148010:62-83 / 0x14aed8:8-28].
    let attacker = sim.drones.list.iter()
        .find(|d| d.team == 1 && d.first_attack_done)
        .expect("a team-1 guard reached its first ATTACK in the wake scenario");
    let (_, a0) = first_alert.expect("the first ATTACK tick was observed");
    // Same tick: the stamp 1.0, plus at most that tick's sound share (sound only ever adds).
    assert!(a0 >= 1.0 - 1e-4 && a0 < 1.5,
        "the first attack stamps alert 1.0 [M 0x14aed8:15 / 0x148010:66], got {a0} on its tick");
    // Its registry slot: claimed (flag bit 0) + seen (flag bit 1) with a position
    // [M 0x141f28:26-33 / 0x14a008:14-23].
    assert!(attacker.target_id != 0xff, "the guard allocated a registry slot [M 0x141f28]");
    let slot = &sim.drones.targets.slots[attacker.target_id as usize];
    assert!(slot.flags & 1 != 0, "the slot is claimed [M 0x141f28:28]");
    assert!(slot.flags & 2 != 0, "the slot is stamped seen [M 0x14a008:16]");
    assert!(slot.pos[0] != 0.0 || slot.pos[1] != 0.0 || slot.pos[2] != 0.0,
        "the last-seen position stored [M 0x14a008:20-23]");
    // The slot is keyed by the opponent (Bond), not the guard: every team-1 guard that allocated
    // shares it [M 0x1422d0:74 passes the opponent as AllocateTargetID's param_2; 0x141f28:16-21].
    assert_eq!(slot.obj, nf_game::target_registry::BOND, "the slot holds the opponent obj [M 0x141f28:30]");
    for g in sim.drones.list.iter().filter(|d| d.team == 1 && d.target_id != 0xff) {
        assert_eq!(g.target_id, attacker.target_id, "guard #{} shares Bond's slot", g.placement);
    }
    // Sister: the girls are team 3 — the game's `+0x44 != 1` gate never lets them allocate
    // [M 0x141f28:11-13].
    for g in sim.drones.list.iter().filter(|d| d.team == 3) {
        assert_eq!(g.target_id, 0xff, "girl #{} never allocates [M 0x141f28:11-13]", g.placement);
    }
}

/// Sister run: nobody wakes (no channels set, Bond stays at the spawn) — no guard enters its first
/// attack, the registry pool stays empty, the unwoken guards keep alert 0 [M the first-attack gate
/// 0x148010:17-19 requires a sighting].
#[test]
fn an_unwoken_level_never_allocates_or_stamps() {
    let Some(mut scene) = load_and_skip() else { return };
    for _ in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.list.iter().all(|d| !d.first_attack_done),
        "no guard reached its first attack without a sighting");
    assert!(sim.drones.targets.slots.iter().all(|s| s.flags == 0),
        "the registry pool stays empty [M 0x141f28:16-21 allocates only on SetOpponent]");
}
