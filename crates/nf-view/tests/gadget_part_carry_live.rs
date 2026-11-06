//! gadget-part-carry: the tracked gadgets across the 14 within-mission part changes, through the
//! REAL exit path — `warm_exit_save` (the capture `Player_RamSave` 0x195670 builds) →
//! `load_with_carry` (`Player_RamLoad` 0x1957a8 + `Player_InitWeapon` 0x1a7ef8's tail), the same
//! pair the App's exit chain drives (async_level_load_live pins the parity). The part pairs are
//! the mission::LEVELS parts in part order per mission (campaign_flow_live's missions()):
//! 3+3+2+1+3+2 = 14 within-mission part changes.
//!
//! THE GAME'S CARRY RULE, and why the drops are the game's own [M]:
//! - `Player_RamSave` 0x195670 copies the whole inventory — the 0x42 spare-ammo words (+0x038)
//!   and the 114 12-byte weapon entries (+0x07C: clip s16 at +4, owned u8 at +6) — so every
//!   gadget's owned bit and count rides the save, and `Player_RamLoad` 0x1957a8 restores them.
//! - The load itself wipes the object table: `ResetMap_Load` 0x1d6360's LoadState-3 arm runs
//!   `Mem_Init` + `hashtable_initialise` (:130-134) on EVERY level load, then the chain loads
//!   the shared 07000500 pack and the level's own archive (:28-36, 0x1d6360:28-36 — 07000500.bin
//!   holds no objects/skins [D script-census.md]). The table a warm load's gate sees is
//!   boot + shared pack + THIS level's archive — NOT the accumulated session (the g-021/§2
//!   comments' reading; the measured warm-15 Grapple loss [L 2026-10-14, fix_start_loadouts_live]
//!   is the empirical proof — 05's archive carries the Grapple fp skin, 05's own boot equip owns
//!   it [L ee_start_05 owned 0x50], yet the game's warm 15 drops it).
//! - `Player_InitWeapon` 0x1a7ef8's common tail then runs `Player_CheckWeaponsLoaded` 0x1a97c8:
//!   for every record with `weapon_data+4 != 0` (the port's `selectable()`), `+0xDC == 0` clears
//!   owned, and a `hashtable_getitem(+0xDC)` miss clears it. `Player_EquipWeapon` 0x1a8a58:20
//!   refuses the same equips at kit time.
//! - So a carried gadget survives a part change IFF its fp skin (+0xDC) is in the DESTINATION's
//!   own archive (or its record is not selectable). The sweep pins exactly that, per hop, with
//!   the [D] carrier scan.
//!
//! The grant: the tracked gadget ids set owned+clip on the live inventory (the fields the game's
//! pickup grants write [M Pickup_Create 0x18dda0 → the BLData entries]), plus the P2K owned so a
//! RamLoad destination's unowned-P2K kit rebuild never fires [M 0x1a7ef8's p2k gates]. 07000007
//! is the party disguise: its case runs `Player_InitAmmoWeapons` + the kit EVEN warm
//! [M 0x1a7ef8 case 0x7000007] — the game wipes the carried gadgets there by design.
//!
//! Opt-in: NF_ISO (the slim ISO is enough).

use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The 14 within-mission part changes, in story order: (from, to). The mission/part assignment
/// is mission::LEVELS [L ee_start_05.bin 0x2a4350].
const PART_CHANGES: [(u32, u32); 14] = [
    (0x07000001, 0x07000002), (0x07000002, 0x07000003), (0x07000003, 0x07000004), // mission 1
    (0x07000005, 0x07000006), (0x07000006, 0x07000007), (0x07000007, 0x07000008), // mission 2
    (0x07000009, 0x0700000a), (0x0700000a, 0x0700000b), // mission 3
    (0x0700000c, 0x0700000d), // mission 4
    (0x07000011, 0x07000012), (0x07000012, 0x0700004a), (0x0700004a, 0x07000013), // mission 6
    (0x07000014, 0x07000015), (0x07000015, 0x07000016), // mission 7
];

/// The tracked gadget ids: the families from gadget_carry_live's TRACKED plus the family twins
/// (mines 0x38/0x39, satchel 0x3c-0x40, decryptor 0x57) — every id the carry sweep can lose.
const GRANT: [u16; 18] = [
    0x2e, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x54, 0x56,
    0x57, 0x58,
];
/// The P2K (id 2): owned so the RamLoad destinations' unowned-P2K fallback never rebuilds the kit
/// [M 0x1a7ef8 case 0x7000006/8/0d/12/15's `p2k_owned` gate].
const P2K: u16 = 2;

/// The story archives in campaign order (the session the game PLAYS in — used only for the log's
/// carrier lists; the per-load rule below keys on the DESTINATION's own archive).
const STORY: [u32; 20] = [
    0x07000001, 0x07000002, 0x07000003, 0x07000004, 0x07000005, 0x07000006, 0x07000007, 0x07000008,
    0x07000009, 0x0700000a, 0x0700000b, 0x0700000c, 0x0700000d, 0x07000011, 0x07000012, 0x07000013,
    0x07000014, 0x07000015, 0x07000016, 0x0700001b,
];

struct Disc { img: nf_formats::iso9660::IsoImage<std::fs::File> }

impl Disc {
    fn open(iso: &str) -> Disc {
        Disc { img: nf_formats::iso9660::IsoImage::open(iso).expect("open ISO") }
    }
    fn level(&self, name: &str) -> Option<Level> {
        let (entries, bin) = nf_formats::disc::files_bin_entries(&self.img).ok()?;
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        let data = self.img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
        Level::parse(&data).ok()
    }
}

