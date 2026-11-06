//! The 'R'-weapon placement census (`drone-delete-turret-blast`): `DroneWeap_FireWeapon`
//! 0x178f68:122's mounted-minigun deploy branch fires only when the bot's held weapon byte
//! (BOT_vars+0x768) is 'R' (0x52) — and the whole branch is gated on `ext+0xc5 == 0x1e`, the
//! MP-bot mode row [M 00178f68:94; drone.rs:7557 "Mode 0x1e is the bot row"]. A guard's held
//! weapon byte starts at `BOTWEAP_InitWeapon` 0x129e48:48 `BOT_vars+0x768 = startweap` — the
//! MP loadout row (`PickupMatrix + MPSettings…*10`, [M 0x129e48:43-44]) — and only MP code
//! ever writes ext+0xd1c (`BOT_init` 0x1249e8:153, called by `BOT_respawn` 0x124e38:76 and
//! `MP_Start` 0x183a50:155 only [M callers]). So the SP guards' weapon is the placement's own
//! param 9 (`weapon_id(skin, param(9))`, the port's `NDrone2_DefaultInit` fold), and the
//! deploy branch can never select for them regardless of what a placement carries.
//!
//! This census pins the disc side: across every 07* level, NO drone placement (kind 0x0F)
//! resolves its weapon (param 9, the `weapon_id` input) to 0x52 — there is no 'R'-weapon
//! placement anywhere on retail data. Opt-in like the goldens: set `NF_ISO`; without it the
//! test prints "skip" and passes.

use nf_formats::iso9660::IsoImage;
use std::collections::BTreeMap;

#[test]
fn no_drone_placement_carries_the_r_weapon() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    // (level file, placement index) of every kind-0x0F placement whose weapon (param 9) is 0x52,
    // plus the weapon histogram keyed by level — the pin.
    let mut r_placements: Vec<(String, u32, u32)> = Vec::new();
    let mut hist: BTreeMap<String, BTreeMap<u32, usize>> = BTreeMap::new();
    for fe in entries.iter().filter(|e| e.name.len() == 12 && e.name.starts_with("070") && e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(level) = nf_formats::level::Level::parse(&data) else { continue };
        let Some(m) = level.main_map() else { continue };
        for (i, p) in level.maps[m].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x0f) {
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            // `weapon_id(skin, param(9))`: param 9 != 0 IS the weapon id (nf_formats::drone::weapon_id).
            let w = if param(9) != 0 { param(9) } else if param(0) == 0x0500_001e { 0 } else { 1 };
            *hist.entry(fe.name.to_ascii_uppercase()).or_default().entry(w).or_insert(0) += 1;
            if w == 0x52 {
                r_placements.push((fe.name.to_ascii_uppercase(), i as u32, param(0)));
            }
        }
    }
    assert!(hist.len() > 20, "the census saw {} levels — the disc walk broke", hist.len());
    assert!(r_placements.is_empty(),
        "an 'R' (0x52) drone placement exists on disc — the census premise broke: {r_placements:?}");
}
