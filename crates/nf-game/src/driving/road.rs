//! The driving missions' road network: the `.crp` track's `RNgp` article and the path-following
//! geometry. Port of the DRIVING.ELF side (docs/systems/driving-ai.md; evidence tags there).
//!
//! Loader provenance: `FUN_00220c28` 0x220c28 ("Init RoadNetwork", 1,744 B) reads the track's
//! `RNgp` article (0x524e6770) out of the `.crp` (docs/formats/crp.md), then within the article's
//! window:
//! - `RNhd` (0x524e6864): payload = five u16 counts — rn, rs, ri, rr, rj in that order
//!   [M 0x220c28:82-86 — `puVar9[0..4]` → DAT_003c8360/8280/8368/8258/8364];
//! - indexed short-tag records `rn  #i` / `rs  #i` / `ri  #i` / `rr  #i` / `rj  #i`
//!   (tag word = code | index, e.g. 0x726e0000 | i) [M 0x220c28:99-201], each contributing its
//!   **payload** (the game's inline data resolution: record-relative when flags bit 0x02, else
//!   absolute) into the pointer tables DAT_0033a5fc (rn) / a600 (rs) / a604 (ri) / a608 (rj) /
//!   a60c (rr) [M 0x220c28:91-96,105-108];
//! - a record whose lookup lands on the window end leaves its table slot untouched (the game
//!   leaves the malloc'd slot uninitialized); this port errors instead — malformed data must not
//!   pass silently (the sister-assertion hook for the tests);
//! - `rs  #i` records are stamped with the global visit counter at rs+0x60 and a segment with
//!   `(flags & 0x8108) == 0` raises DAT_003c825c [M 0x220c28:113-116] — kept as `has_plain_road`;
//! - `rr  #i` route occupancy bytes (+0x2e/+0x2f) are zeroed at load [M 0x220c28:196-198].
//!
//! Record layouts (every offset proven — docs/systems/driving-ai.md §1):
//! - `rn  ` node: +0x00 3×f32 position, +0x0e u8 link count, +0x10 u16[] linked segment ids
//!   [M 0x225c48:179-186, 0x2247b0:99,139-173];
//! - `rs  ` segment: +0x00 / +0x10 the Bezier control points beside node A / node B (flags &
//!   0x4000) [M 0x222d40, 0x226538], +0x0c u16 node A, +0x0e u16 node B [M 0x2215f8:7-8], +0x1c /
//!   +0x1e s16 the junction at node A / B [M 0x223a30], +0x2c f32 chord length [M 0x226538], +0x3c s16 route id, +0x3e s16 own id, +0x40 3×f32 unit
//!   direction A→B [M 0x2216b0:8-10], +0x4c f32 speed limit [M 0x12e808:165], +0x50 u16 flags
//!   [M 0x2247b0:115-118 et al.], +0x52/+0x53 i8 lanes −/+ side [M 0x221f20:28-29], +0x5a i8
//!   car group [M 0x225490:133];
//! - `rr  ` route: +0x2e/+0x2f u8 occupancy in/out [M 0x2213d8:25-26, 0x126c98:134/142].

use nf_formats::crp::{Crp, Record};

/// The `RNgp` article tag ("RoadNetwork") [M 0x220c28:71].
pub const TAG_RNGP: u32 = 0x524e_6770;
/// The `RNhd` header tag [M 0x220c28:75].
pub const TAG_RNHD: u32 = 0x524e_6864;
/// Normalized short-tag codes of the five indexed record kinds [M 0x220c28:99-201].
pub const TAG_RN: u32 = 0x726e_2020;
pub const TAG_RS: u32 = 0x7273_2020;
pub const TAG_RI: u32 = 0x7269_2020;
pub const TAG_RR: u32 = 0x7272_2020;
pub const TAG_RJ: u32 = 0x726a_2020;

/// `rs` flag bits [M — docs/systems/driving-ai.md §1.2].
pub const RSF_CURVED: u16 = 0x4000;
pub const RSF_TURN_RESTR_A: u16 = 0x0200;
pub const RSF_TURN_RESTR_B: u16 = 0x0400;

/// One `rn  ` road node record's payload [M 0x225c48/0x2247b0].
#[derive(Clone, Copy)]
pub struct RoadNode<'a> {
    pub data: &'a [u8],
}

