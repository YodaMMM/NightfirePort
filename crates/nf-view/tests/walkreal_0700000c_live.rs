//! walkreal-c: 0700000c (Chain Reaction, part 1 — the power station) walked with the player's own
//! movement — `step_bond` input only: no channel or state pokes. The REAL legs this session:
//! the laser-watch hinge burn (the four #139-142 locks through the held-fire path), the vent
//! drop firing plate #274 mid-fall (objective 1 on the game's own route), the 11.27 hall +
//! the upper east lane south, the yard reached WEST through the open secret door, and the three wire
//! crossings (jump-grab + hand-over-hand rides), and the STAIRCASE descent to the display
//! ground (the hall floor's own opening over `obj_stor2_staircase01` #359's ramp [D] — the
//! display eye reached by real input). The DOCUMENTED stand-ins left (each with its evidence
//! inline): the case's trench exit (the vent shaft is a ONE-WAY DROP in the disc's own data:
//! the south crawl pocket is sealed by the wall slab z −112.34..−112.363 and the crouch
//! capsule stops ON it at −111.813 [D this session's block dump + M 0x194df0 case 3]; the
//! game holds the body too [L 2026-10-26 hatch-3]; no breakable glass exists [D census]), the
//! plates reach (every
//! walking route off the display-ground complex is closed — the slab's south face leaves a
//! 0.23..0.4 slot above the 2.37 deck [D probe rays]; the game's own connection is the
//! warehouse-side corridor/annex, a live check), the east
//! climb ground→band (PINNED an island, level-0c-east-climb: the disc's own collision connects
//! NOTHING to the band deck — staircase01's bottom 9.52 floats over the 6.35 under-stairs [D
//! eastclimb3/4], the turb4 stair chain's slot x 81.5..83.7 is EMPTY 6.5..9.3 [D eastclimb5/6],
//! the band strip gaps 10.4 m over the chasm, the north-strip mezzanine breaks at x 54..64.5,
//! the crates4 gap is 3.1 m + 2.06 [D eastclimb6/7]; the deck is unreachable in the flood-fill
//! at step law 0.55 AND 0.80 [D eastclimb7]; no lift kind, no east-side mover, west-only
//! ladders/grapples [D kind census]; the nav patrols G3/G4 are separate [D nav dump]; the
//! stand-in lifts Bond onto the band at the gate — the live checks decide whether the real
//! game has a dynamic the parse cannot see), and the ladder495 sister test's west-ground placement. Route: the ST-1
//! walkway start #1 (8.98, 15.07, −149.65 [D]) → north along the walkway (14.61 floor [D probe blk],
//! east of the raised block x 5..10 [D probe0c-lane]) → the inner balcony repro spot (7.61, 15.71,
//! −113.39 [L 991]) → the LASER-WATCH burn of the four hinge locks #139-142 through the held-fire
//! path [M Lock_Update 0x1ae3d8:82-101 + Player_WeaponFiring 0x1a5460:653-658] → the AND #505 →
//! ch 18 sweeps the vent open (SS #138/#179, the y-16.3 ray the end-pose pin) → through the open
//! vent into the display-case trench (x 5..6.5, z −110.5..−111.5, floor 11.84 [D probe blk]) →
//! the fall crosses plate #274 (4.63..6.95, 13.63..14.56, −112.08..−110.12 [D]; the game fires it
//! while the body's volume crosses and clears at rest [L 2026-10-26 hatch-2]) → ch 0x1b (objective 1)
//! → the lower hall east lane south to the STAIRCASE HEAD — the hall floor's own opening
//! (x 9..12.4, z −115.5..−131.5 [D probe rays]) crossed by `obj_stor2_staircase01` #359's ramp
//! (the landing 11.12..11.29 at z −127..−128, down to 6.29 at z −120 [D probe rays; the kind-0x19
//! mesh bounds x 9.97..12.29, y 6.13..12.14, z −127.5..−117.1]) → the ramp descent to the
//! display-ground level (6.29 bottom, x 10.5..12.3, z −119.5..−121) → the south ledge (6.27,
//! x 12.6..14.5 [D probe rays]) north to the display pocket → the REAL camera photograph (0x54
//! [M 0x1a7ef8 case 0x700000c]) into the CamSubject #454 volume → ch 0x1c (objective 2) → the
//! DOCUMENTED plates-reach stand-in (the walking routes off the display-ground complex are
//! all closed — see Leg 4; this session also mapped the far side: the plates stand in a
//! ground-level corridor (0.90) running north under the warehouse mezzanine, whose only
//! proven entrances are the secret door and an unproven north end) onto the secret-door plates
//! #558/#559 (52.24..53.29, 0.90..3.20, −174.45..−172.73 [D]): the REAL body-touch + E-arm AND chain
//! → Door_secret1 #544 sweeps (rate 400 [D]) → through the open door WEST for real onto the
//! yard (0.69 [D this session's scan]) — the master's yard-descent stand-in is GONE — → the yard ladder #164 (9.67, 5.53, −187.34 [D])
//! → the REAL walk-in grab + climb (the walk substate rides `Player_Collision` 0x194df0's DEFAULT
//! hittest arm r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]) → the patrol-ring diagonal walkway
//! (the ladder map: (6-9,−188) → (12-13,−194) [D probes]) → the ring north-west corner (12,−202..−208)
//! → west edge (0-8,−208) → the REAL west-gap wire crossing: the jump grab at the type-3 icon #4
//! (−3.30, 12.36, −207.51 [D icons]) onto `wire_firstsilo2mainsilos` #284 (grab A (−4.63, 13.67,
//! −209.88) B (−8.47, 13.67, −216.52) [D probe blk], hand-over-hand [M Player_CollWire 0x197130 +
//! Player_Wire 0x197658; the port's wire.rs]) → drop at B onto the west ledge (11.24-11.38, x −12..−8,
//! z −216..−218 [D probe blk2 west map]) → the diagonal ledges south-east (11.24: (−8,−219) → (−2,−223)
//! [D]) → the jump grab at icon #6 (−1.54, −223.57 [D]) onto `wire_silosilo2` #282 (A (5.85, 14.11,
//! −223.15) B (−0.25, 14.11, −223.15) [D probe blk]) → ride east to A, drop onto the (4..8, −221..−226)
//! ledge (11.24 [D]) → the ring south-west arc (12-28,−218) → the jump grab at icon #12 (28.42, 12.24,
//! −218.43 [D]) onto `wire_silo2turbine` #283 (grab A (29.58, 14.44, −216.96) B (35.89, 14.44, −209.44)
//! [D probe blk; the body sphere-HIT at (28, 12.3, −217.5) confirmed]) → ride to B, drop onto the ring
//! SE corner (11.09..11.17, x ≤ 40.5, z −210..−208 [D]) → the REAL door2turbine #281 E press [M
//! 0x135be0/0x135618] through to the corner → the REAL jump over the x 41..43 slot onto the gate band
//! → the band TouchOnce #0xb4 (45.68..46.66, 10.34..11.32, −208.63..−206.52 [D]) → the band east to
//! its true end x 55.0 [D probe door4 fine map] → the band crossing is REPLACED by the game's
//! own ground route, walked on real input (the eastroute legs, green on master): turb2's
//! staircase flights down [D band9 plc20-22], the y-0 yard east under the chasm [D the nav
//! kind-0x202 ring path], the running jump onto the turb4 lower ramp [D the 0.25-grid step
//! scan], up to the east ground 6.17 [D plc42] — the straight crossing is dead ([L 2026-10-30
//! fresh boot: Bond drops into the pit] + the void sisters below) → the east climb STAND-IN (the
//! only unproven link: the deck is a collision ISLAND — the disc's static data connects nothing;
//! staircase01's bottom step 9.52 floats over the 6.35 under-stairs at x 84.5 [D eastclimb3/4],
//! the stair slot x 81.5..83.7 is empty 6.5..9.3 [D eastclimb5/6], the band's south rail 11.17
//! walls the deck [D eastclimb3], the 0.78 jump apex [M player.rs]; sisters 5b/5c pin the slot
//! and the three remaining crossing attempts) lifting Bond onto the band
//! at (95, −209.5), deck 10.34 [D plc44] → the gate steps → E → level_exit 0x0700000d ([M Player_Activate
//! 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318:111], the key gate ch 0x1c on) → the warm RamSave
//! carry into 0700000d → the spawn TouchOnce #436 (179.53, 11.58, −205.40, out 0x23 [D 0d]) fires
//! → objective 3 completes and 0d's own objective reveals ([M Mission_Init 0x182270:92-121 carries
//! record 11 across the load]).
//!
//! Legs 1-9c (the spawn asserts through the slot jump's band landing) live in
//! common/walkreal0c.rs as `walk_spawn_to_the_band` — moved verbatim from this file (same
//! load, same ticks, same order) and shared with walkreal_0700000c_eastroute_live, which now
//! replays the game's own route to the band west end instead of teleporting to it. The two
//! documented stand-ins inside (the trench exit, the mezzanine reach) moved with it, reason
//! and sisters intact.

