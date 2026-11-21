//! Driving tracks: the effects placed on article parts (`ef` lists), the Giotto effect library
//! they name (`GIOT::<name>` → a `GANM` of the track's effects gallery), and the decal styles of
//! `data\render\decal.dat`. docs/systems/driving-effects.md §7 is the write-up.
//!
//! Evidence (DRIVING.ELF, EE addresses; [M] decompile/disassembly, [D] disc bytes):
//! - An article part's `as` payload is `{'Base' ref, 'ef' ref, 'an' ref, ...}`: Init World
//!   `FUN_0022faf8` reads the effect list through the item reference's `+4` word, and the model
//!   through its `+0` [M]. The list is 0x40-byte entries, ended by a zero type byte at `+0x14`
//!   (`iVar3 + 0x54`, then `+ 0x40`) [M]. Every part of the 8 tracks with an `ef` reference has a
//!   list, model parts included (the traffic light's lamp models carry their glare) [D].
//! - Static items (no in#0 flag 0x10): Init World sets entry flag 1 when `+0x10` has bit 0x10
//!   or the type is 2; with bit 0x10, type 6 starts the effect named by `+0x18`
//!   (`FUN_002330b0`) [M]. The static draw `FUN_001ad1a0` walks the flagged entries: type 2 →
//!   glare `FUN_001f1d18(lod_distance², glare manager, entry, item matrix)` when DAT_00333784
//!   is set; type 6 → `FUN_00233190` (an empty function: the effect keeps its start position);
//!   any other type hits the error print `FUN_002a8148(0x381150)` [M].
//! - Scene objects (flag 0x10): `FUN_001ca930` walks the effects whose bit is set in the
//!   object's mask; `+0x15` is the model part whose matrix is used (`FUN_001c9368`), `+0x16` a
//!   bit of the object's `+0x1c` part mask that hides the effect unless flag 0x2000 is set;
//!   `FUN_001cab38` then runs type 2 as a glare with distance 0 (never culled by distance),
//!   type 5 as a point light (`FUN_001af8c0`, `RLightManager` + 0x2e0), types 6 and 8 as no-ops
//!   [M]. Type 6 on a scene object starts on object events (`FUN_001ca198`), not at load.
//! - The glare entry (type 2), read by `FUN_001f1d18` and `FUN_001f1b60` [M]: `+0x00` position
//!   (item-local), `+0x10` flags (0x400 pulse, 0x800 triangle pulse, 0x4000 with 0x800: the
//!   ramp-hold-ramp pulse, 0x20 one-sided), `+0x18` glare type (0..79, `Render\Glare` tuning),
//!   `+0x1c` size, `+0x20` facing direction (item-local), `+0x2c` pulse phase (ticks), `+0x30`
//!   rate, `+0x34` duty, `+0x38` base, `+0x3c` amplitude.
//! - The particle entry (type 6): `+0x18` `sr` reference `GIOT::<name>`, `+0x20` a quaternion
//!   (stored in the instance, unused by the static position path) [M `FUN_00232af8`,
//!   `FUN_00234d80`, `FUN_00233460`]. The instance position is the item origin + the entry's
//!   `+0x00` offset, unrotated, and its orientation the identity (0x355ba0 = (0, 0, 0, 1) [D])
//!   [M `FUN_00233460` no-object branch].
//! - Giotto: the manager `FUN_00233618` loads one gallery chosen by `FUN_0024b7d0`:
//!   `sav_gall1.gal` for the tracks named snow1a_mis3 / snow2a_mis4 / snow1a / snow2a,
//!   `ears_gall1.gal` for junglea/b/c (with or without `_mis13x`), else `gall1.gal` (so RACE,
//!   `snow2a_race`, uses gall1) [M + D strings 0x3ac580-0x3ac718]. A name resolves to the root
//!   child whose `Name` matches case-insensitively (`FUN_002339e8` → `FUN_002a9198`) [M].
//!   The instance runs its elements for frames 0..=LIFE (`FUN_002345d8`): `gp` = a particle
//!   emitter (`FUN_002360c0`), `gs` a sound, `gg` a glare, `gd` a decal burst (`FUN_00235810`,
//!   once) [M `FUN_00234818`]. Key set `gk`#0x3000 (`FUN_002339c8(3, 0)`) [M].
//! - A `gp` key: `+0x00` offset (rotated by the instance orientation, then added to its
//!   position), `+0x10` a quaternion for the emitter axes (rotating (0,1,0) → main axis, (1,0,0)
//!   → second axis), `+0x30` a 0xb0-byte particle record (`drivefx::ParticleType`) that
//!   `FUN_00235ed0` copies, scales (`+0x4c` and `+0x50` × 127, `+0x54` × 360) and adds to the
//!   particle library under the key `n + 5` (n = types added so far), once per key; `+0xd8` the
//!   start frame, honoured only when `+0xdc` is 0 [M `FUN_002360c0`].

