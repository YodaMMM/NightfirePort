//! The driving engine's hostile AI and AI vehicle lifecycle (docs/systems/driving-ai.md §10).
//!
//! Every function here is a port of one DRIVING.ELF function, read in the decompile and, where
//! the decompile drops an argument, the disassembly (EE = file offset + 0x107080):
//!
//! - the AI element database: load `FUN_0012b8b0`, per-element range check `FUN_0012afb8`,
//!   distance/priority `FUN_0012b440`, the slot manager `FUN_0012bec8`, spawn `FUN_0012cb08`,
//!   despawn `FUN_0012c740`, the "may despawn" test `FUN_0012dc18`;
//! - the traffic vehicle set `FUN_0013fae0` (which pvehicle `.atr` each traffic car is built
//!   from);
//! - the chase target speed `FUN_0012e808` (profiles 1, 0xd, 0xe, 0xf — the arms the traffic
//!   port left as `None`);
//! - the ram roll and the target sight rule of `FUN_00131770`;
//! - the weapons: dispatch `FUN_00130050`, machine guns `FUN_0012f730`, rockets
//!   `FUN_0012fa28`, missiles `FUN_0012fd58`, and the line-of-fire test `FUN_0012f530`.
//!
//! The port is headless and pure: the caller supplies the live sim values the game reads off
//! its actors (positions, velocities, health, wheel state, ray hits) and acts on the returned
//! decisions. No function here draws from a random stream except `ram_tick` and
//! `traffic_types`, which take the driving generator (`ai::DriveRng`) explicitly.

use crate::driving::ai::DriveRng;
use crate::driving::CarParams;
use nf_formats::driveai::{flag, AiElement, MapAi, TYPE_HENCHMAN, TYPE_VEHICLE};

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn len(a: [f32; 3]) -> f32 {
    let s = dot(a, a);
    if s <= 0.0 { 0.0 } else { s.sqrt() }
}
/// The VU normalize; a zero vector stays zero [S, as in `ai.rs`].
fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 0.0 { [a[0] / l, a[1] / l, a[2] / l] } else { [0.0; 3] }
}

// ------------------------------------------------------------------------------------------
// Constants
// ------------------------------------------------------------------------------------------

/// Slot defaults when the world attribute has no `HENCH_SLOTS_AVAIL` / `VEHICLE_SLOTS_AVAIL` /
/// `HELI_SLOTS_AVAIL` [M 0x12b8b0: 0x20 / 6 / 0x10; names 0x36b120/0x36b138/0x36b150 [D]].
pub const HENCH_SLOTS_DEFAULT: i32 = 32;
pub const VEHICLE_SLOTS_DEFAULT: i32 = 6;
pub const HELI_SLOTS_DEFAULT: i32 = 16;
/// Per-element runtime radii: in range below 200, a live element despawns beyond 220
/// [M FUN_0012af48: +0x10 = 0x43480000, +0x14 = 0x435c0000; read in 0x12afb8].
pub const ACTIVATE_RADIUS: f32 = 200.0;
pub const DEACTIVATE_RADIUS: f32 = 220.0;
/// The range-check period grows by one tick per out-of-range check up to 4 and resets to 1 in
/// range [M FUN_0012b0e0 `< 4 → +1`, FUN_0012b0d0 `= 1`].
pub const CHECK_PERIOD_MAX: i32 = 4;
/// The manager's mission flag 1 is cleared while a live element with health > 0 is inside 120
/// units and set otherwise [M 0x12bec8 tail: `< 120.0` → FUN_00208568(mgr, 1) (flag = 0) else
/// FUN_00208540(mgr, 1) (flag = 1); the flags live at mgr+0x24c+i·4].
pub const HOSTILES_NEAR_RADIUS: f32 = 120.0;
/// A vehicle spawn parks every traffic car within `its box + 6.0` of the spawn point
/// [M 0x12cb08:166-195].
pub const SPAWN_TRAFFIC_CLEAR_PAD: f32 = 6.0;
/// The sight range of `FUN_001330d0`: a target beyond 200 is never seen [M 0x1330d0].
pub const SIGHT_RANGE: f32 = 200.0;
/// The sight test runs every `(short)(int)(fps · 0.25)` ticks [M 0x131770].
pub const SIGHT_PERIOD_S: f32 = 0.25;
/// The ram roll: every 5 s, `rand/65536 < chance` → ram for 3 s, 1 s when the target is the
/// player [M 0x131770, disassembly 0x131960-0x1319ec].
pub const RAM_ROLL_S: i32 = 5;
pub const RAM_S: i32 = 3;
pub const RAM_S_VS_PLAYER: i32 = 1;
/// `1/65536` as the game spells it (0x37800000) [M 0x131988].
pub const RAND_SCALE: f32 = 1.525_878_9e-5;
/// Aim cones: machine guns 0.99, rockets 0.9, missiles 0.707 [M 0x12f730 / 0x12fa28 / 0x12fd58];
/// the line-of-fire block cone 0.99 [M 0x12f530].
pub const GUN_CONE: f32 = 0.99;
pub const ROCKET_CONE: f32 = 0.9;
pub const MISSILE_CONE: f32 = 0.707;
pub const LOF_CONE: f32 = 0.99;
/// Rockets and missiles never reload faster than half a second [M 0x12fa28 / 0x12fd58: the
/// `max(fps·rate, fps·0.5)`]; machine guns have no floor [M 0x12f730].
pub const LAUNCHER_MIN_S: f32 = 0.5;

/// AIVehicle profiles (AI +0x84) the chase arms switch on [M 0x12e808].
pub const PROFILE_WAIT: u32 = 0x1;
pub const PROFILE_PURSUE: u32 = 0xd;
pub const PROFILE_ESCORT: u32 = 0xe;
pub const PROFILE_ROAD: u32 = 0xf;
/// Weapon profiles that bypass the per-weapon flags [M 0x130050].
pub const PROFILE_GUNS_ONLY: u32 = 9;
pub const PROFILE_ROCKETS_ONLY: u32 = 10;

// ------------------------------------------------------------------------------------------
// The world attribute and the traffic set
// ------------------------------------------------------------------------------------------

/// The integer keys the driving engine reads from the world attribute (`data\sim\attrib\world\
/// <world>.atr`) through `FUN_00169bf0(DAT_0033aac8, name, &found)` [M 0x12b8b0, 0x13fae0].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorldAttr {
    pub hench_slots: Option<i32>,
    pub vehicle_slots: Option<i32>,
    pub heli_slots: Option<i32>,
    pub max_traffic: Option<i32>,
    pub traffic_types: Option<i32>,
}

