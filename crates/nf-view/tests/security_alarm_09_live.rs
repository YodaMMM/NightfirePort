//! security-room-alarm-09 — the 07000009 security guards raise the alarm on Bond through the
//! REAL path (`nf_view::load` + `Sim::step_bond`), and the whole level's fail-on-sight stays
//! silent when nobody sees him.
//!
//! The game's chain (docs/systems/security-room-alarm-09.md, every step [M]-addressed there):
//! CivilianPatrol 0x15d610 case 3 — NearArmedPerson 0x1787c8 (prop 0x3c + the sight flag; the
//! radius arg is dead) → `+0x4F8 |= 0x8000000` → SetState(0x56 Attack); Attack ENTER 0x164210 →
//! FirstAttack 0x148010 → FirstSightState 0x147c48 → prop 0x1A (DoModeSettingsNEW sets it with
//! 0x56 on 09/0A) → 0x3E CivilianChallenge 0x162558 (anim 0x3E + the 0xC3+Rand(9) bark, follow
//! 0x89 RunToAlarm); RunToAlarm 0x1736f8 → FindAlarmPoint 0x153008 → PressAlarm 0x168ea8 →
//! SetMissionFailReason(0x10) 0x13e560 case 0x10 → `switch_channels[nearest switch] = 1` — the
//! security-room switch #? at (−23.06, −368.05, 37.20) on ch 0x7D [D] → OR mux #278 → ch 0x61
//! [D] → the monitor's fail hook [M 0x1827d8] → FailLabel 0x4000032 "Alarm Raised" + ch 99 →
//! objective 1 failed.
//!
//! The guards patrol from the load; the panel-side guard (drone 0) walks the lobby and enters the
//! security room ~t2400, where the sight latch fires. The old port froze him in CivilianHiding
//! (the CIVILIAN_SCARED reroute) — no alarm, no fail.

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The walk window. The alarm's tick depends on the patrol phase (the game's own timing depends
/// on where the guard is when Bond appears — the live run saw it "within a few seconds" after the
/// guards had patrolled; a cold load needs the walk to the room first) — the bound is the test's
/// window, not a pinned game number. The pinned facts are the chain and the channels.
const WINDOW: u32 = 4500;

/// Bond at the elevator panel (the playtest spot: the game pos (−18.1, −367.4, 26.1), the port's
/// panel at (−18.08, −368.23, 25.02) [D placements], the port run's `--bond -18.3,-368.4,27.0`).
const PANEL: [f32; 3] = [-18.3, -368.4, 27.0];

#[test]
fn the_panel_guard_raises_the_alarm_and_fails_the_mission() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009").sim.expect("sim");
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    sim.bond.pos = PANEL;
    sim.bond.yaw = 3.1416;
    sim.bond.pitch = -0.25;

    let (mut seen, mut challenge, mut press) = (None, None, None);
    let mut alarm = None;
    for t in 0..WINDOW {
        sim.step_bond(Input::default(), false, false, false);
        for (i, d) in sim.drones.list.iter().enumerate() {
            if seen.is_none() && d.ever_seen { seen = Some((t, i)); }
            if challenge.is_none() && d.state == state::CIVILIAN_CHALLENGE { challenge = Some((t, i)); }
            if press.is_none() && d.state == crate_kiko_press() { press = Some((t, i)); }
        }
        if sim.drones.channels.on(0x61) {
            if alarm.is_none() { alarm = Some(t); }
            if t >= alarm.unwrap() + 12 { break; } // let the monitor's next pass raise ch 99 [M 0x1827d8]
        }
    }
    let (s_at, s_i) = seen.expect("a guard latches seen on the panel Bond [M ProcessDroneSight 0x176a28]");
    let (c_at, c_i) = challenge.expect("the guarding drone passes CivilianChallenge 0x3E [M 0x147c48 prop 0x1a + 0x162558]");
    assert_eq!(c_i, s_i, "the same guard challenges [M 0x15d610 case 3 → 0x164210 → 0x148010]");
    assert!(c_at >= s_at, "the challenge follows the sight latch, {c_at} >= {s_at}");
    let (p_at, p_i) = press.expect("the guard reaches PressAlarm 0x8A [M 0x168ea8]");
    assert_eq!(p_i, c_i, "the same guard presses");
    assert!(p_at > c_at, "the press follows the challenge anim (follow 0x89 RunToAlarm [M 0x162558 case 1])");
    let a_at = alarm.expect("the alarm raises within the window");
    assert!(a_at >= p_at, "ch 0x61 fires at/after the press [M 0x13e560 case 0x10]");
    assert!(sim.drones.channels.on(0x7d), "the security-room switch's ch 0x7D rose [D the (−23.06, −368.05, 37.20) switch; M 0x13e560 case 0x10]");
    assert!(sim.drones.channels.on(0x61), "OR mux #278: ch 0x7D → the alarm ch 0x61 [D]");
    assert!(sim.drones.channels.on(99), "the mission-fail channel ch 99 [M 0x1827d8]");
    assert_eq!(sim.drones.mission.states[8][0], 5, "objective 1 (ch 0x61, flags 2) FAILED [M mission.rs]");
    assert_eq!(sim.hud.fail_label, 0x0400_0032, "FailLabel = Alarm Raised [M 0x1827d8:100]");
}

fn crate_kiko_press() -> u8 { nf_game::kiko::PRESS_ALARM }

/// Sister: the same panel spot with the patrol held (`Drones::hold_npcs`, the dev hold the
/// renderer's `--hold-npcs` sets [M lib.rs:5511]) — every guard stays at his spawn, the closest
/// 19.9 units from the panel (drone 0's placement (1.42, −368.43, 22.28) [D], sight range 12.0
/// [D props]), so the sight latch can never fire and no alarm channel may rise. This isolates
/// the detection: the alarm needs a guard within his sight of Bond, not merely the level's
/// machinery. (A static Bond at the car-park START is NOT such a spot — the car-park patroller
/// walks within his 12-unit sight of the spawn and runs the same challenge chain in the port;
/// whether the game alarms on a player who stands at the spawn for a minute is unverified — a
/// live check in docs/systems/security-room-alarm-09.md.)
#[test]
fn bond_outside_their_sight_keeps_the_alarm_off() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009").sim.expect("sim");
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    sim.drones.hold_npcs = true;
    sim.bond.pos = PANEL;
    sim.bond.yaw = 3.1416;
    sim.bond.pitch = -0.25;
    let spawns: Vec<[f32; 3]> = sim.drones.list.iter().map(|d| d.pos).collect();
    for _ in 0..WINDOW {
        sim.step_bond(Input::default(), false, false, false);
        for (i, d) in sim.drones.list.iter().enumerate() {
            assert!(!d.ever_seen, "guard {i} latched seen from his spawn {:?} — beyond sight 12.0 [D props]", spawns[i]);
            assert!(!sim.drones.channels.on(0x7c) && !sim.drones.channels.on(0x7d) && !sim.drones.channels.on(0x61) && !sim.drones.channels.on(99),
                "no alarm channel fired without a sighting");
        }
    }
    for (i, d) in sim.drones.list.iter().enumerate() {
        assert!((d.pos[0] - spawns[i][0]).abs() < 0.01 && (d.pos[2] - spawns[i][2]).abs() < 0.01,
            "the hold keeps guard {i} at his spawn");
    }
    assert_eq!(sim.drones.mission.states[8][0], 2, "objective 1 still fail-pending, never failed [D flags 2]");
}
