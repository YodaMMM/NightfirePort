//! The MP bots' weapon layer — the `BOTWEAP_*` family (ACTION.ELF 0x129d28..0x12b200) and the
//! weapon-choice half of `BOTSTATE_*` (`combatWeaponChangeChoice` 0x128708, `isPreferredWeapon`
//! 0x129580, `pickupWeaponChangeChoice` 0x129728), plus the bot half of `Pickup_Handler`
//! 0x18e650 (the class-2 taker path) and `BOT_handlePain` 0x125ed8's damage arithmetic.
//!
//! Evidence tags: [M] the decompile (`work/decomp/ACTION.ELF/<addr>_*.c`, `:NN` = line) or the
//! disassembly (`tools/crosscheck/mipsdis.py`, address); [D] bytes of ACTION.ELF; [S] port choice.
//! Spec: `docs/systems/mp-bot-weapons.md`.
//!
//! Every bot owns a [`BotArsenal`] (the `BOT_vars` weapon records): `+0x144 + w·0xC` loaded rounds,
//! `+0x146 + w·0xC` the weapon-present flag, `+0x698 + t·2` the ammo pool per ammo type, `+0x769`
//! the armour [M docs/fields/bot-vars.md]. The `weapon_data` rows the layer reads are lifted once
//! into a [`BotWeaponTable`] by the caller (the disc's table + which first-person skins the arena
//! loaded — the `hashtable_getitem` gate of `BOTWEAP_EquipWeapon`).

use nf_formats::weapon::WeaponData;

/// The per-bot weapon records: `listHeldLoadedWeapons` / `CheckWeaponsLoaded` walk `w < 0x53`
/// [M 0x12aa80, 0x12aec0].
pub const BOT_WEAPONS: usize = 0x53;
/// The ammo pool `BOT_vars+0x698`: `BOTWEAP_InitWeapon` zeroes 0x21 s16 [M 0x129e48:24-29].
pub const AMMO_TYPES: usize = 0x21;

/// `DAT_002f3e10` — the bots' weapon preference order (60 u16, best first), copied to the stack
/// by `listHeldLoadedWeapons` and searched for each held weapon's rank [M 0x12aa80; D ACTION.ELF
/// file 0x1edd10].
pub const PREFERENCE: [u8; 60] = [
    0x45, 0x42, 0x2e, 0x2f, 0x2d, 0x2c, 0x2a, 0x2b, 0x33, 0x41, 0x1a, 0x19, 0x16, 0x18, 0x13,
    0x12, 0x15, 0x14, 0x1d, 0x1c, 0x23, 0x21, 0x1f, 0x22, 0x20, 0x1e, 0x29, 0x27, 0x25, 0x28,
    0x26, 0x24, 0x11, 0x08, 0x09, 0x04, 0x05, 0x0e, 0x0f, 0x10, 0x0d, 0x0b, 0x0a, 0x0c, 0x06,
    0x07, 0x03, 0x02, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x34, 0x52, 0x3a, 0x35, 0x36, 0x37,
];

/// `PickupMatrix` @0x2b9100 rows 0..4 (the static weapon sets; row 5 is the Random set
/// `Pickup_MakeRandomWeaponSet` 0x18f098 fills at run time) [D ACTION.ELF file 0x1b3000].
pub const PICKUP_MATRIX: [[u16; 10]; 5] = [
    [0x06, 0x1c, 0x1e, 0x18, 0x2c, 0x02, 0x06, 0x0a, 0x10, 0x42],
    [0x12, 0x15, 0x18, 0x1c, 0x52, 0x0e, 0x1e, 0x11, 0x24, 0x32],
    [0x2a, 0x2c, 0x3a, 0x2e, 0x1a, 0x35, 0x34, 0x3a, 0x37, 0x3b],
    [0x02, 0x11, 0x37, 0x24, 0x42, 0x0a, 0x1e, 0x52, 0x1a, 0x32],
    [0x1a, 0x3b, 0x32, 0x3a, 0x2c, 0x15, 0x16, 0x37, 0x11, 0x24],
];

/// `Plr_DMod_Multi` — `ReadTuningVars` 0x1d3a80:272 writes 4.0 for the MP arenas
/// 0x7000021..29/4B/4C (the same group as `Plr_DMod_Head` 4.0, the limbs 0.8) [M].
pub const DMOD_MULTI: f32 = 4.0;
pub const DMOD_HEAD: f32 = 4.0;
pub const DMOD_LIMB: f32 = 0.8;

/// The bomb (`weapon 0x3b`) — the demolition/protection bonus weapon [M 0x129e48:50-56].
pub const BOMB: u8 = 0x3b;
/// The fists (weapon 1, equipped with 999 rounds at init [M 0x129e48:47]).
pub const FISTS: u8 = 1;

