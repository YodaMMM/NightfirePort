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
//!   `V3-8` normal = byte / 127.5 − 1 (|n| = 1 ± 5 % on 99.9 % of them; these are NOT vertex
//!   colours: car batches carry no colour upload), then two `V4-16` arrays, the microcode's
//!   `DmgVec` / `DmgNrm` damage offsets (not decoded) [M bondrm.o parameter names, §10 of the doc];
//! - layout B: five `V4-32` arrays: position (w = 1), normal, UV (u, v, 1, 1), two not decoded;
//! - positions only (`V3-32`).
//!
//! Strip list: the `V4-8` bytes before the positions, as on the track, but in units of 3 per
//! vertex when the batch carries any attribute besides positions and 2 for positions-only
//! batches, and wrapping past 255 (lists of 160-vertex batches go 98, 128, …, 236, 24, 48) [D:
//! with these two rules every list of every car batch is increasing and below the count; the
//! track's fixed unit 3 refuses 70 car batches]. Unit 3 is the output stride of the `MatCar*`
//! microcode (ST, RGBAQ, XYZ2 per vertex; the `SplitReg` values index the XYZ2 quadwords whose ADC
//! bit it sets) [M bondrm.o]; unit 2 for positions-only batches is data-proven only.
//!
//! Each strip carries its render method ([`CarMaterial`], the GeoPrim header's `+8` external
//! symbol) and the CarRender class its render list uploads ([`CarLightClass`]); the shading those
//! feed is docs/formats/car-model.md §10.
//!
//! Pure decoding: no drawing.

use std::collections::{BTreeMap, HashMap};

use crate::buf::{invalid, FormatError, Result};
use crate::crp::{tag, Crp, Record};
use crate::crp_mesh::TrackModule;
use crate::geoprim::ModelIndex;
pub use crate::rendertun::CarLightClass;

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
/// `Base` — the article header every `as` payload's first word names [D] (docs/formats/car-model.md §11.4).
pub const TAG_BASE: u32 = 0x4261_7365;
/// The car UV rule: `(u16 − 0x8000) / 1024` [D; M every textured `MatCar*` microcode: `ITOF0`, then
/// `× I (0x3a7ffff7 ≈ 1/1024) + (−32)`].
pub const UV_SCALE: f32 = 1024.0;
/// The car normal rule: `byte / 127.5 − 1` [M every `MatCar*` microcode's set-up loop: `ITOF0`,
/// `× I (0x3c008080 = 1/127.5) + (−1)`; the I value an `{I}` pair loads is the one the next pair
/// reads — read the other way the same loop would scale the normals by 1/1024].
pub const NORMAL_SCALE: f32 = 127.5;

/// The EAGL render method a car batch is drawn with: the undefined symbol its GeoPrim header's `+8`
/// word is relocated against (the header = the group entry's `+0` word, the object whose `+0` is
/// the render list `FUN_002a3290` walks [M 0x2843a0]) [D: every entry of every car module names
/// one; `data\render\bondrm.o` (MISC.VIV) defines each with its `<name>0__EAGLMicroCode`, the block
/// the render list's op 1 uploads (`FUN_002a3290` case 1)]. The passes each microcode sends are in
/// docs/formats/car-model.md §10.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CarMaterial {
    Texture,
    TexturePacked,
    TextureDull,
    TextureDullPacked,
    TextureSwitchPacked,
    Gouraud,
    GouraudPacked,
    GouraudDull,
    GouraudDullPacked,
    Glass,
    GlassPacked,
    /// Same microcode bytes as `MatCarTexturePacked` [D]; differs by its GS states and props.
    Chrome,
    Dash,
    /// `BondCarShadow`: positions only, coloured by `EAGL::CarShadowColour`.
    Shadow,
}

impl CarMaterial {
    pub const ALL: [CarMaterial; 14] = [
        CarMaterial::Texture,
        CarMaterial::TexturePacked,
        CarMaterial::TextureDull,
        CarMaterial::TextureDullPacked,
        CarMaterial::TextureSwitchPacked,
        CarMaterial::Gouraud,
        CarMaterial::GouraudPacked,
        CarMaterial::GouraudDull,
        CarMaterial::GouraudDullPacked,
        CarMaterial::Glass,
        CarMaterial::GlassPacked,
        CarMaterial::Chrome,
        CarMaterial::Dash,
        CarMaterial::Shadow,
    ];

    /// The module symbol (`MatCarTexturePacked`).
    pub fn symbol(self) -> &'static str {
        match self {
            CarMaterial::Texture => "MatCarTexture",
            CarMaterial::TexturePacked => "MatCarTexturePacked",
            CarMaterial::TextureDull => "MatCarTextureDull",
            CarMaterial::TextureDullPacked => "MatCarTextureDullPacked",
            CarMaterial::TextureSwitchPacked => "MatCarTextureSwitchPacked",
            CarMaterial::Gouraud => "MatCarGouraud",
            CarMaterial::GouraudPacked => "MatCarGouraudPacked",
            CarMaterial::GouraudDull => "MatCarGouraudDull",
            CarMaterial::GouraudDullPacked => "MatCarGouraudDullPacked",
            CarMaterial::Glass => "MatCarGlass",
            CarMaterial::GlassPacked => "MatCarGlassPacked",
            CarMaterial::Chrome => "MatCarChrome",
            CarMaterial::Dash => "MatCarDash",
            CarMaterial::Shadow => "BondCarShadow",
        }
    }

    pub fn from_symbol(s: &str) -> Option<CarMaterial> { CarMaterial::ALL.into_iter().find(|m| m.symbol() == s) }

    /// The microcode draws a specular-map pass after the base pass [M bondrm.o: the block that
    /// reads `SpecularMat` (VU1 0x50) and `ReflectionInfo2.z`].
    pub fn specular_pass(self) -> bool {
        !matches!(
            self,
            CarMaterial::TextureDull
                | CarMaterial::TextureDullPacked
                | CarMaterial::TextureSwitchPacked
                | CarMaterial::GouraudDull
                | CarMaterial::GouraudDullPacked
                | CarMaterial::Shadow
        )
    }

    /// The microcode draws the reflection-map pass [M bondrm.o: the block that reads
    /// `ReflectionInfo2.x/.y` and `LightingProperties.w`]; every specular material but `Dash`.
    pub fn reflection_pass(self) -> bool { self.specular_pass() && self != CarMaterial::Dash }

    /// The base pass samples a texture (the `Texture` parameter slot) [M bondrm.o parameter names].
    pub fn textured(self) -> bool {
        !matches!(self, CarMaterial::Gouraud | CarMaterial::GouraudPacked | CarMaterial::GouraudDull | CarMaterial::GouraudDullPacked | CarMaterial::Shadow)
    }
}

