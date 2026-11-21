//! The driving engine's weapons: the 32-entry weapon table, the per-vehicle weapon state (ammo,
//! clip, barrel cycling, reload), the weapon cycle order, the projectile launch table and the
//! homing step, hit damage, the bullet-streak cadence and the car's damage/EMP bookkeeping.
//! docs/systems/driving-weapons.md is the write-up.
//!
//! Sources (DRIVING.ELF, decompile under work/decomp/DRIVING.ELF/):
//! - `FUN_0020f2b8` builds the table at 0x4165c0, stride 0x54, 32 entries [M].
//! - `FUN_0020bbe0` copies it per vehicle and sets `+0x3c = (int)(secs[w] · fps)` from the
//!   float table at 0x33a030 [M + ELF data]; `FUN_0020bd80` ranks the weapons by the order list
//!   at 0x33a4b0 [M + data]; the per-weapon barrel/action lists are at 0x33a0b0 (8 words each,
//!   0-terminated) [M + data].
//! - `FUN_0020de20` fires, `FUN_0020c078` (the per-tick weapon manager) reloads, `FUN_0020d378`
//!   picks up, `FUN_0020d420` removes, `FUN_0020bdf8`/`FUN_0020bf20`/`FUN_0020d598`/
//!   `FUN_0020d608`/`FUN_0020d4c0` cycle, `FUN_0020bff0` sets the gadget, `FUN_0020bf80` reads
//!   MISSILES_FIRED_SIMULTANEOUSLY, `FUN_00205498` gives the start loadout [M].
//! - `FUN_00180bc8` is the projectile (class `7Missile`) constructor with the per-kind flight
//!   numbers; `FUN_00181e70` is its homing/thrust step; `FUN_00182978` its life/hit pass [M].
//! - `FUN_0014ca60` is the fire event that turns a weapon id into a shell or a projectile and
//!   picks the damage [M]. `FUN_001d7620` registers the bullet-streak tuning [M].
//! - `FUN_00187608` (car vtable +0x1ac) books a hit; `FUN_0018ac28` (car vtable +0x254) is the
//!   EMP hit; `FUN_00188200` ends the EMP [M]; `EMP_LIFETIME` defaults to 120 at 0x3319b8
//!   (`FUN_00185668`) [M + data].
//!
//! Ticks: `now` is the sim tick counter (0x339f54) and `fps` the integer display rate at
//! 0x339f4c that `FUN_001feda0` stores once (60 on NTSC per docs/systems/drive-physics.md).
//!
//! Not ported (stated in the doc): the POV/on-foot weapon object path (`DAT_00327670`, ids
//! 0x17..0x1f in POV mode), sounds, animations, the explosion radius damage and the effects.

/// Number of weapon ids [M FUN_0020bbe0's 0x1f..0 loop and the 0x4165c0..0x417040 copy].
pub const WEAPON_COUNT: usize = 32;
/// Ids from here up are the "POV" weapons: the cycle list takes `w > 0x16` only in POV mode and
/// `w <= 0x16` otherwise [M FUN_0020bdf8].
pub const POV_FIRST: usize = 0x17;

pub const GUN: usize = 0;
pub const ROCKET: usize = 1;
pub const MISSILE: usize = 2;
pub const Q_BOOSTER: usize = 3;
pub const TIME_MINE: usize = 4;
pub const PROX_MINE: usize = 5;
pub const REM_MINE: usize = 6;
pub const FLOATING_MINE: usize = 7;
pub const DETONATOR: usize = 8;
pub const SPIKES: usize = 9;
pub const Q_SLICK: usize = 10;
pub const Q_SMOKE: usize = 11;
pub const Q_WORM: usize = 12;
pub const LASER: usize = 13;
pub const Q_PULSE: usize = 14;
pub const RADAR: usize = 15;
pub const CANNON: usize = 16;
pub const Q_VISION: usize = 17;
pub const TWO_WHEELS: usize = 18;
pub const GRENADE: usize = 19;
pub const Q_CHARGE: usize = 20;
pub const TORPEDO: usize = 21;
pub const REMOTE_TORPEDO: usize = 22;
pub const P2K: usize = 23;
pub const SNOWMOBILE_GUN: usize = 24;
pub const SNOWMOBILE_ROCKET: usize = 25;
pub const SNIPER_RIFLE: usize = 26;
pub const ULTRALIGHT_GUN: usize = 27;
pub const ULTRALIGHT_ROCKET: usize = 28;
pub const FOD_GUN: usize = 29;
pub const FOD_ROCKET: usize = 30;
pub const CH6_REMOTE_ROCKET: usize = 31;

/// One row of the built table (`FUN_0020f2b8`) plus the two side tables the copy reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponDef {
    /// +0x00.
    pub id: u8,
    /// +0x04 (three rows point at shared strings: 0x398a40 "Gun", 0x398b00 "EMP" is the Q-Pulse
    /// sound stem, 0x398ba8 "P2K").
    pub name: &'static str,
    /// +0x08/+0x0c/+0x10: three 16-bit ids (0x64b…, 3000 = 0xbb8 on most unused rows). Their
    /// reader is not traced; they are kept verbatim.
    pub text_ids: [u16; 3],
    /// +0x1c: rounds loaded at build time.
    pub clip: i32,
    /// +0x20: clip size.
    pub clip_max: i32,
    /// +0x34: damage. `FUN_0014ca60` multiplies it by the shooter's scale [M :LAB_0014dbe4].
    pub damage: i32,
    /// +0x38: when set, every shot calls `FUN_002068e0(mission, n)` (a shot counter; what the
    /// mission does with it is not traced).
    pub counts_shots: bool,
    /// +0x40: the sound bank passed to `FUN_002f4b10` (4 on every row; < 0 would mute).
    pub sfx_bank: i32,
    /// +0x44: the sound stem — "SFX_%sFire", "SFX_<stem>Reload", "SFX_<stem>Init" [M
    /// FUN_0014ca60 0x36f450, FUN_0020c078 0x3987f0/0x398808].
    pub sfx_stem: &'static str,
    /// The reload/refire interval in seconds, 0x33a030[w] [ELF data].
    pub interval_secs: f32,
    /// The vehicle action (barrel) ids the weapon cycles through, 0x33a0b0[w·8..] [ELF data].
    pub barrels: &'static [u8],
}

macro_rules! w {
    ($id:expr, $name:expr, [$a:expr, $b:expr, $c:expr], $clip:expr, $dmg:expr, $st:expr, $stem:expr,
     $secs:expr, $barrels:expr) => {
        WeaponDef {
            id: $id,
            name: $name,
            text_ids: [$a, $b, $c],
            clip: $clip,
            clip_max: $clip,
            damage: $dmg,
            counts_shots: $st,
            sfx_bank: 4,
            sfx_stem: $stem,
            interval_secs: f32::from_bits($secs),
            barrels: $barrels,
        }
    };
}

