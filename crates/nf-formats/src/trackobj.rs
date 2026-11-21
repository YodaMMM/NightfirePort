//! Driving-track objects beyond the static world: the dynamic scene objects of `Map in#0`, their
//! `ps#0` instance records, the trigger actions that wake, hide or show them, the effect-only and
//! collision-only article parts the geometry decoder leaves out, and the `smackable` attribute
//! files that give a knocked-away prop its mass and hit points (docs/formats/trackobj.md).
//!
//! Evidence (DRIVING.ELF, EE addresses):
//! - "Init World" `FUN_0022faf8` 0x22faf8 walks `in#0`. An item with flag `0x10` whose `ps#0`
//!   record (`+0x0e` index, 0x20 bytes) has `+7 != 0xff` becomes a scene object: a 0x40-byte
//!   `RSceneObj` (`FUN_001c7098`, alloc label "RSceneObj" 0x39b3e0) built from the item's
//!   article (`FUN_001c7658`) with the item matrix (`FUN_001c9258`); the record's `+8` is set to
//!   it. Flag `0x20` also gives it a `WTargetable` (`FUN_001c8998`). `ps` `+4` u32 `>> 8` bit 4
//!   registers it with the sim at once (`FUN_002015d8`) unless the model's `+0x28` is 0x696 [M].
//!   Every other item stays in the static world, whose part effects (`as` `+4` → `ef` list) get
//!   flag 1 when `+0x10` bit 0x10 is set or the type is 2, and type 6 starts the effect named
//!   at `+0x18` (`FUN_002330b0` → the "GFXParticleSystems" manager `DAT_0033aad0`) [M].
//! - `FUN_001c7098`: the scene object is drawn while `+0x18` bit 24 is set (set by the ctor;
//!   `FUN_001c8b78` clears it, `FUN_001c8b90` sets it; the draw `FUN_001c8c70` tests it) and is
//!   put in the world grid with a radius from item word 15: `(w & 0x3ff) × {0.25, 16.0}[bit 30]`
//!   (table 0x386b80), 10.0 when that is 0 [M].
//! - The trigger actions (creator table 0x36fda0, `FUN_0022c890`): 0x17 `FUN_0014f2c8` hides the
//!   item's scene object (`FUN_0014f218`), 0x18 `FUN_00159be8` shows it (`FUN_00159b38`), 0x95
//!   `FUN_0014b6e0` hides it and takes it out of the sim (`FUN_0014b630` → `FUN_00201690`),
//!   0x96 `FUN_0014bc28` shows it and puts it back (`FUN_0014bb58` → `FUN_002015d8`), 0x23
//!   `FUN_0015e4b0` queues an `EWakeupSmackable` (`FUN_0015dda0`, run by `FUN_0015de48`). The
//!   argument block's `+4` (`+8` for 0x23, with the `ps` record at `+0xc`) holds an `sr`
//!   reference whose string is `CARP::<<Map>>::{in  0000}::<byte offset>` [M + D].
//! - `EWakeupSmackable` (`FUN_0015de48`): `ps` behaviour bit 2 spawns an `ASmackable`
//!   (`FUN_001ffa90` → `FUN_0019e100`), bit 0x40 takes the other path (`FUN_002002e0`) [M].
//!   `ASmackable` reads its attributes from namespace "smackable" (`FUN_001696f0`, "default"
//!   when the scene object names none); hit points = `HITPOINTS × (1 − ps[+4] / 256)` [M
//!   0x19e100, `FUN_00230c48` finds the record]. At most 36 run at once: a new one retires the
//!   active smackable farthest from it (`FUN_002012d0`, cap 0x24; `FUN_0019e610`) [M].

use std::collections::HashMap;

use crate::buf::{invalid, FormatError, Result};
use crate::crp::{tag, Crp, Record};
use crate::geoprim::{decode_placement_item, parse_part_ref, PLACEMENT_SIZE, TAG_AS, TAG_IN, TAG_SR};
use crate::trackstart::map_triggers;

/// `ps  ` — the Map's instance-record array [M 0x22faf8: `FUN_002d5920(map, 0x70732020)`].
pub const TAG_PS: u32 = 0x7073_2020;
/// One `ps#0` record [M 0x22faf8: `index * 0x20`].
pub const PS_SIZE: usize = 0x20;
/// `ef  ` — an article's effect list (0x40-byte entries) [M 0x22faf8 static path, D].
pub const TAG_EF: u32 = 0x6566_2020;
/// `ca  ` — an article's collision piece [D: the only geometry of the `{as  ffff}` articles].
pub const TAG_CA: u32 = 0x6361_2020;
/// One effect entry [M 0x22faf8: `iVar3 + 0x40`, next type byte at `+0x54`].
pub const EF_SIZE: usize = 0x40;

