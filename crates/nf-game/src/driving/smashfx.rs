//! The driving engine's event-started effects, per tick: Giotto effects started by object
//! events (glass, sparks, splinters and debris when a prop or a car panel is hit), the
//! explosions of rockets and missiles, their animated glares (`gg`), debris bursts (`gd`) and
//! sounds (`gs`), the 3-D debris pool, and the impact decal ring (scorch marks, oil). The data
//! is `nf_formats::trackfx` (docs/systems/driving-effects.md §9-§12); this is logic only and
//! draws nothing.
//!
//! Ported from DRIVING.ELF [M = decompile/disassembly, D = disc bytes]:
//! - the Giotto instance step `FUN_002345d8` and its element pass `FUN_00234818`: the frame
//!   rules (an effect whose `Flgs` bit 0 is clear never counts its run frames and never ends;
//!   one with bit 0 runs frames 0..=LIFE; the `gg` key frame counts in both), `gg` glares
//!   (`FUN_00233df8` key pair, `FUN_00235ea0` size, `FUN_00235d50` jitter, `FUN_001f1f88` add),
//!   `gd` bursts (`FUN_00235810`, once), `gs` sounds (`FUN_002363b0`, the first run);
//! - the Giotto random stream (0x33ba74, seeded `FUN_0024b328(0x642f2b)` from `FUN_0024afe0`),
//!   separate from the engine stream (0x333e60, `effects::DriveRng`) the debris spawn draws from;
//! - the debris pool (`FUN_001d94a8`: 23 types x 64 pieces; spawn `FUN_001d9bb8`, the
//!   per-frame fade and expiry `FUN_001d9920`, walked by `FUN_001d9560` for every type but 4);
//! - a scene object's effect block (`FUN_001a1f58`, the entry filter `FUN_001a1bc8`), its events
//!   (`FUN_001caba8`: 0-3 start/stop by mask or id `FUN_001ca4b0`, 9 the damage bits
//!   `FUN_001ca7a0`), the start/stop of one entry `FUN_001ca198`, the build-time starts
//!   `FUN_001ca048`, the 600-tick out-of-range stop `FUN_001ca340`;
//! - which events a hit sends: smackables `FUN_0019b5a8` (with `FUN_00195a58`), car panels
//!   `FUN_00189598`;
//! - the explosion's effect and sound choice (`ESpawnExplosion`, handler `FUN_0015a098`; the
//!   static-entry route of `ESpawnExplosionStatic`, `FUN_0015b2f8`); what a missile's ground
//!   hit passes (`FUN_00182978` -> `FUN_00200088` -> `FUN_00159fb8`);
//! - the decal ring (`FUN_001da6c8` add, `FUN_001da978` the per-draw fade-in) and the flat
//!   ground decal placement `FUN_001eb078` (explosion scorch: radius 8, record 5; Q-Slick oil:
//!   radius 4.2, record 4).
//!
//! Not traced (stated, not guessed): how the debris pieces move and turn (the VU1 program that
//! draws them reads pos, velocity, birth tick, axis and spin; it is not disassembled), who
//! walks debris type 4, what `FUN_002331e8` (intensity) and `FUN_002331c0` (events 4/5) do to
//! a running effect, the units of the explosion sound clock `FUN_00261038`, the readers of the
//! `gs` voice words +0x5c/+0x60/+0x70, and the bullet-hole decal `FUN_001ea8e8`.

use nf_formats::drivefx::Decal;
use nf_formats::gal::argb_to_gs;
use nf_formats::trackfx::{decal_style, kind, GiottoBurst, GiottoEffect, GiottoElement, GiottoGlareKey, GiottoSound, PlacedEffect};

use super::effects::{DriveRng, Emitter, EmitterFrame, ParticleLib, ParticleSystem};
use super::trackfx::quat_rotate;

const R16: f32 = 1.525_878_9e-5;

/// `FUN_0024afe0` seeds the Giotto stream with `FUN_0024b328(0x642f2b)` [M].
pub const GIOTTO_SEED_ARG: i32 = 0x0064_2f2b;

/// The Giotto random stream at 0x33ba74: the same generator as the engine's (0x75bcd15,
/// state = low 16 bits), its own state [M `FUN_0024b328`].
pub fn giotto_rng() -> DriveRng { DriveRng::seeded(GIOTTO_SEED_ARG) }

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn len(a: [f32; 3]) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn scale(a: [f32; 3], k: f32) -> [f32; 3] { [a[0] * k, a[1] * k, a[2] * k] }

/// The Hamilton product `a * b` as the VU code builds it [M `FUN_00235810` kind-0 branch,
/// `FUN_002360c0`]: xyz = a x b + a.xyz b.w + b.xyz a.w, w = a.w b.w - a.xyz . b.xyz.
pub fn quat_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let (av, bv) = ([a[0], a[1], a[2]], [b[0], b[1], b[2]]);
    let c = cross(av, bv);
    [
        c[0] + av[0] * b[3] + bv[0] * a[3],
        c[1] + av[1] * b[3] + bv[1] * a[3],
        c[2] + av[2] * b[3] + bv[2] * a[3],
        a[3] * b[3] - (av[0] * bv[0] + av[1] * bv[1] + av[2] * bv[2]),
    ]
}

// ---------------------------------------------------------------------------------------------
// gg: animated glares
// ---------------------------------------------------------------------------------------------

/// `FUN_00233df8` [M]: the key pair for `frame`: (earlier key, its frame, later key, its frame).
/// Frame 0 gives (0, 0, 0, 0); past the last key both are the last key.
pub fn glare_key_pair(keys: &[GiottoGlareKey], frame: i32) -> (usize, i32, usize, i32) {
    let (mut a, mut fa, mut b, mut fb) = (0, 0, 0, 0);
    if frame != 0 {
        for i in 0..keys.len().saturating_sub(1) {
            let (k1, k2) = (keys[i].frame, keys[i + 1].frame);
            if frame >= k1 && frame <= k2 {
                return (i, k1, i + 1, k2);
            }
            a = i + 1;
            fa = k2;
            b = i + 1;
            fb = k2;
        }
    }
    (a, fa, b, fb)
}

/// One glare a `gg` element adds this frame (`FUN_001f1f88`: an entry with flags 0, kind 2,
/// pos.w = 1, appended to the glare manager's list while it holds fewer than 128). Draw it with
/// `trackfx::glare_sprites(pos, size, 0, gtype, ..)`: no pulse, alpha 0x20.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GiottoGlare {
    pub pos: [f32; 3],
    pub gtype: u32,
    pub size: f32,
}

