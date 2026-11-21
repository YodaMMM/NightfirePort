//! The physics-side seams of the driving AI: the re-exported physics paths and the sim view the
//! brain reads off the port's `Car` (pos, vel, the forward and side rows). The brain-on-physics
//! behaviour tests live in driving_ai_traffic.rs.

use nf_game::driving::ai::SimCar;
use nf_game::driving::{Car, CarParams, DT, Rigid, Tuning};

/// Bond's shipped pvehicle numbers [D vanquish.atr].
fn vanquish() -> CarParams {
    CarParams::vanquish()
}

// ---------------------------------------------------------------- the tests

#[test]
fn the_506_public_paths_survive_the_module_move() {
    // The layout fix moved 506's `driving.rs` to `driving/physics.rs`; every public path 506's
    // tests and callers use must stay. The imports above ARE the compile-time assertion
    // (`nf_game::driving::{Car, CarInput, CarParams, DT, FlatWorld, Rigid, Tuning}`).
    assert_eq!(DT, 1.0 / 60.0, "the fixed step [M 0x2d6b90's {{60,60,50,60}} table]");
    assert_eq!(Rigid::game_defaults().gravity, -34.0, "Physics:Rigid gravity [M 0x3390d0]");
    assert_eq!(Rigid::game_defaults().bodge_player_gravity, -13.0, "the player bodge [M]");
    assert_eq!(vanquish().mass, 1660.0, "the shipped vanquish mass [D vanquish.atr]");
    assert_eq!(vanquish().maxacc, 3.0, "the shipped maxacc [D vanquish.atr]");
    // The .tun database still parses through the re-exported path.
    let t = Tuning::parse("ROLLING_RESISTANCE=0.1\r\n");
    assert_eq!(t.get("ROLLING_RESISTANCE"), Some(0.1), "the .tun lookup [M 0x1426a8]");
}

#[test]
fn the_sim_view_mirrors_the_physics_car() {
    let mut car = Car::new(vanquish(), [3.0, 0.0, 4.0], 0.5);
    car.vel = [1.0, 0.0, 2.0];
    let sim = SimCar::of_car(&car);
    assert_eq!(sim.pos, car.pos, "pos = instance+0x10 [M 0x133ab0:41]");
    assert_eq!(sim.vel, car.vel, "vel = instance+0x20 [M 0x133ab0:103-113]");
    let (s, c) = 0.5f32.sin_cos();
    assert_eq!(sim.forward, [s, 0.0, c], "the forward row is the yaw's [sin, 0, cos] [M]");
    assert_eq!(sim.right, [c, 0.0, -s], "the side row is d(forward)/d(yaw) [S pairing]");
    let len = (sim.forward[0] * sim.forward[0] + sim.forward[2] * sim.forward[2]).sqrt();
    assert!((len - 1.0).abs() < 1e-6, "the facing the dots multiply is unit [M]");
    assert_eq!(sim.alive, 1.0, "a live actor [S — FUN_00195a30 is the actor layer]");
}

// The rails-model wiring tests (steer = dot(dir, forward) = 1 on the nose, target = the
// segment limit, the car snapped to the spline) pinned values FUN_00133ab0 / FUN_0012e808 do
// not produce; the game-checked replacements are in driving_ai_traffic.rs
// (docs/systems/driving-ai.md §9).
