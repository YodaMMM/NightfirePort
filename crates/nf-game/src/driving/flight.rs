//! The MIS01 (Paris Prelude) sniper phase beyond the weapon mode in `pov.rs`: the
//! helicopter's flight along its keyed path, the scripted throttle events, the trigger contact
//! rule, the mission timers with the rule that ends the phase, the gunner's view limits and the
//! Sniper Rifle's shot test. Write-up: docs/systems/driving-mis01-opening.md §7-§11.
//!
//! Evidence: [M] DRIVING.ELF decompile / disassembly (EE addresses), [D] disc bytes,
//! [S] suggested. Every function cites the code it ports. Not ported (stated plainly): the
//! stick-to-angle integration of the auto-drive camera (`FUN_001b72f0`'s deadzone/autoaim
//! paths), the zoom input that feeds the scope FOV (camera +0x2ac), the camera matrix build
//! from the angles, the vehicle box refinement of the shot test (`FUN_001f5eb0`), the damage
//! a hit deals once it reaches the car (`TakeDamage` through the target's vtable +0x1c) and the
//! box trigger shape (shape 1, `FUN_001f5eb0`).

use nf_formats::drive_script::{PathTrack, TrackChannel, TriggerRecord};
use std::collections::BTreeMap;

/// A row-major 4x4 matrix as the game stores it: rows 0-2 the axes, row 3 the position.
pub type Mat4 = [[f32; 4]; 4];

pub const IDENTITY: Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