impl WorldAttr {
    /// Reads the file's first section — the one named after the world (`[paris]`, `[jungle]`,
    /// …); the mission sections after it (`[junglea_mis13a]`) carry chain keys only [D every
    /// world .atr]. Which attribute node DAT_0033aac8 resolves to was not traced [S].
    pub fn parse(text: &str) -> WorldAttr {
        let mut w = WorldAttr::default();
        let mut sections = 0;
        for line in text.lines() {
            let line = line.split("//").next().unwrap_or("").trim();
            if line.starts_with('[') {
                sections += 1;
                continue;
            }
            if sections > 1 {
                break;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            let k = k.trim().split('.').next().unwrap_or("").trim();
            let Ok(v) = v.trim().parse::<i32>() else { continue };
            match k {
                "HENCH_SLOTS_AVAIL" => w.hench_slots = Some(v),
                "VEHICLE_SLOTS_AVAIL" => w.vehicle_slots = Some(v),
                "HELI_SLOTS_AVAIL" => w.heli_slots = Some(v),
                "MAX_TRAFFIC" => w.max_traffic = Some(v),
                "NUM_TRAFFIC_TYPES_PS2" => w.traffic_types = Some(v),
                _ => {}
            }
        }
        w
    }

    /// `[hench, vehicle, heli]` slot budgets [M 0x12b8b0].
    pub fn slots(&self) -> [i32; 3] {
        [
            self.hench_slots.unwrap_or(HENCH_SLOTS_DEFAULT),
            self.vehicle_slots.unwrap_or(VEHICLE_SLOTS_DEFAULT),
            self.heli_slots.unwrap_or(HELI_SLOTS_DEFAULT),
        ]
    }
}

/// The traffic vehicle names, the table at 0x36c070 [D: pointers to 0x36c028..0x36c068].
pub const TRAFFIC_TYPE_NAMES: [&str; 7] = ["coupe", "sedan2", "hatchback", "sedan", "station_wagon", "suv1", "taxi"];

/// `FUN_0013fae0` 0x13fae0: the traffic set. `max = MAX_TRAFFIC` (20 when absent); `types =
/// NUM_TRAFFIC_TYPES_PS2`, or when absent `max` if `(uint)max < 8` else 7. With fewer than 7
/// types each type slot is drawn as `rand·7 >> 16`, else slot i = i (the slot fill runs once,
/// DAT_0032b756). Then car i uses slot `i % types` and draws its colour as `n·rand >> 16`, n =
/// the attribute FUN_00196818 returns for the type (1 when it returns 0) [M]. Returns
/// `(pvehicle name, colour)` per car; `colours(name)` stands in for FUN_00196818 [S: it is
/// read here as the type's colour count, `RENDER_NUMCOLORS` in the shipped .atr files — the
/// attribute name FUN_00196818 reads was not traced]. No cars without a road network
/// (DAT_003c8254) — the caller checks that.
pub fn traffic_types(w: &WorldAttr, rng: &mut DriveRng, colours: &dyn Fn(&str) -> i32) -> Vec<(&'static str, i32)> {
    let max = w.max_traffic.unwrap_or(20);
    let types: i32 = match w.traffic_types {
        Some(t) => t,
        None => {
            if (max as u32) < 8 { max } else { 7 }
        }
    };
    let mut slots = Vec::new();
    for i in 0..types.max(0) {
        if (types as u32) < 7 {
            slots.push(((rng.next() as u32 * 7) >> 16) as usize);
        } else {
            slots.push(i as usize);
        }
    }
    let mut out = Vec::new();
    if types <= 0 {
        return out;
    }
    for i in 0..max.max(0) {
        let ty = slots[(i % types) as usize];
        let name = TRAFFIC_TYPE_NAMES[ty.min(6)];
        let n = colours(name);
        let n = if n != 0 { n } else { 1 };
        let colour = (n * rng.next() as i32) >> 16;
        out.push((name, colour));
    }
    out
}

/// A mission's AI vehicles with their own car parameters: each spawned vehicle is built from
/// `data\sim\attrib\pvehicle\<name>.atr` (`FUN_001862c0` 0x1862c0 → `FUN_001968a8(obj, name)`
/// at +0x5c) [M], not from the player's car. `CarParams::parse_atr` fills keys the file omits
/// from the schema defaults; the attribute system's own fallback (pvehicle `default.atr`) is not
/// modelled [S].
#[derive(Debug, Clone, Default)]
pub struct AiVehicleSet {
    pub cars: Vec<(String, CarParams)>,
}

impl AiVehicleSet {
    pub fn insert(&mut self, name: &str, atr_text: &str) {
        self.cars.retain(|(n, _)| n != name);
        self.cars.push((name.to_string(), CarParams::parse_atr(atr_text)));
    }
    pub fn params(&self, name: &str) -> Option<&CarParams> {
        self.cars.iter().find(|(n, _)| n == name).map(|(_, p)| p)
    }
}

// ------------------------------------------------------------------------------------------
// The AI element database (AIEl lifecycle)
// ------------------------------------------------------------------------------------------

/// `FUN_0012cb08`'s difficulty filter on AIEl +0x42 against the mission difficulty (mgr
/// +0x3b8, compared unsigned): 1 → {1, 2}; 2 → {2}; 3 → {0}; 4 → {1}; 5 → {0, 1}; 6 → {0, 2};
/// anything else → every difficulty [M 0x12cb08:113-146]. Which difficulty number is which
/// menu setting was not traced.
pub fn difficulty_allows(filter: u16, difficulty: u32) -> bool {
    match filter {
        1 => difficulty.wrapping_sub(1) <= 1,
        2 => difficulty == 2,
        3 => difficulty == 0,
        4 => difficulty == 1,
        5 => difficulty <= 1,
        6 => difficulty == 0 || difficulty == 2,
        _ => true,
    }
}

/// `FUN_0012dc18`: an element may be despawned by range unless it is PERSISTENT [M].
pub fn may_despawn(flags: u16) -> bool {
    flags & flag::PERSISTENT == 0
}

/// AIEl +0x48 [M 0x12c740 writes 0 / 2, 0x12cb08 writes 1].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElemState {
    /// 0: can spawn.
    Idle,
    /// 1: live.
    Live,
    /// 2: gone for the rest of the mission.
    Gone,
}

