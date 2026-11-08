//! mp-fists-skin: `Player_Init` 0x1910f8's multiplayer weapon_data arm — the port had the
//! single-player arm only, so an MP match's fists carried the SP PunchSkin row and the 1 twin link
//! instead of the game's MP values.
//!
//! - The real MP load (`load_mp_carry`, the confirm page's path) leaves `weapon_data[1]+0xDC`
//!   (Ghidra's DECIMAL `weapon_data._488_4_`, 488 = 0x10c + 0xDC) = 0x050000b0 and
//!   `weapon_data[0x3999]` (a single byte at record 0x37 + 5, the twin/variant link) = 2
//!   [M 0x1910f8:108-113, disasm 0x1913bc `sw $v1, 0x1e8($a0)` / 0x1913c4 `sb $a3, 0x3999($a0)`;
//!   the SP arm's same two stores are 0x191394/0x19138c]. The fists are then OWNED by the real
//!   `MP_EquipPlayer` equip (the loadout gate `Player_CheckWeaponsLoaded` 0x1a97c8 sees the MP
//!   skin — on the old code 0x050000ab is on no arena, the gate refused the fists and an MP
//!   player had no fists to cycle to) and the model resolves from the arena's own archive [D].
//! - Single-player loads keep their PunchSkin rows and the 1 link [M 0x1910f8:83-107; L
//!   ee_start_05.bin @ 0x2bf338 (= weapon_data 0x2bf150 + 0x1E8) = 0x050000ab and @ 0x2c2ae9
//!   (= +0x3999) = 1, measured after the SP boot on 05] — the decompile's four-row switch
//!   re-pinned per row.
//! - Every MP arena's own archive carries 0x050000b0 [D disc scan] — every online machine
//!   resolves it identically from the same disc bytes. The two MP values are load-time
//!   constants: no Rand draw anywhere on the path (the reload-with-another-seed pin).
//!
//! Opt-in via NF_ISO.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// `weapon_data[1]+0xDC` after `Player_Init`'s MP arm [M 0x1913bc].
const MP_FISTS_SKIN: u32 = 0x0500_00b0;

/// The game's MP arenas [M `ResetMap_LevelToLoad` 0x1d3800's set; D mp_core_live's spawn pins].
const ARENAS: [u32; 8] = [
    0x0700_0023, 0x0700_0024, 0x0700_0025, 0x0700_0026,
    0x0700_0027, 0x0700_0028, 0x0700_0029, 0x0700_004b,
];

fn read_level(path: &str, code: u32) -> Level {
    let img = IsoImage::open(path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let name = format!("{code:08x}.bin");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name))
        .unwrap_or_else(|| panic!("arena {code:08x}.bin missing from FILES.BIN"));
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap_or_else(|e| panic!("{code:08x}.bin: {e}"))
}

