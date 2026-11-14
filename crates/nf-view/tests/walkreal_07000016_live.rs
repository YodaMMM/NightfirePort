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

/// The guard drops Bond's own feet took on the way: weapon pickups with a dropping drone
/// (`Pickup::from`) that went from live to gone this tick within reach of Bond — the game's
/// walk-over take [M Pickup_Update's take, 0x18e650]. capsule-join: with the game's standing
/// bodies the route's fights shift and a guard's 0x1a lands on the chaser route; walking over a
/// dropped gun takes it in the game too, so the load-rule pin below adds exactly the drops this
/// tracker saw taken — nothing else may appear or vanish.
fn track_drop_takes(w: &mut Walker) -> std::rc::Rc<std::cell::RefCell<Vec<usize>>> {
    let taken = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let t2 = taken.clone();
    let mut was_live: Vec<bool> = Vec::new();
    w.on_step = Some(Box::new(move |sc: &nf_view::scene::Scene, _t: u32| {
        let sim = sc.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        let ps = &sim.drones.pickups;
        if was_live.len() < ps.len() { was_live.resize(ps.len(), false); }
        for (i, p) in ps.iter().enumerate() {
            let live = !p.gone;
            if was_live[i] && !live && p.from.is_some() && p.kind == nf_game::pickup::kind::WEAPON {
                let at = p.m[3];
                let d = ((at[0] - b[0]).powi(2) + (at[2] - b[2]).powi(2)).sqrt();
                if d < 2.5 {
                    eprintln!("[016] the walk took a guard's dropped weapon {:#x} at ({:.2},{:.2},{:.2})", p.weapon, at[0], at[1], at[2]);
                    let mut t = t2.borrow_mut();
                    if !t.contains(&(p.weapon as usize)) { t.push(p.weapon as usize); }
                }
            }
            was_live[i] = live;
        }
    }));
    taken
}

/// `want` plus the tracked drop takes, sorted.
fn with_takes(want: &[usize], taken: &std::rc::Rc<std::cell::RefCell<Vec<usize>>>) -> Vec<usize> {
    let mut v: Vec<usize> = want.to_vec();
    for &t in taken.borrow().iter() { if !v.contains(&t) { v.push(t); } }
    v.sort();
    v
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
    // The allowance holds until the pulse rises (the guards' fire before it is not the volume's
    // contact), then is OFF for every tick of the pulse. Through both, a player in the room
    // answers a guard shooting him (a guard who followed him in stood 1.7 u away and shot him
    // dead 448 ticks into pulse 2) — he aims and fires; nothing moves his feet.
    let mut last_tap = 0u32;
    let mut waited = 0u32;
    while !w.scene.sim.as_ref().unwrap().drones.channels.on(243) {
        shelter_tick(w, &mut last_tap);
        waited += 1;
        assert!(waited < 8000, "{label}: not within 8000 ticks");
    }
    w.top_up_health = false;
    let mut n = 0u32;
    while w.scene.sim.as_ref().unwrap().drones.channels.on(243) {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let h = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("HurtPoly-ShuttleLaunch");
            assert!(!h.contains(p, [p[0], p[1] + 1.8, p[2]], 0.55), "{label}: the volume touches Bond at {p:?} (+{n})");
            assert!(sim.drones.health.health > 0.0, "{label}: Bond died at {p:?} (+{n})");
        }
        shelter_tick(w, &mut last_tap);
        n += 1;
        assert!(n < 1000, "{label}: the pulse never fell");
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.health.health > 0.0 && sim.drones.mission.status() != 3, "{label}: Bond lives, no fail");
    w.top_up_health = allowance;
    n
}

/// The real-damage walk's pulse-1 sit (drone-block-swaps-g055-3, opt-in): the same pulse
/// skeleton with the PROACTIVE shelter tick — the pit soldiers are answered on sight through
/// the door (see shelter_tick_proactive), so they fall with loaded drops instead of
/// emptying their clips into the run. Nothing moves Bond's feet; the same volume/live
/// asserts stand.
fn sit_out_pulse_proactive(w: &mut Walker, label: &str) -> u32 {
    let allowance = w.top_up_health;
    let mut last_tap = 0u32;
    let mut waited = 0u32;
    while !w.scene.sim.as_ref().unwrap().drones.channels.on(243) {
        shelter_tick_proactive(w, &mut last_tap);
        waited += 1;
        assert!(waited < 8000, "{label}: not within 8000 ticks");
    }
    w.top_up_health = false;
    let mut n = 0u32;
    while w.scene.sim.as_ref().unwrap().drones.channels.on(243) {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let h = sim.hurts.iter().find(|h| h.gate == 0xf3).expect("HurtPoly-ShuttleLaunch");
            assert!(!h.contains(p, [p[0], p[1] + 1.8, p[2]], 0.55), "{label}: the volume touches Bond at {p:?} (+{n})");
            assert!(sim.drones.health.health > 0.0, "{label}: Bond died at {p:?} (+{n})");
        }
        shelter_tick_proactive(w, &mut last_tap);
        n += 1;
        assert!(n < 1000, "{label}: the pulse never fell");
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.health.health > 0.0 && sim.drones.mission.status() != 3, "{label}: Bond lives, no fail");
    w.top_up_health = allowance;
    n
}

/// One tick in the shelter: with a visible guard shooting Bond (`DamageLog::shooters`), turn the
/// view onto his exposed part and fire (a tap every 8 ticks once on target, a reload on an empty
/// clip); otherwise an idle tick. Turn/look only — Bond's feet never move, so the pulse's
/// per-tick volume check reads the same standing spot.
fn shelter_tick(w: &mut Walker, last_tap: &mut u32) {
    let pick = {
        let sim = w.scene.sim.as_ref().unwrap();
        let armed = sim.gun.clip + sim.gun.spare > 0 && (2..=0x47).contains(&sim.gun.weapon);
        if !armed || sim.disabled { None } else {
            w.dmg.shooters(sim).into_iter()
                .find_map(|i| walker::DamageLog::exposed_part(sim, i).map(|at| (i, at)))
        }
    };
    shelter_tick_picked(w, last_tap, pick);
}

/// The proactive shelter tick (drone-block-swaps-g055-3, opt-in — the real-damage walk only):
/// the pick drops the shooters-only filter. The flush [M 0x148010:100-131] sends the pit
/// soldiers hunting from their first attack, so they pour fire the whole pit phase and their
/// AR drops land DRY whatever Bond does later (the measured dry-stand death at 14401: the
/// squad phase started with clip 1 spare 1 and the cycling found no round in any owned gun).
/// A player in the shelter shoots everyone he can see through the door, reactive or not —
/// the soldiers die inside the pulse window with their clips mostly intact, and their drops
/// (each with its own clip [M DroneWeap_DropWeapon 0x17a2a0]) arm the finale squad phase.
fn shelter_tick_proactive(w: &mut Walker, last_tap: &mut u32) {
    let pick = {
        let sim = w.scene.sim.as_ref().unwrap();
        let armed = sim.gun.clip + sim.gun.spare > 0 && (2..=0x47).contains(&sim.gun.weapon);
        if !armed || sim.disabled { None } else {
            sim.drones.list.iter().enumerate()
                .filter(|&(_, d)| d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0
                    && walker::dxz(d.pos, sim.bond.pos) < 30.0)
                .find_map(|(i, _)| walker::DamageLog::exposed_part(sim, i).map(|at| (i, at)))
        }
    };
    shelter_tick_picked(w, last_tap, pick);
}

fn shelter_tick_picked(w: &mut Walker, last_tap: &mut u32, pick: Option<(usize, [f32; 3])>) {
    let Some((_, at)) = pick else { w.tick(Input::default(), false); return };
    let (input, fire, reload) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
        let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
        let h = (dx * dx + dz * dz).sqrt().max(0.5);
        let err = walker::wrap_pi(dx.atan2(dz) - sim.bond.yaw);
        let perr = (dy / h).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
        let reload = sim.gun.clip == 0;
        let fire = !reload && err.abs() < 0.03 && perr.abs() < 0.04 && w.total_ticks.saturating_sub(*last_tap) >= 8;
        (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, fire, reload)
    };
    if reload {
        w.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false);
        w.total_ticks += 1;
        w.after_step();
    } else if fire {
        *last_tap = w.total_ticks;
        w.tick_fire(input);
    } else {
        w.tick(input, false);
    }
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
/// consider-explosive-gate-3 2026-11-02 UN-IGNORED with the two walk-side laws the faithful
/// throw-roll cadence [M 0x17a798 disasm — the 50% Rand(2) precedes the dist check, so the frag
/// holders roll every aim-stand 0x68 tick] exposed (both measured on this branch):
/// 1. the re-arm — the chain carry reaches the chaser nearly dry (the measured run: the Delta
///    0x11 at clip 3 spare 9 at her spawn), so the walk now re-arms from the corridor guards'
///    AR drops after her kill (the walk-over take [M 0x18e650] + the auto-equip [M 0x1a6eb0];
///    the task's own "ammo pickups, weapon choice" tactic — no ammo writes);
/// 2. the unskipped lift-out — AND #286 (0xbf·0xc0 → 0x82 [D]) fires the moment both inputs
///    latch, and the retime can land ch 0xbf AFTER ch 0xc0 @7808 [D stream 20] (measured:
///    +8293), so 0x82 went on while the walk's cut-scene skip was still up and the fast-forward
///    played the whole 0x06000632 cinema inside one tick — the disable window was never
///    observed and the old 3000-tick wait starved. The skip now comes off before the squad
///    phase and the mission's own transition always plays out.
/// Live check: in the real game the pit squad (all four ch 0x17/0x18/0x19/
/// 0x1a deaths) falls to a walked Bond with the Delta Repeater's own ammo, or the mission's
/// flow expects the guards' frag rolls (and their self-flee) here.
#[test]
fn walk_the_chasers_defeat_completes_the_mission() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    let (chaser, t0) = walk_to_the_chaser_fight(&mut w, false, false);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &CHAIN_OWNED, "the 15 → 16 chain");
    // Dry mid-fight (the 15 carry's Delta Repeater holds 12 rounds here): the player switches to
    // another carried gun with ammo (or, with none, picks up a fallen guard's), then fights on.
    let killed = w.shoot(chaser, "the chaser")
        || ((w.switch_to_armed("the chaser ammo") || w.resupply("the chaser ammo")) && w.shoot(chaser, "the chaser"));
    assert!(killed, "the chaser dies to Bond's fire");
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
    let taken = track_drop_takes(&mut w);
    walk_to_the_chaser_fight(&mut w, false, false);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &with_takes(&CHAIN_OWNED, &taken), "the 15 → 16 chain");
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
    let taken = track_drop_takes(&mut w);
    walk_to_the_chaser_fight(&mut w, false, false);
    assert_armed_for_the_chaser(w.scene.sim.as_ref().unwrap(), &with_takes(&COLD_OWNED, &taken), "the cold 16 load");
    let diff: Vec<usize> = COLD_OWNED.iter().copied().filter(|w| !CHAIN_OWNED.contains(w)).collect();
    assert_eq!(diff, vec![0x50], "the two paths differ only by the Grapple, as in the game [M 0x1a97c8]");
}