/// The base matrix a path element with +0x1c == 0 gets (`FUN_001b1258` copies 0x381f70..0x381fb0)
/// [D DRIVING.ELF data]: a quarter turn that maps the model's right axis to the path's
/// backward axis and its forward axis to the path's right axis.
pub const BASE_TURN: Mat4 = [[0.0, 0.0, -1.0, 0.0], [0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

/// The path element's rate (+0x18) the spline controller ctor `FUN_0013da80` writes: 0.5
/// (`puVar3 + 0xb` = +0x58 = 0x3f000000) [M]. The follower copies it to +0xa4.
pub const CONTROLLER_RATE: f32 = 0.5;

/// `FUN_002cbb88`: `out[i] = a[i].x*b[0] + a[i].y*b[1] + a[i].z*b[2] + a[i].w*b[3]` (the VU
/// `vmula/vmadda/vmadda/vmadd` chain, decoded from the raw words) [M].
pub fn mat_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut o = [[0.0; 4]; 4];
    for i in 0..4 {
        for k in 0..4 {
            o[i][k] = a[i][0] * b[0][k] + a[i][1] * b[1][k] + a[i][2] * b[2][k] + a[i][3] * b[3][k];
        }
    }
    o
}

/// `FUN_002cc0c8`: quaternion (x, y, z, w) to matrix rows, row 3 = (0, 0, 0, 1). Decoded from
/// the VU words (the decompile drops the broadcast fields): row0 = (1-2yy-2zz, 2xy+2zw,
/// 2xz-2yw), row1 = (2xy-2zw, 1-2xx-2zz, 2yz+2xw), row2 = (2xz+2yw, 2yz-2xw, 1-2xx-2yy) [M].
pub fn quat_rows(q: [f32; 4]) -> Mat4 {
    let [x, y, z, w] = q;
    let (xx, xy, xz, xw) = (2.0 * x * x, 2.0 * x * y, 2.0 * x * z, 2.0 * x * w);
    let (yy, yz, yw) = (2.0 * y * y, 2.0 * y * z, 2.0 * y * w);
    let (zz, zw) = (2.0 * z * z, 2.0 * z * w);
    [
        [1.0 - yy - zz, xy + zw, xz - yw, 0.0],
        [xy - zw, 1.0 - xx - zz, yz + xw, 0.0],
        [xz + yw, yz - xw, 1.0 - xx - yy, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// `FUN_002dafe0` with flag 0: `a + (b - a) * t` on all four lanes [M].
pub fn lerp4(t: f32, a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [0, 1, 2, 3].map(|i| a[i] + (b[i] - a[i]) * t)
}

/// `FUN_002cc018`: the shortest-arc normalised blend. `a*(1-t) + b*t` when
/// `|b-a|² - |b+a|² <= 0` (tested on the float's bit pattern, `blez`), else `b*t - a*(1-t)`;
/// then scaled by `rsqrt(|q|²)` over four lanes [M, decoded from the VU words].
pub fn nlerp(t: f32, a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let d = [0, 1, 2, 3].map(|i| b[i] - a[i]);
    let s = [0, 1, 2, 3].map(|i| b[i] + a[i]);
    let dot = |v: [f32; 4]| v[0] * v[0] + v[1] * v[1] + v[2] * v[2] + v[3] * v[3];
    let diff = dot(d) - dot(s);
    let bt = b.map(|v| v * t);
    let a1 = a.map(|v| v * (1.0 - t));
    let q = if (diff.to_bits() as i32) <= 0 { [0, 1, 2, 3].map(|i| bt[i] + a1[i]) } else { [0, 1, 2, 3].map(|i| bt[i] - a1[i]) };
    let r = 1.0 / dot(q).sqrt();
    q.map(|v| v * r)
}

/// `FUN_002dae58`: the key at or before `t`. The cached key is tried first (then its
/// neighbours) on `times[c] <= t < times[c+1]`; else `lower_bound(times, t) - 1`
/// (`FUN_002dbfa0` is a lower bound), clamped to the last key [M]. Bit-3 channels go through
/// `FUN_002bfb50` instead (not ported: `None`).
pub fn key_index(ch: &TrackChannel, t: f32, cache: usize) -> Option<usize> {
    if ch.flags & 8 != 0 || ch.times.is_empty() { return None; }
    let n = ch.times.len();
    let last = n - 1;
    let fits = |i: usize| i < last && ch.times[i] <= t && t < ch.times[i + 1];
    if cache <= last {
        // At `cache == last` the game compares against the word past the table; the binary
        // search below gives the same key for every t there, so the port skips that read.
        if fits(cache) { return Some(cache); }
        if fits(cache + 1) { return Some(cache + 1); }
        if cache > 0 && fits(cache - 1) { return Some(cache - 1); }
    }
    let lb = ch.times.partition_point(|&k| k < t);
    Some(if lb == 0 { 0 } else { (lb - 1).min(last) })
}

/// `FUN_002db080`: blend key `i` and `i + (i < last)` by `(t - t0) / (t1 - t0)` (0 when
/// `t0 >= t1`, `FUN_002dafb8`); bit 2 → one float, bit 1 → `nlerp`, else `lerp4`. Updates the
/// cache with the key index [M]. Bit-0 channels (`FUN_002db188`) are not ported.
pub fn channel_value(ch: &TrackChannel, t: f32, cache: &mut usize) -> Option<[f32; 4]> {
    if ch.flags & 1 != 0 { return None; }
    let i = key_index(ch, t, *cache)?;
    *cache = i;
    let j = i + (i < ch.times.len() - 1) as usize;
    let (t0, t1) = (ch.times[i], ch.times[j]);
    let f = if t0 < t1 { (t - t0) / (t1 - t0) } else { 0.0 };
    let (a, b) = (ch.keys[i], ch.keys[j]);
    Some(if ch.flags & 4 != 0 {
        let v = a[0] + (b[0] - a[0]) * f;
        [v; 4]
    } else if ch.flags & 2 != 0 { nlerp(f, a, b) } else { lerp4(f, a, b) })
}

/// The per-channel key caches a follower keeps (+0xb0 translation, +0xb4 rotation) [M].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyCache {
    pub pos: usize,
    pub rot: usize,
}

/// `FUN_002db370`: the track's pose at `t`, the rotation channel's quaternion through
/// `quat_rows`, then the translation key (four lanes, w included) as row 3 [M]. `None` when
/// a channel is missing or uses an unported mode.
pub fn track_pose(tr: &PathTrack, t: f32, cache: &mut KeyCache) -> Option<Mat4> {
    let p = channel_value(&tr.channels[tr.pos?], t, &mut cache.pos)?;
    let q = channel_value(&tr.channels[tr.rot?], t, &mut cache.rot)?;
    let mut m = quat_rows(q);
    m[3] = p;
    Some(m)
}

/// `FUN_001b0f48(tick)`: the follower clock in 60 Hz frames, `tick * (1/fps) * 60`
/// (`DAT_00333ff4 = 1.0 / fps`, `FUN_001d1ec0`; the callers pass `(float)DAT_00339f54`, the
/// frame counter) [M].
pub fn clock60(tick: i32, fps: i32) -> f32 {
    let inv = 1.0 / fps as f32;
    tick as f32 * inv * 60.0
}

/// The path follower (the 0xd4-byte object `FUN_001b1110` builds; the spline controller
/// `FUN_0013da80` owns one at +0x60) [M].
#[derive(Clone, Debug, PartialEq)]
pub struct PathFollower {
    /// +0x98: the track being played.
    pub current: Option<u16>,
    /// +0x9c: the track taken at the wrap (`FUN_001b1258` sets it to the current one, so a
    /// track loops; `ESetNextAISpline` `FUN_00158370` → `FUN_0013dd28` → `FUN_001b1838`
    /// replaces it).
    pub next: Option<u16>,
    /// +0x94: the clock at the last update.
    pub last: f32,
    /// +0xa4: the element rate.
    pub rate: f32,
    /// +0xa8: the last advance.
    pub advance: f32,
    /// +0xac: the time along the current track (60 Hz frames).
    pub time: f32,
    /// +0xbc: the speed, +0xc0 its step per update, +0xc4 the desired speed.
    pub speed: f32,
    pub speed_step: f32,
    pub desired: f32,
    /// +0xc8: updates left before the speed starts moving (`EPathChangeThrottleStatic`).
    pub hold: i32,
    /// +0xcc: playing.
    pub playing: bool,
    /// +0x48: the base matrix.
    pub base: Mat4,
    /// The target object's matrix (the follower writes it).
    pub pose: Mat4,
    /// +0x88: the velocity in units per second.
    pub velocity: [f32; 3],
    pub cache: KeyCache,
}

impl PathFollower {
    /// `FUN_0013da80` + `FUN_001b1c20` + `FUN_001b1258` for an `AISp` track: element time 0,
    /// rate 0.5, base `BASE_TURN` (+0x5c == 0), playing (+0x44 bit 15 set by the ctor); speed,
    /// step and desired 1.0, hold 0; the pose evaluated at the start time, velocity zero [M].
    pub fn start(track: u16, tracks: &BTreeMap<u16, PathTrack>, now: f32) -> PathFollower {
        let mut f = PathFollower {
            current: Some(track), next: Some(track), last: now, rate: CONTROLLER_RATE, advance: 0.0, time: 0.0,
            speed: 1.0, speed_step: 1.0, desired: 1.0, hold: 0, playing: true, base: BASE_TURN, pose: IDENTITY,
            velocity: [0.0; 3], cache: KeyCache::default(),
        };
        if let Some(m) = tracks.get(&track).and_then(|t| track_pose(t, f.time, &mut f.cache)) {
            f.pose = mat_mul(&f.base, &m);
        }
        f
    }

    /// `EPathSetDesiredThrottleStatic` `FUN_00152cb0` → `FUN_001b1868` (+0xc4) [M].
    pub fn set_desired(&mut self, v: f32) { self.desired = v; }

    /// `EPathChangeThrottleStatic` `FUN_00151dc0`: hold (+0xc8), step (+0xc0), desired
    /// (+0xc4), playing (+0xcc = 1) [M].
    pub fn change_throttle(&mut self, hold: i32, step: f32, desired: f32) {
        self.hold = hold;
        self.speed_step = step;
        self.desired = desired;
        self.playing = true;
    }

    /// `EPathSetTimeStatic` `FUN_00153340` → `FUN_001b1880`: time = duration * fraction, only
    /// while not linked to another follower (+0xa0 == 0; this port never links) [M].
    pub fn set_time_fraction(&mut self, frac: f32, tracks: &BTreeMap<u16, PathTrack>) {
        if let Some(t) = self.current.and_then(|c| tracks.get(&c)) { self.time = t.duration * frac; }
    }

    /// `FUN_001b14b8` with `now = clock60(tick, fps)`: returns true when the time wrapped [M].
    pub fn update(&mut self, now: f32, fps: i32, tracks: &BTreeMap<u16, PathTrack>) -> bool {
        if now - self.last <= 0.0 { return false; }
        if !self.playing { self.last = now; return false; }
        let Some(cur) = self.current else { self.current = self.next; self.last = now; return false; };
        if self.hold == 0 {
            let (s, d) = (self.speed, self.desired);
            if s != d {
                if s < d {
                    self.speed = if self.speed_step <= d - s { s + self.speed_step } else { d };
                } else {
                    self.speed = if s - d < self.speed_step { d } else { s - self.speed_step };
                }
            } else if s == 0.0 {
                self.playing = false;
            }
        } else {
            self.hold -= 1;
        }
        self.advance = (now - self.last) * self.speed * self.rate;
        self.time += self.advance;
        let dur = |id: u16| tracks.get(&id).map_or(0.0, |t| t.duration);
        let mut wrapped = false;
        let mut cur = cur;
        if dur(cur) < self.time {
            wrapped = true;
            self.time -= dur(cur);
        } else if self.time < 0.0 {
            wrapped = true;
        }
        if wrapped {
            match self.next {
                None => {
                    self.time = if self.time < 0.0 { 0.0 } else { dur(cur) };
                    self.speed = 0.0;
                    self.playing = false;
                }
                Some(n) => {
                    let t = self.time;
                    self.current = Some(n);
                    cur = n;
                    if t < 0.0 { self.time = t + dur(n); } else if dur(n) < t { self.time = dur(n); }
                }
            }
        }
        if let Some(m) = tracks.get(&cur).and_then(|t| track_pose(t, self.time, &mut self.cache)) {
            let m = mat_mul(&self.base, &m);
            if !wrapped {
                let fps = fps as f32;
                self.velocity = [0, 1, 2].map(|i| (m[3][i] - self.pose[3][i]) * fps);
            }
            self.pose = m;
        }
        self.last = now;
        wrapped
    }
}

// ---------------------------------------------------------------------------------------------
// Triggers

/// The class mask a body's +0x6d class selects in the trigger walk `FUN_0022d240`: 1 → 4,
/// 2 → 8, anything else → 0x40 [M]. The body ctor `FUN_001f6e98` stores the class its callers
/// derive from an index (`< 0` → 2, 0/1 → 1, 2 → 2, 3 → 3) [M 0x185e50 / 0x1862c0]; that
/// index 0/1 are the human players is [S].
pub fn class_mask(class: u8) -> u16 {
    match class { 1 => 4, 2 => 8, _ => 0x40 }
}

/// A body as the trigger and shot tests read it: +0x10 position, +0x20 velocity, +0x7c the
/// radius (`FUN_001f6e98`: the length of the half-extent vector) [M].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub radius: f32,
    pub class: u8,
}

/// The trigger's shape byte (+0x11) [M `FUN_0022eb28`]: 2 sphere, 3 upright cylinder, 1 box.
pub fn trigger_shape(t: &TriggerRecord) -> u8 { t.raw[0x11] }

fn raw_f32(t: &TriggerRecord, o: usize) -> f32 { f32::from_le_bytes(t.raw[o..o + 4].try_into().unwrap()) }

/// `FUN_0022ca68`: flag 0x800 makes a trigger one-way, `dot(+0x30, body velocity) >= 0`
/// (three lanes) [M].
fn facing(t: &TriggerRecord, vel: [f32; 3]) -> bool {
    raw_f32(t, 0x30) * vel[0] + raw_f32(t, 0x34) * vel[1] + raw_f32(t, 0x38) * vel[2] >= 0.0
}

/// `FUN_0022eb28`: does the body touch the trigger? The reach is `(body radius + trigger
/// radius)²` against the squared distance, three lanes for a sphere, x and z for the other
/// shapes (decoded from the VU words), strictly inside. Flag 0x800 adds the one-way test.
/// A cylinder (3) then needs `body.y - r < trigger.y + height` and `body.y + r >= trigger.y`
/// (height = +0x14). A box (1) runs a box-box test `FUN_001f5eb0` (not ported: `None`) [M].
pub fn trigger_touches(t: &TriggerRecord, b: &Body) -> Option<bool> {
    let reach = (b.radius + t.radius) * (b.radius + t.radius);
    let d = [0, 1, 2].map(|i| t.pos[i] - b.pos[i]);
    let one_way = t.flags & 0x800 != 0;
    match trigger_shape(t) {
        2 => {
            if reach <= d[0] * d[0] + d[1] * d[1] + d[2] * d[2] { return Some(false); }
            Some(!one_way || facing(t, b.vel))
        }
        3 => {
            if reach <= d[0] * d[0] + d[2] * d[2] { return Some(false); }
            if one_way && !facing(t, b.vel) { return Some(false); }
            if b.pos[1] + b.radius < t.pos[1] { return Some(false); }
            Some(b.pos[1] - b.radius < t.pos[1] + raw_f32(t, 0x14))
        }
        _ => None,
    }
}

/// The trigger table's runtime flags (`DAT_0033aa88`) [M].
#[derive(Clone, Debug, PartialEq)]
pub struct Triggers {
    pub flags: Vec<u16>,
    /// +0x08: 1 at load (`FUN_0022cba0`), cleared by `EPlayerLose` / `EPlayerWin`; flag 0x400
    /// triggers need it [M 0x22d240, 0x154cb0, 0x1551b8].
    pub mission_live: bool,
}

impl Triggers {
    /// `FUN_0022cbe0`: copy the records; flag 0x200 clears 0x400 [M].
    pub fn load(records: &[TriggerRecord]) -> Triggers {
        Triggers {
            flags: records.iter().map(|t| if t.flags & 0x200 != 0 { t.flags & !0x400 } else { t.flags }).collect(),
            mission_live: true,
        }
    }

    /// The gate in `FUN_0022d240`: bit 0 on, bit 0x2000 off, 0x400 only while the mission is
    /// live, and the body's class mask [M].
    pub fn armed(&self, i: usize, class: u8) -> bool {
        let f = self.flags[i];
        f & 1 != 0 && f & 0x2000 == 0 && (f & 0x400 == 0 || self.mission_live) && f & class_mask(class) != 0
    }

    /// One trigger against one body: the gate, the contact test, then the fire's tail
    /// (`FUN_0022c890`: flag 2 clears bit 0). Returns true when the trigger fires (its list
    /// runs) [M].
    pub fn touch(&mut self, i: usize, rec: &TriggerRecord, b: &Body) -> bool {
        if !self.armed(i, b.class) || trigger_touches(rec, b) != Some(true) { return false; }
        if self.flags[i] & 2 != 0 { self.flags[i] &= !1; }
        true
    }

    /// `EEnableTrigger` `FUN_0014bcd8` (bit 0 set) / `EDisableTrigger` `FUN_0014b790` (cleared).
    pub fn enable(&mut self, i: usize, on: bool) {
        if let Some(f) = self.flags.get_mut(i) { if on { *f |= 1 } else { *f &= !1 } }
    }
}

// ---------------------------------------------------------------------------------------------
// Mission timers and the rule that ends the phase

/// One of the ten 0x18-byte timers at weapon manager `DAT_0033a024`+0x150 [M `FUN_00209c88`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MissionTimer {
    pub running: bool,
    pub start: i32,
    pub base: i32,
    pub rate: i32,
}

impl MissionTimer {
    /// `FUN_00209c88`: `base + rate * (now - start)` while running, else `base` (ticks).
    pub fn value(&self, now: i32) -> i32 {
        if self.running { self.base + self.rate * (now - self.start) } else { self.base }
    }

    /// `ESetTimer` (kind 47): the ctor `FUN_00159178` calls `FUN_00209bf0(timer,
    /// (int)(seconds * fps + 1.0))`, base and start = now [M].
    pub fn set_seconds(&mut self, seconds: f32, fps: i32, now: i32) {
        self.base = (seconds * fps as f32 + 1.0) as i32;
        self.start = now;
    }

    /// Kind 48 (ctor `FUN_0015ca58` → `FUN_00209c08`): when stopped, rate and start = now,
    /// running [M]. MIS01 starts timer 5 with rate -1 (a count-down) [D].
    pub fn start_counting(&mut self, rate: i32, now: i32) {
        if !self.running { self.rate = rate; self.running = true; self.start = now; }
    }
}

/// Rule type 9 (typeinfo "10SRuleSpeed" at 0x397cc8 through the descriptor 0x397aa8): despite
/// the name it watches a mission timer. Ctor `FUN_0020a858`: timer = payload +0x00, rising =
/// payload +0x04 != 0, threshold = payload +0x08; the previous value starts at threshold - 1
/// (falling) or + 1 (rising) [M].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimerRule {
    pub timer: usize,
    pub rising: bool,
    pub threshold: f32,
    pub prev: f32,
}

