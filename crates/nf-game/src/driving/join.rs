//! The nearest-road join and the chasers' lane pick, from DRIVING.ELF (docs/systems/driving-ai.md
//! §12.5). Every function names its address; [M] = read in `work/decomp/DRIVING.ELF/` (with the
//! disassembly where the decompile drops arguments or VU lanes), [D] = disc data.
//!
//! - `nearest_piece` = `FUN_00225490`: the road piece under a world point, through the CGrd grid.
//! - `RoadNav::join_nearest` = `FUN_00225c48`: put a navigator on that piece: the end it drives
//!   toward, its t and its lane offset (per navigator kind).
//! - `lane_pick` = `FUN_00135f70`: the chasers' (pattern 3 off state 4, pattern 4) lane change,
//!   a 64-slot lateral cost map over the road, other cars marked on it.
//! - `chase_target_piece` / `nav_for_pattern`: the inputs the kind-3/4 junction choosers read
//!   (`FUN_00131770`'s AI +0x1fc, `FUN_0012e468`'s join).
//! - `car_half_extents`: the sim box (`FUN_001c9778` → `FUN_001f6e98`) from the car's `Base` box.
//!
//! Nothing here draws a random number.

use super::ai::{dot, first_plain_link, len, normalize, sub, RoadGrid, RoadNav};
use super::road::{LaneSet, RoadNet, LANE_TABLE_DEFAULT, LANE_TABLE_SNOW2A, RSF_CURVED};

fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// The horizontal speed `sqrt(vx² + vz²)` the lane pick scales its lane change by [M 0x135f70
/// 0x136058-0x136074: `vmul.xz`, then x² + z² only].
pub fn speed_xz(v: [f32; 3]) -> f32 {
    let s = v[0] * v[0] + v[2] * v[2];
    if s <= 0.0 {
        0.0
    } else {
        s.sqrt()
    }
}

/// The game's `(int)(x ± 0.5)` rounding (`+0.5` for x ≥ 0, `−0.5` below, then `cvt.w.s`, which
/// truncates) [M 0x135d88 / 0x135f70 0x136640-0x1366b8].
fn round_trunc(x: f32) -> i32 {
    let h = if x < 0.0 { -0.5 } else { 0.5 };
    (x + h) as i32
}

// ------------------------------------------------------------------------------------------
// Geometry: the clamped projection, the lane snap, the road widths
// ------------------------------------------------------------------------------------------

/// `FUN_00222a30` 0x222a30 with the clamp (param_6 = 1): project `p` on the line `a → b`. With
/// `u = n(b − a)`, `s = |p − a|·dot(u, n(p − a))` and `l = |a − b|`: `s ≥ l` gives (1, b), `s ≤ 0`
/// gives (0, a), else (s / l, a + u·s) [M].
pub fn project_clamped(a: [f32; 3], b: [f32; 3], p: [f32; 3]) -> (f32, [f32; 3]) {
    let u = normalize(sub(b, a));
    let s = len(sub(p, a)) * dot(u, normalize(sub(p, a)));
    let l = len(sub(a, b));
    if l <= s {
        return (1.0, b);
    }
    if s <= 0.0 {
        return (0.0, a);
    }
    (s / l, add(a, scale(u, s)))
}

fn lane_table(set: LaneSet) -> &'static [[f32; 6]; 4] {
    match set {
        LaneSet::Snow2a => &LANE_TABLE_SNOW2A,
        LaneSet::Default => &LANE_TABLE_DEFAULT,
    }
}

/// `FUN_00221f20` 0x221f20 — snap a signed lateral offset `x` to a lane of piece `seg`: the first
/// slot k = 1..5 of the class table with `|x| < table[k] + 2.5` (none: lane 0, offset 0), then
/// clamped to the piece's lane count on that side (rs+0x52 for x < 0, rs+0x53 for x > 0, the
/// offset re-read at the clamped slot); the lane and offset take the sign of x (x = 0 counts as
/// positive) [M].
pub fn snap_lane(net: &RoadNet, set: LaneSet, seg: usize, x: f32) -> Result<(i8, f32), String> {
    let s = net.seg(seg)?;
    let t = &lane_table(set)[s.road_class()];
    let mut lane: i32 = 0;
    let mut off = 0.0f32;
    for k in 1..6 {
        if x.abs() < t[k] + 2.5 {
            lane = k as i32;
            off = t[k];
            break;
        }
    }
    let side = if x < 0.0 {
        Some(s.lanes_neg() as i32)
    } else if 0.0 < x {
        Some(s.lanes_pos() as i32)
    } else {
        None
    };
    if let Some(n) = side {
        if n < lane {
            if !(0..6).contains(&n) {
                return Err(format!("piece {seg}: lane count {n} outside the lane table"));
            }
            off = t[n as usize];
            lane = n;
        }
    }
    if x < 0.0 {
        Ok((-lane as i8, -off))
    } else {
        Ok((lane as i8, off))
    }
}

/// `FUN_00221ac8` 0x221ac8 (`side_pos` false) / `FUN_00221cb0` 0x221cb0 (true) — how far the
/// road reaches to the negative / positive side of piece `seg` [M]:
/// - flags & 0x8008: the f32 at rs+0x00 (negative side) / rs+0x04 (positive side);
/// - a junction connector (flag bit 0): the widest of the first plain link (`FUN_002270e8`, not
///   this piece) at node A and at node B, if any is found;
/// - else (0x10 → 5) + (0x800 → 2.5) + (0x200 / 0x400 → 5) + lanes·5 on that side, or with flag
///   0x1000 `(lanes− + lanes+)·5·0.5`;
/// - a result of 0 becomes 5.
pub fn road_width(net: &RoadNet, seg: usize, side_pos: bool) -> Result<f32, String> {
    road_width_depth(net, seg, side_pos, 0)
}

