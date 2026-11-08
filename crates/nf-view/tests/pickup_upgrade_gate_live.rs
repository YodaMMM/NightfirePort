//! The weapon-10 pickup remap gate, on a real level.
//!
//! The game's rule [M]: `Pickup_Handler` 0x18e650:142-144 runs the pickup id through
//! `Upgrade_Weapon` 0x1a8f30, whose case 10 returns 0xC only when the BYTE `GameState[0x52]` is
//! non-zero (0x1a8f30:18-22). That byte is the game-complete flag: `Menu_SetLevelBonus` 0x1ffa10
//! sets it when 0x0700001B is finished, `Boot_LoadPTPData` 0x1c8560 and
//! `Menu_MapDefaultCodename` 0x1f7fa8 restore it from the profile — nothing else writes it [M].
//! Record 0xC is the stronger record of 10: damage +0x0C 4.0 vs 2.0, burst +0x28 999 vs 1, its
//! own name 0x05000073 [L weapon_data]. The port used the tick count as the gate, so every
//! weapon-10 drop handed Bond record 0xC on a fresh game.
//!
//! 07000008 has 15 guards holding weapon 10 [D placements, param 9]; `DroneWeap_DropWeapon`
//! 0x17a2a0 drops `weapon_data[10]+0x02` = 10 through `Pickup_CreateSimple` [M], which is the
//! pickup made here at Bond's feet; the sim's own pickup walk takes it.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::player::Input;

fn load(level: &str) -> Option<nf_view::scene::Scene> {
    let iso = std::env::var("NF_ISO").ok()?;
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, level, false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 {
        if !sim.disabled { break; }
        sim.drones.scripts.skip_request = true;
        sim.step_bond(Default::default(), false, false, false);
    }
    Some(scene)
}

/// A guard's drop of `id` (`Pickup_CreateSimple(M, id, rounds, 0)`) half a unit above Bond's feet,
/// then the sim ticks until the pickup walk takes it.
fn drop_at_bond(sim: &mut nf_view::scene::Sim, id: u16, rounds: i16) {
    let p = sim.bond.pos;
    let m = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [p[0], p[1] + 0.5, p[2], 1.0]];
    let pk = nf_game::pickup::Pickup::create_simple(&m, id, rounds, nf_game::pickup::kind::WEAPON, &sim.weapons, &sim.models)
        .expect("the level carries the weapon-10 model");
    sim.drones.pickups.push(pk);
    let n = sim.drones.pickups.len() - 1;
    for _ in 0..120 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.pickups[n].gone { break; }
    }
    assert!(sim.drones.pickups[n].gone, "the drop at Bond's feet is taken");
}

#[test]
fn a_weapon_10_drop_gives_weapon_10_until_the_game_is_complete() {
    let Some(mut scene) = load("07000008.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let sim = scene.sim.as_mut().unwrap();
    assert!(sim.drones.list.iter().any(|d| d.weapon == 10), "07000008's guards hold weapon 10 [D]");
    assert!(!sim.drones.progress.game_complete, "a fresh profile has not finished the game");
    assert!(!sim.drones.inventory.owned[10] && !sim.drones.inventory.owned[0xc], "neither record owned at the start");
    // The record pair the gate chooses between differs in play [L weapon_data].
    let f = |w: usize, o: usize| f32::from_le_bytes(sim.weapons.records[w][o..o + 4].try_into().unwrap());
    assert_eq!((f(10, 0x0c), f(0xc, 0x0c)), (2.0, 4.0), "0xC hits twice as hard");

    drop_at_bond(sim, 10, 9);
    assert!(sim.drones.inventory.owned[10], "the drop gives weapon 10 [M 0x1a8f30:18-22: GameState[0x52] = 0]");
    assert!(!sim.drones.inventory.owned[0xc], "not the post-game record 0xC");
    assert_eq!(sim.drones.inventory.clip[10], sim.weapons.clip_size(10), "a full clip in record 10's slot");

    // Sister: with the game-complete byte set the same drop lands on 0xC.
    sim.drones.inventory.owned[10] = false;
    sim.drones.progress.game_complete = true;
    drop_at_bond(sim, 10, 9);
    assert!(sim.drones.inventory.owned[0xc], "game complete: 10 → 0xC [M 0x1a8f30:18-22]");
    assert!(!sim.drones.inventory.owned[10], "and not record 10");
}
