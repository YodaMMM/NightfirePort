//! level-0c-east-side-route: the game's own GROUND route across the gate band — walked on real
//! input. The straight band crossing is dead ([L 2026-10-30 fresh boot: the player drops into the
//! pit and stops at x 63.4-63.5 against its east wall; the port's collision matches — the
//! level-0c-gate-band-crossing report]), and the game's own nav graph routes its ground patrols
//! AROUND: west ground path nodes (42.97, 6.17, -207.22)..(62.18, 6.17, -207.35) + the south leg
//! to (54.80, 6.17, -237.96), east ground path nodes (80.63, 6.17, -207.68)..(94.34, 6.17,
//! -207.78) [D the 0c nav dump, 27 paths; the two sides are SEPARATE paths — no band-level link
//! and no ground link crosses the chasm]. This test walks the west→east leg of that network on
//! the player's own input:
//!  1. the band west end → turb2's staircase flights down (obj_turb2_staircase01/02/03, y
//!     2.97..11.17, x 49.29..58.73, z -208.96..-211.67 [D band9 plc20-22]),
//!  2. the crate-field/y-0 yard east — the game declares it walkable too ([D] nav path kind 0x202,
//!     the ring nodes (76.33..85.04, 0.00..0.01, -215.21..-230.69)),
//!  3. the running JUMP onto the east ramp (obj_turb4_staircase02/03's lower flight, steps
//!     1.75→7.19 at x 80.75..87.25, z -209.25..-211.25 [D the step scan]),
//!  4. up the ramp to the east ground (obj_turb4_walkway03, y 6.16..7.19, x 73.22..87.58 [D
//!     plc42]) — the east side reached at ground level, past the chasm x 64.05..72.82 that no
//!     band-level route crosses.
//!
//! The final east climb (ground 6.17 → the east band 10.34) is NOT pinned — every scanned face
//! dead-ends (staircase01's 9.0..9.34 west face over the under-stair void, walkway02's south
//! face at z -227.26, the crates4 tops 8.31, the band's south rail 11.17) — live check in the
//! report. Opt-in via NF_ISO.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn floor_at(w: &Walker, x: f32, y: f32, z: f32) -> Option<f32> {
    w.scene.sim.as_ref().unwrap().world
        .segment_hit([x, y, z], [x, y - 20.0, z]).map(|(t, _)| y - 20.0 * t)
}

#[test]
fn the_ground_route_crosses_to_the_east_side() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    // Start: the band west end (the whole walk's leg-10 spot, floor 10.34 [D probe door4]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [52.0, 11.4, -207.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = floor_at(&w, p[0], p[1], p[2]);
        assert!(sim.bond.grounded && f.map_or(false, |y| (10.2..10.6).contains(&y)),
            "Bond must start grounded on the band west (10.34 [D]), got {p:?} under {f:?}");
    }
    // 1. The staircase descent (the turb2 flights, real walk legs).
    w.walk_to(Wp::new(51.5, 11.0, -209.6).r(1.2), "onto the turb2 staircase top");
    w.walk_to(Wp::new(51.5, 4.0, -211.4).r(1.2), "down the turb2 flight");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 49.0 && p[0] < 54.0 && p[2] < -210.5,
            "the descent must reach the flight's south end, got {p:?}");
    }
    // 2. The crate field / y-0 yard east (the nav ring's level [D kind 0x202 path]).
    w.walk_to(Wp::new(62.0, 4.0, -212.0).r(1.5), "east along the south");
    w.walk_to(Wp::new(70.0, 2.0, -212.0).r(1.5), "into the crate field");
    w.walk_to(Wp::new(76.0, 1.2, -210.0).r(1.5), "the yard east");
    w.walk_to(Wp::new(79.0, 1.2, -209.0).r(0.8), "the ramp's west approach");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = floor_at(&w, p[0], p[1], p[2]);
        assert!(p[0] > 77.0 && p[0] < 80.0 && f.map_or(false, |y| y < 0.5),
            "the yard trunk must reach the ramp's west side at the y-0 floor, got {p:?} under {f:?}");
    }
    // 3. The running jump onto the east ramp's first step (1.75 at x 81.25..81.5 [D the step
    // scan]) — the walk test's own jump idiom (the slot jump, leg 9).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [79.6, 1.1, -210.2];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::FRAC_PI_2; // face +x (east)
    }
    w.wait(20);
    let mut jumped = false;
    for t in 0..300u32 {
        let (p, grounded) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.grounded)
        };
        if jumped && grounded && t > 20 { break; }
        let jump = !jumped && grounded && p[0] >= 80.0;
        if jump { jumped = true; }
        w.tick(nf_game::player::Input { forward: 1.0, jump, ..Default::default() }, false);
    }
    assert!(jumped, "the ramp jump never fired");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = floor_at(&w, p[0], p[1], p[2]);
        assert!(p[0] > 80.3 && f.map_or(false, |y| y > 1.0),
            "the jump must land on the ramp's lower steps (1.75+ [D]), got {p:?} under {f:?}");
    }
    // 4. Up the ramp (0.2 steps) to the east ground.
    w.walk_to(Wp::new(84.0, 4.5, -210.2).r(1.0), "up the ramp");
    w.walk_to(Wp::new(87.0, 7.0, -209.8).r(1.0), "the ramp top");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = floor_at(&w, p[0], p[1], p[2]);
        assert!(f.map_or(false, |y| (5.5..7.4).contains(&y)),
            "the ramp top must stand on the east ground level (6.16..7.19 [D plc42]), got {p:?} under {f:?}");
    }
    // The east side reached: walk west along the east ground past the chasm's east edge.
    w.walk_to(Wp::new(86.0, 7.4, -208.2).r(1.0), "the east ground west");
    w.walk_to(Wp::new(78.0, 7.4, -208.2).r(1.2), "the east ground at the chasm's east");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = floor_at(&w, p[0], p[1], p[2]);
        assert!(sim.bond.grounded && p[0] > 76.0 && f.map_or(false, |y| (6.0..7.4).contains(&y)),
            "the east ground leg must hold (6.17 [D]), got {p:?} under {f:?}");
    }
    eprintln!("[eastroute] the ground crossing walked in {} ticks; stuck {} falls {}",
        w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
    // Sister: the band-height void over the chasm stands — the ground route is the only route
    // the parsed disc declares, exactly as the crossing report concluded.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let gap = sim.world.segment_hit([59.0, 12.0, -207.5], [59.0, 8.0, -207.5]);
        assert!(gap.is_none(), "the band crossing gained floor at band height; the route map changed");
    }
}
