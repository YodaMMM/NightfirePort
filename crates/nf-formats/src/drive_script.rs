//! The driving missions' scripted events — the `el` event lists and `sr` strings of a track
//! `.crp`'s `Map ` record, and the two chunks that fire them (`Trgr` triggers, `Rule` rules).
//! Write-up: docs/systems/driving-mis01-opening.md (the MIS01 sniper phase is the worked case).
//!
//! Evidence (DRIVING.ELF decompile, EE addresses; [M] code, [D] disc bytes, [S] suggested):
//! - **Event-list shape** [M]: a list is `{count u32, 12 pad bytes}` then `count` entries of
//!   0x10 bytes `{kind u32, size u32, rel u32, pad}`; the parameter block sits at
//!   `entry + rel`. Three walkers read it the same way: the trigger fire `FUN_0022c890`
//!   0x22c890 (`puVar4 = list + 0x10; … FUN_001757b8(*puVar4)(puVar4 + puVar4[2]); puVar4 += 4`),
//!   the rule fire `FUN_002095f8` 0x2095f8 (`piVar5 = piVar1 + 4; … (piVar5 + piVar1[6])`) and
//!   `FUN_0014c568` 0x14c568. `FUN_0022ca18` 0x22ca18 tests "the list holds kind k" over the
//!   same stride.
//! - **Kind → handler**: `FUN_001757b8` 0x1757b8 returns `*(0x36fda0 + kind*4)`, a table of
//!   factories; each factory allocates the event and copies params from the block — e.g.
//!   `FUN_001577d8` (kind 77) reads block+4 and block+8 into `ESetAutoDriveCamOn` [M]. Block
//!   +0 repeats the kind [D: every MIS01 block].
//! - **Event names** [M]: each factory's ctor installs a vtable whose slot 0 is a typeinfo
//!   accessor calling `FUN_0024e7a8(…, name, …)` with the GCC type name ("18ESetAutoDriveCamOn"
//!   at 0x372540 for kind 77 via `FUN_00163408`); [`event_name`] lists the kinds resolved that
//!   way.
//! - **References** [D]: a reference word is `index u16` + a 2-letter instance code in the high
//!   half — `0x656c` "el" (an event list), `0x7372` "sr" (a string). Both name instance records
//!   (`flags & 1`, the tag's low half = the index) in the `Map ` record's own window; every
//!   MIS01 `Trgr`/`Rule`/event reference resolves to an existing record. The game's loader
//!   that turns them into the runtime pointers the walkers use is not traced [S].
//! - **Trigger record** (0x40 bytes): +0x00 position xyz, +0x0c radius [D: the start trigger's
//!   2.5 / 5.0], +0x12 u16 flags (bit 0x200 clears 0x400 at load, bit 2 clears bit 1 after a
//!   fire) [M 0x22cbe0, 0x22c890], +0x18 the event list [M 0x22ca18 / 0x22c890 read +0x18].
//!   The world init `FUN_0022cd30` fires every trigger whose list holds kind 0x16
//!   (`EStartMission`) once [M 0x22cd30: `FUN_0022ca18(trig, 0x16)` → `FUN_0022c890`].
//! - **Rule entry** (0x20 bytes): +0x0c is the event list the rule fires [D: all 114 MIS01
//!   entries hold an `el` reference there] (drive-mission-scripts.md read it as a name hash);
//!   +0x08 the rule group, which `EChangeStage`/`ECallStage` select [M 0x206310 / 0x206340].

use std::collections::BTreeMap;

use crate::buf::{invalid, Result};
use crate::crp::Crp;

/// The `Map ` pool record tag [M 0x22faf8].
pub const MAP: u32 = 0x4d61_7020;
/// The 2-letter instance code of an event list ("el").
pub const CODE_EL: u16 = 0x656c;
/// The 2-letter instance code of a string ("sr").
pub const CODE_SR: u16 = 0x7372;
/// The 2-letter instance code of a path track ("pt"): the keyed flight paths `AISp` records
/// point at (`FUN_00230ab0` walks the `Map ` window for `pt  ` records by name) [M].
pub const CODE_PT: u16 = 0x7074;