/// The `gg` branch of `FUN_00234818` for one element [M]: the pair for the instance's key
/// frame (`+0x68`), t = (frame - fa) / (fb - fa) (0 when equal), size =
/// |a.size + (b.size - a.size) t|, type and jitter from the earlier key, position = instance
/// position + the earlier key's offset. With jitter: three glares, each moved by
/// `2r/65536 - 1` per axis (three draws of the Giotto stream, x then y then z).
pub fn glare_element(keys: &[GiottoGlareKey], frame: i32, pos: [f32; 3], grng: &mut DriveRng, out: &mut Vec<GiottoGlare>) {
    if keys.is_empty() {
        return;
    }
    let (a, fa, b, fb) = glare_key_pair(keys, frame);
    let span = fb as f32 - fa as f32;
    let t = if span == 0.0 { 0.0 } else { (frame as f32 - fa as f32) / span };
    let (ka, kb) = (&keys[a], &keys[b]);
    let size = (ka.size + (kb.size - ka.size) * t).abs();
    let at = add(pos, ka.offset);
    if ka.jitter == 0 {
        out.push(GiottoGlare { pos: at, gtype: ka.gtype, size });
    } else {
        for _ in 0..3 {
            let j = [grng.bits(), grng.bits(), grng.bits()].map(|r| (r as f32 + r as f32) * R16);
            out.push(GiottoGlare { pos: [(at[0] - 1.0) + j[0], (at[1] - 1.0) + j[1], (at[2] - 1.0) + j[2]], gtype: ka.gtype, size });
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Debris
// ---------------------------------------------------------------------------------------------

/// The 23 debris types `FUN_001d94a8` builds from the table at 0x338e28 (stride 0x14): the
/// model name at `+0x10` (four characters) [M + ELF data].
pub const DEBRIS_MODELS: [&str; 23] = [
    "dmge", "dmge", "pipe", "dgls", "shel", "dmge", "dmge", "dgls", "dgls", "coin", "mail", "mail", "grbg", "grbg",
    "frut", "frut", "dmge", "wdsp", "wdsp", "dgls", "wdsp", "wdsp", "dmge",
];
/// Pieces per type: the spawn adds only while the count `+0xd18` is below 0x40 [M].
pub const DEBRIS_CAP: usize = 64;
/// `+0xd14`: the life every spawn writes, ticks [M `FUN_001d9bb8`].
pub const DEBRIS_LIFE: u32 = 70;
/// `FUN_001d9560` walks every type but this one [M].
pub const DEBRIS_UNWALKED: usize = 4;
/// The `gd` kind -> debris type table at 0x39c980 (kinds 2 and 3 are special-cased) [M + data].
pub const BURST_KIND_TYPE: [usize; 20] = [0, 3, 0, 0, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22];

/// `FUN_00235810`: the debris type of piece `i` of a burst of `kind` (1..19) [M].
pub fn burst_debris_type(kind: u32, i: usize) -> Option<usize> {
    match kind {
        2 => Some(i % 3),
        3 => Some(i % 2 + 5),
        1 | 4..=19 => Some(BURST_KIND_TYPE[kind as usize]),
        _ => None,
    }
}

/// One piece [M `FUN_001d9bb8` layout: +0x000 pos, +0x400 velocity (w = birth tick as a float),
/// +0x800 axis (w = spin), +0xc00 GS colour].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DebrisPiece {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub birth: f32,
    /// Normalised xyz (the VU `VMULq` writes xyz only, so the spin is not scaled).
    pub axis: [f32; 3],
    /// `r/65536 x 0.08975979` (pi/35).
    pub spin: f32,
    /// GS RGBA (`argb_to_gs`): R low, alpha (0x80 = 1.0) high.
    pub rgba: u32,
}

/// One debris type's pool.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DebrisType {
    pub pieces: Vec<DebrisPiece>,
    /// `+0xd10`: the piece size; every spawn overwrites it (shared by the type).
    pub size: f32,
    /// `+0xd14`.
    pub life: u32,
}

/// The debris manager (`DAT_00334a00`).
#[derive(Clone, Debug, PartialEq)]
pub struct DebrisPool {
    pub types: Vec<DebrisType>,
}

impl Default for DebrisPool {
    fn default() -> Self { DebrisPool { types: vec![DebrisType::default(); DEBRIS_MODELS.len()] } }
}

impl DebrisPool {
    /// `FUN_001d95e8(size, manager, ty, pos, vel, argb, gate)` -> `FUN_001d9bb8` [M]: nothing when
    /// `gate` is 0 or the type holds 64; else the piece with birth = `tick` and four draws of
    /// the engine stream: axis xyz = `2r/65536 - 1` each, normalised, spin = `r/65536 x pi/35`.
    pub fn spawn(&mut self, size: f32, ty: usize, pos: [f32; 3], vel: [f32; 3], argb: u32, gate: u32, tick: u32, rng: &mut DriveRng) -> bool {
        if gate == 0 {
            return false;
        }
        let Some(t) = self.types.get_mut(ty) else { return false };
        if t.pieces.len() >= DEBRIS_CAP {
            return false;
        }
        t.life = DEBRIS_LIFE;
        t.size = size;
        let a = [rng.bits(), rng.bits(), rng.bits()].map(|r| (r as f32 + r as f32) * R16 - 1.0);
        let spin = rng.bits() as f32 * R16 * 0.089_759_79;
        let l = len(a);
        let axis = if l > 0.0 { scale(a, 1.0 / l) } else { a };
        t.pieces.push(DebrisPiece { pos, vel, birth: tick as f32, axis, spin, rgba: argb_to_gs(argb) });
        true
    }

    /// `FUN_001d9560` -> `FUN_001d9920` per type but type 4 [M]: with `t = now - life`, a piece
    /// born before `t` is dropped (the last piece moves into its slot and is looked at next);
    /// any other piece gets alpha = round(alpha x t / birth) (half away from zero), read from
    /// and written to its own colour word each frame. `now` is `DAT_00334000` (the sim tick as a
    /// float, `FUN_001d1ef8`).
    pub fn update(&mut self, now: f32) {
        for (k, t) in self.types.iter_mut().enumerate() {
            if k == DEBRIS_UNWALKED || t.pieces.is_empty() {
                continue;
            }
            let cut = now - t.life as f32;
            let mut i = 0;
            while i < t.pieces.len() {
                if t.pieces[i].birth < cut {
                    t.pieces.swap_remove(i);
                    continue;
                }
                let p = &mut t.pieces[i];
                let q = if p.birth == 0.0 { f32::MAX.copysign(cut) } else { cut / p.birth };
                let f = (p.rgba >> 24) as f32 * q;
                let a = (f + if f < 0.0 { -0.5 } else { 0.5 }) as i32;
                p.rgba = (p.rgba & 0x00ff_ffff) | ((a as u32) << 24);
                i += 1;
            }
        }
    }

    pub fn count(&self) -> usize { self.types.iter().map(|t| t.pieces.len()).sum() }
}

/// `FUN_00235810(key, pos, orientation, centre = 0)` [M], one `gd` burst:
/// - kind 0: one piece of type 4, size 0.0049 (0x3ba0902e), gate 100, at `pos`, velocity =
///   (inst * key quaternion) applied to (0, side_spread x size, 0), plus (0, 0.018, 0); the
///   colour goes through `argb_to_gs` here and again in the spawn;
/// - otherwise count = trunc(2 x size), at most 7 when size <= 15; per piece three Giotto
///   draws for the velocity (x = r1 xz - xz/2, y = up + r2 yv - yv/2, z = r3 xz - xz/2, with
///   xz = side_spread x size, yv = up_spread x size, up = up x size), then, when any spread is
///   non-zero, three more for the offset (x from +0x4c, y from +0x54, z from +0x50, each
///   `r v - v/2`) turned by the instance orientation; position = offset + `pos`; types
///   [`burst_debris_type`]; a kind of 20 or more draws but spawns nothing.
pub fn burst(b: &GiottoBurst, pos: [f32; 3], quat: [f32; 4], grng: &mut DriveRng, rng: &mut DriveRng, debris: &mut DebrisPool, tick: u32) {
    let s = b.size;
    let n = (s + s) as i32;
    let count = if s <= 15.0 && n > 7 { 7 } else { n };
    let xz = b.side_spread * s;
    let yv = b.up_spread * s;
    let up = b.up * s;
    if b.kind == 0 {
        let q = quat_mul(quat, b.quat);
        let v = quat_rotate(q, [0.0, xz, 0.0]);
        let vel = [v[0], v[1] + 0.018, v[2]];
        debris.spawn(f32::from_bits(0x3ba0_902e), 4, pos, vel, argb_to_gs(b.argb), 100, tick, rng);
        return;
    }
    let [sx, sz, sy] = b.spread;
    for i in 0..count.max(0) as usize {
        let r = [grng.bits(), grng.bits(), grng.bits()].map(|r| r as f32);
        let vel = [(r[0] * xz * R16) - xz * 0.5, (up + r[1] * yv * R16) - yv * 0.5, (r[2] * xz * R16) - xz * 0.5];
        let mut off = [0.0f32; 3];
        if sx != 0.0 || sz != 0.0 || sy != 0.0 {
            let r = [grng.bits(), grng.bits(), grng.bits()].map(|r| r as f32);
            off = [-sx * 0.5 + r[0] * sx * R16, -sy * 0.5 + r[1] * sy * R16, -sz * 0.5 + r[2] * sz * R16];
            off = quat_rotate(quat, off);
        }
        if let Some(ty) = burst_debris_type(b.kind, i) {
            debris.spawn(b.piece_size, ty, add(off, pos), vel, b.argb, b.gate, tick, rng);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Giotto instances
// ---------------------------------------------------------------------------------------------

/// A sound an effect starts (`gs`, `FUN_002363b0` on the first run) [M].
#[derive(Clone, Debug, PartialEq)]
pub struct SoundStart {
    /// Which `gs` element of the instance (slot 0..7).
    pub slot: usize,
    pub bank: u32,
    pub name: String,
    /// Voice +0x6c: volume x 0.01.
    pub volume: f32,
    /// Voice +0x70 = key +0x5c x 0.01.
    pub w70: f32,
    /// Voice +0x5c / +0x60 (+0x64 = far squared).
    pub near: f32,
    pub far: f32,
    pub pos: [f32; 3],
    /// The mix group: `Giotto` (0x39ca00) for effect sounds.
    pub group: &'static str,
}

/// What one frame of an instance produced.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GiottoFrame {
    pub glares: Vec<GiottoGlare>,
    pub sounds: Vec<SoundStart>,
    /// Sound slots whose position was updated this frame (`FUN_002363b0` with no key).
    pub sound_moves: Vec<(usize, [f32; 3])>,
}

/// The particle types the runtime effects add: `FUN_00235ed0` adds each `gp` element's record
/// once, under key `n + 5` with n the number added so far [M]. `first` continues the count of
/// a track's load-started types (`TrackFx` adds those first).
#[derive(Clone, Debug, Default)]
pub struct GiottoTypes {
    pub added: u32,
    cache: Vec<(String, Vec<u32>)>,
}

impl GiottoTypes {
    pub fn starting_at(added: u32) -> GiottoTypes { GiottoTypes { added, cache: Vec::new() } }

    /// The library keys of `effect`'s `gp` elements, adding them on first use.
    pub fn keys(&mut self, effect: &GiottoEffect, lib: &mut ParticleLib) -> Vec<u32> {
        if let Some((_, k)) = self.cache.iter().find(|(n, _)| n.eq_ignore_ascii_case(&effect.name)) {
            return k.clone();
        }
        let mut k = Vec::new();
        for em in &effect.emitters {
            let key = self.added + 5;
            lib.add(key, &em.ptype);
            self.added += 1;
            k.push(key);
        }
        self.cache.push((effect.name.clone(), k.clone()));
        k
    }
}

#[derive(Clone, Debug)]
struct GpSlot {
    offset: [f32; 3],
    quat: [f32; 4],
    start_frame: i32,
    start_override: i32,
    frame: EmitterFrame,
    emitter: Option<Emitter>,
}

/// One running Giotto effect (`FUN_00234d80` instance, 0x90 bytes).
#[derive(Clone, Debug)]
pub struct GiottoRun {
    pub name: String,
    pub pos: [f32; 3],
    pub quat: [f32; 4],
    pub life: u32,
    /// `Flgs` bit 0 -> instance bit 43: the run frames count and the effect ends after LIFE.
    pub counts_frames: bool,
    /// `+0x5c`: runs so far (stays 0 when `counts_frames` is false).
    pub run_frame: u32,
    /// `+0x68` low word: ticks since the start (the `gg` key frame).
    pub key_frame: i32,
    pub ended: bool,
    elements: Vec<GiottoElement>,
    gp: Vec<GpSlot>,
    burst_done: bool,
    sound_started: [bool; 8],
}

impl GiottoRun {
    /// Start `effect` at `pos` with orientation `quat` [M `FUN_00234d80`]: the `gp` types are
    /// added to `lib` through `types`.
    pub fn new(effect: &GiottoEffect, pos: [f32; 3], quat: [f32; 4], types: &mut GiottoTypes, lib: &mut ParticleLib) -> GiottoRun {
        let keys = types.keys(effect, lib);
        let gp = effect
            .emitters
            .iter()
            .zip(keys)
            .map(|(em, key)| GpSlot { offset: em.offset, quat: em.quat, start_frame: em.start_frame, start_override: em.start_override, frame: EmitterFrame::new(key), emitter: None })
            .collect();
        GiottoRun {
            name: effect.name.clone(),
            pos,
            quat,
            life: effect.life,
            counts_frames: effect.flags & 1 != 0,
            run_frame: 0,
            key_frame: 0,
            ended: false,
            elements: effect.elements.clone(),
            gp,
            burst_done: false,
            sound_started: [false; 8],
        }
    }

    /// `FUN_002345d8` for this instance, then `frames` steps of its emitters [M]:
    /// - without `counts_frames` the run frame is reset to 0 every tick (so the elements always
    ///   see frame 0, and only a LIFE of 0 ends it); with it, the elements run while the run
    ///   frame is <= LIFE; the run at LIFE sets the end bit first, so it runs the `gp` emitters
    ///   only (`gg`, `gd` and `gs` test the bit);
    /// - elements in order: `gp` as in `TrackFx` (start frame, frame at pos + the offset turned
    ///   by the orientation, axes from the instance x key quaternion, a forever type switched on
    ///   at frame 0 / its start frame), `gg` [`glare_element`] at the key frame, `gd` [`burst`]
    ///   once, `gs` a [`SoundStart`] on the first run of each of the first 8 sound elements, a
    ///   position update after;
    /// - the key frame counts every tick.
    pub fn tick(&mut self, frames: i32, camera: [f32; 3], lib: &ParticleLib, rng: &mut DriveRng, grng: &mut DriveRng, pool: &mut ParticleSystem, debris: &mut DebrisPool, tick: u32) -> GiottoFrame {
        let mut out = GiottoFrame::default();
        if !self.counts_frames {
            self.run_frame = 0;
        }
        if !self.ended && self.run_frame <= self.life {
            // The end bit is set before the element pass: the last run is emitters only.
            if self.run_frame >= self.life {
                self.ended = true;
            }
            let last = self.ended;
            let fr = self.run_frame as i32;
            let mut gs_slot = 0usize;
            for el in &self.elements {
                match el {
                    GiottoElement::Emitter(i) => {
                        let Some(s) = self.gp.get_mut(*i) else { continue };
                        if s.start_override == 0 && fr < s.start_frame {
                            continue;
                        }
                        s.frame.pos = add(self.pos, quat_rotate(self.quat, s.offset));
                        let q = quat_mul(self.quat, s.quat);
                        s.frame.axis_a = quat_rotate(q, [0.0, 1.0, 0.0]);
                        s.frame.axis_b = quat_rotate(q, [1.0, 0.0, 0.0]);
                        let d = len(sub(s.frame.pos, camera));
                        let e = s.emitter.get_or_insert_with(|| Emitter::new(d, &s.frame, lib));
                        e.distance = d;
                        if (fr == 0 || fr == s.start_frame) && lib.types[e.ty].flags & nf_formats::drivefx::FLAG_FOREVER != 0 {
                            e.active = true;
                            e.age = 0;
                        }
                    }
                    GiottoElement::Glare(keys) => {
                        if !last {
                            glare_element(keys, self.key_frame, self.pos, grng, &mut out.glares);
                        }
                    }
                    GiottoElement::Burst(b) => {
                        if !last && (!self.burst_done || fr == 0) {
                            burst(b, self.pos, self.quat, grng, rng, debris, tick);
                            self.burst_done = true;
                        }
                    }
                    GiottoElement::Sound(s) => {
                        let slot = gs_slot;
                        gs_slot += 1;
                        if slot >= 8 || last {
                            continue;
                        }
                        if !self.sound_started[slot] {
                            if !s.makes_voice() {
                                continue;
                            }
                            self.sound_started[slot] = true;
                            out.sounds.push(sound_start(slot, s, self.pos));
                        } else {
                            out.sound_moves.push((slot, self.pos));
                        }
                    }
                }
            }
            if self.counts_frames {
                self.run_frame += 1;
            }
        }
        self.key_frame += 1;
        for s in &mut self.gp {
            if let Some(e) = s.emitter.as_mut() {
                e.tick(frames, &s.frame, lib, rng, pool);
            }
        }
        out
    }
}

fn sound_start(slot: usize, s: &GiottoSound, pos: [f32; 3]) -> SoundStart {
    SoundStart {
        slot,
        bank: s.bank,
        name: s.name.clone(),
        volume: s.volume as f32 * 0.01,
        w70: s.w5c as f32 * 0.01,
        near: s.near as f32,
        far: s.far as f32,
        pos,
        group: "Giotto",
    }
}

// ---------------------------------------------------------------------------------------------
// Scene-object effect blocks and their events
// ---------------------------------------------------------------------------------------------

/// `+0x10` flag bits the block reads [M `FUN_001a1f58`, `FUN_001ca048`, `FUN_001ca340`].
pub mod oflag {
    /// Started by the damage bits (event 9) -> block mask `+0x28`.
    pub const DAMAGE: u32 = 0x08;
    /// Any of these -> block mask `+0x38` (startable by events 0-5).
    pub const EVENTS: u32 = 0x96;
    /// Started when the block is built.
    pub const AT_BUILD: u32 = 0x10;
    /// Started when the block is built with the reset argument set.
    pub const AT_RESET: u32 = 0x04;
}

/// One entry of an object's effect block.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockEntry {
    pub kind: u8,
    pub flags: u32,
    /// `+0x12`: the event mask (the flags word's high half).
    pub mask: u16,
    /// `+0x17`: the id events 1/3/5 name.
    pub id: u8,
    pub name: Option<String>,
    pub pos: [f32; 3],
    /// `+0x1c`: the running Giotto handle.
    pub handle: Option<usize>,
}

/// What a block asks the effect system to do.
#[derive(Clone, Debug, PartialEq)]
pub enum BlockRequest {
    /// `FUN_002330b0`: start the entry's Giotto effect (its `GIOT::` name) at its position.
    Start { entry: usize, name: String, pos: [f32; 3] },
    /// `FUN_00233160`: stop the running effect.
    Stop { entry: usize, handle: usize },
    /// `FUN_002331c0` on a running effect (events 4/5); what it does is not traced.
    Touch { entry: usize, handle: usize },
}

/// A scene object's (or a car's) effect block (`RSceneObj +0x28`, built by `FUN_001a1f58`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EffectBlock {
    pub entries: Vec<BlockEntry>,
    /// `+0x20`: drawn / running.
    pub running: u64,
    /// `+0x28`, `+0x30`, `+0x38`.
    pub damage: u64,
    pub giotto: u64,
    pub events: u64,
    /// `RSceneObj +0x1e`: the damage bits seen so far (event 9).
    pub seen: u16,
    /// `RSceneObj +0x20`: the tick of the next out-of-range check (0 = none).
    pub deadline: u32,
}

/// `FUN_001ca340`'s range: 28900 = 170 squared [M].
pub const KEEP_RANGE2: f32 = 28900.0;
/// Ticks added to the deadline after an event or a check in range [M].
pub const KEEP_TICKS: u32 = 600;

impl EffectBlock {
    /// Build from one object's placed entries [M `FUN_001a1f58`]: only the entries
    /// `FUN_001a1bc8` keeps (`PlacedEffect::scene_kept`), in order; mask bits from the flags.
    pub fn from_entries<'a>(placed: impl IntoIterator<Item = &'a PlacedEffect>) -> EffectBlock {
        let mut b = EffectBlock::default();
        for p in placed.into_iter().filter(|p| p.scene_kept()) {
            let i = b.entries.len() as u64;
            if i >= 64 {
                break;
            }
            let f = p.e.flags;
            if f & oflag::DAMAGE != 0 {
                b.damage |= 1 << i;
            }
            if f & oflag::EVENTS != 0 {
                b.events |= 1 << i;
            }
            if p.e.kind == kind::GIOTTO {
                b.giotto |= 1 << i;
            }
            b.entries.push(BlockEntry { kind: p.e.kind, flags: f, mask: (f >> 16) as u16, id: p.e.b17, name: p.giotto_name().map(str::to_owned), pos: p.world, handle: None });
        }
        b
    }

    fn bit(m: u64, i: usize) -> bool { m >> i & 1 != 0 }

    /// `FUN_001ca198(-1, obj, on, i, entry)` [M]. On: when not running, a non-Giotto entry is
    /// marked running; a Giotto entry without a handle asks `start` (which returns the handle,
    /// None when the effect is missing or keeps none: `Flgs` bit 0), and is marked running only
    /// with a handle. Off: a running entry is unmarked, and a Giotto one with a handle stopped.
    pub fn set(&mut self, i: usize, on: bool, start: &mut impl FnMut(&str, [f32; 3]) -> Option<usize>, out: &mut Vec<BlockRequest>) {
        let Some(e) = self.entries.get_mut(i) else { return };
        if !on {
            if Self::bit(self.running, i) {
                self.running &= !(1 << i);
                if Self::bit(self.giotto, i) {
                    if let Some(h) = e.handle.take() {
                        out.push(BlockRequest::Stop { entry: i, handle: h });
                    }
                }
            }
            return;
        }
        if Self::bit(self.running, i) {
            return;
        }
        if !Self::bit(self.giotto, i) {
            self.running |= 1 << i;
            return;
        }
        if e.handle.is_none() {
            if let Some(name) = e.name.clone() {
                out.push(BlockRequest::Start { entry: i, name: name.clone(), pos: e.pos });
                e.handle = start(&name, e.pos);
            }
            if e.handle.is_some() {
                self.running |= 1 << i;
            }
        }
    }

    fn touch_deadline(&mut self, tick: u32) { self.deadline = if self.running != 0 { tick + KEEP_TICKS } else { 0 }; }

    /// `FUN_001ca048(obj, reset)` [M]: every entry outside the damage mask starts when it has
    /// flag 0x10, or flag 4 with `reset`. (Damage entries with a zero event mask only get a model
    /// part update, not ported.)
    pub fn build(&mut self, reset: bool, start: &mut impl FnMut(&str, [f32; 3]) -> Option<usize>) -> Vec<BlockRequest> {
        let mut out = Vec::new();
        for i in 0..self.entries.len() {
            if Self::bit(self.damage, i) {
                continue;
            }
            let f = self.entries[i].flags;
            if f & oflag::AT_BUILD != 0 || (reset && f & oflag::AT_RESET != 0) {
                self.set(i, true, start, &mut out);
            }
        }
        out
    }

    /// `FUN_001caba8(obj, event, arg)` [M]:
    /// - 0 / 2 start / stop every event entry outside the damage mask whose mask matches `arg`'s
    ///   high half (`mask << 16 & arg & 0xffff0000`, or both zero); 1 / 3 the entry whose id is
    ///   `arg` (no mask test); then the deadline is tick + 600 while anything runs, else 0;
    ///   nothing at all when no entry is an event entry;
    /// - 4 / 5: running Giotto event entries, by mask / by id, get a [`BlockRequest::Touch`];
    /// - 9: when `arg` holds bits not seen before, every damage entry is started when its mask
    ///   meets the accumulated bits and stopped otherwise; a Giotto one that still has no handle
    ///   is marked running anyway; then the deadline;
    /// - 10 drives model part animations (`FUN_001a2b50(.., 0x12, ..)`), not effects: nothing.
    pub fn event(&mut self, event: u32, arg: u32, tick: u32, start: &mut impl FnMut(&str, [f32; 3]) -> Option<usize>) -> Vec<BlockRequest> {
        let mut out = Vec::new();
        match event {
            0..=3 => {
                if self.events == 0 {
                    return out;
                }
                let on = event < 2;
                if event & 1 == 0 {
                    for i in 0..self.entries.len() {
                        if !Self::bit(self.events, i) || Self::bit(self.damage, i) {
                            continue;
                        }
                        let m = self.entries[i].mask as u32;
                        let hit = (arg & 0xffff_0000 == 0 && m == 0) || (m << 16 & arg & 0xffff_0000) != 0;
                        if hit {
                            self.set(i, on, start, &mut out);
                        }
                    }
                } else if let Some(i) = self.entries.iter().position(|e| e.id as u32 == arg) {
                    self.set(i, on, start, &mut out);
                }
                self.touch_deadline(tick);
            }
            4 | 5 => {
                if self.events == 0 {
                    return out;
                }
                for i in 0..self.entries.len() {
                    let e = &self.entries[i];
                    let hit = if event == 4 {
                        Self::bit(self.events, i)
                            && !Self::bit(self.damage, i)
                            && Self::bit(self.giotto, i)
                            && ((arg & 0xffff_0000 == 0 && e.mask == 0) || ((e.mask as u32) << 16 & arg & 0xffff_0000) != 0)
                    } else {
                        e.id as u32 == arg && e.kind == kind::GIOTTO
                    };
                    if hit {
                        if let Some(h) = e.handle {
                            out.push(BlockRequest::Touch { entry: i, handle: h });
                        }
                        if event == 5 {
                            break;
                        }
                    }
                }
            }
            9 => {
                let arg = arg as u16;
                if !self.seen & arg == 0 {
                    return out;
                }
                let bits = self.seen | arg;
                self.seen = bits;
                if self.damage == 0 {
                    return out;
                }
                for i in 0..self.entries.len() {
                    if !Self::bit(self.damage, i) {
                        continue;
                    }
                    let on = self.entries[i].mask & bits != 0;
                    self.set(i, on, start, &mut out);
                    if on && Self::bit(self.giotto, i) && self.entries[i].handle.is_none() {
                        self.running |= 1 << i;
                    }
                }
                self.touch_deadline(tick);
            }
            _ => {}
        }
        out
    }

    /// `FUN_001ca930`'s check -> `FUN_001ca340` [M]: once the deadline has passed, an object
    /// within 170 of the camera gets another 600 ticks; else every running Giotto entry
    /// without flag 0x14 is stopped, and the deadline clears when nothing runs.
    pub fn range_check(&mut self, tick: u32, dist2: f32, start: &mut impl FnMut(&str, [f32; 3]) -> Option<usize>) -> Vec<BlockRequest> {
        let mut out = Vec::new();
        if self.running == 0 || self.deadline == 0 || self.deadline > tick {
            return out;
        }
        if dist2 < KEEP_RANGE2 {
            self.deadline = tick + KEEP_TICKS;
            return out;
        }
        if self.giotto != 0 && self.running != 0 {
            for i in 0..self.entries.len() {
                if Self::bit(self.running, i) && Self::bit(self.giotto, i) && self.entries[i].flags & 0x14 == 0 {
                    self.set(i, false, start, &mut out);
                }
            }
        }
        if self.running == 0 {
            self.deadline = 0;
        }
        out
    }
}

