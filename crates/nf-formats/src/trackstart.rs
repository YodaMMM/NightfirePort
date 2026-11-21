//! Where a driving mission puts the player vehicle at the start: the `Map` `Trgr` trigger that
//! carries the `EStartMission` action (docs/formats/trackstart.md).
//!
//! The game's chain (DRIVING.ELF):
//! 1. Mission setup `FUN_00176658` 0x176658 runs "Init World" (`FUN_0022faf8`), which hands the
//!    `Map` window's `Trgr` record to `FUN_0022cbe0` 0x22cbe0 (the 0x40-byte trigger array). That
//!    calls `FUN_0022cd30` 0x22cd30, which walks every trigger and, when its action list holds an
//!    action of id **0x16** (`FUN_0022ca18(trigger, 0x16)`), fires the trigger at once
//!    (`FUN_0022c890(trigger, 1, -1, 0)`) [M].
//! 2. Firing (`FUN_0022c890` 0x22c890) stores the trigger in the "current trigger" global
//!    `DAT_0032e738` and runs each action: list `+0` = count, entries of 16 bytes from `+0x10`,
//!    entry `+0` = id, entry `+8` = byte offset (from the entry) of the action's arguments; the id
//!    indexes the creator table at 0x36fda0 (`FUN_001757b8`) [M].
//! 3. Creator 0x16 is `FUN_0015c920` 0x15c920: it bump-allocates an event in the event buffer
//!    (`FUN_00175730`, `DAT_0032e6fc`) and builds an `EStartMission` (vtable 0x3703c0, typeinfo
//!    "13EStartMission" 0x3728e8) holding `DAT_0032e738` (the trigger) and the argument word `+4`
//!    [M 0x15c920 / 0x15c718].
//! 4. Its handler `FUN_0015c738` 0x15c738 reads the trigger's `+0x00` (x, y, z) and `+0x30`
//!    (direction x, y, z), calls `FUN_001fee68(0x339190, pos, dir)` — the player car placement —,
//!    sets the car's velocity to the normalised direction × the argument × the car's `+0x4c`, and
//!    marks the mission started (`FUN_00167d80()+4 = 1`) [M].
//! 5. `FUN_001fee68` 0x1fee68 keeps x and z, replaces y with the ground height under the point plus
//!    1.0 (`FUN_002151b8`; 0 + 1.0 when the query finds nothing), and builds the car matrix from the
//!    direction (`FUN_00210540`: row 2 = the normalised direction, row 1 from world up) [M].
//!
//! Before that event runs, "Spawn player car" in the same setup places the car at
//! (0, ground(0, 0, 0) + 1, 0) facing +z (`FUN_001ff850` with the constant (0, 0, 1, 0) at
//! 0x376db0) [M 0x176658]. That the queued `EStartMission` is dispatched after the spawn (the event
//! queue is filled during "Init World", before "Spawn player car") is inferred from the order;
//! the dispatch loop is not traced.
//!
//! The trigger's `+0x18` word is a reference: high half `0x656c` (`el`), low half the index of the
//! `Map` window's `el  #N` record whose payload is the action list [D: every trigger of the 8
//! tracks resolves; each track has exactly one trigger with an id-0x16 action].

use std::collections::HashMap;

use crate::buf::{invalid, FormatError, Result};
use crate::crp::{tag, Crp};
use crate::crp_mesh::CollisionTri;

/// `Trgr` — the `Map` window's trigger array (0x40-byte records) [M 0x22faf8: `FUN_002d5a10(map, 'Trgr')`].
pub const TAG_TRGR: u32 = 0x5472_6772;
/// `el  ` — an action-list record of the `Map` window [D].
pub const TAG_EL: u32 = 0x656c_2020;
/// The trigger record size [M 0x22cd30: `iVar11 * 0x40`].
pub const TRIGGER_SIZE: usize = 0x40;
/// The `EStartMission` action id: `FUN_0022cd30` fires triggers holding it at world init; the
/// creator table entry 0x16 (0x36fda0 + 0x58) is `FUN_0015c920` [M].
pub const ACTION_START_MISSION: u32 = 0x16;
/// `FUN_001fee68` puts the car this far above the ground under the marker [M 0x1fee68: `+ 1.0`].
pub const SPAWN_LIFT: f32 = 1.0;
/// The ground query takes triangles whose lowest corner is below the probe y plus this [M 0x214338 /
/// 0x213ff8: `(y + 0.5) - min_y > 0`].
pub const GROUND_SLACK: f32 = 0.5;

/// One action of a trigger's list [M 0x22c890].
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerAction {
    /// The creator-table index.
    pub id: u32,
    /// The argument bytes (from the entry's `+8` offset to the end of the list payload). The
    /// block starts with the id word again [D: every action of the 8 tracks]; the creators read
    /// their parameters from `+4` on (`FUN_0015c920`: `*(param_1 + 4)`) [M].
    pub args: Vec<u8>,
}

impl TriggerAction {
    /// The argument word at byte `4 * i` (word 0 = the id copy; the creators' `param_1 + 4*i`).
    pub fn word(&self, i: usize) -> u32 {
        self.args.get(4 * i..4 * i + 4).map_or(0, |b| u32::from_le_bytes(b.try_into().unwrap()))
    }
    /// The same word read as an f32 (the creators' float parameters).
    pub fn float(&self, i: usize) -> f32 { f32::from_bits(self.word(i)) }
}

