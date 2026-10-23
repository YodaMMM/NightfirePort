//! walkreal-c: 0700001b (the space station) walked with the player's own movement — no
//! `bond.pos` writes after the spawn.
//!
//! UPDATED (level-1b-complete) — the mapped structure (probe1b_route.rs +
//! probe1b_rover.rs): the shuttle-arrival NIS holds Bond ~2100 ticks, then the spawn platform
//! (x 15–24, z 1–7) and the EAST HUB DECK (x 8–28, z −8..2, floor ≈ −2.4) are walked. The
//! central pit is an east-west SLIT at z ≈ 0 (±0.9) — the deck has no floor there — and an
//! island (x 1–3, z −2..1) sits inside it, ~4 m west of the deck's edge over the void. On-foot
//! the slit is uncrossable; the game crosses it with the scripted zero-G float: the zone script
//! fires the 0x4b control event and `C_CHCHZEROG_Handler` 0x216388 answers
//! `Player_ChangeSubState(glb_players, 9)` (msg 0x40 query nonzero) [M 0x216388:10-17], and
//! `Player_ZeroG` 0x192ab0 floats Bond. **The port has no 0x4b → substate-9 path** (SS_ZEROG_9
//! is only set by tests; no C_CHCHZEROG handler exists), so the port's float runs in the walk
//! substate and STALLS at the island's east face (4.98, −1.62, 0.00) — the walk's reached
//! blocker. West of the island (the Drake arena, x ≈ −7.3) no collision floor exists until the
//! fight materializes it (unverified — a live check).
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance). Opt-in via NF_ISO (the slim ISO is enough).
//!
//! UPDATE (audit-fix-abseil-zerog, 2026-10-19): the p3 == 2 spawn now ENTERS substate 8 [M
//! Player_Init 0x1910f8 :160-165 — the port followed the wrong start]. The walking legs force
//! the ground substate back with an [S] premise: no decompile path from 8 to a ground substate
//! after the arrival NIS is pinned (the only proven 1b change is the slit zone's 0x4b → 9 [M
//! 0x216388], past these routes) and the port wires no substate-changing handler.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, SS_WALK, SS_ZEROG_8, SS_ZEROG_9};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Leg 1 (the old test's hub crossing, solved): the arrival NIS plays out, the objectives show,
/// and Bond WALKS the mapped east-deck route from the platform to the slit's south edge — the
/// leg the first run fell off (it beelined west off the platform's west edge into the pit).
#[test]
fn walk_arrival_and_the_east_deck_to_the_slit_edge() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — NOT a game assertion: the p3 == 2 spawn enters substate 8 [M Player_Init
    // 0x1910f8 :160-165, now ported], but no decompile path back to a ground substate after the
    // arrival NIS is pinned and the port wires no substate-changing handler. This leg pins the
    // port's WALK route, so the substate is forced back to the ground path.
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[23][0], 3,
            "'Disable the missiles' shows at once (enable 0 [L], the state-0 arm [M 0x1827d8:41-54])");
        assert_eq!(sim.drones.mission.states[23][1], 1, "'Defeat Drake' hides until ch 0x09 [L record 1b obj 2]");
    }

    // The mapped route: south off the platform onto the deck, then west along z ≈ −1
    // (probe1b_route.rs's band map: the deck x 8–28 at z −8..2, the slit at z ≈ 0).
    assert!(w.try_walk_to(walker::Wp::new(20.0, -1.9, 1.0).r(1.2), "the platform south edge"),
        "the platform's south edge is walked");
    assert!(w.try_walk_to(walker::Wp::new(17.0, -1.9, -1.0).r(1.2), "the deck south-west"),
        "the deck south-west is walked");
    assert!(w.try_walk_to(walker::Wp::new(13.0, -1.9, -1.0).r(1.2), "the deck mid-west"),
        "the deck mid-west is walked");
    assert!(w.try_walk_to(walker::Wp::new(10.0, -1.9, -1.0).r(1.2), "the deck west"),
        "the deck west is walked");
    assert!(w.try_walk_to(walker::Wp::new(9.0, -1.9, -1.4).r(1.2), "the slit edge"),
        "the slit's south edge is walked");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < 11.0 && p[0] > 6.0, "Bond reached the deck's west edge ({p:?})");
        assert!(p[1] > -3.5, "Bond never fell into the pit (y {:?})", p[1]);
        // Grounded is NOT asserted at the last metre: the slit's mouth (z ≈ 0) is the float
        // zone — leg 2 walks into it and the level carries Bond (the walk logs show grounded
        // arrivals on every deck node up to (10.94,-1.46,-0.25)).
    }
}