fn archive_name(id: u32) -> String { format!("{id:08x}.bin") }

/// Which story archives carry each granted id's fp skin [D] (for the log; the pin itself keys on
/// the destination's own archive).
fn fp_carriers(disc: &Disc, fps: &std::collections::HashMap<u16, u32>) -> std::collections::HashMap<u16, Vec<u32>> {
    let mut out: std::collections::HashMap<u16, Vec<u32>> = Default::default();
    let mut order: Vec<u32> = STORY.to_vec();
    order.push(0x0700_004a);
    for &lvl in order.iter() {
        let Some(level) = disc.level(&archive_name(lvl)) else { continue };
        for (&id, &fp) in fps.iter() {
            if level.skins.contains_key(&fp) { out.entry(id).or_default().push(lvl); }
        }
    }
    out
}

/// One hop: cold-load `from`, grant the kit, capture the warm-exit save (`warm_exit_save` =
/// `Player_RamSave` 0x195670), land `to` with `load_with_carry`, return (owned, clip) per id.
fn hop(iso_path: &str, from: u32, to: u32) -> Vec<(u16, bool, i16)> {
    let mut scene = nf_view::load(iso_path, &archive_name(from), false)
        .unwrap_or_else(|e| panic!("{from:08x}: {e}"));
    {
        let sim = scene.sim.as_mut().unwrap();
        let inv = &mut sim.drones.inventory;
        for &id in GRANT.iter() {
            let w = id as usize;
            if w < inv.owned.len() { inv.owned[w] = true; inv.clip[w] = 9; }
        }
        let p = P2K as usize;
        if p < inv.owned.len() { inv.owned[p] = true; inv.clip[p] = 0x30; }
    }
    let save = nf_view::warm_exit_save(scene.sim.as_ref().unwrap());
    let scene2 = nf_view::load_with_carry(iso_path, &archive_name(to), false, Some(&save))
        .unwrap_or_else(|e| panic!("{to:08x}: {e}"));
    let sim = scene2.sim.as_ref().unwrap();
    GRANT.iter().map(|&id| {
        let w = id as usize;
        (id, sim.drones.inventory.owned.get(w).copied().unwrap_or(false),
             sim.drones.inventory.clip.get(w).copied().unwrap_or(0))
    }).collect()
}

#[test]
fn the_part_change_carry_sweep() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let disc = Disc::open(&iso);
    let elf = {
        let e = disc.img.find("/ACTION.ELF").expect("ACTION.ELF");
        disc.img.read_file(e).expect("read ACTION.ELF")
    };
    let wd = nf_formats::weapon::WeaponData::from_action_elf(&elf).expect("weapon_data");
    let mut fps = std::collections::HashMap::new();
    for &id in GRANT.iter() {
        let fp = wd.first_person(id as usize);
        fps.insert(id, fp);
    }
    let carriers = fp_carriers(&disc, &fps);

    eprintln!("=== the 14 part changes: granted -> survived (the per-load rule) ===");
    for (from, to) in PART_CHANGES {
        let res = hop(&iso, from, to);
        // The destination's own archive [D]: the game's per-load table is boot + 07000500 +
        // this archive [M 0x1d6360:28-36, :130-134], so a carried gadget survives iff its fp
        // skin is HERE (or the record is not selectable — the game skips it [M 0x1a97c8]).
        let dst = disc.level(&archive_name(to)).unwrap_or_else(|| panic!("{to:08x} parses"));
        let kit_wipe = to == 0x0700_0007; // the party disguise [M 0x1a7ef8 case 0x7000007]
        for &(id, owned, _clip) in res.iter() {
            let fp = fps[&id];
            let selectable = wd.selectable(id as u8);
            // The party kit's own list [M 0x1a7ef8 case 0x7000007]: P2K/taser/laser/watch/camera.
            let kit_id = to == 0x0700_0007 && [2u16, 0x4a, 0x50, 0x54].contains(&id);
            let game_keeps = if to == 0x0700_0007 {
                // The wipe re-equips ONLY the kit list [M 0x1a7ef8 case 0x7000007]; anything
                // else the carry held is gone even if the fp skin is present.
                kit_id && fp != 0 && dst.skins.contains_key(&fp)
            } else {
                // The strip skips non-selectable records entirely [M 0x1a97c8's +4 gate]; a
                // selectable record keeps owned iff its fp is in the per-load table.
                !selectable || (fp != 0 && dst.skins.contains_key(&fp))
            };
            assert_eq!(owned, game_keeps,
                "{from:08x} -> {to:08x}: {id:#04x} owned={owned} but the game {} (fp {fp:08x}, in dst own archive: {}, selectable {selectable}, carriers {:?})",
                if game_keeps { "keeps it" } else { "drops it [M 0x1a97c8 + 0x1d6360's per-load wipe]" },
                dst.skins.contains_key(&fp), carriers.get(&id).map(|v| v.as_slice()).unwrap_or(&[]));
        }
        let kept: Vec<String> = res.iter().filter(|(_, o, _)| *o).map(|(id, _, _)| format!("{id:#04x}")).collect();
        let dropped: Vec<String> = res.iter().filter(|(_, o, _)| !*o).map(|(id, _, _)| format!("{id:#04x}")).collect();
        eprintln!("{from:08x} -> {to:08x}: kept [{}] dropped [{}]", kept.join(","), dropped.join(","));
    }
}