/// One decoded strip of a car model, in the model's frame.
#[derive(Clone, Debug, PartialEq)]
pub struct CarStrip {
    /// The sheet entry name (the descriptor's +4 word, see `geoprim::ModelIndex::packet_texture`);
    /// `None` when the batch has no UVs or no texture was set yet in this model.
    pub texture: Option<String>,
    /// The render method of the strip's group entry ([`CarMaterial`]).
    pub material: CarMaterial,
    /// The CarRender class whose COORD4 is in VU1 `LightingProperties` when the strip is drawn: the
    /// `GAME::CarLighting<X>Props` the entry's render list uploads (op 5, 1 quadword to VU1 9)
    /// [M `FUN_002a3290` case 5 copies from the op's `+8` pointer; `FUN_001a8930` resolves the
    /// symbol]. An entry whose list uploads none keeps the previous entry's within the model (most
    /// lists after a model's first entry of a material are a bare op 4 [D]; that VU1 memory carries
    /// over is inferred, as for the texture). `None` for `Shadow` and before any upload.
    pub lighting: Option<CarLightClass>,
    pub positions: Vec<[f32; 3]>,
    /// Empty for untextured batches.
    pub uvs: Vec<[f32; 2]>,
    /// Unit normals (approximately), empty for positions-only batches.
    pub normals: Vec<[f32; 3]>,
    /// The `GeoPrimState` strings in VU1 when the strip is drawn, by VU1 quadword address (relative
    /// to the double-buffer base, as the `UNPACK` of each op 7 names it), without the
    /// `__EAGL::GeoPrimState:::STATE::` prefix [M `FUN_002a3290` case 7 copies the 0x80-byte state
    /// object behind the op's `+8` pointer; D: every car op 7 ends its VIF words with that
    /// `UNPACK`]. Carried over within the model like [`Self::lighting`] (inferred).
    pub states: BTreeMap<u16, String>,
    /// The module textures in VU1 by address (op 6 uploads of a module descriptor; `0x0a` is the
    /// base texture, `0x23` the glass-damage texture of `MatCarGlassPacked`) [M case 6; D]. The
    /// run-time `GAME::SpecularMap` / `GAME::ReflectionMap` uploads are not module textures and are
    /// not listed (see [`CarPassTexture`]). Carried over within the model.
    pub slot_textures: BTreeMap<u16, String>,
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
    /// Any other byte, with the item's `+0x0e`: a run-time transform (0xfc / 0xfb spin, 0xef dial,
    /// 0xf3 own matrix, 0xee camera-facing); `crate::carpose::part_matrix` evaluates them.
    /// [`CarPart::apply`] places the part with its own matrix only.
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
    /// The first 16 words of the `Base` payload as f32 (word 0 is the 0xffffffff placeholder;
    /// words 4-6 / 8-10 the model's box min / max, word 14 the value the wheel rig subtracts from)
    /// [D]; `None` when the article has no `Base` record.
    pub base: Option<[f32; 16]>,
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

    /// The parts the game draws for this vehicle in `state`: the list `FUN_00188200` selects
    /// ([`crate::carpose::selected_list`]) through [`CarMesh::drawn`] (a missing `vd` record draws
    /// every item) [M FUN_00188200 0x1883b4-0x18844c, FUN_001d2618, FUN_001c8ba8, FUN_001d35d8].
    pub fn list_for(&self, state: &crate::carpose::DrawState) -> Vec<&CarPart> { self.drawn(crate::carpose::selected_list(state)) }

