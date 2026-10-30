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

/// The full chain (solved this task — the walk is UN-IGNORED): the deck → the slit float → the
/// westward crossing → the fight gate → Drake spawns (spawner #187, trigger 10 [D]) → the defeat
/// through the real fire path → the defeat NIS → ch 0x0b + ch 100 → MissionState 6 → 0x0730001b
/// [M 0x182c60:114-116].
///
/// THE CROSSING IS NOT A SCRIPTED CARRY — it is the boundary's own OPEN Z-END, on real-input
/// float. The prior premise ("the game's crossing past x 4.43 must be a scripted carry", the
/// SP#211 chain as the candidate) is REFUTED on the disc:
/// - The whole 0700001b script archive carries exactly ONE player-touching event row: NIS
///   0x06000a20's stream-4 ev05 (`Drone_EnableAll`) [D sweep of every kind-7 entry, decoded
///   through the 5,263-stream-validated parser]. No event 4 (`Player_Disable`), no event 6 (the
///   only player-matrix writer — `Script_EventHandler` 0x1d94c0 case 6 anchors `SCRIPTINFO+0x40`
///   at the player/entity matrix, and `Script_Free` 0x1da880 re-enables the player AT that matrix
///   under flag 0x100) fires anywhere on this level, so the script system can never move Bond.
/// - The SP#211 chain decodes clean [D]: 0x060007fd (len 4780) = one sky entity (glist 0x02000823,
///   type 31) on a 240-key track (1051,174,527)→(−187,158,52) + an event-3 hold pair + the engine
///   sound — the station-exterior fly-by; 0x060008f0 (len 470) = 8 sky entities (0x02000919..20)
///   scattering from (144,26,21) + SUBs 0x06000971 (t384-391: a type-0xe1 destroy prop + ev0x0d
///   [75,1]@t2/[75,0]@t4) and 0x0600096e (t392-399: the same with ch 74) + the fight gate
///   ev0x0d [10,1] at t400. No player-position write, no attach, no teleport: the chain pulses
///   the float channel and opens the gate, and the float physics do the rest.
/// - The boundary box (plc #128 OBJECT_Boundary_1, mat 0x40) is sealed on THREE sides only
///   [D segment probes on the ported collision]: the west wall at x 4.433 spans y −16.56..+16.74
///   for z −20..+20 (present at z ±20, ABSENT at z ±22), the shaft's top slab at y 16.74 (normal
///   down, covers x 4.5..24 — no chimney at the wall) and the pit floor at y −16.56. There are NO
///   z-facing faces anywhere at x 4.6..8 between z 12 and 30 (north→south rays at y 0/9.45/−5/−10
///   all pass): the box's z-ends are OPEN. Past z ≈ ±21 the wall simply ends.
/// - The float body cannot filter the wall [M re-verified this task: `Player_Collision` 0x194df0's
///   default arm (substate 9) writes the shape word +0x7d4 = 0x800 and never the filter word
///   +0x7d8 (only substate 15 zipline writes 0x1c0 :382), so `Collide_Filter` 0x1ed888's
///   `(surf & 0x40) && (flags & 0x800)` reject can never fire for the body] — so the crossing
///   goes AROUND the z-end: rise off the deck band, drift north inside the shaft past z 21,
///   cross west at z ≈ 22.8 (no wall, no slab, no floor there [D]), come south over the void,
///   and hover above the island. Every leg is the port's own float under real thrust input
///   [M Player_ZeroG 0x192ab0: the ±0.05-smoothed thrusters], steered by the same closed loop
///   the walk legs use (turn = the clamped yaw error [M the float's action-0 yaw rate]).
///
/// The two documented stand-ins stay (their disc routes are data-dead above the unwritten ch 194
/// [D sweep — 194 has NO writer in the archive]): the ch 30 + ch 46 drive that fires SP#211's
/// trigger, and the ch 0x0a drive for the fight gate (0x060008f0's own t400 write — the chain is
/// played, so on the port it lands too; the drive only makes the test independent of its timing).
/// The zero-G ENTRY itself is the disc's own ch-75 pulse on the ported `C_CHCHZEROG` handler
/// [M 0x216388:10-17] — no raw ch-0x4b poke anywhere.
///
/// Evidence trail for the geometry: the sister `the_boundary_wall_stops_the_float_and_passes_bullets`
/// still pins the wall body-solid at the slit's z band (a thrust at z 0 stalls at 4.98 = the wall
/// plane + the 0.55 capsule); this walk pins the z-end route past it. Both hold together: the wall
/// blocks at |z| ≤ 20 and is absent beyond ±22 [D probes].
#[test]
#[ignore = "THE SHAFT BOX IS SEALED FOR THE PLAYER BODY (this task's conclusion, every face probed): the west wall x 4.433 (plc #128 OBJECT_Boundary_1, m40, y −16.56..+16.74, z ±20.4 — body-solid [M Player_Collision 0x194df0's default arm: shape 0x800, the filter word +0x7d8 stays 0 for substate 9, so Collide_Filter 0x1ed888's 0x40 reject can never fire]), the top slab y 16.74 (normal down, x 4.5..24 — no chimney), the pit floor y −16.56, and END-WALLS at z ±20.40 (m40, both directions [D] — the_end_walls_seal_the_shaft_box pins them) whose only openings are deck-level windows 0.8..1.0 tall [D 0.2-step scans: x 7.5/8.0 open y −1.8..−1.0, x 10.0 −2.2..−1.2] — narrower than the float body (~1.58 tall [M the default arm's capsule A = pos+up·0.55, B = pos+up·(+0x7CC−cc), r 0.55; cc 1.0347 [L ee_start_05]]). The crossing is NOT a scripted carry: the whole 0700001b archive carries ONE player-touching event row (0x06000a20 s4 ev05 Drone_EnableAll) [D sweep through the 5,263-stream-validated parser] — no event 4/6 anywhere, so the script system never moves Bond (event 6 anchors SCRIPTINFO+0x40 at the player [M 0x1d94c0 case 6] and Script_Free 0x1da880 re-enables the player AT it under flag 0x100 — never armed here); the SP#211 chain decodes clean [D: 0x060007fd = the sky fly-by (one entity, 240 keys, 4780 ticks), 0x060008f0 = 8 sky entities + SUBs 0x06000971/0x0600096e (type-0xe1 props + ev0x0d [75,*]@t384-391 / [74,*]@t392-399) + the fight gate ev0x0d [10,1]@t400]. The pilots below walk the deck, enter the float on the chain's own ch-75 pulse, rise, and drift south past the deck's corner — then wedge at the end-wall inside the window band (min z −19.85 = −20.40 + 0.55). LIVE CHECKS for the main agent: (1) fresh boot 0700001b, float at the slit — does the game's Bond pass x 4.43 westward at all, and in which substate? (2) gdbwatch the float substate's HITTEST (+0x7CC/+0x7D0.. and cc at obj+0xDC+0xCC) during the crossing — is the float body a DIFFERENT (smaller) shape than the walk's? (3) watch switch_channels[74] through 0x060008f0's t392-399 — the allie​s-talk line, not a carry (NDrone2_HandleTalking 0x13f688 is its only reader [D ACTION.ELF sweep]). If the live game crosses, the missing piece is the body shape/handler the port has not seen; if it does not, the level's own route is elsewhere (the mission's 1b exit may not need the arena on foot — the defeat NIS path via fix_finale already ends the mission)."]
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
    for _ in 0..1800u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().bond.substate == SS_ZEROG_9 { entered = true; break; }
    }
    assert!(entered, "the SP#211 chain's own ch-75 pulse enters the zero-G float [M 0x216388:10-17; D SUB 0x6000971 s2]");

    // THE CROSSING — real input only, around the boundary's open z-end [D the doc comment]:
    // the same closed-loop steer the walk legs use (turn = the clamped yaw error toward
    // atan2(dx, dz) [M the float's action-0 yaw rate, Player_ZeroG 0x192ab0]), the lift thruster
    // riding the body's up row [M 0x193600:284-295]. Phases: rise off the deck band, drift north
    // inside the shaft to z ≈ 22.8 (past the wall's z-end: the wall is present at z ±20, absent
    // at ±22 [D]), cross west past the x 4.433 plane, come south over the void, hover above the
    // island. min_x < 4.4 is the crossing pin: with the wall body-solid at |z| ≤ 20 [M
    // Player_Collision 0x194df0's default arm — no filter word for the float] the plane is
    // unreachable except around its end (the sister wall test stalls at 4.98 at z 0).
    let steer = |w: &mut Walker, tx: f32, tz: f32, lift: f32| -> Input {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let desired = (tx - p[0]).atan2(tz - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() < 0.35 { 1.0 } else { 0.15 }, lift, ..Default::default() }
    };
    let mut min_x = f32::MAX;
    let mut max_z = f32::MIN;
    let mut fell = false;
    // Rise: the proven lift pin (the walk-substate bond wedges at y −1.62 on the island's face).
    let mut rose = false;
    for _ in 0..900u32 {
        w.tick(Input { lift: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().bond.pos[1] > 2.0 { rose = true; break; }
    }
    assert!(rose, "the float rises off the deck band under the lift thruster (the walk-substate bond wedges at y -1.62)");
    // THE SOUTH LEG — the [D-probed deck-level route]: the boundary box's end-walls at z ±20.40
    // carry a deck-level window at (x ≈ 7.5, y −1.6..−0.8) [D two-direction probes], the lane
    // x 7.5 is open from z −2 to −19.5 at y −1.2 (the north lane's stub at z +8.13 has no south
    // twin), the west leg is CLEAR at z −20.9/−21.5/−22.5 (the boundary wall's south end sits in
    // (−20.4, −20.8] [D]), and the island's south face (z −2, top −0.7) forces the climb before
    // the final north leg. Phases, all real input through the float's thrusters:
    let hold = |w: &mut Walker, tx: f32, tz: f32, ty: f32| -> Input {
        let sim = w.scene.sim.as_ref().unwrap();
        let lift = ((ty - sim.bond.pos[1]) * 2.0).clamp(-1.0, 1.0);
        steer(w, tx, tz, lift)
    };
    let mut min_x = f32::MAX;
    let mut min_z = f32::MAX;
    let mut fell = false;
    // (a) south at deck level, past the deck's south-west rail (m06 z −8.13 spans y −1.6..−1.0
    // only [D] — deck level y ≈ −0.2 clears it) and the deck's south edge (z −8), to z −9.6
    let mut s1 = false;
    for _ in 0..14000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[2] < -9.6 { s1 = true; break; }
        let input = steer(&mut w, 7.0, -11.0, 0.0);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        min_x = min_x.min(p[0]); min_z = min_z.min(p[2]);
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell && s1, "the float drifts south at deck level past the deck's corner (min z {min_z:.2})");
    // (b) descend BELOW both south rails (z −8.13 band −1.6..−1.0, z −14.25 band −1.9..−0.7
    // [D]; the capsule top at y −3.4 is −2.19, clear of both bottoms) — off the deck, void below
    let mut s2 = false;
    for _ in 0..8000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[1] < -3.3 { s2 = true; break; }
        let input = hold(&mut w, 7.0, -11.5, -3.6);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        min_x = min_x.min(p[0]);
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell && s2, "the float descends under the south rails (y {:.2})", w.scene.sim.as_ref().unwrap().bond.pos[1]);
    // (c) south under both rails to z −15.5
    let mut s3 = false;
    for _ in 0..14000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[2] < -15.5 { s3 = true; break; }
        let input = hold(&mut w, 7.5, -22.0, -3.4);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        min_x = min_x.min(p[0]); min_z = min_z.min(p[2]);
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell && s3, "the float passes under both south rails (min z {min_z:.2})");
    // (c2) rise into the window band y −1.2 (the end-wall window at y −1.6..−0.8 spans x ≈ 5.5..10.5 [D])
    let mut s4 = false;
    for _ in 0..8000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[1] > -1.35 && p[1] < -0.85 { s4 = true; break; }
        let input = hold(&mut w, 8.0, -17.0, -1.2);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        min_x = min_x.min(p[0]); min_z = min_z.min(p[2]);
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell && s4, "the float rises into the window band (y {:.2})", w.scene.sim.as_ref().unwrap().bond.pos[1]);
    // (c3) south through the end-wall window to z −21.2
    let mut s5 = false;
    for _ in 0..16000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[2] < -21.2 { s5 = true; break; }
        let input = hold(&mut w, 8.0, -22.0, -1.2);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        min_x = min_x.min(p[0]); min_z = min_z.min(p[2]);
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell && s5, "the float threads the south end-wall window at deck level (min z {min_z:.2})");
    // (d) west past the boundary plane (its south end is north of z −20.9 [D])
    let mut crossed = false;
    for _ in 0..8000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[0] < 4.4 { crossed = true; break; }
        let input = hold(&mut w, 2.2, -21.2, -1.2);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        min_x = min_x.min(p[0]); min_z = min_z.min(p[2]);
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell, "Bond never fell over the west void (min x {min_x:.2})");
    assert!(crossed && min_x < 4.4,
        "the float crosses the boundary plane x 4.433 around its south end [D plc #128; the wall is body-solid at |z| ≤ 20.4 — M Player_Collision 0x194df0 default arm; the end-wall window at (7.5, −1.2, −20.4) is the opening] — min x {min_x:.3}, min z {min_z:.2}");
    // (e) climb over the island's south face (top −0.7 [D]), then north to the island's band
    let mut s6 = false;
    for _ in 0..3000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[1] > 0.5 { s5 = true; break; }
        w.tick(Input { lift: 1.0, ..Default::default() }, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell && s5, "the float climbs over the island's south face");
    let mut s7 = false;
    for _ in 0..16000u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if p[2] > 0.4 { s7 = true; break; }
        let input = hold(&mut w, 2.0, 0.4, 0.5);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        min_x = min_x.min(p[0]);
        if p[1] < -6.0 { fell = true; break; }
    }
    assert!(!fell && s7, "the float comes north over the arena void to the island's band");
    let _ = rose;

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
        assert!(aim.is_some(), "Drake's aim sphere is exposed from the hover over the island, west of the boundary");
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

