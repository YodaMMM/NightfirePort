//! 07000007, the library door pair #404/#417 (CivilianDoorGuard 0x31) in front of door #244.
//!
//! The pairing [M]: `Drone_PostLoad_Init` 0x136938 runs each drone's `DefaultInit` and start-state
//! ENTER in list order, so the 0x31 ENTER's `NDrone2_SetDronePartner` 0x141360 scan sees only the
//! guards already initialised. 404 (first) finds no one; 417 pairs 404, takes the 0xff mark, and
//! the zone planes are built on 404 with the normal toward the hall (south) [M 0x141360:121-150].
//! 404 evaluates `NDrone2_DoorGuardHandler` 0x1416d8 each UPDATE; 417 mirrors 404's zone.
//! - zone 2 (within 2.0 in front of the pair's line, between the two): anim 0x5f variant 1 and,
//!   from 404 only, the warning line `SpeechSFX(ChooseSFX(0x4f, 0x50), 0, 1, 1)`
//!   [M 0x160648:73-84];
//! - zone 3 (past the line, within 6.0): `SpeechSFX(ChooseSFX(0x8f, 0x92))`,
//!   `DroneFunc_SetMissionFailReason(0xc)` (FailLabel 0x4000033 + ch 99) and Attack
//!   [M 0x160648:89-94, ally::mission_fail_reason].
//! The old port let every guard be a boot candidate, chained 404→417→442↔441 and left 404 and
//! 417 both 0xff mirrors: no warning, no fail.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The walkreal route from the party start to the hall in front of the library door.
fn walk_to_the_hall(w: &mut Walker) {
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    let legs: &[(&str, [f32; 3])] = &[
        ("west aisle south", [35.7, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
        ("the hall west mouth", [76.6, 43.6, -50.7]),
        ("the hall floor", [83.1, 43.6, -50.5]),
        ("the hall east", [93.3, 43.6, -50.4]),
    ];
    for &(name, at) in legs { w.walk_nav_to(at, name, 1.5); }
}

fn idx(w: &Walker, placement: usize) -> usize {
    w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.placement == placement)
        .unwrap_or_else(|| panic!("drone #{placement} created"))
}

/// Walks Bond (his own input, fists out) up to the gap between the pair and holds there.
/// Returns whether 404 spoke a warning line and whether 417 played the challenge.
fn approach(w: &mut Walker, target: [f32; 3], ticks: u32) -> (Option<usize>, bool) {
    let (g404, g417) = (idx(w, 404), idx(w, 417));
    let mut line = None;
    let mut mirror_challenged = false;
    for t in 0..ticks {
        let input = {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.gun.weapon = 0; sim.gun.switch_request = 0; // unarmed: the zones run only past NearArmedPerson [M 0x160648:39]
            let p = sim.bond.pos;
            let near = walker::dxz(p, target) < 0.3;
            let err = walker::wrap_pi((target[0] - p[0]).atan2(target[2] - p[2]) - sim.bond.yaw);
            Input { turn: err.clamp(-0.08, 0.08), forward: if near || err.abs() > 1.0 { 0.0 } else { 0.5 }, ..Default::default() }
        };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let (a, b) = (&sim.drones.list[g404], &sim.drones.list[g417]);
        if line.is_none() && a.door_zone_step == 2 { line = a.talk_line; }
        if b.door_zone_step == 2 { mirror_challenged = true; }
        if t % 120 == 0 {
            eprintln!("[07 pair t{t}] bond {:?} 404 zone {} step {} talk {:?} state {:#04x} | 417 zone {} step {} talk {:?}",
                sim.bond.pos, a.door_zone, a.door_zone_step, a.talk_line, a.state, b.door_zone, b.door_zone_step, b.talk_line);
        }
    }
    (line, mirror_challenged)
}

/// Walked up to the library door before Drake's reveal, an unarmed Bond is warned: 404 plays the
/// challenge and speaks line 0x4f or 0x50, 417 plays the challenge silently, and nothing fails.
#[test]
fn the_library_door_pair_warns_an_unarmed_bond() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    walk_to_the_hall(&mut w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(21), "Drake's reveal has not played (ch 21 off)");
        assert_eq!(sim.drones.list[idx(&w, 404)].state, state::CIVILIAN_DOOR_GUARD, "#404 guards the door");
    }
    // In front of the gap, 1.5 south of the pair's line (z −48.0): the challenge band [0, 2).
    let (line, mirror) = approach(&mut w, [94.3, 43.6, -49.5], 600);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(matches!(line, Some(0x4f) | Some(0x50)), "#404 speaks the warning line 0x4f/0x50 [M 0x160648:78-79], got {line:?}");
    assert!(mirror, "#417 plays the challenge from 404's zone (the mirror read [M 0x1416d8:19-21])");
    assert!(!sim.drones.channels.on(99), "the warning band does not fail the mission");
    assert_eq!(sim.drones.list[idx(&w, 404)].state, state::CIVILIAN_DOOR_GUARD, "a warned, unarmed Bond leaves 404 at his post");
    let (a, b) = (&sim.drones.list[idx(&w, 404)], &sim.drones.list[idx(&w, 417)]);
    assert_eq!((a.door_partner, a.door_side), (Some(idx(&w, 417)), 0), "#404 pairs #417 and owns the zone planes");
    assert_eq!((b.door_partner, b.door_side), (Some(idx(&w, 404)), 0xff), "#417 pairs #404 and mirrors him");
}

/// Past the pair's line (Bond put inside the zone-3 band behind them, on the real level after the
/// real walk): 404 raises the mission fail (FailLabel 0x4000033, ch 99) and attacks.
#[test]
fn bond_past_the_library_door_pair_fails_the_mission() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    walk_to_the_hall(&mut w);
    let g404 = idx(&w, 404);
    let mut failed = false;
    for _ in 0..120 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.gun.weapon = 0; sim.gun.switch_request = 0;
            // Between the two guards' x (93.66..94.89), 0.5 north of their line z −48.0: s0 = −0.5.
            sim.bond.pos = [94.3, sim.bond.pos[1], -47.5];
            sim.bond.vel = [0.0; 3];
        }
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(99) { failed = true; break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let g = &sim.drones.list[g404];
    eprintln!("[07 pair past] 404 zone {} state {:#04x} talk {:?} fail_label {:#x}", g.door_zone, g.state, g.talk_line, sim.drones.mission.fail_label);
    assert!(failed, "Bond past the pair's line raises the 0xc mission fail (ch 99) [M 0x160648:89-94]");
    assert_eq!(sim.drones.mission.fail_label, 0x0400_0033, "the fail label for reason 0xc [M ally::mission_fail_reason]");
    assert!(matches!(g.talk_line, Some(0x8f..=0x92)), "#404 speaks a 0x8f..0x92 line, got {:?}", g.talk_line);
}
