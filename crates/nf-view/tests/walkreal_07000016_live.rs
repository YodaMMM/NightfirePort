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
//! header) — `walk_arrival_to_the_chasers_spawn` stops at her spawn; `walk_the_chasers_defeat_completes_the_mission`
//! walks the rest (the blast-room shelter through both launch pulses, the pit squad, the lift-out
//! cinema 0x06000632, the button) to MissionState 6. Opt-in
//! via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The save the game hands 16 at the end of the chain: 07000014's start kit [M 0x1a7ef8 case
/// 0x7000014] carried warm into 07000015 [M case 0x7000015: `Player_RamLoad`, the saved weapon
/// re-selected, `Player_CheckWeaponsLoaded` 0x1a97c8 sweeping the skins 15 lacks] and captured by
/// 15's exit (`warm_exit_save` = Trigger_Activate's `Player_RamSave` 0x195670 [M 0x1b2318:97-116]).
/// Built once per test process.
fn chain_save(iso: &str) -> RamSave {
    static SAVE: std::sync::OnceLock<RamSave> = std::sync::OnceLock::new();
    SAVE.get_or_init(|| {
        let s14 = nf_view::warm_exit_save(nf_view::load(iso, "07000014.bin", false).unwrap().sim.as_ref().unwrap());
        let s15 = nf_view::warm_exit_save(nf_view::load_with_carry(iso, "07000015.bin", false, Some(&s14)).unwrap().sim.as_ref().unwrap());
        s15
    }).clone()
}

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    // Bond's player state from the real chain (health, armour, weapons, held weapon), with only the
    // carried gates as channels: the save Trigger_Activate would make [M 0x1b2318:97-116]. This used
    // to be the 07000005 start inventory with a saved weapon byte of 0 — `Player_InitWeapon`'s tail
    // turns an unowned request into the fists [M 0x1a7ef8:321-324], so every walk reached the
    // chaser holding the fists, which no chain into 16 produces.
    let mut s = chain_save(&iso());
    s.channels = ch.v.clone();
    s.ch_time = ch.time.clone();
    s.mission = None;
    s.tally = None;
    s
}

/// The weapons the 15 → 16 chain lands on 16 [M 0x1a7ef8 case 0x7000016 → RamLoad keeps the
/// carry; 0x1a97c8 drops the skins 16 lacks]: the fists, the P2K, the Delta Repeater 0x11, the
/// frag 0x34, the mine 0x37, the laser 0x4e and the camera 0x54. The Grapple 0x50 was already
/// swept on 15 (its skin is not in 15's archive), the decryptor 0x56 is swept here.
const CHAIN_OWNED: [usize; 7] = [1, 2, 0x11, 0x34, 0x37, 0x4e, 0x54];
/// A cold load of 16 runs the fresh 14 kit [M 0x1a7ef8 case 0x7000016: the P2K is unowned on a cold
/// load (`Player_RamLoad` is skipped, 0x1957a8:9) → `switchD_001a8284_caseD_7000014`]. 16's archive
/// holds the Grapple's skin, so its equip lands; the Stunner 0x4a and the decryptor 0x56 refuse
/// [M 0x1a8a58:20]. The only difference from the chain is the Grapple, and the game has it too.
const COLD_OWNED: [usize; 8] = [1, 2, 0x11, 0x34, 0x37, 0x4e, 0x50, 0x54];

fn owned(sim: &nf_view::scene::Sim) -> Vec<usize> {
    let inv = &sim.drones.inventory;
    (0..inv.owned.len()).filter(|&w| inv.owned[w]).collect()
}

/// Bond's weapons where the chaser fight starts: what he owns and what is in his hand.
fn assert_armed_for_the_chaser(sim: &nf_view::scene::Sim, want: &[usize], ctx: &str) {
    assert_eq!(owned(sim), want, "{ctx}: Bond's weapons at the chaser fight");
    assert_eq!(sim.gun.weapon, 0x11, "{ctx}: the Delta Repeater is in Bond's hand at the chaser fight (the request 0x11 drawn) [M 0x1a7ef8]");
    assert!(sim.gun.clip + sim.gun.spare > 0, "{ctx}: the weapon in hand has bolts (clip {} spare {})", sim.gun.clip, sim.gun.spare);
    let inv = &sim.drones.inventory;
    assert!(inv.clip[2] + inv.carried[1] > 0, "{ctx}: the P2K has rounds (clip {} spare {})", inv.clip[2], inv.carried[1]);
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
    // The port walks the game's own route onto #255 (APPROACH: the corridor, the station E for
    // door[0]'s 0x24, the panels #684/#685/#617 for 0x27/0x28/0xb4 — tower_door1 #392's lock is
    // 0xb4, the south room's only way in [D placements]) and steps onto the slab in real time.
    // The live session placed Bond on #255 instead [L]; the_real_order_launch_chain_matches_the_teleported_one
    // pins that the two orders run the chain on the same clock.
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);
    walk_onto_the_slab(&mut w);
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