/// One `weapon_data` row as the bot layer reads it (offsets per docs/fields/weapon_data.md).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BotWeaponRow {
    /// +0x02 the base weapon (the owned-slot selector).
    pub base: u8,
    /// +0x04 selectable/ownable (`CheckWeaponsLoaded`'s gate).
    pub selectable: bool,
    /// +0x05 != 0 — the twin/variant link (the clip shared with the base when the ammo types match).
    pub twin: bool,
    /// +0x06 the class (4 = explosive; `isPreferredWeapon` reads 1/2/3).
    pub class: u8,
    /// +0x08 the min engagement range (`tooCloseForWeapon`, the explosive pick).
    pub min_range: f32,
    /// +0x0C the damage per hit.
    pub damage: f32,
    /// +0x18 the bullets per shot (`DroneWeap_FireWeapon`'s `Bullet_init` loop [M 0x178f68:248-257]).
    pub bullets: u8,
    /// +0x1C the range.
    pub range: f32,
    /// +0x24 the spread (`Bullet_init`, the port's `bullet::spread`).
    pub spread: f32,
    /// +0x2C the shots per burst (`DroneWeap_NextBulletTime` 0x179d40:52).
    pub burst: u16,
    /// +0x40 the refire interval in frames (s16 × FRAME_RATE_DIV [M 0x179d40:26-47]).
    pub interval: i16,
    /// +0x68 the flags (0x10 clip weapon — `EquipAmmo`; 0x100 one-at-a-time reload — `ReloadAmmoType`).
    pub flags: u32,
    /// +0x90 the ammo type.
    pub ammo_type: u8,
    /// +0x92 the clip capacity — the word `BOTWEAP_getWeaponClipSize` 0x12b010 returns
    /// (`weapon_data + w*0x10C + 0x92`) [M].
    pub clip: i16,
    /// +0xDC != 0 and (weapon 1 or the first-person skin loaded) — `BOTWEAP_EquipWeapon`'s gates
    /// [M 0x12a138:18-25] and `CheckWeaponsLoaded`'s [M 0x12aec0].
    pub usable: bool,
}

/// The `weapon_data` + `ammo_data` subset the bot layer reads, lifted once per arena.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BotWeaponTable {
    pub rows: Vec<BotWeaponRow>,
    /// `ammo_data[t]+0x02` — the most rounds a pool holds (`EquipWeapon`/`EquipAmmo` clamps).
    pub ammo_cap: Vec<i16>,
}

fn f32_at(r: &[u8], o: usize) -> f32 { f32::from_le_bytes([r[o], r[o + 1], r[o + 2], r[o + 3]]) }

impl BotWeaponTable {
    /// Lift the rows from the disc's `weapon_data`; `loaded(w)` = the weapon's first-person skin
    /// (`+0xDC`) is in the arena's hashtable (the port: the scene's skins map).
    pub fn from_weapon_data(wd: &WeaponData, loaded: impl Fn(usize) -> bool) -> BotWeaponTable {
        let rows = (0..BOT_WEAPONS).map(|w| {
            let Some(r) = wd.records.get(w) else { return BotWeaponRow::default() };
            let fp = u32::from_le_bytes([r[0xdc], r[0xdd], r[0xde], r[0xdf]]);
            BotWeaponRow {
                base: u16::from_le_bytes([r[2], r[3]]) as u8,
                selectable: r[4] != 0,
                twin: r[5] != 0,
                class: r[6],
                min_range: f32_at(r, 0x08),
                damage: f32_at(r, 0x0c),
                bullets: r[0x18],
                range: f32_at(r, 0x1c),
                spread: f32_at(r, 0x24),
                burst: u16::from_le_bytes([r[0x2c], r[0x2d]]),
                interval: i16::from_le_bytes([r[0x40], r[0x41]]),
                flags: u32::from_le_bytes([r[0x68], r[0x69], r[0x6a], r[0x6b]]),
                ammo_type: r[0x90],
                clip: i16::from_le_bytes([r[0x92], r[0x93]]),
                usable: fp != 0 && (w == FISTS as usize || loaded(w)),
            }
        }).collect();
        BotWeaponTable { rows, ammo_cap: wd.ammo_cap.clone() }
    }

    pub fn row(&self, w: u8) -> BotWeaponRow { self.rows.get(w as usize).copied().unwrap_or_default() }
    fn cap(&self, t: u8) -> i16 { self.ammo_cap.get(t as usize).copied().unwrap_or(0) }

    /// The clip slot: the base's when `+0x05` is set and the ammo types agree [M 0x12ae40,
    /// 0x12a138:28-33, 0x129d28:21-25].
    pub fn clip_slot(&self, w: u8) -> usize {
        let r = self.row(w);
        if r.twin && self.row(r.base).ammo_type == r.ammo_type { r.base as usize } else { w as usize }
    }

    /// `startweap` = `weapon_data[PickupMatrix[set·10]].base` [M 0x129e48:44-45]. Sets past the
    /// static rows (the Random set) fall back to row 0 [S].
    pub fn start_weapon(&self, set: u32) -> u8 {
        let row = PICKUP_MATRIX.get(set as usize).unwrap_or(&PICKUP_MATRIX[0]);
        self.row(row[0] as u8).base
    }
}

/// `BOTWEAP_isValidWeapon` 0x12b0f0: the switch over `w − 1` [M].
pub fn is_valid_weapon(w: u8) -> bool {
    matches!(w, 0x01..=0x16 | 0x18..=0x1a | 0x1c..=0x2f | 0x33..=0x37 | 0x3a..=0x42 | 0x45 | 0x52)
}

/// `BOTWEAP_punchIsBetterIfClose` 0x12b1b8: the switch over `w − 0x34` [M].
pub fn punch_is_better_if_close(w: u8) -> bool {
    matches!(w, 0x34..=0x37 | 0x3a..=0x40 | 0x52 | 0x53)
}

/// `BOTWEAP_reloadAnimForWeapon` 0x12b200 (disasm): the weapons that play the reload anim 0x52.
pub fn reload_anim_for_weapon(w: u8) -> bool {
    !matches!(w, 1 | 0x34..=0x37 | 0x3a..=0x41 | 0x45 | 0x52 | 0x53)
}