use std::collections::HashMap;

use crate::buf::{FormatError, Result};
use crate::crp::{tag, Crp};
use crate::drivefx::{Decal, ParticleType, PARTICLE_TYPE_SIZE};
use crate::gal::{Anim, Gal};
use crate::geoprim::{map_item_word15, map_placements, Placement, PLACEMENT_SIZE, TAG_AS, TAG_IN, TAG_SR};
use crate::trackobj::{track_objects, EF_SIZE, TAG_EF};

/// `+0x14` type bytes the game tests [M `FUN_0022faf8`, `FUN_001ad1a0`, `FUN_001cab38`].
pub mod kind {
    /// A glare sprite (`FUN_001f1d18`).
    pub const GLARE: u8 = 2;
    /// A point light (`FUN_001af8c0`); scene objects only.
    pub const LIGHT: u8 = 5;
    /// A Giotto effect named by `+0x18`.
    pub const GIOTTO: u8 = 6;
    /// Accepted by `FUN_001cab38` and does nothing.
    pub const NOOP: u8 = 8;
}

/// `+0x10` flag bits [M].
pub mod eflag {
    /// Set by Init World on the entries the static draw walks.
    pub const ACTIVE: u32 = 0x01;
    /// Glare: one-sided, faded by the angle to the camera (`FUN_001f1d18`).
    pub const ONE_SIDED: u32 = 0x20;
    /// Static items: start at load (type 6) / draw (any type) [M 0x22faf8].
    pub const AT_LOAD: u32 = 0x10;
    /// Glare: multiply the size by the pulse (`FUN_001f1b60`).
    pub const PULSE: u32 = 0x400;
    /// Pulse shape: a triangle instead of a flat on-period.
    pub const TRIANGLE: u32 = 0x800;
    /// With `TRIANGLE`: ramp up over 0..0.4, a dip around 0.5, ramp down after 0.6.
    pub const SHAPED: u32 = 0x4000;
    /// Scene objects: draw even when the `+0x16` part bit is set [M 0x1ca930].
    pub const IGNORE_PART_BIT: u32 = 0x2000;
}

fn f(b: &[u8], o: usize) -> f32 { f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) }
fn u(b: &[u8], o: usize) -> u32 { u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) }

/// One 0x40-byte `ef` entry, every field the traced readers use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EfEntry {
    /// +0x00: position in the item's (or the part's) frame.
    pub pos: [f32; 3],
    /// +0x10: flags ([`eflag`]).
    pub flags: u32,
    /// +0x14: type ([`kind`]).
    pub kind: u8,
    /// +0x15: scene objects: the model part whose matrix places the effect.
    pub part: u8,
    /// +0x16: scene objects: the bit of the object's part mask that hides it.
    pub part_bit: u8,
    /// +0x17: not read by the traced code.
    pub b17: u8,
    /// +0x18: glare type (type 2) or the `sr` reference word of the Giotto name (type 6).
    pub word18: u32,
    /// +0x1c: glare size.
    pub size: f32,
    /// +0x20..+0x2c: glare facing direction (type 2) / orientation quaternion (type 6).
    pub dir: [f32; 4],
    /// +0x2c: pulse phase in ticks (type 2; it overlaps `dir[3]`).
    pub phase: u32,
    /// +0x30..+0x3c: pulse rate, duty, base, amplitude.
    pub rate: f32,
    pub duty: f32,
    pub base: f32,
    pub amp: f32,
}