/// The table `FUN_0020f2b8` builds [M], every row's +0x1c equal to its +0x20 (checked by test),
/// with the interval seconds [0x33a030] and the barrel lists [0x33a0b0] beside it.
pub static WEAPONS: [WeaponDef; WEAPON_COUNT] = [
    w!(0, "Gun", [0x64b, 0x64b, 0x64b], 1, 10, true, "mntmach", 0x3e2aaaab, &[0x09]),
    w!(1, "Rocket", [0x642, 0x680, 0x680], 1, 250, true, "Rocket", 0x3e800000, &[0x05, 0x06]),
    w!(2, "Missile", [0x64f, 0x681, 0x681], 4, 250, true, "Missile", 0x3f2aaaab, &[0x07, 0x08, 0x34, 0x35]),
    w!(3, "Q-Booster", [0x64e, 0x68a, 0x68a], 1, 0, false, "Boost", 0x40800000, &[0x0d]),
    w!(4, "Time Mine", [0x65c, 0x65c, 0x65c], 1, 100, true, "", 0x3f800000, &[0x19]),
    w!(5, "Prox Mine", [0x65b, 0x65b, 0x65b], 1, 100, true, "Hmine", 0x3f800000, &[0x19]),
    w!(6, "Rem Mine", [0x65d, 0x65d, 0x65d], 1, 500, true, "", 0x3f800000, &[0x19]),
    w!(7, "Floating Mine", [0x65d, 0x65d, 0x65d], 1, 1000, true, "", 0x3f800000, &[0x41]),
    w!(8, "Detonator", [0xbb8, 0xbb8, 0xbb8], 1, 0, false, "", 0x3f800000, &[]),
    w!(9, "Spikes", [0xbb8, 0xbb8, 0xbb8], 1, 25, false, "", 0x40000000, &[0x17]),
    w!(10, "Q-Slick", [0x650, 0x67f, 0x67f], 1, 0, false, "", 0x40000000, &[0x0c]),
    w!(11, "Q-Smoke", [0x655, 0x682, 0x682], 1, 0, false, "smkscrn", 0x40000000, &[0x0e]),
    w!(12, "Q-Worm", [0xbb8, 0xbb8, 0xbb8], 1, 25, false, "", 0x40000000, &[]),
    w!(13, "Laser", [0x64a, 0x64a, 0x64a], 1, 25, false, "", 0x40000000, &[0x16]),
    w!(14, "Q-Pulse", [0x646, 0x68b, 0x68b], 1, 50, true, "EMP", 0x3f800000, &[0x24]),
    w!(15, "Radar", [0x668, 0xbb8, 0xbb8], 1, 0, false, "qlocate", 0x3f800000, &[0x36]),
    w!(16, "Cannon", [0xbb8, 0xbb8, 0xbb8], 1, 250, true, "cannon", 0x3f800000, &[0x48]),
    w!(17, "Q-Vision", [0x671, 0x685, 0x685], 1, 0, false, "", 0x40000000, &[]),
    w!(18, "2 Wheels", [0xbb8, 0xbb8, 0xbb8], 1, 0, false, "TwoWheel", 0x40000000, &[]),
    w!(19, "Grenade", [0xbb8, 0xbb8, 0xbb8], 1, 500, true, "grenade", 0x3f800000, &[0x26]),
    w!(20, "Q-Charge", [0xbb8, 0xbb8, 0xbb8], 1, 5000, false, "Hmine", 0x3f800000, &[]),
    w!(21, "Torpedo", [0x9f9, 0x9fb, 0x9fa], 2, 250, true, "vtorpedo", 0x40400000, &[0x3d, 0x3e]),
    w!(22, "Remote Torpedo", [0x9fc, 0x9fe, 0x9fd], 2, 250, true, "vGtorp", 0x3e800000, &[0x3f, 0x40]),
    w!(23, "P2K", [0xbb8, 0xbb8, 0xbb8], 10, 10, true, "HHandGun", 0x3daaaaab, &[0x21]),
    w!(24, "Snowmobile gun", [0xa8c, 0x686, 0x68c], 50, 50, true, "snomach", 0x3daaaaab, &[0x22]),
    w!(25, "Snowmobile Rocket", [0xa8d, 0x69d, 0x68d], 1, 250, true, "grenade", 0x3daaaaab, &[0x23]),
    w!(26, "Sniper Rifle", [0x9f6, 0x9f8, 0x9f7], 30, 10, true, "sniper", 0x3daaaaab, &[0x25]),
    w!(27, "Ultralight gun", [0xa90, 0xbb8, 0xbb8], 10, 30, true, "Umntmach", 0x3d4ccccd, &[0x2e]),
    w!(28, "Ultralight rocket", [0xa91, 0x683, 0x683], 10, 250, true, "Umissile", 0x3f800000, &[0x2d]),
    w!(29, "FOD Gun", [0x2205, 0x686, 0x68c], 50, 50, true, "fingergun", 0x3daaaaab, &[0x46]),
    w!(30, "FOD Rocket", [0x2206, 0x687, 0x68d], 1, 250, true, "finger", 0x40a00000, &[0x47]),
    w!(31, "CH-6 Remote Rocket", [0x66f, 0x693, 0x68e], 10, 250, true, "Rocket", 0x3daaaaab, &[0x2f]),
];

/// The cycle order list at 0x33a4b0 [ELF data]: Q-Pulse ranks right after Q-Booster.
pub const CYCLE_ORDER: [u8; WEAPON_COUNT] =
    [0, 1, 2, 3, 14, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];

/// `+0x3c` for one weapon: `(int)(secs · (float)fps)` [M FUN_0020bbe0].
pub fn interval_ticks(w: usize, fps: i32) -> i32 {
    (WEAPONS[w].interval_secs * fps as f32) as i32
}

/// Slot flag bits (+0x14) [M].
pub const OWNED: u32 = 1; // set by pickup, cleared by remove; the cycle list needs it
pub const BUSY: u32 = 2; // the weapon's current barrel action is running (FUN_001a2d10); blocks fire
pub const RELOADING: u32 = 4; // set after a shot; the manager refills the clip while it is set
pub const INIT_SFX_PENDING: u32 = 0x20;
pub const INIT_SFX_DONE: u32 = 0x40;

/// One vehicle's copy of a table row (the 0x54-byte record at `sys+0x10`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    pub flags: u32,
    /// +0x18.
    pub ammo: i32,
    /// +0x1c.
    pub clip: i32,
    /// +0x20.
    pub clip_max: i32,
    /// +0x24..+0x30: the tick each barrel last fired (-1000 at build).
    pub fired_at: [i32; 4],
    /// +0x34.
    pub damage: i32,
    /// +0x3c.
    pub interval: i32,
    /// +0x48: the next barrel to fire.
    pub fire_cursor: usize,
    /// +0x4c: the next barrel to reload.
    pub reload_cursor: usize,
    /// +0x50: shots fired.
    pub shots: i32,
}

/// The weapon manager's state for one vehicle (`DAT_0033a530` for the player).
#[derive(Clone, Debug, PartialEq)]
pub struct WeaponSystem {
    pub slots: [Slot; WEAPON_COUNT],
    /// +0x00 (-1 after reset).
    pub primary: i32,
    /// +0x04: the selected weapon the cycle moves (0 after reset).
    pub secondary: i32,
    /// +0x08: the gadget slot (-1 = none).
    pub gadget: i32,
    /// +0x0c: the swapped-out weapon when the POV mode flips.
    pub prev: i32,
    /// +0x18: the last weapon fired (-1 after reset).
    pub last_fired: i32,
    /// +0x4c: fire every barrel at once (missile salvo, torpedo pair). Reset sets it from the
    /// global option bit 0x1009a0 & 8 [M FUN_0020bbe0]; what sets that bit is not traced.
    pub salvo: bool,
    /// +0x50: no fire until `now > switch_until`.
    pub switch_until: i32,
    /// +0x88: rank[w] = w's index in CYCLE_ORDER.
    pub rank: [i32; WEAPON_COUNT],
    /// +0x108 / +0x188: the cycle list.
    pub cycle: Vec<i32>,
}

/// What one `fire` call did.
#[derive(Clone, Debug, PartialEq)]
pub enum FireResult {
    /// ammo < 1 (+0x18) [M :0x20df..].
    NoAmmo,
    /// The vehicle may only be hit by the EMP: "SFX_DryFire" unless the weapon is the gun or the
    /// Q-Pulse [M ONLY_TARGET_WITH_EMP check at the top of FUN_0020de20].
    DryFire,
    /// The switch delay, the refire interval, the BUSY flag or a barrel action still running.
    Blocked,
    /// Fired: the barrels whose action played (in order) and the rounds taken off the ammo.
    Fired { barrels: Vec<u8>, rounds: i32 },
}

/// The inputs `FUN_0020de20` reads from outside the weapon record.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireCtx {
    /// The current lock-on target carries ONLY_TARGET_WITH_EMP (the bomb van, [D] paris_bombvan.atr).
    pub target_only_emp: bool,
    /// `FUN_001a2d90(action) == 2` for the weapon's current barrel. Only consulted for the ids
    /// outside the exempt list {0,1,2,4,5,6,7,10,12,16,18,20}.
    pub barrel_ready: bool,
    /// The shooter's MISSILES_FIRED_SIMULTANEOUSLY / ROCKETS_FIRED_SIMULTANEOUSLY attrs
    /// (None = absent → 2) [M FUN_00169bf0 found-flag pattern].
    pub missiles_simul: Option<i32>,
    pub rockets_simul: Option<i32>,
    /// `DAT_0033a008` (no ammo cost and no reload flag); its writer is not traced.
    pub free_fire: bool,
    /// `0x1009a0 & 2` (no ammo cost only); its writer is not traced.
    pub free_ammo_bit: bool,
}