/// The events a smackable hit sends to its scene object [M `FUN_0019b5a8`, `FUN_00195a58`]:
/// nothing unless hit points remain before the hit; the damage (negative counts as 0) comes
/// off, floored at 0. Still standing: health = hp x (1 / start hp) (`+0x8c`); event 10 (part
/// animations) when health > 0.15, event 9 with all bits when health > 0.75. Destroyed: event
/// 10 with all bits only. Returns (hp after, events).
pub fn smackable_hit(hp: f32, inv_start_hp: f32, damage: f32) -> (f32, Vec<(u32, u32)>) {
    if hp <= 0.0 {
        return (hp, Vec::new());
    }
    let hp = (hp - damage.max(0.0)).max(0.0);
    let mut ev = Vec::new();
    if hp > 0.0 {
        let health = hp * inv_start_hp;
        if health > f32::from_bits(0x3e19_999a) {
            ev.push((10, 0xffff));
        }
        if health > f32::from_bits(0x3f40_0000) {
            ev.push((9, 0xffff));
        }
    } else {
        ev.push((10, 0xffff));
    }
    (hp, ev)
}

/// A car's panel hit [M `FUN_00189598`]: panel i's damage is the larger of its two words
/// (`+0x2f0`/`+0x2f4` + 8i, after the hit's increments); bit i of event 10 when it is above 0.35,
/// of event 9 when above 0.75 (the event-9 bits start the panel's glass / smoke entries:
/// `Ex_Glass`, `Cr_Dmg1`, `Cr_Dmg2` on the disc's car models).
pub fn car_panel_events(panel: &[[f32; 2]; 16]) -> (u32, u32) {
    let (mut e10, mut e9) = (0u32, 0u32);
    for (i, p) in panel.iter().enumerate() {
        let d = if p[1] <= p[0] { p[0] } else { p[1] };
        if d > f32::from_bits(0x3eb3_3333) {
            e10 |= 1 << i;
        }
        if d > f32::from_bits(0x3f40_0000) {
            e9 |= 1 << i;
        }
    }
    (e10, e9)
}

