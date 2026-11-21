//! The driving engine's vehicle damage and health model (DRIVING.ELF), headless: the player's
//! health/armour words at mission+0x378..+0x384 and where their numbers come from, the car's
//! `InflictDamage` virtual (the armour soak, the hit points, the 16 damage zones the car
//! microcode's `DamageZones` reads, the tyres, the wreck), the collision damage of the rigid
//! pass, the health/armour power-ups, the low-health regeneration, the flipped-car damage, the
//! `SRuleDamage`/`SRuleHealth` mission rules and the `EPlayerLose` exit code.
//! docs/systems/driving-damage.md is the write-up with the per-function table.
//!
//! Sources (DRIVING.ELF, decompile under work/decomp/DRIVING.ELF/, addresses = EE):
//! - `FUN_00206ec0` sets the player's health/armour from the mission set; `FUN_00206810` /
//!   `FUN_00206850` / `FUN_00206890` add health/armour; `FUN_00206b20` / `FUN_00206b40` read the
//!   ratios; `FUN_002068b8` books damage taken; `FUN_00207228` regenerates [M].
//! - `FUN_00195a08` points a car's hit-point word (+0x54) at a float: the player car at
//!   mission+0x378 (`FUN_001566b0:70`, `FUN_00176658:232`), every other car at its own +0x42c
//!   (700.0, `FUN_001862c0:217-218`); `FUN_001931f0` (vtable +0x20c, called by `FUN_00205a90`)
//!   points the car's armour word (+0x434) at mission+0x380 [M].
//! - `FUN_00189598` is `PBondCar`'s (and `PTank`'s) vtable +0x1c, the damage virtual
//!   (vtables 0x379ef0 / 0x37a9f8, typeinfo names "8PBondCar" / "5PTank") [M].
//! - `FUN_001fc198` (world contact) and `FUN_001fbbd8` (car against car) turn a contact into a
//!   damage call [M]; `FUN_002db798` / `FUN_002db930` pick the damage zone [M + ELF data].
//! - `FUN_00209ff0` / `FUN_0020a1f8` are the `SRuleDamage` / `SRuleHealth` tests, `FUN_00209820`
//!   their value read [M]; `FUN_00154cb0` is `EPlayerLose`'s handler [M].
//!
//! Inputs this module takes instead of computing (the caller's, and named in the doc): the
//! segment-against-box test (`FUN_002d3950`), the world→car transform, the random draws
//! (`FUN_0017dc28`, one 16-bit value per call, used as value / 65536), the HUD object.

// ---------------------------------------------------------------------------------- constants

/// `FUN_00206ec0`'s health when the mission set's +0x280 word is 0 (0x461c4000) [M].
pub const DEFAULT_HEALTH: f32 = 10_000.0;
/// `FUN_00206ec0`'s armour maximum on the same path (0x447a0000); the armour itself is 0 [M].
pub const DEFAULT_ARMOUR_MAX: f32 = 1_000.0;
/// A non-player car's own hit points, +0x42c = 0x442f0000 [M FUN_001862c0:217, FUN_00187578:29].
pub const CAR_HIT_POINTS: f32 = 700.0;
/// The scale the first hit fixes: +0x2dc = 32 / hit points (starts at −1.0) [M FUN_00189598,
/// FUN_001862c0's 0xbf800000 store].
pub const ZONE_SCALE_NUMERATOR: f32 = 32.0;
/// `TYRE_DAMAGE_RADIUS` (Physics:Physical 0x3319bc = 1.25) [M FUN_00185668:20 + ELF data].
pub const TYRE_DAMAGE_RADIUS: f32 = 1.25;
/// `DAMAGE_SCALE_COLLISION` (Physics:Physical 0x3319b4 = 1.0): registered by `FUN_00185668:26`
/// but read by no function [M-negative: the decompile, and a scan of every load/store whose low
/// half is 0x19b4, finds only the registration at 0x18592c]. Kept for the record; unused.
pub const DAMAGE_SCALE_COLLISION: f32 = 1.0;
/// The contact damage scale, .data 0x33916c = 5.0 (no tuner name) [M FUN_001fc198, FUN_001fbbd8].
pub const COLLISION_DAMAGE_SCALE: f32 = 5.0;
/// The world-contact extra for car bodies (kind < 4), .data 0x339170 = 6.0 [M FUN_001fc198].
pub const CAR_WORLD_CONTACT_SCALE: f32 = 6.0;
/// The dent share of a world contact, .data 0x33915c = 0.35 [M FUN_001fc198].
pub const WORLD_CONTACT_DENT: f32 = 0.35;
/// The dent share of a car-against-car contact, .data 0x339168 = 0.35 [M FUN_001fbbd8].
pub const CAR_CONTACT_DENT: f32 = 0.35;
/// The weapon-hit push, .data 0x331a58 = 1000.0 [M FUN_00189598].
pub const HIT_PUSH: f32 = 1_000.0;
/// The impact-sound buckets (0x332068..0x332074) and their event ids [M FUN_00189598 + data].
pub const SEVERITY: [(f32, u32); 4] = [(0.75, 6), (0.5, 5), (0.25, 4), (0.001, 0x19)];
/// The zone stage thresholds 0x332080..0x33208c; stage k fires event 0x13 + k [M + data].
pub const STAGES: [f32; 4] = [0.15, 0.35, 0.65, 0.85];
/// `PBondCar`'s "model call 10" level (0x332084 = 0.35) [M FUN_00189598]. `PHelicopter`
/// (`FUN_00193938`) reads 0x332080 = 0.15 for the same call instead [M].
pub const MODEL10_LEVEL: f32 = 0.35;
/// The "model call 9" level (0x332090 = 0.75) [M].
pub const MODEL9_LEVEL: f32 = 0.75;
/// The neighbour-spread level 0x332094: .data holds 0.75 but every car reset `FUN_00187350`
/// stores 0.0 there (`swc1 $f0, 0x2094($v1)` at 0x1873f4 after `mtc1 $zero, $f0` at 0x18739c),
/// and the car constructor calls that reset, so in play it reads 0.0 [M disasm].
pub const SPREAD_LEVEL_IN_PLAY: f32 = 0.0;
/// The weights 0x379d20: a neighbour zone takes 0.2 of the hit, the struck zone 1.0 [M + data].
pub const ZONE_WEIGHT: [f32; 2] = [0.2, 1.0];
/// The tyre blow-out events 0x379cc8 = 1, 2, 3, 4 (tyre 0..3) [M + data].
pub const TYRE_BLOWOUT_EVENT: [u32; 4] = [1, 2, 3, 4];

// ---------------------------------------------------------------------------- the mission set

/// The health/armour words of the mission set (`MSet` payload, or the built-in record at
/// 0x4162b8) that `FUN_00206ec0` reads [M].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HealthSet {
    /// +0x27c: the health maximum.
    pub health_max: i32,
    /// +0x280: the starting health; 0 selects the 10000/10000/1000/0 defaults.
    pub health: i32,
    /// +0x284: the armour maximum (also the base `FUN_00206890` multiplies).
    pub armour_max: i32,
    /// +0x288: the starting armour.
    pub armour: i32,
    /// +0x294 / +0x298 / +0x29c: the health percent for difficulty 0 (else), 1, 2. +0x294 > 0 is
    /// also the gate for the whole difficulty step.
    pub health_pct: [i32; 3],
    /// +0x2a0 / +0x2a4 / +0x2a8: the armour percent for difficulty 0 (else), 1, 2.
    pub armour_pct: [i32; 3],
}