/// From the arrival to the chaser fight: the approach, the pack cinema (played out), pulse 1 in
/// the shelter, the chaser's spawn and the door reopening. Returns the chaser's index and the
/// hand-back tick. `re_arm` (consider-explosive-gate-3, opt-in — only the real-damage finale walk
/// sets it): after the route's fights, collect the guards' drops lying on the walked ground — the
/// faithful throw-roll cadence [M 0x17a798] retimes every fight and the measured run reached the
/// 100-health squad clone #12 with five P2K rounds because every gun was dry (the walk-over take
/// [M Pickup_Handler 0x18e650] auto-took ONE AR on the way; the player collects the rest while
/// they are still a few steps behind, not from the pit floor 70 u away, past the walker's own
/// 45 u resupply reach). The other callers keep their measured timelines.
/// The 07000014 clear_hostiles idiom, test-local for this file: kill every live team-1 guard
/// inside `radius` (the flush hunts from the first attack [M 0x148010:100-131], so anyone left
/// in the lane chews the next leg), cycling the dropped guns in between (the clear's resupply
/// — the swarm outshoots one clip), then walk the NAV web back to the spot the caller left
/// Bond at (the fights displace him; the direct steer hits the door-wall jambs).
fn clear_hostiles_16(w: &mut Walker, radius: f32, cap: u32) {
    let home = w.scene.sim.as_ref().unwrap().bond.pos;
    for _ in 0..10 {
        let sim = w.scene.sim.as_ref().unwrap();
        // The LEVEL gate (drone-block-swaps-g055-4): the route's ambushers are the ring
        // guards — placements 186-191 all spawn at y 6.71 on the upper ring (hp 10-25 [L
        // probe_16_guards]) while the approach legs run at floor 0 — and a Bond on the
        // ground cannot fight them: the measured run burned 10800 ticks on the unexposed
        // drone #0 (dist 14-16, `exposed false`), the flank nav dragged him up the ramp wall
        // (head_bump, target (15.13, 6.71, -74.68)), and the below-fights fed the crossfire
        // that emptied his 100 (the 10684 death at the ramp foot). A player climbs first and
        // fights the ring AT LEVEL — the ring legs' own clears (Bond at y ~6.7) do that
        // cheaply (hp 10 each); the ground guard (#187, y 0.80) still fights from the floor.
        let bp = sim.bond.pos;
        let Some(di) = sim.drones.list.iter().position(|d| {
            d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                && walker::dxz(d.pos, bp) < radius
                && d.pos[1] <= bp[1] + 2.0 && d.pos[1] >= bp[1] - 2.0
        }) else { return };
        // The far+unexposed stand is not a fight (measured both ways on this tree): run A
        // burned 10800 futile ticks on the unexposed drone #0 (dist 14-16, `exposed false`)
        // and run B — with the level gate alone — stood at the launch centre shooting a wall
        // at dist 39 while the above ring guards poured free (the 2609 death, 71 hits, 0
        // return-fire fights). A player keeps walking the legs when nobody is exposed to
        // answer; the geometry opens as he advances, and the ring legs fight the ring guards
        // AT LEVEL (hp 10 each [L probe_16_guards]). Close and unexposed keeps the flank
        // dance (shoot_bounded's own repositioning — the run-A fights that landed).
        // guard-radii-16-chasers MEASURED: a far-exposed arm (fight exposed level guards out
        // to 35 u) took the gallery pair 1v1 from the ring but the ring guards' close fire
        // poured free meanwhile — the 2152 death, 71 hits for 100, the d2 lever (b)'s ring
        // trades again. REVERTED: the pair's bleed is answered by the #596 armour detour
        // below instead (the plinks absorb [M Player_HandlePain's type-0 arm]).
        if walker::dxz(sim.drones.list[di].pos, bp) > 15.0 { return; }
        if walker::dxz(sim.drones.list[di].pos, bp) > 8.0 && w.exposed_aim(di).is_none() { return; }
        // A nearly-dry gun starts a fight it cannot finish (the measured ring-2 fight opened
        // on clip 1 spare 3 and cost 28 health in the flank dance): restock from the
        // trail-corridor boxes first — the station's 0x47 box (100 rounds [L probe_16_pickups])
        // sits 3 u off the station legs.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[clear] at {:?} holds {:#04x} clip {} spare {} owns {:x?}",
                sim.bond.pos, sim.gun.weapon, sim.gun.clip, sim.gun.spare,
                (0..sim.drones.inventory.owned.len()).filter(|&w| sim.drones.inventory.owned[w]).collect::<Vec<_>>());
            if sim.gun.clip + sim.gun.spare < 8 { w.resupply("the hostile clear low"); }
        }
        // The ring fights run 7-20 u: a player takes them with the P2K, not the Delta Repeater
        // the chain hands over. The Delta's bolts fly a gravity arc (measured on the stream
        // join: a bolt aimed level at a guard 19.7 u off fell 0.75 u under his feet, and twelve
        // of them — the whole carry — missed while the gallery pair poured 60 health in).
        switch_off_the_delta(w);
        w.shoot_bounded(di, "the hostile clear", cap);
        // The kill's own loot, immediately (drone-block-swaps-g055-4): the guard's dropped
        // gun lies at the fight floor — walk over it NOW (the walk-over take [M
        // Pickup_Handler 0x18e650] files it + its clip), not after ten fights. The measured
        // stands went dry at +468 and kept standing while the shared resupply's walked-path
        // corridor filter refused the 10-25 u drops ("no pickup within the walked-path
        // corridor") — the cycle then crawled through every dry gun to the fists.
        if w.scene.sim.as_ref().unwrap().drones.list[di].corpse {
            // Loot by the DROP's live position, not the corpse spot: the gun bounces/slides
            // off where the guard fell (the measured loot walk arrived at the corpse and the
            // take never fired — the next fight opened on clip 1 spare 3). Close drops only:
            // the loot walk itself is exposure; a farther one stays for the walker's own
            // trail-corridor resupply on the route legs. Arrive 0.6 — the walk-over take's
            // radius is under a unit [M Pickup_Handler 0x18e650].
            let drop = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.pickups.iter().filter(|p| !p.gone)
                    .filter(|p| (p.m[3][1] - bp[1]).abs() < 1.5)
                    .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                    .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)))
                    .filter(|t| walker::dxz(*t, bp) < 8.0)
            };
            if let Some(t) = drop {
                w.walk_nav_to(t, "the clear's kill loot", 0.6);
                switch_to_a_deep_firearm(w, "the clear's kill loot");
            }
        }
    }
    // The drops feed the run (the dry-finale lesson — the measured 14401 death opened the
    // squad phase on clip 1 spare 1): walk the corpses Bond just made. The walk-over take
    // [M Pickup_Handler 0x18e650] files their guns AND their remaining clips into the carry;
    // the walker's own resupply refuses off-corridor detours, so the corpses ARE the walk.
    let spots: Vec<[f32; 3]> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        sim.drones.list.iter()
            .filter(|d| d.corpse && d.team == 1 && walker::dxz(d.pos, bp) < 45.0)
            .map(|d| [d.pos[0], d.pos[1], d.pos[2]]).collect()
    };
    for (k, s) in spots.iter().enumerate() {
        w.walk_nav_to(*s, &format!("the clear's corpse walk {k}"), 1.0);
    }
    w.walk_nav_to(home, "back to the clear's home spot", 1.0);
}


/// Swap the held Delta Repeater 0x11 for the P2K 0x02 when the P2K has rounds (see the clear).
fn switch_off_the_delta(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        let p2k_rounds = inv.clip.get(2).copied().unwrap_or(0) as i32 + inv.carried.get(1).copied().unwrap_or(0) as i32;
        if sim.gun.weapon != 0x11 || !inv.owned.get(2).copied().unwrap_or(false) || p2k_rounds <= 0 { return; }
    }
    for _ in 0..12 {
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); } // the holster/draw cycle
        if w.scene.sim.as_ref().unwrap().gun.weapon == 2 { return; }
    }
}

/// Cycle to a real firearm with a deep mag (drone-block-swaps-g055-4): the plain
/// switch_to_armed's cycle lands on whatever has rounds in the game's weapon order — the
/// measured loot walk left Bond holding the frag 0x34 (clip 1 spare 2) and the squad phase
/// threw three grenades at hp-20 clones before dying dry. The dropped 0x1a SMGs (50-round
/// mags [L probe_16_pickups]) are the pit floor's own guns; a player swaps to the deepest
/// magazine he just picked up, not to the grenade bandolier.
fn switch_to_a_deep_firearm(w: &mut Walker, label: &str) {
    for _ in 0..12 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            if (2..=0x47).contains(&g.weapon) && g.weapon != 0x34 && g.weapon != 0x37
                && g.clip + g.spare >= 10 { return; }
        }
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); } // the holster/draw cycle
    }
    w.switch_to_armed(label); // anything armed beats the fists
}