const FIRE_EXEMPT: [usize; 12] = [0, 1, 2, 0x10, 10, 4, 5, 6, 7, 0x14, 0xc, 0x12];

impl WeaponSystem {
    /// `FUN_0020bbe0`: copy the table, set the intervals for `fps`, reset the indices, rank the
    /// weapons [M]. `salvo_bit` is the 0x1009a0 & 8 option.
    pub fn new(fps: i32, salvo_bit: bool) -> Self {
        let mut slots = [Slot {
            flags: 0, ammo: 0, clip: 0, clip_max: 0, fired_at: [-1000; 4], damage: 0, interval: 0,
            fire_cursor: 0, reload_cursor: 0, shots: 0,
        }; WEAPON_COUNT];
        for (w, s) in slots.iter_mut().enumerate() {
            let d = &WEAPONS[w];
            s.clip = d.clip;
            s.clip_max = d.clip_max;
            s.damage = d.damage;
            s.interval = interval_ticks(w, fps);
        }
        // the copy then sets slot 0's flags |= 1 (`*(iVar4 + 0x14) |= 1`, iVar4 = the copy base)
        slots[0].flags |= OWNED;
        let mut rank = [-1; WEAPON_COUNT];
        for w in 0..WEAPON_COUNT {
            rank[w] = CYCLE_ORDER.iter().position(|&o| o as usize == w).map_or(-1, |i| i as i32);
        }
        WeaponSystem {
            slots, primary: -1, secondary: 0, gadget: -1, prev: 0, last_fired: -1, salvo: salvo_bit,
            switch_until: 0, rank, cycle: Vec::new(),
        }
    }

    /// `FUN_0020d378(sys, w, amount, clip_override, damage_override)` [M].
    pub fn pickup(&mut self, w: usize, amount: i32, clip_override: i32, damage_override: i32, pov: bool) {
        let s = &mut self.slots[w];
        s.flags |= OWNED;
        if clip_override > 0 && pov {
            s.clip_max = clip_override;
        }
        if damage_override > 0 {
            s.damage = damage_override;
        }
        if pov {
            s.clip = amount.min(s.clip_max);
        }
        s.ammo += amount;
    }

    /// `FUN_0020d420`: drop a weapon; clears the selection/gadget if it was either [M].
    pub fn remove(&mut self, w: usize) {
        self.slots[w].flags &= !OWNED;
        self.slots[w].ammo = 0;
        if w as i32 == self.secondary {
            self.secondary = -1;
        }
        if w as i32 == self.gadget {
            self.gadget = -1;
        }
    }

    /// `FUN_0020bff0`: the gadget slot. The old gadget's ammo is zeroed; gadget 2 (Missile) is
    /// instead given as 200 missiles on the selection with no gadget [M].
    pub fn set_gadget(&mut self, g: i32, pov: bool) {
        if self.gadget != -1 {
            self.slots[self.gadget as usize].ammo = 0;
        }
        if g == MISSILE as i32 {
            self.pickup(MISSILE, 200, 0, 0, pov);
            self.gadget = -1;
            self.secondary = MISSILE as i32;
        } else {
            self.gadget = g;
        }
    }

    /// `FUN_0020bf80`: a vehicle's MISSILES_FIRED_SIMULTANEOUSLY sets the missile clip to twice
    /// the value [M]. Absent attr: untouched (the table's 4).
    pub fn apply_missiles_simul(&mut self, attr: Option<i32>) {
        if let Some(v) = attr {
            self.slots[MISSILE].clip_max = v << 1;
            self.slots[MISSILE].clip = v << 1;
        }
    }

    /// `FUN_0020bdf8`: the cycle list — weapons of this mode that are the selection or owned with
    /// ammo, never the gadget, sorted by rank [M].
    pub fn build_cycle(&mut self, pov: bool) {
        let mut list = Vec::new();
        for w in 0..WEAPON_COUNT {
            let in_mode = if pov { w >= POV_FIRST } else { w < POV_FIRST };
            if !in_mode {
                continue;
            }
            let s = &self.slots[w];
            let eligible = w as i32 == self.secondary || (s.flags & OWNED != 0 && s.ammo > 0);
            if eligible && w as i32 != self.gadget {
                list.push(w as i32);
            }
        }
        // the bubble pass swaps only on a strictly smaller rank: a stable sort by rank
        let rank = self.rank;
        list.sort_by_key(|&w| rank[w as usize]);
        self.cycle = list;
    }

    /// `FUN_0020bf20`: the selection's index in the list, 0 when absent or the list is empty [M].
    pub fn cycle_index(&self) -> usize {
        self.cycle.iter().position(|&w| w == self.secondary).unwrap_or(0)
    }

    /// `FUN_0020d608` (next) / `FUN_0020d598` (previous). An empty list is rebuilt once; still
    /// empty → weapon 0 [M].
    pub fn neighbour(&mut self, next: bool, pov: bool) -> i32 {
        if self.cycle.is_empty() {
            self.build_cycle(pov);
            if self.cycle.is_empty() {
                return 0;
            }
        }
        let n = self.cycle.len();
        let i = self.cycle_index();
        if next { self.cycle[(i + 1) % n] } else { self.cycle[(i + n - 1) % n] }
    }

    /// `FUN_0020d4c0(sys, dir)` without its POV clip top-up: TOGGLESECONDARY/UP pass 1 (next),
    /// DOWN passes 0 (previous). A change starts the switch delay `now + fps/3` [M].
    pub fn select(&mut self, next: bool, now: i32, fps: i32, pov: bool) {
        let w = self.neighbour(next, pov);
        if w != self.secondary {
            self.secondary = w;
            self.switch_until = now + fps / 3;
        }
    }

    /// `FUN_0020de20` on the vehicle (non-POV) path [M].
    pub fn fire(&mut self, w: usize, now: i32, ctx: &FireCtx) -> FireResult {
        if ctx.target_only_emp && w != Q_PULSE && w != GUN {
            return FireResult::DryFire;
        }
        if self.slots[w].ammo < 1 {
            return FireResult::NoAmmo;
        }
        if now <= self.switch_until {
            return FireResult::Blocked;
        }
        {
            let s = &self.slots[w];
            if now < s.fired_at[s.fire_cursor] + s.interval {
                return FireResult::Blocked;
            }
            if s.flags & BUSY != 0 {
                return FireResult::Blocked;
            }
        }
        if !FIRE_EXEMPT.contains(&w) && !ctx.barrel_ready {
            return FireResult::Blocked;
        }
        let barrels = WEAPONS[w].barrels;
        let salvo = self.salvo;
        let mut played = Vec::new();
        let s = &mut self.slots[w];
        let shoot = |s: &mut Slot, played: &mut Vec<u8>| {
            let c = s.fire_cursor;
            s.fired_at[c] = now;
            s.fire_cursor = c + 1;
            played.push(barrels.get(c).copied().unwrap_or(0));
        };
        if w == MISSILE {
            if salvo && s.ammo >= s.clip_max {
                // the salvo restarts at barrel 0 and fires two extra barrels first [M 0x20e0..]
                s.fire_cursor = 0;
                shoot(s, &mut played);
                if s.clip_max == 4 {
                    s.clip -= 1;
                }
                shoot(s, &mut played);
                s.clip -= 1;
            }
            shoot(s, &mut played);
            if s.clip_max == 4 {
                s.clip -= 1;
            }
        } else if w == TORPEDO && salvo && s.ammo >= s.clip_max {
            s.fired_at[0] = now;
            s.fire_cursor = 0;
            shoot(s, &mut played);
            s.clip -= 1;
        }
        // the common barrel: fire, take a round off the clip, wrap at the list's 0 terminator
        shoot(s, &mut played);
        s.clip -= 1;
        if barrels.get(s.fire_cursor).copied().unwrap_or(0) == 0 {
            s.fire_cursor = 0;
        }
        s.shots += 1;
        // the ammo cost [M LAB_0020e5c0]
        let mut rounds = 0;
        if w != GUN && !ctx.free_fire && !ctx.free_ammo_bit {
            rounds = match w {
                ULTRALIGHT_ROCKET => 2,
                MISSILE => {
                    let n = ctx.missiles_simul.unwrap_or(2);
                    if salvo && n << 1 <= s.ammo { n << 1 } else { n }
                }
                ROCKET => ctx.rockets_simul.unwrap_or(2),
                TORPEDO => if salvo && s.ammo > 1 { 2 } else { 1 },
                _ => 1,
            };
            s.ammo -= rounds;
        }
        if !ctx.free_fire {
            s.flags |= RELOADING;
        }
        self.last_fired = w as i32;
        FireResult::Fired { barrels: played, rounds }
    }