#[path = "common/walkreal0c.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn walk_the_whole_mission_from_the_start_to_the_0d_handoff() {
    let iso = iso();
    // Legs 1-9c (the spawn asserts through the slot jump's band landing) are the shared
    // prologue `walk_spawn_to_the_band` (common/walkreal0c.rs) — moved verbatim, same
    // ticks, so the eastroute test can replay the game's own route to the band instead
    // of teleporting to it.
    let mut w = walker::walk_spawn_to_the_band(&iso);

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
    // extents]. The supportbeams are columns (turb2 pair x 53.57..54.57 y 14.25..18.03; turb3
    // #30 x 68.02..68.82 y 6.08..17.24 — 0.8 wide, no deck [D band9]). The 10.85/10.17 strip
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
    // master), then the ONLY stand-in left: the east climb ground→band.
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
        assert!(under.is_none(), "a floor appeared under staircase01's west end (8.7..6.7 at (84.5, −210.25)); the east climb reopened");
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
    // Sister 5c (the band deck is an ISLAND — the flood-fill over the sim's own collision, 0.25
    // grid, directed edges, step law 0.55 AND 0.80 [S the law itself; the island holds at both]):
    // the gate deck + staircase01's deck form one component unreachable from the east ground, the
    // yard, the west band, the corridor, the gallery and the catwalks [D eastclimb7 this
    // session]. The remaining crossing attempts, each beyond the pinned apex:
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
    // Sister 6: the gate does NOT fire from the ground (the east-route report's probe: Bond at
    // (95.0, 7.3, −209.5), 120 use pulses — no level_exit): the trigger quad is at y 11.25..13.98
    // [D plc556]; 120 real E pulses pin the negative on this tree.
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), true); }
    assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(),
        "the gate #556 fired from the ground — the climb stand-in's evidence changed");
    // STAND-IN (now ONLY the east climb): lift Bond from the east ground onto the band deck
    // above the same spot — obj_turb5_walkway02's deck 10.34 at (95, −209.5) [D plc44 x
    // 92.38..95.31, y 10.16..11.17, z −227.26..−206.31; the scan z −209.5 x 95.0 = 10.34 [D
    // eastclimb3]]. The deck is a COLLISION ISLAND pinned by sisters 5b/5c: the disc's own
    // static data connects NOTHING to it (the turb4 stair flights are disjoint with an empty
    // slot between; the band strip gaps 10.4 m over the chasm; the mezzanine strips break; the
    // crates4 gap is 3.1 m + 2.06), no mover serves it (no lift kind on 0c [D kind census]; the
    // 8 SSs, 2 doors, 2 ScriptPlayers are west-side [D audit §6]; ladders #164/#495 west [D];
    // the five graple_pulley points elsewhere [D #308/346/500/508/511]), and the game's own nav
    // patrols the east band (G3, 10.34) and the east ground (G4, 6.17) as SEPARATE paths with
    // no links [D the 0c nav dump]. Live checks in the report settle whether the REAL game has
    // a dynamic the parse cannot see.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [95.0, 11.3, -209.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(14);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let under = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
            .map(|(t, _)| p[1] - 3.0 * t);
        assert!(sim.bond.grounded && under.map_or(false, |y| (10.2..10.6).contains(&y)),
            "the climb stand-in must land on the band deck at (95, −209.5) (10.34 [D plc44]), got pos {p:?} under {under:?}");
    }
    eprintln!("[0c] the ground route walked + the climb stand-in — on the band at the gate steps");
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
    let mut w2 = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, Some(&save)).unwrap());
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

