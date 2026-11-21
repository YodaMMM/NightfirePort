//! The driving tracks' scripted paths and camera placements (docs/systems/driving-intros.md):
//! the `Map` window's `AISp` splines, `Cams` camera records and the `pt` keyframe animations the
//! splines name.
//!
//! Evidence: [M] DRIVING.ELF decompile (EE addresses), [D] the eight tracks' data.
//!
//! - **Record strides** [M 0x2da838 registers the `Map` window fix-ups]: `Cams` → `FUN_002da088`
//!   walks `count = +8` records of 0x40 bytes and resolves the word at `+0x1c`; `AISp` →
//!   `FUN_002da590` walks 0x70-byte records and resolves the word at `+0x40`. The resolver
//!   `FUN_002dab48` looks the word up as a record tag of the window (`FUN_002d5a10`) and stores
//!   the record's payload pointer (0 stays 0) [M].
//! - **Script references** name a record by byte offset: `CARP::<<Map>>::{AISp}::224` is spline
//!   224 / 0x70 = 2, `{Cams}::64` is camera 1 [D: every offset on the eight tracks is a multiple
//!   of the stride].
//! - **The `pt` animation** (the payload a spline's `+0x40` resolves to) is sampled by
//!   `FUN_002db370` [M]: `+0` duration (path time units, below), `+0xc` the rotation channel
//!   (−1 none), `+0x10` the position channel (−1 none); channel `i` is a 0x14-byte descriptor at
//!   `+0x40 + 0x14 i` (`FUN_002db320`: flags at `+0x10`): `+4` key-time array offset, `+8` value
//!   array offset, `+0xc` key count, `+0x10` flags. Flags bit 0 = cubic keys (0x30 bytes each,
//!   `FUN_002db188`), bit 2 = scalar keys, bit 3 = an external clock (`FUN_002bfb50`); otherwise
//!   16-byte vec4 keys blended linearly (`FUN_002db080` → `FUN_002dafe0`), bit 1 = the blend is
//!   the normalised quaternion one (`FUN_002cc018`) [M]. The key search `FUN_002dae58` keeps the
//!   last key whose time is ≤ t (a binary search `FUN_002dbfa0` for the first key ≥ t, minus one,
//!   clamped to the array; it first tries the previous answer and its neighbours, which can pick
//!   the key on the other side of an exact key time, with the same sampled value); the next key is the following one unless at the end; the fraction is
//!   `(t − t0) / (t1 − t0)` when `t0 < t1`, else 0 (`FUN_002dafb8`) [M].
//! - **Time units.** The path player advances its time by `FUN_001b0f48(seconds) = seconds ×
//!   DAT_00333ff4 × 60` [M 0x1b14b8], so a path second is 60 units at the default rate; that
//!   `DAT_00333ff4` is 1 in normal play is not traced.

use std::collections::BTreeMap;

use crate::buf::{invalid, FormatError, Result};
use crate::crp::{tag, Crp};
use crate::trackstart::MapScript;

/// `AISp` record size [M 0x2da590].
pub const SPLINE_SIZE: usize = 0x70;
/// `Cams` record size [M 0x2da088].
pub const CAM_SIZE: usize = 0x40;
/// The `pt` channel descriptor stride [M 0x2db320].
pub const CHANNEL_SIZE: usize = 0x14;
/// The `Map` window record tags [M 0x2da838].
pub const TAG_AISP: u32 = 0x4149_5370;
pub const TAG_CAMS: u32 = 0x4361_6d73;
/// The short tag of the `pt` instance records [D].
pub const TAG_PT: u32 = 0x7074_2020;

fn u32_at(d: &[u8], o: usize) -> Result<u32> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| FormatError::Invalid(format!("u32 at {o:#x} past the end ({})", d.len())))
}
fn f32_at(d: &[u8], o: usize) -> Result<f32> { Ok(f32::from_bits(u32_at(d, o)?)) }
fn mat_at(d: &[u8], o: usize) -> Result<[[f32; 4]; 4]> {
    let mut m = [[0.0; 4]; 4];
    for (r, row) in m.iter_mut().enumerate() {
        for (c, v) in row.iter_mut().enumerate() {
            *v = f32_at(d, o + 16 * r + 4 * c)?;
        }
    }
    Ok(m)
}
fn cstr(d: &[u8]) -> String {
    let e = d.iter().position(|&b| b == 0).unwrap_or(d.len());
    String::from_utf8_lossy(&d[..e]).into_owned()
}