/// One `Map` `Trgr` record with its resolved action list. Field offsets from the volume tests
/// `FUN_0022eb28` 0x22eb28 / `FUN_0022e3a0` 0x22e3a0 / `FUN_0022eec0` 0x22eec0 and the walkers
/// `FUN_0022d240` 0x22d240 / `FUN_0022dae8` 0x22dae8 [M].
#[derive(Clone, Debug, PartialEq)]
pub struct Trigger {
    pub index: usize,
    /// `+0x00` x, y, z: the volume's base point.
    pub pos: [f32; 3],
    /// `+0x0c`: the radius (sphere / cylinder) [M 0x22eb28: `trigger+0xc + car+0x7c`].
    pub radius: f32,
    /// `+0x10`: a byte the trigger code traced here does not read.
    pub kind: u8,
    /// `+0x11`: the volume shape: 1 oriented box, 2 sphere, 3 vertical cylinder [M 0x22eb28's
    /// switch; any other value logs 0x39af58 and never fires].
    pub shape: u8,
    /// `+0x12`: the flag word (the `TRIGGER_*` bits).
    pub flags: u16,
    /// `+0x14`: the height (cylinder top = y + height; box height) [M 0x22eb28 / 0x22e3a0].
    pub height: f32,
    /// `+0x20` x, y, z: the box's side axis [M 0x22eb28: the first OBB axis].
    pub axis: [f32; 3],
    /// `+0x2c`: the box width along `axis` [M 0x22e3a0: `param_4[0xb] * 0.5`].
    pub width: f32,
    /// `+0x30` x, y, z: the direction `EStartMission` gives the car [M 0x15c738]; also the box's
    /// forward axis and the one-way test direction (flag 0x800) [M 0x22ca68].
    pub dir: [f32; 3],
    /// `+0x3c`: the box length along `dir` [M 0x22e3a0: `param_4[0xf] * 0.5`].
    pub length: f32,
    pub actions: Vec<TriggerAction>,
}

/// `+0x12` bit 0: the trigger can fire; `EEnableTrigger`/`EDisableTrigger` set/clear it
/// [M 0x14bcd8 / 0x14b790]; every walker skips a trigger without it [M 0x22d240].
pub const TRIGGER_ENABLED: u16 = 0x0001;
/// Bit 1: fire once: `FUN_0022c890` clears bit 0 after running the list [M 0x22c890 tail].
pub const TRIGGER_ONCE: u16 = 0x0002;
/// Bit 2: fired by a car whose kind byte (`+0x6d`) is 1, the player car (spawned with type 0,
/// which `FUN_001862c0` maps to kind 1 [M 0x176658 `FUN_001ff850(.., 0, ..)`]) [M 0x22d240].
pub const TRIGGER_BY_PLAYER: u16 = 0x0004;
/// Bit 3: fired by a car of kind 2 [M 0x22d240].
pub const TRIGGER_BY_KIND2: u16 = 0x0008;
/// Bit 6: fired by a car of any other kind [M 0x22d240].
pub const TRIGGER_BY_OTHER: u16 = 0x0040;
/// Bit 9: the class `FUN_0022d5a8` tests (a moving-object walk); clears bit 10 at load
/// [M 0x22cbe0, 0x22d5a8].
pub const TRIGGER_OBJECT_CLASS: u16 = 0x0200;
/// Bit 10: fires only while the trigger manager's `+8` word is set: 1 from world init
/// [M 0x22cd30: `DAT_0033aa88[2] = 1`], 0 once `EPlayerWin`/`EPlayerLose` run [M 0x1551b8 /
/// 0x154cb0].
pub const TRIGGER_NEEDS_LIVE: u16 = 0x0400;
/// Bit 11: one-way: only while `dot(dir, car velocity) >= 0` [M 0x22ca68].
pub const TRIGGER_ONE_WAY: u16 = 0x0800;
/// Bit 13: never fires from the walkers [M 0x22d240: `(flags & 0x2000) != 0` skips].
pub const TRIGGER_SUPPRESSED: u16 = 0x2000;
/// `sr  `: the `Map` window's path-string records; an argument word `0x7372_nnnn` names one,
/// whose payload is a path like `CARP::<<Map>>::{Trgr}::47680` (a byte offset into `Trgr`) [D].
pub const TAG_SR: u32 = 0x7372_2020;
/// `Rule`: the rule table (0x20-byte entries) [M 0x22faf8 -> 0x205220].
pub const TAG_RULE: u32 = 0x5275_6c65;
/// `MSet`: the mission settings [M 0x22faf8 -> 0x205220 `param_3`].
pub const TAG_MSET: u32 = 0x4d53_6574;

/// One `Rule` entry with its resolved action list: entry `+0x0c` is an `el` reference (the
/// rule object's `+0` [M 0x209318]) that `FUN_002095f8` runs when the rule's test passes [M].
#[derive(Clone, Debug, PartialEq)]
pub struct RuleDef {
    pub index: usize,
    /// The 0x20 file bytes (type, flags, stage, list ref, type payload) [M 0x209318 / 0x209390].
    pub raw: [u8; 0x20],
    pub actions: Vec<TriggerAction>,
}

impl RuleDef {
    /// The entry word at byte `off`.
    pub fn word(&self, off: usize) -> u32 { u32::from_le_bytes(self.raw[off..off + 4].try_into().unwrap()) }
}

