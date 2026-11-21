//! The driving engine's effect data (docs/systems/driving-effects.md): the particle type records
//! DRIVING.ELF carries in its own `.data`, the per-track `Render\FX\<track>.tun`, the shared
//! `Render\Glare\default.tun` and `Render\SwitchEffects\default.tun`, and `data\render\decal.dat`.
//!
//! Tags: [M] DRIVING.ELF decompile/disassembly (EE address = file offset + 0x107080, the one
//! PT_LOAD maps 0x107100 at file offset 0x80), [D] disc bytes. Nothing here is a live capture.
//!
//! Particle types are **not** loaded from a file: the library loader builds the name
//! `"%sparticle.dat"` and never opens it (`FUN_001e7f60`: the `sprintf` result is unused) and adds
//! the `.data` record at 0x3351c0 under the key `'dflt'` instead [M]. The vehicle effects add nine
//! more `.data` records (`FUN_001edf38`) [M]; track effect parts add their own (`FUN_00235ed0`,
//! not decoded here).

use crate::buf::{invalid, Result};

/// DRIVING.ELF: EE address = file offset + this [M the PT_LOAD header: vaddr 0x107100, offset 0x80].
pub const DRIVING_ELF_EE_BASE: u32 = 0x0010_7080;

/// Size of one particle type record [M `FUN_001e7cf0`: the library copies 0xb0 bytes per slot].
pub const PARTICLE_TYPE_SIZE: usize = 0xb0;

/// The particle library holds at most 256 types [M `FUN_001e7cf0`: `if count < 0x100`].
pub const LIBRARY_CAPACITY: usize = 0x100;

/// A big-endian four-character code as the game compares it (`'SMRT'` = 0x534d5254).
pub const fn fourcc(s: &[u8; 4]) -> u32 { u32::from_be_bytes(*s) }

/// The `.data` particle types the game registers, in registration order: key, EE address [M].
/// `dflt` by `FUN_001e7f60`; the rest by `FUN_001edf38` (the vehicle-effect init).
pub const ELF_TYPES: [(u32, u32); 10] = [
    (fourcc(b"dflt"), 0x3351c0),
    (fourcc(b"SMRT"), 0x335440),
    (fourcc(b"RDDF"), 0x3354f0),
    (fourcc(b"RDDR"), 0x3355a0),
    (fourcc(b"RDGR"), 0x335700),
    (fourcc(b"SNOW"), 0x3357b0),
    (fourcc(b"SPRK"), 0x335910),
    (fourcc(b"WATR"), 0x335650),
    (fourcc(b"SPRL"), 0x335a70),
    (fourcc(b"SPKB"), 0x3359c0),
];

/// The vehicle's fourteen effect slots and the type each slot is built with [M `FUN_001ee238`
/// reads the 56-byte table at EE 0x393870, one little-endian word per slot; D: the bytes].
/// Slots 0-3 are the tyre smoke of wheels 0-3, 4-7 the wheels' surface spray (the type is
/// replaced per surface each frame, `FUN_001e5fa0`), 9-12 the wheels' sparks [M `FUN_001ee4a0`].
/// Slot 8 and 13 have no writer in `FUN_001ee4a0`.
pub const SLOT_TYPES: [u32; 14] = [
    fourcc(b"SMRT"), fourcc(b"SMRT"), fourcc(b"SMRT"), fourcc(b"SMRT"),
    fourcc(b"RDDF"), fourcc(b"RDDF"), fourcc(b"RDDR"), fourcc(b"RDDR"),
    fourcc(b"SPRK"), fourcc(b"SPRK"), fourcc(b"SPRK"), fourcc(b"SPRK"), fourcc(b"SPRK"),
    fourcc(b"SPRL"),
];