impl EfEntry {
    pub fn parse(e: &[u8]) -> EfEntry {
        EfEntry {
            pos: [f(e, 0), f(e, 4), f(e, 8)],
            flags: u(e, 0x10),
            kind: e[0x14],
            part: e[0x15],
            part_bit: e[0x16],
            b17: e[0x17],
            word18: u(e, 0x18),
            size: f(e, 0x1c),
            dir: [f(e, 0x20), f(e, 0x24), f(e, 0x28), f(e, 0x2c)],
            phase: u(e, 0x2c),
            rate: f(e, 0x30),
            duty: f(e, 0x34),
            base: f(e, 0x38),
            amp: f(e, 0x3c),
        }
    }

    /// The glare type the draw indexes (`slot + 0x18` × 0x40 into the type table) [M 0x1f2058].
    pub fn glare_type(&self) -> u32 { self.word18 }
}

/// An `ef` list: entries until the type byte is 0 [M 0x22faf8].
pub fn parse_ef_list(b: &[u8]) -> Vec<EfEntry> {
    let mut out = Vec::new();
    for e in b.chunks_exact(EF_SIZE) {
        if e[0x14] == 0 {
            break;
        }
        out.push(EfEntry::parse(e));
    }
    out
}

/// One effect entry of one placed item.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedEffect {
    /// The `in#0` item index.
    pub item: usize,
    pub article: String,
    pub part: u16,
    /// The `as` part the entry came from (for a scene object: a part of its article's sub-items).
    pub source_part: u16,
    /// The entry's index in the item's effect list (a scene object's list spans its sub-items).
    pub entry: usize,
    /// The item is a scene object (in#0 flag 0x10).
    pub scene_object: bool,
    pub e: EfEntry,
    /// Type 6: the `sr` string the reference names (`GIOT::<name>`).
    pub name: Option<String>,
    /// World position. Static glare: the entry through the item matrix. Static type 6: item
    /// origin + the entry offset, unrotated [M `FUN_00233460`]. Scene object: the entry through
    /// its sub-item matrix, then the item matrix (the composition order of `FUN_001c9368` is
    /// not traced; the sub-items of the 8 tracks are translations + axis swaps).
    pub world: [f32; 3],
    /// The entry direction through the item axes (type 2).
    pub world_dir: [f32; 3],
    /// The item origin and its word 15 (the static glare's LOD radius, `FUN_001ad128`).
    pub item_origin: [f32; 3],
    pub item_word15: u32,
}

impl PlacedEffect {
    /// Static item, type 6, flag 0x10: started by Init World [M 0x22faf8].
    pub fn starts_at_load(&self) -> bool { !self.scene_object && self.e.kind == kind::GIOTTO && self.e.flags & eflag::AT_LOAD != 0 && self.e.word18 != 0 }
    /// Static item walked by `FUN_001ad1a0`: flag 0x10 or type 2 [M 0x22faf8].
    pub fn static_active(&self) -> bool { !self.scene_object && (self.e.flags & eflag::AT_LOAD != 0 || self.e.kind == kind::GLARE) }
    /// The `GIOT::` name without the namespace.
    pub fn giotto_name(&self) -> Option<&str> { self.name.as_deref().and_then(|s| s.strip_prefix("GIOT::")) }
}

fn rotate(p: &Placement, v: [f32; 3]) -> [f32; 3] {
    let a = &p.axes;
    [
        v[0] * a[0][0] + v[1] * a[1][0] + v[2] * a[2][0],
        v[0] * a[0][1] + v[1] * a[1][1] + v[2] * a[2][1],
        v[0] * a[0][2] + v[1] * a[1][2] + v[2] * a[2][2],
    ]
}

/// The article part's effect list reference: the `as` payload's `+4` word `{u16 index, "ef"}`
/// [M 0x22faf8 `*(ref + 4)`, D].
pub fn part_ef_index(as_payload: &[u8]) -> Option<u16> {
    (as_payload.len() >= 8 && &as_payload[6..8] == b"fe").then(|| u16::from_le_bytes([as_payload[4], as_payload[5]]))
}

fn cstr(p: &[u8]) -> String {
    let n = p.iter().position(|&b| b == 0).unwrap_or(p.len());
    String::from_utf8_lossy(&p[..n]).into_owned()
}

