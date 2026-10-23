//! hatch-crouch-0c3: the 0700000c roof-hatch traversal as ONE walked route — from the duct ramp
//! top on the roof, through the open hatch's hole, crouched, south along the balcony's east half
//! to the far walkway, and on into the main room's walkway (the z −149.65 / y 15.07 walkway the
//! walkthrough spawns on [D walkreal_0700000c spawn #1]) — driven with `Sim::step_bond` inputs
//! only (the walker's own steering convention: `desired = atan2(dx, dz)` —
//! common/walker.rs:209). The hatch opening itself is the established laser stand-in
//! (`Lock_Update::burn`, the helper `hatch_crouch_live` uses; the beam CONTACT side is
//! fix-gadgets' and covered there). The duct ascent above the ramp top remains the level's one
//! unidentified leg [L open live check — the game's climb is
//! ladder-only, M Player_CollLadder 0x196118, and the vent carries no ladder [D]], so the route
//! starts on the ramp top like the sister tests do; every metre after that is walked.
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}

/// Burn the four display-vent hinges (locks with out chs 14–17 [D #139-142 param 5]); the AND mux
/// #505 raises ch 18 and `ventmesh-front` #138 sweeps down in its rate-50 ticks [M 0x1a9c58:51].
fn burn_display_hinges(sim: &mut nf_view::scene::Sim) {
    for k in 0..sim.drones.locks.len() {
        if (14..=17).contains(&sim.drones.locks[k].out) {
            sim.drones.locks[k].burn_through(&mut sim.drones.channels, 1);
        }
    }
}

/// Walk toward `target` with step_bond inputs (the walker's steering: turn toward
/// `atan2(dx, dz)`, walk while the heading error is under 1 rad). Returns (arrived, ticks).
fn walk_toward(sim: &mut nf_view::scene::Sim, target: [f32; 3], arrive: f32, max_ticks: u32) -> (bool, u32) {
    for tick in 1..=max_ticks {
        let p = sim.bond.pos;
        let dx = target[0] - p[0];
        let dz = target[2] - p[2];
        if (dx * dx + dz * dz).sqrt() < arrive { return (true, tick); }
        let desired = dx.atan2(dz);
        let err = wrap_pi(desired - sim.bond.yaw);
        let input = Input {
            turn: err.clamp(-0.08, 0.08),
            forward: if err.abs() > 1.0 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
    }
    (false, max_ticks)
}

#[test]
fn from_the_roof_ramp_through_the_hatch_crouch_south_into_the_main_room() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = {
        let iso = iso().expect("NF_ISO");
        let mut scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        for _ in 0..100 { sim.step_bond(Input::default(), false, false, false); }
        let sim = scene.sim.as_mut().unwrap();
        // The duct ramp top (x 4.5..7.0 topping 17.2-17.5 [D probes]) — the route's stand-in
        // start for the level's unidentified ascent leg; the walked route begins here.
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [5.8, 18.6, -110.8], std::f32::consts::PI);
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
        scene
    };

    // ── leg 1: the hatch. Burn the hinges, wait out the sweep, and WALK off the duct edge ──
    {
        let sim = scene.sim.as_mut().unwrap();
        burn_display_hinges(sim);
    }
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        let ss138 = sim.drones.sss.of_placement(138).expect("ventmesh-front SS exists [D #138]");
        let last138 = (sim.drones.sss.list[ss138].keys.len() - 1) as u16;
        let open138 = nf_game::door::interp(&sim.drones.sss.list[ss138].keys, 0, last138, last138 as f32, sim.drones.sss.list[ss138].spline).unwrap();
        assert_eq!(sim.drones.sss.list[ss138].matrix, open138, "the hatch is fully open before the drop (the held pose is the last key's)");
        let p = sim.bond.pos;
        assert!(p[1] > 16.5, "Bond stands on the duct ramp top; at y {:.2}", p[1]);
    }
    // The walked approach: crouch, then hold forward off the duct edge, through the wall plane
    // (z −112.25) and down through the open hole onto the balcony.
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input { crouch: true, ..Default::default() }, false, false, false);
    assert!(sim.bond.crouched, "the crouch key crouched Bond");
    let mut landed = None;
    for tick in 1..=1200u32 {
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        let p = sim.bond.pos;
        if p[2] < -112.6 && p[1] < 15.5 && sim.bond.grounded { landed = Some((tick, p)); break; }
    }
    let (tick, p) = landed.expect("the walked crouch drops through the open hole onto the balcony");
    assert!(sim.bond.grounded, "landed grounded");
    assert!(p[1] > 14.5 && p[1] < 15.5, "root over the balcony floor 14.61-14.81, at y {:.2}", p[1]);
    assert!(p[0] > 4.5 && p[0] < 7.0, "landed under the vent opening (x 4.54..7.02 [D #138]), at x {:.2}", p[0]);
    eprintln!("[real] leg 1: the hatch drop landed at tick {tick}: ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);

    // ── leg 2: the balcony's east half south to the far walkway ──
    // The pier ramp occupies x ≤ 9 [D probes]; the verified south route is the east half
    // (x 10-14 — hatch-crouch-0c). Walk to the x 11 lane first, then cross the well's south
    // edge (z −121.5) crouched to the far walkway (floor 14.61 continuing at z −126.5+).
    let (ok, ticks) = walk_toward(sim, [11.0, 0.0, -116.0], 0.6, 600);
    assert!(ok, "the crouch-walk reaches the east-half lane (x 11): {:?}", sim.bond.pos);
    eprintln!("[real] leg 2a: on the east-half lane at tick {ticks}: {:?}", sim.bond.pos);
    let (ok, ticks) = walk_toward(sim, [11.0, 0.0, -126.0], 0.9, 900);
    assert!(ok, "the crouch-walk crosses the well's south edge to the far walkway: {:?}", sim.bond.pos);
    let p = sim.bond.pos;
    assert!(p[1] > 15.0 && p[1] < 15.5, "still on the 14.61 floor (crouch root ~15.26), at y {:.2}", p[1]);
    eprintln!("[real] leg 2b: the far walkway at tick {ticks}: ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);

    // ── leg 3: the far walkway south into the main room ──
    // Stand up and walk the remaining ~25 m of walkway south to the walkthrough's spawn area
    // (z −149.65, y 15.07 [D walkreal_0700000c spawn #1] — the main room's walkway).
    sim.step_bond(Input { crouch: true, ..Default::default() }, false, false, false);
    let (ok, ticks) = walk_toward(sim, [9.0, 0.0, -148.5], 1.2, 4500);
    let p = sim.bond.pos;
    assert!(ok, "the walk reaches the main room's walkway (z −148): {p:?}");
    assert!(sim.bond.grounded, "arrived grounded");
    assert!(p[1] > 14.4 && p[1] < 15.9, "on the main room's walkway floor (spawn #1 root 15.07 [D]; the walkway steps vary the root along the route), at y {:.2}", p[1]);
    eprintln!("[real] leg 3: the main room at tick {ticks}: ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);
}
