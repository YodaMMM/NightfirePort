//! walkreal: 07000041 (the space-station approach — the scripted interlude wrapper) walked with
//! the player's own movement — no `bond.pos` writes after the spawn. The route (docs/levels/
//! 07000041.md; the main map's own records [D #0/#99/#100/#81]): the airlock start #0
//! (27.22, 1.10, −30.15, the only type-0x24 start) → the station floor west/south → the
//! player-touch plate #100 (out 0x6e, mask 0x8 [D #100]) fires ch 0x6e.
//!
//! What the disc PROVES about the rest of the wrapper (pinned here, not guessed):
//! - ch 0x6e arms the drone-touch plate #99 (lock 0x6e, mask 0x4, out 0x01 [D #99]) — and nothing
//!   on the retail disc can ever fire it: the main map has NO type-0x0f drone placement [D dump],
//!   the four type-0x71 placements are ROTORS, not spawners (0x1d04f0:284-290 case 0x71 →
//!   `rotor_init` 0x18f4b8; DroneSpawner_Create is case 0xf1 only and nothing calls it — the
//!   audit retracts this file's older "scale-float window" cite), and the archive's kind-7
//!   scripts (0600004f/6f/27/44/45/52/7c4 [D the archive decode; the older header cite listed
//!   anim scripts and ids that do not exist]) are presentation-only: no Event kind 0x11 channel
//!   write, no 0x0f Entity, and — the audit's sharper finding — no class-0xd9 script player with
//!   a real hash mounts anywhere on the level (the two 0xd9 SPs carry the 0x06000000 sentinel),
//!   so no script player ever runs them on the disc or in the port. The game's own AI treats the
//!   level as guard-capable [M DroneFunc_FirstSightState 0x147c48's
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
use walker::{wrap_pi, Act, Walker, Wp};

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
/// floor south to the wire #26's sphere (11.26, 2.51, −54.13 [D #26]) — and at THAT spot the disc
/// gives him no grab: the level's two 0xFA icons are #105 (ctx 2, on ch 1, at (8.59, 0, −29.47))
/// and #114 (ctx 3, always-on, at (10.48, 0, −59.13)) [D #105/#114 — the older docstring's
/// "#82..#85 third icons" cite was the kind-0x2c lightning census, the audit's correction], and
/// #114's ctx-3 reach does not extend 5 m north to the wire's midpoint, so the press here is a
/// plain jump [M Player_HandleJump 0x1916b0:126-147 — the grab arm needs BLData+0x95f == 3, set
/// by ThirdIcon_Update 0x196e80 only inside the icon's reach]. The grab itself is real where the
/// icon is: interactables' `the_tether_wire_grabs_and_carries_bond` takes #26 from the #114 spot,
/// and the continuation walk below crosses on it. Sister: the five 0x3E cables themselves parse
/// [D #26/#36/#102/#103/#104].
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
        assert!(sim.bond.icon_ctx != 3, "no wire icon arms at the tether midpoint [D #105/#114 — #114 (10.48,0,−59.13) is ~5 m south, out of reach]");
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