/// One `AISp` record [D layout; M stride + the `+0x40` fix-up].
#[derive(Clone, Debug, PartialEq)]
pub struct SplineRecord {
    pub index: usize,
    /// `+0x00..+0x40`: a 4×4 matrix, rows = x axis, y axis, z axis, position (w 1) [D: the last
    /// row's w is 1.0 on every record of the eight tracks].
    pub matrix: [[f32; 4]; 4],
    /// `+0x40`: the reference the loader resolves to the `pt` animation [M 0x2da590]; on disc an
    /// `sr` word whose path is `CARP::<<Map>>::{pt  NNNN}` [D].
    pub anim_ref: u32,
    /// `+0x44`: the spline's name, e.g. `path_BondMaster01` [D].
    pub name: String,
    /// `+0x6c`: a word that is 0 or 1 [D]; no reader traced.
    pub last: u32,
}

/// One `Cams` record [D layout; M stride, the `+0x1c` fix-up and `FUN_001bc510`'s reads].
#[derive(Clone, Debug, PartialEq)]
pub struct CamRecord {
    pub index: usize,
    /// The 4×4 placement; `+0x30` is the position `FUN_001bc510` pushes when `+0x1c` is 0
    /// (`FUN_001a7cf8(dir+0x350, cam+0x30)`) [M].
    pub matrix: [[f32; 4]; 4],
    /// `+0x1c` (row 1's w): 0, or a reference to an `as` record (`0x6173_nnnn`) the camera
    /// animates with [M 0x2da088 resolves it; D: only MIS13A's camera 6 has one].
    pub anim_ref: u32,
}

/// One `pt` channel: key times and vec4 values (linear channels only; see [`PathAnim::parse`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Channel {
    pub flags: u32,
    /// Flag bit 0 (0x30-byte cubic keys, `FUN_002db188`): the key times are read but the
    /// values are left empty, since the order of the four VU rows that function blends is not
    /// visible in the decompile [D: one channel on the eight tracks, MIS13C `pt#1`, has it].
    pub cubic: bool,
    pub times: Vec<f32>,
    pub values: Vec<[f32; 4]>,
}

impl Channel {
    /// `FUN_002dae58` + `FUN_002db028`: (key, next key, fraction) at time `t` [M].
    pub fn key(&self, t: f32) -> (usize, usize, f32) {
        let n = self.times.len();
        if n == 0 {
            return (0, 0, 0.0);
        }
        // FUN_002dbfa0: first key with time >= t; the answer is one before it, clamped.
        let lb = self.times.partition_point(|&k| k < t);
        let i = if lb == 0 { 0 } else { (lb - 1).min(n - 1) };
        let j = if i < n - 1 { i + 1 } else { i };
        let (t0, t1) = (self.times[i], self.times[j]);
        let f = if t0 < t1 { (t - t0) / (t1 - t0) } else { 0.0 };
        (i, j, f)
    }

    /// The channel's value at `t` [M 0x2db080]: linear blend, or with flag bit 1 the
    /// normalised quaternion blend [`nlerp`].
    pub fn sample(&self, t: f32) -> Option<[f32; 4]> {
        if self.values.is_empty() {
            return None;
        }
        let (i, j, f) = self.key(t);
        let (a, b) = (self.values[i], self.values[j]);
        Some(if self.flags & 2 != 0 { nlerp(a, b, f) } else { std::array::from_fn(|k| a[k] + (b[k] - a[k]) * f) })
    }
}

/// `FUN_002cc018`: blend two quaternions, flipping the second when it is nearer as its negative
/// (`|b − a|² > |b + a|²`), then normalise [M; the VU field selects are not visible in the
/// decompile, the weights (1 − f, f) are inferred].
pub fn nlerp(a: [f32; 4], b: [f32; 4], f: f32) -> [f32; 4] {
    let d: f32 = (0..4).map(|k| (b[k] - a[k]) * (b[k] - a[k])).sum();
    let s: f32 = (0..4).map(|k| (b[k] + a[k]) * (b[k] + a[k])).sum();
    let sign = if d - s <= 0.0 { 1.0 } else { -1.0 };
    let q: [f32; 4] = std::array::from_fn(|k| a[k] * (1.0 - f) + sign * b[k] * f);
    let n = (q.iter().map(|v| v * v).sum::<f32>()).sqrt();
    if n > 0.0 { std::array::from_fn(|k| q[k] / n) } else { q }
}

/// A `pt` keyframe animation [M 0x2db370 + D].
#[derive(Clone, Debug, PartialEq)]
pub struct PathAnim {
    pub instance: u16,
    /// `+0`: the end time (`FUN_001b14b8` wraps or stops past it).
    pub duration: f32,
    /// `+0x18`: the name, e.g. `path_BondMain01` [D].
    pub name: String,
    /// `+0xc` / `+0x10` (−1 = none).
    pub rot: Option<usize>,
    pub pos: Option<usize>,
    pub channels: Vec<Channel>,
}

