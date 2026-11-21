//! The driving engine's particle effects: tyre smoke, surface spray (dirt, gravel, grass, water,
//! snow), wheel sparks, and the glare pulse. docs/systems/driving-effects.md is the write-up.
//!
//! Ported from DRIVING.ELF [M] (decompile + disassembly; EE = file offset + 0x107080):
//! - the particle pool (`RParticleSystemManager`, 0x100c0 bytes; the pool at +0x40): two
//!   0x8000-byte buffers of 1024 x 32-byte particles, a step copies the survivors of the current
//!   buffer into the other and swaps (`FUN_001f4878`, `FUN_001f3ca8`, `FUN_001f3c20`); new
//!   particles go into the other buffer too (`FUN_001f3bb8`);
//! - the frame driver `FUN_001f47f8`: min(tick delta, 3) steps, then the draw build;
//! - spawn `FUN_001f29d8`, the emitter `FUN_001e5e10` / `FUN_001e5fa0` / `FUN_001e5cd8` /
//!   `FUN_001e65c0`, the draw build `FUN_001f3dc8`;
//! - the vehicle effects `FUN_001ee238` (slots), `FUN_001ee4a0` (per wheel), `FUN_001eed18`
//!   (sparks);
//! - the glare pulse `FUN_001f1b60` and its size cull `FUN_001f1d18`.
//!
//! This is logic only: it produces per-tick draw items (atlas cell, centre, size, rotation,
//! colour, bucket) and does not draw. The quad's corner expansion happens in VU1 microcode, which
//! is not traced: [`ParticleDraw::Quad`] carries the values the CPU uploads, nothing more.

use std::collections::BTreeMap;

use nf_formats::drivefx::{self, ParticleType, FLAG_ADDITIVE, FLAG_BYTE2C, FLAG_FOREVER, FLAG_NIBBLE_ALPHA, FLAG_NIBBLE_SIZE, FLAG_STREAK};

/// The driving engine's shared random generator (state DAT_00333e60, multiplier DAT_00333e64)
/// [M `FUN_001c08c0` seeds it; every draw is `p = state*mul; r = (p & 0xffff00) >> 8;
/// state = p & 0xffff`, `FUN_001f29d8` and ~10 other callers]. The AI and other systems draw from
/// the same stream; this port gives the effects their own copy, so the order against those
/// callers is not modelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriveRng {
    pub state: u32,
}

pub const RNG_MUL: u32 = 0x075b_cd15;
pub const RNG_SEED: u32 = 0x3ade_68b1;

impl DriveRng {
    /// `FUN_001c08c0(n)`: state = 0x3ade68b1, then n % 500 masked steps [M].
    pub fn seeded(n: i32) -> DriveRng {
        let mut s = RNG_SEED;
        for _ in 0..(n % 500) { s = s.wrapping_mul(RNG_MUL) & 0xffff; }
        DriveRng { state: s }
    }

    /// One draw: 16 bits, the product's bits 8..23.
    pub fn bits(&mut self) -> u32 {
        let p = self.state.wrapping_mul(RNG_MUL);
        self.state = p & 0xffff;
        (p & 0x00ff_ff00) >> 8
    }

    /// `-v + 2*v*r` as the spawn writes it: `((x - v) + f) + f` with f = bits*v*2^-16 (the
    /// caller adds x).
    fn spread(&mut self, v: f32) -> f32 { self.bits() as f32 * v * 1.525_878_9e-5 }
}

/// One live particle, the 32-byte slot [M `FUN_001f29d8` writes it, `FUN_001f3ca8` steps it,
/// `FUN_001f3dc8` reads it].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Particle {
    /// +0x00..+0x08. World space, or relative to the attached object (`attach` != 0xff).
    pub pos: [f32; 3],
    /// +0x0c: steps left (counts down; the particle dies when it reaches 0).
    pub age: u8,
    /// +0x0d: the type's flag byte (bit 0x40 per `FLAG_BYTE2C`).
    pub flags: u8,
    /// +0x0e: gravity, -1/4096 units per step per step (positive falls).
    pub gravity: i8,
    /// +0x0f: the library slot.
    pub ty: u8,
    /// +0x10..+0x18: per-step velocity.
    pub vel: [f32; 3],
    /// +0x1c: alpha nibble (high) / size nibble (low).
    pub b1c: u8,
    /// +0x1d: drag*255 (written, not read by the traced code).
    pub drag: u8,
    /// +0x1e: attached object id, 0xff = none.
    pub attach: u8,
    /// +0x1f: spin * 40.7.
    pub spin: i8,
}

/// The particle library: up to 256 prepared types under four-character keys [M `FUN_001e7cf0`].
#[derive(Clone, Debug, Default)]
pub struct ParticleLib {
    pub types: Vec<ParticleType>,
    keys: BTreeMap<u32, usize>,
}

impl ParticleLib {
    /// Add a type (prepared on the way in, `FUN_001e8040`). Returns the slot, None when full.
    pub fn add(&mut self, key: u32, t: &ParticleType) -> Option<usize> {
        if self.types.len() >= drivefx::LIBRARY_CAPACITY { return None; }
        let slot = self.types.len();
        self.types.push(t.prepare(slot as u32));
        self.keys.insert(key, slot);
        Some(slot)
    }

    /// The library the game builds from DRIVING.ELF's own records: `dflt`, then the nine vehicle
    /// types (cells assigned) in `FUN_001edf38`'s order.
    pub fn from_elf_types(v: &[(u32, ParticleType)]) -> ParticleLib {
        let mut l = ParticleLib::default();
        for (k, t) in v { l.add(*k, t); }
        l
    }

