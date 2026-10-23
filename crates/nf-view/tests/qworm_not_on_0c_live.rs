//! qworm-not-on-0c: the Q-Worm (0x58) must not join the loadout on levels whose session hashtable
//! misses its first-person skin (seen at frame 12579: Bond holding the Q-Worm on 0700000c, which
//! does not carry it).
//!
//! The game's rule, with addresses:
//! - `Player_InitWeapon` 0x1a7ef8 case 0x700000c (disasm 0x1a8568-0x1a85e4) equips the whole kit —
//!   handgun@0x30, `Upgrade_Weapon(0x1e)`@999, stunner@999, 0x50@0, 0x54@0, **0x58@0** — and holds
//!   the sniper (`sb s6,99(v1)` at 0x1a85e4). The kit LISTS the Q-Worm on 0c.
//! - `Player_EquipWeapon` 0x1a8a58 refuses an equip whose fp model (+0xDC) misses the hashtable
//!   (0x1a8ab8-0x1a8ae0: `hashtable_getitem` → 0 when missing), so the listed 0x58@0 only OWNS when
//!   the skin is loaded; `Player_CheckWeaponsLoaded` 0x1a97c8 (the 0x1a7ef8 tail) re-strips every
//!   selectable record whose fp model the hashtable misses.
//! - The Q-Worm's fp skin 0x050005b is carried by 07000009/0a/0b (+ the MP/bonus 041/046 containers)
//!   and by NO other story archive — in particular NOT by 0700000c.bin [D scan of every .bin
//!   container's kind-3 sub-files, the same scan gadget-carry-map used]. The game's hashtable only
//!   holds what its containers registered [M: the only deletions are `hashtable_delete_hashtype`
//!   0x04000000 calls — LoadableLoad 0x1d69e8 / LoadableReload 0x1d6ab0, top byte 0x04 = sequences].
//! - The port's dev/`--give` asset merge is a VIEWING aid: it merged foreign fp skins into
//!   `level.skins`, and the loadout's availability gate then owned the Q-Worm on 0c (reproduced
//!   through `load_dev("0700000c.bin")`: owned[0x58] = true before the fix). The fix: the gate sees
//!   the level's own skins + the game-accurate g-021 session pulls — not the viewing-aid merges.
//!
//! Tests are numbers through the real loader (NF_ISO), not images.

use std::collections::HashSet;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// owned flags after `load_dev` (the dev asset merge active — the case that leaked).
fn owned_dev(archive: &str) -> Option<(HashSet<u8>, u8, HashSet<u32>)> {
    let scene = nf_view::load_dev(&iso()?, archive, false).ok()?;
    let sim = scene.sim.as_ref().unwrap();
    let owned: HashSet<u8> = sim.drones.inventory.owned.iter().enumerate()
        .filter(|(_, &o)| o).map(|(i, _)| i as u8).collect();
    (owned, sim.gun.switch_request, sim.skins.keys().copied().collect()).into()
}

/// owned flags after a plain cold `load` (no asset merge).
fn owned_cold(archive: &str) -> Option<(HashSet<u8>, u8)> {
    let scene = nf_view::load(&iso()?, archive, false).ok()?;
    let sim = scene.sim.as_ref().unwrap();
    let owned: HashSet<u8> = sim.drones.inventory.owned.iter().enumerate()
        .filter(|(_, &o)| o).map(|(i, _)| i as u8).collect();
    Some((owned, sim.gun.switch_request))
}

/// 0c's start inventory equals the game's [M 0x1a8568 + 0x1a97c8; D the skin-carrier scan]: the
/// kit's grapple/camera/laser own (their fp skins ride 0700000c.bin [D]), the decryptor and the
/// Q-Worm do NOT (0x050005c and 0x050005b are on no 0c-session container [D]), and the held weapon
/// is the sniper (request 0x1e [M `sb s6,99(v1)` 0x1a85e4]). This is the repro: before the fix the
/// dev asset merge supplied 0x050005b and owned[0x58] was true — the repro screenshot.
#[test]
fn the_0700000c_start_inventory_is_the_games() {
    let Some((owned, req, _skins)) = owned_dev("0700000c.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    assert!(!owned.contains(&0x58), "the Q-Worm must not be owned on 0c — the fp skin 0x050005b is on no 0c-session container [D]; the dev/give merge's foreign skins must not leak into the loadout [M 0x1a97c8]");
    assert!(!owned.contains(&0x56), "the Q-Decryptor likewise (0x050005c carriers: 09/0a/0b/14/15/41/46 [D scan])");
    assert!(owned.contains(&0x50), "the grapple owns (fp skin 0x0500044 rides 0700000c.bin [D])");
    assert!(owned.contains(&0x54), "the micro-camera owns (0x0500045 rides 0700000c.bin [D])");
    assert!(owned.contains(&0x4e), "the watch laser owns (0x050004b rides 0700000c.bin [D]; the common tail equips it 999 [M 0x1a7ef8:318-319])");
    assert!(owned.contains(&0x1e), "the sniper owns (the kit's Upgrade_Weapon(0x1e)@999 [M 0x1a8590])");
    assert_eq!(req, 0x1e, "the held weapon is the sniper [M 0x1a85e4]");
    // Sister: the cold load (no asset merge at all) — the same inventory.
    let Some((owned_cold, req_cold)) = owned_cold("0700000c.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    assert!(!owned_cold.contains(&0x58) && !owned_cold.contains(&0x56));
    assert!(owned_cold.contains(&0x50) && owned_cold.contains(&0x54) && owned_cold.contains(&0x4e));
    assert_eq!(req_cold, 0x1e);
}

/// A level that does carry the Q-Worm still has it [M 0x1a7ef8 case 0x7000009; D 07000009.bin
/// carries 0x050005b]: the island kit owns 0x58 even under the dev asset merge. Sister: 0a's cold
/// load runs the same fresh-09 kit through the RamLoad fallback (save = None → the gate weapon
/// unowned → `switchD_001a8284_caseD_7000009` [M 0x1a8418-0x1a8448]).
#[test]
fn the_island_still_carries_the_qworm() {
    let Some((owned, _req, _skins)) = owned_dev("07000009.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    assert!(owned.contains(&0x58), "the Q-Worm owns on 07000009 [M case 09; D the fp skin rides 07000009.bin]");
    assert!(owned.contains(&0x56), "the decryptor owns there too [M case 09; D]");
    let Some((owned_a, _)) = owned_cold("0700000a.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    assert!(owned_a.contains(&0x58), "0a's fresh-09 fallback owns it [M 0x1a8418 gate → caseD_7000009]");
}

/// Sister: 0d (Power Station A2) — the cold load's RamLoad fallback re-runs the 0c kit [M
/// 0x1a8538 gate owned[2] → caseD_700000c], and the gate keeps the Q-Worm off there too.
#[test]
fn the_0700000d_fallback_strips_it_too() {
    let Some((owned, req)) = owned_cold("0700000d.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    assert!(!owned.contains(&0x58), "0d's fallback 0c kit must not own the Q-Worm (no local fp skin [D])");
    assert_eq!(req, 0x1e, "the fallback holds the sniper like 0c [M 0x1a85e4]");
}