impl HealthSet {
    /// The built-in record used when the track has no `MSet`: `FUN_00204de8` zeroes 0x2f8
    /// bytes at 0x4162b8 and stores 3000 (+0x27c), 3000 (+0x280), 1000 (+0x284), 0 (+0x288)
    /// [M FUN_00204de8: DAT_00416534/38/3c/40].
    pub const BUILT_IN: HealthSet = HealthSet {
        health_max: 3000, health: 3000, armour_max: 1000, armour: 0, health_pct: [0; 3], armour_pct: [0; 3],
    };

    pub fn parse(mset: &[u8]) -> HealthSet {
        let le = |o: usize| i32::from_le_bytes(mset.get(o..o + 4).map_or([0; 4], |b| b.try_into().unwrap()));
        HealthSet {
            health_max: le(0x27c),
            health: le(0x280),
            armour_max: le(0x284),
            armour: le(0x288),
            health_pct: [le(0x294), le(0x298), le(0x29c)],
            armour_pct: [le(0x2a0), le(0x2a4), le(0x2a8)],
        }
    }
}

/// What `FUN_00206ec0` reads besides the set [M].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StartInputs {
    /// mission+0x3b8 = PTP+0x984 at construction (`FUN_00204de8`: piVar7[0xee] = iRam00100984).
    pub difficulty: i32,
    /// `DAT_0033a008 || DAT_0033a00c`: health and its maximum ×100. No writer of either word
    /// is in the decompile.
    pub times_100: bool,
    /// PTP+0x61c / +0x620: the health / armour ratios the previous drive left
    /// (`FUN_00205c30` writes them from `FUN_00206b20` / `FUN_00206b40`); > 0 applies.
    pub carry_health: f32,
    pub carry_armour: f32,
    /// mission+0x5f4: mission loads before this one (`FUN_00205220:99` adds 1 after the set-up).
    pub loads_before: i32,
    /// PTP+0x9a0 option bits (0x200 / 0x400 / 0x800: armour ×2 / ×3 / ×4).
    pub option_bits: u32,
}

/// The player's health block, mission+0x378..+0x3e0 [M].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerHealth {
    /// +0x378 / +0x37c / +0x380 / +0x384.
    pub health: f32,
    pub health_max: f32,
    pub armour: f32,
    pub armour_max: f32,
    /// +0x3c8: damage booked against the player car (summed into the PTP stats at +0x5ac).
    pub damage_taken: f32,
    /// +0x3e0: the tick of the last booked hit.
    pub last_hit: i32,
    /// +0x3cc: the control-lock bits (`FUN_002065d0` sets, `FUN_00206620` clears); while any is
    /// set no damage is booked and the player car's hits do not take hit points.
    pub lock_bits: u32,
    /// +0x3c4: cars destroyed (`FUN_00206960`, clamped at 0).
    pub kills: i32,
    /// +0x3bc / +0x3c0 / +0x5f0: the group-target counters `FUN_002068e0` / `FUN_00206900` keep.
    pub group_total: i32,
    pub group_hits: i32,
    pub group_tick: i32,
}

impl PlayerHealth {
    /// `FUN_00206ec0` [M], in its order.
    pub fn start(set: &HealthSet, inp: &StartInputs) -> PlayerHealth {
        let mut p = PlayerHealth::default();
        if set.health == 0 {
            p.health_max = DEFAULT_HEALTH;
            p.armour_max = DEFAULT_ARMOUR_MAX;
            p.health = DEFAULT_HEALTH;
            p.armour = 0.0;
        } else {
            p.health = set.health as f32;
            p.health_max = set.health_max as f32;
            p.armour = set.armour as f32;
            p.armour_max = set.armour_max as f32;
        }
        if inp.times_100 {
            p.health *= 100.0;
            p.health_max *= 100.0;
        }
        if inp.carry_health > 0.0 {
            p.health *= inp.carry_health;
        }
        if inp.carry_armour > 0.0 {
            p.armour = (p.armour_max * inp.carry_armour).max(p.armour);
        }
        if inp.loads_before >= 1 {
            p.health = (p.health_max * 0.5).max(p.health);
        }
        if set.health_pct[0] > 0 {
            let k = match inp.difficulty { 1 => 1, 2 => 2, _ => 0 };
            let hp = set.health_pct[k] as f32 / 100.0;
            let ap = set.armour_pct[k] as f32 / 100.0;
            p.health *= hp;
            p.health_max *= hp;
            p.armour *= ap;
            p.armour_max *= ap;
        }
        for (bit, k) in [(0x200u32, 2), (0x400, 3), (0x800, 4)] {
            if inp.option_bits & bit != 0 {
                p.set_armour_multiple(set, k);
            }
        }
        p
    }

    /// `FUN_00206890(k)`: armour = armour maximum = set+0x284 × k (the raw word, also on the
    /// default path) [M]. The in-drive cheat handler `FUN_00241bb8` calls it with 2/3/4 too.
    pub fn set_armour_multiple(&mut self, set: &HealthSet, k: i32) {
        let v = (set.armour_max * k) as f32;
        self.armour = v;
        self.armour_max = v;
    }

    /// `FUN_00206810` (`EPowerUpHealth`, percent / 100): health += (int)(fraction × max),
    /// capped at the maximum [M].
    pub fn add_health(&mut self, fraction: f32) {
        self.health = (self.health + ((fraction * self.health_max) as i32) as f32).min(self.health_max);
    }

    /// `FUN_00206850` (the armour power-up `FUN_00155b20`, percent / 100) [M].
    pub fn add_armour(&mut self, fraction: f32) {
        self.armour = (self.armour + ((fraction * self.armour_max) as i32) as f32).min(self.armour_max);
    }

    /// `FUN_00206b20`: health, or health / max when `ratio` [M].
    pub fn health_ratio(&self, ratio: bool) -> f32 {
        if ratio { self.health / self.health_max } else { self.health }
    }

    /// `FUN_00206b40`: 0 when the maximum is ≤ 0, else armour (× 1/max when `ratio`) [M].
    pub fn armour_ratio(&self, ratio: bool) -> f32 {
        if self.armour_max <= 0.0 {
            return 0.0;
        }
        if ratio { self.armour * (1.0 / self.armour_max) } else { self.armour }
    }

    /// `FUN_002068b8`: book a hit on the player car unless a lock bit is set [M].
    pub fn book_damage(&mut self, amount: f32, now: i32) {
        if self.lock_bits == 0 {
            self.damage_taken += amount;
            self.last_hit = now;
        }
    }

    /// `FUN_00206960`: kills += n, clamped at 0 [M].
    pub fn add_kills(&mut self, n: i32) {
        self.kills = (self.kills + n).max(0);
    }

    /// `FUN_002068e0`: a group target registered (+0x3bc += n, +0x5f0 = now) [M].
    pub fn add_group_targets(&mut self, n: i32, now: i32) {
        self.group_total += n;
        self.group_tick = now;
    }

    /// `FUN_00206900(windowed)`: +0x3c0 += 1 while below +0x3bc, and, when `windowed`, only
    /// while `now <= +0x5f0 + now / 10` (integer division) [M].
    pub fn credit_group(&mut self, windowed: bool, now: i32) {
        if self.group_hits < self.group_total && (!windowed || now <= self.group_tick + now / 10) {
            self.group_hits += 1;
        }
    }

