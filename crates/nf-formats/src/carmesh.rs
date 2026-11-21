//! The driving missions' vehicle models (`data\car\model\<car>.crp`) decoded into local-space
//! strips per part, with the part placements, the way the game draws them
//! (docs/formats/car-model.md §9). The container and record vocabulary are `carmodel.rs`; the
//! module loader and the draw chain are shared with the track decoder (`crp_mesh::TrackModule`,
//! `geoprim::ModelIndex`).
//!
//! The car draw path [M, DRIVING.ELF]:
//!
//! 1. `FUN_001d35d8` 0x1d35d8 (the vehicle draw, vtable 0x3885d8 slot 3) walks the car's `in#0`
//!    items (0x40 bytes each): all of them through `FUN_001ad6b8` when no draw list is selected
//!    (`car + 0x348 == 0`, the value `FUN_001d2338` 0x1d2338 leaves), otherwise the items whose
//!    indices the selected `vd#N` record lists, through `FUN_001ad758` (`items + u16[k] × 0x40`).
//!    `FUN_001d2618` / `FUN_001c8ba8` select the list: `vd  #N` record, payload = the u16 item
//!    indices, the record's count = their number.
//! 2. `FUN_001ad498` per item: skipped when `+0x0c & 1`; the item matrix comes from
//!    `FUN_001beee0` = the function table at 0x333a60 indexed by the item's `+0x0d` byte:
//!    0xff → `FUN_001bf140` (the item's own 4x4, words 3/7/11 zeroed, word 15 = 1.0);
//!    0xfe → `FUN_001bf600` (the same, then multiplied by the run-time matrix
//!    `FUN_001bef68(u16 +0x0e − 1)` — a `Skel` part such as a wheel); 0xef → `FUN_001bf598`
//!    (a run-time matrix from the table at 0x333a50 indexed by `+0x0e`); below 100 → animation
//!    (`FUN_001a30c0` via `FUN_001bf318`); the result is multiplied by the car's matrix
//!    (`FUN_002cbb88(item, car, item)`).
//! 3. The item's `+0x1c` word is an `as  #N` reference (in the file the short tag `as` + N, the
//!    loader turns it into a pointer). That `as` payload is the part's LOD set
//!    [M `FUN_001ad2e0` 0x1ad2e0]: `+0x04` effect ref (`FUN_001ad1a0` when set), `+0x08`
//!    animation ref, `+0x1c` u8 LOD count, `+0x1d` u8 pass (drawn now when 2, else queued through
//!    `FUN_001ab050`), `+0x1e` u8 render-state hook (table 0x3337a0), `+0x1f` u8 flags,
//!    `+0x20` `count × {f32 distance, model ref}` (the ref is an `sr  #N` short tag in the file;
//!    `sr#N` names `EAGL::<car>::{as  NNNN}::Model[L].00`).
//! 4. The model draw `FUN_002843a0` 0x2843a0 walks the group table (+0xcc), skipping a group
//!    whose visibility short (`*(model+0xc4) + 4g + 2`) is 0, and runs each entry's render list
//!    through `FUN_002a3290`; op 4 indexes `table[part]` with the part index (+0xc0) — the
//!    chain `geoprim::ModelIndex::draw_chain` follows. The loop stops at the first word whose
//!    low 16 bits are not above the first header's (one group per model on every car [D]).
//!
//! The car batches are laid out differently from the track's [D, all 35 car files of the 8
//! driving archives; tests `crates/nf-formats/tests/carmesh_disc.rs`]:
//! - layout A: `V3-32` positions (masked, w from the row), `V2-16` UV = (u16 − 0x8000) / 1024
//!   (every textured batch spans 0..1024 raw — the track's /256 would put car UVs at 0..4),
//!   `V3-8` normal = (byte − 0x80) / 127 (|n| = 127 ± 6 on 99.9% of them; these are NOT vertex
//!   colours: car batches carry no colour upload), then two `V4-16` arrays (not decoded);
//! - layout B: five `V4-32` arrays: position (w = 1), normal, UV (u, v, 1, 1), two not decoded;
//! - positions only (`V3-32`): the untextured shapes (e.g. the vanquish `{as  0002}` hull).
//!
//! Strip list: the `V4-8` bytes before the positions, as on the track, but in units of 3 per
//! vertex when the batch carries any attribute besides positions and 2 for positions-only
//! batches, and wrapping past 255 (lists of 160-vertex batches go 98, 128, …, 236, 24, 48) [D:
//! with these two rules every list of every car batch is increasing and below the count; the
//! track's fixed unit 3 refuses 70 car batches]. The unit is presumably the GIF quadwords the
//! microcode writes per vertex (ST/RGBAQ/XYZ vs RGBAQ/XYZ) — not proven, the car microcode
//! (`MatCar*` in the car module) is not disassembled.
//!
//! Pure decoding: no drawing.