    /// `Base +0x38 − (Base +0x24 − Base +0x14)`: the constant `FUN_001d2648` subtracts from a wheel
    /// item's origin y to get the wheel's rest offset [M 0x1d26a8-0x1d26d8, 0x1d27b4-0x1d27c0].
    /// That `*(car+0xc)` is this `Base` payload is read off the data (item 0's `as` payload starts
    /// with the `Base` tag on every car [D]), not traced through the loader.
    pub fn wheel_ref_height(&self) -> Option<f32> { self.base.map(|b| b[14] - (b[9] - b[5])) }
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

/// Decodes the VIF byte ranges of one car draw (a `draw_chain`) into strips drawn with `material`
/// (texture and lighting class left `None`) [D, see the module docs]. `label` names the batch in
/// errors.
pub fn decode_car_vif(img: &[u8], ranges: &[(usize, usize)], label: usize, material: CarMaterial) -> Result<Vec<CarStrip>> {
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
                                                let c = |k: usize| data[i * 3 + k] as f32 / NORMAL_SCALE - 1.0;
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
            material,
            lighting: None,
            positions: positions[w[0]..w[1]].to_vec(),
            uvs: if uvs.is_empty() { Vec::new() } else { uvs[w[0]..w[1]].to_vec() },
            normals: if normals.is_empty() { Vec::new() } else { normals[w[0]..w[1]].to_vec() },
            states: BTreeMap::new(),
            slot_textures: BTreeMap::new(),
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
pub fn car_model_strips(ix: &ModelIndex<'_>, ext: &HashMap<usize, String>, sym_value: u32) -> Result<Vec<CarStrip>> {
    let part = ix.model_part(sym_value)?;
    let mut out = Vec::new();
    let mut tex: Option<String> = None;
    let mut lighting: Option<CarLightClass> = None;
    let mut states: BTreeMap<u16, String> = BTreeMap::new();
    let mut slot_textures: BTreeMap<u16, String> = BTreeMap::new();
    for po in model_entries(ix, sym_value)? {
        if let Some(t) = ix.packet_texture(po, part) {
            tex = Some(t);
        }
        let material = entry_material(ix, ext, po)?;
        if let Some(c) = entry_lighting(ix, ext, po)? {
            lighting = Some(c);
        }
        for u in entry_uploads(ix, ext, po, part)? {
            match u.what {
                Upload::State(st) => {
                    states.insert(u.vu, st);
                }
                Upload::Texture(Some(t)) => {
                    slot_textures.insert(u.vu, t);
                }
                Upload::Texture(None) | Upload::ExternalTexture(_) | Upload::Symbol(_) => {
                    // A run-time texture or COORD4: the slot no longer holds a module texture.
                    slot_textures.remove(&u.vu);
                }
            }
        }
        for mut s in decode_car_vif(&ix.module.image, &ix.draw_chain(po, part)?, po, material)? {
            s.texture = if s.uvs.is_empty() { None } else { tex.clone() };
            s.lighting = if material == CarMaterial::Shadow { None } else { lighting };
            s.states = states.clone();
            s.slot_textures = slot_textures.clone();
            out.push(s);
        }
    }
    Ok(out)
}

/// The render method of group entry `po`: the external symbol at its GeoPrim header + 8
/// ([`CarMaterial`]); an error when the word names none or an unknown one.
pub fn entry_material(ix: &ModelIndex<'_>, ext: &HashMap<usize, String>, po: usize) -> Result<CarMaterial> {
    let hdr = ix.data_offset + u32_at(&ix.module.image, po)? as usize;
    let sym = ext.get(&(hdr + 8)).ok_or_else(|| FormatError::Invalid(format!("entry {po:#x}: header {hdr:#x} +8 names no render method")))?;
    CarMaterial::from_symbol(sym).ok_or_else(|| FormatError::Invalid(format!("entry {po:#x}: render method {sym} is not a car material")))
}

/// The `CarLighting<X>Props` class the entry's render list uploads, if any: an op 5 whose source
/// pointer (`+8`) is relocated against `__COORD4:::GAME::CarLighting<X>Props` [M `FUN_002a3290`
/// case 5: `puVar17 = puVar33[2]`, `*puVar32` doublewords copied]. Two different classes in one list
/// are an error (none occurs [D]).
pub fn entry_lighting(ix: &ModelIndex<'_>, ext: &HashMap<usize, String>, po: usize) -> Result<Option<CarLightClass>> {
    let mut found = None;
    for (at, o) in ix.render_ops(po)? {
        if o >> 24 != 5 {
            continue;
        }
        if let Some(c) = ext.get(&(at + 8)).and_then(|s| CarLightClass::from_props_symbol(s)) {
            if found.is_some_and(|f| f != c) {
                return invalid(format!("entry {po:#x}: render list uploads both {found:?} and {c:?}"));
            }
            found = Some(c);
        }
    }
    Ok(found)
}

/// What one op 5 / 6 / 7 of a render list puts into VU1.
#[derive(Clone, Debug, PartialEq)]
pub enum Upload {
    /// Op 7: a `__EAGL::GeoPrimState:::STATE::…` object (the string after the prefix).
    State(String),
    /// Op 6: a module texture descriptor (its sheet entry name; `None` when no named descriptor is
    /// at that address).
    Texture(Option<String>),
    /// Op 6 of an external descriptor (`__EAGL::TAR:::GAME::SpecularMap`, `GAME::ReflectionMap`).
    ExternalTexture(String),
    /// Op 5: the external symbol it copies from (`__COORD4:::GAME::CarLightingGlossyProps`, the
    /// view matrices), or `""` for a module pointer.
    Symbol(String),
}

/// One upload of a render list with its VU1 quadword address.
#[derive(Clone, Debug, PartialEq)]
pub struct EntryUpload {
    pub vu: u16,
    pub what: Upload,
}

/// The `__EAGL::GeoPrimState:::STATE::` prefix of the state symbols car modules import.
pub const STATE_PREFIX: &str = "__EAGL::GeoPrimState:::STATE::";

/// The VU1 uploads of group entry `po`'s render list, in list order [M `FUN_002a3290` cases 5-7:
/// `n` VIF words at `+0xc` go into the DMA chain ahead of the data copied from the `+8` pointer;
/// case 6 adds `KK × part × 0x60` to that pointer]. The address is the last VIF word's, which must
/// be an `UNPACK` (an error otherwise) [D: true for every op 5-7 of the car modules].
pub fn entry_uploads(ix: &ModelIndex<'_>, ext: &HashMap<usize, String>, po: usize, part: u32) -> Result<Vec<EntryUpload>> {
    let img = &ix.module.image;
    let mut out = Vec::new();
    for (at, o) in ix.render_ops(po)? {
        let op = o >> 24;
        if !(5..=7).contains(&op) {
            continue;
        }
        let n = (o & 0xffff) as usize;
        if n == 0 {
            return invalid(format!("entry {po:#x}: op {o:#x} at {at:#x} has no VIF words"));
        }
        let unpack = u32_at(img, at + 8 + 4 * n)?;
        if (unpack >> 24) & 0x60 != 0x60 {
            return invalid(format!("entry {po:#x}: op {o:#x} at {at:#x} ends in {unpack:#x}, not an UNPACK"));
        }
        let vu = (unpack & 0x3ff) as u16;
        let sym = ext.get(&(at + 8));
        let what = match op {
            7 => match sym.and_then(|s| s.strip_prefix(STATE_PREFIX)) {
                Some(st) => Upload::State(st.to_owned()),
                None => return invalid(format!("entry {po:#x}: op 7 at {at:#x} names no GeoPrimState ({sym:?})")),
            },
            6 => match sym {
                Some(x) => Upload::ExternalTexture(x.clone()),
                None => {
                    let k = (o >> 16) & 0xff;
                    let w = u32_at(img, at + 8)?.wrapping_add(k.wrapping_mul(part).wrapping_mul(0x60));
                    Upload::Texture(ix.texture_name(w).map(str::to_owned))
                }
            },
            _ => Upload::Symbol(sym.cloned().unwrap_or_default()),
        };
        out.push(EntryUpload { vu, what });
    }
    Ok(out)
}

/// The GS registers an EAGL `GeoPrimState` string sets, as `FUN_001ce158` 0x1ce158 builds them
/// (docs/formats/car-model.md §10.6). The 0x80-byte state object is a GS packet: `+0x10` PRIM,
/// `+0x30` TEST_1, `+0x40` ALPHA_1 (then FRAME_1, ZBUF_1, FBA_1) [M `FUN_00283568`: register
/// addresses 0x47 / 0x42 / 0x4c / 0x4e / 0x4a beside the data words]. Only these three words are
/// modelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeoPrimState {
    pub prim: u64,
    pub test: u64,
    pub alpha: u64,
}

/// The GS `ALPHA` blend `((A − B) · C >> 7) + D`: A, B, D ∈ {0 Cs, 1 Cd, 2 zero}, C ∈ {0 As, 1 Ad,
/// 2 FIX} (GS register layout: A bits 0-1, B 2-3, C 4-5, D 6-7, FIX 32-39).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GsBlend {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub fix: u8,
}

