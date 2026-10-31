//! walkreal-c: 07000016 (the missile base launch centre) walked with the player's own movement —
//! no `bond.pos` writes after the spawn. The route (docs/levels/07000016.md; the walkreal_probe_c
//! dump): the arrival (5.17, 0.90, −65.30, the carried 07000014/15 chain) → the spawn plate #252
//! fires ch 0x8e (completes the carried record 21) → the south corridor (doors [7]/[0]/[1]/[3]/[2]
//! — [0]/[3]/[2] are locked by ch 0x24/0x27/0x28, whose writers are the mask-0x04 activate
//! triggers beside them; the stuck-mitigation's E press presses them) → the silo plate #253 fires
//! ch 0x8f (completes "Reach Drake's launch silo", reveals the Kiko objective) → the south room's
//! ch-0x81 touch trigger at (2.4, 1.0, −176.4) fires the launch chain [D] (the cinema plays OUT —
//! no skip: its stream writes ch 185 → SP #272 → ch 240/242 → the spawners) → the chaser (Kiko,
//! #261) spawns. Her death raises NOTHING (param 12 = 0 [D]; kiko-16-defeat removed the init
//! stand-in) — the mission's defeat writer is the KikoDeathButton #693 (the E press; pinned by
//! interactables_07000016_live and fix_finale_live) → ch 0x90 → AND #271 → ch 100 → MissionState
//! 6 → the driving hand-off 0x7300017 [M 0x182c60:111-112].
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance). Kiko herself is not the objective's writer (see the
//! header) — the walk stops at her spawn; the button press is the finale tail's business. Opt-in
//! via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    // A fresh Bond with only the carried gates: the save Trigger_Activate would make [M 0x1b2318:97-116].
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

/// The landed door[7] fix, focused: the door opens on Bond's pitched E from the corridor and he
/// walks through the leaf's span. FAIL-FIRST: before the straddle walk-in in door.rs the
/// E never moved the leaf (14 use_door calls, state 0x2 — the blocker), and with the old x 3.5
/// lane the walk wedged on the sealed west half at (3.5, 0.9, −73.58) even with the door open
/// (the lane is fixed here too). Sisters: the static corridor floor holds Bond throughout (grounded, no sink), and
/// walkreal_07000012_live's doors still open (the harness's own sister).
#[test]
fn door7_opens_on_bonds_e_and_bond_passes() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);
    w.walk_route(&[("the corridor south", walker::Wp::new(4.3, 0.6, -72.5).r(1.2))]);
    // The static floor holds him: grounded on the corridor floor for a full second.
    for _ in 0..60 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded, "the static corridor floor holds Bond");
    }
    let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| (d.pos[2] + 74.2).abs() < 0.5)
        .expect("door[7] is in reach");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0x2, 0x2, "door[7] starts closed");
    w.use_door(di, "door[7] on Bond's E");
    // The leaf's travel: ~30 ticks at 1/30 [L] — wait it out, then assert the travelled pose.
    for _ in 0..60 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[di];
        assert!(d.state == nf_game::door::OPEN || d.state == nf_game::door::OPENING,
            "door[7] opens on Bond's E (state {:#x}, pos01 {:.3}) — the walkreal-c blocker", d.state, d.pos01);
        assert!(d.pos01 > 0.5, "the leaf travelled (pos01 {:.3})", d.pos01);
    }
    // Through the leaf's span (x 4.35..5.62 — the operable east half; the west half is sealed
    // static geometry, the probe's crossing sweep).
    w.walk_to(walker::Wp::new(4.9, -0.1, -78.0).r(1.2), "door[7] through");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[2] < -75.5, "Bond passed the doorway plane (z {:.2})", p[2]);
        assert!(sim.bond.grounded, "the floor beyond the door holds him");
    }
}