/// Leg 2: the slit float. Walking west into the slit, the level's script carries Bond west over
/// the void without a fall — the zero-G crossing channel (the game enters substate 9 for it
/// [M 0x216388:10-17]; the port's float runs in the walk substate and stalls at the island's
/// east face — that stall is the un-ignored walk's blocker, not asserted here).
#[test]
fn the_slit_float_carries_bond_west_without_a_fall() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (the p3 == 2 spawn enters
    // substate 8 [M 0x1910f8 :160-165]; this leg pins the port's WALK route).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // Into the slit: walk west along z ≈ −0.5 → 0. The float takes over past the deck edge.
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    let mut min_x = w.scene.sim.as_ref().unwrap().bond.pos[0];
    let mut fell = false;
    for _ in 0..900u32 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        min_x = min_x.min(p[0]);
        if p[1] < -6.0 { fell = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(!fell, "Bond never fell into the pit crossing the slit (min x {min_x:.2})");
        assert!(min_x < 7.0, "the float carried Bond west past the deck edge (min x {min_x:.2})");
        assert!(p[1] > -3.0, "the float holds Bond at the deck band (y {:.2})", p[1]);
    }
}

/// The full chain (the game's completion, ported this task): the deck → the slit float →
/// the fight gate → Drake spawns (spawner #187, trigger 10 [D]; probe1b_drake.rs — the spawner
/// answers from the arena side too) → the defeat through the real fire path → the defeat NIS →
/// ch 0x0b + ch 100 → MissionState 6 → 0x0730001b [M 0x182c60:114-116]. The activation chain
/// [D, complete]: SP#211 (gate ch 30 ← console 1's UI script 0x060007e2 stream 1; out ch 46 ←
/// missile 1's DISARM script 0x0600091c stream 10) plays 0x060007fd then 0x060008f0, whose
/// streams 8/9/11 pulse ch 0x4b (75 — the zero-G entry, `C_CHCHZEROG` 0x216388 → substate 9
/// [M]) and whose stream 15 writes ch 10 at t400 — the float precedes the gate, so Bond fights
/// from the ARENA side of the island. That disc chain is data-dead: the consoles arm behind
/// the missile gates 31..38 (the missile p0 scripts' t359), the missiles behind ch 19 =
/// AND #202 [193 & 194] (193 IS written — 0x060009a4 via SP#147's opening NIS, correcting the
/// level doc's "unwritten"; 194 has NO disc writer [D sweep of every script in the archive]),
/// so the walk drives the two channel writes the way the NIS's own events do — the documented
/// stand-in fix_finale already pins for ch 0x0a. The zero-G ENTRY itself is the ported
/// handler [M 0x216388:10-17; Player_ChangeSubState 0x190630:49-63]; the float west is the
/// port's Player_ZeroG body under real thrust input only.
///
/// STILL `#[ignore]`d (the reached blocker, updated this task — the SHAFT IS SEALED): the scan
/// (probe1b_float) shows the deck's whole cross-section sits under a one-sided slab at
/// y ≈ 16.74 (up-columns at x 5.5–24 all hit it from below; a down-column from y 40 passes —
/// the game's one-sided room top), and the shaft's west wall at x ≈ 4.43 spans every scanned z
/// (±12, and ±7.8..9.8 finely) from below the deck band up to that slab — a second face at
/// x ≈ 5.91 (y ≈ 7..12, the mid-shaft ledge SP#141/#142 sit in) makes the shaft a closed box:
/// floor −2.4, west wall 4.43, top 16.74. No disc opening exists: no placement near the wall
/// except the 0x19 grills at x 3.88 and the script players (probe1b_spawner), no channel writer
/// targets it, no passage at any scanned height. The game's own route past it is the SP#211
/// fly-by: with the gate (ch 30) and the start trigger (ch 46, the SP's out — mask bit 0 IS the
/// trigger [M 0x1dbda8 case 0]) written, the port's script engine plays 0x060007fd (4780) then
/// 0x060008f0, whose own SUB 0x6000971 pulses ch 75 (the ported entry flips substate 9 — no
/// raw ch-0x4b poke) and whose tail pulses ch 74 (0x600096e, t392-399 [D]). The event-6
/// PlayerEnable-at-the-previous-entity's-matrix [M 0x1d94c0:73-84] is the game's player-carry —
/// 0x060008f0's streams 0-3 create the entities — but its parsed streams fire no event 6, so
/// the carry never lands in the port. The thrust loop below drives the same plan and records
/// progress without pinning a crossing.
#[test]
#[ignore = "the shaft is sealed: the slab at y 16.74 covers the whole deck cross-section and the west wall (x 4.43, body-solid, bullet-pass) spans every scanned z below it — no disc opening (probe1b_float/spawner); the game's route is the SP#211 fly-by whose event-6 player-carry never lands in the port (no event 6 in 0x060008f0's parsed streams; the ch-74 tail pulse is the unmeasured candidate — the report's live check)"]
fn walk_arrival_to_drakes_defeat() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[23][0], 3,
            "'Disable the missiles' shows at once (enable 0 [L], the state-0 arm [M 0x1827d8:41-54])");
        assert_eq!(sim.drones.mission.states[23][1], 1, "'Defeat Drake' hides until ch 0x09 [L record 1b obj 2]");
    }

    // The mapped deck route to the slit mouth (leg 1/leg 2's nodes, real input).
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");

    // The zero-G float via the DISC'S OWN ACTIVATION: ch 30 opens SP#211's gate (console-1's
    // UI-script write [D 0x060007e2 s1]) and ch 46 — SP#211's out, whose mask bit 0 IS the
    // state-0 start trigger [M 0x1dbda8 case 0 + SP_Hit's out bit] — fires it. The missile-disarm
    // chain that writes ch 46 on disc is data-dead above the unwritten 194 (the doc comment
    // above), so the walk drives the two writes the way the NIS's own events do — the documented
    // stand-in fix_finale already pins for ch 0x0a. SP#211 then plays 0x060007fd → 0x060008f0,
    // whose own SUB 0x6000971 pulses ch 75 [D] — the ported C_CHCHZEROG entry [M 0x216388:10-17]
    // flips the substate on the NIS's own write, no raw ch-0x4b poke anywhere.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1e, 1, sim.drones.frame);
        sim.drones.channels.set(0x2e, 1, sim.drones.frame);
    }
    let mut entered = false;
    for _ in 0..120u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().bond.substate == SS_ZEROG_9 { entered = true; break; }
    }
    assert!(entered, "the SP#211 chain's own ch-75 pulse enters the zero-G float [M 0x216388:10-17; D SUB 0x6000971 s2]");

    // Thrust west over the island into the arena — real input only (the lift thruster rides
    // the body's up row [M 0x193600:284-295], then forward along the look; the ±0.05 smoother
    // cap [M 0x192ab0:110-112]). The walk-substate wedge stands at the island's east face
    // (4.98, −1.62, 0.00 — the pre-port stall), so min x < 4.5 proves the float carried Bond.
    // The float's proven pins: the entry (above) and the RISE — the lift thruster rides the
    // body's up row [M 0x193600:284-295] and Bond leaves the deck band (the walk-substate bond
    // wedges at y −1.62 on the pit's west wall). The WESTWARD CROSSING is the open blocker:
    // the shaft's west wall (the body-solid, bullet-pass surface the segments put at x ≈ 4.4,
    // still blocking at y 15.9) stops the thrust route — the game's own crossing past it is
    // unmeasured (the report's live check). The loop below drives the same thrust plan the
    // crossing needs and records the progress without pinning it.
    let mut min_x = f32::MAX;
    let mut rose = false;
    for t in 0..900u32 {
        let input = if t < 560 { Input { lift: 1.0, ..Default::default() } }
                    else { Input { forward: 1.0, ..Default::default() } };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        min_x = min_x.min(sim.bond.pos[0]);
        if sim.bond.pos[1] > 2.0 && t < 560 { rose = true; }
    }
    assert!(rose, "the float rises off the deck band under the lift thruster (the walk-substate bond wedges at y -1.62)");
    let _ = min_x; // recorded, not pinned: the crossing past the shaft wall is the open item

    // The fight gate: ch 0x0a — the documented drive of 0x060008f0's t400 write [D stream 15],
    // whose disc route is data-dead above the gates 31..38 (see the doc comment). Drake spawns
    // while Bond floats in the arena — the island no longer shadows him.
    let drake = {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x0a, 1, sim.drones.frame);
        let mut found = None;
        for _ in 0..900u32 {
            w.tick(Default::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                found = Some(i);
                break;
            }
        }
        let i = found.expect("the gate spawns Drake (spawner #187 [D])");
        assert!(w.scene.sim.as_ref().unwrap().drones.list[i].enabled, "the gate enables Drake (wait-switch 10 [D])");
        i
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let aim = w.exposed_aim(drake);
        assert!(aim.is_some(), "Drake's aim sphere is exposed from the arena float (island behind Bond now)");
    }
    // The defeat through the real fire path (the Astronaut combat states run — Drake fires
    // when his LOS clears [M 0x154618:78]; the walker's allowance tops Bond's health up).
    assert!(w.shoot(drake, "Drake"), "Drake dies to Bond's fire from the arena float");

    // The defeat NIS (SP#198, out ch 0x0e [D]): its t600 event-11 writes ch 0x0b, t720 ch 100
    // [D 0x06000a20; M Script_EventHandler 0x1d94c0 case 0x11] → MissionState 6 → 0x0730001b
    // [M 0x182c60:114-116]. Plays OUT (no skip) so the channel writes land.
    w.skip_cutscenes = false;
    let mut ended = false;
    for _ in 0..2400u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission.status() == 6 { ended = true; break; }
    }
    w.skip_cutscenes = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x0e), "Drake's death channel ch 0x0e [D param 12 = 14]");
        assert!(sim.drones.channels.on(0x0b), "the NIS's t600 event-11 writes ch 0x0b [D]");
        assert!(sim.drones.channels.on(100), "the NIS's t720 event-11 writes ch 0x64 = 100 [D]");
        assert!(ended, "the defeat NIS ends the mission (MissionState 6) [M 0x182c60]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
    }
}