/// The sealed-shaft wall, decided: the face that stops the float at x ≈ 4.98
/// IS the game's own boundary, and the game's body collides with it — the port's capsule stop is
/// the game's behaviour, not a port artefact.
///
/// [D probe1b_wall] the face is plc #128 OBJECT_Boundary_1 (kind 0x8019, cel 0x8000, glist 0x0):
/// a two-triangle pair at x 4.433 spanning y −16.56..+16.74, z −20.40..+20.40 (both stored
/// normals face EAST, +x), surface byte 0x40 — present in the level archive at the slit band,
/// at the mouths' height (y 9.45) and everywhere between.
///
/// [M] the game's rules stop the player body on it: the surface byte's 0x40 sets HITDATA+0x51
/// bit 1 on every hit (Collide_Intersect 0x1ebaa8 disasm 0x1ebf78-0x1ebf80) and that bit drops
/// hits only for the HITTEST+0x98 bit-0x8 bullet queries (0x1ebff0-0x1ec010) — the player body
/// rides Collide_CylinderIntersect 0x1ecc00:61's kind 0x800, which carries no bit 0x8;
/// Intersect_CylGeom 0x109da0's per-triangle loop rejects no face by side (the head-on cull
/// :302-322 only skips faces within ~2.6° of horizontal), and Player_ZeroG 0x192ab0 composes
/// only thrust/rotation — the shared movement applies the same body collision. The z ±8.82
/// mouths do not reopen the route: the iris doors (SP#141/#142's meshes, x 5.23 [D]) sit EAST
/// of the boundary, and the boundary spans the doors' whole y band — open doors dead-end at it.
#[test]
fn the_boundary_wall_stops_the_float_and_passes_bullets() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    // The body ray east→west hits the 4.433 plane across the slit band and at the mouths'
    // height, with the boundary's surface byte [D mat 0x40].
    for (y, z) in [(-2.4f32, 0.0), (0.0, -4.0), (2.5, 0.0), (9.45, 8.82), (9.45, -8.82)] {
        let (t, n, mat, _) = sim.world
            .segment_surface_flags([5.5, y, z], [3.5, y, z], true)
            .unwrap_or_else(|| panic!("the boundary ray (y {y}, z {z}) must hit [D plc #128 at x 4.433]"));
        let x = 5.5 - t * 2.0;
        assert!((x - 4.433).abs() < 0.01, "hit at x {x:.3} = the boundary plane 4.433 [D plc #128], got t {t}");
        assert!((n[0] - 1.0).abs() < 0.01, "the stored normal faces east [D], got ({:.2},{:.2},{:.2})", n[0], n[1], n[2]);
        assert_eq!(mat & 0x3f, 0, "the boundary's surface index is 0 [D mat 0x40]");
        assert!(mat & 0x40 != 0, "the boundary carries the 0x40 surface flag [D]");
    }
    // Sisters (the game's ray rules): the bullet query passes (the 0x40 drop [M 0x1ebff0]) and
    // the west→east ray passes (the stored normals face east; the ray test skips n·dir ≥ 0
    // [M 0x10a8c8:168]).
    for (y, z) in [(0.0f32, 0.0), (9.45, 8.82), (9.45, -8.82)] {
        // The bullet is never stopped AT the boundary: the 0x40 drop [M 0x1ebff0-0x1ec010]
        // releases the hit and the ray runs on (west of the boundary the 0x19 grills at x ≈ 3.8
        // [D] can catch it — that is not the boundary).
        if let Some(t) = sim.world.segment_bullet([5.5, y, z], [3.5, y, z]) {
            let x = 5.5 - t * 2.0;
            assert!(x < 4.4, "the bullet passes the boundary westward (the 0x40 drop [M 0x1ebff0-0x1ec010]) \
                at y {y} z {z} — stopped at x {x:.3} instead");
        }
        assert!(sim.world.segment([3.5, y, z], [5.5, y, z]).is_none(),
            "the west→east ray passes the east-facing boundary [M 0x10a8c8:168] at y {y} z {z}");
    }
    drop(sim);
    // The float, real thrust only: enter at the slit mouth (the documented ch-75 stand-in for
    // the disc's own SP#211 pulse — the chain test above drives the chain itself), lift clear
    // of the deck band, then thrust west. The boundary stops the capsule at
    // 4.433 + 0.55 (the body radius) ≈ 4.98 — min x must land in that window, never past it.
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as the entry test (the p3 == 2 spawn
    // enters substate 8 [M 0x1910f8 :160-165]; this test pins the WALL, not the substate).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4b, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_9,
        "the ch-75 rise enters the float [M 0x216388:10-17]");
    let mut min_x = f32::MAX;
    for t in 0..1100u32 {
        let input = if t < 220 { Input { lift: 1.0, ..Default::default() } }
                    else { Input { forward: 1.0, ..Default::default() } };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        min_x = min_x.min(sim.bond.pos[0]);
    }
    assert!(min_x > 4.85 && min_x < 5.15,
        "the float thrusts west and the boundary stops the capsule at 4.433 + 0.55 ≈ 4.98 \
         [D plc #128; M the kind-0x800 body, no bit-0x8 drop; M CylGeom two-sided], got min x {min_x:.3}");
}