// RESOLVED (fix-final-blockers): the walkreal-c stall at door[7] was TWO things. ① The door itself
// opens — state 0x4 / pos01 1.00 / matrix row3 y 2.175 at the stuck dump (the walkreal-c report's
// "state stays 0x2" predates the straddle walk-in that landed in door.rs after that report).
// ② The stall is the ROUTE LANE: the doorway's operable half is the leaf's own span x 4.35..5.62
// (the closed leaf's world bbox, probe walkreal_probe_fb3) and the static-only crossing sweep
// shows x 4.4..5.5 OPEN at the door plane while x 2.8..4.2 is a SEALED wall (the corridor cel
// mesh "Start_corridor1" carries the west filler at z −74.13 — the doorway's left leaf is fixed
// geometry). The walkreal-c route aimed Bond at x 3.5 — into the sealed west half — and his
// capsule wedged on it at (3.5, 0.9, −73.58) (segment_to_target 0.124 → z −74.13). The lane moves
// to x 4.9 (through the leaf's span), the E still reaches (the door pos is the leaf's west edge).
//
// RESOLVED (level-16-complete): door[0]'s lock 0x24 writer is the SecurityStation touch trigger
// #250 (kind 0xEA TOUCH_ONCE, out 0x24, mask 0x04 = the player's E inside the volume [M
// Trigger_Activate 0x1b2318:38-49 — mask&4 gates the use path]) at (−15.2, 6.6, −94.6) [D] — the
// upper WEST walkway of the trench hall, y 5.7. The real ascent is the east ramp: the room's east
// floor (x 13..16) climbs 0.4→4.1 northward (z −88→−82, under the 10.8 slab at z −86) onto the
// upper ring (y 5.7..6.8) that wraps the hall — north (z −78..−80), west (x −18..−9) — to the
// station (probe16_writers::probe_station_full_walk, every leg arrived). Pressing E there fires
// ch 0x24 and door[0]'s lock arm opens it (door.rs's rising-edge unlock calls Door_Activate —
// state 0x4 observed). No port fix was needed: the doors, the trigger and the E path all worked —
// the WALK's route was the bug (it never visited the station; the parked wedge at door[0]'s east
// jamb was the symptom, and the 0x24 writer was 20 m up and 20 m west of it).
#[test]
fn walk_arrival_to_the_chasers_spawn() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 3.0 && p[0] < 7.0 && p[1] > -1.0 && p[1] < 2.0 && p[2] > -67.0 && p[2] < -63.0,
            "the arrival spawns at the Omega exit's far side (#1 [D]), got {p:?}");
    }
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8e), "the spawn plate #252 fires ch 0x8e [D]");
        assert!(sim.drones.mission.states[21].iter().all(|&s| s == 4), "the carried record 21 (07000015) fully completes");
        assert_eq!(sim.drones.mission.states[22][0], 3, "'Reach Drake's launch silo' announces [L record 16 obj 1]");
    }

    // The south corridor. Doors: [7] (−74.2, unlocked), [0] (−114.2, lock 0x24 — unlocked by the
    // station E above), [1] (−124.2, unlocked), [3] (−141.9, lock 0x27), [2] (−152.3, lock 0x28).
    // [3]/[2]'s writers are the mask-0x04 activate triggers beside them ((3.4,0.8,−141.5) → 0x27,
    // (3.4,0.8,−151.9) → 0x28) — the player's E on the panel; the stuck-mitigation presses E at
    // each closed door.
    let route: &[(&str, walker::Wp)] = &[
        ("the corridor south", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        // The operable east half of the doorway (the leaf's span x 4.35..5.62; the west half is a
        // sealed static wall — the probe's crossing sweep). The E reaches from here too.
        ("door[7] through", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        // THE SECURITY STATION DETOUR (the door[0] unlock): east along the hall's south lip, then
        // up the east ramp (x 13..16, 0.4→4.1 northward) onto the upper ring, west along it to the
        // station trigger #250 at (−15.2, 6.6, −94.6), E. The ring walk passes north of the 6.4
        // deck rail at x −14 (the approach hugs x −16 west of it — the east-side approach wedges
        // on the rail).
        ("the lane east", walker::Wp::new(10.0, 0.0, -82.0).r(1.3)),
        ("the lane south", walker::Wp::new(12.0, 0.0, -88.0).r(1.3)),
        ("the ramp foot", walker::Wp::new(13.8, 0.0, -90.8).r(1.1)),
        ("the ramp 1", walker::Wp::new(14.5, 0.6, -87.5).r(1.2)),
        ("the ramp 2", walker::Wp::new(14.5, 2.4, -84.5).r(1.2)),
        ("the ramp 3", walker::Wp::new(14.5, 4.6, -81.5).r(1.2)),
        ("the ring east", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the ring 1", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the ring 2", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the ring 3", walker::Wp::new(-11.0, 5.9, -81.0).r(1.4)),
        ("the ring 4", walker::Wp::new(-13.0, 5.9, -86.0).r(1.4)),
        ("the station west", walker::Wp::new(-16.5, 5.9, -88.0).r(1.3)),
        ("the station south", walker::Wp::new(-16.3, 5.9, -93.5).r(1.2)),
        ("the station E", walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2)),
        // Back down: the same ring east to the ramp, down to the lane, rejoin the trench bypass.
        ("the return 1", walker::Wp::new(-16.3, 5.9, -89.0).r(1.3)),
        ("the return 2", walker::Wp::new(-13.0, 5.9, -85.0).r(1.4)),
        ("the return 3", walker::Wp::new(-10.0, 5.9, -81.0).r(1.4)),
        ("the return 4", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the return 5", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the return 6", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the descent 1", walker::Wp::new(14.5, 4.6, -82.0).r(1.2)),
        ("the descent 2", walker::Wp::new(14.5, 2.4, -85.0).r(1.2)),
        ("the descent 3", walker::Wp::new(14.5, 0.6, -88.0).r(1.2)),
        // The trench bypass: the corridor floor dips to −4.8 across the trench (x −2..6,
        // z −88..−102); the east lane x 8..11 holds a continuous −0.2 floor past it (the probe's
        // ledge grid). The real player keeps east; the walkreal-c lane (x 2.5) walked into the pit
        // and wedged on its south wall at (2.56,−3.33,−102.7) (the southward segment 0.126 →
        // z −102.83).
        ("the east lane south", walker::Wp::new(9.5, 0.0, -100.0).r(1.6)),
        ("the trench's west end", walker::Wp::new(4.0, 0.0, -108.0).r(1.6)),
        // fix-final-blockers2: the old single approach (r 1.3) let the walker arrive 0.7 east and
        // drift into the doorway's east jamb — the stuck hop put him ON the east furniture
        // (5.16,1.75,-113.63). Hug the leaf's span (x 1.84..3.11) tightly and turn south INSIDE it.
        ("door[0]'s west approach", walker::Wp::new(2.3, 0.0, -110.0).r(0.7)),
        ("door[0]'s threshold", walker::Wp::new(2.3, 0.0, -113.2).r(0.7)),
        ("door[0] through", walker::Wp::new(2.0, -0.1, -119.0).r(1.2)),
        ("door[1] through", walker::Wp::new(2.0, -0.1, -132.0).r(1.2)),
        // The locked doors' unlock panels (mask-0x04 triggers on the corridor's east wall — the
        // player's E inside the volume [M Trigger_Activate 0x1b2318:38-49]): door[3]'s at
        // (3.4,0.8,−141.5) → 0x27 [D], door[2]'s at (3.4,0.8,−151.9) → 0x28 [D], door[4]'s at
        // (3.5,1.0,−165.7) → 0xb4 [D]. The door-side stuck-E cannot reach them (the panels sit
        // ~1.6 m east of the door lanes).
        ("door[3]'s panel", walker::Wp::at(3.4, 0.8, -141.5, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        // Off the panel and back to the door LANE north of the doorway (the direct diagonal rides
        // the east jamb — the y-2.1 hop); then straight south through the leaf span x 1.84..3.11.
        ("door[3] off panel", walker::Wp::new(2.0, 0.0, -140.0).r(1.0)),
        ("door[3] through", walker::Wp::new(2.0, -0.1, -147.0).r(1.2)),
        ("door[2]'s panel", walker::Wp::at(3.4, 0.8, -151.9, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[2] off panel", walker::Wp::new(2.2, 0.0, -150.4).r(1.0)),
        ("door[2] through", walker::Wp::new(2.2, -0.1, -151.5).r(1.2)),
    ];
    w.walk_route(route);

    // The station's unlock rode the route: ch 0x24 rose on the E (the walk crossed door[0]'s
    // approach below, so the door was already open or opens on the E now).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x24), "the SecurityStation E fires ch 0x24 [D #250 mask 0x04]");
        assert_eq!(sim.drones.doors.list[0].ch_lock, 0, "door[0]'s lock cleared by the rising edge [M 0x13660c arm]");
    }

    // The silo plate #253 (0.45-4.62, −0.20..1.88, −154.93..−152.85 [D]): walk onto its slab.
    w.walk_to(walker::Wp::new(2.5, 0.3, -153.9).r(1.0), "the silo plate");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8f), "the silo plate #253 fires ch 0x8f [D]");
        assert_eq!(sim.drones.mission.states[22][0], 4, "the monitor completes 'Reach Drake's launch silo'");
        assert_eq!(sim.drones.mission.states[22][1], 3, "ch 0x8f reveals 'Prevent Kiko from launching into space' [L record 16 obj 2]");
    }

    // The launch chain's head: the ch-0x81 BOND touch trigger at (2.4, 1.0, −176.4) [D — the
    // walkreal_probe_c dump; the doc's "no writer" scan missed the touch trigger]. Walk south
    // through door[4] (2.5, 1.0, −166.4, lock 0xb4 — its writer is the mask-0x04 trigger at
    // (3.5, 1.0, −165.7), the E panel, pressed above) into the south room.
    w.walk_to(walker::Wp::at(3.5, 1.0, -165.7, walker::Act::Use { use_ticks: 90 }).r(0.8), "door[4]'s panel");
    w.walk_to(walker::Wp::new(2.5, 1.0, -169.0).r(1.3), "door[4] through");
    w.walk_to(walker::Wp::new(2.4, 1.0, -175.0).r(1.2), "the south room");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x81), "the touch trigger at (2.4,1.0,−176.4) fires ch 0x81 [D]");
    }

    // The chain PLAYS OUT (no skip): SP #262's pack cinema 0x06000631 (750 ticks) writes ch 185
    // at t748 → SP #272 "ShuttleLaunch" → ch 240/242 → spawners #257/#293 [D]. Bond stands while
    // the cinema runs (the real player watches it).
    w.skip_cutscenes = false;
    let chaser = {
        let mut found = None;
        for _ in 0..6000u32 {
            w.tick(Input::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            // The chaser is the skin-0x1e dtype-0xf drone NOT in the death-ch 23..26 pit squad
            // (#294–297 [D]) — her param 12 is 0 [D #261] (the old 0x90 init stand-in is gone).
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)) {
                found = Some(i);
                break;
            }
        }
        w.skip_cutscenes = true;
        found.expect("the launch chain spawns the chaser (spawner #257, group 6 [D])")
    };
    eprintln!("[walk] the chaser spawned at {:?}", w.scene.sim.as_ref().unwrap().drones.list[chaser].pos);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[chaser].present, "the chaser lives");
        assert!(sim.drones.channels.on(0x81), "the launch chain ran to the spawners [D]");
    }
}

