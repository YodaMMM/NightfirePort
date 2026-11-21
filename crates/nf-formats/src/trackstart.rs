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
    /// The `EStartMission` argument word (`+4` of the arguments), read as f32: the start speed
    /// factor (× the car's `+0x4c`) [M 0x15c920 / 0x15c738].
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

/// The player start of a track `.crp`: the (last, as `FUN_0022cd30` fires them in index order)
/// trigger holding an `EStartMission` action, placed as `FUN_001fee68` places the car.
/// `tris` = the track's collision triangles (`Crp::extract_collision_tris`).
pub fn track_spawn(crp: &Crp<'_>, tris: &[CollisionTri]) -> Result<TrackSpawn> {
    let trig = map_triggers(crp)?;
    let t = trig
        .iter()
        .rev()
        .find(|t| t.actions.iter().any(|a| a.id == ACTION_START_MISSION))
        .ok_or_else(|| FormatError::Invalid("no Trgr record carries an EStartMission (0x16) action".into()))?;
    let a = t.actions.iter().find(|a| a.id == ACTION_START_MISSION).unwrap();
    let speed_factor = f32_at(&a.args, 4)?;
    let ground = ground_height(tris, t.pos[0], t.pos[1], t.pos[2]);
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
}
