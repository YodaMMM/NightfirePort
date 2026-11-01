//! objective-opening — the first objective texts every on-foot level opens with, on a direct
//! load (no carried mission snapshot: `nf_view::load`, the `nf-view <iso> <level>` / headless
//! `--shot` shape) and on the real in-mission hand-off (`warm_exit_save` of the previous part →
//! `load_with_carry`, the app's campaign arm).
//!
//! The fault this pins: a direct load of 07000006/07/08 opened with 05's "Breach the castle
//! walls" (0x04000000) and 0b with 09's "Don't let the security guards raise the alarm"
//! (0x04000016): every record started on the disc's state 0 [D ACTION.ELF .data], so
//! `Mission_MonitorObjectives` case 0 (`record.level <= GameState._12_4_` [M 0x1827d8:41-54])
//! re-announced the earlier parts. The game keeps the records in `.data` across every in-boot
//! load and `Mission_Init` never writes a prior part's record [M 0x182270:92-95, 120-121]; a
//! player only reaches part N through part N-1, so those records are already announced
//! ([L] the 10-17 PCSX2 start sweep's in-boot goto_level loads: 07 opened with its own
//! 0x04000002 only). `Mission::seed_campaign_arrival` gives the no-carry load that shape.

use nf_game::mission::LEVELS;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The level's archive, or None when the disc has none (0x0700000e/0f are not FILES.BIN levels).
fn load(iso: &str, level: u32, save: Option<&nf_game::player::RamSave>) -> Option<nf_view::scene::Scene> {
    match nf_view::load_with_carry(iso, &format!("{level:08x}.bin"), false, save) {
        Ok(s) => Some(s),
        Err(e) if e.to_string().contains("in FILES.BIN") => None,
        Err(e) => panic!("load {level:08x}: {e:#}"),
    }
}

const COMPLETE: u32 = 0x0200_0053;
const TICKS: u32 = 400;

/// (label, text) of every objective notice, by first appearance — the OBJECTIVE COMPLETE fold
/// notices are left out (they carry no objective text).
fn openings(scene: &mut nf_view::scene::Scene, ticks: u32) -> Vec<(u32, String)> {
    let mut seen: Vec<(u32, String)> = Vec::new();
    for _ in 0..ticks {
        let sim = scene.sim.as_mut().unwrap();
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for m in &sim.hud.objective_msgs {
            if m.label != COMPLETE && !seen.iter().any(|(l, _)| *l == m.label) { seen.push((m.label, m.text.clone())); }
        }
    }
    seen
}

/// Every objective text label of `level`'s mission whose part is BEFORE `level`'s part.
fn earlier_labels(level: u32) -> Vec<u32> {
    let me = LEVELS.iter().find(|l| l.level == level).unwrap();
    LEVELS.iter().filter(|l| l.mission == me.mission && l.part < me.part)
        .flat_map(|l| l.objectives.iter().map(|o| o.text)).collect()
}

fn own_labels(level: u32) -> Vec<u32> {
    LEVELS.iter().find(|l| l.level == level).unwrap().objectives.iter().map(|o| o.text).collect()
}