/// Wheel surface (0-15) → the spray type a wheel on it throws, 0 = none (tyre smoke instead)
/// [M `FUN_001ee4a0` reads EE 0x3938a8 + surface*4; D: the bytes].
pub const SURFACE_SPRAY: [u32; 16] = [
    0, 0, fourcc(b"RDDR"), fourcc(b"RDGR"), 0, fourcc(b"RDDR"), fourcc(b"WATR"), 0,
    0, fourcc(b"SNOW"), 0, 0, 0, 0, 0, 0,
];

/// EE address of [`SLOT_TYPES`] / [`SURFACE_SPRAY`] in DRIVING.ELF [M].
pub const SLOT_TYPES_EE: u32 = 0x393870;
pub const SURFACE_SPRAY_EE: u32 = 0x3938a8;

fn f(b: &[u8], o: usize) -> f32 { f32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) }
fn u(b: &[u8], o: usize) -> u32 { u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]) }

/// One 0xb0-byte particle type record, fields as the code reads them. Names come from the
/// readers: [M `FUN_001f29d8`] (spawn), [M `FUN_001f3ca8`] (step), [M `FUN_001f3dc8`] (draw),
/// [M `FUN_001f3188`] (load-time preparation), [M `FUN_001e5cd8`/`FUN_001e65c0`] (emitters).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleType {
    /// +0x00 / +0x08: half-width of the random spawn offset along world x / z.
    pub spread_x: f32,
    pub spread_z: f32,
    /// +0x04: base speed along the emitter's main axis; +0x14 its +- variance.
    pub speed: f32,
    pub speed_var: f32,
    /// +0x10 / +0x18: +- speed along the emitter's second / third axis.
    pub side_var: f32,
    pub third_var: f32,
    /// +0x0c: particles per lifetime (the emitter rate's numerator); halved at load.
    pub count: u32,
    /// +0x1c: flag bits (see the `FLAG_*` constants).
    pub flags: u32,
    /// +0x20: per-step velocity multiplier.
    pub drag: f32,
    /// +0x24: a four-character name, compared only to pick the atlas cell ([`vehicle_cell`]).
    pub name: u32,
    /// +0x2c: low byte copied to the particle's +0x1c byte at spawn.
    pub byte_2c: u32,
    /// +0x30: 1 / lifetime (written at load).
    pub inv_life: f32,
    /// +0x34 / +0x38: size at birth / death on the file side (rewritten at load, see `prepare`).
    pub size0: f32,
    pub size1: f32,
    /// +0x3c: lifetime in steps (capped at 255); +0x40 / +0x44: emission window (on / length).
    pub life: u32,
    pub delay: u32,
    pub duration: u32,
    /// +0x48: pre-warm (lifetime*0.75 at load when non-zero).
    pub prewarm: u32,
    /// +0x4c / +0x50: gravity base / +- variance (stored per particle as a signed byte).
    pub gravity: f32,
    pub gravity_var: f32,
    /// +0x54: +- spin, degrees on disc, radians after load.
    pub spin_var: f32,
    /// +0x58 (not read by the traced code) and +0x5c: max(size0, size1) written at load.
    pub f58: f32,
    pub max_size: f32,
    /// +0x60: atlas cell (u, v, size, unused) on the 256x256 sheet.
    pub cell: [f32; 4],
    /// +0x70 / +0x80 / +0x90: colour keys (A, R, G, B), 0..512 = 0..1 on disc.
    pub keys: [[f32; 4]; 3],
    /// +0xa0 / +0xa4: size / alpha randomisers (> 0 enables).
    pub size_rand: f32,
    pub alpha_rand: f32,
    /// +0xa8 (not read by the traced code), +0xac: the library slot (written when added).
    pub fa8: u32,
    pub slot: u32,
}

