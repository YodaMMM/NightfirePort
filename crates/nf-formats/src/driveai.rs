//! The driving tracks' AI element set: the `Map ` article's **`AIEl`** (AI elements, the
//! hostile vehicles and on-foot henchmen a mission spawns) and **`AICo`** (AI commands, the
//! per-vehicle behaviour a script hands a spawned car) chunks (docs/systems/driving-ai.md §10).
//!
//! Evidence: [M] DRIVING.ELF decompile (EE = file offset + 0x107080), [D] disc bytes.
//!
//! - `AIEl`: the record's `count` is the element count and the payload is `count × 0x100` bytes,
//!   copied verbatim into the runtime database (`FUN_0012b8b0` 0x12b8b0:
//!   `FUN_002a7e84(db+0xc, chunk+dref, count << 8)`) [M]; measured 25/45/28/37/43/63/33/3
//!   elements over MIS01/MIS11/MIS13A/MIS13B/MIS13C/MIS3/MIS4/RACE, each payload exactly
//!   `count × 0x100` bytes [D].
//! - `AICo`: fixed 0xa0-byte records (MIS01 61 × 0xa0 = 9,760 B [D]); the field map is the one
//!   `FUN_00145ff8` 0x145ff8 copies into the AIVehicle when a script command runs [M].

use crate::buf::{invalid, Result};
use crate::crp::{tag, Crp};

/// `AIEl` — 0x4149456c [M 0x176bb8-0x176be4 `sub_2D5A10(·, 0x4149456C)`].
pub const TAG_AIEL: u32 = 0x4149_456c;
/// `AICo` — the AI command records [D: Map child of every track].
pub const TAG_AICO: u32 = 0x4149_436f;

/// One AIEl element record size [M 0x12b8b0: `iVar12 * 0x100`].
pub const AIEL_RECORD: usize = 0x100;
/// One rider (sub-record) size [M 0x12b8b0: `iVar9 * 0x20 + 0x80`].
pub const AIEL_RIDER: usize = 0x20;
/// One AICo record size [D: every track's AICo payload is a multiple of 0xa0].
pub const AICO_RECORD: usize = 0xa0;

/// `+0x44` element types [M 0x12b8b0 / 0x12cb08 `*(int *)(rec + 0x44)`].
pub const TYPE_HENCHMAN: i32 = 0;
pub const TYPE_VEHICLE: i32 = 1;

/// `+0x40` flag bits [M, per bit: the reader].
pub mod flag {
    /// Copy the live transform back into the record on despawn (it respawns where it left)
    /// [M 0x12c740: `((flags ^ 1) & 1) == 0` skips the copy].
    pub const KEEP_PLACE: u16 = 0x1;
    /// Held off: the range manager never spawns it and despawns it when live (unless
    /// PERSISTENT); scripts clear it to release the element [M 0x12bec8 skip, 0x12afb8
    /// despawn, 0x1476b8 `& 0xfffd`, 0x14b510 `| 2`].
    pub const HELD: u16 = 0x2;
    /// Persistent: spawned at any range, never despawned by range [M 0x12dc18
    /// `(flags >> 2 ^ 1) & 1` is the "may despawn" test].
    pub const PERSISTENT: u16 = 0x4;
    /// Chase brain: the vehicle is created as sim kind 2 (an AIVehicle in state 5) instead of
    /// kind 4 [M 0x12cb08 `uVar5 = 4; if (flags & 8) uVar5 = 2`, 0x13f118 dispatch].
    pub const CHASE_BRAIN: u16 = 0x8;
    /// Runtime: set at the first spawn — the kill counter counts an element once [M 0x12cb08].
    pub const COUNTED: u16 = 0x10;
    /// Runtime: set when a live vehicle has lost a wheel or its controller reports +0x16c /
    /// +0x168 [M 0x12b440 tail].
    pub const DAMAGED: u16 = 0x20;
    /// Not counted toward the mission's kill counter (mgr+0x3dc); the actor's vtbl+0x16c(0) /
    /// heli +0x138 = 0 [M 0x12cb08].
    pub const UNCOUNTED: u16 = 0x40;
}

fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn i32_at(b: &[u8], o: usize) -> i32 {
    i32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u16_at(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(b[o..o + 2].try_into().unwrap())
}
fn cstr(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

/// One rider of a vehicle element — an AICharacter seated in the car (window shooter, sunroof
/// gunner, …) [M 0x12cb08:280-440].
#[derive(Clone, Debug, PartialEq)]
pub struct AiRider {
    /// +0x00 seat: 3 window (`AIC_EWIND_POS`), 4 sunroof (`AIC_ESUNR_POS`), 0xe snowmobile
    /// (`AIC_ESNOW_POS`), 0xf vehicle gunner (`AIC_EVGUN_POS`), 0x10 driver (`AIC_EVDRI_POS`),
    /// 0x11 Bond's seat (`AIC_BOND_POS`); any other value spawns no rider [M switch].
    pub seat: u32,
    /// +0x04 index into the character-model table at 0x327ba8 [M].
    pub model: u32,
    /// +0x08 index into the weapon-name table at 0x327b90 (1 = `ssmg`, 2 = `bazooka` [D the
    /// ELF's .data]); seats 0xf/0x10/0x11 pass no weapon [M].
    pub weapon: u32,
    /// +0x0c / +0x10 / +0x18: not read by the spawn path (kept raw).
    pub raw_0c: i32,
    pub raw_10: i32,
    pub raw_18: i32,
    /// +0x14 health, × the difficulty health factor at load (`FUN_002084a8`); a rider with
    /// health ≤ 0 is not spawned [M 0x12b8b0, 0x12cb08 `0.0 < puVar19[5]`].
    pub health: f32,
    /// +0x1c: 2/4 → the character's +0x64 × 4.0, 3/5 → × 9.0 [M 0x12cb08:400-406].
    pub skill: i32,
}

impl AiRider {
    /// The seat attribute name the rider is placed at, read from the vehicle's `.atr`
    /// [M 0x12cb08: `FUN_00169e48(vehicle + 0x5c, <name>)`; strings 0x36b178..0x36b1c8 [D]].
    pub fn seat_attr(&self) -> Option<&'static str> {
        Some(match self.seat {
            3 => "AIC_EWIND_POS",
            4 => "AIC_ESUNR_POS",
            0xe => "AIC_ESNOW_POS",
            0xf => "AIC_EVGUN_POS",
            0x10 => "AIC_EVDRI_POS",
            0x11 => "AIC_BOND_POS",
            _ => return None,
        })
    }

    /// The weapon name from the ELF's 0x327b90 table [D]: index 1 `ssmg`, 2 `bazooka`; the
    /// other slots read 0 in the file (filled at run time or unused — not traced).
    pub fn weapon_name(&self) -> Option<&'static str> {
        match self.weapon {
            1 => Some("ssmg"),
            2 => Some("bazooka"),
            _ => None,
        }
    }
}

/// One AIEl element (0x100 bytes) [M 0x12b8b0 / 0x12cb08 / 0x12b440 / 0x12c740].
#[derive(Clone, Debug, PartialEq)]
pub struct AiElement {
    /// +0x00..+0x30: the placement rows; +0x20 is the facing row passed to the spawn as the
    /// direction [M 0x12cb08: uStack_150 = +0x20..+0x28].
    pub facing: [f32; 3],
    /// +0x30: the spawn position [M 0x12cb08 uStack_140 = +0x30..+0x38; 0x12b440 reads it as
    /// the element's position while it is not live].
    pub pos: [f32; 3],
    /// +0x40 flags (see [`flag`]).
    pub flags: u16,
    /// +0x42 difficulty filter (0 = any; 1..6 see `difficulty_allows`) [M 0x12cb08 switch].
    pub difficulty: u16,
    /// +0x44 type: 0 henchman, 1 vehicle.
    pub kind: i32,
    /// +0x48 runtime state in the file (0 in every shipped record [D]).
    pub state: i32,
    /// +0x4c health (× the difficulty health factor at load) — the live actor's health points
    /// at this field [M 0x12cb08 `FUN_00195a08(actor, rec + 0x4c)`].
    pub health: f32,
    /// +0x54 the pvehicle attribute name (`data\sim\attrib\pvehicle\<name>.atr`), char[0x20]
    /// [M 0x12b8b0 `FUN_00196740(·, rec + 0x54)` → rendername; 0x1862c0 builds the car from it].
    pub name: String,
    /// +0x58..+0x70 henchman fields (type 0): +0x58 class (only 0, 10 and 0x12 spawn), +0x5c
    /// model index, +0x60 weapon index [M 0x12b8b0:113-115, 0x12cb08:445-499].
    pub hench_class: i32,
    pub hench_model: i32,
    pub hench_weapon: i32,
    /// +0x74 byte passed to the vehicle ctor (0x1862c0 param_4 → +0x2d0) [M].
    pub ctor_byte: u8,
    /// +0x77 ≠ 0: a helicopter (FUN_002003c8 path, the heli slot pool) [M 0x12cb08, 0x12bec8].
    pub heli: bool,
    /// +0x7c rider count, riders at +0x80 (stride 0x20).
    pub riders: Vec<AiRider>,
    /// +0xf8 the script/path reference handed to FUN_00145fc8 at spawn (an `sr` short tag in
    /// the file, 0x7372_xxxx [D]) [M 0x12cb08:226-229].
    pub command_ref: u32,
}

