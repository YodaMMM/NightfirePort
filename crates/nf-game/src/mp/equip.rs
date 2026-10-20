//! mp-lobby-settings: the MP spawn loadout and the MP pain scaling — the two places the lobby's
//! Weapon Set, grapple, locational-damage and Player Mod 0x1ac words take effect in a match.
//!
//! - `PickupMatrix` @0x2b9100 (110 bytes = 11 rows × 5 s16 — the readers index it
//!   `PickupMatrix + set·10 + column·2` in BYTES [M Pickup_CreateFromSet 0x18f1b0:56,
//!   MP_EquipPlayer 0x1855f8:18, BOTWEAP_InitWeapon 0x129e48:44; Pickup_MakeRandomWeaponSet
//!   0x18f098 fills the 5 entries at +100 = row 10]). The values are the ELF's static image [D
//!   ACTION.ELF file bytes at 0x2b9100]; row 10 (Random) is zeros until the match init fills it.
//! - `MP_EquipPlayer` 0x1855f8 — the spawn loadout (every `MP_ReSpawn` and the MP arm of
//!   `Player_InitWeapon` 0x1a7ef8:143-145).
//! - the MP arm of `Player_HandlePain` 0x1902c8:51-82 / `BOT_handlePain` 0x125ed8:48-79.

use crate::drone::Rng;
use crate::mp::MpSettings;

/// `PickupMatrix` @0x2b9100: row = the weapon set (MPSettings+0x1B4), column 0 = the spawn weapon,
/// columns 0..4 = the set pickups' kinds 7..11 [M the readers above; D the ELF bytes]. Row 10 is
/// the Random set's — zeros on the disc, filled by [`random_weapon_row`].
pub const PICKUP_MATRIX: [[u16; 5]; 11] = [
    [6, 28, 30, 24, 44],   // Normal
    [2, 6, 10, 16, 66],    // Pistols
    [18, 21, 24, 28, 82],  // Automatic
    [14, 30, 17, 36, 50],  // Sniping
    [42, 44, 58, 46, 26],  // Explosives 1
    [53, 52, 58, 55, 59],  // Explosives 2
    [2, 17, 55, 36, 66],   // MI6 Operative
    [10, 30, 82, 26, 50],  // Phoenix Weapons
    [26, 59, 50, 58, 44],  // State of the Art
    [21, 22, 55, 17, 36],  // Cloak and Dagger
    [0, 0, 0, 0, 0],       // Random (filled at the match init)
];

/// `UseableGuns` @0x2b9170 (54 bytes = 27 s16) [D the ELF bytes] — the Random set's pool.
pub const USEABLE_GUNS: [u16; 27] = [2, 6, 10, 12, 14, 16, 18, 20, 22, 25, 26, 28, 30, 36, 42, 44, 46, 51, 55, 53, 54, 52, 58, 59, 66, 17, 82];

/// `Pickup_MakeRandomWeaponSet` 0x18f098: the row zeroed, then 5 picks without repeats — the
/// first from the pool's first 0x1a entries, the rest from all 0x1b, each re-drawn while it
/// equals an earlier pick [M :17-50].
pub fn random_weapon_row(rng: &mut Rng) -> [u16; 5] {
    let mut row = [0u16; 5];
    for i in 0..5 {
        loop {
            let r = rng.rand(if i == 0 { 0x1a } else { 0x1b }) as usize;
            let w = USEABLE_GUNS[r.min(USEABLE_GUNS.len() - 1)];
            if row[..i].contains(&w) { continue; }
            row[i] = w;
            break;
        }
    }
    row
}

/// The weapon-set row the match reads. The game draws the Random row from the shared stream in
/// every `MP_Init` 0x183438:11 [M]; the port draws it only for the Random set, from a stream of its
/// own seeded by the lockstep seed — equal on every machine, and the shared stream's positions
/// (which the port does not match to the game's anyway) stay unchanged for the other sets [S].
pub fn weapon_row(s: &MpSettings) -> [u16; 5] {
    let set = (s.weapon_set as usize).min(10);
    if set == 10 {
        let mut rng = Rng::new(s.seed ^ 0x5745_4150);
        return random_weapon_row(&mut rng);
    }
    PICKUP_MATRIX[set]
}

