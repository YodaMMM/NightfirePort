//! Track-mesh layer over the CARP container (`docs/formats/crp.md`; task `drive-track-mesh`).
//!
//! What this module decodes, all measured on our disc data (the 8 mission `.crp` tracks) and
//! matched to DRIVING.ELF code (EE = file offset + 0x107080):
//!
//! 1. **Articles** — the pool `Arti` records are the track's geometry/deco pieces. Each article's
//!    window holds: a `Base` record whose payload carries the piece's bounding box (float min at
//!    +0x10, max at +0x20 [D `S6_CustomLowRBnknRoad` measured −10.06/−7.5/−10.06 … 18.33/0.5/15.0;
//!    consumer semantics [S] Raikaru/nightfire-engine docs/driving.md]), a `Name` string record
//!    [M 0x2dc1b0], `as#N` records whose payload ends in the 4-byte word `{u16 sr_index, "sr"}`
//!    (little-endian) — the part's render reference [D measured: `as#0` tail `05 00 72 73` →
//!    `sr#5` = `EAGL::S6_CustomLowRBnknRoad::{as  0000}::Model[0].00`], `ca#0` (per-article
//!    collision strips, layout in `Crp::extract_collision_tris` [M]), `in#0/#1`
//!    (64 B transforms, cnt=1) and `mp#0` (4 B).
//! 2. **The embedded module** — `ELFd` (head: ELF header + `.data`) + `ELFr` (continuation:
//!    section-name strings, `.strtab`, `.symtab`, `.rel.data`, section headers) form ONE MIPS32
//!    ET_REL image, split at the ELFd record's payload size [M 0x1a9600 lines 143-150 passes
//!    `(ELFd_ptr, ELFd_size, ELFr_ptr)` into `FUN_0029f820`; the resolver `FUN_0029ea00` 0x29ea00
//!    resolves every section `sh_addr` and `e_shoff` as
//!    `x < elfd_size ? ELFd+x : ELFr + (x - elfd_size)`]. Measured on paris (MIS01.VIV#4): 6
//!    sections; `e_shoff` 0x4642b0 lands exactly on the last 0xF0 bytes of the combined image;
//!    sections `.data` (0x3be7d0) + `.shstrtab` + `.strtab` + `.symtab` (3993 syms) + `.rel.data`
//!    (54,613 R_MIPS_32 relocs) [D].
//! 3. **Model symbols** — the `.symtab` carries `__Model:::<article>::{as  0000}::Model[0].NN`
//!    names (paris: 1,890) whose `st_value` is a flat offset into the combined image [D: the
//!    relocations targeting those slots use the `.data` section symbol (st_value 0), so the stored
//!    word IS the flat offset — verified by following `S12_LowRiverBnk_n_Road_00 …Model[0].1d`'s
//!    slot to a model-node object whose +0xC holds the segment index 0x1C and +0x18 a
//!    segment-list pointer into a shared array of `{value, 0xA000FFFF}` records referencing
//!    `__GPRenderMethod_<article>::…::Model[0].NN_<x>_name` strings]. Applying `.rel.data` (every
//!    entry `R_MIPS_32`: `word += symbol.st_value` [M `EAGL::DynamicLoader::Resolve` 0x2bb300;
//!    measured: 54,613/54,613 paris entries are type 2]) reconstructs the loaded image. The render
//!    geometry itself (the VIF chains the EAGL engine builds from these objects at load time) is
//!    decoded by `TrackModule::extract_article_mesh`: each batch-table entry reaches its VIF
//!    packet through its render-method list's DMA CALL (`extract_segments`) [D].
//! 4. **Collision** — the pool `CDat` article is the collision world: window = `CGrd` (36 B grid
//!    header) + `ci#0` (63,616 B, count=994 items on paris) + ~1600 `cn#N` node records (38-88 B)
//!    [D measured]. `RNgp` is the road network, `Map ` the mission scene database (AICo/AIEl/AISp/
//!    Trgr/…/aq#N placement arrays) [D]. `extract_collision_tris` decodes `ci#0` + `ca#0` [M];
//!    `extract_map_instances` / `extract_map_triggers` read `Map` (`in#0`, `Trgr`) [D, the
//!    instance matrix direction is not traced].

use crate::buf::{invalid, FormatError, Result};
use crate::crp::{tag, Crp, Record};

/// `Base` — an article's bounding-box record [D measured].
pub const TAG_BASE: u32 = 0x4261_7365;
/// `as  ` — a part reference record; short-tag instances carry the index in the low half [D].
pub const TAG_AS: u32 = 0x6173_2020;
/// `sr  ` — a named resource reference (payload = the EAGL symbol string) [D].
pub const TAG_SR: u32 = 0x7372_2020;
/// `CGrd` — the CDat collision-grid header record [D measured].
pub const TAG_CGRD: u32 = 0x4347_5264;
/// `ci  ` — CDat collision items [D measured].
pub const TAG_CI: u32 = 0x6369_2020;
/// `cn  ` — CDat collision nodes [D measured].
pub const TAG_CN: u32 = 0x636e_2020;
/// `ca  ` — article collision strip mesh (`ca#0`) [D measured / M 0x216EC0].
pub const TAG_CA: u32 = 0x6361_2020;
/// `in  ` — article 4x4 instance transform (64 bytes, row-major f32) [D measured].
pub const TAG_IN: u32 = 0x696e_2020;

/// One world-space triangle extracted from `.crp` `CDat` (`ci#0` + `Arti` `ca#0`)
/// [M 0x2138b0 / 0x216ec0; layout in `Crp::extract_collision_tris`].
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionTri {
    /// World-space corners, wound for the game's start-winding-0 test [M 0x2138b0].
    pub v: [[f32; 3]; 3],
    /// The triangle word's low half: bits 0-7 surface byte (+0x2c), bits 8-15 class byte
    /// (+0x2d) [M 0x2138b0; 0x213ff8 copies both into the hit record].
    pub flags: u16,
}

impl CollisionTri {
    /// The surface byte (+0x2c), copied into the hit record by the ground query [M 0x213ff8].
    pub fn surface(&self) -> u8 {
        self.flags as u8
    }
    /// The class byte (+0x2d), tested against the querier's mask [M 0x2138b0, 0x213d68].
    pub fn class(&self) -> u8 {
        (self.flags >> 8) as u8
    }
    /// Class bit 0x04: the ground query skips the triangle [M 0x2138b0].
    pub fn ground_query_skips(&self) -> bool {
        self.class() & 4 != 0
    }
}

/// One placed article instance from `.crp` `Map` (`in#0` + `sr`) with its inverted
/// local-to-world 3x4 transform (`v_world = (v_local - tr) * M^T / |r0|^2`) [D measured].
#[derive(Clone, Debug, PartialEq)]
pub struct MapInstance {
    pub article: String,
    pub r0: [f32; 3],
    pub r1: [f32; 3],
    pub r2: [f32; 3],
    pub tr: [f32; 3],
    pub inv_scale2: f32,
}

impl MapInstance {
    #[inline]
    pub fn transform(&self, p: [f32; 3]) -> [f32; 3] {
        let dx = p[0] - self.tr[0];
        let dy = p[1] - self.tr[1];
        let dz = p[2] - self.tr[2];
        [
            (dx * self.r0[0] + dy * self.r1[0] + dz * self.r2[0]) * self.inv_scale2,
            (dx * self.r0[1] + dy * self.r1[1] + dz * self.r2[1]) * self.inv_scale2,
            (dx * self.r0[2] + dy * self.r1[2] + dz * self.r2[2]) * self.inv_scale2,
        ]
    }
}

/// One trigger volume from `.crp` `Map` `Trgr` (64-byte records, `[x, y, z, radius]` at
/// `+0..+16`, flags at `+16`, id at `+24`) [D measured].
#[derive(Clone, Debug, PartialEq)]
pub struct MapTrigger {
    pub pos: [f32; 3],
    pub radius: f32,
    pub flags: u32,
    pub id: u32,
}

/// A parsed track article: name + bbox + the part references [D].
#[derive(Clone, Debug, PartialEq)]
pub struct ArticleMeta {
    /// The article's `Name` payload (`S6_CustomLowRBnknRoad`, `sky02`, …) [M 0x2dc1b0].
    pub name: String,
    /// `Base` payload floats: min at +0x10, max at +0x20 [D measured].
    pub bbox_min: [f32; 3],
    pub bbox_max: [f32; 3],
    /// The article's `as#N` part references [D measured].
    pub parts: Vec<PartRef>,
}

/// One `as#N` record's render reference: the `sr#<index>` record in the same article window names
/// the EAGL resource (`EAGL::<article>::{as  0000}::Model[0].NN`) [D measured]. The tail word is
/// `{u16 sr_index, "sr"}` on the geometry pieces [D: `as#0`→`sr#5`, `as#1`→`sr#6` on
/// `S6_CustomLowRBnknRoad`]; other articles carry `as` records whose tail does not end in "sr"
/// [D measured: MIS01 `trafficlight` as#1 tail `00 02 05 00`] — those get `sr_index: None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartRef {
    /// The `as` record's instance index.
    pub as_instance: u16,
    /// The `sr` record's instance index inside the same article window, when the payload ends in
    /// the `{u16, "sr"}` reference form [D measured].
    pub sr_index: Option<u16>,
}

