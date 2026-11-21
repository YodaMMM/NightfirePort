//! The driving tracks' world geometry — roads, ground, buildings, props and sky — decoded from the
//! track `.crp` into world-space triangle strips (docs/formats/geoprim.md).
//!
//! There is **no run-time road generator**. MISC.VIV's `data\render\eaglrm.o` holds only a
//! `.data` section (VU1 microcode `*0__EAGLMicroCode` and the render-method tables `LitTexture`,
//! `ClipTexture`, … — 113 symbols, no executable section [D]). The road tiles are ordinary EAGL
//! models baked into the track's embedded module (`ELFd`+`ELFr`, `crp_mesh::TrackModule`); an EAGL
//! "GeoPrim" is one VIF packet of such a model (`__GPRenderMethod_<article>::…_name` header +
//! `__EAGL::GeoPrimState:::…primtype=tristrip` state). The chain this module walks:
//!
//! 1. **Placements** — the `Map ` article's `in#0` record: `count` items of 0x40 bytes
//!    [M `FUN_0022faf8` 0x22faf8, "Init World": `FUN_002d5920(map, 'in  ')`, the item walk steps
//!    `+0x40`, the count is the record's `+8`]. Each item is an affine matrix whose fourth words
//!    carry metadata: the game copies words 0-2, 4-6, 8-10, 12-14, zeroes words 3/7/11 and sets
//!    word 15 to 1.0f before handing the matrix to the instance (`FUN_001c9258`) [M 0x22faf8,
//!    the `uStack_100..uStack_c8` copy]. So `world = x·axis0 + y·axis1 + z·axis2 + origin` with
//!    axis*i* = words 4i..4i+2 and origin = words 12-14. +0x0c byte = flags, +0x0e u16 = `ps#0`
//!    index, +0x1c = the reference word (`'sr'` short tag + index into the Map's `sr#N` strings,
//!    resolved by the loader into a pointer) [M 0x22faf8 lines 195-226, 0x1b1988].
//!    The forward direction (not the inverse) is also measured [D]: with it a decoded surface lies
//!    within 1 m of 772/880 of paris's `RNgp` road nodes, and it agrees with the `CDat` collision
//!    surface within 5 cm at 730 of the 739 nodes where both exist (docs/formats/geoprim.md §4).
//! 2. **Part → model** — the Map `sr` string is `CARP::<article>::{as  NNNN}`; the article's own
//!    window holds `as#NNNN`, whose payload ends in `{u16 index, "sr"}` [D, crp_mesh.rs], naming the
//!    article's `sr#index` string `EAGL::<model article>::{as  0000}::Model[0].XX`; the module
//!    symbol `__Model:::<model article>::{as  0000}::Model[0].XX` is the model object [D].
//! 3. **Model object** (offsets from the symbol's `.data` address): `+0xc8` u32 group count,
//!    `+0xcc` pointer to the packet table — `{0xA0000000, 0, n}` followed by `n/2` pairs
//!    `{0xA000FFFF, packet}`, or bare `{0xA000FFFF, packet}` entries [D]. Pointers in the relocated
//!    image are `.data`-relative (every relocation is `R_MIPS_32` against the `.data` section
//!    symbol, crp_mesh.rs) [D].
//! 4. **Packet** — `+0x00` pointer to the GeoPrim header (whose `+0x00` is the render-method
//!    instruction list; its first `0x0300000N` word is followed by the `__EAGL::TAR` texture
//!    descriptor pointer, the texture name is the symbol's `TEX0::<name>` tail [D]), `+0x08` the
//!    DMA tag (low 16 bits = quadword count), `+0x10` the VIF stream [D]. One batch per packet
//!    (1,778 of 1,779 paris packets: one `MSCNT`, one header; the other is empty) [D]:
//!    - `UNPACK V4-32` ×1 at VU address 0: word 0 = vertex count;
//!    - `UNPACK V4-8` ×1 (one to three of them): the strip-end list, bytes in order, 0-terminated,
//!      each = 3 × the vertex index where a new strip starts; the last strip ends at the count
//!      (every value of every packet of all 8 tracks is a multiple of 3 below 3 × count; the ends
//!      fall exactly on the geometric breaks of the decoded strips) [D];
//!    - positions: `UNPACK V3-16` unsigned, `coord = (u16 − 0x8000) / 256`, or `UNPACK V3-32`
//!      floats (masked, mask rows leave x/y/z as data) [D: the 16-bit rule reproduces each
//!      article's `Base` bounding box to the 1/256 unit, e.g. `S12_RiverBnk_n_Road_00`
//!      −14.0078..14.207 × −3.5..4.5 × −30.0078..30.0078];
//!    - UVs: `UNPACK V2-16` unsigned with the same rule, or `UNPACK V2-32` floats [D: road strips
//!      span v = 0..1 exactly].
//!    The VU1 microcode that consumes these (eaglrm.o) was not disassembled — there is no VU
//!    disassembler in the repo — so the /256 scale and the strip-list unit are data-proven only.
//! 5. **Draw chain** — the batch stored in a group entry is NOT what that entry draws. The model
//!    draw sets the part index (`+0xc0`) and runs each entry's render list; the list's op 4 puts
//!    `table[part]` (a DMA call) into the chain, which leads to that part's colour upload and the
//!    batch it calls [M `FUN_002843a0` 0x2843a0, `FUN_002a3290` 0x2a3290 case 4]. Per-part
//!    textures come from the same list (op 3 / op 6, `+ part × 0x60`). See
//!    [`ModelIndex::draw_chain`] and docs/formats/geoprim.md §4b for the proof on all 8 tracks.
//!
//! Pure decoding: no drawing. Malformed data errors instead of overrunning.

use std::collections::HashMap;

use crate::buf::{invalid, FormatError, Result};
use crate::crp::{tag, Crp, Record};
use crate::crp_mesh::TrackModule;