/// `flags` bits [M]. 0x02: emit forever (+0x44 = 99999 at load; emitters start inactive,
/// `FUN_001e5e10`). 0x08: the quad rotates (set at load when spin != 0 or bit 0x10). 0x10: fixed
/// rotation (spin, not age*spin). 0x20: drawn as a velocity streak triangle, sizes kept as-is.
/// 0x40: alpha from the particle's +0x1c high nibble. 0x80: size from its low nibble. 0x100: set
/// the particle's 0x40 bit when `byte_2c` != 0. 0x200: the additive bucket.
pub const FLAG_FOREVER: u32 = 0x02;
pub const FLAG_ROTATE: u32 = 0x08;
pub const FLAG_FIXED_SPIN: u32 = 0x10;
pub const FLAG_STREAK: u32 = 0x20;
pub const FLAG_NIBBLE_ALPHA: u32 = 0x40;
pub const FLAG_NIBBLE_SIZE: u32 = 0x80;
pub const FLAG_BYTE2C: u32 = 0x100;
pub const FLAG_ADDITIVE: u32 = 0x200;

impl ParticleType {
    /// Decode a record from its 0xb0 bytes (little-endian, as in `.data`).
    pub fn parse(b: &[u8]) -> Result<ParticleType> {
        if b.len() < PARTICLE_TYPE_SIZE {
            return invalid(format!("particle type: {} bytes, need 0xb0", b.len()));
        }
        let k = |o: usize| [f(b, o), f(b, o + 4), f(b, o + 8), f(b, o + 12)];
        Ok(ParticleType {
            spread_x: f(b, 0x00), speed: f(b, 0x04), spread_z: f(b, 0x08), count: u(b, 0x0c),
            side_var: f(b, 0x10), speed_var: f(b, 0x14), third_var: f(b, 0x18), flags: u(b, 0x1c),
            drag: f(b, 0x20), name: u(b, 0x24), byte_2c: u(b, 0x2c), inv_life: f(b, 0x30),
            size0: f(b, 0x34), size1: f(b, 0x38), life: u(b, 0x3c), delay: u(b, 0x40),
            duration: u(b, 0x44), prewarm: u(b, 0x48), gravity: f(b, 0x4c), gravity_var: f(b, 0x50),
            spin_var: f(b, 0x54), f58: f(b, 0x58), max_size: f(b, 0x5c), cell: k(0x60),
            keys: [k(0x70), k(0x80), k(0x90)], size_rand: f(b, 0xa0), alpha_rand: f(b, 0xa4),
            fa8: u(b, 0xa8), slot: u(b, 0xac),
        })
    }

    /// Read the record at EE address `ee` of a DRIVING.ELF image.
    pub fn from_driving_elf(elf: &[u8], ee: u32) -> Result<ParticleType> {
        let o = ee.checked_sub(DRIVING_ELF_EE_BASE).map(|o| o as usize);
        match o.and_then(|o| elf.get(o..o + PARTICLE_TYPE_SIZE)) {
            Some(b) => ParticleType::parse(b),
            None => invalid(format!("particle type at {ee:#x} outside the ELF")),
        }
    }

    /// The record the library stores once added: `FUN_001e80b0` (lifetime cap, pre-warm) then
    /// `FUN_001f3188` (in place) [M]. `slot` is the library index `FUN_001e7cf0` writes to +0xac.
    pub fn prepare(&self, slot: u32) -> ParticleType {
        let mut p = *self;
        p.slot = slot;
        // FUN_001e80b0: cap, then the pre-warm (float of the capped lifetime * 0.75, truncated).
        if p.life > 0xff { p.life = 0xff; }
        if p.prewarm != 0 { p.prewarm = (p.life as f32 * 0.75) as u32; }
        // FUN_001f3188: +0x5c = max(+0x38, +0x34) (the `<=` keeps +0x34 on a tie).
        p.max_size = if p.size1 <= p.size0 { p.size0 } else { p.size1 };
        if p.flags & FLAG_FOREVER != 0 { p.duration = 99999; }
        p.inv_life = 1.0 / p.life as f32;
        p.count = (p.count as f32 * 0.5) as u32;
        if p.flags & FLAG_STREAK == 0 {
            // Birth size at t = 1, death at t = 0: size = size0 + t*size1 after this.
            let s0 = p.size0;
            p.size0 = p.size1 * 1.4142;
            p.size1 = s0 * 1.4142 - p.size1 * 1.4142;
        }
        if p.spin_var != 0.0 || p.flags & FLAG_FIXED_SPIN != 0 { p.flags |= FLAG_ROTATE; }
        p.spin_var *= 0.017453292;
        for k in p.keys.iter_mut() {
            for c in k.iter_mut() { *c *= 0.001953125; }
            k[0] *= 0.5;
        }
        p.keys.swap(0, 2); // the age counts down: key 0 is the death colour after this
        p
    }
}

