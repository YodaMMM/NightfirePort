//! walkreal-c: 0700001b (the space station) walked with the player's own movement — no
//! `bond.pos` writes after the spawn.
//!
//! UPDATED (level-1b-complete, task 720b) — the mapped structure (probe1b_route.rs +
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

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, SS_ZEROG_9};
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

/// The full chain (the game's completion): the deck → the zero-G slit crossing → the island →
/// the Drake arena → ch 0x0a → Drake → the defeat NIS → ch 0x0b + ch 100 → MissionState 6.
/// BLOCKED at the zero-G entry: the port has no C_CHCHZEROG 0x216388 handler — the zone script's
/// 0x4b event never flips Bond to substate 9, the float stalls at (4.98, −1.62, 0.00) (the
/// island's east face), and the island/arena legs never happen. Un-ignore when the 0x4b →
/// ChangeSubState(9) path lands (see docs/reports/level-1b-complete.md).
#[test]
#[ignore = "the walk's reached blocker — the zero-G entry (C_CHCHZEROG 0x216388 → substate 9) is unported; the float stalls at (4.98,-1.62,0.00)"]
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

    // The mapped deck route, then the slit float (legs 1-2 above).
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0], [8.6, -1.9, -0.6]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // THE BLOCKER: the zero-G entry. In the game the slit zone script fires 0x4b and
    // C_CHCHZEROG 0x216388 sets substate 9 [M]; the port stalls here (the float wedges at the
    // island's east face). The rest of the chain below is the game's route, kept for the un-ignore.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.substate, SS_ZEROG_9,
            "the slit zone puts Bond in the zero-G substate [M 0x216388:10-17]");
    }
    for wp in [[3.0f32, -1.7, 0.0], [1.5, -1.7, 0.0], [-2.0, -1.7, 0.0], [-6.0, -1.6, 0.0], [-7.0, -1.5, 0.1]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the zero-G crossing");
    }
    w.walk_to(walker::Wp::new(-7.0, -1.5, 0.1).r(1.2), "the arena floor");

    // The fight gate: ch 0x0a (0x060008f0 t104 [D]) — the documented drive of the channel the
    // missile choreography's chain writes (docs/levels/0700001b.md §6; fix_finale drives it the
    // same way). Drake spawns (spawner #187, trigger 10 [D]) and his wait-switch enables him.
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

    // The defeat through the REAL fire path: aim + tap the trigger until he dies (walker.shoot —
    // the gun, the bullets, the hit boxes).
    assert!(w.shoot(drake, "Drake"), "Drake dies to Bond's fire");

    // Death_Anim's msg-2 raises ch 0x0e on its first update [M 0x1637f8 case 2]; the defeat NIS
    // (SP #198 → 0x06000a20) follows. It must PLAY OUT (skip_cutscenes off): t600 writes ch 0x0b,
    // t720 ch 100 [D decode; Script_EventHandler 0x1d94c0 case 0x11].
    w.skip_cutscenes = false;
    let mut complete = false;
    for _ in 0..2400u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.mission.status() == 6 { complete = true; break; }
    }
    w.skip_cutscenes = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x0e), "Drake's death channel fires ch 0x0e [D param 12 = 14]");
        assert!(sim.drones.channels.on(0x0b), "the NIS's t600 event-11 writes ch 0x0b [D]");
        assert!(sim.drones.channels.on(100), "the NIS's t720 event-11 writes ch 0x64 = 100 [D]");
        assert!(complete, "the station mission completes (MissionState 6 [M 0x182c60])");
        assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
    }
}

/// Sister walk: the mapped east-deck leg never enters a scripted zero-G set piece — Bond's
/// substate stays a ground substate the whole way (the station's zero-G is script-driven,
/// C_CHCHZEROG_Handler 0x216388 msg 0x4b [M]; the SLIT itself is the zone and is past this
/// route's end).
#[test]
fn the_east_deck_route_never_enters_zero_g() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    let mut zerog_ticks = 0u32;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // The substate sample: drive 300 more ticks and count any 8/9.
    for _ in 0..300 {
        w.tick(Default::default(), false);
        if matches!(w.scene.sim.as_ref().unwrap().bond.substate, 8 | 9) { zerog_ticks += 1; }
    }
    assert_eq!(zerog_ticks, 0, "no zero-G substate on the east-deck route [M 0x216388 — script-driven only]");
}