/// One 32-bit reference word (see the module doc).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ref {
    EventList(u16),
    Str(u16),
    /// Anything else (a plain number, or a code this decoder does not name).
    Other(u32),
}

impl Ref {
    pub fn from_word(w: u32) -> Ref {
        match (w >> 16) as u16 {
            CODE_EL => Ref::EventList(w as u16),
            CODE_SR => Ref::Str(w as u16),
            _ => Ref::Other(w),
        }
    }
}

/// The event kinds named through their typeinfo [M, see the module doc]: (kind, class name,
/// the handler run when the event executes).
pub const EVENT_NAMES: &[(u32, &str, u32)] = &[
    (9, "EPlayEffect", 0x153f80),
    (20, "EStreamEvent", 0x15d000),
    (22, "EStartMission", 0x15c738),
    (28, "ESpawnTraffic", 0x15c080),
    (35, "EWakeupSmackable", 0x15de48),
    (36, "EPlaySystemAnim", 0x154ad8),
    (40, "ECallStage", 0x148d50),
    // 41: the ctor `FUN_00149828` does the work (`mgr+0x10 = n`, `FUN_00206338`); the vtable
    // 0x371350 types it "12EChangeStage" [M].
    (41, "EChangeStage", 0x149828),
    (44, "EDisplayText", 0x14b8f8),
    (45, "EPlayerLose", 0x154cb0),
    (46, "EPlayerWin", 0x1551b8),
    (47, "ESetTimer", 0x1591f8),
    (53, "EDisableTrigger", 0x14b790),
    (54, "EEnableTrigger", 0x14bcd8),
    (63, "EPlayCameraAnim", 0x153700),
    (65, "EAICommand", 0x145ff8),
    (66, "EActivateAIElement", 0x1476b8),
    (67, "EDeactivateAIElement", 0x14b510),
    (68, "EEndCameraAnim", 0x14bdb8),
    (76, "ESetAutoDriveCamOff", 0x157460),
    (77, "ESetAutoDriveCamOn", 0x1576e8),
    (78, "ESetVideo", 0x1592d8),
    (97, "EPlayCarCameraAnim", 0x153c48),
    (103, "ELetterBoxOff", 0x150ce8),
    (104, "ELetterBoxOn", 0x150dc8),
    (107, "EPlayPOVDeath", 0x154570),
    (108, "EAutoDriveSpeedChange", 0x1482f8),
    (112, "ESetSimRate", 0x158d28),
    (116, "EPathSetDesiredThrottleStatic", 0x152cb0),
    (140, "EResetPlayerCarPos", 0x156460),
    (145, "EAIElementFireOn", 0x146b48),
    (149, "EDisablePowerUp", 0x14b630),
    (153, "EPathSetTimeStatic", 0x153340),
    (154, "ESpawnExplosionStatic", 0x15b010),
    (161, "EKillAIElement", 0x150748),
    (162, "EDamagePlayer", 0x14b3e0),
    (193, "EButtonMsgOff", 0x148ad8),
    (196, "ENuclearBlast", 0x151408),
    (202, "EControlToSpline", 0x14ae60),
    (203, "EFireRandomTrigger", 0x14c688),
    (204, "EPlayCameraSpline", 0x1539a8),
    (215, "EPathChangeThrottleStatic", 0x151dc0),
    (225, "EObjectiveAdd", 0x151538),
    (229, "EObjectiveSetCurrent", 0x1519c0),
    (232, "ESetRecSpeedOffset", 0x1587f8),
    (233, "ESetAutoDriveCamOffMovie", 0x157540),
    (234, "ESetAutoDriveCamOnMovie", 0x157838),
    (239, "EEndControlToSpline", 0x14c110),
    (240, "ECustomTransition", 0x14b020),
    (241, "ESetAudioMix", 0x157308),
    (242, "ECameraLockOn", 0x148e30),
    (251, "ECameraShake", 0x149118),
    (273, "EReleaseStream", 0x1560d8),
    (279, "ESetReflectiveObject", 0x1589f0),
];