impl TimerRule {
    pub fn from_payload(p: &[u8; 16]) -> TimerRule {
        let w = |o: usize| u32::from_le_bytes(p[o..o + 4].try_into().unwrap());
        let threshold = f32::from_bits(w(8));
        let rising = w(4) != 0;
        TimerRule { timer: w(0) as usize, rising, threshold, prev: if rising { threshold + 1.0 } else { threshold - 1.0 } }
    }

    /// `FUN_0020a940` (descriptor +0x14): nothing while the timer is stopped; v = ticks / fps;
    /// falling: fires when `v < threshold` and the previous value was `>= threshold`; rising:
    /// fires when `v > threshold` and the previous was `<= threshold`. The value is stored on
    /// every running evaluation [M].
    pub fn eval(&mut self, timers: &[MissionTimer], now: i32, fps: i32) -> bool {
        let Some(t) = timers.get(self.timer) else { return false };
        if !t.running { return false; }
        let v = t.value(now) as f32 / fps as f32;
        let fire = if !self.rising {
            if self.threshold <= v { self.prev = v; return false; }
            self.threshold <= self.prev
        } else {
            if v <= self.threshold { self.prev = v; return false; }
            self.prev <= self.threshold
        };
        self.prev = v;
        fire
    }
}

// ---------------------------------------------------------------------------------------------
// The gunner's view