/// `in  ` — the Map's placement record (instance 0) [M 0x22faf8: `FUN_002d5920(map, 0x696e2020)`].
pub const TAG_IN: u32 = 0x696e_2020;
/// `sr  ` — a string reference record (short tag + index) [D].
pub const TAG_SR: u32 = 0x7372_2020;
/// `as  ` — an article part record [D].
pub const TAG_AS: u32 = 0x6173_2020;
/// One placement item's size [M 0x22faf8: the walk steps 0x40].
pub const PLACEMENT_SIZE: usize = 0x40;
/// The 16-bit vertex/UV rule: `(u16 - BIAS) / SCALE` [D].
pub const Q_BIAS: f32 = 32768.0;
pub const Q_SCALE: f32 = 256.0;

/// One `Map in#0` placement [M 0x22faf8 + D].
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    /// The item's index in `in#0`.
    pub index: usize,
    /// `<article>` of the `CARP::<article>::{as  NNNN}` reference.
    pub article: String,
    /// `NNNN` — the article part.
    pub part: u16,
    /// Words 0-2, 4-6, 8-10: the images of the local x, y, z axes.
    pub axes: [[f32; 3]; 3],
    /// Words 12-14: the image of the local origin.
    pub origin: [f32; 3],
    /// The +0x0c flags byte [M 0x22faf8: bit 0x10 = a dynamic instance].
    pub flags: u8,
}

impl Placement {
    /// `x·axis0 + y·axis1 + z·axis2 + origin` [M 0x22faf8 matrix copy, D direction].
    pub fn apply(&self, p: [f32; 3]) -> [f32; 3] {
        let a = &self.axes;
        [
            p[0] * a[0][0] + p[1] * a[1][0] + p[2] * a[2][0] + self.origin[0],
            p[0] * a[0][1] + p[1] * a[1][1] + p[2] * a[2][1] + self.origin[1],
            p[0] * a[0][2] + p[1] * a[1][2] + p[2] * a[2][2] + self.origin[2],
        ]
    }
}

/// One decoded triangle strip of a model, in the model's frame [D].
#[derive(Clone, Debug, PartialEq)]
pub struct LocalStrip {
    pub texture: Option<String>,
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Per-vertex RGB from the chain's masked `UNPACK V3-8` (unsigned bytes, the w lane comes from
    /// the VIF row) [D]; empty when the chain carries none. 0x80 is the GS's 1.0 for a modulated
    /// texture; that the VU1 microcode passes these to the GS unscaled is not proven (eaglrm.o is
    /// not disassembled).
    pub colors: Vec<[u8; 3]>,
}

/// One world-space strip: a placed model strip.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldStrip {
    pub article: String,
    pub part: u16,
    /// The `in#0` item that placed it.
    pub placement: usize,
    pub texture: Option<String>,
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    /// Per-vertex RGB, see [`LocalStrip::colors`].
    pub colors: Vec<[u8; 3]>,
}

impl WorldStrip {
    /// Triangles of the strip: (k−2, k−1, k) for k ≥ 2 (winding alternates; the consumer picks
    /// the face rule — the state symbols say `primtype=tristrip`, culling is per render state).
    pub fn triangles(&self) -> impl Iterator<Item = [[f32; 3]; 3]> + '_ {
        (2..self.positions.len()).map(move |k| [self.positions[k - 2], self.positions[k - 1], self.positions[k]])
    }
}