impl GsBlend {
    pub fn from_alpha(reg: u64) -> GsBlend {
        GsBlend { a: (reg & 3) as u8, b: ((reg >> 2) & 3) as u8, c: ((reg >> 4) & 3) as u8, d: ((reg >> 6) & 3) as u8, fix: (reg >> 32) as u8 }
    }

    /// One channel: `cs` / `cd` colours and `as_` / `ad` alphas in GS units (0x80 = 1.0 for the
    /// alphas). Not clamped (the COLCLAMP setting is not traced).
    pub fn apply(&self, cs: f32, as_: f32, cd: f32, ad: f32) -> f32 {
        let pick = |k: u8| match k {
            0 => cs,
            1 => cd,
            _ => 0.0,
        };
        let c = match self.c {
            0 => as_,
            1 => ad,
            _ => self.fix as f32,
        };
        (pick(self.a) - pick(self.b)) * c / 128.0 + pick(self.d)
    }
}

/// `primtype=` values → GS PRIM type [M `FUN_001cd448` table 0x386ee0 → `FUN_00283fc8` byte table
/// 0x3483b0 = 00 01 02 ff 03 04 05 ff ff ff 06; 0xff = no change].
const PRIM_TYPES: &[(&str, u64)] =
    &[("pointlist", 0), ("linelist", 1), ("lineloop", 0xff), ("trilist", 3), ("tristrip", 4), ("trifan", 5), ("quadlist", 0xff), ("quadstrip", 0xff)];

impl GeoPrimState {
    /// The state before the string is applied [M `FUN_00283568` + `FUN_001ce158`]: PRIM = triangle
    /// (`FUN_00283fc8(4)`) | IIP | TME | FGE | ABE; TEST = ZTE, ZTST GEQUAL, and the low 14 bits
    /// 0x3a0d (ATE, ATST GREATER, AREF 0xa0, AFAIL RGB_ONLY); ALPHA = 0x80_0000_0044
    /// ((Cs − Cd)·As + Cd).
    pub const DEFAULT: GeoPrimState = GeoPrimState { prim: 0x7b, test: 0x5_3a0d, alpha: 0x80_0000_0044 };

    /// Applies `s` (with or without [`STATE_PREFIX`]) the way `FUN_001ce158` does: every
    /// `key=value` token in order (`FUN_001ce3b0`, handler table 0x333f58), then once more every
    /// token whose handler returned non-zero (of the handlers below only `modify=specmap`).
    /// Handlers [M]:
    /// - `primtype` (`FUN_001cd448`): PRIM bits 0-2 from `PRIM_TYPES`;
    /// - `texture` (`FUN_001cd520`): `off` clears TME, `on` sets it;
    /// - `alphatest` (`FUN_001cd7f8`): TEST.ATE = on;
    /// - `texalpha` (`FUN_001cd850`), only while TEST.ATE is set: `noA` clears ATE and ABE; `deepA`
    ///   (1) / `1bitA` (0x70) set AREF to that value, AFAIL = 3 and ABE. Its `ALPHA = 0x44` write is
    ///   guarded by the caller's `$s1`, the non-null string cursor of `FUN_001ce3b0`, so it never
    ///   runs;
    /// - `cull` (`FUN_001cd4f8`): looks the value up and changes nothing;
    /// - `modify` (`FUN_001cd938`, table 0x387250): `specmap` (`FUN_001cdd18`, returns 1) and
    ///   `glaregen` (`FUN_001cdd58`) clear ABE and ATE; `shadowmap` (`FUN_001cdd80`) sets PRIM
    ///   IIP|TME|ABE, TEST low bits 0x315d and ALPHA 0x44; `chromemap`, `glossmap`, `envmap`,
    ///   `watermap`, `glassdmgmap`, `bumpmap` return 0 and change nothing;
    /// - `name` (`FUN_001cdc80`, table 0x3872a0): `envmap` (`FUN_001cda30`): ALPHA = 0x48
    ///   (Cs·As + Cd), PRIM bits 3-6 = IIP|TME|ABE (fog off), ATE on; `specmap` (`FUN_001cdad0`):
    ///   ALPHA = 0x58 (Cs·Ad + Cd), the same PRIM, ATE off; `glossmap` (`FUN_001cd9c0`): PRIM bits
    ///   3, 4, 6 = IIP|TME (ABE off), ATE off; `glassdmgmap` (`FUN_001cdc08`): ATE on, PRIM bits
    ///   3, 4, 6 = IIP|TME|ABE, ALPHA 0x44; `bumpmap` (`FUN_001cdc78`) nothing. The `FUN_00284038`
    ///   calls of `envmap` / `specmap` (5) and `glossmap` (4) hit −1 entries of table 0x3483c8: no
    ///   ZTST change.
    ///
    /// Any other key or value is an error: those handlers (`shading`, `depthtest`, `watermap`, …)
    /// are not modelled.
    pub fn parse(s: &str) -> Result<GeoPrimState> {
        let s = s.strip_prefix(STATE_PREFIX).unwrap_or(s);
        let mut st = GeoPrimState::DEFAULT;
        let toks: Vec<(&str, &str)> = s
            .split(';')
            .filter(|t| !t.is_empty())
            .map(|t| t.split_once('=').ok_or_else(|| FormatError::Invalid(format!("GeoPrimState token {t:?} has no '='"))))
            .collect::<Result<_>>()?;
        let mut again = Vec::new();
        for &(k, v) in &toks {
            if st.apply(k, v)? {
                again.push((k, v));
            }
        }
        for (k, v) in again {
            st.apply(k, v)?;
        }
        Ok(st)
    }