/// SISTER: an idle Bond at the pit floor dies to the ch-243 pulse and
/// the mission fails ~449 ticks after the pulse rises — the game's own +454 [L fresh boot, god
/// mode, Bond placed on trigger #255] within sampling noise — NOT one tick. The chain, all [M]:
/// the pulse makes HurtPoly-ShuttleLaunch #309 live (Hurt_Update 0x17e5d0:98-103 arms state 1 on
/// the raw channel byte); the pit-floor capsule contacts a plane of the six-band stack and
/// `Player_DealWithObjHit` 0x194458:95-101 (the class-0x27 arm) calls
/// `Player_HandlePain(impact + Hurt_GetDamage = 1000 [D params], type 2)` — types 1..7 give
/// armourFraction 0 [M 0x1902c8:84-99] so health 100−1000 → 0 the same tick [M 0x1902c8:131-134;
/// Hurt_Update itself carries NO damage and NO delay: state 1 with key count 0 returns at once,
/// 0x17e5d0:106-108]; the death sequence then runs `Player_HandleDeath` 0x190bb0:29-33, which
/// raises ch 0x62 only when the blood pane's state is non-zero, and `HUD_UpdateBloodPane`
/// 0x19a6a8:23-30 grows that pane 1 px/tick to the 448 cover (`RED_FADE_TICKS`, death.rs) —
/// `Mission_Update` 0x182c60:21,35-42 reads ch 0x62 → MissionState 3 (the fail). The port models
/// every step (crate::death), so the fail lands at pulse+~450; the live run's +454 (with god mode
/// rewriting health each frame — the fail is NOT health-gated, it rides the death sequence) is the
/// same chain plus the live session's sampling. ch 240 and ch 80 stay 0 through the fail on both
/// sides [L; D stream ticks 2180/2240 land after it]. Bond's small drift during the window
/// ((3.57,−22.76,−179.55) → (3.62,−23.90,−178.31) [L]) is the settle onto the grate plus the death
/// anim — the port's settle differs in y because its capsule rests differently [S], not asserted.
#[test]
fn the_pit_pulse_kills_an_idle_bond_about_450_ticks_later_like_the_game() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    // The sister pins the death/fail TIMING, so the walkreal allowances are off: the hurt contact
    // must land (no top-up) and the pack cinema must play out (no skip) — the live session's own
    // shape [L no input].
    w.top_up_health = false;
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
        // The live session's own setup [L]: Bond placed on the bridge trigger #255, no input —
        // the touch fires the chain head (ch 0x81), the pack cinema drops him at the pit floor.
        sim.bond.pos = [2.4, 1.2, -175.5];
    }
    let (mut t_reenable, mut t243, mut t_dead, mut t_fail) = (None, None, None, None);
    for t in 0..4200u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t_reenable.is_none() && sim.bond.pos[1] < -20.0 { t_reenable = Some(t); }
        if t243.is_none() && sim.drones.channels.on(243) { t243 = Some(t); }
        if t243.is_some() && t_dead.is_none() && sim.drones.health.health <= 0.0 { t_dead = Some(t); }
        if sim.drones.mission.status() == 3 { t_fail = Some(t); break; }
    }
    let t_reenable = t_reenable.expect("the cinema re-enables Bond at the pit floor [D stream-8 t747 key]");
    let t243 = t243.expect("the launch script's stream 6 raises ch 243 [D @1611 absolute]");
    let t_dead = t_dead.expect("the pulse's 1000-damage contact kills Bond [M 0x194458 + 0x1902c8]");
    let t_fail = t_fail.expect("the death sequence raises ch 0x62 at the 448 cover and Mission_Update fails [M]");
    eprintln!("[pit] re-enable t{t_reenable}, ch243 t{t243} (+{}), dead t{t_dead} (+{}), MissionState 3 t{t_fail} (+{})",
        t243 - t_reenable, t_dead - t243, t_fail - t243);
    // The kill is the same tick family as the live run's health collapse (the live god-mode rewrite
    // hides the exact tick; the port's contact applies the 1000 on the pulse's first contact [M]).
    assert!(t_dead - t243 <= 2, "the contact kills within the pulse's first ticks (dead +{}), not delayed", t_dead - t243);
    // THE TIMING PIN: fail − pulse ≈ 448 (the blood-pane cover [M 0x19a6a8 + RED_FADE_TICKS]) + the
    // death tick + Mission_Update's read [M 0x182c60] ≈ 450; the game measured +454 [L fresh boot].
    let delta = t_fail - t243;
    assert!((440..=470).contains(&delta),
        "the mission fail lands ~450 ticks after the pulse (got +{delta}) — the game's own +454 [L], not one tick");
    // The "escapes" are post-pulse on both sides: ch 240 (the launch script's stream 7 @2180 [D])
    // and ch 80 (the lift summon, stream 21 @2240 [D]) are still cold at the fail [L].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(240), "ch 240 stays 0 through the fail [L; D @2180 lands later]");
        assert!(!sim.drones.channels.on(80), "ch 80 stays 0 through the fail [L; D @2240 lands later]");
        assert_eq!(sim.drones.death.ch62, true, "the fail rode the death sequence's ch 0x62 [M 0x190bb0:32]");
    }
}