    /// `FUN_00207228`, once per tick (`FUN_00206b88`). `hud` = `FUN_0023c618()` exists and its
    /// +0x2a0 word is 0. Option bit 1: below a third of the maximum, health climbs by
    /// 1 % of the maximum per second (0.01 / fps per tick) while `now − last_hit ≥ 0`. Option
    /// bit 4: health = maximum = 10000 [M].
    pub fn regen_tick(&mut self, now: i32, fps: i32, option_bits: u32, hud: bool) {
        if !hud {
            return;
        }
        if option_bits & 1 != 0 && self.health_ratio(true) < 0.33 && now - self.last_hit > -1 {
            self.health = (self.health + (0.01 / fps as f32) * self.health_max).min(self.health_max);
        }
        if option_bits & 4 != 0 {
            self.health = DEFAULT_HEALTH;
            self.health_max = DEFAULT_HEALTH;
        }
    }

    /// The four numbers the driving HUD reads (`HudInput` health/max/armour/max) [M FUN_0023da30].
    pub fn hud(&self) -> (f32, f32, f32, f32) {
        (self.health, self.health_max, self.armour, self.armour_max)
    }
}

/// `FUN_0023e270`: the HUD hit shake. While hud+0x328 < 5, hud+0x32c += (int)(amount + 0.5) + 1
/// (capped at 20) and hud+0x328 = hud+0x32c [M]. Returns the new (+0x328, +0x32c).
pub fn hud_hit(amount: f32, shake: u32, level: u32) -> (u32, u32) {
    if shake >= 5 {
        return (shake, level);
    }
    let l = (level.wrapping_add((amount + 0.5) as i32 as u32).wrapping_add(1)).min(20);
    (l, l)
}

// ------------------------------------------------------------------------------ damage zones

/// The 16 zones as (box code, neighbour mask), .data 0x3c1fe8 [M FUN_002db798 / FUN_002db930].
pub const ZONES: [(u16, u16); 16] = [
    (0x168, 0x4602), (0x440, 0x4c05), (0x154, 0x480a), (0x054, 0xd816),
    (0x094, 0xd868), (0x294, 0x9050), (0x480, 0xb0a0), (0x2a8, 0xa140),
    (0x0a8, 0xe6c0), (0x068, 0xe503), (0x049, 0x2a03), (0x045, 0x140e),
    (0x085, 0x2870), (0x089, 0x15c0), (0x042, 0x820f), (0x082, 0x41f0),
];

/// `FUN_002db798(box, p)`: the zone a car-local point falls in. The point is first mirrored by
/// (−1, 1, −1) (.data 0x3c1f40), then coded against the box `min`/`max` (box +0x40.. / +0x50..);
/// a code with no table row gives zone 0 [M].
pub fn zone_of(min: [f32; 3], max: [f32; 3], p: [f32; 3]) -> usize {
    let (x, y, z) = (-p[0], p[1], -p[2]);
    let mut c: u16 = 0x14;
    if min[0] <= x {
        c = 4;
        if (min[0] + max[0]) * 0.5 <= x {
            c = 0x28;
            if x <= max[0] {
                c = 8;
            }
        }
    }
    if z < min[2] {
        c |= 0x140;
    } else if z < (min[2] + max[2]) * 0.5 {
        c |= 0x40;
    } else if max[2] < z {
        c |= 0x280;
    } else {
        c |= 0x80;
    }
    if y < min[1] {
        c = c & 0xf8c3 | 2;
    } else if max[1] < y {
        c = c & 0xfccf | 1;
    }
    if c & 3 == 0 && c & 0xc != 0 && c & 0x30 == 0 {
        c = c & 0xfcf3 | 0x400;
    }
    ZONES.iter().position(|z| z.0 == c).unwrap_or(0)
}

/// `FUN_002db930(zone)`: the neighbour mask [M].
pub fn zone_neighbours(zone: usize) -> u32 {
    ZONES[zone & 15].1 as u32
}

// ------------------------------------------------------------------------- the car's damage

/// A pvehicle `.atr`'s damage words, as `FUN_001862c0` / `FUN_00187350` read them [M + D].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VehicleDamageAttrs {
    /// `TYRE_DAMAGE_POINTS` (`FUN_00169cb8`, 0 when no set in the chain has it): every tyre's
    /// points. `None` = this file has no line. On the disc only `default.atr` has one (30).
    pub tyre_points: Option<u32>,
    pub tyres_invulnerable: bool,
    pub is_tank: bool,
    pub is_helicopter: bool,
    /// CarPhysics `IS_SUB` (+0xbc), `IS_SNOWMOBILE` (+0xc8), `SUB_PHYSICS` (+0xc0).
    pub is_sub: bool,
    pub is_snowmobile: bool,
    pub sub_physics: bool,
    /// CarPhysics `WHEEL_RADIUS` (+0x6c); `None` when the file has no line.
    pub wheel_radius: Option<f32>,
    /// `IS_POLICE` (the `FUN_00187608` hit-limit gate).
    pub is_police: bool,
}

impl VehicleDamageAttrs {
    /// The `.atr` text: `KEY[.suffix]=value` lines, `//` comments (the same rules as
    /// `CarParams::parse_atr`).
    pub fn parse_atr(text: &str) -> VehicleDamageAttrs {
        let mut a = VehicleDamageAttrs::default();
        for line in text.lines() {
            let line = line.split("//").next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('[') { continue; }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let key = key.split('.').next().unwrap_or(key).trim();
            // `.b` values are spelled true / false [D default.atr].
            let raw = value.trim();
            let v = if raw.eq_ignore_ascii_case("true") {
                1.0
            } else if raw.eq_ignore_ascii_case("false") {
                0.0
            } else {
                let num: &str = raw
                    .split(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')))
                    .next().unwrap_or("");
                let Ok(v) = num.parse::<f32>() else { continue };
                v
            };
            match key {
                "TYRE_DAMAGE_POINTS" => a.tyre_points = Some(v.max(0.0) as u32),
                "TYRES_INVUNERABLE" => a.tyres_invulnerable = v != 0.0,
                "IS_TANK" => a.is_tank = v != 0.0,
                "IS_HELICOPTER" => a.is_helicopter = v != 0.0,
                "IS_SUB" => a.is_sub = v != 0.0,
                "IS_SNOWMOBILE" => a.is_snowmobile = v != 0.0,
                "SUB_PHYSICS" => a.sub_physics = v != 0.0,
                "WHEEL_RADIUS" => a.wheel_radius = Some(v),
                "IS_POLICE" => a.is_police = v != 0.0,
                _ => {}
            }
        }
        a
    }

    /// The attribute lookup `FUN_001699f8` walks a parent chain (set+0x1c) when a key is
    /// missing. `inherit` fills this file's missing tyre points / wheel radius from `parent`
    /// (e.g. `default.atr`); whether a car's set has `default` as its parent is not traced.
    pub fn inherit(mut self, parent: &VehicleDamageAttrs) -> VehicleDamageAttrs {
        if self.tyre_points.is_none() {
            self.tyre_points = parent.tyre_points;
        }
        if self.wheel_radius.is_none() {
            self.wheel_radius = parent.wheel_radius;
        }
        self
    }

    /// `FUN_001862c0:162-181`: tyres take damage only on a non-player car (category ≠ 0) that
    /// is not a sub, snowmobile, tank or helicopter and not `TYRES_INVUNERABLE` [M].
    pub fn tyres_vulnerable(&self, category: i32) -> bool {
        category != 0 && !self.is_sub && !self.is_snowmobile && !self.is_tank && !self.is_helicopter && !self.tyres_invulnerable
    }
}

/// Where the hit points live: the player car points +0x54 at mission+0x378, others at +0x42c.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitPoints {
    /// The player car: health and armour are the mission's (`PlayerHealth`).
    Player,
    /// Any other car: its own +0x42c word, no armour (+0x434 = 0).
    Own(f32),
}