/// The continuation walk: from the player plate #100 the real walk goes on across everything the
/// game's own writers placed on the wrapper's floor — the DynoRod's deck (gate ch 1, action
/// 0x6e [D #117 params]), the tether crossing (the ctx-3 icon #114's spot [D #114] → the jump
/// grab [M 0x1916b0:126-147] → the hand-over-hand traverse [M Player_Wire 0x1977c0]) — and ends
/// where the wrapper's own data ends: no level-exit trigger exists [D the placement scan], the
/// mission layer is inert (no 0x07000041 row in the 24-record MissionData table [L ee_start_05.bin
/// @0x2a4350 stride 0x28] → Mission_Init 0x182270:53-56 leaves BaseMap = −1 and InternalState 5,
/// and Mission_Update's case 5 is a bare no-op), and ch 0x01 — the only channel left that could
/// drive anything — has no disc toucher (see the module header). So the walk's end IS the level's
/// end on this disc: the next level's load / mission end comes from the driving-side flow, from
/// nothing the walk can reach [D].
#[test]
fn walk_from_the_plate_to_the_tether_crossing_and_the_levels_end() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    w.walk_route(&[
        ("off the airlock west", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("the station floor", Wp::new(19.0, 1.1, -32.5).r(1.4)),
        ("through the plate #100 volume", Wp::new(15.7, 1.1, -33.4).r(0.8)),
        ("the DynoRod deck", Wp::new(18.8, 1.1, -43.6).r(1.6)),
        ("the tether deck", Wp::new(13.0, 1.1, -47.0).r(2.0)),
        ("under the wire #26", Wp::new(11.3, 1.1, -54.1).r(0.9)),
        ("the ctx-3 icon #114 spot", Wp::new(10.48, 1.1, -59.13).r(0.25)),
    ]);
    // Settle at the spot (the interactables grab stands exactly here before the press), then the
    // ctx-3 icon arms (ThirdIcon_Update 0x196e80 writes BLData+0x95f when the player is in reach
    // and its 4-tick re-arm timer is clear [M]).
    for _ in 0..60 { w.tick(Input::default(), false); }
    let mut ctx = 0u8;
    for _ in 0..90 {
        ctx = w.scene.sim.as_ref().unwrap().bond.icon_ctx;
        if ctx == 3 { break; }
        w.tick(Input::default(), false);
    }
    eprintln!("[walkreal-41] at the #114 spot: icon_ctx {ctx} pos {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    assert_eq!(ctx, 3, "the ctx-3 icon arms at its spot on the walk-in [M 0x196e80; D #114]");
    // Micro-approach: the walk-in stops within ~0.25 of the spot, and the grab segment's end is
    // right there — the attach must happen at the spot itself (the interactables grab stands at
    // (10.48, 1.4, −59.13)). Turn to the spot and close the last steps on the real path.
    for _ in 0..180 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let dx = 10.48 - p[0];
        let dz = -59.13 - p[2];
        if dx * dx + dz * dz < 0.015 { break; }
        let desired = dx.atan2(dz);
        let err = wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.08, 0.08), forward: 0.4, ..Default::default() };
        w.tick(input, false);
    }
    eprintln!("[walkreal-41] micro-approach done: pos {:?} ctx {}",
        w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.icon_ctx);
    for _ in 0..30 { w.tick(Input::default(), false); }
    // Face yaw 0 (the interactables grab's facing): the hand-over-hand strafe axis is
    // Bond-relative [M Player_Wire 0x1977c0's Input_Action(…,1,1)], so the ride direction depends
    // on the facing at the grab.
    for _ in 0..120 {
        let err = wrap_pi(0.0 - w.scene.sim.as_ref().unwrap().bond.yaw);
        if err.abs() < 0.05 { break; }
        w.tick(Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
    }
    // The jump grab from the standstill [M 0x1916b0:126-147 — the jump edge with BLData+0x95f == 3
    // takes the nearest class-0x2c object = wire #26].
    w.tick(Input { jump: true, ..Default::default() }, false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, nf_game::player::SS_WIRE,
        "the jump grab attaches Bond to the tether [M 0x1916b0]");
    // The hand-over-hand traverse [M Player_Wire 0x1977c0 — the strafe carries Bond along the
    // cable] until the wire drops him, like the interactables grab's ride.
    let grab_pos = w.scene.sim.as_ref().unwrap().bond.pos;
    let mut rode = false;
    for _ in 0..480 {
        let on = w.scene.sim.as_ref().unwrap().bond.substate;
        if on != nf_game::player::SS_WIRE && on != nf_game::player::SS_ZIPLINE { break; }
        rode = true;
        w.tick(Input { strafe: 1.0, ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let ride_end = sim.bond.pos;
    eprintln!("[walkreal-41] the ride: {} end ({:.2},{:.2},{:.2}) grounded {}",
        if rode { "rode" } else { "instant drop" }, ride_end[0], ride_end[1], ride_end[2], sim.bond.grounded);
    assert!(rode, "the traverse carried Bond [M 0x1977c0]");
    assert!((ride_end[0] - grab_pos[0]).abs() > 1.0, "the strafe moved Bond along the cable >1 u [M 0x1977c0]");
    // Bond is back on the deck south of the grab: the walk goes on across the deck's south half.
    let mut grounded = false;
    for _ in 0..180 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().bond.grounded { grounded = true; break; }
    }
    assert!(grounded, "the wire drops Bond back onto the deck");
    // The walk goes on across the deck as far as its geometry carries it (lenient: a wedged leg
    // gives up and the next re-aims — the far-south cells are the dev-materials zone, and the
    // crossing deck's walls are the port's own collision [S]). The pins that matter are the
    // wrapper's own tail below, not how deep the south cells go.
    w.lenient = true;
    w.walk_route(&[
        // The ride drops Bond beside the ladder (#480 at (6.55, 2.76, −58.14) [D]) — walk east
        // around its grab zone before turning south.
        ("clear the ladder east", Wp::new(11.5, 1.1, -57.0).r(1.6)),
        ("the deck south of the crossing", Wp::new(8.0, 1.1, -68.0).r(2.5)),
        ("the far south floor", Wp::new(0.0, 1.1, -82.0).r(3.0)),
    ]);
    w.lenient = false;
    let sim = w.scene.sim.as_ref().unwrap();
    let end = sim.bond.pos;
    eprintln!("[walkreal-41] the walk's end: ({:.2},{:.2},{:.2})", end[0], end[1], end[2]);
    // The walk's own far reach: south of the crossing the level's floor really continues (the
    // dev-materials statics stand on it [D #500-#518 at z −70..−89]), and the walk deterministically
    // covers it past z −80 (the port's collision + the walker; a route fact, not a game claim).
    assert!(end[2] < -70.0, "the walk reaches the wrapper's far south floor, got z {:.2}", end[2]);
    // The wrapper's own tail, pinned where the walk ends — exactly as cold as the disc makes it:
    // ch 0x01 never rose (no toucher for plate #99 [D the audit's mount scan + per-stream
    // decode]), no level-exit trigger exists [D the placement scan], and the mission layer is
    // inert [L/M above]. So the walk's end IS the level's end on this disc: the next level's
    // load / mission end comes from the driving-side flow, from nothing the walk can reach [D].
    // The writer-placed objects the walk passed, pinned at the walk's end:
    // - the DynoRod still stands: the action ch 0x6e armed at the plate, but its gate ch 1 never
    //   rose — the delete needs BOTH [M DynamicObject_Update 0x18d3b8's gate + mode-0 delete;
    //   D #117 params 0:0x1 1:0x6e].
    assert_eq!(sim.drones.worldobjs.dynobjs.len(), 1, "the DynoRod parses [D #117]");
    assert!(!sim.drones.worldobjs.dynobjs[0].dead, "the DynoRod stands — the gate ch 1 is cold [M 0x18d3b8]");
    // - and the wrapper's own tail stays exactly as cold as the disc makes it: ch 0x01 never rose
    //   (no toucher for plate #99 [D the audit's mount scan + per-stream decode]), no level-exit
    //   trigger exists [D the placement scan], and the mission layer is inert [L/M above].
    assert!(!sim.drones.channels.on(1), "ch 0x01 stays cold through the whole walk [D #99 mask 0x4 — no disc toucher]");
    assert_eq!(sim.drones.level_exit, None, "no level-exit trigger on the wrapper [D the placement scan]");
    assert_eq!(sim.drones.mission_end, None, "no mission record → the mission layer never ends the level [M Mission_Init 0x182270:53-56]");
}