/// `in#0` item flag bits (+0x0c) [M].
pub mod flag {
    /// A scene object (dynamic instance) [M 0x22faf8, 0x1b1988, 0x230c48].
    pub const DYNAMIC: u8 = 0x10;
    /// Also gets a `WTargetable` [M 0x22faf8 → 0x1c8998].
    pub const TARGETABLE: u8 = 0x20;
    /// Put in the world grid after the model load and `+0x18` bit 26 set [M 0x1c7940].
    pub const GRIDDED: u8 = 0x08;
}

/// `ps` `+4` u32 `>> 8` bits [M].
pub mod behaviour {
    /// `EWakeupSmackable` spawns an `ASmackable` [M 0x15de48 `uVar5 & 2`].
    pub const SMACKABLE: u32 = 0x2;
    /// Registered with the sim at world init [M 0x22faf8 `uVar10 & 4` → 0x2015d8].
    pub const SIM_AT_INIT: u32 = 0x4;
    /// The wake takes the `FUN_002002e0` path instead [M 0x15de48 `uVar5 & 0x40`].
    pub const ALT_WAKE: u32 = 0x40;
}

/// Trigger action ids that act on placements (creator table 0x36fda0) [M].
pub mod action {
    pub const HIDE: u32 = 0x17;
    pub const SHOW: u32 = 0x18;
    pub const WAKE_SMACKABLE: u32 = 0x23;
    pub const DISABLE: u32 = 0x95;
    pub const ENABLE: u32 = 0x96;
}

/// The `ASmackable` pool cap [M 0x2012d0: `0x24` with the third argument 0, as 0x1ffa90 passes].
pub const SMACKABLE_CAP: usize = 0x24;

/// One `ps#0` record [M 0x22faf8 / 0x1b1988 / 0x15de48 / 0x19e100].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceRecord {
    /// `+0`: 2 = a path-following object (`FUN_001b1988` builds those) [M]; 1 on the props [D].
    pub kind: u8,
    /// `+4` byte: damage already taken, in 1/256 of the hit points [M 0x19e100].
    pub damage: u8,
    /// `+4` u32 `>> 8`: the [`behaviour`] bits [M].
    pub behaviour: u32,
    /// `+7`: 0xff = no scene object; otherwise passed to the model load (`FUN_001c7658`) [M].
    pub scene_part: u8,
}

impl InstanceRecord {
    pub fn parse(r: &[u8]) -> Result<InstanceRecord> {
        if r.len() < PS_SIZE {
            return invalid(format!("ps record {} < 0x20 bytes", r.len()));
        }
        let w = u32::from_le_bytes(r[4..8].try_into().unwrap());
        Ok(InstanceRecord { kind: r[0], damage: r[4], behaviour: w >> 8, scene_part: r[7] })
    }
}

/// The grid radius of a scene object from item word 15 [M 0x1c7098, table 0x386b80 = {0.25, 16.0}].
pub fn cull_radius(word15: u32) -> f32 {
    let r = (word15 & 0x3ff) as f32 * if word15 & 0x4000_0000 != 0 { 16.0 } else { 0.25 };
    if r == 0.0 { 10.0 } else { r }
}

/// One `in#0` item with flag 0x10 [M 0x22faf8].
#[derive(Clone, Debug, PartialEq)]
pub struct TrackObject {
    /// The `in#0` index.
    pub index: usize,
    /// The `CARP::<article>::{as  NNNN}` article and part, when the reference is one.
    pub article: Option<(String, u16)>,
    pub flags: u8,
    /// `+0x0e`: the `ps#0` index.
    pub ps: u16,
    pub record: InstanceRecord,
    pub origin: [f32; 3],
    /// [`cull_radius`] of word 15.
    pub radius: f32,
}

impl TrackObject {
    /// Built as a scene object at world init [M 0x22faf8: flag 0x10, `ps+7 != 0xff`, a reference].
    pub fn scene_object(&self) -> bool { self.record.scene_part != 0xff }
    pub fn smackable(&self) -> bool { self.record.behaviour & behaviour::SMACKABLE != 0 }
    pub fn targetable(&self) -> bool { self.flags & flag::TARGETABLE != 0 }
}