/// `Upgrade_MPWeapon` 0x1a9040 with the boot `Upgrades` table (all zero [D the ELF's 0x2beea8]
/// — the port has no MP upgrade shop): 2/4 → `UpgradedPPK_MP[0]` = 2, 6/8 → `UpgradedP99_MP[0]` =
/// 6, the snipers 0x1e/0x20/0x22 → 0x1e, the silenced 0x24/0x26/0x28 → 0x24, 0x50 →
/// `UpgradedGrapple_MP[0]` = 0x50; anything else unchanged [M :12-41; D the tables].
pub fn upgrade_mp_weapon(id: u16) -> u16 {
    match id {
        2 | 4 => 2,
        6 | 8 => 6,
        0x1e | 0x20 | 0x22 => 0x1e,
        0x24 | 0x26 | 0x28 => 0x24,
        0x50 => 0x50,
        _ => id,
    }
}

/// `MP_EquipPlayer` 0x1855f8 for one human: `Player_InitAmmoWeapons`, the set's spawn weapon
/// (`Upgrade_MPWeapon(base(PickupMatrix[set][0]))` with `clip_size·2` rounds) after the fists, the
/// AR's grenade ammo, the grapple when MPSettings+0x1D0 is set, the scenario bonus weapon 0x3B
/// (Protection team 0 / Demolition team 1), `Player_CheckWeaponsLoaded` [M :14-53]. Returns the
/// switch request (`M_CONTROL+99` = the spawn weapon [M :57]). The `M_CONTROL+0x94 == 0x5000085`
/// arm (equip 0x45) is not modelled — its anim-set test has no port reader [S].
pub fn mp_equip_player(
    s: &MpSettings,
    row: &[u16; 5],
    slot: usize,
    inv: &mut crate::pickup::Inventory,
    w: &nf_formats::weapon::WeaponData,
    loaded: impl Fn(u8) -> bool + Copy,
) -> u8 {
    crate::player::init_ammo_weapons(inv, w, loaded);
    let start = upgrade_mp_weapon(w.base(row[0] as usize));
    let rounds = w.clip_size(start as usize).wrapping_mul(2);
    inv.equip_weapon(w, 1, 0, loaded(1));
    inv.equip_weapon(w, start as usize, rounds, loaded(start as u8));
    if start == 0x1a {
        inv.equip_ammo(w, 0x1b, w.clip_size(0x1b));
    }
    if s.grapple != 0 {
        let g = upgrade_mp_weapon(0x50);
        inv.equip_weapon(w, g as usize, 0, loaded(g as u8));
    }
    let team = s.players.get(slot).map_or(0, |p| p.team);
    if (s.scenario == crate::mp::scenario::PROTECTION && team == 0)
        || (s.scenario == crate::mp::scenario::DEMOLITION && team == 1) {
        inv.equip_weapon(w, 0x3b, 1, loaded(0x3b));
    }
    // `Player_CheckWeaponsLoaded` 0x1a97c8 — the loaded refusal already ran per equip.
    for id in 0..inv.owned.len().min(0x72) {
        if inv.owned[id] && !loaded(id as u8) && w.selectable(id as u8) { inv.owned[id] = false; }
    }
    start as u8
}

/// The bot's start weapon and its rounds — `BOTWEAP_InitWeapon` 0x129e48:44-48: `startweap =
/// base(PickupMatrix[set][0])` with `clip_size·2` rounds [M].
pub fn bot_start_weapon(row: &[u16; 5], w: &nf_formats::weapon::WeaponData) -> (u8, i16) {
    let start = w.base(row[0] as usize);
    (start as u8, w.clip_size(start as usize).wrapping_mul(2))
}

/// The pain multipliers `ReadTuningVars` 0x1d3a80 sets for MP (`Plr_DMod_Multi`, `_Head`,
/// `_UpperLimb`, `_LowerLimb`).
#[derive(Clone, Copy, Debug)]
pub struct MpPainMods { pub multi: f32, pub head: f32, pub upper: f32, pub lower: f32 }

impl MpPainMods {
    pub fn from_tuning(t: &nf_formats::tuning::Tuning) -> MpPainMods {
        MpPainMods { multi: t.Plr_DMod_Multi, head: t.Plr_DMod_Head, upper: t.Plr_DMod_UpperLimb, lower: t.Plr_DMod_LowerLimb }
    }
}