impl PathAnim {
    /// Decodes a `pt` payload. Linear vec4 channels are decoded; a cubic channel keeps its
    /// times and no values ([`Channel::cubic`]); a scalar or external-clock channel (bits 2, 3)
    /// is an error, so a track that used one would fail the disc test instead of sampling
    /// garbage [D: none of the eight tracks has one].
    pub fn parse(instance: u16, p: &[u8]) -> Result<PathAnim> {
        let count = u32_at(p, 4)? as usize;
        if count > 16 {
            return invalid(format!("pt#{instance}: {count} channels"));
        }
        let ch = |w: u32| (w != u32::MAX).then_some(w as usize);
        let (rot, pos) = (ch(u32_at(p, 0xc)?), ch(u32_at(p, 0x10)?));
        let mut channels = Vec::new();
        for i in 0..count {
            let d = 0x40 + CHANNEL_SIZE * i;
            let (to, vo, n, flags) = (u32_at(p, d + 4)? as usize, u32_at(p, d + 8)? as usize, u32_at(p, d + 0xc)? as usize, u32_at(p, d + 0x10)?);
            if flags & 0xc != 0 {
                return invalid(format!("pt#{instance} channel {i}: flags {flags:#x} not decoded"));
            }
            let cubic = flags & 1 != 0;
            let times = (0..n).map(|k| f32_at(p, to + 4 * k)).collect::<Result<Vec<_>>>()?;
            let values = (0..if cubic { 0 } else { n })
                .map(|k| Ok([f32_at(p, vo + 16 * k)?, f32_at(p, vo + 16 * k + 4)?, f32_at(p, vo + 16 * k + 8)?, f32_at(p, vo + 16 * k + 12)?]))
                .collect::<Result<Vec<_>>>()?;
            channels.push(Channel { flags, cubic, times, values });
        }
        for c in [rot, pos].into_iter().flatten() {
            if c >= channels.len() {
                return invalid(format!("pt#{instance}: channel {c} of {}", channels.len()));
            }
        }
        Ok(PathAnim { instance, duration: f32_at(p, 0)?, name: cstr(p.get(0x18..0x40).unwrap_or(&[])), rot, pos, channels })
    }

    /// The position at path time `t` (xyz of the position channel) [M 0x2db370].
    pub fn position(&self, t: f32) -> Option<[f32; 3]> {
        self.pos.and_then(|c| self.channels[c].sample(t)).map(|v| [v[0], v[1], v[2]])
    }

    /// The rotation quaternion at `t` [M 0x2db370 → `FUN_002cc0c8` turns it into the matrix].
    pub fn rotation(&self, t: f32) -> Option<[f32; 4]> { self.rot.and_then(|c| self.channels[c].sample(t)) }
}

/// A track's splines, cameras and path animations.
#[derive(Clone, Debug, Default)]
pub struct MapPaths {
    pub splines: Vec<SplineRecord>,
    pub cams: Vec<CamRecord>,
    pub anims: BTreeMap<u16, PathAnim>,
}

fn offset_ref(script: &MapScript, word: u32, table: &str, stride: usize, len: usize) -> Option<usize> {
    let off: usize = script.ref_path(word)?.strip_prefix(&format!("CARP::<<Map>>::{{{table}}}::"))?.parse().ok()?;
    (off % stride == 0 && off / stride < len).then_some(off / stride)
}

impl MapPaths {
    /// The spline an event argument names (`CARP::<<Map>>::{AISp}::<byte offset>`).
    pub fn spline_ref(&self, script: &MapScript, word: u32) -> Option<usize> {
        offset_ref(script, word, "AISp", SPLINE_SIZE, self.splines.len())
    }
    /// The camera an event argument names (`CARP::<<Map>>::{Cams}::<byte offset>`).
    pub fn cam_ref(&self, script: &MapScript, word: u32) -> Option<usize> {
        offset_ref(script, word, "Cams", CAM_SIZE, self.cams.len())
    }
    /// The `pt` animation a spline's `+0x40` names (`CARP::<<Map>>::{pt  NNNN}`, NNNN the
    /// instance in hex [D: the instance's name matches the spline's own on every spline]).
    pub fn spline_anim(&self, script: &MapScript, spline: usize) -> Option<&PathAnim> {
        let s = self.splines.get(spline)?;
        let path = script.ref_path(s.anim_ref)?;
        let hex = path.strip_prefix("CARP::<<Map>>::{pt  ")?.strip_suffix('}')?;
        self.anims.get(&u16::from_str_radix(hex, 16).ok()?)
    }
}