/// The class name of an event kind, when resolved [M].
pub fn event_name(kind: u32) -> Option<&'static str> {
    EVENT_NAMES.iter().find(|e| e.0 == kind).map(|e| e.1)
}

/// One scripted event: its kind and its raw parameter block (block +0 repeats the kind).
#[derive(Clone, Debug, PartialEq)]
pub struct ScriptEvent {
    pub kind: u32,
    pub params: Vec<u8>,
}

impl ScriptEvent {
    /// The parameter word `i` (word 0 is the repeated kind; factories read from word 1).
    pub fn word(&self, i: usize) -> Option<u32> {
        self.params.get(i * 4..i * 4 + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
    }
    pub fn f32(&self, i: usize) -> Option<f32> { self.word(i).map(f32::from_bits) }
    pub fn reference(&self, i: usize) -> Option<Ref> { self.word(i).map(Ref::from_word) }
    pub fn name(&self) -> Option<&'static str> { event_name(self.kind) }
}

/// Parses one `el` payload [M the walkers' stride; see the module doc].
pub fn parse_event_list(p: &[u8]) -> Result<Vec<ScriptEvent>> {
    let le = |o: usize| -> Result<u32> {
        p.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .ok_or_else(|| crate::buf::FormatError::Invalid(format!("event list: read at {o:#x} past {}", p.len())))
    };
    let n = le(0)? as usize;
    if 0x10 + n * 0x10 > p.len() {
        return invalid(format!("event list: {n} entries do not fit {} bytes", p.len()));
    }
    let mut out = Vec::with_capacity(n);
    for j in 0..n {
        let e = 0x10 + j * 0x10;
        let (kind, size, rel) = (le(e)?, le(e + 4)? as usize, le(e + 8)? as usize);
        let at = e + rel;
        let params = p.get(at..at + size)
            .ok_or_else(|| crate::buf::FormatError::Invalid(format!("event {j}: block {at:#x}+{size} past {}", p.len())))?;
        out.push(ScriptEvent { kind, params: params.to_vec() });
    }
    Ok(out)
}

/// One 0x40-byte `Trgr` record (see the module doc).
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerRecord {
    pub pos: [f32; 3],
    pub radius: f32,
    /// +0x12: the flag word [M 0x22cbe0 / 0x22c890].
    pub flags: u16,
    /// +0x18: the event list fired [M 0x22c890 reads it; [D] the `el` reference].
    pub events: Option<u16>,
    pub raw: [u8; 0x40],
}

impl TriggerRecord {
    pub fn parse(r: &[u8]) -> Option<TriggerRecord> {
        let raw: [u8; 0x40] = r.get(..0x40)?.try_into().ok()?;
        let f = |o: usize| f32::from_le_bytes(raw[o..o + 4].try_into().unwrap());
        let w = u32::from_le_bytes(raw[0x18..0x1c].try_into().unwrap());
        Some(TriggerRecord {
            pos: [f(0), f(4), f(8)],
            radius: f(0xc),
            flags: u16::from_le_bytes([raw[0x12], raw[0x13]]),
            events: match Ref::from_word(w) { Ref::EventList(i) => Some(i), _ => None },
            raw,
        })
    }
}

/// One 0x20-byte `Rule` entry, with +0x0c read as the fired event list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuleRecord {
    pub kind: u32,
    pub flags: u32,
    pub group: i32,
    pub events: Option<u16>,
    pub payload: [u8; 16],
}

/// The script part of a track's `Map ` record.
#[derive(Clone, Debug, Default)]
pub struct MapScript {
    pub lists: BTreeMap<u16, Vec<ScriptEvent>>,
    pub strings: BTreeMap<u16, String>,
    pub triggers: Vec<TriggerRecord>,
    pub rules: Vec<RuleRecord>,
    /// Every other child chunk's payload by tag (`MSet`, `AIEl`, `AISp`, `Cams`, …).
    pub chunks: BTreeMap<String, Vec<u8>>,
    /// The `pt  ` path tracks by instance index.
    pub tracks: BTreeMap<u16, PathTrack>,
}