// ---------------------------------------------------------------------------------------------
// Explosions
// ---------------------------------------------------------------------------------------------

/// The `ESpawnExplosion` fields the effect choice reads (`FUN_00159fb8` stores them) [M].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExplosionArgs {
    /// `[0x13]`: a named effect (none on the `FUN_00200088` path).
    pub name: Option<String>,
    /// `[0xe]`: the weapon id (`FUN_00200088`'s third argument).
    pub weapon: u32,
    /// `[0x10]`: the surface byte of the ground query at the hit.
    pub surface: u32,
    /// `[0xf]`: the damage.
    pub damage: f32,
    /// `[0xc]` is the player's car (`FUN_00201ae0`).
    pub hit_player: bool,
    /// The track's underwater flag (player car vtable +0x1d4 -> +0xbc).
    pub underwater: bool,
}

/// `FUN_0015a098`'s choice when the event names no entry [M]: the explicit name; weapon 0x1e
/// (FOD rocket) `FODHitEffect`; surface 9 `snowspecificexplosion`; surface 6 `rocketsplash`;
/// underwater `Un_ExplL` / `M` / `S` by damage (> 600, > 200); surfaces 2..5, 10, 0x50, 0x60
/// `Ex_SmallD`, `2`, `3` by tick % 3; else `Ex_SmB00..02` when the player's car was hit and
/// `Ex_Small`, `2`, `3` otherwise, by a shared counter (0x32bbdc) that steps 0, 1, 2.
pub fn explosion_effect(a: &ExplosionArgs, tick: u32, counter: &mut u32) -> String {
    if let Some(n) = &a.name {
        return n.clone();
    }
    if a.weapon == 0x1e {
        return "FODHitEffect".into();
    }
    if a.surface == 9 {
        return "snowspecificexplosion".into();
    }
    if a.surface == 6 {
        return "rocketsplash".into();
    }
    if a.underwater {
        return if a.damage > 600.0 { "Un_ExplL" } else if a.damage > 200.0 { "Un_ExplM" } else { "Un_ExplS" }.into();
    }
    if (2..6).contains(&a.surface) || matches!(a.surface, 10 | 0x50 | 0x60) {
        return ["Ex_SmallD", "Ex_SmallD2", "Ex_SmallD3"][(tick % 3) as usize].into();
    }
    let t = if a.hit_player { ["Ex_SmB00", "Ex_SmB01", "Ex_SmB02"] } else { ["Ex_Small", "Ex_Small2", "Ex_Small3"] };
    let n = t[(*counter % 3) as usize];
    *counter = (*counter + 1) % 3;
    n.into()
}