/// The proven approach (walk_arrival_to_the_chasers_spawn's legs: the corridor, the station
/// detour for door[0]'s 0x24, the panels for 0x27/0x28/0xb4, the silo plate) on to the bridge
/// slab #255 — the ch-0x81 touch.
const APPROACH: &[(&str, walker::Wp)] = &[
    ("the corridor south", walker::Wp { at: [4.3, 0.6, -72.5], act: walker::Act::None, arrive: 1.2 }),
    ("door[7] through", walker::Wp { at: [4.9, -0.1, -78.0], act: walker::Act::None, arrive: 1.2 }),
    ("the lane east", walker::Wp { at: [10.0, 0.0, -82.0], act: walker::Act::None, arrive: 1.3 }),
    ("the lane south", walker::Wp { at: [12.0, 0.0, -88.0], act: walker::Act::None, arrive: 1.3 }),
    ("the ramp foot", walker::Wp { at: [13.8, 0.0, -90.8], act: walker::Act::None, arrive: 1.1 }),
    ("the ramp 1", walker::Wp { at: [14.5, 0.6, -87.5], act: walker::Act::None, arrive: 1.2 }),
    ("the ramp 2", walker::Wp { at: [14.5, 2.4, -84.5], act: walker::Act::None, arrive: 1.2 }),
    ("the ramp 3", walker::Wp { at: [14.5, 4.6, -81.5], act: walker::Act::None, arrive: 1.2 }),
    ("the ring east", walker::Wp { at: [14.5, 5.9, -79.0], act: walker::Act::None, arrive: 1.2 }),
    ("the ring 1", walker::Wp { at: [6.0, 5.9, -78.5], act: walker::Act::None, arrive: 1.4 }),
    ("the ring 2", walker::Wp { at: [-4.0, 5.9, -79.0], act: walker::Act::None, arrive: 1.4 }),
    ("the ring 3", walker::Wp { at: [-11.0, 5.9, -81.0], act: walker::Act::None, arrive: 1.4 }),
    ("the ring 4", walker::Wp { at: [-13.0, 5.9, -86.0], act: walker::Act::None, arrive: 1.4 }),
    ("the station west", walker::Wp { at: [-16.5, 5.9, -88.0], act: walker::Act::None, arrive: 1.3 }),
    ("the station south", walker::Wp { at: [-16.3, 5.9, -93.5], act: walker::Act::None, arrive: 1.2 }),
    ("the station E", walker::Wp { at: [-15.2, 5.9, -94.6], act: walker::Act::Use { use_ticks: 120 }, arrive: 1.2 }),
    ("the return 1", walker::Wp { at: [-16.3, 5.9, -89.0], act: walker::Act::None, arrive: 1.3 }),
    ("the return 2", walker::Wp { at: [-13.0, 5.9, -85.0], act: walker::Act::None, arrive: 1.4 }),
    ("the return 3", walker::Wp { at: [-10.0, 5.9, -81.0], act: walker::Act::None, arrive: 1.4 }),
    ("the return 4", walker::Wp { at: [-4.0, 5.9, -79.0], act: walker::Act::None, arrive: 1.4 }),
    ("the return 5", walker::Wp { at: [6.0, 5.9, -78.5], act: walker::Act::None, arrive: 1.4 }),
    ("the return 6", walker::Wp { at: [14.5, 5.9, -79.0], act: walker::Act::None, arrive: 1.2 }),
    ("the descent 1", walker::Wp { at: [14.5, 4.6, -82.0], act: walker::Act::None, arrive: 1.2 }),
    ("the descent 2", walker::Wp { at: [14.5, 2.4, -85.0], act: walker::Act::None, arrive: 1.2 }),
    ("the descent 3", walker::Wp { at: [14.5, 0.6, -88.0], act: walker::Act::None, arrive: 1.2 }),
    ("the east lane south", walker::Wp { at: [9.5, 0.0, -100.0], act: walker::Act::None, arrive: 1.6 }),
    ("the trench's west end", walker::Wp { at: [4.0, 0.0, -108.0], act: walker::Act::None, arrive: 1.6 }),
    ("door[0]'s west approach", walker::Wp { at: [2.3, 0.0, -110.0], act: walker::Act::None, arrive: 0.7 }),
    ("door[0]'s threshold", walker::Wp { at: [2.3, 0.0, -113.2], act: walker::Act::None, arrive: 0.7 }),
    ("door[0] through", walker::Wp { at: [2.0, -0.1, -119.0], act: walker::Act::None, arrive: 1.2 }),
    ("door[1] through", walker::Wp { at: [2.0, -0.1, -132.0], act: walker::Act::None, arrive: 1.2 }),
    ("door[3]'s panel", walker::Wp { at: [3.4, 0.8, -141.5], act: walker::Act::Use { use_ticks: 90 }, arrive: 0.8 }),
    ("door[3] off panel", walker::Wp { at: [2.0, 0.0, -140.0], act: walker::Act::None, arrive: 1.0 }),
    ("door[3] through", walker::Wp { at: [2.0, -0.1, -147.0], act: walker::Act::None, arrive: 1.2 }),
    ("door[2]'s panel", walker::Wp { at: [3.4, 0.8, -151.9], act: walker::Act::Use { use_ticks: 90 }, arrive: 0.8 }),
    ("door[2] off panel", walker::Wp { at: [2.2, 0.0, -150.4], act: walker::Act::None, arrive: 1.0 }),
    ("door[2] through", walker::Wp { at: [2.2, -0.1, -151.5], act: walker::Act::None, arrive: 1.2 }),
    ("the silo plate", walker::Wp { at: [2.5, 0.3, -153.9], act: walker::Act::None, arrive: 1.0 }),
    ("door[4]'s panel", walker::Wp { at: [3.5, 1.0, -165.7], act: walker::Act::Use { use_ticks: 90 }, arrive: 0.8 }),
    ("door[4] through", walker::Wp { at: [2.5, 1.0, -169.0], act: walker::Act::None, arrive: 1.3 }),
    ("the bridge slab #255", walker::Wp { at: [2.4, 1.0, -176.8], act: walker::Act::None, arrive: 1.2 }),
];

/// APPROACH up to the staging point north of the ch-0x81 slab (its z span −178.50..−174.32 [D
/// trigger bbox]), then a real-time step south onto it with the walkreal allowances OFF: the pack
/// cinema plays out (no skip) and the hurt contact lands (no top-up) — the sisters pin the chain's
/// TIMING from the touch on. Returns with ch 0x81 just raised and Bond's input otherwise idle.
fn walk_onto_the_slab(w: &mut Walker) {
    w.walk_route(&APPROACH[..APPROACH.len() - 1]);
    w.walk_route(&[("the slab's north edge", walker::Wp::new(2.4, 1.0, -172.0).r(0.8))]);
    w.top_up_health = false;
    w.skip_cutscenes = false;
    for _ in 0..900u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x81) { return; }
        w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
    }
    panic!("the walked step south never touched #255 (ch 0x81) at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
}