/// `BOTWEAP_getWeaponAnimSet` 0x12b140 [M] — the weapon id → anim-set switch
/// (`DroneWeap_ChangeWeapon` writes it to Drone+0xDA [M 0x12a028]). Unlisted ids fall to 0x14
/// (the default arm), as do 0x16 and 0x1a — the switch has no arms for them.
pub fn weapon_anim_set(w: u8) -> u8 {
    // fix-849: the switch index is `(short)(w − 1)` with a `< 83` bound [M-dis 0x12b140-0x12b154:
    // `lui v0,0xffff; sll a0,a0,16; addu; sra 16; sltiu v1,83`] — the arms below are INDICES
    // (case 0 = weapon 1); weapon 0 (index −1) misses the table and takes the default.
    match w.wrapping_sub(1) {
        0 | 0x33..=0x36 | 0x39..=0x3F | 0x44 | 0x51 | 0x52 => 0x18,
        0x01..=0x0B | 0x40 | 0x41 => 0x16,
        0x0C => 0x1A,
        0x0D..=0x10 => 0x17,
        0x11..=0x14 | 0x29 | 0x2A | 0x32 => 0x15,
        0x15 | 0x17..=0x28 => 0x14,
        0x2B..=0x2E => 0x19,
        _ => 0x14, // the default arm [M]
    }
}

/// `BOTSTATE_isPreferredWeapon` 0x129580 — `class_pref` is the record's preferred-weapon-class
/// byte (`BOT_vars+0xAA`).
pub fn is_preferred_weapon(t: &BotWeaponTable, class_pref: u8, w: u8) -> bool {
    if w == 1 || w == 6 || class_pref == 0 {
        return true;
    }
    let c = t.row(w).class;
    match class_pref {
        1 => c == 1,
        2 => c == 2,
        3 => matches!(w, 3 | 7 | 0x14 | 0x24 | 0x25),
        4 => c == 3,
        5 => c == 4,
        _ => false,
    }
}

/// `BOT_handlePain` 0x125ed8:58-96 — the damage a hit does to a bot after the MP modifiers and the
/// armour: `× Plr_DMod_Multi` when the hit carries a location (`param_4 != −1`), the locational
/// factor (`MPSettings+0x1C8`, head 5 → ×4.0, the upper limbs 0x14/0x15/0x20/0x23 → ×0.8, the lower
/// limbs 0x31..0x38 → ×0.8), `× 3.0` when `MPSettings+0x1BC` is set; the armour absorbs pain types
/// 0 and > 7 only (`min(d, armour)`, the pool keeps the u8 remainder). Returns (health loss,
/// armour after). `location` = the bone id or −1; `multi` = the level's `Plr_DMod_Multi`
/// ([`DMOD_MULTI`] on every arena).
pub fn bot_pain(damage: f32, multi: f32, location: i16, pain_type: u16, armour: u8, locational: bool, triple: bool) -> (f32, u8) {
    let mut d = damage;
    if location != -1 {
        d *= multi;
    }
    if locational {
        match location - 5 {
            0 => d *= DMOD_HEAD,
            0xf | 0x10 | 0x1b | 0x1e => d *= DMOD_LIMB,
            0x2c..=0x33 => d *= DMOD_LIMB,
            _ => {}
        }
    }
    if triple {
        d *= 3.0;
    }
    let frac = if pain_type == 0 || pain_type > 7 { 1.0 } else { 0.0 };
    let a = armour as f32;
    let absorbed = (d * frac).min(a);
    ((d - absorbed), (a - absorbed) as u8)
}

/// `Player_HandlePain` 0x1902c8's MP arm for a bullet hit on a human (pain type 0 with a hit
/// location): `× Plr_DMod_Multi` [M :51-53] (the locational/×3 options as in [`bot_pain`] are off —
/// the port's settings carry no +0x1BC/+0x1C8), the armour (`BL+0x8B0`) absorbs `min(d, armour)`
/// [M :96-101], the health floors at 0 and anything below 1.0 reads 0 [M :103-106, 130-133].
pub fn player_pain_mp(health: &mut f32, armour: &mut f32, damage: f32, multi: f32) {
    if *health <= 0.0 || damage <= 0.0 {
        return; // [M :17-23]
    }
    let d = damage * multi;
    let absorbed = d.min(*armour);
    *armour -= absorbed;
    let mut h = *health - (d - absorbed);
    if h <= 0.0 {
        h = 0.0;
    }
    if h < 1.0 {
        h = 0.0;
    }
    *health = h;
}

/// What a pickup did for a bot (`Pickup_Handler`'s class-2 arm).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BotTake {
    /// Refused — the pickup stays (only an un-owned weapon whose `EquipWeapon` fails).
    Refused,
    /// Taken; `choice` = the `pickupWeaponChangeChoice` call's (has_weapon flag) — `None` when the
    /// game makes no choice call (the weapon is not loaded afterwards, or armour).
    Taken { choice: Option<bool> },
}

/// The `BOT_vars` weapon records of one bot.
#[derive(Clone, Debug, PartialEq)]
pub struct BotArsenal {
    /// `BOTWEAP_InitWeapon` ran (the port arms a bot on its first tick with a table).
    pub armed: bool,
    /// `+0x144 + w·0xC` (s16) — the loaded rounds per clip slot.
    pub clip: [i16; BOT_WEAPONS],
    /// `+0x146 + w·0xC` (u8) — the weapon is present.
    pub held: [bool; BOT_WEAPONS],
    /// `+0x698 + t·2` (s16) — the ammo pool per type.
    pub pool: [i16; AMMO_TYPES],
    /// `+0x769` (u8) — the armour pool (`Pickup_Handler` adds, capped at 50; `BOT_handlePain` drains).
    /// The game's armour-pickup case applies the add inline and then calls
    /// `BOTWEAP_setDroneArmour` 0x12af70 — an empty stub in the shipped binary (`jr ra; nop`,
    /// 8 bytes; its only caller, `Pickup_Handler` 0x18e650:133, reached on every armour take) —
    /// so the port's inline add with no setter call is the whole behaviour [M].
    pub armour: u8,
    /// The match's `startweap` global (copied per bot so the arsenal is self-contained).
    pub start_weapon: u8,
}