/// The opening (first new-objective notice) of each on-foot level on a direct load, as measured
/// on this build; `None` = no objective notice in the first 400 ticks (the level's records wait
/// on enable channels its start does not raise without the previous part's carried channels).
const DIRECT: &[(u32, Option<u32>)] = &[
    (0x0700_0005, Some(0x0400_0000)), // "Breach the castle walls" (part 0 keeps its own)
    (0x0700_0006, Some(0x0400_0001)), // "Find a way inside to the party"
    (0x0700_0007, Some(0x0400_0002)), // "Rendezvous with undercover agents and maintain cover as party guest"
    (0x0700_0008, None),              // 0x04000004 waits on enable ch 0x49 (07's exit raises it)
    (0x0700_0001, Some(0x0400_0006)), // "Escort Mayhew to bunker"
    (0x0700_0002, Some(0x0400_0007)), // "Destroy Mayhew's computer"
    (0x0700_0003, Some(0x0400_000a)), // "Retrieve Dragon Safe contents"
    (0x0700_0004, Some(0x0400_000d)), // "Defeat the assassin"
    (0x0700_0009, Some(0x0400_0016)), // "Don't let the security guards raise the alarm" (part 0)
    (0x0700_000a, Some(0x0400_0019)), // "Install Q-Worm on office computer systems"
    (0x0700_000b, Some(0x0400_001c)), // "Install Q-Worm on Mayhew's computer system"
    (0x0700_000c, Some(0x0400_002c)), // "Investigate building ST- 1 "
    (0x0700_000d, Some(0x0400_002f)), // "Investigate building ST- 3 "
    (0x0700_0011, Some(0x0400_0035)), // "Escape from the tower"
    (0x0700_0012, Some(0x0400_0037)), // "Activate elevator override control in security center"
    (0x0700_004a, Some(0x0400_0039)), // "Gain access to lobby"
    (0x0700_0013, Some(0x0400_003a)), // "Raise security gate and escape through front entrance"
    (0x0700_0014, Some(0x0400_0049)), // "Infiltrate Drake's facility undetected"
    (0x0700_0015, Some(0x0400_004d)), // "Find the exit to Omega Sector"
    (0x0700_0016, Some(0x0400_004e)), // "Reach Drake's launch silo"
    (0x0700_001b, Some(0x0400_0059)), // "Disable the missiles before they are launched"
];

#[test]
fn every_level_opens_with_its_own_objective_on_a_direct_load() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut got: Vec<(u32, Option<(u32, String)>)> = Vec::new();
    let mut bad = Vec::new();
    for l in LEVELS.iter() {
        let Some(mut scene) = load(&iso, l.level, None) else { eprintln!("[open direct] {:08x}: no archive on the disc (not an ACTION level)", l.level); continue };
        let seen = openings(&mut scene, TICKS);
        eprintln!("[open direct] {:08x}: {:02x?}", l.level, seen.iter().map(|(l, t)| (*l, t.as_str())).collect::<Vec<_>>());
        let early = earlier_labels(l.level);
        if let Some((lab, t)) = seen.iter().find(|(lab, _)| early.contains(lab)) {
            bad.push(format!("{:08x} re-announced an earlier part's {lab:08x} {t:?}", l.level));
        }
        got.push((l.level, seen.first().cloned()));
    }
    assert!(bad.is_empty(), "earlier parts' objectives re-announced on a direct load: {bad:#?}");
    for (lvl, want) in DIRECT {
        let first = got.iter().find(|(l, _)| l == lvl).expect("level measured").1.clone();
        assert_eq!(first.as_ref().map(|f| f.0), *want, "{lvl:08x} opening objective (got {first:?})");
        if let Some((lab, _)) = first {
            assert!(own_labels(*lvl).contains(&lab), "{lvl:08x}'s opening {lab:08x} is one of its own records");
        }
    }
    // The reported three by text (the bound USATxt strings).
    let text = |lvl: u32| got.iter().find(|(l, _)| *l == lvl).and_then(|(_, f)| f.clone()).map(|f| f.1);
    assert_eq!(text(0x0700_0006).as_deref(), Some("Find a way inside to the party"));
    assert!(text(0x0700_0007).is_some_and(|t| t.starts_with("Rendezvous with undercover agents")));
    assert_ne!(text(0x0700_0006), Some("Breach the castle walls".to_string()));
    // Sister: two levels give different text (the comparison can fail).
    assert_ne!(text(0x0700_0006), text(0x0700_0007), "06 and 07 open with different objectives");
    assert_eq!(text(0x0700_0005).as_deref(), Some("Breach the castle walls"), "05 (part 0) keeps its own");
}