/// SISTER for the placed order: with ch 0xb4 cold, tower_door1 #392 (2.46, 1.04, −166.38; lock
/// 0xb4 [D params]) stays shut under a walked Bond's forward press (no E) — the locked leaf is
/// solid [M Door lock 0x136810:10-16] — so #255 (ch 0x81) cannot be reached without the panel
/// #617's E. Bond must actually reach the leaf (z < −164.6) for the hold to count.
fn tower_door1_holds_without_its_panel(w: &mut Walker) {
    let di = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0xb4), "0xb4 is cold before the panel");
        let di = sim.drones.doors.list.iter().position(|d| (d.pos[2] + 166.38).abs() < 0.5 && (d.pos[0] - 2.46).abs() < 0.5)
            .expect("tower_door1 #392");
        assert_eq!(sim.drones.doors.list[di].ch_lock, 0xb4, "tower_door1's lock is 0xb4 [D]");
        di
    };
    let mut min_z = f32::MAX;
    for _ in 0..360u32 {
        w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        min_z = min_z.min(w.scene.sim.as_ref().unwrap().bond.pos[2]);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.doors.list[di];
    eprintln!("[sister] tower_door1 state {:#x} pos01 {:.3}, Bond min z {min_z:.2}", d.state, d.pos01);
    assert!(min_z < -164.6, "the walk reached the leaf (min z {min_z:.2})");
    assert!(min_z > -167.0, "the locked leaf held Bond north of it (min z {min_z:.2})");
    assert!(d.pos01 < 0.05 && !sim.drones.channels.on(0xb4) && !sim.drones.channels.on(0x81),
        "no E → no 0xb4 → the leaf stays shut and #255 untouched (state {:#x}, pos01 {:.3})", d.state, d.pos01);
}

/// From the pit floor into the west blast room (blast_room_shelter_16_live: the room is outside
/// HurtPoly-ShuttleLaunch's polygon [D], its door leaves shut over both pulses [D 0600090a
/// streams 17/18]).
const INTO_THE_SHELTER: &[(&str, walker::Wp)] = &[
    ("the pit floor west", walker::Wp { at: [-8.5, -22.2, -173.5], act: walker::Act::None, arrive: 1.0 }),
    ("the west blast door", walker::Wp { at: [-11.0, -22.0, -170.0], act: walker::Act::None, arrive: 0.9 }),
    ("the west blast room", walker::Wp { at: [-15.2, -22.0, -170.4], act: walker::Act::None, arrive: 0.8 }),
];

/// Ticks until `done`, or panics with `what` after `max`.
fn tick_until(w: &mut Walker, max: u32, what: &str, done: impl Fn(&nf_view::scene::Sim) -> bool) -> u32 {
    for t in 0..max {
        if done(w.scene.sim.as_ref().unwrap()) { return t; }
        w.tick(Input::default(), false);
    }
    panic!("{what}: not within {max} ticks");
}

/// Sits one ch-243 pulse out where Bond stands, with the walker's damage allowance OFF (no
/// top-up can hide a contact): every tick of the pulse Bond's own hurt capsule (the port's
/// `Hurt::update` body) must miss HurtPoly-ShuttleLaunch, and he ends it alive with the
/// mission not failed. Returns the pulse length.
fn sit_out_pulse(w: &mut Walker, label: &str) -> u32 {
    let allowance = w.top_up_health;
    w.top_up_health = false;
    tick_until(w, 8000, label, |s| s.drones.channels.on(243));
    let mut n = 0u32;
    while w.scene.sim.as_ref().unwrap().drones.channels.on(243) {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let h = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("HurtPoly-ShuttleLaunch");
            assert!(!h.contains(p, [p[0], p[1] + 1.8, p[2]], 0.55), "{label}: the volume touches Bond at {p:?} (+{n})");
            assert!(sim.drones.health.health > 0.0, "{label}: Bond died at {p:?} (+{n})");
        }
        w.tick(Input::default(), false);
        n += 1;
        assert!(n < 1000, "{label}: the pulse never fell");
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.health.health > 0.0 && sim.drones.mission.status() != 3, "{label}: Bond lives, no fail");
    w.top_up_health = allowance;
    n
}

/// E beats aimed at `at` until `done` (the interactables press shape: the game's Player_Activate
/// dispatch, one edge per 6 ticks).
fn press_use_until(w: &mut Walker, at: [f32; 3], max: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) -> bool {
    for k in 0..max {
        if done(w.scene.sim.as_ref().unwrap()) { return true; }
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (at[0] - sim.bond.pos[0]).atan2(at[2] - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let to = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
        let want_pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: (want_pitch - sim.bond.pitch).clamp(-0.06, 0.06), ..Default::default() };
        w.tick(input, k % 6 == 0);
    }
    false
}