/// The live actor values the database reads off a live element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveStatus {
    /// The vehicle controller's position (`FUN_0013e0b8`) / the character's +0x30 [M 0x12b440].
    pub pos: [f32; 3],
    /// The health the actor writes through its pointer to AIEl +0x4c [M 0x12cb08 FUN_00195a08].
    pub health: f32,
    /// Wheels still attached (`vtbl+0x13c` = FUN_001879c8, 0..4) [M].
    pub wheels: u8,
    /// The vehicle's +0x3e8 word (`vtbl+0x6c` = FUN_00192ff0); ≠ 0 → gone on despawn [M; the
    /// field's meaning was not traced].
    pub vehicle_3e8: bool,
    /// For a henchman: the character's vtbl+0x14 query (≠ 0 → it may come back) [M 0x12c740].
    pub hench_returns: bool,
}

impl LiveStatus {
    pub fn at(pos: [f32; 3], health: f32) -> LiveStatus {
        LiveStatus { pos, health, wheels: 4, vehicle_3e8: false, hench_returns: true }
    }
}

/// What the database asks the world to do.
#[derive(Debug, Clone, PartialEq)]
pub enum AiAction {
    /// Create the element's actor at its record transform.
    Spawn(Spawn),
    /// Remove the element's actor (and its riders); `then` is the element's new state.
    Despawn { index: usize, then: ElemState },
}

/// A spawn order [M 0x12cb08].
#[derive(Debug, Clone, PartialEq)]
pub struct Spawn {
    pub index: usize,
    /// The vehicle sim kind: 2 = AIVehicle in chase state 5 (`FUN_0013f748`), 4 = no AIVehicle
    /// controller (`FUN_0013f118` has no arm for 4) [M]; 0 for henchmen and helicopters.
    pub sim_kind: u32,
    pub heli: bool,
    pub pos: [f32; 3],
    pub facing: [f32; 3],
    /// Traffic cars within `their box + pad` of `pos` are parked (vehicles only) [M].
    pub clear_traffic_pad: Option<f32>,
    /// The riders spawned (index into the element's riders; health > 0, a known seat) [M].
    pub riders: Vec<usize>,
}

/// One element's runtime record (0x50 B, init `FUN_0012af48`) [M].
#[derive(Debug, Clone, PartialEq)]
pub struct ElementRt {
    /// +0x00 the tick of the next range check.
    pub next_check: i32,
    /// +0x04 the period (1..4).
    pub period: i32,
    /// +0x08 distance to the player.
    pub dist: f32,
    /// +0x0c the slot priority `(dot + 1)·0.5 / dist` [M 0x12b440].
    pub key: f32,
    /// +0x18 in range (dist < 200).
    pub in_range: bool,
    /// +0x1c kept by the slot pass.
    pub keep: bool,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub rec: AiElement,
    pub state: ElemState,
    pub rt: ElementRt,
    /// The riders this spawn created (the controllers at +0xe0+k) [M].
    pub live_riders: Vec<usize>,
}

/// The AI element database at DAT_00327c30 [M 0x12b8b0].
#[derive(Debug, Clone)]
pub struct ElementDb {
    pub elements: Vec<Element>,
    /// +0x00/+0x04/+0x08: [hench, vehicle, heli] slot budgets.
    pub slots: [i32; 3],
    /// +0x44/+0x48/+0x4c: live counts.
    pub live: [i32; 3],
    /// The mission difficulty (mgr +0x3b8).
    pub difficulty: u32,
    /// DAT_00326000: henchmen and riders are created only when it is set [M 0x12cb08:278,445];
    /// the global's writer was not traced.
    pub characters_on: bool,
    /// mgr +0x3dc: the mission's hostile count, +1 per element (+ riders) at the first spawn
    /// unless UNCOUNTED [M 0x12cb08].
    pub kill_count: i32,
    /// +0x34: the nearest live element (health > 0) checked this tick (1e6 when none) [M 0x12b440].
    pub nearest: f32,
    /// mgr flag 1: no live hostile inside 120 [M 0x12bec8 tail].
    pub clear_of_hostiles: bool,
}

impl ElementDb {
    /// `FUN_0012b8b0` 0x12b8b0: copy the records, scale every henchman's health (+0x4c) and
    /// every vehicle's health and rider healths (+0x14) by the difficulty health factor
    /// (`FUN_002084a8`: MSet f32 +0x2c4/+0x2c8/+0x2cc by difficulty 0/1/2, +0x2c4 otherwise),
    /// runtime records next = now, period 1 [M]. The snow2a_mis4 `paris_hench` → `junglesnow`
    /// swap (when the PTP word 0x100984 == 2) runs on the map before this:
    /// `hostiles::snow_hench_swap` (it needs the attribute's rendername) [M 0x12b8b0:96-125].
    pub fn new(map: &MapAi, slots: [i32; 3], difficulty: u32, health_factor: f32, now: i32) -> ElementDb {
        let elements = map
            .elements
            .iter()
            .map(|e| {
                let mut rec = e.clone();
                if rec.kind == TYPE_VEHICLE {
                    for r in &mut rec.riders {
                        r.health *= health_factor;
                    }
                }
                if rec.kind == TYPE_HENCHMAN || rec.kind == TYPE_VEHICLE {
                    rec.health *= health_factor;
                }
                let state = match rec.state {
                    1 => ElemState::Live,
                    2 => ElemState::Gone,
                    _ => ElemState::Idle,
                };
                Element {
                    rec,
                    state,
                    rt: ElementRt { next_check: now, period: 1, dist: 0.0, key: 0.0, in_range: false, keep: false },
                    live_riders: Vec::new(),
                }
            })
            .collect();
        ElementDb {
            elements,
            slots,
            live: [0; 3],
            difficulty,
            characters_on: true,
            kill_count: 0,
            nearest: 1.0e6,
            clear_of_hostiles: true,
        }
    }