/// One car's damage state, the `PBondCar` words `FUN_00189598` reads and writes [M].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarDamage {
    /// +0x2c8: the category the constructor gets (0 = player; 4 is stored as 2).
    pub category: i32,
    pub hp: HitPoints,
    /// +0x430: takes no hit points (and the zone caps halve).
    pub invulnerable: bool,
    /// +0x440: a kill still counts for the player (cleared once credited), 1 at construction.
    pub counts_for_player: bool,
    /// +0x2dc: the zone scale, −1 until the first hit fixes 32 / hit points.
    pub zone_scale: f32,
    /// +0x2f0 + 8·i: per zone (a, b). The localised path caps a at 1 and b at 1 (2 for kind 3),
    /// halved while invulnerable; the wreck path caps a at 1 and b at 2.
    pub zones: [[f32; 2]; 16],
    /// +0x37c: tyres take damage.
    pub tyres_vulnerable: bool,
    /// +0x2a0..+0x2ac: tyre points (`TYRE_DAMAGE_POINTS` each); 0 = gone or never damageable.
    pub tyres: [u32; 4],
    /// CarPhysics `SUB_PHYSICS`.
    pub sub_physics: bool,
    /// +1000 (0x3e8): the flipped-on-ground tick count.
    pub flip_ticks: i32,
}

impl CarDamage {
    /// The constructor's damage words (`FUN_001862c0` + `FUN_00187350` + `FUN_00187578`) [M].
    pub fn new(category: i32, attrs: &VehicleDamageAttrs) -> CarDamage {
        let cat = if category == 4 { 2 } else { category };
        CarDamage {
            category: cat,
            hp: if category == 0 { HitPoints::Player } else { HitPoints::Own(CAR_HIT_POINTS) },
            invulnerable: false,
            counts_for_player: true,
            zone_scale: -1.0,
            zones: [[0.0; 2]; 16],
            tyres_vulnerable: attrs.tyres_vulnerable(category),
            tyres: [attrs.tyre_points.unwrap_or(0); 4],
            sub_physics: attrs.sub_physics,
            flip_ticks: 0,
        }
    }

    /// `FUN_00187578` (vtable +0x19c, the repair `EPowerUpHealth` calls on the player car):
    /// zones cleared, +0x42c = 700 [M]. On the player car the +0x42c word is no longer the one
    /// +0x54 points at, so the mission health is untouched by this step.
    pub fn repair(&mut self) {
        self.zones = [[0.0; 2]; 16];
        if let HitPoints::Own(h) = &mut self.hp {
            *h = CAR_HIT_POINTS;
        }
    }

    /// `FUN_00195a30`: the hit points (1.0 when +0x54 is null, not reachable for a car).
    pub fn hit_points(&self, player: &PlayerHealth) -> f32 {
        match self.hp {
            HitPoints::Player => player.health,
            HitPoints::Own(h) => h,
        }
    }

    fn take(&mut self, amount: f32, player: &mut PlayerHealth) {
        // FUN_00195a58: hp −= max(amount, 0), floored at 0 [M].
        let h = match &mut self.hp {
            HitPoints::Player => &mut player.health,
            HitPoints::Own(h) => h,
        };
        *h -= amount.max(0.0);
        if *h < 0.0 {
            *h = 0.0;
        }
    }
}

/// One damage call, `vtable+0x1c(amount, dent, car, at, from, kind, source)` [M].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub amount: f32,
    /// param_2: the share of the scaled amount that goes to zone b (the rest to zone a).
    pub dent: f32,
    /// param_6. The callers: 0 = `FUN_00139be8` (1000, 0.5) / `FUN_00150390` + `FUN_00150748`
    /// (1e6, 0.5); 1 = the contacts; 2 = bullets (`FUN_0015de48`, 0.25) and the projectile's
    /// direct hit (`FUN_00182978`, 32, 0), `FUN_0019a710`, `FUN_0019d080`; 3 = `EDamagePlayer`,
    /// `EInflictDamage` and the flip damage of `FUN_00188200` (100, 0.5) [M].
    pub kind: u32,
    /// The world positions `at` (param_4) and `from` (param_5).
    pub at: [f32; 3],
    pub from: [f32; 3],
    /// The source belongs to the player (`FUN_001ff280` + `FUN_00195c70`).
    pub from_player: bool,
}

/// The facts `FUN_00189598` reads from elsewhere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitContext {
    pub now: i32,
    /// `FUN_002d3950`: where the at→from segment meets the car's box, car-local; `None` = no
    /// intersection (the call then returns 0 having done only the armour/booking steps).
    pub box_point: Option<[f32; 3]>,
    /// The model's vtable +0x54 value added to half the point's y before the zone pick.
    pub box_y_offset: f32,
    /// The zone box (model+0xc → +0x40 min, +0x50 max).
    pub box_min: [f32; 3],
    pub box_max: [f32; 3],
    /// mission+0x350 (`EPlayerLose` prints it as "AutoDriveEnabled").
    pub mission_350: bool,
    /// Per tyre: the squared distance the game measures from the box point to the tyre (the
    /// wheel position at model+0x5c+0x80 with x or z swapped for the point's, by which box
    /// face was struck) and whether the tyre is on the struck side. The wheel positions are
    /// the caller's (this module does not hold the model).
    pub tyre_d2: [f32; 4],
    pub tyre_side: [bool; 4],
    /// CarPhysics `WHEEL_RADIUS` of this car.
    pub wheel_radius: f32,
    /// The spread level 0x332094 (0.0 in play, see `SPREAD_LEVEL_IN_PLAY`).
    pub spread_level: f32,
}

/// What a damage call did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HitResult {
    /// The return value: 0 (no box hit), 0x90 (the armour took it all), 0x10 / 0x70 / 0x80.
    pub code: u32,
    /// The armour soaked this much.
    pub armour_soaked: f32,
    /// The zone struck and the zone mask the hit spread over.
    pub zone: usize,
    pub mask: u32,
    /// Hit points went to 0 with this call.
    pub destroyed: bool,
    /// A kill was credited to the player (`FUN_00206960(mission, 1)`).
    pub kill_credited: bool,
    /// `FUN_001a2b50` / `FUN_001a2868` / `FUN_001a27a0` calls as (event, mask): the model's
    /// damage events (stages 0x13..0x16, severity 4/5/6/0x19, wreck 2 and 6/7, tyre 1..4).
    pub events: Vec<(u32, u32)>,
    /// The model vtable +0x84 masks with argument 10 and 9.
    pub model10: u32,
    pub model9: u32,
    /// A tyre blew (index).
    pub tyre_blown: Option<usize>,
    /// Kind 2: the push `FUN_00195728` gets (linear part) and the torque seed
    /// `(!r1 << 16) | r2` of `FUN_002d0b08` (the torque itself is not ported).
    pub push: Option<([f32; 3], u32)>,
    /// The HUD shake (`hud_hit`) is raised with this amount (kind 2/3, amount > 1, a
    /// category 0/1 car, mission+0x350 set, not invulnerable; the game does not check the HUD
    /// pointer).
    pub hud_hit: Option<f32>,
}