    /// `FUN_001e7db0`: the slot under `key`; an unknown key returns the slot of the smallest key
    /// (the map's leftmost node) [M].
    pub fn find(&self, key: u32) -> Option<usize> {
        self.keys.get(&key).or_else(|| self.keys.values().next()).copied()
    }

    pub fn get(&self, slot: u8) -> &ParticleType { &self.types[slot as usize] }
}

/// The pool's capacity: current + pending < 0x400 [M `FUN_001f3bb8`].
pub const POOL_CAPACITY: usize = 0x400;
/// The draw build works in 512-particle chunks [M `FUN_001f4920` / `FUN_001f3dc8`].
pub const CHUNK: usize = 0x200;

/// The double-buffered pool (`RParticleSystemManager` + 0x40).
#[derive(Clone, Debug, Default)]
pub struct ParticleSystem {
    /// The buffer the step and the draw read (+0x04 = its count).
    pub cur: Vec<Particle>,
    /// The other buffer: new particles, then the step's survivors (+0x10044 = its count).
    pub next: Vec<Particle>,
    /// The tick of the last frame (+0x0c).
    pub last_tick: u32,
}

impl ParticleSystem {
    /// `FUN_001f3bb8`: a slot in the other buffer, None when the pool is full.
    pub fn alloc(&mut self) -> Option<&mut Particle> {
        if self.cur.len() + self.next.len() < POOL_CAPACITY {
            self.next.push(Particle::default());
            self.next.last_mut()
        } else {
            None
        }
    }

    /// One step over the current buffer, then the swap (`FUN_001f4878` + `FUN_001f3ca8`).
    pub fn step(&mut self, lib: &ParticleLib) {
        for mut p in std::mem::take(&mut self.cur) {
            let g = p.gravity as i32 as f32 * -0.000_244_140_62;
            let drag = lib.get(p.ty).drag;
            for i in 0..3 { p.pos[i] += p.vel[i]; } // vadd.xyz pos, pos, vel [M 0x1f3d4c]
            for i in 0..3 { p.vel[i] *= drag; } // vmulx.xyz vel, vel, drag [M 0x1f3d58]
            p.vel[1] += g;
            p.age = p.age.wrapping_sub(1);
            if p.age != 0 { self.next.push(p); }
        }
        self.cur = std::mem::take(&mut self.next);
    }

    /// `FUN_001f47f8`'s step part: (tick - last) steps, at most 3 (an unsigned delta >= 4 gives
    /// 3). Returns the steps run. The tick is DAT_00334010 (copied from DAT_00339f54 by
    /// `FUN_001d1ef8`); its rate is not traced here.
    pub fn run_steps(&mut self, tick: u32, lib: &ParticleLib) -> u32 {
        let d = tick.wrapping_sub(self.last_tick);
        let n = if d < 4 { d } else { 3 };
        for _ in 0..n { self.step(lib); }
        self.last_tick = tick;
        n
    }

    /// The draw build `FUN_001f3dc8` over every chunk of the current buffer, in the game's
    /// submission order: per 512-chunk the default bucket, the additive bucket, then the streaks.
    /// `attach(id)` gives an attached object's position; an id it does not know is cleared to
    /// 0xff on a quad, as the game does (streaks have no such check in the game, they add
    /// nothing here).
    pub fn draws(&mut self, lib: &ParticleLib, attach: impl Fn(u8) -> Option<[f32; 3]>) -> Vec<ParticleDraw> {
        let mut out = Vec::new();
        for chunk in self.cur.chunks_mut(CHUNK) {
            let (mut a, mut b, mut s) = (Vec::new(), Vec::new(), Vec::new());
            for p in chunk.iter_mut() {
                match draw_one(p, lib.get(p.ty), &attach) {
                    d @ ParticleDraw::Quad { additive: false, .. } => a.push(d),
                    d @ ParticleDraw::Quad { additive: true, .. } => b.push(d),
                    d => s.push(d),
                }
            }
            out.extend(a);
            out.extend(b);
            out.extend(s);
        }
        out
    }
}

/// What the draw build uploads for one particle [M `FUN_001f3dc8`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleDraw {
    /// A sprite from the 256x256 `MAIN` sheet [M `FUN_001f34d8` binds 'MAIN' to both quad
    /// buckets]. `cell` = (u, v, size) in sheet units. `additive`: the 0x200 bucket, GS ALPHA
    /// 0x48 (Cs*As + Cd) with FIX 0x80 [M `DAT_003e6030 = 0x8000000048`]; the default bucket is
    /// GS ALPHA 0x44 ((Cs - Cd)*As + Cd, plain alpha blending) [M `FUN_001a0f80` on 0x3e3f70],
    /// the streak bucket 0x68 with FIX 0x80 (Cs*FIX + Cd) [M `FUN_001a0fb8(0x3e8070, 0x80)`].
    Quad { centre: [f32; 3], size: f32, rotation: f32, cell: [f32; 3], rgba: [u8; 4], additive: bool },
    /// A triangle on the `sprk` texture: head + (0, width, 0), head, tail with UVs (0,0), (0,1),
    /// (1,0) [M the 0x40fdb0 table `FUN_001f34d8` fills].
    Streak { verts: [[f32; 3]; 3], rgba: [u8; 4] },
}

/// Pack a (A, R, G, B) 0..1 colour the way the draw does: *255, `vftoi0` (truncation), then the
/// halfword/byte packs keep the low byte of each lane, R G B A in memory [M 0x1f3f60..].
pub fn pack_colour(c: [f32; 4]) -> [u8; 4] {
    let b = |x: f32| ((x * 255.0) as i32) as u8;
    [b(c[1]), b(c[2]), b(c[3]), b(c[0])]
}