impl MapScript {
    pub fn load(crp: &Crp<'_>) -> Result<MapScript> {
        let map = crp.pool_find(MAP)?.ok_or_else(|| crate::buf::FormatError::Invalid("no Map record".into()))?;
        let mut s = MapScript::default();
        for r in crp.record_window(&map)? {
            let pay = crp.payload(&r)?;
            if let Some(idx) = r.instance() {
                match (r.tag >> 16) as u16 {
                    CODE_EL => { s.lists.insert(idx, parse_event_list(pay)?); }
                    CODE_SR => {
                        let end = pay.iter().position(|&b| b == 0).unwrap_or(pay.len());
                        s.strings.insert(idx, String::from_utf8_lossy(&pay[..end]).into_owned());
                    }
                    CODE_PT => { s.tracks.insert(idx, PathTrack::parse(pay)?); }
                    _ => {}
                }
                continue;
            }
            let tag = String::from_utf8_lossy(&r.tag_bytes()).into_owned();
            match tag.as_str() {
                "Trgr" => s.triggers = pay.chunks_exact(0x40).filter_map(TriggerRecord::parse).collect(),
                "Rule" => {
                    s.rules = pay.chunks_exact(0x20).map(|e| {
                        let le = |o: usize| u32::from_le_bytes(e[o..o + 4].try_into().unwrap());
                        RuleRecord {
                            kind: le(0), flags: le(4), group: le(8) as i32,
                            events: match Ref::from_word(le(0xc)) { Ref::EventList(i) => Some(i), _ => None },
                            payload: e[0x10..0x20].try_into().unwrap(),
                        }
                    }).collect();
                }
                _ => {}
            }
            s.chunks.insert(tag, pay.to_vec());
        }
        Ok(s)
    }

    pub fn string(&self, r: Ref) -> Option<&str> {
        match r { Ref::Str(i) => self.strings.get(&i).map(|s| s.as_str()), _ => None }
    }

    /// The triggers whose list holds an event of `kind` — `FUN_0022ca18` over every record.
    pub fn triggers_with(&self, kind: u32) -> Vec<usize> {
        (0..self.triggers.len()).filter(|&i| {
            self.triggers[i].events.and_then(|l| self.lists.get(&l))
                .is_some_and(|l| l.iter().any(|e| e.kind == kind))
        }).collect()
    }

    /// The `AISp` record a `CARP::<<Map>>::{AISp}::<byte offset>` string names (offset / 0x70).
    pub fn spline_of(&self, r: Ref) -> Option<usize> {
        let (chunk, off) = parse_carp_ref(self.string(r)?)?;
        (chunk == "AISp" && off % AI_SPLINE_SIZE == 0).then_some(off / AI_SPLINE_SIZE)
    }

    /// Every `AISp` record.
    pub fn splines(&self) -> Vec<AiSplineRecord> {
        self.chunks.get("AISp").map(|p| parse_ai_splines(p)).unwrap_or_default()
    }

    /// The path track an `AISp` record's +0x40 reference names (a `CARP::<<Map>>::{pt  nnnn}`
    /// string, the index in hex [D]).
    pub fn track_of_spline(&self, sp: &AiSplineRecord) -> Option<u16> {
        let s = self.string(sp.track?)?;
        let hex = s.strip_prefix("CARP::<<Map>>::{pt  ")?.strip_suffix('}')?;
        u16::from_str_radix(hex, 16).ok()
    }

    /// The trigger index a `CARP::<<Map>>::{Trgr}::<byte offset>` string names (offset / 0x40).
    pub fn trigger_of(&self, r: Ref) -> Option<usize> {
        let (chunk, off) = parse_carp_ref(self.string(r)?)?;
        (chunk == "Trgr" && off % 0x40 == 0).then_some(off / 0x40)
    }
}

/// Splits a `CARP::<<Map>>::{<chunk>}::<byte offset>` string [D: the MIS01 `sr` strings] into
/// the chunk tag and the byte offset inside its payload.
pub fn parse_carp_ref(s: &str) -> Option<(&str, usize)> {
    let rest = s.strip_prefix("CARP::<<Map>>::{")?;
    let (chunk, off) = rest.split_once("}::")?;
    Some((chunk, off.parse().ok()?))
}

