//! objectives-0b-start — the 0700000b start's objective sequence, driven through the REAL path
//! (`nf_view::load` — a load with no carried mission snapshot, the direct-load shape — +
//! `Sim::step_bond`).
//!
//! History. 2026-10-17: a PCSX2 goto_level load of 0700000b announced 0x04000017/18/19 (the
//! earlier parts' records) and the port was changed to start every record on the disc's state 0
//! ([D] ACTION.ELF `.data`), so the monitor's case 0 (`record.level <= GameState._12_4_`
//! [M 0x1827d8:41-54]) announced all six earlier records. That PCSX2 load was an in-boot level
//! change into a mission the boot had not touched; the same sweep's in-boot loads of 06/07/08
//! after 05 opened WITHOUT 05's text ([L] 10-17 start sweep), because `Mission_Init` never writes
//! a prior part's record [M 0x182270:92-95, 120-121] and the records survive in `.data`. A player
//! only reaches 0b through 0a, so the earlier records are always announced on arrival ([L
//! ee_start_0700000a.bin @ 0x2a3f78: 07000009's records 2/3/4 at 0700000a's start]).
//! 2026-10-31 (level-objective-carryover): the no-carry load seeds that arrival shape
//! (`Mission::seed_campaign_arrival` — the previous part announced, older parts complete, the
//! fail-objective armed) and 0b opens with its own records.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The banner ids by first appearance, tick by tick.
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

/// The direct 0700000b start: no earlier record announces; the spawn plate's ch 0x4b folds
/// 0700000a's balcony record (seeded announced — the previous part) into OBJECTIVE COMPLETE
/// [M 0x1827d8:78-85] and reveals the current part's first two records (enable 0x4b [L
/// c0700000b]) [M 0x1827d8:55-67].
#[test]
fn the_0700000b_start_opens_with_its_own_objectives() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b");
    let seen = appear_order(&mut scene, 700);
    eprintln!("[obj-0b] 0700000b direct-start banner ids in first-appearance order: {seen:02x?}");
    assert_eq!(seen, vec![0x0200_0053, 0x0400_001c, 0x0400_001d],
        "the balcony fold, then 'Install Q-Worm on Mayhew's computer system' and 'Locate the Secure Terminal Room'");
    // Sister: none of the six earlier records announce, and the level's later records stay hidden.
    for early in [0x0400_0016, 0x0400_0017, 0x0400_0018, 0x0400_0019, 0x0400_001a, 0x0400_001b] {
        assert!(!seen.contains(&early), "the earlier part's {early:08x} must not re-announce");
    }
    for late in [0x0400_001e, 0x0400_001f, 0x0400_0020] {
        assert!(!seen.contains(&late), "{late:08x} must not announce at the idle start");
    }
}

/// The pause OBJECTIVES tab at the direct start lists the arrival shape in table order
/// (`Mission_ObjectiveState` 0x182630's walk: state > 1, prior parts included).
#[test]
fn the_pause_objectives_tab_carries_the_earlier_parts_at_the_start() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b");
    let _ = appear_order(&mut scene, 700);
    let sim = scene.sim.as_ref().unwrap();
    let rows: Vec<(u32, u32)> = (0..12).filter_map(|v| {
        sim.drones.mission.objective_state(v).map(|s| (s.text, s.state))
    }).collect();
    eprintln!("[obj-0b] pause tab rows (text, state): {rows:02x?}");
    let want: [(u32, u32); 8] = [
        (0x0400_0016, 2),                   // the fail-objective stays armed (its ch-0x61 alarm off)
        (0x0400_0017, 4), (0x0400_0018, 4), // part 0 (two back): complete
        (0x0400_0019, 3), (0x0400_001a, 3), // part 1 (the previous part): announced
        (0x0400_001b, 4),                   // the balcony record folded on the spawn plate's ch 0x4b
        (0x0400_001c, 3), (0x0400_001d, 3), // the current part's two revealed rows
    ];
    assert_eq!(rows, want.to_vec(), "pause tab rows [M 0x182630's walk over the announced records]");
}

/// The case-0 announce binds the PLAIN text — the alarm fail-objective's banner carries no
/// "Fail: " prefix (the "%s: %s" format is case 2's only [M 0x1827d8:43-45 vs 72-74]); its
/// landing state is the fail-armed 2 [M 0x1827d8:47-51]. Driven on 07000009, where the record
/// is the current part's and announces at the start.
#[test]
fn the_fail_objective_announces_plain_text_and_arms_state_2() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009");
    let alarm_text = {
        let sim = scene.sim.as_ref().unwrap();
        let lang = sim.lang.as_ref().expect("USATxt.dat loaded");
        lang.bind(0x0400_0016).expect("the alarm objective binds").to_string()
    };
    let mut plain = false;
    for _ in 0..30 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(m) = sim.hud.objective_msgs.iter().find(|m| m.label == 0x0400_0016) {
            plain |= m.text == alarm_text && !m.failed;
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(plain, "the alarm banner is the plain bound text {alarm_text:?}");
    assert_eq!(sim.drones.mission.states[8][0], 2, "the fail-objective lands fail-armed (state 2 [M 0x1827d8:47-51])");
}