    /// The reload pass `FUN_0020c078` runs each tick for the primary, the selection and the
    /// gadget (vehicle path): while RELOADING, every barrel whose last shot is older than
    /// `interval - 5` ticks loads one round if the ammo covers it; the pass stops when the reload
    /// cursor catches the fire cursor [M 0x20c9xx loop]. Returns the barrels reloaded.
    pub fn reload_tick(&mut self, now: i32) -> Vec<u8> {
        let mut out = Vec::new();
        let ids = [self.primary, self.secondary, self.gadget];
        for w in 0..WEAPON_COUNT {
            if !ids.contains(&(w as i32)) {
                continue;
            }
            let barrels = WEAPONS[w].barrels;
            let s = &mut self.slots[w];
            while s.flags & RELOADING != 0 {
                let rc = s.reload_cursor;
                if s.fired_at[rc.min(3)] + s.interval - 5 >= now {
                    break;
                }
                if s.ammo - s.clip <= 0 {
                    break;
                }
                out.push(barrels.get(rc).copied().unwrap_or(0));
                if w != MISSILE || rc & 1 != 0 || s.clip_max == 4 {
                    s.clip += 1;
                }
                s.reload_cursor = rc + 1;
                if barrels.get(s.reload_cursor).copied().unwrap_or(0) == 0 {
                    s.reload_cursor = 0;
                }
                if s.reload_cursor == s.fire_cursor {
                    s.flags &= !RELOADING;
                    break;
                }
            }
        }
        out
    }
}

/// `FUN_00205498`'s fallback loadout, used when the mission config pointer is the built-in
/// default record (0x4162b8, car "vanquish"): 99 of each, the selection set by `FUN_0020d488`.
/// `underwater` is the `uw_map` name compare (`FUN_002a9508`, strcmp-shaped) [M].
pub fn default_loadout(sys: &mut WeaponSystem, underwater: bool) {
    let (list, sel): (&[usize], usize) = if underwater {
        (&[TORPEDO, REMOTE_TORPEDO, Q_BOOSTER, FLOATING_MINE], TORPEDO)
    } else {
        (&[GUN, ROCKET, MISSILE, Q_PULSE, Q_SLICK, Q_SMOKE, Q_BOOSTER, RADAR], ROCKET)
    };
    for &w in list {
        sys.pickup(w, 99, 0, 0, false);
    }
    sys.secondary = sel as i32;
}

/// One weapon's entry in a mission config (`cfg+0x2c+w` enable byte, `+0x4c+4w` ammo,
/// `+0xcc+4w` clip, `+0x14c+4w` damage).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LoadoutEntry {
    pub enabled: bool,
    pub ammo: i32,
    pub clip: i32,
    pub damage: i32,
}

/// `FUN_00205498`'s mission path: every enabled weapon but the P2K (0x17) with ammo ≤ 10000 is
/// picked up; the selection is the first enabled one after the gun with ammo > 0, else the gun
/// with a zero pickup [M]. The Ultralight gun → rocket swap at the end (a name compare against
/// 0x397408) is left to the caller.
pub fn mission_loadout(sys: &mut WeaponSystem, cfg: &[LoadoutEntry; WEAPON_COUNT], pov: bool) -> i32 {
    let mut sel = -1;
    for w in 0..WEAPON_COUNT {
        if w == P2K || !cfg[w].enabled || cfg[w].ammo >= 10001 {
            continue;
        }
        let e = cfg[w];
        sys.pickup(w, e.ammo, e.clip, e.damage, pov);
        if sel == -1 && w != GUN && e.ammo >= 1 {
            sel = w as i32;
        }
    }
    if sel == -1 {
        sys.pickup(GUN, 0, 0, 0, pov);
        sel = 0;
    }
    sys.secondary = sel;
    sel
}

// ---------------------------------------------------------------- hit damage (FUN_0014ca60)

/// Who fired, as `FUN_0014ca60` sorts it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shooter {
    /// The shooter object's kind (+0x40) is 10 [M uStack_174].
    pub kind10: bool,
    /// The player's own shot in POV mode, or a remote-controlled shot [M iStack_168].
    pub pov_or_remote: bool,
    /// The damage scale `FUN_0014ca60` applies (1.0 for the player; see [`ai_damage_scale`]).
    pub scale: f32,
}

/// An AI car's damage scale: `clamp(100 + modifier, 0, 200) / 100` with the modifier the
/// signed byte at its driver record +0x131 [M].
pub fn ai_damage_scale(modifier: i8) -> f32 {
    (100 + modifier as i32).clamp(0, 200) as f32 / 100.0
}

/// The damage a shot carries [M LAB_0014dbe4 + the per-id cases]. Reads the *player's* table
/// (`DAT_0033a530+0x10`), so a pickup's damage override also changes the enemies' shots.
pub fn shot_damage(table: &WeaponSystem, w: usize, shooter: &Shooter) -> f32 {
    let base = match w {
        GUN | P2K => {
            if shooter.kind10 {
                5
            } else if shooter.pov_or_remote {
                table.slots[SNOWMOBILE_GUN].damage
            } else {
                table.slots[GUN].damage
            }
        }
        SNOWMOBILE_GUN | ULTRALIGHT_GUN | FOD_GUN => return table.slots[ULTRALIGHT_GUN].damage as f32,
        SNIPER_RIFLE => return table.slots[SNIPER_RIFLE].damage as f32,
        _ => table.slots[w].damage,
    };
    base as f32 * shooter.scale
}

/// How a weapon id leaves the barrel in `FUN_0014ca60` [M switch].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delivery {
    /// `FUN_001fffa8(a, b, …)`: the gun's (dmg, 0); the cannon's (0.75·dmg, 0.25·dmg). What the
    /// second value does is not traced.
    Shell { a: f32, b: f32 },
    /// A RayShell (`FUN_00198e80`, first float 220 or 400; its meaning is not traced).
    RayShell { first: f32, damage: f32 },
    /// A `Missile` object (`FUN_001ffc40` → `FUN_00180bc8`).
    Projectile { kind: usize, damage: f32 },
    /// The floating mine (`FUN_001ffd70`).
    FloatingMine,
    /// The grenade (`FUN_001ffea8`, first float 55), thrown with `dir.y = dir.y·1.6 + 0.2`.
    Grenade,
    /// No projectile from the fire event (the gadgets act through their own switch cases).
    None,
}

pub fn delivery(table: &WeaponSystem, w: usize, shooter: &Shooter) -> Delivery {
    let dmg = shot_damage(table, w, shooter);
    match w {
        GUN | P2K => Delivery::Shell { a: dmg, b: 0.0 },
        CANNON => Delivery::Shell { a: dmg * 0.75, b: dmg * 0.25 },
        SNOWMOBILE_GUN | ULTRALIGHT_GUN | FOD_GUN => Delivery::RayShell { first: 220.0, damage: dmg },
        SNIPER_RIFLE => Delivery::RayShell { first: 400.0, damage: dmg },
        ROCKET | MISSILE | Q_PULSE | TORPEDO | REMOTE_TORPEDO | SNOWMOBILE_ROCKET | ULTRALIGHT_ROCKET
        | FOD_ROCKET | CH6_REMOTE_ROCKET => Delivery::Projectile { kind: w, damage: dmg },
        FLOATING_MINE => Delivery::FloatingMine,
        GRENADE => Delivery::Grenade,
        _ => Delivery::None,
    }
}