struct ArticleFx {
    sr: HashMap<u16, String>,
    ef: HashMap<u16, Vec<EfEntry>>,
    asp: HashMap<u16, Option<u16>>,
    /// The article's own `in#N` records: 0x40-byte sub-items (matrix + `'as'` part reference at
    /// `+0x1c`), the parts a scene object is built from.
    inr: HashMap<u16, Vec<Vec<u8>>>,
}

/// A sub-item of an article's `in#N` list: its matrix and the `as` part it names.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubItem {
    pub axes: [[f32; 3]; 3],
    pub origin: [f32; 3],
    pub part: Option<u16>,
}

impl SubItem {
    pub fn parse(b: &[u8]) -> SubItem {
        let v = |i: usize| [f(b, i * 16), f(b, i * 16 + 4), f(b, i * 16 + 8)];
        let r = u(b, 0x1c);
        SubItem { axes: [v(0), v(1), v(2)], origin: v(3), part: (r >> 16 == 0x6173).then_some(r as u16) }
    }
    pub fn apply(&self, p: [f32; 3]) -> [f32; 3] {
        let a = &self.axes;
        let o = self.origin;
        [
            p[0] * a[0][0] + p[1] * a[1][0] + p[2] * a[2][0] + o[0],
            p[0] * a[0][1] + p[1] * a[1][1] + p[2] * a[2][1] + o[1],
            p[0] * a[0][2] + p[1] * a[1][2] + p[2] * a[2][2] + o[2],
        ]
    }
    fn rotate(&self, v: [f32; 3]) -> [f32; 3] {
        let a = &self.axes;
        [
            v[0] * a[0][0] + v[1] * a[1][0] + v[2] * a[2][0],
            v[0] * a[0][1] + v[1] * a[1][1] + v[2] * a[2][1],
            v[0] * a[0][2] + v[1] * a[1][2] + v[2] * a[2][2],
        ]
    }
}

fn article_fx(crp: &Crp<'_>) -> Result<HashMap<String, ArticleFx>> {
    let mut arts = HashMap::new();
    for arti in crp.pool()? {
        if arti.tag != tag::ARTI {
            continue;
        }
        let Some(name) = crp.article_name(&arti)? else { continue };
        let (s, e) = arti.window(crp.data)?;
        let mut a = ArticleFx { sr: HashMap::new(), ef: HashMap::new(), asp: HashMap::new(), inr: HashMap::new() };
        for r in crp.window_records(s, e)? {
            let i = r.instance().unwrap_or(0);
            match r.tag_code() {
                TAG_SR => {
                    a.sr.insert(i, cstr(crp.payload(&r)?));
                }
                TAG_EF => {
                    a.ef.insert(i, parse_ef_list(crp.payload(&r)?));
                }
                TAG_AS => {
                    a.asp.insert(i, part_ef_index(crp.payload(&r)?));
                }
                TAG_IN => {
                    a.inr.insert(i, crp.payload(&r)?.chunks_exact(PLACEMENT_SIZE).map(|c| c.to_vec()).collect());
                }
                _ => {}
            }
        }
        if !a.ef.is_empty() {
            arts.insert(name.to_owned(), a);
        }
    }
    Ok(arts)
}