/// The atlas cell `FUN_001ede48` gives a vehicle type, chosen by comparing its `name` with the
/// names of five registered records in this order: SMRT, RDDF, RDGR, SNOW, WATR [M]. Cells are in
/// 1/256 units: u = (x + 0.5)/256, v = (y + 0.5)/256, size = (n - 1)/256. A name matching none of
/// them gets (0.5/256, 0.5/256, 255/256). `names` = (smrt, rddf, rdgr, snow, watr).
pub fn vehicle_cell(name: u32, names: [u32; 5]) -> [f32; 4] {
    let [smrt, rddf, rdgr, snow, watr] = names;
    let (x, y, n) = if name == smrt || (name != rddf && name != rdgr && name != snow && name == watr) {
        (0x60, 0xa0, 0x20)
    } else if name == rddf {
        (0x80, 0x40, 0x40)
    } else if name == rdgr {
        (0xa0, 0xe0, 0x20)
    } else if name == snow {
        (0x80, 0x60, 0x40)
    } else {
        (0, 0, 0x100)
    };
    [(x as f32 + 0.5) * 0.00390625, (y as f32 + 0.5) * 0.00390625, (n as f32 - 1.0) * 0.00390625, 0.0]
}

/// The ten `.data` types read from a DRIVING.ELF image, vehicle cells assigned (`FUN_001edf38`
/// calls `FUN_001ede48` on six of them before registering: SMRT, RDDF, RDDR, WATR, RDGR, SNOW),
/// in registration order, **not** prepared.
pub fn elf_types(elf: &[u8]) -> Result<Vec<(u32, ParticleType)>> {
    let mut v = Vec::new();
    for (key, ee) in ELF_TYPES {
        v.push((key, ParticleType::from_driving_elf(elf, ee)?));
    }
    let name = |k: &[u8; 4]| v.iter().find(|(x, _)| *x == fourcc(k)).map(|(_, t)| t.name).unwrap_or(0);
    let names = [name(b"SMRT"), name(b"RDDF"), name(b"RDGR"), name(b"SNOW"), name(b"WATR")];
    for (key, t) in v.iter_mut() {
        if [b"SMRT", b"RDDF", b"RDDR", b"WATR", b"RDGR", b"SNOW"].iter().any(|k| fourcc(k) == *key) {
            t.cell = vehicle_cell(t.name, names);
        }
    }
    Ok(v)
}

/// Read `n` little-endian words at an EE address of DRIVING.ELF.
pub fn elf_words(elf: &[u8], ee: u32, n: usize) -> Result<Vec<u32>> {
    let o = (ee - DRIVING_ELF_EE_BASE) as usize;
    match elf.get(o..o + 4 * n) {
        Some(b) => Ok((0..n).map(|i| u(b, 4 * i)).collect()),
        None => invalid(format!("{ee:#x}+{n} words outside the ELF")),
    }
}

// ---------------------------------------------------------------------------------------------
// Text tunings. Keys are the registered tuner names with their spaces removed (`Enable vehicle
// sparks` = EE 0x393818 registered by FUN_001ee1f0 → `Enablevehiclesparks`; `Halo Tex` 0x394ee0
// → `HaloTex`) [M names, D files].