/// One 0x100-byte `AIEl` record, minimally: the marker position at +0x30 and the vehicle
/// name at +0x54 [D: the MIS01 records; the consumer of the disc layout is not traced].
#[derive(Clone, Debug, PartialEq)]
pub struct AiElementRecord {
    pub pos: [f32; 3],
    pub vehicle: String,
}

pub fn parse_ai_elements(p: &[u8]) -> Vec<AiElementRecord> {
    p.chunks_exact(0x100).map(|r| {
        let f = |o: usize| f32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        let name = &r[0x54..0x74];
        let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
        AiElementRecord { pos: [f(0x30), f(0x34), f(0x38)], vehicle: String::from_utf8_lossy(&name[..end]).into_owned() }
    }).collect()
}

/// The size of one `AISp` record: `FUN_002da590` walks the chunk at a 0x70 stride and fixes
/// up the reference at +0x40 of each [M].
pub const AI_SPLINE_SIZE: usize = 0x70;

/// One `AISp` record [M `FUN_002da590` / `FUN_0013da80` / `FUN_002db690`; D the MIS01 records].
#[derive(Clone, Debug, PartialEq)]
pub struct AiSplineRecord {
    /// +0x00: a 4x4 row matrix (rows 0-2 the axes, row 3 the position) [D: every MIS01 record
    /// holds a rotation and a w = 1 position row]. The follower does not read it [M 0x13da80].
    pub matrix: [[f32; 4]; 4],
    /// +0x40: the path track (an `sr` reference); `EControlToSpline` acts only when the
    /// fixed-up pointer is non-zero (`FUN_0014ae60`) [M].
    pub track: Option<Ref>,
    /// +0x44: the name; `FUN_0013da80` stores its address in the follower (+0xd0) [M].
    pub name: String,
    /// +0x6c: non-zero → the controller's offset matrix (+0x70) is the identity; zero → it
    /// comes from the track's pose at time 0 through `FUN_002db478` (not decomposed)
    /// [M `FUN_002db690`].
    pub identity_offset: bool,
}

pub fn parse_ai_splines(p: &[u8]) -> Vec<AiSplineRecord> {
    p.chunks_exact(AI_SPLINE_SIZE).map(|r| {
        let f = |o: usize| f32::from_le_bytes(r[o..o + 4].try_into().unwrap());
        let mut matrix = [[0.0; 4]; 4];
        for (i, row) in matrix.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() { *v = f(i * 16 + j * 4); }
        }
        let w = u32::from_le_bytes(r[0x40..0x44].try_into().unwrap());
        let name = &r[0x44..0x6c];
        let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
        AiSplineRecord {
            matrix,
            track: (w != 0).then(|| Ref::from_word(w)),
            name: String::from_utf8_lossy(&name[..end]).into_owned(),
            identity_offset: r[0x6c] != 0,
        }
    }).collect()
}

/// One keyed channel of a path track (0x14 bytes at track +0x40 + i*0x14) [M: `FUN_002db320`
/// reads the flags at channel +0x10, `FUN_002dae58` the time table at +0x04 and the key count
/// at +0x0c, `FUN_002db080` the keys at +0x08; all offsets are from the track's start].
#[derive(Clone, Debug, PartialEq)]
pub struct TrackChannel {
    /// +0x10: bit 0 → `FUN_002db188` (not ported), bit 1 → quaternion keys blended by
    /// `FUN_002cc018`, bit 2 → one float per key, bit 3 → the `FUN_002bfb50` time mapping
    /// (not ported).
    pub flags: u32,
    /// The name at the +0x00 offset (MIS01: "xform" / "xlate" [D]); no code reads it.
    pub name: String,
    pub times: Vec<f32>,
    /// The keys, four floats each (a bit-2 channel's single float is repeated).
    pub keys: Vec<[f32; 4]>,
}