/// The finale tail: the mission completes ONLY through the KikoDeathButton #693 [D hash
/// 020008c1]: the chaser's defeat raises nothing (param 12 = 0 [D]; the init-forced 0x90
/// stand-in was removed in kiko-16-defeat), so this walk's kill-then-watch shape pins the REAL
/// rule — ch 0x90 and ch 100 stay cold and the mission does not complete. PARKED (re-derived in
/// the pit work) on the pulse-1 survival window, with the round-2 timeline CORRECTED:
/// - **The corrected timeline.** The doc's round-2 expectation "ch 243 rises ~863 ticks after the
///   re-enable" was an arithmetic error (1611−748 double-counts the cinema): the launch script
///   0x0600090a starts at the pack cinema 0x06000631's ch-185 write (cinema t748 [D stream 7]),
///   one tick AFTER the re-enable key (stream 8's single t747 pit-floor pose [D]), so the model is
///   re-enable → ch 243 = 1611 − 1 ≈ 1610 ticks [M nis.rs:446-451 absolute Waits + D dumps].
///   The live fresh boot measured 1591 [L] — the model CONFIRMED within 20; the port's probe
///   (chain+2355 = 748 + 1611 + the SP-start latency) confirms it in-port. The remaining open
///   number is the live ch-0x81 → re-enable delta (measured 553 [L] vs the modeled 747 [D t747])
///   — see the report's live checks.
/// - **The kill is not instant on either side.** The pulse's contact applies the 1000 damage the
///   same tick family (Player_DealWithObjHit 0x194458:95-101 → Player_HandlePain type 2 → health
///   0 [M 0x1902c8:84-134]), but the MISSION fail rides the death sequence: the blood pane covers
///   in 448 ticks, Player_HandleDeath raises ch 0x62 at the cover [M 0x190bb0:29-33,
///   0x19a6a8:23-30] and Mission_Update → MissionState 3 [M 0x182c60:21,35-42]. Port +449, game
///   +454 [L] — pinned by the un-ignored sister above.
/// - **No disc-visible real-input escape during pulse 1 (559 ticks, 1611..2170 [D])**, re-checked
///   against the data this round: the six-band hurt stack (world y −24.08..−21.32) covers the
///   whole pit footprint with OVERLAPPING quads — no gaps to stand in [D tri dump]; the blast
///   rooms on the pit floor (the obvious shelters, base −23.0, roofs −19.53) are SEALED by static
///   BlockingPolys (2-tri walls x 19.60..23.22 / z −174.84..−169.00, y −22.08..−19.53 [D obj
///   117/118]) and their roofs are 4.4 above the grate — no jump, no ladder (the pit's only
///   kind-0x2d ladder is a 0.5×2×0.8 "PlayerStart" box at the rim [D place[256]/obj[167]]); the
///   shuttle-bay lift's LOW point (−21.22 [D keys]) sits 0.10 above the top band — the one safe
///   floor in the pit — but ch 80's ONLY writers are the scripts (0600090a stream 21 @2240,
///   06000632 stream 19 @1606 [D]; the full-placement census found NO trigger/AND/blocker naming
///   80), so the platform does not exist until 630 ticks into the pulse; the pit squad
///   (spawners #294-296, ch 187 @1512 [D]) is dropped into the pit 99 ticks before the pulse to
///   die in it — the game's own demonstration that the floor is lethal; and no pickup/substate/
///   Bond-actor stream exists [D+M, the round-2 census]. ch 240's writer is the stream-7 event
///   @2180 — post-pulse; no door placement carries it [D census].
/// The un-ignore requires the live check that could still hide a mechanism: the game's
/// ch-0x81 → re-enable delta of 553 [L] vs the modeled 747 [D]. If the pack cinema really started
/// ~194 ticks before the touch trigger's ch 0x81, something else starts it — that starter is the
/// missing escape. gdbwatch on switch_channels[0x81] (0x26fc90+129·4) and Bond's obj+0x30 from
/// the south-room approach through the cinema, FRESH boot.
#[test]
#[ignore = "finale-pit-4 round: the survival-window mechanism is now [M]-COMPLETE and the contradiction is sharpened to ONE link. (a) The volume really deals the damage: Hurt_Update 0x17e5d0:51-104's who-mask is a START gate only (gated → uVar5 = the channel byte; 1 != 0 → state 1) with no per-victim filter, the victim pass Player_DealWithObjHit 0x194458:95-102 is ungated (Player_CollisionHandler 0x1949a8:24 gates only on the enabled byte), and the collide candidate law ACCEPTS the volume while the gate is high (Collide_PickObj 0x1eae80:82-90 — the mode-0x100 sphere test short-circuits the +0xFA clause; the +0xF0 0x20 gate-low reject clears when 0xf3 rises; Hurt_Create 0x17e870 sets no other skip bit). (b) The hand-back chain: event 6 snapshots record(i−1) [M 0x1d94c0:73-84; Script_Update 0x1da998:29-31 increments +0xab3 AFTER Script_Run, so record(0x80+i·0x28) = the previous stream]; for 06000631 that is stream 8, whose Entity command (t747, hash 0, obj_type 31 — a model-less dummy [M Script_CreateEntity 0x1d8968's generic tail]) sets record kind 1 and poses the dummy AT CREATE via Script_EntityStart 0x1d8d78:111's own Script_Interp call from key 204 = (3.57, −22.76, −179.55) [D]; Script_Free 0x1da880:44-48 → Player_Enable 0x195498:54-68 teleports Bond there. The shipped Bond re-enables INSIDE the volume box 1611 ticks before the pulse, with the door opening 9 ticks after it drops (ch 0xf0 @2180 [D]) and the lift summoned 70 after that (ch 0x50 @2240 [D]). (c) No gate relief: the button's inhibit ch 0x8d has NO other disc writer — 07900016.bin holds ONLY 06000632 and its single [8d,0] @t420 needs ch 0x82 = mux #286 (0xbf·0xc0) [D], whose 0xc0 writer is the launch script's OWN stream 20 @t7840 [D — new this round] and whose 0xbf needs all four pit-squad deaths (mux #264, chs 0x17..0x1a [D]). THE OPEN LINK: Script_Interp 0x1d9a08's mode-0 (the Entity record's spline byte is FALSE → +0x22 = 0 [M 0x1d8d78:102-104]) behaviour for a one-key track — if the create-time interp does NOT apply key 204, the shipped dummy (and the hand-back) is control_create_object's ORIGIN pose, not the pit floor; the port applies the key (the phase-A assert below pins the port's own reading). Live checks in the report decide it: a fresh-boot watch of Bond's obj+0x30 across the hand-back (expected the pit-floor key) and of switch_channels[243] + the health through ticks 2355..2915 (expected: either the position is elsewhere — that difference IS the port gap — or the damage never lands)."]
fn walk_the_chasers_defeat_completes_the_mission() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(100);
    // Straight to the launch (the walkreal-c positions: the objectives' route is covered by
    // walk_arrival_to_the_chasers_spawn; this tail needs the chain + the fight only).
    w.walk_to(walker::Wp::new(2.5, 1.0, -169.0).r(1.3), "door[4] through");
    w.walk_to(walker::Wp::new(2.4, 1.0, -175.0).r(1.2), "the south room");
    w.skip_cutscenes = false;
    let chaser = {
        let mut found = None;
        for _ in 0..6000u32 {
            w.tick(Input::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            // The chaser is the skin-0x1e dtype-0xf drone NOT in the death-ch 23..26 pit squad
            // (#294–297 [D]) — her param 12 is 0 [D #261] (the old 0x90 init stand-in is gone).
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)) {
                found = Some(i);
                break;
            }
        }
        w.skip_cutscenes = true;
        found.expect("the launch chain spawns the chaser (spawner #257, group 6 [D])")
    };
    let cpos = w.scene.sim.as_ref().unwrap().drones.list[chaser].pos;
    let _ = w.try_walk_to(walker::Wp::new(cpos[0], cpos[1] + 6.0, cpos[2] + 4.0).r(2.0), "the chaser's rim");
    assert!(w.shoot(chaser, "the chaser"), "the chaser dies to Bond's fire");

    // The REAL rule (kiko-16-defeat): her death raises nothing — ch 0x90's writer is the
    // KikoDeathButton #693 [D], and the mission does not complete on the kill [D AND #271].
    for _ in 0..100 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x90), "the chaser's death raises ch 0x90 — her death channel is param 12 = 0 [D #261]; the writer is the switch #693 [D]");
        assert!(!sim.drones.channels.on(100), "mux #618 stays cold on the kill [D]");
        assert!(sim.drones.mission.status() != 6, "the mission does not complete on the kill [D AND #271 waits for the button]");
    }
}

