//! The driving tracks' placed effects, per tick: glare sprites (street and wall lamps, the
//! flashing red traffic lights, the sawhorse beacons, building lights) and the particle
//! effects the static world starts at load (chimney smoke, steam, mist, fire). The data is
//! `nf_formats::trackfx` (docs/systems/driving-effects.md §7); this is logic only and draws
//! nothing.
//!
//! Ported from DRIVING.ELF [M]:
//! - the glare instance `FUN_001f1d18`: pulse (`FUN_001f1b60`, `effects::glare_pulse`), the size
//!   cull `lod² / (size × pulse) <= 1.6` (`effects::glare_size`; static items pass the squared
//!   level-of-detail distance, scene objects 0 [M `FUN_001ad1a0`, `FUN_001cab38`]), the
//!   one-sided fade (flag 0x20: size × −dot(camera row 2, direction) − 0.5, dropped at <= 0,
//!   then doubled), at most 128 a frame;
//! - the glare draw build `FUN_001f2058`: two sprites per instance (halo, spike) from the type
//!   table, each pulled towards the camera by `ZBias × size`, width `size × 1.4`, colour = the
//!   type colour's low 24 bits with alpha 0x20 (flag 0x20: `trunc(size × 32)`), rotation
//!   `frac(spin × distance / 2π) × 2π − 3.93` (the fraction by truncation);
//! - the Giotto particle emitter `FUN_002360c0`: the type added once per element
//!   (`FUN_00235ed0`, key `n + 5`), the frame at instance position + key offset, axes from the
//!   key quaternion, the emitter distance = |frame − camera|, a forever-type emitter switched on
//!   at the instance's first frame (`FUN_001e8f30(e, 1)`, age 0); the instance runs its
//!   elements while its frame count <= LIFE (`FUN_002345d8`).
//!
//! Not ported or not traced (stated, not guessed):
//! - what feeds the static glare's LOD distance: `FUN_001ad0b8` gives
//!   `max(0, |camera − item origin| − radius) × view[+0x30]`; the view word is not traced, so
//!   callers pass it ([`TrackFx::glares`] `lod_scale`);
//! - which way the camera's row 2 points (the one-sided test uses it as given);
//! - the Giotto `gg` (animated glare), `gd` (debris/decal bursts) and `gs` (sound) elements,
//!   and every scene-object effect that starts on an object event (smash sparks, glass);
//! - how the particle manager ticks the emitters it owns (`FUN_001e65c0` has no direct caller;
//!   it is reached through a vtable): here every emitter ticks once per call, like the vehicle
//!   slots; the tick rate of the instance frame counter is not traced either.

use nf_formats::drivefx::{glare_cell, Glare, GLARE_TYPES};
use nf_formats::trackfx::{eflag, GiottoEffect, PlacedEffect};

use super::effects::{glare_pulse, glare_size, DriveRng, Emitter, EmitterFrame, ParticleLib, ParticleSystem};

/// One glare type as the draw reads it: the 0x40-byte entry at manager + 0x2038 + 0x40 × id
/// [M `FUN_001f14b0` tuner offsets, `FUN_001f19f0` cells, `FUN_001f2058` reads].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlareType {
    /// +0x00 / +0x10: (u, v, size) of the halo / spike cell on the `flar` sheet.
    pub cell: [[f32; 3]; 2],
    /// +0x0c / +0x1c: `HaloSpin` / `SpikeSpin`.
    pub spin: [f32; 2],
    /// +0x20 / +0x24: the colour words (`FUN_00144740`: "a,b,c,d" → a<<24 | d<<16 | c<<8 | b).
    pub colour: [u32; 2],
    /// +0x28 / +0x2c: `HaloSize` / `SpikeSize`.
    pub size: [f32; 2],
    /// +0x38: `ZBias` (2.0 at init; 0.3 for ids 33, 34, 52 [M `FUN_001f14b0`]; every id of the
    /// disc's tuning sets it [D]).
    pub zbias: f32,
}

/// `FUN_00144740`'s colour word from the four file values [M, docs/formats/geoprim.md §7].
pub fn colour_word(c: [u8; 4]) -> u32 { (c[0] as u32) << 24 | (c[3] as u32) << 16 | (c[2] as u32) << 8 | c[1] as u32 }