/// The colour at t = age/lifetime: keys 0→1 over t in [0, 0.5], 1→2 over (0.5, 1] [M].
pub fn colour_at(t: &ParticleType, life_t: f32) -> [f32; 4] {
    let mut x = life_t + life_t;
    let hi = 1.0 < x;
    if hi { x -= 1.0; }
    let (k0, k1) = if hi { (t.keys[1], t.keys[2]) } else { (t.keys[0], t.keys[1]) };
    let mut c = [0.0; 4];
    for i in 0..4 { c[i] = k0[i] + (k1[i] - k0[i]) * x; }
    c
}

fn draw_one(p: &mut Particle, t: &ParticleType, attach: &impl Fn(u8) -> Option<[f32; 3]>) -> ParticleDraw {
    let life_t = p.age as f32 * t.inv_life;
    let mut c = colour_at(t, life_t);
    if t.flags & FLAG_NIBBLE_ALPHA != 0 { c[0] = (p.b1c >> 4) as f32 * 0.0625; }
    let rgba = pack_colour(c);
    if t.flags & FLAG_STREAK != 0 {
        let head = [p.pos[0] + p.vel[0] * t.size1, p.pos[1] + p.vel[1] * t.size1, p.pos[2] + p.vel[2] * t.size1];
        let mut v = [[head[0], head[1] + t.size0, head[2]], head, p.pos];
        if p.attach != 0xff {
            if let Some(o) = attach(p.attach) {
                for vert in v.iter_mut() { for i in 0..3 { vert[i] += o[i]; } }
            }
        }
        return ParticleDraw::Streak { verts: v, rgba };
    }
    let rotation = if p.flags & 0x08 == 0 {
        -0.785_398_2 // 0xbf490fdb
    } else if p.flags & 0x10 == 0 {
        (life_t * p.spin as f32) / 40.7
    } else {
        p.spin as f32 / 40.7
    };
    let mut centre = p.pos;
    if p.attach != 0xff {
        match attach(p.attach) {
            Some(o) => for i in 0..3 { centre[i] += o[i] },
            None => p.attach = 0xff,
        }
    }
    let size = if t.flags & FLAG_NIBBLE_SIZE == 0 { t.size0 + life_t * t.size1 } else { (p.b1c & 0xf) as f32 / 1.5 };
    ParticleDraw::Quad { centre, size, rotation, cell: [t.cell[0], t.cell[1], t.cell[2]], rgba, additive: t.flags & FLAG_ADDITIVE != 0 }
}

/// The 0x40-byte frame block an emitter reads (`FUN_001ee238` builds fourteen per vehicle):
/// +0x00 position, +0x20 the second axis, +0x2c the type key, +0x30 the main axis, +0x3c the
/// attached object id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmitterFrame {
    pub pos: [f32; 3],
    pub axis_b: [f32; 3],
    pub tag: u32,
    pub axis_a: [f32; 3],
    pub attach: u8,
}

impl EmitterFrame {
    /// `FUN_001ee238`'s defaults: axis_b (1, 0, 0), axis_a (0, 1, 0), attach 0xff.
    pub fn new(tag: u32) -> EmitterFrame {
        EmitterFrame { pos: [0.0; 3], axis_b: [1.0, 0.0, 0.0], tag, axis_a: [0.0, 1.0, 0.0], attach: 0xff }
    }
}

/// Particles per step from the type's count and window (`FUN_001e5cd8`) [M].
pub fn emit_rate(count: u32, life: u32, delay: u32, duration: u32) -> f32 {
    let (c, l, d) = (count as i32, life as i32, delay as i32);
    if duration != 0 {
        let end = d.wrapping_add(duration as i32);
        if d <= l {
            if l <= end { return c as f32 / d as f32; }
            return c as f32 / ((l as f32 / end as f32) * d as f32);
        }
    }
    c as f32 / l as f32
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// `v * rsqrt(|v|^2)`; a zero vector stays zero (the VU's rsqrt of 0 is the largest float, not
/// infinity).
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if l2 == 0.0 { return [0.0; 3]; }
    let k = 1.0 / l2.sqrt();
    [v[0] * k, v[1] * k, v[2] * k]
}

/// One emitter (`RParticleSystem`, 0x70 bytes) [M `FUN_001e5e10`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Emitter {
    /// +0x00: the third velocity axis, cross(frame.axis_b, frame.axis_a) at build [M FUN_001e5d78].
    pub axis_c: [f32; 3],
    /// +0x10: the frame position at the last tick.
    pub last_pos: [f32; 3],
    /// +0x20: steps since (re)start.
    pub age: u32,
    /// +0x24: the camera distance the size cull divides by (the vehicle code writes 10.0).
    pub distance: f32,
    /// +0x28 / +0x2c: particles emitted / owed.
    pub emitted: f32,
    pub owed: f32,
    /// +0x30 / +0x5c: the type's library slot.
    pub ty: usize,
    /// +0x38.
    pub active: bool,
    /// +0x3c: bit 0x02 = the type has no size.
    pub flags: u16,
    /// +0x44: the carrier's velocity (per second; the spawn adds it / 60). The vehicle slots
    /// leave it unset.
    pub carrier: Option<[f32; 3]>,
    /// +0x48 / +0x4c: particles per step and its multiplier.
    pub rate: f32,
    pub scale: f32,
    /// +0x58: count * detail >> 8 (detail DAT_00425148 = 0x100 [M FUN_001d6e18]).
    pub count_scaled: u32,
}