/// One auto-drive camera arm's view limits, as `FUN_001a49e0` loads them from camera.ini
/// (`[<cam>:Arm<k>]`): `maxYaw`/`minYaw` (turns, clamped to ±0.5) and `maxPitch`/`minPitch`
/// (sines, clamped to ±0.999) for the base set and the sets 1..4 (`maxYaw%d`, …), at arm
/// +0xc0 / +0xac / +0xe8 / +0xd4 [M]. Absent keys read 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewLimits {
    pub min_yaw: [f32; 5],
    pub max_yaw: [f32; 5],
    pub min_pitch: [f32; 5],
    pub max_pitch: [f32; 5],
}

/// The other arm keys the loader reads, with its conversions [M `FUN_001a49e0`]: deadzones
/// and autoaim cones × 1/(2π) (radians → turns), cursor speeds `NormCursor*MoveSpeed` × π/2.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimArm {
    pub limits: ViewLimits,
    /// +0x58 `aimFov`, +0x5c `minFov` (degrees).
    pub aim_fov: f32,
    pub min_fov: f32,
    /// +0x40 `maxDeadzonePitch`, +0x44 `maxDeadzoneYaw` (turns).
    pub deadzone_pitch: f32,
    pub deadzone_yaw: f32,
    /// +0x48 `maxAutoaimPitch`, +0x4c `maxAutoaimYaw` (turns), +0x50 `autoaimInterpolSpeed`.
    pub autoaim_pitch: f32,
    pub autoaim_yaw: f32,
    pub autoaim_speed: f32,
    /// +0x7c / +0x80 (× π/2), +0x84 / +0x88 (as read).
    pub norm_cursor: f32,
    pub norm_cursor_end: f32,
    pub target_cursor: f32,
    pub target_cursor_end: f32,
    /// +0x54 `AllowDeadzone` (default 1), +0x68 `LockArmToCar` (default 0).
    pub allow_deadzone: bool,
    pub lock_to_car: bool,
}

const INV_TAU: f32 = 0.159_154_95;