/// The AI aim roll: skill = clamp(driver +0x130 + mission bonus (`FUN_00208428`), 0, 100);
/// the shot goes at the exact aim point when `rand16·100 >> 16 < skill`, else at the wobbled
/// one [M]. `rand16` is `FUN_0017dc28`'s value.
pub fn aim_is_exact(accuracy: i8, mission_bonus: i32, rand16: i32) -> bool {
    let skill = (accuracy as i32 + mission_bonus).clamp(0, 100);
    (rand16 * 100) >> 16 < skill
}

/// The wobble amplitude by the difficulty word (mission +0x3b8): 1 → 0.01, 2 → 0.005, else
/// 0.015 [M]. The wobble is `2·amp·sin(now·0.02 + seed)` on two axes (`FUN_0017c788`, `FUN_00210b60`).
pub fn aim_spread(difficulty: i32) -> f32 {
    match difficulty {
        1 => 0.01,
        2 => 0.005,
        _ => 0.015,
    }
}

// ---------------------------------------------------------------- bullet streaks

/// The streak type names, index = the .atr BULLET_STREAK_TYPE value [M 0x3349b0 list read by
/// `FUN_001d7620`]; the FX .tun carries BulletStreak{Colour,Texture,Width,Length,Speed,Count}{name}.
pub const STREAK_TYPES: [&str; 7] = ["Henchmen", "Vanquish", "SuperSnowmobile", "SUV", "Captain", "Ultralight", "Sentinel"];

/// Every `count`-th shot of a shooter draws a streak: the shooter's counter (+100) steps and
/// wraps at BulletStreakCount; only the shot that found it at 0 streaks [M FUN_0014ca60].
pub fn streak_this_shot(counter: &mut i32, count: i32) -> bool {
    let old = *counter;
    *counter += 1;
    if *counter >= count {
        *counter = 0;
    }
    old == 0
}

// ---------------------------------------------------------------- the projectile

/// The flight numbers `FUN_00180bc8` sets per kind [M].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectileParams {
    /// +0x70: the cruise speed the thrust aims for.
    pub max_speed: f32,
    /// +0x74: the thrust (velocity change per second).
    pub accel: f32,
    /// +0x78: the homing turn fraction.
    pub turn: f32,
    /// +0x7c (torpedo only, -1 or 0.8; its reader is not traced).
    pub torpedo_7c: f32,
    /// The launch speed: shooter speed + bonus, or a fixed value.
    pub launch_speed: f32,
    /// +0x84: ticks before homing starts.
    pub homing_delay: i32,
    /// +0x90: ticks of life.
    pub life: i32,
    /// +0x8c: ticks the spent object lingers after the hit (2·fps; the Q-Pulse fps).
    pub linger: i32,
    /// +0x708: already lined up (turns at the full rate).
    pub aligned: bool,
    /// Takes the current lock (`FUN_0022af58`) as its target.
    pub locks: bool,
    /// +0x718: the damage (the weapon's, or a torpedo's own).
    pub damage: f32,
    /// +0x88 offset to the first contact check, ticks.
    pub arm_delay: i32,
}

/// Who launched it, for the torpedo's branches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Launcher {
    Player,
    /// An AI car; `alpha_sub` is the car name compare against 0x378530 "alpha_sub";
    /// `lock_dist2` the squared distance to the lock, if any.
    AiCar { alpha_sub: bool, lock_dist2: Option<f32> },
    /// Anything else (no kind-1 owner).
    Other,
}

/// `FUN_00180bc8` [M]. `shooter_speed` is the launcher's speed (param_1), `damage` the
/// fire event's value. CH-6 (31) reads its numbers from 0x331930.. (34.5, 7.2, 2.0·0.05, +3,
/// 6 s) [ELF data]. Missile against a kind-10 / kind-11 target is not covered here.
pub fn launch_params(kind: usize, shooter_speed: f32, damage: f32, fps: i32, launcher: Launcher) -> ProjectileParams {
    let f = fps as f32;
    let mut p = ProjectileParams {
        max_speed: 200.0, accel: 0.0, turn: 1.0, torpedo_7c: 0.0, launch_speed: 200.0,
        homing_delay: (f * 0.1) as i32, life: fps * 5, linger: fps << 1, aligned: true, locks: false,
        damage, arm_delay: 0,
    };
    match kind {
        ROCKET => {
            p.accel = 125.0; p.max_speed = 250.0; p.launch_speed = shooter_speed + 5.0;
            p.homing_delay = (f * 0.2) as i32; p.arm_delay = 10;
        }
        MISSILE => {
            p.locks = true; p.max_speed = 250.0; p.accel = 50.0; p.aligned = false;
            p.launch_speed = shooter_speed + 7.0; p.homing_delay = (f * 0.2) as i32; p.life = fps << 2;
        }
        Q_PULSE => {
            p.locks = true; p.max_speed = 175.0; p.accel = 17.5; p.aligned = false;
            p.launch_speed = shooter_speed + 4.0; p.homing_delay = (f * 0.33) as i32; p.linger = fps;
        }
        TORPEDO => {
            p.locks = true; p.homing_delay = 0; p.launch_speed = shooter_speed + 1.0;
            let (sp, ac, tu, x, life, dmg) = match launcher {
                Launcher::AiCar { alpha_sub: true, .. } => (20.0, 10.0, f32::from_bits(0x3cf5c28f), 0.8, fps * 5, None),
                Launcher::AiCar { lock_dist2: Some(d2), .. } if d2 < 10000.0 => (60.0, 30.0, 1.0, -1.0, fps << 3, Some(2000.0)),
                Launcher::AiCar { .. } => (40.0, 12.0, f32::from_bits(0x3c23d70a), 0.8, fps << 3, Some(750.0)),
                _ => (50.0, 25.0, f32::from_bits(0x3dcccccd), -1.0, fps * 5, None),
            };
            p.max_speed = sp; p.accel = ac; p.turn = tu; p.torpedo_7c = x; p.life = life;
            if let Some(d) = dmg { p.damage = d; }
        }
        REMOTE_TORPEDO => {
            p.accel = 4.0; p.max_speed = 40.0; p.launch_speed = 8.0; p.aligned = false;
            p.homing_delay = (f * 0.33) as i32; p.life = fps * 10;
        }
        SNOWMOBILE_ROCKET => {
            p.max_speed = 250.0; p.accel = 125.0; p.homing_delay = 0; p.launch_speed = shooter_speed + 5.0;
        }
        ULTRALIGHT_ROCKET => {
            p.locks = true; p.max_speed = 800.0; p.accel = 80.0; p.launch_speed = 10.0;
            p.homing_delay = (f * 0.5) as i32;
        }
        FOD_ROCKET => {
            p.max_speed = 300.0; p.launch_speed = 300.0;
        }
        CH6_REMOTE_ROCKET => {
            p.max_speed = 34.5; p.accel = f32::from_bits(0x40e66666); p.turn = 2.0 * 0.05;
            p.launch_speed = shooter_speed + 3.0; p.life = fps * 6; p.arm_delay = 6;
        }
        _ => {}
    }
    p
}

type V3 = [f32; 3];
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn len(a: V3) -> f32 { dot(a, a).sqrt() }
fn scale(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn norm(a: V3) -> V3 { let l = len(a); if l > 0.0 { scale(a, 1.0 / l) } else { a } }

/// Rotate unit `a` toward unit `b` by fraction `t` of the angle (the forward axis of the game's
/// quaternion slerp `FUN_002cc018`; the roll the game also carries is not modelled).
fn slerp_dir(a: V3, b: V3, t: f32) -> V3 {
    let c = dot(a, b).clamp(-1.0, 1.0);
    let th = c.acos();
    if th < 1e-6 {
        return a;
    }
    let s = th.sin();
    let wa = ((1.0 - t) * th).sin() / s;
    let wb = (t * th).sin() / s;
    norm(add(scale(a, wa), scale(b, wb)))
}

/// A flying projectile: the fields of the `Missile` object the step reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projectile {
    pub kind: usize,
    pub p: ProjectileParams,
    pub pos: V3,
    pub vel: V3,
    /// The body's forward axis.
    pub fwd: V3,
    /// +0x94: the launch tick.
    pub born: i32,
    /// The lock is still held (+0x6c non-null).
    pub has_target: bool,
    /// +0x70c: detonate this tick.
    pub detonate: bool,
}