/// The emission detail (DAT_00425148) [M: its only writer stores 0x100].
pub const DETAIL: u32 = 0x100;

impl Emitter {
    /// `FUN_001e5e10(distance, emitter, frame, 1)` including `FUN_001e5d78` [M].
    pub fn new(distance: f32, frame: &EmitterFrame, lib: &ParticleLib) -> Emitter {
        let ty = lib.find(frame.tag).unwrap_or(0);
        let t = &lib.types[ty];
        let mut e = Emitter {
            axis_c: cross(frame.axis_b, frame.axis_a),
            last_pos: frame.pos,
            age: 0,
            distance,
            emitted: 1.0,
            owed: 0.0,
            ty,
            active: true,
            flags: 0,
            carrier: None,
            rate: emit_rate(t.count, t.life, t.delay, t.duration),
            scale: 1.0,
            count_scaled: t.count.wrapping_mul(DETAIL) >> 8,
        };
        e.owed += e.rate * e.scale * t.prewarm as f32;
        if t.size0 == 0.0 && t.size1 == 0.0 { e.flags |= 2; }
        if t.flags & FLAG_FOREVER != 0 { e.active = false; }
        e
    }

    /// `FUN_001e5fa0`: switch the frame's type (the surface spray changes type per surface).
    pub fn set_type(&mut self, frame: &mut EmitterFrame, tag: u32, lib: &ParticleLib) {
        if frame.tag == tag { return; }
        frame.tag = tag;
        let ty = lib.find(tag).unwrap_or(0);
        let t = &lib.types[ty];
        self.count_scaled = t.count.wrapping_mul(DETAIL) >> 8;
        self.rate = emit_rate(self.count_scaled, t.life, t.delay, t.duration);
        self.ty = ty;
    }

    /// `FUN_001e65c0(emitter, frames)`: emit what the elapsed steps owe [M].
    pub fn tick(&mut self, frames: i32, frame: &EmitterFrame, lib: &ParticleLib, rng: &mut DriveRng, pool: &mut ParticleSystem) {
        if !self.active { return; }
        let delta = [self.last_pos[0] - frame.pos[0], self.last_pos[1] - frame.pos[1], self.last_pos[2] - frame.pos[2]];
        self.last_pos = frame.pos;
        let t = &lib.types[self.ty];
        let after = self.age.wrapping_add(frames as u32);
        if t.duration != 0 && t.delay < self.age {
            let end = t.delay.wrapping_add(t.duration);
            if end < self.age {
                self.age -= end;
                if t.flags & FLAG_FOREVER != 0 { self.active = false; }
                return;
            }
            self.age = after;
            return;
        }
        let nf = frames as f32;
        let rate = self.rate * self.scale;
        self.owed += rate * nf;
        if 0.0 < self.distance && t.max_size / self.distance < 0.004 {
            self.emitted = self.owed;
            self.age = after;
            return;
        }
        while self.emitted < self.owed {
            let frac = ((self.owed - self.emitted) / rate) / nf;
            if let Some(slot) = pool.alloc() {
                *slot = spawn(frac, nf, frame.pos, t, frame, self.axis_c, self.carrier, delta, rng);
            }
            self.emitted += 1.0;
        }
        self.age = after;
    }
}

/// `FUN_001f29d8`: one new particle [M]. `frac` = how far back in this tick's interval it was
/// owed (0..1), `frames` = the tick's steps, `delta` = last frame position - this one.
#[allow(clippy::too_many_arguments)]
pub fn spawn(frac: f32, frames: f32, origin: [f32; 3], t: &ParticleType, frame: &EmitterFrame, axis_c: [f32; 3], carrier: Option<[f32; 3]>, delta: [f32; 3], rng: &mut DriveRng) -> Particle {
    let mut p = Particle { pos: origin, ..Default::default() };
    let back = frac * frames;
    let r = rng.spread(t.spread_x);
    p.pos[0] = ((p.pos[0] + -t.spread_x) + r) + r;
    let r = rng.spread(t.spread_z);
    p.pos[2] = ((p.pos[2] + -t.spread_z) + r) + r;
    let r = rng.spread(t.speed_var);
    let s = ((t.speed - t.speed_var) + r) + r;
    for i in 0..3 { p.vel[i] = frame.axis_a[i] * s; }
    let r = rng.spread(t.side_var);
    let s = (-t.side_var + r) + r;
    for i in 0..3 { p.vel[i] += frame.axis_b[i] * s; }
    let r = rng.spread(t.third_var);
    let s = (-t.third_var + r) + r;
    for i in 0..3 { p.vel[i] += axis_c[i] * s; }
    if let Some(c) = carrier {
        for i in 0..3 { p.vel[i] += c[i] / 60.0; }
    }
    let half = if back < 0.0 { -0.5 } else { 0.5 };
    let age = (t.life as u8).wrapping_sub((half + back) as i32 as u8);
    p.age = if age == 0 { 1 } else { age };
    let r = rng.spread(t.gravity_var);
    p.gravity = (((t.gravity - t.gravity_var) + r) + r) as i32 as i8;
    let r = rng.spread(t.spin_var);
    p.spin = (((-t.spin_var + r) + r) * 40.7) as i32 as i8;
    p.ty = t.slot as u8;
    p.attach = frame.attach;
    p.b1c = t.byte_2c as u8;
    p.drag = (t.drag * 255.0) as i32 as u8;
    let fl = t.flags as u8;
    p.flags = if t.flags & FLAG_BYTE2C != 0 && p.b1c != 0 { fl | 0x40 } else { fl & 0xbf };
    // Type flag 0x10: the game replaces the spin with the screen-space velocity angle here
    // (camera matrix + atan2); not ported, the random spin stays. No traced type sets it.
    if 0.0 < t.alpha_rand {
        p.flags &= 0xbf;
        let (a, b) = (t.keys[2][0], t.keys[0][0]);
        let d = ((a - b) as i32).abs();
        let lo = if a < b { a } else { b };
        let r = rng.bits() as f32;
        p.b1c = (((lo + d as f32 * (1.0 - r * t.alpha_rand * 1.525_878_9e-5)) * 58.823_532) as i32 as u8) << 4;
    }
    if 0.0 < t.size_rand {
        p.flags &= 0xbf;
        let d = (t.size1 as i32).abs();
        let r = rng.bits() as f32;
        let v = p.b1c as u32 | (((t.size0 + t.size1 + d as f32 * (1.0 - r * t.size_rand * 1.525_878_9e-5)) * 1.5) as i32 as u32 & 0xf);
        p.b1c = if v < 2 { 1 } else { v as u8 };
    }
    let fm = frac - 1.0;
    let g = p.gravity as i32 as f32 * -0.000_244_140_62 * back * back;
    p.pos[0] = p.pos[0] + delta[0] * frac + p.vel[0] * frames * fm;
    p.pos[1] = p.pos[1] + delta[1] * frac + p.vel[1] * frames * fm + g;
    p.pos[2] = p.pos[2] + delta[2] * frac + p.vel[2] * frames * fm;
    p
}