impl Default for BotArsenal {
    fn default() -> Self {
        BotArsenal { armed: false, clip: [0; BOT_WEAPONS], held: [false; BOT_WEAPONS], pool: [0; AMMO_TYPES], armour: 0, start_weapon: 0 }
    }
}

impl BotArsenal {
    fn pool_mut(&mut self, t: u8) -> &mut i16 {
        let i = (t as usize).min(AMMO_TYPES - 1);
        &mut self.pool[i]
    }
    fn pool_of(&self, t: u8) -> i16 { self.pool.get(t as usize).copied().unwrap_or(0) }
    fn held_of(&self, w: u8) -> bool { self.held.get(w as usize).copied().unwrap_or(false) }

    /// `BOTWEAP_InitWeapon` 0x129e48: every record cleared, the fists with 999 rounds, the start
    /// weapon with two clips, the current weapon := the start weapon; the bomb for the attackers
    /// (Protection team 0 / Demolition team 1 [M :50-56]); character 0x1A gets weapon 0x45 and
    /// holds it [M :57-62]; then `CheckWeaponsLoaded`. Returns the current weapon (`+0x768`).
    pub fn init(&mut self, t: &BotWeaponTable, start: u8, scenario: u32, team: u32, character: u8) -> u8 {
        *self = BotArsenal::default();
        self.armed = true;
        self.start_weapon = start;
        self.equip_weapon(t, FISTS, 999);
        let clip2 = (t.row(start).clip as i32).wrapping_shl(1) as i16; // `(sVar1 << 0x11) >> 0x10`
        self.equip_weapon(t, start, clip2);
        let mut current = start;
        let attacker = (scenario == crate::mp::scenario::PROTECTION && team == 0)
            || (scenario == crate::mp::scenario::DEMOLITION && team == 1);
        if attacker {
            self.equip_weapon(t, BOMB, 1);
        }
        if character == 0x1a {
            self.equip_weapon(t, 0x45, 1);
            current = 0x45;
        }
        self.check_weapons_loaded(t);
        current
    }

    /// `BOTWEAP_CheckWeaponsLoaded` 0x12aec0: a selectable weapon without a first-person skin (or,
    /// except the fists, whose skin is not loaded) is not present.
    pub fn check_weapons_loaded(&mut self, t: &BotWeaponTable) {
        for w in 0..BOT_WEAPONS {
            let r = t.row(w as u8);
            if r.selectable && !r.usable {
                self.held[w] = false;
            }
        }
    }

    /// `BOTWEAP_EquipWeapon` 0x12a138.
    pub fn equip_weapon(&mut self, t: &BotWeaponTable, w: u8, rounds: i16) -> bool {
        let r = t.row(w);
        if w != FISTS && !r.usable {
            return false; // `+0xDC == 0` / `hashtable_getitem` [M :18-25]
        }
        if (w as usize) >= BOT_WEAPONS || (r.base as usize) >= BOT_WEAPONS {
            return false;
        }
        if !self.held[r.base as usize] {
            self.held[r.base as usize] = true; // [M :28-29]
            self.held[w as usize] = true;
            let slot = t.clip_slot(w);
            self.clip[slot] = if (rounds as i32) < r.clip as i32 { rounds } else { r.clip }; // [M :34-38]
            let extra = (rounds as i32 - r.clip as u16 as i32) as i16; // `(p3 − clip) · 0x10000 >> 16`
            if extra > 0 {
                let p = self.pool_mut(r.ammo_type);
                *p = p.wrapping_add(extra);
            }
        } else {
            if self.pool_of(r.ammo_type) >= t.cap(r.ammo_type) || rounds == 0 {
                return false; // [M :45-49]
            }
            let p = self.pool_mut(r.ammo_type);
            *p = p.wrapping_add(rounds);
        }
        let cap = t.cap(r.ammo_type);
        let p = self.pool_mut(r.ammo_type);
        *p = (*p).min(cap); // [M :52-58]
        true
    }

    /// `BOTWEAP_EquipAmmo` 0x12a308.
    pub fn equip_ammo(&mut self, t: &BotWeaponTable, w: u8, rounds: i16) -> bool {
        let r = t.row(w);
        if self.pool_of(r.ammo_type) >= t.cap(r.ammo_type) || rounds == 0 {
            return false; // [M :17-20]
        }
        let mut rest = rounds as i32;
        if r.flags & 0x10 != 0 {
            let slot = t.clip_slot(w);
            if slot < BOT_WEAPONS && self.clip[slot] == 0 {
                self.clip[slot] = if (rounds as i32) < r.clip as i32 { rounds } else { r.clip }; // [M :29-38]
            }
            rest = ((rest - r.clip as u16 as i32) as i16) as i32; // [M :42-46]
            if rest < 1 {
                return true;
            }
        }
        let cap = t.cap(r.ammo_type);
        let p = self.pool_mut(r.ammo_type);
        *p = p.wrapping_add(rest as i16).min(cap); // [M :49-57]
        true
    }