/// The target the step reads: its position and velocity, and whether it is a ground object
/// (target +0x34 != 0, the 0.25 proximity rule) — for a car lock that word is not traced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub pos: V3,
    pub vel: V3,
    pub prox_kind: bool,
}

impl Projectile {
    pub fn launch(kind: usize, p: ProjectileParams, pos: V3, dir: V3, now: i32) -> Self {
        let d = norm(dir);
        Projectile { kind, p, pos, vel: scale(d, p.launch_speed), fwd: d, born: now, has_target: p.locks, detonate: false }
    }

    /// One tick of `FUN_00181e70` (homing + thrust, AI/auto path, no player steering, no
    /// wobble) then the life count of `FUN_00182978` and the position step [M except the
    /// last: the body integration runs in `FUN_001fd448`'s physics object and is modelled as
    /// `pos += vel·dt`, unproven].
    pub fn step(&mut self, target: Option<Target>, now: i32, dt: f32, fps: i32) {
        let speed = len(self.vel);
        let fwd0 = self.fwd;
        let mut want = self.p.max_speed;
        if !self.has_target {
            self.p.aligned = true;
        } else if !self.p.aligned {
            want *= 0.3;
        }
        if self.p.max_speed < self.p.accel {
            self.p.accel = self.p.max_speed;
        }
        let age = now - self.born;
        let homing = self.has_target && age > self.p.homing_delay;
        match (homing, target) {
            (true, Some(t)) => {
                let aim_pt = add(t.pos, scale(t.vel, dt * 3.0));
                let to = sub(aim_pt, self.pos);
                let d2 = dot(to, to);
                let aim = norm(to);
                if d2 > 4.0 {
                    let mut turn = if self.p.aligned { self.p.turn } else { self.p.turn * 0.5 };
                    let ramp = (age - self.p.homing_delay) as f32 * dt * 0.5;
                    if ramp <= 1.0 {
                        turn *= ramp;
                    }
                    self.fwd = slerp_dir(self.fwd, aim, turn);
                    self.vel = scale(self.fwd, speed);
                }
                let c = dot(aim, norm(self.vel));
                want *= c.max(0.1);
                if t.prox_kind && d2 <= 0.25 {
                    self.detonate = true;
                }
                if self.kind == TORPEDO {
                    if c < 0.5 {
                        self.has_target = false;
                        self.p.life = fps >> 3; // +0x90 = fps >> 3: it dies a few ticks later
                    }
                } else if !t.prox_kind && d2 <= 2.0 {
                    self.has_target = false;
                }
                if c > 0.99 {
                    self.p.aligned = true;
                }
            }
            (true, None) => self.has_target = false,
            _ => {
                if age <= self.p.homing_delay && speed > 0.0 {
                    // before homing the body turns 0.1 toward its flight direction
                    self.fwd = slerp_dir(self.fwd, norm(self.vel), 0.1);
                }
            }
        }
        let a = if speed < want { self.p.accel } else { -self.p.accel };
        self.vel = add(self.vel, scale(fwd0, a * dt));
        self.p.life -= 1;
        if self.p.life < 1 {
            self.detonate = true;
        }
        self.pos = add(self.pos, scale(self.vel, dt));
    }
}

// ---------------------------------------------------------------- the car's side

/// The two words a car keeps for weapon hits [M FUN_00187608 / FUN_0018ac28 / FUN_00188200].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarHits {
    /// +0x370: damage taken (reset by `FUN_00187350` / `FUN_00187578`).
    pub taken: f32,
    /// +0x374: set to fps on each hit (its reader is not traced).
    pub hit_timer: u8,
    /// +0x380 / +0x384: EMP'd until the tick.
    pub emp: bool,
    pub emp_until: i32,
}

/// `EMP_LIFETIME` default, ticks [0x3319b8 = 120, `FUN_00185668`].
pub const EMP_LIFETIME: i32 = 120;

impl CarHits {
    /// `FUN_00187608(damage, car)`: add the hit; for a kind-3 car or an IS_POLICE car, more
    /// than 499 taken (or more than its +0x54 value, default 1.0) raises mission request 2
    /// (`FUN_00208540(mission, 2)`, the +0x254 word; its consumer is not traced). Returns
    /// whether the request was raised.
    pub fn hit(&mut self, damage: f32, fps: i32, kind3: bool, is_police: bool, limit: f32) -> bool {
        self.taken += damage;
        self.hit_timer = fps as u8;
        if !kind3 && !is_police {
            return false;
        }
        self.taken > 499.0 || limit < self.taken
    }

    /// `FUN_0018ac28`: the Q-Pulse hit; a car already EMP'd is not extended.
    pub fn emp_hit(&mut self, now: i32, lifetime: i32) -> bool {
        if self.emp {
            return false;
        }
        self.emp = true;
        self.emp_until = now + lifetime;
        true
    }