/// A whole track's decoded geometry.
#[derive(Clone, Debug, Default)]
pub struct TrackGeometry {
    pub strips: Vec<WorldStrip>,
    /// `in#0` items that name an article part.
    pub placements: usize,
    /// Placements whose part has no model reference (`as` tail not `sr`: effects, invisible
    /// barriers `{as  ffff}` and the `trafficlight`-form tails — not decoded) [D].
    pub unresolved_parts: usize,
    /// Model references with no `__Model:::` symbol in the module.
    pub missing_models: usize,
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

fn window(crp: &Crp<'_>, rec: &Record) -> Result<Vec<Record>> {
    let (s, e) = rec.window(crp.data)?;
    crp.window_records(s, e)
}

/// Parses `CARP::<article>::{as  NNNN}` [D].
pub fn parse_part_ref(s: &str) -> Option<(&str, u16)> {
    let rest = s.strip_prefix("CARP::")?;
    let i = rest.find("::{as  ")?;
    let hex = rest.get(i + 7..i + 11)?;
    if rest.get(i + 11..i + 12) != Some("}") {
        return None;
    }
    Some((&rest[..i], u16::from_str_radix(hex, 16).ok()?))
}

/// Decodes one 0x40-byte placement item [M 0x22faf8]. Returns the axes, origin, flags and the
/// reference word (+0x1c).
pub fn decode_placement_item(item: &[u8]) -> Result<([[f32; 3]; 3], [f32; 3], u8, u32)> {
    if item.len() < PLACEMENT_SIZE {
        return invalid(format!("placement item {} < 0x40 bytes", item.len()));
    }
    let v = |i: usize| -> Result<[f32; 3]> { Ok([f32_at(item, i * 16)?, f32_at(item, i * 16 + 4)?, f32_at(item, i * 16 + 8)?]) };
    Ok(([v(0)?, v(1)?, v(2)?], v(3)?, item[0x0c], u32_at(item, 0x1c)?))
}

/// The `Map in#0` placements that name an article part [M 0x22faf8 + D]. Items whose reference
/// is not an `sr` string of the `CARP::<article>::{as  NNNN}` form are skipped.
pub fn map_placements(crp: &Crp<'_>) -> Result<Vec<Placement>> {
    let map = crp
        .pool()?
        .into_iter()
        .find(|r| r.tag == tag::MAP)
        .ok_or_else(|| FormatError::Invalid("no Map article in the pool".into()))?;
    let mut sr: HashMap<u16, String> = HashMap::new();
    let mut inst: Option<Record> = None;
    for r in window(crp, &map)? {
        if r.tag_code() == TAG_SR {
            sr.insert(r.instance().unwrap_or(0), cstr(crp.payload(&r)?));
        } else if r.tag_code() == TAG_IN && r.instance() == Some(0) && inst.is_none() {
            inst = Some(r);
        }
    }
    let inst = inst.ok_or_else(|| FormatError::Invalid("no in#0 record in the Map window".into()))?;
    let at = inst.data_addr(crp.data)?;
    let n = inst.count.max(0) as usize;
    let end = at + n * PLACEMENT_SIZE;
    if end > crp.data.len() {
        return invalid(format!("Map in#0: {n} items at {at:#x} run past the file ({})", crp.data.len()));
    }
    let mut out = Vec::new();
    for i in 0..n {
        let (axes, origin, flags, refw) = decode_placement_item(&crp.data[at + i * PLACEMENT_SIZE..at + (i + 1) * PLACEMENT_SIZE])?;
        if refw >> 16 != 0x7372 {
            continue;
        }
        let Some(s) = sr.get(&(refw as u16)) else { continue };
        let Some((article, part)) = parse_part_ref(s) else { continue };
        out.push(Placement { index: i, article: article.to_owned(), part, axes, origin, flags });
    }
    Ok(out)
}

/// `(article, part) → EAGL model name` (`EAGL::<a>::{as  0000}::Model[0].XX`) for every article part
/// whose `as` payload ends in `{u16 index, "sr"}` [D].
pub fn part_models(crp: &Crp<'_>) -> Result<HashMap<(String, u16), String>> {
    let mut out = HashMap::new();
    for arti in crp.pool()? {
        if arti.tag != tag::ARTI {
            continue;
        }
        let recs = window(crp, &arti)?;
        let mut name = None;
        let mut srs: HashMap<u16, String> = HashMap::new();
        let mut parts: Vec<(u16, u16)> = Vec::new();
        for r in &recs {
            if r.tag == tag::NAME {
                name = Some(cstr(crp.payload(r)?));
            } else if r.tag_code() == TAG_SR {
                srs.insert(r.instance().unwrap_or(0), cstr(crp.payload(r)?));
            } else if r.tag_code() == TAG_AS {
                let p = crp.payload(r)?;
                if p.len() >= 4 && &p[p.len() - 2..] == b"rs" {
                    let idx = u16::from_le_bytes([p[p.len() - 4], p[p.len() - 3]]);
                    parts.push((r.instance().unwrap_or(0), idx));
                }
            }
        }
        let Some(name) = name else { continue };
        for (part, idx) in parts {
            if let Some(s) = srs.get(&idx) {
                out.insert((name.clone(), part), s.clone());
            }
        }
    }
    Ok(out)
}

/// Lookups over a loaded module: model symbols and texture descriptors.
pub struct ModelIndex<'m> {
    pub module: &'m TrackModule,
    /// File offset of `.data` in the module image (pointers are `.data`-relative) [D].
    pub data_offset: usize,
    models: HashMap<String, u32>,
    textures: HashMap<u32, String>,
}

impl<'m> ModelIndex<'m> {
    pub fn new(module: &'m TrackModule) -> Result<Self> {
        let data = module
            .sections
            .iter()
            .find(|s| s.name == ".data")
            .ok_or_else(|| FormatError::Invalid("no .data section in the module".into()))?;
        let mut models = HashMap::new();
        let mut textures = HashMap::new();
        for s in &module.symbols {
            if let Some(m) = s.name.strip_prefix("__Model:::") {
                models.insert(m.to_owned(), s.value);
            } else if s.name.starts_with("__EAGL::TAR:::") {
                if let Some(i) = s.name.find("TEX0::") {
                    // The sheet name is the descriptor's +4 word (up to 4 ASCII bytes, 0-padded):
                    // it equals the symbol's `TEX0::` tail on every `{T0@…}` descriptor of the 8
                    // tracks, and names the real sheet entry on the per-part `{T0|…}` ones whose tail
                    // is an export label (paris `pb01_1` → `pb02`) [D]. Falls back to the tail.
                    let at = data.data_offset as usize + s.value as usize;
                    let raw = module.image.get(at + 4..at + 8).unwrap_or(&[]);
                    let n = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
                    let name = if (1..=4).contains(&n) && raw[..n].iter().all(|b| b.is_ascii_graphic()) {
                        String::from_utf8_lossy(&raw[..n]).into_owned()
                    } else {
                        s.name[i + 6..].to_owned()
                    };
                    textures.entry(s.value).or_insert(name);
                }
            }
        }
        Ok(ModelIndex { module, data_offset: data.data_offset as usize, models, textures })
    }

    fn img(&self) -> &[u8] { &self.module.image }

    fn ptr(&self, p: u32) -> Result<usize> {
        let at = self.data_offset + p as usize;
        if at >= self.img().len() {
            return invalid(format!("pointer {p:#x} past the module image"));
        }
        Ok(at)
    }

    /// The `.data` value of `__Model:::<name>` (name = the part after `EAGL::`).
    pub fn model_symbol(&self, eagl_name: &str) -> Option<u32> {
        self.models.get(eagl_name.strip_prefix("EAGL::").unwrap_or(eagl_name)).copied()
    }

    /// The packet image offsets of a model object [D: +0xc8 count, +0xcc table].
    pub fn model_packets(&self, sym_value: u32) -> Result<Vec<usize>> {
        let obj = self.ptr(sym_value)?;
        let groups = u32_at(self.img(), obj + 0xc8)? as usize;
        let mut cur = self.ptr(u32_at(self.img(), obj + 0xcc)?)?;
        let mut out = Vec::new();
        for _ in 0..groups.min(4096) {
            match u32_at(self.img(), cur)? {
                0xA000_0000 => {
                    let n = u32_at(self.img(), cur + 8)? as usize;
                    cur += 12;
                    for _ in 0..n / 2 {
                        if u32_at(self.img(), cur)? != 0xA000_FFFF {
                            return invalid(format!("model {sym_value:#x}: group entry {:#x} is not 0xA000FFFF", u32_at(self.img(), cur)?));
                        }
                        out.push(self.ptr(u32_at(self.img(), cur + 4)?)?);
                        cur += 8;
                    }
                }
                0xA000_FFFF => {
                    out.push(self.ptr(u32_at(self.img(), cur + 4)?)?);
                    cur += 8;
                }
                w => return invalid(format!("model {sym_value:#x}: packet table word {w:#x}")),
            }
        }
        Ok(out)
    }

