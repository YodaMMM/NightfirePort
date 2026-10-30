//! walkreal-a: 07000005 (Mayhew A, the castle approach) walked with the player's own movement — no
//! teleports after the spawn. Two routes (docs/levels/07000005.md):
//!
//! 1. THE TRUCK + GATE ROUTE: the cold start #26 (104.95, 25.12, 108.83) → walk to the gatehouse
//!    (the ch-0x5C TouchOnce at (98.49, 27.05, 106.36); the spawn's own ch-0x45 reveal plate fires
//!    under Bond) → the truck chain: SP #37's truck script drives up, SP #38 hands Bond aboard
//!    (t1884/t2028 [D]), the crouched ride parks on the ch-0x70 plate [D; fix-truck] → mux #316 →
//!    SP #320 → ch 0x1D → the gate door #118 loads 07000006.
//! 2. THE CABLE: the cold start → west along the courtyard → door #153 (57.75, 26.0, 89.31) →
//!    ladder #151 (60.93, 29.91, 89.20, top 34.7) → the upper ledge west → wire #251 "Wire_Cable"
//!    (centre (−23.5, 34.5, 6.3), r 4.0, yaw −1.57 [D dump]) → E-grab (the game's action-key grab
//!    [M Player_HandleJump 0x1916b0]) → the hand-over-hand ride [M Player_Wire 0x197658] → E on the
//!    zip door #112 (fallback 0x1E [D]) loads 07000006.

#[path = "common/walker.rs"]
pub mod walker;

use nf_game::player::Input;
use walker::{Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The truck + gate route: the walk to the gatehouse, the scripted ride (the crouch is the player's
/// own input [D fix-truck's trace]), the park, the chain's automatic exit.
/// UN-PARKED (mayhew-exit-walk 2026-10-17): the approach now walks the road to the gatehouse
/// walkway's WEST edge (98.6, 26.0, 106.5) — inside the ch-0x5C plate's XZ [D the entity
/// matrix (98.49, 27.05, 106.36), extent ±1.95/±1.75] — instead of the old (100, 26, 108)
/// target whose line runs along the lower lane into the arch's south face (the old stall:
/// the walked Bond oscillated at (101.2, 24.94, 107.6) [D Room_Road_A1's tris]). The chain
/// (0x5C → SP #38 → the bed 0x71 → the crouched ride → 0x70 → the exit) is pinned green in
/// crates/nf-view/tests/mayhew_handover_live.rs; this walk asserts the same from the
/// walkreal harness.
#[test]
fn walk_to_the_gatehouse_and_ride_the_truck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 104.95).abs() < 2.0 && (p[2] - 108.83).abs() < 2.0, "cold start #26 [D], got {p:?}");
        assert!(!sim.disabled, "no intro blocks this level's start");
    }
    w.wait(30);
    // Walk to the gatehouse (the truck's boarding spot; the fix-truck teleport stood at (100,26,108)).
    // Lenient: the gatehouse floor sits ~1.1 above the courtyard and the truck's own Script_Free
    // hand-off takes Bond from where he stands (the enable's `at` is the truck's matrix).
    w.walk_nav_to([98.6, 26.0, 106.5], "to the gatehouse", 1.6);
    {
        // The spawn's reveal plate (ch 0x45 at feet+0.47) sits below the toucher stand-in's reach —
        // a documented stand-in miss (see drone.rs's toucher comment); the truck chain does not
        // need it (fix_truck_live drives the whole ride without it).
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05] at the gatehouse tick {} bond {:?}; ch 0x45 {} ch 0x5c {}", w.total_ticks, sim.bond.pos,
            sim.drones.channels.on(0x45) as u8, sim.drones.channels.on(0x5c) as u8);
    }
    // The ride: the hand-off re-enables Bond on the truck bed (the disable→enable transition); the
    // crouch is the trace's ride stance [D fix-truck]; the park raises ch 0x70; the chain ends the
    // level into 07000006. The same integration fix_truck_live drives, from a WALKED start.
    let mut prev_dis = w.scene.sim.as_ref().unwrap().disabled;
    let mut crouch_armed = false;
    let mut input = Input::default();
    for t in 0..9000u32 {
        let dis = w.scene.sim.as_ref().unwrap().disabled;
        if !dis && prev_dis && w.scene.sim.as_ref().unwrap().bond.pos[0] < 95.0 { crouch_armed = true; }
        prev_dis = dis;
        input.crouch = if crouch_armed && !w.scene.sim.as_ref().unwrap().bond.crouched { crouch_armed = false; true } else { false };
        w.tick(input, false);
        if t % 500 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[05 ride t{t}] bond {:?} disabled {} 0x5c {} 0x70 {} 0x1d {}", sim.bond.pos, sim.disabled,
                sim.drones.channels.on(0x5c) as u8, sim.drones.channels.on(0x70) as u8, sim.drones.channels.on(0x1d) as u8);
        }
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x70), "the crouched ride parks on the ch-0x70 plate [D; fix-truck]");
    assert_eq!(sim.drones.level_exit, Some(0x0700_0006), "the truck chain's ch 0x1D ends the level into 07000006");
    // The objective chain: the spawn reveal plate fired on the boot tick (the walker's first step)
    // [L ee_start_05.bin switch_channels[0x45] = 1 at boot], so the monitor announced the mission's
    // first objective (state 3). It is NOT complete here: its condition ch 0x46 has no writer on
    // 07000005 — the OR #498 on 07000006 (list 0x50/0x51/0x52) is the writer [D triggers dump], and
    // the live ride-time snapshot agrees: record state 3 with ch 0x73 (the truck script's end) on
    // [L ee_snapshot.bin @0x2a3e28+0x14 = 3, switch_channels = {45, 73, 97}].
    assert!(sim.drones.channels.on(0x45), "the ch-0x45 reveal plate fired [L ee_start_05]");
    assert!(!sim.drones.channels.on(0x46), "ch 0x46 has no writer on 07000005 [D the full placement+script census]");
    assert_eq!(sim.drones.mission.states[0][0], 3, "the objective sits announced, not complete [L ee_snapshot record state 3]");
    eprintln!("[05] truck route done in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);

    // The game's own hand-over: Player_RamSave at the exit [M 0x1b2318] → the warm load of
    // 07000006 → start #496 (the 0x1D-gated truck-ride spawn, level_carry_live pins the position).
    let save = nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
        &sim.drones.inventory, &sim.drones.channels);
    drop(w);
    let mut next = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false, Some(&save))
        .expect("load 07000006 with the truck route's save"));
    {
        let sim = next.scene.sim.as_ref().unwrap();
        assert!((sim.bond.pos[0] - 57.18).abs() < 2.0 && (sim.bond.pos[2] - 22.61).abs() < 2.0,
            "the truck route spawns at the 0x1D entry [D placement 496], got {:?}", sim.bond.pos);
    }
    next.wait(2); // Mission_Init latches on the first tick (the port's init-per-tick latch)
    {
        let sim = next.scene.sim.as_ref().unwrap();
        // The truck-ride spawn (57.18, 39.73, 22.61) sits INSIDE the 0x51 arrival plate cluster
        // (#743-745/#497, (55.3-58.8, 39.2-39.9, 21.2-23.9) [D dump]) — the real game's arrival:
        // the plate fires on the first tick, OR #498 raises ch 0x46, and the monitor completes the
        // carried "Breach the castle walls" record and reveals the part-1 record [M 0x1827d8
        // cases 1/3; the live ride-time snapshot sat at state 3 only because it was taken on 05
        // (L ee_snapshot), before any arrival plate could fire].
        assert!(sim.drones.channels.on(0x51), "the truck-ride spawn's arrival plate raises ch 0x51 [D #743-745]");
        assert!(sim.drones.channels.on(0x46), "OR #498 (0x50|0x51|0x52) writes the objective's condition ch 0x46 [D dump]");
        assert_eq!(sim.drones.mission.states[0][0], 4,
            "'Breach the castle walls' completes at the 07000006 arrival [M 0x1827d8 case 3; D OR #498]");
        assert_eq!(sim.drones.mission.states[1][0], 3,
            "the next objective (part 1, cond ch 0x02) reveals on ch 0x46 [M the record's enable byte 0x46; L ee_start_05 @0x2a3e40+0x0f]");
        eprintln!("[05] the objective chain complete: 05 record state 4, 06 record state 3, ch 0x46 on");
    }
}

