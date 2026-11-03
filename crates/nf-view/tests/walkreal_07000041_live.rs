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

/// Real damage: the same walk with the health top-up off for every walker it makes (the
/// tests' own `top_up_health = true` included) and the walker's return fire on — Bond's death
/// panics with the death spot and the last hits (common/damage_log.rs), so a pass pins that the
/// walk completes on the port's own damage.
#[test]
fn walk_from_the_plate_to_the_tether_crossing_and_the_levels_end_real_damage() {
    walker::with_real_damage(walk_from_the_plate_to_the_tether_crossing_and_the_levels_end);
}

// ---------------------------------------------------------------------------
// level-41-every-objective: the whole wrapper played as a player, both orders.
//
// The objective-list finding the task asks for, pinned where the game reads it:
// the MissionData table (ACTION.ELF 0x2a4350, 24 records × 0x28 [L ee_start_05.bin — re-read
// row by row this session]) has **no 0x07000041 row**, so `Mission_Init` 0x182270:53-56 takes
// the no-record path (BaseMap = −1) and leaves InternalState = State = 5 [M 0x182270:199-203];
// `Mission_Update`'s case 5 is a bare no-op [M 0x182c60:361-367] and `monitor()` never runs —
// **the wrapper has zero objective records: nothing can announce and nothing can complete**.
// The mission layer's own state (base_map == u32::MAX, internal 5, state 5, no events) is what
// the assertions below check instead of a position.
//
// What the level DOES define as its play is its trigger chain [D the placements]: the chain
// trigger #99 (out ch 1, mask 0x4 = the E dispatch, lock 0x6e [D PL#99 params 0:0x1 1:0x4
// 7:0x6e]) and the walk-through plate #100 (out 0x6e, mask 0x8 player [D PL#100]). The chain
// channel's consumers are the turret #584's arm, the DynoRod #117's gate, the ctx-2 icon #105
// and the two voice triggers [D params; the audit's consumers list]. Two real orders exist and
// both are played here on real input from the fresh start:
// - `the_chain_order_played_from_the_fresh_start` — the E on #99 FIRST, then the plate: ch 1
//   rises, the chain consumers answer, the plate arms 0x6e, and the DynoRod deletes (both
//   channels [D #117 params 0:0x1 1:0x6e; M DynamicObject_Update 0x18d3b8's gate + the mode-0
//   delete]).
// - `the_plate_first_walk_locked_e_dyno_rod_tether_and_the_far_south` — the plate FIRST, no E
//   anywhere (ch 1 stays cold — the E is #99's only toucher and the walk never presses it),
//   the standing rod, the tether crossing and the far south floor. (The plate does NOT lock
//   the E out — the case-1 activate arm reads no lock [M 0x1b2318:38-49]; the older
//   "locked-out" sister in interactables was fixed this task — see the report.)
// - `the_north_detour_walks_to_the_movie_trigger_and_its_e_is_a_no_op` — the one-way north
//   walk to the movie trigger #81's volume (the older tests teleport to it) and the real E
//   there (the no-op pin).
//
// The exit boundary, named for 30-level-41-exit-event-2: the port carries a documented
// stand-in — ch 1 on for 241 ticks latches `level_exit = Some(0x0700001b)` (drone.rs, the
// "wrapper-end stand-in" block) — which the pending exit-event task owns. The chain-order walk
// therefore STOPS at the plate/rod (it reaches them inside the window and asserts the state
// there); the plate-first walk never raises ch 1 (the disc's own toucher for #99 does not exist
// [D the audit's mount scan + per-stream decode]), so it walks to the wrapper's far end with no
// exit involved. Neither test asserts the stand-in's exit value.
// ---------------------------------------------------------------------------