/// `FUN_00189598`, `PBondCar` / `PTank` vtable +0x1c [M]. `rand` is `FUN_0017dc28`, drawn in the
/// game's order (kind 2: two draws for the torque seed; an intact car: one for the code; the
/// sub-physics wreck: three).
pub fn inflict(car: &mut CarDamage, player: &mut PlayerHealth, hit: &Hit, cx: &HitContext, rand: &mut dyn FnMut() -> u16) -> HitResult {
    let mut r = HitResult::default();
    let mut amount = hit.amount;
    let mut invulnerable = car.invulnerable;
    let player_side = (car.category as u32) < 2;
    if player_side {
        player.book_damage(amount, cx.now);
        if player.lock_bits != 0 {
            invulnerable = true;
        }
    }
    if (hit.kind == 2 || hit.kind == 3) && amount > 1.0 && player_side && cx.mission_350 && !invulnerable {
        r.hud_hit = Some(amount);
    }
    // The armour word (+0x434): only the player car has one.
    if car.hp == HitPoints::Player {
        if amount < player.armour {
            player.armour -= amount;
            r.armour_soaked = amount;
            r.code = 0x90;
            return r;
        }
        amount -= player.armour;
        r.armour_soaked = player.armour;
        player.armour = 0.0;
    }
    let Some(bp) = cx.box_point else {
        return r; // FUN_002d3950 == 0 → return 0
    };
    let mut credit = car.counts_for_player;
    if !hit.from_player && hit.kind == 2 {
        credit = false;
    }
    if hit.kind == 1 {
        credit = false;
    }
    if credit && car.category == 2 && amount >= 5.0 && car.hit_points(player) > 0.0 {
        player.credit_group(hit.kind == 2, cx.now);
    }
    if car.zone_scale < 0.0 {
        car.zone_scale = ZONE_SCALE_NUMERATOR / car.hit_points(player);
    }
    let zp = [bp[0], bp[1] * 0.5 + cx.box_y_offset, bp[2]];
    let zone = zone_of(cx.box_min, cx.box_max, zp);
    r.zone = zone;
    let mut mask = 1u32 << zone;

    // The tyres (kind 2 on a car with vulnerable tyres): the first live tyre on the struck side
    // within (WHEEL_RADIUS × TYRE_DAMAGE_RADIUS)² takes (uint)amount; at 0 it blows. Either way
    // the hit stops there (amount = 0).
    if car.tyres_vulnerable && hit.kind == 2 {
        let lim = cx.wheel_radius * TYRE_DAMAGE_RADIUS;
        let lim2 = lim * lim;
        let pts = if amount > 0.0 { amount as u32 } else { 0 };
        for i in 0..4 {
            if car.tyres[i] != 0 && cx.tyre_side[i] && cx.tyre_d2[i] < lim2 {
                if pts < car.tyres[i] {
                    car.tyres[i] -= pts;
                } else {
                    car.tyres[i] = 0;
                    r.events.push((TYRE_BLOWOUT_EVENT[i], 3));
                    r.tyre_blown = Some(i);
                }
                amount = 0.0;
                break;
            }
        }
    }

    let before = car.hit_points(player);
    let was_dead = before <= 0.0;
    if !invulnerable {
        car.take(amount, player);
    }
    let after = car.hit_points(player);
    let wrecked = if !car.sub_physics { after <= 0.0 } else if was_dead { hit.kind == 1 } else { false };
    let dent = amount * hit.dent * car.zone_scale;
    let scuff = amount * (1.0 - hit.dent) * car.zone_scale;
    if wrecked && !was_dead {
        r.destroyed = true;
        if car.category == 2 && car.counts_for_player {
            player.add_kills(1);
            car.counts_for_player = false;
            r.kill_credited = true;
        }
    }
    if !was_dead && car.sub_physics && after <= 0.0 {
        r.destroyed = true;
        if car.counts_for_player {
            player.add_kills(1);
            car.counts_for_player = false;
            r.kill_credited = true;
        }
        // The sub's spin draws (+0x240 / +0x244), kept for the stream.
        for _ in 0..3 {
            rand();
        }
    }
    if hit.kind == 2 {
        let d = [hit.from[0] - hit.at[0], hit.from[1] - hit.at[1], hit.from[2] - hit.at[2]];
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let k = amount * HIT_PUSH;
        let mut v = if n > 0.0 { [d[0] / n * k, d[1] / n * k, d[2] / n * k] } else { [0.0; 3] };
        let y = v[1].max(k * 0.1);
        v[1] = y + y;
        let r1 = rand() as u32;
        let r2 = rand() as u32;
        // Not applied to the player car while mission+0x350 is set [M :0x18a020 tail].
        if !(car.hp == HitPoints::Player && cx.mission_350) {
            r.push = Some((v, (!r1 << 16) | r2));
        }
    }
    if !wrecked {
        let mut sev = None;
        for (t, e) in SEVERITY {
            if t < scuff + dent {
                sev = Some(e);
                break;
            }
        }
        if hit.kind == 3 || cx.spread_level < car.zones[zone][0] + car.zones[zone][1] {
            mask |= zone_neighbours(zone);
        }
        let (cap_a, mut cap_b) = if invulnerable { (0.5, 0.5) } else { (1.0, 1.0) };
        if hit.kind == 3 {
            cap_b += cap_b;
        }
        let mut stage = [0u32; 4];
        let mut changed = 0u32;
        for i in 0..16 {
            let bit = 1u32 << i;
            let z = &mut car.zones[i];
            let m;
            if mask & bit == 0 {
                m = z[0].max(z[1]);
            } else {
                let ca = z[0].max(cap_a);
                let cb = z[1].max(cap_b);
                let w = ZONE_WEIGHT[(i == zone) as usize];
                z[0] = (z[0] + scuff * w).min(ca);
                z[1] = (z[1] + dent * w).min(cb);
                changed |= bit;
                m = z[1].max(z[0]);
                if let Some(k) = (0..4).rev().find(|&k| m >= STAGES[k]) {
                    stage[k] |= bit;
                }
            }
            if MODEL10_LEVEL < m {
                r.model10 |= bit;
            }
            if MODEL9_LEVEL < m {
                r.model9 |= bit;
            }
        }
        for (k, s) in stage.iter().enumerate() {
            if *s != 0 && (!car.sub_physics || k as u32 + 0x13 != 0x16) {
                r.events.push((0x13 + k as u32, *s));
            }
        }
        if let Some(e) = sev {
            if changed != 0 {
                r.events.push((e, changed));
            }
        }
    } else {
        r.events.push((2, 0));
        r.events.push((if hit.kind == 3 { 7 } else { 6 }, 0));
        for e in [0x16u32, 0x15, 0x14, 0x13] {
            r.events.push((e, 0xffff));
        }
        r.model10 = 0xffff;
        if hit.kind == 0 {
            for z in car.zones.iter_mut() {
                z[0] = (z[0] + scuff).min(1.0);
                z[1] = (z[1] + dent).min(2.0);
            }
        } else {
            mask |= zone_neighbours(zone);
            for (i, z) in car.zones.iter_mut().enumerate() {
                if mask & (1 << i) != 0 {
                    z[0] = (z[0] + scuff).min(1.0);
                    z[1] = (z[1] + dent).min(2.0);
                }
            }
        }
    }
    r.mask = mask;
    r.code = 0x10;
    if !wrecked {
        let bit = 1u32 << zone;
        let roll = rand() as f32 * (1.0 / 65536.0);
        if bit & 0x3f18 != 0 && car.category == 1 && roll < 0.5 {
            r.code = 0x80;
        } else if bit & 0x3ca5 != 0 && roll < 0.5 {
            r.code = 0x70;
        }
    }
    r
}

/// The `DamageZones` the car microcode reads per zone (car-model.md §10.5: position += DmgVec ·
/// zone.x …). Which of (a, b) lands in .x/.y/.z is set by the model's upload, which is not
/// traced; this returns the stored pair as is.
pub fn damage_zones(car: &CarDamage) -> [[f32; 2]; 16] {
    car.zones
}

// ------------------------------------------------------------------------------- contacts