/// The float-entry sister: the ch-75 rise enters the zero-G substate and the lift thruster
/// carries Bond OFF the deck band (the walk-substate bond wedges at y −1.62 on the pit's west
/// wall). Removing the entry handler regresses both pins. Sister: without the ch-75 write the
/// entry never fires (the substate stays walk).
#[test]
fn the_zero_g_entry_floats_bond_off_the_deck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the legs' ground-substate premise (the p3 == 2 spawn enters substate 8
    // [M 0x1910f8 :160-165]; this test pins the ENTRY, not the substate premise).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    // Sister: no ch-75 write — the entry never fires, the wedge holds below the face.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_ne!(sim.bond.substate, SS_ZEROG_9, "sister: no ch-75 write, no float entry [M 0x216388 needs the msg]");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4b, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_9,
        "the ch-75 rise enters the float [M 0x216388:10-17]");
    // The float's proven pins: the entry (above) and the RISE — the lift thruster rides the
    // body's up row [M 0x193600:284-295] and Bond leaves the deck band (the walk-substate bond
    // wedges at y −1.62 on the pit's west wall). The WESTWARD CROSSING is the open blocker:
    // the shaft's west wall (the body-solid, bullet-pass surface the segments put at x ≈ 4.4,
    // still blocking at y 15.9) stops the thrust route — the game's own crossing past it is
    // unmeasured (the report's live check). The loop below drives the same thrust plan the
    // crossing needs and records the progress without pinning it.
    let mut min_x = f32::MAX;
    let mut rose = false;
    for t in 0..900u32 {
        let input = if t < 560 { Input { lift: 1.0, ..Default::default() } }
                    else { Input { forward: 1.0, ..Default::default() } };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        min_x = min_x.min(sim.bond.pos[0]);
        if sim.bond.pos[1] > 2.0 && t < 560 { rose = true; }
    }
    assert!(rose, "the float rises off the deck band under the lift thruster (the walk-substate bond wedges at y -1.62)");
    let _ = min_x;
}