/// The per-vehicle effect set (`RVehicleParticle`, `FUN_001ee238`): fourteen frames and emitters.
#[derive(Clone, Debug)]
pub struct VehicleFx {
    pub frames: [EmitterFrame; 14],
    pub emitters: [Emitter; 14],
    /// +0x40 / +0x44: a slip boost until this tick, per remaining tick (both 0 / 1.0 at build;
    /// the writer is not traced).
    pub boost_until: u32,
    pub boost_per_tick: f32,
}

/// One wheel's inputs to `FUN_001ee4a0` (vehicle vtable calls; the port's physics side supplies
/// them). Which physical quantity each vtable slot returns is inferred from use, not traced.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelFxInput {
    /// The wheel's world position (the car's +0x5c wheel table entry + the car position).
    pub pos: [f32; 3],
    /// vtable +0x1fc, & 0xf.
    pub surface: u8,
    /// vtable +0x14c / +0x214 (both gate sparks; +0x214 also gates the SNOW / RDDR speed rule).
    pub flag_a: bool,
    pub flag_b: bool,
    /// vtable +0x21c: drives smoke and the gravel / grass / water spray.
    pub slip: f32,
}

/// The car's inputs to `FUN_001ee4a0` / `FUN_001eed18`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarFxInput {
    /// The physics body's +0x20 vector (velocity).
    pub vel: [f32; 3],
    /// The car matrix's +0x20 row (the spray leans back along it by 3.66).
    pub axis: [f32; 3],
    /// vtable +0xac (SNOW: /10, RDDR: /14).
    pub speed: f32,
    /// vtable +0x1d4 → +200 != 0: no snow spray.
    pub no_snow: bool,
    /// The car's attached-object id (sparks are stored relative to it) and that object's position.
    pub attach: u8,
    pub attach_pos: [f32; 3],
}

impl VehicleFx {
    /// `FUN_001ee238`: frames from the slot table, emitters built inactive.
    pub fn new(lib: &ParticleLib) -> VehicleFx {
        let frames = drivefx::SLOT_TYPES.map(EmitterFrame::new);
        let emitters = frames.map(|f| {
            let mut e = Emitter::new(0.0, &f, lib);
            e.active = false;
            e
        });
        VehicleFx { frames, emitters, boost_until: 0, boost_per_tick: 1.0 }
    }

    /// `FUN_001ee4a0(vehicle, enabled)` [M]. `sparks_enabled` = `Enable vehicle sparks`.
    pub fn update(&mut self, enabled: bool, car: &CarFxInput, wheels: &[WheelFxInput; 4], tick: u32, sparks_enabled: bool, lib: &ParticleLib) {
        if !enabled {
            for e in self.emitters.iter_mut() { e.active = false; }
            return;
        }
        for s in 8..13 { self.frames[s].attach = car.attach; }
        let v = car.vel;
        for (i, w) in wheels.iter().enumerate() {
            let surface = w.surface & 0xf;
            if matches!(surface, 1 | 2 | 4 | 10 | 12) && w.flag_a && w.flag_b {
                let v2 = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
                if 5.0 < v2 && sparks_enabled {
                    self.sparks(i + 9, w.pos, [2.0 * v[0], 2.0 * v[1], 2.0 * v[2]], car);
                }
            }
            let tag = drivefx::SURFACE_SPRAY[surface as usize];
            let mut x = w.slip;
            if tick < self.boost_until {
                let b = (self.boost_until - tick) as f32 * self.boost_per_tick;
                if x < b { x = b; }
            }
            // tyre smoke (slot i)
            if tag == 0 && 0.2 <= x {
                if 1.0 < x { x = 1.0; }
                let e = &mut self.emitters[i];
                self.frames[i].pos = w.pos;
                e.distance = 10.0;
                e.active = true;
                e.scale = (x - 0.2) * 4.0 + 0.1;
                self.frames[i].axis_a = [v[0], v[1] + 0.8, v[2]];
            } else {
                self.emitters[i].active = false;
            }
            if tag == drivefx::fourcc(b"SNOW") && w.flag_b {
                x = (car.speed / 10.0).min(1.0).max(0.0);
                if car.no_snow { x = 0.0; }
            }
            if tag == drivefx::fourcc(b"RDDR") && w.flag_b {
                x = (car.speed / 14.0).min(2.0).max(0.0);
            }
            // surface spray (slot i + 4)
            let s = i + 4;
            if tag != 0 && 0.3 <= x {
                if 1.0 < x && tag != drivefx::fourcc(b"RDDR") { x = 1.0; }
                self.frames[s].pos = w.pos;
                self.emitters[s].scale = x;
                let (e, f) = (&mut self.emitters[s], &mut self.frames[s]);
                e.set_type(f, tag, lib);
                e.distance = 10.0;
                e.active = true;
                f.axis_a = [v[0] - car.axis[0] * 3.66, v[1] + 0.8, v[2] - car.axis[2] * 3.66];
            } else {
                self.emitters[s].active = false;
            }
        }
    }