fn road_width_depth(net: &RoadNet, seg: usize, side_pos: bool, depth: u32) -> Result<f32, String> {
    if depth > 64 {
        return Err("road width recursion past 64 pieces".into());
    }
    let s = net.seg(seg)?;
    let f = s.flags();
    let mut w = 0.0f32;
    if f & 0x8008 != 0 {
        let c = s.curve_ctrl_a();
        w = if side_pos { c[1] } else { c[0] };
    } else if f & 1 == 0 {
        if f & 0x10 != 0 {
            w = 5.0;
        }
        if f & 0x800 != 0 {
            w += 2.5;
        }
        let turn = if side_pos { 0x400 } else { 0x200 };
        if f & turn != 0 {
            w += 5.0;
        }
        let (k, lanes) = if f & 0x1000 == 0 {
            (5.0, if side_pos { s.lanes_pos() } else { s.lanes_neg() } as f32)
        } else {
            (0.5, (s.lanes_neg() as i32 + s.lanes_pos() as i32) as f32 * 5.0)
        };
        w += lanes * k;
    } else {
        let mut best = -10000.0f32;
        for node in [s.node_a(), s.node_b()] {
            if let Some(l) = first_plain_link(net, node, Some(s.own_id()))? {
                let v = road_width_depth(net, l, side_pos, depth + 1)?;
                if best <= v {
                    best = v;
                }
            }
        }
        if -10000.0 < best {
            w = best;
        }
    }
    if w == 0.0 {
        w = 5.0;
    }
    Ok(w)
}

// ------------------------------------------------------------------------------------------
// The nearest road piece (FUN_00225490)
// ------------------------------------------------------------------------------------------

/// What `FUN_00225490` returns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Nearest {
    /// The nearest admitted piece, its nearest centre-line point and that point's t (A → B).
    Piece { seg: usize, point: [f32; 3], t: f32 },
    /// The cell has pieces but none is admitted for this navigator: −1.
    None,
    /// The cell has no road list (null or count 0): the game returns piece **0** without writing
    /// the point or t [M 0x2255ac-0x2255c4: `move v0, zero` on both exits]. `FUN_00225c48` then
    /// joins piece 0 with whatever its t slot held, which the port cannot reproduce.
    EmptyCell,
}

/// The grid cell `FUN_00225490` reads: `(coord − min)·inv` through `cvt.w.s` (truncation; a
/// positive overflow saturates to 0x7fffffff); an index past the grid (as unsigned) goes to 0
/// when it is above 0x7ffffffe (negative or overflowed), else to the last cell [M 0x225500-
/// 0x225588]; x against the columns (+0x1c), z against the rows (+0x18); cell = z·cols + x.
pub fn nearest_cell(grid: &RoadGrid, p: [f32; 3]) -> u32 {
    let clamp = |v: f32, n: u32| -> u32 {
        let i = if v.is_nan() { 0 } else { v as i32 }; // `as` saturates like cvt.w.s
        let u = i as u32;
        if u < n {
            u
        } else if 0x7fff_fffe < u {
            0
        } else {
            n.wrapping_sub(1)
        }
    };
    let x = clamp((p[0] - grid.min_x) * grid.inv_cell, grid.cols);
    let z = clamp((p[2] - grid.min_z) * grid.inv_cell, grid.rows);
    z.wrapping_mul(grid.cols).wrapping_add(x)
}

/// Is piece `seg` admitted for a navigator of `kind` (car group `group`)? [M 0x225634-0x2256a4]
/// - flags 0x100: never;
/// - kind 4: only flag-0x8 pieces whose car group (rs+0x5a) is the navigator's +0x04;
///   other kinds: never a flag-0x8 piece;
/// - kind 5: only flag-0x8000 pieces; other kinds: never one.
pub fn admitted(net: &RoadNet, seg: usize, kind: u32, group: u8) -> Result<bool, String> {
    let s = net.seg(seg)?;
    let f = s.flags();
    if f & 0x100 != 0 {
        return Ok(false);
    }
    if kind == 4 {
        if f & 8 == 0 || s.car_group() as u8 != group {
            return Ok(false);
        }
    } else if f & 8 != 0 {
        return Ok(false);
    }
    Ok(if kind == 5 { f & 0x8000 != 0 } else { f & 0x8000 == 0 })
}

/// `FUN_00225490` 0x225490 (816 B) — the nearest road piece to `p` among the pieces listed in
/// `p`'s grid cell [M]: each listed piece once (the rs+0x60 stamp), below the piece count
/// (`DAT_003c8280` = RNhd count 1), `admitted`; its nearest point is the clamped projection on
/// the centre chord A → B (`FUN_00222968` → `FUN_00222a30`), moved onto the Bezier at that t on a
/// curved piece (`FUN_002223b0` → `FUN_00222468`); the 3-D distance decides, the first of equals
/// wins. Only the one cell is read, so a point near a cell edge does not see the next cell's
/// pieces.
pub fn nearest_piece(net: &RoadNet, grid: &RoadGrid, kind: u32, group: u8, p: [f32; 3]) -> Result<Nearest, String> {
    let cell = nearest_cell(grid, p);
    let Some(list) = grid.cells.get(&cell).filter(|l| !l.is_empty()) else {
        return Ok(Nearest::EmptyCell);
    };
    let mut seen = vec![false; net.segs.len()];
    let mut best: Option<(usize, [f32; 3], f32, f32)> = None;
    for &e in list {
        if e as i32 >= net.segs.len() as i32 {
            continue;
        }
        if e < 0 {
            return Err(format!("cell {cell}: negative piece index {e} (the game reads outside its table)"));
        }
        let seg = e as usize;
        if seen[seg] {
            continue;
        }
        seen[seg] = true;
        if !admitted(net, seg, kind, group)? {
            continue;
        }
        let s = net.seg(seg)?;
        let a = net.node(s.node_a() as usize)?.pos();
        let b = net.node(s.node_b() as usize)?.pos();
        let (t, mut pt) = project_clamped(a, b, p);
        if s.flags() & RSF_CURVED != 0 {
            pt = net.seg_point(seg, t.clamp(0.0, 1.0))?;
        }
        let d = len(sub(p, pt));
        if best.map_or(true, |(_, _, _, bd)| d < bd) {
            best = Some((seg, pt, t, d));
        }
    }
    Ok(match best {
        Some((seg, point, t, _)) => Nearest::Piece { seg, point, t },
        None => Nearest::None,
    })
}