/// `FUN_001fc198(impulse, body, at)`: a world contact. `kind` = rigid+0x6d (cars 1..3, props
/// 4+); `hit_timer` = the owner's +0x374 byte (vtable +0x1b4); `immune` = the kind-1 owner's
/// vtable +0x64 (`FUN_0018ad28`: +0x2e4 set and `FUN_0013e0b0()`). Returns the damage call
/// (amount, dent, kind 1) and whether the owner first books the amount (vtable +0x1ac =
/// `FUN_00187608`, `weapons::CarHits::hit`) [M].
pub fn world_contact(impulse: f32, kind: i8, hit_timer: u8, immune: bool) -> (Option<(f32, f32)>, bool) {
    let car = kind < 4;
    let gate = if car { 0.2 } else { 0.05 };
    if impulse <= gate {
        return (None, false);
    }
    let mut d = (impulse + impulse) * COLLISION_DAMAGE_SCALE;
    let mut book = false;
    if car {
        d *= CAR_WORLD_CONTACT_SCALE;
        book = hit_timer != 0;
    }
    if kind == 1 && immune {
        d = 0.0;
    }
    ((d > 0.0).then_some((d, WORLD_CONTACT_DENT)), book)
}

/// `FUN_001fbbd8`'s damage step for one side of a car-against-car contact: `impulse` = the
/// contact's (doubled when this side is a car body, kind < 4), `share` = this side's share from
/// the contact solver (`FUN_001f9d98`, not ported), damage = impulse × share × 5 (× 3 for a car
/// body) above an impulse of 0.05; dent 0.35, kind 1 [M].
pub fn car_contact(impulse: f32, share: f32, kind: i8) -> Option<(f32, f32)> {
    if impulse <= 0.05 {
        return None;
    }
    let car = kind < 4;
    let i = if car { impulse + impulse } else { impulse };
    let mut d = i * share * COLLISION_DAMAGE_SCALE;
    if car {
        d *= 3.0;
    }
    (d > 0.0).then_some((d, CAR_CONTACT_DENT))
}

/// `FUN_00188200`'s flip counter (+1000) for one tick: `up_y` = the body's up·(0,1,0)
/// (`FUN_001f7c00`), `contact` = rigid+0x6c. Below 0.3 with contact the count rises (the player,
/// category 0, and category 1 stop at 30); otherwise it resets. Categories ≥ 2 cap at 120 and,
/// while the count is non-zero and the car has hit points, take a (100, 0.5, kind 3) hit each
/// tick [M disasm 0x1887ec-0x18887c]. Returns that hit's (amount, dent).
pub fn flip_tick(car: &mut CarDamage, up_y: f32, contact: bool, hit_points: f32) -> Option<(f32, f32)> {
    let tilted = up_y < 0.3 && contact;
    if car.category == 0 || car.category == 1 {
        if tilted {
            if car.flip_ticks < 30 {
                car.flip_ticks += 1;
            }
        } else {
            car.flip_ticks = 0;
        }
        return None;
    }
    if tilted { car.flip_ticks += 1 } else { car.flip_ticks = 0 }
    if car.flip_ticks >= 0x79 {
        car.flip_ticks = 0x78;
    }
    (car.flip_ticks != 0 && hit_points > 0.0).then_some((100.0, 0.5))
}

// ------------------------------------------------------------------------------ mission rules

/// The value a damage/health rule watches (`FUN_00209820`) [M].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuleSubject {
    /// 0: (int) the player's health (mission+0x378).
    Player,
    /// 3: a pair from `FUN_0012c440(DAT_00327c30)` (not traced).
    Pair,
    /// 4: (int) the mission target `index`'s hit points (record +0x20 → +0x4c).
    Target(u32),
    /// Anything else reads 0.
    Zero,
}

impl RuleSubject {
    /// `FUN_00209738(entry+0x10, entry+0x14)`: the index is kept only for subject 4 [M].
    pub fn from_entry(subject: u32, index: u32) -> RuleSubject {
        match subject {
            0 => RuleSubject::Player,
            3 => RuleSubject::Pair,
            4 => RuleSubject::Target(index),
            _ => RuleSubject::Zero,
        }
    }
}

/// `SRuleDamage` (type 2, ctor `FUN_00209f38`): fires when the watched value dropped by more
/// than `threshold` since the last test; subject 3 fires when the pair's +8 float exceeds it
/// [M FUN_00209ff0].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageRule {
    pub subject: RuleSubject,
    /// entry+0x18.
    pub threshold: i32,
    /// obj+0x20, 0 at construction.
    pub last: i32,
}

impl DamageRule {
    pub fn new(subject: RuleSubject, threshold: i32) -> DamageRule {
        DamageRule { subject, threshold, last: 0 }
    }

    /// `value` = the subject's (int) value (subject 3: the pair's +8 word).
    pub fn test(&mut self, value: i32) -> bool {
        if self.subject == RuleSubject::Pair {
            return self.threshold < value;
        }
        let drop = self.last.wrapping_sub(value);
        self.last = value;
        self.threshold < drop
    }
}

/// `SRuleHealth` (type 4, ctor `FUN_0020a128`): fires when the value crosses `threshold` -
/// downward (falls below it) when `rising` is false, upward (goes above it) when true. The
/// previous value starts at threshold − 1 / + 1 so a value already past it does not fire
/// [M FUN_0020a1f8].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HealthRule {
    pub subject: RuleSubject,
    /// entry+0x18 != 0.
    pub rising: bool,
    /// entry+0x1c.
    pub threshold: i32,
    /// obj+0x24.
    pub last: i32,
}

impl HealthRule {
    pub fn new(subject: RuleSubject, rising: bool, threshold: i32) -> HealthRule {
        HealthRule { subject, rising, threshold, last: if rising { threshold + 1 } else { threshold - 1 } }
    }

    /// `value` = the subject's (int) value (subject 3: the pair's [0] when rising, [1] else).
    pub fn test(&mut self, value: i32) -> bool {
        let fire = if !self.rising {
            value < self.threshold && self.threshold <= self.last
        } else {
            self.threshold < value && self.last <= self.threshold
        };
        self.last = value;
        fire
    }
}