/// THE FINALE, walked on real input to the mission's completion. The chain, every link from the
/// disc [D] or the decompile [M]; the times are the launch script 0x0600090a's absolute clock,
/// which starts one tick after the pack cinema 0x06000631 hands Bond back on the pit floor
/// (stream 8's single key — the bridge slab #255's touch fires ch 0x81 → the cinema):
/// - **pulse 1 (1611..2170) in the shelter.** Bond runs into the west blast room while its door
///   is open (0..1000) — HurtPoly-ShuttleLaunch's polygon leaves the room out and the leaves shut
///   over 1350..2230 [D; blast_room_shelter_16_live pins both, with the lethal-floor sisters].
/// - **the chaser.** ch 240 @2180 wakes spawner #604 (group 6: Kiko #608 + three); her death
///   raises nothing (param 12 = 0 [D]) — ch 0x90's writer is the KikoDeathButton #693.
/// - **pulse 2 (6810..7320) in the shelter again** (the leaves shut over 6660..7360 [D]).
/// - **the pit squad.** ch 242 @7332 wakes spawner #640 (group 7: #641..#644, death channels
///   0x1a/0x19/0x18/0x17 [D]); their deaths → mux #611 → ch 0xbf, with ch 0xc0 @7808 (stream 20)
///   → mux #633 → ch 0x82 → Script Player #610 → pack cinema 0x06000632 [D].
/// - **the lift-out.** 0x06000632 (07900016.bin): event 4 disables Bond at t0, stream 16 clears
///   ch 141 = 0x8d (the button's inhibit gate, armed by the corridor touch #616 on the way in) at
///   t1056, and stream 13's event 6 at t1057 hands Bond back to stream 12's dummy at key 1868 =
///   (5.39, 1.34, −158.31) — the shuttle-bay floor beside the button [D keys; M
///   Script_EventHandler 0x1d94c0 event 6 → Script_Free → Player_Enable 0x195498].
/// - **the button.** Bond's E on #693 (gate 0x8d now off [M Switch_Activate 0x1b02a8:22]) →
///   ch 0x90 → mux #618 → ch 100 → MissionState 6 → the driving hand-off 0x7300017
///   [M 0x182c60:111-112].
/// Input only after the spawn (steer/fire/use; the walker's damage allowance tops up Bond's
/// health against the guards — the volume's own contact count is asserted 0 instead, so the
/// allowance cannot hide a pulse death).
#[test]
fn walk_the_chasers_defeat_completes_the_mission() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    let (chaser, t0) = walk_to_the_chaser_fight(&mut w);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &CHAIN_OWNED, "the 15 → 16 chain");
    assert!(w.shoot(chaser, "the chaser"), "the chaser dies to Bond's fire");
    finish_after_the_chaser(&mut w, t0);
}

/// Bond's weapons at the chaser fight after the real 15 → 16 hand-off: 16 loads with 15's exit
/// save and `Player_InitWeapon` case 0x7000016 keeps the carry and re-selects the saved weapon
/// [M 0x1a7ef8:302-307; 0x1957a8]. Before the harness carried the chain's save, the walks reached
/// her with the fists in hand. Sister: the arrival holds the same set, so nothing on the way (the
/// pack cinema's Player_Disable 0x195f70 / Player_Enable 0x195498 touch no weapon state [M]) takes
/// a weapon away.
#[test]
fn bond_is_armed_at_the_chaser_fight_after_the_15_handoff() {
    let iso = iso();
    let save = carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]);
    assert_eq!(save.weapon, 0x11, "15's exit saves the Delta Repeater as the held weapon [M 0x195670:18-19]");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&save)).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(60);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &CHAIN_OWNED, "the 15 → 16 arrival");
    walk_to_the_chaser_fight(&mut w);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &CHAIN_OWNED, "the 15 → 16 chain");
}

/// The same pin on a direct (cold) load of 16: the fresh 14 kit [M 0x1a7ef8 case 0x7000016 →
/// caseD_7000014], the Delta Repeater in hand. Sister of the hand-off test: the two sets differ
/// only by the Grapple 0x50, which 15's skin sweep removes on the chain [M 0x1a97c8] and 16's
/// own archive keeps on a cold load.
#[test]
fn bond_is_armed_at_the_chaser_fight_on_a_direct_load() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000016.bin", false).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(60);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &COLD_OWNED, "the cold 16 arrival");
    walk_to_the_chaser_fight(&mut w);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &COLD_OWNED, "the cold 16 load");
    let diff: Vec<usize> = COLD_OWNED.iter().copied().filter(|w| !CHAIN_OWNED.contains(w)).collect();
    assert_eq!(diff, vec![0x50], "the two paths differ only by the Grapple, as in the game [M 0x1a97c8]");
}

/// From the arrival to the chaser fight: the approach, the pack cinema (played out), pulse 1 in
/// the shelter, the chaser's spawn and the door reopening. Returns the chaser's index and the
/// hand-back tick.
fn walk_to_the_chaser_fight(w: &mut Walker) -> (usize, u32) {
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(100);
    w.walk_route(APPROACH);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x8d), "the route crossed the corridor touch #616 — the button's gate is armed [D]");
    // The launch chain plays out (no skip): ch 0x81 → the pack cinema → the hand-back on the pit
    // floor [D stream-8 key].
    w.skip_cutscenes = false;
    tick_until(w, 3000, "the pack cinema's hand-back on the pit floor", |s| s.drones.channels.on(0x81) && !s.disabled && s.bond.pos[1] < -20.0);
    let t0 = w.total_ticks;
    eprintln!("[finale] hand-back at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w.skip_cutscenes = true;
    w.walk_route(INTO_THE_SHELTER);
    let inside_at = w.total_ticks - t0;
    eprintln!("[finale] in the shelter +{inside_at}");
    assert!(inside_at < 1000, "Bond is in the room before its door starts to shut (+{inside_at}, the leaves move 1000..1350 [D])");
    // Pulse 1.
    let n1 = sit_out_pulse(w, "pulse 1 in the shelter");
    assert!((555..=562).contains(&n1), "pulse 1 is 1611..2170 = 559 ticks [D stream 6], got {n1}");
    assert!(w.scene.sim.as_ref().unwrap().drones.mission.status() != 3, "no fail");
    // The chaser: ch 240 @2180 spawns her; the door reopens 2230..2580.
    let chaser = {
        tick_until(w, 2000, "the chaser spawns", |s| s.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)));
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)).unwrap()
    };
    while w.total_ticks - t0 < 2600 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[finale] the chaser #{chaser} at {:?}, +{}; Bond holds {:#x} (clip {} spare {}), owns {:x?}",
            sim.drones.list[chaser].pos, w.total_ticks - t0, sim.gun.weapon, sim.gun.clip, sim.gun.spare, owned(sim));
    }
    (chaser, t0)
}