/// The cable route, MEASURED (level-05-west-ledge 2026-10-23): the cold start → door #153 →
/// ladder #151 → the tower top → the game's own exit from it — a walk-off FALL — → the west roofs
/// → the west lane → the WIRE (jump-grab in the ctx-3 icon zone) → the hand-over-hand crossing.
///
/// What the probes measured on this tree (west_ledge_probe.rs, all [D] geometry through the port's
/// collision of the disc's own data):
/// - The tower top the ladder delivers to (climb top = centre_y + radius + 0.85 = 29.91 + 3.9 + 0.85
///   = 34.66 [M Player_CollLadder 0x196118:22-24 — the port's 34.7 IS the game's top]) is caged at
///   floor level by the tower's own parapet ring (surfaces 36.3-37.6 above the top slot; every
///   ground-level ray from the top hits within 0.55-4.7 u) — the game leaves it by WALKING OFF the
///   edge and falling ~6.7 u to the y-28 roof (measured landing (58.99, 27.97, 89.61)).
/// - The roofs run west (y 26-29) to the west lane (x −27..−22), which runs south (z 42 → 13,
///   floors 28.8-35.5, continuous) to the wire's north grab zone.
/// - The wire #251 (−23.49, 34.47, 6.26, yaw −1.57 → the cable runs along Z; travel segment
///   z 2.72..9.8 [M Player_Wire 0x1976f8-0x197724]) spans the west lane's GAP: no floor at
///   z 5-8 from y 40 down to y 24 — the cable crosses a 4 u void. Three ctx-3 icons (the
///   jump-grab context) sit at (−23.5..−23.8, 32.2-32.7, z 0.21/2.65/12.33) [D probe] — the grab
///   is the JUMP press in the icon's straddle-cel set, not a walk-in [M Player_HandleJump
///   0x1916b0:118-147 — the ctx-3 press scans the wires and grabs the nearest within 1000 u].
/// - The old test's anchors ("ledge west 1/2/3", "the cable mouth") predate these heights and
///   the lane they assumed does not exist at the walked heights — the anchors were the TEST's
///   guess, now replaced by the measured lane above.
/// - ROUTE CORRECTION: the disc's collision census (the connectivity walk
///   over the port's world of the disc's own data, up-steps ≤ 0.55 [M Player_Collision 0x194df0
///   case 0: the capsule's r-0.55 bottom sphere], walls respected) shows the y-28 landing is an
///   ISLAND — 19-38 cells (x 57..63, z 85..91) with no walk edge to the courtyard, the roofs, or
///   the west lane. The roofs-west continuation this doc claimed is NOT in the disc. The game's
///   real lane is the castle-wall lane south of the courtyard strip (the west-lane-2 map), and
///   the route off the tower top's west edge drops onto the y-33.5 walkway slab — see
///   `walk_the_west_ledge_and_cross_the_cable`. This test still pins the TRUE geometry: the
///   perch cage and the walk-off fall's landing.
#[test]
fn walk_the_tower_top_and_the_walk_off_fall() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    // West along the courtyard, through door #153, up ladder #151.
    for (name, at) in [
        ("courtyard west", [62.0f32, 26.0, 96.0]),
        ("door #153", [57.8, 26.0, 89.4]),
        ("ladder #151 base", [60.9, 29.9, 89.2]),
    ] {
        w.walk_nav_to(at, name, 1.4);
    }
    w.walk_to(walker::Wp { at: [60.9, 29.9, 89.2], act: walker::Act::Ladder { top_y: 34.7 }, arrive: 0.9 }, "ladder #151");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05] the ladder top at tick {} bond {:?}", w.total_ticks, sim.bond.pos);
        assert!(sim.bond.pos[1] > 33.0, "Bond must stand on the ledge (y {:.2})", sim.bond.pos[1]);
    }
    // ── the tower top → the game's exit: the walk-off fall ────────────────────────────────────────
    // The parapet ring cages the top at floor level (the port's collision = the tower's own
    // surfaces [D]); the game leaves by walking off the edge and falling ~6.7 u to the y-28 roof
    // (the probe's measured landing). Sister (below): a pure west walk stalls at the parapet —
    // the fall is the route, not a port bug.
    {
        let mut landed = false;
        for t in 0..300u32 {
            let input = Input { forward: 1.0, ..Default::default() };
            // yaw −0.3: the probe's measured fall direction off the north-west edge.
            w.scene.sim.as_mut().unwrap().bond.yaw = -0.3;
            w.tick(input, false);
            let sim = w.scene.sim.as_ref().unwrap();
            if t % 30 == 0 { eprintln!("[05 fall t{t}] bond {:?}", sim.bond.pos); }
            if sim.bond.grounded && sim.bond.pos[1] < 32.0 && t > 30 { landed = true; break; }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05] the fall landed at tick {} bond {:?}", w.total_ticks, sim.bond.pos);
        assert!(landed, "the walk-off must land on the lower roof (bond {:?})", sim.bond.pos);
        assert!(sim.bond.pos[1] > 20.0, "the landing is the y-28 roof, not the valley (y {:.2})", sim.bond.pos[1]);
    }
    eprintln!("[05] tower-top + walk-off done in {} ticks", w.total_ticks);
}

/// The west lane — the WALKED legs, un-ignored. The route is the
/// west-lane-2 castle-wall map, settled against the disc's collision [D every waypoint below is
/// walked through the port's collision of the disc's own data; the port's collision matches the
/// disc per west-lane-2's ray probes]:
///
/// - the courtyard's west strip (x 28-56, z 92) → the lane south along the castle wall
///   (x 24-48, z 44-92). The 29.3 → 30.3 lip at x 42-44 is a > 0.55 face [D the 1-unit map],
///   taller than the game's step rule — the walk capsule's bottom sphere (r 0.55 [M
///   Player_Collision 0x194df0 case 0], B = pos + (0.55 − cc)) rides a step only below its
///   radius and `Player_SSWalk` 0x192310 → `Player_Move` 0x191dd0 add no step raycast — so the
///   lane takes the strip's sidestep columns (z 92-96 x 32-56 at floors 26-29.3, then the
///   x 26-35 lane band [D the 1-unit map]) — exactly what the game's own geometry offers.
/// - the tower top (x 28-34, z 39-64, floor 36.0 [D census]) — the lane's walked end. The way off
///   its west edge onto the y-33.5 walkway slab (x −19..25, z 45..55, floors 33-34 [D census]) is
///   NOT over the west parapet (top 37.08 along x 26.0-26.75, z ~48-63 [D the 0.25-unit map]): the
///   top's SOUTH edge has no parapet — the surface slopes down southward (36.0 at z 62 → 33.4 at
///   z 76, x 27-28.75 [D the 0.25 map]) and wraps west onto the south ledge (x 23.5-26.0, z 73-76,
///   floors 32.4-33.2), which runs north along the parapet's west side to BOTH the crawl shelf
///   (the wall-hug #299's entry) and the slab's east steps — see
///   `the_cable_legs_the_crawl_notch_and_ride`, which walks the whole link.
#[test]
fn walk_the_west_ledge_and_cross_the_cable() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    // The strip columns [D the 1-unit map]: the anchors stay on the mapped columns; the
    // x 42-44 lip is sidestepped by the strip's z 92-96 columns.
    for (name, at) in [
        ("courtyard west", [52.0f32, 26.0, 96.0]),
        ("the strip west", [42.0, 28.0, 94.5]),
        ("the strip, lane approach", [35.0, 28.0, 94.0]),
        ("the strip-lane junction", [34.0, 29.0, 92.0]),
        ("the lane head", [30.0, 30.0, 90.0]),
        ("the lane, wall south 1", [30.0, 31.0, 88.0]),
        ("the lane, wall south 2", [29.0, 32.6, 82.0]),
        ("the lane, tower north", [29.5, 33.7, 75.0]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05w] {name}: tick {} bond {:?}", w.total_ticks, sim.bond.pos);
    }
    w.walk_nav_to([30.0, 36.0, 60.0], "the tower top", 1.5);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[05w] the tower top: tick {} bond {:?}", w.total_ticks, p);
        assert!(p[0] > 26.0 && p[0] < 35.0 && p[1] > 34.5 && p[2] > 40.0 && p[2] < 66.0,
            "Bond walks the lane onto the tower top (floor 36.0 [D census]), got {p:?}");
    }
    eprintln!("[05w] the west lane walked in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[05w] fall: {f}"); }
    for st in &w.report.stuck_events { eprintln!("[05w] stuck: {st}"); }
}