/// The door[0] unlock chain, focused: the SecurityStation trigger #250 (kind 0xEA, out 0x24,
/// mask 0x04 [D]; the use-key path gates on mask&4 [M Trigger_Activate 0x1b2318:38-49]) sits on
/// the trench hall's upper west walkway; the east ramp (x 13..16) is the real ascent. Bond walks
/// the whole way (no teleport), presses E in the volume, and door[0]'s rising-edge lock arm
/// unlocks AND opens it [M door.rs ch_lock arm — Door_Activate on the edge; state 0x4 observed].
/// FAIL-FIRST: every walkreal revision before level-16-complete skipped the station — the walk
/// wedged at door[0]'s sealed approach with ch 0x24 cold (the parked blocker in the header).
#[test]
fn door0_unlocks_from_the_security_station_use() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(100);
    let route: &[(&str, walker::Wp)] = &[
        ("the corridor south", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        ("door[7] through", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        ("the lane east", walker::Wp::new(10.0, 0.0, -82.0).r(1.3)),
        ("the lane south", walker::Wp::new(12.0, 0.0, -88.0).r(1.3)),
        ("the ramp foot", walker::Wp::new(13.8, 0.0, -90.8).r(1.1)),
        ("the ramp 1", walker::Wp::new(14.5, 0.6, -87.5).r(1.2)),
        ("the ramp 2", walker::Wp::new(14.5, 2.4, -84.5).r(1.2)),
        ("the ramp 3", walker::Wp::new(14.5, 4.6, -81.5).r(1.2)),
        ("the ring east", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the ring 1", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the ring 2", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the ring 3", walker::Wp::new(-11.0, 5.9, -81.0).r(1.4)),
        ("the ring 4", walker::Wp::new(-13.0, 5.9, -86.0).r(1.4)),
        ("the station west", walker::Wp::new(-16.5, 5.9, -88.0).r(1.3)),
        ("the station south", walker::Wp::new(-16.3, 5.9, -93.5).r(1.2)),
        ("the station E", walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2)),
    ];
    for (label, wp) in route { w.walk_to(*wp, label); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x24), "the station E fires ch 0x24 [D #250]");
        assert_eq!(sim.drones.doors.list[0].ch_lock, 0, "door[0] unlocks on the rising edge [M door.rs]");
        assert!(sim.drones.doors.list[0].state & 0x6 != 0, "the unlock's Door_Activate opens door[0] (state {:#x})", sim.drones.doors.list[0].state);
    }
}

/// Sister: with ch 0x24 cold, door[0] refuses the E (the lock arm [M 0x13660c:484 — use_door
/// fails while ch_lock is set and its channel is off]). Teleport to reach (allowed: the test then
/// operates the door itself through the real E path).
#[test]
fn door0_stays_locked_without_the_station_use() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert!(!sim.drones.channels.on(0x24), "ch 0x24 starts cold");
        sim.bond = nf_game::player::Bond::new([2.3, 1.2, -111.0], std::f32::consts::PI);
    }
    for _ in 0..30 { w.tick(Input::default(), false); }
    let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[2] + 114.23).abs() < 0.5).expect("door[0] in reach");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].ch_lock, 0x24, "door[0] carries lock 0x24 [D #4 par 3]");
    w.use_door(di, "door[0] on Bond's E while locked");
    for _ in 0..60 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x24), "no E anywhere fired the station");
        assert_eq!(sim.drones.doors.list[di].state & 0x2, 0x2, "door[0] stays closed under lock 0x24 [M 0x13660c]");
        assert!(sim.drones.doors.list[di].pos01 < 0.1, "the leaf never travelled");
    }
}

/// level-16-security-station-use: the end-to-end leg the other two tests split in half.
/// `dynobj_glass_live::the_16_screens_swap_with_their_channels` SETS ch 0x24 directly (no player);
/// `door0_unlocks_from_the_security_station_use` above walks the real E but asserts only the
/// channel + the door. This walks the SAME real route (the east ramp → the upper ring → the
/// station, docs/levels/07000016.md §7) and asserts the whole downstream set the game does on the
/// station's E — [D #597 "SecurityStation" params 0:0x24 1:0x4 (mask 4 = the activate dispatch)]
/// reached by [M Player_Activate 0x1a5170:100-137 (the use sphere from the head along the look)
/// → Trigger_Activate 0x1b2318:38-43 (type 2, `mask & 4` gate → `switch_channels[0x24] = 1`)]:
///   1. ch 0x24 rises;
///   2. the screens' 0x3b pair swaps — #682 Screen_door_unlocked (p0 = 0x24) hidden at create
///      [M DynamicObject_Create 0x18dbf8:27-29 `|0x30 when p0 != 0`] becomes DRAWN (watch A ON
///      restores the base flags [M DynamicObject_Update 0x18d3b8:32-33]) and #683
///      Screen_door_locked (p1 = 0x24, mode 0) becomes DEAD [M 0x18d3b8:26] — in the sim AND in
///      the draw palette (the GAP-16-1 per-tick re-sync the 1b glass task landed);
///   3. door[0] unlocks on the rising edge and opens [M door.rs — Door_Activate on the edge].
/// The screens sit at the disc's origin (0, 0.02, 0) [D the 07000016-audit kind-0x3b census] —
/// the swap is asserted through their slots' sim flags and palette matrices, not a world spot.
#[test]
fn the_station_use_swaps_the_screens_end_to_end() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(100);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x24), "ch 0x24 starts cold [D the census: #597 is its only writer]");
    }
    let opts = || nf_view::scene::ViewOpts::default();
    let (layout, _attrs) = w.scene.skin_layout();
    let base = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.doors.list.len() + sim.drones.sss.list.len() + sim.drones.breaks.list.len()
            + 2 * sim.drones.locks.len()
            + (1 + nf_game::gunimp::TURRET_ARTS.len()) * w.scene.gunimp_placements
            + 3 * sim.drones.sensors.list.len()
    };
    // The two screens by their channel signature [D params]: #682 watches A = 0x24, #683 B = 0x24.
    let sim = w.scene.sim.as_ref().unwrap();
    let di_unlocked = sim.drones.worldobjs.dynobjs.iter().find(|d| d.ch_a == 0x24).map(|d| d.slot as usize)
        .expect("#682 Screen_door_unlocked (p0 = 0x24) among the 0x3b rows");
    let di_locked = sim.drones.worldobjs.dynobjs.iter().find(|d| d.ch_b == 0x24).map(|d| d.slot as usize)
        .expect("#683 Screen_door_locked (p1 = 0x24) among the 0x3b rows");
    let shown = |scene: &nf_view::scene::Scene, di: usize| -> bool {
        let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
        pal[layout.door_base as usize + base + di][3][1] > -9000.0
    };
    assert!(sim.drones.worldobjs.dynobjs[di_unlocked].flags & 0x30 != 0,
        "#682 hidden at create [M 0x18dbf8:27-29 |0x30 when p0 != 0]");
    assert!(!shown(&w.scene, di_unlocked), "#682 not drawn before the use (palette)");
    assert!(shown(&w.scene, di_locked), "#683 drawn before the use (palette)");
    drop(sim);

    // The door[0]-unlock route (the same real walk the sister test pins): the corridor south,
    // door[7], the east ramp, the upper ring west, the station — then the E press.
    let route: &[(&str, walker::Wp)] = &[
        ("the corridor south", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        ("door[7] through", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        ("the lane east", walker::Wp::new(10.0, 0.0, -82.0).r(1.3)),
        ("the lane south", walker::Wp::new(12.0, 0.0, -88.0).r(1.3)),
        ("the ramp foot", walker::Wp::new(13.8, 0.0, -90.8).r(1.1)),
        ("the ramp 1", walker::Wp::new(14.5, 0.6, -87.5).r(1.2)),
        ("the ramp 2", walker::Wp::new(14.5, 2.4, -84.5).r(1.2)),
        ("the ramp 3", walker::Wp::new(14.5, 4.6, -81.5).r(1.2)),
        ("the ring east", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the ring 1", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the ring 2", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the ring 3", walker::Wp::new(-11.0, 5.9, -81.0).r(1.4)),
        ("the ring 4", walker::Wp::new(-13.0, 5.9, -86.0).r(1.4)),
        ("the station west", walker::Wp::new(-16.5, 5.9, -88.0).r(1.3)),
        ("the station south", walker::Wp::new(-16.3, 5.9, -93.5).r(1.2)),
        ("the station E", walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2)),
    ];
    for (label, wp) in route { w.walk_to(*wp, label); }

    // The trigger writes ch 0x24 at Bond's turn in the object walk; the doors read it the next
    // tick and the palette re-syncs per tick — settle a few real-input ticks.
    for _ in 0..10 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x24),
            "the station E fires ch 0x24 [D #597 params; M 0x1b2318:38-43]");
        // The screens' sim state after the edge.
        let un = sim.drones.worldobjs.dynobjs.iter().find(|d| d.slot as usize == di_unlocked)
            .expect("#682 alive after the edge");
        assert!(!un.dead && un.flags & 0x8010 == 0,
            "#682 drawn after the edge [M 0x18d3b8:32-33 watch A restores the base]");
        assert!(un.flags & 0x30 == 0, "#682's create-hide cleared [M 0x18dbf8:31 → 0x18d3b8:32]");
        let locked_hidden = sim.drones.worldobjs.dynobjs.iter().find(|d| d.slot as usize == di_locked)
            .map_or(true, |d| d.dead || d.flags & 0x8010 != 0);
        assert!(locked_hidden,
            "#683 not drawn after the edge [M 0x18d3b8:26 mode 0 + armed B → obj+0xFE |= 1]");
        // The palette (the draw the player sees) swapped for both.
        assert!(shown(&w.scene, di_unlocked), "#682 NOW drawn (palette re-sync)");
        assert!(!shown(&w.scene, di_locked), "#683 NOW hidden (palette re-sync)");
        // Downstream: door[0] unlocks on the rising edge and opens.
        let d0 = sim.drones.doors.list.iter().find(|d| (d.pos[2] + 114.23).abs() < 0.5).expect("door[0]");
        assert_eq!(d0.ch_lock, 0, "door[0] unlocks on the rising edge [M door.rs]");
        assert!(d0.state & 0x6 != 0, "door[0] opened by the unlock's Door_Activate (state {:#x})", d0.state);
        eprintln!("[draw-for-main-agent] the security-station E (07000016): \
            1) the corridor screen pair swaps — #683 Screen_door_locked (watch B 0x24, at the \
            disc's origin (0,0.02,0) [D census]) disappears and #682 Screen_door_unlocked \
            (watch A 0x24) appears in its slot; 2) door[0] 'Door_1' at (1.84, −0.10, −114.23) \
            slides open (state {:#x}, leaf {}); 3) nothing else draws — ch 0x24 has no other \
            draw-side consumer [D the channel census]", d0.state, d0.pos01);
    }
}