/// The type table from the parsed `Render\Glare\default.tun`.
pub fn glare_types(g: &[Glare]) -> Vec<GlareType> {
    (0..GLARE_TYPES)
        .map(|i| {
            let t = g.get(i).copied().unwrap_or_default();
            let cell = |k: usize| {
                let (u, v, s) = glare_cell(t.tex[k]);
                [u, v, s]
            };
            GlareType { cell: [cell(0), cell(1)], spin: t.spin, colour: [colour_word(t.col[0]), colour_word(t.col[1])], size: t.size, zbias: t.zbias }
        })
        .collect()
}

/// One sprite the glare draw uploads [M `FUN_001f2058`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlareSprite {
    /// 0 = halo, 1 = spike (the halo goes first).
    pub layer: u8,
    pub centre: [f32; 3],
    /// The `w` word: size × 1.4.
    pub width: f32,
    /// GS RGBA: R in the low byte, alpha (0x80 = 1.0) in the top byte.
    pub rgba: u32,
    /// (u, v, size) on the `flar` sheet.
    pub cell: [f32; 3],
    pub rotation: f32,
}

/// The glare bucket's GS state [M `FUN_001f14b0`: `DAT_00339050 = 0x8000000048` (ALPHA: Cs×As +
/// Cd, FIX 0x80), `FUN_001a0f30(…, 0x70)` (TEST: alpha test GEQUAL 0x70, AFAIL = RGB only)].
pub const GLARE_GS_ALPHA: u64 = 0x80_0000_0048;

/// A placed glare (type 2 entry) in world space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackGlare {
    pub item: usize,
    pub pos: [f32; 3],
    pub dir: [f32; 3],
    pub flags: u32,
    pub gtype: u32,
    pub size: f32,
    pub phase: u32,
    pub rate: f32,
    pub duty: f32,
    pub base: f32,
    pub amp: f32,
    /// A static item (distance-culled) rather than a scene object (distance 0).
    pub culled_by_distance: bool,
}

/// A Giotto particle effect started at load on a static item.
#[derive(Clone, Debug)]
pub struct GiottoInstance {
    pub item: usize,
    pub name: String,
    /// Item origin + entry offset [M `FUN_00233460`].
    pub pos: [f32; 3],
    pub life: u32,
    /// `+0x5c`: frames run.
    pub frame: u32,
    pub emitters: Vec<GiottoSlot>,
}

/// One `gp` element of an instance: its frame and emitter.
#[derive(Clone, Debug)]
pub struct GiottoSlot {
    pub key: u32,
    pub offset: [f32; 3],
    pub quat: [f32; 4],
    pub start_frame: i32,
    pub start_override: i32,
    pub frame: EmitterFrame,
    pub emitter: Option<Emitter>,
}

/// Rotate `v` by the unit quaternion (x, y, z, w) [M `FUN_002cbc00` / `FUN_002cbcf8`; the
/// handedness is the standard q v q*, not checked against a non-identity case on the disc].
pub fn quat_rotate(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let t = [2.0 * (y * v[2] - z * v[1]), 2.0 * (z * v[0] - x * v[2]), 2.0 * (x * v[1] - y * v[0])];
    [
        v[0] + w * t[0] + (y * t[2] - z * t[1]),
        v[1] + w * t[1] + (z * t[0] - x * t[2]),
        v[2] + w * t[2] + (x * t[1] - y * t[0]),
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn len(a: [f32; 3]) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }

/// The track's effects, built once from the decoded placements.
#[derive(Clone, Debug, Default)]
pub struct TrackFx {
    pub glares: Vec<TrackGlare>,
    pub giotto: Vec<GiottoInstance>,
    /// The item origins and LOD radii the static glare distance uses (by item index).
    pub item_lod: Vec<(usize, [f32; 3], f32)>,
}

/// `FUN_001ad128`'s radius from item word 15: `(w & 0x3ff) × {0.25, 16}[bit 30]` (table
/// 0x381170) [M]; no 10.0 fallback on this path.
pub fn lod_radius(word15: u32) -> f32 { (word15 & 0x3ff) as f32 * if word15 & 0x4000_0000 != 0 { 16.0 } else { 0.25 } }

/// What one frame of the glare pass produced: the instances kept and their sprites.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlareFrame {
    /// (glare index into `TrackFx::glares`, kept size).
    pub kept: Vec<(usize, f32)>,
    pub sprites: Vec<GlareSprite>,
}