    /// `FUN_0012b440`: distance to the player and the slot priority. The element's position is
    /// the live actor's while live, else the record's +0x30. `key = (dot(n(pos − player),
    /// n(fwd)) + 1)·0.5 / dist`, floored at 0, 1.0 at dist 0; `fwd` is the player object's +0x20
    /// row (DAT_00333ed8 → +4 → +0x20) [M; read as the player car's velocity, the +0x20 of the
    /// sim layout §8 — S]. Live elements with health > 0 update the nearest distance; a live
    /// ground vehicle with fewer than 4 wheels is flagged DAMAGED [M].
    fn measure(&mut self, i: usize, live: Option<LiveStatus>, player: [f32; 3], fwd: [f32; 3]) {
        let e = &mut self.elements[i];
        let pos = match (e.state, live) {
            (ElemState::Live, Some(l)) => {
                e.rec.health = l.health;
                l.pos
            }
            _ => e.rec.pos,
        };
        let d = sub(pos, player);
        let dist = len(d);
        e.rt.dist = dist;
        let c = dot(normalize(fwd), normalize(d));
        e.rt.key = if dist > 0.0 { ((1.0 / dist) * (c + 1.0) * 0.5).max(0.0) } else { 1.0 };
        if e.state == ElemState::Live && e.rec.health > 0.0 {
            self.nearest = self.nearest.min(dist);
            if e.rec.kind == TYPE_VEHICLE && !e.rec.heli {
                if let Some(l) = live {
                    if l.wheels < 4 {
                        e.rec.flags |= flag::DAMAGED;
                    }
                }
            }
        }
    }

    /// `FUN_0012c740` 0x12c740: despawn a live element. KEEP_PLACE copies the live position
    /// back into the record. A henchman stays respawnable when its character says so, else is
    /// gone. A vehicle with health ≤ 0 is gone; alive, it is gone when its +0x3e8 word is set
    /// or a wheel is missing, else it may respawn. A helicopter: gone at health ≤ 0, else may
    /// respawn. The live counts drop (riders included) [M].
    pub fn despawn(&mut self, i: usize, live: Option<LiveStatus>) -> AiAction {
        let e = &mut self.elements[i];
        if e.state != ElemState::Live {
            return AiAction::Despawn { index: i, then: e.state };
        }
        let l = live.unwrap_or(LiveStatus::at(e.rec.pos, e.rec.health));
        if e.rec.flags & flag::KEEP_PLACE != 0 {
            e.rec.pos = l.pos;
        }
        e.rec.health = l.health;
        let then = if e.rec.kind == TYPE_HENCHMAN {
            self.live[0] -= 1;
            if l.hench_returns { ElemState::Idle } else { ElemState::Gone }
        } else {
            self.live[0] -= e.live_riders.len() as i32;
            e.live_riders.clear();
            let (ok, lost_wheel) = if e.rec.heli {
                self.live[2] -= 1;
                (true, false)
            } else {
                self.live[1] -= 1;
                (!l.vehicle_3e8, l.wheels < 4)
            };
            if l.health > 0.0 && ok && !lost_wheel { ElemState::Idle } else { ElemState::Gone }
        };
        e.state = then;
        AiAction::Despawn { index: i, then }
    }

    /// `FUN_0012cb08` 0x12cb08: spawn an idle element. Not idle, or filtered out by difficulty →
    /// nothing. Henchmen need `characters_on` and a +0x58 class of 0, 10 or 0x12 [M 0x12cb08:
    /// 445-499]. A ground vehicle parks overlapping traffic and is created as sim kind 2 (CHASE_
    /// BRAIN) or 4; a helicopter takes the heli pool. The first spawn adds 1 + riders to the
    /// kill counter unless UNCOUNTED, and sets COUNTED. Riders with health > 0 and a known seat
    /// are seated when `characters_on`; a rider with an unknown seat ends the seating (the
    /// switch default reaches the `controller == 0 → return`) [M 0x12cb08:280-440].
    pub fn spawn(&mut self, i: usize) -> Option<AiAction> {
        let difficulty = self.difficulty;
        let characters_on = self.characters_on;
        let e = &mut self.elements[i];
        if e.state != ElemState::Idle || !difficulty_allows(e.rec.difficulty, difficulty) {
            return None;
        }
        let mut s = Spawn {
            index: i,
            sim_kind: 0,
            heli: e.rec.heli,
            pos: e.rec.pos,
            facing: e.rec.facing,
            clear_traffic_pad: None,
            riders: Vec::new(),
        };
        if e.rec.kind == TYPE_VEHICLE {
            if e.rec.heli {
                self.live[2] += 1;
            } else {
                s.sim_kind = if e.rec.flags & flag::CHASE_BRAIN != 0 { 2 } else { 4 };
                s.clear_traffic_pad = Some(SPAWN_TRAFFIC_CLEAR_PAD);
                self.live[1] += 1;
            }
            e.state = ElemState::Live;
            if e.rec.flags & flag::COUNTED == 0 && e.rec.flags & flag::UNCOUNTED == 0 {
                self.kill_count += 1 + e.rec.riders.len() as i32;
            }
            e.rec.flags |= flag::COUNTED;
            if characters_on {
                for (k, r) in e.rec.riders.iter().enumerate() {
                    if r.health > 0.0 {
                        if r.seat_attr().is_none() {
                            break;
                        }
                        s.riders.push(k);
                    }
                }
            }
            self.live[0] += s.riders.len() as i32;
            e.live_riders = s.riders.clone();
            Some(AiAction::Spawn(s))
        } else if e.rec.kind == TYPE_HENCHMAN {
            if !characters_on || !matches!(e.rec.hench_class, 0 | 10 | 0x12) {
                return None;
            }
            e.state = ElemState::Live;
            self.live[0] += 1;
            if e.rec.flags & flag::COUNTED == 0 && e.rec.flags & flag::UNCOUNTED == 0 {
                self.kill_count += 1;
            }
            e.rec.flags |= flag::COUNTED;
            Some(AiAction::Spawn(s))
        } else {
            None
        }
    }

    /// `FUN_0012afb8` 0x12afb8: one element's range check. Skipped when gone or not yet due.
    /// In range (< 200): period 1. Out of range: period +1 up to 4, and a live element beyond
    /// 220 despawns. A HELD live element despawns too. Despawns only when `may_despawn`. Next
    /// check = now + period [M; disassembly 0x12b000-0x12b088].
    fn range_check(&mut self, i: usize, now: i32, player: [f32; 3], fwd: [f32; 3], live: Option<LiveStatus>, out: &mut Vec<AiAction>) {
        let st = self.elements[i].state;
        if st == ElemState::Gone || now < self.elements[i].rt.next_check {
            return;
        }
        self.measure(i, live, player, fwd);
        let e = &mut self.elements[i];
        let mut drop = false;
        if e.rt.dist < ACTIVATE_RADIUS {
            e.rt.period = 1;
            e.rt.in_range = true;
        } else {
            if e.rt.period < CHECK_PERIOD_MAX {
                e.rt.period += 1;
            }
            e.rt.in_range = false;
            if e.state == ElemState::Live && DEACTIVATE_RADIUS < e.rt.dist {
                drop = true;
            }
        }
        if e.rec.flags & flag::HELD != 0 && e.state == ElemState::Live {
            drop = true;
        }
        let flags = e.rec.flags;
        if drop && may_despawn(flags) {
            out.push(self.despawn(i, live));
        }
        let e = &mut self.elements[i];
        e.rt.next_check = now + e.rt.period;
    }