/// `EPlayerLose`'s exit word: PTP+0x978 = 4 when the player's health is ≤ 0 at the event,
/// else 1; at health ≤ 0 (or the record's 4th word set) the speech "SFX_BondDeath" plays
/// [M FUN_00154cb0]. The event does nothing once the mission state (+0x34c) is 3 (passed), or
/// while PTP+0x628 is set (then it counts a restart instead).
pub fn player_lose_exit(health: f32) -> u32 {
    if health <= 0.0 { 4 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_rand() -> impl FnMut() -> u16 {
        || 0
    }

    #[test]
    fn start_defaults_when_the_set_health_is_zero() {
        let p = PlayerHealth::start(&HealthSet::default(), &StartInputs::default());
        assert_eq!(p.hud(), (10_000.0, 10_000.0, 0.0, 1_000.0));
    }

    #[test]
    fn start_uses_the_built_in_record() {
        let p = PlayerHealth::start(&HealthSet::BUILT_IN, &StartInputs::default());
        assert_eq!(p.hud(), (3000.0, 3000.0, 0.0, 1000.0));
    }

    #[test]
    fn start_order_carry_then_retry_then_difficulty() {
        let set = HealthSet { health_max: 4000, health: 4000, armour_max: 800, armour: 100, health_pct: [100, 150, 50], armour_pct: [100, 200, 25] };
        // carry 0.25 → 1000; second load → max(2000, 1000) = 2000; difficulty 2 → ×0.5.
        let inp = StartInputs { difficulty: 2, carry_health: 0.25, carry_armour: 0.5, loads_before: 1, ..Default::default() };
        let p = PlayerHealth::start(&set, &inp);
        assert_eq!(p.health, 1000.0);
        assert_eq!(p.health_max, 2000.0);
        // armour max(800·0.5, 100) = 400, then ×0.25; max 800 × 0.25.
        assert_eq!((p.armour, p.armour_max), (100.0, 200.0));
        // Difficulty 1 takes the +0x298/+0x2a4 column; 0 and 7 take +0x294/+0x2a0.
        let p1 = PlayerHealth::start(&set, &StartInputs { difficulty: 1, ..Default::default() });
        assert_eq!((p1.health, p1.armour_max), (6000.0, 1600.0));
        let p7 = PlayerHealth::start(&set, &StartInputs { difficulty: 7, ..Default::default() });
        assert_eq!((p7.health, p7.armour_max), (4000.0, 800.0));
        // The +0x294 gate: 0 skips the whole step.
        let gated = HealthSet { health_pct: [0, 150, 50], ..set };
        assert_eq!(PlayerHealth::start(&gated, &StartInputs { difficulty: 1, ..Default::default() }).health, 4000.0);
    }

    #[test]
    fn armour_option_bits_use_the_raw_word() {
        let set = HealthSet { armour_max: 300, ..HealthSet::default() };
        let p = PlayerHealth::start(&set, &StartInputs { option_bits: 0x200 | 0x800, ..Default::default() });
        // Default path (health 0) for health, but the ×k reads set+0x284 = 300; 0x800 wins.
        assert_eq!((p.health, p.armour, p.armour_max), (10_000.0, 1200.0, 1200.0));
        let x = PlayerHealth::start(&HealthSet::BUILT_IN, &StartInputs { times_100: true, ..Default::default() });
        assert_eq!((x.health, x.health_max), (300_000.0, 300_000.0));
    }

    #[test]
    fn pickups_truncate_and_cap() {
        let mut p = PlayerHealth { health: 100.0, health_max: 333.0, armour: 0.0, armour_max: 999.0, ..Default::default() };
        p.add_health(0.5); // (int)166.5 = 166
        assert_eq!(p.health, 266.0);
        p.add_health(1.0);
        assert_eq!(p.health, 333.0);
        p.add_armour(0.25); // (int)249.75 = 249
        assert_eq!(p.armour, 249.0);
        assert_eq!(p.armour_ratio(false), 249.0);
        let none = PlayerHealth { armour: 5.0, armour_max: 0.0, ..Default::default() };
        assert_eq!(none.armour_ratio(true), 0.0);
    }

    #[test]
    fn regen_below_a_third_only() {
        let mut p = PlayerHealth { health: 300.0, health_max: 1000.0, last_hit: 50, ..Default::default() };
        p.regen_tick(100, 60, 1, true);
        assert!((p.health - (300.0 + 0.01 / 60.0 * 1000.0)).abs() < 1e-4);
        let mut q = PlayerHealth { health: 340.0, health_max: 1000.0, ..Default::default() };
        q.regen_tick(100, 60, 1, true);
        assert_eq!(q.health, 340.0, "0.34 is not below 0.33");
        let mut g = PlayerHealth { health: 1.0, health_max: 50.0, ..Default::default() };
        g.regen_tick(0, 60, 4, false);
        assert_eq!(g.health, 1.0, "no HUD, no step");
        g.regen_tick(0, 60, 4, true);
        assert_eq!((g.health, g.health_max), (10_000.0, 10_000.0));
    }

    #[test]
    fn zone_lookup_matches_the_table() {
        let (mn, mx) = ([-1.0, 0.0, -2.0], [1.0, 1.0, 2.0]);
        // Inside the box, centre-right-front after the mirror: x' = 0.5 (≥ mid, ≤ max) → 8;
        // z' = 1.0 (≥ mid, ≤ max) → 0x80; y in → 0x88 | the 0x400 rule (x in-box, no y bits)
        // → (0x88 & 0xfcf3) | 0x400 = 0x480 → zone 6.
        assert_eq!(zone_of(mn, mx, [-0.5, 0.5, -1.0]), 6);
        // Above the roof: y > max → (c & 0xfccf) | 1. x' = −0.5 → 4, z' = −1 → 0x40: 0x45 → 11.
        assert_eq!(zone_of(mn, mx, [0.5, 2.0, 1.0]), 11);
        // Below, left of the box: x' < min → 0x14, z' > max → 0x280 → 0x294 & 0xf8c3 | 2 = 0x82 → 15.
        assert_eq!(zone_of(mn, mx, [5.0, -1.0, -5.0]), 15);
        assert_eq!(zone_neighbours(6), 0xb0a0);
    }

    fn cx() -> HitContext {
        HitContext {
            now: 10, box_point: Some([-0.5, 1.0, -1.0]), box_y_offset: 0.0,
            box_min: [-1.0, 0.0, -2.0], box_max: [1.0, 1.0, 2.0],
            mission_350: false, tyre_d2: [100.0; 4], tyre_side: [false; 4], wheel_radius: 0.4,
            spread_level: SPREAD_LEVEL_IN_PLAY,
        }
    }

    #[test]
    fn armour_soaks_first_and_returns_0x90() {
        let mut p = PlayerHealth { health: 1000.0, health_max: 1000.0, armour: 50.0, armour_max: 100.0, ..Default::default() };
        let mut car = CarDamage::new(0, &VehicleDamageAttrs::default());
        let h = Hit { amount: 30.0, dent: 0.5, kind: 3, at: [0.0; 3], from: [0.0; 3], from_player: false };
        let r = inflict(&mut car, &mut p, &h, &cx(), &mut no_rand());
        assert_eq!((r.code, p.armour, p.health, p.damage_taken), (0x90, 20.0, 1000.0, 30.0));
        // The rest goes through: 30 − 20 = 10 off the health.
        let r = inflict(&mut car, &mut p, &h, &cx(), &mut no_rand());
        assert_eq!((p.armour, p.health), (0.0, 990.0));
        assert_eq!(r.armour_soaked, 20.0);
        // The zone scale was fixed at the first box hit: 32 / 1000 (health before the hit).
        assert_eq!(car.zone_scale, 32.0 / 1000.0);
    }

    #[test]
    fn localised_zone_step_and_events() {
        let mut p = PlayerHealth::default();
        let mut car = CarDamage::new(2, &VehicleDamageAttrs::default());
        assert_eq!(car.hp, HitPoints::Own(700.0));
        // amount 175 → scale 32/700; scuff = 175·0.5·32/700 = 4.0, dent = 4.0.
        let h = Hit { amount: 175.0, dent: 0.5, kind: 1, at: [0.0; 3], from: [0.0; 3], from_player: true };
        let r = inflict(&mut car, &mut p, &h, &cx(), &mut || 0x8000);
        assert_eq!(car.hit_points(&p), 525.0);
        assert_eq!(r.zone, 6);
        // Spread level 0 < 0 + 0 is false on a fresh car: no neighbours yet.
        assert_eq!(r.mask, 1 << 6);
        assert_eq!(car.zones[6], [1.0, 1.0], "capped at 1 / 1 (kind 1)");
        assert_eq!(r.events, vec![(0x16, 1 << 6), (6, 1 << 6)]);
        assert_eq!((r.model10, r.model9), (1 << 6, 1 << 6));
        // Roll 0x8000/65536 = 0.5 is not < 0.5 → 0x10.
        assert_eq!(r.code, 0x10);
        // Second hit spreads (0 < 2.0), neighbours take 0.2 of it.
        let small = Hit { amount: 7.0, ..h };
        let r = inflict(&mut car, &mut p, &small, &cx(), &mut || 0);
        assert_eq!(r.mask, (1 << 6) | 0xb0a0);
        let w = 7.0 * 0.5 * (32.0 / 700.0) * 0.2;
        assert!((car.zones[5][0] - w).abs() < 1e-6, "{:?}", car.zones[5]);
        // Zone 6 bit 0x40 & 0x3ca5 is 0, & 0x3f18 is 0 → 0x10 even with roll 0.
        assert_eq!(r.code, 0x10);
    }

    #[test]
    fn wreck_counts_one_kill_and_fills_every_zone_on_kind_0() {
        let mut p = PlayerHealth::default();
        let mut car = CarDamage::new(2, &VehicleDamageAttrs::default());
        let h = Hit { amount: 1.0e6, dent: 0.5, kind: 0, at: [0.0; 3], from: [0.0; 3], from_player: false };
        let r = inflict(&mut car, &mut p, &h, &cx(), &mut no_rand());
        assert!(r.destroyed && r.kill_credited);
        assert_eq!((p.kills, car.counts_for_player), (1, false));
        assert!(car.zones.iter().all(|z| *z == [1.0, 2.0]));
        assert_eq!(&r.events[..2], &[(2, 0), (6, 0)]);
        assert_eq!(r.model10, 0xffff);
        assert_eq!(r.code, 0x10);
        // A second hit on the wreck credits nothing more.
        let r = inflict(&mut car, &mut p, &Hit { kind: 3, ..h }, &cx(), &mut no_rand());
        assert!(!r.destroyed && !r.kill_credited);
        assert_eq!(r.events[1], (7, 0));
        assert_eq!(p.kills, 1);
    }

    #[test]
    fn tyre_takes_a_bullet_and_blows_at_zero() {
        let attrs = VehicleDamageAttrs { tyre_points: Some(30), ..Default::default() };
        let mut car = CarDamage::new(2, &attrs);
        assert!(car.tyres_vulnerable);
        assert!(!CarDamage::new(0, &attrs).tyres_vulnerable, "the player's tyres never");
        let mut p = PlayerHealth::default();
        let mut c = cx();
        c.tyre_side = [false, true, false, false];
        c.tyre_d2 = [0.0, 0.2, 0.0, 0.0]; // (0.4·1.25)² = 0.25
        let h = Hit { amount: 20.9, dent: 0.25, kind: 2, at: [0.0; 3], from: [0.0, 1.0, 0.0], from_player: true };
        let r = inflict(&mut car, &mut p, &h, &c, &mut no_rand());
        assert_eq!(car.tyres, [30, 10, 30, 30]);
        assert_eq!(car.hit_points(&p), 700.0, "the tyre stopped the hit");
        assert!(r.tyre_blown.is_none());
        let r = inflict(&mut car, &mut p, &h, &c, &mut no_rand());
        assert_eq!((car.tyres[1], r.tyre_blown), (0, Some(1)));
        assert!(r.events.contains(&(2, 3)));
        // Push: up 1 × 0 (amount zeroed) → y = 2·max(0, 0) = 0.
        assert_eq!(r.push.unwrap().0, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn push_follows_the_hit_line() {
        let mut p = PlayerHealth::default();
        let mut car = CarDamage::new(3, &VehicleDamageAttrs::default());
        let h = Hit { amount: 2.0, dent: 0.0, kind: 2, at: [0.0; 3], from: [3.0, 0.0, 4.0], from_player: true };
        let mut n = 0u16;
        let r = inflict(&mut car, &mut p, &h, &cx(), &mut || { n += 1; n });
        let (v, seed) = r.push.unwrap();
        assert_eq!(v, [1200.0, 400.0, 1600.0], "2000 along (0.6, 0, 0.8); y = 2·max(0, 200)");
        assert_eq!(seed, (!1u32 << 16) | 2);
    }

    #[test]
    fn contacts() {
        assert_eq!(world_contact(0.2, 1, 1, false), (None, false));
        assert_eq!(world_contact(0.3, 2, 1, false), (Some((18.0, 0.35)), true)); // 0.6·5·6
        assert_eq!(world_contact(0.3, 2, 0, false).1, false);
        assert_eq!(world_contact(0.3, 1, 1, true).0, None);
        assert_eq!(world_contact(0.06, 5, 0, false), (Some(((0.06f32 + 0.06) * 5.0, 0.35)), false)); // prop: 0.12·5
        assert_eq!(car_contact(0.05, 1.0, 1), None);
        assert_eq!(car_contact(1.0, 0.5, 2), Some((15.0, 0.35))); // 2·0.5·5·3
        assert_eq!(car_contact(1.0, 0.5, 6), Some((2.5, 0.35)));
    }

    #[test]
    fn flipped_ai_car_takes_100_a_tick() {
        let mut ai = CarDamage::new(2, &VehicleDamageAttrs::default());
        assert_eq!(flip_tick(&mut ai, 0.9, true, 700.0), None);
        assert_eq!(flip_tick(&mut ai, 0.1, true, 700.0), Some((100.0, 0.5)));
        for _ in 0..200 {
            flip_tick(&mut ai, 0.1, true, 700.0);
        }
        assert_eq!(ai.flip_ticks, 120);
        assert_eq!(flip_tick(&mut ai, 0.1, true, 0.0), None, "no hit points, no hit");
        let mut me = CarDamage::new(0, &VehicleDamageAttrs::default());
        for _ in 0..50 {
            assert_eq!(flip_tick(&mut me, -1.0, true, 1.0), None);
        }
        assert_eq!(me.flip_ticks, 30);
        flip_tick(&mut me, -1.0, false, 1.0);
        assert_eq!(me.flip_ticks, 0);
    }

    #[test]
    fn rules_fire_on_the_crossing_only() {
        let mut d = DamageRule::new(RuleSubject::Player, 99);
        assert!(!d.test(1000), "0 − 1000 is no drop");
        assert!(!d.test(901), "a drop of 99 is not > 99");
        assert!(d.test(801));
        let mut h = HealthRule::new(RuleSubject::Player, false, 500);
        assert!(!h.test(700));
        assert!(h.test(499));
        assert!(!h.test(100), "already below");
        assert!(!HealthRule::new(RuleSubject::Player, false, 500).test(100), "started below");
        let mut up = HealthRule::new(RuleSubject::Player, true, 10);
        assert!(!up.test(20), "started above");
        assert!(!up.test(5));
        assert!(up.test(11));
        assert_eq!(RuleSubject::from_entry(4, 7), RuleSubject::Target(7));
        assert_eq!(RuleSubject::from_entry(1, 7), RuleSubject::Zero);
        assert_eq!((player_lose_exit(0.0), player_lose_exit(0.5)), (4, 1));
    }

    #[test]
    fn atr_words_and_the_parent_fill() {
        let d = VehicleDamageAttrs::parse_atr("[default]
WHEEL_RADIUS=0.36
TYRE_DAMAGE_POINTS.u=30
TYRES_INVUNERABLE.b=false
");
        assert_eq!((d.tyre_points, d.tyres_invulnerable, d.wheel_radius), (Some(30), false, Some(0.36)));
        let c = VehicleDamageAttrs::parse_atr("[x]
TYRES_INVUNERABLE.b=true // c
IS_SUB=1
");
        assert!(c.tyres_invulnerable && c.is_sub && c.tyre_points.is_none());
        let c = c.inherit(&d);
        assert_eq!((c.tyre_points, c.wheel_radius), (Some(30), Some(0.36)));
        assert!(!c.tyres_vulnerable(2));
    }

    #[test]
    fn hud_shake_caps_at_twenty() {
        assert_eq!(hud_hit(3.4, 0, 0), (4, 4));
        assert_eq!(hud_hit(50.0, 4, 4), (20, 20));
        assert_eq!(hud_hit(50.0, 5, 4), (5, 4));
    }
}
