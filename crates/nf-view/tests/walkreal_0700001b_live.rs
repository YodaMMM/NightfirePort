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
use walker::{wrap_pi, Walker};

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

/// The full chain (the game's completion): the deck → the slit float → ch 0x0a → Drake spawns
/// (spawner #187 answers the gate from the pit's edge — the arena floor is never needed;
/// probe1b_drake.rs) → the defeat through the real fire path → the defeat NIS → ch 0x0b +
/// ch 100 → MissionState 6 → 0x0730001b.
///
/// The void crossing is the game's own zero-G float: the level's script chain pulses the
/// control channel 0x4b [D 0x06000971 streams 2/3 (`ev0x0d[0x4b,1]` / `ev0x0d[0x4b,0]`) — the
/// sub-scripts the missile choreography's 0x060008a3 stream 6 and the fly-by NIS 0x060008f0
/// streams 8/9/11 play] and `C_CHCHZEROG_Handler` 0x216388 folds
/// `Player_ChangeSubState(glb_players, 9)` on it [M 0x216388:10-17; the port wires the entry on
/// the pulse's write frame — the channel→control binding is the port's model, [S] unverified,
/// listed as a live check in the report]. Substate 9 is the steerable spacewalk
/// (`Player_ZeroG` 0x192ab0 [M]: the thruster quad forward/strafe/(jump−crouch), the body-frame
/// rotation) — the player rounds the island (x 1–3, z −2..1) on its south face and lands on the
/// arena floor west of the pit.
///
/// The ch 0x4b pulse itself is DRIVEN, not played: every disc writer above it sits behind the
/// launch-order channel 19, which has no writer in the level's data [D scans — the level doc §6
/// "ch 19 is unreachable on disc data"], the same documented stand-in class as the ch 0x0a
/// fight-gate drive this test already pins (fix_finale drives it the same way). The disc's own
/// pulse shape is honoured: a one-tick rise [D 06000971 s2/s3], released the next tick.
///
/// The fight gate ch 0x0a: the documented drive of the channel the missile choreography's chain
/// writes (0x060008f0 t104/t400 [D]; docs/levels/0700001b.md §6; fix_finale drives it the same
/// way). Drake spawns INSTANTLY from the pit's edge (spawner #187, trigger 10 [D];
/// probe1b_drake.rs) and his wait-switch enables him — the arena floor is never stood on first.
#[test]
#[ignore = "the reached blocker: the sealed shaft wall — the float (substate 9, entered via the ported 0x4b handler [M 0x216388]) is capsule-stopped at x 4.98 (face ≈ 4.43) across z −8..−0.5 at y −2.4..+2.5, while the BULLET pass crosses everywhere (glass — probe1b_float::scan_the_x5_wall_openings); the game's route out is the shaft mouths at z ±8.82 (SP#141/#142's looping zero-G scripts 0x06000818) — map those entries next"]
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
        assert_ne!(sim.bond.substate, SS_ZEROG_9,
            "sister: the walk starts on the ground — the zero-G float needs the level's 0x4b pulse (the entry is load-bearing [M 0x216388])");
    }

    // The fight gate from the deck (the mapped route's south-west firing corner): ch 0x0a —
    // Drake spawns from the pit's edge while Bond is on the deck (the spawner's min-distance
    // screen [M 0x137cb0:30] refuses spots closer than min_dist, so the game's own beat fires
    // the gate from the deck, never from the arena floor).
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [14.0, -1.9, -4.0], [12.0, -1.9, -6.5]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.3), "the deck south-west");
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < 16.0 && p[2] < -3.0 && p[1] > -3.5, "Bond reached the deck's south-west ({p:?})");
    }
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

    // The mapped deck route to the slit mouth (z ≈ 0 is the float zone's mouth).
    for wp in [[14.0f32, -1.9, -4.0], [10.0, -1.9, -1.0], [8.6, -1.9, -0.6]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.3), "the slit mouth");
    }

    // The level's zero-G pulse: one-tick rise [D 06000971 s2], released the next tick [D s3].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4b, 1, sim.drones.frame);
    }
    w.tick(Default::default(), false);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4b, 0, sim.drones.frame);
        assert_eq!(sim.bond.substate, SS_ZEROG_9,
            "the pulse grabs Bond into the zero-G float (substate 9) [M C_CHCHZEROG 0x216388:10-17]");
    }

    // The steered float: round the island on its south face (the island x 1–3, z −2..1 — the
    // straight west line stalls at its east face (4.98, −1.62, 0.00)) and come down onto the
    // arena floor. All real thruster input [M Player_ZeroG 0x192ab0: forward/strafe/(jump−crouch)].
    let mut min_x = f32::MAX;
    let mut landed = false;
    for t in 0..2400u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let yaw = sim.bond.yaw;
        drop(sim);
        // The waypoint: south around the island first (its east face stops the capsule at
        // x ≈ 4.98 — the straight west line can never pass), then west over the arena, then
        // down onto the floor.
        let target: [f32; 2] = if p[0] > 6.5 {
            [5.5, -1.6] // west over the deck edge
        } else if p[2] > -3.2 {
            [4.9, -3.8] // south, clearing the island's south face (z −2)
        } else if p[0] > -4.8 {
            [-6.5, -3.8] // west along the pit's south channel
        } else {
            [-6.5, -1.6] // north onto the arena floor
        };
        let want_yaw = wrap_pi((target[0] - p[0]).atan2(target[1] - p[2]));
        let dist_t = ((target[0] - p[0]).powi(2) + (target[1] - p[2]).powi(2)).sqrt();
        let (fwd, down) = if p[0] > -4.8 {
            (1.0f32, p[1] > -1.0) // a gentle counter-thrust against the float's climb
        } else {
            (0.4f32, true) // over the arena: come down to the floor
        };
        let input = Input {
            forward: fwd,
            turn: wrap_pi(want_yaw - yaw).clamp(-0.06, 0.06),
            crouch: down,
            ..Default::default()
        };
        let _ = dist_t;
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if std::env::var("NF_FLOATLOG").is_ok() && t % 30 == 0 {
            eprintln!("[float t {t}] pos ({:.2},{:.2},{:.2}) yaw {:.2}", p[0], p[1], p[2], yaw);
        }
        min_x = min_x.min(p[0]);
        if p[0] < -4.5 && p[1] < -1.2 && sim.bond.grounded {
            landed = true;
            break;
        }
        if p[1] < -8.0 { panic!("the float fell into the void at ({p:?}) t {t}"); }
    }
    assert!(landed, "the float never landed on the arena floor (min x {min_x:.2}, at {:?})",
        w.scene.sim.as_ref().unwrap().bond.pos);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < -4.5 && p[2] > -6.0, "Bond stands in the arena basin ({p:?})");
    }

    // The fight from the arena floor: Drake's own aim sphere is exposed here (the island no
    // longer sits between Bond and his hover (−7.32, −1.42, 0.06)).
    assert!(w.shoot(drake, "Drake"), "Drake dies to Bond's fire");
    // The defeat NIS plays out and ends the mission.
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
        assert!(ended, "the NIS's t720 ch 100 ends the mission (MissionState 6) [M 0x182c60]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
    }
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