    fn apply(&mut self, key: &str, value: &str) -> Result<bool> {
        const ABE: u64 = 0x40;
        let bad = || FormatError::Invalid(format!("GeoPrimState {key}={value} not modelled"));
        match key {
            "primtype" => {
                let t = PRIM_TYPES.iter().find(|(n, _)| *n == value).map(|&(_, t)| t).ok_or_else(bad)?;
                if t != 0xff {
                    self.prim = (self.prim & !7) | t;
                }
            }
            "texture" => match value {
                "off" => self.prim &= !0x10,
                "on" => self.prim |= 0x10,
                _ => return Err(bad()),
            },
            "alphatest" => match value {
                "off" => self.test &= !1,
                "on" => self.test |= 1,
                _ => return Err(bad()),
            },
            "texalpha" => {
                let aref: u64 = match value {
                    "noA" => 0,
                    "deepA" => 1,
                    "1bitA" => 0x70,
                    _ => return Err(bad()),
                };
                if self.test & 1 != 0 {
                    if aref == 0 {
                        self.test &= !1;
                        self.prim &= !ABE;
                    } else {
                        self.test = (((self.test & !0xff0) | (aref << 4)) & !0x3000) | 0x3000;
                        self.prim |= ABE;
                    }
                }
            }
            "cull" => {
                if !matches!(value, "off" | "on") {
                    return Err(bad());
                }
            }
            "modify" => match value {
                "specmap" => {
                    self.prim &= !ABE;
                    self.test &= !1;
                    return Ok(true);
                }
                "glaregen" => {
                    self.prim &= !ABE;
                    self.test &= !1;
                }
                "shadowmap" => {
                    self.prim = (self.prim & !0x78) | 0x58;
                    self.test = (self.test & !0x3fff) | 0x315d;
                    self.alpha = 0x80_0000_0044;
                }
                "chromemap" | "glossmap" | "envmap" | "watermap" | "glassdmgmap" | "bumpmap" => {}
                _ => return Err(bad()),
            },
            "name" => match value {
                "envmap" => {
                    self.alpha = 0x80_0000_0048;
                    self.prim = (self.prim & !0x78) | 0x58;
                    self.test |= 1;
                }
                "specmap" => {
                    self.alpha = 0x80_0000_0058;
                    self.prim = (self.prim & !0x78) | 0x58;
                    self.test &= !1;
                }
                "glossmap" => {
                    self.prim = (self.prim & !0x58) | 0x18;
                    self.test &= !1;
                }
                "glassdmgmap" => {
                    self.test |= 1;
                    self.prim = (self.prim & !0x58) | 0x58;
                    self.alpha = 0x80_0000_0044;
                }
                "bumpmap" => {}
                _ => return Err(bad()),
            },
            _ => return Err(bad()),
        }
        Ok(false)
    }

    /// GS PRIM type (3 = triangle list, 4 = triangle strip).
    pub fn prim_type(&self) -> u8 { (self.prim & 7) as u8 }
    /// Gouraud shading (PRIM.IIP).
    pub fn iip(&self) -> bool { self.prim & 0x08 != 0 }
    /// Texture mapping (PRIM.TME).
    pub fn tme(&self) -> bool { self.prim & 0x10 != 0 }
    /// Fog (PRIM.FGE).
    pub fn fge(&self) -> bool { self.prim & 0x20 != 0 }
    /// Alpha blending (PRIM.ABE); when off the GS writes the source colour.
    pub fn abe(&self) -> bool { self.prim & 0x40 != 0 }
    /// `(ATST, AREF, AFAIL)` when TEST.ATE is set.
    pub fn alpha_test(&self) -> Option<(u8, u8, u8)> {
        (self.test & 1 != 0).then(|| (((self.test >> 1) & 7) as u8, ((self.test >> 4) & 0xff) as u8, ((self.test >> 12) & 3) as u8))
    }
    /// The blend equation, `None` when ABE is off.
    pub fn blend(&self) -> Option<GsBlend> { self.abe().then(|| GsBlend::from_alpha(self.alpha)) }
}

/// The GS draws a car strip sends, in microcode order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CarPassKind {
    /// `MatCarGlassPacked`'s first draw, with the `0x23` texture (the damage map).
    GlassDamage,
    Base,
    Specular,
    Reflection,
}

/// The texture a pass samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CarPassTexture {
    /// The module texture uploaded to that VU1 address ([`CarStrip::slot_textures`]).
    Module(u16),
    /// `GAME::SpecularMap` = RReflection `+0xd0` = entry `carS` (32 × 32) of the sheet registered as
    /// key 0, `data\render\common.ssh` [M `FUN_001c0930`: `FUN_001cf340(FUN_001d0420(0),
    /// 0x53726163)`; `FUN_001a8930` resolves the name through `FUN_001c1ad8`; `FUN_001c4008`
    /// registers `data\render\` + `common.ssh` as key 0; D: `carS` is in every archive's
    /// common.ssh]. See [`SPECULAR_MAP_ENTRY`].
    SpecularMap,
    /// `GAME::ReflectionMap` = RReflection `+0xbc`, a 256 × 256 render target (`FUN_0027ed78(0x100,
    /// 0x100, 0x10)`) the reflection view draws each frame (`FUN_001c1ec0`, see
    /// `rendertun::CarRender::reflection_layers`) [M].
    ReflectionMap,
    /// No texture (the Gouraud materials).
    Untextured,
}

/// The `common.ssh` entry the specular pass samples ([`CarPassTexture::SpecularMap`]).
pub const SPECULAR_MAP_ENTRY: &str = "carS";

/// One pass: what the microcode kicks before the strip's vertices.
#[derive(Clone, Debug, PartialEq)]
pub struct CarPass {
    pub kind: CarPassKind,
    /// The VU1 address of the GeoPrimState packet kicked first.
    pub state_vu: u16,
    pub texture: CarPassTexture,
    /// The state string in that slot ([`CarStrip::states`]); `None` when this model uploaded none
    /// before the strip (the GS keeps an earlier draw's state, not traced).
    pub state: Option<String>,
}