use std::collections::HashMap;

use crate::buf::{invalid, FormatError, Result};
use crate::crp::{tag, Crp, Record};
use crate::crp_mesh::TrackModule;
use crate::geoprim::ModelIndex;

/// `in  ` [M FUN_001d35d8: the car's item array].
pub const TAG_IN: u32 = 0x696e_2020;
/// `as  ` — a part's LOD set [M FUN_001ad2e0].
pub const TAG_AS: u32 = 0x6173_2020;
/// `sr  ` — a named resource [D].
pub const TAG_SR: u32 = 0x7372_2020;
/// `vd  ` — a draw list [M FUN_001c8ba8: `0x76640000 | N`].
pub const TAG_VD: u32 = 0x7664_2020;
/// `Skel` — the named part table [M FUN_001c9980].
pub const TAG_SKEL: u32 = 0x536b_656c;
/// The car UV rule: `(u16 − 0x8000) / 1024` [D].
pub const UV_SCALE: f32 = 1024.0;

/// One decoded strip of a car model, in the model's frame.
#[derive(Clone, Debug, PartialEq)]
pub struct CarStrip {
    /// The sheet entry name (the descriptor's +4 word, see `geoprim::ModelIndex::packet_texture`);
    /// `None` when the batch has no UVs or no texture was set yet in this model.
    pub texture: Option<String>,
    pub positions: Vec<[f32; 3]>,
    /// Empty for untextured batches.
    pub uvs: Vec<[f32; 2]>,
    /// Unit normals (approximately), empty for positions-only batches.
    pub normals: Vec<[f32; 3]>,
}

impl CarStrip {
    /// Triangles (k−2, k−1, k) for k ≥ 2 (winding alternates).
    pub fn triangles(&self) -> impl Iterator<Item = [[f32; 3]; 3]> + '_ {
        (2..self.positions.len()).map(move |k| [self.positions[k - 2], self.positions[k - 1], self.positions[k]])
    }
}

/// How an item's matrix is formed [M FUN_001beee0 + the table at 0x333a60].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attach {
    /// 0xff: the item's own matrix [M FUN_001bf140].
    Fixed,
    /// 0xfe: own matrix × the run-time matrix of `Skel` part `n` (0-based; the item stores n + 1)
    /// [M FUN_001bf600 → FUN_001bef68(+0x0e − 1)].
    Skel(u16),
    /// Any other byte: a run-time or animated transform the decoder does not supply (0xef →
    /// FUN_001bf598, below 100 → animation FUN_001a30c0; others per the table). The own matrix is
    /// what this decoder places the part with.
    Other(u8, u16),
}

/// One `in#0` item: a placed part.
#[derive(Clone, Debug, PartialEq)]
pub struct CarPart {
    /// The item index in `in#0` (what `vd` lists name).
    pub index: usize,
    /// +0x0c: 0x01 = not drawn [M FUN_001ad498]; 0x80 = mirrored (the stored x axis is negated
    /// on every such item [D]; the animated path negates it itself [M FUN_001a30c0]).
    pub flags: u8,
    pub attach: Attach,
    /// Words 0-2, 4-6, 8-10: the images of the local x, y, z axes.
    pub axes: [[f32; 3]; 3],
    /// Words 12-14: the image of the local origin.
    pub origin: [f32; 3],
    /// The referenced `as#N`.
    pub set: u16,
    /// `as +0x1d`: 2 = drawn in the car pass, other values are queued for a later pass
    /// [M FUN_001ad498] (e.g. the vanquish glass, pass 1).
    pub pass: u8,
    /// `as +0x20`: `(distance, EAGL model name)`; LOD k draws while the size measure is ≤ its
    /// distance [M FUN_001ad2e0]. Empty for effect-only sets.
    pub lods: Vec<(f32, Option<String>)>,
    /// LOD 0's strips in the part's local frame.
    pub strips: Vec<CarStrip>,
}

impl CarPart {
    /// `x·axis0 + y·axis1 + z·axis2 + origin` (the static matrix; see [`Attach`]).
    pub fn apply(&self, p: [f32; 3]) -> [f32; 3] {
        let a = &self.axes;
        [
            p[0] * a[0][0] + p[1] * a[1][0] + p[2] * a[2][0] + self.origin[0],
            p[0] * a[0][1] + p[1] * a[1][1] + p[2] * a[2][1] + self.origin[1],
            p[0] * a[0][2] + p[1] * a[1][2] + p[2] * a[2][2] + self.origin[2],
        ]
    }