/// The in-mission hand-offs: part N-1's warm exit save (mission snapshot + channels, the app's
/// `warm_exit_save`) into part N. No earlier part's objective may re-announce, and the opening
/// matches the direct load's own-part shape.
#[test]
fn every_in_mission_hand_off_opens_without_earlier_objectives() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut bad = Vec::new();
    let mut firsts: Vec<(u32, Option<u32>)> = Vec::new();
    for l in LEVELS.iter().filter(|l| l.part > 0) {
        let prev = LEVELS.iter().find(|p| p.mission == l.mission && p.part + 1 == l.part).expect("previous part");
        let Some(mut a) = load(&iso, prev.level, None) else { continue };
        let _ = openings(&mut a, 120);
        let save = nf_view::warm_exit_save(a.sim.as_ref().unwrap());
        drop(a);
        let Some(mut b) = load(&iso, l.level, Some(&save)) else { continue };
        let seen = openings(&mut b, TICKS);
        eprintln!("[open hand-off] {:08x} -> {:08x}: {:02x?}", prev.level, l.level, seen.iter().map(|(l, t)| (*l, t.as_str())).collect::<Vec<_>>());
        let early = earlier_labels(l.level);
        if let Some((lab, t)) = seen.iter().find(|(lab, _)| early.contains(lab)) {
            bad.push(format!("{:08x} -> {:08x} re-announced {lab:08x} {t:?}", prev.level, l.level));
        }
        firsts.push((l.level, seen.first().map(|f| f.0)));
    }
    assert!(bad.is_empty(), "earlier parts' objectives re-announced on a hand-off: {bad:#?}");
    // 05 -> 06, the reported path: 06 opens with its own "Find a way inside to the party".
    assert_eq!(firsts.iter().find(|f| f.0 == 0x0700_0006).unwrap().1, Some(0x0400_0001));
    assert_eq!(firsts.iter().find(|f| f.0 == 0x0700_0007).unwrap().1, Some(0x0400_0002));
    // The measured openings of every hand-off (the same own-part texts as the direct loads).
    let want: &[(u32, Option<u32>)] = &[
        (0x0700_0006, Some(0x0400_0001)), (0x0700_0007, Some(0x0400_0002)), (0x0700_0008, None),
        (0x0700_0002, Some(0x0400_0007)), (0x0700_0003, Some(0x0400_000a)), (0x0700_0004, Some(0x0400_000d)),
        (0x0700_000a, Some(0x0400_0019)), (0x0700_000b, Some(0x0400_001c)), (0x0700_000d, Some(0x0400_002f)),
        (0x0700_0012, Some(0x0400_0037)), (0x0700_004a, Some(0x0400_0039)), (0x0700_0013, Some(0x0400_003a)),
        (0x0700_0015, Some(0x0400_004d)), (0x0700_0016, Some(0x0400_004e)),
    ];
    for (lvl, w) in want {
        assert_eq!(firsts.iter().find(|f| f.0 == *lvl).map(|f| f.1), Some(*w), "{lvl:08x} hand-off opening");
    }
    for (lvl, first) in &firsts {
        if let Some(lab) = first { assert!(own_labels(*lvl).contains(lab), "{lvl:08x} hand-off opening {lab:08x} is its own"); }
    }
}

/// The seed itself: a no-carry 07000007 load leaves 05's record complete, 06's announced (it is
/// the previous part) and 07's own records armed on their enables — and the pristine opt-out
/// is documented in `Mission::seed_campaign_arrival` (the env var is read there).
#[test]
fn direct_07_seeds_the_earlier_parts_states() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07");
    let _ = openings(&mut scene, 2);
    let m = &scene.sim.as_ref().unwrap().drones.mission;
    assert_eq!(m.states[0], vec![4], "05's record two parts back sits complete");
    assert_eq!(m.states[1], vec![3], "06's record (the previous part) sits announced");
    // Sister: a carried snapshot wins over the seed (the hand-off path keeps its own states).
    assert_ne!(m.states[0], vec![0], "not the pristine state-0 record that re-announced");
}
