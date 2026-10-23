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

/// The cable route (level-05-west-lane-2, round 2 — the blocker ANSWERED, the crossing still
/// open). Round 1's reason ("every yaw sweep stalls at x ≈ 57.9-58.0 — the game's collision, the
/// port's, or the wrong lane?") is settled: it is the WRONG LANE, stopped by the game's own
/// parapet. The ladder balcony is a closed perch — its west face stops a real walk at x ≈
/// 57.7-58.2 and its south face at z ≈ 85-86 [D the 07000005 collision world: westward rays from
/// (61.6, 34.6, 89.2) hit x 57.74-58.16 at z 86-89.2; southward rays along x 58-60 hit z
/// 85.1-86.2; no walkable floor at x 52-62 south of z 84 down to y 20]. The REAL west lane is a
/// separate system, mapped by the two-hit floor map of the disc collision [D]:
/// - the lane runs from the courtyard's west strip (x 28-56, z 92, y 26.3-30.3) SOUTH along the
///   castle wall (x 24-48, z 44-92, y 28-36; the guard cover nodes #75-#78 sit on it [D]) to the
///   tower top (x 28-32, z 44-62, y 36.0) and west down its slope (x 20-24, z 52-64, y 31-34);
/// - the "Wall Hug" placements are CREEPWALLS (class 0x3F → `CreepWall_Create` 0x196fc8, the
///   crawl substate 7 — the port's creepwall.rs): "Wall Hug Short" #299 (25.79, 31.78, 67.76)
///   with the type-6 third-icon crawl prompts #256/#257 (24.2, 31.9, 60-70.6) [D], and "Wall Hug
///   Window" #35 (−34.10, 37.28, −31.65) on the window ledge;
/// - the wire #251 "Wire_Cable" (−23.49, 34.47, 6.26 [D]) crosses an 8-unit NOTCH in the cable
///   ledge (the ledge y 30.8-33.3 at x −25..−23 spans z −10..46 EXCEPT z ≈ 4-10 where the floor
///   dips to y 17-25 [D the fine map]) — the 3rd-person grab icons #277/#300 (−23.56, 32.24,
///   0.2-2.65) and the big icon #310 (−23.80, 32.74, 12.33) mark the standing grab spots at the
///   notch's edges [D];
/// - STILL OPEN (the round-3 work): the crawl legs through #299/#35 and the walk/crawl link from
///   the lane to the cable ledge (the fine map's y-30-34 pieces at x −26..−24 z 34-46, x −4..0
///   z 46-54 and x 10-24 z 54-56 are separated by 5-10-unit gaps at 2-unit probe resolution —
///   the wall-hug crawls are the suspected bridges); then the jump/E grab at the notch's edge
///   and the ride south.
#[test]
#[ignore = "level-05-west-lane-2 round 2: the wrong-lane blocker is answered (the game's own parapet — see the doc); the remaining link is the wall-hug crawls #299/#35 through the lane's narrow sections and the notch grab — the crawl legs need harness verification (round 3)"]
fn walk_the_west_ledge_and_cross_the_cable() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    // The real west lane [D the floor map in the doc comment]. The first two legs walked green
    // in the round-2 run ("courtyard west" (53.2, 27.7, 96.2); "the lane north end"
    // (44.4, 29.7, 91.1)); the bend anchors are tightened to the lane's 4-8-unit strip (the
    // round-2 direct line (44.4, 91.1) → (30, 84) cut the corner over the void at x 23-24,
    // z 88-92 and Bond fell to y 0.3 at (23.0, 92.8) — the strip is x 28-52 at z 92 but only
    // x 24-36 at z 80-86 [D the fine map]).
    for (name, at) in [
        ("courtyard west", [52.0f32, 26.0, 96.0]),
        ("the lane north end", [44.0, 28.0, 90.0]),
        ("the lane bend", [36.0, 29.5, 90.0]),
        ("the lane, wall south 1", [30.0, 31.5, 86.0]),
        ("the lane, wall south 2", [29.0, 32.6, 80.0]),
        ("the lane, tower north", [29.0, 33.7, 74.0]),
        ("the tower top", [30.0, 36.0, 58.0]),
        ("the tower west slope", [24.0, 34.0, 54.0]),
        ("the lane, west run", [12.0, 29.3, 56.0]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05c] {name}: tick {} bond {:?}", w.total_ticks, sim.bond.pos);
    }
    // The wall-hug crawl [D "Wall Hug Short" #299 (25.79, 31.78, 67.76) + the type-6 icon
    // prompts #256/#257]: face the creepwall and pulse E — the third-icon context 6 routes the
    // use into Player_CollCreepWall [M 0x1a5170:66-97] and the crawl (substate 7) carries Bond
    // along the wall's axis (yaw 1.57 + π/2 = east-west [M 0x196758:80-90]). The round-3 legs
    // continue from the crawl's west exit to the cable ledge and the notch grab.
    w.walk_to(walker::Wp { at: [25.8, 31.8, 67.8], act: walker::Act::Use { use_ticks: 90 }, arrive: 1.2 },
        "the wall-hug creepwall");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05c] after the creepwall E: tick {} bond {:?} substate {} ch113 {}",
            w.total_ticks, sim.bond.pos, sim.bond.substate, sim.drones.channels.on(113) as u8);
    }
}