impl RoadNode<'_> {
    /// +0x00: the node position x,y,z [M 0x225c48:179-186].
    pub fn pos(&self) -> [f32; 3] {
        let f = |o: usize| f32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        [f(0x00), f(0x04), f(0x08)]
    }
    /// +0x0e: the linked-segment count [M 0x2247b0:99,139-150].
    pub fn link_count(&self) -> usize {
        self.data.get(0x0e).copied().unwrap_or(0) as usize
    }
    /// +0x10: the i-th linked segment id [M 0x2247b0:153,173].
    pub fn link(&self, i: usize) -> Option<u16> {
        let at = 0x10 + i * 2;
        let b = self.data.get(at..at + 2)?;
        Some(u16::from_le_bytes(b.try_into().unwrap()))
    }
    /// +0x0c: the node's own index [M 0x2235c8:165 compares it with a segment's node index;
    /// D every RNgp on the disc: equals the record index].
    pub fn own_id(&self) -> u16 {
        u16::from_le_bytes(self.data[0x0c..0x0e].try_into().unwrap())
    }
}

/// Which lane-offset table set the track uses. `FUN_00220c28` 0x220c28 points the six class
/// tables at .data 0x33a618.. when the track is `snow2a_mis4` or `snow2a_race`, else at
/// 0x33a678.. [M 0x220c28:21-61 — the two strcmp calls against .rodata 0x399be8/0x399c08].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneSet {
    /// `snow2a_mis4` / `snow2a_race`.
    Snow2a,
    /// Every other track.
    Default,
}

/// The lane-centre offsets per road class (0..3, `RoadSeg::road_class`) and lane slot 0..5
/// [D DRIVING.ELF .data 0x33a618 (Snow2a) and 0x33a678 (Default), 6 f32 per class; classes 4/5
/// are never selected — FUN_00221e98 returns 0..3].
pub const LANE_TABLE_SNOW2A: [[f32; 6]; 4] = [
    [0.0, 2.5, 7.5, 12.5, 17.5, 22.5],
    [0.0, 4.0, 9.0, 14.0, 19.0, 24.0],
    [0.0, 6.0, 11.5, 16.5, 21.5, 26.5],
    [0.0, 8.0, 13.0, 18.0, 23.0, 28.0],
];
pub const LANE_TABLE_DEFAULT: [[f32; 6]; 4] = [
    [0.0, 2.0, 6.0, 10.0, 14.0, 18.0],
    [0.0, 3.0, 7.0, 11.0, 15.0, 19.0],
    [0.0, 6.0, 11.5, 16.5, 21.5, 26.5],
    [0.0, 8.0, 13.0, 18.0, 23.0, 28.0],
];

/// The signed lane offset: `±table[class][|lane|]`, negative for a negative lane
/// [M 0x2257c0:136-150 (the join), 0x223f98's tail (the next-segment pick)]. A lane past the
/// table's six slots reads past the array in the game; the port clamps to slot 5 [S].
pub fn lane_offset(set: LaneSet, class: usize, lane: i8) -> f32 {
    let t = match set {
        LaneSet::Snow2a => &LANE_TABLE_SNOW2A,
        LaneSet::Default => &LANE_TABLE_DEFAULT,
    };
    let v = t[class.min(3)][(lane.unsigned_abs() as usize).min(5)];
    if lane < 0 {
        -v
    } else {
        v
    }
}

/// One `rs  ` road segment record's payload — the AI spline piece [M 0x2215f8/0x2216b0/0x12e808].
#[derive(Clone, Copy)]
pub struct RoadSeg<'a> {
    pub data: &'a [u8],
}

