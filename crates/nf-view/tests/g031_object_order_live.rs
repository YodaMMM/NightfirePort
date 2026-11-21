//! G-031: the players' place in the per-tick object order — the ground carry.
//!
//! The game's carry (`Player_FeetOnPoint` 0x194188 → `Control_InheritVelocity` 0x133e88) runs in
//! the player's collide slot, `Player_CollisionHandler` 0x1949a8, which `Collide_Update` calls
//! AFTER the update walk [M 0x1336f8:54-68 walk, :238 Collide_Update]. The carrier (a script
//! entity moved by its `SP_Update` inside the walk) has already made this tick's move, so Bond
//! ends every tick standing on the carrier's new pose. The port used to carry before the walk on
//! the carrier's previous move: Bond trailed every lift by one tick (0700004a's descending car:
//! one tick's fall, ~0.03, above the floor at every tick's end).
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The 0700004a lift platform entity's keyed height (the lift SP's stream entity 0x020005b4 [D]).
fn platform_y(sim: &nf_view::scene::Sim) -> Option<f32> {
    sim.drones.scripts.objects.list.iter().flatten().find_map(|o| match o {
        nf_game::nis::Obj::Entity { hash: 0x0200_05b4, matrix, .. } => Some(matrix[3][1]),
        _ => None,
    })
}

/// Bond standing on the descending 0700004a lift keeps the same height over the car's floor at
/// every tick's end as he does while the car is parked — the carry is the car's own move of the
/// same tick. Fails on the old order (pre-walk carry on the previous tick's move): the offset
/// grows by the car's per-tick fall (150 units over 4950 ticks [D keys] ≈ 0.030).
#[test]
fn bond_rides_the_0700004a_lift_on_the_same_tick() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
    }
    let mut rest: Option<f32> = None;
    let mut worst: f32 = 0.0;
    let mut moving_ticks = 0;
    let mut prev_py: Option<f32> = None;
    for t in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.health.health = 100.0; // the shaft garrison's fire is not under test
        let py = platform_y(sim).expect("the lift platform entity exists");
        let off = sim.bond.pos[1] - py;
        let falling = prev_py.is_some_and(|p| py < p - 0.01);
        prev_py = Some(py);
        if t == 60 {
            assert!(!falling, "the car is parked at t60 (its keys start at t90 [D])");
            rest = Some(off);
        }
        if (300..900).contains(&t) && falling {
            moving_ticks += 1;
            let r = rest.unwrap();
            worst = worst.max((off - r).abs());
        }
    }
    assert!(moving_ticks > 500, "the car descends through t300..t900 ({moving_ticks} falling ticks)");
    assert!(worst < 0.005,
        "Bond keeps his parked height over the descending car at every tick's end (the carry is the car's move of the same tick [M 0x1336f8:238 → 0x1949a8 → 0x133e88]); worst drift {worst}");
}

/// A plate Bond stands on writes its channel one tick after the contact: the trigger reads the
/// contact list `Collide_Update` builds AFTER the update walk [M 0x1336f8:238; 0x1ea398:25-53],
/// and every trigger read of a tick (`Trigger_Calc` at the walk's head [M 0x1336f8:24], the walk's
/// own `Trigger_Update` [M 0x1b1a78 → `Trigger_Hit` 0x1b1fc8:18]) comes before that tick's Collide.
/// On 07000005 the cold start stands on the reveal plate #36 (TouchOnce out 0x45 [D]): no Collide
/// has run before the first tick, so ch 0x45 is still off after it and on after the second.
/// Fails on the old order (the toucher read Bond's position of the same tick: on after tick 1).
#[test]
fn the_05_reveal_plate_fires_on_the_tick_after_the_contact() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.drones.channels.on(0x45), "nothing writes ch 0x45 before the first tick");
    sim.step_bond(Input::default(), false, false, false);
    assert!(!sim.drones.channels.on(0x45),
        "tick 1: the plate's contact list is still empty (no Collide_Update has run) — ch 0x45 waits");
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.channels.on(0x45), "tick 2: the reveal plate reads tick 1's contact and writes ch 0x45");
}