fn u32_at(d: &[u8], o: usize) -> Result<u32> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| FormatError::Invalid(format!("u32 at {o:#x} past the end ({})", d.len())))
}

fn cstr(p: &[u8]) -> String {
    let n = p.iter().position(|&b| b == 0).unwrap_or(p.len());
    String::from_utf8_lossy(&p[..n]).into_owned()
}

struct MapWindow {
    sr: HashMap<u16, String>,
    items: Vec<[u8; PLACEMENT_SIZE]>,
    ps: Vec<[u8; PS_SIZE]>,
}

fn map_window(crp: &Crp<'_>) -> Result<MapWindow> {
    let map = crp
        .pool()?
        .into_iter()
        .find(|r| r.tag == tag::MAP)
        .ok_or_else(|| FormatError::Invalid("no Map article in the pool".into()))?;
    let (s, e) = map.window(crp.data)?;
    let mut sr = HashMap::new();
    let (mut inr, mut psr): (Option<Record>, Option<Record>) = (None, None);
    for r in crp.window_records(s, e)? {
        let c = r.tag_code();
        if c == TAG_SR {
            sr.insert(r.instance().unwrap_or(0), cstr(crp.payload(&r)?));
        } else if c == TAG_IN && r.instance() == Some(0) && inr.is_none() {
            inr = Some(r);
        } else if c == TAG_PS && r.instance() == Some(0) && psr.is_none() {
            psr = Some(r);
        }
    }
    let arr = |r: Option<Record>, size: usize| -> Result<Vec<u8>> {
        let Some(r) = r else { return Ok(Vec::new()) };
        let at = r.data_addr(crp.data)?;
        let end = at + r.count.max(0) as usize * size;
        crp.data.get(at..end).map(|b| b.to_vec()).ok_or_else(|| FormatError::Invalid(format!("Map array at {at:#x} runs past the file")))
    };
    let items = arr(inr, PLACEMENT_SIZE)?.chunks_exact(PLACEMENT_SIZE).map(|c| c.try_into().unwrap()).collect();
    let ps = arr(psr, PS_SIZE)?.chunks_exact(PS_SIZE).map(|c| c.try_into().unwrap()).collect();
    Ok(MapWindow { sr, items, ps })
}

/// Every `in#0` item with flag 0x10 and its `ps#0` record [M 0x22faf8 + D].
pub fn track_objects(crp: &Crp<'_>) -> Result<Vec<TrackObject>> {
    let w = map_window(crp)?;
    let mut out = Vec::new();
    for (index, it) in w.items.iter().enumerate() {
        let (_, origin, flags, refw) = decode_placement_item(it)?;
        if flags & flag::DYNAMIC == 0 {
            continue;
        }
        let ps = u16::from_le_bytes([it[0x0e], it[0x0f]]);
        let rec = w
            .ps
            .get(ps as usize)
            .ok_or_else(|| FormatError::Invalid(format!("in#0 item {index}: ps index {ps} past {}", w.ps.len())))?;
        let article = (refw >> 16 == 0x7372)
            .then(|| w.sr.get(&(refw as u16)))
            .flatten()
            .and_then(|s| parse_part_ref(s))
            .map(|(a, p)| (a.to_owned(), p));
        out.push(TrackObject {
            index,
            article,
            flags,
            ps,
            record: InstanceRecord::parse(rec)?,
            origin,
            radius: cull_radius(u32_at(it, 0x3c)?),
        });
    }
    Ok(out)
}

/// `CARP::<<Map>>::{in  0000}::<byte offset>` (or `{ps  0000}`) → (tag code, byte offset) [D].
pub fn parse_map_ref(s: &str) -> Option<(u32, usize)> {
    let rest = s.strip_prefix("CARP::<<Map>>::{")?;
    let (code, tail) = match rest.get(..2)? {
        "in" => (TAG_IN, rest.get(2..)?),
        "ps" => (TAG_PS, rest.get(2..)?),
        _ => return None,
    };
    let off = tail.strip_prefix("  0000}::")?;
    Some((code, off.parse().ok()?))
}

/// What a trigger action does to a placement [M, see the module docs].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectAction {
    /// 0x23: wake the smackable (`in#0` item, `ps#0` record).
    Wake { placement: usize, ps: usize },
    /// 0x17: hide the scene object.
    Hide { placement: usize },
    /// 0x18: show it.
    Show { placement: usize },
    /// 0x95: hide it and take it out of the sim.
    Disable { placement: usize },
    /// 0x96: show it and put it back in the sim.
    Enable { placement: usize },
}