/// The vertex alpha of every car pass: the `ColourSlot` w the set-up loop fills with FTOI0(127.0)
/// and no pass rewrites (the passes store `.xyz` only) [M bondrm.o MatCarTexturePacked 0x00e-0x018,
/// 0x099 `MOVEw` keeps it, 0x227 / 0x269 `SQIxyz`].
pub const CAR_VERTEX_ALPHA: u8 = 127;

/// `C_w` of an undamaged vertex: the `ColourSlot` w (127) the damage pass scales by
/// `1 − DmgVec.w · zone.z` (docs/formats/car-model.md §10.5; the zone index is not followed).
pub const UNDAMAGED_CW: f32 = 127.0;

/// A pass vertex: the ST the microcode stores (Q = 1) and the R = G = B value it FTOI0s (alpha is
/// [`CAR_VERTEX_ALPHA`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarPassVertex {
    pub st: [f32; 2],
    pub rgb: u32,
}

fn row_mul(v: [f32; 3], rows: &[[f32; 3]; 3]) -> [f32; 3] {
    let mut o = [0.0; 3];
    for (k, r) in rows.iter().enumerate() {
        for j in 0..3 {
            o[j] += v[k] * r[j];
        }
    }
    o
}

/// The specular pass of one vertex [M bondrm.o MatCarTexturePacked 0x1f4-0x229]: the program forms
/// `M = W · SpecularMat` (the WorldMatrix rows, not normalised, × the four SpecularMat rows) scaled
/// by `SpecularSpot` (`ReflectionInfo2.z`), then per vertex `ST = (0.5, 0.5) + (n · M).xy`, Q = 1,
/// and `RGB = FTOI0(Specular · C_w)` (`LightingProperties.z` × the damage-scaled `C_w`).
/// `world` = the WorldMatrix rotation rows (row vectors, `p_world = p · W`); `basis` =
/// [`crate::rendertun::specular_basis`] `[b, c, a]`, SpecularMat row i = `(b_i, c_i, a_i, 0)`, so
/// `(n_w · S).xy = (n_w · b, n_w · c)`; `n` = the model normal.
pub fn specular_vertex(n: [f32; 3], world: &[[f32; 3]; 3], basis: &[[f32; 3]; 3], spot: f32, specular: f32, c_w: f32) -> CarPassVertex {
    let nw = row_mul(n, world);
    let d = |v: [f32; 3]| nw[0] * v[0] + nw[1] * v[1] + nw[2] * v[2];
    CarPassVertex { st: [0.5 + spot * d(basis[0]), 0.5 + spot * d(basis[1])], rgb: (specular * c_w).max(0.0) as u32 }
}

/// The reflection pass of one vertex [M bondrm.o MatCarTexturePacked 0x23c-0x269]: `v = Warp ·
/// ((n + Fresnel · p) · W)` (W rotation rows unnormalised, `p` the model position after the damage
/// offset), `ST = (0.5 + v.x, 0.5 + v.z)`, `RGB = FTOI0(min(Reflect · C_w, 100000 · max(0, v.y)))`.
pub fn reflection_vertex(n: [f32; 3], p: [f32; 3], world: &[[f32; 3]; 3], fresnel: f32, warp: f32, reflect: f32, c_w: f32) -> CarPassVertex {
    let q = [n[0] + fresnel * p[0], n[1] + fresnel * p[1], n[2] + fresnel * p[2]];
    let v = row_mul(q, world).map(|c| c * warp);
    let rgb = (reflect * c_w).min(100_000.0 * v[1].max(0.0)).max(0.0);
    CarPassVertex { st: [0.5 + v[0], 0.5 + v[2]], rgb: rgb as u32 }
}

impl CarMaterial {
    /// The passes the microcode draws: per pass the GeoPrimState and texture slots its `XGKICK`s
    /// send before the vertex buffer [M bondrm.o, the `IADDIU vi02, vi14, N` + `XGKICK vi02` pairs]:
    /// - `MatCarTexture`: 0xfe+0x0a, 0x106+0x11, 0x106+0x18;
    /// - `MatCarTexturePacked`, `MatCarChrome`: 0x10f+0x0a, 0x107+0x11, 0x107+0x18;
    /// - `MatCarTextureDull` 0xf0+0x0a, `TextureDullPacked` 0xf9+0x0a, `TextureSwitchPacked`
    ///   0xfb+0x0a, `GouraudDull` 0x11e, `GouraudDullPacked` 0xc6 (base only);
    /// - `MatCarDash`: 0xfe+0x0a, 0x106+0x11 (no reflection);
    /// - `MatCarGlassPacked`: 0xfd+0x23, 0xfd+0x0a, 0x105+0x11, 0x105+0x18 (its lists also upload a
    ///   state to 0xf5, which the program never kicks; the branches around the first draw are not
    ///   followed).
    ///
    /// 0x11 is the `SpecularMap` slot and 0x18 `ReflectionMap2` (§10.1); the car lists upload
    /// `GAME::SpecularMap` / `GAME::ReflectionMap` there [D]. `None` for `Glass`, `Gouraud[Packed]`
    /// (slot names not read; no car strip uses them [D]) and `Shadow` (not traced).
    pub fn pass_slots(self) -> Option<&'static [(CarPassKind, u16, CarPassTexture)]> {
        use CarPassKind::*;
        use CarPassTexture::*;
        Some(match self {
            CarMaterial::Texture => &[(Base, 0xfe, Module(0x0a)), (Specular, 0x106, SpecularMap), (Reflection, 0x106, ReflectionMap)],
            CarMaterial::TexturePacked | CarMaterial::Chrome => {
                &[(Base, 0x10f, Module(0x0a)), (Specular, 0x107, SpecularMap), (Reflection, 0x107, ReflectionMap)]
            }
            CarMaterial::TextureDull => &[(Base, 0xf0, Module(0x0a))],
            CarMaterial::TextureDullPacked => &[(Base, 0xf9, Module(0x0a))],
            CarMaterial::TextureSwitchPacked => &[(Base, 0xfb, Module(0x0a))],
            CarMaterial::GouraudDull => &[(Base, 0x11e, Untextured)],
            CarMaterial::GouraudDullPacked => &[(Base, 0xc6, Untextured)],
            CarMaterial::Dash => &[(Base, 0xfe, Module(0x0a)), (Specular, 0x106, SpecularMap)],
            CarMaterial::GlassPacked => &[
                (GlassDamage, 0xfd, Module(0x23)),
                (Base, 0xfd, Module(0x0a)),
                (Specular, 0x105, SpecularMap),
                (Reflection, 0x105, ReflectionMap),
            ],
            CarMaterial::Glass | CarMaterial::Gouraud | CarMaterial::GouraudPacked | CarMaterial::Shadow => return None,
        })
    }
}

