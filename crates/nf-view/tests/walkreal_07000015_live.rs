//! 07000015 (the missile base silo) walked with the player's own movement — no `bond.pos`
//! writes after the spawn. GREEN end to end (level-15-complete): the spawn plate #484 fires ch
//! 0x4b under Bond; the gantry west and the NAV GRAPH's own silo crossing (the z≈−5 lane threads
//! the silo hole; the direct z≈0 line falls at x 67.45 [D floor grid]) → the north lane through
//! doors [38]/[39]/[46]/[47] → the x34.6 spine's west plank (the door[45] keypad slot 0xc8 has no
//! writer on the level — the east plank dead-ends [D trigger dump]) → door[32]'s doorway round
//! the console (map54 #145) on the NAV GRAPH's east→north→west lane → door[29] → the west room →
//! door[52] → the south corridor doors [50]/[42]/[51] (the great hall's middle pit crosses on the
//! east bridge; the z 4..0 split on the east lane) → door[49] → the USE exit #445 at
//! (0.0, 41.3, −24.8) ([M Player_Activate 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318];
//! `Trigger_Update` case 10 never self-fires with gate 0 [M 0x1b1a78:162-166]) → the load into
//! 07000016. The objective — MissionData row 21's ONLY record (level-15-every-objective [D
//! work/elf/ACTION.ELF MissionData vaddr 0x2a4350, row 21 @ 0x2a42d8: level 0x07000015, count 1;
//! obj 0: text 0x0400004d, fail 0xffffffff (none), hint 0x04000054, ch 0x8e, init 0, enable
//! 0x4b, flags 0 — byte-for-byte the port's c07000015::DEF]) — is "Find the exit to Omega
//! Sector" (complete ch 0x8e [D row 21; L ee_start_05 @ 0x2a42d8]). Mission_MonitorObjectives
//! folds state 3 → 4 (complete) only on the record's channel [M 0x1827d8 case 3], and it
//! completes on 07000016's spawn plate #252 [D 07000016], so the walk carries its own
//! live channels into the fresh 07000016 load and finishes on that plate (the game's own
//! arrival spot — the spawn (5.17, 0.90, −65.30) stands inside the plate's box). No cut-scene
//! holds the route: the level's only script holders are the guards' own scripts; every trigger
//! the route crosses is a touch plate (0xf6/0xfa/0xfb/0xfc/0xfe/0xff/0xec/0xc7/0xed/0x4b) and
//! they all fire [D trigger dump]. The keypad-locked doors [37] 0xf7 / [35] 0xc8 are bypassed
//! by the north lane.
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance — the garrison's pain pipeline still runs). All 20
//! guards stay live and armed. Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn walk_spawn_to_the_omega_sector_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 93.0 && p[0] < 96.0 && p[1] > 40.0 && p[1] < 43.0 && p[2] > -2.0 && p[2] < 2.0,
            "the cold start spawns on the silo gantry (#1 [D]), got {p:?}");
    }
    w.wait_enabled(3000, "the silo arrival");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4b), "the spawn plate #484 fires the record's enable ch 0x4b [D row 21 enable 0x4b]");
        assert_eq!(sim.drones.mission.states[21].len(), 1,
            "the mission data lists exactly ONE objective for 07000015 [D MissionData row 21 count 1]");
        assert_eq!(sim.drones.mission.states[21][0], 3, "'Find the exit to Omega Sector' announces [M 0x1827d8 case 1 → 3 on enable ch 0x4b; L record 15]");
    }

    // The route: the spawn gantry west, then the NAV GRAPH's own route to the exit hall (the
    // z≈0 lane crosses the silo hole at x 66-74 — the walk's first run fell in; the guards' AI
    // paths are the game's own answer for the gantry maze), then the USE exit.
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");

    // The north lane through the grid, then the west lane south, then the exit hall. The x34.6
    // spine SPLITS at z 30: the east plank (x38-44) dead-ends at door[45]'s keypad slot (ch_lock
    // 0xc8 — no writer on the level [D scans; the trigger dump has no AND] — permanent), and the
    // first runs fell into the lower deck (y −3.9) at (33.7, 38.8). The WEST plank (x 24-34,
    // z 28-42, the floor grid) carries the route around the slot to door[46].
    let route: &[(&str, Wp)] = &[
        ("door[38] north", Wp::new(34.6, 41.4, 16.0).r(1.2)),
        ("door[39] north", Wp::new(34.6, 41.4, 28.0).r(1.2)),
        ("the west plank", Wp::new(29.0, 41.4, 31.0).r(1.2)),
        ("the plank north", Wp::new(28.0, 41.4, 35.0).r(1.2)),
        ("the plank north 2", Wp::new(28.5, 41.4, 39.0).r(1.2)),
        ("the spine north", Wp::new(30.5, 41.4, 42.0).r(1.2)),
        ("door[46] north", Wp::new(34.6, 41.4, 46.5).r(1.2)),
        ("door[46] through", Wp::new(34.6, 41.4, 52.0).r(1.2)),
        ("door[47] north", Wp::new(34.6, 41.4, 65.0).r(1.2)),
        // The desk corner: the console placement (map54 #145 at (27.95, 41.59, 69.83), a real
        // collision strip x 26..30.5 at z ≈ 70 [D placement + the probe's floor grid '^' band])
        // walls the direct south-east approach to door[32]'s threshold, and the shaft hole
        // x 25..27 z 67..68 walls the south detour. The NAV GRAPH's own lane (the game's AI
        // route, walkreal_probe15::probe_nav_route_to_exit) wraps east→north→west and reaches
        // the doorway from the north at (26.15, 71.18) — the wall runs north-south at x ≈ 25
        // (the leaf sph (25.10, 41.51, 71.19) r 1.76 [D door 32]), so the passage spans
        // z 69.4..72.9. The player walks the same way round the desk.
        ("the nav lane east", Wp::new(37.6, 41.4, 66.3).r(1.2)),
        ("the nav lane north", Wp::new(41.6, 41.4, 69.2).r(1.2)),
        ("the nav lane top", Wp::new(41.9, 41.4, 71.2).r(1.2)),
        ("the nav lane west", Wp::new(37.6, 41.4, 76.9).r(1.2)),
        ("the nav lane southwest", Wp::new(30.5, 41.4, 74.6).r(1.2)),
        ("the doorway north side", Wp::new(28.7, 41.4, 71.5).r(1.0)),
        ("door[32] threshold", Wp::new(26.2, 41.4, 71.2).r(0.9)),  // in E reach of the leaf (head→pos d² 5.3 < 6.25)
        ("door[32] through", Wp::new(23.0, 41.4, 71.2).r(0.9)),
        ("door[32] west-top", Wp::new(20.0, 41.4, 70.0).r(1.2)),
        // The west lane: door[29]'s doorway sits in the SAME north-south wall pattern (leaf sph
        // (10.56, 41.53, 71.19) [D door 29]); the old (4.0, 70.0) leg hung over the void west of
        // x 5 (the probe's floor grid — z 69..73, x < 5 is a hole) and wedged the walker at the
        // edge. The lane: through the doorway, then southwest through the west room (x 0..8,
        // z 61..67) to door[52] (1.05, 60.67), the corridor's north door.
        ("the west lane", Wp::new(13.0, 41.4, 71.0).r(1.2)),
        ("door[29] threshold", Wp::new(11.5, 41.4, 71.2).r(0.9)),
        ("door[29] through", Wp::new(8.0, 41.4, 71.2).r(1.0)),
        ("the southwest jog", Wp::new(6.0, 41.4, 68.5).r(1.0)),
        ("the west room", Wp::new(6.0, 41.4, 66.0).r(1.2)),
        ("the west room south", Wp::new(1.0, 41.4, 63.0).r(1.2)),
        ("door[52] threshold", Wp::new(1.0, 41.4, 61.5).r(0.8)),
        ("door[52] through", Wp::new(0.5, 41.4, 59.0).r(1.0)),
        // The south corridor (x −3..4): doors [50] z 46.1, [42] z 25.1, [51] z 10.5, [49]
        // z −10.5 — all kind 2, unlocked, ch 0 [D doors 50/42/51/49]. The great hall
        // (z 30..42) has a middle pit (z 32..38, x −4..4 void [D floor grid]) — the east
        // bridge x 5..8 carries the route. The z 4..0 split (the middle x −2..2 is void) —
        // the east lane x 5..8. Then the exit hall mouth (z −8, x 0..8) funnels to door[49].
        ("the corridor south", Wp::new(0.0, 41.4, 48.0).r(1.2)),
        ("door[50] threshold", Wp::new(0.0, 41.4, 47.0).r(0.8)),
        ("door[50] through", Wp::new(0.0, 41.4, 44.0).r(1.0)),
        ("the hall east", Wp::new(5.0, 41.4, 42.0).r(1.2)),
        ("the east bridge", Wp::new(6.5, 41.4, 34.0).r(1.2)),
        ("the hall south", Wp::new(2.0, 41.4, 30.0).r(1.2)),
        ("door[42] threshold", Wp::new(0.0, 41.4, 26.0).r(0.9)),
        ("door[42] through", Wp::new(0.0, 41.4, 23.0).r(1.0)),
        ("door[51] threshold", Wp::new(0.0, 41.4, 11.5).r(0.9)),
        ("door[51] through", Wp::new(0.0, 41.4, 8.5).r(1.0)),
        ("the split east", Wp::new(5.0, 41.4, 4.0).r(1.0)),
        ("the split south", Wp::new(5.0, 41.4, -4.0).r(1.0)),
        ("the hall mouth", Wp::new(1.0, 41.4, -7.0).r(1.0)),
        ("door[49] threshold", Wp::new(0.0, 41.4, -9.3).r(0.9)),
        ("door[49] through", Wp::new(0.5, 41.4, -12.5).r(1.0)),
        ("the exit hall", Wp::new(0.5, 41.4, -21.0).r(1.2)),
        ("the exit approach", Wp::new(-0.6, 41.4, -21.0).r(1.4)),
    ];
    w.walk_route(route);

    // The USE exit #445: face the plane, pulse E (the use sphere: head + look, radius 1 [M 0x1a5170]).
    let exit_at = [0.0f32, 41.3, -24.8];
    w.walk_to(Wp::at(exit_at[0], exit_at[1], exit_at[2] + 1.2, Act::Use { use_ticks: 300 }).r(1.0), "the USE exit #445");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_0016),
            "E at the #445 plane loads 07000016 [D placement #445 param 0]");
        assert!(!sim.drones.channels.on(0x42), "nothing writes ch 0x42 on this level [D scans] — the gated gate #343's channel stays cold");
        assert_eq!(sim.drones.mission.states[21][0], 3,
            "the objective stays announced on 07000015 (its completing channel 0x8e is 07000016's spawn plate [D])");
    }

    // The objective's completion, the way the game does it: the exit loads 07000016 carrying the
    // player's channel state (the mission's persistent gates — `RamSave::capture`'s Channels arm
    // [M Trigger_Activate 0x1b2318:97-116 serves the carried save on the load]) and its spawn
    // plate #252 (4.14..6.21, −0.19..1.88, −67.76..−63.58 [D 07000016]) fires ch 0x8e, record
    // 21's completing channel. The walk carries ITS OWN live channels — the exact state this
    // walk earned — and finishes on the plate, where the game's own arrival spawn (5.17, 0.90,
    // −65.30 [D start #1]) already stands inside the plate's box.
    let carried = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.channels.clone()
    };
    let save = nf_game::player::RamSave::capture(
        &Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &carried);
    let mut w16 = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&save)).unwrap());
    w16.wait_enabled(3000, "the launch-centre arrival");
    w16.wait(200);
    {
        let sim = w16.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8e), "the spawn plate #252 fires ch 0x8e [D 07000016]");
        assert!(sim.drones.mission.states[21].iter().all(|&s| s == 4),
            "the carried record 21 completes — 'Find the exit to Omega Sector' is done");
        assert_eq!(sim.drones.mission.states[22][0], 3,
            "07000016's own objective announces on the arrival [L record 22 obj 1]");
    }
}