impl RoadSeg<'_> {
    /// +0x0c: node A index [M 0x2215f8:7-8].
    pub fn node_a(&self) -> u16 {
        u16::from_le_bytes(self.data[0x0c..0x0e].try_into().unwrap())
    }
    /// +0x0e: node B index [M 0x2215f8:7-8].
    pub fn node_b(&self) -> u16 {
        u16::from_le_bytes(self.data[0x0e..0x10].try_into().unwrap())
    }
    /// +0x3e: the segment's own id [M 0x2247b0:105].
    pub fn own_id(&self) -> u16 {
        u16::from_le_bytes(self.data[0x3e..0x40].try_into().unwrap())
    }
    /// +0x40: the unit direction A→B [M 0x2216b0:8-10].
    pub fn dir(&self) -> [f32; 3] {
        let f = |o: usize| f32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        [f(0x40), f(0x44), f(0x48)]
    }
    /// +0x4c: the per-segment speed limit — the AI traffic target-speed base [M 0x12e808:165].
    pub fn speed_limit(&self) -> f32 {
        f32::from_le_bytes(self.data[0x4c..0x50].try_into().unwrap())
    }
    /// +0x50: the flags word [M 0x2247b0:115-118, 0x222468:11].
    pub fn flags(&self) -> u16 {
        u16::from_le_bytes(self.data[0x50..0x52].try_into().unwrap())
    }
    /// +0x52/+0x53: the lane counts on the −/+ side [M 0x221f20:28-29].
    pub fn lanes_neg(&self) -> i8 {
        self.data[0x52] as i8
    }
    pub fn lanes_pos(&self) -> i8 {
        self.data[0x53] as i8
    }
    /// +0x5a: the car-group id [M 0x225490:133].
    pub fn car_group(&self) -> i8 {
        self.data[0x5a] as i8
    }
    /// +0x20: the unit side vector at the node-A end — the lane offset direction
    /// `FUN_002267d8` 0x2267d8 adds at node A (negated when the car drives B→A) [M 0x2267d8:31-45;
    /// D every RNgp on the disc: unit length, perpendicular to +0x40 on straight pieces].
    pub fn side_a(&self) -> [f32; 3] {
        let f = |o: usize| f32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        [f(0x20), f(0x24), f(0x28)]
    }
    /// +0x30: the unit side vector at the node-B end [M 0x2267d8:31-45].
    pub fn side_b(&self) -> [f32; 3] {
        let f = |o: usize| f32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        [f(0x30), f(0x34), f(0x38)]
    }
    /// +0x58: the lane direction mask — bit k set = lane slot k drives A→B (with the game's
    /// drive-side flag DAT_003c827c = 1, its only write [M 0x220c28:63]) [M 0x13c5e8:43-58,
    /// 0x223f98's lane test].
    pub fn lane_dir_mask(&self) -> u8 {
        self.data[0x58]
    }
    /// The road class that picks the lane-offset table — `FUN_00221e98` 0x221e98: flags 0x10 → 3,
    /// (flags & 0x820) == 0x820 → 2, flags 0x800 → 1, else 0 [M 0x221e98:5-11].
    pub fn road_class(&self) -> usize {
        let f = self.flags();
        if f & 0x10 != 0 {
            3
        } else if f & 0x820 == 0x820 {
            2
        } else if f & 0x800 != 0 {
            1
        } else {
            0
        }
    }
    /// +0x10: the curve's second control point, the one beside node B (flags & 0x4000)
    /// [M 0x222d40:54 passes `param_5 + 0x10` as the Bezier's P2; D every curved Paris piece: it
    /// lies nearer node B than node A].
    pub fn curve_ctrl(&self) -> [f32; 3] {
        let f = |o: usize| f32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        [f(0x10), f(0x14), f(0x18)]
    }
    /// +0x00: the curve's first control point, the one beside node A [M 0x222d40 passes the
    /// record itself (`param_5`) as P1; FUN_00226538 0x226538 reads `rs + 0` for the node-A end].
    pub fn curve_ctrl_a(&self) -> [f32; 3] {
        let f = |o: usize| f32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        [f(0x00), f(0x04), f(0x08)]
    }
    /// +0x2c: the centre-line chord length |A − B| the lane control points are scaled against
    /// [M 0x226538 `rs+0x2c / |P0 − P1|`; D every curved Paris piece: equals |A − B|].
    pub fn chord_len(&self) -> f32 {
        f32::from_le_bytes(self.data[0x2c..0x30].try_into().unwrap())
    }
    /// +0x1c / +0x1e: the junction (`ri`) at node A / node B, −1 for none [M 0x223a30 and
    /// 0x1377c8 read `rs + 0x1c + end·2`; D the Paris junction records list these pieces].
    pub fn junction_at(&self, end: u8) -> i16 {
        let o = 0x1c + (end as usize & 1) * 2;
        i16::from_le_bytes([self.data[o], self.data[o + 1]])
    }
    /// +0x3c: the route (`rr`) index, −1 for none [M 0x13ca60, 0x132d80].
    pub fn route(&self) -> i16 {
        i16::from_le_bytes([self.data[0x3c], self.data[0x3d]])
    }
    /// +0x59: the trigger id that holds a 0x100 piece shut while non-zero [M 0x13d618, 0x126c98:148].
    pub fn trigger_id(&self) -> u8 {
        self.data[0x59]
    }
}