/// After the chaser's death: pulse 2, the pit squad, the lift-out cinema, the button.
fn finish_after_the_chaser(w: &mut Walker, t0: u32) {
    for _ in 0..100 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x90), "the chaser's death raises ch 0x90 — her death channel is param 12 = 0 [D #261]; the writer is the switch #693 [D]");
        assert!(!sim.drones.channels.on(100), "mux #618 stays cold on the kill [D]");
        assert!(sim.drones.mission.status() != 6, "the mission does not complete on the kill [D AND #271 waits for the button]");
    }
    eprintln!("[finale] the chaser down +{}", w.total_ticks - t0);
    // Back into the shelter before the leaves shut again (6310..6660 [D]).
    w.walk_route(INTO_THE_SHELTER);
    let back_at = w.total_ticks - t0;
    eprintln!("[finale] back in the shelter +{back_at}");
    assert!(back_at < 6310, "back in the room before its door shuts for pulse 2 (+{back_at})");
    let n2 = sit_out_pulse(w, "pulse 2 in the shelter");
    assert!((506..=513).contains(&n2), "pulse 2 is 6810..7320 = 510 ticks [D stream 9], got {n2}");
    eprintln!("[finale] pulse 2 over +{}", w.total_ticks - t0);
    // The pit squad (group 7, ch 242 @7332): all four must fall for ch 0xbf.
    tick_until(w, 600, "the pit squad spawns", |s| s.drones.list.iter().any(|d| d.present && !d.corpse && matches!(d.death_channel, 23..=26)));
    for _ in 0..4 {
        let next = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0 && matches!(d.death_channel, 23..=26))
                .min_by(|a, b| walker::dxz(a.1.pos, bp).total_cmp(&walker::dxz(b.1.pos, bp)))
                .map(|(i, _)| i)
        };
        let Some(i) = next else { break };
        eprintln!("[finale] squad #{i} (death ch {:#x}) at {:?}", w.scene.sim.as_ref().unwrap().drones.list[i].death_channel, w.scene.sim.as_ref().unwrap().drones.list[i].pos);
        assert!(w.shoot(i, "the pit squad"), "the pit squad member #{i} dies to Bond's fire");
    }
    tick_until(w, 600, "the pit squad's deaths raise ch 0xbf [D mux #611]", |s| s.drones.channels.on(0xbf));
    eprintln!("[finale] ch 0xbf +{}", w.total_ticks - t0);
    // ch 0xc0 @7808 → ch 0x82 → 0x06000632 plays out (no skip) and hands Bond back at the bay.
    w.skip_cutscenes = false;
    tick_until(w, 3000, "ch 0x82 starts the lift-out cinema [D mux #633]", |s| s.drones.channels.on(0x82) && s.disabled);
    tick_until(w, 1500, "the lift-out cinema hands Bond back", |s| !s.disabled);
    w.skip_cutscenes = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[finale] lift-out hand-back at {p:?} +{}", w.total_ticks - t0);
        assert!((p[0] - 5.39).abs() < 0.5 && (p[1] - 1.34).abs() < 1.0 && (p[2] + 158.31).abs() < 0.5,
            "0x06000632 hands Bond back at its key 1868 (5.39, 1.34, −158.31) [D], got {p:?}");
        assert!(!sim.drones.channels.on(0x8d), "0x06000632 stream 16 cleared the button's gate ch 0x8d [D t1056]");
    }
    // The button (the kiko_button_real_walk lane from the bay to the button's side).
    w.walk_to(walker::Wp::new(6.3, 1.0, -155.45).r(0.6), "the button's side");
    assert!(press_use_until(w, [7.02, 1.46, -155.61], 240, |s| s.drones.channels.on(0x90)),
        "Bond's E on the KikoDeathButton fires ch 0x90 [D #693 p0; M Switch_Activate 0x1b02a8]");
    tick_until(w, 900, "the mission completes", |s| s.drones.mission.status() == 6);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(100), "mux #618 (out 0x64 ← 0x90) fires ch 100 [D #618]");
    assert_eq!(sim.drones.mission.to_end, 0x0730_0017, "the driving hand-off [M 0x182c60:111-112]");
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
    let panel = route.iter().position(|(n, _)| *n == "door[4]'s panel").expect("the route presses #617");
    w.walk_route(&route[..panel]);
    tower_door1_holds_without_its_panel(&mut w);
    w.walk_route(&route[panel..]);
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

    // ---- phase B: the teleported run (the live session's own setup: placed on trigger #255, no input) ----
    // KEPT WRITE: this phase IS the placed order — #255 touched with 0x24/0x27/0x28/0xb4 all
    // cold — and no real input reaches it: tower_door1 #392 (lock 0xb4, writer only the panel
    // #617 [D census]) is the south room's only way in, and phase A's sister
    // `tower_door1_holds_without_its_panel` pins that the leaf holds a walked Bond until the E.
    // The PCSX2 check that would retire it: a fresh-boot walked run of the same route, watching
    // switch_channels[243] (0x26fc90+243) against the hand-back's re-enable — if that delta
    // matches the placed run's 1591 [L], the placed live measurement needs no placed port twin.
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


/// level-16-every-objective: the WHOLE walked mission on real input, no god mode, with a player
/// who stays where the pack cinema hands him back — the SISTER of
/// `walk_the_chasers_defeat_completes_the_mission` (same approach, no run to the blast room):
///   * obj0 "Reach Drake's launch silo" completes on the walked route (the monitor folds it);
///   * the launch chain re-enables Bond INSIDE HurtPoly-ShuttleLaunch #309's polygon at the pit
///     floor [M Player_Enable 0x195498 teleports to the stream-8 key; D the pack cinema's single
///     t747 key (3.57,−22.76,−179.55)];
///   * ch 243 rises at the launch script's tick 1611 and the death sequence fails the mission at
///     pulse+~450 [D stream-6; M 0x19a6a8 RED_FADE_TICKS 448; L 454] — the open pit floor is
///     lethal; the shelter is the blast rooms (blast_room_shelter_16_live), which this player
///     never enters, so obj1 (the KikoDeathButton #693) is never reached.
#[test]
fn the_walked_mission_fails_at_the_launch_blast_when_bond_stays_on_the_pit_floor() {
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
    let t_fail = t_fail.expect("the walked mission fails at the launch blast for a Bond left on the open pit floor [D; U-051]");
    eprintln!("[16-walk] re-enable f{t_reenable}, ch243 f{t243} (+{}), fail f{t_fail} (+{} after ch243)", t243 - t_reenable, t_fail - t243);
    assert!((1540..=1700).contains(&(t243 - t_reenable)),
        "the pulse lands ~1611 after the re-enable on the walked run (got {}) [D stream-6; the model 1610, port 1603, live 1591]", t243 - t_reenable);
    assert!((440..=470).contains(&(t_fail - t243)),
        "the walked fail rides the death sequence at +~450 after the pulse [M 0x19a6a8 RED_FADE_TICKS 448; L 454], got {}", t_fail - t243);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[22][0], 4, "obj0 stays complete through the fail");
        assert_eq!(sim.drones.mission.states[22][1], 3, "obj1 never had its writer reached: the button is past the pulses [D #693; U-051]");
    }
}