    /// `BOTWEAP_AmmoInGun` 0x12ae40: the start weapon always reads 1 [M :10-12].
    pub fn ammo_in_gun(&self, t: &BotWeaponTable, w: u8) -> i16 {
        if w == self.start_weapon {
            return 1;
        }
        self.clip.get(t.clip_slot(w)).copied().unwrap_or(0)
    }

    /// `BOTWEAP_WeaponHasAmmo` 0x12adb0: a pool, rounds in the gun, an ammo-less weapon (type 0) or
    /// the start weapon.
    pub fn weapon_has_ammo(&self, t: &BotWeaponTable, w: u8) -> bool {
        let at = t.row(w).ammo_type;
        self.pool_of(at) != 0 || self.ammo_in_gun(t, w) != 0 || at == 0 || w == self.start_weapon
    }

    /// `BOTWEAP_hasWeapon` 0x12af78 — also the whole body of `BOTSTATE_hasWeapon` 0x129658
    /// (the BOT_vars-typed forwarder: `BOTWEAP_hasWeapon(*(BOT_vars+0x754), w)` [M]).
    pub fn has_weapon(&self, w: u8) -> bool { self.held_of(w) }

    /// `BOTWEAP_hasLoadedWeapon` 0x12b038.
    pub fn has_loaded_weapon(&self, t: &BotWeaponTable, w: u8) -> bool {
        self.held_of(w) && self.weapon_has_ammo(t, w)
    }

    /// `BOTWEAP_getWeaponAmmoAmount` 0x12afa0: the clip slot's rounds + the pool.
    pub fn weapon_ammo_amount(&self, t: &BotWeaponTable, w: u8) -> i32 {
        self.clip.get(t.clip_slot(w)).copied().unwrap_or(0) as i32 + self.pool_of(t.row(w).ammo_type) as i32
    }

    /// `BOTWEAP_ReloadAmmoType` 0x129d28 for the current weapon `w`. Returns (reloaded, the new
    /// `Drone+0xBBC` rounds-in-gun when written). The start weapon with an empty pool refills the
    /// gun's count to a full clip without touching the clip slot [M :27-34] — the game's own
    /// endless start weapon. `dry` = the `param_2` probe (no writes).
    pub fn reload(&mut self, t: &BotWeaponTable, w: u8, dry: bool) -> (bool, Option<i16>) {
        let r = t.row(w);
        let slot = t.clip_slot(w);
        let have = self.pool_of(r.ammo_type);
        if have < 1 {
            if w == self.start_weapon {
                return (true, Some(r.clip));
            }
            return (false, None);
        }
        let in_clip = self.clip.get(slot).copied().unwrap_or(0);
        let mut n = r.clip as i32 - in_clip as i32;
        if n > 0 && r.flags & 0x100 != 0 {
            n = 1; // [M :38-40]
        }
        if (have as i32) < n {
            n = have as i32;
        }
        if n == 0 {
            return (false, None);
        }
        if dry {
            return (true, None);
        }
        let p = self.pool_mut(r.ammo_type);
        *p = p.wrapping_sub(n as i16);
        let c = in_clip.wrapping_add(n as i16);
        if slot < BOT_WEAPONS {
            self.clip[slot] = c;
        }
        (true, Some(c))
    }

    /// `BOTWEAP_decrRounds` 0x12a470 for the held weapon `w`: an empty clip only "fires" for the
    /// start weapon; else the clip drops by `n` (floored at 0).
    pub fn decr_rounds(&mut self, t: &BotWeaponTable, w: u8, n: i16) -> bool {
        let slot = t.clip_slot(w);
        let Some(c) = self.clip.get_mut(slot) else { return false };
        if *c == 0 {
            return w == self.start_weapon;
        }
        *c = (*c).wrapping_sub(n).max(0);
        true
    }

    /// `BOTWEAP_listHeldLoadedWeapons` 0x12aa80 + the `QuickSort` of `combatWeaponChangeChoice`
    /// (comparator 0x1297a8: the low byte ascending — disasm): the held, valid, loaded weapons
    /// (only the preferred ones + the fists when `preferred_only`) as (rank byte, weapon), best
    /// first. The rank byte is `0xFE − (100 − idx)` = `154 + idx` with `idx` the weapon's position
    /// in [`PREFERENCE`] (60 when absent; the fists +1) [M :127-150]. Equal ranks keep the weapon
    /// order [S — the game's QuickSort is not stable; only weapons absent from the table tie].
    pub fn list_held_loaded(&self, t: &BotWeaponTable, preferred_only: bool, class_pref: u8) -> Vec<(u8, u8)> {
        let mut out = Vec::new();
        for w in 0..BOT_WEAPONS as u8 {
            if !self.held_of(w) || !is_valid_weapon(w) || !self.weapon_has_ammo(t, w) {
                continue;
            }
            if preferred_only && !(is_preferred_weapon(t, class_pref, w) || w == FISTS) {
                continue;
            }
            let mut idx = PREFERENCE.iter().position(|&p| p == w).unwrap_or(60) as u8;
            if w == FISTS {
                idx += 1;
            }
            out.push((154u8.wrapping_add(idx), w));
        }
        out.sort_by_key(|&(rank, w)| (rank, w));
        out
    }