    /// The `__Model:::` object's part index (`+0xc0`): the game copies it into the global that the
    /// render-method interpreter indexes per-part data with [M `FUN_002843a0` 0x2843a0:
    /// `DAT_003483fc = *(model + 0xc0)` before the group walk] [D: equals the `Model[0].XX` suffix].
    pub fn model_part(&self, sym_value: u32) -> Result<u32> { u32_at(self.img(), self.ptr(sym_value)? + 0xc0) }

    /// The render-method instruction list of a group entry: entry `+0` → GeoPrim header, header
    /// `+0` → the list the interpreter walks [M `FUN_002843a0`: `FUN_002a3290(*entry[1])`;
    /// `FUN_002a3290`: `puVar32 = *param_1`]. Returns `(image offset, instruction)` for every
    /// instruction up to the 0 terminator, stepping by the interpreter's lengths [M 0x2a3290]:
    /// ops 1 and 4 = 2 words, 2 = n + 2, 3 = 1 + n, 5/6/7 = 3 + n (n = low 16 bits), 0xe/0xf = 1.
    /// Any other op errors (none occurs in the 8 tracks [D]).
    pub fn render_ops(&self, po: usize) -> Result<Vec<(usize, u32)>> {
        let hdr = self.ptr(u32_at(self.img(), po)?)?;
        let rm = self.ptr(u32_at(self.img(), hdr)?)?;
        let mut out = Vec::new();
        let mut i = 0usize;
        loop {
            let at = rm + i * 4;
            let o = u32_at(self.img(), at)?;
            if o == 0 {
                return Ok(out);
            }
            let n = (o & 0xffff) as usize;
            let len = match o >> 24 {
                1 | 4 => 2,
                2 => n + 2,
                3 => 1 + n,
                5..=7 => 3 + n,
                0xe | 0xf => 1,
                op => return invalid(format!("entry {po:#x}: render op {op:#x} ({o:#x}) at {at:#x} not handled")),
            };
            out.push((at, o));
            i += len;
            if out.len() > 4096 {
                return invalid(format!("entry {po:#x}: render list has no terminator"));
            }
        }
    }

    /// The texture a group entry's render list sets for `part`: the first texture reference, in
    /// list order, that resolves to a named `__EAGL::TAR` descriptor.
    /// - op 3 (`0x0300000N`, N words): each word's top two bits pick the address: 0 = the word,
    ///   1 = the word + part × 0x60 (a per-part texture), 2 = a run-time table, 3 = none
    ///   [M `FUN_002a3290` case 3: `uVar25 >> 0x1e`, `(uVar25 & 0x3fffffff) + DAT_003483fc * 0x60`];
    /// - op 6 (`0x06KK00NN`): the word at +8, plus KK × part × 0x60 [M case 6:
    ///   `puVar33[2] + uVar23 * DAT_003483fc * 0x60`, uVar23 = bits 16-23].
    ///
    /// [D: every mode-1 op-3 word of the 8 tracks resolves to a named descriptor for its model's
    /// part, e.g. paris `P_building3_12seg` part 10 → `pb01_10` where part 0 reads `pb01`.]
    /// `None` = the list names no texture.
    pub fn packet_texture(&self, po: usize, part: u32) -> Option<String> {
        for (at, o) in self.render_ops(po).ok()? {
            let addrs: Vec<u32> = match o >> 24 {
                3 => (0..(o & 0xffff) as usize)
                    .filter_map(|j| u32_at(self.img(), at + 4 + j * 4).ok())
                    .filter_map(|w| match w >> 30 {
                        0 => Some(w & 0x3fff_ffff),
                        1 => Some((w & 0x3fff_ffff).wrapping_add(part.wrapping_mul(0x60))),
                        _ => None,
                    })
                    .collect(),
                6 => {
                    let k = (o >> 16) & 0xff;
                    let w = u32_at(self.img(), at + 8).ok()?;
                    vec![w.wrapping_add(k.wrapping_mul(part).wrapping_mul(0x60))]
                }
                _ => continue,
            };
            if let Some(t) = addrs.iter().find_map(|a| self.textures.get(a)) {
                return Some(t.clone());
            }
        }
        None
    }