/// level-16-stuck-overlook: the interactive run parked at the SW dead end of the west walkway
/// (−17.86, 6.81, −97.16, yaw 6.98, beside the control-room glass, ~tick 3700) and could not
/// continue. What the game expects there [D docs/levels/07000016.md §7]: the way on is BACK
/// north-east along the walkway — the SecurityStation trigger #597 (kind 0xEA TOUCH_ONCE, out
/// 0x24, mask 0x04) at (−15.20, 6.52, −94.56) [D]; its E fires ch 0x24 [M Trigger_Activate
/// 0x1b2318:38-49 gates the E dispatch on mask&4] and door[0]'s rising-edge lock arm opens it
/// [M Door_Update 0x1363c8, the lock arm ~0x13660c]. This walk pins the whole arc on real input
/// from the level start: up the east ramp, west along the ring, SOUTH to the reported dead-end
/// spot at the walkway's raised end, back NE to the station, the E, then down and through
/// door[0] — the route on from the overlook.
#[test]
fn the_overlook_dead_end_returns_to_the_station_and_door0_opens() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(200);
    // The proven ascent (the main route's legs): corridor, east ramp, upper ring west to the
    // station area.
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
    ];
    w.walk_route(route);
    // The extension the interactive run made: SOUTH past the station along the walkway's raised
    // end (deck y 5.7..6.8 [D §7]) to the reported spot — the SW dead end by the control-room
    // glass.
    w.walk_to(walker::Wp::new(-17.0, 6.5, -95.5).r(1.0), "the walkway south of the station");
    w.walk_to(walker::Wp::new(-17.8, 6.8, -97.1).r(0.8), "the overlook dead end");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] + 17.86).abs() < 1.2 && (p[2] + 97.16).abs() < 1.2,
            "Bond reaches the reported dead-end spot (−17.86, 6.81, −97.16), got {p:?}");
        assert!(sim.bond.grounded, "the walkway's raised end holds him");
    }
    // The way on is BACK north-east: the station trigger, its E, door[0]'s unlock.
    w.walk_to(walker::Wp::new(-16.3, 5.9, -93.5).r(1.2), "back from the dead end");
    w.walk_to(walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2), "the station E");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x24), "the SecurityStation E fires ch 0x24 [D #597 mask 0x04]");
        assert_eq!(sim.drones.doors.list[0].ch_lock, 0, "door[0]'s lock cleared by the rising edge [M 0x13660c arm]");
    }
    // Downstairs and through door[0] — the route on from the overlook.
    let back: &[(&str, walker::Wp)] = &[
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
    ];
    w.walk_route(back);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[2] < -116.0, "Bond is through door[0] — the route on from the overlook, got {p:?}");
    }
}

/// The pit floor's placed soldiers (the launch-centre guard skins — the first real-damage run
/// died to drones #6/#7/#8, skins 0x5000093/0x5000087/0x5000090, weapon 0x14, at 3–13 u) shoot
/// the chaser-fight stand: the allowance walks top through their fire (the walkreal header's
/// "damage is suppressed"), real health cannot. Explicit return fire, the 07000002 room idiom:
/// answer the nearest live soldier Bond can put a round into (`stand_and_fire`, the dry-tap
/// give-up, the 240-tick cooldown for a guard the stand cannot drop). The chaser herself is
/// excluded — her kill is the range-keeping loop's, and her punches landing during the clearing
/// are the punch pin's data.
fn clear_the_pit_shooters(w: &mut Walker, chaser: usize) {
    w.dmg.dry_taps = Some(5);
    let mut tried: Vec<(usize, u32)> = Vec::new();
    for round in 0..10 {
        let pick = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let fresh = |i: &usize| !tried.iter().any(|&(k, until)| k == *i && w.total_ticks < until);
            let mut near: Vec<(f32, usize)> = sim.drones.list.iter().enumerate()
                .filter(|(i, d)| *i != chaser && d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled && d.present
                    && walker::dxz(d.pos, bp) < 25.0)
                .map(|(i, d)| (walker::dxz(d.pos, bp), i)).collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0));
            near.into_iter().map(|e| e.1).filter(fresh)
                .find(|&i| walker::DamageLog::visible(sim, i))
        };
        let Some(di) = pick else { break };
        let _ = w.switch_to_armed("the pit room shooters");
        let killed = w.stand_and_fire(di, 900);
        if !killed { tried.retain(|&(k, _)| k != di); tried.push((di, w.total_ticks + 240)); }
        eprintln!("[real-damage] pit shooter #{di} down {killed} (round {round}) at health {:.1}",
            w.scene.sim.as_ref().unwrap().drones.health.health);
    }
    w.dmg.dry_taps = None;
}