    pub fn hidden(&self) -> bool { self.flags & 1 != 0 }

    /// The determinant sign of the axes: negative for mirrored parts (their strips wind the other
    /// way once placed).
    pub fn mirrored(&self) -> bool {
        let a = &self.axes;
        let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1]) - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        det < 0.0
    }
}

/// A decoded car model.
#[derive(Clone, Debug, Default)]
pub struct CarMesh {
    /// The article name (`Name`).
    pub name: String,
    /// The texture sheet named by the directory `ns#0` record (`vanquish.ssh`).
    pub sheet: Option<String>,
    /// The `Skel` part names (index n = `Attach::Skel(n)`).
    pub skel: Vec<String>,
    /// Every `in#0` item in order.
    pub parts: Vec<CarPart>,
    /// The `vd#N` draw lists: `(N, item indices)`.
    pub draw_lists: Vec<(u16, Vec<u16>)>,
}

impl CarMesh {
    /// The parts drawn with draw list `list` selected, or with none (`None` = every item, the
    /// state after `FUN_001d2338`) [M FUN_001d35d8]; hidden items are left out. A list index
    /// that names no `vd` record selects none, which draws every item [M FUN_001c8ba8 returns 0].
    pub fn drawn(&self, list: Option<u16>) -> Vec<&CarPart> {
        let sel = list.and_then(|n| self.draw_lists.iter().find(|(k, _)| *k == n));
        let mut out = Vec::new();
        match sel {
            Some((_, idx)) => {
                for &i in idx {
                    if let Some(p) = self.parts.get(i as usize) {
                        if !p.hidden() {
                            out.push(p);
                        }
                    }
                }
            }
            None => out.extend(self.parts.iter().filter(|p| !p.hidden())),
        }
        out
    }
}

fn u32_at(d: &[u8], o: usize) -> Result<u32> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| FormatError::Invalid(format!("u32 at {o:#x} past the end ({})", d.len())))
}

fn f32_at(d: &[u8], o: usize) -> Result<f32> { Ok(f32::from_bits(u32_at(d, o)?)) }

fn cstr(p: &[u8]) -> String {
    let n = p.iter().position(|&b| b == 0).unwrap_or(p.len());
    String::from_utf8_lossy(&p[..n]).into_owned()
}

/// Decodes one 0x40-byte `in#0` item: `(axes, origin, flags, +0x0d, +0x0e, as index)`
/// [M FUN_001bf140 matrix copy, FUN_001ad498 / FUN_001beee0 fields]. The `+0x1c` word must be an
/// `as` short-tag reference.
pub fn decode_item(item: &[u8]) -> Result<([[f32; 3]; 3], [f32; 3], u8, u8, u16, u16)> {
    if item.len() < 0x40 {
        return invalid(format!("in item {} < 0x40 bytes", item.len()));
    }
    let v = |i: usize| -> Result<[f32; 3]> { Ok([f32_at(item, i * 16)?, f32_at(item, i * 16 + 4)?, f32_at(item, i * 16 + 8)?]) };
    let r = u32_at(item, 0x1c)?;
    if r >> 16 != 0x6173 {
        return invalid(format!("in item reference {r:#x} is not an as short tag"));
    }
    let link = u16::from_le_bytes([item[0x0e], item[0x0f]]);
    Ok(([v(0)?, v(1)?, v(2)?], v(3)?, item[0x0c], item[0x0d], link, r as u16))
}

/// The LOD set of one `as` payload: `(pass, [(distance, sr index)])` [M FUN_001ad2e0]. A model
/// reference that is not an `sr` short tag yields `None` for that LOD.
pub fn decode_set(p: &[u8]) -> Result<(u8, Vec<(f32, Option<u16>)>)> {
    if p.len() < 0x20 {
        return invalid(format!("as payload {} < 0x20 bytes", p.len()));
    }
    let n = p[0x1c] as usize;
    let mut lods = Vec::with_capacity(n);
    for k in 0..n {
        let d = f32_at(p, 0x20 + 8 * k)?;
        let r = u32_at(p, 0x24 + 8 * k)?;
        lods.push((d, (r >> 16 == 0x7372).then_some(r as u16)));
    }
    Ok((p[0x1d], lods))
}