/// Task level-16-jetpack-escape-4 step 2: the launch chain run in the REAL mission order (the
/// arrival → the spawn plate → the station/panels → the silo plate → the bridge touch, all walked)
/// against the sister's teleported run (Bond placed on trigger #255). The disc data gives the
/// order nothing to select: no event-0x11 conditional exists in any of the level's 17 scripts [D
/// order16 probe], the ONLY writer of ch 0x81 is the touch trigger #255 [D place[255]], the only
/// SP the order arms before the chain is SP#348's 0x060009f6 — two streams, no keys, no events [D]
/// — and the four keypad SPs it also arms play display anims that write no channels [D 06000999].
/// This pins that in the port: both orders re-enable Bond at the [D] stream-8 pit-floor pose and
/// raise ch 243 on the same clock. The ONE channel-state difference the order makes — the cinema's
/// stream 13 clears ch 180 (0xb4, door[4]'s panel channel) at its t747 [D 06000631 s13 EV0x0d] —
/// is asserted per phase.
#[test]
fn the_real_order_launch_chain_matches_the_teleported_one() {
    let iso = iso();
    // ---- phase A: the real order, walked (the walkreal-c route incl. the station detour) ----
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);
    let route: &[(&str, walker::Wp)] = &[
        ("the corridor south", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        ("door[7] through", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        ("the lane east", walker::Wp::new(10.0, 0.0, -82.0).r(1.3)),
        ("the lane south", walker::Wp::new(12.0, 0.0, -88.0).r(1.3)),
        ("the ramp foot", walker::Wp::new(13.8, 0.0, -90.8).r(1.1)),
        ("the ramp 1", walker::Wp::new(14.5, 0.6, -87.5).r(1.2)),
        ("the ramp 2", walker::Wp::new(14.5, 2.4, -84.5).r(1.2)),
        ("the ramp 3", walker::Wp::new(14.5, 4.6, -81.5).r(1.2)),
        ("the ring east", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the ring 1", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the ring 2", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the ring 3", walker::Wp::new(-11.0, 5.9, -81.0).r(1.4)),
        ("the ring 4", walker::Wp::new(-13.0, 5.9, -86.0).r(1.4)),
        ("the station west", walker::Wp::new(-16.5, 5.9, -88.0).r(1.3)),
        ("the station south", walker::Wp::new(-16.3, 5.9, -93.5).r(1.2)),
        ("the station E", walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2)),
        ("the return 1", walker::Wp::new(-16.3, 5.9, -89.0).r(1.3)),
        ("the return 2", walker::Wp::new(-13.0, 5.9, -85.0).r(1.4)),
        ("the return 3", walker::Wp::new(-10.0, 5.9, -81.0).r(1.4)),
        ("the return 4", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the return 5", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the return 6", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the descent 1", walker::Wp::new(14.5, 4.6, -82.0).r(1.2)),
        ("the descent 2", walker::Wp::new(14.5, 2.4, -85.0).r(1.2)),
        ("the descent 3", walker::Wp::new(14.5, 0.6, -88.0).r(1.2)),
        ("the east lane south", walker::Wp::new(9.5, 0.0, -100.0).r(1.6)),
        ("the trench's west end", walker::Wp::new(4.0, 0.0, -108.0).r(1.6)),
        ("door[0]'s west approach", walker::Wp::new(2.3, 0.0, -110.0).r(0.7)),
        ("door[0]'s threshold", walker::Wp::new(2.3, 0.0, -113.2).r(0.7)),
        ("door[0] through", walker::Wp::new(2.0, -0.1, -119.0).r(1.2)),
        ("door[1] through", walker::Wp::new(2.0, -0.1, -132.0).r(1.2)),
        ("door[3]'s panel", walker::Wp::at(3.4, 0.8, -141.5, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[3] off panel", walker::Wp::new(2.0, 0.0, -140.0).r(1.0)),
        ("door[3] through", walker::Wp::new(2.0, -0.1, -147.0).r(1.2)),
        ("door[2]'s panel", walker::Wp::at(3.4, 0.8, -151.9, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[2] off panel", walker::Wp::new(2.2, 0.0, -150.4).r(1.0)),
        ("door[2] through", walker::Wp::new(2.2, -0.1, -151.5).r(1.2)),
        ("the silo plate", walker::Wp::new(2.5, 0.3, -153.9).r(1.0)),
        ("door[4]'s panel", walker::Wp::at(3.5, 1.0, -165.7, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[4] through", walker::Wp::new(2.5, 1.0, -169.0).r(1.3)),
        // The staging point north of the ch-0x81 slab (its z span is −178.50..−174.32): the route
        // stops SHORT of the touch so the cinema can be observed real-time (skip off) from the
        // exact tick the slab fires — the walkreal legs that cross it run with skip on and FFwd
        // the cinema mid-leg.
        ("the slab's north edge", walker::Wp::new(2.4, 1.0, -172.0).r(0.8)),
    ];
    w.walk_route(route);
    // Manual real-time approach: face south (yaw π = atan2(0, −1)) and walk into the slab, so the
    // touch tick, the cinema's player-disable, the hand-back and ch 243 are all observed from this
    // loop (walk_to is opaque and its legs tick through the disable).
    w.skip_cutscenes = false;
    let mut t81 = None;
    for t in 0..900u32 {
        if t81.is_none() && w.scene.sim.as_ref().unwrap().drones.channels.on(0x81) { t81 = Some(t); }
        w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.disabled || sim.bond.pos[1] < -5.0 { break; }
    }
    let t81 = t81.expect("the real-order touch fires ch 0x81 on the walked approach [D #255]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8e), "the spawn plate fired [D #252]");
        assert!(sim.drones.channels.on(0x24), "the station E fired [D #250]");
        assert!(sim.drones.channels.on(0x8f), "the silo plate fired [D #253]");
        assert!(sim.drones.channels.on(0xb4), "door[4]'s panel set ch 0xb4 in the real order [D #270]");
        assert!(sim.drones.channels.on(0x81), "the bridge touch fired [D #255]");
    }
    // The chain plays out (no skip): record the re-enable, the pose, ch 243, the ch-0xb4 clear and
    // the chaser.
    w.skip_cutscenes = false;
    let (mut t_reenable, mut pose, mut t243, mut b4_cleared) = (None, None, None, None);
    let mut chaser_a = None;
    for t in 0..9000u32 {
        w.tick(Input::default(), false);
        let _ = t81;
        let sim = w.scene.sim.as_ref().unwrap();
        if t_reenable.is_none() && sim.bond.pos[1] < -20.0 { t_reenable = Some(t); pose = Some(sim.bond.pos); }
        if t243.is_none() && sim.drones.channels.on(243) { t243 = Some(t); }
        if b4_cleared.is_none() && t_reenable.is_some() && !sim.drones.channels.on(0xb4) { b4_cleared = Some(t); }
        if chaser_a.is_none() {
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)) {
                chaser_a = Some((i, t));
            }
        }
        if let (Some((_, ct)), Some(t243)) = (chaser_a, t243) { if t > t243.max(ct) + 50 { break; } }
    }
    let t_reenable = t_reenable.expect("the real-order chain re-enables Bond at the pit floor [D stream-8 t747]");
    let pose = pose.expect("the real-order hand-back pose recorded");
    let t243 = t243.expect("ch 243 rises in the real order [D launch-script stream]");
    let (_, ct_a) = chaser_a.expect("the chaser spawns in the real order (spawner #257 via ch 240) [D]");
    eprintln!("[order] REAL   : ch0x81 +{t81}, re-enable +{t_reenable} (touch→re-enable {}) pose ({:.2},{:.2},{:.2}) ch243 +{t243} (+{} after re-enable) 0xb4 cleared +{b4_cleared:?} chaser +{ct_a}",
        t_reenable - t81, pose[0], pose[1], pose[2], t243 - t_reenable);
    assert!((pose[0] - 3.57).abs() < 0.6 && (pose[2] + 179.55).abs() < 0.6 && (pose[1] + 22.76).abs() < 1.2,
        "the real-order hand-back pose is the [D] stream-8 pit-floor key (3.57,−22.76,−179.55) — the port's capsule rests ~0.4 lower [S], got {pose:?}");
    assert!((1540..=1680).contains(&(t243 - t_reenable)),
        "ch 243 lands ~1610 after the re-enable in the real order (got {}) — the [M absolute-Wait + D stream] model (1610), port 1603, live 1591 [L]", t243 - t_reenable);
    let delta_a = t243 - t_reenable;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0xb4), "the cinema's stream-13 t747 write cleared ch 0xb4 [D 06000631 s13 EV0x0d[180,0]]");
    }

    // ---- phase B: the teleported run (the sister's own setup: placed on trigger #255, no input) ----
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.top_up_health = false;
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = [2.4, 1.2, -175.5];
    }
    let (mut t81, mut t_reenable, mut pose, mut t243, mut t_fail) = (None, None, None, None, None);
    for t in 0..4200u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t81.is_none() && sim.drones.channels.on(0x81) { t81 = Some(t); }
        if t_reenable.is_none() && sim.bond.pos[1] < -20.0 { t_reenable = Some(t); pose = Some(sim.bond.pos); }
        if t243.is_none() && sim.drones.channels.on(243) { t243 = Some(t); }
        if sim.drones.mission.status() == 3 { t_fail = Some(t); break; }
    }
    let t81 = t81.expect("the teleported Bond touches #255 at once [L the sister's setup]");
    let t_reenable = t_reenable.expect("the teleported chain re-enables Bond at the pit floor [D stream-8 t747]");
    let pose = pose.expect("the teleported hand-back pose recorded");
    let t243 = t243.expect("ch 243 rises in the teleported run [D]");
    let t_fail = t_fail.expect("the teleported run fails through the death sequence [M; the sister pins +~450]");
    eprintln!("[order] TELEPORT: ch0x81 +{t81}, re-enable +{t_reenable} (touch→re-enable {}) pose ({:.2},{:.2},{:.2}) ch243 +{t243} (+{} after re-enable) fail +{t_fail} (+{} after ch243)",
        t_reenable - t81, pose[0], pose[1], pose[2], t243 - t_reenable, t_fail - t243);
    assert!((pose[0] - 3.57).abs() < 0.6 && (pose[2] + 179.55).abs() < 0.6 && (pose[1] + 22.76).abs() < 1.2,
        "the teleported hand-back pose is the same [D] stream-8 key, got {pose:?}");
    assert!((1540..=1680).contains(&(t243 - t_reenable)),
        "ch 243 lands ~1610 after the re-enable in the teleported run (got {})", t243 - t_reenable);
    assert!((440..=470).contains(&(t_fail - t243)),
        "the teleported fail rides the death sequence at the sister's pinned +~450 [M 0x19a6a8 RED_FADE_TICKS 448 + L 454], got {}", t_fail - t243);
    let delta_b = t243 - t_reenable;
    assert!((delta_a as i32 - delta_b as i32).abs() <= 60,
        "the order does not move ch 243's clock (real {delta_a} vs teleported {delta_b}) — the [D] census says it cannot: no event-0x11 branch exists in the level's scripts and no pre-chain SP touches the chain");
}


/// level-16-every-objective: the WHOLE walked mission on real input, no god mode — where it ends
/// and why. Fresh load, the carried 07000014/15 chain [D], the full route (the objectives' walk),
/// the launch chain, the launch blast. Pins the player-order coverage the port CAN prove and the
/// exact wall the rest sits behind:
///   * obj0 "Reach Drake's launch silo" completes on the walked route (the monitor folds it);
///   * the launch chain re-enables Bond INSIDE HurtPoly-ShuttleLaunch #309's box at the pit floor
///     [M Player_Enable 0x195498 teleports to the stream-8 key; D the pack cinema's single t747
///     key (3.57,−22.76,−179.55); the volume x −30.7..35.3, y −24.1..−21.3, z −197.3..−133.8 [D]];
///   * ch 243 rises at the launch script's tick 1611 and the death sequence fails the mission at
///     pulse+~450 [D stream-6; M 0x19a6a8 RED_FADE_TICKS 448; L 454] — the pit holds no walking
///     escape before the pulse (the pit door locks on ch 240/241, no ladder/wire placement, the
///     lift summons after the pulse) [D; hurt-volumes.md U-051], and the game's escape is the
///     pack (jetpack) flight the port has no substate for [M the substate list] — so the walked
///     mission takes the FAIL exit here, with obj1 ("Prevent Kiko from launching into space",
///     the KikoDeathButton #693 past the escape) unreachable.
#[test]
fn the_walked_mission_ends_at_the_launch_blast_until_the_pack_flight_is_ported() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8e) && sim.drones.mission.states[21].iter().all(|&s| s == 4),
            "the carried record 21 completes on the arrival [D]");
        assert_eq!(sim.drones.mission.states[22][0], 3, "obj0 announces [L record 22]");
    }
    let route: &[(&str, walker::Wp)] = &[
        ("the corridor south", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        ("door[7] through", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        ("the lane east", walker::Wp::new(10.0, 0.0, -82.0).r(1.3)),
        ("the lane south", walker::Wp::new(12.0, 0.0, -88.0).r(1.3)),
        ("the ramp foot", walker::Wp::new(13.8, 0.0, -90.8).r(1.1)),
        ("the ramp 1", walker::Wp::new(14.5, 0.6, -87.5).r(1.2)),
        ("the ramp 2", walker::Wp::new(14.5, 2.4, -84.5).r(1.2)),
        ("the ramp 3", walker::Wp::new(14.5, 4.6, -81.5).r(1.2)),
        ("the ring east", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the ring 1", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the ring 2", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the ring 3", walker::Wp::new(-11.0, 5.9, -81.0).r(1.4)),
        ("the ring 4", walker::Wp::new(-13.0, 5.9, -86.0).r(1.4)),
        ("the station west", walker::Wp::new(-16.5, 5.9, -88.0).r(1.3)),
        ("the station south", walker::Wp::new(-16.3, 5.9, -93.5).r(1.2)),
        ("the station E", walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2)),
        ("the return 1", walker::Wp::new(-16.3, 5.9, -89.0).r(1.3)),
        ("the return 2", walker::Wp::new(-13.0, 5.9, -85.0).r(1.4)),
        ("the return 3", walker::Wp::new(-10.0, 5.9, -81.0).r(1.4)),
        ("the return 4", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("the return 5", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("the return 6", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("the descent 1", walker::Wp::new(14.5, 4.6, -82.0).r(1.2)),
        ("the descent 2", walker::Wp::new(14.5, 2.4, -85.0).r(1.2)),
        ("the descent 3", walker::Wp::new(14.5, 0.6, -88.0).r(1.2)),
        ("the east lane south", walker::Wp::new(9.5, 0.0, -100.0).r(1.6)),
        ("the trench's west end", walker::Wp::new(4.0, 0.0, -108.0).r(1.6)),
        ("door[0]'s west approach", walker::Wp::new(2.3, 0.0, -110.0).r(0.7)),
        ("door[0]'s threshold", walker::Wp::new(2.3, 0.0, -113.2).r(0.7)),
        ("door[0] through", walker::Wp::new(2.0, -0.1, -119.0).r(1.2)),
        ("door[1] through", walker::Wp::new(2.0, -0.1, -132.0).r(1.2)),
        ("door[3]'s panel", walker::Wp::at(3.4, 0.8, -141.5, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[3] off panel", walker::Wp::new(2.0, 0.0, -140.0).r(1.0)),
        ("door[3] through", walker::Wp::new(2.0, -0.1, -147.0).r(1.2)),
        ("door[2]'s panel", walker::Wp::at(3.4, 0.8, -151.9, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[2] off panel", walker::Wp::new(2.2, 0.0, -150.4).r(1.0)),
        ("door[2] through", walker::Wp::new(2.2, -0.1, -151.5).r(1.2)),
        ("the silo plate", walker::Wp::new(2.5, 0.3, -153.9).r(1.0)),
        ("door[4]'s panel", walker::Wp::at(3.5, 1.0, -165.7, walker::Act::Use { use_ticks: 90 }).r(0.8)),
        ("door[4] through", walker::Wp::new(2.5, 1.0, -169.0).r(1.3)),
        ("the south room", walker::Wp::new(2.4, 1.0, -175.0).r(1.2)),
    ];
    w.walk_route(route);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[22][0], 4, "obj0 'Reach Drake's launch silo' completes on the walked route [D #253]");
        assert_eq!(sim.drones.mission.states[22][1], 3, "obj1 revealed, waiting on the button [D]");
    }
    // The ch-0x81 slab crossed on the walked approach (the walk crosses z −174.32..−178.50 into
    // the south room) → the pack cinema plays → the hand-back at the pit floor.
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x81), "the launch chain head fired on the walk [D #255]");
    w.skip_cutscenes = false;
    let (mut t_reenable, mut t243, mut t_fail) = (None, None, None);
    for _ in 0..9000u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t_reenable.is_none() && sim.bond.pos[1] < -20.0 { t_reenable = Some(sim.drones.frame); }
        if t243.is_none() && sim.drones.channels.on(243) { t243 = Some(sim.drones.frame); }
        if sim.drones.mission.status() == 3 { t_fail = Some(sim.drones.frame); break; }
    }
    let t_reenable = t_reenable.expect("the walked chain re-enables Bond at the pit floor [M Player_Enable 0x195498]");
    let t243 = t243.expect("ch 243 rises on the walked run [D stream-6 tick 1611]");
    let t_fail = t_fail.expect("the walked mission fails at the launch blast — the port has no pack flight to escape it [D; U-051]");
    eprintln!("[16-walk] re-enable f{t_reenable}, ch243 f{t243} (+{}), fail f{t_fail} (+{} after ch243)", t243 - t_reenable, t_fail - t243);
    assert!((1540..=1700).contains(&(t243 - t_reenable)),
        "the pulse lands ~1611 after the re-enable on the walked run (got {}) [D stream-6; the model 1610, port 1603, live 1591]", t243 - t_reenable);
    assert!((440..=470).contains(&(t_fail - t243)),
        "the walked fail rides the death sequence at +~450 after the pulse [M 0x19a6a8 RED_FADE_TICKS 448; L 454], got {}", t_fail - t243);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[22][0], 4, "obj0 stays complete through the fail");
        assert_eq!(sim.drones.mission.states[22][1], 3, "obj1 never had its writer reached: the button is past the escape the pack buys [D #693; U-051]");
    }
}