/// Every effect entry of every `in#0` item [M + D]:
/// - a static item contributes the `ef` list of the part it names;
/// - a scene object (flag 0x10) is built from its article's `in#(ps +7)` list (`in#0` when that
///   is missing, `FUN_001c6fb8`) [M `FUN_001c7658`], and its effects are the `ef` lists of those
///   sub-items' parts, in sub-item order (`FUN_001a1f58`). `FUN_001a1f58` keeps an entry only
///   when `FUN_001a1bc8(mode, flags)` passes; that filter is not traced, so every entry is listed.
pub fn placed_effects(crp: &Crp<'_>) -> Result<Vec<PlacedEffect>> {
    let placements = map_placements(crp)?;
    let arts = article_fx(crp)?;
    let words = map_item_word15(crp)?;
    let scene_parts: HashMap<usize, u8> = track_objects(crp)?.into_iter().map(|o| (o.index, o.record.scene_part)).collect();
    let mut out = Vec::new();
    for p in &placements {
        let Some(a) = arts.get(&p.article) else { continue };
        let scene_object = p.flags & 0x10 != 0;
        // (sub-item, part) pairs this item's effects come from.
        let mut parts: Vec<(Option<SubItem>, u16)> = Vec::new();
        match scene_parts.get(&p.index).filter(|_| scene_object) {
            Some(&sp) if sp != 0xff => {
                let list = a.inr.get(&(sp as u16)).or_else(|| a.inr.get(&0));
                for sub in list.into_iter().flatten() {
                    let si = SubItem::parse(sub);
                    if let Some(part) = si.part {
                        parts.push((Some(si), part));
                    }
                }
            }
            _ => parts.push((None, p.part)),
        }
        let mut n = 0;
        for (sub, part) in parts {
            let Some(Some(idx)) = a.asp.get(&part) else { continue };
            let list = a.ef.get(idx).ok_or_else(|| FormatError::Invalid(format!("{}: part {part} names ef#{idx}, which is missing", p.article)))?;
            for e in list {
                let name = (e.kind == kind::GIOTTO && e.word18 >> 16 == 0x7372).then(|| a.sr.get(&(e.word18 as u16)).cloned()).flatten();
                let local = sub.map(|s| s.apply(e.pos)).unwrap_or(e.pos);
                let ldir = [e.dir[0], e.dir[1], e.dir[2]];
                let ldir = sub.map(|s| s.rotate(ldir)).unwrap_or(ldir);
                let world = if e.kind == kind::GIOTTO && !scene_object {
                    // FUN_00233460 without an object: item origin + the entry offset, unrotated.
                    [p.origin[0] + e.pos[0], p.origin[1] + e.pos[1], p.origin[2] + e.pos[2]]
                } else {
                    p.apply(local)
                };
                out.push(PlacedEffect {
                    item: p.index,
                    article: p.article.clone(),
                    part: p.part,
                    source_part: part,
                    entry: n,
                    scene_object,
                    e: *e,
                    name,
                    world,
                    world_dir: rotate(p, ldir),
                    item_origin: p.origin,
                    item_word15: words.get(p.index).copied().unwrap_or(0),
                });
                n += 1;
            }
        }
    }
    Ok(out)
}

/// The effects gallery `FUN_0024b7d0` picks for a track name [M 0x24b7d0, 0x24b740, 0x24b680].
pub fn giotto_gallery(track: &str) -> &'static str {
    const SNOW: [&str; 4] = ["snow1a_mis3", "snow2a_mis4", "snow1a", "snow2a"];
    const JUNGLE: [&str; 6] = ["junglea_mis13a", "jungleb_mis13b", "junglec_mis13c", "junglea", "jungleb", "junglec"];
    // FUN_002a9508 is an exact compare against DAT 0x1004f4 (the track name) [M].
    if SNOW.contains(&track) {
        "sav_gall1.gal"
    } else if JUNGLE.contains(&track) {
        "ears_gall1.gal"
    } else {
        "gall1.gal"
    }
}

/// `FUN_002a9198`: compare with each byte folded to lower case [M].
pub fn eq_ignore_case(a: &str, b: &str) -> bool { a.eq_ignore_ascii_case(b) }

/// The animation a Giotto name resolves to (`FUN_002339e8`: the first root child whose `Name`
/// matches case-insensitively) [M].
pub fn giotto_anim<'g>(gal: &'g Gal, name: &str) -> Option<&'g Anim> { gal.anims.iter().find(|a| eq_ignore_case(&a.name, name)) }

/// The `gk` key set Giotto reads (`FUN_002339c8(3, 0)`) [M].
pub const GIOTTO_KEYSET: u16 = 0x3000;
/// Element kinds [M `FUN_00234818`: tag compares].
pub const EL_GP: u32 = 0x6770_2020;
pub const EL_GS: u32 = 0x6773_2020;
pub const EL_GG: u32 = 0x6767_2020;
pub const EL_GD: u32 = 0x6764_2020;

/// One `gp` element key: a particle emitter of a Giotto effect.
#[derive(Clone, Debug, PartialEq)]
pub struct GiottoEmitter {
    pub element: String,
    /// +0x00: offset from the instance position (rotated by the instance orientation).
    pub offset: [f32; 3],
    /// +0x10: the emitter orientation (x, y, z, w).
    pub quat: [f32; 4],
    /// +0x30: the particle record as `FUN_00235ed0` adds it, before the library's `prepare`
    /// (gravity and its variance × 127, spin × 360).
    pub ptype: ParticleType,
    /// +0x2e of the record > 0: `FUN_00235ed0` writes the previously added slot into `+0x2d`.
    pub links_previous: bool,
    /// +0xd8 / +0xdc: the start frame and its override (start frame honoured when 0).
    pub start_frame: i32,
    pub start_override: i32,
}