    /// `BOTSTATE_combatWeaponChangeChoice` 0x128708 (disasm 0x128708-0x128b48): 0 with no visible
    /// opponent (`Drone+0x228 & 4`) or nothing loaded; the fists when only they are loaded; trait 5
    /// takes the best unless it is the bomb; otherwise the best-ranked weapon, an explosive only
    /// when no non-explosive gun ranks below it or the opponent is beyond its min range. The
    /// missile reorder (`opponentIsMissile`, the GoldenEye satellite) and the demolition-site bomb
    /// test are not modelled [S]. An invalid pick reads as the fists [M 0x128ac8-0x128ad4].
    pub fn combat_choice(&self, t: &BotWeaponTable, visible: bool, dist: f32, trait_id: u8, class_pref: u8) -> u8 {
        if !visible {
            return 0;
        }
        let mut list = self.list_held_loaded(t, true, class_pref);
        if list.len() < 2 {
            list = self.list_held_loaded(t, false, class_pref);
            if list.is_empty() {
                return 0;
            }
        }
        let n = list.len();
        let w = if n < 2 {
            FISTS
        } else if trait_id == 5 {
            list[if list[0].1 == BOMB { 1 } else { 0 }].1
        } else {
            let mut pick = n.saturating_sub(2);
            let mut gun_below = false; // s7: a non-explosive, non-fist gun ranked worse
            for i in (0..n).rev() {
                let w = list[i].1;
                if t.row(w).class != 4 {
                    if w != FISTS {
                        gun_below = true;
                    }
                    pick = i;
                } else if !gun_below || dist > t.row(w).min_range {
                    pick = i; // c.ole.s dist, min_range → keep the earlier pick [disasm 0x128a7c]
                }
            }
            list[pick].1
        };
        if is_valid_weapon(w) { w } else { FISTS }
    }

    /// `BOTWEAP_tooCloseForWeapon` 0x12a960 (disasm): trait 5 never; otherwise true when some
    /// held, loaded non-fist non-explosive weapon exists and the opponent is inside `w`'s min range.
    pub fn too_close_for_weapon(&self, t: &BotWeaponTable, w: u8, dist: f32, trait_id: u8) -> bool {
        if trait_id == 5 {
            return false;
        }
        let other = (0..BOT_WEAPONS as u8).any(|x| {
            self.held_of(x) && is_valid_weapon(x) && self.weapon_has_ammo(t, x) && x != FISTS && t.row(x).class != 4
        });
        other && dist < t.row(w).min_range
    }

    /// `BOTWEAP_hasLoadedExplosiveForRange` 0x12a538: the first held, loaded explosive of
    /// 0x2D, 0x2C, 0x2A, 0x33, 0x2B, 0x2E, 0x2F whose min range is below `dist`, else 0.
    pub fn explosive_for_range(&self, t: &BotWeaponTable, dist: f32) -> u8 {
        for w in [0x2d, 0x2c, 0x2a, 0x33, 0x2b, 0x2e, 0x2f] {
            if self.held_of(w) && self.weapon_has_ammo(t, w) && t.row(w).min_range < dist {
                return w;
            }
        }
        0
    }

    /// `BOTWEAP_changeWeapon` 0x12a028: refused unless present; the new current weapon and the
    /// rounds-in-gun count (the clip slot's, 1 when empty [M :48-52]). Returns the new
    /// `Drone+0xBBC`.
    pub fn change_weapon(&self, t: &BotWeaponTable, w: u8) -> Option<i16> {
        if !self.held_of(w) {
            return None;
        }
        let c = self.clip.get(t.clip_slot(w)).copied().unwrap_or(0);
        Some(if c == 0 { 1 } else { c })
    }

    /// `Pickup_Handler` 0x18e650:34-130, the class-2 (drone/bot) arm. `kind`/`id`/`rounds` are the
    /// PICKUPINFO's +0x22/+0x24/+0x26; `team` the bot's team word; `health`/`max_hp` the drone's
    /// +0xAC and `BOT_vars+0xA4`. The id remap 0xD ↔ 10 by team [M :42-55]. A weapon the bot lacks
    /// is equipped (refused → the pickup stays); one it has gives ammo (`EquipAmmo`'s result is
    /// not read — the pickup is consumed either way [M :88-94]); ammo the same; armour adds 20 HP
    /// (clamped to the max) and its rounds to the armour pool (capped at 50) [M :104-118]. The
    /// one-gun eviction under `MPSettings+0x1BC` [M :60-82] is not modelled (the port's settings
    /// carry no +0x1BC) [S].
    pub fn take_pickup(&mut self, t: &BotWeaponTable, kind: u16, id: u16, rounds: i16, team: u32, health: &mut f32, max_hp: u16) -> BotTake {
        let mut id = id;
        if team != 0 {
            if id == 0xd {
                id = 10;
            }
        } else if id == 10 && self.held_of(10) {
            id = 0xd;
        }
        let w = id.min(0xff) as u8;
        match kind {
            0 => {
                if !self.held_of(w) {
                    if !self.equip_weapon(t, w, rounds) {
                        return BotTake::Refused;
                    }
                    let choice = self.has_loaded_weapon(t, w).then_some(false);
                    BotTake::Taken { choice }
                } else {
                    self.equip_ammo(t, w, rounds);
                    let choice = self.has_loaded_weapon(t, w).then_some(true);
                    BotTake::Taken { choice }
                }
            }
            1 => {
                self.equip_ammo(t, w, rounds);
                let choice = self.has_loaded_weapon(t, w).then_some(true);
                BotTake::Taken { choice }
            }
            3 => {
                let h = *health + 20.0;
                *health = if h < 0.0 { 0.0 } else if h > max_hp as f32 { max_hp as f32 } else { h };
                let a = self.armour.wrapping_add(rounds as u8);
                self.armour = if a > 0x32 { 0x32 } else { a };
                BotTake::Taken { choice: None }
            }
            // kinds 2, 4.. — no bot effect; the take tail still runs (the sound, the respawn)
            // [M :119-121 → switchD_0018e728_caseD_2]; the visit stamp is skipped.
            _ => BotTake::Taken { choice: None },
        }
    }

