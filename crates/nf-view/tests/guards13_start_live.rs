//! level-13-guards-start-alerted: Phoenix Fire Part 4's lobby guards start unaware.
//!
//! Fresh 07000013 load (Agent difficulty — the level's own part record [L c07000013.rs]): the two
//! difficulty-gated, non-template lobby guards (#318/#319 of the placement census — skins
//! 0x05000008, dtype param 15 = 3, behaviour count param 16 = 3 → mode 7 via GetDTYPENEW
//! [M 0x14cbd8 → 0x14c938 case 3]; the other ten placements are difficulty-gated [M
//! Drone_Create 0x136be0:49] or spawner-group templates deleted at load [M DroneSpawner_Init
//! 0x137aa0:56-63]) must sit blind in Global through the intro and only reach the game's own
//! wake chain afterwards.
//!
//! Pins:
//! - The intro ScriptPlayer #65 (script 0x060006D7 [D levels.md §5.1]) freezes the NPCs with its
//!   stream-6 event 5 at script tick 0 and re-enables at tick 349 [D stream decode]; while frozen
//!   (`NPCGlobals = 0` [M Script_EventHandler 0x1d94c0 case 5]) the guards take no first ENTER and
//!   no sight pass.
//! - Even unfrozen, a guard with no vision bit has NO sight pipeline: `DroneVision_ProcessDroneSight`
//!   skips every drone without `+0x4F8 & 0x10000` [M 0x176a28 first/third loops], and the visibility
//!   value is computed only under the same bit — there is no alertness bypass, whatever the
//!   placement's prop-0x20 alertness (0x2f5cc8 = {0, 0, 0.66, 1.0} [L ee_start_05.bin @0x2f5cc8]).

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_lobby_guards_start_unaware_and_wake_through_the_game_chain() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000013.bin", false).expect("load 07000013");
    let sim = scene.sim.as_mut().unwrap();
    // Difficulty 1 roster [M 0x136be0:49 p2 gate + 0x137aa0's template deletion]: exactly two guards.
    let alive: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| sim.drones.list[i].present).collect();
    assert_eq!(alive.len(), 2, "the p2<=1 non-template roster is #318/#319 [D census params]");
    for &i in &alive {
        let dr = &sim.drones.list[i];
        assert_eq!(dr.skin, 0x0500_0008, "the difficulty-1 lobby guards are skin 0x05000008 [D census]");
        assert_eq!(dr.mode, 0x7, "behaviour count param 16 = 3 → GetDTYPENEW case 3 → mode 7 [M 0x14c938:101-110]");
        assert_eq!(dr.switch_channel, 0x28, "wait channel param 3 = 0x28 [D census]");
    }
    // Through the intro freeze: Global, blind. [D event-5 stream + M 0x1d94c0 case 5; M 0x176a28]
    for &t in &[30u32, 200] {
        while sim.drones.frame < t { sim.step_bond(Input::default(), false, false, false); }
        for &i in &alive {
            let dr = &sim.drones.list[i];
            assert_eq!(dr.state, nf_game::drone::state::GLOBAL, "guard frozen in Global at tick {t} (no first ENTER under the event-5 freeze [M 0x1d94c0 case 5])");
            assert!(!dr.sighted, "no sight latch while the vision bit is clear [M 0x176a28]");
            assert_eq!(dr.visibility, 0.0, "no visibility value without the 0x10000 bit — no alertness bypass [M 0x176a28]");
        }
    }
    // The spawn touch fired during the freeze [D TouchOnce #58 out 0x28 at the spawn]: the channel is
    // on by the time the freeze lifts, so the wake goes straight to the start state — never WaitSwitch
    // [M 0x15b830's +0x136 arm reads the channel at Global ENTER; M 0x15b960 case 3].
    while sim.drones.frame < 240 { sim.step_bond(Input::default(), false, false, false); }
    assert!(sim.drones.channels.on(0x28), "the spawn touch #58 writes 0x28 [D]");
    while sim.drones.frame < 420 { sim.step_bond(Input::default(), false, false, false); }
    for &i in &alive {
        let dr = &sim.drones.list[i];
        assert_ne!(dr.state, nf_game::drone::state::WAIT_SWITCH,
            "ch 0x28 is already on at the wake — Global ENTER must not park in WaitSwitch [M 0x15b830]");
        // The Tower-mission resume [M 0x168910:23-31]: full alertness (+0x50c = 1.0, prop 0x20 = 3
        // [D census param 19; L 0x2f5cc8[3] = 1.0]) KEEPS the Attack resume — the wake never lands in
        // Alert 0x05 (the pre-fix port lowered it unconditionally).
        assert_ne!(dr.state, 0x05, "EnemyRunToPoint's 0x13 clause keeps Attack at alert >= 1.0 [M 0x168910:27-30]");
        assert_ne!(dr.state, 0x00, "the guard woke into its start-state chain [M 0x15b960:33-45]");
    }
}