/// The cubic path evaluation of the driving engine's curve object: `FUN_001a85d0` 0x1a85d0 builds
/// `rows = basis · [P0 P1 P2 P3]` (`FUN_002cbb88`, one VU multiply-add chain per row) and returns
/// `rows · [t³, t², t, 1]`. Every road curve object is built by `FUN_001a7ba0` (kind +0x60 = 2),
/// which selects the basis at .data 0x37e4a0 (`FUN_001a7dd0`): the Bezier matrix
/// [−1 3 −3 1 / 3 −6 3 0 / −3 3 0 0 / 1 0 0 0] [M + D DRIVING.ELF .data 0x37e4a0]. The other basis
/// (0x37e4e0, ×0.5, the Catmull-Rom matrix) is for kind 1, which no road curve uses.
pub const BEZIER_BASIS: [[f32; 4]; 4] =
    [[-1.0, 3.0, -3.0, 1.0], [3.0, -6.0, 3.0, 0.0], [-3.0, 3.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]];

pub fn bezier(p: [[f32; 3]; 4], t: f32) -> [f32; 3] {
    let tv = [t * t * t, t * t, t, 1.0];
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let mut acc = 0.0f32;
        for (j, row) in BEZIER_BASIS.iter().enumerate() {
            let r = row[0] * p[0][c] + row[1] * p[1][c] + row[2] * p[2][c] + row[3] * p[3][c];
            acc += r * tv[j];
        }
        out[c] = acc;
    }
    out
}

/// One `ri  ` record: a road junction [M 0x223a30 / 0x1377c8 / 0x13efc8; D every Paris record is
/// 48 bytes, +0x0c equals its index].
#[derive(Clone, Copy)]
pub struct Junction<'a> {
    pub data: &'a [u8],
}

impl Junction<'_> {
    /// +0x00: the junction position (the traffic-light check measures from it) [M 0x13efc8].
    pub fn pos(&self) -> [f32; 3] {
        let f = |o: usize| f32::from_le_bytes(self.data[o..o + 4].try_into().unwrap());
        [f(0x00), f(0x04), f(0x08)]
    }
    /// +0x0c: the junction's own index [D].
    pub fn own_id(&self) -> u16 {
        u16::from_le_bytes([self.data[0x0c], self.data[0x0d]])
    }
    /// +0x0e count, +0x10 s16[]: the neighbouring junctions [M 0x223a30].
    pub fn neighbours(&self) -> Vec<i16> {
        let n = (self.data[0x0e] as usize).min(4);
        (0..n).map(|i| i16::from_le_bytes([self.data[0x10 + i * 2], self.data[0x11 + i * 2]])).collect()
    }
    /// +0x18 count, +0x1a s16[]: the road pieces that meet here [M 0x223a30, 0x1377c8].
    pub fn roads(&self) -> Vec<i16> {
        let n = self.data[0x18] as usize;
        (0..n)
            .filter_map(|i| {
                let o = 0x1a + i * 2;
                self.data.get(o..o + 2).map(|b| i16::from_le_bytes([b[0], b[1]]))
            })
            .collect()
    }
}

/// One `rr  ` route record's payload [M 0x2213d8/0x126c98].
#[derive(Clone, Copy)]
pub struct RouteRec<'a> {
    pub data: &'a [u8],
}

impl RouteRec<'_> {
    /// +0x2e/+0x2f: the occupancy counters (zeroed at load) [M 0x220c28:196-198].
    pub fn occupancy(&self) -> (u8, u8) {
        (self.data[0x2e], self.data[0x2f])
    }
    /// +0x2c: the traffic cap — a route takes a spawn only while +0x2e < +0x2c [M 0x13ca60].
    pub fn traffic_cap(&self) -> u8 {
        self.data[0x2c]
    }
}

/// The parsed road network: the five payload tables exactly as FUN_00220c28 builds them.
pub struct RoadNet<'a> {
    pub nodes: Vec<RoadNode<'a>>,
    pub segs: Vec<RoadSeg<'a>>,
    pub routes: Vec<RouteRec<'a>>,
    /// `ri` payloads — built, no proven field reader [M-negative 0x1377c8:108 only].
    pub ri: Vec<&'a [u8]>,
    /// `rj` payloads — built, no reader in the decompile [M-negative].
    pub rj: Vec<&'a [u8]>,
    /// A segment with `(flags & 0x8108) == 0` exists [M 0x220c28:115-116 → DAT_003c825c].
    pub has_plain_road: bool,
}

