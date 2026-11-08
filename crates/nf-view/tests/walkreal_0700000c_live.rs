//! walkreal-c: 0700000c (Chain Reaction, part 1: the power station) walked with the player's own
//! movement. The plain walk has no position, channel or state writes after the load. Route: the
//! ST-1 walkway start #1 (8.98, 15.07, -149.65 [D]) north to the inner balcony, the laser-watch
//! burn of the four hinge locks #139-142 [M Lock_Update 0x1ae3d8:82-101], AND #505 raises ch 18 and
//! the vent sweeps open, the drop into the display vent crosses plate #274 (ch 0x1b, objective 1),
//! the crouched crawl out through the one-way pane #457 into the hall, the staircase #359 down to
//! the display ground, the camera photograph into CamSubject #454 (ch 0x1c, objective 2), the east
//! ledge and the 6.29 stair to the 2.37 deck, the ramp out of ST-1, the x 46 lane south to the
//! yard, the ladder #164, the patrol ring, the three wire crossings #284/#282/#283, door2turbine
//! #281's E press, the slot jump onto the gate band, the turb2 stairs down, the yard east, the
//! jump-hops onto the east ramp, the east ground, the two crate hops, the south wall, up the
//! sloped girder #30, the sidestep onto the deck, E at the gate #556 (key ch 0x1c on) ->
//! 0700000d, and the warm carry into the 0d start plate #436 (ch 0x23, objective 3).
//!
//! The real-health walk (`with_yard_aimed_stands`) keeps two placements: its fights are a tuned
//! sequence through the mezzanine, the annex vent and the secret door (`mezzanine_door_leg`), and
//! no health, armour or ammo is written.
//!
//! Legs up to the band live in common/walkreal0c.rs (`walk_spawn_to_the_band`), shared with
//! walkreal_0700000c_eastroute_live.

#[path = "common/walkreal0c.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn sim_grounded(w: &Walker) -> bool { w.scene.sim.as_ref().unwrap().bond.grounded }

#[test]
fn walk_the_whole_mission_from_the_start_to_the_0d_handoff() {
    walk_the_whole_mission_from_the_start_to_the_0d_handoff_at(1);
}