    /// `FUN_0012bec8` 0x12bec8, called once per world frame by `FUN_00159d10` [M]. For every
    /// element: the range check; then the wanted set = idle, not HELD, and (in range or
    /// PERSISTENT); the candidates = every non-HELD element that is in range or PERSISTENT.
    /// When the wanted henchmen (+ riders), vehicles and helicopters all fit the free slots,
    /// every wanted element spawns. Otherwise the candidates are sorted by priority (highest
    /// first), PERSISTENT ones reserve their slots first, then the rest take what is left
    /// (a vehicle only when its riders fit too); live elements that got no slot despawn and
    /// idle ones that got one spawn. Last, mission flag 1 = `nearest ≥ 120` [M].
    ///
    /// `live(i)` returns the live actor values of a live element. `player` is the position the
    /// manager measures from (the first controller's sim, DAT_00339f5c[0] [M 0x12bec8:66]).
    pub fn tick(&mut self, now: i32, player: [f32; 3], player_fwd: [f32; 3], live: &dyn Fn(usize) -> Option<LiveStatus>) -> Vec<AiAction> {
        let mut out = Vec::new();
        self.nearest = 1.0e6;
        let mut wanted: Vec<usize> = Vec::new();
        let mut cand: Vec<usize> = Vec::new();
        let (mut want_hench, mut want_veh, mut want_heli) = (0i32, 0i32, 0i32);
        for i in 0..self.elements.len() {
            let l = if self.elements[i].state == ElemState::Live { live(i) } else { None };
            self.range_check(i, now, player, player_fwd, l, &mut out);
            let e = &mut self.elements[i];
            e.rt.keep = false;
            if !e.rt.in_range && may_despawn(e.rec.flags) {
                continue;
            }
            let held = e.rec.flags & flag::HELD != 0;
            if e.state == ElemState::Idle {
                if held {
                    continue;
                }
                match (e.rec.kind, e.rec.heli) {
                    (TYPE_HENCHMAN, _) => want_hench += 1,
                    (TYPE_VEHICLE, false) => {
                        want_veh += 1;
                        want_hench += e.rec.riders.len() as i32;
                    }
                    (TYPE_VEHICLE, true) => {
                        want_heli += 1;
                        want_hench += e.rec.riders.len() as i32;
                    }
                    _ => {}
                }
                wanted.push(i);
            }
            if !held {
                cand.push(i);
            }
        }
        if want_hench + want_veh + want_heli > 0 {
            let fits = self.slots[0] - self.live[0] >= want_hench
                && self.slots[1] - self.live[1] >= want_veh
                && want_heli <= self.slots[2] - self.live[2];
            if fits {
                for i in wanted {
                    if let Some(a) = self.spawn(i) {
                        out.push(a);
                    }
                }
            } else {
                // FUN_002a8198 = qsort(cand, n, 8, FUN_0012dbf0): -1 unless a.key < b.key →
                // descending by key. The sort is not stable; ties keep index order here [S].
                cand.sort_by(|&a, &b| {
                    self.elements[b].rt.key.partial_cmp(&self.elements[a].rt.key).unwrap_or(std::cmp::Ordering::Equal)
                });
                let (mut h, mut v, mut c) = (self.slots[0], self.slots[1], self.slots[2]);
                for &i in &cand {
                    let e = &mut self.elements[i];
                    if may_despawn(e.rec.flags) {
                        continue;
                    }
                    match (e.rec.kind, e.rec.heli) {
                        (TYPE_HENCHMAN, _) => h -= 1,
                        (TYPE_VEHICLE, false) => {
                            v -= 1;
                            h -= e.rec.riders.len() as i32;
                        }
                        (TYPE_VEHICLE, true) => c -= 1,
                        _ => continue,
                    }
                    e.rt.keep = true;
                }
                for &i in &cand {
                    let e = &mut self.elements[i];
                    if !may_despawn(e.rec.flags) {
                        continue;
                    }
                    let n = e.rec.riders.len() as i32;
                    match (e.rec.kind, e.rec.heli) {
                        (TYPE_HENCHMAN, _) => {
                            if h > 0 {
                                h -= 1;
                                e.rt.keep = true;
                            }
                        }
                        (TYPE_VEHICLE, false) => {
                            if v > 0 && n <= h {
                                h -= n;
                                v -= 1;
                                e.rt.keep = true;
                            }
                        }
                        (TYPE_VEHICLE, true) => {
                            if c > 0 {
                                c -= 1;
                                e.rt.keep = true;
                            }
                        }
                        _ => {}
                    }
                }
                for i in 0..self.elements.len() {
                    if self.elements[i].state == ElemState::Live && !self.elements[i].rt.keep {
                        let l = live(i);
                        out.push(self.despawn(i, l));
                    }
                }
                for i in 0..self.elements.len() {
                    if self.elements[i].state == ElemState::Idle && self.elements[i].rt.keep {
                        if let Some(a) = self.spawn(i) {
                            out.push(a);
                        }
                    }
                }
            }
        }
        self.clear_of_hostiles = !(self.nearest < HOSTILES_NEAR_RADIUS);
        out
    }
}

// ------------------------------------------------------------------------------------------
// The chase target speed (FUN_0012e808's chase arms)
// ------------------------------------------------------------------------------------------