fn le_u16_list(payload: &[u8]) -> Result<Vec<u16>, String> {
    if payload.len() < 10 {
        return Err(format!("RNhd payload too small: {} B", payload.len()));
    }
    Ok((0..5)
        .map(|i| u16::from_le_bytes(payload[i * 2..i * 2 + 2].try_into().unwrap()))
        .collect())
}

fn bucket<'a>(
    recs: &[Record],
    data: &'a [u8],
    code: u32,
    expect: usize,
    kind: &str,
) -> Result<Vec<&'a [u8]>, String> {
    let mut picked: Vec<Option<&'a [u8]>> = (0..expect).map(|_| None).collect();
    for r in recs {
        if r.tag_code() != code {
            continue;
        }
        // The game's per-index find: tag = code | index [M 0x220c28:99-101].
        let idx = match r.instance() {
            Some(i) => i as usize,
            None => continue,
        };
        if idx >= expect {
            continue; // beyond the header count — the game never asks for it
        }
        if picked[idx].is_some() {
            return Err(format!("duplicate {} record #{}", kind, idx));
        }
        let at = r.data_addr(data).map_err(|e| e.to_string())?;
        let payload = data
            .get(at..at + r.size())
            .ok_or_else(|| format!("{} record #{} payload past the end", kind, idx))?;
        picked[idx] = Some(payload);
    }
    // A missing index = the game's uninitialized-table-slot case; the port rejects it.
    picked
        .into_iter()
        .enumerate()
        .map(|(i, p)| p.ok_or_else(|| format!("missing {kind} record #{i} (count {expect})")))
        .collect()
}

impl<'a> RoadNet<'a> {
    /// Parse the `RNgp` article out of a decompressed track `.crp` [M 0x220c28]. The article sits
    /// in the root record's table (pool + directory — the top-level finder FUN_002d51f0 0x2d51f0
    /// scans the whole table from the buffer base [M], docs/formats/crp.md §4 puts RNgp in the
    /// track pool).
    pub fn parse_crp(crp: &Crp<'a>) -> Result<RoadNet<'a>, String> {
        let data = crp.data;
        let mut rngp = None;
        for table in [crp.pool(), crp.directory()] {
            for r in table.map_err(|e| e.to_string())? {
                if r.tag == TAG_RNGP {
                    rngp = Some(r);
                    break;
                }
            }
            if rngp.is_some() {
                break;
            }
        }
        let rngp = rngp.ok_or("no RNgp article in the track .crp")?;
        let (ws, we) = rngp.window(data).map_err(|e| e.to_string())?;
        if we > data.len() {
            return Err(format!("RNgp window {ws:#x}..{we:#x} past the end"));
        }
        let recs = crp.window_records(ws, we).map_err(|e| e.to_string())?;

        let rnhd = recs
            .iter()
            .find(|r| r.tag_code() == TAG_RNHD)
            .ok_or("no RNhd header in the RNgp window")?;
        let counts = le_u16_list(crp.payload(rnhd).map_err(|e| e.to_string())?)?;
        let (n_rn, n_rs, n_ri, n_rr, n_rj) =
            (counts[0] as usize, counts[1] as usize, counts[2] as usize, counts[3] as usize, counts[4] as usize);

        let rn = bucket(&recs, data, TAG_RN, n_rn, "rn")?;
        let rs = bucket(&recs, data, TAG_RS, n_rs, "rs")?;
        let ri = bucket(&recs, data, TAG_RI, n_ri, "ri")?;
        let rr = bucket(&recs, data, TAG_RR, n_rr, "rr")?;
        let rj = bucket(&recs, data, TAG_RJ, n_rj, "rj")?;

        let has_plain_road = rs.iter().any(|p| {
            let flags = u16::from_le_bytes(p[0x50..0x52].try_into().unwrap());
            flags & 0x8108 == 0
        });

        Ok(RoadNet {
            nodes: rn.into_iter().map(|data| RoadNode { data }).collect(),
            segs: rs.into_iter().map(|data| RoadSeg { data }).collect(),
            routes: rr.into_iter().map(|data| RouteRec { data }).collect(),
            ri,
            rj,
            has_plain_road,
        })
    }

    /// The centre-line position at `t` — `FUN_00222468` 0x222468: straight pieces
    /// `A + (B − A)·t` (VU `_vsub/_vmulbc/_vadd`); curved pieces (flags & 0x4000) go through
    /// `FUN_00222d40` 0x222d40, the Bezier through P0 = node A, P1 = rs+0x00, P2 = rs+0x10,
    /// P3 = node B (`FUN_001a8380`'s copies with 0x222d40's arguments) [M].
    pub fn seg_point(&self, seg: usize, t: f32) -> Result<[f32; 3], String> {
        let s = self.seg(seg)?;
        if s.flags() & RSF_CURVED != 0 {
            return self.curve_point(seg, t);
        }
        let a = self.node(s.node_a() as usize)?.pos();
        let b = self.node(s.node_b() as usize)?.pos();
        let mut out = [0.0f32; 3];
        for i in 0..3 {
            out[i] = a[i] + (b[i] - a[i]) * t;
        }
        Ok(out)
    }

    /// The curved-piece centre line: `bezier([A, rs+0x00, rs+0x10, B], t)`. `FUN_00222d40`
    /// calls `FUN_001a8380(curve, param_3 = A, param_5 = rs, param_4 = B, rs + 0x10)`, which
    /// stores its 2nd/3rd/5th/4th arguments as P0..P3, so P1 = rs+0x00 and P2 = rs+0x10; the
    /// evaluation is `FUN_001a85d0` with the Bezier basis [M; basis D .data 0x37e4a0]. The old
    /// port's quadratic through rs+0x10 alone was wrong.
    pub fn curve_point(&self, seg: usize, t: f32) -> Result<[f32; 3], String> {
        let s = self.seg(seg)?;
        let a = self.node(s.node_a() as usize)?.pos();
        let b = self.node(s.node_b() as usize)?.pos();
        Ok(bezier([a, s.curve_ctrl_a(), s.curve_ctrl(), b], t))
    }

    /// The `ri` junction record `i` [M 0x223a30's `DAT_0033a604` table].
    pub fn junction(&self, i: usize) -> Result<Junction<'a>, String> {
        self.ri
            .get(i)
            .map(|data| Junction { data })
            .ok_or_else(|| format!("junction #{i} out of range ({})", self.ri.len()))
    }