/// The difficulty-parameterized body (real-health-agent-0c-0d): 1 is the byte-identical
/// master replay; 2+ (the Agent sisters) load the menu way — the front-end's launch path
/// [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty; GameState+0x28 0x2a3790 →
/// Drones::difficulty, asserted after each load]. Agent spawns the min-difficulty-2
/// placements [M Drone_Create 0x136be0:49], scales Bond's pain by Plr_DMod_Normal
/// [M Player_HandlePain 0x1902c8] and the guards' hit chance by Accuracy row 2
/// [M DroneWeap_DoBulletAccuracy 0x179740:74-88]; the route legs are untouched.
fn walk_the_whole_mission_from_the_start_to_the_0d_handoff_at(difficulty: u32) {
    let iso = iso();
    // Legs 1-9c (the spawn asserts through the slot jump's band landing) are the shared
    // prologue `walk_spawn_to_the_band` (common/walkreal0c.rs) — moved verbatim, same
    // ticks, so the eastroute test can replay the game's own route to the band instead
    // of teleporting to it.
    let mut w = walker::walk_spawn_to_the_band_at(&iso, difficulty);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");

    // Leg 10 (REAL): the band east to its true end. The disc's band floor 10.34 runs x 44..55.6
    // (z −209.06..−206.31: obj_turb1_walkway01 x 43.2..49.3 + obj_turb2_walkway01 x 49.3..56.1
    // [D band2 probe's tri dump]); x 56.1..66.3 at band height is VOID — the 6.17 pit (x 55..63.5),
    // the 7.19 shelf (x 60..64 at z −209.0 only), then a floor-less chasm x 64..66.5 down past
    // y 2 [D eastclimb probe's 0.5 map], to the resumed band 10.34 at x 67.5+ (obj_turb3_walkway04
    // [D]; the pit floor itself is obj_turb2_walkway04 plc28 x 49.29..63.66, y 6.16..7.19,
    // z −209.06..−205.12 [D band9 probe's placed extents], continuing east as the sliver
    // obj_turb3_walkway03 plc33 x 63.66..64.05 (y 6.17..7.19) — then the chasm x 64.05..66.27
    // down past y 2, and the resume sits 10.16+ — +3.2 ABOVE the pit floor: the pit route
    // dead-ends, no jump crosses up [D band9 extents vs the 5.2-unit carry]. The staircases are
    // symmetric FLANKS, not a crossing: turb2's rise 2.97→11.17 at x 49.29..58.73 and turb4's
    // 0.16→11.20 at x 78.14..87.59 (both z −208.96..−211.67, SOUTH of the band) — they connect
    // the yards and the gate ground to the band heights, nothing spans x 59..78 [D band9
    // extents]. The turb3 support girder #30 (x 68.02..68.82 y 6.08..17.24 [D band9]) is sloped and
    // walkable: the east climb below rides it. The 10.85/10.17 strip
    // (z −206.33..−205.12) is NO ROUTE: its west part x 49.3..53.57 ends in a 4-unit wall at
    // x 53.57 (y 10.85..14.85 [D lipline probe's tris]) — a cul-de-sac — and the east slope's
    // west tip (x 58.86 at 10.17 [D]) recedes to x 61.5 at the flight's z −205.7 (the tri
    // (58.86,−205.12)|(62.19,−205.34)|(63.32,−206.08) [D]): a 7.98-unit gap against the
    // engine's 5.2-unit drop-jump carry (3.92 m/s up, 9.8 gravity [M player.rs], 5.5 run — the
    // -2 session measured 4.42 flat) — IMPOSSIBLE. Nothing carries across: the
    // full leaf-coverage audit decodes EVERY triangle the disc declares (0 gaps in all 46 maps
    // [D collaudit probe]) and none lands at band height over the void; no SS (all 9 listed:
    // two display vents, two warehouse vents, three storeroom pulleys, Door_secret1, and the
    // VentmeshBlockingPloy [D band7 probe]), no wire (4 total: three ring wires #282/#283/#284
    // + the type-2 zipline x 46.79 y 24.6→11.6 z −207.0→−182.7 — a NORTH-yard ride [D band7/
    // grapple probes]), no grapple point (5 total, none over the void: the near one gp4 is at
    // (54.27, 15.35, −237.02) — the south yard [D grapple probe]), no script object (the
    // PhoenixForkLift SP #341 script 06000762 drives the yard at y 0, x 21..72.25, z −205..−181
    // [D band6 probe's 2400-tick track]), no door, no drone movement in the box. The catwalk
    // (15.53..15.96) is CONTINUOUS x 40.7..79.6 [D catwalk probe] but has no found access (no
    // ladder within 40 units — the level's three ladders are the origin junk, the yard #164 and
    // the (−6.4, −177.4) one [D]). The game's own nav graph links band-height nodes ACROSS the
    // void (path17 n0 (63.06, 10.74, −207.54) ↔ n1 (49.69, 10.74, −207.29) [D probe door4]) and
    // a guard is PLACED mid-void at band height ((62.99, 11.42, −207.43) plc60 [D]) — the game's
    // own data places a guard spawn mid-void — but the link is NOT floor evidence: the game's
    // node-to-node move test sweeps the body from node to node [M NDrone2_MoveTest 0x155330,
    // the port's mirror `nav::move_test`; its gates are |dy| ≤ 4 and the room boundary
    // [M 0x155330 via drone.rs] — and the game's AI is per-path (NDrone2_AssignAIPath assigns
    // ONE path per drone [M 00154190]; AINetwork_CalcRoute walks the assigned path's nodes
    // [M 00156ba8])], and a horizontal sweep across a HOLE hits no wall and no floor — it
    // passes trivially. A designer-linked pair over a pit tests clean; the guard plc60 spawn
    // (62.99, 11.42, −207.43 [D band8 probe]) pins a placement, not a walkable surface. The
    // port's static world equals the disc's declared collision exactly [D collaudit probe], so
    // the port side has nothing to fix in the parse: the crossing mechanism stays UNPINNED by
    // everything the disc declares (live check 1 below — the fresh-boot PCSX2 watch decides);
    // the walk takes the game's own ground route around (the legs below).
    w.walk_to(Wp::new(50.0, 11.0, -207.6).r(1.5), "the band east 1");
    w.walk_to(Wp::new(54.5, 11.0, -207.6).r(0.8), "the band east to its true end x 55.0 [D probe door4]");
    // THE CROSSING IS REPLACED (level-0c-whole-walk-east-route): the sisters below keep the
    // crossing's negative evidence fresh (the straight crossing is dead — [L 2026-10-30 fresh
    // boot: Bond drops into the pit] — so the walk takes the game's own ground route around);
    // then the proven ground-route legs (the walkreal_0700000c_eastroute_live legs, green on
    // master), then the east climb ground→band on the sloped girder.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister 1: the band-height void over the pit/chasm (the pit floor 6.17 [D probe door4])
        // must stay BELOW band height at (59, −207.5).
        let gap = sim.world.segment_hit([59.0, 12.0, -207.5], [59.0, 8.0, -207.5]);
        assert!(gap.is_none(), "the band crossing gained floor at band height; the ground route's reason vanished");
        // Sister 2: the strip cul-de-sac wall (x 53.57, y 10.85..14.85, z −206.33..−205.12 [D
        // lipline]) must stand — if it ever vanishes the strip route reopens and the route map
        // changes. (A horizontal ray — the wall is an x-constant plane.)
        let wall = sim.world.segment_hit([53.0, 12.0, -205.7], [54.2, 12.0, -205.7]);
        assert!(wall.is_some(), "the strip's east wall (x 53.57 [D lipline]) vanished; re-check the crossing route");
        // Sister 3: the pit floor's presence and height (obj_turb2_walkway04 plc28 x 49.29..63.66,
        // y 6.16..7.19 [D band9 extents]) — the walked descent's landing floor (the flight's
        // south end). If it moves the descent legs re-derive.
        let pit = sim.world.segment_hit([59.0, 8.0, -207.5], [59.0, 5.5, -207.5]).map(|(t, _)| 8.0 - 2.5 * t);
        assert!(pit.map_or(false, |y| (5.8..7.6).contains(&y)),
            "the pit floor at (59, −207.5) moved (expected 6.16..7.19 [D band9 plc28]), got {pit:?}");
        // Sister 4: the pit's east sliver (obj_turb3_walkway03 plc33 x 63.66..64.05, y 6.17..7.19
        // [D band9 extents]) — the pit's far end before the chasm x 64.05..66.27.
        let sliver = sim.world.segment_hit([63.9, 8.5, -207.5], [63.9, 5.5, -207.5]).map(|(t, _)| 8.5 - 3.0 * t);
        assert!(sliver.map_or(false, |y| (6.0..7.5).contains(&y)),
            "the pit's east sliver at (63.9, −207.5) moved (expected 6.17..7.19 [D band9 plc33]), got {sliver:?}");
    }
    eprintln!("[0c] the crossing sisters hold — taking the game's own ground route around the band");
    // The ground route (the eastroute legs on the walk's own arrival):
    // 1. The staircase descent (the turb2 flights, real walk legs [D band9 plc20-22]).
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
    w.walk_to(Wp::new(77.0, 1.2, -210.2).r(1.0), "the yard south of the ramp's west landing");
    w.walk_to(Wp::new(79.6, 1.1, -210.2).r(0.5), "the due-east run-up under the ramp's west landing (the eastroute jump start)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(p[0] > 77.0 && p[0] < 80.0 && f.map_or(false, |y| y < 0.5),
            "the yard trunk must reach the ramp's west side at the y-0 floor, got {p:?} under {f:?}");
    }
    // 3. The jump-hops onto the east ramp (1.75 at x 81.5, under-steps 0.75..1.55 at x
    // 80.2..81.2 [D the step scan]) — the player's own idiom: hold forward, jump every time the
    // body is grounded below the exposed steps, stop on them (the walk's slot-jump family).
    let mut hops = 0u32;
    for _t in 0..300u32 {
        let (p, grounded, f) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let (p, grounded) = (sim.bond.pos, sim.bond.grounded);
            let f = if grounded {
                sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
                    .map(|(tt, _)| p[1] - 3.0 * tt)
            } else { None };
            (p, grounded, f)
        };
        if grounded && p[0] > 80.3 && f.map_or(false, |y| y > 1.0) { break; }
        let jump = grounded && p[0] >= 80.0 && !f.map_or(false, |y| y > 1.0);
        if jump { hops += 1; }
        w.tick(nf_game::player::Input { forward: 1.0, jump, ..Default::default() }, false);
    }
    assert!(hops > 0, "the ramp jump never fired");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(p[0] > 80.3 && f.map_or(false, |y| y > 1.0),
            "the jump must land on the ramp's lower steps (1.75+ [D]), got {p:?} under {f:?}");
    }
    // 4. Up the ramp (0.2 steps) to the east ground (obj_turb4_walkway03 y 6.16..7.19 [D plc42]).
    w.walk_to(Wp::new(84.0, 4.5, -210.2).r(1.0), "up the ramp");
    w.walk_to(Wp::new(87.0, 7.0, -209.8).r(1.0), "the ramp top");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(f.map_or(false, |y| (5.5..7.4).contains(&y)),
            "the ramp top must stand on the east ground level (6.16..7.19 [D plc42]), got {p:?} under {f:?}");
    }
    // The east side reached: west along the east ground past the chasm's east edge (the
    // eastroute test's closing leg), then east under the band to the gate's ground spot.
    w.walk_to(Wp::new(86.0, 7.4, -208.2).r(1.0), "the east ground west");
    w.walk_to(Wp::new(78.0, 7.4, -208.2).r(1.2), "the east ground at the chasm's east");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let f = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 20.0, p[2]])
            .map(|(t, _)| p[1] - 20.0 * t);
        assert!(sim.bond.grounded && p[0] > 76.0 && f.map_or(false, |y| (6.0..7.4).contains(&y)),
            "the east ground leg must hold (6.17 [D]), got {p:?} under {f:?}");
    }
    // Sister 5 (the east climb's negative, at the climb zone): the upper flight's bottom step
    // FLOATS — obj_turb4_staircase01's extents y 9.00..11.20, x 83.47..87.59 [D plc35]; the
    // all-surfaces scan gives its bottom 9.52..9.68 at x 84.2..84.5 over the under-stair steps'
    // 6.35 top at x 87.0 [D eastclimb3/4 this session]; nothing spans 6.35..9.52 and the jump
    // apex is 0.78 [M player.rs 3.92 m/s up]. A floor appearing in the gap reopens the climb.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let under = sim.world.segment_hit([84.5, 8.7, -210.25], [84.5, 6.7, -210.25]);
        assert!(under.is_none(), "a floor appeared under staircase01's west end (8.7..6.7 at (84.5, −210.25))");
        let bottom = sim.world.segment_hit([84.5, 11.4, -210.25], [84.5, 8.7, -210.25]).map(|(t, _)| 11.4 - 2.7 * t);
        assert!(bottom.map_or(false, |y| (9.3..9.9).contains(&y)),
            "staircase01's bottom step moved (expected 9.52..9.68 [D eastclimb3]), got {bottom:?}");
    }
    // Sister 5b (level-0c-east-climb: the stair SLOT — the decisive missing link of the climb
    // chain). The turb4 flights are horizontally disjoint: staircase02 (plc36) tops at 8.10 at
    // x 81.3..81.4 and staircase01 (plc35) starts at 9.45 at x 83.8..84.4 [D the eastclimb5
    // level census this session]; between them (x 81.5..83.7, z −211..−209.5) the slot is EMPTY
    // down to the lower flight's steps 1.75..3.75 [D the eastclimb6 fine map]. Crossing needs a
    // 2.4 m jump that GAINS 1.35 — beyond the 0.78 apex [M player.rs]. A surface appearing in
    // the slot band reopens the climb.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for x in [81.75f32, 82.25, 82.75, 83.25] {
            let slot = sim.world.segment_hit([x, 9.3, -210.0], [x, 6.5, -210.0]);
            assert!(slot.is_none(), "a surface appeared in the stair slot at ({x}, −210.0) 9.3..6.5; the east climb reopened");
        }
        let top36 = sim.world.segment_hit([81.35, 8.5, -211.25], [81.35, 7.6, -211.25]).map(|(t, _)| 8.5 - 0.9 * t);
        assert!(top36.map_or(false, |y| (7.9..8.3).contains(&y)),
            "staircase02's top step moved (expected 8.10 [D eastclimb5]), got {top36:?}");
    }
    // Sister 5c: no walked surface joins the deck to the ground; only the girder #30 and a drop
    // off it does. The crossings that stay closed, each beyond the jump apex:
    // (a) the west band's east end 56.08 → turb3_walkway04's deck 66.5 — 10.4 m at deck height
    //     (the [L 2026-10-30 fresh boot] drop at x 63.4 already pinned the walk-off);
    // (b) the north-strip mezzanine (10.85, z −206.0..−205.1) breaks at x 54..64.5 [D eastclimb6
    //     row −206.0: 6.17 ground, no 10.85 floor];
    // (c) crates4 (tops 7.31/8.31) → walkway02's dead south end 10.37 — 3.1 m + 2.06 up.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // (a) widened: the whole band-height strip over the chasm's west half.
        for x in [57.0f32, 59.0, 61.0, 63.0, 65.0] {
            let gap = sim.world.segment_hit([x, 11.9, -207.5], [x, 8.0, -207.5]);
            assert!(gap.is_none(), "band-height floor appeared over the chasm at ({x}, −207.5); the east climb reopened");
        }
        // (b) the north strip's break: the west mezzanine's 10.85 ends at x 53.5 and the next
        //     north-strip surface is room02's isolated 10.17 sliver [58.9..63.3, −206.1..−205.1
        //     [D eastclimb5 plc9]] — nothing spans x 55..58.5 at 9.5..11.5, so the north route
        //     never bridges the chasm (the sliver itself is an island: 5+ m to the west patch,
        //     and the flood at 0.80 leaves it unconnected [D eastclimb7]).
        for x in [55.0f32, 56.5, 58.0] {
            let gap = sim.world.segment_hit([x, 11.5, -205.7], [x, 9.5, -205.7]);
            assert!(gap.is_none(), "north-strip mezzanine floor appeared at ({x}, −205.7) 9.5..11.5; the east climb reopened");
        }
        // (c) the crates4 air gap: no floor 9.4..10.2 at z −229.0/−228.0, x 93..95.
        for z in [-229.0f32, -228.0] {
            for x in [93.0f32, 94.0, 95.0] {
                let gap = sim.world.segment_hit([x, 10.2, z], [x, 9.4, z]);
                assert!(gap.is_none(), "a surface appeared in the crates4 air gap at ({x}, {z}) 9.4..10.2; the east climb reopened");
            }
        }
    }
    w.walk_to(Wp::new(95.0, 7.3, -209.5).r(1.2), "the gate's ground spot under the band");
    // Sister 6: the gate does NOT fire from the ground (Bond at
    // (95.0, 7.3, −209.5), 120 use pulses — no level_exit): the trigger quad is at y 11.25..13.98
    // [D plc556]; 120 real E pulses pin the negative on this tree.
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), true); }
    assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(),
        "the gate #556 fired from the ground");
    // THE EAST CLIMB, WALKED: the sloped support girder `obj_turb3_supportbeam02` #30 (x 68.02..68.82,
    // foot 6.08 at z -238.4, head 17.24 at z -205.6 [D plc30]) is the climb. It is 0.8 wide and
    // walkable: its top rises 0.6 per unit for the first three units, then 0.31 [D the surface
    // scan], and the body rides it on stick input alone. From the east ground the walk goes
    // south over the crate stack, west along the south wall to the girder's foot,
    // up it to the overhead wall at y ~18, and sidesteps east off the head: the fall carries it
    // north under that wall onto the 10.85 strip (z -206.33..-205.12), which steps down to the
    // deck 10.34. Nothing here writes Bond's position.
    w.walk_to(Wp::new(94.3, 7.2, -222.0).r(0.8), "the east ground south");
    w.walk_to(Wp::new(94.3, 7.2, -228.5).r(0.4), "the crate stack's north side");
    // The crate stack (tops 7.31 then 8.31 [D the surface scan]) closes the strip: two running
    // hops take it, and the far side drops to the south wall's ground.
    for _t in 0..240u32 {
        let (p, yaw, grounded) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.bond.yaw, sim.bond.grounded) };
        if p[2] < -236.5 && grounded { break; }
        let err = walker::wrap_pi((94.3 - p[0]).atan2(-5.0) - yaw);
        let jump = grounded && p[2] < -229.8 && p[2] > -235.0;
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, jump, ..Default::default() }, false);
    }
    assert!(w.scene.sim.as_ref().unwrap().bond.pos[2] < -236.0,
        "the hops must clear the crate stack, got {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w.walk_to(Wp::new(71.0, 7.2, -236.6).r(1.0), "west along the south wall");
    w.walk_to(Wp::new(69.6, 7.2, -238.8).r(0.5), "the girder foot's east side");
    w.walk_to(Wp::new(68.45, 7.2, -238.9).r(0.3), "in line with the girder");
    let (mut topped, mut off) = (false, false);
    for _t in 0..900u32 {
        let (p, yaw) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.bond.yaw) };
        if p[2] > -209.6 { topped = true; }
        if topped && p[1] < 12.5 && sim_grounded(&w) { off = true; break; }
        let err = walker::wrap_pi((68.45 - p[0]).atan2(5.0) - yaw);
        let input = nf_game::player::Input {
            turn: if p[2] < -212.0 { err.clamp(-0.08, 0.08) } else { 0.0 },
            forward: if err.abs() > 0.1 && !topped { 0.0 } else { 1.0 },
            strafe: if topped { 1.0 } else { 0.0 },
            ..Default::default()
        };
        w.tick(input, false);
    }
    assert!(topped && off, "the girder climb must reach the head and land on the strip, got {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 70.0 && p[2] > -209.0 && (11.0..12.5).contains(&p[1]),
            "the sidestep off the girder head lands at deck height north of the wall, got {p:?}");
    }
    w.walk_to(Wp::new(84.0, 11.3, -207.4).r(1.0), "east along the deck");
    w.walk_to(Wp::new(95.0, 11.3, -209.5).r(0.8), "the deck to the gate steps");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let under = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
            .map(|(t, _)| p[1] - 3.0 * t);
        assert!(sim.bond.grounded && under.map_or(false, |y| (10.2..10.6).contains(&y)),
            "the walk must reach the band deck at (95, -209.5) (10.34 [D plc44]), got pos {p:?} under {under:?}");
    }
    eprintln!("[0c] the ground route and the girder climb walked — on the band at the gate steps");
    w.walk_to(Wp::new(96.8, 11.2, -209.6).r(0.4), "the gate plane");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0c] at the gate: alarm chs 0x60-63 = {} {} {} {}, key ch 0x1c = {}, mission state 11 = {:?}",
            sim.drones.channels.on(0x60), sim.drones.channels.on(0x61), sim.drones.channels.on(0x62), sim.drones.channels.on(0x63),
            sim.drones.channels.on(0x1c), sim.drones.mission.states.get(11).map(|s| s.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")));
    }
    // The E press: USE-activated LOAD_LEVEL [M 0x1a5170 case 0x1d → 0x1b2318:111].
    let mut loaded = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    assert!(loaded, "the gate #556 loads 0700000d (key ch 0x1c on) [D; M 0x1b2318:111]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_000d), "the hand-off target [D]");
    eprintln!("[0c] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());

    // Leg 11 (REAL hand-off): the warm RamSave carry into 0700000d — the game's own level
    // transition ([M ResetMap_LevelToLoad 0x1b2318:111 → the RamSave is the port's record of the
    // carried channels/inventory; Mission_Init 0x182270:92-121 carries record 11 across the
    // load]). On 0d, the spawn TouchOnce #436 (179.53, 11.58, −205.40, out 0x23, mask 8 [D 0d])
    // sits on the arrival start and fires ch 0x23 under Bond: objective 3 ("Advance through
    // Warehouse TR-2") completes and 0d's first objective reveals.
    let save = {
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    // The warm reload runs the same difficulty the 0c legs were played at (the game's
    // difficulty global survives the level change untouched; the port re-seeds it at build
    // [M nf-view lib.rs load_with_carry's carry comment] — 2+ pins the menu pick the same way
    // the cold load did).
    let mut w2 = Walker::new(if difficulty == 1 {
        nf_view::load_with_carry(&iso, "0700000d.bin", false, Some(&save)).unwrap()
    } else {
        nf_view::load_with_carry_difficulty(&iso, "0700000d.bin", false, Some(&save), difficulty).unwrap()
    });
    assert_eq!(w2.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the 0d reload runs the picked difficulty [M 0x182270:92-121 + GameState+0x28]");
    w2.wait_enabled(3000, "the 0d arrival");
    w2.walk_to(Wp::new(179.53, 11.6, -205.40).r(1.2), "onto the TR-2 arrival plate");
    w2.wait(30);
    {
        let sim = w2.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x23), "the 0d spawn TouchOnce #436 fires ch 0x23 [D #436]");
        assert_eq!(sim.drones.mission.states[11][2], 4, "objective 3 completes on the 0d arrival [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[12][0], 3, "ch 0x23 reveals 0d's first objective [L record 12 obj 0]");
    }
    eprintln!("[0c] objective 3 complete on the 0d arrival — the whole mission walked (0c legs {} ticks + 0d {} ticks)",
        w.total_ticks, w2.total_ticks);
}

/// THE WHOLE MISSION ON REAL HEALTH (`walker::walker::with_real_damage` — the b2 walker's
/// damage_log): the same walked route, the top-up off — no health, armour or ammo writes
/// anywhere — and Bond's death panics with the damage log (who, weapon, range). Auto-aim is
/// the app default; the fights are the walker's own plain return fire. The two documented
/// route stand-ins are links from the master walk (neither writes health, armour or ammo).
///
/// The yard's deck-sniper stands are OPT-IN (`walker::with_yard_aimed_stands`): the deck
/// snipers (weapon 0x1e, -16 a hit, 10.7-11 u above the yard floor) sit outside the plain
/// fights' 7.5 u vertical reach and were the real-health run's killer chain (#18 dealt 51.8
/// of 100 in 4 hits from d ~34 u [L the predecessor run's census]; Bond died ON the ladder at
/// y 7.95). The stands kill them AIMED from the strip — the zero-spread aimed act
/// [M Bullet_init 0x12c5a0:139-170; the HIP cone 100.0 → 4.9 u of scatter at 35 u
/// [L ee_start_05.bin @ 0x2c10b8 +0x24] is the miss the player-upward-shot branch pinned] —
/// then the walk climbs and mantles. Real input only: the whole route is walked from the
/// level start, no teleport after the load; the top-up walk and the eastroute replay (flag
/// off) keep master's pinned choreography.
/// The yard's deck-sniper stands are OPT-IN (`walker::with_yard_aimed_stands`): the deck
/// snipers (weapon 0x1e, -16 a hit, 10.7-11 u above the yard floor) sit outside the plain
/// fights' 7.5 u vertical reach and were the real-health run's killer chain (#18 dealt 51.8
/// of 100 in 4 hits from d ~34 u [L the predecessor run's census]; Bond died ON the ladder at
/// y 7.95). level-0c-yard-climb-real-health-3's answer: the STILL-GATE aimed stand — the
/// zero-spread aimed act [M Bullet_init 0x12c5a0:139-170] has no lead, and the sentry's
/// wiggle-step during the round's flight is the measured miss (converged rounds never landed
/// while it stepped; the same spot landed the run it paused [L the probe on this branch]) —
/// so the stand taps only after the target holds still 18 ticks. The emplacement dies
/// MID-CROSSING (from the door it is not exposed at all — aim None [L the same probe]; the
/// -2 census's twin #36 is present false on this tree — the round-vs-twin theory is retired),
/// the pre-climb suppression pass stands on #15/#1, the ladder-top stands on the hitters.
/// Real input only: the whole route is walked from the level start, no teleport after the
/// load; the top-up walk and the eastroute replay (flag off) keep master's pinned
/// choreography.
/// level-0c-yard-climb-real-health-4 (GREEN — un-ignored): the -3 blocker (the west sniper
/// #13 eating the runs at tick 14308) was never the ledge lip — the session's shot trace
/// (NF_RF_TRACE + the player_impacts dump) showed the stands' rounds flying 2-3° OFF the
/// converged view and dying on the wall BEHIND #13 at (-4.20,12.58,-221.83) /
/// (-4.26,13.04,-221.78), because (a) the taps fired with the aim bit false (the raise never
/// ran: the aim-press edge fired into the switch anim, and the game queues the raise only on
/// the pressed edge from the idle machine [M 0x1a24c8:113-152]) so every round rode the HIP
/// cone (spread base 100.0 [L ee_start_05.bin @ 0x2c10b8 +0x24]), and (b) the port scanned
/// auto-aim and froze the walk-legs' assist offset through the aim hold, steering the first
/// aimed rounds -12° off (aoff -0.216) — the game eases the offsets toward the manual offsets
/// EVERY aim tick and scans only button-up ticks [M Player_AutoAim 0x1a7c98:50-59]. The port
/// fixes live in scene.rs (the per-tick aim-branch ease + the raw-button scan gate); the
/// stands gained the raise loop (settle, then hold aim until the bit sets), the aim-bit fire
/// gate and the cursor-aware convergence (the effective aim = the view + the eased cursor
/// offsets). Result: #13 dies AIMED from the west edge (part 4, t284 [L this session's
/// impact trace]), the #284 crossing + the ledge diagonals + the #282 ride + the 0d legs all
/// pass on real input, and the mission completes with Bond ALIVE at 26.2 hp ([L] the green
/// run: 'objective 3 complete on the 0d arrival — the whole mission walked (0c legs 19287
/// ticks + 0d 838 ticks)').
#[test]
fn walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage() {
    walker::with_yard_aimed_stands(|| {
        walker::with_far_sniper_stands(|| walker::walker::with_real_damage(walk_the_whole_mission_from_the_start_to_the_0d_handoff))
    });
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-0c-0d): the same walked
/// route and the same end, loaded at difficulty 2 the menu way — the front-end's launch
/// path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty; GameState+0x28 →
/// Drones::difficulty, asserted at load]. Agent spawns the min-difficulty-2 placements
/// [M Drone_Create 0x136be0:49], scales Bond's pain by Plr_DMod_Normal [M Player_HandlePain
/// 0x1902c8] and the guards' hit chance by Accuracy row 2 [M DroneWeap_DoBulletAccuracy
/// 0x179740:74-88]. Real input only; no health, ammo or channel writes.
///
/// IGNORED with the measured run (real-health-agent-0c-0d): the Operative real-damage walk is
/// byte-identical through tick 4054 (the same fights, the same landed hits — the shared RNG
/// draws pass both accuracy rows [M DroneWeap_DoBulletAccuracy 0x179740:74-88]) and finishes
/// the mission at 26.2 hp; the Agent rows (pain Plr_DMod_Normal, hit chance Accuracy row 2)
/// land the draws d1 dodges and Bond dies. Three lever rounds measured on this branch, each
/// deterministic:
/// - run 1 (master fights): DIED tick 4054 (12.96,12.35,-124.38) — 65 hits for 100.0; the
///   plain nearest-visible pick stood 900-tick fights on the display-ground guards whose own
///   boxes the hall slab eats (#27 twice, ticks 2707/3637, 26.6 + 29.6 health) while the
///   d 19.8 machine pistol #30 chewed 26.3 in 25 hits.
/// - run 2 (+lane fights, the 07000013 law): the fights WIN — #33/#27/#24/#30 all killed —
///   but each stand still pays the crossfire: DIED tick 6716 (7.18,7.40,-113.15, the display
///   ground) at 12.5 hp, the below-floor shooter #31 (y 3.2, d 26.3) unkillable from every
///   stand the route reaches (the down-rays die on the floor edge) chewing the last 12.5.
/// - run 3 (+escalating failed-fight cooldowns 240→1920): DIED tick 8579 (53.98,6.94,-163.03,
///   the warehouse plates) — the fights kept winning (#37 killed) but the one tick-5903 fight
///   on #30 paid 19.0 to the machine pistol's own cadence and the walk arrived at the plates
///   with 3.0. A 4th round (+dry_taps 3, faster abandonment) RESHUFFLED WORSE: DIED tick 4234
///   (the fight cycle re-engages sooner) — reverted.
/// NEXT LEVERS: a d2-gated staircase-head down-shot stand on the display-ground pair (the
/// 0c yard-stand law — the zero-spread aimed act from the stair head, where the game's own
/// hall-floor opening x 9..12.4 gives the line); an aimed stand on #31 from the display
/// pocket (the photograph leg's own spot); the shooter-side read (NF_RF_TRACE + NF_RF_GEOM on
/// the tick-5903 fight) to see whether the blocker hold or the slab eats the taps. The
/// Operative baseline this sister must match: 100.0 → 26.2 hp over 19287+838 ticks.
#[test]
#[ignore = "Agent: the fights win but the crossfire cadence out-bleeds the route — run 3 (lane fights + escalating cooldowns) died tick 8579 at the warehouse plates (3.0 hp left; the tick-5903 fight paid 19.0 to the d 18-20 machine pistols); run 1 (master fights) died tick 4054, run 2 tick 6716, run 4 (dry_taps) reshuffled worse, tick 4234; levers: a d2-gated stair-head down-shot stand on the display-ground pair, an aimed stand on #31 from the display pocket, the NF_RF_TRACE read of the blocked fights"]
fn walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage_agent() {
    walker::with_yard_aimed_stands(|| {
        walker::walker::with_real_damage(|| walk_the_whole_mission_from_the_start_to_the_0d_handoff_at(2))
    });
}


/// Sister (ladder-0c): the 0c's OTHER ladder, #495 `Ladder` (−6.42, 1.0, −177.39, yaw −0.2618,
/// mesh bounds x ±0.204 / y 0..2.0 / z degenerate [D placement #495 + its map object]) — the same
/// walk-in grab + climb through the real input path, on the flat west ground (floor ~0.0 [D
/// probes]; the small 1.936 ledge sits ~0.9 east of the plane). The placement below is a
/// MEASUREMENT SPOT, not a route stand-in: the test's subject is the ladder grab/climb classes
/// ([M 0x1953f4's DEFAULT arm + 0x196118/0x196310]) at #495's own plane, not the route to the
/// ladder — the walkreal route never crosses the yard's west edge (the walkreal's route stand-ins
/// are all documented in the header), so the only honest setup is Bond on the west ground beside
/// the ladder; the walk-in and the climb after it are real input.
#[test]
fn ladder495_walk_grab_and_climb_to_the_top() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-7.6, 1.1, -176.9];
        sim.bond.vel = [0.0; 3];
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister (the measurement spot's ground truth): the west ground's floor ~0.0 under the
        // spot [D probes] — if it moves, the measurement spot re-derives (the climb asserts
        // below key on the mesh's own top 2.0 [D] and climb_top 2.87 [M 0x196118], both
        // floor-independent, but the grab approach's walk-in starts here).
        let f = sim.world.segment_hit([-7.6, 1.1, -176.9], [-7.6, -2.0, -176.9])
            .map(|(t, _)| 1.1 - 3.1 * t);
        assert!(f.map_or(false, |y| (-0.3..0.4).contains(&y)),
            "the west ground under the #495 measurement spot moved (floor ~0.0 [D probes]), got {f:?}");
    }
    w.wait(20);
    eprintln!("[0c] the #495 stand-in — the west ground (floor ~0.0 [D probes])");
    // The walk-in: face the ladder and push (the same real-input shape as the yard leg).
    let mut climbed = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbed = true; break; }
        // yaw = atan2(dx, dz) — forward = (sin yaw, cos yaw) — [S] unverified (Bond::look is the port's look maths; the game's heading convention is not pinned here).
        let desired = (-6.422f32 - sim.bond.pos[0]).atan2(-177.391f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: if err.abs() > 0.2 { err.clamp(-0.08, 0.08) } else { 0.0 },
            forward: if err.abs() > 0.2 { 0.3 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        w.total_ticks += 1;
    }
    assert!(climbed,
        "walking into the #495 ladder never grabbed it — the walk substate rides the DEFAULT arm r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]");
    // Climb: hold forward; the mantle fires at climb_top = 1.0 + 1.0206 + 0.85 = 2.87 [M
    // 0x196118: `+0x84 + +0x8C + 0.85`]. The top has no landing (the ledge 1.936 is 0.9 east [D
    // probes]; the lunge falls back to the ground), so the pin is the climb passing the mesh's
    // own top (2.0 [D]) through the real path — the grab-snap lift [M 0x196118:23-31] starts him
    // at bottom + 0.02 = 1.0.
    let mut max_y = 0.0f32;
    let mut done = false;
    for _ in 0..1200 {
        let sim = w.scene.sim.as_mut().unwrap();
        max_y = max_y.max(sim.bond.pos[1]);
        if sim.bond.substate != nf_game::player::SS_CLIMB && sim.bond.pos[1] > 1.9 && max_y > 2.7 { done = true; break; }
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    assert!(max_y > 2.7, "the #495 climb never passed its top 2.87 [M 0x196118] — max y {max_y}");
    assert!(done, "the #495 climb never dismounted past its mesh top 2.0 [D]");
    eprintln!("[0c] the #495 ladder climbed for real to y {max_y:.2}");
}