/// The sealing completes this task's scan: the boundary box (plc #128 OBJECT_Boundary_1 + its
/// end-walls) is closed on every face for the player's body. The west wall (x 4.433), the shaft's
/// top slab (y 16.74, normal down) and the pit floor (y −16.56) are the sisters above/`probe1b_wall`;
/// THIS test pins the two END-WALLS at z ±20.40 (m40) and their only openings — deck-level windows
/// ~0.8-1.2 tall — against the body's own height [M Player_Collision 0x194df0's default arm: the
/// capsule A = pos + up·0.55, B = pos + up·(+0x7CC − cc), radius 0.55 [M 0x3f0ccccd], standing cc
/// ≈ 1.0347 [L ee_start_05.bin BL 0x16cf0f0 per docs/fields/obj-tag-player.md] — ~1.58 tall,
/// taller than every window band]. With no script player-write on the disc [D the walk's doc
/// comment sweep], the box is sealed for real-input crossing; the game's own crossing (if any)
/// must use a body shape or a mechanism outside this scan — the report's live checks.
#[test]
fn the_end_walls_seal_the_shaft_box() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let w = &sim.world;
    // The end-walls exist at z ±20.40 across the shaft's x/y span (both ray directions — the
    // segment query is one-sided [M 0x10a8c8:168]).
    for (x, y) in [(6.5f32, -9.5), (6.5, 0.5), (6.5, 8.5), (10.0, -6.5), (14.0, -3.5), (19.0, 2.5), (23.0, -9.5)] {
        for zs in [20.4f32, -20.4] {
            let (a, b) = if zs > 0.0 { (26.0, 14.0) } else { (-26.0, -14.0) };
            let mut hit = None;
            for (p, q) in [(a, b), (b, a)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z - zs).abs() < 0.6 { hit = Some((z, mat)); break; }
                }
            }
            let (z, mat) = hit.unwrap_or_else(|| panic!("the end-wall at z {zs:+.1} must face ({x:.1},{y:+.1}) [D probes]"));
            assert!((z - zs).abs() < 0.6 && mat & 0x3f == 0 && mat & 0x40 != 0,
                "the end-wall face at ({x:.1},{y:+.1}) sits at z {z:+.2} with the boundary material [D m40], got m{mat:02x}");
        }
    }
    // The only openings: deck-level windows, y bands [D 0.2-step scans]:
    //   x 7.5/8.0: y −1.8..−1.0 (0.8 tall); x 9.0: −2.0..−1.2; x 10.0: −2.2..−1.2 (1.0 tall).
    for (x, y_lo, y_hi) in [(7.5f32, -1.8, -1.0), (8.0, -1.8, -1.0), (10.0, -2.2, -1.2)] {
        let mut y = y_lo;
        while y <= y_hi + 0.01 {
            let mut hitz = false;
            for (p, q) in [(-26.0f32, -14.0), (-14.0, -26.0)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z + 20.4).abs() < 0.6 { hitz = true; break; }
                }
            }
            assert!(!hitz, "the window band at (x {x:.1}, y {y:+.1}) must be open [D probes]");
            y += 0.4;
        }
        // The band's edges are wall (the window is bounded).
        for y in [y_lo - 0.6, y_hi + 0.6] {
            let mut hitz = false;
            for (p, q) in [(-26.0f32, -14.0), (-14.0, -26.0)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z + 20.4).abs() < 0.6 { hitz = true; break; }
                }
            }
            assert!(hitz, "the window band's edge at (x {x:.1}, y {y:+.1}) is end-wall [D probes]");
        }
        assert!(y_hi - y_lo < 1.3, "the window at x {x:.1} is {:+.1} tall [D probes] — narrower than the body", y_hi - y_lo);
    }
    // The body cannot pass any band: the capsule's total height ~1.58 [M 0x194df0's default arm]
    // exceeds every window (0.8..1.0 [D]).
    let capsule_height = 0.55 + (1.0347 - 0.55) + 2.0 * 0.55; // A to B span + the radius bulge at both ends
    assert!(capsule_height > 1.3,
        "the float body (~{capsule_height:.2} tall [M Player_Collision 0x194df0 default arm + cc 1.0347 [L]]) exceeds the widest window (1.0 [D])");
}

