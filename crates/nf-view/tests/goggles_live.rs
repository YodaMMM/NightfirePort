//! goggles-nightsight-xray: the night-sight/x-ray goggles flow through the REAL input path
//! (`Sim::step_bond` with `Input::goggles_action` — the game's `Input_Action(p, 0xd, 4)`
//! [M HUD_MonitorNightSight 0x19e9d8:74]). Evidence map: docs/systems/goggles.md.
//!
//! The disc grants the goggles NOWHERE (docs/systems/goggles.md census: no loadout in
//! `Player_InitWeapon` 0x1a7ef8's per-level switch, no 0xF0 pickup with id 0x5d/0x5e in any
//! FILES.BIN archive [D]) — the only grantor is the all-weapons cheat handler
//! `C_CHCHWEAP_Handler` 0x215dc8 (0x4a..0x5e, 999 rounds) [M]. The tests seed the inventory
//! the way that grant leaves it (owned 0x5d), then drive the key through `step_bond`.
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// A loaded 07000005 sim with the goggles granted (the all-weapons cheat's state), EQUIPPED
/// through the real request path (a goggles-key press → the switch machine draws 0x5d → the
/// arm forces mode 1 → the arm's restore auto-stows the item), waited out to the settled
/// state: item stowed, mode 1, latch set.
fn granted_scene(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000005.bin", false).expect("load 07000005");
    {
        let sim = scene.sim.as_mut().unwrap();
        // The cheat grant [M 0x215dc8]: Player_EquipWeapon(0x5d, 999) → owned + full clip. The
        // port's inventory seam holds owned/clip per weapon id (pickup.rs Inventory).
        sim.drones.inventory.owned[0x5d] = true;
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        // The goggles key press: with the latch clear it requests the item [M 0x19e9d8:56-58].
        let mut press = Input::default();
        press.goggles_action = true;
        sim.step_bond(press, false, false, false);
        assert_eq!(sim.gun.switch_request, 0x5d, "the press requested the goggles item [M :56-58]");
        // Wait out the draw + the arm + the arm's auto-stow (the restore clamps the pre-goggles
        // weapon 2 back into the request slot [M :56-66]; the goggles' own holster script is 0
        // [D weapon_data 0x5d] so the stow is the instant path + weapon 2's draw).
        for t in 0..600 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.gun.weapon == 2 && sim.gun.switch_state == 0 { eprintln!("[stow done t{t}]"); break; }
        }
        sim.step_bond(Input::default(), false, false, false);
    }
    scene
}

#[test]
fn the_goggles_key_cycles_the_mode_through_the_real_input_path() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = granted_scene(&iso);
    let sim = scene.sim.as_mut().unwrap();
    // The arm path armed night sight and latched [M 0x19e9d8:49-72], the item auto-stowed,
    // and the mode persisted through the stow [M: nothing in the switch path writes +0x236].
    assert_eq!(sim.gun.weapon, 2, "the item settled back on the pre-goggles weapon");
    assert_eq!(sim.goggle_mode(), 1, "the arm set mode 1 and it persisted");
    assert!(sim.hud.goggles.was_goggles, "the arm path latched BL+0x961");

    // The key cycles 1 → 2 → 0 [M 0x19e9d8:86-152: mode 1 → 2, mode 2 → 0; the +0x60 aim bit
    // is clear (no aim input)].
    let mut press = Input::default();
    press.goggles_action = true;
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.goggle_mode(), 2, "the key's edge cycles night sight → x-ray");
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.goggle_mode(), 0, "the key's edge cycles x-ray → off");
    // Off again: the next edge re-enters night sight (0 → 1) [M :89-100].
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.goggle_mode(), 1, "the key's edge cycles off → night sight");
}