/// The shared cable-route walk: the cold start → the courtyard strip →
/// the castle lane → the tower top → THE SOUTH SLOPE (the round-4 discovery that replaces the
/// parapet hunt) → the west-of-parapet ledge → the crawl shelf → the wall-hug crawl #299 → the
/// wide ledge → the walkway slab → the cable ledge → the notch's north edge.
///
/// The round-3/4 blockers answered by the 0.25-unit map (floor_at over x 23..28.75, z 40..76 at
/// 0.25 [D the port's collision of the disc's own data — the collision matches the disc per
/// west-lane-2's ray probes]):
/// - BLOCKER (a), the tower-top west parapet: real (top 37.08 along x 26.0-26.75, z ~48-63 [D]),
///   but the top's SOUTH edge has no parapet — the surface slopes down southward (36.0 at z 62 →
///   33.4 at z 76 [D]) and wraps west onto the south ledge (x 23.5-26.0, z 73-76, 32.4-33.2),
///   which runs NORTH along the parapet's west side to the crawl shelf AND the slab's east steps.
///   The walk-off onto the slab is walked through that ledge — no jump, no gap.
/// - BLOCKER (b), the crawl shelf: the shelf (30.9-31.4 at x 23-25.75, z 58-76) is reached by
///   that same south ledge (the walked stall at (27.06, 35.27, 71.63) was the UPHILL approach at
///   z 71.5; the downhill wrap at z 73-76 is clear [D the 0.25 map]). The waist north of the
///   shelf (x 25.25-25.75, z 67.5-70.5, the dip west of it reads 7.0-8.7 [D]) is the crawl band:
///   creepwall #299's game axis (the world-sphere centre (25.79, 31.77, 65.81) ± the bound radius
///   5.60 [M Control_BuildWorldSph 0x133070; D the parsed bounds]) spans z 60.21..71.41, and the
///   type-6 prompts sit at z 60.01/70.56 [D placements #256/#257] — the game routes the waist
///   through the wall-hug, not a walk.
pub fn walk_to_the_slab_east_end(w: &mut Walker) {
    for (name, at) in [
        ("courtyard west", [52.0f32, 26.0, 96.0]),
        ("the strip west", [42.0, 28.0, 94.5]),
        ("the strip, lane approach", [35.0, 28.0, 94.0]),
        ("the strip-lane junction", [34.0, 29.0, 92.0]),
        ("the lane head", [30.0, 30.0, 90.0]),
        ("the lane, wall south 1", [30.0, 31.0, 88.0]),
        ("the lane, wall south 2", [29.0, 32.6, 82.0]),
        ("the lane, tower north", [29.5, 33.7, 75.0]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    w.walk_nav_to([30.0, 36.0, 60.0], "the tower top", 1.5);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 26.0 && p[0] < 35.0 && p[1] > 34.5 && p[2] > 40.0 && p[2] < 66.0,
            "Bond walks the lane onto the tower top (floor 36.0 [D census]), got {p:?}");
    }
    // The south slope [D the 0.25-unit map]: down the top's bare south edge, west along the south
    // ledge (the z-74 floor hole at x 26.25 is passed on its west at z ≥ 74.5), north up the ledge.
    for (name, at) in [
        ("the top's south edge", [28.2f32, 36.0, 64.0]),
        ("the slope 1", [27.6, 35.3, 68.0]),
        ("the slope 2", [27.4, 34.4, 71.0]),
        ("the slope bottom", [27.3, 33.8, 74.5]),
        ("the south ledge", [26.0, 32.6, 75.5]),
        ("the ledge north", [25.75, 32.1, 73.5]),
        ("the crawl shelf edge", [25.6, 31.0, 71.8]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05c] {name}: tick {} bond {:?}", w.total_ticks, sim.bond.pos);
    }
    // The wall-hug crawl #299: the Use press at the shelf — the live entry spot (25.51, 32.11,
    // 71.02) [L fresh boot: one action press enters the wall-hug view of #299].
    w.walk_to(walker::Wp { at: [25.5, 30.96, 71.0], act: walker::Act::Use { use_ticks: 36 }, arrive: 1.0 },
        "the wall-hug crawl prompt");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05c] after the E: tick {} bond {:?} sub {:#x} ctx {:#x}", w.total_ticks, sim.bond.pos, sim.bond.substate, sim.bond.icon_ctx);
        assert!(sim.bond.substate == nf_game::player::SS_CREEP,
            "the E press at the prompt must enter the crawl substate 7 [M Player_Activate 0x1a5170 ctx 6 → Player_CollCreepWall 0x196758], bond {:?} sub {:#x}", sim.bond.pos, sim.bond.substate);
    }
    // Crawl NORTH across the waist: forward pushes toward the wall's A end — the axis direction
    // (sin(yaw+π/2), cos(yaw+π/2)) with yaw 1.57 points −z [M Player_Creep 0x1969d0: the input
    // direction decides the end; the port's 0.02/tick documented shape].
    let crawl_tick = w.total_ticks;
    let mut exited = false;
    for t in 0..1500u32 {
        let input = Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate != nf_game::player::SS_CREEP && w.total_ticks - crawl_tick > 10 {
            exited = true;
            eprintln!("[05c] the crawl exited at +{} ticks bond {:?}", w.total_ticks - crawl_tick, sim.bond.pos);
            break;
        }
        if t % 100 == 0 { eprintln!("[05c crawl t{t}] bond {:?} sub {:#x}", sim.bond.pos, sim.bond.substate); }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(exited, "the crawl must exit at a wall end [M Player_Creep 0x1969d0:134-156], bond {:?}", sim.bond.pos);
        assert!(sim.bond.pos[2] < 66.5 && sim.bond.pos[1] < 32.5,
            "the crawl must carry Bond north across the waist onto the wide ledge (z 58-66, floors 30.0-31.4 [D the 0.25 map]), got {:?}", sim.bond.pos);
    }
    // The wide ledge north → the slab's east steps (31.7-33.8 at z 49-55 [D the 0.25 map]) →
    // the slab's east end. THE STOP: the slab crossing west is the parked round-4 leg (see
    // `the_cable_legs_the_crawl_notch_and_ride`'s reason).
    for (name, at) in [
        ("the ledge north 1", [25.5f32, 31.1, 63.0]),
        ("the ledge north 2", [25.4, 31.2, 59.0]),
        ("the ledge steps", [25.6, 31.9, 54.0]),
        ("the slab east end", [25.4, 33.0, 50.5]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05c] {name}: tick {} bond {:?}", w.total_ticks, sim.bond.pos);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[05c] the slab east end: tick {} bond {:?}", w.total_ticks, p);
        assert!(p[0] > 23.0 && p[0] < 26.5 && p[1] > 32.5 && p[1] < 35.0 && p[2] > 48.0 && p[2] < 53.5,
            "Bond stands on the slab's east end (the steps' top, floors 33.0-33.8 at x 24-25.75, z 49-52 [D the 0.25 map]), got {p:?}");
    }
    // THE SLAB LINK (the round-5 answer): the slab is a NARROW strip at z ~54.3-55.7 (the 0.25
    // map: z 54 solid only at x ≥ 23.25; z 55 solid 33.3 from x 14 to 24; z 56 is the y-29-32
    // lower roof 3+ below; the z 52-54 band west of x 23.25 is VOID) — the game's link from the
    // east steps runs north onto the steps' top (33.8 at x 23.25-23.75, z 54) and WEST along the
    // strip. The straight-to-(x,52) legs of the earlier rounds targeted the void — Bond stalled
    // at the void's east face (23.61, 53.96) and the walker's rail-jump dropped him under the
    // slab. Steering keeps Bond grounded here (the bisect: byte-identical at f56c1829c^,
    // f56c1829c and the branch tip — the one-contact rework is NOT the cause; the transient
    // ungrounded ticks recover within a few frames).
    for (name, at) in [
        ("the steps top", [23.5f32, 33.7, 54.2]),
        ("the strip east", [20.0, 33.6, 54.9]),
        ("the strip mid", [8.0, 33.6, 55.0]),
        ("the strip nose", [0.5, 33.5, 54.6]),
        ("the slab bend", [-2.0, 33.4, 51.3]),
        ("the slab diagonal", [-5.0, 32.8, 48.6]),
        ("the slab low", [-9.0, 32.0, 46.9]),
    ] {
        w.walk_to(walker::Wp { at, act: walker::Act::None, arrive: 1.5 }, name);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05c] {name}: tick {} bond {:?}", w.total_ticks, sim.bond.pos);
    }
    // The band's west end. THE CONTINUATION (level-05-west-lane-6): the band's 33-floor does NOT
    // end at x −9.5 — the collision column dump (examples/probe_gap_05.rs, level.instances(true),
    // the world's exact static set) shows the floor continuing SOUTH-WEST (z 45.0: x −14..−8 at
    // 33.0; z 45.5: x −12.5..−8) and a narrow z 44.5-45.0 corridor (x −15..−14.5, 33.3-33.8)
    // linking it to the west pieces (x −18..−16.5, z 46.5-47.5, 33.2-34.0). The lane-5 "void gap
    // x −16..−9.5, z 45.5-48" was a misread: the voids sit NORTH of the corridor (the deep
    // y-17-21 slot x −13..−9.5 at z 47-48 and the y-29 roof x −16..−11.5 at z 46-47).
}