/// The entry objects of a model in draw order, as `FUN_002843a0` walks the group table: groups
/// whose visibility short is 0 are skipped, the walk ends at the first word whose low half is not
/// above the first header's [M 0x2843a0].
pub fn model_entries(ix: &ModelIndex<'_>, sym_value: u32) -> Result<Vec<usize>> {
    let img = &ix.module.image;
    let ptr = |p: u32| -> Result<usize> {
        let at = ix.data_offset + p as usize;
        if at >= img.len() {
            return invalid(format!("pointer {p:#x} past the module image"));
        }
        Ok(at)
    };
    let obj = ptr(sym_value)?;
    let vis = u32_at(img, obj + 0xc4)?;
    let mut cur = ptr(u32_at(img, obj + 0xcc)?)?;
    let mut w = u32_at(img, cur)?;
    let first = w & 0xffff;
    let mut g = 0usize;
    let mut out = Vec::new();
    for _ in 0..65536 {
        if w & 0xffff == 0xffff {
            let e = u32_at(img, cur + 4)?;
            cur += 8;
            if e != 0 {
                out.push(ptr(e)?);
            }
        } else {
            let shown = vis == 0 || u16::from_le_bytes(img.get(ptr(vis)? + g * 4 + 2..ptr(vis)? + g * 4 + 4).ok_or_else(|| FormatError::Invalid("group flag past the image".into()))?.try_into().unwrap()) != 0;
            cur += if shown { 12 } else { 12 + 4 * u32_at(img, cur + 8)? as usize };
            g += 1;
        }
        w = u32_at(img, cur)?;
        if first >= w & 0xffff {
            return Ok(out);
        }
    }
    invalid(format!("model {sym_value:#x}: group table has no end"))
}