/// What `ESpawnExplosionStatic` (`FUN_0015b2f8`) does with an entry of a scene object [M]: an
/// entry outside the block filter (flags & 0x109e == 0) starts its own effect directly
/// (`Ex_Snow` on surface 9 / `Ex_Small` when it names none); an entry in the block gets event 1
/// with its id.
#[derive(Clone, Debug, PartialEq)]
pub enum StaticExplosion {
    Start(String),
    Event1 { id: u8 },
}

pub fn static_explosion(entry_flags: u32, entry_name: Option<&str>, entry_id: u8, surface: u32) -> StaticExplosion {
    if entry_flags & 0x109e != 0 {
        return StaticExplosion::Event1 { id: entry_id };
    }
    StaticExplosion::Start(match entry_name {
        Some(n) => n.to_owned(),
        None if surface == 9 => "Ex_Snow".into(),
        None => "Ex_Small".into(),
    })
}

/// The four explosion sounds at 0x32bc10, tried from a rotating index [M `FUN_0015a098`].
pub const EXPLOSION_SOUNDS: [&str; 4] = ["SFX_HugeExp", "SFX_LrgExp", "SFX_MedExp", "SFX_SmallExp"];

/// The explosion's two voices [M `FUN_0015a098`]: bank slot 4 (`PTR_DAT_003562b0`); the
/// first a name from [`EXPLOSION_SOUNDS`] in the `Explosions` group, the second
/// `SFX_DistExp` in `ExplosionBass` (+0x5c 3, +0x60 350, +0x64 350 squared).
#[derive(Clone, Debug, PartialEq)]
pub struct ExplosionSound {
    pub name: &'static str,
    /// The first voice's +0x5c / +0x60: 100 / 400, x5 on `jungleb_mis13b`.
    pub near: f32,
    pub far: f32,
}

/// The rotating choice [M]: starting at `index & 3`, a name is free when the clock is past its
/// stored time; the first free one is used and its time set to clock + 30; when none is free
/// there is no sound (and no bass voice). `clock` is `FUN_00261038` (a global counter whose
/// unit is not traced).
pub fn explosion_sound(clock: i32, index: &mut u32, until: &mut [i32; 4], track: &str) -> Option<ExplosionSound> {
    let start = *index & 3;
    while clock <= until[(*index & 3) as usize] {
        *index = index.wrapping_add(1);
        if *index & 3 == start {
            return None;
        }
    }
    let k = (*index & 3) as usize;
    until[k] = clock + 0x1e;
    let m = if track.eq_ignore_ascii_case("jungleb_mis13b") { 5.0 } else { 1.0 };
    Some(ExplosionSound { name: EXPLOSION_SOUNDS[k], near: m * 100.0, far: m * 400.0 })
}

/// A missile or rocket hitting the ground [M `FUN_00182978`, the branch after the ground query
/// hit]: weapon 0xe (Q-Pulse) makes nothing; else the hit decal (`FUN_0019ddb8(damage, damage x
/// 0.008, .., 5)`, through `FUN_001ea8e8`, not ported) and `ESpawnExplosion` with the weapon, the
/// damage and the ground's surface byte (`FUN_00200088(damage, sim, 2, weapon, dir, pos,
/// surface, 0)`: no car).
pub fn projectile_ground_hit(weapon: u32, damage: f32, surface: u32, underwater: bool) -> Option<ExplosionArgs> {
    (weapon != 0xe).then(|| ExplosionArgs { name: None, weapon, surface, damage, hit_player: false, underwater })
}

// ---------------------------------------------------------------------------------------------
// The decal ring
// ---------------------------------------------------------------------------------------------

/// Slots in the ring (`+0x294` wraps past 0x5f) [M `FUN_001da6c8`].
pub const DECAL_SLOTS: usize = 96;

/// One decal: two triangles, (c0, c0 + a, c0 + b) and (c0 + a, c0 + b, c0 + a + b), UV rows 0, 1,
/// 2 and 1, 2, 3 of its record [M `FUN_001da6c8`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecalQuad {
    pub verts: [[f32; 3]; 6],
    pub uv: [[f32; 4]; 6],
    /// Draws left before the alpha appears (`0x3dc820`).
    pub delay: u8,
    pub alpha: u8,
    /// GS RGBA of all six vertices: 0x007f7f7f until the delay runs out, then alpha << 24.
    pub rgba: u32,
    pub record: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DecalRing {
    pub slots: Vec<Option<DecalQuad>>,
    /// `+0x294`.
    pub next: usize,
}

impl DecalRing {
    pub fn new() -> DecalRing { DecalRing { slots: vec![None; DECAL_SLOTS], next: 0 } }

    /// `FUN_001da6c8(w, h, manager, centre, normal, up, record)` [M]: a = (up x normal) w,
    /// b = ((up x normal) x normal) h, c0 = centre - a/2 - b/2; the record's `+0x44` (0 = 1)
    /// and `+0x48` words (`decal_style`); the slot after the last, wrapping at 96. (Both edge
    /// vectors are perpendicular to `normal`: it is the surface normal.)
    pub fn add(&mut self, w: f32, h: f32, centre: [f32; 3], normal: [f32; 3], up: [f32; 3], records: &[Decal], record: usize) {
        if self.slots.len() != DECAL_SLOTS {
            self.slots = vec![None; DECAL_SLOTS];
        }
        let Some(d) = records.get(record) else { return };
        let st = decal_style(d);
        let a0 = cross(up, normal);
        let b0 = cross(a0, normal);
        let (a, b) = (scale(a0, w), scale(b0, h));
        let c0 = add(add(centre, scale(a, -0.5)), scale(b, -0.5));
        let (c1, c2) = (add(c0, a), add(c0, b));
        let c3 = add(c1, b);
        let r = d.corners;
        self.slots[self.next] = Some(DecalQuad {
            verts: [c0, c1, c2, c1, c2, c3],
            uv: [r[0], r[1], r[2], r[1], r[2], r[3]],
            delay: st.delay_draws,
            alpha: st.alpha,
            rgba: 0x007f_7f7f,
            record,
        });
        self.next += 1;
        if self.next > 0x5f {
            self.next = 0;
        }
    }

    /// `FUN_001da978`, once per decal draw [M]: a non-zero delay counts down, and on reaching 0
    /// the six colours become alpha << 24 + 0x7f7f7f.
    pub fn draw_tick(&mut self) {
        for q in self.slots.iter_mut().flatten() {
            if q.delay != 0 {
                q.delay -= 1;
                if q.delay == 0 {
                    q.rgba = (q.alpha as u32) << 24 | 0x007f_7f7f;
                }
            }
        }
    }

    pub fn live(&self) -> usize { self.slots.iter().flatten().count() }
}

/// The ground query result `FUN_002322c8` fills, the fields the decal placement reads [M].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundHit {
    /// `+0x30`: 1 = a triangle was hit (anything else: no flat decal).
    pub hit: u32,
    /// The triangle's corners (`+0x00`, `+0x10`, `+0x20`).
    pub tri: [[f32; 3]; 3],
    /// `FUN_002327a0`: the ground height under the query point.
    pub height: f32,
    /// `+0x2c`: the surface byte (6 refuses decals, 9 refuses the flatness pass).
    pub surface: u8,
    /// `+0x2d` bit 1 refuses decals.
    pub no_decal: bool,
    /// `+0x34` names an object whose `+0x1c -> +0x0c` byte is set: refuses decals.
    pub on_object: bool,
}