impl AimArm {
    /// Reads `[section]` (e.g. `CopterAutoDriveCam:Arm0`) with the loader's defaults.
    pub fn from_ini(text: &str, section: &str) -> AimArm {
        let g = |k: &str, d: f32| super::camera::ini_f32(text, section, k).unwrap_or(d);
        let clamp = |v: f32, lo: f32, hi: f32| { let v = if v < lo { lo } else { v }; if hi < v { hi } else { v } };
        let mut l = ViewLimits { min_yaw: [0.0; 5], max_yaw: [0.0; 5], min_pitch: [0.0; 5], max_pitch: [0.0; 5] };
        for i in 0..5 {
            let k = |n: &str| if i == 0 { n.to_string() } else { format!("{n}{i}") };
            l.max_yaw[i] = clamp(g(&k("maxYaw"), 0.0), -0.5, 0.5);
            l.min_yaw[i] = clamp(g(&k("minYaw"), 0.0), -0.5, 0.5);
            l.max_pitch[i] = clamp(g(&k("maxPitch"), 0.0), -0.999, 0.999);
            l.min_pitch[i] = clamp(g(&k("minPitch"), 0.0), -0.999, 0.999);
        }
        let half_pi = 0.5 * 3.141_592_5;
        AimArm {
            limits: l,
            aim_fov: g("aimFov", 0.0),
            min_fov: g("minFov", 0.0),
            deadzone_pitch: g("maxDeadzonePitch", 0.1) * INV_TAU,
            deadzone_yaw: g("maxDeadzoneYaw", 0.1) * INV_TAU,
            autoaim_pitch: g("maxAutoaimPitch", 0.1) * INV_TAU,
            autoaim_yaw: g("maxAutoaimYaw", 0.1) * INV_TAU,
            autoaim_speed: g("autoaimInterpolSpeed", 1.0),
            norm_cursor: g("NormCursorMoveSpeed", 1.0) * 3.141_592_5 * 0.5,
            norm_cursor_end: g("NormCursorEndMoveSpeed", 1.0) * half_pi,
            target_cursor: g("TargetCursorMoveSpeed", 1.0),
            target_cursor_end: g("TargetCursorEndMoveSpeed", 1.0),
            allow_deadzone: g("AllowDeadzone", 1.0) != 0.0,
            lock_to_car: g("LockArmToCar", 0.0) != 0.0,
        }
    }
}

/// `FUN_001b25c0`: `asin(clamp(x, -1, 1))` in turns [M].
fn asin_turns(x: f32) -> f32 { x.clamp(-1.0, 1.0).asin() * INV_TAU }

fn clamp01(v: f32) -> f32 { let v = if 1.0 < v { 1.0 } else { v }; if v < 0.0 { 0.0 } else { v } }