/// Turn Bond toward `target` on the REAL input path (turn + look, the game's own axes) until the
/// aim is inside the E's tolerance, then hold the use key until `done` says the use fired.
fn aim_and_use(w: &mut Walker, target: [f32; 3], done: impl Fn(&nf_view::scene::Sim) -> bool) -> bool {
    for _ in 0..120 {
        let sim = w.scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        let dx = target[0] - head[0];
        let dy = target[1] - head[1];
        let dz = target[2] - head[2];
        let dist = (dx * dx + dz * dz).sqrt().max(0.5);
        let want_yaw = dx.atan2(dz);
        let want_pitch = (dy / dist).atan() / std::f32::consts::FRAC_PI_2;
        let yaw_err = wrap_pi(want_yaw - sim.bond.yaw);
        let pitch_err = want_pitch - sim.bond.pitch;
        if yaw_err.abs() < 0.03 && pitch_err.abs() < 0.03 { break; }
        let input = Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            look: pitch_err.clamp(-0.05, 0.05),
            ..Default::default()
        };
        w.tick(input, false);
    }
    for _ in 0..120 {
        w.tick(Input::default(), true);
        if done(w.scene.sim.as_ref().unwrap()) { return true; }
    }
    false
}

/// The mission layer's no-record state — the game's own "there are no objectives here" [L the
/// MissionData table has no 0x07000041 row; M Mission_Init 0x182270:199-203].
fn assert_mission_layer_inert(sim: &nf_view::scene::Sim, when: &str) {
    assert_eq!(sim.drones.mission.base_map, u32::MAX,
        "{when}: BaseMap stays −1 — no MissionData row [L ee_start_05.bin @0x2a4350, 24 rows; M 0x182270:22-24, 199-203]");
    assert_eq!(sim.drones.mission.internal, 5, "{when}: InternalState 5 [M 0x182270:199-203]");
    assert_eq!(sim.drones.mission.state, 5, "{when}: Mission_Status 5 [M 0x182270 set_state(5)]");
    assert!(sim.drones.mission_events.is_empty(),
        "{when}: no objective announce/complete ever ran [M Mission_Update 0x182c60 case 5 is a no-op]");
}

/// The chain order, played from the fresh start on real input only: the E on the chain trigger
/// #99 first, then the plate #100. (The E path reads no lock — the case-1 activate arm is
/// mask&4 → the out write, nothing else [M 0x1b2318:38-49] — so both orders fire ch 1 on the
/// E; this test plays the E-then-plate order the chain's presentation implies.)
///
/// - The E fires ch 1 [D PL#99 params 0:0x1 1:0x4; M Trigger_Activate 0x1b2318 case 1 — the
///   mask&4 use dispatch while the lock is clear] — checked on the channel itself.
/// - The chain consumers answer the real channel (not the set_script stand-in the interactables
///   tests use): the turret's one shot [D #584 param 0 = 1; M Shooter_Update 0x17d0c0 — the
///   channel-on fire + the ext+0xfe flag write] and both voice triggers [D #588/#589 param 0 = 1;
///   M SoundTrigger_Update's channel gate].
/// - The plate fires ch 0x6e on the walk-through [D PL#100; M Trigger_TouchOnce 0x1b2b88].
/// - The DynoRod deletes with BOTH channels armed [D #117 params 0:0x1 1:0x6e; M
///   DynamicObject_Update 0x18d3b8's gate + the mode-0 delete] — the interactables pin used the
///   stand-in for ch 1; this is the real E path.
/// - The mission layer stays inert throughout (the no-record state above) and the walk stops
///   before the stand-in exit window (the doc above names it for 30-level-41-exit-event-2).
#[test]
fn the_chain_order_played_from_the_fresh_start() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_mission_layer_inert(sim, "at the load");
        assert!(!sim.drones.channels.on(1) && !sim.drones.channels.on(0x6e),
            "both chain channels cold at the load [D]");
    }
    // The airlock #0 west off the pad, to the chain trigger #99's spot (20.59, 0, −33.44
    // [D PL#99 — the census re-run #572]).
    w.walk_route(&[
        ("west off the pad", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("the #99 spot", Wp::new(20.59, 1.1, -32.2).r(0.7)),
    ]);
    // The E into #99's volume: ch 1 must rise — checked on the channel [M 0x1b2318 case 1].
    let fired = aim_and_use(&mut w, [20.59, 0.5, -33.44],
        |sim| sim.drones.channels.on(1));
    assert!(fired, "the E on #99 fires ch 1 [D PL#99 p0=1 p1=0x4; M 0x1b2318:38-49 case 1 — mask&4, no lock read]");
    // The consumers answer the real channel (a few ticks for the world's update order — the
    // channel write lands after the world objects' tick on the firing tick).
    for _ in 0..10 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.worldobjs.shooters.iter().all(|s| s.fired),
            "the turret fires on the real ch-1 edge [D #584 p0=1; M 0x17d0c0 the fire + the ext+0xfe flag]");
        assert!(sim.drones.sound_triggers.list.iter().all(|t| t.on == 1),
            "both voices arm on the real ch-1 edge [D #588/#589 p0=1; M SoundTrigger_Update's channel gate]");
    }
    // South-west into the plate #100 volume (14.51..16.71, 0..0.75, −34.32..−32.12 [D PL#100]).
    w.walk_route(&[("through the plate #100 volume", Wp::new(15.7, 1.1, -33.4).r(0.8))]);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x6e), "the plate fires ch 0x6e on the walk [D PL#100; M 0x1b2b88]");
        // Both chain channels armed → the DynoRod deletes [D #117; M 0x18d3b8].
        assert!(sim.drones.worldobjs.dynobjs.is_empty(),
            "the DynoRod deletes with ch 1 (the E) + 0x6e (the plate) both armed [M 0x18d3b8's gate + mode-0 delete]");
        // The walk reached the plate inside the stand-in's 241-tick window: no exit latched yet.
        // (The stand-in itself is NOT asserted — 30-level-41-exit-event-2 owns it.)
        assert_eq!(sim.drones.level_exit, None,
            "the chain-order walk reached the plate/rod inside the window; the exit stand-in is the pending task's, not asserted here");
        assert_mission_layer_inert(sim, "at the plate/rod");
        assert!(sim.drones.health.health > 0.0, "Bond alive at the chain boundary");
    }
}

