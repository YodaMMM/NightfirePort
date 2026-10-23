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

/// The rest of the cable route (the roofs lane → the west lane → the wire jump-grab → the
/// crossing): STILL #[ignore]d — with the reason. The lane from the y-28 landing to the west
/// lane's head (x −27, z 44) runs partly UNDER higher surfaces (the tower/walkway at y 34-37
/// overhangs the y 27-29 roofs), so `floor_below`-based BFS cannot express it; the drive legs
/// below are coded to the MEASURED lane (the west-lane scan: z 42→13 continuous, floors
/// 28.8-35.5; the wire's z 5-8 void; the ctx-3 icons) but the roofs section between the landing
/// (59, 27, 89.6) and the lane head needs its waypoints measured cell-by-cell next pass (the
/// probe's straight-line drive walked off a roof edge into the valley at (−7.5, 64.5)). The
/// wire grab + ride legs are already written here and run green once the lane drives.
#[test]
#[ignore]
fn walk_the_west_lane_and_cross_the_cable() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    for (name, at) in [
        ("courtyard west", [62.0f32, 26.0, 96.0]),
        ("door #153", [57.8, 26.0, 89.4]),
        ("ladder #151 base", [60.9, 29.9, 89.2]),
    ] {
        w.walk_nav_to(at, name, 1.4);
    }
    w.walk_to(walker::Wp { at: [60.9, 29.9, 89.2], act: walker::Act::Ladder { top_y: 34.7 }, arrive: 0.9 }, "ladder #151");
    // the measured walk-off fall (the tower-top test pins it)
    {
        let mut landed = false;
        for t in 0..300u32 {
            let input = Input { forward: 1.0, ..Default::default() };
            w.scene.sim.as_mut().unwrap().bond.yaw = -0.3;
            w.tick(input, false);
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.grounded && sim.bond.pos[1] < 32.0 && t > 30 { landed = true; break; }
        }
        assert!(landed, "the walk-off must land on the lower roof");
    }
    // ── the roofs west, then the west lane south ──, then the west lane south ──────────────────────────────────────────────────
    // A raw-input leg: hold the yaw toward the next waypoint, real steps only. The roofs run
    // west (y 26-29) to the lane head (x −27, z 44), then the lane runs south (z 42 → 13).
    fn drive(w: &mut Walker, target: [f32; 2], max_ticks: u32, arrive: f32, label: &str) -> bool {
        let start = w.total_ticks;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let dx = target[0] - p[0];
            let dz = target[1] - p[2];
            let d = (dx * dx + dz * dz).sqrt();
            if d < arrive { eprintln!("[05 {label}] arrived at tick {} bond {:?}", w.total_ticks, p); return true; }
            if w.total_ticks - start > max_ticks {
                eprintln!("[05 {label}] TIMEOUT at {:?}", p);
                return false;
            }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.09, 0.09), forward: if err.abs() > 1.2 { 0.35 } else { 1.0 }, ..Default::default() };
            w.tick(input, false);
        }
    }
    assert!(drive(&mut w, [-20.0, 60.0], 3600, 6.0, "the roofs west"),
        "the walked roofs must reach the lane head");
    assert!(drive(&mut w, [-25.5, 44.0], 1800, 4.0, "the lane head"),
        "the west lane head (x −27..−22, z 42-47, y ~29) [D the west-lane scan]");
    assert!(drive(&mut w, [-23.5, 13.5], 3600, 2.5, "the wire grab zone"),
        "the lane's south end at the wire's north side (z 10-13, y 30.5-31.3) [D the west-lane scan]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > -27.0 && p[0] < -20.0 && p[2] > 8.0 && p[2] < 16.0 && p[1] > 28.0 && p[1] < 34.0,
            "Bond stands in the wire's north grab zone (the ctx-3 icon at (−23.80, 32.74, 12.33) r 6.02 [D probe]), got {p:?}");
    }
    // ── the cable: the jump-grab and the hand-over-hand crossing ──────────────────────────────────
    // [M Player_HandleJump 0x1916b0:118-147]: with the ctx-3 icon armed (Bond stands in the
    // icon's straddle-cel set), the JUMP press grabs the nearest class-0x2C wire within 1000 u
    // instead of jumping. The ride then runs Player_Wire 0x197658 (substate 6).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let p = sim.bond.pos;
        sim.bond.yaw = (-23.49 - p[0]).atan2(6.26 - p[2]);
        sim.bond.vel = [0.0; 3];
    }
    let mut grabbed = false;
    for t in 0..60u32 {
        let input = Input { jump: t == 5, ..Default::default() };
        w.tick(input, false);
        if w.scene.sim.as_ref().unwrap().bond.substate == nf_game::player::SS_WIRE
            || w.scene.sim.as_ref().unwrap().bond.substate == nf_game::player::SS_ZIPLINE {
            grabbed = true;
            break;
        }
    }
    assert!(grabbed, "the jump press in the ctx-3 zone must grab the wire [M Player_HandleJump 0x1916b0:118-147]");
    eprintln!("[05] the wire grabbed at tick {}", w.total_ticks);
    // The hand-over-hand crossing: the travel segment z 2.72..9.8 [M Player_Wire 0x1976f8-0x197724].
    // Hold forward (the wire move is input-driven) until the ride ends (the substate leaves).
    let grab_tick = w.total_ticks;
    let mut rode = false;
    for _ in 0..1200u32 {
        let input = Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate != nf_game::player::SS_WIRE && sim.bond.substate != nf_game::player::SS_ZIPLINE
            && w.total_ticks - grab_tick > 10 {
            rode = true;
            break;
        }
        if w.total_ticks % 100 == 0 {
            eprintln!("[05 ride] bond {:?} sub {:#x}", sim.bond.pos, sim.bond.substate);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05] after the ride at tick {} bond {:?}", w.total_ticks, sim.bond.pos);
        assert!(rode, "the ride must run and end (the cable's far side)");
        // The crossing crossed the z 5-8 void: Bond ends on the lane's SOUTH piece (z ≤ 4,
        // y 29.8-31.2 [D the west-lane scan]) — the side the ctx-3 icons at z 0.21/2.65 mark.
        let p = sim.bond.pos;
        assert!(p[2] < 6.0 && p[1] > 24.0,
            "the crossing must land Bond south of the z 5-8 void on the wire ledge (y 29.8-31.2), got {p:?}");
    }
    eprintln!("[05] cable route done in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[05] fall: {f}"); }
    for st in &w.report.stuck_events { eprintln!("[05] stuck: {st}"); }
}