impl GiottoEmitter {
    pub fn from_key(element: &str, raw: &[u8]) -> Option<GiottoEmitter> {
        if raw.len() < 0x30 + PARTICLE_TYPE_SIZE {
            return None;
        }
        let rec = &raw[0x30..0x30 + PARTICLE_TYPE_SIZE];
        let mut t = ParticleType::parse(rec).ok()?;
        t.gravity *= 127.0;
        t.gravity_var *= 127.0;
        t.spin_var *= 360.0;
        Some(GiottoEmitter {
            element: element.to_owned(),
            offset: [f(raw, 0), f(raw, 4), f(raw, 8)],
            quat: [f(raw, 0x10), f(raw, 0x14), f(raw, 0x18), f(raw, 0x1c)],
            ptype: t,
            links_previous: (rec[0x2e] as i8) > 0,
            start_frame: u(raw, 0xd8) as i32,
            start_override: u(raw, 0xdc) as i32,
        })
    }
}

/// A Giotto effect as the track effects use it.
#[derive(Clone, Debug, PartialEq)]
pub struct GiottoEffect {
    pub name: String,
    /// `Flgs`: bit 0 = no handle is kept (`FUN_00232af8` returns 0); bit 3 = attach.
    pub flags: u32,
    pub nump: u32,
    /// `LIFE`: the instance runs its elements while its frame count <= LIFE [M 0x2345d8].
    pub life: u32,
    /// The `gp` elements (first key of each).
    pub emitters: Vec<GiottoEmitter>,
    /// The other elements: (kind, name, key count). Not ported.
    pub others: Vec<(String, String, usize)>,
}

pub fn giotto_effect(gal: &Gal, name: &str) -> Option<GiottoEffect> {
    let a = giotto_anim(gal, name)?;
    let mut emitters = Vec::new();
    let mut others = Vec::new();
    for el in &a.elements {
        let keys = el.keysets.iter().find(|(id, _)| *id == GIOTTO_KEYSET).map(|(_, k)| k.as_slice()).unwrap_or(&[]);
        if el.kind == EL_GP {
            if let Some(k) = keys.first().and_then(|k| GiottoEmitter::from_key(&el.name, &k.raw)) {
                emitters.push(k);
            }
        } else {
            others.push((el.kind_str(), el.name.clone(), keys.len()));
        }
    }
    Some(GiottoEffect { name: a.name.clone(), flags: a.flags, nump: a.nump, life: a.life, emitters, others })
}

/// What `FUN_001da6c8` takes from a `decal.dat` record besides its UV rows [M]: `+0x44` (w1) is
/// the number of decal draws (`FUN_001da978`, one decrement each) before the decal gets its
/// alpha, 0 meaning 1; `+0x48` (w2) is that alpha byte (colour = alpha << 24 | 0x7f7f7f; until
/// then the alpha is 0). `+0x40` and `+0x4c` are not read by either [M].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecalStyle {
    pub delay_draws: u8,
    pub alpha: u8,
}