    /// `FUN_001eed18(slot, wheel pos, v)` [M]: d = -v, lifted by 0.2*max(|d.x|, |d.z|).
    fn sparks(&mut self, slot: usize, pos: [f32; 3], v: [f32; 3], car: &CarFxInput) {
        let f = &mut self.frames[slot];
        f.attach = car.attach;
        f.pos = [pos[0] - car.attach_pos[0], pos[1] - car.attach_pos[1] + 0.0, pos[2] - car.attach_pos[2]];
        let mut d = [-v[0], -v[1], -v[2]];
        let m = if d[2].abs() <= d[0].abs() { d[0].abs() } else { d[2].abs() };
        d[1] += m * 0.2;
        f.axis_a = d;
        let mut up = [0.0, 1.0, 0.0]; // EE 0x393830 [D]
        let flat = |a: f32| a.abs() < 0.1 || 1.0 < (d[1] / a).abs();
        if flat(d[0]) && flat(d[2]) {
            up = [1.0, 0.0, 0.0];
            if d[1] < 0.1 { d = [0.0, 0.0, 1.0]; }
        }
        let mut b = normalize(cross(up, d));
        if (b == [0.0; 3]) || (d[0] == 0.0 && d[1] == 0.0 && d[2] == 0.0) {
            b = [1.0, 0.0, 0.0];
            d = [0.0, 1.0, 0.0];
        }
        f.axis_b = b;
        let e = &mut self.emitters[slot];
        e.axis_c = normalize(cross(b, d));
        e.active = true;
        e.age = 0;
        e.distance = 10.0;
    }

    /// Tick every active emitter (the game ticks them from its system list, `FUN_001f0148` and
    /// the manager; that order is not traced, slot order here).
    pub fn tick(&mut self, frames: i32, lib: &ParticleLib, rng: &mut DriveRng, pool: &mut ParticleSystem) {
        for (e, f) in self.emitters.iter_mut().zip(self.frames.iter()) {
            e.tick(frames, f, lib, rng, pool);
        }
    }
}

/// A glare instance's pulse (`FUN_001f1b60`) [M]: `x = frac((tick - phase) * rate)`; 0 past
/// `duty`; with flag 0x800 a triangle (0x4000: a ramp-hold-ramp shape); `base + x*amp`. The
/// fraction uses `FUN_002bfb50`, which is floorf (exponent test, mantissa mask, -1 for negative
/// fractions) [M]. The instance fields come from the track `ef` entries
/// (`nf_formats::trackfx`, used by `driving::trackfx`).
pub fn glare_pulse(tick: u32, phase: u32, rate: f32, duty: f32, base: f32, amp: f32, flags: u32) -> f32 {
    let x = tick.wrapping_sub(phase) as f32 * rate;
    let mut x = x - x.floor();
    if duty < x { return 0.0; }
    if flags & 0x800 != 0 {
        if flags & 0x4000 == 0 {
            x += x;
            if x <= 1.0 { return base + x * amp; }
            x = 2.0 - x;
        } else if 0.4 < x {
            if x < 0.6 {
                x = if x < 0.5 { 2.0 - (x + 0.1 + x + 0.1) } else { (x - 0.1) + (x - 0.1) };
            } else {
                x = 1.0 - (x - 0.6) * 2.5;
            }
        } else {
            x *= 2.5;
        }
    }
    base + x * amp
}

