//! probe14_kiko_sight: the Kiko-corridor sightline measurement the middle-lane lever needed
//! (real-health-00agent-14-landing-3). The 11-15 ledger closed the corridor crossings by runs
//! (the north lane z 36.2 dies to #29's explosive kill line, the z 34.4 band dies to #20's
//! tracked fire); this probe pins the GEOMETRY behind both results so the next round does not
//! re-try a line choice. One load at the walk's difficulty (3), probe teleports only — no walk
//! state is written.
//!
//! Measured [L this run, deterministic load]:
//! - The corridor squad sits HIDDEN (present=false) at load: #20 (0x5000090) at (-51.3, 32.4)
//!   — the pocket, #21 (0x5000087) at (-47.9, 33.5) — the east machine cover, #19 (0x5000087)
//!   at (-45.9, 27.3) — the second-hold area; #29 (0x5000093, the explosive 0x14 shooter) is
//!   scripted in later (not in the load list).
//! - The nav route from the Kiko hold (-51.3, 37.9) to the second hold (-44.2, 27.6) runs
//!   (-48.64, 0.40, 32.13) → (-48.63, 0.40, 30.49): the shipped "node 1" pass is d ~2.5 from
//!   #20's pocket — the 15.0 point-blank the ledger records.
//! - #29's sightline from his measured firing spot (-49.1, 30.2): CLEAR across x -50..-48 at
//!   every z 34.0-34.5 (d 3.8-4.4 — CLOSER than the north lane's measured 6.1 kill), blocked
//!   only west of x -51 (t 0.44-0.9) and east of x -47 (t 0.65-0.8). The middle lane is NOT
//!   protected from the explosive by geometry.
//! - #20's pocket sightline from (-51.2, 32.6): blocked to x -52..-50 at z 34.0-34.5 (his hide
//!   cover, t 0.21-0.28), CLEAR to x -49..-48 (d 2.6-3.7), blocked east of x -47; the north
//!   lane (z 36.2) is blocked from the pocket too (t 0.11) — the north run's 0 #20 hits.
//! - The only z 34.0-34.5 band clear of BOTH sightlines is x ≤ -51.5 (inside #20's pocket
//!   itself — the 3.0 u auto-hit band [M DroneWeap_DoBulletAccuracy 0x179740:113-115]) or the
//!   east cover at x ≤ -47 (d 1.3-1.5 from #21's spot). No walkable line exists; the lever is
//!   closed by geometry, matching the measured t27096 death.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::nav_route;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The corridor squad's load state (hidden guards, positions [D placements; L the run]).
#[test]
fn the_kiko_squad_sits_hidden_at_its_pockets() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000014.bin", false, None, 3).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let near = |p: [f32; 3], x: f32, z: f32| ((p[0] - x).powi(2) + (p[2] - z).powi(2)).sqrt() < 1.0;
    let find = |x: f32, z: f32| sim.drones.list.iter().position(|d| near(d.pos, x, z))
        .expect("the corridor guard at the pocket [D placements; L this run]");
    let p20 = find(-51.3, 32.4);
    let d20 = &sim.drones.list[p20];
    assert_eq!(d20.skin, 0x500_0090, "#20 is the pocket's 0x5000090 [D; L this run]");
    assert!(!d20.present && !d20.enabled, "#20 sits HIDDEN until the corridor trigger [L this run]");
    let p21 = find(-47.9, 33.5);
    let d21 = &sim.drones.list[p21];
    assert_eq!(d21.skin, 0x500_0087, "#21 is the east cover's 0x5000087 [D; L this run]");
    assert!(!d21.present && !d21.enabled, "#21 sits HIDDEN too [L this run]");
    // #29 (the explosive 0x5000093 shooter) is NOT in the load list — scripted in later.
    assert!(!sim.drones.list.iter().any(|d| d.skin == 0x500_0093 && (d.pos[0] + 49.9).abs() < 1.0 && (d.pos[2] - 32.7).abs() < 1.0),
        "#29 is not a load placement at (-49.9, 32.7) — scripted [L this run]");
}

/// The nav route's node list from the hold to the second hold (the shipped "node 1" pass).
#[test]
fn the_hold_route_passes_node_1_at_2_5u_from_the_pocket() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000014.bin", false, None, 3).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let pts = nav_route(sim, [-51.3, 0.9, 37.9], [-44.2, 0.9, 27.6]).expect("the route exists [L this run]");
    assert_eq!(pts.len(), 2, "the route is the two-node south leg [L this run]");
    assert!((pts[0][0] - -48.64).abs() < 0.1 && (pts[0][2] - 32.13).abs() < 0.1,
        "node 0 is (-48.64, 32.13) [L this run]");
    assert!((pts[1][0] - -48.63).abs() < 0.1 && (pts[1][2] - 30.49).abs() < 0.1,
        "node 1 is (-48.63, 30.49) [L this run]");
    let d = ((pts[0][0] - -51.33).powi(2) + (pts[0][2] - 32.42).powi(2)).sqrt();
    assert!((d - 2.7).abs() < 0.2, "the node-0 pass sits d ~2.7 from #20's pocket — the 3.0 u auto-hit band [M DroneWeap_DoBulletAccuracy 0x179740:113-115; L this run d {d:.2}]");
}

/// The sightline matrix: #29's line covers the middle lane; #20's pocket line does not.
#[test]
fn the_middle_lane_is_inside_29s_sightline_and_outside_20s() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000014.bin", false, None, 3).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let sight = |from: [f32; 3], to: [f32; 3]| sim.world.segment_bullet_scoped(sim.world.find_cel(from), from, to).is_none();
    // #29's measured firing spot (-49.1, 30.2) [L the north-lane run's trace]: the middle lane
    // z 34.0-34.5 at x -50..-48 is CLEAR — and closer than the north lane's 6.1 kill.
    for z in [34.0f32, 34.4, 34.5] {
        for x in [-50.0f32, -49.0, -48.0] {
            assert!(sight([-49.1, 1.0, 30.2], [x, 1.2, z]),
                "#29's line to ({x:.0}, {z:.1}) is clear — the lane is exposed [L this run]");
        }
    }
    // The east cover (x -47) and the west wall (x -51) block it — outside the walkable band.
    assert!(!sight([-49.1, 1.0, 30.2], [-51.0, 1.2, 34.4]), "x -51 blocks #29's line [L this run]");
    assert!(!sight([-49.1, 1.0, 30.2], [-47.0, 1.2, 34.4]), "x -47 blocks #29's line [L this run]");
    // The north lane (z 36.2) — the MEASURED kill line — is clear from the same spot.
    assert!(sight([-49.1, 1.0, 30.2], [-50.0, 1.2, 36.2]), "the north lane stays in #29's line [L this run]");
    // #20's pocket (-51.2, 32.6): his hide cover blocks x -52..-50 (the middle-lane premise
    // half that DID hold), and the north lane too (the north run's 0 #20 hits).
    assert!(!sight([-51.2, 1.0, 32.6], [-50.0, 1.2, 34.4]), "the pocket's cover blocks x -50 [L this run]");
    assert!(!sight([-51.2, 1.0, 32.6], [-50.0, 1.2, 36.2]), "the pocket cannot see the north lane [L this run]");
    assert!(sight([-51.2, 1.0, 32.6], [-48.0, 1.2, 34.4]), "the pocket sees x -48 [L this run]");
}