/// The body part the port's hit test resolves (the game passes the bone index of the joint the
/// bullet met; −1 = no part [M Player_HandlePain's param_5]).
pub const PART_NONE: i16 = -1;
/// Part 5: the head [M 0x1902c8:56 — `case 0` of `part − 5`].
pub const PART_HEAD: i16 = 5;
/// A torso part — any part outside the switch's lists keeps the factor 1 [M the default arm].
pub const PART_TORSO: i16 = 1;
/// 0x14: an upper limb [M the 0xf/0x10/0x1b/0x1e cases = parts 0x14/0x15/0x20/0x23].
pub const PART_UPPER_LIMB: i16 = 0x14;
/// 0x31: a lower limb [M the 0x2c..0x33 cases = parts 0x31..0x38].
pub const PART_LOWER_LIMB: i16 = 0x31;

/// The MP arm of `Player_HandlePain` 0x1902c8:51-82 (the same in `BOT_handlePain` 0x125ed8:48-79):
/// a hit with a part × `Plr_DMod_Multi`; with MPSettings+0x1C8 (locational damage) the part's
/// factor (head / upper limb / lower limb); with MPSettings+0x1BC (Player Mod 0x1ac) × 3.0 [M].
pub fn mp_pain(s: &MpSettings, mods: &MpPainMods, damage: f32, part: i16) -> f32 {
    let mut d = damage;
    if part != PART_NONE { d *= mods.multi; }
    if s.location_damage != 0 {
        let f = match part - 5 {
            0 => Some(mods.head),
            0xf | 0x10 | 0x1b | 0x1e => Some(mods.upper),
            0x2c..=0x33 => Some(mods.lower),
            _ => None,
        };
        if let Some(f) = f { d *= f; }
    }
    if s.capture_reward != 0 { d *= 3.0; }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_row_has_no_repeats_and_draws_from_the_pool() {
        for seed in 0..50u32 {
            let mut rng = Rng::new(seed);
            let row = random_weapon_row(&mut rng);
            for i in 0..5 {
                assert!(USEABLE_GUNS.contains(&row[i]));
                assert!(!row[..i].contains(&row[i]), "seed {seed}: repeat in {row:?}");
            }
            // The first pick never takes the pool's last entry (0x1a, not 0x1b) [M :26-29].
            assert_ne!(row[0], 82);
        }
    }

    #[test]
    fn pain_mods() {
        let mods = MpPainMods { multi: 4.0, head: 4.0, upper: 0.8, lower: 0.8 };
        let mut s = MpSettings::deathmatch(0x0700_0023);
        s.location_damage = 0;
        s.capture_reward = 0;
        assert_eq!(mp_pain(&s, &mods, 2.0, PART_HEAD), 8.0);
        assert_eq!(mp_pain(&s, &mods, 2.0, PART_NONE), 2.0);
        s.location_damage = 1;
        assert_eq!(mp_pain(&s, &mods, 2.0, PART_HEAD), 32.0);
        assert_eq!(mp_pain(&s, &mods, 2.0, PART_TORSO), 8.0);
        assert!((mp_pain(&s, &mods, 2.0, PART_LOWER_LIMB) - 6.4).abs() < 1e-5);
        s.capture_reward = 1;
        assert_eq!(mp_pain(&s, &mods, 2.0, PART_TORSO), 24.0);
        // Sister: the mod off really is ×1.
        s.capture_reward = 0;
        assert_ne!(mp_pain(&s, &mods, 2.0, PART_TORSO), 24.0);
    }

    #[test]
    fn matrix_rows_follow_the_byte_stride() {
        // Row 0's spawn weapon is 6 and its five set columns are the first 10 bytes [D].
        assert_eq!(PICKUP_MATRIX[0], [6, 28, 30, 24, 44]);
        assert_eq!(PICKUP_MATRIX[10], [0; 5]);
        let mut s = MpSettings::deathmatch(0x0700_0023);
        s.weapon_set = 1;
        assert_eq!(weapon_row(&s), [2, 6, 10, 16, 66]);
        s.weapon_set = 10;
        s.seed = 7;
        let a = weapon_row(&s);
        assert_eq!(a, weapon_row(&s), "the Random row is the seed's, equal on every machine");
        assert_ne!(a, [0; 5]);
    }
}