/// The kink ramp + the ledge run to the notch's north edge, RAW full-forward real input
/// (level-05-west-lane-8). The 45° ramp is mat 0x0c, n (−0.520, 0.680, 0.517), its base edge
/// (−26.027, 33.0, 44.441) → (−23.393, 33.0, 47.090) — the wall-top line under it [D
/// examples/probe_kink_05.rs]. On this ramp NEITHER the game nor the port grounds a
/// capsule-tangent rest:
/// - the feet-ray band needs |pos − hit| − 0.1 ≤ cc [M Player_FeetOnPoint 0x194188:106,
///   decompile `*(float *)(hit + 0xc) - 0.1 <= fVar11`], but resting the walk capsule
///   (B = pos − 0.485, r 0.55 [M Player_Collision 0x194df0 disasm 0x195444/0x195464]) tangent
///   on the 47° face leaves the upright ray 0.485 + 0.55/0.68 = 1.294 long — over the 1.135
///   band, always;
/// - the pick word's bit 0 needs dot(normal, axis) > 0.7 [M Intersect_CylGeom 0x10a72c-0x10a734
///   disasm: lhu +0x9c / bnez skip / the 0x3f33 0.7 gates / ori 1 / sh] and the ramp's 0.68
///   never sets it;
/// - the pick word's second feed — Collide_Intersect ORs a stack halfword into HITTEST+0x9c per
///   intersect-loop pass [M 0x1ec138-0x1ec150 disasm: lhu +0x9c, lhu 0xec($sp), or, gated by
///   lhu +0x94 & 0x200, sh] (cel+0x94 holds the glist flags [D MapObject.flags, level.rs]) — is
///   CLOSED on this ramp: its instances (Room_Outerwall_Corner, Room_Outerwall_B2, Room_Road_B)
///   carry glist flags 0x0 [D examples/probe_flags_05.rs], so the store is skipped.
/// So the game's player on this ramp is push-borne ungrounded exactly like the port — the
/// slope-blind push holds him out of the face while gravity accumulates
/// [M Player_Collision 0x194df0:104 applies gravity iff ungrounded OR +0x110 < 0.5], and the
/// push's horizontal part drifts him NW (up-slope-ward, toward the base edge and the wall cliff)
/// — measured ~0.024/tick once airborne [L probe_kink2 run 1: at the walker's 0.45 near-target
/// forward the drift beat the walk; Bond crossed the base edge at tick 11 of the raw steer and
/// fell to the y-29 roof]. A player takes it at FULL forward, each leg aimed 0.35 SE (down-slope)
/// of its waypoint so the walk outruns the drift. Real inputs only: turn + forward, no
/// teleports, no state pokes; a fall to the roof or a wedged leg fails the helper.
fn walk_the_kink_ramp(w: &mut Walker) {
    let targets: [([f32; 3], &str); 16] = [
        ([-23.9, 33.6, 45.8], "the step spine 4"),
        ([-24.3, 33.6, 45.4], "the step spine 5"),
        ([-24.7, 33.6, 45.0], "the step spine 6"),
        ([-25.1, 33.5, 44.6], "the step spine 7"),
        ([-25.5, 33.4, 44.2], "the step spine 8"),
        ([-25.8, 33.4, 43.8], "the step spine 9"),
        ([-26.0, 33.4, 43.2], "the ledge bend"),
        ([-26.2, 33.4, 42.6], "the ledge neck"),
        ([-26.5, 33.5, 41.6], "the ledge north west"),
        ([-26.3, 33.5, 40.6], "the ledge 40"),
        ([-25.9, 33.5, 38.6], "the ledge 38"),
        ([-24.9, 33.4, 36.5], "the ledge 36"),
        ([-24.2, 33.2, 32.5], "the ledge 32"),
        ([-23.6, 33.4, 28.5], "the ledge 28"),
        ([-23.5, 33.5, 22.5], "the ledge 22"),
        ([-23.5, 34.5, 18.5], "the ledge 18"),
    ];
    let se_bias: f32 = 0.35;
    let mut leg = 0usize;
    let mut ticks_on_leg = 0u32;
    while leg < targets.len() {
        let (at, name) = targets[leg];
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(p, at) < 0.42 {
            eprintln!("[05g] {name}: +{ticks_on_leg} ticks bond {:?}", p);
            leg += 1;
            ticks_on_leg = 0;
            continue;
        }
        let aim = [at[0] + 0.709 * se_bias, at[1], at[2] - 0.705 * se_bias];
        let desired = (aim[0] - p[0]).atan2(aim[2] - p[2]);
        let err = walker::wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: 1.0, ..Default::default() };
        w.tick(input, false);
        ticks_on_leg += 1;
        let p1 = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(p1[1] > 33.2,
            "the kink walk stays on the ramp/ledge run (centre y > 33.2; the y-29 roof rest is 30.1): leg {name} fell, bond {p1:?}\n{}",
            walker::fail_dump_pub(&w.scene, at, name));
        assert!(ticks_on_leg <= 900,
            "the kink walk leg {name} wedged at bond {p1:?}\n{}", walker::fail_dump_pub(&w.scene, at, name));
    }
}

/// The gap crossing, the west pieces and the cable ledge to the notch's north edge — the
/// level-05-west-lane-6 route, walked through real input. The geometry [D the collision column
/// dump, examples/probe_gap_05.rs over `level.instances(true)` — the world's exact static set]:
/// - the band's 33-floor continues south-west past x −9.5 (z 45.0: x −14..−8 at 33.0; z 45.5:
///   x −12.5..−8 at 33.0) — the lane-5 "void gap x −16..−9.5, z 45.5-48" was a misread;
/// - a narrow z 44.5-45.0 corridor (x −15..−14.5, floors 33.3-33.8; the void at z 44.0 south,
///   the y-29.3-29.5 roof at z 45.5-46.0 north) links it to the pieces' south strip
///   (x −16.5..−15.5, z 45.0-46.0, 33.5-33.8);
/// - the west pieces (x −18..−16.5, z 46.5-47.5, 33.2-34.0) run to the cable ledge
///   (x −23..−20.5, z 46-47, 33.2-33.8) across a 0.5-unit void seam at x −20.0 (z 44..48) —
///   narrower than the capsule's 1.1 diameter, crossed on the edge-pick contacts
///   [M Player_FeetOnPoint 0x194188 grounds the pick bit; the round-6 walk exercises it].
pub fn walk_the_gap_to_the_notch(w: &mut Walker) {
    // The band's south-west edge → the corridor → the pieces' strip → the pieces → the seam →
    // the cable ledge. Tight legs (arrive 0.9): the corridor is 1.0-1.5 units wide with drops
    // on both sides.
    for (name, at) in [
        ("the band's west end", [-9.0f32, 33.0, 46.9]),
        ("the band's south-west edge", [-9.4, 33.0, 45.9]),
        ("the band's south-west corner", [-10.8, 33.0, 45.4]),
        ("the corridor mouth", [-12.6, 33.0, 45.1]),
        ("the corridor", [-14.2, 33.2, 44.9]),
        ("the corridor west", [-15.3, 33.5, 45.1]),
        ("the pieces' south strip", [-16.1, 33.7, 45.7]),
        ("the pieces' strip north", [-16.5, 33.7, 46.3]),
        ("the west pieces", [-17.4, 33.5, 46.9]),
        ("the pieces north", [-18.6, 33.6, 47.0]),
        ("the seam east side", [-19.5, 33.8, 47.0]),
        ("the seam crossing", [-20.8, 33.7, 47.0]),
        ("the cable ledge", [-22.2, 33.5, 47.0]),
        ("the cable ledge step", [-23.2, 33.2, 46.8]),
    ] {
        w.walk_to(walker::Wp { at, act: walker::Act::None, arrive: 0.9 }, name);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05g] {name}: tick {} bond {:?}", w.total_ticks, sim.bond.pos);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > -24.5 && p[0] < -21.5 && p[1] > 31.5 && p[1] < 35.5 && p[2] > 44.5 && p[2] < 49.5,
            "Bond crosses the seam onto the cable ledge (x −23..−20.5, z 46-47, floors 33.2-33.8 [D the column dump]), got {p:?}");
    }
    // The cable ledge south to the notch's north edge. The heights re-verified against the
    // column dump [D]: the ledge's east edge recedes west going south — z 45.0: x −25..−24.5
    // (33.4-33.7); z 43.5: x −26..−25.5 (33.3-33.8); z 40: x −26.5..−26 (33.3-33.8); z 36: x −25..−23.5
    // (33.2-33.8); z 28-20: x −24..−23 (33.2-34.2); z 16-14: x −24..−23 (33.2-35.4, the step up
    // from the 33.2 run) with the notch's north shelf 30.8-31.0 at x −25.5..−24.5 — the census's
    // coarse "x −28..−23, z 19..40, floors 30-34" [D] at the column dump's resolution.
    // The step is a 0.5-1.0-unit diagonal shelf [D the column dump]: z 46.5: x −23.5..−22;
    // z 46.0: x −24..−23.5; z 45.5: x −24.5..−24; z 45.0: x −25..−24.5; z 44.5: x −25.5..−25;
    // z 44.0: x −26..−25.5 — walked on its spine, 0.5-unit legs, arrive 0.5 (the capsule's
    // diameter is 1.1: the axis stays on the shelf, the overhang rides the edge picks
    // [M Player_FeetOnPoint 0x194188]).
    // The spine's first legs (flat cable-ledge floor, the walker's own steering): spine 1-3.
    // From spine 3 the 45° kink ramp starts; its legs run in walk_the_kink_ramp below — the
    // round-6/7 park ("a wall contact stalls the steering and the stuck ladder's sidestep shoves
    // Bond off the shelf's west edge" [L]) is answered there: the ramp walk is taken at FULL
    // forward with a 0.35 down-slope aim bias, real inputs only (see walk_the_kink_ramp's doc).
    for (name, at) in [
        ("the step spine 1", [-22.8f32, 33.5, 46.7]),
        ("the step spine 2", [-23.2, 33.5, 46.4]),
        ("the step spine 3", [-23.6, 33.5, 46.1]),
    ] {
        w.walk_to(walker::Wp { at, act: walker::Act::None, arrive: 0.4 }, name);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05g] {name}: tick {} bond {:?}", w.total_ticks, sim.bond.pos);
    }
    walk_the_kink_ramp(w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[05g] the walked end: tick {} bond {:?}", w.total_ticks, p);
        assert!(p[0] > -24.2 && p[0] < -22.8 && p[1] > 33.4 && p[1] < 35.8 && p[2] > 16.5 && p[2] < 20.5,
            "Bond walks the seam, the kink ramp and the ledge run to the notch's north edge (the ledge 18 end x −23.5, z 18.5, centre y ~34.6-34.8 [L probe_kink2 run 2]), got {p:?}");
    }
}