// ------------------------------------------------------------------------------------------
// The join (FUN_00225c48) and its relatives
// ------------------------------------------------------------------------------------------

/// What `RoadNav::join_nearest` did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinOutcome {
    /// Joined `nav.seg`.
    Joined,
    /// `FUN_00225490` found no admitted piece: the game writes nav+0x3e = 0xffff and returns; the
    /// port leaves the navigator untouched (its `seg` is a usize).
    NoRoad,
    /// The point's grid cell has no road list (see `Nearest::EmptyCell`); not joined.
    EmptyCell,
}

impl RoadNav {
    /// `FUN_002233d0` 0x2233d0 — a cleared navigator: everything 0 except the link slot +0xb0 =
    /// −1 [M].
    pub fn cleared(kind: u32, lane_set: LaneSet) -> RoadNav {
        RoadNav {
            kind,
            pos: [0.0; 3],
            dir: [0.0; 3],
            end: 0,
            seg: 0,
            t: 0.0,
            p0: [0.0; 3],
            p1: [0.0; 3],
            lane: 0,
            lane_off: 0.0,
            repeated: false,
            lane_set,
            c1: [0.0; 3],
            c2: [0.0; 3],
            lane_from: 0.0,
            lane_to: 0.0,
            blend_frac: 0.0,
            blend_done: 0.0,
            blend_len: 0.0,
            slot: -1,
            hint_toward: false,
            hint_away: false,
            group: 0,
        }
    }

    /// `FUN_00225c48` 0x225c48 (2,288 B) — join the road piece nearest `pos`, heading `dir`
    /// [M; the disassembly for the 0xb0 mask, the `max.s`/`min.s` at 0x22627c / 0x226294 /
    /// 0x2263e8 and the call arguments]:
    /// 1. `nearest_piece` (kind and +0x04 group of this navigator); the carrot takes its point.
    /// 2. The end: toward B (P0 = node A, P1 = node B, t, dir = rs+0x40) when
    ///    `dot(dir, rs+0x40) ≥ 0` (a zero `dir` counts as 1), or for a kind-4 navigator with
    ///    neither +0xb1 nor +0xb2 set; else toward A (P0 = B, P1 = A, `|1 − t|`, the dir negated).
    /// 3. Lane 0 and offset 0; with `centre` (param_4 ≠ 0) they stay so. Otherwise the offset is
    ///    the distance from the point to the piece, lateral only when the point lies behind P0
    ///    (`dot(pos − P0, dir) < 0`), signed by the side of `n(pos − point)` against
    ///    `(d.z, d.y, −d.x)`, d = the dir while t ≤ 0.2, else `n(point − P0)`; then by kind:
    ///    - 1 (traffic): `snap_lane(max(2.5, offset))` — always a positive lane;
    ///    - 2 (pedestrians): beyond the outermost lane: lane `!lanes−` / `lanes+ + 1`, offset
    ///      `∓table[lanes + 1] ± 1`, 2 further out when the other side has no lanes;
    ///    - 4: clamped to `[−road_width(−), road_width(+)]`;
    ///    - 3, 5: the offset as it is.
    /// 4. +0x98/+0x9c/+0xa0 take the offset; P0/P1 at that offset and the curve controls are
    ///    rebuilt (`FUN_00226a28` / `FUN_00226788`); t is `pos` projected on P0 → P1, clamped
    ///    (`FUN_00222a30`); the carrot is the lerp, or the Bezier on a curved piece.
    ///
    /// +0x96 and the blend fields +0xa4..+0xac are not touched by the game and are not modelled.
    pub fn join_nearest(&mut self, net: &RoadNet, grid: &RoadGrid, pos: [f32; 3], dir: [f32; 3], centre: bool) -> Result<JoinOutcome, String> {
        let (seg, point, t0) = match nearest_piece(net, grid, self.kind, self.group, pos)? {
            Nearest::Piece { seg, point, t } => (seg, point, t),
            Nearest::None => return Ok(JoinOutcome::NoRoad),
            Nearest::EmptyCell => return Ok(JoinOutcome::EmptyCell),
        };
        self.pos = point;
        self.repeated = false;
        self.seg = seg;
        let s = net.seg(seg)?;
        let sd = s.dir();
        let mut d = dot(dir, sd);
        if dir == [0.0, 0.0, 0.0] {
            d = 1.0;
        }
        let a = net.node(s.node_a() as usize)?.pos();
        let b = net.node(s.node_b() as usize)?.pos();
        let kind4_free = self.kind == 4 && !self.hint_toward && !self.hint_away;
        if kind4_free || 0.0 <= d {
            self.end = 1;
            self.dir = sd;
            self.t = t0;
            self.p0 = a;
            self.p1 = b;
        } else {
            self.end = 0;
            self.dir = [-sd[0], -sd[1], -sd[2]];
            self.t = (1.0 - t0).abs();
            self.p0 = b;
            self.p1 = a;
        }
        self.lane = 0;
        let mut off = 0.0f32;
        if !centre {
            let v = sub(pos, self.p0);
            let dist = if dot(v, self.dir) < 0.0 {
                let perp = [self.dir[2], self.dir[1], -self.dir[0]];
                let s = dot(v, perp);
                len(sub(add(point, scale(perp, s)), point))
            } else {
                len(sub(pos, point))
            };
            off = dist;
            let perp2 = if self.t <= 0.2 {
                [self.dir[2], self.dir[1], -self.dir[0]]
            } else {
                let u = normalize(sub(point, self.p0));
                [u[2], u[1], -u[0]]
            };
            let w = normalize(sub(pos, point));
            if dot(perp2, w) < 0.0 {
                off = -off;
            }
            match self.kind {
                4 => {
                    off = if off < 0.0 {
                        let m = -road_width(net, seg, false)?;
                        if off < m { m } else { off }
                    } else {
                        let m = road_width(net, seg, true)?;
                        if m < off { m } else { off }
                    };
                }
                2 => {
                    let table = lane_table(self.lane_set);
                    let flat = |idx: i32| -> Result<f32, String> {
                        let k = s.road_class() as i32 * 6 + idx;
                        if !(0..24).contains(&k) {
                            return Err(format!("piece {seg}: lane slot {idx} outside the lane table"));
                        }
                        Ok(table[(k / 6) as usize][(k % 6) as usize])
                    };
                    if off < 0.0 {
                        self.lane = !s.lanes_neg();
                        off = -flat(s.lanes_neg() as i32 + 1)? + 1.0;
                        if s.lanes_pos() == 0 {
                            off += 2.0;
                        }
                    } else {
                        self.lane = s.lanes_pos().wrapping_add(1);
                        off = flat(s.lanes_pos() as i32 + 1)? - 1.0;
                        if s.lanes_neg() == 0 {
                            off -= 2.0;
                        }
                    }
                }
                1 => {
                    let x = if off < 2.5 { 2.5 } else { off };
                    let (lane, o) = snap_lane(net, self.lane_set, seg, x)?;
                    self.lane = lane;
                    off = o;
                }
                _ => {}
            }
        }
        self.lane_off = off;
        self.lane_from = off;
        self.lane_to = off;
        self.rebuild_points(net, off)?;
        let (t, _) = project_clamped(self.p0, self.p1, pos);
        self.t = t;
        self.pos = self.eval(net)?;
        Ok(JoinOutcome::Joined)
    }