/// Decodes the VIF byte ranges of one car draw (a `draw_chain`) into strips (texture left `None`)
/// [D, see the module docs]. `label` names the batch in errors.
pub fn decode_car_vif(img: &[u8], ranges: &[(usize, usize)], label: usize) -> Result<Vec<CarStrip>> {
    let po = label;
    let mut mode = 0u32;
    let mut row = [0u32; 4];
    let mut mask = 0u32;
    let mut count: Option<usize> = None;
    let mut ends: Vec<u8> = Vec::new();
    let mut positions: Option<Vec<[f32; 3]>> = None;
    let mut uvs: Option<Vec<[f32; 2]>> = None;
    let mut normals: Option<Vec<[f32; 3]>> = None;
    // Attribute arrays after the positions (any format), for the strip-list unit.
    let mut extra = 0usize;
    let mut v4_arrays = 0usize;
    for &(start, end) in ranges {
        if end > img.len() {
            return invalid(format!("batch {po:#x}: VIF range {start:#x}..{end:#x} runs past the image"));
        }
        let mut cur = start;
        while cur + 4 <= end {
            let code = u32_at(img, cur)?;
            cur += 4;
            let cmd = ((code >> 24) & 0x7f) as u8;
            let imm = code & 0xffff;
            match cmd {
                0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x06 | 0x07 | 0x10 | 0x11 | 0x13 | 0x14 | 0x15 | 0x17 => {}
                0x05 => mode = imm & 3,
                0x20 => {
                    mask = u32_at(img, cur)?;
                    cur += 4;
                }
                0x30 | 0x31 => {
                    if cmd == 0x30 {
                        for (k, r) in row.iter_mut().enumerate() {
                            *r = u32_at(img, cur + k * 4)?;
                        }
                    }
                    cur += 16;
                }
                0x50 | 0x51 => cur += if imm == 0 { 0x10000 } else { imm as usize } * 16,
                c if c & 0x60 == 0x60 => {
                    let num = match (code >> 16) & 0xff { 0 => 256, n => n as usize };
                    let vn = ((c >> 2) & 3) as usize;
                    let vl = (c & 3) as usize;
                    let masked = c & 0x10 != 0;
                    let usn = code & 0x4000 != 0;
                    let addr = code & 0x3ff;
                    let bytes = if vn == 3 && vl == 3 { num * 2 } else { num * (vn + 1) * [4, 2, 1][vl.min(2)] };
                    let len = (bytes + 3) & !3;
                    if cur + len > end {
                        return invalid(format!("batch {po:#x}: unpack at {cur:#x} runs past the range"));
                    }
                    let data = &img[cur..cur + bytes];
                    let comp_is_data = |k: usize| !masked || (0..4).all(|r| (mask >> (r * 8 + k * 2)) & 3 == 0);
                    let fl = |i: usize, k: usize, stride: usize| -> Result<f32> {
                        let v = u32_at(data, i * stride + k * 4)?;
                        Ok(f32::from_bits(if mode == 1 { v.wrapping_add(row[k]) } else { v }))
                    };
                    match (vn, vl, num, positions.is_some()) {
                        (3, 0, 1, false) if addr == 0 && count.is_none() => count = Some(u32_at(data, 0)? as usize),
                        (3, 2, 1, false) => ends.extend_from_slice(&data[..4]),
                        (2, 0, _, false) => {
                            if !(0..3).all(comp_is_data) {
                                return invalid(format!("batch {po:#x}: position unpack {code:#x} masks x/y/z"));
                            }
                            positions = Some((0..num).map(|i| Ok([fl(i, 0, 12)?, fl(i, 1, 12)?, fl(i, 2, 12)?])).collect::<Result<_>>()?);
                        }
                        (3, 0, n, false) if n > 1 => {
                            // Layout B: array 0 = positions (x, y, z, 1).
                            positions = Some((0..num).map(|i| Ok([fl(i, 0, 16)?, fl(i, 1, 16)?, fl(i, 2, 16)?])).collect::<Result<_>>()?);
                            v4_arrays = 1;
                        }
                        (_, _, n, true) if Some(n) == count => {
                            extra += 1;
                            match (vn, vl) {
                                (1, 1) if uvs.is_none() => {
                                    if !usn || masked {
                                        return invalid(format!("batch {po:#x}: uv unpack {code:#x} signed or masked"));
                                    }
                                    uvs = Some(
                                        (0..num)
                                            .map(|i| {
                                                let c = |k: usize| (u16::from_le_bytes([data[i * 4 + k * 2], data[i * 4 + k * 2 + 1]]) as f32 - 32768.0) / UV_SCALE;
                                                [c(0), c(1)]
                                            })
                                            .collect(),
                                    );
                                }
                                (1, 0) if uvs.is_none() => {
                                    uvs = Some((0..num).map(|i| Ok([fl(i, 0, 8)?, fl(i, 1, 8)?])).collect::<Result<_>>()?);
                                }
                                (2, 2) if normals.is_none() => {
                                    if !usn || masked {
                                        return invalid(format!("batch {po:#x}: normal unpack {code:#x} signed or masked"));
                                    }
                                    normals = Some(
                                        (0..num)
                                            .map(|i| {
                                                let c = |k: usize| (data[i * 3 + k] as f32 - 128.0) / 127.0;
                                                [c(0), c(1), c(2)]
                                            })
                                            .collect(),
                                    );
                                }
                                (3, 0) if v4_arrays > 0 => {
                                    // Layout B: array 1 = normal (x, y, z, ~1/255), array 2 = UV (u, v, 1, 1).
                                    if v4_arrays == 1 && normals.is_none() {
                                        normals = Some((0..num).map(|i| Ok([fl(i, 0, 16)?, fl(i, 1, 16)?, fl(i, 2, 16)?])).collect::<Result<_>>()?);
                                    } else if v4_arrays == 2 && uvs.is_none() {
                                        uvs = Some((0..num).map(|i| Ok([fl(i, 0, 16)?, fl(i, 1, 16)?])).collect::<Result<_>>()?);
                                    }
                                    v4_arrays += 1;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    cur += len;
                }
                other => return invalid(format!("batch {po:#x}: unhandled VIF command {other:#x} at {:#x}", cur - 4)),
            }
        }
    }
    let (Some(count), Some(positions)) = (count, positions) else { return Ok(Vec::new()) };
    if positions.len() != count {
        return invalid(format!("batch {po:#x}: header count {count} != {} positions", positions.len()));
    }
    let uvs = uvs.unwrap_or_default();
    let normals = normals.unwrap_or_default();
    for (what, n) in [("uvs", uvs.len()), ("normals", normals.len())] {
        if n != 0 && n != count {
            return invalid(format!("batch {po:#x}: {n} {what} for {count} positions"));
        }
    }
    let starts = strip_starts(&ends, count, if extra == 0 { 2 } else { 3 })
        .ok_or_else(|| FormatError::Invalid(format!("batch {po:#x}: strip list {ends:?} invalid for {count} vertices")))?;
    Ok(starts
        .windows(2)
        .map(|w| CarStrip {
            texture: None,
            positions: positions[w[0]..w[1]].to_vec(),
            uvs: if uvs.is_empty() { Vec::new() } else { uvs[w[0]..w[1]].to_vec() },
            normals: if normals.is_empty() { Vec::new() } else { normals[w[0]..w[1]].to_vec() },
        })
        .collect())
}

/// The strip start vertices from a strip list (bytes up to the first 0), `unit` bytes per vertex,
/// the running value wrapping past 255 [D]; `None` when a value is not a multiple of `unit`, not
/// increasing, or not below `count`. Ends with `count`.
pub fn strip_starts(list: &[u8], count: usize, unit: usize) -> Option<Vec<usize>> {
    let mut starts = vec![0usize];
    let mut base = 0usize;
    let mut prev = 0usize;
    for &b in list.iter().take_while(|&&b| b != 0) {
        let mut v = base + b as usize;
        if v <= prev {
            base += 256;
            v += 256;
        }
        if v % unit != 0 || v / unit >= count || v / unit <= *starts.last().unwrap() {
            return None;
        }
        prev = v;
        starts.push(v / unit);
    }
    starts.push(count);
    Some(starts)
}

/// The strips of one model's LOD as the game draws it for its part index: per group entry the draw
/// chain's batch ([`ModelIndex::draw_chain`]) with the entry's texture
/// ([`ModelIndex::packet_texture`]). An entry whose list sets no texture keeps the previous entry's
/// within the model — op 3 is the only texture upload of the interpreter [M FUN_002a3290 case 3;
/// that the GS state carries over between entries is inferred]; batches without UVs get `None`.
pub fn car_model_strips(ix: &ModelIndex<'_>, sym_value: u32) -> Result<Vec<CarStrip>> {
    let part = ix.model_part(sym_value)?;
    let mut out = Vec::new();
    let mut tex: Option<String> = None;
    for po in model_entries(ix, sym_value)? {
        if let Some(t) = ix.packet_texture(po, part) {
            tex = Some(t);
        }
        for mut s in decode_car_vif(&ix.module.image, &ix.draw_chain(po, part)?, po)? {
            s.texture = if s.uvs.is_empty() { None } else { tex.clone() };
            out.push(s);
        }
    }
    Ok(out)
}

fn window(crp: &Crp<'_>, rec: &Record) -> Result<Vec<Record>> {
    let (s, e) = rec.window(crp.data)?;
    crp.window_records(s, e)
}

/// Decodes a car `.crp` (decompressed bytes): every `in#0` item with its LOD-0 strips.
pub fn car_mesh(crp_bytes: &[u8]) -> Result<CarMesh> {
    let crp = Crp::parse(crp_bytes)?;
    let arti = crp
        .pool()?
        .into_iter()
        .find(|r| r.tag == tag::ARTI)
        .ok_or_else(|| FormatError::Invalid("no Arti in the car .crp pool".into()))?;
    let mut mesh = CarMesh::default();
    let mut sr: HashMap<u16, String> = HashMap::new();
    let mut sets: HashMap<u16, (u8, Vec<(f32, Option<u16>)>)> = HashMap::new();
    let mut items: Option<Record> = None;
    for r in window(&crp, &arti)? {
        let code = r.tag_code();
        if r.tag == tag::NAME {
            mesh.name = cstr(crp.payload(&r)?);
        } else if r.tag == TAG_SKEL {
            let p = crp.payload(&r)?;
            for i in 0..r.count.max(0) as usize {
                let off = p.get(i * 0x20 + 0x1c..i * 0x20 + 0x1e).ok_or_else(|| FormatError::Invalid(format!("Skel entry {i} past the payload")))?;
                let off = u16::from_le_bytes([off[0], off[1]]) as usize;
                mesh.skel.push(cstr(p.get(off..).ok_or_else(|| FormatError::Invalid(format!("Skel name {off:#x} past the payload")))?));
            }
        } else if code == TAG_SR {
            sr.insert(r.instance().unwrap_or(0), cstr(crp.payload(&r)?));
        } else if code == TAG_AS {
            sets.insert(r.instance().unwrap_or(0), decode_set(crp.payload(&r)?)?);
        } else if code == TAG_VD {
            let p = crp.payload(&r)?;
            let n = r.count.max(0) as usize;
            if p.len() < n * 2 {
                return invalid(format!("vd#{:?}: payload {} < count {n} × 2", r.instance(), p.len()));
            }
            mesh.draw_lists.push((r.instance().unwrap_or(0), (0..n).map(|k| u16::from_le_bytes([p[2 * k], p[2 * k + 1]])).collect()));
        } else if code == TAG_IN && r.instance() == Some(0) && items.is_none() {
            items = Some(r);
        }
    }
    if let Some(ns) = crp.find_instance(tag::NS, u32::MAX)? {
        mesh.sheet = Some(cstr(crp.payload(&ns)?));
    }
    let items = items.ok_or_else(|| FormatError::Invalid("no in#0 record in the car article".into()))?;
    let p = crp.payload(&items)?;
    let n = items.count.max(0) as usize;
    if p.len() < n * 0x40 {
        return invalid(format!("in#0: payload {} < count {n} × 0x40", p.len()));
    }
    let module = TrackModule::load(&crp)?;
    let ix = ModelIndex::new(&module)?;
    let mut cache: HashMap<u32, Vec<CarStrip>> = HashMap::new();
    for i in 0..n {
        let (axes, origin, flags, kind, link, set) = decode_item(&p[i * 0x40..(i + 1) * 0x40])?;
        let (pass, lods) = sets.get(&set).cloned().ok_or_else(|| FormatError::Invalid(format!("in item {i}: no as#{set}")))?;
        let lods: Vec<(f32, Option<String>)> = lods.into_iter().map(|(d, s)| (d, s.and_then(|k| sr.get(&k).cloned()))).collect();
        let mut strips = Vec::new();
        if let Some((_, Some(model))) = lods.first() {
            let sym = ix.model_symbol(model).ok_or_else(|| FormatError::Invalid(format!("in item {i}: no module symbol for {model}")))?;
            if !cache.contains_key(&sym) {
                cache.insert(sym, car_model_strips(&ix, sym)?);
            }
            strips = cache[&sym].clone();
        }
        let attach = match kind {
            0xff => Attach::Fixed,
            0xfe => Attach::Skel(link.wrapping_sub(1)),
            k => Attach::Other(k, link),
        };
        mesh.parts.push(CarPart { index: i, flags, attach, axes, origin, set, pass, lods, strips });
    }
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(b: &mut Vec<u8>, w: u32) { b.extend_from_slice(&w.to_le_bytes()); }

    /// A layout-A batch: header, strip list, V3-32 masked positions (mode 1, row w), V2-16 UV,
    /// V3-8 normals, two V4-16 arrays, MSCNT — the vanquish shape [D].
    fn batch_a(pos: &[[f32; 3]], list: [u8; 4], uv: &[[u16; 2]], nrm: &[[u8; 3]]) -> Vec<u8> {
        let n = pos.len() as u32;
        let mut v = Vec::new();
        put(&mut v, 0x6c01_c000);
        put(&mut v, n);
        for _ in 0..3 {
            put(&mut v, 0);
        }
        put(&mut v, 0x6e01_c01f);
        v.extend_from_slice(&list);
        put(&mut v, 0x2000_0000);
        put(&mut v, 0x4040_4040);
        put(&mut v, 0x3000_0000);
        for w in [0, 0, 0, 0x3f80_0000] {
            put(&mut v, w);
        }
        put(&mut v, 0x0500_0001);
        put(&mut v, 0x7800_c022 | (n << 16));
        for p in pos {
            for c in p {
                put(&mut v, c.to_bits());
            }
        }
        put(&mut v, 0x2000_0000);
        put(&mut v, 0);
        put(&mut v, 0x0500_0000);
        put(&mut v, 0x6500_c04e | (n << 16));
        for t in uv {
            v.extend_from_slice(&t[0].to_le_bytes());
            v.extend_from_slice(&t[1].to_le_bytes());
        }
        put(&mut v, 0x6a00_c07a | (n << 16));
        for c in nrm {
            v.extend_from_slice(c);
        }
        while v.len() % 4 != 0 {
            v.push(0);
        }
        for a in [0xa6u32, 0xd2] {
            put(&mut v, 0x6d00_c000 | a | (n << 16));
            for _ in 0..n {
                put(&mut v, 0x8000_8000);
                put(&mut v, 0x8000_8000);
            }
        }
        put(&mut v, 0x1700_0000);
        v
    }

    #[test]
    fn a_car_batch_reads_uv_over_1024_normals_biased_and_the_strip_list_in_its_unit() {
        let pos = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [-1.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 2.0, 0.5]];
        let uv = [[0x8000, 0x8000], [0x8400, 0x8000], [0x8000, 0x8400], [0x8400, 0x8400], [0x8200, 0x8200]];
        let nrm = [[0x80, 0x80, 0xff], [0x80, 0xff, 0x80], [0x01, 0x80, 0x80], [0x80, 0x80, 0x01], [0xff, 0x80, 0x80]];
        // One break at vertex 3 (byte 9 = 3 × 3).
        let b = batch_a(&pos, [9, 0, 0, 0], &uv, &nrm);
        let s = decode_car_vif(&b, &[(0, b.len())], 0).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].positions, pos[..3].to_vec());
        assert_eq!(s[1].positions, pos[3..].to_vec());
        assert_eq!(s[0].uvs[1], [1.0, 0.0]);
        assert_eq!(s[1].uvs[1], [0.5, 0.5]);
        assert_eq!(s[0].normals[0], [0.0, 0.0, 1.0]);
        assert_eq!(s[0].normals[2], [-1.0, 0.0, 0.0]);
        // The track reader takes these bytes as colours and the UVs at /256 (0x8400 → 4.0).
        let t = crate::geoprim::decode_vif(&b, &[(0, b.len())], 0).unwrap();
        assert_eq!(t[0].colors[0], [0x80, 0x80, 0xff]);
        assert_eq!(t[0].uvs[1], [4.0, 0.0]);
        // A list byte that is not a multiple of 3 is refused.
        let bad = batch_a(&pos, [10, 0, 0, 0], &uv, &nrm);
        assert!(decode_car_vif(&bad, &[(0, bad.len())], 0).is_err());
    }

    #[test]
    fn strip_lists_wrap_past_255_and_use_2_per_vertex_without_attributes() {
        // A 160-vertex positions-only batch: 2 × 120 = 240, 2 × 140 = 280 → stored 24.
        assert_eq!(strip_starts(&[200, 240, 24, 0], 160, 2), Some(vec![0, 100, 120, 140, 160]));
        assert_eq!(strip_starts(&[200, 240, 24], 160, 3), None);
        assert_eq!(strip_starts(&[], 4, 3), Some(vec![0, 4]));
        // Past the count.
        assert_eq!(strip_starts(&[12], 4, 3), None);
    }

    fn item(axes: [[f32; 3]; 3], origin: [f32; 3], flags: u8, kind: u8, link: u16, set: u16) -> Vec<u8> {
        let mut it = vec![0u8; 0x40];
        for (i, a) in axes.iter().chain(std::iter::once(&origin)).enumerate() {
            for (k, c) in a.iter().enumerate() {
                it[i * 16 + k * 4..i * 16 + k * 4 + 4].copy_from_slice(&c.to_le_bytes());
            }
        }
        it[0x0c] = flags;
        it[0x0d] = kind;
        it[0x0e..0x10].copy_from_slice(&link.to_le_bytes());
        it[0x1c..0x20].copy_from_slice(&(0x6173_0000u32 | set as u32).to_le_bytes());
        it
    }

    /// The vanquish front-left wheel: item 24 names `as#0x18`, mirrored item 25 names it too; the
    /// `as` payload's own +0x18 word is its own index (0x18). The old reading placed `as#k` with
    /// item k's matrix (`as_bone = payload +0x18`), so this wheel would sit at item 0x18's origin.
    #[test]
    fn a_part_is_placed_by_the_item_that_names_its_set_not_by_the_set_index() {
        let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mir = [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let (axes, origin, flags, kind, link, set) = decode_item(&item(id, [-0.75, 0.353, 1.409], 0, 0xfe, 3, 0x18)).unwrap();
        assert_eq!((axes, origin, flags, kind, link, set), (id, [-0.75, 0.353, 1.409], 0, 0xfe, 3, 0x18));
        let (_, _, f2, _, _, s2) = decode_item(&item(mir, [0.75, 0.353, 1.409], 0x80, 0xfe, 7, 0x18)).unwrap();
        assert_eq!((f2, s2), (0x80, 0x18));
        let part = |axes, origin, flags| CarPart { index: 0, flags, attach: Attach::Skel(2), axes, origin, set: 0x18, pass: 2, lods: vec![], strips: vec![] };
        let l = part(id, [-0.75, 0.353, 1.409], 0);
        let r = part(mir, [0.75, 0.353, 1.409], 0x80);
        assert_eq!(l.apply([0.1, 0.0, 0.0]), [-0.65, 0.353, 1.409]);
        assert_eq!(r.apply([0.1, 0.0, 0.0]), [0.65, 0.353, 1.409]);
        assert!(r.mirrored() && !l.mirrored());
        // A reference that is not an `as` tag is refused.
        let mut bad = item(id, [0.0; 3], 0, 0xff, 0, 1);
        bad[0x1e] = b'x';
        assert!(decode_item(&bad).is_err());
        // The as payload: pass 2, one LOD at 1.0 → sr#0x6a.
        let mut p = vec![0u8; 0x28];
        p[0x18] = 0x18;
        p[0x1c] = 1;
        p[0x1d] = 2;
        p[0x20..0x24].copy_from_slice(&1.0f32.to_le_bytes());
        p[0x24..0x28].copy_from_slice(&0x7372_006au32.to_le_bytes());
        assert_eq!(decode_set(&p).unwrap(), (2, vec![(1.0, Some(0x6a))]));
    }

    #[test]
    fn the_drawn_set_follows_the_selected_draw_list() {
        let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let part = |index, flags| CarPart { index, flags, attach: Attach::Fixed, axes: id, origin: [0.0; 3], set: 0, pass: 2, lods: vec![], strips: vec![] };
        let m = CarMesh { parts: vec![part(0, 0), part(1, 1), part(2, 0)], draw_lists: vec![(1, vec![2, 1])], ..Default::default() };
        assert_eq!(m.drawn(None).iter().map(|p| p.index).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(m.drawn(Some(1)).iter().map(|p| p.index).collect::<Vec<_>>(), vec![2]);
        assert_eq!(m.drawn(Some(9)).len(), 2);
    }
}