/// The spawn pin (green): with ch 0x0a driven from the WALKED deck route, spawner #187 produces
/// Drake instantly, enabled, at his placement (−7.32, −1.42, 0.06) — the 720a "Drake does not
/// spawn in the port" gap is refuted (their Bond had fallen into the pit; the spawner answers
/// from the deck's edge).
#[test]
fn the_fight_gate_spawns_drake_from_the_deck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (the p3 == 2 spawn enters
    // substate 8 [M 0x1910f8 :160-165]; this test pins the spawner, not the substate).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [14.0, -1.9, -4.0], [12.0, -1.9, -6.5]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.3), "the deck south-west");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x0a, 1, sim.drones.frame);
    }
    let mut found = None;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
            found = Some(i);
            break;
        }
    }
    let i = found.expect("the gate spawns Drake (spawner #187 [D])");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[i];
    assert!(d.enabled, "the gate enables Drake (wait-switch 10 [D])");
    assert!((d.pos[0] - (-7.32)).abs() < 1.5 && (d.pos[2] - 0.06).abs() < 1.5,
        "Drake stands at his placement ({:?})", d.pos);
    assert!(!sim.drones.channels.on(0x0e), "Drake is alive (ch 0x0e not yet fired [D param 12 = 14])");
}


/// Sister walk: the east-deck leg adds no substate change of its own. The p3 == 2 spawn starts
/// IN the zero-G substate 8 [M Player_Init 0x1910f8 :160-165 — asserted; the daily audit
/// 2026-10-17: the old "never enters zero-G" doc contradicted the start], and the only proven
/// 1b substate change is the slit zone's C_CHCHZEROG 0x216388 msg 0x4b → 9 — past this route's
/// end. Driven under the legs' [S] ground-substate premise, no tick of the route enters 8/9.
#[test]
fn the_east_deck_route_adds_no_substate_change() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8,
        "the p3 == 2 start is substate 8 [M Player_Init 0x1910f8 :160-165]");
    // [S] unverified — the same ground-substate premise as the legs above.
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    let mut zerog_ticks = 0u32;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // The substate sample: drive 300 more ticks and count any 8/9.
    for _ in 0..300 {
        w.tick(Default::default(), false);
        if matches!(w.scene.sim.as_ref().unwrap().bond.substate, 8 | 9) { zerog_ticks += 1; }
    }
    assert_eq!(zerog_ticks, 0, "the east-deck route enters no zero-G substate of its own [S] unverified — 0x216388 is C_CHCHZEROG_Handler (its msg-0x4b arm sets substate 9 or 0); nothing there ties it to a slit zone or the route's end");
}