fn walk_to_the_chaser_fight(w: &mut Walker, re_arm: bool, station_armour: bool) -> (usize, u32) {
    w.wait_enabled(3000, "the launch-centre arrival");
    w.wait(100);
    if re_arm && !w.tops_up() {
        // The route leg-wise with the hostile clears between (drone-block-swaps-g055-3, the
        // 07000014 clear_hostiles idiom, test-local): under the flush the ambush guards hunt
        // from their first attack and the reactive return fire bled ~80 of Bond's 100 across
        // 8 fights before the pit (the measured 20.2 arrival and the 14401 dry death). A
        // player clears the lane BEFORE walking into the next leg — each kill is hp-20 cheap
        // and the dropped guns cycle into the carry (the clear's own resupply), which is also
        // the finale squad's ammo. walk_route's legs ARE walk_to calls, so the structure is
        // the route's own; the clear caps at 1200 per shooter and walks the NAV web home so
        // the next leg starts where it would have.
        //
        // drone-block-swaps-g055-4: the proactive stands are the RING legs' tool only. The
        // measured ground-leg stands on this tree bled 72 health by the lane-east fight (the
        // ring guards #0/#2-#5 overlook the ground from y 6.71 [L probe_16_guards] and pour
        // free while Bond fights what he cannot see) and the fights' displacement pushed him
        // off the lane into the trench (the y -4.65 deaths). The ground legs keep MOVING —
        // the walker's reactive return fire answers what actually shoots (the same economy
        // that held 07000014's fights at 74.0 on this tree) — while the ring legs (y 5.9,
        // the guards' own floor) clear at level, hp 10 each. The gate is 5.5 (the ring
        // floor), not 4.0: the ramp legs top at y 4.6 and the measured run took the gate
        // there — Bond still at y 2.9 under the whole ring — and the stand burned ~1400
        // ticks on a hunting guard 17-27 u out (12 rounds, zero damage) while the ring
        // guards emptied him (the 1988 death).
        for (label, wp) in APPROACH {
            if wp.at[1] > 5.5 { clear_hostiles_16(w, 40.0, 900); }
            // The Agent walk (d2): the ramp foot and the whole ramp see all
            // three gallery riflemen (#3/#4/#5, 0x1a) [the 11-13 cover map]; the pocket at
            // x 19-20 / z -92..-98 sees none. Dash there (no return fire, straight legs - the nav's
            // hop toward (20.7, -95.6) wedged 771 ticks), arm, and peek-fight from the pocket edge.
            if re_arm && *label == "the lane east"
                // 00 Agent too (11-13): on the d3 roster the same pocket holds - the live
                // bullet-line scan has the gallery lines dead south of the ramp foot - and the
                // dash moves the death t1020 -> t4379 (pocket entered at 72.0).
                && w.scene.sim.as_ref().unwrap().drones.difficulty >= 2 {
                let rf = w.dmg.return_fire;
                w.dmg.return_fire = false;
                w.lenient = true;
                for (i, at) in [[10.0f32, 0.0, -84.0], [11.0, 0.0, -90.0], [16.5, 0.0, -91.6], [20.5, 0.0, -92.4], [20.6, 0.0, -95.0]].iter().enumerate() {
                    w.walk_to(walker::Wp::new(at[0], at[1], at[2]).r(0.9), &format!("the pocket dash {i}"));
                }
                w.lenient = false;
                w.dmg.return_fire = rf;
                let _ = w.switch_to_armed("the pocket");
                eprintln!("[016] in the pocket: health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
                let pocket = [20.6f32, 0.9, -95.0];
                // d3: (19.3, -93.6) - the old peek (19.0, -92.2) wedged its 3000-tick cap 2.1 u short while the
                // pocket bled 72 -> 4.7; from here the cycles clear the gallery at 61.5 [L the 11-13 sweep:
                // (19.6, -93.0) t2841, (20.2, -93.2) t2723, (19.3, -93.6) 61.5 at the fight's end].
                let peek = if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 { [19.3f32, 0.9, -93.6] } else { [19.0f32, 0.9, -92.2] };
                for k in 0..10 {
                    let alive = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        (0..sim.drones.list.len()).filter(|&i| {
                            let d = &sim.drones.list[i];
                            d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && d.pos[1] > 5.5 && d.pos[0] < 0.0 && d.pos[2] < -84.0
                        }).collect::<Vec<_>>()
                    };
                    if alive.is_empty() { break; }
                    w.lenient = true;
                    w.walk_to(walker::Wp::new(peek[0], peek[1], peek[2]).r(0.5), &format!("the gallery peek {k}"));
                    w.lenient = false;
                    let tgt = { let sim = w.scene.sim.as_ref().unwrap(); alive.iter().copied().find(|&i| walker::DamageLog::visible(sim, i)) };
                    if let Some(gi) = tgt { let _ = w.shoot_bounded(gi, &format!("the gallery peek {k}"), 300); }
                    w.lenient = true;
                    w.walk_to(walker::Wp::new(pocket[0], pocket[1], pocket[2]).r(0.6), &format!("back to the pocket {k}"));
                    w.lenient = false;
                    for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
                }
                eprintln!("[016] the gallery pocket fight ends, health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
                // d3: the gallery loop ends with two ground guards INSIDE the pocket (#0 hp 25 at 1.9 u,
                // #5 hp 20 at 4.3 u, both unseen by the visible pick) - backing out past them died at
                // t3972. Fight them here before the back-out, the 0x50000b2 man first (#5 hit for 14 a
                // round; nearest-first left 26.5 after the pair, #5-first keeps all 61.5 and reaches the
                // hand-back at 37.0 with the AR loaded - death t12054 -> t12518 in the chaser fight).
                if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 {
                    for _ in 0..4 {
                        let near = {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let bp = sim.bond.pos;
                            (0..sim.drones.list.len()).filter(|&i| {
                                let d = &sim.drones.list[i];
                                d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                                    && d.pos[1] < bp[1] + 2.0 && walker::dxz(d.pos, bp) < 6.0
                            }).min_by(|&a, &b| {
                                let key = |i: usize| { let d = &sim.drones.list[i]; let pri = d.skin == 0x50000b2; (if pri { 0.0 } else { 100.0 }) + walker::dxz(d.pos, bp) };
                                key(a).total_cmp(&key(b))
                            })
                        };
                        let Some(di) = near else { break };
                        let _ = w.shoot_bounded(di, "the pocket's close pair", 600);
                    }
                    eprintln!("[016] the pocket close pair ends, health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
                }
                // Back out the way the dash came (the straight steer from the pocket to the lane
                // pins on the trench's east edge).
                w.lenient = true;
                for (i, at) in [[20.5f32, 0.0, -92.4], [16.5, 0.0, -91.6], [11.0, 0.0, -90.0], [10.0, 0.0, -84.0]].iter().enumerate() {
                    w.walk_to(walker::Wp::new(at[0], at[1], at[2]).r(0.9), &format!("out of the pocket {i}"));
                }
                w.lenient = false;
            }
            // los-mask-repins-16 (the real-damage walks' east-lane re-route): under the game's
            // guard sight the east guards (#1 skin 0x50000b1 from (24.1, 0.8, −96.6), #3 skin
            // 0x500008c from (22.1, 0.8, −104.2) [L the 10864 death ledger]) hunt Bond on the
            // lane, and the fights' own flank answer (the shoot loop's 540-tick flank) navigated
            // Bond 9 u east into the east strip pocket — the determined pass killed #1 there, and
            // the resumed straight steer could not cross back west over the trench's east edge:
            // Bond pinned at (19.03, 0.92, −103.36) for ~5000 ticks (the measured bounce −100.2 ↔
            // −103.36), dead at 10864 under the ramp-foot rifleman #16's (weapon 0x1a, hp 10,
            // (14.5, 0.8, −92.4)) unseen fire — 28.6 in 20 hits, the run's biggest dealer. Two
            // player answers, both keyed to the re-arm (real-damage) walks only — the tops-up
            // walks keep the plain route:
            // 1. the ramp-foot rifleman stands hp 10 dead ahead of the lane head — a player kills
            //    him before walking the lane (a bounded stand from where Bond stands);
            // 2. the lane leg rides the NAV web (the flank nav demonstrably crossed it: the 4
            //    nodes (17.15, −92.35) → (21.04, −94.43) [L the same trace]) so the leg RE-ROUTES
            //    from wherever the corridor fights leave Bond instead of straight-steering into
            //    the pocket pin; the ring legs already ride the web for the same reason.
            if re_arm && *label == "the east lane south" {
                let gi = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    sim.drones.list.iter().position(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                        && d.pos[1] < 2.0 && d.pos[0] > 12.0 && d.pos[2] > -97.0 && d.pos[2] < -88.0)
                };
                if let Some(gi) = gi {
                    if walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), gi) {
                        let _ = w.shoot_bounded(gi, "the ramp-foot rifleman from the lane head", 1800);
                    } else {
                        eprintln!("[016] the ramp-foot rifleman #{gi} is not exposed from the lane head");
                    }
                }
                w.walk_nav_to([9.5, 0.0, -100.0], label, 1.6);
                continue;
            }
            // capsule-join (the punch walk only: re-arm, no station armour): the south gallery's
            // AR guard #2 (weapon 0x1a, hp 10) plinks the ring's north side from (9.5, 6.7,
            // −110.6), 32 u off, at Bond's own level — 22 hits for 30.8 of the route's 70, and
            // Bond reached the punch window at 24 (under its 30 survival line) [L capsule-join
            // trace: t893-1891 on the outbound ring, t3699-4340 on the return]. Once the ring legs
            // have cleared the close ring guards, a player answers the gallery rifleman across
            // the hall from the ring before walking it — a bounded stand, only while he is
            // exposed from where Bond stands.
            if re_arm && !station_armour && *label == "the ring 1" {
                let gi = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    sim.drones.list.iter().position(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                        && d.pos[2] < -100.0 && d.pos[1] > 5.5 && d.weapon == 0x1a)
                };
                if let Some(gi) = gi {
                    if walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), gi) {
                        let _ = w.shoot_bounded(gi, "the gallery rifleman from the ring", 1800);
                    } else {
                        eprintln!("[016] the gallery rifleman #{gi} is not exposed from the ring east");
                    }
                }
            }
            // guard-radii-16-chasers: the security-station armour #596 (−19.03, 6.67, −94.48)
            // [D the census/audit placement tables] — 3.8 u west of the station E's console, a
            // player straps the BodyArmourHeavy on while the door[0] press runs (the walk-over
            // take [M Pickup_Handler 0x18e650]). The south gallery's AR pair (#2/#3, y 6.7
            // above the trench [L the ledgers: 53.2-76.2 of the route's bleed, both alive at
            // hp 10]) has no cheap fight — the far-exposed ring stands measured WORSE (the
            // 2152 death, the d2 lever (b)'s trades) and the trench stands never saw them —
            // so the plinks ABSORB into this armour instead [M Player_HandlePain's type-0
            // arm; the punch pin's own precondition is that the route SPENDS it again — the
            // window's armour-0 assert stays].
            // armour detour (guard-radii-16-chasers, `station_armour`): the defeat walk straps
            // the #596 BodyArmourHeavy on and the gallery plinks absorb; the PUNCH pin cannot
            // carry armour INTO the window (the punch's health-delta pin — see the test) and
            // the route after the take deals too little to spend it, so the punch test walks
            // past (station_armour false).
            if re_arm && !w.tops_up() && station_armour && *label == "the station E" {
                w.walk_to(walker::Wp::new(-19.0, 6.0, -94.5).r(0.5), "the security-station armour #596 [D]");
            }
            // The ring/station legs ride the NAV web: the direct steer from the ramp top
            // walked off the ring's edge and fell to the ground below (the measured runs:
            // "the ring east: arrived at (13.39, 0.91, -79.41)" from the ramp top y 4.74) —
            // the whole ring phase then runs at ground level under the ring guards' fire
            // (the y 5.9 leg targets arrive on xz alone). The nav web holds the ring nodes
            // (the flank nav reached (15.13, 6.71, -74.68) [L the run-A STUCK dump]) and
            // climbs the ramp properly; at ring level the guards are at BOND's level — the
            // band-gated fights land (the measured ring-gate kill: drone 0 at 6.8 u, ~8
            // health) and the pour dies with them.
            // The station legs walk directly: the station platform is flat ring level (the
            // measured nav runs arrived y 6.83-6.84 with no fall) and their walk_to carries
            // the station E's console Use (the chain's gate) — the nav route to it wandered
            // north and gave up at the 3000-tick cap without the press. Only the ramp->ring
            // transition legs ride the nav (the direct steer fell off the ring edge there).
            // capsule-join: the shifted route fights (the game's standing bodies) had Bond reach
            // door[1] while it was still opening (state 0x10), and the straight leg from
            // door[0] carried him 0.4 u west of the lane onto the jamb/leaf: (1.58, 0.93,
            // −123.60), the line blocked at 0.067, for the whole 6000-tick cap [L capsule-join
            // red]. A player squares up on the doorway's centre line before stepping through,
            // the way the route already does at door[0]'s threshold.
            if *label == "door[1] through" {
                w.walk_to(walker::Wp::new(2.0, -0.1, -122.6).r(0.5), "door[1]'s threshold");
            }
            // capsule-join (the punch walk): with the gallery rifleman answered, the route's
            // fire no longer spends the security-station armour #596 (−19.03, 6.67, −94.48),
            // which the station E leg's 1.2 arrival took in passing at (−17.46, 6.82, −93.97)
            // (armour 0 → 50 at t3070 [L capsule-join trace]); the punch pin needs armour 0 at
            // the window. A player who wants his health on the line steps up to the console
            // from its east side and leaves the straps on the rack: the console E at a tight
            // 0.5 arrival on the console's own spot.
            if re_arm && !station_armour && *label == "the station E" {
                w.walk_to(walker::Wp::at(-15.0, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(0.5), "the station E from the east");
                continue;
            }
            if wp.at[1] > 5.0 && (label.starts_with("the ring") || label.starts_with("the return")) {
                w.walk_nav_to([wp.at[0], wp.at[1], wp.at[2]], label, wp.arrive);
            } else {
                w.walk_to(*wp, label);
            }
        }
    } else {
        w.walk_route(APPROACH);
    }
    if re_arm { w.resupply("the route fights' drops (the corridor guards' ARs)"); }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x8d), "the route crossed the corridor touch #616 — the button's gate is armed [D]");
    // The launch chain plays out (no skip): ch 0x81 → the pack cinema → the hand-back on the pit
    // floor [D stream-8 key].
    w.skip_cutscenes = false;
    tick_until(w, 3000, "the pack cinema's hand-back on the pit floor", |s| s.drones.channels.on(0x81) && !s.disabled && s.bond.pos[1] < -20.0);
    let t0 = w.total_ticks;
    eprintln!("[finale] hand-back at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w.skip_cutscenes = true;
    // The pit soldiers answered at the hand-back (drone-block-swaps-g055-3, opt-in —
    // re_arm and !tops_up): the ONE window where they stand visible with their clips
    // mostly intact. They hunt from the hand-back and empty everything into the pit phase
    // (the measured dry death at 14401: the squad phase opened on clip 1 spare 1 and the
    // weapon cycling found no round in any owned gun; the shelter's walls block the eye
    // rays from inside, so the pulse-1 sit sees nobody). A player turns and fights on the
    // open floor before the shelter door shuts (1000..1350 [D]) — the stand caps at +550
    // so the walk in keeps inside_at < 1000, and breaks the moment nobody is exposed.
    if re_arm && !w.tops_up() {
        let mut last_tap = 0u32;
        while w.total_ticks - t0 < 550 {
            let any = {
                let sim = w.scene.sim.as_ref().unwrap();
                let armed = sim.gun.clip + sim.gun.spare > 0 && (2..=0x47).contains(&sim.gun.weapon);
                armed && !sim.disabled && sim.drones.list.iter().enumerate()
                    .any(|(i, d)| d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0
                        && walker::dxz(d.pos, sim.bond.pos) < 30.0
                        && walker::DamageLog::exposed_part(sim, i).is_some())
            };
            if !any { break; }
            shelter_tick_proactive(w, &mut last_tap);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[real-damage] the hand-back stand: soldiers answered at +{}, health {:.1}, holds {:#04x} clip {} spare {}",
            w.total_ticks - t0, sim.drones.health.health, sim.gun.weapon, sim.gun.clip, sim.gun.spare);
    }
    w.walk_route(INTO_THE_SHELTER);
    let inside_at = w.total_ticks - t0;
    eprintln!("[finale] in the shelter +{inside_at}");
    assert!(inside_at < 1000, "Bond is in the room before its door starts to shut (+{inside_at}, the leaves move 1000..1350 [D])");
    // Pulse 1. The real-damage walk sits it out PROACTIVELY (drone-block-swaps-g055-3): the
    // pit soldiers are shot on sight through the door so their drops keep their clips for the
    // finale squad (the reactive sit let them pour fire the whole phase and the drops landed
    // dry). The punch test and the tops-up walks keep the exact original sit (re_arm and
    // !tops_up gate it, the file's established opt-in pattern).
    let n1 = if re_arm && !w.tops_up() {
        sit_out_pulse_proactive(w, "pulse 1 in the shelter (the pit soldiers answered)")
    } else {
        sit_out_pulse(w, "pulse 1 in the shelter")
    };
    assert!((555..=562).contains(&n1), "pulse 1 is 1611..2170 = 559 ticks [D stream 6], got {n1}");
    assert!(w.scene.sim.as_ref().unwrap().drones.mission.status() != 3, "no fail");
    // The chaser: ch 240 @2180 spawns her; the door reopens 2230..2580.
    let chaser = {
        tick_until(w, 2000, "the chaser spawns", |s| s.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)));
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)).unwrap()
    };
    // The game's FirstAttack flush [M 0x148010:100-131] writes her return state 0x56 [M :102],
    // and she works the shelter door at 2.3 m while Bond waits out the pulse — her -1.20 stars
    // every 6 ticks took the idle stand from >30 to dead inside the wait (the 2026-11-03 8715
    // death, both chaser walks). Bond fires short bursts back through the door while she is
    // above 55 health (the punch window still needs her alive with her punch intact).
    while w.total_ticks - t0 < 2600 {
        // One tick per iteration — a dry gun must not wedge the wait (the stand_and_fire cap
        // counts shots, and with no spare ammo it never returns).
        let (alive, hold, err, dist) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let her = &sim.drones.list[chaser];
            let bp = sim.bond.pos;
            let alive = her.present && !her.corpse && her.health > 0.0;
            // Above 55 at any range: the far arm also fired below it, and with the AR in hand
            // (the P2K ring clears leave its 30 rounds) the door wait shot her dead at range
            // (measured on the stream join: corpse at +2600, the punch window never opened).
            let hold = alive && her.health >= 55.0;
            let err = walker::wrap_pi((her.pos[0] - bp[0]).atan2(her.pos[2] - bp[2]) - sim.bond.yaw);
            (alive, hold, err, walker::dxz(her.pos, bp))
        };
        if !alive || w.tops_up() {
            // tops_up walks keep their exact pre-flush timeline (the counter-fire's shots
            // shift the shared stream); the real-damage walks (tops_up false) need the
            // return fire to live.
            w.tick(Input::default(), false);
            continue;
        }
        // guard-radii-16-chasers: the pit shooters answer while she works the door — the
        // chasers' chips: #6 skin 0x5000087 −1.0 at 13.3 u and #7 skin 0x5000090 −1.0 at 5.4 u
        // [L the 8519/10959 ledgers; hp 10 each [L probe_16_guards]]. While she is outside
        // punch range (the band 2.0 [M 0x1789c0:26-40] — 4.0 leaves her star cadence a trade
        // worth taking), the loop taps the nearest visible one instead: a few rounds each, and
        // the chips' source dies before the window's idle stand. Inside 4.0 the her-logic
        // below keeps the measured counter-fire.
        let side = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|&(i, d)| i != chaser && d.team == 1 && d.present && d.enabled && !d.corpse
                    && d.health > 0.0 && !matches!(d.death_channel, 23..=26)
                    && walker::dxz(d.pos, bp) < 25.0
                    && walker::DamageLog::visible(sim, i))
                .min_by(|a, b| walker::dxz(a.1.pos, bp).total_cmp(&walker::dxz(b.1.pos, bp)))
                .map(|(i, d)| (i, [d.pos[0], d.pos[1] + 0.3, d.pos[2]], walker::dxz(d.pos, bp)))
        };
        if let Some((_, at, sdist)) = side.filter(|_| dist >= 4.0) {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let serr = walker::wrap_pi((at[0] - bp[0]).atan2(at[2] - bp[2]) - sim.bond.yaw);
            let want_pitch = ((at[1] - eye[1]) / sdist.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on = serr.abs() < 0.04 && perr.abs() < 0.05;
            let input = Input { turn: serr.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
            if on && sdist >= 2.0 { w.tick_fire(input); } else { w.tick(input, false); }
            continue;
        }
        if !hold {
            w.tick(Input::default(), false);
            continue;
        }
        let input = Input { turn: err.clamp(-0.08, 0.08), ..Default::default() };
        if err.abs() < 0.06 && dist < 12.0 { w.tick_fire(input); } else { w.tick(input, false); }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[finale] the chaser #{chaser} at {:?}, +{}; Bond holds {:#x} (clip {} spare {}), owns {:x?}",
            sim.drones.list[chaser].pos, w.total_ticks - t0, sim.gun.weapon, sim.gun.clip, sim.gun.spare, owned(sim));
    }
    (chaser, t0)
}

