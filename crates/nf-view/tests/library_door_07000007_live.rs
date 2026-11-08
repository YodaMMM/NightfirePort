//! 07000007, the library's two doorways, walked with the player's own input from the party start.
//!
//! THE WEST DOORWAY (door #245, (80.00, 43.00, −49.87), a free swing door [D placements]): the
//! waiter #426 (spawn (80.71, −50.47), switch ch 23, second ch 21 [D p3/p6]) wakes on ch 23 (the
//! group plate #415 Bond crosses on the north row) and walks his kind-8 route Path_3 west through
//! the doorway (nodes 7 → 6 → 0 → 11 → 12 → 13 → 14 ... → 5 (65.72, −52.09) [D path dump]). He
//! pushes the door open from its east side, so the leaf swings west and lies along z −49.87, x
//! 78.7..80.0 [the port's door keys]; his creep then cuts the corner toward node 13 (77.48, −49.77)
//! across the leaf's south face. The game's drone is pushed out of what it walks into —
//! `Drone_CollisionHandler` 0x140968:204-208 adds the contact DeltaP to obj+0x30 — so he slides
//! along the leaf and goes on. The port's `move_by` rejected the step outright and he stood in the
//! doorway for good, walking on the spot.
//!
//! THE LIBRARY DOOR (door #244, (94.89, 43.00, −47.25), free [D]): the CivilianDoorGuard pair
//! #404/#417 (state 0x31) stands in front of it at z −48.06, 1.03 apart, until ch 21 (Drake's
//! reveal) takes them off [D p6 = 21, p8 = 100; the port's second-tag arm]. Within 2.0 of Bond a
//! drone's collision is ON [M NDrone2_DoCollision 0x149ec8 → DroneFunc_NearPlayerCollision
//! 0x14b190] and the SP Jointy push moves the player the full overlap off the bot's capsule
//! [M Collide_Jointy 0x1ec180:229-281] — Bond cannot walk between them to the hall walk-in plate
//! #412 (ch 0x48). The port pushed only an airborne Bond, so he walked through both guards.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The walkreal route from the party start to the hall's west mouth (the main walk's anchors).
fn walk_to_the_hall(w: &mut Walker, upto_west_mouth: bool) {
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
    for &(name, at) in legs {
        w.walk_nav_to(at, name, 1.5);
        if upto_west_mouth && name == "the hall west mouth" { break; }
    }
}

fn drone(w: &Walker, placement: usize) -> &nf_game::drone::Drone {
    w.scene.sim.as_ref().unwrap().drones.list.iter().find(|d| d.placement == placement)
        .unwrap_or_else(|| panic!("drone #{placement} created"))
}

/// The waiter #426 opens the west doorway's door on his own route and walks on through it to his
/// route's end — he does not stand in the doorway.
#[test]
fn the_waiter_walks_on_through_the_west_library_doorway() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    {
        let g = drone(&w, 426);
        assert_eq!((g.switch_channel, g.second_channel), (23, 21), "#426 wakes on ch 23 and leaves on ch 21 [D p3/p6]");
    }
    walk_to_the_hall(&mut w, true);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(23), "the north-row plate #415 woke the waiter (ch 23) [D]");
    let mut min_x = f32::MAX;
    let mut doorway_ticks = 0u32;
    for t in 0..2400u32 {
        w.tick(Input::default(), false);
        let g = drone(&w, 426);
        min_x = min_x.min(g.pos[0]);
        // The doorway band: between the door's hinge line and the leaf's far end.
        if (78.7..80.3).contains(&g.pos[0]) && (-50.9..-49.6).contains(&g.pos[2]) { doorway_ticks += 1; }
        if t % 400 == 0 { eprintln!("[07 waiter t{t}] #426 {:?} state {:#04x}", g.pos, g.state); }
    }
    let door = w.scene.sim.as_ref().unwrap().drones.doors.list.iter().find(|d| d.placement == 245).unwrap();
    eprintln!("[07 waiter] door #245 state {:#04x}; west-most x {min_x:.2}; ticks in the doorway {doorway_ticks}", door.state);
    let g = drone(&w, 426);
    assert!(min_x < 77.0, "the waiter must get past the open leaf (x 78.7..80.0) — west-most x {min_x:.2}, now at {:?}", g.pos);
    assert!(doorway_ticks < 400, "the waiter must not stand in the doorway ({doorway_ticks} ticks there)");
    let end = [65.72f32, -52.09];
    let d_end = ((g.pos[0] - end[0]).powi(2) + (g.pos[2] - end[1]).powi(2)).sqrt();
    assert!(d_end < 1.5, "the waiter reaches his route's end node 5 (65.72, −52.09) [D Path_3], {d_end:.2} away at {:?}", g.pos);
}

/// The library door guards hold the doorway: Bond opens door #244 (the free E-door) but cannot walk
/// between #404 and #417 to the hall walk-in plate #412 while ch 21 is off.
#[test]
fn the_library_door_guards_hold_the_doorway_before_drakes_reveal() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000007.bin", false).expect("load 07000007"));
    walk_to_the_hall(&mut w, false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(21), "Drake's reveal has not played (ch 21 off)");
        for p in [404usize, 417] {
            let g = drone(&w, p);
            assert!(g.present && g.state == nf_game::drone::state::CIVILIAN_DOOR_GUARD,
                "#{p} guards the library door (state 0x31), got {:#04x}", g.state);
        }
    }
    // E the door from in front of the pair (the use sphere reaches it past them).
    w.walk_to(Wp { at: [94.3, 43.6, -48.9], act: Act::Use { use_ticks: 240 }, arrive: 0.8 }, "E the library door");
    // Then push for the plate between the guards for 20 s, pulsing E on a closed door.
    let target = [94.3f32, 43.6, -45.9];
    let mut north = f32::MIN;
    for t in 0..1200u32 {
        let (input, use_key) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let closed = sim.drones.doors.list.iter().any(|d| d.placement == 244 && d.state & 0xc == 0);
            let err = walker::wrap_pi((target[0] - p[0]).atan2(target[2] - p[2]) - sim.bond.yaw);
            (Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() },
             closed && t % 60 == 30)
        };
        w.tick(input, use_key);
        let sim = w.scene.sim.as_ref().unwrap();
        north = north.max(sim.bond.pos[2]);
        if t % 300 == 0 { eprintln!("[07 guards t{t}] bond {:?} door #244 {:#04x}", sim.bond.pos,
            sim.drones.doors.list.iter().find(|d| d.placement == 244).unwrap().state); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x48), "the guards keep Bond off the hall walk-in plate #412 (ch 0x48) before ch 21");
    assert!(north < -48.2, "Bond never gets between or past the guards (z −48.06) — north-most z {north:.2}");
}