    /// A reduced next-segment walk modelled on `FUN_002247b0` 0x2247b0 — the nav-kind-2 chooser,
    /// which the AI cars never run (traffic navs are kind 1 → `ai::next_segment_traffic`, the
    /// port of FUN_00223f98 [M 0x12e468 sets the kind, 0x224e00:127-141 dispatches on it]).
    /// Review verdict (docs/systems/driving-ai.md §9): the car-group filter is not in 0x2247b0
    /// and the 0x200/0x400 bits are read per travel end there, not as blanket exclusions — this
    /// walk is kept for its old tests only and is not used by the car AI.
    /// `entry` = the segment being left, `entry_node` = the shared node, `avoid_group` = the
    /// car-group filter (rs+0x5a) the callers pass through FUN_00225490:133.
    pub fn next_segment(
        &self,
        entry: usize,
        entry_node: u16,
        car_group: i8,
    ) -> Result<Option<(usize, bool)>, String> {
        let entry_seg = self.seg(entry)?;
        let node = self.node(entry_node as usize)?;
        for i in 0..node.link_count() {
            let cand = match node.link(i) {
                Some(c) => c as usize,
                None => break,
            };
            let cs = self.seg(cand)?;
            if cs.own_id() == entry_seg.own_id() {
                continue; // the self-id skip [M 0x2247b0:105]
            }
            let f = cs.flags();
            if f & (RSF_TURN_RESTR_A | RSF_TURN_RESTR_B | 0x2) != 0 {
                continue; // the turn-restriction / blocked filters [M 0x2247b0:115-118]
            }
            if cs.car_group() != car_group {
                continue; // the car-group filter [M 0x225490:133]
            }
            // Direction: does the candidate leave through the shared node? [M 0x2247b0:209-255]
            let leaves_shared = cs.node_a() == entry_node || cs.node_b() == entry_node;
            let _ = entry_seg; // the entry's own end bookkeeping happens at the caller
            return Ok(leaves_shared.then_some((cand, true)));
        }
        Ok(None)
    }

    pub fn node(&self, i: usize) -> Result<RoadNode<'a>, String> {
        self.nodes
            .get(i)
            .copied()
            .ok_or_else(|| format!("node #{i} out of range ({})", self.nodes.len()))
    }
    pub fn seg(&self, i: usize) -> Result<RoadSeg<'a>, String> {
        self.segs
            .get(i)
            .copied()
            .ok_or_else(|| format!("segment #{i} out of range ({})", self.segs.len()))
    }
}