/// `FUN_001eb078(radius, ground, pos, record)` [M]: a flat decal on the ground under `pos`.
/// Refused on surface 6, with `no_decal`, on an object; r = min(radius/2, 6); the normal =
/// (v1 - v0) x (v0 - v2) normalised (y made positive, capped at 0.9999), (0, 1, 0) without a
/// triangle; refused when the normal's y is below 0.95 or the hit is not a triangle. Other than
/// record 4 (oil), the four points pos +/- r on x and z must hit triangles within 0.15 of the
/// height (and the last one not surface 9). Size r - (pos.y - height)/2 must be positive; the
/// decal goes at height + 0.025, 2 x size square, with `up` = (0, 0, 1).
pub fn ground_decal(ring: &mut DecalRing, records: &[Decal], radius: f32, pos: [f32; 3], record: usize, ground: impl Fn([f32; 3]) -> GroundHit) -> bool {
    let g = ground(pos);
    if g.no_decal || g.surface == 6 || g.on_object {
        return false;
    }
    let r = (radius * 0.5).min(6.0);
    let drop = pos[1] - g.height;
    let mut n = [0.0f32, 1.0, 0.0];
    if g.hit != 0 {
        let [v0, v1, v2] = g.tri;
        let c = cross(sub(v1, v0), sub(v0, v2));
        if c != [0.0; 3] {
            n = scale(c, 1.0 / len(c));
        }
        if n[1] < 0.0 {
            n = scale(n, -1.0);
        }
        if n[1] >= 0.9999 {
            n[1] = 0.9999;
        }
    }
    if n[1] < 0.95 || g.hit != 1 {
        return false;
    }
    if record != 4 {
        let h0 = g.height;
        let probe = |p: [f32; 3], last: bool| {
            let q = ground(p);
            q.hit == 1 && (h0 - q.height).abs() < 0.15 && !(last && q.surface == 9)
        };
        let [x, y, z] = pos;
        let ok = probe([x + r, y, z], false) && probe([x - r, y, z], false) && probe([x - r, y, z + r], false) && probe([x - r, y, z - r], true);
        if !ok {
            return false;
        }
    }
    let size = r - drop * 0.5;
    if size <= 0.0 {
        return false;
    }
    ring.add(size + size, size + size, [pos[0], (pos[1] - drop) + 0.025, pos[2]], n, [0.0, 0.0, 1.0], records, record);
    true
}

