//! objectives-0b-start — the 0700000b start's objective sequence, driven through the REAL path
//! (`nf_view::load` — a cold load, no RamSave, the boot level-select shape — + `Sim::step_bond`).
//!
//! The main-agent finding (PCSX2 vs the port at the 0700000b start, 2026-10-17, both fresh
//! loads): PCSX2 announced "Get to the lobby and activate the main elevator system" (0x04000017),
//! "Take main elevator to office floor" (0x04000018) and "Install Q-Worm on office computer
//! systems" (0x04000019 — part 1's record) over the first ~520 frames, while the port showed
//! "OBJECTIVE COMPLETE" then "Install Q-Worm on Mayhew's computer system" (0x0400001c) / "Locate
//! the Secure Terminal Room" (0x0400001d) — the port completed the balcony objective at spawn and
//! revealed only the current part's records.
//!
//! Root cause, all three legs:
//! - [D] the disc ships EVERY objective record STATE 0 (ACTION.ELF `.data`: all 24 MissionData
//!   rows' 55 records carry +0x14 = 0 — mission 07000009's rows @ 0x2a3f78 / 0x2a3fc0 / 0x2a4008
//!   included), so a fresh boot's `.data` holds 0 everywhere;
//! - [M] `Mission_Init` (0x182270) leaves prior parts' records untouched (the mode-1/mode-4 arms
//!   read the count, never write a record [0x182270:92-95, 120-121]) and
//!   `Mission_MonitorObjectives` (0x1827d8) case 0 announces every state-0 record whose
//!   `record.level <= GameState._12_4_` (0x1827d8:41-54) — 0x07000009/0x0700000a ≤ 0x0700000b —
//!   in table order, as plain "%s" texts (0x1827d8:43-45; the "%s: %s" fail format is case 2's
//!   only, 0x1827d8:72-74);
//! - [L] a PCSX2 fresh load shows exactly those announcements.
//!
//! The port's removed `fix-objective-messages` fabricator had marked prior parts announced
//! (3/2) on every no-snapshot load, so nothing announced, the spawn plate's ch 0x4b folded the
//! fabricated balcony record into "OBJECTIVE COMPLETE", and ch 0x4b revealed 0x0400001c/0x0400001d.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The banner ids by first appearance, tick by tick (the queue entries live 300 ticks and the
/// early ones expire — the first-appearance order is the pin, not the live queue).
fn appear_order(scene: &mut nf_view::scene::Scene, ticks: u32) -> Vec<u32> {
    let mut seen: Vec<u32> = Vec::new();
    for _ in 0..ticks {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for m in &sim.hud.objective_msgs {
            if !seen.contains(&m.label) { seen.push(m.label); }
        }
    }
    seen
}

/// The fresh 0700000b start announces the mission's PRIOR parts' objectives first, in the
/// monitor's table order, then the spawn plate's ch-0x4b fold ("OBJECTIVE COMPLETE" for 0700000a's
/// balcony record) and the current part's first two records — the exact PCSX2 sequence.
#[test]
fn the_0700000b_start_announces_the_prior_parts_then_the_current_ones() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b cold");
    // The mission layer starts on the disc's pristine `.data` [D]: every record state 0.
    {
        let sim = scene.sim.as_ref().unwrap();
        for (i, l) in nf_game::mission::LEVELS.iter().enumerate() {
            if l.mission == 0x0700_0009 {
                assert!(sim.drones.mission.states[i].iter().all(|&s| s == 0),
                    "{:08x}'s records start on the disc's state-0 `.data` [D], got {:?}",
                    l.level, sim.drones.mission.states[i]);
            }
        }
    }
    let seen = appear_order(&mut scene, 700);
    eprintln!("[obj-0b] 0700000b cold-start banner ids in first-appearance order: {seen:02x?}");
    // The monitor's walk order [M 0x1827d8:22-97 — rows 8/9/10, records in order], all six prior
    // records announcing at the first monitor tick (case 0: level ≤ current [D]+[M] above):
    let priors = [0x0400_0016, 0x0400_0017, 0x0400_0018, 0x0400_0019, 0x0400_001a, 0x0400_001b];
    for (k, want) in priors.iter().enumerate() {
        assert_eq!(seen.get(k), Some(want),
            "banner {k} of the 0700000b cold start must be {want:08x} (the monitor's table order [M 0x1827d8]), got {:02x?}; the pre-fix fabricator hid the priors and led with OBJECTIVE COMPLETE", seen.get(k));
    }
    // The spawn plate #176 fires ch 0x4b within the first ticks [D placement; the walk_0700000b
    // harness] — 0700000a's balcony record (0x0400001b, condition ch 0x4b) folds → the game's
    // case-3 "OBJECTIVE COMPLETE" [M 0x1827d8:78-85] — and ch 0x4b is the enable channel of the
    // current part's first two records (0x0400001c/0x0400001d, enable 0x4b [L c0700000b]) → their
    // announces queue right after [M 0x1827d8:55-67].
    assert_eq!(seen.get(6), Some(&0x0200_0053),
        "the balcony fold's OBJECTIVE COMPLETE follows the six priors, got {seen:02x?}");
    assert_eq!(seen.get(7), Some(&0x0400_001c), "'Install Q-Worm on Mayhew's computer system' reveals after the fold (enable 0x4b)");
    assert_eq!(seen.get(8), Some(&0x0400_001d), "'Locate the Secure Terminal Room' reveals after the fold (enable 0x4b)");
    // Sister: the level's later records must NOT appear — their enables (0x4d/0x4f/0x59 [L
    // c0700000b]) stay unfired through the idle start.
    for late in [0x0400_001e, 0x0400_001f, 0x0400_0020] {
        assert!(!seen.contains(&late), "{late:08x} must not announce at the idle start");
    }
}

