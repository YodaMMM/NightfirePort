//! 07000014: the guards the fusebox release (ch 0x87 + ch 0x88 -> 0x8C) sends toward the locked
//! west door must not path through it. The game's A* rejects a link between two door-type nodes
//! while the node's bound door is locked [M AINetwork_DoAStarPath 0x1571b0:262-271 →
//! Door_IsLocked 0x136810], so their combat route to Bond fails and they hold
//! COMBAT_NO_ROUTE 0x5f at the door — no OpenDoor 0x81 [L 2026-10-30 PCSX2 fresh boot, Bond
//! held in the generator hall: CUR 0x5f / PREV 0x5a at the door, never 0x81].
//! The route ends: mission routes arrive at 0.5 [M NDrone2_InitAIPath 0x153720:113; L the live
//! route+0x8C = 0.500].
//!
//! The game's hold is TOTAL — 17 live samples at 30-frame spacing, the state never leaves 0x5f —
//! while the port's guard cycles 0x5f → STEP_AIM_LEFT/RIGHT 0x73/0x74. The game's
//! `NDrone2_DSTATE_CombatNoRoute` case 3 keeps the state exactly on ChooseCombatMove picks 0 /
//! 0x67 / 0x6f and SetStates anything else [M 0x164c08:36-62]; the port's handler mirrors that
//! keep-set. The game never PICKS the step-aims here because every Can* probe of
//! `NDrone2_ChooseCombatMove` gates on the +0x31C side bits (`CanStepLeft` 0x14f640:50 `& 8`,
//! the Right probe `& 0x10`) — and those bits are set EVERY tick by `DroneMove_NoBunching`
//! 0x150ad0 (called from ControlSTANDARD 0x1490f0:202-226, which also zeroes +0x31C and stamps
//! the scan's return byte into drone+0x17 = the `cVar1` of ChooseCombatMove 0x150c18:58): the
//! two guards stand ~2.4 apart at the door [L: (-51.6,41.2) / (-51.2,38.8)], each reads the
//! other as the +0x314 neighbor, and the 8/0x10 pair lands on BOTH — every dodge/strafe/step
//! probe refuses, every pick cascades to 0x67 (Rand(4) ≠ 0) or 0, and 0x5f holds.
//! The port's NoBunching mirror is HELD INERT (`NB_SCAN_LIVE = false` in drone.rs — the
//! documented 07000001 escort-walk blocker), so the port's side bits stay 0, the probes pass,
//! and the guard cycles. Proven in the port: flipping only that const still cycles, because the
//! first guard's step-aim wandering separates the pair past the 3.0 scan radius before the
//! second guard arrives — the cycle and the missing scan feed each other from the first tick.
//! The 0x5f hold therefore cannot be pinned without landing the scan (and re-pinning the 01
//! escort fight it regresses); the ignored test below names exactly that blocker.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const OPEN_DOOR: u8 = 0x81;
const COMBAT_NO_ROUTE: u8 = 0x5f;

#[test]
fn released_guards_do_not_work_the_locked_west_door() {
    let mut scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let door = scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| d.ch_lock == 0x85)
        .expect("the west door with lock ch 0x85 [D placements 151/152]");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x87, 1, 1);
        sim.drones.channels.set(0x88, 1, 1);
    }
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    let mut at_door_max = 0usize;
    let mut came = false;
    let mut seen_states: Vec<u8> = Vec::new();
    for t in 0..3000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = hall;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.is_locked(door, &sim.drones.channels), "the door stays locked (no cascade fired)");
        let near: Vec<&nf_game::drone::Drone> = sim.drones.list.iter()
            .filter(|d| d.health > 0.0 && (d.pos[0] + 51.5).powi(2) + (d.pos[2] - 40.0).powi(2) < 9.0).collect();
        at_door_max = at_door_max.max(near.len());
        came |= !near.is_empty();
        for d in &near {
            assert_ne!(d.state, OPEN_DOOR, "t{t}: a released guard works the LOCKED door at ({:.1},{:.1}) — the A* door gate [M 0x1571b0:262-271]", d.pos[0], d.pos[2]);
            if !seen_states.contains(&d.state) { seen_states.push(d.state); }
        }
    }
    assert!(came, "the released guards reach the door side [L: two guards stop at (-51.6,41.2) / (-51.2,38.8)]");
    eprintln!("[014] guards near the door (max at once): {at_door_max}; states sampled at the door: {:?}",
        seen_states.iter().map(|s| format!("{s:#04x}")).collect::<Vec<_>>());
}

/// The game's hold, pinned only when the port can run the same scan. The game's door guards sit
/// in COMBAT_NO_ROUTE 0x5f at EVERY sample (17 live samples, 30-frame spacing, CUR 0x5f / PREV
/// 0x5a, opponent set, grace 0 [L 2026-10-30]); the mechanism is the +0x31C side bits the
/// NoBunching scan stamps per tick [M 0x150ad0:42-55 via 0x1490f0:202-226], which the port's
/// mirror does not run (`NB_SCAN_LIVE = false` — the 07000001 escort-walk blocker quoted at the
/// const). Un-ignore when that scan lands and its 01 regression is re-pinned; the assert below
/// is the game's behaviour and must not be weakened.
#[test]
#[ignore = "the port's NoBunching mirror is inert (NB_SCAN_LIVE) — the game's +0x31C side bits never set, so the guard cycles 0x5f→0x73/0x74 instead of holding; landing the scan regresses walkreal_07000001_live's escort fight (the documented blocker at the const) and needs that re-pin first [L 2026-10-30: the game holds 0x5f over 17 samples]"]
fn the_door_guards_hold_combat_no_route_like_the_game() {
    let mut scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x87, 1, 1);
        sim.drones.channels.set(0x88, 1, 1);
    }
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    let mut samples = 0usize;
    for t in 0..3000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = hall;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let near: Vec<&nf_game::drone::Drone> = sim.drones.list.iter()
            .filter(|d| d.health > 0.0 && (d.pos[0] + 51.5).powi(2) + (d.pos[2] - 40.0).powi(2) < 9.0).collect();
        if near.is_empty() { continue; }
        if t % 30 == 0 {
            samples += 1;
            for d in &near {
                assert_eq!(d.state, COMBAT_NO_ROUTE,
                    "t{t}: the door guard holds COMBAT_NO_ROUTE 0x5f at every sample in the game (17 live samples, CUR 0x5f / PREV 0x5a [L 2026-10-30]); the port's guard is in {:#04x} at ({:.1},{:.1}) — the NoBunching side bits [M 0x150ad0:42-55] are not stamping while NB_SCAN_LIVE is inert",
                    d.state, d.pos[0], d.pos[2]);
            }
        }
    }
    assert!(samples >= 17, "the sampling window must cover the live window's span (17 samples at 30 frames), got {samples}");
}

/// Sister: the gate is the LOCK, not the door. With the lock channel on (the cascade's ch 0x85),
/// the same door is no route block (the A* expands through it).
#[test]
fn the_unlocked_door_is_no_route_block() {
    let scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_ref().unwrap();
    let door = sim.drones.doors.list.iter().position(|d| d.ch_lock == 0x85).unwrap();
    let mut ch = sim.drones.channels.clone();
    assert!(sim.drones.doors.is_locked(door, &ch), "locked at load");
    ch.set(0x85, 1, 1);
    assert!(!sim.drones.doors.is_locked(door, &ch), "ch 0x85 unlocks it [M Door_IsLocked 0x136810:10-16]");
}