impl ObjectAction {
    pub fn placement(&self) -> usize {
        match *self {
            ObjectAction::Wake { placement, .. }
            | ObjectAction::Hide { placement }
            | ObjectAction::Show { placement }
            | ObjectAction::Disable { placement }
            | ObjectAction::Enable { placement } => placement,
        }
    }
}

/// One placement action and the `Trgr` index that carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerObjectAction {
    pub trigger: usize,
    pub action: ObjectAction,
}

/// Every placement-acting action of the `Map` triggers [M creator table 0x36fda0 + D the refs].
pub fn object_actions(crp: &Crp<'_>) -> Result<Vec<TriggerObjectAction>> {
    let w = map_window(crp)?;
    // A null reference makes the action a no-op [M 0x14f218 / 0x159b38 / 0x14b630 / 0x14bb58:
    // `if (puVar[1] != 0)`]; MIS11 trigger 32 carries one [D].
    let resolve = |args: &[u8], at: usize, code: u32, size: usize, t: usize| -> Result<Option<usize>> {
        let refw = u32_at(args, at)?;
        if refw == 0 {
            return Ok(None);
        }
        let s = (refw >> 16 == 0x7372).then(|| w.sr.get(&(refw as u16))).flatten();
        match s.and_then(|s| parse_map_ref(s)) {
            Some((c, off)) if c == code && off % size == 0 && off / size < w.items.len().max(w.ps.len()) => Ok(Some(off / size)),
            _ => invalid(format!("trigger {t}: action argument +{at:#x} {refw:#010x} ({s:?}) is not a Map {code:#x} reference")),
        }
    };
    let mut out = Vec::new();
    for t in map_triggers(crp)? {
        for a in &t.actions {
            let item = |at| resolve(&a.args, at, TAG_IN, PLACEMENT_SIZE, t.index);
            let action = match a.id {
                action::WAKE_SMACKABLE => {
                    let (Some(placement), Some(ps)) = (item(8)?, resolve(&a.args, 0xc, TAG_PS, PS_SIZE, t.index)?) else { continue };
                    ObjectAction::Wake { placement, ps }
                }
                action::HIDE => match item(4)? { Some(placement) => ObjectAction::Hide { placement }, None => continue },
                action::SHOW => match item(4)? { Some(placement) => ObjectAction::Show { placement }, None => continue },
                action::DISABLE => match item(4)? { Some(placement) => ObjectAction::Disable { placement }, None => continue },
                action::ENABLE => match item(4)? { Some(placement) => ObjectAction::Enable { placement }, None => continue },
                _ => continue,
            };
            out.push(TriggerObjectAction { trigger: t.index, action });
        }
    }
    Ok(out)
}

/// One `ef` entry of an article part [M 0x22faf8 static path; D].
#[derive(Clone, Debug, PartialEq)]
pub struct PartEffect {
    /// `+0x14`: 2 and 6 are tested by Init World; 6 with a name starts a particle system [M].
    pub kind: u8,
    /// `+0x10` u32 (bit 0x10 tested at Init World) [M].
    pub flags: u32,
    /// `+0x18` `sr` string when present (the particle system name for kind 6) [D].
    pub name: Option<String>,
}

/// What an article part holds.
#[derive(Clone, Debug, PartialEq)]
pub enum PartKind {
    /// The `as` payload ends in an `sr` model reference (decoded by geoprim).
    Model,
    /// No model; the `as` payload's `+4` names an `ef` list [D].
    Effect(Vec<PartEffect>),
    /// `{as  ffff}`: the article has no `as` parts and only `ca` collision pieces [D].
    CollisionOnly,
    /// Anything else.
    Other,
}

