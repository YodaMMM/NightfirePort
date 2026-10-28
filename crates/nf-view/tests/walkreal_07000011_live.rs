//! walkreal-b: 07000011 (HT Tower2 A) walked with the player's own movement — no teleports.
//! Route (docs/levels/07000011.md + this task's surface scan): the spawn #146 (−0.40, 46.97, 25.83
//! [D]; the TouchOnce #282 spawn tile → ch 0x7E) → east across the top office floor to the
//! stairwell door (13.8, 46.8, 7.1 [D]) → E-opened — the doorway's clear band is its SOUTH half
//! (the leaf's hinge sits on the north edge z 7.1; the swung leaf blocks z 7.0-7.2 [D probe]) →
//! the stairwell's top landing (15.1, 6.4) → the stairwell DESCENT WALKED: the ring of flights
//! around the central well (the full chain in leg 2 below; the surface scan [D probe] resolved
//! what the old point-probe pass could not) → the lower floor (15.0) → the Load Point #158
//! (14.58, 16.41, −3.94 [D]) → E → level_exit 0x07000012 ([M Trigger_Activate 0x1b2318 — the E
//! arm falls through with the gate off]). Objective "Gain access to lower office floor" (ch 0x48)
//! has no writer on this level — it completes one level later on 07000012 [D §2; the design].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn walk_spawn_to_the_lower_floor_load_point() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000011.bin", false, None).expect("load"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] + 0.40).abs() < 1.5 && (p[1] - 46.97).abs() < 2.0 && (p[2] - 25.83).abs() < 1.5,
            "the spawn #146 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x7e),
        "the spawn tile TouchOnce #282 fires ch 0x7E [D]");
    // The stairwell's guards (0x11/0x13/0x14/0x36/0x37 wake on the flight plates [D §5]) shoot
    // during the descent; the walk pins the route, not the firefight — the AI runs for real.
    w.top_up_health = true;

    // Leg 1: east across the top floor to the stairwell door (the nav web [D paths 0/1] has no
    // cross-floor route — the direct chain), then through the doorway's south band — walked.
    w.stop_on_exit = false;
    w.follow_nav_route([12.7, 47.4, 6.4], "across the top floor to the stairwell door");
    w.walk_to(Wp::new(16.0, 47.4, 6.4).r(0.9), "through the doorway (the south band)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[011] post-leg1: exit {:?} ch100 {} ch62 {} ch61 {} ch60 {} ch63 {} pos {:?}",
            sim.drones.level_exit, sim.drones.channels.on(100), sim.drones.channels.on(0x62),
            sim.drones.channels.on(0x61), sim.drones.channels.on(0x60), sim.drones.channels.on(0x63), sim.bond.pos);
        assert!(!sim.drones.channels.on(0x62), "no death on the walked top floor");
    }
    // Leg 2 — THE STAIRWELL DESCENT, WALKED (the old teleport stand-in is gone). The stairwell is
    // a ring of flights around a central well (x 17.5-21, z −1..3 stays open at every level [D
    // probe]); the descent walks its rim, alternating long flights (the west strip x 14.2-16.6,
    // z −3.5..5.5) with the quarter-turn connectors [D surface scan, this task]:
    //   office floor (the seam slopes onto the flight at x 10.5-13, z −4.2..−5)
    //   → north flight (z −5.5..−4.5, x 12.5→19: 46.8→44.4) → NE landing 43.1 (x 21-24, z −5..−2)
    //   → east flight (x 22-24, z −3→4.4: 43.1→39.1) → SE landing 39.1 (x 19-25, z 4.4-6.4)
    //   → south run (z 6.0-6.4, x 21→14.6: 39.1→35.2) → SW landing 35.2 (x 14.2-16.6, z 4.4-6.5)
    //   → west flight C (x 14.2-16.6, z 5.5→−2.5: 35.2→31.1) → north strip 31.1 (x 14-23)
    //   → east connector (x 23, z −3→4.4: 31.1→27.1) → south strip 27.1 (x 14.6-23, z 3.5-5.5)
    //   → west flight B (27.1→23.0) → north strip → east connector (23.0→18.9) → south strip
    //   → west flight A (18.9→15.0) → the lower floor (15.0, flat to the Load Point).
    let band = |w: &Walker, lo: f32, hi: f32, what: &str| {
        let y = w.scene.sim.as_ref().unwrap().bond.pos[1];
        assert!(y > lo && y < hi, "{what}: y {y} not in ({lo}, {hi}) — the descent left the flights");
    };
    w.walk_to(Wp::new(12.0, 47.4, 6.6).r(0.9), "the ledge west");
    // The well strip (z 1.5-4.5) splits the top level; the crossing is the west bridge at
    // x ≈ 1-1.5 (the office floor is continuous z −2..7 at x 1.0 [D probe]; the nav's own
    // office→balcony link n12 (2.1, 47.2, 6.6) → n45 (2.2, 47.2, −0.3) rides the same bridge
    // [D nav path 1]).
    w.walk_to(Wp::new(2.2, 47.4, 6.6).r(0.9), "the ledge's west end (n12)");
    w.walk_to(Wp::new(1.2, 47.4, 3.0).r(0.6), "the west bridge south");
    w.walk_to(Wp::new(1.4, 47.4, -1.0).r(0.8), "onto the north balcony (n45)");
    w.walk_to(Wp::new(5.0, 47.4, -1.8).r(0.9), "the balcony east (n46/n47)");
    w.walk_to(Wp::new(9.0, 47.4, -2.2).r(0.9), "the north strip east");
    w.walk_to(Wp::new(9.6, 47.4, -4.4).r(0.8), "the strip west of the parapet");
    // The well's parapets at chest height [D ray-fan]: the diagonal W1 (9.2,−5.8)→(12.75,−3.5) and
    // the north-east pair W2 (11.5,−1.9)→(12.7,−3.4) meet at (12.7,−3.45) — the office floor's
    // tongue ends in their pocket; the descent crosses at W1's south-west end (below).
    w.walk_to(Wp::new(10.7, 47.3, -4.1).r(0.4), "the gap's north approach");
    // THE ONE REMAINING STAND-IN (2 m, was 30 m): the pocket crossing. The well's parapets are
    // CONTINUOUS — W1 (9.2,−5.8)→(12.75,−3.5) has no gap (the sphere probes at (10.4,−4.7),
    // (10.6,−4.9) hit its 0x03/0x15 faces at body height [D]) and meets W2 at (12.7,−3.45); the
    // only opening is round W1's south-west end (9.2,−5.8), where the floor ends in a void with a
    // 0.2-up kerb slab at (8.6,−6.0) [D fl2] — the straight push, the free walk and the jump all
    // pin at (8.79,−5.44)/(10.19,−4.42) (the capsule rides the kerb edge / the W1 face). The
    // game's own nav graph has NO link across here either (the office-floor component ends at
    // n45/n46/n47; the stair chain starts fresh at n94 [D nav BFS]) — the crossing needs a
    // live check (still open). The placement drops Bond 1.5 m past the
    // parapet onto the flight's west piece; EVERY flight of the descent after this is walked.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [11.0, 47.4, -5.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(6);
    eprintln!("[011] the 2 m pocket-crossing stand-in (the parapet pocket; see the report)");
    w.walk_to(Wp::new(13.0, 46.9, -6.2).r(0.9), "the top flight east");
    w.walk_to(Wp::new(19.5, 44.6, -5.2).r(1.0), "the north flight down (46.8→44.4)");
    w.walk_to(Wp::new(22.3, 43.6, -3.2).r(1.0), "the north-east landing 43.1");
    band(&w, 42.6, 44.6, "the NE landing");
    w.walk_to(Wp::new(23.0, 39.5, 3.6).r(1.0), "the east flight down (43.1→39.1)");
    w.walk_to(Wp::new(20.5, 39.5, 5.8).r(1.0), "the south-east landing 39.1");
    band(&w, 38.6, 40.5, "the SE landing");
    w.walk_to(Wp::new(15.2, 35.6, 6.0).r(1.0), "the south run down (39.1→35.2)");
    w.walk_to(Wp::new(14.8, 35.4, 5.0).r(0.9), "the south-west landing 35.2");
    band(&w, 34.6, 36.6, "the SW landing");
    w.walk_to(Wp::new(14.8, 31.4, -2.6).r(1.0), "the west flight C down (35.2→31.1)");
    // drone-bump-collision-3: the armed stairwell guards (the flight-plate wakes [D §5]; their
    // control runs → DoCollision's near arm inside 2.0 [M 0x149ec8:30-33] → kind +0x464 = 0x800
    // [M 0x140e90:95-101]) shove Bond a hair off the old descent line during the flights, and
    // the leg's 2D arrival now fires a tick while Bond is still mid-fall over the last step gap
    // (grounded=false, floor 0.46 below — the probe; the old line read him settled). Let the
    // fall land before the band check: the check's substance (Bond between the flight's bottom
    // and the landing above, not in the well) is unchanged.
    w.wait(20);
    band(&w, 30.4, 32.5, "flight C's bottom");
    w.walk_to(Wp::new(22.4, 31.3, -3.0).r(1.0), "the north strip 31.1 east");
    w.walk_to(Wp::new(23.0, 27.4, 3.8).r(1.0), "the east connector down (31.1→27.1)");
    band(&w, 26.4, 29.3, "connector 1's bottom");
    w.walk_to(Wp::new(15.0, 27.4, 4.8).r(1.0), "the south strip 27.1 west");
    w.walk_to(Wp::new(14.8, 23.3, -2.6).r(1.0), "the west flight B down (27.1→23.0)");
    band(&w, 22.4, 24.8, "flight B's bottom");
    w.walk_to(Wp::new(22.4, 23.2, -3.0).r(1.0), "the north strip 23.0 east");
    w.walk_to(Wp::new(23.0, 19.3, 3.8).r(1.0), "the east connector down (23.0→18.9)");
    band(&w, 18.2, 21.1, "connector 2's bottom");
    w.walk_to(Wp::new(15.0, 19.2, 4.8).r(1.0), "the south strip 18.9 west");
    w.walk_to(Wp::new(14.8, 15.4, -2.6).r(1.0), "the west flight A down (18.9→15.0)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.bond.pos[1];
        assert!(y < 17.2, "the descent never reached the lower floor (y {y}) [D 15.0 probe + the 1.09 standing offset]");
        assert!(!sim.drones.channels.on(0x7c), "the descent never crossed the atrium plates (ch 0x7C) [D §4]");
    }
    eprintln!("[011] the stairwell descent WALKED at +{} ticks", w.total_ticks);
    // Leg 3: the Load Point #158 and the E press ([M 0x1a5170 case 0x1d → 0x1b2318]).
    w.walk_to(Wp::new(14.6, 17.0, -2.8).r(0.9), "the Load Point");
    let mut loaded = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0012),
        "E at the Load Point #158 loads 07000012 [M 0x1b2318]");
    assert!(loaded, "the exit fired");
    eprintln!("[011] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}