impl AiElement {
    pub fn parse(b: &[u8]) -> Result<AiElement> {
        if b.len() < AIEL_RECORD {
            return invalid(format!("AIEl record {} B < 0x100", b.len()));
        }
        let kind = i32_at(b, 0x44);
        let n = i32_at(b, 0x7c);
        let mut riders = Vec::new();
        // The rider loop runs only for vehicles [M 0x12b8b0: `== 1` before the 0x7c loop]; the
        // sub-records live inside the 0x100 bytes, so at most 4 fit (0x80 + 4 × 0x20).
        if kind == TYPE_VEHICLE && n > 0 {
            if n > 4 {
                return invalid(format!("AIEl vehicle rider count {n} > 4"));
            }
            for k in 0..n as usize {
                let s = &b[0x80 + k * AIEL_RIDER..0x80 + (k + 1) * AIEL_RIDER];
                riders.push(AiRider {
                    seat: u32_at(s, 0),
                    model: u32_at(s, 4),
                    weapon: u32_at(s, 8),
                    raw_0c: i32_at(s, 0xc),
                    raw_10: i32_at(s, 0x10),
                    health: f32_at(s, 0x14),
                    raw_18: i32_at(s, 0x18),
                    skill: i32_at(s, 0x1c),
                });
            }
        }
        let v3 = |o| [f32_at(b, o), f32_at(b, o + 4), f32_at(b, o + 8)];
        Ok(AiElement {
            facing: v3(0x20),
            pos: v3(0x30),
            flags: u16_at(b, 0x40),
            difficulty: u16_at(b, 0x42),
            kind,
            state: i32_at(b, 0x48),
            health: f32_at(b, 0x4c),
            name: if kind == TYPE_VEHICLE { cstr(&b[0x54..0x74]) } else { String::new() },
            hench_class: if kind == TYPE_HENCHMAN { i32_at(b, 0x58) } else { 0 },
            hench_model: if kind == TYPE_HENCHMAN { i32_at(b, 0x5c) } else { 0 },
            hench_weapon: if kind == TYPE_HENCHMAN { i32_at(b, 0x60) } else { 0 },
            ctor_byte: b[0x74],
            heli: b[0x77] != 0,
            riders,
            command_ref: u32_at(b, 0xf8),
        })
    }
}