/// Reads the `AISp`, `Cams` and `pt` records of a track's `Map` window.
pub fn map_paths(crp: &Crp<'_>) -> Result<MapPaths> {
    let map = crp
        .pool()?
        .into_iter()
        .find(|r| r.tag == tag::MAP)
        .ok_or_else(|| FormatError::Invalid("no Map article in the pool".into()))?;
    let mut out = MapPaths::default();
    for r in crp.record_window(&map)? {
        let p = crp.payload(&r)?;
        if r.instance().is_some() {
            if r.tag_code() == TAG_PT {
                let i = r.instance().unwrap();
                out.anims.insert(i, PathAnim::parse(i, p)?);
            }
            continue;
        }
        match r.tag {
            TAG_AISP => {
                for (index, s) in p.chunks_exact(SPLINE_SIZE).enumerate() {
                    out.splines.push(SplineRecord {
                        index,
                        matrix: mat_at(s, 0)?,
                        anim_ref: u32_at(s, 0x40)?,
                        name: cstr(&s[0x44..0x6c]),
                        last: u32_at(s, 0x6c)?,
                    });
                }
            }
            TAG_CAMS => {
                for (index, c) in p.chunks_exact(CAM_SIZE).enumerate() {
                    out.cams.push(CamRecord { index, matrix: mat_at(c, 0)?, anim_ref: u32_at(c, 0x1c)? });
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chan(flags: u32, times: &[f32], values: &[[f32; 4]]) -> Channel {
        Channel { flags, cubic: false, times: times.to_vec(), values: values.to_vec() }
    }

    #[test]
    fn key_search_keeps_the_last_key_at_or_before_t() {
        let c = chan(0, &[0.0, 10.0, 30.0], &[[0.0; 4], [1.0; 4], [3.0; 4]]);
        assert_eq!(c.key(-5.0), (0, 1, -0.5));
        assert_eq!(c.key(0.0), (0, 1, 0.0));
        assert_eq!(c.key(5.0), (0, 1, 0.5));
        // A key exactly on t: the binary search finds that key as "first >= t", the answer is
        // the one before (fraction 1.0), which samples the same value.
        assert_eq!(c.key(10.0), (0, 1, 1.0));
        assert_eq!(c.key(20.0), (1, 2, 0.5));
        // Past the end: the last key twice, fraction 0 (t0 == t1).
        assert_eq!(c.key(99.0), (2, 2, 0.0));
        assert_eq!(c.sample(20.0), Some([2.0; 4]));
        assert_eq!(c.sample(99.0), Some([3.0; 4]));
    }

    #[test]
    fn quaternion_blend_takes_the_short_way_and_normalises() {
        let a = [0.0, 0.0, 0.0, 1.0];
        let b = [0.0, -0.0, 0.0, -1.0]; // the same rotation, negated
        let q = nlerp(a, b, 0.5);
        assert!((q[3] - 1.0).abs() < 1e-6, "{q:?}");
        let c = [0.0, 1.0, 0.0, 0.0];
        let h = nlerp(a, c, 0.5);
        let r = std::f32::consts::FRAC_1_SQRT_2;
        assert!((h[1] - r).abs() < 1e-6 && (h[3] - r).abs() < 1e-6, "{h:?}");
    }

    #[test]
    fn pt_payload_decodes_both_channels() {
        // duration 100, 2 channels, rot = channel 0, pos = channel 1.
        let mut p = vec![0u8; 0x200];
        let put = |p: &mut Vec<u8>, o: usize, w: u32| p[o..o + 4].copy_from_slice(&w.to_le_bytes());
        put(&mut p, 0, 100f32.to_bits());
        put(&mut p, 4, 2);
        put(&mut p, 0xc, 0);
        put(&mut p, 0x10, 1);
        p[0x18..0x1f].copy_from_slice(b"path_xy");
        // ch0: times @0x100 (1 key), values @0x110, flags 2.
        put(&mut p, 0x44, 0x100);
        put(&mut p, 0x48, 0x110);
        put(&mut p, 0x4c, 1);
        put(&mut p, 0x50, 2);
        put(&mut p, 0x110 + 12, 1f32.to_bits());
        // ch1: times @0x120 (2 keys: 0, 100), values @0x130.
        put(&mut p, 0x54 + 4, 0x120);
        put(&mut p, 0x54 + 8, 0x130);
        put(&mut p, 0x54 + 0xc, 2);
        put(&mut p, 0x124, 100f32.to_bits());
        put(&mut p, 0x140, 10f32.to_bits()); // key 1 x = 10
        let a = PathAnim::parse(7, &p).unwrap();
        assert_eq!((a.duration, a.name.as_str(), a.rot, a.pos), (100.0, "path_xy", Some(0), Some(1)));
        assert_eq!(a.position(25.0), Some([2.5, 0.0, 0.0]));
        assert_eq!(a.rotation(50.0), Some([0.0, 0.0, 0.0, 1.0]));
        // A cubic channel keeps no values (it samples to None); a scalar one is refused.
        put(&mut p, 0x50, 1);
        let c = PathAnim::parse(7, &p).unwrap();
        assert!(c.channels[0].cubic && c.rotation(50.0).is_none());
        put(&mut p, 0x50, 4);
        assert!(PathAnim::parse(7, &p).is_err());
    }
}