impl ViewLimits {
    /// `FUN_001b6c60(cam, arm, &pitch, &yaw)`: keeps the view inside the arm's limit polygon
    /// (pitch range interpolated between the yaw sets), returns true when a limit was hit
    /// [M]. Angles in turns; the pitch bound of a set is `-asin(minPitch)` (upper) and
    /// `-asin(maxPitch)` (lower).
    pub fn clamp_view(&self, pitch: &mut f32, yaw: &mut f32) -> bool {
        let (mny, mxy, mnp, mxp) = (&self.min_yaw, &self.max_yaw, &self.min_pitch, &self.max_pitch);
        if mxp[0] == 0.0 && mnp[0] == 0.0 { return false; }
        let yaw_lim = !(mxy[0] == 0.0 && mny[0] == 0.0);
        let mut mid = 0.0f32;
        let mut i = 0usize;
        // The search for the yaw set; `None` = ran past set 4 (LAB_001b7134).
        let found: Option<usize> = 'search: {
            if mxy[0] != 0.0 {
                loop {
                    if yaw_lim && mny[i] == mxy[i] {
                        if i == 0 && *yaw < mxy[0] { break 'search Some(0); }
                        if i > 0 && mny[i - 1] <= *yaw && *yaw < mxy[i] { break 'search Some(i); }
                    } else {
                        mid = (mny[i] + mxy[i]) * 0.5;
                        let inside = if *yaw < mid { mny[i] < *yaw } else { *yaw < mxy[i] };
                        if inside { break 'search Some(i); }
                    }
                    i += 1;
                    if i > 4 { break 'search None; }
                    if mxy[i] == 0.0 { break; }
                }
                i -= (i > 0) as usize;
            }
            Some(i)
        };
        let (mut up, mut lo, ymax, ymin);
        match found {
            None => {
                ymax = mxy[4];
                ymin = mny[4];
                lo = -asin_turns(mxp[4]);
                up = -asin_turns(mnp[4]);
            }
            Some(i) => {
                // Set i-1's bounds (index -1 reads the neighbouring table in the game; those
                // values are never used for i = 0).
                let prev = |a: &[f32; 5], b: &[f32; 5]| if i > 0 { a[i - 1] } else { b[4] };
                up = -asin_turns(mnp[i]);
                let up_p = -asin_turns(prev(mnp, mxy));
                lo = -asin_turns(mxp[i]);
                let lo_p = -asin_turns(prev(mxp, mnp));
                if yaw_lim && mny[i] == mxy[i] {
                    if i < 1 {
                        ymin = mny[i];
                        ymax = mxy[i];
                    } else {
                        let y0 = mny[i - 1];
                        let span = mxy[i] - y0;
                        let f = if span != 0.0 { clamp01((*yaw - y0) / span) } else { span };
                        let dup = up - up_p;
                        let dlo = lo - lo_p;
                        up = up_p + f * dup;
                        lo = lo_p + f * dlo;
                        if up < *pitch && span.abs() < dup.abs() {
                            let g = if dup != 0.0 { clamp01((*pitch - up_p) / dup) } else { dup };
                            up = up_p + g * dup;
                            *yaw = y0 + g * (mxy[i] - y0);
                        }
                        let p = *pitch;
                        if p < lo && span.abs() < dlo.abs() {
                            let g = if dlo != 0.0 { clamp01((p - lo_p) / dlo) } else { dlo };
                            lo = lo_p + g * dlo;
                            *yaw = y0 + g * (mxy[i] - y0);
                        }
                        ymin = y0;
                        ymax = mxy[i];
                    }
                } else {
                    ymax = mxy[i];
                    ymin = mny[i];
                    if i > 0 {
                        let (from, span) = if *yaw < mid { (mny[i - 1], mny[i] - mny[i - 1]) } else { (mxy[i - 1], ymax - mxy[i - 1]) };
                        let f = if span != 0.0 { clamp01((*yaw - from) / span) } else { span };
                        lo = lo_p + f * (lo - lo_p);
                        up = up_p + f * (up - up_p);
                    }
                }
            }
        }
        let mut hit = false;
        if yaw_lim {
            if 0.5 <= ymax && ymax < *yaw { *yaw -= 1.0; }
            if ymin <= -0.5 && *yaw < ymin { *yaw += 1.0; }
            if ymax <= *yaw { *yaw = ymax; hit = true; }
            if *yaw <= ymin { *yaw = ymin; hit = true; }
        } else if 1.0 < *yaw {
            *yaw -= 1.0;
        } else if *yaw < -1.0 {
            *yaw += 1.0;
        }
        if up <= *pitch { *pitch = up; hit = true; }
        if *pitch <= lo { *pitch = lo; hit = true; }
        hit
    }
}

/// The scope FOV step of `FUN_001b72f0` (camera +0x298 the target FOV, +0xa4 the zoomed FOV,
/// +0x2ac the zoom rate): unscoped → the camera's default FOV; scoped → at least `minFov`;
/// a non-zero rate moves +0xa4 by it within `[minFov, default]`, stored only above 2.0 [M].
/// Returns the FOV handed to `FUN_001b8740`. What writes the rate is not traced.
pub fn scope_fov(scoped: bool, target: &mut f32, zoomed: &mut f32, rate: f32, default_fov: f32, min_fov: f32) -> f32 {
    if !scoped { *target = default_fov; } else if *target < min_fov { *target = min_fov; }
    if rate == 0.0 { return *target; }
    let f = *zoomed + rate;
    let f = if default_fov < f { default_fov } else { f };
    let f = if min_fov < f { f } else { min_fov };
    if 2.0 < f { *zoomed = f; }
    *target = *zoomed;
    *target
}

/// Which gunner kinds the gunner matrix `FUN_00124848` moves: kinds 1-2 lag behind the
/// vehicle in POV mode (rates dt*0.5 and dt*7, y clamped to ±0.1333, scale 0.15) and kinds
/// 1/4/6 jitter while firing (state 0xb). The sniper (kind 3) gets neither: its matrix is the
/// camera's [M].
pub fn gunner_sway(kind: i64) -> (bool, bool) {
    (matches!(kind, 1 | 2), matches!(kind, 1 | 4 | 6))
}

// ---------------------------------------------------------------------------------------------
// The shot

/// The Sniper Rifle's projectile speed argument (case 0x1a of `FUN_0014ca60`: 0x43c80000 =
/// 400) and the cap `FUN_00198e80` applies (`< 220` kept, else 220) [M].
pub const SNIPER_SPEED_ARG: f32 = 400.0;
pub fn shot_speed(arg: f32) -> f32 { if arg < 220.0 { arg } else { 220.0 } }

/// The range `FUN_00198e80` uses when the vehicle pass found nothing (150) [M].
pub const NO_HIT_RANGE: f32 = 150.0;

/// `FUN_00211098`: the ray against a sphere, no hit when the origin is inside; else the
/// point of the segment `origin .. origin + normalize(dir) * 1000` nearest the centre, a hit
/// when that point is within the radius (inclusive). Returns that point [M].
pub fn ray_sphere(radius: f32, origin: [f32; 3], dir: [f32; 3], center: [f32; 3]) -> Option<[f32; 3]> {
    let d = [0, 1, 2].map(|i| center[i] - origin[i]);
    if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < radius * radius { return None; }
    let inv = 1.0 / (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let seg = [0, 1, 2].map(|i| origin[i] + dir[i] * inv * 1000.0 - origin[i]);
    let l2 = seg[0] * seg[0] + seg[1] * seg[1] + seg[2] * seg[2];
    let il = if l2 != 0.0 { 1.0 / l2 } else { 0.0 };
    let t = (d[0] * seg[0] + d[1] * seg[1] + d[2] * seg[2]) * il;
    let t = if t < 1.0 { t } else { 1.0 };
    let t = if 0.0 < t { t } else { 0.0 };
    let p = [0, 1, 2].map(|i| origin[i] + t * seg[i]);
    let e = [0, 1, 2].map(|i| p[i] - center[i]);
    (e[0] * e[0] + e[1] * e[1] + e[2] * e[2] <= radius * radius).then_some(p)
}

/// A shot target for the vehicle pass of `FUN_001998c0`: the body centre (+0x10) and radius
/// (+0x7c) [M `FUN_00195808` / `FUN_001956d8`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotTarget {
    pub id: usize,
    pub center: [f32; 3],
    pub radius: f32,
}

/// The nearest sphere hit of a shot (`FUN_001998c0` sphere stage + the first branch of
/// `FUN_0019a548`: the first hit is kept, a nearer one replaces it; the distance is from the
/// origin to the ray point nearest the target's centre) [M]. The vehicle box refinement and
/// the second-hit slot are not ported.
pub fn nearest_hit(origin: [f32; 3], dir: [f32; 3], targets: &[ShotTarget]) -> Option<(usize, [f32; 3], f32)> {
    let mut best: Option<(usize, [f32; 3], f32)> = None;
    for t in targets {
        let Some(p) = ray_sphere(t.radius, origin, dir, t.center) else { continue };
        let e = [0, 1, 2].map(|i| p[i] - origin[i]);
        let dist = (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt();
        if best.map_or(true, |b| dist < b.2) { best = Some((t.id, p, dist)); }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(flags: u32, times: &[f32], keys: &[[f32; 4]]) -> TrackChannel {
        TrackChannel { flags, name: String::new(), times: times.to_vec(), keys: keys.to_vec() }
    }

    #[test]
    fn quaternion_rows_and_the_base_turn() {
        // A quarter turn about +y: q = (0, sin 45, 0, cos 45).
        let s = std::f32::consts::FRAC_1_SQRT_2;
        let m = quat_rows([0.0, s, 0.0, s]);
        // row0 = (1-2yy, 0, -2yw) = (0, 0, -1); a transposed matrix would give (0, 0, 1).
        assert!((m[0][0]).abs() < 1e-6 && (m[0][2] + 1.0).abs() < 1e-6, "{:?}", m[0]);
        assert!((m[2][0] - 1.0).abs() < 1e-6, "{:?}", m[2]);
        // BASE_TURN · M: row0 = -M.row2, row2 = M.row0, the position row untouched.
        let mut p = quat_rows([0.1, 0.2, 0.3, 0.927_361_8]);
        p[3] = [5.0, 6.0, 7.0, 1.0];
        let o = mat_mul(&BASE_TURN, &p);
        assert_eq!(o[0], p[2].map(|v| -v + 0.0));
        assert_eq!(o[2], p[0]);
        assert_eq!(o[3], [5.0, 6.0, 7.0, 1.0]);
    }

    #[test]
    fn key_search_and_blends() {
        let c = ch(0, &[0.0, 10.0, 20.0], &[[0.0; 4], [10.0, 0.0, 0.0, 1.0], [30.0, 0.0, 0.0, 1.0]]);
        let mut k = 0;
        assert_eq!(channel_value(&c, 15.0, &mut k).unwrap()[0], 20.0);
        assert_eq!(k, 1);
        // On a key the cache keeps the interval that starts there (fraction 0); past the end
        // the last key holds.
        let mut k = 1;
        assert_eq!(channel_value(&c, 10.0, &mut k).unwrap()[0], 10.0);
        assert_eq!(k, 1);
        assert_eq!(channel_value(&c, 99.0, &mut k).unwrap()[0], 30.0);
        assert_eq!(k, 2);
        // Without a matching cache the lower bound minus one is taken: t = 10 lands on key 0
        // with fraction 1.
        assert_eq!(key_index(&c, 10.0, 5), Some(0));
        // Quaternions take the short way: b = -a blends to ±a, never through zero.
        let a = [0.0, 0.0, 0.0, 1.0];
        let q = nlerp(0.5, a, [0.0, 0.0, 0.0, -1.0]);
        assert!((q[3].abs() - 1.0).abs() < 1e-6, "{q:?}");
        // Bit 0 and bit 3 channels are not ported.
        assert_eq!(channel_value(&ch(1, &[0.0], &[[0.0; 4]]), 0.0, &mut 0), None);
        assert_eq!(key_index(&ch(8, &[0.0], &[[0.0; 4]]), 0.0, 0), None);
    }

    fn line_track(dur: f32) -> PathTrack {
        PathTrack {
            duration: dur, rot: Some(0), pos: Some(1), name: "t".into(),
            channels: vec![
                ch(2, &[0.0, dur], &[[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]]),
                ch(0, &[0.0, dur], &[[0.0, 0.0, 0.0, 1.0], [dur, 0.0, 0.0, 1.0]]),
            ],
        }
    }

    #[test]
    fn the_follower_advances_at_half_rate_and_wraps_into_the_next_track() {
        let mut tracks = BTreeMap::new();
        tracks.insert(1, line_track(100.0));
        tracks.insert(2, line_track(50.0));
        let mut f = PathFollower::start(1, &tracks, clock60(0, 60));
        f.set_desired(0.71);
        // The step (1.0) covers 1.0 → 0.71 at once: 0.355 frames per frame.
        f.update(clock60(1, 60), 60, &tracks);
        assert_eq!(f.speed, 0.71);
        assert!((f.time - 0.355).abs() < 1e-6, "{}", f.time);
        assert!((f.pose[3][0] - 0.355).abs() < 1e-5);
        assert!((f.velocity[0] - 0.355 * 60.0).abs() < 1e-3, "{:?}", f.velocity);
        // A slow change: step 0.01 toward 0, after a 2-update hold.
        f.change_throttle(2, 0.01, 0.0);
        f.update(clock60(2, 60), 60, &tracks);
        f.update(clock60(3, 60), 60, &tracks);
        assert_eq!((f.hold, f.speed), (0, 0.71));
        f.update(clock60(4, 60), 60, &tracks);
        assert!((f.speed - 0.70).abs() < 1e-6);
        // Wrap: the track loops unless a next one is set.
        f.change_throttle(0, 1.0, 1.0);
        f.next = Some(2);
        let mut n = 5;
        while f.current == Some(1) { f.update(clock60(n, 60), 60, &tracks); n += 1; assert!(n < 400); }
        assert_eq!(f.current, Some(2));
        assert!(f.time < 1.0, "{}", f.time);
        // Desired 0 reached → the next update stops it.
        f.change_throttle(0, 1.0, 0.0);
        f.update(clock60(n, 60), 60, &tracks);
        f.update(clock60(n + 1, 60), 60, &tracks);
        assert!(!f.playing);
    }

    fn trig(shape: u8, pos: [f32; 3], radius: f32, flags: u16, height: f32) -> TriggerRecord {
        let mut r = [0u8; 0x40];
        for (i, v) in pos.iter().enumerate() { r[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes()); }
        r[0xc..0x10].copy_from_slice(&radius.to_le_bytes());
        r[0x11] = shape;
        r[0x12..0x14].copy_from_slice(&flags.to_le_bytes());
        r[0x14..0x18].copy_from_slice(&height.to_le_bytes());
        TriggerRecord::parse(&r).unwrap()
    }

    #[test]
    fn trigger_shapes_gates_and_one_shot() {
        let b = |pos| Body { pos, vel: [0.0; 3], radius: 1.0, class: 1 };
        // Sphere: 3-D reach, strictly inside.
        let s = trig(2, [0.0, 0.0, 0.0], 2.5, 0x407, 0.0);
        assert_eq!(trigger_touches(&s, &b([3.4, 0.0, 0.0])), Some(true));
        assert_eq!(trigger_touches(&s, &b([3.5, 0.0, 0.0])), Some(false));
        assert_eq!(trigger_touches(&s, &b([0.0, 3.4, 0.0])), Some(true));
        assert_eq!(trigger_touches(&s, &b([0.0, 3.6, 0.0])), Some(false), "a sphere measures y too");
        // Cylinder: x/z reach, then the height band [y, y + h].
        let c = trig(3, [0.0, 0.0, 0.0], 2.5, 0x407, 1.0);
        assert_eq!(trigger_touches(&c, &b([0.0, 1.9, 3.0])), Some(true));
        assert_eq!(trigger_touches(&c, &b([0.0, 2.0, 0.0])), Some(false));
        assert_eq!(trigger_touches(&c, &b([0.0, -1.1, 0.0])), Some(false));
        // Box: not ported.
        assert_eq!(trigger_touches(&trig(1, [0.0; 3], 2.5, 0x407, 1.0), &b([0.0; 3])), None);
        // Gates: class 1 needs mask 4; 0x400 needs the mission live; bit 1 makes it one-shot.
        let recs = vec![s.clone(), trig(2, [0.0; 3], 2.5, 0x40a, 0.0), trig(2, [0.0; 3], 2.5, 0x605, 0.0)];
        let mut t = Triggers::load(&recs);
        assert_eq!(t.flags[2], 0x205, "0x200 clears 0x400 at load");
        assert!(t.touch(0, &recs[0], &b([0.0; 3])));
        assert!(!t.touch(0, &recs[0], &b([0.0; 3])), "one-shot");
        assert!(!t.touch(1, &recs[1], &b([0.0; 3])), "bit 0 off, and mask 8 only");
        t.enable(1, true);
        assert!(!t.touch(1, &recs[1], &b([0.0; 3])), "class 1 does not match mask 8");
        assert!(t.touch(1, &recs[1], &Body { class: 2, ..b([0.0; 3]) }));
        t.mission_live = false;
        t.enable(0, true);
        assert!(!t.armed(0, 1), "0x400 needs the mission live");
    }

    #[test]
    fn the_timer_rule_fires_on_the_crossing() {
        // ESetTimer(5, 4.0) + start(5, -1) at tick 1000: base 241 ticks.
        let mut timers = [MissionTimer::default(); 10];
        timers[5].set_seconds(4.0, 60, 1000);
        timers[5].start_counting(-1, 1000);
        assert_eq!(timers[5].base, 241);
        let mut p = [0u8; 16];
        p[0] = 5;
        p[8..12].copy_from_slice(&1.0f32.to_le_bytes());
        let mut r = TimerRule::from_payload(&p);
        assert_eq!((r.rising, r.prev), (false, 0.0));
        let fired: Vec<i32> = (1001..1400).filter(|&n| r.eval(&timers, n, 60)).collect();
        assert_eq!(fired, vec![1000 + 182], "v = (241 - n) / 60 first below 1.0 at n = 182");
        // A threshold above the start value never fires on a fall.
        p[8..12].copy_from_slice(&4.9f32.to_le_bytes());
        let mut r = TimerRule::from_payload(&p);
        assert!((1001..1400).all(|n| !r.eval(&timers, n, 60)));
        // A stopped timer does nothing.
        let stopped = [MissionTimer::default(); 10];
        assert!(!TimerRule::from_payload(&p).eval(&stopped, 5, 60));
    }

    fn copter() -> ViewLimits {
        let mut l = ViewLimits { min_yaw: [0.0; 5], max_yaw: [0.0; 5], min_pitch: [0.0; 5], max_pitch: [0.0; 5] };
        l.min_yaw[..3].copy_from_slice(&[-0.315, -0.09, -0.07]);
        l.max_yaw[..3].copy_from_slice(&[-0.315, -0.09, -0.07]);
        l.max_pitch[..3].copy_from_slice(&[0.57, 0.28, 0.28]);
        l.min_pitch[..3].copy_from_slice(&[-0.75, -0.53, 0.28]);
        l
    }

    #[test]
    fn the_copter_view_limits() {
        let l = copter();
        // Yaw below the first set: pinned to it, pitch inside set 0's band.
        let (mut p, mut y) = (0.0, -0.4);
        assert!(l.clamp_view(&mut p, &mut y));
        assert_eq!((p, y), (0.0, -0.315));
        // Past the last set, low: pinned to -0.07, where both pitch bounds are -asin(0.28).
        let (mut p, mut y) = (-0.2, 0.1);
        assert!(l.clamp_view(&mut p, &mut y));
        assert_eq!(y, -0.07);
        assert!((p - (-(0.28f32).asin() * INV_TAU)).abs() < 1e-6, "{p}");
        // Past the last set, high: the pitch edge between sets 1 and 2 is steeper than the
        // yaw span, so the view slides along it, the yaw comes back to about -0.0842 and the
        // pitch stays (on the edge).
        let (mut p, mut y) = (0.05, 0.1);
        assert!(l.clamp_view(&mut p, &mut y));
        assert!((y - -0.084_19).abs() < 1e-4, "{y}");
        assert!((p - 0.05).abs() < 1e-5, "{p}");
        // The script's start view (-60°, -20° → yaw -1/6, pitch +1/18 after FUN_001bb7c0's
        // negation) is inside: nothing moves.
        let (mut p, mut y) = (20.0 / 360.0, -60.0 / 360.0);
        assert!(!l.clamp_view(&mut p, &mut y));
        assert_eq!((p, y), (20.0 / 360.0, -60.0 / 360.0));
        // Halfway between sets 0 and 1 the upper pitch bound is the mean of the two.
        let (mut p, mut y) = (0.4, (-0.315 + -0.09) / 2.0);
        l.clamp_view(&mut p, &mut y);
        let up = (-(-0.75f32).asin() * INV_TAU + -(-0.53f32).asin() * INV_TAU) / 2.0;
        assert!((p - up).abs() < 2e-3, "{p} vs {up}");
    }

    #[test]
    fn scope_fov_and_sway() {
        let (mut t, mut z) = (33.0, 33.0);
        assert_eq!(scope_fov(false, &mut t, &mut z, 0.0, 33.0, 2.0), 33.0);
        assert_eq!(scope_fov(true, &mut t, &mut z, -10.0, 33.0, 2.0), 23.0);
        // The floor is minFov, and 2.0 itself is never stored.
        z = 3.0;
        assert_eq!(scope_fov(true, &mut t, &mut z, -5.0, 33.0, 2.0), 3.0);
        assert_eq!(gunner_sway(3), (false, false), "the sniper has no lag and no jitter");
        assert_eq!(gunner_sway(1), (true, true));
    }

    #[test]
    fn the_shot_test() {
        assert_eq!(shot_speed(SNIPER_SPEED_ARG), 220.0);
        let o = [0.0, 0.0, 0.0];
        let d = [0.0, 0.0, 2.0];
        assert_eq!(ray_sphere(1.0, o, d, [0.5, 0.0, 50.0]), Some([0.0, 0.0, 50.0]));
        assert_eq!(ray_sphere(1.0, o, d, [1.5, 0.0, 50.0]), None);
        assert_eq!(ray_sphere(1.0, o, d, [0.0, 0.0, -50.0]), None, "behind");
        assert_eq!(ray_sphere(1.0, o, d, [0.0, 0.0, 0.5]), None, "origin inside");
        assert_eq!(ray_sphere(1.0, o, d, [0.0, 0.0, 1000.5]), Some([0.0, 0.0, 1000.0]), "segment end");
        let targets = [
            ShotTarget { id: 7, center: [0.0, 0.5, 80.0], radius: 2.0 },
            ShotTarget { id: 9, center: [0.0, -0.5, 40.0], radius: 2.0 },
        ];
        let (id, p, dist) = nearest_hit(o, d, &targets).unwrap();
        assert_eq!((id, p, dist), (9, [0.0, 0.0, 40.0], 40.0));
    }
}
