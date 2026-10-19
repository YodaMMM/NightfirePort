//! walkreal-c: 07000016 (the missile base launch centre) walked with the player's own movement —
//! no `bond.pos` writes after the spawn. The route (docs/levels/07000016.md; the walkreal_probe_c
//! dump): the arrival (5.17, 0.90, −65.30, the carried 07000014/15 chain) → the spawn plate #252
//! fires ch 0x8e (completes the carried record 21) → the south corridor (doors [7]/[0]/[1]/[3]/[2]
//! — [0]/[3]/[2] are locked by ch 0x24/0x27/0x28, whose writers are the mask-0x04 activate
//! triggers beside them; the stuck-mitigation's E press presses them) → the silo plate #253 fires
//! ch 0x8f (completes "Reach Drake's launch silo", reveals the Kiko objective) → the south room's
//! ch-0x81 touch trigger at (2.4, 1.0, −176.4) fires the launch chain [D] (the cinema plays OUT —
//! no skip: its stream writes ch 185 → SP #272 → ch 240/242 → the spawners) → the chaser (Kiko,
//! #261) spawns and is defeated with Bond's own fire → ch 0x90 → AND #271 → ch 100 → MissionState
//! 6 → the driving hand-off 0x7300017 [M 0x182c60:111-112].
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance). Kiko dies to Bond's own bullets. Opt-in via NF_ISO.

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
/// walks through the leaf's span. FAIL-FIRST: on the walkreal-c branch (before the straddle
/// walk-in landed in door.rs) the E never moved the leaf (14 use_door calls, state 0x2 — the
/// report's blocker), and with the old x 3.5 lane the walk wedged on the sealed west half at
/// (3.5, 0.9, −73.58) even with the door open (the walkreal-c stuck dump; the lane fix is part of
/// this task). Sisters: the static corridor floor holds Bond throughout (grounded, no sink), and
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
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && d.death_channel == 0x90) {
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

/// The finale tail: the chaser's defeat → ch 0x90 → AND #271 → ch 100 → MissionState 6 → the
/// driving hand-off 0x7300017 [M 0x182c60:111-112]. PARKED on U-051 (docs/UNKNOWNS.md): the
/// ch-0xf3-gated HurtPoly-ShuttleLaunch #309 (type 2, 1000 damage [D params 0:2 1:0x3e8 2:0xf3])
/// goes live while this walk's Bond stands on the silo-pit floor (−23.2 — he crosses the ch-0x81
/// slab on the y-0.25 bridge, the launch chain plays out, and the pit floor is where both this
/// walk and fix_finale_live's parked Bond end up), and the volume's 1000 damage kills him before
/// the chaser's defeat — the walk observes the mission FAIL exit request instead of the complete.
/// Open [L]: where the GAME's Bond stands when ch 243 rises, and when ch 243 rises — a fresh-boot
/// PCSX2 watch of switch_channels[243] (0x26fc90+243) and Bond's position through the launch
/// cinema settles whether the port's cinema-Bond placement or the volume's live window is wrong.
#[test]
#[ignore = "U-051 ANSWERED (docs/systems/hurt-volumes.md): the volume window is 0x0600090a stream 6 ticks 1611..2170 [D] and the game's Bond escapes on the pack flight — the port has no jetpack substate, so pit-floor Bond dies; the re-seat stand-in is documented in fix_finale_live"]
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
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && d.death_channel == 0x90) {
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

    let mut complete = false;
    for _ in 0..900u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission.status() == 6 { complete = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x90), "the chaser's defeat raises ch 0x90 [S kiko.rs]");
        assert!(sim.drones.channels.on(100), "AND #271 [ch 0x90] → ch 100 [D #271]");
        assert!(complete, "the final mission completes (MissionState 6 [M 0x182c60:85-116])");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0017, "the driving hand-off [M 0x182c60:111-112]");
        assert!(sim.drones.mission.states[21].iter().all(|&s| s == 4), "the carried 07000015 record stays complete");
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