/// Sister: the tower top is caged by the game's own parapet — a pure WEST walk from the ladder top
/// stalls at the parapet's inner face (x ≈ 57.9, y held ~34.7), the same stall the old report
/// measured; and a JUMP press there does a plain jump (no ctx-3 zone → no wire grab [M
/// Player_HandleJump 0x1916b0:118-147 — the ctx-3 scan only runs inside the icon's cel set]).
/// The route off the tower is the walk-off fall (the main test's leg), not a walk or a jump.
#[test]
fn the_tower_top_is_caged_and_the_grab_needs_its_zone() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    for (name, at) in [
        ("courtyard west", [62.0f32, 26.0, 96.0]),
        ("door #153", [57.8, 26.0, 89.4]),
        ("ladder #151 base", [60.9, 29.9, 89.2]),
    ] {
        w.walk_nav_to(at, name, 1.4);
    }
    w.walk_to(walker::Wp { at: [60.9, 29.9, 89.2], act: walker::Act::Ladder { top_y: 34.7 }, arrive: 0.9 }, "ladder #151");
    // walk WEST against the parapet: Bond must stall at x ≥ 57.9 with y held (the measured cage).
    let mut input = Input { forward: 1.0, ..Default::default() };
    let mut stalled = false;
    for t in 0..300u32 {
        w.scene.sim.as_mut().unwrap().bond.yaw = -std::f32::consts::FRAC_PI_2;
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t > 60 && (sim.bond.pos[0] - 57.93).abs() < 0.35 {
            stalled = true;
            break;
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05 sister] the west stall at {:?}", sim.bond.pos);
        assert!(stalled, "the west walk must stall at the parapet (bond {:?})", sim.bond.pos);
        assert!(sim.bond.pos[0] > 57.0, "Bond never passes x 57.9 on foot (got x {:.2})", sim.bond.pos[0]);
        assert!(sim.bond.pos[1] > 33.5, "the parapet holds the walk at the tower-top level (y {:.2})", sim.bond.pos[1]);
    }
    // The jump press here is a plain jump: the ctx-3 icons are 80+ u west — no grab fires.
    let input = Input { jump: true, ..Default::default() };
    w.tick(input, false);
    for _ in 0..90u32 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.substate != nf_game::player::SS_WIRE && sim.bond.substate != nf_game::player::SS_ZIPLINE,
            "no ctx-3 zone → the jump must not grab any wire [M Player_HandleJump 0x1916b0:118-147]");
    }
}
