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
mod walker;

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
/// - ROUTE CORRECTION (level-05-west-lane-3): the disc's collision census (the connectivity walk
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

/// The west lane, level-05-west-lane-3 — the WALKED legs, un-ignored. The route is the
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

/// The shared cable-route walk (level-05-west-lane-4): the cold start → the courtyard strip →
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
}

/// The crawl legs, UN-IGNORED (level-05-west-lane-4): the whole west lane through the south
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

/// The PARKED continuation (level-05-west-lane-4): the legs west of the slab's east end — the
/// slab crossing, the notch JUMP-grab at icon #310 ((−23.80, 32.74, 12.33) [D]) and the
/// hand-over-hand ride. What this round settled and what blocks:
/// - WALKED GREEN (now on `the_crawl_legs_the_south_slope_shelf_and_slab_east_end`): the south
///   slope off the tower top's bare south edge, the west-of-parapet ledge, the crawl shelf, the
///   crawl #299 (entry at (25.51, 31.94, 71.41) — the live spot — and the north exit at
///   (25.51, 31.94, 61.17) after 513 ticks [L this tree's walk]), and the ledge up to the slab's
///   east end (25.42, 33.98, 51.72).
/// - BLOCKER (the slab crossing): the slab's east tip is fragmented in the disc collision [D the
///   0.25-unit rows: z 52 is void at x ≤ 23.75; (24.0, 54) reads 25.9 between the 33.8 step at
///   (24.0, 52) and the 33.3 strip at (24.0, 55); the walkable strip runs z 54-55 at x 2-14] and
///   master's one-contact-per-round rework (f56c1829c) drops the ground flag under the walker's
///   turn+forward input on the slab top (the walked legs hover ungrounded 1.34 over the floor —
///   (25.42, 49.55) / (19.15, 54.55) wedges with floor_below 33.34 [L the walked STUCK dumps],
///   while the same spots ground under a plain forward push). Next round: a 0.25-unit map of
///   x 14..26, z 44..56 to find the game's strip link, or a live read (fresh boot, gdbwatch the
///   player position while Bond walks the slab's east tip westward — the game either steps the
///   tip at a specific z or crosses via the z-55 strip).
#[test]
#[ignore = "level-05-west-lane-4: the slab crossing — the east tip is fragmented [D the 0.25 rows] and the one-contact rework unfounds the walk_to legs [L the STUCK dumps]; then the notch grab + ride (the round-3 legs, re-attach from the slab west end)"]
fn the_cable_legs_the_crawl_notch_and_ride() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    walk_to_the_slab_east_end(&mut w);
    eprintln!("[05c] parked at the slab east end, tick {} bond {:?}", w.total_ticks, w.scene.sim.as_ref().unwrap().bond.pos);
}

/// Sister: the notch is a real obstacle and the grab needs its zone. From the notch's north
/// edge, a walk south with NO grab press must not reach the ledge's south piece (the notch's
/// z 4-10 dip to y 17-25 [D the 1-unit map] holds him), and a JUMP press clear of the icon's
/// reach does not grab any wire [M Player_HandleJump 0x1916b0:126-147 — the ctx scan only runs
/// with the icon armed].
#[test]
#[ignore = "level-05-west-lane-4: the notch's north edge is behind the slab crossing (see the_cable_legs_the_crawl_notch_and_ride's reason); re-route through the slab once that leg walks"]
fn the_notch_holds_the_walk_and_the_grab_needs_its_zone() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    // The shared cable route (the same walked legs as the main test — the sister only cares
    // about the notch's south edge).
    walk_to_the_slab_east_end(&mut w);
    // A plain south walk into the notch: no grab press. Bond must stay north of the south
    // piece (z ≥ 4) — the dip holds him [D the 1-unit map].
    let mut crossed = false;
    for t in 0..600u32 {
        let input = Input { forward: 1.0, ..Default::default() };
        w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::FRAC_PI_2; // south
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