/// One AICo record (0xa0 bytes) — the fields `FUN_00145ff8` 0x145ff8 copies into the AIVehicle
/// (AI offsets in brackets) [M 0x145ff8:153-240].
#[derive(Clone, Debug, PartialEq)]
pub struct AiCommand {
    /// +0x40 the goal point (`FUN_0012e670` mode 1 → AI +0x1a8).
    pub goal: [f32; 3],
    /// +0x4c → AI +0x220, the follow distance.
    pub follow: f32,
    /// +0x50 → `FUN_0012e770`: the target element reference (0 = none) [D `sr` short tags].
    pub target_ref: u32,
    /// +0x5c → AI +0x13c, the catch-up multiplier.
    pub catchup: f32,
    /// +0x60 → AI +0x138, the speed cap.
    pub speed_cap: f32,
    /// +0x64 → AI +0x228, seconds between shots (< 0 never fires) [M 0x12f730/0x12fa28/0x12fd58].
    pub fire_rate: f32,
    /// +0x68 → AI +0x14c, the ram chance per 5-second roll [M 0x131770].
    pub ram_chance: f32,
    /// +0x6c → AI +0x150, the lead (read by 0x131770's pattern-3 aim; not ported).
    pub lead: f32,
    /// +0x70 / +0x74: an initial velocity set-up (0 and 1.5 skip the +0x70 arm) [M 0x145ff8:178-206].
    pub launch_a: f32,
    pub launch_b: f32,
    /// +0x78 target mode → `FUN_0012e670`: 0 = chase the player (manager handle 0), 1 = drive
    /// to `goal`, else keep [M 0x12e670].
    pub target_mode: u8,
    /// +0x79 → `FUN_0012e468` pattern (AI +0x78).
    pub pattern: u8,
    /// +0x7a → `FUN_0012e6f8` profile (AI +0x84).
    pub profile: u8,
    /// +0x7b ≠ 0: park traffic cars overlapping the vehicle when the command runs [M 0x145ff8].
    pub clear_traffic: u8,
    /// +0x7c == 0 → AI +0x154 = 1 (avoidance on) [M 0x145ff8:153].
    pub no_avoid: u8,
    /// +0x7d ≠ 0 → AI +0x158 [M].
    pub follow_target: u8,
    /// +0x7e lane (< 0 keeps) → nav +0xb0 [M].
    pub lane: i8,
    /// +0x7f → nav +0x04 [M].
    pub nav_byte: u8,
    /// +0x80 pattern argument (an `sr` reference on pattern-5 commands [D]).
    pub pattern_arg: u32,
    /// +0x87 / +0x92 machine guns, +0x90 rockets, +0x91 missiles/torpedoes — each honoured only
    /// when the vehicle's `.atr` declares the weapon (`HAS_MACHINEGUNS`/`HAS_CANNONS`,
    /// `HAS_ROCKETS`, `HAS_MISSLES`/`HAS_TORPEDOS`) [M 0x145ff8:216-238, strings 0x36e390.. [D]].
    pub guns: (u8, u8),
    pub rockets: u8,
    pub missiles: u8,
}

impl AiCommand {
    pub fn parse(b: &[u8]) -> Result<AiCommand> {
        if b.len() < AICO_RECORD {
            return invalid(format!("AICo record {} B < 0xa0", b.len()));
        }
        Ok(AiCommand {
            goal: [f32_at(b, 0x40), f32_at(b, 0x44), f32_at(b, 0x48)],
            follow: f32_at(b, 0x4c),
            target_ref: u32_at(b, 0x50),
            catchup: f32_at(b, 0x5c),
            speed_cap: f32_at(b, 0x60),
            fire_rate: f32_at(b, 0x64),
            ram_chance: f32_at(b, 0x68),
            lead: f32_at(b, 0x6c),
            launch_a: f32_at(b, 0x70),
            launch_b: f32_at(b, 0x74),
            target_mode: b[0x78],
            pattern: b[0x79],
            profile: b[0x7a],
            clear_traffic: b[0x7b],
            no_avoid: b[0x7c],
            follow_target: b[0x7d],
            lane: b[0x7e] as i8,
            nav_byte: b[0x7f],
            pattern_arg: u32_at(b, 0x80),
            guns: (b[0x87], b[0x92]),
            rockets: b[0x90],
            missiles: b[0x91],
        })
    }
}

/// A track's AI set: the parsed `AIEl` elements and `AICo` commands.
#[derive(Clone, Debug, Default)]
pub struct MapAi {
    pub elements: Vec<AiElement>,
    pub commands: Vec<AiCommand>,
}

impl MapAi {
    /// Locates the pool `Map ` record and reads its `AIEl` / `AICo` children [M 0x176bb8..
    /// 0x176be4: `sub_2D51F0(crp, 'Map ')` then `sub_2D5A10(·, 'AIEl')`].
    pub fn from_crp(crp: &Crp) -> Result<MapAi> {
        let Some(map) = crp.pool_find(tag::MAP)? else {
            return Ok(MapAi::default());
        };
        let mut out = MapAi::default();
        for r in crp.record_window(&map)? {
            if r.tag == TAG_AIEL {
                // The game copies `count << 8` bytes from the payload address [M 0x12b8b0].
                let at = r.data_addr(crp.data)?;
                let n = r.count.max(0) as usize;
                let end = at + n * AIEL_RECORD;
                if end > crp.data.len() {
                    return invalid(format!("AIEl {n} records past the end"));
                }
                for k in 0..n {
                    out.elements.push(AiElement::parse(&crp.data[at + k * AIEL_RECORD..at + (k + 1) * AIEL_RECORD])?);
                }
            } else if r.tag == TAG_AICO {
                let p = crp.payload(&r)?;
                if p.len() % AICO_RECORD != 0 {
                    return invalid(format!("AICo payload {} B is not a multiple of 0xa0", p.len()));
                }
                for c in p.chunks_exact(AICO_RECORD) {
                    out.commands.push(AiCommand::parse(c)?);
                }
            }
        }
        Ok(out)
    }