#[test]
fn the_battery_drains_while_a_mode_is_up_and_drops_the_mode_at_zero() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = granted_scene(&iso);
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.goggle_mode(), 1);
    let b0 = sim.goggle_battery();
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    let b1 = sim.goggle_battery();
    assert!((b1 - (b0 - 10.0)).abs() < 1e-3, "mode 1 drains −FRAME_RATE_MUL/tick [M 0x19e9d8:159-161], {b0} → {b1}");

    // x-ray drains too [M the same tail reads the mode ≠ 0].
    let mut press = Input::default();
    press.goggles_action = true;
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.goggle_mode(), 2);
    let b2 = sim.goggle_battery();
    for _ in 0..5 { sim.step_bond(Input::default(), false, false, false); }
    assert!((sim.goggle_battery() - (b2 - 5.0)).abs() < 1e-3, "mode 2 drains too");

    // The recharge: mode off recharges +3/tick [M :155-158, the EE MIN.S cap 1800 at 0x19ec68].
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.goggle_mode(), 0);
    let b3 = sim.goggle_battery();
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    assert!((sim.goggle_battery() - (b3 + 30.0).min(1800.0)).abs() < 1e-3, "+3/tick while off, capped 1800");

    // At 0 the mode drops [M :163-193: battery ≤ 0 with mode 1/2 → battery = 0, mode = 0,
    // both pane enables cleared]. Re-arm first, then seed near-empty.
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.goggle_mode(), 1, "re-armed for the drop test");
    sim.hud.goggles.fade = 2.0; // seed near-empty (the game's BL+0x920 value, poked for the test)
    // Tick one at a time: the drain reaches 0 (mode 1: the press tick took 2.0 → 1.0, then
    // 1.0 → 0.0) and the drop zeroes it — assert on THAT tick, before the mode-0 recharge
    // (+3/tick) the next ticks run [M :163-193].
    let mut dropped = false;
    for _ in 0..5 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.goggle_mode() == 0 { dropped = true; break; }
    }
    assert!(dropped, "the mode drops when the battery hits 0");
    assert_eq!(sim.goggle_battery(), 0.0, "the battery is zeroed [M :172]");
    assert!(!sim.hud.goggles.night_pane_on && !sim.hud.goggles.xray_pane_on, "both panes cleared [M :174-192]");
}

#[test]
fn a_level_without_the_goggles_never_arms_from_the_request_alone() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    // The disc census [D]: no loadout and no pickup grants 0x5d — the level-start inventory
    // does not own it (the [L]-pinned 07000005 start: weapons 1, 2, 0x4a, 0x4e, 0x50, 0x54).
    assert!(!sim.drones.inventory.owned.get(0x5d as usize).copied().unwrap_or(true), "the level start owns no goggles");
    assert_eq!(sim.goggle_mode(), 0, "fresh level: mode 0");
    assert!(!sim.hud.goggles.was_goggles, "the latch is clear [M: only the arm path sets BL+0x961]");

    // With the latch clear the key's edge only REQUESTS the item [M 0x19e9d8:56-58, disasm
    // 0x19eb40-0x19eb50: weapon+0x63 = 0x5d] — it does not touch the mode or the latch.
    let mut press = Input::default();
    press.goggles_action = true;
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.gun.switch_request, 0x5d, "the request seam wrote +0x63 = 0x5d [M :56-58]");
    assert_eq!(sim.goggle_mode(), 0, "the request alone never arms the mode [M]");
    assert!(!sim.hud.goggles.was_goggles, "the latch is still clear [M]");
}

#[test]
fn the_arm_path_auto_stows_the_item_and_the_mode_persists() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = granted_scene(&iso);
    let sim = scene.sim.as_mut().unwrap();
    // The arm fired (mode 1, latch set) and its +0x63 restore [M 0x19e9d8:56-66] stowed the
    // item back to the pre-goggles weapon (2 — not a clamped special, so verbatim).
    assert_eq!(sim.gun.weapon, 2, "the goggles item auto-stowed to the pre-goggles weapon");
    assert_eq!(sim.goggle_mode(), 1, "the mode persists after the stow [M: nothing in the switch path writes +0x236]");
    assert!(sim.hud.goggles.was_goggles, "the latch survives [M: only Player_StandAtNewPosition 0x190d88:87 clears it]");
    // And with the latch set, the key now CYCLES instead of re-requesting [M :82-84].
    let mut press = Input::default();
    press.goggles_action = true;
    sim.step_bond(press, false, false, false);
    assert_eq!(sim.goggle_mode(), 2, "latch set: the key cycles to x-ray");
}