/// The melee band hold, factored (level-16-pit-squad-real-damage-2): while any LIVE HOSTILE
/// (team 1 — the fists clones AND the pit-room AR guards alike) stands inside the 3.0
/// always-hit band [M DroneWeap_DoBulletAccuracy 0x179740:113-115 — under it every drone
/// round hits], backpedal facing the nearest and tap its exposed aim until the band opens
/// (cap `cap`). The punches fire only inside 2.0 [M DroneWeap_CloseCombatImpact 0x1789c0:
/// 26-40], so holding the band outside that keeps them whiffing; the AR guards' first landed
/// round inside the band is guaranteed — the measured 14656 death's finishers were #15/#14
/// (skins 0x50000b2/0x50000b1, weapon 0x14, hp 14/10) at d 1.2-1.3 in Bond's back while the
/// old clone-only hold faced away. A player surrounded turns to the nearest muzzle.
fn hold_the_band(w: &mut Walker, cap: u32) {
    let mut held = 0u32;
    // los-mask-repins-16 (additive diagnostic, NF_BAND_TRACE): what the hold sees — the closest
    // hostile's distance/state and whether an exposed aim exists — to explain a hold that never
    // fires.
    let band_trace = std::env::var_os("NF_BAND_TRACE").is_some();
    while held < cap {
        let close = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0 && d.team == 1
                    && walker::dxz(d.pos, bp) < 3.0)
                .min_by(|a, b| walker::dxz(a.1.pos, bp).total_cmp(&walker::dxz(b.1.pos, bp)))
                .map(|(mi, _)| mi)
        };
        let Some(mi) = close else { break };
        held += 1;
        let (input, fire) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let dp = sim.drones.list[mi].pos;
            let dist = walker::dxz(dp, b.pos);
            let aim = w.exposed_aim(mi);
            if band_trace && held % 60 == 1 {
                let st = sim.drones.list[mi].state;
                eprintln!("[band] +{held} #{mi} skin {:#x} state {st:#x} hp {:.1} d {dist:.2} aim {} gun {:#x} clip {} spare {}",
                    sim.drones.list[mi].skin, sim.drones.list[mi].health, aim.is_some(), sim.gun.weapon, sim.gun.clip, sim.gun.spare);
            }
            let at = aim.unwrap_or([dp[0], dp[1] + 0.3, dp[2]]);
            let eye = [b.pos[0], b.pos[1] + b.eye - b.crouch_drop, b.pos[2]];
            let err = walker::wrap_pi((at[0] - b.pos[0]).atan2(at[2] - b.pos[2]) - b.yaw);
            let want_pitch = ((at[1] - eye[1]) / dist.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - b.pitch;
            let on = err.abs() < 0.03 && perr.abs() < 0.04 && held % 16 == 0;
            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), forward: -1.0, ..Default::default() },
             aim.is_some() && on)
        };
        if fire { w.tick_fire(input); } else { w.tick(input, false); }
    }
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
    // The re-arm walk (consider-explosive-gate-3): the walk-over take of the drops already
    // auto-took the corridor guards' AR on the route [M Player_IsBetterWeapon 0x1a6eb0], so this
    // pass measuredly GAINS nothing (the pit-floor remnants are dry piles) — but the WALKS are
    // the point for the shared timeline: they consume the pit phase's guard-AI ticks, and the
    // measured passing run of walk_the_chasers_defeat_completes_the_mission includes them
    // (removing the pass shifted that walk's stream and broke its pass). No ammo writes.
    w.resupply("the pit-squad re-arm (the corridor guards' AR drops)");
    // Back into the shelter before the leaves shut again (6310..6660 [D]) — from where the fight
    // left Bond: a player who fought her from inside the room (west of the blast door at x
    // −11.0) stays in; the full route out to the pit floor and back in wedged on its first leg
    // for 1523 ticks and pulse 2 caught him outside.
    if w.scene.sim.as_ref().unwrap().bond.pos[0] < -12.0 {
        w.walk_route(&INTO_THE_SHELTER[INTO_THE_SHELTER.len() - 1..]);
    } else {
        w.walk_route(INTO_THE_SHELTER);
    }
    let back_at = w.total_ticks - t0;
    eprintln!("[finale] back in the shelter +{back_at}");
    // gas-rng-draws: the old < 6310 pin (before the shut COMMAND) was the measured stream's
    // shape — the shared stream's chaser phase runs ~57 ticks longer and the walk lands at
    // +6367, mid-travel. The physical law the pin protects: Bond is inside the room before
    // the leaves MEET at +6660 [D script 1350] (the walk arrived at +6367 with +97 to spare,
    // the leaf reversal on entry [M door.rs kind-2 anti-crush] notwithstanding), and the
    // room's pulse safety is the POLYGON exclusion [D HurtPoly-ShuttleLaunch excludes the
    // room] — pinned every pulse by sit_out_pulse's zero-contact assert below.
    assert!(back_at < 6660, "back in the room before the pulse-2 leaves meet (+{back_at}, they meet at 6660 [D script 1350])");
    let n2 = sit_out_pulse(w, "pulse 2 in the shelter");
    assert!((506..=513).contains(&n2), "pulse 2 is 6810..7320 = 510 ticks [D stream 9], got {n2}");
    eprintln!("[finale] pulse 2 over +{}", w.total_ticks - t0);
    // The cut-scene skip comes OFF before the squad phase (consider-explosive-gate-3): the
    // lift-out's AND #286 [0xbf·0xc0 → 0x82 [D — the placement's list 191,192]] fires the moment
    // BOTH inputs latch, and under the faithful throw-roll cadence [M 0x17a798] the squad's
    // deaths can land AFTER ch 0xc0 @7808 [D stream 20] — the measured run reached ch 0xbf at
    // +8293 with ch 0xc0 already latched, so 0x82 went on while the skip was still up and the
    // walker's fast-forward played the whole 0x06000632 cinema within one tick (its disable
    // window, event 4 at t0 to event 6 at t1057 [D], was never observed between samples). The
    // mission's own transition must play out unskipped whenever it fires.
    w.skip_cutscenes = false;
    // The pit squad (group 7, ch 242 @7332): all four must fall for ch 0xbf.
    tick_until(w, 600, "the pit squad spawns", |s| s.drones.list.iter().any(|d| d.present && !d.corpse && matches!(d.death_channel, 23..=26)));
    let t_squad = w.scene.sim.as_ref().unwrap().drones.frame;
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
        // The melee band hold — the factored `hold_the_band` (its doc carries the 14656
        // cause: the pit-room AR guards closed INSIDE the band while the old clone-only hold
        // faced away). The threat set is ANY live hostile inside 3.0.
        hold_the_band(w, 240);
        // Dry mid-fight: another carried gun with ammo or the fallen squad's guns, then fight on.
        // (consider-explosive-gate-3: a 3-step re-arm ladder was tried here and REVERTED — its
        // extra wheel cycles shifted the top-up walk's measured stream and broke its pass. The
        // real-damage finale walk's remaining blocker is upstream ammo, not this ladder.)
        // level-16-pit-squad-real-damage-2: the kill runs in BOUNDED slices with the band hold
        // re-checked between them — the member kill is a fixed-target loop (`shoot_bounded`
        // drives Bond directly, no threat reaction), and the measured death sat INSIDE it
        // while the pit guards walked to point blank over ~7300 ticks. A player answers the
        // muzzle that closes on him the tick it arrives, then returns to the squad member.
        let mut killed = false;
        // los-mask-repins-16: the slices are 300 ticks (36 of them, the same budget the 900 x12
        // ladder had) so the band hold re-checks the closing muzzle every ~5 s — the measured
        // death sat INSIDE a 900-tick slice while #15 walked to d 1.7 and poured. Between the
        // slices the OTHER muzzle is answered too: a pit-room guard that hurt Bond in the last
        // 240 frames, exposed, within 20 u, gets a bounded stand of its own before the member —
        // the measured 14785 death poured through the whole ladder from #15 (the hold's
        // backpedal exits his 3.0 band and the slice resumed on the far clone; the hold only
        // faces the nearest INSIDE-3.0 hostile, and #15 sat just outside it at 2.5-2.9).
        for _ in 0..36 {
            hold_the_band(w, 240);
            let threat = {
                let sim = w.scene.sim.as_ref().unwrap();
                let now = sim.drones.frame;
                let bp = sim.bond.pos;
                sim.drones.player_hits.iter().rev()
                    .take_while(|h| now.saturating_sub(h.0) <= 240)
                    .filter(|h| h.1 != i && (h.1 as usize) < sim.drones.list.len())
                    .filter_map(|h| {
                        let d = &sim.drones.list[h.1 as usize];
                        (d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0
                            && walker::dxz(d.pos, bp) < 20.0
                            && walker::DamageLog::visible(sim, h.1 as usize)).then(|| (h.1 as usize, h.2))
                    })
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .map(|(gi, _)| gi)
            };
            if let Some(g) = threat {
                if w.shoot_bounded(g, "the squad phase's dealer", 300) { continue; }
            }
            // 00 Agent: the far member's slices let #15 (0x14, 10.0 a hit) and the brute #12 walk in
            // unanswered; any other live hostile inside 10 u gets a short stand first.
            if w.scene.sim.as_ref().unwrap().drones.difficulty >= 3 && !w.tops_up() {
                let r = 10.0f32;
                let close = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let bp = sim.bond.pos;
                    (0..sim.drones.list.len()).filter(|&k| { let d = &sim.drones.list[k];
                        k != i && d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0 && walker::dxz(d.pos, bp) < r })
                        .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp)))
                };
                // 120-tick stands: the slice loop re-checks between them (measured: 120 completes the
                // walk; 300 died t17678, 600/1200 t17860).
                if let Some(g) = close { if w.shoot_bounded(g, "the squad phase's close one", 120) { continue; } }
            }
            if w.shoot_bounded(i, "the pit squad", 300) { killed = true; break; }
        }
        let killed = killed
            || ((w.switch_to_armed("the pit squad ammo") || w.resupply("the pit squad ammo"))
                // los-mask-repins-16: the fallback is BOUNDED (1800) — the bare `shoot` runs
                // 10800 ticks with no threat reaction, and the measured death happened inside
                // exactly that loop (the pit guards closed and poured while the stand held).
                && w.shoot_bounded(i, "the pit squad ammo", 1800));
        assert!(killed, "the pit squad member #{i} dies to Bond's fire");
    }
    tick_until(w, 600, "the pit squad's deaths raise ch 0xbf [D mux #611]", |s| s.drones.channels.on(0xbf));
    // The squad phase's per-hit audit (level-16-pit-squad-real-damage): no squad-phase hit may
    // take more health than the game's strongest single deal in that room — the melee clones'
    // punch, record 99 [D +0x0C = 50.0, chaser_melee_records pins the bytes] × 0.4 [M
    // 0x1d4424's Operative row] = 20.0, which fires only inside the 2.0 band [M 0x1789c0:26-40].
    // Every other dealer's chain lands single digits: the clones' star record [D 0x12's +0x0C]
    // ×(1|2 TooClose [M 0x139fac-0x139fc8])×0.4 and the guards' weapon records ×(1|2)×0.4 [D
    // +0x0C]. Armour absorbs before health [M Player_HandlePain 0x1902c8:84-104's type-0 arm],
    // so a recorded delta never exceeds the raw chain — a port that over-deals (the ignore's
    // ~48-health death) shows deltas above this cap.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The pain row per menu pick: 14-16's Plr_DMod Easy/Normal/Hard = 0.4/0.6/1.0 [M
        // ReadTuningVars 0x1d3a80 case 0x7000014-16]; the cap is the punch × the pick's row.
        let row = match sim.drones.difficulty { 1 => 0.4f32, 2 => 0.6, _ => 1.0 };
        let cap = 50.0 * row;
        let mut audited = 0usize;
        for &(f, owner, delta) in sim.drones.player_hits.iter() {
            if f <= t_squad || owner == usize::MAX { continue; }
            audited += 1;
            assert!(delta <= cap + 0.02,
                "squad-phase hit from dealer {owner:?} dealt {delta:.2} health — above the game's strongest squad-phase deal (the punch 50.0 [D record 99] × {row} [M 0x1d3a80] = {cap})");
        }
        eprintln!("[real-damage] the squad-phase audit: {audited} hits, every delta within the game's chain [D record 99; M 0x1d4424/0x139fac]");
    }
    eprintln!("[finale] ch 0xbf +{}", w.total_ticks - t0);
    // ch 0xc0 @7808 → ch 0x82 → 0x06000632 plays out (the skip is already off — see the squad
    // phase above) and hands Bond back at the bay.
    {
        // consider-explosive-gate-3 diagnostic: the AND's own state at the wait's head.
        let sim = w.scene.sim.as_ref().unwrap();
        for t in &sim.drones.triggers {
            if t.out == 0x82 || t.out == 0xbf {
                eprintln!("[finale] trigger #{} kind {} out {:#x} list {:?} lock {} gone {}",
                    t.placement, t.kind, t.out, t.list, t.lock, t.gone);
            }
        }
        eprintln!("[finale] at +{}: ch 0xc0 {} ch 0xbf {} ch 0x82 {} mission {}",
            w.total_ticks - t0, sim.drones.channels.on(0xc0), sim.drones.channels.on(0xbf),
            sim.drones.channels.on(0x82), sim.drones.mission.status());
    }
    // gas-rng-draws: the AND (0xbf·0xc0 → 0x82) fires the MOMENT ch 0xbf latches [D #286] —
    // on the shared stream that is still inside the squad loop's tail, so the unskipped
    // 1057-tick cinema (event 4 t0 → event 6 t1057 [D 0x06000632]) can play out BEFORE this
    // wait's head (the fresh ledger: at +9660 ch 0x82 true, disabled false, Bond already
    // handed back). The cinema's completion is observable: the hand-back key (5.39, 1.34,
    // −158.31) sits at y +1.34 while every pre-cinema spot (the pit, the shelter) is y −22 —
    // so the wait accepts either the disable window or the already-handed-back bay (y > 0).
    tick_until(w, 3000, "ch 0x82 starts the lift-out cinema [D mux #633]", |s| {
        s.drones.channels.on(0x82) && (s.disabled || s.bond.pos[1] > 0.0)
    });
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
fn clear_the_pit_shooters(w: &mut Walker, chaser: usize, stand_cap: u32) {
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
        // The flush [M 0x148010:100-131] puts the pit soldiers on their second blocks and they
        // hunt the stand — the stand's 900 ticks can out-damage Bond from >30 to dead inside ONE
        // window iteration (the 2026-11-03 8715 death). Survival first: bail the clear when the
        // health floor would break the caller's loop anyway.
        if w.scene.sim.as_ref().unwrap().drones.health.health < 30.0 { break; }
        let _ = w.switch_to_armed("the pit room shooters");
        let killed = w.stand_and_fire(di, stand_cap);
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
/// consider-explosive-gate-3 2026-11-02 UN-IGNORED with a player's approach tactic: the
/// faithful throw-roll cadence [M 0x17a798 — the 50% Rand(2) precedes the dist check, so every
/// grenade-holding guard in 0x68 consumes rolls] retimed the launch centre, and the unanswered
/// corridor walk died at tick 2362 (the measured run: 47 hits for 100.0 — the AR guards #0/#2/#3
/// skins 0x500008c, #4/#5 0x50000b1 [D p9 = 26] pour fire down the south corridor; the last hit
/// d 1.9 at (−12.1, 6.8, −83.4)). The no-return-fire premise is the PUNCH WINDOW's (an idle
/// stand while she closes and swings), not the approach's — a player fights through, so the
/// walk raises the walker's own `dmg.return_fire` for the corridor legs (real-input fights, the
/// walker's plain pick) and drops it again at the hand-back. No common/ change: the field is
/// the walker's own pub opt-in, set by this test only.
#[test]
fn the_chasers_punch_lands_the_record_damage() {
    walker::with_real_damage_no_return_fire(|| {
        let iso = iso();
        let save = carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]);
        let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000016.bin", false, Some(&save), 1).unwrap());
        // The approach fights back (consider-explosive-gate-3): the walker's own return fire for
        // the corridor legs — the unanswered walk died at 2362 under the AR guards' fire (the
        // measured 47 hits for 100.0). The walker's plain pick, real input; the walker's damage
        // log still audits every landed hit.
        w.dmg.return_fire = true;
        // guard-radii-16-chasers: the d1 route economy (the defeat walk's proven set). The plain
        // route's reactive fights traded 75.6 of Bond's 100 with the ring/gallery pair from the
        // ground (the measured 8519 death: 17.4 left at the window, the chaser's punch took the
        // last 9.4 at 0.2 u) — the ring legs clear at level (hp 10 each [L probe_16_guards]) and
        // the fights finish, so Bond arrives with health to spend on the punch pin.
        w.dmg.finish_fights = true;
        w.dmg.level_fights = true;
        // A tap with the frag in hand THROWS it (the defeat walk's measured self-blast — the
        // kill loop's dry re-arm cycled onto the 0x34 and the loot walks strolled into the
        // blast): every tap in this walk is a bullet.
        w.no_grenade_throws = true;
        // The head tap (×5 [M NDrone2_HitDamage's part-5]) keeps the pit shooters' clears inside
        // their hp-10 budgets and the window's stands short.
        w.prefer_head = true;
        // los-mask-repins-16: the corridor fights leave Bond wherever they end (the flank nav's
        // east pocket) — a wedged nav leg re-routes over the clear nav lines instead of wedging
        // (the 01 walk's opt-in).
        w.bridge_wedged_nav = true;
        // gas-draws-8: the merged stream's plain route bleeds out before the window (the
        // measured 1843 run: 86.8 of 100 through the corridor from the gallery/ring pair alone —
        // hits at 66-90 u the reactive fights never answer — Bond at the window with ~9, the
        // chaser shot out of the punch window by the counter-fire loop). The defeat walk's FULL
        // economy (the ring legs' level clears, the pit stand) is what held this route on the
        // gas stream (the defeat walk: green in the same binary); the punch pin's only stake is
        // the armour-0 window (station_armour stays false — the #596 straps stay on the rack).
        let (chaser, t0) = walk_to_the_chaser_fight(&mut w, true, false);
        // The route's return-fire fights spend bolts — the arrival SET is the chain's [M
        // 0x1a7ef8], but the gun in hand may be dry; the fight continues on the carried P2K
        // (rounds pinned [D the chain save]). The allowance sister pins the arrival's held gun.
        // guard-radii-16-chasers MEASURED: the plain route's fights left the Delta DRY (clip 0
        // spare 0 in hand at the window) — the counter-fire's side taps and the window's
        // clear stands then had no bullets (#7 died, #6/#8 outlived their 900-tick stands,
        // 32 health of crossfire, and she never closed to swing). Cycle to a loaded gun (the
        // route's dropped SMG 0x1a is owned [M Player_IsBetterWeapon's auto-take]; the
        // no_grenade_throws gate keeps the cycle off the frag).
        assert!(w.switch_to_armed("the punch window's gun"), "a carried gun has rounds at the window");
        // The doorway meet (guard-radii-16-chasers MEASURED): she parks outside the leaves at
        // 4.67 (state 0x7c, zero landed stars at d1 across the whole window) and the interior
        // idle stand never sees her punch — the 8519-shaped stream had her slip in, this one
        // does not. A player steps to the doorway and lets her reach the band: Bond walks to
        // the west blast door (the INTO_THE_SHELTER mid leg) and idles there; she is then
        // ~1.2 u off — inside the 2.0 punch band [M 0x1789c0:26-40] — and swings. The clear
        // rounds start after that first second (clear_at below), so the punch owns the first
        // window.
        // The punch window goes back to the unanswered stand: the pin is the DAMAGE her punch
        // carries, and it lands on an idle Bond while the pit shooters are cleared explicitly.
        w.dmg.return_fire = false;
        let star_damage = {
            let sim = w.scene.sim.as_ref().unwrap();
            assert_eq!(sim.drones.progress.difficulty, 1, "Operative rode the load path [M 0x1d4424 row]");
            // The #596 armour the route picks up now (guard-radii-16-chasers) must be SPENT by
            // the route's fire before the window [M Player_HandlePain's type-0 arm] — armour
            // here would absorb the punch and the pin would read the absorption, not the 20.0.
            assert_eq!(sim.drones.health.armour, 0.0, "no armour at the punch — the health loss IS the punch (the route's #596 armour [D] must be spent by the route's fire first)");
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
        // A pit soldier standing on the meet spot is shot first (measured on the stream join:
        // #6, hp 5, held the spot 1.3 u from Bond and chipped him dead after the punch while
        // the walk pushed against him). Close soldiers only, never the chaser.
        for _ in 0..3 {
            let pick = {
                let sim = w.scene.sim.as_ref().unwrap();
                let meet = [-11.0f32, -22.0, -170.0];
                sim.drones.list.iter().enumerate()
                    .filter(|&(i, d)| i != chaser && d.team == 1 && d.present && d.enabled && !d.corpse
                        && d.health > 0.0 && (walker::dxz(d.pos, meet) < 4.0 || walker::dxz(d.pos, sim.bond.pos) < 4.0)
                        && walker::DamageLog::visible(sim, i))
                    .min_by(|a, b| walker::dxz(a.1.pos, sim.bond.pos).total_cmp(&walker::dxz(b.1.pos, sim.bond.pos)))
                    .map(|(i, _)| i)
            };
            let Some(di) = pick else { break };
            let _ = w.switch_to_armed("the meet spot's soldier");
            let killed = w.stand_and_fire(di, 300);
            eprintln!("[real-damage] the meet spot's soldier #{di} down {killed} at health {:.1}",
                w.scene.sim.as_ref().unwrap().drones.health.health);
        }
        // The meet ends at the spot or as soon as she stands inside the 2.0 punch band [M
        // 0x1789c0:26-40]: on this stream she holds the doorway spot herself, and the plain leg
        // pushed into her for 600 ticks while her stars finished Bond (measured on the stream
        // join: never within 1.4 u of the spot). Plain forward steps toward the spot, no fire.
        for _ in 0..900 {
            let (done, err) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                let meet = [-11.0f32, -22.0, -170.0];
                let her = &sim.drones.list[chaser];
                let close = her.present && !her.corpse && walker::dxz(her.pos, bp) < 2.0;
                let err = walker::wrap_pi((meet[0] - bp[0]).atan2(meet[2] - bp[2]) - sim.bond.yaw);
                (close || walker::dxz(meet, bp) < 0.9, err)
            };
            if done { break; }
            w.tick(Input { forward: if err.abs() < 0.5 { 1.0 } else { 0.0 }, turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
        }
        let mut chaser_hits: Vec<f32> = Vec::new();
        let mut clear_at = 900u32;
        while w.total_ticks - t0 < 6300 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                chaser_hits = sim.drones.player_hits.iter().filter(|&&(_, o, _)| o == chaser).map(|&(_, _, d)| d).collect();
            }
            if chaser_hits.iter().any(|&hit| (hit - 20.0).abs() < 0.01) { break; }
            if w.scene.sim.as_ref().unwrap().drones.health.health < 30.0 { break; } // her point-blank bursts — survival first
            if w.total_ticks >= clear_at { clear_the_pit_shooters(&mut w, chaser, 900); clear_at = w.total_ticks + 900; continue; }
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
    // The pit shooters answer the melee fight (drone-block-swaps-g055-3, the task's clear —
    // the pit soldiers, NEVER the chaser herself): while the range-keeping loop holds the
    // band, the soldiers' steady fire is the run's other dealer, and their drops (each with
    // its own clip [M DroneWeap_DropWeapon 0x17a2a0]) land on the backpedal floor for the
    // finale squad phase. A player clears the doorway while backing away from the fists —
    // the loop picks the nearest VISIBLE live pit soldier (team 1, not a finale clone —
    // their death channels are 23..=26 [D mux #611]) alongside the chaser's aim refresh,
    // and fires at it with the same tap discipline whenever she is outside the punch band
    // (2.0 [M 0x1789c0:26-40]) or out of sight; her own taps resume the moment she is
    // inside 2.0. The final squad phase then fights on the soldiers' dropped rounds instead
    // of Bond's dried carry (the measured dry-stand death at 14551).
    let mut side: Option<(usize, [f32; 3], f32)> = None;
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
        if t % 30 == 0 {
            aim = w.exposed_aim(chaser);
            side = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.list.iter().enumerate()
                    .filter(|&(i, d)| i != chaser && d.team == 1 && d.present && d.enabled
                        && !d.corpse && d.health > 0.0 && !matches!(d.death_channel, 23..=26)
                        && walker::dxz(d.pos, bp) < 25.0
                        && walker::DamageLog::visible(sim, i))
                    .min_by(|a, b| walker::dxz(a.1.pos, bp).total_cmp(&walker::dxz(b.1.pos, bp)))
                    .map(|(i, d)| (i, [d.pos[0], d.pos[1] + 0.3, d.pos[2]], walker::dxz(d.pos, bp)))
            };
        }
        // The fire plan: the side soldier when she is outside the punch band (her approach
        // legs, not her swings) or out of sight, else the chaser herself.
        let shooting_side = side.is_some() && (aim.is_none() || dist >= 2.0);
        let (input, fire) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let (at, tdist, have) = match (shooting_side, side) {
                (true, Some((_, at, sdist))) => (Some(at), sdist, true),
                _ => (aim, dist, aim.is_some()),
            };
            let at = at.unwrap_or([dp[0], dp[1] + 0.3, dp[2]]);
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let err = walker::wrap_pi((at[0] - bp[0]).atan2(at[2] - bp[2]) - sim.bond.yaw);
            let want_pitch = ((at[1] - eye[1]) / tdist.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0;
            let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
            let fire = if !have { false } else { on_target && tdist >= 2.0 };
            (input, fire)
        };
        // Out of sight and far: close for the view (she is coming anyway — the chain closes);
        // with a side soldier exposed, stand and tap it instead of walking into her.
        if aim.is_none() {
            let forward = if shooting_side { 0.0 } else if dist > 5.0 { 1.0 } else { 0.0 };
            let input = Input { forward, ..input };
            if fire { w.tick_fire(input); } else { w.tick(input, false); }
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
/// THE PIT-SQUAD LEG, RESOLVED (level-16-pit-squad-real-damage, 2026-11-02): the ignore's
/// ~48-health death does not reproduce on master d699b536's sim — the walked finale completes
/// on real health (measured: end health 9.4, armour 43.2, 75 hits for 90.6; the squad-phase
/// dealers were the clones' 1.2-value stars and the door guard's 2.0-value rounds, no punch
/// landed). The damage audit pins WHY that is the game's own math, per hit: the squad's clones
/// are skin 0x500001e → kind 0xb melee [M NDrone2_DefaultInit 0x14b300's switch] whose only
/// deals are the punch 50.0 [D record 99 +0x0C, chaser_melee_records] × 0.4 [M 0x1d4424's
/// Operative row] = 20.0 inside the 2.0 band [M 0x1789c0:26-40] and their placement weapon's
/// star record damage ×(1|2 TooClose [M 0x139fac-0x139fc8]) × 0.4; the door guard (#8, skin
/// 0x5000093 [D], weapon 0x14 [D]) deals its record's damage ×(1|2)×0.4 [D +0x0C]. The walker's
/// tactics are the honest set: the shelters (both pulses), the range-keeping chaser kill, and
/// for the squad the melee band hold (backpedal-with-fire while a clone is inside 3.0 — under
/// it every drone round hits [M 0x179740:113-115]; the punch only inside 2.0) with
/// stand-and-fire from the shelter's cover. No health top-ups, no state pokes.
/// The difficulty-1 expression is the pre-sister master's byte-for-byte; 2+ loads the front
/// end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty] and asserts the
/// sim's difficulty at load [M GameState+0x28 0x2a3790 → Drones::difficulty].
fn walk_the_chasers_defeat_completes_the_mission_on_real_damage_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap()
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b])), difficulty).unwrap()
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // The route fights' economy (drone-block-swaps-g055-3, d1 only — the Agent sister keeps
    // its own measured tactics): under the flush the guards hunt from their first attack and
    // the un-dropped hunter re-opens at every leg — the measured d1 route bled ~80 of Bond's
    // 100 across 8 fights before the pit (he arrived at 20.2 and the finale squad finished
    // him dry). The lane pick measured WORSE (the stand traded 61.6 in one fight and the run
    // died on the route at 1926 — reverted); the opt-in finish_fights escalation is the
    // keeper: a player finishes the guard he is fighting (hp 20 — the rounds are cheaper
    // than the bleed, and the route has no hard window).
    if difficulty == 1 {
        w.dmg.finish_fights = true;
        // The melee band (1b-zero-g-floor §3, the 14656 death): the route's chasers closed to
        // d 0.8-1.3 on the stands and finished Bond INSIDE the 3.0 always-hit band
        // [M DroneWeap_DoBulletAccuracy 0x179740:113-115 — under it every drone round hits] —
        // the plain stand held position while they swam in. A player backs out of the band
        // while he answers: the opt-in back-off (the 08 walk's 4.0 idiom, 3.5 here — just
        // over the band) steps Bond back, aim held, while any live guard stands inside, so
        // the chasers' rounds ride the outside-of-band accuracy instead of the guaranteed
        // hits, and the punches (only inside 2.0 [M 0x1789c0:26-40]) never reach.
        w.dmg.back_off = Some(3.5);
        // los-mask-repins-16 (the squad-phase dealer answer, the measured 14818 death): the
        // pit-room AR guard #15 (skin 0x50000b2, weapon 0x14, hp 20) closed to d 1.7-1.9 while
        // the squad kill loop stood on the far clone #12 and poured 45.8 in 26 hits — the
        // return-fire pick had him on a FAILED-fight cooldown and the plain pick skipped him
        // BECAUSE he was cooled. A player answers the muzzle that is actually hurting him:
        // the opt-in hurting_first (the 07000008 Agent sisters' law) — cooled candidates are
        // dropped only while they are NOT the real dealer, and the damage-weighted order
        // fights him first.
        w.dmg.hurting_first = true;
        // drone-block-swaps-g055-4: the reactive pick skips the ring guards a level above
        // (the shared flag's doc) — the ground legs keep moving under the overhead fire and
        // the ring legs clear at level.
        w.dmg.level_fights = true;
    }
    // guard-radii-16-chasers: a tap with the frag in hand THROWS it — the measured 10959 death
    // was Bond's OWN frag (the kill loop's dry re-arm cycled onto the 0x34, the next taps threw
    // it, and the loot walks strolled into the fuse's blast; the ledger's two "world" hits were
    // the frag record's radius 8 / damage 100 [L USATxt.dat]). Every tap below is a bullet, and
    // the head tap (×5 [M NDrone2_HitDamage's part-5]) stretches the kill's ~43 rounds over the
    // chaser's 100 hp.
    w.no_grenade_throws = true;
    w.prefer_head = true;
    let (chaser, t0) = walk_to_the_chaser_fight(&mut w, true, true);
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
        // HER first (consider-explosive-gate-3 reorder): the faithful throw-roll cadence [M
        // 0x17a798] retimed the launch centre and the measured run made her the fight's real dps
        // by a mile — 39.6 of the run's 90.6 attrition (one band breach + her approach stars)
        // against the pit shooters' ~8 — while the old shooters-first order kept her plinking
        // through the whole ~2700-tick clearing. The band hold (range keeping) is unchanged.
        // The Agent tactics (real-health-agent-difficulty-16-1b-41-4a): the pit-door guard is
        // answered BEFORE the chaser — the measured Agent run ate 15.0 of her AR rounds
        // (skin 0x5000093, weapon 0x14, hp 20) through the whole chaser stand while the range-
        // keeping loop held Bond on her; a player clears the doorway shooter first. The spot is
        // the death log's (-3.1, -22.0, -163.5); the roster at d2 is found by proximity to it.
        if difficulty >= 2 {
            let door_guard = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| {
                d.present && !d.corpse && d.health > 0.0 && d.team == 1
                    && (d.pos[0] - -3.1).powi(2) + (d.pos[1] - -22.0).powi(2) + (d.pos[2] - -163.5).powi(2) < 36.0
            });
            // 00 Agent (d3): the blast room's own soldier (#6, skin 0x5000087, 0x14) stands 2.6-3.9 u
            // from Bond at the chaser stage and landed 25.0 in 6 while the stands faced the door;
            // he is fought first (inside 6 u, never the chaser), and the door guard is taken only
            // when exposed - the hidden-guard stand let the chaser walk in point-blank (t12411).
            // Measured: the chaser goes down (+2806) and the walk dies in the pit-squad stage,
            // t12518 -> t17276.
            if difficulty >= 3 {
                let r = 6.0f32;
                for _ in 0..3 {
                    let near = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let bp = sim.bond.pos;
                        (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                            i != chaser && d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                                && walker::dxz(d.pos, bp) < r && (d.pos[1] - bp[1]).abs() < 2.0 })
                            .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp)))
                    };
                    let Some(di) = near else { break };
                    let _ = w.shoot_bounded(di, "the blast room's soldier before the chaser", 600);
                }
                eprintln!("[016] room soldiers done, health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
                // The room-soldier fight leaves Bond against the south wall (-14.6, -173.1): the
                // chaser (her 0x14 gun) then landed 4 x 5.0 at 2.4 u while the range keeping could
                // not back off. Back to the room's open floor first (the shelter route's own stand):
                // the chaser fight then costs nothing (37.0 into the squad stage, was 17.0).
                w.lenient = true;
                w.walk_to(walker::Wp::new(-14.45, -22.0, -170.16).r(0.5), "the blast room's open floor");
                w.lenient = false;
            }
            let skip_hidden = difficulty >= 3;
            if let Some(g) = door_guard {
                if !(skip_hidden && !walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), g)) {
                    w.shoot(g, "the pit-door guard (skin 0x5000093) before the chaser");
                }
            }
        }
        kill_the_chaser_with_range_keeping(&mut w, chaser, t0);
        // Then the pit room's soldiers (their steady fire is what remains; the first run died to
        // them at the shelter). The 300-tick stand cap: the faithful-stream measured run had
        // a 900-tick stand pour ~56 rounds of the AR into a shooter it never dropped (#8,
        // down false) and leave the squad fight bone dry — a player shorts the futile stand
        // and keeps the magazine for the clones.
        clear_the_pit_shooters(&mut w, chaser, 300);
        // The loot pass (consider-explosive-gate-3): the route's fallen guards' drops (each with
        // its own clip [M DroneWeap_DropWeapon 0x17a2a0]) lie where they fell — a player walks
        // over them. The walker's own resupply is blind here twice over: its trail filter holds
        // 9 u around the NAV nodes (the fight stands are not trail points, so the pit shooters'
        // drops 10-25 u out never pass) and its success check reads only the held gun's rounds
        // (the 0x14 drops file into the inventory WITHOUT equipping — the held AR outranks them
        // [M Player_IsBetterWeapon 0x1a6eb0] — so a successful take still reports `false`). So
        // the walk goes corpse to corpse itself: every fallen team-1 guard within reach, walk to
        // its spot, let the game's own walk-over take [M Pickup_Handler 0x18e650] do the rest.
        {
            // guard-radii-16-chasers: the pit-room armour #648 (−14.41, −22.97, −172.75) [D the
            // audit's placement table] — 2.4 u from the shelter's stand: a player straps the
            // BodyArmourHeavy on while the pit is quiet (the old measured pass's armour 43.2 was
            // this pickup; the walk-over take [M Pickup_Handler 0x18e650], arrive 0.6 — the
            // take's radius is under a unit [M]). The floor target y (the walk's plane), the
            // placement's −22.97 is the box's own rest.
            // From the room's north side (with the guards' aim stop (Drone+0x38) the pit fight
            // ends with Bond at (-17.0, -168.1)) every hop toward the armour slid along a wall
            // out to (-10.5, -182) for the 3000-tick cap and pulse 2 caught him outside: the
            // room's own stand (the shelter route's last point) first, then the armour.
            if w.scene.sim.as_ref().unwrap().bond.pos[2] > -169.5 {
                w.lenient = true;
                w.walk_to(walker::Wp { at: [-15.2, -22.0, -170.4], act: walker::Act::None, arrive: 0.8 }, "the blast room stand (before the armour)");
                w.lenient = false;
            }
            w.walk_nav_to([-14.41, -22.0, -172.75], "the pit-room armour #648 [D]", 0.6);
            let spots: Vec<[f32; 3]> = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.list.iter()
                    .filter(|d| d.corpse && d.team == 1 && walker::dxz(d.pos, bp) < 45.0)
                    // gas-rng-draws round 2: the wire filter removed the deadly east drop, and
                    // the trimmed walks still left Bond at the shelter door at +6367 — the
                    // door-shut law wants < 6310 [D script 1000..1350]. The east side (x > -8,
                    // the chaser's (-4.5, -165.7) among them) is the wander AWAY from the
                    // shelter — every east corpse is a detour the walk pays twice. A player
                    // loots toward where he is going next; the pass keeps the shelter-side
                    // drops only (they are dry piles anyway — the arming happens on the route
                    // drops already auto-taken [M Player_IsBetterWeapon 0x1a6eb0]).
                    .filter(|d| d.pos[0] < -8.0)
                    // gas-rng-draws: the pit floor carries always-on hurt volumes (the fresh
                    // ledger: the loot walk 2 to (-5.76, -166.00) ate 10.0 + 21.2 "the world"
                    // at 10804/10932 and the run died there — the pulse poly is 1000/tick and
                    // event-gated, so this is the wire class). A player does not loot a drop
                    // standing in a live wire; the spot is probed with the standing capsule
                    // (feet → feet+1.5, r 0.4 — Hurt::contains' own shape [M hurt.rs]) and
                    // skipped when any volume claims it.
                    .filter(|d| {
                        let s = [d.pos[0], d.pos[1], d.pos[2]];
                        let head = [s[0], s[1] + 1.5, s[2]];
                        !sim.hurts.iter().any(|h| h.contains(s, head, 0.4))
                    })
                    .map(|d| [d.pos[0], d.pos[1], d.pos[2]]).collect()
            };
            for (k, s) in spots.iter().enumerate() {
                w.walk_nav_to(*s, &format!("the loot walk {k}"), 1.0);
            }
            switch_to_a_deep_firearm(&mut w, "the loot walk");
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[real-damage] the loot walk: {} corpses, owns {:?}, in hand {:#04x} clip {} spare {}",
                spots.len(), owned(sim), sim.gun.weapon, sim.gun.clip, sim.gun.spare);
        }
        finish_after_the_chaser(&mut w, t0);
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.health.health > 0.0, "Bond survives the finale on the port's real damage");
        eprintln!("[real-damage] the finale walked: {}", w.dmg.summary(sim));
}