    /// `FUN_00225358` 0x225358 — turn the navigator round on its piece: the end flips, the dir
    /// negates, `t = |1 − t|`, P0 ↔ P1, the controls swap, lane and offset negate (+0x98/+0x9c/
    /// +0xa0 all take −+0x98) [M].
    pub fn reverse(&mut self) {
        self.end = (self.end == 0) as u8;
        self.dir = [-self.dir[0], -self.dir[1], -self.dir[2]];
        self.t = (1.0 - self.t).abs();
        std::mem::swap(&mut self.p0, &mut self.p1);
        std::mem::swap(&mut self.c1, &mut self.c2);
        let o = -self.lane_off;
        self.lane = self.lane.wrapping_neg();
        self.lane_off = o;
        self.lane_from = o;
        self.lane_to = o;
    }
}

// ------------------------------------------------------------------------------------------
// The pattern → navigator set-up (FUN_0012e468) and the chase target piece (FUN_00131770)
// ------------------------------------------------------------------------------------------

/// `FUN_0012e468` 0x12e468 — what a drive pattern does to the AI's navigator (+0x54) [M]:
/// - state 4 (traffic): pattern 1 → kind 0, any other → kind 1 (no join);
/// - pattern 3 → kind 3, joined at the car's position and heading with the lane offset;
/// - pattern 4 → kind 4, joined the same way, on the centre line (param_4 = 1) when the
///   avoidance +0x154 is off;
/// - pattern 5 → kind 0 and the scripted path (`FUN_0013dfd8`, not ported);
/// - pattern 1 → kind 0; pattern 2 and pattern 0 leave the kind as it was.
/// Returns the join's outcome when one ran. Pattern 4's +0x124 / +0x128 writes are not ported.
pub fn nav_for_pattern(
    net: &RoadNet,
    grid: &RoadGrid,
    nav: &mut RoadNav,
    state: u32,
    pattern: u32,
    avoid: bool,
    pos: [f32; 3],
    fwd: [f32; 3],
) -> Result<Option<JoinOutcome>, String> {
    if pattern == 0 {
        return Ok(None);
    }
    if state == super::ai::STATE_TRAFFIC {
        nav.kind = if pattern == 1 { 0 } else { 1 };
        return Ok(None);
    }
    match pattern {
        4 => {
            nav.kind = 4;
            Ok(Some(nav.join_nearest(net, grid, pos, fwd, !avoid)?))
        }
        3 => {
            nav.kind = 3;
            Ok(Some(nav.join_nearest(net, grid, pos, fwd, false)?))
        }
        1 | 5 => {
            nav.kind = 0;
            Ok(None)
        }
        // Pattern 2 (and any other) returns before the kind write [M 0x12e468: `if (iVar2 != 5)
        // return;`].
        _ => Ok(None),
    }
}

/// The per-car scratch navigators `FUN_001351d0` keeps on an AI car for others to read: +0xf0
/// (kind 3, stamp +0xe8) and +0x100 (kind 4, stamp +0xf8) [M].
#[derive(Debug, Clone)]
pub struct ScratchNavs {
    pub stamp3: i32,
    pub nav3: RoadNav,
    pub stamp4: i32,
    pub nav4: RoadNav,
}

impl ScratchNavs {
    pub fn new(lane_set: LaneSet) -> ScratchNavs {
        ScratchNavs { stamp3: 0, nav3: RoadNav::cleared(3, lane_set), stamp4: 0, nav4: RoadNav::cleared(4, lane_set) }
    }
}