/// The AIVehicle and sim values `FUN_0012e808` 0x12e808 reads on the chase arms (AI offsets).
#[derive(Debug, Clone, Copy)]
pub struct ChaseSpeedInputs {
    /// +0x70 / +0x84 / +0x78.
    pub state: u32,
    pub profile: u32,
    pub pattern: u32,
    /// FUN_00195a30(actor); the actor's vtbl+0x144 stop query; +0x170.
    pub alive: f32,
    pub stop_query: bool,
    pub stop_byte: bool,
    /// The own sim: +0x10 position, +0x20 velocity, the +0x5c → +0x20 facing row, +0x5c → +0x4c8 box.
    pub self_pos: [f32; 3],
    pub self_vel: [f32; 3],
    pub self_forward: [f32; 3],
    pub self_box: f32,
    /// +0x1a4 ≠ 0, and whether that target's actor is the player (vtbl+0x9c == 1).
    pub has_target: bool,
    pub target_is_player: bool,
    /// The target sim's +0x20 velocity and box.
    pub target_vel: [f32; 3],
    pub target_box: f32,
    /// +0x1c0 aim point, +0x1d0 distance to it, +0x1e0 unit direction to it, +0x210 the
    /// second direction 0x131770 stores (read by profile 0xf) [M].
    pub aim: [f32; 3],
    pub dist: f32,
    pub dir: [f32; 3],
    pub dir_210: [f32; 3],
    /// +0x220 follow distance, +0x13c catch-up, +0x138 cap, +0x134 multiplier.
    pub follow: f32,
    pub catchup: f32,
    pub cap: f32,
    pub speed_mul: f32,
    /// +0x148 ≠ 0 (ramming), +0x208 (1 = target lost), +0x204 ≠ 0 (an element target),
    /// +0x158 ≠ 0, +0x164 (spawn delay).
    pub ramming: bool,
    pub lost: bool,
    pub element_target: bool,
    pub follow_target: bool,
    pub spawn_delay: i16,
    /// The current segment's rs+0x4c and the mission speed factor `FUN_00208468` (MSet f32
    /// +0x2b8/+0x2bc/+0x2c0 by difficulty 0/1/2).
    pub seg_limit: f32,
    pub speed_factor: f32,
}

/// The result: the AI +0x8c target speed, and whether the call cleared +0x148 (the ram).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChaseSpeed {
    pub speed: f32,
    pub ram_cleared: bool,
}

/// `FUN_0012e808` 0x12e808 for every profile [M; the target/own actor of each `FUN_0013dfc8`
/// call read from the disassembly's `$a0` (0x12e90c own, 0x12ea14 / 0x12ea38 / 0x12eab0 /
/// 0x12ed74 / 0x12eff8 target, 0x12eb00 / 0x12ee10 own)]:
///
/// - gates as `ai::target_speed` (state 5 needs alive > 0, state 4 alive ≥ 0.5, stop query,
///   +0x170) → 0;
/// - **1 wait**: `|aim − own pos| ≤ 2.5` → 0; else `max(follow, 10) < that` → cap; else
///   `dist · 0.5`;
/// - **0xc**: 7.5;
/// - **0xd pursue**: start 30. Target seen (`lost` clear) with a target: follow = +0x220, or
///   for a player target `max(follow, 1.25·(box_t + box_s))`, and a ram inside that box sum
///   ends. Pattern 4 with `dot(n(own vel), dir) < 0.707` → `|v_t|·0.6`, ram off. Else inside
///   the follow distance: ramming → `max(|v_t|·catchup, 30)`, else `dot(n(v_t), dir) > 0` →
///   `|v_t|·0.8`, else 0; outside: `max(|v_t| · min(catchup, dist/max(follow, 1)·0.75), 30)`,
///   × catchup again unless `dist ≤ 2·follow` and not ramming. Target lost: `max(|v_t|·catchup,
///   25)` (25 without a target). Then `min(cap', ·)`, cap' = cap × speed factor when +0x204 ≠ 0;
/// - **0xe escort**: 10; target seen: `dist < follow` or `dot(own facing, dir) ≥ 0` →
///   `|v_t|·catchup`, else `|v_t|·0.8`; lost → 25. Then `min(cap, max(·, 10))`;
/// - **0xf road**: rs+0x4c; with +0x158 and a target: `follow·factor ≤ dist` and
///   `dot(dir_210, dir) < 0` → (`2·follow·factor ≤ dist` → ×0.75, else `max(·0.85,
///   |v_t|·0.85)`); otherwise `max(·, |v_t|·1.1)`; then `min(cap·factor, ·)`, and 0 while the
///   spawn delay runs. Result × +0x134;
/// - any other profile → 0.
pub fn chase_target_speed(i: &ChaseSpeedInputs) -> ChaseSpeed {
    let zero = ChaseSpeed { speed: 0.0, ram_cleared: false };
    if i.state == 5 && i.alive <= 0.0 {
        return zero;
    }
    if i.state == 4 && i.alive < 0.5 {
        return zero;
    }
    if i.stop_query || i.stop_byte {
        return zero;
    }
    let ts = len(i.target_vel);
    match i.profile {
        PROFILE_WAIT => {
            let d = len(sub(i.aim, i.self_pos));
            if d <= 2.5 {
                return zero;
            }
            let f = i.follow.max(10.0);
            let s = if f < d { i.cap } else { i.dist * 0.5 };
            ChaseSpeed { speed: s, ram_cleared: false }
        }
        0xc => ChaseSpeed { speed: 7.5, ram_cleared: false },
        PROFILE_PURSUE => {
            let mut t = 30.0f32;
            let mut ram = i.ramming;
            let mut cleared = false;
            if !i.lost {
                if i.has_target {
                    let mut follow = i.follow;
                    if i.target_is_player {
                        let boxes = i.target_box * 1.25 + i.self_box * 1.25;
                        follow = follow.max(boxes);
                        if ram && i.dist <= boxes {
                            ram = false;
                            cleared = true;
                        }
                    }
                    let mut aligned = true;
                    if i.pattern == 4 && dot(normalize(i.self_vel), i.dir) < 0.707 {
                        aligned = false;
                        if ram {
                            cleared = true;
                        }
                        t = ts * 0.6;
                    }
                    if aligned {
                        let d = i.dist;
                        if d < follow {
                            let c = dot(normalize(i.target_vel), i.dir);
                            if !ram {
                                t = if c > 0.0 { ts * 0.8 } else { 0.0 };
                            } else {
                                t = (ts * i.catchup).max(30.0);
                            }
                        } else {
                            let k = i.catchup.min(d / follow.max(1.0) * 0.75);
                            t = (ts * k).max(30.0);
                            if !(d <= follow + follow && !ram) {
                                t *= i.catchup;
                            }
                        }
                    }
                }
            } else {
                t = if i.has_target { (ts * i.catchup).max(25.0) } else { 25.0 };
            }
            let cap = if i.element_target { i.cap * i.speed_factor } else { i.cap };
            ChaseSpeed { speed: cap.min(t), ram_cleared: cleared }
        }
        PROFILE_ESCORT => {
            let t = if !i.lost {
                let c = dot(i.self_forward, i.dir);
                if i.dist < i.follow || c >= 0.0 { ts * i.catchup } else { ts * 0.8 }
            } else {
                25.0
            };
            ChaseSpeed { speed: i.cap.min(t.max(10.0)), ram_cleared: false }
        }
        PROFILE_ROAD => {
            let mut s = i.seg_limit;
            if i.follow_target && i.has_target {
                let f9 = i.follow * i.speed_factor;
                let far = f9 <= i.dist;
                if far && dot(i.dir_210, i.dir) < 0.0 {
                    if f9 + f9 <= i.dist {
                        s *= 0.75;
                    } else {
                        s = (s * 0.85).max(ts * 0.85);
                    }
                } else {
                    s = s.max(ts * 1.1);
                }
                s = (i.cap * i.speed_factor).min(s);
                if i.spawn_delay != 0 {
                    s = 0.0;
                }
            }
            ChaseSpeed { speed: s * i.speed_mul, ram_cleared: false }
        }
        _ => zero,
    }
}