/// The real MP match: the weapon_data arm, the fists ownership through the real equip, the
/// arena's own model, and the seed-independence (lockstep: the same table on every machine).
#[test]
fn the_mp_match_carries_the_mp_fists_skin_the_2_link_and_own_fists() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let settings = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    let mut scene = nf_view::load_mp_carry(&path, "07000023.bin", false, None, settings)
        .expect("the arena loads through the app's MP carry path");
    let sim = scene.sim.as_ref().expect("the match builds a sim");

    // [M 0x1913bc] weapon_data + 0x1E8 (record 1 + 0xDC) = 0x050000b0.
    let fp = u32::from_le_bytes(sim.weapons.records[1][0xdc..0xe0].try_into().unwrap());
    assert_eq!(fp, MP_FISTS_SKIN, "the MP arm's fists fp skin (weapon_data._488_4_, decimal 488)");
    // [M 0x1913c4] weapon_data byte 0x3999 (record 0x37 + 5, the twin link) = 2.
    assert_eq!(sim.weapons.records[0x37][5], 2, "the twin link (Player_HandleHasNoAmmo 0x1a2ba8's w + link)");

    // The arena's own archive carries the model — the load's skin table resolves it [D].
    assert!(sim.skins.contains_key(&MP_FISTS_SKIN), "07000023.bin carries the 0x050000b0 skin");

    // The loadout gate (`Player_CheckWeaponsLoaded`'s port: mp_weapon_loaded over the fp hashes)
    // sees the MP skin, so the real `MP_EquipPlayer` equip owns the fists.
    assert!(sim.mp_weapon_loaded.get(1).copied().unwrap_or(false),
        "the loadout gate sees fists fp 0x050000b0");
    assert!(sim.drones.inventory.owned.get(1).copied().unwrap_or(false),
        "MP_EquipPlayer owns the fists (the old code's SP row is on no arena — the gate refused them)");

    // Lockstep: the same two constants and the disc table on every machine, whatever the match
    // seed — no Rand draw touches the path. A second load with a different seed is bit-identical.
    let mut other = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    other.seed = 0x1234_5678;
    let scene2 = nf_view::load_mp_carry(&path, "07000023.bin", false, None, other).expect("the reload");
    let sim2 = scene2.sim.as_ref().unwrap();
    assert_eq!(sim.weapons.records, sim2.weapons.records,
        "weapon_data is seed-independent (the shared Rand stream is untouched)");
    assert_eq!(sim.mp_weapon_loaded, sim2.mp_weapon_loaded, "the loadout gate is seed-independent");
}

/// Single-player keeps its rows: the real SP load re-pins the decompile's default row, the other
/// three classes are pinned at the table level (the real loads would re-run the same one arm).
#[test]
fn single_player_loads_keep_their_punchskin_rows_and_the_1_link() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&path, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_ref().expect("the SP sim builds");
    let fp = u32::from_le_bytes(sim.weapons.records[1][0xdc..0xe0].try_into().unwrap());
    // [M 0x1910f8 default arm → PunchSkin_204[0]; L ee_start_05.bin @ 0x2bf338 = 0x050000ab,
    // @ 0x2c2ae9 = 1 — the disc's own post-boot values on this very level.]
    assert_eq!(fp, 0x0500_00ab, "the SP default row (PunchSkin[0])");
    assert_eq!(sim.weapons.records[0x37][5], 1, "the SP twin link is 1");

    // The other three GameState classes [M 0x1910f8:84-104; L ee_start_05.bin @ 0x2b9430].
    let img = IsoImage::open(&path).unwrap();
    let elf = img.read_file(img.find("/ACTION.ELF").unwrap()).unwrap();
    let rows: [(u32, u32); 3] = [
        (0x0700_0001, 0x0500_0099), // 0x7000001..=0x7000004 → PunchSkin[1]
        (0x0700_0007, 0x0500_003d), // 0x7000007/0x7000008   → PunchSkin[2]
        (0x0700_001b, 0x0500_00a1), // 0x700001b             → PunchSkin[3]
    ];
    for (level, want) in rows {
        let mut wd = nf_formats::weapon::WeaponData::from_action_elf(&elf).unwrap();
        wd.apply_player_init_override(level);
        let got = u32::from_le_bytes(wd.records[1][0xdc..0xe0].try_into().unwrap());
        assert_eq!(got, want, "level {level:#x}: the SP arm's fists skin");
        assert_eq!(wd.records[0x37][5], 1, "level {level:#x}: the SP twin link");
    }
}

/// [D] the disc scan: every MP arena's own archive carries the MP fists skin the MP arm writes
/// into weapon_data[1]+0xDC — the model each machine resolves from its own load.
#[test]
fn every_mp_arena_carries_the_mp_fists_skin() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    for code in ARENAS {
        let level = read_level(&path, code);
        assert!(level.skins.contains_key(&MP_FISTS_SKIN),
            "{code:08x}.bin: the MP fists skin {MP_FISTS_SKIN:#x} is not in the arena's skins");
    }
}
