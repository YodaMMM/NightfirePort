//! walkreal: 07000041 (the space-station approach — the scripted interlude wrapper) walked with
//! the player's own movement — no `bond.pos` writes after the spawn. The route (docs/levels/
//! 07000041.md; the main map's own records [D #0/#99/#100/#81]): the airlock start #0
//! (27.22, 1.10, −30.15, the only type-0x24 start) → the station floor west/south → the
//! player-touch plate #100 (out 0x6e, mask 0x8 [D #100]) fires ch 0x6e.
//!
//! What the disc PROVES about the rest of the wrapper (pinned here, not guessed):
//! - ch 0x6e arms the drone-touch plate #99 (lock 0x6e, mask 0x4, out 0x01 [D #99]) — and nothing
//!   on the retail disc can ever fire it: the main map has NO type-0x0f drone placement [D dump],
//!   the four type-0x71 spawners carry their scale-float window at +0x2c where the game reads the
//!   spawner params [D record dump; M DroneSpawner_Create 0x1378d0's `1 < *(uint*)(rec+0x2c)`
//!   gate + Control 0x137e68's mode-2/3 switch] so no live spawner template exists, and the 7
//!   archive scripts are presentation-only (streaming static props/lights/sounds; no Event kind
//!   0x1..0xF channel writes, no 0x0f Entity) [D decoded 06000027/44/45/4f/52/c4/c5/cf/d2/ef/944].
//!   The game's own AI treats the level as guard-capable [M DroneFunc_FirstSightState 0x147c48's
//!   `GameState._12_4_ == 0x7000041` arm] — a live check below asks where the game's toucher comes
//!   from.
//! - the wrapper has NO level-exit trigger [D dump] and no script event 0x12 loadmap [D decoded]:
//!   the game's route into/out of it is the mission/outro flow, not this level's data — 0x07000016
//!   ends → outro movie 0x07300017 → GetLevelToLoadFromMovieCode returns the FRONT END 0x07000048
//!   [M Mission_Update 0x182c60 case 6 → ResetMap_LevelToLoad 0x1d3800 → ResetMap_Load 0x1d6360
//!   phase 2 → GetLevelToLoadFromMovieCode 0x1d3560's 0x73 table] → the front end starts 0x0700001b
//!   whose briefing code is 0x07100018 [M 0x1d3800's `case 0x700001b: param_1 = 0x7100018`] and
//!   whose phase-0 remap lands on 0x0700001b [M 0x1d6360 phase 0's `uVar7 == 0x7000018 →
//!   0x700001b`]. The port's host already models that hand-off (the mission-end → results/front-end
//!   drain, nf-view lib.rs); the wrapper itself contributes no exit.
//! Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The walk: the airlock start → the station floor → the plate #100 volume. Every leg through the
/// real input path (`Sim::step_bond`); the E press is the game's own use action on the doors in
/// reach. Ch 0x6e is the plate's out channel [D #100].
#[test]
fn walk_from_the_airlock_to_the_player_plate() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    assert!(
        (w.scene.sim.as_ref().unwrap().bond.pos[0] - 27.22).abs() < 0.5,
        "the wrapper starts Bond at the airlock #0 [D #0]"
    );
    // West off the airlock pad, then south onto the plate #100 volume (15.60, 0, −33.22 [D #100];
    // the old wrapper test's touch spot (15.5, 1.4, −30.0) walks straight in).
    w.walk_route(&[
        ("off the airlock west", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("the station floor", Wp::new(19.0, 1.1, -32.5).r(1.4)),
    ]);
    // South through the volume (14.51..16.71, y 0..0.75, −34.32..−32.12 [D #100]).
    w.walk_to(Wp::at(15.7, 1.1, -33.4, Act::None).r(0.8), "through the plate #100 volume");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x6e), "the player plate #100 fires ch 0x6e on the walk [D #100]");
    // The chain the channel arms, pinned to the disc: the drone plate #99 is UNLOCKED (its lock is
    // 0x6e [D #99]) — the port's lock model keeps the armed gate; and nothing else moved.
    assert_eq!(sim.drones.level_exit, None, "the wrapper has no level-exit trigger [D dump]");
    assert_eq!(sim.drones.mission_end, None, "no mission record → no mission end [M Mission_Init 0x182270:53-56 — with no mission row, BaseMap = −1 and InternalState = 5]");
    assert_eq!(sim.drones.list.len(), 0, "no placed drones at load [D main-map dump]");
}

/// Sister: walking the same floor but stopping 3 m north of the volume never fires ch 0x6e — the
/// plate is the volume's touch, not the walk itself.
#[test]
fn without_the_plate_touch_the_chain_stays_cold() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    w.walk_route(&[
        ("off the airlock west", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("north of the plate", Wp::new(15.6, 1.1, -28.0).r(1.0)),
    ]);
    for _ in 0..240 {
        let mut input = Input::default();
        input.forward = 1.0;
        w.tick(input, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x6e), "ch 0x6e stays cold without the volume touch");
    assert!(sim.drones.channels.on(0x6e) == sim.drones.channels.on(1), "ch 0x01 rides 0x6e's arm [D #99 lock]");
}