pub fn decal_style(d: &Decal) -> DecalStyle {
    DecalStyle { delay_draws: if d.w1 as u8 == 0 { 1 } else { d.w1 as u8 }, alpha: d.w2 as u8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(words: &[(usize, u32)]) -> Vec<u8> {
        let mut b = vec![0u8; EF_SIZE];
        for &(o, w) in words {
            b[o..o + 4].copy_from_slice(&w.to_le_bytes());
        }
        b
    }

    /// The paris `trafficlight` ef#1 entry 0 as on the disc [D].
    fn traffic_glare() -> Vec<u8> {
        entry(&[
            (0x00, 0x3d99521c), (0x04, 0x3f966143), (0x08, 0x3d209c59), (0x0c, 0x3f800000),
            (0x10, 0x00000c34), (0x14, 0x01000002), (0x18, 0x31), (0x1c, 0x3f800000),
            (0x20, 0xbf57e880), (0x24, 0xb2a8859f), (0x28, 0xbf098c78), (0x30, 0x3ca3d70a),
            (0x34, 0x3f2aaaab), (0x38, 0x3f000000), (0x3c, 0x3f800000),
        ])
    }

    #[test]
    fn glare_entry_fields() {
        let mut b = traffic_glare();
        b.extend_from_slice(&[0u8; EF_SIZE]);
        let l = parse_ef_list(&b);
        assert_eq!(l.len(), 1, "the zero type byte ends the list");
        let e = l[0];
        assert_eq!(e.kind, kind::GLARE);
        assert_eq!(e.glare_type(), 49);
        assert_eq!(e.flags, 0xc34);
        assert_eq!(e.flags & (eflag::PULSE | eflag::TRIANGLE | eflag::ONE_SIDED | eflag::AT_LOAD), 0xc30);
        assert_eq!(e.b17, 1);
        assert_eq!(e.size, 1.0);
        assert_eq!(e.phase, 0);
        assert!((e.rate - 0.02).abs() < 1e-7 && (e.duty - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!((e.base, e.amp), (0.5, 1.0));
        assert!((e.dir[0] + 0.843).abs() < 1e-3 && (e.dir[2] + 0.537).abs() < 1e-3);
    }

    #[test]
    fn part_ef_reference() {
        // trafficlight as#1: {'Base', ef#2, 0, 0, ...} [D]
        let p = [0x65, 0x73, 0x61, 0x42, 0x02, 0x00, 0x66, 0x65, 0, 0, 0, 0];
        assert_eq!(part_ef_index(&p), Some(2));
        assert_eq!(part_ef_index(&[0x65, 0x73, 0x61, 0x42, 0, 0, 0, 0]), None);
    }

    #[test]
    fn galleries_by_track_name() {
        assert_eq!(giotto_gallery("paris_mis01"), "gall1.gal");
        assert_eq!(giotto_gallery("uw_mis11"), "gall1.gal");
        assert_eq!(giotto_gallery("snow2a_race"), "gall1.gal", "not in the snow list");
        assert_eq!(giotto_gallery("snow1a_mis3"), "sav_gall1.gal");
        assert_eq!(giotto_gallery("snow2a_mis4"), "sav_gall1.gal");
        assert_eq!(giotto_gallery("junglec_mis13c"), "ears_gall1.gal");
    }

    #[test]
    fn gp_key_scaling_follows_fun_00235ed0() {
        // gall1 Wd_Sprk "particle" key 0 [D]: +0x7c gravity 0.179, +0x80 variance 0.064, +0x84 -1.
        let mut raw = vec![0u8; 0xf4];
        let put = |raw: &mut Vec<u8>, o: usize, w: u32| raw[o..o + 4].copy_from_slice(&w.to_le_bytes());
        put(&mut raw, 0x7c, 0x3e37ced9);
        put(&mut raw, 0x80, 0x3d8346dc);
        put(&mut raw, 0x84, 0xbf800000);
        put(&mut raw, 0x3c, 0x28);
        put(&mut raw, 0x1c, 0x3f800000);
        raw[0x5e] = 1; // record +0x2e
        put(&mut raw, 0xdc, 10);
        let g = GiottoEmitter::from_key("particle", &raw).unwrap();
        assert!((g.ptype.gravity - f32::from_bits(0x3e37ced9) * 127.0).abs() < 1e-6);
        assert!((g.ptype.gravity_var - f32::from_bits(0x3d8346dc) * 127.0).abs() < 1e-6);
        assert_eq!(g.ptype.spin_var, -360.0);
        assert_eq!(g.ptype.count, 40);
        assert_eq!(g.quat, [0.0, 0.0, 0.0, 1.0]);
        assert!(g.links_previous);
        assert_eq!((g.start_frame, g.start_override), (0, 10));
    }

    #[test]
    fn decal_words() {
        let d = Decal { corners: [[0.0; 4]; 4], w0: 0, w1: 0, w2: 0x3f, index: 0 };
        assert_eq!(decal_style(&d), DecalStyle { delay_draws: 1, alpha: 0x3f });
        let d = Decal { w1: 5, w2: 0x28, ..d };
        assert_eq!(decal_style(&d), DecalStyle { delay_draws: 5, alpha: 0x28 });
    }
}