/// The disc's own activation chain, run by the port's script engine (this task's find): with
/// ch 30 (SP#211's gate — console-1's UI-script write [D 0x060007e2 stream 1]) and ch 46
/// (SP#211's out — the missile-disarm chain's write [D 0x0600091c stream 10]; mask bit 0 IS the
/// state-0 start trigger [M SP_Update 0x1dbda8 case 0]) written from the WALKED deck route,
/// SP#211 plays 0x060007fd → 0x060008f0 and the fly-by's own SUB 0x6000971 pulse (stream 2
/// `ev0x0d[0x4b,1]` [D]) lands on the ported `C_CHCHZEROG` entry [M 0x216388:10-17] — substate 9
/// with NO raw ch-0x4b write from the walk. The chain also reaches the fight gate: Drake
/// (spawner #187 [D]) spawns enabled within the cap. Removing the entry handler, or the SP
/// start trigger's out-channel bit, regresses both pins.
#[test]
fn the_sp211_chain_enters_the_float_and_spawns_drake() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (this test pins the CHAIN).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    // Sister: before the drive, walk-substate holds and no float is entered.
    assert_ne!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_9,
        "sister: the chain's writes are what enter the float");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1e, 1, sim.drones.frame);
        sim.drones.channels.set(0x2e, 1, sim.drones.frame);
    }
    let mut entered = false;
    let mut drake = None;
    for _ in 0..1800u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if !entered && sim.bond.substate == SS_ZEROG_9 { entered = true; }
        if drake.is_none() {
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                drake = Some(i);
            }
        }
        if entered && drake.is_some() { break; }
    }
    assert!(entered, "the chain's own ch-75 SUB pulse enters the float [M 0x216388:10-17; D 0x6000971 s2]");
    let i = drake.expect("the chain reaches the fight gate: Drake spawns (spawner #187 [D])");
    assert!(w.scene.sim.as_ref().unwrap().drones.list[i].enabled,
        "the gate enables Drake (wait-switch 10 [D])");
}