impl TrackFx {
    /// Build from `nf_formats::trackfx::placed_effects`. `effects(name)` resolves a Giotto name
    /// in the track's gallery. Particle types are added to `lib` once per element, under keys
    /// n + 5.
    pub fn new(placed: &[PlacedEffect], effects: impl Fn(&str) -> Option<GiottoEffect>, lib: &mut ParticleLib) -> TrackFx {
        let mut fx = TrackFx::default();
        let mut added = 0u32;
        let mut cache: Vec<(String, Vec<u32>)> = Vec::new();
        for p in placed {
            if p.e.kind == nf_formats::trackfx::kind::GLARE && (p.scene_object || p.static_active()) {
                fx.glares.push(TrackGlare {
                    item: p.item,
                    pos: p.world,
                    dir: p.world_dir,
                    flags: p.e.flags,
                    gtype: p.e.glare_type(),
                    size: p.e.size,
                    phase: p.e.phase,
                    rate: p.e.rate,
                    duty: p.e.duty,
                    base: p.e.base,
                    amp: p.e.amp,
                    culled_by_distance: !p.scene_object,
                });
                if !p.scene_object && !fx.item_lod.iter().any(|(i, _, _)| *i == p.item) {
                    fx.item_lod.push((p.item, p.item_origin, lod_radius(p.item_word15)));
                }
            } else if p.starts_at_load() {
                let Some(name) = p.giotto_name() else { continue };
                let Some(eff) = effects(name) else { continue }; // "Could not locate Giotto effect"
                // FUN_00235ed0 caches by element: one library type per gp element.
                let keys = match cache.iter().find(|(n, _)| n.eq_ignore_ascii_case(&eff.name)) {
                    Some((_, k)) => k.clone(),
                    None => {
                        let mut k = Vec::new();
                        for em in &eff.emitters {
                            let key = added + 5;
                            lib.add(key, &em.ptype);
                            added += 1;
                            k.push(key);
                        }
                        cache.push((eff.name.clone(), k.clone()));
                        k
                    }
                };
                let emitters = eff
                    .emitters
                    .iter()
                    .zip(keys)
                    .map(|(em, key)| GiottoSlot {
                        key,
                        offset: em.offset,
                        quat: em.quat,
                        start_frame: em.start_frame,
                        start_override: em.start_override,
                        frame: EmitterFrame::new(key),
                        emitter: None,
                    })
                    .collect();
                fx.giotto.push(GiottoInstance { item: p.item, name: eff.name.clone(), pos: p.world, life: eff.life, frame: 0, emitters });
            }
        }
        fx
    }

    /// The glare pass for one frame: `FUN_001f1d18` per placed glare, then `FUN_001f2058`.
    /// `drawn(item)` says whether the item is drawn this frame: the game reaches the glare only
    /// from an item's draw (`FUN_001ad1a0` / `FUN_001ca930`), and the 128 cap counts in that
    /// order; the world's view cull that decides it is not part of this module.
    /// `camera_row2` is the camera matrix's +0x20 row, `lod_scale` the view's +0x30 word (see
    /// the module notes).
    pub fn glares(&self, tick: u32, camera: [f32; 3], camera_row2: [f32; 3], lod_scale: f32, types: &[GlareType], drawn: impl Fn(usize) -> bool) -> GlareFrame {
        let mut out = GlareFrame::default();
        for (i, g) in self.glares.iter().enumerate() {
            if !drawn(g.item) {
                continue;
            }
            let lod = if g.culled_by_distance {
                match self.item_lod.iter().find(|(it, _, _)| *it == g.item) {
                    Some((_, o, r)) => (len(sub(camera, *o)) - r).max(0.0) * lod_scale,
                    None => 0.0,
                }
            } else {
                0.0
            };
            let pulse = (g.flags & eflag::PULSE != 0).then(|| glare_pulse(tick, g.phase, g.rate, g.duty, g.base, g.amp, g.flags));
            let Some(mut size) = glare_size(lod * lod, g.size, pulse, out.kept.len()) else { continue };
            if g.flags & eflag::ONE_SIDED != 0 {
                size = size * -dot(camera_row2, g.dir) - 0.5;
                if size <= 0.0 {
                    continue;
                }
                size += size;
            }
            out.kept.push((i, size));
        }
        for &(i, size) in &out.kept {
            let g = &self.glares[i];
            let Some(t) = types.get(g.gtype as usize) else { continue };
            let to = sub(g.pos, camera);
            let dist = len(to);
            let n = if dist == 0.0 { [0.0; 3] } else { [to[0] / dist, to[1] / dist, to[2] / dist] };
            let alpha: u32 = if g.flags & eflag::ONE_SIDED != 0 { ((size * 32.0).max(0.0) as u32) << 24 } else { 0x2000_0000 };
            for layer in 0..2 {
                let s = t.size[layer] * size;
                let k = -t.zbias * s;
                let f = t.spin[layer] * dist / 6.283185;
                out.sprites.push(GlareSprite {
                    layer: layer as u8,
                    centre: [g.pos[0] + n[0] * k, g.pos[1] + n[1] * k, g.pos[2] + n[2] * k],
                    width: s * 1.4,
                    rgba: (t.colour[layer] & 0x00ff_ffff) | alpha,
                    cell: t.cell[layer],
                    rotation: (f - f.trunc()) * 6.283185 - 3.93,
                });
            }
        }
        out
    }