/// The iris doors, decided: the z ±8.82 booths' doors — SP#141/#142
/// "ScriptPlayer_IrisDoorOpen" [D placement names/positions (5.23, 9.45, ∓8.82)], script
/// 0x06000818 [D p0] — are LOAD-TIME LOOPERS, not missile-triggered pieces: the create runs
/// `SP_LoadScript` for mode ∉ {0,1} [M SP_CreateScriptPlayer 0x1db9a8 tail] and its mode-8 arm
/// starts the script LOOPING (`Script_Play(script, 1)`) in state 6 [M SP_LoadScript 0x1dbc08:46-48].
/// The script's per-loop shape [D decode]: the 8 iris-panel entities (0x02000834..3b) animate
/// OPEN at keys 0..120, the script HOLDS at t208 (stream 9 ev0x03 p0=0 — the −1 delay), the
/// panels CLOSE at 480..600 with the door sounds (stream 10: 0x000004c4 at start, 0x000004c3 at
/// t480), and the missile-done channels 0x18/0x1c are CLEARED at t550 (streams 12/13) — each
/// ~10 s loop clears the missile bookkeeping. The channel-RISE restart does NOT apply: [M
/// SP_Update 0x1dbda8 case 6] reloads only through the OTHER script slot, and both SPs carry a
/// single script (hash[1] = 0 [D live players]) — the rise never starts them. The doors are NOT
/// the route west either way: their meshes sit at x 5.23, east of the boundary [D probe1b_wall;
/// the boundary pin test above].
#[test]
fn the_iris_doors_are_load_time_loopers() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The create/load semantics: primed PLAYING loopers (state 6 = the mode-8 arm's
        // Script_Play(loop=1) [M 0x1dbc08:46-48]) on their own out channels [D p6].
        for plc in [141usize, 142] {
            let p = sim.drones.scripts.players.iter().find(|p| p.placement == plc).unwrap();
            assert_eq!(p.scripts[0], 0x0600_0818, "SP#{plc} carries the iris-door script [D p0]");
            assert_eq!(p.modes[0], 8, "SP#{plc} is a mode-8 looper [D p2]");
            assert_eq!((p.state, p.info[0].is_some()), (6, true),
                "SP#{plc} primed PLAYING (state 6, script loaded) from the level load [M 0x1db9a8 + 0x1dbc08]");
            assert_eq!(p.out, if plc == 141 { 0x1c } else { 0x18 }, "SP#{plc} fires on its own out channel [D p6]");
        }
        // The 8 iris-panel entities exist (the playing script created them [D streams 1-8,
        // hashes 0x02000834..3b]). Their stored matrices are the streams' local keys — the world
        // placement folds the parent chain (Script_Interp's live-matrix walk), so only the
        // existence and the hash band are pinned here.
        let mut panels = 0usize;
        for o in sim.drones.scripts.objects.list.iter().flatten() {
            if let nf_game::nis::Obj::Entity { hash, .. } = o {
                if (0x0200_0834..=0x0200_083b).contains(hash) { panels += 1; }
            }
        }
        assert!(panels >= 8, "the 8 iris-panel entities exist [D streams 1-8], found {panels}");
        // The channel-rise sister: [M 0x1dbda8 case 6] reloads only through the other slot
        // (hash[1] = 0 [D]) — a ch rise on the out channel starts NOTHING.
        assert_eq!(sim.drones.scripts.players.iter().find(|p| p.placement == 141).unwrap().state, 6,
            "sister: the doors stay looping loopers regardless of the missile channels");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1c, 1, sim.drones.frame);
        sim.drones.channels.set(0x18, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for plc in [141usize, 142] {
            let p = sim.drones.scripts.players.iter().find(|p| p.placement == plc).unwrap();
            assert_eq!(p.state, 6,
                "sister: the ch 0x18/0x1c rise does NOT restart SP#{plc} [M 0x1dbda8 case 6 needs the other slot]");
        }
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