impl CarStrip {
    /// The strip's passes with the states in their slots ([`CarMaterial::pass_slots`]).
    pub fn passes(&self) -> Vec<CarPass> {
        self.material
            .pass_slots()
            .unwrap_or(&[])
            .iter()
            .map(|&(kind, state_vu, texture)| CarPass { kind, state_vu, texture, state: self.states.get(&state_vu).cloned() })
            .collect()
    }

    /// Per-vertex ST and colour of a specular or reflection pass for this strip placed with
    /// WorldMatrix rotation rows `world` ([`specular_vertex`] / [`reflection_vertex`], undamaged:
    /// `C_w` = [`UNDAMAGED_CW`]). `props` = the strip's CarRender class values. Empty for the base
    /// and glass-damage passes (their ST are the strip UVs) and for strips without normals.
    pub fn pass_vertices(
        &self,
        kind: CarPassKind,
        world: &[[f32; 3]; 3],
        basis: &[[f32; 3]; 3],
        entry: &crate::rendertun::CarRenderEntry,
        props: crate::rendertun::CarMaterialProps,
    ) -> Vec<CarPassVertex> {
        if self.normals.len() != self.positions.len() {
            return Vec::new();
        }
        match kind {
            CarPassKind::Specular => {
                self.normals.iter().map(|&n| specular_vertex(n, world, basis, entry.specular_spot, props.specular, UNDAMAGED_CW)).collect()
            }
            CarPassKind::Reflection => self
                .normals
                .iter()
                .zip(&self.positions)
                .map(|(&n, &p)| reflection_vertex(n, p, world, entry.fresnel, entry.reflection_warp, props.reflect, UNDAMAGED_CW))
                .collect(),
            _ => Vec::new(),
        }
    }
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
        } else if r.tag == TAG_BASE {
            let p = crp.payload(&r)?;
            if p.len() >= 0x40 {
                let mut b = [0f32; 16];
                for (k, w) in b.iter_mut().enumerate() {
                    *w = f32_at(p, 4 * k)?;
                }
                mesh.base = Some(b);
            }
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
    let ext = module.external_refs()?;
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
                cache.insert(sym, car_model_strips(&ix, &ext, sym)?);
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
    /// V3-8 normals, two V4-16 arrays (DmgVec, DmgNrm), MSCNT — the vanquish shape [D].
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
        let s = decode_car_vif(&b, &[(0, b.len())], 0, CarMaterial::TexturePacked).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].positions, pos[..3].to_vec());
        assert_eq!(s[1].positions, pos[3..].to_vec());
        assert_eq!(s[0].uvs[1], [1.0, 0.0]);
        assert_eq!(s[1].uvs[1], [0.5, 0.5]);
        // byte / 127.5 − 1 (the microcode's rule): 0xff → 1, 0x80 → 1/255, 0x01 → −253/255. The
        // old (byte − 128) / 127 reading gave exactly 0 for 0x80 and −1 for 0x01.
        let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-6);
        assert!(near(s[0].normals[0], [1.0 / 255.0, 1.0 / 255.0, 1.0]), "{:?}", s[0].normals[0]);
        assert!(near(s[0].normals[2], [-253.0 / 255.0, 1.0 / 255.0, 1.0 / 255.0]), "{:?}", s[0].normals[2]);
        assert!(s.iter().all(|t| t.material == CarMaterial::TexturePacked && t.lighting.is_none()));
        // The track reader takes these bytes as colours and the UVs at /256 (0x8400 → 4.0).
        let t = crate::geoprim::decode_vif(&b, &[(0, b.len())], 0).unwrap();
        assert_eq!(t[0].colors[0], [0x80, 0x80, 0xff]);
        assert_eq!(t[0].uvs[1], [4.0, 0.0]);
        // A list byte that is not a multiple of 3 is refused.
        let bad = batch_a(&pos, [10, 0, 0, 0], &uv, &nrm);
        assert!(decode_car_vif(&bad, &[(0, bad.len())], 0, CarMaterial::TexturePacked).is_err());
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

    #[test]
    fn geoprim_state_strings_give_the_traced_gs_registers() {
        // The specular / reflection state of the paint, chrome and glass passes.
        let env = GeoPrimState::parse("__EAGL::GeoPrimState:::STATE::name=envmap;primtype=tristrip").unwrap();
        assert_eq!((env.prim, env.alpha), (0x5c, 0x80_0000_0048));
        assert!(env.iip() && env.tme() && env.abe() && !env.fge() && env.prim_type() == 4);
        assert_eq!(env.alpha_test(), Some((6, 0xa0, 3)));
        let b = env.blend().unwrap();
        assert_eq!((b.a, b.b, b.c, b.d), (0, 2, 0, 1));
        // Cs·As + Cd: additive, scaled by the source alpha, not a lerp.
        assert_eq!(b.apply(100.0, 64.0, 50.0, 0.0), 100.0);
        assert_eq!(b.apply(100.0, 128.0, 200.0, 0.0), 300.0);
        // The dash specular state: Cs·Ad + Cd, scaled by the frame buffer's alpha.
        let spec = GeoPrimState::parse("name=specmap;primtype=tristrip").unwrap();
        assert_eq!(spec.alpha, 0x80_0000_0058);
        assert!(spec.abe() && spec.alpha_test().is_none());
        let b = spec.blend().unwrap();
        assert_eq!((b.apply(100.0, 128.0, 50.0, 0.0), b.apply(100.0, 0.0, 50.0, 64.0)), (50.0, 100.0));
        // The dash base: modify=glaregen/specmap turn blending and the alpha test off, so the later
        // texalpha=deepA (only applied while the test is on) changes nothing.
        let dash = GeoPrimState::parse("modify=bumpmap;modify=glaregen;modify=specmap;texalpha=deepA;primtype=tristrip").unwrap();
        assert_eq!((dash.prim, dash.test & 0x3fff), (0x3c, 0x3a0c));
        assert!(dash.blend().is_none() && dash.fge());
        // The paint base: alphatest=off first, so texalpha=noA is skipped and ABE stays on.
        let paint = GeoPrimState::parse("modify=envmap;alphatest=off;texalpha=noA;primtype=tristrip").unwrap();
        assert_eq!((paint.prim, paint.alpha), (0x7c, 0x80_0000_0044));
        assert!(paint.alpha_test().is_none());
        // With the test on, texalpha=noA turns blending off; deepA sets AREF 1, AFAIL RGB_ONLY.
        assert!(!GeoPrimState::parse("texalpha=noA;primtype=tristrip").unwrap().abe());
        let deep = GeoPrimState::parse("texalpha=deepA;primtype=tristrip").unwrap();
        assert_eq!(deep.alpha_test(), Some((6, 1, 3)));
        assert!(!GeoPrimState::parse("texture=off;primtype=tristrip").unwrap().tme());
        assert_eq!(GeoPrimState::parse("name=envmap;primtype=trilist").unwrap().prim_type(), 3);
        // Handlers that are not modelled are refused, not ignored.
        assert!(GeoPrimState::parse("shading=flat").is_err());
        assert!(GeoPrimState::parse("name=watermap").is_err());
    }

    #[test]
    fn the_specular_and_reflection_passes_map_normals_as_the_microcode_does() {
        let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        // Light straight up, view along −z: a = (0, .707, .707), b = (1, 0, 0), c = (0, .707, −.707).
        let basis = crate::rendertun::specular_basis([0.0, 1.0, 0.0], [0.0, 0.0, -1.0]);
        let h = std::f32::consts::FRAC_1_SQRT_2;
        // A normal along the half vector samples the map centre.
        let v = specular_vertex([0.0, h, h], &id, &basis, 1.5, 1.0, UNDAMAGED_CW);
        assert!((v.st[0] - 0.5).abs() < 1e-6 && (v.st[1] - 0.5).abs() < 1e-6, "{v:?}");
        assert_eq!(v.rgb, 127);
        // +x moves s by +Spot; a flipped cross product would give 0.5 − 1.5.
        let v = specular_vertex([1.0, 0.0, 0.0], &id, &basis, 1.5, 0.8, UNDAMAGED_CW);
        assert!((v.st[0] - 2.0).abs() < 1e-5 && (v.st[1] - 0.5).abs() < 1e-5, "{v:?}");
        assert_eq!(v.rgb, 101);
        // Straight up: t = 0.5 + Spot · 0.707.
        let v = specular_vertex([0.0, 1.0, 0.0], &id, &basis, 1.0, 1.0, UNDAMAGED_CW);
        assert!((v.st[1] - (0.5 + h)).abs() < 1e-5, "{v:?}");
        // A model turned 90° about y (row-vector W): the model +z normal is world +x.
        let turn = [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let v = specular_vertex([0.0, 0.0, 1.0], &turn, &basis, 1.5, 1.0, UNDAMAGED_CW);
        assert!((v.st[0] - 2.0).abs() < 1e-5, "{v:?}");

        // Reflection: an upward normal at the origin looks at the map centre at full strength.
        let r = reflection_vertex([0.0, 1.0, 0.0], [0.0; 3], &id, 0.21, 0.37, 0.8, UNDAMAGED_CW);
        assert_eq!((r.st, r.rgb), ([0.5, 0.5], 101));
        // Position pulls the lookup by Fresnel · p, scaled by Warp: s = 0.5 + 0.37·(0.21·2).
        let r = reflection_vertex([0.0, 1.0, 0.0], [2.0, 0.0, -1.0], &id, 0.21, 0.37, 0.8, UNDAMAGED_CW);
        assert!((r.st[0] - (0.5 + 0.37 * 0.42)).abs() < 1e-6 && (r.st[1] - (0.5 - 0.37 * 0.21)).abs() < 1e-6, "{r:?}");
        // Faces pointing down get none; a grazing one (v.y = 1e-6) is clamped by 1e5 · v.y.
        assert_eq!(reflection_vertex([0.0, -1.0, 0.0], [0.0; 3], &id, 0.21, 0.37, 0.8, UNDAMAGED_CW).rgb, 0);
        let g = reflection_vertex([1.0, 5.05e-4 / 0.37, 0.0], [0.0; 3], &id, 0.0, 0.37, 0.8, UNDAMAGED_CW);
        assert_eq!(g.rgb, 50);
    }

    #[test]
    fn each_material_kicks_its_traced_state_and_texture_slots() {
        let k = |m: CarMaterial| m.pass_slots().unwrap().iter().map(|&(k, s, t)| (k, s, t)).collect::<Vec<_>>();
        use CarPassKind::*;
        use CarPassTexture::*;
        assert_eq!(k(CarMaterial::TexturePacked), vec![(Base, 0x10f, Module(0x0a)), (Specular, 0x107, SpecularMap), (Reflection, 0x107, ReflectionMap)]);
        assert_eq!(k(CarMaterial::Chrome), k(CarMaterial::TexturePacked));
        assert_eq!(k(CarMaterial::Dash), vec![(Base, 0xfe, Module(0x0a)), (Specular, 0x106, SpecularMap)]);
        assert_eq!(k(CarMaterial::TextureDullPacked), vec![(Base, 0xf9, Module(0x0a))]);
        assert_eq!(k(CarMaterial::GlassPacked)[0], (GlassDamage, 0xfd, Module(0x23)));
        // pass_slots agrees with the older specular_pass / reflection_pass flags where both exist.
        for m in CarMaterial::ALL {
            if let Some(p) = m.pass_slots() {
                assert_eq!(p.iter().any(|x| x.0 == Specular), m.specular_pass(), "{m:?}");
                assert_eq!(p.iter().any(|x| x.0 == Reflection), m.reflection_pass(), "{m:?}");
            }
        }
    }
}