/// The crawl legs, UN-IGNORED: the whole west lane through the south
/// slope, the crawl shelf, the wall-hug crawl #299 (the real Use press, the entry + the north
/// exit across the waist) and the ledge up onto the walkway slab's east end. The route and the
/// two round-3/4 blocker answers are documented on `walk_to_the_slab_east_end`. The legs west
/// of the slab's east end (the slab crossing → the notch grab → the ride) stay parked on
/// `the_cable_legs_the_crawl_notch_and_ride` with their exact blockers.
#[test]
fn the_crawl_legs_the_south_slope_shelf_and_slab_east_end() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    walk_to_the_slab_east_end(&mut w);
    eprintln!("[05c] the crawl legs done in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[05c] fall: {f}"); }
    for st in &w.report.stuck_events { eprintln!("[05c] stuck: {st}"); }
}

/// The cable legs, UN-IGNORED: the whole west lane through the crawl
/// #299, the slab strip and the slab band's diagonal down to its west end (x −9.5). The route
/// and the blocker answers are documented on `walk_to_the_slab_east_end`. The legs beyond (the
/// gap crossing → the cable ledge → the notch grab → the ride → the south ledge) stay parked on
/// `the_cable_legs_the_crawl_notch_and_ride` with their exact blockers.
#[test]
fn the_cable_legs_the_crawl_notch_and_ride() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    walk_to_the_slab_east_end(&mut w);
    eprintln!("[05c] the slab band walked in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[05c] fall: {f}"); }
    for st in &w.report.stuck_events { eprintln!("[05c] stuck: {st}"); }
}

/// The gap legs, UN-IGNORED (level-05-west-lane-6): the whole west lane through the crawl
/// #299, the slab band's diagonal, the band's south-west continuation, the corridor, the west
/// pieces, the x −20 void seam, the cable ledge south to the notch's north edge, the notch
/// JUMP-grab at icon #310 ((−23.80, 32.74, 12.33) [D]) and the hand-over-hand ride to the
/// ledge's south piece. The route and the round-6 gap answer are documented on
/// `walk_the_gap_to_the_notch`.
#[test]
fn the_cable_legs_the_gap_and_the_ledge_step() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    walk_to_the_slab_east_end(&mut w);
    walk_the_gap_to_the_notch(&mut w);
    eprintln!("[05g] the gap legs done in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[05g] fall: {f}"); }
    for st in &w.report.stuck_events { eprintln!("[05g] stuck: {st}"); }
}

/// The PARKED notch legs: the step's spine (a 0.5-1.0-unit diagonal shelf [D the column dump])
/// walked to its z-46.0 kink; the kink's wall contact stalls the steering (the stuck ladder's
/// sidestep then shoves Bond off the shelf's west edge onto the y-29 roof [L the round-6 runs]).
/// The shelf walk to the notch's north edge, the notch JUMP-grab at icon #310 ((−23.80, 32.74,
/// 12.33) [D]) and the hand-over-hand ride to the ledge's south piece run as written once the
/// kink is legged. The grab/ride legs are the round-3 shape (git 96cebefd1) with the heights
/// re-verified against the column dump [D]: the ledge run z 28-20 at x −24..−23 (33.2-34.2),
/// z 16-14 (33.2-35.4), the notch's north shelf 30.8-31.0 at x −25.5..−24.5.
#[test]
fn the_notch_grab_and_ride_parked() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    walk_to_the_slab_east_end(&mut w);
    walk_the_gap_to_the_notch(&mut w);
    // The grab: the JUMP press in the ctx-3 zone [M Player_HandleJump 0x1916b0:126-147 — with
    // BLData+0x95F = 3 the press scans the level's wires for the NEAREST class-0x2C within
    // 1000 units and grabs instead of jumping]. The ledge-18 end (z 18.8) sits north of the
    // icon #310 zone ((−23.80, 32.74, 12.33) [D placements]): walk SOUTH into the zone with
    // real input (steered at the icon, the walker's own turn rule) until the icon ctx arms 3 —
    // the game's own armed-grab signal — before the press. The dip guard: the notch's dip
    // drops to y 17-25 [D the 1-unit map]; a centre y under 29.0 means the walk fell in.
    let mut in_zone = false;
    for _ in 0..600u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if sim.bond.icon_ctx == 3 { in_zone = true; break; }
        if p[1] < 29.0 {
            panic!("the zone walk fell into the notch's dip at {p:?}\n{}",
                walker::fail_dump_pub(&w.scene, [-23.8, 32.7, 12.3], "the notch zone walk"));
        }
        let desired = (-23.8 - p[0]).atan2(12.33 - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    assert!(in_zone, "the ctx-3 wire zone must arm before the notch's dip (icon #310 [D placements])");
    // Settle into the press: walk at the icon (real input, steered, the dip guard armed) until
    // the 0.95 head LOS clears — the press must pass the game's own gates [M Player_HandleJump
    // 0x1916b0 via the port's jump arm: grounded_history, head_bump, and the 0.95 head LOS] —
    // and the walk momentum decays (the first press attempt off the kink walk's residual drift
    // was eaten with no jump and no grab [L the press diag run]; from the settled rest the same
    // press grabs). The walk also nudges Bond off the x −23.55 line whose overhead tri of the
    // upper ledge band blocked the head line.
    let mut settled = false;
    for _ in 0..600u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let los = sim.world.line_of_sight(p, [p[0], p[1] + 0.95, p[2]], nf_game::world::LOS_PLAYER_HEADROOM);
        // slow = under 0.032/tick (the kink walk's residual drift ran ~0.056/tick and kept him
        // under the overhead tri whose down-push set head_bump at the press tick — the arm the
        // press rides skips on head_bump [M the port's jump arm, Player_HandleJump's gates]);
        // the wedged rest runs ~0 and clears the bump.
        let slow = sim.bond.vel[0] * sim.bond.vel[0] + sim.bond.vel[2] * sim.bond.vel[2] < 0.001;
        if los && slow && !sim.bond.head_bump { settled = true; break; }
        if p[1] < 29.0 {
            panic!("the close walk fell into the notch's dip at {p:?}\n{}",
                walker::fail_dump_pub(&w.scene, [-23.8, 32.7, 12.3], "the notch close walk"));
        }
        let desired = (-23.8 - p[0]).atan2(12.33 - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: if los { 0.3 } else { 1.0 }, ..Default::default() };
        w.tick(input, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let los = sim.world.line_of_sight(p, [p[0], p[1] + 0.95, p[2]], nf_game::world::LOS_PLAYER_HEADROOM);
        eprintln!("[05g] the zone: ctx {:#x} los {los} bond {:?} ghist {} bump {}",
            sim.bond.icon_ctx, p, sim.bond.grounded_history, sim.bond.head_bump);
        assert!(settled && los && sim.bond.icon_ctx == 3,
            "the press settle must reach the armed ctx-3 zone with clear head LOS (icon #310 [D placements])");
    }
    // Face the wire's south run and press: the jump press at t == 5 of the loop (an edge).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.yaw = (-23.49 - sim.bond.pos[0]).atan2(6.26 - sim.bond.pos[2]);
        sim.bond.vel = [0.0; 3];
    }
    // The press pulses one edge per 6 ticks (the harness's edge-trigger rule — the game's
    // jump is edge-triggered [M Input_Action mask 4]): the ledge-18 rest flaps grounded/airborne
    // on the push dither and a down-push on a press tick sets head_bump, which skips the whole
    // HandleJump arm that tick [M the port's jump arm]; a later pulse lands clean. [L the press
    // diag runs: a single press from the drifted rest was eaten with no jump and no grab.]
    let mut grabbed = false;
    for t in 0..600u32 {
        let input = Input { jump: t % 6 == 0, ..Default::default() };
        w.tick(input, false);
        if w.scene.sim.as_ref().unwrap().bond.substate == nf_game::player::SS_WIRE
            || w.scene.sim.as_ref().unwrap().bond.substate == nf_game::player::SS_ZIPLINE {
            grabbed = true;
            break;
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05g] the press: grabbed {grabbed} ctx {:#x} sub {:#x} ghist {} bump {} bond {:?}",
            sim.bond.icon_ctx, sim.bond.substate, sim.bond.grounded_history, sim.bond.head_bump, sim.bond.pos);
    }
    assert!(grabbed, "the jump press at the notch edge must grab the wire [M Player_HandleJump 0x1916b0:126-147]");
    eprintln!("[05g] the wire grabbed at tick {}", w.total_ticks);
    // The hand-over-hand ride south across the notch [M Player_Wire 0x197658; the travel
    // segment z 2.72..9.8 0x1976f8-0x197724]. The ride's motion is STRAFE-driven — the strafe
    // axis picks the stroke mode (7 toward B, 8 toward A, 6 idle) [M 0x1977c0-0x197834]; the
    // old forward-only ride never advanced (the stroke never armed [L the ride diag: z stuck
    // 8.2-8.9 for 1200 ticks]). Strafe 1.0 rides toward B; the drop paths are the action press
    // or the 1800-tick auto-drop [M 0x197748-0x197834] — ride until the stroke ends at the far
    // segment end (the strafe arm zeroes within 0.5² of the travelled-to end) or the wire drops.
    let grab_tick = w.total_ticks;
    let mut last_z = w.scene.sim.as_ref().unwrap().bond.pos[2];
    let mut stall_ticks = 0u32;
    let mut rode = false;
    for _ in 0..1800u32 {
        let input = Input { strafe: 1.0, ..Default::default() };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate != nf_game::player::SS_WIRE && sim.bond.substate != nf_game::player::SS_ZIPLINE
            && w.total_ticks - grab_tick > 10 {
            rode = true;
            break;
        }
        let z = sim.bond.pos[2];
        if (z - last_z).abs() < 0.01 { stall_ticks += 1; } else { stall_ticks = 0; }
        last_z = z;
        if stall_ticks > 60 { rode = true; break; } // the stroke ended at the segment's far end
        if w.total_ticks % 100 == 0 {
            eprintln!("[05g ride] bond {:?} sub {:#x}", sim.bond.pos, sim.bond.substate);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05g] after the ride: tick {} (+{}) bond {:?} sub {:#x}",
            w.total_ticks, w.total_ticks - grab_tick, sim.bond.pos, sim.bond.substate);
        assert!(rode, "the ride must run and end (the stroke's far end or the drop)");
        let p = sim.bond.pos;
        assert!(p[2] < 8.0 && p[1] > 24.0,
            "the crossing must carry Bond south across the notch (the travel segment z 2.72..9.8 [M Player_Wire 0x1976f8-0x197724]; the landing z −11..4, floors 30-31.4 [D census]), got {p:?}");
    }
    eprintln!("[05g] the gap, notch and ride done in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[05g] fall: {f}"); }
    for st in &w.report.stuck_events { eprintln!("[05g] stuck: {st}"); }
}