/// The pause OBJECTIVES tab at the fresh start lists the same announced records in table order —
/// the fail-objective first at state 2, the priors announced, the balcony folded complete, the
/// current part's first two rows present after their reveal (`Mission_ObjectiveState` 0x182630's
/// walk: state > 1, prior parts included, the walk stops at the first part beyond ThisOrderNum).
#[test]
fn the_pause_objectives_tab_carries_the_prior_parts_at_the_start() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b cold");
    let _ = appear_order(&mut scene, 700);
    let sim = scene.sim.as_ref().unwrap();
    let rows: Vec<(u32, u32)> = (0..12).filter_map(|v| {
        sim.drones.mission.objective_state(v).map(|s| (s.text, s.state))
    }).collect();
    eprintln!("[obj-0b] pause tab rows (text, state): {rows:02x?}");
    // The walk's head — the six priors in table order, all announced (state > 1):
    let want: [(u32, u32); 6] = [
        (0x0400_0016, 2), // the fail-objective arms at 2 (its ch-0x61 alarm still off)
        (0x0400_0017, 3), (0x0400_0018, 3), (0x0400_0019, 3), (0x0400_001a, 3),
        (0x0400_001b, 4), // the balcony record folded complete on the spawn plate's ch 0x4b
    ];
    for (k, want) in want.iter().enumerate() {
        assert_eq!(rows.get(k), Some(want),
            "pause tab row {k} must be {want:02x?} [M 0x182630's walk over the announced records], got {rows:02x?}");
    }
    // The current part's first two rows close the list (announced on the plate's ch 0x4b).
    assert!(rows.contains(&(0x0400_001c, 3)) && rows.contains(&(0x0400_001d, 3)),
        "the revealed 0x0400001c/0x0400001d rows are on the tab, got {rows:02x?}");
}

/// The case-0 announce binds the PLAIN text — the alarm fail-objective's banner carries no
/// "Fail: " prefix (the "%s: %s" format is case 2's only [M 0x1827d8:43-45 vs 72-74]); its
/// landing state is still the fail-armed 2 [M 0x1827d8:47-51].
#[test]
fn the_fail_objective_announces_plain_text_and_arms_state_2() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b cold");
    let alarm_text = {
        let sim = scene.sim.as_ref().unwrap();
        let lang = sim.lang.as_ref().expect("USATxt.dat loaded");
        lang.bind(0x0400_0016).expect("the alarm objective binds").to_string()
    };
    let mut armed = false;
    let mut plain = false;
    for _ in 0..30 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(m) = sim.hud.objective_msgs.iter().find(|m| m.label == 0x0400_0016) {
            plain |= m.text == alarm_text && !m.failed;
            armed |= sim.drones.mission.states[8][0] == 2;
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(plain, "the alarm banner is the plain bound text {alarm_text:?}, got {:02x?}",
        sim.hud.objective_msgs.iter().filter(|m| m.label == 0x0400_0016).map(|m| m.text.clone()).collect::<Vec<_>>());
    assert!(armed || sim.drones.mission.states[8][0] == 2,
        "the fail-objective lands fail-armed (state 2 [M 0x1827d8:47-51]), got {}", sim.drones.mission.states[8][0]);
}