/// `FUN_001351d0` 0x1351d0 — where another car is on the road, as the asking car sees it [M]:
/// the kind-3 scratch (`kind4` false) or the kind-4 one of group `group`, re-joined (lane offset
/// on) at `pos + vel·0.25` heading `fwd` when its stamp is 0 or 15 ticks old (kind 4: or its
/// group differs), the stamp set to `now`; then turned round (`reverse`) when its dir is against
/// `asker_fwd`. The turn is kept in the scratch.
pub fn scratch_join<'n>(
    net: &RoadNet,
    grid: &RoadGrid,
    sc: &'n mut ScratchNavs,
    pos: [f32; 3],
    vel: [f32; 3],
    fwd: [f32; 3],
    asker_fwd: [f32; 3],
    kind4: bool,
    group: u8,
    now: i32,
) -> Result<&'n RoadNav, String> {
    let ahead = add(pos, scale(vel, 0.25));
    let nav = if !kind4 {
        if sc.stamp3 == 0 || sc.stamp3 + 15 <= now {
            sc.stamp3 = now;
            sc.nav3.kind = 3;
            sc.nav3.join_nearest(net, grid, ahead, fwd, false)?;
        }
        &mut sc.nav3
    } else {
        if sc.stamp4 == 0 || sc.stamp4 + 15 <= now || sc.nav4.group != group {
            sc.stamp4 = now;
            sc.nav4.kind = 4;
            sc.nav4.group = group;
            sc.nav4.join_nearest(net, grid, ahead, fwd, false)?;
        }
        &mut sc.nav4
    };
    if dot(nav.dir, asker_fwd) < 0.0 {
        nav.reverse();
    }
    Ok(nav)
}

/// The target piece (AI +0x1fc) the kind-3/4 junction chooser routes toward, `FUN_00131770`
/// 0x131770 lines 466-517 [M]. `tracking` is the function's bVar19 before its last overrides
/// (set by the earlier target/goal arms, not ported); it is dropped for profiles 0xe/0xf and for
/// any pattern but 3. With `hold` (AI +0x94) nothing changes (`None`). Not tracking → −1. With
/// no target car: the scratch navigator +0x114 gets kind 4 (own nav kind 4) or 3 and joins the
/// goal point (AI +0x1c0, the +0x1a8 waypoint in goal mode) heading the car's forward row; its
/// piece is the target. With a target car: that car's kind-3 scratch (`scratch_join`, asked with
/// this car's forward row) gives the piece. A NoRoad join gives −1 (the game's 0xffff).
/// Not ported: the follow-up when the target piece differs from the car's own (`FUN_00226c88`,
/// a rejoin behind the target).
pub fn chase_target_piece(
    net: &RoadNet,
    grid: &RoadGrid,
    profile: u32,
    pattern: u32,
    tracking: bool,
    hold: bool,
    own_kind: u32,
    own_fwd: [f32; 3],
    goal: [f32; 3],
    scratch: &mut RoadNav,
    target: Option<(&mut ScratchNavs, [f32; 3], [f32; 3], [f32; 3])>,
    now: i32,
) -> Result<Option<i16>, String> {
    let tracking = tracking && !(profile == 0xe || profile == 0xf) && pattern == 3;
    if hold {
        return Ok(None);
    }
    if !tracking {
        return Ok(Some(-1));
    }
    match target {
        None => {
            scratch.kind = if own_kind == 4 { 4 } else { 3 };
            Ok(Some(match scratch.join_nearest(net, grid, goal, own_fwd, false)? {
                JoinOutcome::Joined => scratch.seg as i16,
                JoinOutcome::NoRoad => -1,
                JoinOutcome::EmptyCell => 0,
            }))
        }
        Some((sc, pos, vel, fwd)) => {
            let n = scratch_join(net, grid, sc, pos, vel, fwd, own_fwd, false, 0, now)?;
            Ok(Some(n.seg as i16))
        }
    }
}

/// The chase hint AI +0x1e0 = `n(goal − own position)` (`FUN_00131770` lines 420-447) that
/// `FUN_00130140` hands the kind-3/4 choosers as the direction [M].
pub fn chase_hint(goal: [f32; 3], own_pos: [f32; 3]) -> [f32; 3] {
    normalize(sub(goal, own_pos))
}

// ------------------------------------------------------------------------------------------
// The lane pick (FUN_00135f70) and its helpers
// ------------------------------------------------------------------------------------------

/// The lane mask tables `FUN_00222060` copies: ten 32-byte half masks (0xff = off the road),
/// [D DRIVING.ELF .data 0x33a6d8 (snow2a) / 0x33a818 (every other track), 32-byte stride; the
/// pointer tables at 0x427018 are built by `FUN_00220c28` beside the lane tables].
pub const LANE_MASKS_SNOW2A: [[u8; 32]; 10] = [
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 32, 0, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 32, 0, 0, 0, 0, 0, 0, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 32, 0, 0, 32, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 32, 0, 0, 0, 0, 0, 0, 0, 32, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 32, 0, 0, 0, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 32, 0, 0, 0, 0, 0, 0, 0, 32, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 64, 32, 0, 0, 32, 64, 64, 64, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 32, 0, 0, 0, 0, 0, 0, 0, 32, 64, 64, 64, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 32, 0, 0, 0],
];
pub const LANE_MASKS_DEFAULT: [[u8; 32]; 10] = [
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 0, 0, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 32, 0, 0, 32, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 32, 0, 0, 0, 0, 0, 0, 0, 32, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 32, 0, 0, 0, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 32, 0, 0, 0, 0, 0, 0, 0, 32, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 64, 32, 0, 0, 32, 64, 64, 64, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 64, 64, 64, 64, 32, 0, 0, 0, 0, 0, 0, 0, 32, 64, 64, 64, 64, 64],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 0, 0],
    [255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 64, 32, 0, 0, 0],
];

/// `FUN_00222060`'s table index for a piece [M]: flags 0x10 → 6, (0x820) → 4, 0x800 → 2, else 0;
/// then +1 with 3 or more lanes in all, 8 with exactly one; 9 when that is still 0 and the piece
/// has no flag 0x200.
pub fn lane_mask_index(flags: u16, lanes: i32) -> usize {
    let mut i = if flags & 0x10 != 0 {
        6
    } else if flags & 0x820 == 0x820 {
        4
    } else if flags & 0x800 != 0 {
        2
    } else {
        0
    };
    if lanes >= 3 {
        i += 1;
    } else if lanes == 1 {
        i = 8;
    }
    if flags & 0x200 == 0 && i == 0 {
        i = 9;
    }
    i
}