/// The cable route's LAST legs: from the wire ride's landing on the ledge's south piece to the
/// zip door #112 "Locked_Door_Top_I" ((37.25, 49.00, −69.00) [D placement], whose E writes the
/// fallback ch 0x1E and loads 07000006 [M Trigger_Activate 0x1b2318:91-107; level_carry_live pins
/// the 0x1E spawn]) — the whole route on real input.
///
/// THE ROUTE (level-05-zip-door; the old "south walk stalls" negative is superseded — the stall
/// was real but the route leaves the x −23.5 line). The disc's own data [D the probe census +
/// examples/probe_southwest05.rs's band maps, x −44..24 step 1 at y0 60/48/34]:
/// - the landing's south piece (x −26..−22, z 0..−8, floors 30.4-31.4) past the landing's south
///   rail, then a NOTCH in the strip at z −11..−13 (floor 20-25) — jumped;
/// - the strip's approach (x −29..−23, z −16..−18, floors 32.2-35.4) sloping west down to the
///   window-ledge approach (x −35..−34, z −20..−27, floors 27-33.7 under the gallery's 45.5
///   roof) and up onto the window ledge (x −35, z −28, floor 35.3);
/// - THE WALL-HUG WINDOW CRAWL #35 "Wall Hug Window" ((−34.10, 37.28, −31.65), yaw 1.571,
///   half_len 5.36×scale → the band z −26.3..−37.0 [D placement + CreepWall_Create 0x196fc8;
///   the crawl prompts #84/#85 (type 6) at z −37.3/−25.5, the progress plates #311-#314
///   (out 127/128/129/133) riding the band]) — the E press enters substate 7 [M Player_Activate
///   0x1a5170:66-97 → Player_CollCreepWall 0x196758], forward crawls to the wall's south end
///   [M Player_Creep 0x1969d0: the input direction decides the end];
/// - the roofed corridor south (x −36, z −38..−44, floor 32.5-36.2 under the 46.0 roof), then a
///   second notch at z −46..−50 (floor 18-25) — jumped;
/// - the south gallery (x −35..−34, z −52..−60, floors 31-33.7) and the SW loop: the lower path
///   bends west/south (x −39..−41, z −64..−72, floors 27.8-32.6), the west walkway (x −41..−40,
///   z −70..−76, floors 37.1-37.6), the z −78 band (x −38..−27, floors 29-37.3), the z −76 spine
///   (x −30..−26, 37.4→32.9) up to the battlement's west end (x −28..−16, z −72..−74, floors
///   36.7-37.5) — the SWITCH_Helicopterstart(SC3) plate #269 ((−22.76, 37.15, −72.36) out 0x42
///   [D placement]) fires en route;
///
/// WALKED NOW (level-05-every-objective-2): the approach → the WINDOW LEDGE (x −35, the
/// disc's own floors 36.0-36.43 running z −19..−25.5 [D probe_sealtris05]) → the wall-hug
/// window crawl #35 (the real E at the prompt #85 zone, the forward crawl south to the band's
/// end z ~−36). The band's north seal (the negative below) is REAL but OFF-ROUTE: the player's
/// lane leaves west over the ledge — the seal's tri anatomy (the notches closed by their
/// pillars, the piers) is cited on the negative's comment.
/// - THE MOAT INLET at x −15..−5 (z −72..−76 floors 20-23 — a 14-unit cut through the wall-walk)
///   is crossed by THE Q-CLAW GRAPPLE: pulley #281 "graple_pulley" ((−3.40, 42.94, −71.77) [D
///   placement] with its "Grapple_from bottom" CEL #280 (−3.39, 42.82, −70.96)) stands 17.6
///   units east of the west battlement's east end — inside the claw's bolt range 50.0
///   [M 0x12e7b0:158 + weapon_data[0x50]+0x1c]; the reel [M Player_SSGrapple 0x192860: phase 1
///   pulls at 0.05→0.5/tick toward BLData+0x140, phase 2 hangs within 2.0, the fire RELEASE
///   detaches] carries Bond over the inlet onto the east battlement (x −4..+4, z −72, floors
///   36.3-36.9) — the route's own grapple icons #277/#300 (the landing, type 3) and #310 (the
///   notch, type 3 big) mark the claw legs [D placements];
/// - the east battlement east through the trough (x 5..8, floors 30.2-34.2) and up the rising
///   stairs (x 9..24, 37.0→40.4) to the door's y-49 band (x 16..36 at z −68 [D the old scan])
///   and the door at its east end; the door guards #271/#272 (41.5/46.4, 46, −69.7) and the
///   ammo #322/#323 (49.2/48.1, 49, −60.5/−59.3) stand past it [D placements].
#[test]
fn the_cable_landing_walks_the_south_walkway_toward_the_zip_door() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    walk_to_the_slab_east_end(&mut w);
    walk_the_gap_to_the_notch(&mut w);
    // The grab + ride, the round-8 shape (the notch test's exact legs): the ctx-3 zone walk,
    // the settle, the jump press, the strafe ride.
    let mut in_zone = false;
    for _ in 0..600u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if sim.bond.icon_ctx == 3 { in_zone = true; break; }
        if p[1] < 29.0 {
            panic!("the zone walk fell into the notch's dip at {p:?}\n{}",
                walker::fail_dump_pub(&w.scene, [-23.8, 32.7, 12.3], "the landing-leg zone walk"));
        }
        let desired = (-23.8 - p[0]).atan2(12.33 - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    assert!(in_zone, "the ctx-3 wire zone must arm (icon #310 [D placements])");
    let mut grabbed = false;
    for t in 0..600u32 {
        let input = Input { jump: t % 6 == 0, ..Default::default() };
        w.tick(input, false);
        let s = w.scene.sim.as_ref().unwrap().bond.substate;
        if s == nf_game::player::SS_WIRE || s == nf_game::player::SS_ZIPLINE { grabbed = true; break; }
    }
    assert!(grabbed, "the jump press must grab the wire [M Player_HandleJump 0x1916b0:126-147]");
    let grab_tick = w.total_ticks;
    let mut last_z = w.scene.sim.as_ref().unwrap().bond.pos[2];
    let mut stall_ticks = 0u32;
    let mut rode = false;
    for _ in 0..1800u32 {
        let input = Input { strafe: 1.0, ..Default::default() };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let s = sim.bond.substate;
        if s != nf_game::player::SS_WIRE && s != nf_game::player::SS_ZIPLINE && w.total_ticks - grab_tick > 10 {
            rode = true; break;
        }
        let z = sim.bond.pos[2];
        if (z - last_z).abs() < 0.01 { stall_ticks += 1; } else { stall_ticks = 0; }
        last_z = z;
        if stall_ticks > 60 { rode = true; break; }
    }
    assert!(rode, "the ride must run and end");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[05z] the landing: tick {} bond {:?}", w.total_ticks, p);
        assert!(p[2] < 8.0 && p[1] > 24.0,
            "the crossing lands on the ledge's south piece (z −11..4, floors 30-31.4 [D census]), got {p:?}");
    }
    // The stroke's end leaves Bond HANGING at the wire's B end (the ride loop exits on the
    // stall): the game's drop paths are the action press or the 1800-tick auto-drop
    // [M Player_Wire 0x197748-0x197834; the port's wire drop reads the jump/action-7 press].
    for t in 0..120u32 {
        let s = w.scene.sim.as_ref().unwrap().bond.substate;
        if s != nf_game::player::SS_WIRE && s != nf_game::player::SS_ZIPLINE { break; }
        w.tick(Input { jump: t % 6 == 0, ..Default::default() }, false);
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05z] after the drop: tick {} bond {:?} sub {:#x}",
            w.total_ticks, sim.bond.pos, sim.bond.substate);
        assert!(sim.bond.pos[1] < 34.0,
            "the drop must put Bond on the landing's floors (30-31.4 [D census]), got {:?}", sim.bond.pos);
    }

    // ---- THE SOUTH LEGS (all real input; the raw-steer helper + jump pulses) ----

    // Leg 1: past the landing's south rail (an L: the x −23.8 lane stops at z −2, the x −24.5
    // lane runs to z −8 [L the two wedge runs]) then cut EAST to the crack's north lip — the
    // crack (z −11.5..−12.5) swallows x −25..−24; the lip lane x −23.5 keeps floors 31.0-31.2
    // [D probe_dip05's fine map].
    walk_legs(&mut w, &[
        ([-23.7, 31.3, 0.5], "the landing's south edge"),
        ([-24.6, 31.6, -2.0], "the south piece west"),
        ([-24.9, 31.4, -5.0], "the south piece 5"),
        ([-25.0, 31.2, -8.0], "the south piece 8"),
        ([-24.4, 31.0, -9.8], "the crack cut east"),
        ([-23.7, 31.1, -11.2], "the crack north edge"),
    ], 26.0, 900);

    // Leg 2: HOP the strip's crack (z −11.5..−12.5, floor 23.5-28 [D probe_dip05]) — a 1.5-unit
    // hop along x −23.5; the far side resumes 31.4 at z −12.5.
    run_jump(&mut w, [-23.6, 31.4, -13.4], "the strip crack hop", 420, 17.0);

    // Leg 3: the approach (z −16..−19, floors 32.2-36.6 [D the fine map]) west onto the window
    // ledge itself — the ledge at x −35 runs z −22..−28 at y 37.0-37.3 [D the y0-48 band map;
    // the y0-34 hits under it are a lower shelf], the crawl prompt #85 at its middle. The
    // ledge's north lip (a step off the 0.95-normal slope [L the wedge run]) hops.
    walk_legs(&mut w, &[
        ([-25.8, 32.6, -16.6], "the approach 16"),
        ([-27.5, 33.8, -18.0], "the approach 18"),
        ([-24.6, 34.4, -18.4], "the east strip south"),
        ([-24.3, 34.8, -19.6], "the east strip 19.6"),
    ], 19.0, 1200);
    // THE BAND'S NORTH SEAL (a [D]-backed negative, kept): the west band (the AI line at
    // y 36.5-37.6 under the 39.8/45.5 roofs [D ai point #259 (−23.41, 36.51, −31.57) + the
    // guards #65/#68 (−31.3/−32.8, 37.6, −22..−37)]) is sealed from the strip at z −20.5:
    //   - the x −23.7 lane walls at z −19.5 under an overhang lip (the jump bounces at 34.4
    //     under a ceiling at ~35.3) [L the wedge runs];
    //   - the west lanes (x −31..−32) wall at z −20.45, floor_ny 0.95 [L the wedge runs + the
    //     sealwalk probe: the x −31.44 capsule wedges on the ramp (n 0.09,0.86,0.50 [D]) against
    //     the x −31.75..−31.51 pier (36.55..38.0 [D])];
    //   - the x −26 lane tops out on the face's rim at y ~38 with 1.6 headroom — Bond's own
    //     1.86 body cannot stand there [L the rim run]; the east lane x −26 walls at z −19.45
    //     on the disc's own face (34.20..34.98 at z −20.0 [D probe_sealtris05]);
    //   - the band's north notches (x −31.75..−30.25 and −26.5..−25.24, tops 36.86 [D]) are
    //     closed by their pillars (x −31.38..−30.63 / −26.38..−25.62, 37.0..38.78 at z −21.24
    //     [D]) — the clear sub-lanes are 0.12-0.38 wide against the capsule's 1.1 diameter.
    // The walk below PINS the seal as a [D]-backed negative (the disc's own wall faces hold
    // the line). IT IS NOT THE ROUTE: the player's lane leaves WEST over the window ledge —
    // the legs below (the ledge walk + the wall-hug window crawl #35) continue the mission;
    // see the report's "the seal is off-route".
    let mut best_z = f32::MAX;
    let start_tick = w.total_ticks;
    for t in 0..420u32 {
        let input = Input { forward: 1.0, ..Default::default() };
        w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if p[2] < best_z { best_z = p[2]; }
        assert!(p[1] > 19.0, "the seal walk fell: bond {p:?}\n{}",
            walker::fail_dump_pub(&w.scene, [-24.4, 34.8, -21.0], "the seal walk"));
        if t % 100 == 0 { eprintln!("[05r seal t{t}] bond {:?} g {}", p, sim.bond.grounded); }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05r] the seal walk's end: tick {} (+{}) bond {:?} best_z {best_z:.2}",
            w.total_ticks, w.total_ticks - start_tick, sim.bond.pos);
        // The seal: the band's north faces hold every lane (best z never crosses −20.0).
        assert!(best_z > -20.0,
            "the band's north seal held every lane [L the wedge runs]; best z {best_z:.2} — if that \
             crossed z −20.5 the collision grew a route the walk's lanes could not take");
        assert!(!sim.disabled, "no cut-scene fires on the walked legs");
    }

    // ---- THE WINDOW-LEDGE BYPASS + THE WALL-HUG WINDOW CRAWL #35 (the mission's real route,
    // all real input) ----
    // The disc's own chain [D probe_sealtris05 + probe_ledgewalk05/probe_crawl05 over the
    // disc's tri set]: the approach's west end (x −35..−34, z −19.5, floors 35.9-36.1) runs
    // onto the window ledge (x −36.26..−34, floors 36.0-36.43, n(0,0.99,−0.15)) — the ledge's
    // z −20 rise is the 45° slope (37.0..38.0, n(0,0.71,0.71) at x −35..−33) whose north edge
    // sits 0.57 over the ledge floor and rides; the ledge continues south (36.0-36.27 to
    // z −25.5) to the crawl prompt #85 zone (the crawl #35 "Wall Hug Window" at (−34.10,
    // 37.28, −31.65), band z −26.3..−37.0, prompts #84/#85 at z −37.3/−25.5 [D placements]).
    walk_legs(&mut w, &[
        ([-34.6, 36.0, -18.6], "the approach west end"),
        ([-35.2, 36.1, -20.6], "the ledge lip"),
        ([-35.3, 36.1, -22.2], "the ledge south 1"),
        ([-35.3, 36.0, -24.0], "the ledge south 2"),
        ([-34.78, 36.0, -24.85], "the prompt 85 zone"),
    ], 26.0, 900);
    // The E press enters the crawl substate 7 [M Player_Activate 0x1a5170:66-97 →
    // Player_CollCreepWall 0x196758; the type-6 prompt ctx 0x6 arms along the ledge's south
    // end [L probe_crawl05: ctx 0x6 from (−35.03, −24.25) through (−34.78, −24.845)]]. The
    // press shape is the probe's: Bond faces ALONG the crawl axis (yaw π, −z — the #35 band's
    // run [D yaw 1.571]) and the use key pulses one edge per 6 (the game's Activate is
    // edge-triggered [M Input_Action mask 4]). Two walk_to::Use attempts were REFUSED with ctx
    // armed (sub stayed 0) — the act faces the target point, which turned Bond north, off the
    // wall axis; the crawl entry needs the player's facing on the band's axis [L this file's
    // first two runs vs probe_crawl05's yaw-π press].
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
    {
        let mut crawling = false;
        for t in 0..180u32 {
            w.tick(Input::default(), t % 6 == 0);
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate == nf_game::player::SS_CREEP {
                crawling = true;
                eprintln!("[05r] the window crawl entered at tick {} bond {:?}", w.total_ticks, sim.bond.pos);
                break;
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05r] after the E: tick {} bond {:?} sub {:#x} ctx {:#x}",
            w.total_ticks, sim.bond.pos, sim.bond.substate, sim.bond.icon_ctx);
        assert!(crawling, "the E press at the prompt #85 zone must enter the window crawl substate 7 \
            [M Player_Activate 0x1a5170 → Player_CollCreepWall 0x196758], bond {:?} sub {:#x}",
            sim.bond.pos, sim.bond.substate);
    }
    // Forward crawls SOUTH to the wall's A end [M Player_Creep 0x1969d0: the input direction
    // decides the end; the crawl rides the #35 band z −26.3..−37.0 [D placements]]. The
    // probe's shape: 0.02/tick, entered at z −26.29, exited at z −36.07 [L probe_crawl05].
    let crawl_tick = w.total_ticks;
    let mut exited = false;
    for t in 0..2400u32 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate != nf_game::player::SS_CREEP && w.total_ticks - crawl_tick > 10 {
            exited = true;
            eprintln!("[05r] the window crawl exited at +{} ticks bond {:?}",
                w.total_ticks - crawl_tick, sim.bond.pos);
            break;
        }
        if t % 200 == 0 { eprintln!("[05r crawl t{t}] bond {:?} sub {:#x}", sim.bond.pos, sim.bond.substate); }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(exited, "the window crawl must exit at the wall's south end [M Player_Creep 0x1969d0], bond {:?}", sim.bond.pos);
        assert!(sim.bond.pos[2] < -35.0 && sim.bond.pos[1] > 34.0 && sim.bond.pos[1] < 39.0,
            "the crawl must carry Bond south along the #35 band (z −26.3..−37.0 [D placements], \
             the exit stands at the band's south end z ~−36 [L probe_crawl05]), got {:?}", sim.bond.pos);
    }
    eprintln!("[05r] the walked legs complete in {} ticks; falls {:?}",
        w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[05r] fall: {f}"); }
    for st in &w.report.stuck_events { eprintln!("[05r] stuck: {st}"); }
}