/// The kind of every `(article, part)` a `Map in#0` item names [D].
pub fn placement_part_kinds(crp: &Crp<'_>) -> Result<HashMap<(String, u16), PartKind>> {
    let w = map_window(crp)?;
    let mut wanted: HashMap<String, Vec<u16>> = HashMap::new();
    for it in &w.items {
        let refw = u32_at(it, 0x1c)?;
        if refw >> 16 != 0x7372 {
            continue;
        }
        if let Some((a, p)) = w.sr.get(&(refw as u16)).and_then(|s| parse_part_ref(s)) {
            wanted.entry(a.to_owned()).or_default().push(p);
        }
    }
    let mut out = HashMap::new();
    for arti in crp.pool()? {
        if arti.tag != tag::ARTI {
            continue;
        }
        let Some(name) = crp.article_name(&arti)? else { continue };
        let Some(parts) = wanted.get(name) else { continue };
        let (s, e) = arti.window(crp.data)?;
        let recs = crp.window_records(s, e)?;
        let mut sr: HashMap<u16, String> = HashMap::new();
        let mut ef: HashMap<u16, &[u8]> = HashMap::new();
        let mut asp: HashMap<u16, &[u8]> = HashMap::new();
        let mut has_ca = false;
        for r in &recs {
            match r.tag_code() {
                TAG_SR => {
                    sr.insert(r.instance().unwrap_or(0), cstr(crp.payload(r)?));
                }
                TAG_EF => {
                    ef.insert(r.instance().unwrap_or(0), crp.payload(r)?);
                }
                TAG_AS => {
                    asp.insert(r.instance().unwrap_or(0), crp.payload(r)?);
                }
                TAG_CA => has_ca = true,
                _ => {}
            }
        }
        for &part in parts {
            let kind = match asp.get(&part) {
                None if part == 0xffff && asp.is_empty() && has_ca => PartKind::CollisionOnly,
                None => PartKind::Other,
                Some(p) if p.len() >= 4 && &p[p.len() - 2..] == b"rs" => PartKind::Model,
                Some(p) if p.len() >= 8 && &p[6..8] == b"fe" => {
                    let idx = u16::from_le_bytes([p[4], p[5]]);
                    match ef.get(&idx) {
                        Some(list) => {
                            let mut effects = Vec::new();
                            for e in list.chunks_exact(EF_SIZE) {
                                let kind = e[0x14];
                                if kind == 0 {
                                    break;
                                }
                                let refw = u32_at(e, 0x18)?;
                                let name = (refw >> 16 == 0x7372).then(|| sr.get(&(refw as u16)).cloned()).flatten();
                                effects.push(PartEffect { kind, flags: u32_at(e, 0x10)?, name });
                            }
                            PartKind::Effect(effects)
                        }
                        None => PartKind::Other,
                    }
                }
                Some(_) => PartKind::Other,
            };
            out.insert((name.to_owned(), part), kind);
        }
    }
    Ok(out)
}

/// The `PHYS::<name>` string an article's window carries, by article name [D].
pub fn article_phys_names(crp: &Crp<'_>) -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    for arti in crp.pool()? {
        if arti.tag != tag::ARTI {
            continue;
        }
        let Some(name) = crp.article_name(&arti)? else { continue };
        let (s, e) = arti.window(crp.data)?;
        for r in crp.window_records(s, e)? {
            if r.tag_code() == TAG_SR {
                let v = cstr(crp.payload(&r)?);
                if let Some(p) = v.strip_prefix("PHYS::") {
                    out.insert(name.to_owned(), p.to_owned());
                }
            }
        }
    }
    Ok(out)
}

/// A `data\sim\attrib\smackable\*.atr` file: the `PhysicsData` schema registered by
/// `FUN_00194e18` 0x194e18 (namespace "smackable", type "PhysicsData") with the defaults of
/// `FUN_00194cd0` 0x194cd0 [M].
#[derive(Clone, Debug, PartialEq)]
pub struct SmackableAttrib {
    /// The `[Section]` name.
    pub section: String,
    /// `MASS` (+0x04, float; default 100.0).
    pub mass: f32,
    /// `HITPOINTS` (+0x08, float; default 500.0).
    pub hit_points: f32,
    /// `FORCE_DETACH` (+0x0c, bool; default false).
    pub force_detach: bool,
    /// `MOMENT` (registered at +0x0c too, as a vector of up to 4 floats) — see the docs: the
    /// two registrations share the offset; which value the game keeps is not traced.
    pub moment: Option<[f32; 3]>,
    pub default_sound: Option<String>,
    pub warning_sound: Option<String>,
    pub impact_sound: [Option<String>; 3],
    pub scrape_sound: Option<String>,
}

impl Default for SmackableAttrib {
    fn default() -> Self {
        SmackableAttrib {
            section: String::new(),
            mass: 100.0,
            hit_points: 500.0,
            force_detach: false,
            moment: None,
            default_sound: None,
            warning_sound: None,
            impact_sound: [None, None, None],
            scrape_sound: None,
        }
    }
}