/// The plate-first order, played from the fresh start on real input only: the plate, the
/// locked-out E on #99, the standing DynoRod, the tether crossing and the far south floor.
///
/// Every completion is checked on the channel/state the game reads (ch 0x6e, ch 1 COLD, the
/// mission layer's no-record state, `level_exit`), never on a position — except the walk's own
/// far reach (z < −70), which is the port's route fact like the continuation walk's. The north
/// detour to the movie trigger #81 is its own one-way test below: this route stays on the
/// proven southbound legs, because the north deck's mid-floor jams the walker's straight
/// steers — this task's probes dumped the jam at (22.8, −4.1) with CLEAR segments to every
/// target (the port's own collision [S]; the game-side geometry was never measured), and each
/// jammed lenient leg burns its 3000-tick cap.
#[test]
fn the_plate_first_walk_locked_e_dyno_rod_tether_and_the_far_south() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_mission_layer_inert(sim, "at the load");
    }
    // South-west into the plate #100 volume (14.51..16.71, 0..0.75, −34.32..−32.12 [D PL#100]).
    w.walk_route(&[
        ("west off the pad", Wp::new(24.0, 1.1, -31.0).r(1.4)),
        ("through the plate #100 volume", Wp::new(15.7, 1.1, -33.4).r(0.8)),
    ]);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x6e), "the plate fires ch 0x6e on the walk [D PL#100; M 0x1b2b88]");
        assert!(!sim.drones.channels.on(1), "ch 1 stays cold — the disc has no toucher for #99 [D the audit's mount scan]");
    }
    // East past the chain trigger #99's spot — NO E here: the disc's only toucher for #99 is
    // the player's E (mask 0x4 [D PL#99]) and this order leaves it unpressed, so ch 1 stays
    // cold. (The aimed E would fire ch 1 regardless of the plate — the case-1 arm reads no
    // lock [M 0x1b2318:38-49] — and that pins ch 1 + the stand-in exit window, which this
    // walk stays off; the decompile-true after-the-plate E is the interactables fix's leg 2.)
    w.walk_route(&[("past the #99 spot", Wp::new(20.59, 1.1, -32.2).r(0.7))]);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(1),
            "no E pressed → ch 1 stays cold on the plate-first walk [D PL#99 mask 0x4 — the only toucher is the E; the audit's mount scan]");
    }
    // The DynoRod deck: the rod STANDS (its gate ch 1 never rose) [M 0x18d3b8's gate].
    w.walk_route(&[("the DynoRod deck", Wp::new(18.8, 1.1, -43.6).r(1.6))]);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.worldobjs.dynobjs.len(), 1, "the DynoRod parses [D #117]");
        assert!(!sim.drones.worldobjs.dynobjs[0].dead, "the rod stands — the gate ch 1 is cold [M 0x18d3b8]");
    }
    // The tether crossing at the ctx-3 icon #114's spot (the continuation walk's proven legs).
    w.walk_route(&[
        ("the tether deck", Wp::new(13.0, 1.1, -47.0).r(2.0)),
        ("under the wire #26", Wp::new(11.3, 1.1, -54.1).r(0.9)),
        ("the ctx-3 icon #114 spot", Wp::new(10.48, 1.1, -59.13).r(0.25)),
    ]);
    for _ in 0..60 { w.tick(Input::default(), false); }
    let mut ctx = 0u8;
    for _ in 0..90 {
        ctx = w.scene.sim.as_ref().unwrap().bond.icon_ctx;
        if ctx == 3 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(ctx, 3, "the ctx-3 icon arms at its spot on the walk-in [M 0x196e80; D PL#114]");
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
    for _ in 0..30 { w.tick(Input::default(), false); }
    // Face yaw 0 (the strafe axis is Bond-relative [M Player_Wire 0x1977c0's Input_Action(…,1,1)]).
    for _ in 0..120 {
        let err = wrap_pi(0.0 - w.scene.sim.as_ref().unwrap().bond.yaw);
        if err.abs() < 0.05 { break; }
        w.tick(Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
    }
    // The jump grab from the standstill [M 0x1916b0:126-147].
    w.tick(Input { jump: true, ..Default::default() }, false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, nf_game::player::SS_WIRE,
        "the jump grab attaches Bond to the tether [M 0x1916b0]");
    let grab_pos = w.scene.sim.as_ref().unwrap().bond.pos;
    let mut rode = false;
    for _ in 0..480 {
        let on = w.scene.sim.as_ref().unwrap().bond.substate;
        if on != nf_game::player::SS_WIRE && on != nf_game::player::SS_ZIPLINE { break; }
        rode = true;
        w.tick(Input { strafe: 1.0, ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let ride_end = sim.bond.pos;
        eprintln!("[walkreal-41] the plate-first ride: {} end ({:.2},{:.2},{:.2})",
            if rode { "rode" } else { "instant drop" }, ride_end[0], ride_end[1], ride_end[2]);
        assert!(rode, "the traverse carried Bond [M 0x1977c0]");
        assert!((ride_end[0] - grab_pos[0]).abs() > 1.0, "the strafe moved Bond along the cable >1 u [M 0x1977c0]");
    }
    let mut grounded = false;
    for _ in 0..180 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().bond.grounded { grounded = true; break; }
    }
    assert!(grounded, "the wire drops Bond back onto the deck");
    // The far south floor (lenient: the dev-materials zone's wedges give up and re-aim — the
    // continuation walk's own precedent).
    w.lenient = true;
    w.walk_route(&[
        ("clear the ladder east", Wp::new(11.5, 1.1, -57.0).r(1.6)),
        ("the deck south of the crossing", Wp::new(8.0, 1.1, -68.0).r(2.5)),
        ("the far south floor", Wp::new(0.0, 1.1, -82.0).r(3.0)),
    ]);
    w.lenient = false;
    let sim = w.scene.sim.as_ref().unwrap();
    let end = sim.bond.pos;
    eprintln!("[walkreal-41] the plate-first walk's end: ({:.2},{:.2},{:.2})", end[0], end[1], end[2]);
    assert!(end[2] < -70.0, "the walk reaches the wrapper's far south floor, got z {:.2}", end[2]);
    // The wrapper's tail at the walk's end — the same cold tail the continuation walk pins:
    assert!(!sim.drones.channels.on(1), "ch 0x01 stays cold through the whole walk [D #99's only toucher does not exist]");
    assert_eq!(sim.drones.level_exit, None, "no level-exit trigger on the wrapper [D the placement scan]");
    assert_eq!(sim.drones.mission_end, None, "no mission record → no mission end [M Mission_Init 0x182270:53-56]");
    assert_mission_layer_inert(sim, "at the far south end");
    assert!(sim.drones.health.health > 0.0, "Bond reaches the level end alive");
}

/// The north detour as its own one-way walk: the movie trigger #81's volume (16.13, 0, −0.41
/// [D PL#81 — placement #554]) reached ON REAL INPUT from the airlock — the older tests
/// teleport to it. The route (proven by this task's probes, deterministic): the airlock #0 →
/// north off the pad (25, −24) → north-west (20, −14) → the middle leg aims at (17, −4) and
/// deterministically wedges on the port's own collision (a lenient give-up; the stuck ladder
/// carries Bond to the north-west deck at ~(6.3, 4.1) — both probe runs landed that same spot)
/// → the movie trigger's volume. Then the real E: the game's own no-op chain — the request
/// latches the OUT slot's code 0 and nothing loads [M Trigger_Activate 0x1b2318:51-62 case 0xd
/// — GameState+0x10 := 0 + the same-tick-popping GameFlow state 6 (psiMovieFinished 0x1082e8
/// pops it with no movie started) + the fade push; nothing on the arm calls
/// ResetMap_LevelToLoad]. No return leg: the pins are the volume + the E; the southbound jam
/// the probes dumped sits between here and the pad (the plate-first test's doc names it).
#[test]
fn the_north_detour_walks_to_the_movie_trigger_and_its_e_is_a_no_op() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000041.bin", false).unwrap());
    w.wait_enabled(600, "the wrapper's load");
    w.lenient = true;
    w.walk_route(&[
        ("north off the pad", Wp::new(25.0, 1.1, -24.0).r(1.4)),
        ("north west", Wp::new(20.0, 1.1, -14.0).r(1.6)),
        ("the north deck aim", Wp::new(17.0, 1.1, -4.0).r(1.6)),
        ("the movie trigger volume", Wp::new(16.13, 1.1, -0.6).r(0.9)),
    ]);
    w.lenient = false;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let d = ((p[0] - 16.13).powi(2) + (p[2] + 0.41).powi(2)).sqrt();
        assert!(d < 2.5, "the walk reaches the movie trigger #81's volume on real input, got d {d:.2} at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        assert_eq!(sim.drones.movie_request, None, "cold before the E [D]");
        assert_eq!(sim.drones.level_exit, None, "no exit before the E [D]");
    }
    // The real E on #81: the request latches, no movie, no exit [M 0x1b2318:51-62 case 0xd].
    let fired = aim_and_use(&mut w, [16.13, 0.3, -0.41],
        |sim| sim.drones.movie_request.is_some());
    assert!(fired, "the E on the movie trigger raises the request [D PL#81 mask 0x4; M 0x1b2318 case 0xd]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.movie_request, Some(0), "the OUT slot's code 0 [D PL#81 p0=0]");
    assert_eq!(sim.drones.level_exit, None, "no level load rides the movie path [M]");
    assert_mission_layer_inert(sim, "at the movie trigger");
}

/// Real damage: the plate-first walk with the health top-up off and the return fire on — the
/// only armed hazard on the route is nothing (no drones, the turret's ch-1 edge never comes,
/// the laser mine #109 sits at x 52 off the route [D PL#109]), so a pass pins the walk
/// completes on the port's own damage with Bond alive at the far south end.
#[test]
fn the_plate_first_walk_real_damage() {
    walker::with_real_damage(the_plate_first_walk_locked_e_dyno_rod_tether_and_the_far_south);
}