    /// The VIF byte ranges the game sends for group entry `po` drawn as `part`
    /// [M `FUN_002a3290` case 4 + the PS2 DMA tag rules; D shapes]. The render list's op 4
    /// (`0x04000002`; exactly one per list in the 8 tracks [D]) copies one quadword from
    /// `table + 16 × part` into the chain [M: `DAT_003483fc * 0x10 + *puVar32`, 2 doublewords].
    /// That quadword is a DMA `call` tag with no payload (`0x50000000`, address X). X is either
    /// - a `call` tag (id 5): its payload is the part's colour upload (`STMASK`/`STROW`/`UNPACK
    ///   V3-8`), and its address is a `ret` tag (id 6) whose payload is the shared geometry batch; or
    /// - a `ret` tag (id 6): one batch holding the geometry and its colours, if any.
    ///
    /// A tag's VIF stream starts at tag + 8 (its upper 64 bits). The geometry reached this way is
    /// NOT the batch stored inside the entry object itself: paris `S04_s4s_00`'s entry 0x36858 calls
    /// the batch stored in object 0x355f8 (8 vertices). Two independent checks
    /// (docs/formats/geoprim.md §4b): every model's `Colours` list names the chain batches' vertex
    /// counts (paris 1,354 of 1,357 lists, the 3 others match as a prefix; the objects' own batches
    /// match 81, each of which the chain matches too), and the chain batches lie inside the
    /// article's `Base` box where the own batches stray out of it.
    pub fn draw_chain(&self, po: usize, part: u32) -> Result<Vec<(usize, usize)>> {
        let ops = self.render_ops(po)?;
        let (at, _) = *ops
            .iter()
            .find(|(_, o)| o >> 24 == 4)
            .ok_or_else(|| FormatError::Invalid(format!("entry {po:#x}: render list has no op 4")))?;
        let table = self.ptr(u32_at(self.img(), at + 4)?)?;
        let e = table + 16 * part as usize;
        let tag = u32_at(self.img(), e)?;
        if tag != 0x5000_0000 {
            return invalid(format!("entry {po:#x} part {part}: table word {tag:#x} at {e:#x} is not an empty DMA call"));
        }
        let x = self.ptr(u32_at(self.img(), e + 4)?)?;
        let payload = |t: usize| -> Result<(u32, (usize, usize))> {
            let w = u32_at(self.img(), t)?;
            let end = t + 16 + (w & 0xffff) as usize * 16;
            if end > self.img().len() {
                return invalid(format!("DMA tag {t:#x}: {} quadwords run past the image", w & 0xffff));
            }
            Ok(((w >> 28) & 7, (t + 8, end)))
        };
        let (id, r) = payload(x)?;
        match id {
            6 => Ok(vec![r]),
            5 => {
                let g = self.ptr(u32_at(self.img(), x + 4)?)?;
                let (gid, gr) = payload(g)?;
                if gid != 6 {
                    return invalid(format!("entry {po:#x} part {part}: colour call {x:#x} targets tag id {gid}, not ret"));
                }
                Ok(vec![r, gr])
            }
            _ => invalid(format!("entry {po:#x} part {part}: table target {x:#x} has DMA tag id {id}")),
        }
    }

    /// Every strip of a model, in the model frame, as the game draws it for the model's part
    /// (`+0xc0`): per group entry, the draw chain's batch ([`Self::draw_chain`]) with the entry's
    /// texture ([`Self::packet_texture`]). An entry whose list names no texture keeps the previous
    /// entry's [inferred from the list contents; the EAGL state machine itself is not traced].
    /// Each model starts untextured.
    pub fn model_strips(&self, sym_value: u32) -> Result<Vec<LocalStrip>> {
        let part = self.model_part(sym_value)?;
        let mut out = Vec::new();
        let mut tex = None;
        for po in self.model_packets(sym_value)? {
            if let Some(t) = self.packet_texture(po, part) {
                tex = Some(t);
            }
            for mut s in decode_vif(self.img(), &self.draw_chain(po, part)?, po)? {
                s.texture = tex.clone();
                out.push(s);
            }
        }
        Ok(out)
    }
}

/// Decodes one packet object's own VIF stream (tag at `po + 8`) into strips (texture left `None`)
/// [D, see the module docs]. The game does not draw an entry object's own batch for that entry
/// (see [`ModelIndex::draw_chain`]); this stays for inspecting one batch.
pub fn decode_packet(img: &[u8], po: usize) -> Result<Vec<LocalStrip>> {
    let qwc = (u32_at(img, po + 8)? & 0xffff) as usize;
    decode_vif(img, &[(po + 0x10, po + 0x10 + qwc * 16)], po)
}