/// A `pt  ` record: a keyed path, one rotation and one translation channel
/// [M `FUN_002db370`].
#[derive(Clone, Debug, PartialEq)]
pub struct PathTrack {
    /// +0x00: the length in 60 Hz frames; `FUN_001b1840` returns it and the follower wraps
    /// there [M `FUN_001b14b8`].
    pub duration: f32,
    /// +0x0c: the rotation channel (-1 none) [M `FUN_002db370`].
    pub rot: Option<usize>,
    /// +0x10: the translation channel (-1 none) [M `FUN_002db370`].
    pub pos: Option<usize>,
    /// +0x18: the name (`FUN_00230ab0` compares it).
    pub name: String,
    pub channels: Vec<TrackChannel>,
}

impl PathTrack {
    pub fn parse(p: &[u8]) -> Result<PathTrack> {
        let b = crate::buf::Buf(p);
        let cstr = |o: usize| -> String {
            let t = p.get(o..).unwrap_or(&[]);
            let end = t.iter().position(|&c| c == 0).unwrap_or(t.len());
            String::from_utf8_lossy(&t[..end]).into_owned()
        };
        let n = b.u32(4)? as usize;
        if n > 16 { return invalid(format!("pt: {n} channels")); }
        let idx = |o: usize| -> Result<Option<usize>> {
            let v = b.u32(o)? as i32;
            Ok((v >= 0).then_some(v as usize))
        };
        let mut channels = Vec::with_capacity(n);
        for i in 0..n {
            let c = 0x40 + i * 0x14;
            let name = b.u32(c)? as usize;
            let times_at = b.u32(c + 4)? as usize;
            let keys_at = b.u32(c + 8)? as usize;
            let count = b.u32(c + 12)? as usize;
            let flags = b.u32(c + 16)?;
            let times = (0..count).map(|k| b.f32(times_at + 4 * k)).collect::<Result<Vec<_>>>()?;
            let keys = (0..count).map(|k| {
                if flags & 4 != 0 {
                    let v = b.f32(keys_at + 4 * k)?;
                    Ok([v; 4])
                } else {
                    let o = keys_at + 16 * k;
                    Ok([b.f32(o)?, b.f32(o + 4)?, b.f32(o + 8)?, b.f32(o + 12)?])
                }
            }).collect::<Result<Vec<_>>>()?;
            channels.push(TrackChannel { flags, name: cstr(name), times, keys });
        }
        let t = PathTrack { duration: b.f32(0)?, rot: idx(0xc)?, pos: idx(0x10)?, name: cstr(0x18), channels };
        for c in [t.rot, t.pos].into_iter().flatten() {
            if c >= t.channels.len() { return invalid(format!("pt {}: channel {c} of {}", t.name, t.channels.len())); }
        }
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(entries: &[(u32, &[u32])]) -> Vec<u8> {
        let mut head = Vec::new();
        head.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        head.extend_from_slice(&[0; 12]);
        let mut blocks: Vec<Vec<u8>> = Vec::new();
        for (_, ws) in entries {
            let mut b = Vec::new();
            for w in *ws { b.extend_from_slice(&w.to_le_bytes()); }
            blocks.push(b);
        }
        // entries then blocks, each rel measured from its own entry
        let n = entries.len();
        let mut at = 0x10 + n * 0x10;
        let mut out = head;
        for (j, (k, _)) in entries.iter().enumerate() {
            let e = 0x10 + j * 0x10;
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&(blocks[j].len() as u32).to_le_bytes());
            out.extend_from_slice(&((at - e) as u32).to_le_bytes());
            out.extend_from_slice(&[0; 4]);
            at += blocks[j].len();
        }
        for b in blocks { out.extend_from_slice(&b); }
        out
    }

    #[test]
    fn event_list_blocks_resolve_relative_to_their_own_entry() {
        let p = list(&[(77, &[77, (-60.0f32).to_bits(), (-20.0f32).to_bits()]), (41, &[41, 19])]);
        let ev = parse_event_list(&p).unwrap();
        assert_eq!(ev.len(), 2);
        assert_eq!(ev[0].kind, 77);
        assert_eq!(ev[0].f32(1), Some(-60.0));
        assert_eq!(ev[0].f32(2), Some(-20.0));
        assert_eq!(ev[1].word(1), Some(19));
        assert_eq!(ev[0].name(), Some("ESetAutoDriveCamOn"));
        assert_eq!(ev[1].name(), Some("EChangeStage"));
    }