    /// The arsenal's lockstep fold.
    pub fn hash_into(&self, h: &mut crate::mp::hash::Hasher) {
        h.bool(self.armed);
        for c in self.clip { h.i16(c); }
        for b in self.held { h.bool(b); }
        for p in self.pool { h.i16(p); }
        h.u8(self.armour);
        h.u8(self.start_weapon);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic table: the start pistol 6 (type 2, clip 16, cap 90), the AR 0x16 (type 5, clip
    /// 30), the rocket launcher 0x2c (type 14, clip 4, cap 4, min range 8, class 4), the fists.
    fn table() -> BotWeaponTable {
        let mut rows = vec![BotWeaponRow::default(); BOT_WEAPONS];
        let mk = |base: u8, class: u8, at: u8, clip: i16, minr: f32| BotWeaponRow {
            base, selectable: true, twin: false, class, min_range: minr, damage: 5.0, bullets: 1, range: 50.0,
            spread: 0.0, burst: 1, interval: 8, flags: 0, ammo_type: at, clip, usable: true,
        };
        rows[1] = mk(1, 0, 0, 1, 0.0);
        rows[6] = mk(6, 1, 2, 16, 0.0);
        rows[0x16] = mk(0x16, 2, 5, 30, 0.0);
        rows[0x2c] = mk(0x2c, 4, 14, 4, 8.0);
        let mut ammo_cap = vec![0i16; AMMO_TYPES];
        ammo_cap[2] = 90;
        ammo_cap[5] = 90;
        ammo_cap[14] = 4;
        BotWeaponTable { rows, ammo_cap }
    }

    /// `BOTWEAP_InitWeapon`: fists + the start weapon with two clips (one loaded, one in the pool);
    /// the start weapon reads 1 round in the gun forever and reloads from nothing [M 0x129e48,
    /// 0x12ae40, 0x129d28:27-34].
    #[test]
    fn init_and_endless_start_weapon() {
        let t = table();
        let mut a = BotArsenal::default();
        let cur = a.init(&t, 6, crate::mp::scenario::DEATHMATCH, 0, 0);
        assert_eq!(cur, 6);
        assert!(a.held[1] && a.held[6] && !a.held[0x16]);
        assert_eq!(a.clip[6], 16);
        assert_eq!(a.pool[2], 16, "the second clip goes to the pool");
        // empty the clip, reload from the pool, empty again, reload from nothing (start weapon)
        for _ in 0..16 { assert!(a.decr_rounds(&t, 6, 1)); }
        assert_eq!(a.clip[6], 0);
        assert_eq!(a.reload(&t, 6, false), (true, Some(16)));
        assert_eq!(a.pool[2], 0);
        for _ in 0..16 { a.decr_rounds(&t, 6, 1); }
        assert_eq!(a.reload(&t, 6, false), (true, Some(16)), "the start weapon's endless refill");
        assert_eq!(a.clip[6], 0, "…without touching the clip slot");
        assert!(a.decr_rounds(&t, 6, 1), "an empty start weapon still fires");
        // Sister: a non-start weapon with nothing left cannot reload or fire.
        assert!(a.equip_weapon(&t, 0x16, 30));
        for _ in 0..30 { a.decr_rounds(&t, 0x16, 1); }
        assert_eq!(a.reload(&t, 0x16, false), (false, None));
        assert!(!a.decr_rounds(&t, 0x16, 1));
        assert!(!a.weapon_has_ammo(&t, 0x16));
    }

    /// The choice: the AR outranks the pistol (PREFERENCE); the rocket launcher outranks both but
    /// is skipped inside its 8.0 min range while a gun is loaded [disasm 0x1289c0-0x128ab4].
    #[test]
    fn combat_choice_ranks_and_min_range() {
        let t = table();
        let mut a = BotArsenal::default();
        a.init(&t, 6, crate::mp::scenario::DEATHMATCH, 0, 0);
        assert_eq!(a.combat_choice(&t, true, 10.0, 0, 0), 6, "pistol + fists → the pistol");
        assert_eq!(a.combat_choice(&t, false, 10.0, 0, 0), 0, "no visible opponent → no choice");
        a.equip_weapon(&t, 0x16, 60);
        assert_eq!(a.combat_choice(&t, true, 10.0, 0, 0), 0x16);
        a.equip_weapon(&t, 0x2c, 4);
        assert_eq!(a.combat_choice(&t, true, 20.0, 0, 0), 0x2c, "far: the launcher");
        assert_eq!(a.combat_choice(&t, true, 5.0, 0, 0), 0x16, "inside 8.0: the AR");
        assert!(a.too_close_for_weapon(&t, 0x2c, 5.0, 0));
        assert!(!a.too_close_for_weapon(&t, 0x2c, 5.0, 5), "trait 5 is never too close");
        assert_eq!(a.explosive_for_range(&t, 20.0), 0x2c);
        assert_eq!(a.explosive_for_range(&t, 5.0), 0);
    }

    /// `Pickup_Handler`'s bot arm: an owned weapon's pickup is ammo and always consumed; armour
    /// caps at 50 and adds 20 HP to the max; an unusable weapon is refused.
    #[test]
    fn take_pickup_paths() {
        let mut t = table();
        let mut a = BotArsenal::default();
        a.init(&t, 6, crate::mp::scenario::DEATHMATCH, 0, 0);
        let mut hp = 50.0;
        assert_eq!(a.take_pickup(&t, 0, 0x16, 60, 0, &mut hp, 100), BotTake::Taken { choice: Some(false) });
        assert_eq!((a.clip[0x16], a.pool[5]), (30, 30));
        assert_eq!(a.take_pickup(&t, 0, 0x16, 60, 0, &mut hp, 100), BotTake::Taken { choice: Some(true) });
        assert_eq!(a.pool[5], 90, "clamped to the cap");
        assert_eq!(a.take_pickup(&t, 1, 0x16, 60, 0, &mut hp, 100), BotTake::Taken { choice: Some(true) }, "a full pool still consumes");
        assert_eq!(a.take_pickup(&t, 3, 0, 40, 0, &mut hp, 100), BotTake::Taken { choice: None });
        assert_eq!((a.armour, hp), (40, 70.0));
        a.take_pickup(&t, 3, 0, 40, 0, &mut hp, 100);
        assert_eq!((a.armour, hp), (50, 90.0));
        a.take_pickup(&t, 3, 0, 40, 0, &mut hp, 100);
        assert_eq!(hp, 100.0, "clamped to the max HP");
        t.rows[0x2c].usable = false;
        assert_eq!(a.take_pickup(&t, 0, 0x2c, 4, 0, &mut hp, 100), BotTake::Refused);
    }

    /// `BOT_handlePain`: ×4.0 (MP) then the armour absorbs; pain types 1..7 bypass the armour; the
    /// locational head factor ×4.0.
    #[test]
    fn pain_multi_and_armour() {
        assert_eq!(bot_pain(5.0, DMOD_MULTI, 0, 0, 0, false, false), (20.0, 0));
        assert_eq!(bot_pain(5.0, DMOD_MULTI, 0, 0, 50, false, false), (0.0, 30));
        assert_eq!(bot_pain(5.0, DMOD_MULTI, 0, 0, 12, false, false), (8.0, 0));
        assert_eq!(bot_pain(5.0, DMOD_MULTI, 0, 3, 50, false, false), (20.0, 50), "type 3 bypasses the armour");
        assert_eq!(bot_pain(5.0, DMOD_MULTI, -1, 0, 0, false, false), (5.0, 0), "no location → no ×4");
        assert_eq!(bot_pain(5.0, DMOD_MULTI, 5, 0, 0, true, false), (80.0, 0), "head with locational damage");
        assert_eq!(bot_pain(5.0, DMOD_MULTI, 5, 0, 0, false, true), (60.0, 0), "the ×3 flag");
        // the human arm: ×4, the armour first, below 1.0 → 0
        let (mut h, mut a) = (100.0, 10.0);
        player_pain_mp(&mut h, &mut a, 5.0, DMOD_MULTI);
        assert_eq!((h, a), (90.0, 0.0));
        let (mut h, mut a) = (20.5, 0.0);
        player_pain_mp(&mut h, &mut a, 5.0, DMOD_MULTI);
        assert_eq!(h, 0.0, "0.5 left reads as dead");
    }

    #[test]
    fn valid_and_punch_sets() {
        assert!(is_valid_weapon(1) && is_valid_weapon(0x16) && !is_valid_weapon(0x17) && !is_valid_weapon(0x30));
        assert!(is_valid_weapon(0x45) && !is_valid_weapon(0x46) && is_valid_weapon(0x52));
        assert!(punch_is_better_if_close(0x3b) && !punch_is_better_if_close(0x16) && !punch_is_better_if_close(0x38));
    }

    /// `BOTSTATE_isPreferredWeapon` 0x129580: the fists (1) and the P2K (6) are always preferred
    /// whatever the record's class pref (`BOT_vars+0xAA`), pref 0 prefers everything, prefs 1/2/4/5
    /// test the weapon's class byte (`weapon_data+0x06`), pref 3 is the hardcoded gadget set
    /// {3, 7, 0x14, 0x24, 0x25}, an unmapped pref refuses [M 0x129580:8-52].
    #[test]
    fn is_preferred_weapon_table() {
        let t = table();
        // the always-preferred ids and the everything pref [M :10-16]
        for pref in 0..=8u8 {
            assert!(is_preferred_weapon(&t, pref, 1), "the fists are always preferred (pref {pref})");
            assert!(is_preferred_weapon(&t, pref, 6), "the P2K is always preferred (pref {pref})");
        }
        assert!(is_preferred_weapon(&t, 0, 0x2c), "pref 0 prefers everything");
        // the class prefs against the fixture's classes (0x16 class 2, 0x2c class 4)
        assert!(is_preferred_weapon(&t, 2, 0x16));
        assert!(!is_preferred_weapon(&t, 1, 0x16), "class 2 is not pref 1");
        assert!(is_preferred_weapon(&t, 5, 0x2c));
        assert!(!is_preferred_weapon(&t, 2, 0x2c), "class 4 is not pref 2");
        // pref 3's hardcoded set [M :34-44]
        for w in [3u8, 7, 0x14, 0x24, 0x25] {
            assert!(is_preferred_weapon(&t, 3, w), "pref 3 gadget {w:#x}");
        }
        assert!(!is_preferred_weapon(&t, 3, 0x16), "the AR is not a pref-3 gadget");
        // Sister (can fail): an unmapped class pref prefers nothing but the fists/P2K.
        assert!(!is_preferred_weapon(&t, 6, 0x16));
    }
}
