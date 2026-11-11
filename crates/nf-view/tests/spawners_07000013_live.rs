//! The 07000013 lobby reinforcements, end to end on the real level.
//!
//! The chain [D placements]: guards #317/#318 carry death channel p12 = 0x2A, #319/#320 carry
//! 0x29; spawner #327 (group 1, mode 3, max 4, waves 0 = 32000, start ch 0x2A, min dist 14) and
//! #323 (group 2, mode 3, max 2, waves 3, start ch 0x29, min dist 10) wait on them. The first
//! guard down brings the lift-lobby squad in.
//!
//! The spawner rules pinned here [M `DroneSpawner_Control` 0x137e68]:
//! - mode 3 opens a WAVE, not a guard: the +0x1C latch set while nobody is alive stays up until
//!   the rollover (:145-172), so all `max` guards arrive one a tick;
//! - a guard leaves the wave when it falls, not when its corpse fades: the slot scan clears on
//!   health <= 0 or `+0x4F8 & 0x600` (:95-124);
//! - every spawn is a new object from the saved template (0x137cb0 `NDrone2_CreateObj`): the next
//!   wave stands at the template spots, alive.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Bond in the north lobby, facing the entrance: past both min-dist screens and with the lift
/// lobby behind him (the reach-only teleport; nothing else below writes a position).
fn north_lobby(w: &mut Walker) {
    let sim = w.scene.sim.as_mut().unwrap();
    sim.bond.pos = [0.0, -369.1, 44.5];
    sim.bond.yaw = 0.0;
    sim.bond.pitch = 0.0;
    sim.bond.vel = [0.0; 3];
}

/// A guard falls: the death state runs its own channel write and body.
fn down(dr: &mut nf_game::drone::Drone) {
    dr.health = 0.0;
    dr.set_state(nf_game::drone::state::DEATH_ANIM);
}

/// The game's lethal hit is a routed message whose `SetState(0x44)` is applied inside that same
/// route [M 0x171df8: the pending change runs EXIT → ENTER before the route returns], so no later
/// UPDATE can replace it. The fixture's `set_state` only queues the change, and a combat UPDATE
/// in the same tick (`SetState` is last-write-wins [M 0x1758c0]) can overwrite it: re-issue the
/// fall until the death ENTER has run (the corpse flag [M 0x1637f8 case 1]).
fn keep_down(w: &mut Walker, i: usize) {
    let dr = &mut w.scene.sim.as_mut().unwrap().drones.list[i];
    if !dr.corpse { down(dr); }
}

fn group1(w: &Walker) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.group == 1 && d.present && !d.corpse && d.health > 0.0)
        .map(|(i, _)| i).collect()
}

#[test]
fn the_first_guard_down_brings_the_whole_lift_squad_and_the_next_wave_comes_alive() {
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso(), "07000013.bin", false, None, 3).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x2a), "ch 0x2A waits on the first guard's death [D]");
        assert!(group1(&w).is_empty(), "the squad is held back until then [M 0x137aa0]");
        assert_eq!(sim.drones.spawners.len(), 2, "both lobby spawners [D #323/#327]");
    }
    let spots: Vec<[f32; 3]> = w.scene.sim.as_ref().unwrap().drones.list.iter().map(|d| d.pos).collect();
    north_lobby(&mut w);
    // The writer: guard #318 (p12 = 0x2A) dies through the drone's own death state.
    let g318 = {
        let sim = w.scene.sim.as_mut().unwrap();
        let i = sim.drones.list.iter().position(|d| d.death_channel == 0x2a && d.present).expect("a ch-0x2A guard");
        down(&mut sim.drones.list[i]);
        i
    };
    let mut armed = None;
    for t in 0..400 {
        north_lobby(&mut w);
        keep_down(&mut w, g318);
        w.wait(1);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x2a) { armed = Some(t); break; }
    }
    assert!(armed.is_some(), "guard #{g318}'s death raises ch 0x2A [M 0x13e660; D p12]");

    // One tick per guard once the spots pass the screens.
    let mut most = 0;
    for _ in 0..240 {
        north_lobby(&mut w);
        w.wait(1);
        most = most.max(group1(&w).len());
        if most == 4 { break; }
    }
    assert_eq!(most, 4, "mode 3 fills the wave to max 4 while the first guards stand [M 0x137e68:176-210; D #327 p2]");

    north_lobby(&mut w);
    w.wait(5); // the last arrival runs its first update

    // Wave 1 down, corpses still lying: the slots clear on the fall and wave 2 walks in.
    let wave1 = group1(&w);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for &i in &wave1 { down(&mut sim.drones.list[i]); }
    }
    let mut wave2 = Vec::new();
    for _ in 0..1800 {
        north_lobby(&mut w);
        for &i in &wave1 { keep_down(&mut w, i); }
        w.wait(1);
        let g = group1(&w);
        if g.len() == 4 { wave2 = g; break; }
    }
    assert_eq!(wave2.len(), 4, "the endless spawner brings the next whole wave [D #327 waves 0 = 32000; M 0x137e68:126-172]");
    let sim = w.scene.sim.as_ref().unwrap();
    // A record whose earlier guard still lies there makes a NEW object [M 0x137cb0
    // `NDrone2_CreateObj`], so a wave-2 guard can sit past the load-time list: measure him from
    // the nearest of the group-1 spawner's template records (their load-time spots).
    let records: Vec<[f32; 3]> = sim.drones.spawners.iter().filter(|s| s.group == 1)
        .flat_map(|s| s.templates.iter().map(|&t| spots[t])).collect();
    assert!(!records.is_empty(), "the group-1 spawner holds its templates");
    for &i in &wave2 {
        let dr = &sim.drones.list[i];
        let moved = records.iter().map(|t| ((dr.pos[0] - t[0]).powi(2) + (dr.pos[2] - t[2]).powi(2)).sqrt())
            .fold(f32::MAX, f32::min);
        assert!(moved < 12.0, "wave 2's guard {i} starts from its template spot, not a corpse's [M 0x137cb0], moved {moved:.1}");
        assert!(dr.health > 0.0 && !dr.corpse, "a fresh guard, not the last one's body");
    }
}
