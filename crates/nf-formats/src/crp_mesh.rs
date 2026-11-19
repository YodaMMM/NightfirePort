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
//!    collision: 256-7040 B float quads + packed words [D measured; layout next phase]), `in#0/#1`
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
//!    the next phase's work — this module pins everything that walk needs.
//! 4. **Collision** — the pool `CDat` article is the collision world: window = `CGrd` (36 B grid
//!    header) + `ci#0` (63,616 B, count=994 items on paris) + ~1600 `cn#N` node records (38-88 B)
//!    [D measured]. `RNgp` is the road network, `Map ` the mission scene database (AICo/AIEl/AISp/
//!    Trgr/…/aq#N placement arrays) [D]. Their binary layouts are the next phase's work.

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

/// One world-space triangle extracted from `.crp` `CDat` (`ci#0` + `Arti` `ca#0`) [M 0x216EC0].
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionTri {
    pub v: [[f32; 3]; 3],
    pub flags: u16,
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

    /// Extract all world-space collision triangles from `CDat` (`ci#0` instances + `sr` refs +
    /// each `Arti`'s `ca#0` triangle strips) [M 0x216EC0 / D measured].
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
            for item in ci_data.chunks_exact(64) {
                let sr_tag = u32::from_le_bytes(item[28..32].try_into().unwrap());
                let sr_idx = (sr_tag & 0xffff) as u16;
                let Some(aname) = cdat_sr.get(&sr_idx) else {
                    continue;
                };
                let Some(&ca) = arti_ca.get(aname) else {
                    continue;
                };
                if ca.len() < 32 {
                    continue;
                }
                let r0 = [
                    f32::from_le_bytes(item[0..4].try_into().unwrap()),
                    f32::from_le_bytes(item[4..8].try_into().unwrap()),
                    f32::from_le_bytes(item[8..12].try_into().unwrap()),
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
                let xform = |lx: f32, ly: f32, lz: f32| -> [f32; 3] {
                    [
                        tr[0] + lx * r0[0] + lz * r2[0],
                        tr[1] + lx * r0[1] + ly + lz * r2[1],
                        tr[2] + lx * r0[2] + lz * r2[2],
                    ]
                };

                let gcnt = u16::from_le_bytes([ca[0], ca[1]]) as usize;
                let geom_sz = u16::from_le_bytes([ca[2], ca[3]]) as usize;
                let geom_end = ca.len().min(32 + geom_sz);
                if 32 + gcnt * 16 > ca.len() {
                    continue;
                }
                for g in 0..gcnt {
                    let gh = &ca[32 + g * 16..32 + (g + 1) * 16];
                    let voff = u16::from_le_bytes([gh[14], gh[15]]) as usize;
                    let next_voff = if g + 1 < gcnt {
                        let ngh = &ca[32 + (g + 1) * 16..32 + (g + 2) * 16];
                        32 + u16::from_le_bytes([ngh[14], ngh[15]]) as usize
                    } else {
                        geom_end
                    };
                    let end = geom_end.min(next_voff);
                    let mut cur = 32 + voff;
                    while cur + 16 <= end {
                        let vc = u32::from_le_bytes(ca[cur + 12..cur + 16].try_into().unwrap())
                            as usize;
                        if vc < 3 || cur + vc * 16 > end {
                            break;
                        }
                        let mut verts = Vec::with_capacity(vc);
                        let mut words = Vec::with_capacity(vc);
                        for k in 0..vc {
                            let vo = cur + k * 16;
                            let lx = f32::from_le_bytes(ca[vo..vo + 4].try_into().unwrap());
                            let ly = f32::from_le_bytes(ca[vo + 4..vo + 8].try_into().unwrap());
                            let lz = f32::from_le_bytes(ca[vo + 8..vo + 12].try_into().unwrap());
                            let w = u32::from_le_bytes(ca[vo + 12..vo + 16].try_into().unwrap());
                            verts.push(xform(lx, ly, lz));
                            words.push(w);
                        }
                        for k in 2..vc {
                            let w = words[k];
                            if (w & 1) != 0 {
                                let flags = (w >> 16) as u16;
                                tris.push(CollisionTri {
                                    v: [verts[k - 2], verts[k - 1], verts[k]],
                                    flags,
                                });
                            }
                        }
                        cur += vc * 16;
                    }
                }
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

    /// Decode all VIF DMA packets for `article` into dequantized triangle-strip vertices
    /// (`V3-16` positions mapped into `bbox_min..bbox_max`, `V2-16` UVs scaled by 1/4096,
    /// `V3-8` RGB/normal bytes).
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

    /// Decode VIF DMA packets across ALL `{as  XXXX}` parts of `article` (used for car `.crp`
    /// models such as `vanquish` where chassis, wheels, and attachments have separate `as` parts).
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

    fn extract_segments(
        &self,
        segs: &[(u8, u32)],
        bbox_min: [f32; 3],
        bbox_max: [f32; 3],
    ) -> Vec<MeshStrip> {
        let mut strips = Vec::new();
        let mut seen_pkts = std::collections::HashSet::new();
        for &(_seg, addr) in segs {
            let Some(off) = self.addr_to_offset(addr) else {
                continue;
            };
            // Model header +0xc8 = group count, +0xcc = pointer to the `[0xA0000000, 0, count]` groups.
            let group_count = self.u32_at_img(off + 0xc8).unwrap_or(1).min(1024) as usize;
            let Some(tbl_addr) = self.u32_at_img(off + 0xcc) else {
                continue;
            };
            let Some(mut cur) = self.addr_to_offset(tbl_addr) else {
                continue;
            };
            let mut current_tex: Option<String> = None;
            let limit = self.image.len().saturating_sub(8);
            for _ in 0..group_count {
                if cur + 12 > limit {
                    break;
                }
                let tag_w = self.u32_at_img(cur).unwrap_or(0);
                if tag_w == 0xA000_0000 {
                    let count = self.u32_at_img(cur + 8).unwrap_or(0) as usize;
                    let n_pairs = count / 2;
                    cur += 12;
                    for _ in 0..n_pairs {
                        if cur + 8 > limit {
                            break;
                        }
                        if self.u32_at_img(cur) != Some(0xA000_FFFF) {
                            break;
                        }
                        let pkt_addr = self.u32_at_img(cur + 4).unwrap_or(0);
                        if seen_pkts.insert(pkt_addr) {
                            if let Some(po) = self.addr_to_offset(pkt_addr) {
                                if let Some(tex) = self.packet_texture(po) {
                                    current_tex = Some(tex);
                                }
                                for mut strip in self.parse_vif_packet(po, bbox_min, bbox_max) {
                                    strip.texture = current_tex.clone();
                                    strips.push(strip);
                                }
                            }
                        }
                        cur += 8;
                    }
                } else if tag_w == 0xA000_FFFF {
                    let pkt_addr = self.u32_at_img(cur + 4).unwrap_or(0);
                    if seen_pkts.insert(pkt_addr) {
                        if let Some(po) = self.addr_to_offset(pkt_addr) {
                            if let Some(tex) = self.packet_texture(po) {
                                current_tex = Some(tex);
                            }
                            for mut strip in self.parse_vif_packet(po, bbox_min, bbox_max) {
                                strip.texture = current_tex.clone();
                                strips.push(strip);
                            }
                        }
                    }
                    cur += 8;
                } else {
                    break;
                }
            }
        }
        strips
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

    fn packet_texture(&self, po: usize) -> Option<String> {
        let rm_list_addr = self.u32_at_img(po.checked_sub(0x18)?)?;
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

    fn parse_vif_packet(
        &self,
        po: usize,
        bbox_min: [f32; 3],
        bbox_max: [f32; 3],
    ) -> Vec<MeshStrip> {
        let Some(dma_tag) = self.u32_at_img(po + 8) else {
            return Vec::new();
        };
        let qwc = (dma_tag & 0xffff) as usize;
        let mut cur = po + 0x10;
        let end = (cur + qwc * 16).min(self.image.len());

        let mut out = Vec::new();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut colors: Vec<[u8; 3]> = Vec::new();
        let mut batch_attr_idx: usize = 0;

        let flush_batch = |out: &mut Vec<MeshStrip>,
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

        while cur + 4 <= end {
            let Some(vif) = self.u32_at_img(cur) else {
                break;
            };
            cur += 4;
            let op = ((vif >> 24) & 0x7f) as u8;
            if op < 0x20 {
                if op == 0x14 || op == 0x15 || op == 0x17 {
                    flush_batch(&mut out, &mut positions, &mut uvs, &mut colors);
                    batch_attr_idx = 0;
                }
                continue;
            }
            if op == 0x20 {
                cur += 4;
                continue;
            }
            if op == 0x30 || op == 0x31 {
                cur += 16;
                continue;
            }
            if op == 0x50 || op == 0x51 {
                let imm = (vif & 0xffff) as usize;
                let q = if imm == 0 { 0x10000 } else { imm };
                cur += q * 16;
                continue;
            }
            if (op & 0x60) == 0x60 {
                let num = ((vif >> 16) & 0xff) as usize;
                let vn = ((op >> 2) & 3) as usize;
                let vl = (op & 3) as usize;
                let elem_sz = [4usize, 2, 1, 2][vl];
                let byte_len = (num * (vn + 1) * elem_sz + 3) & !3;
                if cur + byte_len > end {
                    break;
                }
                if vn == 3 && vl == 0 && num > 1 {
                    // Uncompressed V4-32 attribute batch:
                    // attr 0 = positions, attr 1 = normals, attr 2 = UVs, attr 3/4 = colors/weights.
                    if batch_attr_idx == 0 {
                        for k in 0..num {
                            let o = cur + k * 16;
                            let x = f32::from_le_bytes([self.image[o], self.image[o + 1], self.image[o + 2], self.image[o + 3]]);
                            let y = f32::from_le_bytes([self.image[o + 4], self.image[o + 5], self.image[o + 6], self.image[o + 7]]);
                            let z = f32::from_le_bytes([self.image[o + 8], self.image[o + 9], self.image[o + 10], self.image[o + 11]]);
                            positions.push([x, y, z]);
                        }
                    } else if batch_attr_idx == 2 {
                        for k in 0..num {
                            let o = cur + k * 16;
                            let u = f32::from_le_bytes([self.image[o], self.image[o + 1], self.image[o + 2], self.image[o + 3]]);
                            let v = f32::from_le_bytes([self.image[o + 4], self.image[o + 5], self.image[o + 6], self.image[o + 7]]);
                            uvs.push([u, v]);
                        }
                    }
                    batch_attr_idx += 1;
                } else if vn == 2 && vl == 0 && num > 1 {
                    // V3-32 float positions
                    if batch_attr_idx == 0 {
                        for k in 0..num {
                            let o = cur + k * 12;
                            let x = f32::from_le_bytes([self.image[o], self.image[o + 1], self.image[o + 2], self.image[o + 3]]);
                            let y = f32::from_le_bytes([self.image[o + 4], self.image[o + 5], self.image[o + 6], self.image[o + 7]]);
                            let z = f32::from_le_bytes([self.image[o + 8], self.image[o + 9], self.image[o + 10], self.image[o + 11]]);
                            positions.push([x, y, z]);
                        }
                    }
                    batch_attr_idx += 1;
                } else if vn == 2 && vl == 1 && num > 1 {
                    // V3-16 positions quantized to [-32768..32767] across bbox_min..bbox_max
                    if batch_attr_idx == 0 {
                        for k in 0..num {
                            let o = cur + k * 6;
                            let ix = i16::from_le_bytes([self.image[o], self.image[o + 1]]) as f32;
                            let iy = i16::from_le_bytes([self.image[o + 2], self.image[o + 3]]) as f32;
                            let iz = i16::from_le_bytes([self.image[o + 4], self.image[o + 5]]) as f32;
                            let deq = |v: f32, axis: usize| {
                                bbox_min[axis]
                                    + ((v + 32768.0) / 65535.0) * (bbox_max[axis] - bbox_min[axis])
                            };
                            positions.push([deq(ix, 0), deq(iy, 1), deq(iz, 2)]);
                        }
                    }
                    batch_attr_idx += 1;
                } else if vn == 1 && vl == 1 && num > 1 {
                    // V2-16 UVs
                    for k in 0..num {
                        let o = cur + k * 4;
                        let iu = i16::from_le_bytes([self.image[o], self.image[o + 1]]) as f32;
                        let iv = i16::from_le_bytes([self.image[o + 2], self.image[o + 3]]) as f32;
                        uvs.push([iu / 4096.0, iv / 4096.0]);
                    }
                    batch_attr_idx += 1;
                } else if vn == 2 && vl == 2 && num > 1 {
                    // V3-8 colours / normals
                    for k in 0..num {
                        let o = cur + k * 3;
                        colors.push([self.image[o], self.image[o + 1], self.image[o + 2]]);
                    }
                    batch_attr_idx += 1;
                } else if num > 1 {
                    batch_attr_idx += 1;
                }
                cur += byte_len;
            } else {
                break;
            }
        }

        flush_batch(&mut out, &mut positions, &mut uvs, &mut colors);
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

    #[test]
    fn extracts_vif_strip_from_track_module() {
        // Build a minimal relocated TrackModule image:
        // 0x000..0x0d0: Model[0].00 header with +0xcc = 0x100 (table addr)
        // 0x100..0x108: {0xA000FFFF, 0x120}
        // 0x120..0x154: VIF DMA packet: +0x08 DMA RET QWC=3, +0x10 V3-16 (2 verts), +0x20 V2-16 (2 UVs), +0x2c V3-8 (2 colors), +0x34 MSCNT
        let mut img = vec![0u8; 0x180];
        img[0xc8..0xcc].copy_from_slice(&1u32.to_le_bytes());
        img[0xcc..0xd0].copy_from_slice(&0x100u32.to_le_bytes());
        img[0x100..0x104].copy_from_slice(&0xA000_FFFFu32.to_le_bytes());
        img[0x104..0x108].copy_from_slice(&0x120u32.to_le_bytes());
        // DMA RET QWC=3 (48 bytes from 0x130..0x160)
        img[0x128..0x12c].copy_from_slice(&0x6000_0003u32.to_le_bytes());
        // VIF UNPACK V3-16 NUM=2 (0x6902_c000) -> 12 bytes
        img[0x130..0x134].copy_from_slice(&0x6902_c000u32.to_le_bytes());
        img[0x134..0x136].copy_from_slice(&(-32768i16).to_le_bytes());
        img[0x136..0x138].copy_from_slice(&(-32768i16).to_le_bytes());
        img[0x138..0x13a].copy_from_slice(&(-32768i16).to_le_bytes());
        img[0x13a..0x13c].copy_from_slice(&32767i16.to_le_bytes());
        img[0x13c..0x13e].copy_from_slice(&32767i16.to_le_bytes());
        img[0x13e..0x140].copy_from_slice(&32767i16.to_le_bytes());
        // VIF UNPACK V2-16 NUM=2 (0x6502_c010) -> 8 bytes
        img[0x140..0x144].copy_from_slice(&0x6502_c010u32.to_le_bytes());
        img[0x144..0x146].copy_from_slice(&4096i16.to_le_bytes());
        img[0x146..0x148].copy_from_slice(&(-4096i16).to_le_bytes());
        // VIF UNPACK V3-8 NUM=2 (0x7a02_c020) -> 6 bytes padded to 8
        img[0x14c..0x150].copy_from_slice(&0x7a02_c020u32.to_le_bytes());
        img[0x150..0x156].copy_from_slice(&[10, 20, 30, 40, 50, 60]);
        // MSCNT
        img[0x158..0x15c].copy_from_slice(&0x1700_0000u32.to_le_bytes());

        let m = TrackModule {
            image: img,
            elfd_size: 0x180,
            sections: vec![ModuleSection {
                name: ".data".into(),
                sh_type: 1,
                offset: 0,
                size: 0x180,
                data_offset: 0,
                link: 0,
                entsize: 0,
            }],
            symbols: vec![ModuleSymbol {
                name: "__Model:::TestRoad::{as  0000}::Model[0].00".into(),
                value: 0,
                size: 4,
                info: 0,
                shndx: 1,
            }],
            relocations: 0,
        };
        let strips = m.extract_article_mesh("TestRoad", [-10.0, -5.0, 0.0], [10.0, 5.0, 20.0]);
        assert_eq!(strips.len(), 1);
        assert_eq!(strips[0].positions.len(), 2);
        assert!((strips[0].positions[0][0] - (-10.0)).abs() < 1e-3);
        assert!((strips[0].positions[1][0] - 10.0).abs() < 1e-3);
        assert!((strips[0].uvs[0][0] - 1.0).abs() < 1e-4);
        assert_eq!(strips[0].colors, vec![[10, 20, 30], [40, 50, 60]]);
    }

    #[test]
    fn extracts_cdat_collision_tris() {
        // Minimal CARP with 1 Arti ("road", ca#0 with 1 group, 1 strip of 4 verts -> 2 tris)
        // and 1 CDat (sr#1 = "CARP::road", ci#0 = 1 identity instance at (100, 5, 200)).
        let mut buf = vec![0u8; 0x300];
        let w32 = |b: &mut [u8], o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let wi32 = |b: &mut [u8], o: usize, v: i32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let wf32 = |b: &mut [u8], o: usize, v: f32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let w16 = |b: &mut [u8], o: usize, v: u16| b[o..o + 2].copy_from_slice(&v.to_le_bytes());

        // Pool records at 0x10 (Arti), 0x20 (CDat); root CARP window at 0x30 (len 0 -> 1 dummy)
        // 0x00: CARP root: pool at 0x10 (2 records -> 0x30), dir count=0
        w32(&mut buf, 0x00, crate::crp::CARP);
        w32(&mut buf, 0x04, 2 << 5);
        wi32(&mut buf, 0x08, 0);
        wi32(&mut buf, 0x0c, 0x10);

        // 0x10: Arti -> window at 0x40, count=2 (Name, ca#0)
        w32(&mut buf, 0x10, crate::crp::tag::ARTI);
        w32(&mut buf, 0x14, 0);
        wi32(&mut buf, 0x18, 2);
        wi32(&mut buf, 0x1c, 0x40);

        // 0x20: CDat -> window at 0x60, count=2 (sr#1, ci#0)
        w32(&mut buf, 0x20, crate::crp::tag::CDAT);
        w32(&mut buf, 0x24, 0);
        wi32(&mut buf, 0x28, 2);
        wi32(&mut buf, 0x2c, 0x60);

        // 0x40: Name ("road\0", 5 bytes at 0x100)
        w32(&mut buf, 0x40, crate::crp::tag::NAME);
        w32(&mut buf, 0x44, 5 << 8);
        wi32(&mut buf, 0x48, 0);
        wi32(&mut buf, 0x4c, 0x100);
        buf[0x100..0x105].copy_from_slice(b"road\0");

        // 0x50: ca#0 (96 bytes at 0x120: 32 hdr + 16 grp + 64 4-vert strip)
        w32(&mut buf, 0x50, TAG_CA | 0x0000); // ca#0, short tag bit 0x01
        w32(&mut buf, 0x54, (112 << 8) | 0x01);
        wi32(&mut buf, 0x58, 0);
        wi32(&mut buf, 0x5c, 0x120);
        w16(&mut buf, 0x120, 1); // gcnt = 1
        w16(&mut buf, 0x122, 80); // geom_sz = 16 (grp hdr) + 64 (verts)
        w16(&mut buf, 0x140 + 14, 16); // grp 0 voff = 16 from ca+32 -> 0x150
        // 4 strip verts at 0x150
        let pts = [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 1.0]];
        let ws = [4u32, 0, 0x0007_0001, 0x0007_0001];
        for k in 0..4 {
            let vo = 0x150 + k * 16;
            wf32(&mut buf, vo, pts[k][0]);
            wf32(&mut buf, vo + 4, pts[k][1]);
            wf32(&mut buf, vo + 8, pts[k][2]);
            w32(&mut buf, vo + 12, ws[k]);
        }

        // 0x60: sr#1 ("CARP::road\0", 11 bytes at 0x1a0)
        w32(&mut buf, 0x60, (TAG_SR & 0xFFFF_0000) | 1);
        w32(&mut buf, 0x64, (11 << 8) | 0x01);
        wi32(&mut buf, 0x68, 0);
        wi32(&mut buf, 0x6c, 0x1a0);
        buf[0x1a0..0x1ab].copy_from_slice(b"CARP::road\0");

        // 0x70: ci#0 (1 item = 64 bytes at 0x1c0)
        w32(&mut buf, 0x70, TAG_CI & 0xFFFF_0000);
        w32(&mut buf, 0x74, (64 << 8) | 0x01);
        wi32(&mut buf, 0x78, 1);
        wi32(&mut buf, 0x7c, 0x1c0);
        wf32(&mut buf, 0x1c0, 1.0); // r0.x
        wf32(&mut buf, 0x1c0 + 40, 1.0); // r2.z
        w32(&mut buf, 0x1c0 + 28, 0x7372_0001); // sr#1
        wf32(&mut buf, 0x1c0 + 48, 100.0);
        wf32(&mut buf, 0x1c0 + 52, 5.0);
        wf32(&mut buf, 0x1c0 + 56, 200.0);

        let crp = crate::crp::Crp::parse(&buf).unwrap();
        let tris = crp.extract_collision_tris().unwrap();
        assert_eq!(tris.len(), 2);
        assert_eq!(tris[0].flags, 7);
        assert_eq!(tris[0].v[0], [100.0, 5.0, 200.0]);
        assert_eq!(tris[1].v[2], [101.0, 5.0, 201.0]);
    }

    #[test]
    fn extracts_map_triggers_and_masked_packet_textures() {
        // Test Map -> Trgr 64-byte record parsing.
        let mut buf = vec![0u8; 0x200];
        let w32 = |b: &mut [u8], o: usize, v: u32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let wi32 = |b: &mut [u8], o: usize, v: i32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let wf32 = |b: &mut [u8], o: usize, v: f32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());

        // 0x00: CARP root, pool at 0x10 (1 record = Map), dir count=0
        w32(&mut buf, 0x00, crate::crp::CARP);
        w32(&mut buf, 0x04, 1 << 5);
        wi32(&mut buf, 0x08, 0);
        wi32(&mut buf, 0x0c, 0x10);
        // 0x10: Map -> window at 0x20, count=1 (Trgr)
        w32(&mut buf, 0x10, crate::crp::tag::MAP);
        w32(&mut buf, 0x14, 0);
        wi32(&mut buf, 0x18, 1);
        wi32(&mut buf, 0x1c, 0x20);
        // 0x20: Trgr -> 64 bytes at 0x40
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

        // Test packet_texture with 0x0300_0002 + 0x4000_0000 high bit and V3-32 after STMOD (0x05).
        let mut img = vec![0u8; 0x200];
        // Texture descriptor at 0x20: [0, "plow"]
        img[0x24..0x28].copy_from_slice(b"plow");
        // RM list at 0x40: [0x0300_0002, 0x4000_0020]
        w32(&mut img, 0x40, 0x0300_0002);
        w32(&mut img, 0x44, 0x4000_0020);
        // Packet at 0x80: po - 0x18 = 0x68 -> 0x40
        w32(&mut img, 0x68, 0x40);
        w32(&mut img, 0x88, 3); // qwc = 3 (48 bytes at 0x90..0xc0)
        // VIF stream: STMOD (0x0500_0001), then V3-32 UNPACK (0x7802_0000, 2 verts)
        w32(&mut img, 0x90, 0x0500_0001);
        w32(&mut img, 0x94, 0x7802_0000);
        wf32(&mut img, 0x98, 1.0);
        wf32(&mut img, 0x9c, 2.0);
        wf32(&mut img, 0xa0, 3.0);
        wf32(&mut img, 0xa4, 4.0);
        wf32(&mut img, 0xa8, 5.0);
        wf32(&mut img, 0xac, 6.0);
        // Segment table at 0xc0: [0xA000_0000, 0, 2, 0xA000_FFFF, 0x80]
        w32(&mut img, 0xc0, 0xA000_0000);
        w32(&mut img, 0xc8, 2);
        w32(&mut img, 0xcc, 0xA000_FFFF);
        w32(&mut img, 0xd0, 0x80);
        // Model header at 0x100: +0xc8 = 1, +0xcc = 0xc0
        img.resize(0x200, 0);
        w32(&mut img, 0x1c8, 1);
        w32(&mut img, 0x1cc, 0xc0);

        let m = TrackModule {
            image: img,
            elfd_size: 0x200,
            sections: vec![ModuleSection {
                name: ".data".into(),
                sh_type: 1,
                offset: 0,
                data_offset: 0,
                size: 0x200,
                link: 0,
                entsize: 0,
            }],
            symbols: vec![ModuleSymbol {
                name: "__Model:::Car::{as  0000}::Model[0].00".into(),
                value: 0x100,
                size: 0,
                info: 0,
                shndx: 1,
            }],
            relocations: 0,
        };
        let strips = m.extract_all_article_parts("Car", [-1.0; 3], [1.0; 3]);
        assert_eq!(strips.len(), 1);
        assert_eq!(strips[0].texture.as_deref(), Some("plow"));
        assert_eq!(strips[0].positions, vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
    }

    #[test]
    fn ignores_secondary_v4_32_attribute_blocks_in_batch() {
        let mut img = vec![0u8; 0x300];
        let w32 = |buf: &mut [u8], o: usize, v: u32| {
            buf[o..o + 4].copy_from_slice(&v.to_le_bytes());
        };
        let wf32 = |buf: &mut [u8], o: usize, v: f32| w32(buf, o, v.to_bits());

        // VIF packet at 0x20: dma_tag qwc=8 (128 bytes) at 0x28
        w32(&mut img, 0x28, 8);
        // Header V4-32 (num=1) at 0x30 -> skipped (20 bytes -> 0x44)
        w32(&mut img, 0x30, 0x6c01_c000);
        // Batch attr 0: V4-32 positions (num=2) at 0x44 -> 36 bytes -> 0x68
        w32(&mut img, 0x44, 0x6c02_c022);
        wf32(&mut img, 0x48, 0.1);
        wf32(&mut img, 0x4c, 0.2);
        wf32(&mut img, 0x50, 0.3);
        wf32(&mut img, 0x54, 1.0);
        wf32(&mut img, 0x58, 0.4);
        wf32(&mut img, 0x5c, 0.5);
        wf32(&mut img, 0x60, 0.6);
        wf32(&mut img, 0x64, 1.0);
        // Batch attr 1: V4-32 normals (num=2) at 0x68 -> must NOT be pushed to positions
        w32(&mut img, 0x68, 0x6c02_c04e);
        wf32(&mut img, 0x6c, 1.0);
        wf32(&mut img, 0x70, 0.0);
        wf32(&mut img, 0x74, 0.0);
        wf32(&mut img, 0x78, 0.0);
        wf32(&mut img, 0x7c, 0.0);
        wf32(&mut img, 0x80, 1.0);
        wf32(&mut img, 0x84, 0.0);
        wf32(&mut img, 0x88, 0.0);
        // Batch attr 2: V4-32 UVs (num=2) at 0x8c
        w32(&mut img, 0x8c, 0x6c02_c07a);
        wf32(&mut img, 0x90, 0.25);
        wf32(&mut img, 0x94, 0.75);
        wf32(&mut img, 0xa0, 0.5);
        wf32(&mut img, 0xa4, 0.5);
        // MSCNT at 0xb0
        w32(&mut img, 0xb0, 0x1700_0000);

        let m = TrackModule {
            image: img,
            elfd_size: 0x300,
            sections: vec![ModuleSection {
                name: ".data".into(),
                sh_type: 1,
                offset: 0,
                data_offset: 0,
                size: 0x300,
                link: 0,
                entsize: 0,
            }],
            symbols: Vec::new(),
            relocations: 0,
        };
        let strips = m.parse_vif_packet(0x20, [-1.0; 3], [1.0; 3]);
        assert_eq!(strips.len(), 1);
        assert_eq!(strips[0].positions, vec![[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]]);
        assert_eq!(strips[0].uvs, vec![[0.25, 0.75], [0.5, 0.5]]);
    }
}