// ------------------------------------------------------------------------------------------
// Pursuit timers: the ram roll and the sight rule (FUN_00131770)
// ------------------------------------------------------------------------------------------

/// AI +0x144 / +0x148 / +0x164 [M 0x12ded8 init: +0x144 = fps·5, +0x148 = 0, +0x164 = 0].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RamState {
    pub roll: i32,
    pub ram: i32,
    pub spawn_delay: i16,
}

impl RamState {
    pub fn new(fps: i32) -> RamState {
        RamState { roll: fps * RAM_ROLL_S, ram: 0, spawn_delay: 0 }
    }
}

/// The timer block of `FUN_00131770` 0x131770 (disassembly 0x131900-0x1319ec): the spawn
/// delay, ram and roll count down to 0; then, unless +0x118 holds, a roll that reached 0 while
/// the target is seen re-arms to 5 s and draws once: `rand·(1/65536) < chance` → ram for 3 s,
/// or 1 s when the target is the player. Returns whether a ram started [M].
pub fn ram_tick(st: &mut RamState, hold_118: bool, lost: bool, has_target: bool, target_is_player: bool, chance: f32, fps: i32, rng: &mut DriveRng) -> bool {
    st.spawn_delay = (st.spawn_delay as i32 - 1).max(0) as i16;
    st.ram = (st.ram - 1).max(0);
    st.roll = (st.roll - 1).max(0);
    if hold_118 || st.roll != 0 || lost {
        return false;
    }
    st.roll = fps * RAM_ROLL_S;
    let r = rng.next() as i32 as f32 * RAND_SCALE;
    if r < chance {
        st.ram = fps * RAM_S;
        if has_target && target_is_player {
            st.ram = fps * RAM_S_VS_PLAYER;
        }
        true
    } else {
        false
    }
}

/// AI +0x174 (last sight test), +0x1cc (seen), +0x20c (ever seen).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SightState {
    pub last: i32,
    pub seen: bool,
    pub ever_seen: bool,
}

/// `FUN_001330d0` 0x1330d0: seen = within 200 and the world ray (FUN_00215cc8) reports fewer
/// than 2 hits [M].
pub fn can_see(dist: f32, ray_hits: u32) -> bool {
    dist <= SIGHT_RANGE && ray_hits < 2
}

/// The sight block of `FUN_00131770`: with no navigator repeat (+0x94 = 0), every
/// `(short)(int)(fps·0.25)` ticks: a target → seen = `can_see`; no target and target mode 1
/// (drive to a point) → seen. Returns +0x208 = `!seen` (1 = lost) [M].
pub fn sight_tick(st: &mut SightState, now: i32, fps: i32, nav_repeat: bool, has_target: bool, target_mode: u32, dist: f32, ray_hits: u32) -> bool {
    if !nav_repeat {
        let period = (fps as f32 * SIGHT_PERIOD_S) as i32 as i16 as i32;
        if now >= st.last + period {
            st.last = now;
            if !has_target {
                if target_mode == 1 {
                    st.seen = true;
                }
            } else {
                st.seen = can_see(dist, ray_hits);
                if st.seen {
                    st.ever_seen = true;
                }
            }
        }
    }
    !st.seen
}

// ------------------------------------------------------------------------------------------
// Weapons (FUN_00130050 and the three fire functions)
// ------------------------------------------------------------------------------------------

/// AI +0x226 gun cooldown, +0x22c rocket cooldown / +0x22e side, +0x230 missile cooldown /
/// +0x232 side (all 0 at construction [M 0x12ded8]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WeaponState {
    pub gun_cd: i16,
    pub rocket_cd: i16,
    pub rocket_side: u8,
    pub missile_cd: i16,
    pub missile_side: u8,
}

/// What the fire functions read.
#[derive(Debug, Clone, Copy)]
pub struct FireInputs {
    pub alive: f32,
    /// +0x170, +0x224 (weapons on; 1 at construction), +0x1a4 ≠ 0, +0x208, +0x1cc.
    pub stop_byte: bool,
    pub weapons_on: bool,
    pub has_target: bool,
    pub lost: bool,
    pub seen: bool,
    /// +0x84.
    pub profile: u32,
    /// +0x12c turret (`HAS_TURRET`), +0x238 == 1 (turret on target).
    pub turret: bool,
    pub turret_on_target: bool,
    /// +0x12d / +0x12e / +0x12f — the AICo weapon bytes the vehicle's `.atr` allows.
    pub guns: bool,
    pub rockets: bool,
    pub missiles: bool,
    /// `dot(+0x1e0, shot direction)` and `FUN_0012f530`'s verdict for that shot.
    pub aim_dot: f32,
    pub line_clear: bool,
    /// +0x228 seconds between shots (< 0 never fires), DAT_00339f4c the tick rate.
    pub rate: f32,
    pub fps: i32,
    /// The launcher animation channels still playing (`FUN_001a2d10`): rockets [ch 6, ch 5],
    /// missiles [ch 8, ch 7].
    pub rocket_busy: [bool; 2],
    pub missile_busy: [bool; 2],
}