fn u32_at(data: &[u8], o: usize) -> Result<u32> {
    let b = data.get(o..o + 4).ok_or_else(|| {
        FormatError::Invalid(format!("u32 read at {o:#x} past the end ({})", data.len()))
    })?;
    Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn f32_at(data: &[u8], o: usize) -> Result<f32> {
    Ok(f32::from_bits(u32_at(data, o)?))
}

fn payload_string(data: &[u8]) -> String {
    let s = data.split(|&b| b == 0).next().unwrap_or(&[]);
    String::from_utf8_lossy(s).into_owned()
}

/// The inverse of a `ci` item's world-to-local matrix: rows r0 (+0x00), r1, r2 (+0x20) and
/// translation (+0x30); r1 = (0,1,0) unless flags (+0x18) & 3, then r2 × r0
/// [M 0x212ed0: `FUN_0025a8e8(r2, r0)` / VU `vopmula r2,r0` + `vopmsub r0,r2`].
/// Returns (M⁻¹, translation) so that world = (local − translation) · M⁻¹.
fn item_local_to_world(item: &[u8]) -> Result<([[f32; 3]; 3], [f32; 3])> {
    let row = |o: usize| -> Result<[f32; 3]> {
        Ok([f32_at(item, o)?, f32_at(item, o + 4)?, f32_at(item, o + 8)?])
    };
    let r0 = row(0x00)?;
    let r2 = row(0x20)?;
    let t = row(0x30)?;
    let r1 = if u32_at(item, 0x18)? & 3 == 0 {
        [0.0, 1.0, 0.0]
    } else {
        [
            r2[1] * r0[2] - r2[2] * r0[1],
            r2[2] * r0[0] - r2[0] * r0[2],
            r2[0] * r0[1] - r2[1] * r0[0],
        ]
    };
    let m = [r0, r1, r2];
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if !det.is_finite() || det.abs() < 1e-8 {
        return invalid(format!("ci matrix is singular (det {det})"));
    }
    let mut inv = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            // inv[i][j] = cofactor(j, i) / det (cyclic-index form of the adjugate).
            let (a, b) = ((j + 1) % 3, (j + 2) % 3);
            let (c, d) = ((i + 1) % 3, (i + 2) % 3);
            inv[i][j] = (m[a][c] * m[b][d] - m[a][d] * m[b][c]) / det;
        }
    }
    Ok((inv, t))
}

/// Appends the world triangles of one `ca` record placed by `to_world` (the layout and its
/// evidence: `Crp::extract_collision_tris`).
fn ca_triangles(
    ca: &[u8],
    to_world: &([[f32; 3]; 3], [f32; 3]),
    out: &mut Vec<CollisionTri>,
) -> std::result::Result<(), String> {
    let (inv, t) = to_world;
    let xf = |p: [f32; 3]| -> [f32; 3] {
        let d = [p[0] - t[0], p[1] - t[1], p[2] - t[2]];
        [
            d[0] * inv[0][0] + d[1] * inv[1][0] + d[2] * inv[2][0],
            d[0] * inv[0][1] + d[1] * inv[1][1] + d[2] * inv[2][1],
            d[0] * inv[0][2] + d[1] * inv[1][2] + d[2] * inv[2][2],
        ]
    };
    let rd = |o: usize| -> std::result::Result<u32, String> {
        ca.get(o..o + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .ok_or_else(|| format!("ca read at {o:#x} past the end ({} B)", ca.len()))
    };
    if ca.len() < 0x20 {
        return Err(format!("ca record {} B < the 0x20-byte header", ca.len()));
    }
    let groups = u16::from_le_bytes([ca[0], ca[1]]) as usize;
    for g in 0..groups {
        let gh = 0x20 + g * 0x10;
        // +0x0e u16: the strip offset from +0x20 [M 0x216ec0 `lhu 0xe` + 0x20].
        let voff = (rd(gh + 0x0c)? >> 16) as usize;
        let base = 0x20 + voff;
        // Vertex 0's word: the vertex count, an integer `lw` [M 0x217000].
        let count = rd(base + 0x0c)? as usize;
        if count > 0x10000 || base + count * 0x10 > ca.len() {
            return Err(format!(
                "group {g}: strip of {count} vertices at {base:#x} overruns {} B",
                ca.len()
            ));
        }
        if count < 3 {
            continue;
        }
        let vert = |k: usize| -> std::result::Result<[f32; 3], String> {
            let o = base + k * 0x10;
            Ok([f32::from_bits(rd(o)?), f32::from_bits(rd(o + 4)?), f32::from_bits(rd(o + 8)?)])
        };
        // Vertex 1's word bit 0: the start winding, toggled per triangle [M 0x2138b0].
        let mut parity = rd(base + 0x1c)? & 1;
        for k in 0..count - 2 {
            let word = rd(base + (k + 2) * 0x10 + 0x0c)?;
            let (a, mut b, mut c) = (xf(vert(k)?), xf(vert(k + 1)?), xf(vert(k + 2)?));
            if parity == 1 {
                std::mem::swap(&mut b, &mut c);
            }
            out.push(CollisionTri { v: [a, b, c], flags: word as u16 });
            parity ^= 1;
        }
    }
    Ok(())
}

impl<'a> Crp<'a> {
    /// Every pool article's mesh metadata: name, bounding box, part references. Errors only on
    /// malformed records (the game's data always carries `Base`+`Name` [D, all 8 tracks]).
    pub fn track_articles(&self) -> Result<Vec<ArticleMeta>> {
        let mut out = Vec::new();
        for arti in self.pool()? {
            if arti.tag != tag::ARTI {
                continue;
            }
            let (start, end) = arti.window(self.data)?;
            let mut meta = ArticleMeta {
                name: String::new(),
                bbox_min: [0.0; 3],
                bbox_max: [0.0; 3],
                parts: Vec::new(),
            };
            for o in (start..end).step_by(0x10) {
                let rec = Record::parse(self.data, o)?;
                if rec.tag == tag::NAME {
                    meta.name = payload_string(self.payload(&rec)?);
                } else if rec.tag == TAG_BASE {
                    let p = self.payload(&rec)?;
                    if p.len() < 0x2c {
                        return invalid(format!(
                            "article {}: Base payload {} < 0x2c bytes",
                            meta.name,
                            p.len()
                        ));
                    }
                    for i in 0..3 {
                        meta.bbox_min[i] = f32_at(p, 0x10 + i * 4)?;
                        meta.bbox_max[i] = f32_at(p, 0x20 + i * 4)?;
                    }
                } else if rec.tag_code() == TAG_AS {
                    let p = self.payload(&rec)?;
                    if p.len() < 4 {
                        return invalid(format!(
                            "article {}: as#{} payload {} < 4 bytes",
                            meta.name,
                            rec.instance().unwrap_or(0),
                            p.len()
                        ));
                    }
                    let tail = u32_at(p, p.len() - 4)?;
                    // The tail word is {u16 sr_index, "sr"} on the geometry pieces [D measured:
                    // 05 00 72 73]; other articles' `as` records carry other tails [D: MIS01
                    // `trafficlight` as#1 = 00 02 05 00] — recorded without the sr reference.
                    let sr_index = (tail >> 16 == 0x7372).then_some(tail as u16);
                    meta.parts.push(PartRef {
                        as_instance: rec.instance().unwrap_or(0),
                        sr_index,
                    });
                }
            }
            out.push(meta);
        }
        Ok(out)
    }

    /// An article window's `sr#<index>` payload string (`EAGL::<name>::{as  0000}::Model[0].NN`)
    /// — the EAGL symbol reference a `PartRef` points at [D measured].
    pub fn article_sr_string(&self, arti: &Record, index: u16) -> Result<Option<String>> {
        let (start, end) = arti.window(self.data)?;
        for o in (start..end).step_by(0x10) {
            let rec = Record::parse(self.data, o)?;
            if rec.tag_code() == TAG_SR && rec.instance() == Some(index) {
                return Ok(Some(payload_string(self.payload(&rec)?)));
            }
        }
        Ok(None)
    }

    /// The pool `CDat` article's record census — the collision world's top structure [D measured
    /// on paris: `CGrd` 36 B, `ci#0` 63,616 B with count=994 items, ~1600 `cn#N` nodes].
    pub fn collision_census(&self) -> Result<CollisionCensus> {
        let mut out = CollisionCensus {
            cdat_found: false,
            grd_records: 0,
            ci_records: 0,
            ci_total_items: 0,
            cn_records: 0,
        };
        for rec in self.pool()? {
            if rec.tag != tag::CDAT {
                continue;
            }
            out.cdat_found = true;
            let (start, end) = rec.window(self.data)?;
            for o in (start..end).step_by(0x10) {
                let r = Record::parse(self.data, o)?;
                match r.tag_code() {
                    TAG_CGRD => out.grd_records += 1,
                    TAG_CI => {
                        out.ci_records += 1;
                        out.ci_total_items += r.count.max(0) as usize;
                    }
                    TAG_CN => out.cn_records += 1,
                    _ => {}
                }
            }
        }
        Ok(out)
    }

    /// Every world-space collision triangle of the track: each `CDat` `ci#0` instance placed with
    /// its article's `ca#0` strip mesh.
    ///
    /// Layouts, all from the game's collision code [M DRIVING.ELF, EE = file offset + 0x107080]:
    ///
    /// - `ci` item (64 B): +0x00 row 0 with the local half-extent x at +0x0c; +0x18 flags (bits
    ///   0-1 set: the y row is `r2 × r0` instead of (0,1,0); bit 0 also flips the height
    ///   compare); +0x1c the article reference (`{u16 index, "sr"}` on disc, a `ca` pointer once
    ///   loaded); +0x20 row 2 with the half-extent z at +0x2c; +0x30 row 3 (translation); +0x3c
    ///   the bounding radius [M 0x212ed0 builds the matrix; 0x213ff8 reads +0x0c/+0x18/+0x1c/
    ///   +0x2c; 0x217228 reads +0x3c]. The matrix maps WORLD to the article's LOCAL frame: the
    ///   ground query transforms the world point by it (0x213ff8 → 0x212ed0 → 0x25b890) and
    ///   tests the untransformed `ca` vertices against the result, so world vertices are
    ///   `(local − row3) · M⁻¹`.
    /// - `ca`: +0x00 u16 group count; groups of 16 B from +0x20 (float centre, +0x0c u16 radius ×
    ///   1/16, +0x0e u16 strip offset from +0x20) — one strip per group [M 0x216ec0, 0x213ff8].
    /// - strip: 16-byte vertices (x, y, z, word). Vertex 0's word = the vertex count (`lw`);
    ///   vertex 1's word = strip flags (bit 0: start winding, toggled per triangle; bit 1: both
    ///   windings accepted); triangle k = vertices k, k+1, k+2 for k in 0..count-2, described by
    ///   vertex k+2's word: byte +0x2c surface, byte +0x2d class (bit 0x04: skipped by the ground
    ///   query; the rest is tested against the querier's class mask), u16 +0x2e radius × 1/16
    ///   [M 0x2138b0, 0x213d68, 0x216ec0].
    ///
    /// Every triangle is emitted (none filtered); `flags` = vertex k+2's low half. Odd-parity
    /// triangles swap `v[1]`/`v[2]` so all pass the game's winding-0 test (all three xz edge
    /// functions ≥ 0) [M 0x2138b0]. A `ci` item whose `sr` does not reach an article `ca` record
    /// is an error: every item on the disc resolves [D crp_mesh_disc test].
    pub fn extract_collision_tris(&self) -> Result<Vec<CollisionTri>> {
        use std::collections::HashMap;
        let mut arti_ca: HashMap<String, &[u8]> = HashMap::new();
        let mut cdat_sr: HashMap<u16, String> = HashMap::new();
        let mut ci_payloads: Vec<&[u8]> = Vec::new();

        for rec in self.pool()? {
            if rec.tag == tag::ARTI {
                let (start, end) = rec.window(self.data)?;
                let mut name: Option<String> = None;
                let mut ca: Option<&[u8]> = None;
                for o in (start..end).step_by(0x10) {
                    let r = Record::parse(self.data, o)?;
                    if r.tag == tag::NAME {
                        name = Some(payload_string(self.payload(&r)?));
                    } else if r.tag_code() == TAG_CA {
                        ca = Some(self.payload(&r)?);
                    }
                }
                if let (Some(n), Some(c)) = (name, ca) {
                    arti_ca.insert(n, c);
                }
            } else if rec.tag == tag::CDAT {
                let (start, end) = rec.window(self.data)?;
                for o in (start..end).step_by(0x10) {
                    let r = Record::parse(self.data, o)?;
                    if r.tag_code() == TAG_SR {
                        if let Some(inst) = r.instance() {
                            let s = payload_string(self.payload(&r)?);
                            let clean = s.strip_prefix("CARP::").unwrap_or(&s).to_string();
                            cdat_sr.insert(inst, clean);
                        }
                    } else if r.tag_code() == TAG_CI {
                        ci_payloads.push(self.payload(&r)?);
                    }
                }
            }
        }

        let mut tris = Vec::new();
        for ci_data in ci_payloads {
            if ci_data.len() % 64 != 0 {
                return invalid(format!(
                    "ci payload {} B is not a whole number of 64-byte items",
                    ci_data.len()
                ));
            }
            for (ii, item) in ci_data.chunks_exact(64).enumerate() {
                let sr_tag = u32_at(item, 0x1c)?;
                if sr_tag >> 16 != 0x7372 {
                    return invalid(format!("ci item {ii}: +0x1c {sr_tag:#010x} is not an sr reference"));
                }
                let sr_idx = sr_tag as u16;
                let aname = cdat_sr.get(&sr_idx).ok_or_else(|| {
                    FormatError::Invalid(format!("ci item {ii}: no CDat sr#{sr_idx}"))
                })?;
                let ca = *arti_ca.get(aname).ok_or_else(|| {
                    FormatError::Invalid(format!("ci item {ii}: article {aname} has no ca record"))
                })?;
                let to_world = item_local_to_world(item)?;
                ca_triangles(ca, &to_world, &mut tris)
                    .map_err(|e| FormatError::Invalid(format!("ci item {ii} ({aname}): {e}")))?;
            }
        }
        Ok(tris)
    }

    /// Extract all placed world article instances from `Map` (`in#0` 64-byte world-to-local
    /// matrices + `sr` article symbol references) [D measured on `paris_mis01.crp`].
    pub fn extract_map_instances(&self) -> Result<Vec<MapInstance>> {
        use std::collections::HashMap;
        let mut map_sr: HashMap<u16, String> = HashMap::new();
        let mut in_payloads: Vec<&[u8]> = Vec::new();

        for rec in self.pool()? {
            if rec.tag != tag::MAP {
                continue;
            }
            let (start, end) = rec.window(self.data)?;
            for o in (start..end).step_by(0x10) {
                let r = Record::parse(self.data, o)?;
                if r.tag_code() == TAG_SR {
                    if let Some(inst) = r.instance() {
                        let s = payload_string(self.payload(&r)?);
                        let clean = s.strip_prefix("CARP::").unwrap_or(&s);
                        let art = clean.split("::{").next().unwrap_or(clean).to_string();
                        map_sr.insert(inst, art);
                    }
                } else if r.tag_code() == TAG_IN {
                    in_payloads.push(self.payload(&r)?);
                }
            }
        }

        let mut out = Vec::new();
        for in_data in in_payloads {
            for item in in_data.chunks_exact(64) {
                let sr_tag = u32::from_le_bytes(item[28..32].try_into().unwrap());
                if (sr_tag >> 16) != 0x7372 {
                    continue;
                }
                let sr_idx = (sr_tag & 0xffff) as u16;
                let Some(article) = map_sr.get(&sr_idx) else {
                    continue;
                };
                let r0 = [
                    f32::from_le_bytes(item[0..4].try_into().unwrap()),
                    f32::from_le_bytes(item[4..8].try_into().unwrap()),
                    f32::from_le_bytes(item[8..12].try_into().unwrap()),
                ];
                let r1 = [
                    f32::from_le_bytes(item[16..20].try_into().unwrap()),
                    f32::from_le_bytes(item[20..24].try_into().unwrap()),
                    f32::from_le_bytes(item[24..28].try_into().unwrap()),
                ];
                let r2 = [
                    f32::from_le_bytes(item[32..36].try_into().unwrap()),
                    f32::from_le_bytes(item[36..40].try_into().unwrap()),
                    f32::from_le_bytes(item[40..44].try_into().unwrap()),
                ];
                let tr = [
                    f32::from_le_bytes(item[48..52].try_into().unwrap()),
                    f32::from_le_bytes(item[52..56].try_into().unwrap()),
                    f32::from_le_bytes(item[56..60].try_into().unwrap()),
                ];
                let s2 = r0[0] * r0[0] + r0[1] * r0[1] + r0[2] * r0[2];
                if s2 < 1e-6 {
                    continue;
                }
                out.push(MapInstance {
                    article: article.clone(),
                    r0,
                    r1,
                    r2,
                    tr,
                    inv_scale2: 1.0 / s2,
                });
            }
        }
        Ok(out)
    }

    /// Extract all `Trgr` trigger records from `Map` (64-byte records, `[x, y, z, radius]` at
    /// `+0..+16`).
    pub fn extract_map_triggers(&self) -> Result<Vec<MapTrigger>> {
        const TAG_TRGR: u32 = 0x5472_6772; // "Trgr"
        let mut out = Vec::new();
        for rec in self.pool()? {
            if rec.tag != tag::MAP {
                continue;
            }
            let (start, end) = rec.window(self.data)?;
            for o in (start..end).step_by(0x10) {
                let r = Record::parse(self.data, o)?;
                if r.tag != TAG_TRGR {
                    continue;
                }
                let payload = self.payload(&r)?;
                for item in payload.chunks_exact(64) {
                    let pos = [
                        f32::from_le_bytes(item[0..4].try_into().unwrap()),
                        f32::from_le_bytes(item[4..8].try_into().unwrap()),
                        f32::from_le_bytes(item[8..12].try_into().unwrap()),
                    ];
                    let radius = f32::from_le_bytes(item[12..16].try_into().unwrap());
                    let flags = u32::from_le_bytes(item[16..20].try_into().unwrap());
                    let id = u32::from_le_bytes(item[24..28].try_into().unwrap());
                    out.push(MapTrigger { pos, radius, flags, id });
                }
            }
        }
        Ok(out)
    }
}

/// The pool `CDat` article's record census [D measured].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CollisionCensus {
    pub cdat_found: bool,
    pub grd_records: usize,
    pub ci_records: usize,
    /// Sum of the `ci` records' `count` fields (paris: 994) [D measured].
    pub ci_total_items: usize,
    pub cn_records: usize,
}