/// The chaser's punch, pinned on the disc's own numbers. Her skin 0x0500001e makes
/// `NDrone2_DefaultInit`'s switch give kind 0xb, ANIM SET 7 and the fists (weapon 0) [M
/// 0x14b300's switch — the port's nf-formats `drone_setup`; the decompiler dropped the store,
/// the disasm carries it]. Anim set 7 sends `DroneWeap_CloseCombatImpact` (0x1789c0:26-40)
/// down the kick arm: `Bullet_init(-1, obj, obj, 99, obj+0x30, facing, 0xffff)` inside the
/// 2.0 band, no facing gate, and record 99's damage f32 is 50.0 [D +0x0C —
/// chaser_melee_records.rs pins the bytes]. The player side is `Player_HandlePain`
/// (0x1902c8) × the per-level/difficulty factor: 07000016's Plr_DMod row is 0.4/0.6/1.0
/// [M 0x1d4424 group], so the 50-raw punch lands **20.0 health a hit at Operative** and 30.0
/// at Agent. The chain the port mirrors: record → Bullet → `Player_DealWithObjHit` →
/// `pain_from` (armourFraction 1.0 — the chain save carries no armour, asserted, so the
/// health loss IS the punch).
#[test]
fn the_chasers_punch_lands_the_record_damage() {
    walker::with_real_damage_no_return_fire(|| {
        let iso = iso();
        let save = carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]);
        let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000016.bin", false, Some(&save), 1).unwrap());
        let (chaser, t0) = walk_to_the_chaser_fight(&mut w);
        let star_damage = {
            let sim = w.scene.sim.as_ref().unwrap();
            assert_eq!(sim.drones.progress.difficulty, 1, "Operative rode the load path [M 0x1d4424 row]");
            assert_eq!(sim.drones.health.armour, 0.0, "the chain save carries no armour — the health loss IS the punch");
            assert!(sim.drones.list[chaser].present && sim.drones.list[chaser].health > 0.0, "the chaser lives");
            // Her ANIM SET is 7 [M 0x14b300 skin 0x0500001e → (0xb, 7)] and the placement's
            // weapon (param 9) does not replace it — the 0x12/0x14/0x15 arm guards on
            // kind != 0xb [M the same switch]. The kick arm [M 0x1789c0:26-40] reads the set,
            // so her close-combat impact is record 99 regardless of the carried gun.
            assert_eq!(sim.drones.list[chaser].anim_set, 7, "the chaser's anim set stands at 7 — the kick arm [M 0x1789c0]");
            // Her ranged hits (the states' Fire(1) with the placement weapon [M 0x16c248 etc.])
            // ride that record [D +0x0C]; they land before she closes and are pinned too.
            let w_id = sim.drones.list[chaser].weapon as usize;
            let r = &sim.data().weapons.records[w_id];
            eprintln!("[punch] her placement weapon {w_id:#x}, record damage {:+.3} [D +0x0C]", f32::from_le_bytes(r[0x0c..0x10].try_into().unwrap()));
            f32::from_le_bytes(r[0x0c..0x10].try_into().unwrap())
        };
        // The pit room's soldiers shoot the stand (the first run's death) — explicit return fire
        // while she works her way in; her punches landing during the clearing are the data. Then
        // the idle stand for the punch (no fire — the pin is the DAMAGE the punch carries, not
        // the kill). Her chain: 0xab GetClose ≤ 2 → 0xac … [M 0x16cb70:40-45] — she closes and
        // swings; her approach stars (0xa9/0xaa's Fire(1), 1.5 [D record 0x12] × 0.4 = 0.6) land
        // on the way. The stand ends before pulse 2 (6810 [D stream 9]) — an idle Bond outside
        // the shelter eats its 1000.
        let mut chaser_hits: Vec<f32> = Vec::new();
        let mut clear_at = 0u32;
        while w.total_ticks - t0 < 6300 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                chaser_hits = sim.drones.player_hits.iter().filter(|&&(_, o, _)| o == chaser).map(|&(_, _, d)| d).collect();
            }
            if chaser_hits.iter().any(|&hit| (hit - 20.0).abs() < 0.01) { break; }
            if w.scene.sim.as_ref().unwrap().drones.health.health < 30.0 { break; } // her point-blank bursts — survival first
            if w.total_ticks >= clear_at { clear_the_pit_shooters(&mut w, chaser); clear_at = w.total_ticks + 900; continue; }
            w.tick(Input::default(), false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[chaser];
        eprintln!("[punch] chaser hits {chaser_hits:?} — Bond health {:.1}, her state {:#x} at dist {:.2}",
            sim.drones.health.health, d.state, walker::dxz(d.pos, sim.bond.pos));
        assert!(!chaser_hits.is_empty(), "the chaser closed and fired at the idle Bond at {:?}", sim.bond.pos);
        // Every landed hit follows the chain: the punch 50 [D record 99 +0x0C] × 0.4 [M 0x1d4424
        // Operative] = 20.0, or her star 1.5 [D record 0x12] × (2.0 TooClose [M 0x139fac-0x139fc8
        // — the shooter's tracked dist < the level's TooClose]) × 0.4 = 1.2 point-blank. Her
        // point-blank 0xaa bursts at 1.5 u (the sword 0xac never reached inside the window) are
        // the report's live check — the punch VALUE is pinned here whenever she swings.
        let punches: Vec<f32> = chaser_hits.iter().copied().filter(|&hit| (hit - 20.0).abs() < 0.01).collect();
        for &hit in &chaser_hits {
            assert!((hit - 20.0).abs() < 0.01 || (hit - star_damage * 2.0 * 0.4).abs() < 0.02 || (hit - star_damage * 0.4).abs() < 0.02,
                "every hit is the punch (50 [D 99] × 0.4 [M]) or her star (1.5 [D 0x12] × TooClose × 0.4 [M]), got {hit}");
        }
        if punches.is_empty() {
            eprintln!("[punch] no punch landed in the window — her point-blank 0xaa star bursts only (live check; the punch's 20.0 is the pinned chain 50 [D] × 0.4 [M])");
        } else {
            eprintln!("[punch] the punch landed: {punches:?} = 50 [D record 99] × 0.4 [M Operative]");
        }
        assert!(sim.drones.health.health > 0.0, "Bond survives the pinned punches ({} landed)", chaser_hits.len());
    });
}