    #[test]
    fn event_list_rejects_a_count_that_overruns() {
        let mut p = list(&[(22, &[22, 0])]);
        p[0] = 9;
        assert!(parse_event_list(&p).is_err());
    }

    #[test]
    fn reference_words_split_into_code_and_index() {
        assert_eq!(Ref::from_word(0x656c_02ae), Ref::EventList(686));
        assert_eq!(Ref::from_word(0x7372_0a3d), Ref::Str(0xa3d));
        assert_eq!(Ref::from_word(19), Ref::Other(19));
        assert_eq!(parse_carp_ref("CARP::<<Map>>::{Trgr}::43712"), Some(("Trgr", 43712)));
        assert_eq!(parse_carp_ref("CHAR::paris"), None);
    }

    #[test]
    fn trigger_record_reads_position_radius_and_list() {
        let mut r = [0u8; 0x40];
        r[0..4].copy_from_slice(&778.25f32.to_le_bytes());
        r[0xc..0x10].copy_from_slice(&2.5f32.to_le_bytes());
        r[0x12..0x14].copy_from_slice(&0x0407u16.to_le_bytes());
        r[0x18..0x1c].copy_from_slice(&0x656c_02aeu32.to_le_bytes());
        let t = TriggerRecord::parse(&r).unwrap();
        assert_eq!(t.pos[0], 778.25);
        assert_eq!(t.radius, 2.5);
        assert_eq!(t.flags, 0x0407);
        assert_eq!(t.events, Some(686));
    }

    /// A two-channel track laid out like MIS01's `pt  0001`: header, the channel table at
    /// +0x40 (stride 0x14: name, times, keys, count, flags), keys, times, names.
    #[test]
    fn path_track_reads_the_channel_table() {
        let mut p = vec![0u8; 0x100];
        let w = |p: &mut Vec<u8>, o: usize, v: u32| p[o..o + 4].copy_from_slice(&v.to_le_bytes());
        w(&mut p, 0, 120.0f32.to_bits());
        w(&mut p, 4, 2);
        w(&mut p, 0xc, 0);
        w(&mut p, 0x10, 1);
        p[0x18..0x1d].copy_from_slice(b"path0");
        // channel 0: quaternions, 2 keys
        for (o, v) in [(0x40, 0xf0u32), (0x44, 0xc0), (0x48, 0x70), (0x4c, 2), (0x50, 2)] { w(&mut p, o, v); }
        // channel 1: positions, 2 keys
        for (o, v) in [(0x54, 0xf6u32), (0x58, 0xc8), (0x5c, 0x90), (0x60, 2), (0x64, 0)] { w(&mut p, o, v); }
        w(&mut p, 0x70 + 12, 1.0f32.to_bits());
        w(&mut p, 0x80 + 12, 1.0f32.to_bits());
        w(&mut p, 0x90, 5.0f32.to_bits());
        w(&mut p, 0xa0, 7.0f32.to_bits());
        w(&mut p, 0xc4, 120.0f32.to_bits());
        w(&mut p, 0xcc, 60.0f32.to_bits());
        p[0xf0..0xf5].copy_from_slice(b"xform");
        p[0xf6..0xfb].copy_from_slice(b"xlate");
        let t = PathTrack::parse(&p).unwrap();
        assert_eq!((t.duration, t.rot, t.pos, t.name.as_str()), (120.0, Some(0), Some(1), "path0"));
        assert_eq!((t.channels[0].name.as_str(), t.channels[0].flags), ("xform", 2));
        assert_eq!(t.channels[0].keys[1], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(t.channels[1].times, vec![0.0, 60.0]);
        assert_eq!(t.channels[1].keys[1][0], 7.0);
        // A channel index past the table fails.
        w(&mut p, 0x10, 2);
        assert!(PathTrack::parse(&p).is_err());
    }
}