/// `FUN_00222060` 0x222060 called as (piece, end, buf, 64, 32) by `FUN_00135d88` — the 64-slot
/// lateral mask, slot 32 = the lane offset 0 [M]: a junction connector reads the first plain link
/// at its exit node (rs node[end], `FUN_002270e8` with no exclusion) instead; a flag-0x8008 piece
/// opens `rs+0x00` / `rs+0x04` units to either side (0 open, 0xff off; the near side by `end`);
/// else slots 0..31 = the class half mask and 32..63 its mirror.
pub fn lane_mask(net: &RoadNet, set: LaneSet, seg: usize, end: u8, buf: &mut [u8; 64]) -> Result<(), String> {
    let mut s = net.seg(seg)?;
    if s.flags() & 0x8008 == 0 {
        if s.flags() & 1 != 0 {
            let node = if end == 0 { s.node_a() } else { s.node_b() };
            if let Some(l) = first_plain_link(net, node, None)? {
                s = net.seg(l)?;
            }
        }
        let i = lane_mask_index(s.flags(), s.lanes_neg() as i32 + s.lanes_pos() as i32);
        let t = match set {
            LaneSet::Snow2a => &LANE_MASKS_SNOW2A[i],
            LaneSet::Default => &LANE_MASKS_DEFAULT[i],
        };
        for k in 0..32 {
            buf[k] = t[k];
            buf[32 + k] = t[31 - k];
        }
    } else {
        let c = s.curve_ctrl_a();
        let (wa, wb) = if end == 0 { (c[1] as i32, c[0] as i32) } else { (c[0] as i32, c[1] as i32) };
        for k in 0..32 {
            buf[k] = if 32 - wa < k as i32 { 0 } else { 0xff };
            buf[32 + k] = if (k as i32) < wb { 0 } else { 0xff };
        }
    }
    Ok(())
}

/// `FUN_00135d88` 0x135d88 — rebuild the cost map AI +0x98 (64 bytes): the lane mask of the
/// navigator's piece and end, plus `round(|c − i|·5)` per slot (c = 32 + the rounded current
/// offset), capped at 255; 0xff slots stay [M].
pub fn lane_costs(net: &RoadNet, nav: &RoadNav, hist: &mut [u8; 64]) -> Result<(), String> {
    lane_mask(net, nav.lane_set, nav.seg, nav.end, hist)?;
    let c = round_trunc(nav.lane_off) + 0x20;
    for i in 0..64 {
        if hist[i] == 0xff {
            continue;
        }
        let v = ((c - i as i32) as f32).abs() * 5.0;
        let n = hist[i] as i32 + round_trunc(v);
        hist[i] = n.clamp(0, 255) as u8;
    }
    Ok(())
}

/// `FUN_00135cf0` 0x135cf0 — the window of `w` slots (AI +0xd8) with the lowest cost: start s from
/// 0 while `s < 64 − w`, the first strictly lowest sum (below `w·255`) wins; returns `s + w/2`,
/// 32 when none [M; its second argument is overwritten at once, 0x135cf0 `lbu a1, 0xd8(a0)`].
pub fn best_window(hist: &[u8; 64], w: u8) -> i32 {
    let w = w as i32;
    let mut best = w * 0x100 - w;
    let mut k = 0x20;
    let mut s = 0;
    while s < 0x40 - w {
        let sum: i32 = (s..s + w).map(|i| hist[i as usize] as i32).sum();
        if sum < best {
            k = s + (w >> 1);
            best = sum;
        }
        s += 1;
    }
    k
}

/// `FUN_00135b10` 0x135b10 — is `other` a car to make room for? Within 5 height of the asker
/// (unless both overtake, AI +0x118), within `clamp(2·speed, 20, 35)` (15 when `other_speed > 0`
/// and the other is at least 1 faster) in 3-D, and not behind (`dot(n(fwd), n(other − me)) ≥ 0`)
/// [M + the disassembly: the distances are 3-D, the speeds horizontal].
pub fn makes_room_for(speed: f32, other_speed: f32, me: [f32; 3], other: [f32; 3], fwd: [f32; 3], both_overtake: bool) -> bool {
    if !both_overtake && 5.0 < (me[1] - other[1]).abs() {
        return false;
    }
    let d = len(sub(me, other));
    let mut r = speed + speed;
    if r < 20.0 {
        r = 20.0;
    }
    if 35.0 < r {
        r = 35.0;
    }
    if 0.0 < other_speed && speed + 1.0 <= other_speed {
        r = 15.0;
    }
    if r < d {
        return false;
    }
    0.0 <= dot(normalize(fwd), normalize(sub(other, me)))
}

/// One other vehicle as `FUN_00135f70` reads it off the world's vehicle list (`DAT_00339f5c`)
/// and its controller [M field reads]. Whether the player's own controller answers these reads
/// the same way was not traced.
#[derive(Debug, Clone)]
pub struct LaneCar {
    /// sim +0x46 (the asker skips its own).
    pub sim_id: i16,
    /// sim +0x6d (< 4 is read) and +0x6e (3 = parked, skipped).
    pub kind: i8,
    pub status: i8,
    /// sim +0x10 / +0x20 / sim+0x5c → +0x20 (the forward row) / +0x4c8 (the box z half extent).
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub fwd: [f32; 3],
    pub box_z: f32,
    /// AI +0x118 (overtaking), +0xd8 (its own lane window), +0x84, +0x70.
    pub overtake: bool,
    pub width: u8,
    pub profile: u32,
    pub state: u32,
    /// AI +0x1a4 is the asker.
    pub targets_asker: bool,
    /// The actor's vtbl+0x14c(0..3) all return 0 [M 0x1363c0-0x136428]; what it asks is not traced.
    pub actor_clear: bool,
    /// `FUN_00195a30` of the actor.
    pub alive: f32,
    /// The actor's vtbl+0x9c() (1 skips the side test) [M 0x1364c8-0x1364e8]; not traced.
    pub actor_9c: i32,
    /// The car's scratch navigators (`FUN_001351d0`).
    pub scratch: ScratchNavs,
}