/// Decodes the VIF byte ranges of one draw into strips [D]; the VIF state (mode, mask, row) carries
/// across the ranges, as one DMA chain feeds one VIF. `label` names the batch in errors.
/// Colours: the masked `UNPACK V3-8` unsigned (every one in the 8 tracks has STMASK 0x40404040 —
/// x/y/z data, w from the row — mode 0, and `num` = the vertex count [D]).
pub fn decode_vif(img: &[u8], ranges: &[(usize, usize)], label: usize) -> Result<Vec<LocalStrip>> {
    let po = label;
    let mut mode = 0u32;
    let mut row = [0u32; 4];
    let mut mask = 0u32;
    let mut count: Option<usize> = None;
    let mut ends: Vec<u8> = Vec::new();
    let mut positions: Option<Vec<[f32; 3]>> = None;
    let mut uvs: Option<Vec<[f32; 2]>> = None;
    let mut colors: Option<Vec<[u8; 3]>> = None;
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
                        return invalid(format!("packet {po:#x}: unpack at {cur:#x} runs past the batch"));
                    }
                    let data = &img[cur..cur + bytes];
                    // A component is plain data when its mask bits are 00 in every row [VIF STMASK].
                    let comp_is_data = |k: usize| !masked || (0..4).all(|r| (mask >> (r * 8 + k * 2)) & 3 == 0);
                    let offset = |k: usize, v: u32| if mode == 1 { v.wrapping_add(row[k]) } else { v };
                    match (vn, vl) {
                        (3, 0) if num == 1 && addr == 0 && count.is_none() => count = Some(u32_at(data, 0)? as usize),
                        // Positions as unmasked `V4-32` floats (x, y, z, 1.0) at VU 0x36: the first
                        // unpack after the strip list, `num` = the header count, every w lane exactly
                        // 1.0f [D: the MIS11 water `AnthWater`, `Siloroom1Water`, `Tanker_Ocean1`/`1b`/
                        // `1c`/`2b`/`3` — the slot the sibling `EndCoast_Ocean_*` layout fills with
                        // `V3-16`, colours at VU 0x7a in both; the decoded vertices lie in each
                        // article's `Base` box]. The other `V4-32` position form (VU 0x10, after
                        // strip lists at 0x0d..0x0f: `Rock_Collapse`, `JA_SAL`, `JB_SAL`,
                        // `LookOutTowerExplosion`) is NOT decoded: its strip-list bytes are not
                        // multiples of 3 (e.g. 14, 30, 40 for 54 vertices), so that layout's
                        // strip-list unit differs and is not traced.
                        (3, 0) if positions.is_none() && addr == 0x36 && count == Some(num) => {
                            if masked {
                                return invalid(format!("packet {po:#x}: V4-32 position unpack {code:#x} masked"));
                            }
                            let mut v = Vec::with_capacity(num);
                            for i in 0..num {
                                if u32_at(data, i * 16 + 12)? != 0x3f80_0000 {
                                    return invalid(format!("packet {po:#x}: V4-32 position {i} has w {:#x}, not 1.0", u32_at(data, i * 16 + 12)?));
                                }
                                let c = |k: usize| -> Result<f32> { Ok(f32::from_bits(offset(k, u32_at(data, i * 16 + k * 4)?))) };
                                v.push([c(0)?, c(1)?, c(2)?]);
                            }
                            positions = Some(v);
                        }
                        (3, 2) if num == 1 && positions.is_none() => ends.extend_from_slice(&data[..4]),
                        (2, 1) | (2, 0) if positions.is_none() => {
                            if !(0..3).all(comp_is_data) || (vl == 1 && !usn) {
                                return invalid(format!("packet {po:#x}: position unpack {code:#x} masked or signed"));
                            }
                            let mut v = Vec::with_capacity(num);
                            for i in 0..num {
                                let c = |k: usize| -> Result<f32> {
                                    Ok(if vl == 1 {
                                        let u = u16::from_le_bytes([data[i * 6 + k * 2], data[i * 6 + k * 2 + 1]]) as u32;
                                        (offset(k, u) as f32 - Q_BIAS) / Q_SCALE
                                    } else {
                                        f32::from_bits(offset(k, u32_at(data, i * 12 + k * 4)?))
                                    })
                                };
                                v.push([c(0)?, c(1)?, c(2)?]);
                            }
                            positions = Some(v);
                        }
                        (1, 1) | (1, 0) if positions.is_some() && uvs.is_none() => {
                            if !(0..2).all(comp_is_data) || (vl == 1 && !usn) {
                                return invalid(format!("packet {po:#x}: uv unpack {code:#x} masked or signed"));
                            }
                            let mut v = Vec::with_capacity(num);
                            for i in 0..num {
                                let c = |k: usize| -> Result<f32> {
                                    Ok(if vl == 1 {
                                        let u = u16::from_le_bytes([data[i * 4 + k * 2], data[i * 4 + k * 2 + 1]]) as u32;
                                        (offset(k, u) as f32 - Q_BIAS) / Q_SCALE
                                    } else {
                                        f32::from_bits(offset(k, u32_at(data, i * 8 + k * 4)?))
                                    })
                                };
                                v.push([c(0)?, c(1)?]);
                            }
                            uvs = Some(v);
                        }
                        (2, 2) if colors.is_none() => {
                            if !(0..3).all(comp_is_data) || !usn || mode != 0 {
                                return invalid(format!("packet {po:#x}: colour unpack {code:#x} masked, signed or offset"));
                            }
                            colors = Some((0..num).map(|i| [data[i * 3], data[i * 3 + 1], data[i * 3 + 2]]).collect());
                        }
                        _ => {}
                    }
                    cur += len;
                }
                other => return invalid(format!("packet {po:#x}: unhandled VIF command {other:#x} at {:#x}", cur - 4)),
            }
        }
    }
    let (Some(count), Some(positions)) = (count, positions) else { return Ok(Vec::new()) };
    if positions.len() != count {
        return invalid(format!("packet {po:#x}: header count {count} != {} positions", positions.len()));
    }
    let uvs = uvs.unwrap_or_default();
    if !uvs.is_empty() && uvs.len() != count {
        return invalid(format!("packet {po:#x}: {} uvs for {count} positions", uvs.len()));
    }
    let colors = colors.unwrap_or_default();
    if !colors.is_empty() && colors.len() != count {
        return invalid(format!("packet {po:#x}: {} colours for {count} positions", colors.len()));
    }
    let mut starts = vec![0usize];
    for &b in ends.iter().take_while(|&&b| b != 0) {
        if b % 3 != 0 || (b / 3) as usize <= *starts.last().unwrap() || (b / 3) as usize >= count {
            return invalid(format!("packet {po:#x}: strip list {ends:?} invalid for {count} vertices"));
        }
        starts.push((b / 3) as usize);
    }
    starts.push(count);
    Ok(starts
        .windows(2)
        .map(|w| LocalStrip {
            texture: None,
            positions: positions[w[0]..w[1]].to_vec(),
            uvs: if uvs.is_empty() { Vec::new() } else { uvs[w[0]..w[1]].to_vec() },
            colors: if colors.is_empty() { Vec::new() } else { colors[w[0]..w[1]].to_vec() },
        })
        .collect())
}