/// Raw full-forward steer along a waypoint chain: real inputs only (turn + forward, with an
/// auto-unstuck sidestep burst), a per-leg wedge guard and a fall guard. The kink-walk shape.
fn walk_legs(w: &mut Walker, targets: &[([f32; 3], &str)], min_y: f32, max_leg_ticks: u32) {
    let mut leg = 0usize;
    let mut ticks_on_leg = 0u32;
    let mut side = 0i32;
    while leg < targets.len() {
        let (at, name) = targets[leg];
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(p, at) < 0.7 {
            eprintln!("[05r] {name}: +{ticks_on_leg} ticks bond {:?}", p);
            leg += 1;
            ticks_on_leg = 0;
            side = 0;
            continue;
        }
        let desired = (at[0] - p[0]).atan2(at[2] - p[2]);
        let err = walker::wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let burst = ticks_on_leg > 120 && (ticks_on_leg - 120) % 146 < 26;
        let (forward, strafe) = if burst {
            (0.35, if side % 2 == 0 { 1.0 } else { -1.0 })
        } else {
            (1.0, 0.0)
        };
        if ticks_on_leg > 120 && (ticks_on_leg - 120) % 146 == 0 { side += 1; }
        let input = Input { turn: err.clamp(-0.07, 0.07), forward, strafe, ..Default::default() };
        w.tick(input, false);
        ticks_on_leg += 1;
        let p1 = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(p1[1] > min_y,
            "leg {name} fell (centre y {:.2} < {min_y}): bond {p1:?}\n{}",
            p1[1], walker::fail_dump_pub(&w.scene, at, name));
        assert!(ticks_on_leg <= max_leg_ticks,
            "leg {name} wedged at bond {p1:?}\n{}", walker::fail_dump_pub(&w.scene, at, name));
    }
}