fn kv(text: &str) -> Vec<(String, String)> { crate::rendertun::parse(text) }

fn split_key(k: &str) -> (&str, Option<&str>) {
    match k.split_once('{') {
        Some((a, b)) => (a, Some(b.trim_end_matches('}'))),
        None => (k, None),
    }
}

fn colour(v: &str) -> Option<[u8; 4]> {
    let p: Vec<u8> = v.split(',').filter_map(|x| x.trim().parse::<u32>().ok().map(|n| n as u8)).collect();
    (p.len() == 4).then(|| [p[0], p[1], p[2], p[3]])
}

/// One shooter class's bullet streak (`BulletStreak*{Class}`, the tuners FUN_001d7620 registers).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BulletStreak {
    pub class: String,
    /// The text's four numbers in file order (first = alpha, as the other `*Col` keys) [D].
    pub colour: [u8; 4],
    pub texture: u32,
    pub width: f32,
    pub length: f32,
    pub speed: f32,
    pub count: u32,
}

/// `data\tuning\Render\FX\<track>.tun` [D].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FxTuning {
    /// `Do glow`.
    pub do_glow: bool,
    /// `Enable vehicle sparks` — the gate `FUN_001eed18` tests (DAT_00335418) [M].
    pub vehicle_sparks: bool,
    /// `Enable Particulate` (the weather layer, RParticulate) [M name 0x3912a0].
    pub particulate: bool,
    pub water_scale_x: [f32; 4],
    pub water_scale_y: [f32; 4],
    pub water_scroll_u: [f32; 4],
    pub water_scroll_v: [f32; 4],
    pub water_sun_dir: f32,
    /// `Lightning*` in file order: width, squiggle, spread, driftrate, driftvariance,
    /// lifevariance, segments, levels, passes.
    pub lightning: Vec<(String, f32)>,
    /// `Shadowwidth{car}` / `Shadowlength{car}` (the last value wins on repeats, as a map).
    pub shadow: Vec<(String, f32, f32)>,
    pub bullet_streaks: Vec<BulletStreak>,
}