/// The objective chain at the other two menu picks with the walker's health allowance: the
/// min-difficulty placements [M Drone_Create 0x136be0:49] change the roster at Agent (2) and
/// 00 Agent (3), so each pick's real path to ch 0x8f, the button's ch 0x90 and ch 100 is
/// walked. The real-health sisters stay the damage pins.
#[test]
fn walk_the_chasers_defeat_completes_the_mission_agent() {
    walk_the_chasers_defeat_completes_the_mission_on_real_damage_at(2);
}

#[test]
fn walk_the_chasers_defeat_completes_the_mission_00agent() {
    walk_the_chasers_defeat_completes_the_mission_on_real_damage_at(3);
}

/// UN-IGNORED (level-16-pit-squad-operative, 2026-11-13): the ignore's tick-15337 death does
/// not reproduce on master 1105786cd's sim — the walked finale completes on real health
/// (measured on this tree, NF_DMG_TRACE: end health 46.8, armour 32.0, 69 hits for 53.2 with
/// the lowest health 46.8 — the whole pit phase's hits land on the armour and take NO health;
/// the chaser fell +3349 into her window, the squad #12/#13/#14/#15 fell inside the bounded
/// slices, ch 0xbf at +8461, the button played the lift-out and the mission completed). The
/// accuracy byte the ignore tuned against is the game's own rule, confirmed in the decompile:
/// `NDrone2_DefaultInit` 0x14b300's mode-0x0D arm WRITES `+0xB4 = 0` (with health 100.0,
/// +0x8F0 8.0, +0xEC 2.0 and the 0x3c/0x36/0x35 props) after DoModeSettingsNEW's stats-table
/// read, and `DroneWeap_DoBulletAccuracy` 0x179740 reads it at :52 as `chance = 100 − (+0xB4)·5`
/// — byte 0 is 100 before the factors — then ×TooClose_Accuracy under TooClose_Distance (:53-55
/// and again :95-97), ×TargetFirstMoved_Accuracy on +0x1DD (:56-58), ×TargetFirstStopped_Accuracy
/// for level 12/13's anim-set-0x13 drones (:60-70), ×TargetMoving_Accuracy on +0x1DC (:78-80),
/// ×the difficulty factor (Operative = Easy 0.5, tuning 0x1d3a80's 0x07000014-16 row; :81-93);
/// a hit needs Rand_FRand(100) < chance (:100-101) AND, beyond TooClose, +0x270 past
/// NewSighting_TimeToHit·FRAME_RATE with +0x41 clear (:103-109); under 3.0 EVERY round hits
/// (:113-115). The walk's existing stand already tunes against exactly those levers: the
/// band-hold backpedal keeps every dealer outside the 3.0 always-hit band, the bounded slices
/// answer the muzzle that hurts (hurting_first, d1-gated in the walk body), and the pit-room
/// armour #648 [D] carries the squad's stars. The intervening master commits (the guards'
/// AimStandFire root stop 7df2f148d and the room-visibility draw gate) shifted the fight away
/// from the ignore's measured death; the sim is bit-exact deterministic, so this pass is not a
/// coin toss. No health top-ups, no damage lowered, no state pokes.
#[test]
fn walk_the_chasers_defeat_completes_the_mission_on_real_damage() {
    walker::with_real_damage_head_hunting(|| walk_the_chasers_defeat_completes_the_mission_on_real_damage_at(1));
}