/// The track's script data: triggers, rules, the `MSet` payload and the `sr` path strings.
#[derive(Clone, Debug, Default)]
pub struct MapScript {
    pub triggers: Vec<Trigger>,
    pub rules: Vec<RuleDef>,
    pub mset: Option<Vec<u8>>,
    /// `sr` tag word -> path string.
    pub refs: HashMap<u32, String>,
}

impl MapScript {
    /// The path an argument word names, when it is an `sr` reference.
    pub fn ref_path(&self, word: u32) -> Option<&str> {
        if word >> 16 != TAG_SR >> 16 {
            return None;
        }
        self.refs.get(&word).map(|s| s.as_str())
    }
    /// The trigger an argument word names (`CARP::<<Map>>::{Trgr}::<byte offset>`), as an index
    /// into [`MapScript::triggers`]. The loader turns these words into trigger pointers, which
    /// `EEnableTrigger` / `EDisableTrigger` / `EFireRandomTrigger` dereference [M 0x14bcd8].
    pub fn trigger_ref(&self, word: u32) -> Option<usize> {
        let off: usize = self.ref_path(word)?.strip_prefix("CARP::<<Map>>::{Trgr}::")?.parse().ok()?;
        (off % TRIGGER_SIZE == 0 && off / TRIGGER_SIZE < self.triggers.len()).then_some(off / TRIGGER_SIZE)
    }
}

/// The player start of a track.
#[derive(Clone, Debug, PartialEq)]
pub struct TrackSpawn {
    /// The `Trgr` index of the `EStartMission` trigger.
    pub trigger: usize,
    /// The trigger position as stored.
    pub marker: [f32; 3],
    /// Where the car is put: the marker's x/z, y = ground + [`SPAWN_LIFT`] (or 0 + 1 with no
    /// ground) [M 0x1fee68].
    pub pos: [f32; 3],
    /// The ground height found under the marker, if any.
    pub ground: Option<f32>,
    /// The normalised direction (the car matrix's row 2) [M 0x210540].
    pub dir: [f32; 3],
    /// `atan2(dir.x, dir.z)`: the heading about +y with yaw 0 = +z.
    pub yaw: f32,
    /// The `EStartMission` argument word (`+4` of the arguments), read as f32 (`lwc1 $f12, 4($s0)`
    /// at 0x15c93c): the start speed in m/s. The handler writes `norm(dir) × arg × inst+0x4c`
    /// into the sim instance's momentum (+0x40), and +0x4c is the body mass, so the velocity is
    /// `norm(dir) × arg` [M 0x15c920 / 0x15c738; FUN_001f6e98 :124 stores the mass at +0x4c].
    pub speed_factor: f32,
}

fn u32_at(d: &[u8], o: usize) -> Result<u32> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| FormatError::Invalid(format!("u32 at {o:#x} past the end ({})", d.len())))
}

fn f32_at(d: &[u8], o: usize) -> Result<f32> { Ok(f32::from_bits(u32_at(d, o)?)) }

fn action_list(el: &HashMap<u16, &[u8]>, refw: u32, what: &str) -> Result<Vec<TriggerAction>> {
    let mut actions = Vec::new();
    if refw >> 16 != 0x656c {
        return Ok(actions);
    }
    let l = el
        .get(&(refw as u16))
        .ok_or_else(|| FormatError::Invalid(format!("{what}: el#{} not in the Map window", refw as u16)))?;
    let n = u32_at(l, 0)? as usize;
    if 0x10 + n * 0x10 > l.len() {
        return invalid(format!("{what}: {n} actions overrun el#{} ({} bytes)", refw as u16, l.len()));
    }
    for j in 0..n {
        let e = 0x10 + j * 0x10;
        let a = e + u32_at(l, e + 8)? as usize;
        actions.push(TriggerAction { id: u32_at(l, e)?, args: l.get(a..).unwrap_or(&[]).to_vec() });
    }
    Ok(actions)
}