/// The explosion's scorch: `FUN_001eb078(8.0, .., record 5)` [M `FUN_0015a098`].
pub const SCORCH: (f32, usize) = (8.0, 5);
/// The Q-Slick's oil: `FUN_001eb078(4.2, .., record 4)` [M `FUN_0020de20` case 10].
pub const OIL_SLICK: (f32, usize) = (4.2, 4);

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::trackfx::EfEntry;

    fn gk(frame: i32, gtype: u32, size: f32, jitter: u32) -> GiottoGlareKey { GiottoGlareKey { offset: [0.0, 1.0, 0.0], frame, gtype, size, jitter } }

    /// gall1 `Wd_Sprk` 'glare' frames 0, 5, 10, 12, 14, 22, 300 [D].
    fn sprk_keys() -> Vec<GiottoGlareKey> {
        vec![gk(0, 43, 0.0, 0), gk(5, 0, 0.5, 0), gk(10, 1, 0.0, 0), gk(12, 1, 0.2, 0), gk(14, 1, 0.5, 0), gk(22, 1, 0.0, 0), gk(300, 0, 0.0, 0)]
    }

    #[test]
    fn glare_key_pair_follows_fun_00233df8() {
        let k = sprk_keys();
        assert_eq!(glare_key_pair(&k, 0), (0, 0, 0, 0));
        assert_eq!(glare_key_pair(&k, 3), (0, 0, 1, 5));
        assert_eq!(glare_key_pair(&k, 5), (0, 0, 1, 5), "an exact key frame stays on the earlier pair");
        assert_eq!(glare_key_pair(&k, 6), (1, 5, 2, 10));
        assert_eq!(glare_key_pair(&k, 400), (6, 300, 6, 300), "past the end: the last key twice");
        let mut g = Vec::new();
        let mut r = giotto_rng();
        glare_element(&k, 3, [10.0, 0.0, 0.0], &mut r, &mut g);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].gtype, 43, "the type of the earlier key");
        assert!((g[0].size - 0.3).abs() < 1e-6, "0 + (0.5 - 0) x 3/5");
        assert_eq!(g[0].pos, [10.0, 1.0, 0.0]);
        glare_element(&k, 8, [0.0; 3], &mut r, &mut g);
        assert!((g[1].size - 0.2).abs() < 1e-6 && g[1].gtype == 0, "0.5 + (0 - 0.5) x 3/5");
    }

    #[test]
    fn jittered_glare_makes_three_from_the_giotto_stream() {
        let k = [gk(0, 7, 2.0, 1)];
        let mut r = giotto_rng();
        let mut want = giotto_rng();
        let mut g = Vec::new();
        glare_element(&k, 0, [0.0; 3], &mut r, &mut g);
        assert_eq!(g.len(), 3);
        for q in &g {
            let j: Vec<f32> = (0..3).map(|_| { let b = want.bits() as f32; (b + b) * R16 - 1.0 }).collect();
            assert!((q.pos[0] - j[0]).abs() < 1e-6 && (q.pos[1] - (j[1] + 1.0)).abs() < 1e-6 && (q.pos[2] - j[2]).abs() < 1e-6);
        }
        assert_eq!(r.state, want.state);
        assert_ne!(g[0].pos, g[1].pos);
    }

    #[test]
    fn giotto_stream_seed_is_175_steps() {
        // 0x642f2b % 500 = 175.
        let mut s = 0x3ade_68b1u32;
        for _ in 0..175 { s = s.wrapping_mul(0x075b_cd15) & 0xffff; }
        assert_eq!(giotto_rng().state, s);
        assert_ne!(giotto_rng().state, DriveRng::seeded(0).state);
    }

    fn burst_key(size: f32, kind: u32) -> GiottoBurst {
        GiottoBurst { quat: [0.0, 0.0, 0.0, 1.0], size, up: 0.015, up_spread: 0.012, side_spread: 0.012, argb: 0xff30_3030, kind, spread: [0.0; 3], piece_size: f32::from_bits(0x3ba0_902e), gate: 100 }
    }

    #[test]
    fn burst_count_and_types() {
        // Wd_Glass 'debris': size 15, kind 3 -> 7 pieces alternating types 5, 6 [D gall1].
        let (mut g, mut r, mut d) = (giotto_rng(), DriveRng::seeded(0), DebrisPool::default());
        burst(&burst_key(15.0, 3), [0.0; 3], [0.0, 0.0, 0.0, 1.0], &mut g, &mut r, &mut d, 10);
        assert_eq!((d.types[5].pieces.len(), d.types[6].pieces.len()), (4, 3));
        // Size 16 is past the cap rule: 32 pieces.
        let mut d = DebrisPool::default();
        burst(&burst_key(16.0, 2), [0.0; 3], [0.0, 0.0, 0.0, 1.0], &mut g, &mut r, &mut d, 10);
        assert_eq!(d.count(), 32);
        assert_eq!((d.types[0].pieces.len(), d.types[1].pieces.len(), d.types[2].pieces.len()), (11, 11, 10));
        // Size 2.5 -> 5 pieces; Sm_Wsplinter kind 14 -> wood splinters (type 17).
        let mut d = DebrisPool::default();
        burst(&burst_key(2.5, 14), [0.0; 3], [0.0, 0.0, 0.0, 1.0], &mut g, &mut r, &mut d, 10);
        assert_eq!(d.types[17].pieces.len(), 5);
        assert_eq!(DEBRIS_MODELS[17], "wdsp");
        assert_eq!(burst_debris_type(20, 0), None);
        assert_eq!(burst_debris_type(1, 0), Some(3));
        assert_eq!(burst_debris_type(19, 0), Some(22));
    }

    #[test]
    fn burst_velocity_uses_the_giotto_stream_and_spawn_the_engine_stream() {
        let b = burst_key(12.0, 2);
        let (mut g, mut r, mut d) = (giotto_rng(), DriveRng::seeded(3), DebrisPool::default());
        let (mut gw, mut rw) = (giotto_rng(), DriveRng::seeded(3));
        burst(&b, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0], &mut g, &mut r, &mut d, 50);
        let p = d.types[0].pieces[0];
        let (xz, yv, up) = (0.012 * 12.0, 0.012 * 12.0, 0.015 * 12.0);
        let v = [gw.bits() as f32, gw.bits() as f32, gw.bits() as f32];
        let want = [v[0] * xz * R16 - xz * 0.5, (up + v[1] * yv * R16) - yv * 0.5, v[2] * xz * R16 - xz * 0.5];
        for k in 0..3 { assert!((p.vel[k] - want[k]).abs() < 1e-6, "{k}"); }
        assert_eq!(p.pos, [1.0, 2.0, 3.0]);
        assert_eq!(p.birth, 50.0);
        assert_eq!(p.rgba, argb_to_gs(0xff30_3030));
        let a = [rw.bits(), rw.bits(), rw.bits()].map(|q| (q as f32 + q as f32) * R16 - 1.0);
        let l = len(a);
        for k in 0..3 { assert!((p.axis[k] - a[k] / l).abs() < 1e-6); }
        assert!((p.spin - rw.bits() as f32 * R16 * 0.089_759_79).abs() < 1e-7);
        assert_eq!(d.types[0].size, f32::from_bits(0x3ba0_902e));
    }

    #[test]
    fn kind_zero_burst_is_one_shell_piece_with_double_converted_colour() {
        let mut b = burst_key(12.0, 0);
        b.argb = 0x25ff_ffff;
        let (mut g, mut r, mut d) = (giotto_rng(), DriveRng::seeded(0), DebrisPool::default());
        let before = g.state;
        burst(&b, [0.0; 3], [0.0, 0.0, 0.0, 1.0], &mut g, &mut r, &mut d, 1);
        assert_eq!(d.types[4].pieces.len(), 1);
        assert_eq!(g.state, before, "kind 0 draws nothing from the Giotto stream");
        let p = d.types[4].pieces[0];
        assert!((p.vel[1] - (0.012 * 12.0 + 0.018)).abs() < 1e-6 && p.vel[0] == 0.0);
        assert_eq!(p.rgba, argb_to_gs(argb_to_gs(0x25ff_ffff)));
    }

    #[test]
    fn debris_fade_and_expiry_follow_fun_001d9920() {
        let mut d = DebrisPool::default();
        let mut r = DriveRng::seeded(0);
        for t in [1000u32, 1010, 1020] {
            d.spawn(0.1, 0, [0.0; 3], [0.0; 3], 0xff00_0000, 1, t, &mut r);
        }
        d.spawn(0.1, 4, [0.0; 3], [0.0; 3], 0xff00_0000, 1, 1000, &mut r);
        assert!(!d.spawn(0.1, 0, [0.0; 3], [0.0; 3], 0xff00_0000, 0, 1000, &mut r), "gate 0");
        // GS alpha of 0xff ARGB is 0x7f. At now 1075: cut 1005, the piece born at 1000 goes.
        d.update(1075.0);
        assert_eq!(d.types[0].pieces.len(), 2);
        assert_eq!(d.types[4].pieces.len(), 1, "type 4 is never walked");
        let born: Vec<f32> = d.types[0].pieces.iter().map(|p| p.birth).collect();
        assert_eq!(born, [1020.0, 1010.0], "the last piece moved into the freed slot");
        // alpha = round(0x7f x 1005 / 1020) = 125, round(0x7f x 1005/1010) = 126.
        assert_eq!(d.types[0].pieces[0].rgba >> 24, 125);
        assert_eq!(d.types[0].pieces[1].rgba >> 24, 126);
        d.update(1075.0);
        assert_eq!(d.types[0].pieces[0].rgba >> 24, 123, "the fade compounds: round(125 x 1005/1020)");
        let mut full = DebrisPool::default();
        for _ in 0..70 { full.spawn(0.1, 1, [0.0; 3], [0.0; 3], 0, 1, 5, &mut r); }
        assert_eq!(full.types[1].pieces.len(), DEBRIS_CAP);
    }

    fn entry(kind: u8, flags: u32, id: u8, name: Option<&str>) -> PlacedEffect {
        let mut b = vec![0u8; 0x40];
        b[0x10..0x14].copy_from_slice(&flags.to_le_bytes());
        b[0x14] = kind;
        b[0x17] = id;
        PlacedEffect {
            item: 1, article: "car".into(), part: 0, source_part: 0, entry: 0, scene_object: true, e: EfEntry::parse(&b),
            name: name.map(|n| format!("GIOT::{n}")), world: [0.0; 3], world_dir: [0.0; 3], item_origin: [0.0; 3], item_word15: 0, sub_bit7: false, sub_index: 0,
        }
    }

    #[test]
    fn block_keeps_what_fun_001a1bc8_keeps() {
        let es = [entry(6, 0, 1, Some("Wd_Glass")), entry(2, 0x14, 2, None), entry(6, 0x0402_200a, 3, Some("Ex_Glass")), entry(6, 0x8000_0002, 4, Some("Cr_Dmg4"))];
        let b = EffectBlock::from_entries(&es);
        assert_eq!(b.entries.len(), 3, "flags 0 is dropped");
        assert_eq!((b.damage, b.events, b.giotto), (0b010, 0b111, 0b110));
        assert_eq!(b.entries[1].mask, 0x0402);
        let mut s = es[1].clone();
        s.sub_bit7 = true;
        assert!(!s.scene_kept(), "bit 7 sub-items need flag 0x200");
        s.e.flags |= 0x200;
        assert!(s.scene_kept());
    }

    #[test]
    fn events_start_and_stop_by_mask_id_and_damage_bits() {
        let es = [entry(2, 0x14, 1, None), entry(6, 0x0402_200a, 3, Some("Ex_Glass")), entry(6, 0x8000_0002, 4, Some("Cr_Dmg4")), entry(6, 0x4000_0002, 5, Some("Cr_Dmg3"))];
        let mut b = EffectBlock::from_entries(&es);
        let mut next = 0usize;
        let mut start = |_: &str, _: [f32; 3]| { next += 1; Some(next) };
        assert!(b.build(false, &mut start).is_empty());
        assert_eq!(b.running, 0b0001, "the glare with flag 0x10 is on from the build");
        // Event 0 with bit 31: Cr_Dmg4 (mask 0x8000) only.
        let r = b.event(0, 0x8000_0000, 100, &mut start);
        assert_eq!(r, vec![BlockRequest::Start { entry: 2, name: "Cr_Dmg4".into(), pos: [0.0; 3] }]);
        assert_eq!(b.running, 0b0101);
        assert_eq!(b.deadline, 700);
        // Event 9 with panel bit 1 (0x0002): Ex_Glass's mask 0x0402 meets it.
        let r = b.event(9, 0x0002, 110, &mut start);
        assert_eq!(r.len(), 1);
        assert!(b.running & 0b0010 != 0);
        assert!(b.event(9, 0x0002, 111, &mut start).is_empty(), "no new bits: nothing");
        // Event 3 by id 4 stops Cr_Dmg4.
        let r = b.event(3, 4, 120, &mut start);
        assert_eq!(r, vec![BlockRequest::Stop { entry: 2, handle: 1 }]);
        // Out of range past the deadline: running Giotto entries without 0x14 stop.
        assert!(b.range_check(500, 1e9, &mut start).is_empty(), "before the deadline");
        assert!(b.range_check(720, 100.0, &mut start).is_empty() && b.deadline == 1320, "in range: 600 more");
        let r = b.range_check(1400, 1e9, &mut start);
        assert_eq!(r, vec![BlockRequest::Stop { entry: 1, handle: 2 }]);
        assert_eq!(b.running, 0b0001, "the glare stays");
    }

    #[test]
    fn fire_and_forget_effects_stay_unmarked() {
        let es = [entry(6, 0x0000_0002, 1, Some("Wd_Spray"))];
        let mut b = EffectBlock::from_entries(&es);
        let mut none = |_: &str, _: [f32; 3]| None;
        let r = b.event(0, 0, 5, &mut none);
        assert_eq!(r.len(), 1, "mask 0 matches an arg with an empty high half");
        assert_eq!(b.running, 0);
        assert_eq!(b.event(0, 0, 6, &mut none).len(), 1, "asked again: no handle was kept");
        assert_eq!(b.deadline, 0);
    }

    #[test]
    fn smackable_and_panel_thresholds() {
        assert_eq!(smackable_hit(10.0, 0.1, 1.0), (9.0, vec![(10, 0xffff), (9, 0xffff)]));
        assert_eq!(smackable_hit(10.0, 0.1, 3.0), (7.0, vec![(10, 0xffff)]), "0.7 is not above 0.75");
        assert_eq!(smackable_hit(10.0, 0.1, 9.0), (1.0, vec![]), "0.1 is not above 0.15");
        assert_eq!(smackable_hit(10.0, 0.1, 20.0), (0.0, vec![(10, 0xffff)]));
        assert_eq!(smackable_hit(0.0, 0.1, 1.0), (0.0, vec![]));
        assert_eq!(smackable_hit(10.0, 0.1, -5.0).0, 10.0);
        let mut p = [[0.0f32; 2]; 16];
        p[0] = [0.8, 0.1];
        p[3] = [0.1, 0.5];
        p[15] = [0.75, 0.75];
        assert_eq!(car_panel_events(&p), (0b1000_0000_0000_1001, 0b1));
    }

    #[test]
    fn explosion_choice() {
        let mut c = 0;
        let a = ExplosionArgs { weapon: 2, damage: 300.0, ..Default::default() };
        assert_eq!(explosion_effect(&a, 0, &mut c), "Ex_Small");
        assert_eq!(explosion_effect(&a, 0, &mut c), "Ex_Small2");
        assert_eq!(explosion_effect(&ExplosionArgs { hit_player: true, ..a.clone() }, 0, &mut c), "Ex_SmB02");
        assert_eq!(c, 0);
        assert_eq!(explosion_effect(&ExplosionArgs { surface: 3, ..a.clone() }, 7, &mut c), "Ex_SmallD2");
        assert_eq!(explosion_effect(&ExplosionArgs { surface: 9, ..a.clone() }, 0, &mut c), "snowspecificexplosion");
        assert_eq!(explosion_effect(&ExplosionArgs { surface: 6, ..a.clone() }, 0, &mut c), "rocketsplash");
        assert_eq!(explosion_effect(&ExplosionArgs { weapon: 0x1e, surface: 9, ..a.clone() }, 0, &mut c), "FODHitEffect");
        assert_eq!(explosion_effect(&ExplosionArgs { underwater: true, ..a.clone() }, 0, &mut c), "Un_ExplM");
        assert_eq!(explosion_effect(&ExplosionArgs { underwater: true, damage: 601.0, ..a.clone() }, 0, &mut c), "Un_ExplL");
        assert_eq!(explosion_effect(&ExplosionArgs { underwater: true, damage: 200.0, ..a.clone() }, 0, &mut c), "Un_ExplS");
        assert!(projectile_ground_hit(0xe, 100.0, 0, false).is_none());
        assert_eq!(projectile_ground_hit(1, 100.0, 4, false).unwrap().surface, 4);
        assert_eq!(static_explosion(0, Some("Wd_Glass"), 4, 0), StaticExplosion::Start("Wd_Glass".into()));
        assert_eq!(static_explosion(0, None, 4, 9), StaticExplosion::Start("Ex_Snow".into()));
        assert_eq!(static_explosion(0x14, None, 4, 0), StaticExplosion::Event1 { id: 4 });
    }

    #[test]
    fn explosion_sound_rotation_and_cooldown() {
        let (mut i, mut until) = (0u32, [0i32; 4]);
        let s = explosion_sound(10, &mut i, &mut until, "paris_mis01").unwrap();
        assert_eq!((s.name, s.near, s.far), ("SFX_HugeExp", 100.0, 400.0));
        assert_eq!(until[0], 40);
        assert_eq!(explosion_sound(11, &mut i, &mut until, "paris_mis01").unwrap().name, "SFX_LrgExp", "0 is cooling down");
        explosion_sound(11, &mut i, &mut until, "x");
        explosion_sound(11, &mut i, &mut until, "x");
        assert!(explosion_sound(12, &mut i, &mut until, "x").is_none(), "all four cooling down");
        assert_eq!(explosion_sound(41, &mut i, &mut until, "JungleB_mis13b").unwrap().far, 2000.0);
    }

    fn records() -> Vec<Decal> {
        (0..8).map(|k| Decal { corners: [[0.0, 0.0, 1.0, 1.0], [1.0, 0.0, 1.0, 1.0], [0.0, 1.0, 1.0, 1.0], [1.0, 1.0, 1.0, 1.0]], w0: 0, w1: if k == 5 { 3 } else { 0 }, w2: 0x28, index: k }).collect()
    }

    #[test]
    fn decal_ring_corners_fade_in_and_wrap() {
        let mut ring = DecalRing::new();
        let rec = records();
        ring.add(2.0, 4.0, [10.0, 0.0, 10.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], &rec, 5);
        let q = ring.slots[0].unwrap();
        // a = (z x y) 2 = (-2, 0, 0); b = (a0 x y) 4 = (0, 0, -4); c0 = centre - a/2 - b/2.
        assert_eq!(q.verts[0], [11.0, 0.0, 12.0]);
        assert_eq!(q.verts[1], [9.0, 0.0, 12.0]);
        assert_eq!(q.verts[2], [11.0, 0.0, 8.0]);
        assert_eq!(q.verts[5], [9.0, 0.0, 8.0]);
        assert_eq!(q.uv[3], rec[5].corners[1]);
        assert_eq!((q.delay, q.rgba), (3, 0x007f_7f7f));
        ring.draw_tick();
        ring.draw_tick();
        assert_eq!(ring.slots[0].unwrap().rgba, 0x007f_7f7f);
        ring.draw_tick();
        assert_eq!(ring.slots[0].unwrap().rgba, 0x287f_7f7f, "alpha after the third draw");
        ring.draw_tick();
        assert_eq!(ring.slots[0].unwrap().rgba, 0x287f_7f7f);
        for _ in 0..96 { ring.add(1.0, 1.0, [0.0; 3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], &rec, 0); }
        assert_eq!(ring.live(), 96);
        assert_eq!(ring.slots[0].unwrap().record, 0, "the 97th add overwrote slot 0");
        assert_eq!(ring.next, 1);
        let mut d = ring.clone();
        d.add(1.0, 1.0, [0.0; 3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], &rec, 0);
        assert_eq!(d.slots[1].unwrap().delay, 1, "a zero delay word means 1 draw");
    }

    fn flat(h: f32) -> impl Fn([f32; 3]) -> GroundHit {
        move |p| GroundHit { hit: 1, tri: [[p[0], h, p[2]], [p[0] + 1.0, h, p[2]], [p[0], h, p[2] + 1.0]], height: h, surface: 1, no_decal: false, on_object: false }
    }

    #[test]
    fn ground_decal_size_and_refusals() {
        let rec = records();
        let mut ring = DecalRing::new();
        assert!(ground_decal(&mut ring, &rec, 8.0, [0.0, 1.0, 0.0], 5, flat(0.0)));
        let q = ring.slots[0].unwrap();
        // r = 4, size = 4 - 1/2 = 3.5, square of 7 at y 0.025 (x 0.9999: the capped normal is
        // not renormalised).
        assert!((len(sub(q.verts[1], q.verts[0])) - 7.0 * 0.9999).abs() < 1e-4);
        assert!((q.verts[0][1] - 0.025).abs() < 1e-6);
        assert!(!ground_decal(&mut ring, &rec, 8.0, [0.0, 9.0, 0.0], 5, flat(0.0)), "too high above the ground");
        let water = |p: [f32; 3]| GroundHit { surface: 6, ..flat(0.0)(p) };
        assert!(!ground_decal(&mut ring, &rec, 8.0, [0.0, 1.0, 0.0], 5, water));
        let step = |p: [f32; 3]| { let h = if p[0] > 1.0 { 0.5 } else { 0.0 }; GroundHit { height: h, ..flat(h)(p) } };
        assert!(!ground_decal(&mut ring, &rec, 8.0, [0.0, 1.0, 0.0], 5, step), "a step under one probe");
        assert!(ground_decal(&mut ring, &rec, 8.0, [0.0, 1.0, 0.0], 4, step), "oil skips the flatness probes");
        let slope = |p: [f32; 3]| GroundHit { tri: [[0.0, 0.0, 0.0], [1.0, 0.5, 0.0], [0.0, 0.0, 1.0]], ..flat(0.0)(p) };
        assert!(!ground_decal(&mut ring, &rec, 8.0, [0.0, 1.0, 0.0], 4, slope), "normal y below 0.95");
    }

    #[test]
    fn giotto_run_frames_and_elements() {
        let eff = GiottoEffect {
            name: "Wd_Test".into(),
            flags: 1,
            nump: 0,
            life: 3,
            emitters: vec![],
            elements: vec![
                GiottoElement::Glare(vec![gk(0, 5, 1.0, 0), gk(4, 5, 0.0, 0)]),
                GiottoElement::Burst(burst_key(2.0, 2)),
                GiottoElement::Sound(GiottoSound { volume: 100, bank: 2, name: "SFX_GlassShatter3".into(), w5c: 100, near: 20, far: 300 }),
                GiottoElement::Sound(GiottoSound { volume: 100, bank: 2, name: "(null)".into(), w5c: 0, near: 0, far: 0 }),
            ],
            others: vec![],
        };
        let (mut lib, mut types) = (ParticleLib::default(), GiottoTypes::default());
        let mut run = GiottoRun::new(&eff, [0.0; 3], [0.0, 0.0, 0.0, 1.0], &mut types, &mut lib);
        let (mut r, mut g, mut pool, mut d) = (DriveRng::seeded(0), giotto_rng(), ParticleSystem::default(), DebrisPool::default());
        let f0 = run.tick(1, [0.0; 3], &lib, &mut r, &mut g, &mut pool, &mut d, 1);
        assert_eq!(d.count(), 4, "the burst ran once at frame 0");
        assert_eq!(f0.sounds.len(), 1, "(null) makes no voice");
        assert_eq!((f0.sounds[0].volume, f0.sounds[0].far, f0.sounds[0].group), (1.0, 300.0, "Giotto"));
        assert_eq!(f0.glares[0].size, 1.0);
        let f1 = run.tick(1, [0.0; 3], &lib, &mut r, &mut g, &mut pool, &mut d, 2);
        assert_eq!(d.count(), 4);
        assert!(f1.sounds.is_empty() && f1.sound_moves.len() == 1);
        assert!((f1.glares[0].size - 0.75).abs() < 1e-6);
        let f2 = run.tick(1, [0.0; 3], &lib, &mut r, &mut g, &mut pool, &mut d, 3);
        assert!(!run.ended && f2.glares.len() == 1);
        let f3 = run.tick(1, [0.0; 3], &lib, &mut r, &mut g, &mut pool, &mut d, 4);
        assert!(run.ended && f3.glares.is_empty() && f3.sound_moves.is_empty(), "the run at LIFE has the end bit set first");
        let f4 = run.tick(1, [0.0; 3], &lib, &mut r, &mut g, &mut pool, &mut d, 5);
        assert!(f4.glares.is_empty());
        // Flgs bit 0 clear: the run frame stays 0, so the burst repeats every tick.
        let mut looped = GiottoRun::new(&GiottoEffect { flags: 0, ..eff.clone() }, [0.0; 3], [0.0, 0.0, 0.0, 1.0], &mut types, &mut lib);
        let mut d = DebrisPool::default();
        for t in 0..10 {
            looped.tick(1, [0.0; 3], &lib, &mut r, &mut g, &mut pool, &mut d, t);
        }
        assert!(!looped.ended && looped.run_frame == 0 && looped.key_frame == 10);
        assert_eq!(d.count(), 40);
    }

    #[test]
    fn quat_mul_is_hamilton() {
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let z90 = [0.0, 0.0, h, h];
        let q = quat_mul(z90, z90);
        assert!((q[2] - 1.0).abs() < 1e-6 && q[3].abs() < 1e-6, "{q:?}");
        let x90 = [h, 0.0, 0.0, h];
        let q = quat_mul(z90, x90);
        assert!((q[0] - 0.5).abs() < 1e-6 && (q[1] - 0.5).abs() < 1e-6 && (q[2] - 0.5).abs() < 1e-6 && (q[3] - 0.5).abs() < 1e-6, "{q:?}");
    }
}