    /// The distinct vehicle attribute names (the mission's AI vehicle set), in first-seen order.
    pub fn vehicle_set(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for e in &self.elements {
            if e.kind == TYPE_VEHICLE && !v.contains(&e.name) {
                v.push(e.name.clone());
            }
        }
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(kind: i32, name: &str, flags: u16, riders: &[(u32, u32, u32, f32, i32)]) -> Vec<u8> {
        let mut b = vec![0u8; AIEL_RECORD];
        b[0x30..0x34].copy_from_slice(&10.0f32.to_le_bytes());
        b[0x38..0x3c].copy_from_slice(&(-20.0f32).to_le_bytes());
        b[0x40..0x42].copy_from_slice(&flags.to_le_bytes());
        b[0x44..0x48].copy_from_slice(&kind.to_le_bytes());
        b[0x4c..0x50].copy_from_slice(&99.0f32.to_le_bytes());
        b[0x54..0x54 + name.len()].copy_from_slice(name.as_bytes());
        b[0x7c..0x80].copy_from_slice(&(riders.len() as i32).to_le_bytes());
        for (k, r) in riders.iter().enumerate() {
            let o = 0x80 + k * 0x20;
            b[o..o + 4].copy_from_slice(&r.0.to_le_bytes());
            b[o + 4..o + 8].copy_from_slice(&r.1.to_le_bytes());
            b[o + 8..o + 12].copy_from_slice(&r.2.to_le_bytes());
            b[o + 0x14..o + 0x18].copy_from_slice(&r.3.to_le_bytes());
            b[o + 0x1c..o + 0x20].copy_from_slice(&r.4.to_le_bytes());
        }
        b
    }

    #[test]
    fn element_fields_at_their_offsets() {
        let b = element(1, "paris_henchbigbox", 0xe, &[(4, 0x14, 2, 400.0, 3)]);
        let e = AiElement::parse(&b).unwrap();
        assert_eq!(e.pos, [10.0, 0.0, -20.0]);
        assert_eq!((e.flags, e.kind, e.health), (0xe, 1, 99.0));
        assert_eq!(e.name, "paris_henchbigbox");
        assert_eq!(e.riders.len(), 1);
        let r = &e.riders[0];
        assert_eq!((r.seat, r.model, r.weapon, r.health, r.skill), (4, 0x14, 2, 400.0, 3));
        assert_eq!(r.seat_attr(), Some("AIC_ESUNR_POS"));
        assert_eq!(r.weapon_name(), Some("bazooka"));
        // A henchman's +0x54.. is not a name and has no riders.
        let h = AiElement::parse(&element(0, "", 0xa, &[])).unwrap();
        assert!(h.riders.is_empty() && h.name.is_empty());
    }

    #[test]
    fn command_fields_at_their_offsets() {
        let mut b = vec![0u8; AICO_RECORD];
        b[0x4c..0x50].copy_from_slice(&15.0f32.to_le_bytes());
        b[0x5c..0x60].copy_from_slice(&1.1f32.to_le_bytes());
        b[0x60..0x64].copy_from_slice(&60.0f32.to_le_bytes());
        b[0x64..0x68].copy_from_slice(&2.0f32.to_le_bytes());
        b[0x68..0x6c].copy_from_slice(&0.1f32.to_le_bytes());
        b[0x79] = 4;
        b[0x7a] = 0xf;
        b[0x7e] = 0xff;
        let c = AiCommand::parse(&b).unwrap();
        assert_eq!((c.follow, c.catchup, c.speed_cap, c.fire_rate, c.ram_chance), (15.0, 1.1, 60.0, 2.0, 0.1));
        assert_eq!((c.pattern, c.profile, c.lane), (4, 0xf, -1));
    }
}