    /// One frame of the Giotto instances started at load, then `frames` steps of their
    /// emitters into `pool`.
    pub fn tick_particles(&mut self, frames: i32, camera: [f32; 3], lib: &ParticleLib, rng: &mut DriveRng, pool: &mut ParticleSystem) {
        for inst in &mut self.giotto {
            if inst.frame <= inst.life {
                let fr = inst.frame as i32;
                for s in &mut inst.emitters {
                    // FUN_002360c0: before its start frame (and with no override) nothing happens.
                    if s.start_override == 0 && fr < s.start_frame {
                        continue;
                    }
                    // Orientation: the identity for a static item (FUN_00233460).
                    let off = s.offset;
                    s.frame.pos = [inst.pos[0] + off[0], inst.pos[1] + off[1], inst.pos[2] + off[2]];
                    s.frame.axis_a = quat_rotate(s.quat, [0.0, 1.0, 0.0]);
                    s.frame.axis_b = quat_rotate(s.quat, [1.0, 0.0, 0.0]);
                    let d = len(sub(s.frame.pos, camera));
                    let e = s.emitter.get_or_insert_with(|| Emitter::new(d, &s.frame, lib));
                    e.distance = d;
                    if fr == 0 || fr == s.start_frame {
                        if lib.types[e.ty].flags & nf_formats::drivefx::FLAG_FOREVER != 0 {
                            e.active = true;
                            e.age = 0;
                        }
                    }
                }
                inst.frame += 1;
            }
            for s in &mut inst.emitters {
                if let Some(e) = s.emitter.as_mut() {
                    e.tick(frames, &s.frame, lib, rng, pool);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::drivefx::ParticleType;
    use nf_formats::trackfx::{EfEntry, GiottoEmitter};

    /// Glare type 49 as the disc's tuning sets it [D]: halo tex 6, col 255,96,0,0, size 0.6;
    /// spike tex 8, col 255,200,20,5, size 0.3; zbias 2.
    fn type49() -> GlareType {
        glare_types(&[Glare { tex: [6, 8], spin: [0.0, 0.0], col: [[255, 96, 0, 0], [255, 200, 20, 5]], size: [0.6, 0.3], zbias: 2.0 }])[0]
    }

    fn traffic(pos: [f32; 3]) -> TrackGlare {
        TrackGlare { item: 0, pos, dir: [-0.843, 0.0, -0.537], flags: 0xc34, gtype: 0, size: 1.0, phase: 0, rate: 0.02, duty: 2.0 / 3.0, base: 0.5, amp: 1.0, culled_by_distance: false }
    }

    #[test]
    fn colour_word_follows_fun_00144740() {
        // "255,96,0,0" → 0xff000060: red 96 in the low byte (GS R).
        assert_eq!(colour_word([255, 96, 0, 0]), 0xff00_0060);
        assert_eq!(colour_word([255, 200, 20, 5]), 0xff05_14c8);
        assert_eq!(type49().cell, [[0.0, 0.75, 0.25], [0.5, 0.75, 0.25]], "tex 6 and 8 through FUN_001f19f0");
    }

    #[test]
    fn traffic_light_flashes_red_on_a_50_tick_triangle() {
        let fx = TrackFx { glares: vec![traffic([10.0, 2.0, 0.0])], ..Default::default() };
        let types = vec![type49()];
        // Camera row 2 pointing against the lamp's facing: -dot = 1.
        let row2 = [0.843, 0.0, 0.537];
        let cam = [0.0, 2.0, 0.0];
        // tick 0: pulse = 0.5 → size 0.5 → one-sided: 0.5*~1 - 0.5 ≈ 0 → dropped.
        assert!(fx.glares(0, cam, row2, 1.0, &types, |_| true).kept.is_empty());
        // tick 25: x = 0.5, triangle 2x = 1.0 → 0.5 + 1.0 = 1.5 (the peak).
        let f = fx.glares(25, cam, row2, 1.0, &types, |_| true);
        assert_eq!(f.kept.len(), 1);
        let d = 0.843f32 * 0.843 + 0.537 * 0.537;
        let want = (1.5 * d - 0.5) * 2.0;
        assert!((f.kept[0].1 - want).abs() < 1e-5, "{:?}", f.kept);
        // tick 34: x = 0.68 > duty 2/3 → off.
        assert!(fx.glares(34, cam, row2, 1.0, &types, |_| true).kept.is_empty());
        // tick 50 wraps to tick 0's value.
        assert_eq!(fx.glares(50, cam, row2, 1.0, &types, |_| true), fx.glares(0, cam, row2, 1.0, &types, |_| true));
        // Facing away: nothing.
        assert!(fx.glares(25, cam, [-0.843, 0.0, -0.537], 1.0, &types, |_| true).kept.is_empty());
        // The sprites: halo then spike, red halo, alpha = trunc(size*32).
        let s = &f.sprites;
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].rgba & 0xff_ffff, 0x60);
        assert_eq!(s[0].rgba >> 24, (want * 32.0) as u32);
        assert_eq!(s[1].rgba & 0xff_ffff, 0x05_14c8);
        assert!((s[0].width - 0.6 * want * 1.4).abs() < 1e-5);
        // Pulled towards the camera (at x = 0) by zbias * size = 2 * 0.6 * want.
        assert!((s[0].centre[0] - (10.0 - 2.0 * 0.6 * want)).abs() < 1e-4);
        assert!((s[0].rotation + 3.93).abs() < 1e-6, "spin 0");
    }

    #[test]
    fn static_glares_cull_on_the_squared_lod_distance() {
        let mut g = traffic([0.0; 3]);
        g.flags = 0x14; // street lamp: no pulse, not one-sided
        g.culled_by_distance = true;
        g.size = 1.0;
        let fx = TrackFx { glares: vec![g], giotto: vec![], item_lod: vec![(0, [0.0; 3], 0.5)] };
        let types = vec![type49()];
        // lod = |cam| - 0.5; kept while lod² <= 1.6.
        assert_eq!(fx.glares(0, [1.7, 0.0, 0.0], [0.0; 3], 1.0, &types, |_| true).kept.len(), 1, "lod 1.2: 1.44");
        assert!(fx.glares(0, [1.8, 0.0, 0.0], [0.0; 3], 1.0, &types, |_| true).kept.is_empty(), "lod 1.3: 1.69");
        let f = fx.glares(0, [1.7, 0.0, 0.0], [0.0; 3], 1.0, &types, |_| true);
        assert_eq!(f.sprites[0].rgba >> 24, 0x20, "two-sided alpha");
        // The 128 cap.
        let fx = TrackFx { glares: vec![TrackGlare { culled_by_distance: false, ..g }; 200], ..Default::default() };
        assert_eq!(fx.glares(0, [0.0; 3], [0.0; 3], 1.0, &types, |_| true).kept.len(), 128);
    }

    #[test]
    fn rotation_uses_the_truncated_fraction() {
        let mut t = type49();
        t.spin = [0.04, 0.0];
        let mut g = traffic([0.0, 0.0, 100.0]);
        g.flags = 0;
        let fx = TrackFx { glares: vec![g], ..Default::default() };
        let f = fx.glares(0, [0.0; 3], [0.0; 3], 1.0, &[t], |_| true);
        let x = 0.04f32 * 100.0 / 6.283185;
        assert!((f.sprites[0].rotation - ((x - x.trunc()) * 6.283185 - 3.93)).abs() < 1e-5);
    }

    #[test]
    fn quaternion_rotation() {
        let r = quat_rotate([0.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
        assert_eq!(r, [0.0, 1.0, 0.0]);
        // 90 degrees about z: y → -x.
        let h = std::f32::consts::FRAC_1_SQRT_2;
        let r = quat_rotate([0.0, 0.0, h, h], [0.0, 1.0, 0.0]);
        assert!((r[0] + 1.0).abs() < 1e-6 && r[1].abs() < 1e-6);
    }

    fn smoke_effect() -> GiottoEffect {
        let mut t = ParticleType::parse(&[0u8; 0xb0]).unwrap();
        t.count = 20;
        t.life = 40;
        t.delay = 0;
        t.size0 = 0.5;
        t.size1 = 1.0;
        t.speed = 0.02;
        t.drag = 1.0;
        GiottoEffect {
            name: "Wd_Smoke".into(),
            flags: 0,
            nump: 1,
            life: 3,
            emitters: vec![GiottoEmitter { element: "wrldsmok".into(), offset: [0.0, 1.0, 0.0], quat: [0.0, 0.0, 0.0, 1.0], ptype: t, links_previous: false, start_frame: 0, start_override: 0 }],
            others: vec![],
        }
    }

    fn placed_smoke() -> PlacedEffect {
        let mut b = vec![0u8; 0x40];
        b[0x14] = 6;
        b[0x10] = 0x10;
        b[0x18..0x1c].copy_from_slice(&0x7372_0001u32.to_le_bytes());
        PlacedEffect { item: 3, article: "P_Building".into(), part: 0, source_part: 0, entry: 0, scene_object: false, e: EfEntry::parse(&b), name: Some("GIOT::wd_smoke".into()), world: [5.0, 0.0, 5.0], world_dir: [0.0; 3], item_origin: [5.0, 0.0, 5.0], item_word15: 0 }
    }

    #[test]
    fn load_started_smoke_emits_from_item_plus_key_offset() {
        let mut lib = ParticleLib::default();
        let fx = TrackFx::new(&[placed_smoke(), placed_smoke()], |n| (n.eq_ignore_ascii_case("wd_smoke")).then(smoke_effect), &mut lib);
        assert_eq!(fx.giotto.len(), 2);
        assert_eq!(lib.types.len(), 1, "one library type per element, shared by both instances");
        assert_eq!(fx.giotto[0].emitters[0].key, 5, "the first Giotto type goes under key 0 + 5");
        let mut fx = fx;
        let (mut rng, mut pool) = (DriveRng::seeded(0), ParticleSystem::default());
        fx.tick_particles(1, [0.0; 3], &lib, &mut rng, &mut pool);
        let s = &fx.giotto[0].emitters[0];
        assert_eq!(s.frame.pos, [5.0, 1.0, 5.0]);
        assert_eq!(s.frame.axis_a, [0.0, 1.0, 0.0]);
        assert!((s.emitter.unwrap().distance - (51.0f32).sqrt()).abs() < 1e-5);
        // Rate count/life = 10/40 per step (count halved at load): 4 steps owe one particle each.
        for _ in 0..7 {
            fx.tick_particles(1, [0.0; 3], &lib, &mut rng, &mut pool);
        }
        assert!(!pool.next.is_empty(), "the emitters filled the pool");
        // The instance stops after LIFE (3) frames.
        assert_eq!(fx.giotto[0].frame, 4);
    }

    #[test]
    fn scene_glares_skip_the_distance_cull_and_static_types_6_without_0x10_do_not_start() {
        let mut lib = ParticleLib::default();
        let mut p = placed_smoke();
        p.e.flags = 0;
        let fx = TrackFx::new(&[p], |_| Some(smoke_effect()), &mut lib);
        assert!(fx.giotto.is_empty());
        let g = TrackGlare { flags: 0x14, ..traffic([1000.0, 0.0, 0.0]) };
        let fx = TrackFx { glares: vec![g], ..Default::default() };
        assert_eq!(fx.glares(0, [0.0; 3], [0.0; 3], 1.0, &[type49()], |_| true).kept.len(), 1);
    }
}