/// `FUN_001f1d18`'s size cull [M]: a glare at `dist` is kept when dist / (size*pulse) <= 1.6
/// (pulse = 1 without flag 0x400, else the pulse, dropped below 0.0001); at most 128 per frame.
/// Returns the drawn size. `dist` is what the caller passes: the squared LOD distance for static
/// items, 0 for scene objects [M `FUN_001ad1a0`, `FUN_001cab38`]. The facing test of flag 0x20
/// is in `driving::trackfx::TrackFx::glares`.
pub fn glare_size(dist: f32, size: f32, pulse: Option<f32>, drawn_so_far: usize) -> Option<f32> {
    if drawn_so_far >= 0x80 { return None; }
    let m = match pulse {
        Some(p) if p < 0.0001 => return None,
        Some(p) => p,
        None => 1.0,
    };
    (dist / (size * m) <= 1.6).then_some(size * m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::drivefx::fourcc;

    fn smoke_type() -> ParticleType {
        // SMRT as on disc (see nf_formats::drivefx tests), built field-wise.
        ParticleType {
            spread_x: 0.0, speed: 0.015, spread_z: 0.0, count: 30, side_var: 0.0, speed_var: 0.0, third_var: 0.0,
            flags: 8, drag: 0.96, name: fourcc(b"smk5"), byte_2c: 0, inv_life: 0.0, size0: 0.2, size1: 0.7, life: 60,
            delay: 60, duration: 0, prewarm: 0, gravity: -0.2, gravity_var: 0.0, spin_var: 180.0, f58: 0.5,
            max_size: 0.0, cell: [0.0, 0.0, 0.25, 0.25], keys: [[0.0, 512.0, 512.0, 512.0], [32.0, 512.0, 512.0, 512.0], [0.0, 512.0, 512.0, 512.0]],
            size_rand: 0.0, alpha_rand: 0.0, fa8: 0, slot: 0,
        }
    }

    fn lib() -> ParticleLib {
        let mut l = ParticleLib::default();
        l.add(fourcc(b"SMRT"), &smoke_type());
        l
    }

    #[test]
    fn rng_matches_the_seeded_sequence() {
        // FUN_001c08c0(0) leaves the unmasked seed; one draw multiplies it.
        let mut r = DriveRng::seeded(0);
        assert_eq!(r.state, 0x3ade68b1);
        let p = 0x3ade68b1u32.wrapping_mul(0x75bcd15);
        assert_eq!(r.bits(), (p & 0xffff00) >> 8);
        assert_eq!(r.state, p & 0xffff);
        assert_eq!(DriveRng::seeded(1).state, 0x3ade68b1u32.wrapping_mul(0x75bcd15) & 0xffff);
        assert_eq!(DriveRng::seeded(500).state, 0x3ade68b1, "n % 500");
    }

    #[test]
    fn emit_rate_follows_fun_001e5cd8() {
        assert_eq!(emit_rate(15, 60, 60, 0), 0.25, "no window: count/life");
        assert_eq!(emit_rate(20, 24, 3, 99999), 20.0 / 3.0, "SPRK prepared: count/delay");
        assert_eq!(emit_rate(20, 24, 3, 1), 20.0 / ((24.0 / 4.0) * 3.0));
        assert_eq!(emit_rate(10, 2, 3, 1), 5.0, "delay > life: count/life");
    }

    #[test]
    fn step_moves_then_drags_then_falls_and_ages() {
        let l = lib();
        let mut s = ParticleSystem::default();
        *s.alloc().unwrap() = Particle { pos: [1.0, 2.0, 3.0], vel: [1.0, 1.0, 0.0], age: 2, gravity: 64, ..Default::default() };
        s.step(&l); // the new particle sits in `next`; this step only swaps it in
        assert_eq!(s.cur[0].pos, [1.0, 2.0, 3.0]);
        s.step(&l);
        let p = s.cur[0];
        assert_eq!(p.pos, [2.0, 3.0, 3.0]);
        assert_eq!(p.vel[0], 0.96);
        assert_eq!(p.vel[1], 0.96 + 64.0 * -0.000_244_140_62);
        assert_eq!(p.age, 1);
        s.step(&l);
        assert!(s.cur.is_empty(), "age 1 → 0 dies");
    }

    #[test]
    fn steps_per_frame_cap_at_three() {
        let l = lib();
        let mut s = ParticleSystem::default();
        assert_eq!(s.run_steps(2, &l), 2);
        assert_eq!(s.run_steps(9, &l), 3);
        assert_eq!(s.run_steps(9, &l), 0);
        assert_eq!(s.run_steps(8, &l), 3, "a backwards tick is a huge unsigned delta");
    }

    #[test]
    fn pool_caps_at_1024() {
        let mut s = ParticleSystem::default();
        for _ in 0..POOL_CAPACITY { assert!(s.alloc().is_some()); }
        assert!(s.alloc().is_none(), "cur + next < 0x400");
    }

    #[test]
    fn smoke_draw_values() {
        let l = lib();
        let t = &l.types[0];
        let mut p = Particle { age: 60, ty: 0, flags: 8, spin: 20, attach: 0xff, ..Default::default() };
        // birth: t = 1 → key 2 (alpha 0), size = birth size 0.2*1.4142
        match draw_one(&mut p, t, &|_| None) {
            ParticleDraw::Quad { size, rotation, rgba, additive, .. } => {
                assert!((size - 0.2 * 1.4142).abs() < 1e-6);
                assert_eq!(rotation, 20.0 / 40.7);
                assert_eq!(rgba, [255, 255, 255, 0]);
                assert!(!additive);
            }
            _ => panic!(),
        }
        // mid-life: t = 0.5 → key 1: alpha 32/512*0.5 = 1/32 → 255/32 = 7
        p.age = 30;
        match draw_one(&mut p, t, &|_| None) {
            ParticleDraw::Quad { rgba, .. } => assert_eq!(rgba, [255, 255, 255, 7]),
            _ => panic!(),
        }
        // no rotate flag → the fixed -pi/4
        p.flags = 0;
        match draw_one(&mut p, t, &|_| None) {
            ParticleDraw::Quad { rotation, .. } => assert_eq!(rotation, -0.785_398_2),
            _ => panic!(),
        }
    }

    #[test]
    fn unknown_attach_is_cleared_on_quads() {
        let l = lib();
        let mut p = Particle { age: 10, attach: 3, ..Default::default() };
        draw_one(&mut p, &l.types[0], &|_| None);
        assert_eq!(p.attach, 0xff);
        let mut p = Particle { age: 10, attach: 3, pos: [1.0, 0.0, 0.0], ..Default::default() };
        match draw_one(&mut p, &l.types[0], &|_| Some([10.0, 0.0, 0.0])) {
            ParticleDraw::Quad { centre, .. } => assert_eq!(centre, [11.0, 0.0, 0.0]),
            _ => panic!(),
        }
    }

    #[test]
    fn spawn_age_rounds_back_time() {
        let l = lib();
        let t = &l.types[0];
        let f = EmitterFrame::new(fourcc(b"SMRT"));
        let mut r = DriveRng::seeded(0);
        let p = spawn(0.5, 3.0, [0.0; 3], t, &f, [0.0; 3], None, [0.0; 3], &mut r);
        assert_eq!(p.age, 60 - 2, "round(1.5) = 2");
        assert_eq!(p.gravity, 0, "(int)-0.2 = 0: smoke does not fall");
        assert_eq!(p.drag, 244, "0.96 * 255 truncated");
        // velocity = axis_a (0,1,0) * 0.015; position pushed back by vel*frames*(frac-1)
        assert_eq!(p.vel, [0.0, 0.015, 0.0]);
        assert_eq!(p.pos[1], 0.015 * 3.0 * -0.5);
        // spin draw: (-pi + 2*pi*r) * 40.7
        let mut r2 = DriveRng::seeded(0);
        for _ in 0..6 { r2.bits(); }
        let rr = r2.spread(t.spin_var);
        assert_eq!(p.spin, (((-t.spin_var + rr) + rr) * 40.7) as i32 as i8);
    }

    #[test]
    fn emitter_owes_rate_times_frames() {
        let l = lib();
        let f = EmitterFrame::new(fourcc(b"SMRT"));
        let mut e = Emitter::new(10.0, &f, &l);
        assert!(e.active);
        assert_eq!(e.rate, 0.25, "15 / 60");
        let mut pool = ParticleSystem::default();
        let mut rng = DriveRng::seeded(0);
        for _ in 0..8 { e.tick(1, &f, &l, &mut rng, &mut pool); }
        // owed 2.0 after 8 steps, emitted starts at 1.0: one particle (at owed 1.25)
        assert_eq!(pool.next.len(), 1);
        e.scale = 0.0;
        e.tick(1, &f, &l, &mut rng, &mut pool);
        assert_eq!(pool.next.len(), 1);
    }

    #[test]
    fn wheel_smoke_scale_and_surface_gate() {
        let mut l = lib();
        let mut sp = smoke_type();
        sp.flags = 0x22;
        l.add(fourcc(b"SPRK"), &sp);
        l.add(fourcc(b"RDDR"), &smoke_type());
        let mut v = VehicleFx::new(&l);
        let car = CarFxInput { vel: [10.0, 0.0, 0.0], speed: 28.0, ..Default::default() };
        let mut w = [WheelFxInput::default(); 4];
        w[0].slip = 0.7;
        w[1].slip = 0.1;
        w[2] = WheelFxInput { surface: 2, slip: 0.7, flag_b: true, ..Default::default() };
        v.update(true, &car, &w, 0, true, &l);
        assert!(v.emitters[0].active);
        assert_eq!(v.emitters[0].scale, (0.7 - 0.2) * 4.0 + 0.1);
        assert_eq!(v.frames[0].axis_a, [10.0, 0.8, 0.0]);
        assert!(!v.emitters[1].active, "slip < 0.2");
        assert!(!v.emitters[2].active, "a spray surface has no tyre smoke");
        assert!(v.emitters[6].active, "RDDR spray on surface 2");
        assert_eq!(v.emitters[6].scale, 2.0, "RDDR: speed/14 capped at 2, not 1");
        assert_eq!(v.frames[6].tag, fourcc(b"RDDR"));
        v.update(false, &car, &w, 0, true, &l);
        assert!(v.emitters.iter().all(|e| !e.active));
    }

    #[test]
    fn sparks_point_back_and_up() {
        let l = lib();
        let mut v = VehicleFx::new(&l);
        let car = CarFxInput { vel: [3.0, 0.0, 0.0], attach: 5, attach_pos: [1.0, 1.0, 1.0], ..Default::default() };
        let w = [WheelFxInput { surface: 1, flag_a: true, flag_b: true, pos: [2.0, 2.0, 2.0], ..Default::default() }; 4];
        v.update(true, &car, &w, 0, true, &l);
        let f = v.frames[9];
        assert_eq!(f.attach, 5);
        assert_eq!(f.pos, [1.0, 1.0, 1.0], "relative to the car");
        assert_eq!(f.axis_a, [-6.0, 1.2, 0.0], "-2v, y + 0.2*6");
        assert!(v.emitters[9].active && v.emitters[9].age == 0);
        let mut v2 = VehicleFx::new(&l);
        v2.update(true, &car, &w, 0, false, &l);
        assert!(!v2.emitters[9].active, "Enable vehicle sparks = 0");
    }

    #[test]
    fn glare_pulse_shapes() {
        assert_eq!(glare_pulse(0, 0, 0.1, 1.0, 0.5, 1.0, 0), 0.5);
        assert_eq!(glare_pulse(5, 0, 0.1, 0.4, 0.0, 1.0, 0), 0.0, "past the duty");
        assert!((glare_pulse(3, 0, 0.1, 1.0, 0.0, 1.0, 0x800) - 0.6).abs() < 1e-6);
        assert!((glare_pulse(7, 0, 0.1, 1.0, 0.0, 1.0, 0x800) - 0.6).abs() < 1e-6, "2 - 1.4");
        assert_eq!(glare_size(1.0, 1.0, None, 0), Some(1.0));
        assert_eq!(glare_size(2.0, 1.0, None, 0), None);
        assert_eq!(glare_size(1.0, 1.0, None, 128), None);
    }
}