/// Every `Map` `Trgr` record with its action list [M 0x22cbe0 / 0x22c890 + D the `el` reference].
pub fn map_triggers(crp: &Crp<'_>) -> Result<Vec<Trigger>> { Ok(map_script(crp)?.triggers) }

/// The whole script of a track: triggers, rules, `MSet`, `sr` paths [M 0x22faf8 resolves `Rule`,
/// `MSet` and `Trgr` from the `Map` window].
pub fn map_script(crp: &Crp<'_>) -> Result<MapScript> {
    let map = crp
        .pool()?
        .into_iter()
        .find(|r| r.tag == tag::MAP)
        .ok_or_else(|| FormatError::Invalid("no Map article in the pool".into()))?;
    let recs = crp.record_window(&map)?;
    let mut el: HashMap<u16, &[u8]> = HashMap::new();
    let mut refs = HashMap::new();
    for r in &recs {
        if r.tag_code() == TAG_EL {
            el.insert(r.instance().unwrap_or(0), crp.payload(r)?);
        } else if r.tag_code() == TAG_SR {
            let p = crp.payload(r)?;
            let s = &p[..p.iter().position(|&b| b == 0).unwrap_or(p.len())];
            refs.insert(r.tag, String::from_utf8_lossy(s).into_owned());
        }
    }
    let mut out = MapScript { refs, ..Default::default() };
    if let Some(trgr) = recs.iter().find(|r| r.tag == TAG_TRGR) {
        let p = crp.payload(trgr)?;
        for (index, it) in p.chunks_exact(TRIGGER_SIZE).enumerate() {
            let v = |o: usize| -> Result<[f32; 3]> { Ok([f32_at(it, o)?, f32_at(it, o + 4)?, f32_at(it, o + 8)?]) };
            out.triggers.push(Trigger {
                index,
                pos: v(0)?,
                radius: f32_at(it, 0xc)?,
                kind: it[0x10],
                shape: it[0x11],
                flags: u16::from_le_bytes([it[0x12], it[0x13]]),
                height: f32_at(it, 0x14)?,
                axis: v(0x20)?,
                width: f32_at(it, 0x2c)?,
                dir: v(0x30)?,
                length: f32_at(it, 0x3c)?,
                actions: action_list(&el, u32_at(it, 0x18)?, &format!("trigger {index}"))?,
            });
        }
    }
    if let Some(rule) = recs.iter().find(|r| r.tag == TAG_RULE) {
        let p = crp.payload(rule)?;
        for (index, e) in p.chunks_exact(0x20).enumerate() {
            let mut raw = [0u8; 0x20];
            raw.copy_from_slice(e);
            out.rules.push(RuleDef { index, raw, actions: action_list(&el, u32_at(e, 0xc)?, &format!("rule {index}"))? });
        }
    }
    if let Some(m) = recs.iter().find(|r| r.tag == TAG_MSET) {
        out.mset = Some(crp.payload(m)?.to_vec());
    }
    Ok(out)
}

/// `aq  `: a prop article's animation track, 0x20-byte keyframes; a keyframe's `+0x1c` word is an
/// `el` reference (0 = none) that the load fixup `FUN_002d9de8` resolves [M 0x2da838 registers it
/// for `aq`; 0x2d9de8 walks `count` keyframes of 0x20 bytes and fixes `+0x1c`].
pub const TAG_AQ: u32 = 0x6171_2020;
/// `an  `: a prop article's animation: 16-byte entries, one per part, whose `+0` names an `aq`
/// track (`0x6171_nnnn`, 0 = no track for that part) [D: every article of the 8 tracks].
pub const TAG_AN: u32 = 0x616e_2020;
/// `aq` keyframe size [M 0x2d9de8: `iVar1 + 0x20`].
pub const KEYFRAME_SIZE: usize = 0x20;

/// One `aq` track of a prop article with the action lists on its keyframes.
#[derive(Clone, Debug, PartialEq)]
pub struct PropTrack {
    /// The `aq` instance index.
    pub index: u16,
    /// Keyframe count (payload / 0x20).
    pub keys: usize,
    /// (keyframe, its action list), for the keyframes whose `+0x1c` names an `el` list.
    pub events: Vec<(usize, Vec<TriggerAction>)>,
}

/// One `an` animation of a prop article: the `aq` track per part (None = that part has none).
#[derive(Clone, Debug, PartialEq)]
pub struct PropAnim {
    pub index: u16,
    pub tracks: Vec<Option<u16>>,
}

/// A prop article (an `Arti` of the pool other than `Map`) with its animations and the action
/// lists its keyframes run. The game's animation player `FUN_001a35f0` runs a keyframe's list
/// when the animation passes that keyframe: each action's creator, with the prop part as the
/// context (`DAT_0032e708`) and no current trigger [M 0x1a35f0 tail]. What starts an animation
/// on a prop (its state machine: `FUN_001a33c0` from `EPlaySystemAnim` 0x154ad8, the car and
/// weapon hits 0x193938 / 0x1fc2f0 / 0x13b2a8, …) is not decoded here.
#[derive(Clone, Debug, PartialEq)]
pub struct PropScript {
    /// The article's `Name` record.
    pub name: String,
    pub tracks: Vec<PropTrack>,
    pub anims: Vec<PropAnim>,
}

impl PropScript {
    /// Every keyframe action list of animation `an`, in part then keyframe order.
    pub fn anim_events(&self, an: u16) -> Vec<&[TriggerAction]> {
        let Some(a) = self.anims.iter().find(|a| a.index == an) else { return Vec::new() };
        a.tracks
            .iter()
            .flatten()
            .filter_map(|aq| self.tracks.iter().find(|t| t.index == *aq))
            .flat_map(|t| t.events.iter().map(|(_, l)| l.as_slice()))
            .collect()
    }
}

/// The prop articles that carry keyframe action lists [D: e.g. MIS13A `JA_SAL`, MIS13C
/// `subMissiles`].
pub fn prop_scripts(crp: &Crp<'_>) -> Result<Vec<PropScript>> {
    let mut out = Vec::new();
    for a in crp.pool()? {
        if a.tag != tag::ARTI {
            continue;
        }
        let Ok(recs) = crp.record_window(&a) else { continue };
        let mut el: HashMap<u16, &[u8]> = HashMap::new();
        for r in &recs {
            if r.tag_code() == TAG_EL {
                el.insert(r.instance().unwrap_or(0), crp.payload(r)?);
            }
        }
        if el.is_empty() {
            continue;
        }
        let name = crp.article_name(&a)?.unwrap_or("").to_string();
        let mut s = PropScript { name, tracks: Vec::new(), anims: Vec::new() };
        for r in &recs {
            let index = r.instance().unwrap_or(0);
            if r.tag_code() == TAG_AQ {
                let p = crp.payload(r)?;
                let keys = p.len() / KEYFRAME_SIZE;
                let mut events = Vec::new();
                for k in 0..keys {
                    let w = u32_at(p, k * KEYFRAME_SIZE + 0x1c)?;
                    if w >> 16 == 0x656c {
                        events.push((k, action_list(&el, w, &format!("{} aq{index} key {k}", s.name))?));
                    }
                }
                s.tracks.push(PropTrack { index, keys, events });
            } else if r.tag_code() == TAG_AN {
                let p = crp.payload(r)?;
                let tracks = p
                    .chunks_exact(0x10)
                    .map(|e| {
                        let w = u32::from_le_bytes(e[0..4].try_into().unwrap());
                        (w >> 16 == TAG_AQ >> 16).then_some(w as u16)
                    })
                    .collect();
                s.anims.push(PropAnim { index, tracks });
            }
        }
        out.push(s);
    }
    Ok(out)
}

/// The ground height at (x, z) as the game's spawn query takes it: of the triangles containing
/// (x, z) in plan view (skipping class bit 0x04, [`CollisionTri::ground_query_skips`]) whose lowest
/// corner lies below `y + 0.5`, the one whose lowest corner is nearest under that level; the height
/// is that triangle's plane at (x, z) [M 0x2324e8 picks the smallest distance from `0x214338` /
/// `0x213ff8`: `(y + 0.5) − min corner y`, kept when > 0; 0x231a90 the plane height]. Not ported:
/// the instance flag bit 0 that measures upward instead, and the grid-cell gathering
/// (`FUN_00217588`) — every triangle is tested here.
pub fn ground_height(tris: &[CollisionTri], x: f32, y: f32, z: f32) -> Option<f32> {
    let mut best: Option<(f32, f32)> = None;
    for t in tris {
        if t.ground_query_skips() {
            continue;
        }
        let [a, b, c] = t.v;
        let den = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
        if den.abs() < 1e-9 {
            continue;
        }
        let l1 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / den;
        let l2 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / den;
        let l3 = 1.0 - l1 - l2;
        if l1 < -1e-5 || l2 < -1e-5 || l3 < -1e-5 {
            continue;
        }
        let d = (y + GROUND_SLACK) - a[1].min(b[1]).min(c[1]);
        if d > 0.0 && best.map_or(true, |(bd, _)| d < bd) {
            best = Some((d, l1 * a[1] + l2 * b[1] + l3 * c[1]));
        }
    }
    best.map(|(_, h)| h)
}

/// What the game's ground query found.
#[derive(Clone, Debug, PartialEq)]
pub struct GroundHit {
    /// The plane height of the hit triangle at the probe's (x, z) [M 0x2151b8 / 0x231a90].
    pub height: f32,
    /// The `ci` item and its article.
    pub item: usize,
    pub article: String,
    /// The world corners of the hit triangle.
    pub tri: [[f32; 3]; 3],
    /// The query's distance measure: `(local y + 0.5) − lowest local corner y` (bit 0 of the item
    /// flags: `lowest corner − (local y − 0.5)`).
    pub distance: f32,
}

/// The game's ground query (`FUN_002151b8` → `FUN_002322c8` → `FUN_00217588` gather →
/// `FUN_002324e8` → `FUN_00214338`) over the `ci` items, as traced [M]:
/// 1. gather: items whose centre `(−t·r0, −t·r2)` [M 0x212da8] is within their `+0x3c` radius of
///    (x, z) by the measure `max(|dx|, |dz|) + min(|dx|, |dz|) / 4` [M 0x217228, radius 0 from
///    0x2322c8]; per item, the groups whose centre is within `+0x0c u16 / 16` of the local point
///    in xz and which hold a triangle whose centroid is within its own `+0x2e u16 / 16` [M 0x216ec0
///    with the planar flag]. An item with no such group is dropped [M 0x217228 `iStack_cc == 0`];
/// 2. per item: the point in the item's local frame must lie within the `+0x0c` / `+0x2c` half
///    extents in x / z [M 0x214338]; per kept group, the FIRST triangle of its strip that holds
///    the point in plan view (winding rule of 0x2138b0, class bit 0x04 skipped) [M 0x2138b0];
///    distance `(ly + 0.5) − min corner y`, or with flags bit 0 `min corner y − (ly − 0.5)`; the
///    smallest distance > 0 wins, items compared by the same local measure [M 0x214338, 0x2324e8];
/// 3. the height is the hit triangle's world plane at (x, z): normal = (v1 − v0) × (v0 − v2),
///    flipped up, n.y capped at 0.9999 [M 0x2151b8, 0x231a90].
/// Not ported: the grid cells that feed the gather (`FUN_0021cf30`; every item is tested here),
/// and the world's class mask at `DAT_0033a540 + 0x2c` (taken as 0).
pub fn game_ground(insts: &[crate::crp_mesh::CollisionInstance], p: [f32; 3]) -> Option<GroundHit> {
    let f = |d: &[u8], o: usize| -> Option<f32> { Some(f32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?)) };
    let w = |d: &[u8], o: usize| -> Option<u32> { Some(u32::from_le_bytes(d.get(o..o + 4)?.try_into().ok()?)) };
    let h16 = |d: &[u8], o: usize| -> Option<f32> { Some(u16::from_le_bytes(d.get(o..o + 2)?.try_into().ok()?) as f32 * 0.0625) };
    let mut best: Option<(f32, usize, [[f32; 3]; 3])> = None;
    for (ii, inst) in insts.iter().enumerate() {
        let (cx, cz) = inst.centre_xz();
        let (ax, az) = ((cx - p[0]).abs(), (cz - p[2]).abs());
        let m = if az < ax { ax + az * 0.25 } else { az + ax * 0.25 };
        if !(m < inst.radius()) {
            continue;
        }
        let l = inst.to_local(p);
        let ca = &inst.ca;
        let groups = ca.get(0..2).map_or(0, |b| u16::from_le_bytes([b[0], b[1]]) as usize);
        let vert = |o: usize| -> Option<[f32; 3]> { Some([f(ca, o)?, f(ca, o + 4)?, f(ca, o + 8)?]) };
        // 1. the planar group prefilter [M 0x216ec0].
        let mut kept = Vec::new();
        for g in 0..groups {
            let gh = 0x20 + g * 0x10;
            let (Some(c), Some(r)) = (vert(gh), h16(ca, gh + 0x0c)) else { continue };
            let (dx, dz) = (c[0] - l[0], c[2] - l[2]);
            if !(dx * dx + dz * dz < r * r) {
                continue;
            }
            let Some(off) = ca.get(gh + 0x0e..gh + 0x10).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize) else { continue };
            let base = 0x20 + off;
            let Some(n) = w(ca, base + 0x0c) else { continue };
            let hit = (0..(n as usize).saturating_sub(2)).any(|k| {
                let o = base + k * 0x10;
                let (Some(a), Some(b), Some(c), Some(rad)) = (vert(o), vert(o + 0x10), vert(o + 0x20), h16(ca, o + 0x2e)) else { return false };
                let (mx, mz) = ((a[0] + b[0] + c[0]) * 0.333_333_34 - l[0], (a[2] + b[2] + c[2]) * 0.333_333_34 - l[2]);
                mx * mx + mz * mz < rad * rad
            });
            if hit {
                kept.push(base);
            }
        }
        if kept.is_empty() {
            continue;
        }
        // 2. extents, first containing triangle per strip, distance [M 0x214338 / 0x2138b0].
        let (hx, hz) = inst.half_extents();
        if hx < l[0] || l[0] < -hx || hz < l[2] || l[2] < -hz {
            continue;
        }
        let up = inst.flags() & 1 != 0;
        let mut ibest: Option<(f32, [[f32; 3]; 3])> = None;
        for base in kept {
            let Some(n) = w(ca, base + 0x0c) else { continue };
            let Some(sf) = w(ca, base + 0x1c) else { continue };
            let both = sf & 2 != 0;
            let mut par = sf & 1;
            for k in 0..(n as usize).saturating_sub(2) {
                let o = base + k * 0x10;
                let (Some(a), Some(b), Some(c)) = (vert(o), vert(o + 0x10), vert(o + 0x20)) else { break };
                let class = ca.get(o + 0x2d).copied().unwrap_or(0);
                let e = |u: [f32; 3], v: [f32; 3]| (u[0] - v[0]) * (l[2] - v[2]) - (l[0] - v[0]) * (u[2] - v[2]);
                let (e1, e2, e3) = (e(a, b), e(b, c), e(c, a));
                let le = e1 <= 0.0 && e2 <= 0.0 && e3 <= 0.0;
                let ge = e1 >= 0.0 && e2 >= 0.0 && e3 >= 0.0;
                let inside = class & 4 == 0 && if both { le || ge } else if par == 0 { ge } else { le };
                if inside {
                    let lo = a[1].min(b[1]).min(c[1]);
                    let d = if up { lo - (l[1] - GROUND_SLACK) } else { (l[1] + GROUND_SLACK) - lo };
                    if d > 0.0 && ibest.map_or(true, |(bd, _)| d < bd) {
                        ibest = Some((d, [a, b, c]));
                    }
                    break;
                }
                if !both {
                    par ^= 1;
                }
            }
        }
        if let Some((d, t)) = ibest {
            if best.map_or(true, |(bd, ..)| d < bd) {
                best = Some((d, ii, t));
            }
        }
    }
    let (distance, ii, lt) = best?;
    let inst = &insts[ii];
    let t = [inst.to_world(lt[0]).ok()?, inst.to_world(lt[1]).ok()?, inst.to_world(lt[2]).ok()?];
    Some(GroundHit { height: plane_height(&t, p[0], p[2]), item: inst.index, article: inst.article.clone(), tri: t, distance })
}