/// The combined `ELFd`+`ELFr` MIPS module of one `.crp`, with the game's address resolution and
/// relocations applied [M 0x1a9600 → 0x29f820 → 0x29ea00].
pub struct TrackModule {
    /// The combined image bytes (ELFd payload, then ELFr payload), relocations applied.
    pub image: Vec<u8>,
    /// The ELFd payload size — the split the resolver uses [M 0x1a9600/0x29ea00].
    pub elfd_size: u32,
    /// Section headers in ELF order (offset = the game's resolved `sh_addr` [M 0x29ea00]).
    pub sections: Vec<ModuleSection>,
    /// Symbols in ELF order (name, flat st_value, size, info, shndx).
    pub symbols: Vec<ModuleSymbol>,
    /// Number of relocations applied.
    pub relocations: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleSection {
    pub name: String,
    pub sh_type: u32,
    /// The game's resolved `sh_addr` — a runtime pointer in the combined-image space
    /// [M 0x29ea00 resolves `sh_addr`; the loader never uses `sh_offset` for addressing].
    pub offset: u32,
    /// The section's file offset (`sh_offset`) into the combined image — where the DATA lives
    /// [D measured: paris `.symtab` sh_offset 0x3e9e70 holds the symbol table the loader reads].
    pub data_offset: u32,
    pub size: u32,
    pub link: u32,
    pub entsize: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleSymbol {
    pub name: String,
    /// Flat value in the combined-image space [D verified via relocation targets].
    pub value: u32,
    pub size: u32,
    pub info: u8,
    pub shndx: u16,
}

impl ModuleSymbol {
    /// Object-file binding/type nibbles.
    pub fn kind(&self) -> u8 { self.info & 0xf }
    pub fn binding(&self) -> u8 { self.info >> 4 }
}

impl TrackModule {
    /// Loads the module: `ELFd`+`ELFr` concatenated, section offsets resolved by the game's rule,
    /// symbols read, then every `.rel.data` entry applied (`R_MIPS_32`: `word += symbol value`)
    /// [M 0x29ea00 + 0x2bb300; measured: every entry of all 8 mission tracks is type 2].
    pub fn load(crp: &Crp<'_>) -> Result<TrackModule> {
        let dir = crp.directory()?;
        let elfd = dir
            .iter()
            .find(|r| r.tag == tag::ELFD)
            .ok_or_else(|| FormatError::Invalid("no ELFd record in the root window".into()))?;
        let elfd_size = elfd.size() as u32;
        let mut image = crp.payload(elfd)?.to_vec();
        let elfr = dir
            .iter()
            .find(|r| r.tag == tag::ELFR)
            .ok_or_else(|| FormatError::Invalid("no ELFr record in the root window".into()))?;
        image.extend_from_slice(crp.payload(elfr)?);

        if image.len() < 0x34 || &image[..4] != b"\x7fELF" {
            return invalid(format!("combined module image starts with {:?} ({})", &image[..4.min(image.len())], image.len()));
        }
        let e_shoff = u32_at(&image, 0x20)? as usize;
        let e_shentsize = u16::from_le_bytes([image[0x2e], image[0x2f]]) as usize;
        let e_shnum = u16::from_le_bytes([image[0x30], image[0x31]]) as usize;
        let shstrndx = u16::from_le_bytes([image[0x32], image[0x33]]) as usize;
        if e_shentsize != 0x28 {
            return invalid(format!("unexpected e_shentsize {e_shentsize:#x} (measured 0x28)"));
        }
        // The resolver: x < elfd_size ? ELFd+x : ELFr+(x-elfd_size) [M 0x29ea00].
        let resolve = |x: u32| -> Result<usize> {
            let at = if x < elfd_size { x as usize } else { elfd_size as usize + (x - elfd_size) as usize };
            if at > image.len() {
                return invalid(format!(
                    "resolved offset {at:#x} past the combined image ({})",
                    image.len()
                ));
            }
            Ok(at)
        };
        let sh_at = resolve(e_shoff as u32)?;
        if sh_at + e_shnum * 0x28 > image.len() {
            return invalid(format!(
                "section headers [{sh_at:#x}..{:#x}) past the combined image ({})",
                sh_at + e_shnum * 0x28,
                image.len()
            ));
        }
        let mut raw = Vec::with_capacity(e_shnum);
        for i in 0..e_shnum {
            let o = sh_at + i * 0x28;
            raw.push([
                u32_at(&image, o)?,
                u32_at(&image, o + 4)?,
                u32_at(&image, o + 8)?,
                u32_at(&image, o + 12)?,
                u32_at(&image, o + 16)?,
                u32_at(&image, o + 20)?,
                u32_at(&image, o + 24)?,
                u32_at(&image, o + 28)?,
                u32_at(&image, o + 32)?,
                u32_at(&image, o + 36)?,
            ]);
        }
        // raw = [name, type, flags, addr, offset, size, link, info, align, entsize]
        let mut sections: Vec<ModuleSection> = raw
            .iter()
            .map(|s| -> Result<ModuleSection> {
                Ok(ModuleSection {
                    name: String::new(),
                    sh_type: s[1],
                    offset: resolve(s[3])? as u32,
                    data_offset: s[4],
                    size: s[5],
                    link: s[6],
                    entsize: s[9],
                })
            })
            .collect::<Result<_>>()?;
        if shstrndx < sections.len() {
            let (st_off, st_size) = (sections[shstrndx].data_offset as usize, sections[shstrndx].size as usize);
            if st_off + st_size > image.len() {
                return invalid(format!(".shstrtab [{st_off:#x}..{:#x}) past the image", st_off + st_size));
            }
            for (i, s) in sections.iter_mut().enumerate() {
                let no = raw[i][0] as usize;
                let base = st_off.saturating_add(no);
                if base >= st_off + st_size {
                    continue;
                }
                let end = image[base..st_off + st_size].iter().position(|&b| b == 0).map(|p| base + p).unwrap_or(base);
                s.name = String::from_utf8_lossy(&image[base..end]).into_owned();
            }
        }
        // Symbols: the SYMTAB section (type 2), names via its sh_link [M 0x29ea00 resolves the
        // linked STRTAB the same way].
        let sym_idx = sections
            .iter()
            .position(|s| s.sh_type == 2)
            .ok_or_else(|| FormatError::Invalid("no SYMTAB section in the module".into()))?;
        let (sym_off, sym_size, sym_link) = {
            let s = &sections[sym_idx];
            (s.data_offset as usize, s.size as usize, s.link as usize)
        };
        let str_off = sections.get(sym_link)
            .map(|s| s.data_offset as usize)
            .ok_or_else(|| FormatError::Invalid("SYMTAB sh_link out of range".into()))?;
        let mut symbols = Vec::with_capacity(sym_size / 16);
        for i in 0..sym_size / 16 {
            let o = sym_off + i * 16;
            let nm = u32_at(&image, o)? as usize;
            let name = if nm == 0 {
                String::new()
            } else {
                let base = str_off + nm;
                if base >= image.len() {
                    return invalid(format!("symbol name at {base:#x} runs past the image"));
                }
                let end = image[base..].iter().position(|&b| b == 0).map(|p| base + p).unwrap_or(base);
                if end > image.len() {
                    return invalid(format!("symbol name at {base:#x} runs past the image"));
                }
                String::from_utf8_lossy(&image[base..end]).into_owned()
            };
            symbols.push(ModuleSymbol {
                name,
                value: u32_at(&image, o + 4)?,
                size: u32_at(&image, o + 8)?,
                info: *image.get(o + 12).ok_or_else(|| FormatError::Invalid("symbol past the image".into()))?,
                shndx: u16::from_le_bytes([
                    *image.get(o + 14).unwrap_or(&0),
                    *image.get(o + 15).unwrap_or(&0),
                ]),
            });
        }
        let mut module = TrackModule { image, elfd_size, sections, symbols, relocations: 0 };
        module.relocations = module.apply_relocations()?;
        Ok(module)
    }

    /// Applies every `.rel.data` entry: `R_MIPS_32` (`word += symbol value`) [M 0x2bb300;
    /// measured: every entry of all 8 mission tracks is type 2 — this loader handles no other
    /// type, matching the game's DynamicLoader (U-004)]. Bounds-checked; unknown types error.
    pub fn apply_relocations(&mut self) -> Result<usize> {
        let rel = self
            .sections
            .iter()
            .find(|s| s.sh_type == 9)
            .cloned()
            .ok_or_else(|| FormatError::Invalid("no .rel.data section in the module".into()))?;
        let ent = if rel.entsize == 0 { 8 } else { rel.entsize as usize };
        let n = rel.size as usize / ent;
        let rel_off = rel.data_offset as usize;
        for i in 0..n {
            let o = rel_off + i * ent;
            let off = u32_at(&self.image, o)? as usize;
            let info = u32_at(&self.image, o + 4)?;
            let sym_idx = (info >> 8) as usize;
            let typ = info & 0xff;
            if typ != 2 {
                return invalid(format!("relocation {i}: type {typ} is not R_MIPS_32"));
            }
            let val = self
                .symbols
                .get(sym_idx)
                .ok_or_else(|| FormatError::Invalid(format!("relocation {i}: symbol {sym_idx} out of range")))?
                .value as u64;
            if off.saturating_add(4) > self.image.len() {
                return invalid(format!("relocation {i} target {off:#x} runs past the image"));
            }
            let word = u32_at(&self.image, off)? as u64;
            let sum = (word + val) & 0xffff_ffff;
            self.image[off..off + 4].copy_from_slice(&(sum as u32).to_le_bytes());
        }
        Ok(n)
    }

    /// The `__Model:::<article>::{as  0000}::Model[0].NN` symbols of one article, sorted by the
    /// `NN` segment number [D measured: paris 1,890 Model symbols over 494 articles].
    pub fn model_segments(&self, article: &str) -> Vec<(u8, u32)> {
        let needle = format!("__Model:::{article}::{{as  0000}}::Model[0].");
        let mut out = Vec::new();
        for s in &self.symbols {
            if let Some(rest) = s.name.strip_prefix(&needle) {
                if rest.len() == 2 {
                    if let Ok(seg) = u8::from_str_radix(rest, 16) {
                        out.push((seg, s.value));
                    }
                }
            }
        }
        out.sort();
        out.dedup_by(|a, b| a.0 == b.0);
        out
    }

    /// Like `model_segments`, but matches any `{as  NNNN}::Model[0].NN` variant (used by car `.crp`s).
    pub fn model_segments_any(&self, article: &str) -> Vec<(u8, u32)> {
        let prefix = format!("__Model:::{article}::{{as  ");
        let mut out = Vec::new();
        for s in &self.symbols {
            if let Some(after_p) = s.name.strip_prefix(&prefix) {
                if let Some(rest) = after_p.split("}::Model[0].").nth(1) {
                    if rest.len() == 2 {
                        if let Ok(seg) = u8::from_str_radix(rest, 16) {
                            out.push((seg, s.value));
                        }
                    }
                }
            }
        }
        out.sort();
        out.dedup_by(|a, b| a.0 == b.0);
        out
    }

    /// The `__GPRenderMethod_<article>::{as  0000}::Model[0].NN_<x>_name` strings in the module
    /// image (the EAGL GeoPrim render-method names) [D measured at paris 0x180544]. Returns
    /// (article, segment, image offset) triples.
    pub fn geoprim_names(&self) -> Vec<(String, u8, u32)> {
        const NEEDLE: &[u8] = b"__GPRenderMethod_";
        let mut out = Vec::new();
        let mut i = 0;
        while i + NEEDLE.len() <= self.image.len() {
            if &self.image[i..i + NEEDLE.len()] != NEEDLE {
                i += 1;
                continue;
            }
            let start = i + NEEDLE.len();
            let end = self.image[start..]
                .iter()
                .position(|&b| b == 0)
                .map(|e| start + e)
                .unwrap_or(start);
            let s = String::from_utf8_lossy(&self.image[start..end]).into_owned();
            if let Some(art_end) = s.find("::{as  0000}::Model[0].") {
                let article = s[..art_end].to_owned();
                let rest = &s[art_end + 23..];
                let seg_end = rest.find('_').unwrap_or(rest.len());
                if let Ok(seg) = u8::from_str_radix(&rest[..seg_end], 16) {
                    out.push((article, seg, i as u32));
                }
            }
            i = end + 1;
        }
        out
    }

    /// Decode the `{as  0000}` model of `article` (any `{as  NNNN}` when that is missing) into
    /// vertex batches in the article's local frame (decoding: `extract_segments`,
    /// `parse_vif_stream`). The vertex data is self-scaled (8.8 fixed point), so `bbox_min` /
    /// `bbox_max` are not used; they stay in the signature for existing callers.
    pub fn extract_article_mesh(
        &self,
        article: &str,
        bbox_min: [f32; 3],
        bbox_max: [f32; 3],
    ) -> Vec<MeshStrip> {
        let mut segs = self.model_segments(article);
        if segs.is_empty() {
            segs = self.model_segments_any(article);
        }
        self.extract_segments(&segs, bbox_min, bbox_max)
    }

    /// Decode every `{as  NNNN}` part of `article` (car `.crp` models such as `vanquish`, where
    /// chassis, wheels and attachments are separate `as` parts). `bbox_min` / `bbox_max` are not
    /// used (see `extract_article_mesh`).
    pub fn extract_all_article_parts(
        &self,
        article: &str,
        bbox_min: [f32; 3],
        bbox_max: [f32; 3],
    ) -> Vec<MeshStrip> {
        let prefix = format!("__Model:::{article}::{{as  ");
        let mut segs = Vec::new();
        for s in &self.symbols {
            if let Some(after_p) = s.name.strip_prefix(&prefix) {
                if let Some(rest) = after_p.split("}::Model[0].").nth(1) {
                    if rest.len() == 2 {
                        if let Ok(seg) = u8::from_str_radix(rest, 16) {
                            segs.push((seg, s.value));
                        }
                    }
                }
            }
        }
        segs.sort_by_key(|&(seg, val)| (val, seg));
        segs.dedup_by_key(|x| x.1);
        self.extract_segments(&segs, bbox_min, bbox_max)
    }

    /// Decode only the intact primary-variant (`vd#1` if present) parts of a car `.crp`,
    /// using the article's `Base` bounding box and `in#0` bone transforms (excluding EMP/damage
    /// spheres and alternate damage variants).
    ///
    /// Superseded by `carmesh::car_mesh`, which follows the game's car draw path. Known wrong here
    /// [M FUN_001d35d8 / FUN_001ad758 / FUN_001ad2e0]: `vd#N` lists `in#0` item indices, not `as`
    /// indices; an item names its `as` set at +0x1c, so `as#k` is not placed by item k (several
    /// items — mirrored pairs — share one set); each entry's batch is the one its render list's
    /// op 4 calls (`geoprim::ModelIndex::draw_chain`), not the one at `*(entry − 0x18)`; car UVs are
    /// /1024 and the `V3-8` bytes are 0x80-biased normals, not colours (docs/formats/car-model.md §9).
    pub fn extract_car_mesh(&self, crp: &crate::crp::Crp, fallback_name: &str) -> Vec<MeshStrip> {
        let Ok(articles) = crp.track_articles() else {
            return self.extract_all_article_parts(fallback_name, [-1.0, -1.0, -2.5], [1.0, 1.0, 2.5]);
        };
        let Some(art) = articles.first() else {
            return self.extract_all_article_parts(fallback_name, [-1.0, -1.0, -2.5], [1.0, 1.0, 2.5]);
        };

        let mut vd1: Option<Vec<u16>> = None;
        let mut in_bones: Vec<[f32; 16]> = Vec::new();
        let mut as_bone: std::collections::HashMap<u16, u16> = std::collections::HashMap::new();
        if let Ok(pool) = crp.pool() {
            for rec in pool {
                if rec.tag != crate::crp::tag::ARTI {
                    continue;
                }
                if let Ok((start, end)) = rec.window(crp.data) {
                    for o in (start..end).step_by(0x10) {
                        let Ok(r) = crate::crp::Record::parse(crp.data, o) else { continue };
                        // "vd  " = 0x7664_2020
                        if r.tag_code() == 0x7664_2020 && r.instance() == Some(1) {
                            if let Ok(p) = crp.payload(&r) {
                                vd1 = Some(
                                    p.chunks_exact(2)
                                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                        .collect(),
                                );
                            }
                        } else if r.tag_code() == TAG_IN {
                            if let Ok(p) = crp.payload(&r) {
                                for m in p.chunks_exact(64) {
                                    let mut mat = [0.0f32; 16];
                                    for i in 0..16 {
                                        mat[i] = f32::from_le_bytes(m[i * 4..i * 4 + 4].try_into().unwrap());
                                    }
                                    in_bones.push(mat);
                                }
                            }
                        } else if r.tag_code() == 0x6173_2020 {
                            if let (Some(as_idx), Ok(p)) = (r.instance(), crp.payload(&r)) {
                                if p.len() >= 0x1a {
                                    as_bone.insert(as_idx, u16::from_le_bytes([p[0x18], p[0x19]]));
                                }
                            }
                        }
                    }
                }
                break;
            }
        }

        let prefix = format!("__Model:::{}::{{as  ", art.name);
        let mut by_as: std::collections::BTreeMap<u16, Vec<(u8, u32)>> =
            std::collections::BTreeMap::new();
        for s in &self.symbols {
            if let Some(after_p) = s.name.strip_prefix(&prefix) {
                let mut parts = after_p.split("}::Model[0].");
                if let (Some(as_hex), Some(rest)) = (parts.next(), parts.next()) {
                    if rest.len() == 2 {
                        if let (Ok(as_idx), Ok(seg)) = (
                            u16::from_str_radix(as_hex, 16),
                            u8::from_str_radix(rest, 16),
                        ) {
                            if vd1.as_ref().map_or(true, |allowed| allowed.contains(&as_idx)) {
                                by_as.entry(as_idx).or_default().push((seg, s.value));
                            }
                        }
                    }
                }
            }
        }

        let mut out = Vec::new();
        for (as_idx, mut segs) in by_as {
            segs.sort_by_key(|&(seg, val)| (val, seg));
            segs.dedup_by_key(|x| x.1);
            let mut strips = self.extract_segments(&segs, art.bbox_min, art.bbox_max);
            if let Some(&bone_idx) = as_bone.get(&as_idx) {
                if let Some(m) = in_bones.get(bone_idx as usize) {
                    for strip in &mut strips {
                        for p in &mut strip.positions {
                            let [x, y, z] = *p;
                            *p = [
                                m[0] * x + m[4] * y + m[8] * z + m[12],
                                m[1] * x + m[5] * y + m[9] * z + m[13],
                                m[2] * x + m[6] * y + m[10] * z + m[14],
                            ];
                        }
                    }
                }
            }
            out.extend(strips);
        }
        out
    }

    /// Walks the descriptors' batch tables and decodes every entry's geometry.
    ///
    /// Descriptor (the `__Model:::` symbol target): +0xc8 group count, +0xcc the batch table
    /// `[0xA0000000][0][2N]` then N × `[0xA000FFFF][entry]` [D measured, carmodel.rs]. An entry
    /// `E` is NOT the start of its own geometry: the word at `E − 0x18` points at the entry's
    /// render-method command list, and that list's `0x04000002 <addr>` command names the DMA
    /// chain that draws it — a CALL (sometimes carrying its own qwords) into the RET-tagged VIF
    /// packet that sits just BEFORE `E` in memory [D measured, all 8 tracks: the decoded
    /// extents land inside the article's `Base` box for ~97% of articles this way and ~25% when
    /// the packet at `E + 8` is decoded instead]. The DMA tag semantics are the EE DMAC's
    /// (CNT/NEXT/REF/REFE/CALL/RET/END).
    fn extract_segments(
        &self,
        segs: &[(u8, u32)],
        bbox_min: [f32; 3],
        bbox_max: [f32; 3],
    ) -> Vec<MeshStrip> {
        let _ = (bbox_min, bbox_max); // self-scaled vertex data: the box is not needed
        let mut strips = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for &(_seg, addr) in segs {
            let Some(off) = self.addr_to_offset(addr) else {
                continue;
            };
            let group_count = self.u32_at_img(off + 0xc8).unwrap_or(1).min(1024) as usize;
            let Some(tbl_addr) = self.u32_at_img(off + 0xcc) else {
                continue;
            };
            let Some(mut cur) = self.addr_to_offset(tbl_addr) else {
                continue;
            };
            let limit = self.image.len().saturating_sub(8);
            let mut entries = Vec::new();
            for _ in 0..group_count {
                if cur + 12 > limit || self.u32_at_img(cur) != Some(0xA000_0000) {
                    break;
                }
                let pairs = self.u32_at_img(cur + 8).unwrap_or(0) as usize / 2;
                cur += 12;
                for _ in 0..pairs {
                    if cur + 8 > limit || self.u32_at_img(cur) != Some(0xA000_FFFF) {
                        break;
                    }
                    entries.push(self.u32_at_img(cur + 4).unwrap_or(0));
                    cur += 8;
                }
            }
            // A packet whose render-method list sets no texture keeps the previous packet's within
            // the model (docs/formats/geoprim.md, `ModelIndex::model_strips`) [D].
            let mut carried: Option<String> = None;
            for entry in entries {
                if !seen.insert(entry) {
                    continue;
                }
                let Some(eo) = self.addr_to_offset(entry) else {
                    continue;
                };
                let Some(stream) = self.entry_dma_start(entry).and_then(|a| self.dma_chain(a)) else {
                    continue;
                };
                if let Some(t) = self.packet_texture(eo) { carried = Some(t); }
                let texture = carried.clone();
                for mut strip in self.parse_vif_stream(&stream) {
                    strip.texture = texture.clone();
                    strips.push(strip);
                }
            }
        }
        strips
    }

    /// The DMA address an entry's render-method list draws: the first `0x04000002 <addr>`
    /// command in the list at `*(entry − 0x18)` whose `addr` holds a DMA tag [D measured on all
    /// 8 tracks; the list ends at its `__GPRenderMethod_…` name string].
    fn entry_dma_start(&self, entry: u32) -> Option<u32> {
        let head = self.addr_to_offset(entry.checked_sub(0x18)?)?;
        let list = self.addr_to_offset(self.u32_at_img(head)?)?;
        let mut prev = 0u32;
        for k in 0..256 {
            let w = self.u32_at_img(list + k * 4)?;
            if w == 0x5047_5f5f {
                break; // "__GP": the render method's name string follows its list
            }
            if prev == 0x0400_0002 && w % 16 == 0 {
                if let Some(to) = self.addr_to_offset(w) {
                    let id = (self.u32_at_img(to)? >> 28) & 7;
                    if matches!(id, 1 | 2 | 5 | 6 | 7) {
                        return Some(w);
                    }
                }
            }
            prev = w;
        }
        None
    }

    /// Runs a DMA chain from `start` the way the EE DMAC sends it to VIF1 with tag transfer on:
    /// each tag's upper 64 bits (two VIF codes) then its qwords. CNT/NEXT/CALL/RET/END data
    /// follows the tag, REF/REFE data sits at the tag's ADDR; CALL pushes the address after its
    /// data, RET pops (an empty stack ends the chain). A word that is not a tag ends it too: an
    /// entry may be a run of CALLs, one per sub-packet, followed directly by render-method
    /// commands [D measured]. Bounded: 4096 tags, call depth 4.
    fn dma_chain(&self, start: u32) -> Option<Vec<u8>> {
        let mut out = Vec::new();
        let mut stack: Vec<u32> = Vec::new();
        let mut at = start;
        for _ in 0..4096 {
            let to = self.addr_to_offset(at)?;
            let tag = self.u32_at_img(to)?;
            if tag & 0x0fff_0000 != 0 {
                // Not a DMA tag (bits 16-27 are zero in every tag on the disc): a CALL run that
                // ends in the render-method list itself [D measured] — the chain stops here.
                return Some(out);
            }
            let addr = self.u32_at_img(to + 4)? & 0x7fff_fff0;
            let qwc = (tag & 0xffff) as u32;
            let id = (tag >> 28) & 7;
            out.extend_from_slice(self.image.get(to + 8..to + 16)?);
            let (data_at, next) = match id {
                0 => (addr, None),                       // REFE
                1 => (at + 16, Some(at + 16 + qwc * 16)), // CNT
                2 => (at + 16, Some(addr)),               // NEXT
                3 | 4 => (addr, Some(at + 16)),           // REF / REFS
                5 => {
                    // CALL
                    if stack.len() >= 4 {
                        return None;
                    }
                    stack.push(at + 16 + qwc * 16);
                    (at + 16, Some(addr))
                }
                6 => (at + 16, stack.pop()), // RET
                _ => (at + 16, None),        // END
            };
            if qwc > 0 {
                let d = self.addr_to_offset(data_at)?;
                out.extend_from_slice(self.image.get(d..d + qwc as usize * 16)?);
            }
            match next {
                Some(n) => at = n,
                None => return Some(out),
            }
        }
        None
    }

    fn addr_to_offset(&self, addr: u32) -> Option<usize> {
        let s = self
            .sections
            .iter()
            .find(|s| s.size > 0 && addr >= s.offset && addr < s.offset.saturating_add(s.size))?;
        let off = s.data_offset as u64 + (addr - s.offset) as u64;
        (off < self.image.len() as u64).then_some(off as usize)
    }

    fn u32_at_img(&self, off: usize) -> Option<u32> {
        let b = self.image.get(off..off + 4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// The entry's texture: the render-method list at `*(entry − 0x18)` carries a
    /// `0x0300000N <tex …>` command; the first texture object's +4 is its 4-character name
    /// [D measured].
    fn packet_texture(&self, eo: usize) -> Option<String> {
        let rm_list_addr = self.u32_at_img(eo.checked_sub(0x18)?)?;
        let rm_off = self.addr_to_offset(rm_list_addr)?;
        for i in 0..20 {
            let op = self.u32_at_img(rm_off + i * 4)?;
            if (op & 0xffff_fff0) == 0x0300_0000 {
                let rm_ptr = self.u32_at_img(rm_off + (i + 1) * 4)? & 0x0fff_ffff;
                let target = self.addr_to_offset(rm_ptr)?;
                let b = self.image.get(target + 4..target + 8)?;
                if b.iter().all(|&c| c.is_ascii_graphic()) {
                    return Some(String::from_utf8_lossy(b).into_owned());
                }
            }
        }
        None
    }

    /// Decodes one VIF1 stream (the bytes `dma_chain` sends) into vertex batches; a batch ends
    /// at MSCAL/MSCALF/MSCNT.
    ///
    /// Sizes are the VIF1 hardware's: UNPACK reads `num` (0 = 256) vectors of `vn + 1` elements
    /// of 32/16/8 bits (V4-5: 2 bytes per vector), padded to a word; STMASK 1 word, STROW/STCOL
    /// 4, MPG `num` × 8 bytes, DIRECT/DIRECTHL `imm` qwords (0 = 65536). Every track UNPACK has
    /// the USN bit set [D measured]. Attribute decoding, all [D measured, not traced to the VU
    /// program]:
    /// - V3-16: position, offset binary 8.8 fixed point `(u − 0x8000) / 256` (the decoded
    ///   extents equal the article's `Base` box to 1/256 on most articles);
    /// - V2-16: UV, offset binary `(u − 0x8000) / 256` — the scale is UNPROVEN (the raw values
    ///   cluster at 0x8000 + {0, 0x80, 0x100}: 1/256 or 1/128 both fit);
    /// - V3-8: colour bytes; V3-32 (the first in a batch) / V2-32: float position / UV;
    /// - V4-32 with `num > 1`: float quads in batch order position, normal, UV (car models).
    ///
    /// The attribute is chosen by format, not by position in the batch: a CALL's own qwords may
    /// unpack colours ahead of the packet it calls [D measured].
    fn parse_vif_stream(&self, s: &[u8]) -> Vec<MeshStrip> {
        let rd = |o: usize| u32::from_le_bytes([s[o], s[o + 1], s[o + 2], s[o + 3]]);
        let f32le = |o: usize| f32::from_bits(rd(o));
        let u16le = |o: usize| u16::from_le_bytes([s[o], s[o + 1]]);
        let fix = |u: u16| (u as f32 - 32768.0) / 256.0;
        let fix_uv = |u: u16| (u as f32 - 32768.0) / 1024.0;

        let mut out = Vec::new();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut colors: Vec<[u8; 3]> = Vec::new();
        let mut v4_quads: usize = 0;
        let flush = |out: &mut Vec<MeshStrip>,
                     positions: &mut Vec<[f32; 3]>,
                     uvs: &mut Vec<[f32; 2]>,
                     colors: &mut Vec<[u8; 3]>| {
            if !positions.is_empty() {
                out.push(MeshStrip {
                    positions: std::mem::take(positions),
                    uvs: std::mem::take(uvs),
                    colors: std::mem::take(colors),
                    texture: None,
                });
            } else {
                uvs.clear();
                colors.clear();
            }
        };

        let mut cur = 0usize;
        while cur + 4 <= s.len() {
            let vif = rd(cur);
            cur += 4;
            let op = ((vif >> 24) & 0x7f) as u8;
            let imm = (vif & 0xffff) as usize;
            let num = ((vif >> 16) & 0xff) as usize;
            if op & 0x60 == 0x60 {
                let n = if num == 0 { 256 } else { num };
                let vn = ((op >> 2) & 3) as usize;
                let vl = (op & 3) as usize;
                let bytes = if vl == 3 { n * 2 } else { n * (vn + 1) * [4, 2, 1][vl] };
                let len = (bytes + 3) & !3;
                if cur + len > s.len() {
                    break;
                }
                match (vn, vl) {
                    (3, 0) if n > 1 => {
                        // Float quads in batch order position, normal, UV (car models).
                        for k in 0..n {
                            let o = cur + k * 16;
                            if v4_quads == 0 {
                                positions.push([f32le(o), f32le(o + 4), f32le(o + 8)]);
                            } else if v4_quads == 2 {
                                uvs.push([f32le(o), f32le(o + 4)]);
                            }
                        }
                        v4_quads += 1;
                    }
                    (2, 0) if n > 1 && positions.is_empty() => {
                        for k in 0..n {
                            let o = cur + k * 12;
                            positions.push([f32le(o), f32le(o + 4), f32le(o + 8)]);
                        }
                    }
                    (2, 1) if n > 1 => {
                        for k in 0..n {
                            let o = cur + k * 6;
                            positions.push([fix(u16le(o)), fix(u16le(o + 2)), fix(u16le(o + 4))]);
                        }
                    }
                    (1, 0) if n > 1 => {
                        for k in 0..n {
                            let o = cur + k * 8;
                            uvs.push([f32le(o), f32le(o + 4)]);
                        }
                    }
                    (1, 1) if n > 1 => {
                        for k in 0..n {
                            let o = cur + k * 4;
                            uvs.push([fix_uv(u16le(o)), fix_uv(u16le(o + 2))]);
                        }
                    }
                    (2, 2) if n > 1 => {
                        for k in 0..n {
                            let o = cur + k * 3;
                            colors.push([s[o], s[o + 1], s[o + 2]]);
                        }
                    }
                    _ => {}
                }
                cur += len;
                continue;
            }
            let skip = match op {
                0x00..=0x07 | 0x10 | 0x11 | 0x13 => 0,
                0x14 | 0x15 | 0x17 => {
                    flush(&mut out, &mut positions, &mut uvs, &mut colors);
                    v4_quads = 0;
                    0
                }
                0x20 => 4,
                0x30 | 0x31 => 16,
                0x4a => (if num == 0 { 256 } else { num }) * 8,
                0x50 | 0x51 => (if imm == 0 { 0x10000 } else { imm }) * 16,
                _ => break,
            };
            cur += skip;
        }
        flush(&mut out, &mut positions, &mut uvs, &mut colors);
        out
    }
}

/// One decoded VIF triangle-strip mesh chunk from a `.crp` `TrackModule` [D measured].
#[derive(Clone, Debug, PartialEq)]
pub struct MeshStrip {
    pub positions: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 3]>,
    pub texture: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The clean hand-built article container (separate layout, no overlaps):
    /// 0x00 root, 0x10 Arti, 0x20 DBN (directory), 0x30..0x70 article window, 0x70 Base payload,
    /// 0xa0 Name payload, 0xb0 as payload, 0xc0 sr payload.
    #[test]
    fn article_metadata_roundtrip() {
        let mut d = vec![0u8; 0x100];
        let put = |d: &mut Vec<u8>, o: usize, tag: u32, flags: u32, count: i32, dref: i32| {
            d[o..o + 4].copy_from_slice(&tag.to_le_bytes());
            d[o + 4..o + 8].copy_from_slice(&flags.to_le_bytes());
            d[o + 8..o + 12].copy_from_slice(&(count as u32).to_le_bytes());
            d[o + 12..o + 16].copy_from_slice(&(dref as u32).to_le_bytes());
        };
        let (ARTI, DBN, NAME) = (
            crate::crp::tag::ARTI,
            crate::crp::tag::DBN,
            crate::crp::tag::NAME,
        );
        // Root: table base 0x10 (relative, dref=1 record), skip 1 record -> pool [0x10..0x20),
        // directory = 1 record [0x20..0x30).
        put(&mut d, 0x00, crate::crp::CARP, 2 | (1 << 5), 1, 1);
        put(&mut d, 0x10, ARTI, 0, 4, 0x30); // window [0x30..0x70)
        put(&mut d, 0x20, DBN, 6 << 8, 0, 0xf0); // "track" payload at 0xf0
        d[0xf0..0xf6].copy_from_slice(b"track\0");
        // Article window: Base, Name, as#0, sr#5.
        put(&mut d, 0x30, TAG_BASE, 0x2c << 8, 0, 0x70);
        put(&mut d, 0x40, NAME, 5 << 8, 0, 0xa0);
        put(&mut d, 0x50, (TAG_AS & 0xffff_0000) | 0, (4 << 8) | 2 | 1, 0, 0x60); // payload 0x50+0x60 = 0xb0
        put(&mut d, 0x60, (TAG_SR & 0xffff_0000) | 5, (0x2a << 8) | 1, 0, 0xc0);
        let f = |v: f32| v.to_le_bytes();
        d[0x80..0x84].copy_from_slice(&f(-10.0));
        d[0x84..0x88].copy_from_slice(&f(-7.5));
        d[0x88..0x8c].copy_from_slice(&f(-10.0));
        d[0x90..0x94].copy_from_slice(&f(18.0));
        d[0x94..0x98].copy_from_slice(&f(0.5));
        d[0x98..0x9c].copy_from_slice(&f(15.0));
        d[0xa0..0xa5].copy_from_slice(b"road\0");
        // as#0 payload: 4 bytes {u16 5, "sr"} = 05 00 72 73.
        d[0xb0..0xb4].copy_from_slice(&[0x05, 0x00, 0x72, 0x73]);
        // sr#5 payload: "EAGL::road::{as  0000}::Model[0].00" (35 bytes + NUL = 0x24).
        let sr = b"EAGL::road::{as  0000}::Model[0].00\0";
        d[0xc0..0xc0 + sr.len()].copy_from_slice(sr);
        let c = crate::crp::Crp::parse(&d).unwrap();
        assert_eq!(c.track_name().unwrap(), Some("track"));
        let arts = c.track_articles().unwrap();
        assert_eq!(arts.len(), 1);
        let a = &arts[0];
        assert_eq!(a.name, "road");
        assert_eq!(a.bbox_min, [-10.0, -7.5, -10.0]);
        assert_eq!(a.bbox_max, [18.0, 0.5, 15.0]);
        assert_eq!(a.parts, vec![PartRef { as_instance: 0, sr_index: Some(5) }]);
        // The sr lookup via the raw record (window walk from the pool).
        let arti = &c.pool().unwrap()[0];
        assert_eq!(
            c.article_sr_string(arti, 5).unwrap().as_deref(),
            Some("EAGL::road::{as  0000}::Model[0].00")
        );
        // A non-"sr" tail yields sr_index: None (the measured trafficlight form).
        d[0xb0..0xb4].copy_from_slice(&[0x00, 0x02, 0x05, 0x00]);
        let c2 = crate::crp::Crp::parse(&d).unwrap();
        assert_eq!(c2.track_articles().unwrap()[0].parts,
            vec![PartRef { as_instance: 0, sr_index: None }]);
        // Malformed: a Base payload too short must error, not panic.
        d[0x34..0x38].copy_from_slice(&(0x04u32 << 8).to_le_bytes()); // Base size 4
        let c3 = crate::crp::Crp::parse(&d).unwrap();
        assert!(c3.track_articles().is_err());
    }

    fn w32(b: &mut [u8], o: usize, v: u32) {
        b[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn wf32(b: &mut [u8], o: usize, v: f32) {
        w32(b, o, v.to_bits());
    }
    fn w16(b: &mut [u8], o: usize, v: u16) {
        b[o..o + 2].copy_from_slice(&v.to_le_bytes());
    }

    fn one_section_module(img: Vec<u8>, symbols: Vec<ModuleSymbol>) -> TrackModule {
        let size = img.len() as u32;
        TrackModule {
            image: img,
            elfd_size: size,
            sections: vec![ModuleSection {
                name: ".data".into(),
                sh_type: 1,
                offset: 0,
                size,
                data_offset: 0,
                link: 0,
                entsize: 0,
            }],
            symbols,
            relocations: 0,
        }
    }

    /// A VIF packet in the measured track shape at `t` (RET tag, then V3-16 / V2-16 / V3-8 /
    /// MSCNT) holding two vertices with the raw u16 lanes `xs`.
    fn put_packet(img: &mut [u8], t: usize, xs: [u16; 2]) {
        w32(img, t, 0x6000_0003); // RET, QWC 3
        w32(img, t + 0x10, 0x6902_c000); // UNPACK V3-16 usn, NUM 2
        for (k, x) in xs.iter().enumerate() {
            w16(img, t + 0x14 + k * 6, *x);
            w16(img, t + 0x16 + k * 6, 0x8000 + 0x100); // y = 1.0
            w16(img, t + 0x18 + k * 6, 0x8000 - 0x200); // z = -2.0
        }
        w32(img, t + 0x20, 0x6502_c010); // UNPACK V2-16 usn, NUM 2
        w16(img, t + 0x24, 0x8000 + 0x400); // u = 1.0
        w16(img, t + 0x26, 0x8000 + 0x100); // v = 0.25
        w16(img, t + 0x28, 0x8000);
        w16(img, t + 0x2a, 0x8000 + 0x200);
        w32(img, t + 0x2c, 0x7a02_c020); // UNPACK V3-8 usn, NUM 2
        img[t + 0x30..t + 0x36].copy_from_slice(&[10, 20, 30, 40, 50, 60]);
        w32(img, t + 0x38, 0x1700_0000); // MSCNT
    }

    /// The measured entry indirection: the table entry `E` is preceded by a word pointing at the
    /// render-method list, whose `0x04000002 <addr>` command names a CALL into the packet that
    /// sits BEFORE `E`; the RET packet at `E + 8` belongs to another entry and must not be read.
    #[test]
    fn entry_geometry_comes_through_its_render_method_call() {
        let mut img = vec![0u8; 0x400];
        // Descriptor at 0x000: +0xc8 = 1 group, +0xcc = table at 0x100.
        w32(&mut img, 0xc8, 1);
        w32(&mut img, 0xcc, 0x100);
        // Table: [A0000000][0][2] then [A000FFFF][0x248].
        w32(&mut img, 0x100, 0xA000_0000);
        w32(&mut img, 0x108, 2);
        w32(&mut img, 0x10c, 0xA000_FFFF);
        w32(&mut img, 0x110, 0x248);
        // The entry's own packet (the one that sits before it) at tag 0x1c0: x = +2.0 / +3.0.
        put_packet(&mut img, 0x1c0, [0x8200, 0x8300]);
        // Entry 0x248: head word at 0x230 → render-method list at 0x300.
        w32(&mut img, 0x230, 0x300);
        w32(&mut img, 0x248, 0x230);
        // A decoy packet at E + 8 (the next entry's geometry): x = -5.0 / -6.0.
        put_packet(&mut img, 0x250, [0x8000 - 0x500, 0x8000 - 0x600]);
        // Render-method list: texture command, then 04000002 → CALL at 0x2c0.
        w32(&mut img, 0x300, 0x0300_0002);
        w32(&mut img, 0x304, 0x4000_0340);
        w32(&mut img, 0x308, 0x0f00_0000);
        w32(&mut img, 0x30c, 0x0400_0002);
        w32(&mut img, 0x310, 0x2c0);
        img[0x344..0x348].copy_from_slice(b"road");
        // The CALL at 0x2c0 (no own qwords) into the RET packet at 0x1c0.
        w32(&mut img, 0x2c0, 0x5000_0000);
        w32(&mut img, 0x2c4, 0x1c0);

        let m = one_section_module(
            img,
            vec![ModuleSymbol {
                name: "__Model:::TestRoad::{as  0000}::Model[0].00".into(),
                value: 0,
                size: 4,
                info: 0,
                shndx: 1,
            }],
        );
        let strips = m.extract_article_mesh("TestRoad", [0.0; 3], [0.0; 3]);
        assert_eq!(strips.len(), 1);
        let s = &strips[0];
        // 8.8 offset-binary positions; the decoy's negative x must not appear.
        assert_eq!(s.positions, vec![[2.0, 1.0, -2.0], [3.0, 1.0, -2.0]]);
        // UVs (u − 0x8000) / 1024.
        assert_eq!(s.uvs, vec![[1.0, 0.25], [0.0, 0.5]]);
        assert_eq!(s.colors, vec![[10, 20, 30], [40, 50, 60]]);
        assert_eq!(s.texture.as_deref(), Some("road"));
    }

    /// A CALL run (one CALL per sub-packet) that ends in render-method words, and a CALL that
    /// carries its own qwords ahead of the packet it calls.
    #[test]
    fn dma_chain_follows_call_runs_and_call_data() {
        let mut img = vec![0u8; 0x300];
        put_packet(&mut img, 0x40, [0x8100, 0x8100]);
        put_packet(&mut img, 0x90, [0x8400, 0x8400]);
        // CALL run at 0x100: CALL 0x40, CALL 0x90, then a render-method word (not a tag).
        w32(&mut img, 0x100, 0x5000_0000);
        w32(&mut img, 0x104, 0x40);
        w32(&mut img, 0x110, 0x5000_0000);
        w32(&mut img, 0x114, 0x90);
        w32(&mut img, 0x120, 0x0300_0002);
        let m = one_section_module(img.clone(), Vec::new());
        let stream = m.dma_chain(0x100).expect("chain");
        let strips = m.parse_vif_stream(&stream);
        assert_eq!(strips.len(), 2);
        assert_eq!(strips[0].positions[0][0], 1.0);
        assert_eq!(strips[1].positions[0][0], 4.0);

        // CALL with QWC 1 of its own: a V3-8 colour unpack that precedes the called packet.
        let mut img2 = img;
        w32(&mut img2, 0x180, 0x5000_0001);
        w32(&mut img2, 0x184, 0x40);
        w32(&mut img2, 0x190, 0x7a02_c0ab); // UNPACK V3-8 NUM 2
        img2[0x194..0x19a].copy_from_slice(&[1, 2, 3, 4, 5, 6]);
        w32(&mut img2, 0x1a0, 0x0300_0002);
        let m2 = one_section_module(img2, Vec::new());
        let strips = m2.parse_vif_stream(&m2.dma_chain(0x180).unwrap());
        assert_eq!(strips.len(), 1);
        // The positions still decode although a colour unpack came first.
        assert_eq!(strips[0].positions.len(), 2);
        assert_eq!(strips[0].colors[0], [1, 2, 3]);
    }

    #[test]
    fn extracts_cdat_collision_tris() {
        // CARP: Arti "road" (ca#0: 1 group, 1 strip of 4 vertices → 2 triangles) and CDat (sr#1 =
        // "CARP::road", ci#0 = 2 items: a 90° yaw at (100, 5, 200) and the identity with the
        // flags bits that make row 1 = r2 × r0).
        let mut buf = vec![0u8; 0x300];
        let wi32 = |b: &mut [u8], o: usize, v: i32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        w32(&mut buf, 0x00, crate::crp::CARP);
        w32(&mut buf, 0x04, 2 << 5);
        wi32(&mut buf, 0x08, 0);
        wi32(&mut buf, 0x0c, 0x10);
        w32(&mut buf, 0x10, crate::crp::tag::ARTI);
        wi32(&mut buf, 0x18, 2);
        wi32(&mut buf, 0x1c, 0x40);
        w32(&mut buf, 0x20, crate::crp::tag::CDAT);
        wi32(&mut buf, 0x28, 2);
        wi32(&mut buf, 0x2c, 0x60);
        // Name "road".
        w32(&mut buf, 0x40, crate::crp::tag::NAME);
        w32(&mut buf, 0x44, 5 << 8);
        wi32(&mut buf, 0x4c, 0x100);
        buf[0x100..0x105].copy_from_slice(b"road\0");
        // ca#0 at 0x120 (32 header + 16 group + 64 strip).
        w32(&mut buf, 0x50, TAG_CA & 0xffff_0000);
        w32(&mut buf, 0x54, (112 << 8) | 0x01);
        wi32(&mut buf, 0x5c, 0x120);
        w16(&mut buf, 0x120, 1); // 1 group
        w16(&mut buf, 0x140 + 14, 16); // strip at +0x20 + 16 = 0x150
        let pts = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0]];
        // v0 word = count, v1 word = strip flags (start winding 0), v2/v3 = surface | class<<8 |
        // radius<<16.
        let ws = [4u32, 0, 0x0010_0405, 0x0010_0003];
        for k in 0..4 {
            let vo = 0x150 + k * 16;
            for i in 0..3 {
                wf32(&mut buf, vo + i * 4, pts[k][i]);
            }
            w32(&mut buf, vo + 12, ws[k]);
        }
        // sr#1.
        w32(&mut buf, 0x60, (TAG_SR & 0xFFFF_0000) | 1);
        w32(&mut buf, 0x64, (11 << 8) | 0x01);
        wi32(&mut buf, 0x6c, 0x1a0);
        buf[0x1a0..0x1ab].copy_from_slice(b"CARP::road\0");
        // ci#0: 2 items at 0x1c0.
        w32(&mut buf, 0x70, TAG_CI & 0xFFFF_0000);
        w32(&mut buf, 0x74, (128 << 8) | 0x01);
        wi32(&mut buf, 0x78, 2);
        wi32(&mut buf, 0x7c, 0x1c0);
        // Item 0, world→local: local.x = −world.z, local.z = world.x (r0 = (0,0,1), r2 = (−1,0,0)
        // as row-vector rows), translation (100, 5, 200).
        let it = 0x1c0;
        wf32(&mut buf, it + 0x08, 1.0); // r0 = (0, 0, 1)
        wf32(&mut buf, it + 0x20, -1.0); // r2 = (−1, 0, 0)
        w32(&mut buf, it + 0x1c, 0x7372_0001);
        wf32(&mut buf, it + 0x30, 100.0);
        wf32(&mut buf, it + 0x34, 5.0);
        wf32(&mut buf, it + 0x38, 200.0);
        // Item 1: flags & 3 set, so row 1 = r2 × r0; with r0 = (1,0,0), r2 = (0,−1,0) that is
        // (0,0,1), not the default (0,1,0).
        let it = 0x200;
        wf32(&mut buf, it + 0x00, 1.0);
        wf32(&mut buf, it + 0x24, -1.0);
        w32(&mut buf, it + 0x18, 1);
        w32(&mut buf, it + 0x1c, 0x7372_0001);

        let crp = crate::crp::Crp::parse(&buf).unwrap();
        let tris = crp.extract_collision_tris().unwrap();
        assert_eq!(tris.len(), 4);
        // Flags = the third vertex's low half; class 0x04 = skipped by the ground query.
        assert_eq!(tris[0].flags, 0x0405);
        assert_eq!((tris[0].surface(), tris[0].class()), (5, 4));
        assert!(tris[0].ground_query_skips());
        assert_eq!(tris[1].flags, 0x0003);
        assert!(!tris[1].ground_query_skips());
        // world = (local − t)·M⁻¹, M rows r0, (0,1,0), r2 = [[0,0,1],[0,1,0],[−1,0,0]],
        // M⁻¹ = Mᵀ = [[0,0,−1],[0,1,0],[1,0,0]]; local (0,0,0): d = (−100, −5, −200) →
        // world (−200, −5, 100). (The forward reading tr + local would give (100, 5, 200).)
        assert_eq!(tris[0].v[0], [-200.0, -5.0, 100.0]);
        // local (1,0,0) → d = (−99, −5, −200) → (−200, −5, 99).
        assert_eq!(tris[0].v[1], [-200.0, -5.0, 99.0]);
        // Triangle 1 has odd parity: (v1, v3, v2) → v[1] is local (1,0,1), v[2] local (0,0,1).
        // local (1,0,1) → d = (−99, −5, −199) → (−199, −5, 99).
        assert_eq!(tris[1].v[1], [-199.0, -5.0, 99.0]);
        // Item 1: M = [[1,0,0],[0,0,1],[0,−1,0]] (row 1 = r2 × r0); local (0,0,1) →
        // world = (0,0,1)·M⁻¹, M⁻¹ = Mᵀ = [[1,0,0],[0,0,−1],[0,1,0]] → (0, 1, 0).
        assert_eq!(tris[2].v[2], [0.0, 1.0, 0.0]);

        // An item whose sr names no article with a ca record is an error.
        w32(&mut buf, 0x1c0 + 0x1c, 0x7372_0002);
        assert!(crate::crp::Crp::parse(&buf).unwrap().extract_collision_tris().is_err());
    }

    #[test]
    fn extracts_map_triggers() {
        let mut buf = vec![0u8; 0x200];
        let wi32 = |b: &mut [u8], o: usize, v: i32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        w32(&mut buf, 0x00, crate::crp::CARP);
        w32(&mut buf, 0x04, 1 << 5);
        wi32(&mut buf, 0x08, 0);
        wi32(&mut buf, 0x0c, 0x10);
        w32(&mut buf, 0x10, crate::crp::tag::MAP);
        wi32(&mut buf, 0x18, 1);
        wi32(&mut buf, 0x1c, 0x20);
        w32(&mut buf, 0x20, 0x5472_6772);
        w32(&mut buf, 0x24, 64 << 8);
        wi32(&mut buf, 0x28, 1);
        wi32(&mut buf, 0x2c, 0x40);
        wf32(&mut buf, 0x40, -10.5);
        wf32(&mut buf, 0x44, 2.0);
        wf32(&mut buf, 0x48, 30.25);
        wf32(&mut buf, 0x4c, 4.5);
        w32(&mut buf, 0x50, 0x0407_0101);
        w32(&mut buf, 0x58, 0x656c_0001);
        let crp = crate::crp::Crp::parse(&buf).unwrap();
        let trgs = crp.extract_map_triggers().unwrap();
        assert_eq!(trgs.len(), 1);
        assert_eq!(trgs[0].pos, [-10.5, 2.0, 30.25]);
        assert!((trgs[0].radius - 4.5).abs() < 1e-6);
        assert_eq!(trgs[0].flags, 0x0407_0101);
        assert_eq!(trgs[0].id, 0x656c_0001);
    }

    /// V4-32 runs (car models): the first run is the positions, the third the UVs; the second
    /// (normals) never reaches the positions. UNPACK sizes follow the hardware (V4-5: 2 bytes).
    #[test]
    fn v4_32_runs_and_unpack_sizes() {
        let mut s = vec![0u8; 0x100];
        w32(&mut s, 0x00, 0x6c02_c022); // V4-32 NUM 2: positions
        for (k, v) in [0.1f32, 0.2, 0.3, 1.0, 0.4, 0.5, 0.6, 1.0].iter().enumerate() {
            wf32(&mut s, 0x04 + k * 4, *v);
        }
        w32(&mut s, 0x24, 0x6c02_c04e); // V4-32 NUM 2: normals
        wf32(&mut s, 0x28, 1.0);
        wf32(&mut s, 0x3c, 1.0);
        w32(&mut s, 0x48, 0x6c02_c07a); // V4-32 NUM 2: UVs
        wf32(&mut s, 0x4c, 0.25);
        wf32(&mut s, 0x50, 0.75);
        wf32(&mut s, 0x5c, 0.5);
        wf32(&mut s, 0x60, 0.5);
        w32(&mut s, 0x6c, 0x6f03_c000); // V4-5 NUM 3: 6 bytes → padded to 8
        w32(&mut s, 0x78, 0x1700_0000); // MSCNT
        let m = one_section_module(vec![0u8; 16], Vec::new());
        let strips = m.parse_vif_stream(&s);
        assert_eq!(strips.len(), 1);
        assert_eq!(strips[0].positions, vec![[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]]);
        assert_eq!(strips[0].uvs, vec![[0.25, 0.75], [0.5, 0.5]]);
    }
}