/// The wrapper's disc-side tail, pinned as data (no pass/fail guess): after the touch, the armed
/// drone plate #99 can never fire on the retail disc — no drone source exists (see the module
/// header for the [D]/[M] cites). This test pins the MECHANISM the port models: the plate's lock
/// cleared and its mask; the scripts' decoded streams carry no channel-write events; and 2400
/// ticks of live sim leave ch 0x01 cold with no exit request. The game's own route in/out is the
/// mission/outro flow (the module header's [M] chain) — a live check below asks for the wrapper's
/// real toucher on a PCSX2 boot.
#[test]
fn the_armed_drone_plate_has_no_disc_toucher() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000041.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    // The 7 scripts: presentation-only streams [D decoded]. Count the channel-writing events and
    // the drone entities across every stream of every script in the lib.
    let mut channel_events = 0usize;
    let mut drone_entities = 0usize;
    for n in sim.drones.scripts.lib.values() {
        for s in &n.streams {
            for c in s.decode() {
                match c {
                    nf_formats::nis::Cmd::Event { kind, .. } => if kind == 0x11 { channel_events += 1 }
                    nf_formats::nis::Cmd::Entity { obj_type, .. } => if obj_type == 0x0f { drone_entities += 1 },
                    _ => {}
                }
            }
        }
    }
    assert_eq!(channel_events, 0, "no script stream writes a channel [D decoded]");
    assert_eq!(drone_entities, 0, "no script stream creates a drone [D decoded]");
    // Arm the chain the way the game would (the touch), then watch: the drone plate's mask is 0x4
    // [D #99] and its lock 0x6e is now satisfied — with no drone alive the mask can never pass.
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [15.5, 1.4, -30.0], std::f32::consts::PI);
    let mut input = Input::default();
    input.forward = 1.0;
    for _ in 0..300 { sim.step_bond(input, false, false, false); if sim.drones.channels.on(0x6e) { break; } }
    for _ in 0..2400 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x6e), "the touch armed the chain");
    assert!(!sim.drones.channels.on(1), "the drone plate #99 never fires without a disc toucher [D]");
    assert_eq!(sim.drones.level_exit, None, "no exit request [D dump]");
    assert_eq!(sim.drones.mission_end, None, "no mission end [M Mission_Init 0x182270:53-56 (BaseMap = −1, InternalState = 5)]");
}

/// The movie-player trigger #81 (type 0xF4, params (0, 4, 0) [D #81]) through the REAL E path:
/// standing in its volume (16.13, 0, −0.41 [D #81]), the use key latches movie_request = Some(0)
/// — the OUT slot's whole u32 — and re-fires on every press (no spent latch [M 0x1b2318:53-61]).
/// Nothing starts a movie and nothing loads a level: the OUT slot is 0 (not in
/// psiStartBackgroundMovie's whitelist [D docs/systems/movies.md]) and the trigger path carries
/// no ResetMap call [M 0x1b2318]. A teleport reaches the volume (the game's own route to it is not
/// on this wrapper's data); the operation is the real E.
#[test]
fn the_movie_trigger_e_path_stays_a_no_op() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000041.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [16.14, 0.0, -0.41], std::f32::consts::PI);
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.drones.movie_request, None, "cold before the E");
    for _ in 0..40 { sim.step_bond(Input::default(), false, false, true); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.movie_request, Some(0), "the E latches the OUT slot's code [M 0x1b2318:53-61]");
    assert_eq!(sim.drones.level_exit, None, "no level load on the movie path [M]");
    // No spent latch: the second press re-fires [M 0x1b2318 has no spent write].
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..40 { sim.step_bond(Input::default(), false, false, false); }
    for _ in 0..40 { sim.step_bond(Input::default(), false, false, true); }
    assert_eq!(scene.sim.as_ref().unwrap().drones.movie_request, Some(0), "re-fires on the second press");
}

/// The tether crossing (§3), pinned to what the disc's own data allows: Bond WALKS the station
/// floor south to the wire #26's sphere (11.26, 2.51, −54.13 [D #26]) — and the disc gives him no
/// way to grab it: the level's only third-icon placements are #82..#85 (contexts 1/2, at
/// (7.97,·,−45.5)/(14.76,38.57,−47.54) [D #82-#85]), none near the wire, and the game's grab that
/// is certain is the JUMP press with the icon context 3 [M Player_HandleJump 0x1916b0:126-147;
/// Player_Activate 0x1a5170 has no class-0x2C arm] — so the press is a plain jump. The ride the
/// approach shows is scripted, not player-operated, on this wrapper's data. Sister: the five
/// 0x3E cables themselves parse [D #26/#36/#102/#103/#104].
#[test]
fn the_tether_wires_refuse_a_player_grab_on_the_disc_data() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.wires.len(), 5, "the tether cables parse as 0x3E wires [D]");
    }
    w.walk_route(&[
        ("off the airlock west", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("the station floor", Wp::new(19.0, 1.1, -32.5).r(1.4)),
        ("south to the tether deck", Wp::new(13.0, 1.1, -47.0).r(2.0)),
        ("under the wire #26", Wp::new(11.3, 1.1, -54.1).r(0.9)),
    ]);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded, "Bond stands on the tether deck");
        assert!(sim.bond.icon_ctx != 3, "no wire icon arms at the tether [D #82-#85 ctx 1/2]");
    }
    // The jump press with no context: a plain jump, no grab [M 0x1916b0's icon gate].
    for _ in 0..120 {
        let mut input = Input::default();
        input.jump = true;
        w.tick(input, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.bond.substate != nf_game::player::SS_WIRE && sim.bond.substate != nf_game::player::SS_ZIPLINE,
        "the tether refuses the player grab on the disc's icon set [M 0x1916b0:126-147]");
}