impl SmackableAttrib {
    /// Parses the INI text. Keys compare case-insensitively (`Mass=` appears on the disc) [D];
    /// that the game's attribute parser does the same is not traced.
    pub fn parse(text: &str) -> SmackableAttrib {
        let mut a = SmackableAttrib::default();
        for line in text.lines() {
            let line = line.split("//").next().unwrap_or("").trim();
            if let Some(s) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                a.section = s.to_owned();
                continue;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim().to_ascii_uppercase(), v.trim());
            let num = |v: &str| v.parse::<f32>().ok();
            match k.as_str() {
                "MASS" => a.mass = num(v).unwrap_or(a.mass),
                "HITPOINTS" => a.hit_points = num(v).unwrap_or(a.hit_points),
                "FORCE_DETACH" => a.force_detach = matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
                "MOMENT" => {
                    let f: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                    if f.len() >= 3 {
                        a.moment = Some([f[0], f[1], f[2]]);
                    }
                }
                "DEFAULTSOUND" => a.default_sound = Some(v.to_owned()),
                "WARNINGSOUND" => a.warning_sound = Some(v.to_owned()),
                "IMPACTSOUNDLO" => a.impact_sound[0] = Some(v.to_owned()),
                "IMPACTSOUNDMED" => a.impact_sound[1] = Some(v.to_owned()),
                "IMPACTSOUNDHI" => a.impact_sound[2] = Some(v.to_owned()),
                "SCRAPESOUND" => a.scrape_sound = Some(v.to_owned()),
                _ => {}
            }
        }
        a
    }

    /// The hit points an `ASmackable` starts with: `HITPOINTS − damage/256 × HITPOINTS`
    /// [M 0x19e100: `+0x74 = attr+8 − ps[+4] × 0.00390625 × attr+8`].
    pub fn start_hit_points(&self, damage: u8) -> f32 { self.hit_points - damage as f32 * 0.003_906_25 * self.hit_points }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cull_radius_uses_the_quarter_or_sixteen_table_and_ten_for_zero() {
        // RACE's POS2_Cautionguard word 15 is 0x00b02c0b [D] → 11 × 0.25.
        assert_eq!(cull_radius(0x00b0_2c0b), 2.75);
        assert_eq!(cull_radius(0x4000_0003), 48.0);
        assert_eq!(cull_radius(0x0000_0400), 10.0);
    }

    #[test]
    fn instance_record_splits_kind_damage_behaviour_and_scene_part() {
        let mut r = [0u8; 32];
        r[..8].copy_from_slice(&[0x01, 0xff, 0, 0, 0x00, 0x03, 0x02, 0x00]);
        let i = InstanceRecord::parse(&r).unwrap();
        assert_eq!((i.kind, i.damage, i.behaviour, i.scene_part), (1, 0, 0x203, 0));
        assert!(i.behaviour & behaviour::SMACKABLE != 0 && i.behaviour & behaviour::SIM_AT_INIT == 0);
        r[7] = 0xff;
        assert_eq!(InstanceRecord::parse(&r).unwrap().scene_part, 0xff);
        assert!(InstanceRecord::parse(&r[..31]).is_err());
    }

    #[test]
    fn map_refs_parse_to_byte_offsets() {
        assert_eq!(parse_map_ref("CARP::<<Map>>::{in  0000}::202816"), Some((TAG_IN, 202816)));
        assert_eq!(parse_map_ref("CARP::<<Map>>::{ps  0000}::352"), Some((TAG_PS, 352)));
        assert_eq!(parse_map_ref("CARP::<<Map>>::{AISp}::0"), None);
        assert_eq!(parse_map_ref("CARP::POS2_Cautionguard::{as  0000}"), None);
    }

    #[test]
    fn smackable_attrib_defaults_and_keys() {
        let d = SmackableAttrib::parse("");
        assert_eq!((d.mass, d.hit_points, d.force_detach), (100.0, 500.0, false));
        let a = SmackableAttrib::parse("[Glass]\r\nMASS=10\r\nHITPOINTS=5\r\nFORCE_DETACH=1\r\nMOMENT=1.0,1.0,1.0\r\nIMPACTSOUNDLO=SFX_GlassShatter1\r\n");
        assert_eq!(a.section, "Glass");
        assert_eq!((a.mass, a.hit_points, a.force_detach), (10.0, 5.0, true));
        assert_eq!(a.moment, Some([1.0, 1.0, 1.0]));
        assert_eq!(a.impact_sound[0].as_deref(), Some("SFX_GlassShatter1"));
        assert_eq!(SmackableAttrib::parse("[X]\nMass=900\n").mass, 900.0);
        assert_eq!(a.start_hit_points(0), 5.0);
        assert_eq!(a.start_hit_points(128), 2.5);
    }
}