/// The chaser's kill with the melee band held off — the walker's own tactics, no game code:
/// return fire with the carried Delta Repeater (`assert_armed_for_the_chaser` pins it in
/// hand [M 0x1a7ef8]) from outside 3.0, backpedal-with-fire inside it. Her punch fires only
/// inside the 2.0 band [M 0x1789c0:26-40] and the band router sends her at Bond ≤ 2 [M
/// 0x16bb50 / 0x16cb70:40-45]; under 3.0 u every drone round hits [M DoBulletAccuracy
/// 0x179740:113-115] — hers included — so the band is held, not traded. She must fall before
/// the second shelter window (the leaves shut 6310..6660 [D]; the walk back takes ~400).
fn kill_the_chaser_with_range_keeping(w: &mut Walker, chaser: usize, t0: u32) {
    assert!(w.switch_to_armed("the chaser's kill"), "a carried gun has rounds");
    let mut aim = w.exposed_aim(chaser);
    for t in 0..6000u32 {
        let (dead, dist, bp, dp) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[chaser];
            (d.corpse || d.health <= 0.0, walker::dxz(d.pos, sim.bond.pos), sim.bond.pos, d.pos)
        };
        if dead {
            eprintln!("[real-damage] the chaser down +{} (range keeping)", w.total_ticks - t0);
            return;
        }
        assert!(w.total_ticks - t0 < 5600,
            "the chaser must fall before the shelter window closes (leaves shut 6310 [D]; at +{})",
            w.total_ticks - t0);
        if t % 300 == 299 {
            let dry = { let g = &w.scene.sim.as_ref().unwrap().gun; g.clip + g.spare == 0 };
            if dry { assert!(w.switch_to_armed("the kill's dry re-arm"), "a carried gun still has rounds"); }
        }
        if t % 30 == 0 { aim = w.exposed_aim(chaser); }
        let (input, fire) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let at = aim.unwrap_or([dp[0], dp[1] + 0.3, dp[2]]);
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let err = walker::wrap_pi((at[0] - bp[0]).atan2(at[2] - bp[2]) - sim.bond.yaw);
            let want_pitch = ((at[1] - eye[1]) / dist.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0;
            let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
            let fire = if aim.is_none() { false } else { on_target && dist >= 2.0 };
            (input, fire)
        };
        // Out of sight and far: close for the view (she is coming anyway — the chain closes).
        if aim.is_none() {
            let forward = if dist > 5.0 { 1.0 } else { 0.0 };
            w.tick(Input { forward, ..input }, false);
        } else if dist < 3.0 {
            // Hold the band off: backpedal facing her, tap only with the view on her (the same
            // tap discipline shoot_point uses — the aim wobbles ±0.05 rad while converging).
            if fire { w.tick_fire(Input { forward: -1.0, ..input }); } else { w.tick(Input { forward: -1.0, ..input }, false); }
        } else if fire {
            w.tick_fire(input);
        } else {
            w.tick(input, false);
        }
    }
    panic!("the chaser never fell under the range-keeping fire");
}

/// THE FINALE ON REAL HEALTH (`damage_log.rs::with_real_damage`): the same walked chain —
/// the arrival → the overlook route's station E → the slab → the launch chain played out →
/// the west blast-room shelter over pulse 1 → the chaser → the shelter again over pulse 2 →
/// the pit squad → the lift-out cinema → the button → MissionState 6 — with the walker's
/// health top-up OFF for every walker it makes and Bond's death panicking with the death
/// spot and the last hits. The survival is the walker's own: return fire with the carried
/// Delta Repeater (`DamageLog::return_fire` answers the route's shooters mid-leg; the
/// carried set is pinned by `assert_armed_for_the_chaser` [M 0x1a7ef8]), range keeping
/// against the chaser's melee (the punch fires only inside the 2.0 band [M 0x1789c0:26-40];
/// the kill loop holds her outside 3.0 — under 3.0 u every drone round hits [M 0x179740:
/// 113-115]), and the blast-room shelters the sisters pin (`sit_out_pulse` asserts the hurt
/// volume never touches Bond). No game code changed: the sim is origin/master's.
#[test]
#[ignore = "the pit-squad phase still kills Bond at ~48 health (the 4-squad + pit shooter #8 at the room's door) — the route, both shelters, the chaser's range-keeping kill (+3364) and pulse 2 all survive real damage; the next leg is the squad-phase cover tactics (the report's 'what's left')"]
fn walk_the_chasers_defeat_completes_the_mission_on_real_damage() {
    walker::with_real_damage(|| {
        let iso = iso();
        let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
        let (chaser, t0) = walk_to_the_chaser_fight(&mut w);
        // The route's return-fire fights spend bolts — the arrival SET is the chain's [M
        // 0x1a7ef8], but the gun in hand may be dry; the fight continues on the carried P2K
        // (rounds pinned [D the chain save]). The allowance sister pins the arrival's held gun.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            // The chain's set carried through the real route; the route's fallen guards may ADD
            // a dropped gun (the real player walks over one too) — subset, not equality.
            let missing: Vec<usize> = CHAIN_OWNED.iter().copied().filter(|w| !owned(sim).contains(w)).collect();
            assert!(missing.is_empty(), "the 15 → 16 chain's weapons survive the route (missing {missing:?}, owns {:?})", owned(sim));
            assert!(sim.drones.inventory.clip[2] + sim.drones.inventory.carried[1] > 0,
                "the P2K has rounds (clip {} spare {})", sim.drones.inventory.clip[2], sim.drones.inventory.carried[1]);
        }
        assert!(w.switch_to_armed("the chaser fight"), "a carried gun has rounds after the route fights");
        // The pit room's soldiers first (their steady fire is the fight's real dps at real
        // health; the first run died to them at the shelter), then her, with the band held off.
        clear_the_pit_shooters(&mut w, chaser);
        kill_the_chaser_with_range_keeping(&mut w, chaser, t0);
        finish_after_the_chaser(&mut w, t0);
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.health.health > 0.0, "Bond survives the finale on the port's real damage");
        eprintln!("[real-damage] the finale walked: {}", w.dmg.summary(sim));
    });
}