/// The asking car's state `FUN_00135f70` reads and writes.
#[derive(Debug, Clone)]
pub struct LanePicker {
    /// AI +0x98..+0xd7: the cost map.
    pub hist: [u8; 64],
    /// AI +0xd8: the lane window width (slots).
    pub width: u8,
    /// AI +0x118.
    pub overtake: bool,
    /// AI +0x154: the avoidance enable (off: nothing runs).
    pub avoid: bool,
    /// AI +0x94: keep the last cost map (skip the rebuild).
    pub hold: bool,
    /// AI +0x1a4: the target car, as an index into the vehicle list.
    pub target: Option<usize>,
    /// AI +0x204 / +0x208 / +0x148 (read as ≠ 0).
    pub f204: bool,
    pub f208: bool,
    pub f148: bool,
}

/// The asking car's own sim.
#[derive(Debug, Clone, Copy)]
pub struct LaneSelf {
    pub sim_id: i16,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// sim+0x5c → +0x00 / +0x20: the right and forward rows.
    pub right: [f32; 3],
    pub fwd: [f32; 3],
}

/// What `lane_pick` did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LanePickOut {
    /// The scratch navigator +0x114's kind written (4 for a kind-4 car, else 3); `None` when the
    /// pick returned before (avoidance off, a lane change running, or AI +0x94 held).
    pub scratch_kind: Option<u32>,
    /// The lane change started (`FUN_00226f18`): the offset and the blend length.
    pub change: Option<(f32, f32)>,
}

/// `FUN_00135f70` 0x135f70 (2,604 B) — the chasers' lane pick (pattern 3 off state 4, pattern 4
/// [M 0x130508]) [M + the disassembly for every VU lane and branch]:
/// 1. Nothing with the avoidance off or a lane change running (+0xa0 ≠ +0x98). With AI +0x94 the
///    last cost map is reused (step 4 only).
/// 2. `lane_costs`; then for every other listed vehicle with sim kind < 4 and not parked: its
///    speed `s` counts only when ≥ 5, moving forward (`dot(fwd, n(vel)) > 0`) and heading within
///    45° of the asker (`dot(own fwd, its fwd) > 0.707`), else 0; when `makes_room_for`, its
///    road offset (`scratch_join`, kind 4 with the asker's group for a kind-4 asker) rounded
///    half away from zero is the slot `c = 32 + offset`, the weight +0x80 (0 when it chases the
///    asker with profile 0xe, the actor clear and alive): for the asker's own target car −0x40
///    off the weight unless the asker is kind 4 (not below 0), and −0x40 outright when +0x148
///    and `|dot(own right, n(it − me))| > 0.9` (skipped when its vtbl+0x9c is 1; +0x208 sends
///    the target to the next rule instead); for any other car with +0x204 standing by (state 1)
///    and `|dot(nav dir, n(it − me))| < 0.2`, −0x40. The window `max(its +0xd8, round(box_z·
///    2.25·|dot(its side, own fwd)|))` around c (clipped to 0..64) takes the weight, each slot
///    clamped to 0..255.
/// 3. The scratch +0x114 kind: 4 or 3.
/// 4. A kind-4 car that marked nothing and is off the centre: the centre three slots (AI
///    +0xb7..+0xb9) are zeroed and it steers to `offset/2` (0 when `|offset| ≤ 2`). Otherwise the
///    best window (`best_window`) minus 32, minus 0.5 for an even window, is the new offset.
/// 5. A lane change to a new offset over `|old − new|·speed_xz·0.1` (`FUN_00226f18`).
pub fn lane_pick(
    net: &RoadNet,
    grid: &RoadGrid,
    picker: &mut LanePicker,
    me: &LaneSelf,
    nav: &mut RoadNav,
    cars: &mut [LaneCar],
    now: i32,
) -> Result<LanePickOut, String> {
    let mut out = LanePickOut { scratch_kind: None, change: None };
    if !picker.avoid {
        return Ok(out);
    }
    if nav.lane_to != nav.lane_off {
        return Ok(out);
    }
    let speed = speed_xz(me.vel);
    if !picker.hold {
        lane_costs(net, nav, &mut picker.hist)?;
        let mut marked = false;
        for (i, c) in cars.iter_mut().enumerate() {
            if c.sim_id == me.sim_id || !(c.kind < 4) || c.status == 3 {
                continue;
            }
            let mut s = speed_xz(c.vel);
            let u = normalize(c.vel);
            if !(5.0 <= s) || dot(c.fwd, u) <= 0.0 || dot(me.fwd, c.fwd) <= 0.707 {
                s = 0.0;
            }
            let both = picker.overtake && c.overtake;
            if !makes_room_for(speed, s, me.pos, c.pos, me.fwd, both) {
                continue;
            }
            let kind4 = nav.kind == 4;
            let off = scratch_join(net, grid, &mut c.scratch, c.pos, c.vel, c.fwd, me.fwd, kind4, nav.group, now)?.lane_off;
            let centre = round_half_away(off) + 0x20;
            let mut wgt: i32 = 0x80;
            if c.targets_asker && c.profile == 0xe && c.actor_clear && 0.0 < c.alive {
                wgt = 0;
            }
            let to = normalize(sub(c.pos, me.pos));
            let standby_side = |wgt: &mut i32| {
                if picker.f204 && c.state == 1 && dot(nav.dir, to).abs() < 0.2 {
                    *wgt = -0x40;
                }
            };
            if picker.target == Some(i) {
                if picker.f208 {
                    standby_side(&mut wgt);
                } else {
                    if nav.kind != 4 {
                        wgt -= 0x40;
                        if wgt < 0 {
                            wgt = 0;
                        }
                    }
                    if c.actor_9c != 1 && picker.f148 && 0.9 < dot(me.right, to).abs() {
                        wgt = -0x40;
                    }
                }
            } else {
                standby_side(&mut wgt);
            }
            if wgt == 0 {
                continue;
            }
            let side = [c.fwd[2], c.fwd[1], -c.fwd[0]];
            let ext = c.box_z * 2.25 * dot(side, me.fwd).abs();
            let mut w = c.width as i32;
            let re = round_trunc(ext);
            if w <= re {
                w = re;
            }
            let mut start = centre - w / 2;
            if start < 0 {
                start = 0;
            }
            let mut end = (start + w).abs();
            if 0x40 < end {
                end = 0x40;
            }
            for k in start..end {
                let v = picker.hist[k as usize] as i32 + wgt;
                picker.hist[k as usize] = v.clamp(0, 255) as u8;
            }
            marked = true;
        }
        let kind = if nav.kind == 4 { 4 } else { 3 };
        out.scratch_kind = Some(kind);
        if !marked && nav.lane_off != 0.0 && nav.kind == 4 {
            picker.hist[0x1f] = 0;
            picker.hist[0x20] = 0;
            picker.hist[0x21] = 0;
            let to = if 2.0 < nav.lane_off.abs() { nav.lane_off * 0.5 } else { 0.0 };
            let length = (nav.lane_off - to).abs() * (speed * 0.1);
            nav.start_lane_change(net, to, length)?;
            out.change = Some((to, length));
            return Ok(out);
        }
    }
    let k = best_window(&picker.hist, picker.width);
    let mut to = (k - 0x20) as f32;
    if picker.width & 1 == 0 {
        to -= 0.5;
    }
    if to != nav.lane_off {
        let length = (nav.lane_off - to).abs() * (speed * 0.1);
        nav.start_lane_change(net, to, length)?;
        out.change = Some((to, length));
    }
    Ok(out)
}