impl FxTuning {
    pub fn parse(text: &str) -> FxTuning {
        let mut t = FxTuning::default();
        for (k, v) in kv(text) {
            let (name, inst) = split_key(&k);
            let fv = v.parse::<f32>().unwrap_or(0.0);
            let idx = inst.and_then(|i| i.parse::<usize>().ok()).filter(|&i| i < 4);
            match (name, inst) {
                ("Doglow", None) => t.do_glow = fv != 0.0,
                ("Enablevehiclesparks", None) => t.vehicle_sparks = fv != 0.0,
                ("EnableParticulate", None) => t.particulate = fv != 0.0,
                ("WaterscaleX", _) => if let Some(i) = idx { t.water_scale_x[i] = fv },
                ("WaterscaleY", _) => if let Some(i) = idx { t.water_scale_y[i] = fv },
                ("WaterscrollU", _) => if let Some(i) = idx { t.water_scroll_u[i] = fv },
                ("WaterscrollV", _) => if let Some(i) = idx { t.water_scroll_v[i] = fv },
                ("Watersundir", None) => t.water_sun_dir = fv,
                (n, None) if n.starts_with("Lightning") => t.lightning.push((n["Lightning".len()..].to_owned(), fv)),
                ("Shadowwidth", Some(c)) | ("Shadowlength", Some(c)) => {
                    let w = name == "Shadowwidth";
                    match t.shadow.iter_mut().find(|e| e.0 == c) {
                        Some(e) => if w { e.1 = fv } else { e.2 = fv },
                        None => t.shadow.push((c.to_owned(), if w { fv } else { 0.0 }, if w { 0.0 } else { fv })),
                    }
                }
                (n, Some(c)) if n.starts_with("BulletStreak") => {
                    let i = match t.bullet_streaks.iter().position(|b| b.class == c) {
                        Some(i) => i,
                        None => { t.bullet_streaks.push(BulletStreak { class: c.to_owned(), ..Default::default() }); t.bullet_streaks.len() - 1 }
                    };
                    let b = &mut t.bullet_streaks[i];
                    match &n["BulletStreak".len()..] {
                        "Colour" => b.colour = colour(&v).unwrap_or_default(),
                        "Texture" => b.texture = fv as u32,
                        "Width" => b.width = fv,
                        "Length" => b.length = fv,
                        "Speed" => b.speed = fv,
                        "Count" => b.count = fv as u32,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        t
    }
}

/// Number of glare types (`Glare ID` 0..0x4f) [M `FUN_001f14b0`: `FUN_00142b68(..., 0, 0x4f)`].
pub const GLARE_TYPES: usize = 80;

/// One glare type: a halo layer and a spike layer [M names: `Halo Tex`/`Halo Spin`/`Halo Col`/
/// `Halo Size`/`Spike *`/`Z Bias`, EE 0x394ee0-0x394f60, a 0x40-byte stride per ID].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Glare {
    /// `HaloTex` / `SpikeTex`: cell index 0..9 on the `flar` sheet ([`glare_cell`]).
    pub tex: [u32; 2],
    pub spin: [f32; 2],
    /// Colours in file order (A, R, G, B).
    pub col: [[u8; 4]; 2],
    pub size: [f32; 2],
    pub zbias: f32,
}

/// `data\tuning\Render\Glare\default.tun` [D]: 80 entries; IDs 64 and 76-79 carry the
/// 0xcbcbcbcb fill pattern (3419130827 / -2.6712e7) in the file — written from unset tuners.
pub fn parse_glare(text: &str) -> Vec<Glare> {
    let mut g = vec![Glare::default(); GLARE_TYPES];
    for (k, v) in kv(text) {
        let (name, inst) = split_key(&k);
        let Some(i) = inst.and_then(|i| i.parse::<usize>().ok()).filter(|&i| i < GLARE_TYPES) else { continue };
        let fv = v.parse::<f64>().unwrap_or(0.0);
        let e = &mut g[i];
        match name {
            "HaloTex" => e.tex[0] = fv as u32,
            "SpikeTex" => e.tex[1] = fv as u32,
            "HaloSpin" => e.spin[0] = fv as f32,
            "SpikeSpin" => e.spin[1] = fv as f32,
            "HaloCol" => e.col[0] = colour(&v).unwrap_or_default(),
            "SpikeCol" => e.col[1] = colour(&v).unwrap_or_default(),
            "HaloSize" => e.size[0] = fv as f32,
            "SpikeSize" => e.size[1] = fv as f32,
            "ZBias" => e.zbias = fv as f32,
            _ => {}
        }
    }
    g
}

/// The `flar` sheet cell for a glare texture index (`FUN_001f19f0`) [M]: 0 and 1 are the two
/// halves of the top row (u = (t&1)*0.5, v = 0, size 0.5); 2.. index a 4x2 grid of quarter cells
/// below it (u = ((t-2)&3)*0.25, v = ((t-2)&0xc)/4*0.25 + 0.5, size 0.25). Returns (u, v, size).
pub fn glare_cell(t: u32) -> (f32, f32, f32) {
    if t < 2 {
        ((t & 1) as f32 * 0.5, 0.0, 0.5)
    } else {
        let k = t.wrapping_sub(2);
        ((k & 3) as f32 * 0.25, ((k & 0xc) >> 2) as f32 * 0.25 + 0.5, 0.25)
    }
}

/// The sixteen switch channels of `Render\SwitchEffects\default.tun`, in file order [D]
/// (`Booster`, `Headlights`, `Brakelights`, … `Misc`), each with its `VScroll`, `Glow` and
/// `Illuminate` value. The section is `Render:SwitchEffects` [M 0x388568, FUN_001d2910].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwitchChannel {
    pub name: String,
    pub vscroll: f32,
    pub glow: f32,
    pub illuminate: f32,
}

pub fn parse_switch_effects(text: &str) -> Vec<SwitchChannel> {
    let mut ch: Vec<SwitchChannel> = Vec::new();
    for (k, v) in kv(text) {
        let (name, Some(inst)) = split_key(&k) else { continue };
        // `Booster[0]` → index 0, name Booster.
        let Some((cname, idx)) = inst.split_once('[') else { continue };
        let Ok(i) = idx.trim_end_matches(']').parse::<usize>() else { continue };
        if i >= 16 { continue; }
        while ch.len() <= i { ch.push(SwitchChannel::default()); }
        ch[i].name = cname.to_owned();
        let fv = v.parse::<f32>().unwrap_or(0.0);
        match name {
            "VScroll" => ch[i].vscroll = fv,
            "Glow" => ch[i].glow = fv,
            "Illuminate" => ch[i].illuminate = fv,
            _ => {}
        }
    }
    ch
}

/// `data\render\decal.dat`: the loader copies exactly 0x280 bytes (`FUN_001da3b0`; a missing
/// file zero-fills them and logs "Unable to load lighting file for config [Decal.dat]") [M].
pub const DECAL_DAT_SIZE: usize = 0x280;

/// One 0x50-byte decal record [D layout: eight records whose four 16-byte rows are
/// (u, v, 1.0, 1.0) atlas corners in 1/4 steps, then four words]. The meaning of `w0`-`w2` is not
/// traced (the decal manager's readers are not decompiled here); `index` counts 0..7 [D].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decal {
    pub corners: [[f32; 4]; 4],
    pub w0: u32,
    pub w1: u32,
    pub w2: u32,
    pub index: u32,
}

pub fn parse_decals(b: &[u8]) -> Result<Vec<Decal>> {
    if b.len() < DECAL_DAT_SIZE {
        return invalid(format!("decal.dat: {} bytes, the loader copies 0x280", b.len()));
    }
    Ok((0..DECAL_DAT_SIZE / 0x50)
        .map(|r| {
            let o = r * 0x50;
            let row = |k: usize| [f(b, o + k), f(b, o + k + 4), f(b, o + k + 8), f(b, o + k + 12)];
            Decal { corners: [row(0), row(0x10), row(0x20), row(0x30)], w0: u(b, o + 0x40), w1: u(b, o + 0x44), w2: u(b, o + 0x48), index: u(b, o + 0x4c) }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(words: &[(usize, u32)]) -> Vec<u8> {
        let mut b = vec![0u8; PARTICLE_TYPE_SIZE];
        for &(o, w) in words { b[o..o + 4].copy_from_slice(&w.to_le_bytes()); }
        b
    }

    /// SMRT (EE 0x335440) as DRIVING.ELF holds it [D bytes, read 11-20 from the disc's ELF].
    fn smrt() -> ParticleType {
        ParticleType::parse(&rec(&[
            (0x04, 0x3c75c28f), (0x0c, 30), (0x1c, 8), (0x20, 0x3f75c28f), (0x24, 0x736d6b35),
            (0x34, 0x3e4ccccd), (0x38, 0x3f333333), (0x3c, 60), (0x40, 60), (0x4c, 0xbe4ccccd),
            (0x54, 0x43340000), (0x58, 0x3f000000), (0x68, 0x3e800000), (0x6c, 0x3e800000),
            (0x74, 0x44000000), (0x78, 0x44000000), (0x7c, 0x44000000), (0x80, 0x42000000),
            (0x84, 0x44000000), (0x88, 0x44000000), (0x8c, 0x44000000), (0x94, 0x44000000),
            (0x98, 0x44000000), (0x9c, 0x44000000),
        ])).unwrap()
    }

    #[test]
    fn prepare_follows_fun_001f3188() {
        let p = smrt().prepare(1);
        assert_eq!(p.slot, 1);
        assert_eq!(p.count, 15, "30 * 0.5");
        assert_eq!(p.inv_life, 1.0 / 60.0);
        assert_eq!(p.max_size, 0.7f32, "max(0.2, 0.7)");
        // birth (t = 1) = 0.2*1.4142, death (t = 0) = 0.7*1.4142
        assert!((p.size0 - 0.7 * 1.4142).abs() < 1e-6);
        assert!((p.size0 + p.size1 - 0.2 * 1.4142).abs() < 1e-6);
        assert!((p.spin_var - std::f32::consts::PI).abs() < 1e-6, "180 degrees");
        assert_eq!(p.flags, 8);
        // keys reversed: key 0 = file key 2 = (0, 512, 512, 512) → alpha 0, rgb 1.0
        assert_eq!(p.keys[0], [0.0, 1.0, 1.0, 1.0]);
        assert_eq!(p.keys[1], [32.0 / 512.0 * 0.5, 1.0, 1.0, 1.0]);
        assert_eq!(p.keys[2], [0.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn streak_types_keep_sizes_and_forever_sets_99999() {
        let mut t = smrt();
        t.flags = 0x22;
        let p = t.prepare(0);
        assert_eq!((p.size0, p.size1), (t.size0, t.size1));
        assert_eq!(p.duration, 99999);
    }

    #[test]
    fn vehicle_cells_follow_fun_001ede48() {
        let n = [fourcc(b"smk5"), fourcc(b"puff"), fourcc(b"gras"), fourcc(b"puff"), fourcc(b"grvl")];
        assert_eq!(vehicle_cell(fourcc(b"smk5"), n), [96.5 / 256.0, 160.5 / 256.0, 31.0 / 256.0, 0.0]);
        assert_eq!(vehicle_cell(fourcc(b"puff"), n), [128.5 / 256.0, 64.5 / 256.0, 63.0 / 256.0, 0.0]);
        assert_eq!(vehicle_cell(fourcc(b"gras"), n), [160.5 / 256.0, 224.5 / 256.0, 31.0 / 256.0, 0.0]);
        // WATR shares the smoke cell (the code jumps to the SMRT label)
        assert_eq!(vehicle_cell(fourcc(b"grvl"), n), vehicle_cell(fourcc(b"smk5"), n));
    }

    #[test]
    fn glare_cells() {
        assert_eq!(glare_cell(0), (0.0, 0.0, 0.5));
        assert_eq!(glare_cell(1), (0.5, 0.0, 0.5));
        assert_eq!(glare_cell(2), (0.0, 0.5, 0.25));
        assert_eq!(glare_cell(9), (0.75, 0.75, 0.25));
    }

    #[test]
    fn fx_text() {
        let t = FxTuning::parse("Doglow=1\r\nEnablevehiclesparks=1\r\nWaterscaleX{2}=0.5\r\nShadowwidth{z8}=0.8\r\nShadowlength{z8}=0.9\r\nBulletStreakColour{SUV}=255,95,150,200\r\nBulletStreakCount{SUV}=1\r\n");
        assert!(t.do_glow && t.vehicle_sparks && !t.particulate);
        assert_eq!(t.water_scale_x[2], 0.5);
        assert_eq!(t.shadow, vec![("z8".to_owned(), 0.8, 0.9)]);
        assert_eq!(t.bullet_streaks[0].colour, [255, 95, 150, 200]);
        assert_eq!(t.bullet_streaks[0].count, 1);
    }

    #[test]
    fn switch_text() {
        let c = parse_switch_effects("VScroll{Brakelights[2]}=0.030000\nGlow{Brakelights[2]}=0.080000\nIlluminate{Brakelights[2]}=0.530000\n");
        assert_eq!(c.len(), 3);
        assert_eq!(c[2], SwitchChannel { name: "Brakelights".into(), vscroll: 0.03, glow: 0.08, illuminate: 0.53 });
    }
}