/// Every placed strip of a track `.crp` (decompressed bytes) in world space.
pub fn track_geometry(crp_bytes: &[u8]) -> Result<TrackGeometry> {
    let crp = Crp::parse(crp_bytes)?;
    let module = TrackModule::load(&crp)?;
    let index = ModelIndex::new(&module)?;
    let parts = part_models(&crp)?;
    let placements = map_placements(&crp)?;
    let mut cache: HashMap<u32, Vec<LocalStrip>> = HashMap::new();
    let mut g = TrackGeometry { placements: placements.len(), ..Default::default() };
    for p in &placements {
        let Some(model) = parts.get(&(p.article.clone(), p.part)) else {
            g.unresolved_parts += 1;
            continue;
        };
        let Some(sym) = index.model_symbol(model) else {
            g.missing_models += 1;
            continue;
        };
        if !cache.contains_key(&sym) {
            cache.insert(sym, index.model_strips(sym)?);
        }
        for s in &cache[&sym] {
            g.strips.push(WorldStrip {
                article: p.article.clone(),
                part: p.part,
                placement: p.index,
                texture: s.texture.clone(),
                positions: s.positions.iter().map(|&v| p.apply(v)).collect(),
                uvs: s.uvs.clone(),
                colors: s.colors.clone(),
            });
        }
    }
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_u32(b: &mut Vec<u8>, w: u32) { b.extend_from_slice(&w.to_le_bytes()); }

    /// A packet: back pointer, DMA tag, then VIF: header (count), strip list, V3-16, V2-16, MSCNT.
    fn packet(count: u32, list: [u8; 4], pos: &[[u16; 3]], uv: &[[u16; 2]]) -> Vec<u8> {
        let mut v = Vec::new();
        put_u32(&mut v, 0x6c01_c000); // UNPACK V4-32 num 1 addr 0
        put_u32(&mut v, count);
        put_u32(&mut v, 0);
        put_u32(&mut v, 0);
        put_u32(&mut v, 0);
        put_u32(&mut v, 0x6e01_c010); // UNPACK V4-8 num 1 (strip list)
        v.extend_from_slice(&list);
        put_u32(&mut v, 0x6900_c011 | ((pos.len() as u32) << 16)); // UNPACK V3-16 usn
        for p in pos {
            for c in p {
                v.extend_from_slice(&c.to_le_bytes());
            }
        }
        while v.len() % 4 != 0 {
            v.push(0);
        }
        put_u32(&mut v, 0x6500_c051 | ((uv.len() as u32) << 16)); // UNPACK V2-16 usn
        for p in uv {
            for c in p {
                v.extend_from_slice(&c.to_le_bytes());
            }
        }
        put_u32(&mut v, 0x1700_0000); // MSCNT
        while v.len() % 16 != 0 {
            put_u32(&mut v, 0);
        }
        let mut out = Vec::new();
        put_u32(&mut out, 0);
        put_u32(&mut out, 0);
        put_u32(&mut out, 0x6000_0000 | (v.len() / 16) as u32);
        put_u32(&mut out, 0);
        out.extend_from_slice(&v);
        out
    }

    #[test]
    fn packet_decodes_the_unsigned_256_rule_and_the_strip_list() {
        // Two strips: 4 + 2 verts; list byte 12 = 3 x 4.
        let q = |x: f32| (x * 256.0 + 32768.0) as u16;
        let pos = [[q(-8.0), q(0.8984375), q(-10.0)], [q(0.0), q(0.8984375), q(-10.0)], [q(-8.0), q(1.0), q(-5.0)],
            [q(0.0), q(1.0), q(-5.0)], [q(10.0), q(0.0), q(0.0)], [q(12.0), q(0.0), q(0.0)]];
        let uv = [[q(0.25), q(0.0)], [q(1.0), q(0.0)], [q(0.25), q(1.0)], [q(1.0), q(1.0)], [q(0.0), q(0.0)], [q(0.5), q(0.0)]];
        let img = packet(6, [12, 0, 0, 0], &pos, &uv);
        let s = decode_packet(&img, 0).unwrap();
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].positions, vec![[-8.0, 0.8984375, -10.0], [0.0, 0.8984375, -10.0], [-8.0, 1.0, -5.0], [0.0, 1.0, -5.0]]);
        assert_eq!(s[1].positions, vec![[10.0, 0.0, 0.0], [12.0, 0.0, 0.0]]);
        assert_eq!(s[0].uvs[3], [1.0, 1.0]);
        // A signed read of the same words (the old decoder's i16) lands far away: 0x77fe as i16
        // is +30718, as the unsigned rule −8.0078 — the rule is load-bearing.
        assert_ne!((0x77feu16 as i16) as f32 / 256.0, s[0].positions[0][0]);
        // A strip list not divisible by 3, or past the count, is refused.
        assert!(decode_packet(&packet(6, [13, 0, 0, 0], &pos, &uv), 0).is_err());
        assert!(decode_packet(&packet(6, [18, 0, 0, 0], &pos, &uv), 0).is_err());
        // A header count that disagrees with the positions is refused.
        assert!(decode_packet(&packet(7, [12, 0, 0, 0], &pos, &uv), 0).is_err());
    }

    #[test]
    fn placement_is_the_forward_matrix_with_origin_last() {
        // A 90-degree turn about y: local +x -> world -z, local +z -> world +x; origin (100, -8.5, 7).
        let mut it = vec![0u8; 0x40];
        let w = |it: &mut Vec<u8>, i: usize, f: f32| it[i * 4..i * 4 + 4].copy_from_slice(&f.to_le_bytes());
        w(&mut it, 0, 0.0); w(&mut it, 1, 0.0); w(&mut it, 2, -1.0);
        w(&mut it, 4, 0.0); w(&mut it, 5, 1.0); w(&mut it, 6, 0.0);
        w(&mut it, 8, 1.0); w(&mut it, 9, 0.0); w(&mut it, 10, 0.0);
        w(&mut it, 12, 100.0); w(&mut it, 13, -8.5); w(&mut it, 14, 7.0);
        it[0x0c] = 0x10;
        it[0x1c..0x20].copy_from_slice(&0x7372_0005u32.to_le_bytes());
        let (axes, origin, flags, refw) = decode_placement_item(&it).unwrap();
        let p = Placement { index: 0, article: "a".into(), part: 1, axes, origin, flags };
        assert_eq!(flags, 0x10);
        assert_eq!(refw, 0x7372_0005);
        assert_eq!(p.apply([2.0, 1.0, 3.0]), [103.0, -7.5, 5.0]);
        assert_eq!(p.apply([0.0; 3]), [100.0, -8.5, 7.0]);
        // The transposed reading (rows as the images of world axes) gives another point.
        let t = |j: usize| (0..3).map(|i| axes[j][i] * [2.0f32, 1.0, 3.0][i]).sum::<f32>() + origin[j];
        assert_ne!([t(0), t(1), t(2)], p.apply([2.0, 1.0, 3.0]));
        assert!(decode_placement_item(&it[..0x3c]).is_err());
    }

    #[test]
    fn part_reference_strings() {
        assert_eq!(parse_part_ref("CARP::S04_s4s_00::{as  001f}"), Some(("S04_s4s_00", 0x1f)));
        assert_eq!(parse_part_ref("CARP::<<Map>>::{AICo}::8960"), None);
        assert_eq!(parse_part_ref("CARP::x::{as  00g1}"), None);
        assert_eq!(parse_part_ref("EAGL::x::{as  0001}"), None);
    }

    /// A module image with one model (part 1) whose single group entry stores one batch of its own
    /// but whose render list's op 4 table calls, per part, a colour upload + a shared batch — the
    /// paris `S04_s4s_00` shape [M 0x2843a0 / 0x2a3290 case 4, D].
    fn chain_module() -> crate::crp_mesh::TrackModule {
        use crate::crp_mesh::{ModuleSection, ModuleSymbol, TrackModule};
        let mut img = vec![0u8; 0x800];
        let w = |img: &mut Vec<u8>, at: usize, v: u32| img[at..at + 4].copy_from_slice(&v.to_le_bytes());
        // Model object at 0: part 1, one group, group table at 0x100.
        w(&mut img, 0xc0, 1);
        w(&mut img, 0xc8, 1);
        w(&mut img, 0xcc, 0x100);
        w(&mut img, 0x100, 0xA000_FFFF);
        w(&mut img, 0x104, 0x200);
        // Entry object 0x200: its own batch is a 3-vertex strip far away.
        let q = |x: f32| (x * 256.0 + 32768.0) as u16;
        let own = packet(3, [0, 0, 0, 0], &[[q(90.0), q(0.0), q(0.0)], [q(91.0), q(0.0), q(0.0)], [q(90.0), q(0.0), q(1.0)]], &[[0; 2]; 3]);
        img[0x200..0x200 + own.len()].copy_from_slice(&own);
        w(&mut img, 0x200, 0x280); // +0 → header
        w(&mut img, 0x280, 0x2c0); // header +0 → render list
        // Render list: op 3 (one per-part texture word, mode 1), op 4 → table 0x300, end.
        w(&mut img, 0x2c0, 0x0300_0001);
        w(&mut img, 0x2c4, 0x4000_0000 | 0x5a0);
        w(&mut img, 0x2c8, 0x0400_0002);
        w(&mut img, 0x2cc, 0x300);
        // Table: part 0 → colour call 0x340, part 1 → colour call 0x380.
        for (k, x) in [0x340u32, 0x380].into_iter().enumerate() {
            w(&mut img, 0x300 + 16 * k, 0x5000_0000);
            w(&mut img, 0x304 + 16 * k, x);
        }
        // Colour calls: call qwc 2 → geometry tag 0x408; STMASK 0x40404040, UNPACK V3-8 num 4.
        for (x, c) in [(0x340usize, 0x10u8), (0x380, 0x40)] {
            w(&mut img, x, 0x5000_0002);
            w(&mut img, x + 4, 0x408);
            w(&mut img, x + 8, 0x2000_0000);
            w(&mut img, x + 12, 0x4040_4040);
            w(&mut img, x + 16, 0x7a04_c093);
            for i in 0..12 {
                img[x + 20 + i] = c + i as u8;
            }
        }
        // The shared batch: tag (ret) at 0x408, two strips (2 + 2 vertices).
        let shared = packet(4, [6, 0, 0, 0], &[[q(-1.0), q(0.0), q(-1.0)], [q(1.0), q(0.0), q(-1.0)], [q(-1.0), q(0.5), q(1.0)], [q(1.0), q(0.5), q(1.0)]], &[[q(0.0), q(0.0)]; 4]);
        img[0x400..0x400 + shared.len()].copy_from_slice(&shared);
        // Texture descriptors 0x60 apart; the sheet name is the +4 word.
        img[0x5a4..0x5a8].copy_from_slice(b"pb01");
        img[0x604..0x608].copy_from_slice(b"pb02");
        let sym = |name: &str, value: u32| ModuleSymbol { name: name.into(), value, size: 4, info: 0x11, shndx: 1 };
        TrackModule {
            image: img,
            elfd_size: 0x800,
            sections: vec![ModuleSection { name: ".data".into(), sh_type: 1, offset: 0, data_offset: 0, size: 0x800, link: 0, entsize: 0 }],
            symbols: vec![
                sym("__Model:::m::{as  0000}::Model[0].01", 0),
                sym("__EAGL::TAR:::{T0|1}TEX0::pb01", 0x5a0),
                sym("__EAGL::TAR:::{T0|1}TEX0::pb01_1", 0x600),
            ],
            relocations: 0,
        }
    }

    #[test]
    fn a_model_draws_its_render_lists_chain_for_its_part_with_that_parts_colours() {
        let m = chain_module();
        let ix = ModelIndex::new(&m).unwrap();
        assert_eq!(ix.model_part(0).unwrap(), 1);
        let s = ix.model_strips(0).unwrap();
        // The geometry is the called batch, not the entry object's own (x = 90..91).
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].positions, vec![[-1.0, 0.0, -1.0], [1.0, 0.0, -1.0]]);
        assert_eq!(s[1].positions, vec![[-1.0, 0.5, 1.0], [1.0, 0.5, 1.0]]);
        assert_ne!(decode_packet(&m.image, 0x200).unwrap()[0].positions, s[0].positions);
        // Part 1's colours (table entry 1), split with the strips.
        assert_eq!(s[0].colors, vec![[0x40, 0x41, 0x42], [0x43, 0x44, 0x45]]);
        assert_eq!(s[1].colors, vec![[0x46, 0x47, 0x48], [0x49, 0x4a, 0x4b]]);
        // Part 0 would take table entry 0.
        let p0 = decode_vif(&m.image, &ix.draw_chain(0x200, 0).unwrap(), 0x200).unwrap();
        assert_eq!(p0[0].colors[0], [0x10, 0x11, 0x12]);
        // The per-part texture: base + part × 0x60, named by the descriptor's +4 word.
        assert_eq!(s[0].texture.as_deref(), Some("pb02"));
        assert_eq!(ix.packet_texture(0x200, 0).as_deref(), Some("pb01"));
        // A part past the table (not an empty DMA call) is refused.
        assert!(ix.draw_chain(0x200, 2).is_err());
    }
}