/// One tick's shots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shots {
    /// A machine-gun burst started (channels 9/0x48 frame 10).
    pub gun: bool,
    /// The gun-flash words (actor +0x4c → +0xb0/+0xb4): `Some(on)` when the gun arm ran.
    pub gun_flash: Option<bool>,
    /// The rocket launcher that fired (channel 6 or 5).
    pub rocket: Option<u8>,
    /// The missile launcher that fired (channel 8 or 7).
    pub missile: Option<u8>,
}

/// `(short)(int)(0.5 + f)` with the sign-matched half [M the three fire functions].
fn round_ticks(f: f32) -> i16 {
    let h = if f < 0.0 { -0.5 } else { 0.5 };
    (h + f) as i32 as i16
}

/// A cooldown step: `cd < 1` → fire (true), else `cd = max(cd − 1, 0)` [M].
fn cooldown(cd: &mut i16) -> bool {
    if *cd < 1 {
        true
    } else {
        *cd = (*cd as i32 - 1).max(0) as i16;
        false
    }
}

/// `FUN_0012f730` 0x12f730 machine guns: needs a target, seen and not lost; aimed = turret on
/// target, or `line_clear` and `aim_dot > 0.99`. Aimed: with rate ≥ 0 the cooldown steps and
/// a burst resets it to `round(fps·rate)`; the flash words are 1 while aimed, 0 otherwise [M].
fn fire_guns(st: &mut WeaponState, i: &FireInputs, out: &mut Shots) {
    if !(i.has_target && !i.lost && i.seen) {
        return;
    }
    let aimed = if i.turret { i.turret_on_target } else { i.line_clear && i.aim_dot > GUN_CONE };
    if aimed {
        if 0.0 <= i.rate && cooldown(&mut st.gun_cd) {
            out.gun = true;
            st.gun_cd = round_ticks(i.fps as f32 * i.rate);
        }
        out.gun_flash = Some(true);
    } else {
        out.gun_flash = Some(false);
    }
}

/// `FUN_0012fa28` 0x12fa28 rockets: cone 0.9 (or turret on target); the side toggles between
/// channel 6 (side 0) and 5; a launcher still animating holds the shot (no cooldown step);
/// reload `round(max(fps·rate, fps·0.5))` [M].
fn fire_rockets(st: &mut WeaponState, i: &FireInputs, out: &mut Shots) {
    if !(i.has_target && !i.lost && i.seen) {
        return;
    }
    let aimed = if i.turret { i.turret_on_target } else { i.line_clear && i.aim_dot > ROCKET_CONE };
    if !aimed || i.rate < 0.0 {
        return;
    }
    if st.rocket_cd < 1 {
        let (ch, busy) = if st.rocket_side == 0 { (6u8, i.rocket_busy[0]) } else { (5u8, i.rocket_busy[1]) };
        if busy {
            return;
        }
        out.rocket = Some(ch);
        let f = (i.fps as f32 * i.rate).max(i.fps as f32 * LAUNCHER_MIN_S);
        st.rocket_cd = round_ticks(f);
        st.rocket_side = (st.rocket_side + 1) & 1;
    } else {
        cooldown(&mut st.rocket_cd);
    }
}

/// `FUN_0012fd58` 0x12fd58 missiles/torpedoes: no turret arm; `line_clear` and `aim_dot >
/// 0.707`; channel 8 (side 0) / 7; same reload rule as the rockets [M].
fn fire_missiles(st: &mut WeaponState, i: &FireInputs, out: &mut Shots) {
    if !(i.has_target && !i.lost && i.seen) {
        return;
    }
    if !(i.line_clear && i.aim_dot > MISSILE_CONE) || i.rate < 0.0 {
        return;
    }
    if st.missile_cd < 1 {
        let (ch, busy) = if st.missile_side == 0 { (8u8, i.missile_busy[0]) } else { (7u8, i.missile_busy[1]) };
        if busy {
            return;
        }
        out.missile = Some(ch);
        let f = (i.fps as f32 * i.rate).max(i.fps as f32 * LAUNCHER_MIN_S);
        st.missile_cd = round_ticks(f);
        st.missile_side = (st.missile_side + 1) & 1;
    } else {
        cooldown(&mut st.missile_cd);
    }
}

/// `FUN_00130050` 0x130050: alive, +0x170 clear, weapons on and a target → profile 10 rockets
/// only, profile 9 guns only, else each weapon the car carries in the order guns, rockets,
/// missiles. The turret aim (`FUN_0012f170`, run first when +0x12c) is not ported: the caller
/// supplies `turret_on_target` [M].
pub fn fire_step(st: &mut WeaponState, i: &FireInputs) -> Shots {
    let mut out = Shots::default();
    if !(0.0 < i.alive && !i.stop_byte && i.weapons_on && i.has_target) {
        return out;
    }
    match i.profile {
        PROFILE_ROCKETS_ONLY => fire_rockets(st, i, &mut out),
        PROFILE_GUNS_ONLY => fire_guns(st, i, &mut out),
        _ => {
            if i.guns {
                fire_guns(st, i, &mut out);
            }
            if i.rockets {
                fire_rockets(st, i, &mut out);
            }
            if i.missiles {
                fire_missiles(st, i, &mut out);
            }
        }
    }
    out
}

/// One other car for the line-of-fire test.
#[derive(Debug, Clone, Copy)]
pub struct LofCar {
    pub id: i16,
    pub pos: [f32; 3],
    /// sim +0x6d < 4, sim +0x6e ≠ 3 (not traffic), its AI controller in state 5.
    pub sim_6d: i8,
    pub sim_6e: i8,
    pub chase_state: bool,
}

/// `FUN_0012f530` 0x12f530: the shot from `origin` along `shot_dir` is blocked when another
/// live chase car (+0x6d < 4, not traffic, controller state 5) lies inside the 0.99 cone and
/// nearer than `range` (the target distance +0x1d0) [M].
pub fn line_clear(range: f32, shooter: i16, shot_dir: [f32; 3], origin: [f32; 3], cars: &[LofCar]) -> bool {
    for c in cars {
        if c.id == shooter || !(c.sim_6d < 4 && c.sim_6e != 3) || !c.chase_state {
            continue;
        }
        let d = normalize(sub(c.pos, origin));
        if LOF_CONE < dot(d, shot_dir) && len(sub(origin, c.pos)) < range {
            return false;
        }
    }
    true
}