/// The Agent-difficulty sister (real-health-agent-difficulty-16-1b-41-4a): the same walked
/// finale on real health, loaded at difficulty 2 the menu way — Bond's pain scales by 16's
/// Plr_DMod_Normal [M Player_HandlePain 0x1902c8; 0x1d3a80's 0x11..=0x13 group 0.5 → 0.6,
/// bullet.rs damage_factor] and the guards'/clones' hit chance by Accuracy Normal [M
/// DroneWeap_DoBulletAccuracy 0x179740:74-88]; Drone_Create 0x136be0:49's min-difficulty gate
/// spawns the param-2 placements. Real input only; no health/ammo/channel writes.
///
/// IGNORED with the measured round (real-health-agent-16-1b-2, NF_DMG_TRACE on this tree):
/// the 71.4 of the 100 the d2 ledger loses is the SOUTH GALLERY — four AR guards
/// (0x500008c/0x50000b2/0x50000b1, 10-25 hp) patrolling z −102..−112 at y 6.7 (#1 at y 0.8) —
/// whose first landed round is tick 455 at the ramp foot (their fields never reach the spawn
/// legs), above the plain shooters window's dy 6.0 so the ramp-foot fights trade with the
/// close ground pair while the gallery plinks free. TRIED this round: (a) a load-time (48, 18)
/// shooters window — the fights engaged the gallery across the pit from the lane east (one
/// fight, dead at 1271; the wide pool breaks the early geometry); (b) the window gated from
/// the ring's east end (y 5.9, dy 0.8) — the fights DID take the gallery 1v1 and killed
/// #5/#1/#3/#1 on the ring, but the ring trades cost 44 and Bond entered the chaser fight at
/// 7.6 (dead 8435); (c) zero-movement shoot_point stands on the gallery from the spawn — every
/// guard fails the eye-ray visible() gate from there (the ring structure blocks; the stands
/// all skipped, the timeline byte-identical); (d) static-spot stands at the ring east — Bond
/// died AT the stands (2189) under the crossfire. NEXT LEVERS: a cover-shaped approach (the
/// fights' own flank machinery on the gallery from the ring, one guard at a time, with the
/// ring's south rim as the break), or an armour pickup before the ramp foot (none found on
/// the chain set), or a live PCSX2 scout of whether the gallery is reachable on foot
/// (the station's west side?) — the player answer this route is missing.
#[test]
/// GREEN 2026-11-13: the gallery trio (#3/#4/#5, 0x1a) sees the whole ramp; the walk dashes to the
/// x 19-20 / z -92..-98 pocket (no line to any of them), peek-fights them from its edge and backs
/// out the way it came (end health 49.2, armour 41.3).
fn walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent() {
    walker::with_real_damage_head_hunting(|| walk_the_chasers_defeat_completes_the_mission_on_real_damage_at(2));
}

/// The 00 Agent sister (real-health-00agent-16-1b-41-4a): the same walked chaser mission on
/// real damage, loaded at difficulty 3 the menu way — pain Plr_DMod_Hard 1.0 (the 0x14..=0x16
/// group [M Player_HandlePain 0x1902c8; bullet.rs damage_factor; 0x1d3a80 case 0x7000014-16:
/// Normal 0.5, Hard 0x3f800000]) and the guards' hit chance by Accuracy Hard [M
/// DroneWeap_DoBulletAccuracy 0x179740:74-88]. The d2 gates (the pit-door guard pre-shoot) fire
/// at 3 too. Real input only; no health/ammo/channel writes.
#[test]
fn walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent() {
    walker::with_real_damage_head_hunting(|| walk_the_chasers_defeat_completes_the_mission_on_real_damage_at(3));
}