    /// `FUN_00188200`'s end check: the EMP clears once `emp_until < now`.
    pub fn emp_tick(&mut self, now: i32) {
        if self.emp && self.emp_until < now {
            self.emp = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FPS: i32 = 60;

    #[test]
    fn table_rows_match_the_builder() {
        // [M FUN_0020f2b8] id/name/clip/damage/stem per row
        let want: [(&str, i32, i32, &str); 32] = [
            ("Gun", 1, 10, "mntmach"), ("Rocket", 1, 250, "Rocket"), ("Missile", 4, 250, "Missile"),
            ("Q-Booster", 1, 0, "Boost"), ("Time Mine", 1, 100, ""), ("Prox Mine", 1, 100, "Hmine"),
            ("Rem Mine", 1, 500, ""), ("Floating Mine", 1, 1000, ""), ("Detonator", 1, 0, ""),
            ("Spikes", 1, 25, ""), ("Q-Slick", 1, 0, ""), ("Q-Smoke", 1, 0, "smkscrn"),
            ("Q-Worm", 1, 25, ""), ("Laser", 1, 25, ""), ("Q-Pulse", 1, 50, "EMP"),
            ("Radar", 1, 0, "qlocate"), ("Cannon", 1, 250, "cannon"), ("Q-Vision", 1, 0, ""),
            ("2 Wheels", 1, 0, "TwoWheel"), ("Grenade", 1, 500, "grenade"), ("Q-Charge", 1, 5000, "Hmine"),
            ("Torpedo", 2, 250, "vtorpedo"), ("Remote Torpedo", 2, 250, "vGtorp"), ("P2K", 10, 10, "HHandGun"),
            ("Snowmobile gun", 50, 50, "snomach"), ("Snowmobile Rocket", 1, 250, "grenade"),
            ("Sniper Rifle", 30, 10, "sniper"), ("Ultralight gun", 10, 30, "Umntmach"),
            ("Ultralight rocket", 10, 250, "Umissile"), ("FOD Gun", 50, 50, "fingergun"),
            ("FOD Rocket", 1, 250, "finger"), ("CH-6 Remote Rocket", 10, 250, "Rocket"),
        ];
        for (i, (d, w)) in WEAPONS.iter().zip(want.iter()).enumerate() {
            assert_eq!(d.id as usize, i);
            assert_eq!((d.name, d.clip_max, d.damage, d.sfx_stem), (w.0, w.1, w.2, w.3), "row {i}");
            assert_eq!(d.clip, d.clip_max);
            assert_eq!(d.sfx_bank, 4);
        }
        assert_eq!(WEAPONS[SNIPER_RIFLE].text_ids, [0x9f6, 0x9f8, 0x9f7]);
        // +0x38 clear only on these rows
        let silent: Vec<usize> = (0..32).filter(|&w| !WEAPONS[w].counts_shots).collect();
        assert_eq!(silent, vec![3, 8, 9, 10, 11, 12, 13, 15, 17, 18, 20]);
    }

    #[test]
    fn intervals_and_barrels_from_the_elf_data() {
        // 0x33a030 · 60, truncated [M FUN_0020bbe0]
        let ticks: Vec<i32> = (0..32).map(|w| interval_ticks(w, FPS)).collect();
        assert_eq!(ticks, vec![10, 15, 40, 240, 60, 60, 60, 60, 60, 120, 120, 120, 120, 120, 60, 60, 60, 120, 120,
                               60, 60, 180, 15, 5, 5, 5, 5, 3, 60, 5, 300, 5]);
        assert_eq!(WEAPONS[MISSILE].barrels, &[7, 8, 0x34, 0x35]);
        assert_eq!(WEAPONS[TORPEDO].barrels, &[0x3d, 0x3e]);
        assert!(WEAPONS[DETONATOR].barrels.is_empty());
        assert_eq!(CYCLE_ORDER[4], 14);
    }

    #[test]
    fn reset_state() {
        let s = WeaponSystem::new(FPS, false);
        assert_eq!((s.primary, s.secondary, s.gadget, s.last_fired), (-1, 0, -1, -1));
        assert_eq!(s.slots[0].flags, OWNED);
        assert_eq!(s.slots[5].fired_at, [-1000; 4]);
        assert_eq!(s.rank[14], 4);
        assert_eq!(s.rank[4], 5);
    }

    fn armed() -> WeaponSystem {
        let mut s = WeaponSystem::new(FPS, false);
        default_loadout(&mut s, false);
        s.primary = 0;
        s
    }

    #[test]
    fn default_loadout_and_the_cycle() {
        let mut s = armed();
        assert_eq!(s.secondary, ROCKET as i32);
        s.build_cycle(false);
        // ranked: Q-Pulse (14) sits after Q-Booster (3)
        assert_eq!(s.cycle, vec![0, 1, 2, 3, 14, 10, 11, 15]);
        s.select(true, 100, FPS, false);
        assert_eq!((s.secondary, s.switch_until), (MISSILE as i32, 120));
        s.select(false, 200, FPS, false);
        s.select(false, 200, FPS, false);
        assert_eq!(s.secondary, GUN as i32);
        s.select(false, 200, FPS, false); // wraps to the last
        assert_eq!(s.secondary, RADAR as i32);
        // the gadget never enters the list
        s.gadget = Q_SMOKE as i32;
        s.build_cycle(false);
        assert!(!s.cycle.contains(&(Q_SMOKE as i32)));
        let mut u = WeaponSystem::new(FPS, false);
        default_loadout(&mut u, true);
        assert_eq!(u.secondary, TORPEDO as i32);
    }

    #[test]
    fn rocket_fire_gating_ammo_and_reload() {
        let mut s = armed();
        let ctx = FireCtx::default();
        // the switch delay
        s.switch_until = 50;
        assert_eq!(s.fire(ROCKET, 50, &ctx), FireResult::Blocked);
        let r = s.fire(ROCKET, 51, &ctx);
        assert_eq!(r, FireResult::Fired { barrels: vec![5], rounds: 2 });
        assert_eq!((s.slots[ROCKET].ammo, s.slots[ROCKET].clip, s.slots[ROCKET].fire_cursor), (97, 0, 1));
        assert!(s.slots[ROCKET].flags & RELOADING != 0);
        // second barrel is free at once, then both wait for 15 ticks
        assert!(matches!(s.fire(ROCKET, 52, &ctx), FireResult::Fired { .. }));
        assert_eq!(s.slots[ROCKET].fire_cursor, 0);
        assert_eq!(s.fire(ROCKET, 65, &ctx), FireResult::Blocked);
        assert!(matches!(s.fire(ROCKET, 66, &ctx), FireResult::Fired { .. }));
        // reload: a barrel reloads once 15 - 5 ticks have passed since it fired
        let mut t = armed();
        t.secondary = ROCKET as i32;
        t.fire(ROCKET, 100, &ctx);
        assert!(t.reload_tick(110).is_empty());
        assert_eq!(t.reload_tick(111), vec![5]);
        assert_eq!(t.slots[ROCKET].clip, 1);
        assert_eq!(t.slots[ROCKET].flags & RELOADING, 0, "reload cursor met the fire cursor");
        // ROCKETS_FIRED_SIMULTANEOUSLY and the free-ammo flag
        let mut u = armed();
        assert_eq!(u.fire(ROCKET, 10, &FireCtx { rockets_simul: Some(1), ..ctx }),
                   FireResult::Fired { barrels: vec![5], rounds: 1 });
        assert_eq!(u.fire(ROCKET, 11, &FireCtx { free_ammo_bit: true, ..ctx }),
                   FireResult::Fired { barrels: vec![6], rounds: 0 });
    }

    #[test]
    fn gun_costs_nothing_and_missiles_fire_in_pairs() {
        let mut s = armed();
        let ctx = FireCtx::default();
        assert_eq!(s.fire(GUN, 1, &ctx), FireResult::Fired { barrels: vec![9], rounds: 0 });
        assert_eq!(s.slots[GUN].ammo, 99);
        assert_eq!(s.fire(GUN, 10, &ctx), FireResult::Blocked);
        assert!(matches!(s.fire(GUN, 11, &ctx), FireResult::Fired { .. }));
        // missiles: two barrels per fire, two rounds (no attr), clip 4 → 2
        assert_eq!(s.fire(MISSILE, 1, &ctx), FireResult::Fired { barrels: vec![7, 8], rounds: 2 });
        assert_eq!(s.slots[MISSILE].clip, 2);
        assert_eq!(s.fire(MISSILE, 2, &ctx), FireResult::Fired { barrels: vec![0x34, 0x35], rounds: 2 });
        assert_eq!(s.slots[MISSILE].fire_cursor, 0);
        // salvo: all four, twice the rounds
        let mut v = armed();
        v.salvo = true;
        assert_eq!(v.fire(MISSILE, 1, &ctx), FireResult::Fired { barrels: vec![7, 8, 0x34, 0x35], rounds: 4 });
        assert_eq!(v.slots[MISSILE].clip, 0);
        // the bomb van only takes the EMP
        let emp_only = FireCtx { target_only_emp: true, ..ctx };
        assert_eq!(armed().fire(MISSILE, 1, &emp_only), FireResult::DryFire);
        assert!(matches!(armed().fire(Q_PULSE, 1, &FireCtx { barrel_ready: true, ..emp_only }), FireResult::Fired { .. }));
        // non-exempt ids wait for the barrel action
        assert_eq!(armed().fire(Q_SMOKE, 1, &ctx), FireResult::Blocked);
        assert!(matches!(armed().fire(Q_SMOKE, 1, &FireCtx { barrel_ready: true, ..ctx }), FireResult::Fired { .. }));
        // no ammo
        assert_eq!(WeaponSystem::new(FPS, false).fire(ROCKET, 1, &ctx), FireResult::NoAmmo);
    }

    #[test]
    fn pickups_gadget_and_missile_attr() {
        let mut s = WeaponSystem::new(FPS, false);
        s.pickup(SNIPER_RIFLE, 45, 12, 80, true);
        assert_eq!((s.slots[26].ammo, s.slots[26].clip, s.slots[26].clip_max, s.slots[26].damage), (45, 12, 12, 80));
        s.pickup(ROCKET, 5, 3, 0, false);
        assert_eq!((s.slots[1].clip_max, s.slots[1].clip), (1, 1), "vehicle pickups leave the clip");
        s.set_gadget(Q_SMOKE as i32, false);
        assert_eq!(s.gadget, Q_SMOKE as i32);
        s.slots[Q_SMOKE].ammo = 3;
        s.set_gadget(MISSILE as i32, false);
        assert_eq!((s.gadget, s.secondary, s.slots[Q_SMOKE].ammo, s.slots[MISSILE].ammo), (-1, 2, 0, 200));
        s.apply_missiles_simul(Some(1));
        assert_eq!((s.slots[MISSILE].clip_max, s.slots[MISSILE].clip), (2, 2));
        s.remove(MISSILE);
        assert_eq!((s.secondary, s.slots[MISSILE].ammo, s.slots[MISSILE].flags & OWNED), (-1, 0, 0));
    }

    #[test]
    fn mission_loadout_picks_the_first_armed_weapon() {
        let mut cfg = [LoadoutEntry::default(); WEAPON_COUNT];
        cfg[GUN] = LoadoutEntry { enabled: true, ammo: 99, ..Default::default() };
        cfg[P2K] = LoadoutEntry { enabled: true, ammo: 5, ..Default::default() };
        cfg[Q_SMOKE] = LoadoutEntry { enabled: true, ammo: 3, ..Default::default() };
        cfg[MISSILE] = LoadoutEntry { enabled: true, ammo: 20000, ..Default::default() };
        let mut s = WeaponSystem::new(FPS, false);
        assert_eq!(mission_loadout(&mut s, &cfg, false), Q_SMOKE as i32);
        assert_eq!((s.slots[P2K].ammo, s.slots[MISSILE].ammo), (0, 0));
        let mut t = WeaponSystem::new(FPS, false);
        assert_eq!(mission_loadout(&mut t, &[LoadoutEntry::default(); WEAPON_COUNT], false), 0);
    }

    #[test]
    fn shot_damage_rules() {
        let mut s = WeaponSystem::new(FPS, false);
        let player = Shooter { scale: 1.0, ..Default::default() };
        assert_eq!(shot_damage(&s, GUN, &player), 10.0);
        assert_eq!(shot_damage(&s, GUN, &Shooter { pov_or_remote: true, ..player }), 50.0);
        assert_eq!(shot_damage(&s, GUN, &Shooter { kind10: true, scale: 2.0, ..player }), 10.0);
        assert_eq!(shot_damage(&s, FOD_GUN, &player), 30.0, "FOD gun reads the ultralight gun row");
        assert_eq!(shot_damage(&s, SNIPER_RIFLE, &Shooter { scale: 2.0, ..player }), 10.0);
        let ai = Shooter { scale: ai_damage_scale(-40), ..player };
        assert_eq!(shot_damage(&s, MISSILE, &ai), 150.0);
        assert_eq!(ai_damage_scale(127), 2.0);
        assert_eq!(ai_damage_scale(-128), 0.0);
        assert_eq!(delivery(&s, CANNON, &player), Delivery::Shell { a: 187.5, b: 62.5 });
        assert_eq!(delivery(&s, SNIPER_RIFLE, &player), Delivery::RayShell { first: 400.0, damage: 10.0 });
        // a pickup override changes every later shot of that id
        s.pickup(ROCKET, 1, 0, 400, false);
        assert_eq!(delivery(&s, ROCKET, &player), Delivery::Projectile { kind: ROCKET, damage: 400.0 });
        // aim roll + spread
        assert!(aim_is_exact(50, 0, 0x7fff));
        assert!(!aim_is_exact(50, 0, 0x8000));
        assert!(!aim_is_exact(-20, 10, 0));
        assert_eq!((aim_spread(1), aim_spread(2), aim_spread(0)), (0.01, 0.005, 0.015));
    }

    #[test]
    fn streaks_and_names() {
        assert_eq!(STREAK_TYPES[1], "Vanquish");
        assert_eq!(STREAK_TYPES[6], "Sentinel");
        let mut c = 0;
        let shots: Vec<bool> = (0..6).map(|_| streak_this_shot(&mut c, 3)).collect();
        assert_eq!(shots, vec![true, false, false, true, false, false]);
        let mut one = 0;
        assert!((0..4).all(|_| streak_this_shot(&mut one, 1)));
    }

    #[test]
    fn launch_table() {
        let r = launch_params(ROCKET, 30.0, 250.0, FPS, Launcher::Player);
        assert_eq!((r.max_speed, r.accel, r.launch_speed, r.homing_delay, r.life, r.locks), (250.0, 125.0, 35.0, 12, 300, false));
        let m = launch_params(MISSILE, 30.0, 250.0, FPS, Launcher::Player);
        assert_eq!((m.max_speed, m.accel, m.launch_speed, m.life, m.locks, m.aligned), (250.0, 50.0, 37.0, 240, true, false));
        let e = launch_params(Q_PULSE, 0.0, 50.0, FPS, Launcher::Player);
        assert_eq!((e.max_speed, e.accel, e.homing_delay, e.linger), (175.0, 17.5, 19, 60));
        let t = launch_params(TORPEDO, 0.0, 250.0, FPS, Launcher::AiCar { alpha_sub: false, lock_dist2: Some(50.0) });
        assert_eq!((t.max_speed, t.damage, t.life), (60.0, 2000.0, 480));
        let t2 = launch_params(TORPEDO, 0.0, 250.0, FPS, Launcher::AiCar { alpha_sub: false, lock_dist2: None });
        assert_eq!((t2.max_speed, t2.damage), (40.0, 750.0));
        let t3 = launch_params(TORPEDO, 0.0, 250.0, FPS, Launcher::AiCar { alpha_sub: true, lock_dist2: Some(1.0) });
        assert_eq!((t3.max_speed, t3.damage, t3.life), (20.0, 250.0, 300));
        let tp = launch_params(TORPEDO, 3.0, 250.0, FPS, Launcher::Player);
        assert_eq!((tp.max_speed, tp.accel, tp.launch_speed, tp.life, tp.homing_delay), (50.0, 25.0, 4.0, 300, 0));
        let u = launch_params(ULTRALIGHT_ROCKET, 99.0, 250.0, FPS, Launcher::Player);
        assert_eq!((u.max_speed, u.launch_speed, u.homing_delay), (800.0, 10.0, 30));
        let c = launch_params(CH6_REMOTE_ROCKET, 0.0, 250.0, FPS, Launcher::Player);
        assert_eq!((c.max_speed, c.accel, c.life), (34.5, 7.2, 360));
        assert!((c.turn - 0.1).abs() < 1e-7);
        let f = launch_params(FOD_ROCKET, 99.0, 250.0, FPS, Launcher::Player);
        assert_eq!((f.launch_speed, f.max_speed, f.accel, f.homing_delay), (300.0, 300.0, 0.0, 6));
    }

    #[test]
    fn unguided_rocket_accelerates_to_cruise_and_dies() {
        let dt = 1.0 / 60.0;
        let p = launch_params(ROCKET, 0.0, 250.0, FPS, Launcher::Player);
        let mut r = Projectile::launch(ROCKET, p, [0.0; 3], [0.0, 0.0, 1.0], 0);
        assert_eq!(r.vel, [0.0, 0.0, 5.0]);
        r.step(None, 1, dt, FPS);
        assert!((r.vel[2] - (5.0 + 125.0 * dt)).abs() < 1e-4);
        for t in 2..300 {
            r.step(None, t, dt, FPS);
        }
        // above cruise the thrust brakes: it dithers around 250
        assert!((r.vel[2] - 250.0).abs() < 125.0 * dt + 1e-3);
        assert!(!r.detonate);
        r.step(None, 300, dt, FPS);
        assert!(r.detonate, "life of 5·fps ticks");
    }

    #[test]
    fn missile_homes_onto_a_side_target() {
        let dt = 1.0 / 60.0;
        let p = launch_params(MISSILE, 20.0, 250.0, FPS, Launcher::Player);
        let mut m = Projectile::launch(MISSILE, p, [0.0; 3], [0.0, 0.0, 1.0], 0);
        let tgt = Target { pos: [100.0, 0.0, 100.0], vel: [0.0; 3], prox_kind: false };
        // before the 12-tick delay it flies straight
        for t in 1..=12 {
            m.step(Some(tgt), t, dt, FPS);
        }
        assert_eq!(m.fwd, [0.0, 0.0, 1.0]);
        let mut closest = f32::MAX;
        for t in 13..240 {
            m.step(Some(tgt), t, dt, FPS);
            let d = len(sub(tgt.pos, m.pos));
            closest = closest.min(d);
            if !m.has_target {
                break;
            }
        }
        assert!(closest < 1.5, "closest {closest}");
        assert!(m.p.aligned);
    }

    #[test]
    fn car_hits_and_emp() {
        let mut c = CarHits::default();
        assert!(!c.hit(600.0, FPS, false, false, 1.0));
        assert_eq!((c.taken, c.hit_timer), (600.0, 60));
        let mut police = CarHits::default();
        assert!(!police.hit(250.0, FPS, false, true, 1000.0));
        assert!(police.hit(250.0, FPS, false, true, 1000.0), "500 > 499");
        let mut k3 = CarHits::default();
        assert!(k3.hit(5.0, FPS, true, false, 1.0), "over its own limit");
        let mut e = CarHits::default();
        assert!(e.emp_hit(1000, EMP_LIFETIME));
        assert!(!e.emp_hit(1050, EMP_LIFETIME));
        assert_eq!(e.emp_until, 1120);
        e.emp_tick(1120);
        assert!(e.emp);
        e.emp_tick(1121);
        assert!(!e.emp);
    }
}