/// `FUN_002151b8`'s plane: normal = (v1 − v0) × (v0 − v2) normalised, negated when n.y < 0, n.y
/// capped at 0.9999; height = `v0.y − (n.x (x − v0.x) + n.z (z − v0.z)) / n.y` (`FUN_00231a90`;
/// n.y = 0 returns v0.y) [M].
pub fn plane_height(t: &[[f32; 3]; 3], x: f32, z: f32) -> f32 {
    let [v0, v1, v2] = *t;
    let a = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let b = [v0[0] - v2[0], v0[1] - v2[1], v0[2] - v2[2]];
    let c = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let mut n = if c == [0.0; 3] {
        [0.0, 1.0, 0.0]
    } else {
        let s = 1.0 / (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        [c[0] * s, c[1] * s, c[2] * s]
    };
    if n[1] < 0.0 {
        n = [-n[0], -n[1], -n[2]];
    }
    if n[1] >= 0.9999 {
        n[1] = 0.9999;
    }
    if n[1] == 0.0 {
        return v0[1];
    }
    v0[1] - (n[0] * (x - v0[0]) + n[2] * (z - v0[2])) / n[1]
/// `MSet` — the Map window's mission set record [M 0x22faf8 :118 `FUN_002d5a10(map, 'MSet')`].
pub const TAG_MSET: u32 = 0x4d53_6574;

/// The pvehicle name the player is spawned in: the C string that opens the `MSet` payload.
/// The chain [M]: `FUN_0022faf8` passes the MSet record to `FUN_00205220`, which stores its
/// payload pointer at mission+0x1c; `FUN_00204d18` copies it to mission+0x1b44 (only the names
/// `db5` / `z8` are swapped for an unlocked car by `DAT_0033a010..020`); "Spawn player car"
/// (`FUN_00176658` :207-231) takes the name from `FUN_002051f8` (mission+0x1b44, or
/// `jungle_truck` when the low-memory byte 0x1009a1 has bit 0) and builds the car; the
/// constructor's own override `DAT_0033196c` is never written in DRIVING.ELF (no store to it,
/// no pointer to it) [M FUN_001862c0 :117-126]. On the disc [D]: MIS01 helicopter, MIS11
/// vanquishsub, MIS13A jungle_truck, MIS13B ultralight, MIS13C fodbase, MIS3 supersnow, MIS4
/// vanquishalps, RACE cobra_player. None when the Map window has no MSet record.
pub fn mission_vehicle(crp: &Crp<'_>) -> Result<Option<String>> {
    let Some(map) = crp.pool()?.into_iter().find(|r| r.tag == tag::MAP) else { return Ok(None) };
    let Some(ms) = crp.record_window(&map)?.into_iter().find(|r| r.tag == TAG_MSET) else { return Ok(None) };
    let p = crp.payload(&ms)?;
    let end = p.iter().position(|&b| b == 0).unwrap_or(p.len());
    Ok(Some(String::from_utf8_lossy(&p[..end]).into_owned()))
}

/// The player start of a track `.crp`: the (last, as `FUN_0022cd30` fires them in index order)
/// trigger holding an `EStartMission` action, placed as `FUN_001fee68` places the car.
/// The ground comes from [`game_ground`] over the `CDat` items; `tris` (the track's collision
/// triangles, `Crp::extract_collision_tris`) is used only when the items fail to load. On all 8
/// tracks both give the same start height [D, `geoprim_disc`].
pub fn track_spawn(crp: &Crp<'_>, tris: &[CollisionTri]) -> Result<TrackSpawn> {
    let trig = map_triggers(crp)?;
    let t = trig
        .iter()
        .rev()
        .find(|t| t.actions.iter().any(|a| a.id == ACTION_START_MISSION))
        .ok_or_else(|| FormatError::Invalid("no Trgr record carries an EStartMission (0x16) action".into()))?;
    let a = t.actions.iter().find(|a| a.id == ACTION_START_MISSION).unwrap();
    let speed_factor = f32_at(&a.args, 4)?;
    // The traced query over the `ci` items; the flat-triangle reading only when they do not load.
    let ground = match crp.collision_instances() {
        Ok(insts) => game_ground(&insts, t.pos).map(|h| h.height),
        Err(_) => ground_height(tris, t.pos[0], t.pos[1], t.pos[2]),
    };
    let n = (t.dir[0] * t.dir[0] + t.dir[1] * t.dir[1] + t.dir[2] * t.dir[2]).sqrt();
    if !(n > 0.0) {
        return invalid(format!("trigger {}: zero start direction", t.index));
    }
    let dir = [t.dir[0] / n, t.dir[1] / n, t.dir[2] / n];
    Ok(TrackSpawn {
        trigger: t.index,
        marker: t.pos,
        pos: [t.pos[0], ground.unwrap_or(0.0) + SPAWN_LIFT, t.pos[2]],
        ground,
        dir,
        yaw: dir[0].atan2(dir[2]),
        speed_factor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri(v: [[f32; 3]; 3], class: u8) -> CollisionTri { CollisionTri { v, flags: (class as u16) << 8 } }

    #[test]
    fn ground_takes_the_nearest_surface_under_the_marker_plus_half_a_unit() {
        let flat = |y: f32| tri([[-10.0, y, -10.0], [10.0, y, -10.0], [0.0, y, 10.0]], 0);
        let tris = vec![flat(-5.0), flat(0.2), flat(3.0)];
        // The 0.2 floor lies under y 0 + 0.5; the 3.0 deck above is out; -5 is farther below.
        assert_eq!(ground_height(&tris, 0.0, 0.0, 0.0), Some(0.2));
        // A marker on the deck finds the deck.
        assert_eq!(ground_height(&tris, 0.0, 2.8, 0.0), Some(3.0));
        // A class-0x04 triangle is skipped; outside every triangle there is no ground.
        assert_eq!(ground_height(&[tri([[-10.0, 0.0, -10.0], [10.0, 0.0, -10.0], [0.0, 0.0, 10.0]], 4)], 0.0, 0.0, 0.0), None);
        assert_eq!(ground_height(&tris, 50.0, 0.0, 0.0), None);
        // A slope: the plane height at the point, chosen by its lowest corner.
        let slope = tri([[0.0, 0.0, 0.0], [10.0, 10.0, 0.0], [0.0, 0.0, 10.0]], 0);
        assert!((ground_height(&[slope], 5.0, 20.0, 1.0).unwrap() - 5.0).abs() < 1e-5);
    }

    /// A one-item collision world: identity rotation, translation `t`, flags, half extents 20, radius
    /// 100, one group holding one strip of `verts` (strip flags word `sf`, every triangle radius 100).
    fn world(verts: &[[f32; 3]], sf: u32, flags: u32, t: [f32; 3]) -> Vec<crate::crp_mesh::CollisionInstance> {
        let mut ca = vec![0u8; 0x30];
        ca[0] = 1;
        ca[0x2c..0x2e].copy_from_slice(&1600u16.to_le_bytes()); // group radius 100
        ca[0x2e..0x30].copy_from_slice(&0x10u16.to_le_bytes()); // strip at +0x20 + 0x10
        for (k, v) in verts.iter().enumerate() {
            for c in v {
                ca.extend_from_slice(&c.to_le_bytes());
            }
            let word = match k {
                0 => verts.len() as u32,
                1 => sf,
                _ => 1600 << 16,
            };
            ca.extend_from_slice(&word.to_le_bytes());
        }
        let mut item = [0u8; 64];
        let mut put = |o: usize, v: f32| item[o..o + 4].copy_from_slice(&v.to_le_bytes());
        put(0x00, 1.0);
        put(0x28, 1.0);
        put(0x0c, 20.0);
        put(0x2c, 20.0);
        put(0x3c, 100.0);
        put(0x30, t[0]);
        put(0x34, t[1]);
        put(0x38, t[2]);
        item[0x18..0x1c].copy_from_slice(&flags.to_le_bytes());
        vec![crate::crp_mesh::CollisionInstance { index: 0, article: "a".into(), item, ca }]
    }

    #[test]
    fn the_game_query_takes_the_first_triangle_of_a_strip_not_the_nearest() {
        // Triangle 0 (v0..v2) is a slope from -6 up to -1; triangle 1 (v1..v3) is flat at -1. Both
        // hold (0, 0) in plan view. Nearest-corner over all triangles picks the flat one (-1); the
        // game stops at the first triangle of the strip that holds the point [M 0x2138b0].
        let v = [[-10.0, -6.0, -10.0], [10.0, -1.0, -10.0], [0.0, -1.0, 10.0], [-12.0, -1.0, 8.0]];
        let w = world(&v, 2, 0, [0.0; 3]);
        let h = game_ground(&w, [0.0, 0.0, 0.0]).unwrap();
        let want = plane_height(&[v[0], v[1], v[2]], 0.0, 0.0);
        assert!((h.height - want).abs() < 1e-4 && (want + 1.0).abs() > 0.5, "{} vs {want}", h.height);
        assert!((h.distance - 6.5).abs() < 1e-5);
        let flat: Vec<CollisionTri> = (0..2).map(|k| tri([v[k], v[k + 1], v[k + 2]], 0)).collect();
        assert_eq!(ground_height(&flat, 0.0, 0.0, 0.0), Some(-1.0), "the all-triangle reading differs");
    }

    #[test]
    fn the_game_query_works_in_the_items_frame_with_its_extents_and_up_flag() {
        let v = [[-10.0, 5.0, -10.0], [10.0, 5.0, -10.0], [0.0, 5.0, 10.0]];
        // Translation +5 in y: local y 5 is world y 0 (local = world + t).
        let w = world(&v, 2, 0, [0.0, 5.0, 0.0]);
        assert_eq!(game_ground(&w, [0.0, 0.2, 0.0]).map(|h| h.height), Some(0.0));
        // Above the probe + 0.5: not ground.
        assert_eq!(game_ground(&w, [0.0, -2.0, 0.0]), None);
        // Flag bit 0: the query measures upward, so the floor 2 m above is found.
        let up = world(&v, 2, 1, [0.0, 5.0, 0.0]);
        assert_eq!(game_ground(&up, [0.0, -2.0, 0.0]).map(|h| h.height), Some(0.0));
        assert_eq!(game_ground(&up, [0.0, 1.0, 0.0]).map(|h| h.height), None);
        // Outside the +0x0c x half extent (20) the item is not tested, though its triangle is under.
        let wide = world(&[[-40.0, 5.0, -10.0], [40.0, 5.0, -10.0], [0.0, 5.0, 30.0]], 2, 0, [0.0, 5.0, 0.0]);
        assert!(game_ground(&wide, [19.0, 0.2, 0.0]).is_some());
        assert_eq!(game_ground(&wide, [21.0, 0.2, 0.0]), None);
        // One-sided strip: start winding 0 accepts the (all edges >= 0) order only.
        let cw = world(&v, 0, 0, [0.0, 5.0, 0.0]);
        let ccw = world(&[v[0], v[2], v[1]], 0, 0, [0.0, 5.0, 0.0]);
        assert_eq!(cw.len() + ccw.len(), 2);
        assert_ne!(game_ground(&cw, [0.0, 0.2, 0.0]).is_some(), game_ground(&ccw, [0.0, 0.2, 0.0]).is_some());
    }
}