/// The rounding `FUN_00135f70` gives another car's offset: above 0, `floor + 0.5 ≤ x` → ceil else
/// floor; at or below 0, `x ≤ ceil − 0.5` → floor else ceil (`FUN_002bfb50` floorf,
/// `FUN_002bf970` ceilf, the double floor `FUN_002bf460`) — half away from zero [M 0x1362d4-
/// 0x136394].
pub fn round_half_away(x: f32) -> i32 {
    let r = if 0.0 < x {
        if x.floor() + 0.5 <= x { x.ceil() } else { x.floor() }
    } else if x <= x.ceil() - 0.5 {
        x.floor()
    } else {
        x.ceil()
    };
    r as i32
}

// ------------------------------------------------------------------------------------------
// The vehicle box (FUN_001c9778 → FUN_001f6e98)
// ------------------------------------------------------------------------------------------

/// `FUN_001f6e98` 0x1f6e98's floor on the sim box half extents +0x4c0/+0x4c4/+0x4c8 [M].
pub fn floored_extents(h: [f32; 3]) -> [f32; 3] {
    let f = |x: f32| if x < super::ai::BOX_FLOOR { super::ai::BOX_FLOOR } else { x };
    [f(h[0]), f(h[1]), f(h[2])]
}

/// The car's sim box half extents (x, y, z), as `FUN_001862c0` builds them: `FUN_001c9778`
/// takes `(max − min)·0.5` of the box its model object holds at +0xc (min at +0x10, max at
/// +0x20) and `FUN_001f6e98` stores them, floored at 0.001, at sim+0x5c → +0x4c0/+0x4c4/+0x4c8
/// (the disassembly at 0x186318-0x1867d4: param_10 is sp+0x30, which `FUN_001c9778` fills; the
/// inverse inertia of `FUN_00210938` goes to sp+0x90 → param_9) [M]. The model object's +0xc is
/// `*in#0[+0x1c]`, the first word of the `as` record the first `in` item names
/// (`FUN_001c7940`), and on the disc that word is the reference to the article's `Base` record
/// [D every car model: `as` payload +0x00 = the `Base` tag]. That the loader resolves the
/// reference to the `Base` payload is read from the data layout, not from the loader.
pub fn car_half_extents(crp: &nf_formats::crp::Crp) -> Result<[f32; 3], String> {
    use nf_formats::crp::tag;
    const IN: u32 = 0x696e_2020;
    const AS: u32 = 0x6173_2020;
    const BASE: u32 = 0x4261_7365;
    let arti = crp
        .pool()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|r| r.tag == tag::ARTI)
        .ok_or("no Arti article")?;
    let (s, e) = arti.window(crp.data).map_err(|e| e.to_string())?;
    let recs = crp.window_records(s, e).map_err(|e| e.to_string())?;
    let payload = |r: &nf_formats::crp::Record| crp.payload(r).map_err(|e| e.to_string());
    let inr = recs.iter().find(|r| r.tag_code() == IN && r.instance() == Some(0)).ok_or("no in#0")?;
    let ip = payload(inr)?;
    if ip.len() < 0x20 {
        return Err("in#0 payload too small".into());
    }
    let r = u32::from_le_bytes(ip[0x1c..0x20].try_into().unwrap());
    if r >> 16 != AS >> 16 {
        return Err(format!("in#0 +0x1c {r:#010x} is not an as reference"));
    }
    let asr = recs
        .iter()
        .find(|x| x.tag_code() == AS && x.instance() == Some(r as u16))
        .ok_or_else(|| format!("no as#{}", r as u16))?;
    let ap = payload(asr)?;
    let b = u32::from_le_bytes(ap.get(0..4).ok_or("as payload empty")?.try_into().unwrap());
    if b != BASE {
        return Err(format!("as#{} +0x00 {b:#010x} is not the Base reference", r as u16));
    }
    let base = recs.iter().find(|x| x.tag == BASE).ok_or("no Base")?;
    let bp = payload(base)?;
    if bp.len() < 0x2c {
        return Err("Base payload too small".into());
    }
    let f = |o: usize| f32::from_le_bytes(bp[o..o + 4].try_into().unwrap());
    Ok([(f(0x20) - f(0x10)) * 0.5, (f(0x24) - f(0x14)) * 0.5, (f(0x28) - f(0x18)) * 0.5])
}