/// A running jump across a notch: full forward + the jump press on the edge (the harness's
/// edge-trigger rule, the notch grab's shape [M Input_Action mask 4 / Player_HandleJump]).
fn run_jump(w: &mut Walker, at: [f32; 3], name: &str, max_ticks: u32, min_y: f32) {
    for t in 0..max_ticks {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(p, at) < 0.9 {
            eprintln!("[05r] {name}: +{t} ticks bond {:?}", p);
            return;
        }
        let desired = (at[0] - p[0]).atan2(at[2] - p[2]);
        let err = walker::wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: 1.0, jump: t % 6 == 0, ..Default::default() };
        w.tick(input, false);
        let p1 = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(p1[1] > min_y, "{name} fell into the notch: bond {p1:?}\n{}",
            walker::fail_dump_pub(&w.scene, at, name));
    }
    panic!("{name} never arrived\n{}", walker::fail_dump_pub(&w.scene, at, name));
}

/// Sister: the notch is a real obstacle and the grab needs its zone. From the notch's north
/// edge (reached through the walked gap route, `walk_the_gap_to_the_notch`), a walk south with
/// NO grab press must not reach the ledge's south piece (the notch's z 4-10 dip to y 17-25
/// [D the 1-unit map] holds him) [M Player_HandleJump 0x1916b0:126-147 — the ctx scan only runs
/// with the icon armed; the walk presses nothing].
#[test]
fn the_notch_holds_the_walk_and_the_grab_needs_its_zone() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    // The shared cable route (the same walked legs as the main test — the sister only cares
    // about the notch's south edge).
    walk_to_the_slab_east_end(&mut w);
    walk_the_gap_to_the_notch(&mut w);
    // A plain south walk into the notch: no grab press. Bond must stay north of the south
    // piece (z ≥ 4) — the dip holds him [D the 1-unit map].
    let mut crossed = false;
    for t in 0..600u32 {
        let input = Input { forward: 1.0, ..Default::default() };
        // SOUTH = −z here (the notch's dip is at z 4-10, south of the walked ledge-18 edge at
        // z 18.5): the port's forward is (sin yaw, cos yaw) [M walk_to's own atan2(dz, dx)
        // steering], so yaw π faces (0, −1). The old FRAC_PI_2 faced +x (east — off the ledge's
        // east void) and predates the kink walk; the test never ran (ignored since birth).
        w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t % 100 == 0 { eprintln!("[05s notch t{t}] bond {:?}", sim.bond.pos); }
        if sim.bond.pos[2] < 4.0 && sim.bond.grounded { crossed = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!crossed, "the notch must hold a walk without the grab [D the z 4-10 dip], bond {:?}", sim.bond.pos);
    }
    // The jump press here IS in the icon zone (#310's reach covers the notch edge) — the grab
    // fires; the sister pins the WALK claim above, so stop before riding.
    eprintln!("[05s] the notch sister done at tick {} bond {:?}", w.total_ticks, w.scene.sim.as_ref().unwrap().bond.pos);
}
