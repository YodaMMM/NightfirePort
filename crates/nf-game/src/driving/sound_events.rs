//! The driving missions' sound events beyond the player engine (`audio.rs`): the 3-D listener model
//! (distance attenuation, pan, doppler), the stereo pan law down to the SPU2 left/right volumes,
//! collision impacts and the scrape counters, landings, the horn timing, the traffic-vehicle engine,
//! the car-weapon fire one-shots and the vehicle sound classes. Write-up:
//! docs/systems/drive-audio.md ("3-D", "Events"). Everything here is headless: outputs are voice
//! volumes / pitches / pans and one-shot requests; nothing is played.
//!
//! Evidence (DRIVING.ELF, EE addresses; [M] decompile + disassembly, [D] ELF data):
//! - Listener build `FUN_002f4e00` (called from the per-frame audio pass `FUN_00147e30` with
//!   index 2): rows 0..2 of the camera matrix become the listener's local axes, the position the
//!   camera record's +0x10 (`FUN_00195808`), the velocity its +0x20 (`FUN_00195858`), +0x70 = -100000,
//!   +0x74 = 0 (the height band, later set from an ambience region `FUN_00230bb8` +0x14/+0x18).
//! - Per-listener pass `FUN_002ec238`: a sound is heard when its max-distance square (+0x64) is 0
//!   or the squared distance is below it; `FUN_002f1f40` builds the update arguments
//!   [gain, pitch, pan, cone, send, distance, listener]; the sound's update is vtable +0x24.
//! - Pan: `FUN_002f1f40` pan = atan2f(local.x, local.z) / 2pi (`FUN_002c0618` is atan2f), 0 inside
//!   half the min distance, scaled by (2d - min)/min inside it; listeners other than index 2 get a
//!   fixed -0.25 (even) / +0.25 (odd). `FUN_002ddd20` stores pan * 65536 as the voice azimuth,
//!   `FUN_0030a750` adds the patch's own azimuth (`FUN_0030c978` table 0x357c20 [D]: patch pan 0 ->
//!   0xc000, 64 -> 0, 127 -> 0x3fff). The pan table `FUN_0030c668` from the speaker angles
//!   `FUN_0030cf90` (stereo: 0x4000, 0xc000) with the curve `FUN_0030c640` (2x - x*x/127),
//!   `FUN_0030c908` reads row azimuth >> 8. `FUN_0030ec40` sends channel 1 to byte +0x26 and channel 0
//!   to +0x27 of the voice record; SNDDRV.IRX `SNDIOP_spusetvol` writes record +6 / +7 with the
//!   `vol.70` params 0x000 / 0x100 = libsd SD_VP_VOLL / SD_VP_VOLR. So the speaker at azimuth
//!   +0x4000 is the RIGHT output: a positive pan is right, and local +x (camera row 0) is right.
//! - Impacts `FUN_002e0890` (vehicle vtable +0x34), landings `FUN_002dfa00` (+0x2c), both called
//!   by the contact handler `FUN_00149ed8` with |impulse| (+0x48) and the material code from the
//!   jump table at 0x36f0b0 [D].
//! - Horn `FUN_002de9b8` (ctor) / `FUN_002dea48` (step); the random generator `FUN_0025a5ac`
//!   (seed `FUN_0025a684`). Traffic vehicle ctor `FUN_002e5340`, update `FUN_002e5598`, its rpm
//!   source `FUN_002df8a0`. Car weapon fire `FUN_0020de20`.
//!
//! Not traced (stated plainly): which listener matrix the camera record's +0x5c points at (the
//! port assumes the chase camera's look matrix, rows right/up/forward, `camera.rs`); what writes
//! the landing strength (+0x150); the unit of the game clock the impact cooldowns use
//! (`FUN_00261038`); the boat / snowmobile / helicopter / tank / ultralight per-frame updates
//! (their ctor voices are listed in [`CLASS_VOICES`] only); the weapon fire jump table of
//! `FUN_0014ca60` beyond the `SFX_<weapon>Fire` key; which script event starts which stream.

use super::audio::{voice_gate, Listener, Mixer, OneShot, VoiceOut};

pub type V3 = [f32; 3];
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn norm(a: V3) -> V3 { let l = dot(a, a).sqrt(); if l > 0.0 { [a[0] / l, a[1] / l, a[2] / l] } else { a } }

// ---------------------------------------------------------------------------------------------
// The driving random generator (FUN_0025a5ac / FUN_0025a684)
// ---------------------------------------------------------------------------------------------

/// `FUN_0025a5ac` [M]: six words at 0x343810..0x343824 added down a carry chain, the last one
/// counting up; the result is word 0. The horn timing draws from it (so do two other callers,
/// `FUN_0017bd00` and `FUN_002d0aa8`, not ported here: a port that wires the horns in shares
/// the stream with them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriveRng { pub w: [u32; 6] }

impl DriveRng {
    /// The words as the ELF ships them [D 0x343810] = `FUN_0025a684(0)`.
    pub const BOOT: [u32; 6] = [0xf22d_0e56, 0x8831_26e9, 0xc624_dd2f, 0x0702_c49c, 0x9e35_3f7d, 0x6fdf_3b64];

    pub fn boot() -> DriveRng { DriveRng { w: Self::BOOT } }

    /// `FUN_0025a684` [M]: every word = seed + its fixed offset (the ELF values are seed 0).
    pub fn seeded(seed: u32) -> DriveRng {
        let mut w = Self::BOOT;
        for x in &mut w { *x = x.wrapping_add(seed); }
        DriveRng { w }
    }

    /// `FUN_0025a5ac` [M 0x25a5ac-0x25a680].
    pub fn next(&mut self) -> u32 {
        let [mut t0, mut t1, mut t2, mut t3, mut t4, mut t5] = self.w;
        t4 = t4.wrapping_add(t5);
        let mut c = (t4 < t5) as u32;
        let step = |hi: &mut u32, lo: u32, c: &mut u32| {
            *hi = hi.wrapping_add(lo);
            let a2 = (*hi < lo) as u32;
            *hi = hi.wrapping_add(*c);
            *c = (*hi < *c) as u32 + a2;
        };
        step(&mut t3, t4, &mut c);
        step(&mut t2, t3, &mut c);
        step(&mut t1, t2, &mut c);
        t0 = t0.wrapping_add(t1).wrapping_add(c);
        t5 = t5.wrapping_add(1);
        if t5 == 0 {
            t4 = t4.wrapping_add(1);
            if t4 == 0 {
                t3 = t3.wrapping_add(1);
                if t3 == 0 {
                    t2 = t2.wrapping_add(1);
                    if t2 == 0 {
                        t1 = t1.wrapping_add(1);
                        if t1 == 0 { t0 = t0.wrapping_add(1); }
                    }
                }
            }
        }
        self.w = [t0, t1, t2, t3, t4, t5];
        t0
    }
}

// ---------------------------------------------------------------------------------------------
// 3-D: the listener and FUN_002f1f40
// ---------------------------------------------------------------------------------------------

/// The listener object (0x80 bytes, `FUN_002f4e00`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Listener3d {
    /// +0x40 position.
    pub pos: V3,
    /// Rows 0..2 of the camera matrix (`FUN_002f4e00` stores them as the columns of the local
    /// transform, so local = (row0, row1, row2) . (p - pos)).
    pub right: V3,
    pub up: V3,
    pub fwd: V3,
    /// +0x50 the velocity the doppler term uses.
    pub vel: V3,
    /// +0x5c the listener index (the single-player pass uses 2 [M 0x147eb4 / 0x147fac]).
    pub index: u32,
    /// +0x70 / +0x74 the height band (-100000 / 0 from the ctor: no effect).
    pub band_h: f32,
    pub band_w: f32,
}

impl Listener3d {
    /// `FUN_002f4e00` with the camera rows given directly.
    pub fn new(pos: V3, right: V3, up: V3, fwd: V3, vel: V3) -> Listener3d {
        Listener3d { pos, right, up, fwd, vel, index: 2, band_h: -100000.0, band_w: 0.0 }
    }

    /// From a look direction and up, with the driving world's rule right = up x forward
    /// (docs/systems/driving-handedness.md; the chase camera's `FUN_001d5700` builds its rows the
    /// same way, `camera.rs`). The on-foot right-handed rule (forward x up) would swap the channels.
    pub fn look(pos: V3, fwd: V3, up: V3, vel: V3) -> Listener3d {
        let f = norm(fwd);
        let r = norm(cross(up, f));
        Listener3d::new(pos, r, cross(f, r), f, vel)
    }

    /// From the port's chase camera view (rows right / up / fwd, eye).
    pub fn from_view(v: &super::camera::CamView, vel: V3) -> Listener3d { Listener3d::new(v.eye, v.right, v.up, v.fwd, vel) }

    fn local(&self, p: V3) -> V3 { let d = sub(p, self.pos); [dot(self.right, d), dot(self.up, d), dot(self.fwd, d)] }
}

/// A positioned sound (the base sound object, ctor `FUN_002f4780` / the `AOneShotSound`s).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Source3d {
    /// +0x00 position.
    pub pos: V3,
    /// +0x10 velocity.
    pub vel: V3,
    /// +0x20 the facing the cone term reads (ctor (0, 0, 1)).
    pub dir: V3,
    /// +0x5c / +0x60 min / max distance (ctor 3 / 350; +0x64 = max^2 = 122500).
    pub min: f32,
    pub max: f32,
}

impl Source3d {
    /// The ctor distances every one-shot here keeps (3 / 350 [M]).
    pub fn at(pos: V3, vel: V3) -> Source3d { Source3d { pos, vel, dir: [0.0, 0.0, 1.0], min: 3.0, max: 350.0 } }
}

/// The fixed pan of a non-positional sound / a split-screen listener (`FUN_002f1f40`).
fn fixed_pan(index: u32) -> f32 { if index == 2 { 0.0 } else if index & 1 != 0 { 0.25 } else { -0.25 } }

/// `FUN_002ec238` + `FUN_002f1f40` [M]: the update arguments for one sound and one listener.
/// A sound past the max-distance gate gets gain 0. `distance` is the [5] slot the engine blend
/// reads (`audio::EngineSound::layer_update`).
pub fn listen(l: &Listener3d, s: &Source3d) -> Listener {
    let mut out = Listener { gain: 0.0, pitch: 1.0, pan: 0.0, cone: 0.0, send: 0.0, distance: 0.0, index: l.index };
    // FUN_002ec238: |p - listener|^2 against +0x64 (= max^2; 0 means no gate)
    let d0 = sub(s.pos, l.pos);
    let max2 = s.max * s.max;
    if !(max2 == 0.0 || dot(d0, d0) < max2) { return out; }
    if s.max == 0.0 {
        // non-positional: gain 1, pitch 1, the fixed pan [M 0x2f1f80-0x2f1fd8]
        out.gain = 1.0;
        out.pan = fixed_pan(l.index);
        return out;
    }
    // the height band [M 0x2f1fdc-0x2f205c]
    let (sy, ly, h, w) = (s.pos[1], l.pos[1], l.band_h, l.band_w);
    let mut band = 1.0f32;
    if h < sy && ly < h { band = (ly + w) * (1.0 / w); }
    if sy < h - w && h - w < ly { band = -ly * (1.0 / w); }
    if band < 0.001 { return out; }
    let loc = l.local(s.pos);
    let d = dot(loc, loc).sqrt();
    out.distance = d;
    if s.max < d { return out; }
    out.gain = if s.min <= d { (s.max - d) / ((s.max - s.min) + (d - s.min) * 8.0) * band } else { band };
    if 0.0 < d {
        out.pan = if d < s.min * 0.5 {
            0.0
        } else {
            let ang = loc[0].atan2(loc[2]) * 0.159_154_94;
            let p = if l.index == 2 { ang } else { fixed_pan(l.index) };
            if d < s.min { p * (((d + d) - s.min) / s.min) } else { p }
        };
        let dir = norm(d0);
        out.cone = dot(dir, s.dir);
        out.pitch = 400.0 / ((dot(dir, s.vel) + 400.0) - dot(dir, l.vel));
        out.send = if 25.0 <= d { (d - 25.0) / 400.0 } else { 0.0 };
    } else {
        out.pan = fixed_pan(l.index);
    }
    out
}

/// `FUN_002ddd20` [M]: the pan in turns -> the voice's 16-bit azimuth (rounded half away from 0),
/// then `FUN_0030a750` adds the patch's own azimuth `base` (see [`patch_azimuth`]).
pub fn pan_azimuth(pan: f32, base: u16) -> u16 {
    let x = pan * 65536.0;
    let r = if x < 0.0 { x - 0.5 } else { x + 0.5 };
    ((r as i32) as i16 as u16).wrapping_add(base)
}

/// `FUN_0030c978` [M] over the table at 0x357c20 [D, pinned end points]: a patch pan byte
/// (0..127, 64 = centre) -> azimuth. Only the end points and the centre are carried here.
pub fn patch_azimuth(pan_byte: u8) -> Option<u16> {
    match pan_byte { 0 => Some(0xc000), 64 => Some(0), 127 => Some(0x3fff), _ => None }
}

/// The stereo speaker azimuths (`FUN_0030cf90`: 0x357264 = 0x4000, 0x357266 = 0xc000) [M].
pub const STEREO_SPEAKERS: [u32; 2] = [0x4000, 0xc000];

/// `FUN_0030c640` [M]: the pan curve on 0..127.
pub fn pan_curve(x: i32) -> i32 { x * 2 - (x * x) / 0x7f }

/// `FUN_0030c668` [M] for the stereo layout: the pan-table row for an azimuth row (0..255):
/// [speaker 0 (0x4000), speaker 1 (0xc000)], each 0..127.
pub fn pan_row(row: u32) -> [i32; 2] {
    let az = (row * 0x100) as i32;
    let (s0, s1) = (STEREO_SPEAKERS[0] as i32, STEREO_SPEAKERS[1] as i32);
    if s0 < az && az < s1 {
        // between the two speakers (the rear half)
        let (a, b) = (az - s0, s1 - az);
        return [pan_curve((b * 0x7f) / (a + b)), pan_curve((a * 0x7f) / (a + b))];
    }
    // the wrap through azimuth 0 (the front half); az <= s0 or az >= s1
    let (to0, from1) = if s0 < az { (s0 + 0x10000 - az, az - s1) } else { (s0 - az, az + 0x10000 - s1) };
    let sum = to0 + from1;
    [pan_curve((from1 * 0x7f) / sum), pan_curve((to0 * 0x7f) / sum)]
}

/// The left / right output levels (0..127) of a mono voice at `azimuth`: `FUN_0030c908` reads row
/// azimuth >> 8; `FUN_0030ec40` + SNDDRV `SNDIOP_spusetvol` put speaker 1 on VOLL and speaker 0 on
/// VOLR [M]. Returns (left, right).
pub fn stereo_levels(azimuth: u16) -> (i32, i32) {
    let r = pan_row((azimuth >> 8) as u32);
    (r[1], r[0])
}

// ---------------------------------------------------------------------------------------------
// Collision impacts (FUN_002e0890), landings (FUN_002dfa00), the contact codes (FUN_00149ed8)
// ---------------------------------------------------------------------------------------------

/// `FUN_00149ed8`'s jump table at 0x36f0b0 [D]: the other party's material id -> the impact code
/// passed to `FUN_002e0890` (0 for every id not listed).
pub fn contact_code(material: i32) -> i32 {
    match material {
        1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 10 => 10, 12 => 12, 13 | 16 => 13, 48 => 9, 64 => 8, 80 => 5, 96 => 15,
        _ => 0,
    }
}

/// The second party's code in `FUN_00149ed8` [M 0x14a624-0x14a654]: the first party's type 7 -> 1;
/// its material 0x10 or 0xd -> 0xd; else 0xf.
pub fn contact_code_b(type_a: u8, material_a: i32) -> i32 {
    if type_a == 7 { 1 } else if material_a == 0x10 || material_a == 0xd { 0xd } else { 0xf }
}

/// Which call a contact makes on a party's sound object (`FUN_00149ed8` [M]): the landing thump
/// (`FUN_002dfa00`, vtable +0x2c) when the other side is material 0xe, or types 4 / 7; the impact
/// (`FUN_002e0890`, +0x34) with [`contact_code`] when this side's type < 4 and the other's is not
/// 4; else nothing. `ty` / `other_ty` are the event's +0x4c / +0x50 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactCall { Landing, Impact(i32), None }

pub fn contact_call_a(ty: u8, other_ty: u8, other_material: i32) -> ContactCall {
    if other_material == 0xe || (ty == 4 && other_ty == 7) { ContactCall::Landing }
    else if ty < 4 && other_ty != 4 { ContactCall::Impact(contact_code(other_material)) }
    else { ContactCall::None }
}

/// The impact tables [D]: `(threshold, key)`. A at 0x3c27d8 (every class but 4 and 5), B at
/// 0x3c2838 (class 4), C at 0x3c28b8 (class 4 on code 5).
pub const IMPACTS_A: [(f32, &str); 12] = [
    (0.05, "SFX_Cobble1"), (0.10, "SFX_Cobble2"), (0.15, "SFX_Light1"), (0.20, "SFX_Light2"), (0.25, "SFX_Light3"),
    (0.30, "SFX_Light4"), (0.35, "SFX_Med1"), (0.40, "SFX_Med2"), (0.45, "SFX_Med3"), (0.50, "SFX_Med4"),
    (0.55, "SFX_Heavy1"), (0.60, "SFX_Heavy2"),
];
pub const IMPACTS_B: [(f32, &str); 10] = [
    (0.05, "SFX_Light1"), (0.10, "SFX_Light2"), (0.20, "SFX_Light3"), (0.25, "SFX_Light4"), (0.30, "SFX_Med1"),
    (0.35, "SFX_Med2"), (0.40, "SFX_Med3"), (0.45, "SFX_Med4"), (0.50, "SFX_Heavy1"), (0.55, "SFX_Heavy2"),
];
pub const IMPACTS_C: [(f32, &str); 3] = [(0.2, "SFX_sandbump1"), (0.35, "SFX_sandbump2"), (0.5, "SFX_sandbump3")];

/// The smackable alternation `FUN_002e0890` plays first for vehicles with +0x168 set.
pub const METAL_GATES: [&str; 2] = ["SFX_MetalGate2", "SFX_MetalGate3"];

/// A positioned one-shot request (an `AOneShotSound`: bank slot, key, group, volume +0x6c, and the
/// sound's position / velocity; min / max distance 3 / 350).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OneShot3d { pub slot: usize, pub sfx: &'static str, pub group: &'static str, pub volume: f32, pub src: Source3d }

/// The per-vehicle fields the contact sounds read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactVehicle {
    /// +0xd0 the sound class ([`VehicleClass`]).
    pub class: i32,
    /// +0x168: 0 for the class-0xE ctor (player car, traffic), 1 for every other vehicle
    /// (`FUN_002defd0`); such vehicles' impacts go in `Smackables` after a metal-gate clang.
    pub smackable: bool,
    /// +0x13c the smoothed speed (`VehicleSound::speed`).
    pub speed: f32,
    /// +0xd4..+0xe0 this frame's wheel surfaces (0 = no contact) and +0xe4..+0xf0 last frame's.
    pub wheels: [u32; 4],
    pub prev_wheels: [u32; 4],
    /// +0x10 velocity (the landing one-shots copy it).
    pub vel: V3,
    /// +0x150 the landing strength (writer not traced).
    pub landing: f32,
}

/// The game clock `FUN_00261038` [M]: a counter the EE timer-1 interrupt `FUN_00260f70` raises by
/// one per tick. `FUN_00173c60` ("Init Timer") arms it with `FUN_00260d38(rate)`, rate = the
/// video mode's entry of the table at 0x3c1a38 [D: 60, 60, 50, 60] picked by the boot option
/// (`ntsc` / none → 1, `pal` → 2, `pal60` → 3; `FUN_00174bf0`, `FUN_00173c60`). So on an NTSC
/// boot the clock counts 60 per second, independent of the frame rate: the impact cooldown (5)
/// is 1/12 s and the explosion cooldown (30) half a second.
pub const GAME_CLOCK_HZ: i32 = 60;

/// The video-mode rate table at 0x3c1a38 [D] that `FUN_002d6b90` reads.
pub const CLOCK_RATES: [i32; 4] = [60, 60, 50, 60];

/// `FUN_00260d38` [M]: the timer-1 setup for `hz` ticks a second (0 → 100). Returns (mode, target,
/// achieved rate): below 196 Hz the bus clock / 256 (576000), below 3150 Hz bus / 16, else the bus
/// clock; target = (hz / 2 + base) / hz as u16; rate = base / target.
pub fn clock_timer(hz: i32) -> (u32, u16, u32) {
    let hz = if hz == 0 { 100 } else { hz };
    let (mode, base) = if hz < 0xc4e { if hz < 0xc4 { (2, 0x8ca00) } else { (1, 0x8ca000) } } else { (0, 0x8ca0000) };
    let target = ((hz / 2 + base) / hz) as u16;
    (mode, target, base as u32 / target as u32)
}

/// The shared impact state: the cooldown slots `DAT_00355cd8[...]` (game-clock values, see
/// [`GAME_CLOCK_HZ`]) and the metal-gate alternation counter `DAT_00355d0c`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImpactState { pub cooldown: [i32; 16], pub gate: u32 }

fn shot(slot: usize, sfx: &'static str, group: &'static str, volume: f32, pos: V3, vel: V3) -> OneShot3d {
    OneShot3d { slot, sfx, group, volume, src: Source3d::at(pos, vel) }
}

fn pick(table: &[(f32, &'static str)], p: f32, now: i32, st: &mut ImpactState) -> Option<&'static str> {
    let n = table.len();
    let mut i = 0usize;
    if table[0].0 < p {
        i = 1;
        while i < n - 1 && table[i].0 < p { i += 1; }
    }
    // skip every slot still cooling down (the game reads on past the table; so does this)
    while i < st.cooldown.len() && now < st.cooldown[i] { i += 1; }
    if i < n && table[i].0 < p + 0.3 {
        st.cooldown[i] = now + 5;
        Some(table[i].1)
    } else {
        None
    }
}

/// `FUN_002e0890` [M]: one contact on a vehicle's sound object. `impulse` = |event +0x48|, `code`
/// = [`contact_code`], `pos` / `vel` = the event's +4 / +0x24 vectors, `now` = the game clock
/// (`FUN_00261038`). Raises the three scrape counters (`VehicleSound::scrape`: rock / metal / sand)
/// and returns the one-shots. While any counter is above 3 a contact is a scrape, not an impact.
pub fn impact(v: &ContactVehicle, scrape: &mut [i32; 3], impulse: f32, code: i32, pos: V3, vel: V3, now: i32, st: &mut ImpactState) -> Vec<OneShot3d> {
    let mut out = Vec::new();
    // +0x180 metal (code 0xd), +0x184 sand (code 5), +0x17c rock (the rest); +3 while below 10
    let k = match code { 0xd => 1, 5 => 2, _ => 0 };
    if scrape[k] < 10 { scrape[k] += 3; }
    if scrape.iter().any(|&c| 3 < c) { return out; }
    if code == 1 {
        let w = v.wheels;
        let any = if w[0] != 0 { true } else if w[1] == 0 { w[3] != 0 } else if w[2] != 0 { true } else { w[3] != 0 };
        if any {
            // the ctor volume 1.0 is kept; the one-shot rides the vehicle's velocity
            out.push(shot(1, "SFX_TiresHeavy", "Landings", 1.0, pos, v.vel));
            return out;
        }
    }
    let mut p = impulse;
    if code == 0xd && v.class != 4 { p += 0.35; }
    if !(0.05 <= p || st.cooldown[0] < 1) { return out; }
    if v.smackable {
        let sfx = METAL_GATES[(st.gate & 1) as usize];
        st.gate = st.gate.wrapping_add(1);
        let vol = if v.speed < 5.0 { v.speed * 0.2 } else { 1.0 };
        out.push(shot(2, sfx, "Smackables", vol, pos, vel));
    }
    let vol = if v.speed < 2.0 { v.speed * 0.5 } else { 1.0 };
    let (table, group): (&[(f32, &'static str)], &'static str) = match v.class {
        4 if code == 5 => (&IMPACTS_C, "Collisions"),
        4 => (&IMPACTS_B, "Collisions"),
        5 => return out,
        _ => (&IMPACTS_A, if v.smackable { "Smackables" } else { "Collisions" }),
    };
    if let Some(sfx) = pick(table, p, now, st) { out.push(shot(1, sfx, group, vol, pos, vel)); }
    out
}

/// `FUN_002dfa00` [M]: the landing thump. Nothing unless a wheel was off the ground last frame;
/// volume = min(strength + 0.1, 1). Strength >= 0.5: by the number of wheels that were airborne
/// (1: `TiresHeavy`, 2-3: + `Light4`, 4: + `Med3`); below: by this frame's surfaces (paved
/// `TiresHeavy`, wood 7 `TiresWood`, ice/snow `TiresSnow`, else `TiresHeavy`).
pub fn landing(v: &ContactVehicle, pos: V3) -> Vec<OneShot3d> {
    let airborne = v.prev_wheels.iter().filter(|&&s| s == 0).count();
    if airborne < 1 { return Vec::new(); }
    let vol = (v.landing + 0.1).min(1.0);
    let one = |k: &'static str| shot(1, k, "Landings", vol, pos, v.vel);
    if 0.5 <= v.landing {
        return match airborne {
            1 => vec![one("SFX_TiresHeavy")],
            2 | 3 => vec![one("SFX_TiresHeavy"), one("SFX_Light4")],
            _ => vec![one("SFX_TiresHeavy"), one("SFX_Med3")],
        };
    }
    let (mut wood, mut paved, mut snow) = (false, false, false);
    for &s in &v.wheels {
        if s == 7 { wood = true } else if s == 1 { paved = true } else if s == 8 || s == 9 { snow = true }
    }
    vec![one(if paved { "SFX_TiresHeavy" } else if wood { "SFX_TiresWood" } else if snow { "SFX_TiresSnow" } else { "SFX_TiresHeavy" })]
}

// ---------------------------------------------------------------------------------------------
// Horns (FUN_002de9b8 / FUN_002dea48)
// ---------------------------------------------------------------------------------------------

/// The horn object (0x7c bytes): +0 volume, +4 pitch (`SFX_HORNPITCH`), +8 the `Horns` group, the
/// voice (the vehicle's `SFX_HORN` key from slot 0), then three counters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Horn {
    pub volume: f32,
    pub pitch: f32,
    /// +0x70 honks left in the burst, +0x74 ticks of the current honk, +0x78 ticks of the gap.
    pub count: u32,
    pub on: u32,
    pub gap: u32,
}

impl Horn {
    /// `FUN_002df538` -> `FUN_002de9b8` [M]: the traffic ctor passes volume 1.0 and the attribute
    /// pitch; the counters start at 0.
    pub fn new(volume: f32, pitch: f32) -> Horn { Horn { volume, pitch, count: 0, on: 0, gap: 0 } }

    /// `FUN_002dea48` [M 0x2dea48-0x2deba4], once per audio update. `held` = the vehicle's +0x154
    /// (the caller clears it after the call). Returns the horn voice's (volume, pitch, pan).
    pub fn step(&mut self, held: bool, rng: &mut DriveRng, l: &Listener, mix: &mut Mixer) -> (f32, f32, f32) {
        if self.on != 0 {
            self.on -= 1;
            if self.on == 0 {
                self.gap = if self.count == 0 { rng.next() % 240 + 120 } else { rng.next() % 30 + 5 };
            }
        } else if held {
            if self.count != 0 { self.count -= 1 } else { self.count = rng.next() % 3 + 3 }
            if self.gap != 0 { self.gap -= 1 } else { self.on = rng.next() % 60 + 5 }
        }
        let v = if self.on != 0 { mix.volume_of("Horns") * self.volume } else { 0.0 };
        (voice_gate(v * l.gain), self.pitch * l.pitch, l.pan)
    }
}

// ---------------------------------------------------------------------------------------------
// The traffic vehicle (FUN_002e5340 / FUN_002e5598 / FUN_002df8a0)
// ---------------------------------------------------------------------------------------------

/// `FUN_002e5340` [M]: the traffic voices from slot 0 — `SFX_<car>LdEn` / `SFX_<car>LdEx` (the
/// two load layers) and the road voice (`SFX_tanktreads` in `Tank Treads` for the tank, else
/// `SFX_TrafficRN`). Sleds (`supsled` / `oppsled`) get class 5.
pub fn traffic_voices(car: &str) -> [String; 3] {
    let road = if car.eq_ignore_ascii_case("tank") { "SFX_tanktreads" } else { "SFX_TrafficRN" };
    [format!("SFX_{car}LdEn"), format!("SFX_{car}LdEx"), road.to_string()]
}

/// `FUN_002df8a0` [M]: the rpm a traffic vehicle's sound follows. With a horn (+0x1f8) it is a
/// gear model on the smoothed speed: speed * 5000 / g, g = 15 below 15, else the first of
/// 30, 45 .. 135 above the speed (135 past it); plus the wobble. Without one, +0x11c + wobble.
/// (The vtable +0x44 branch — a random rpm for one class — is not ported.)
pub fn traffic_rpm(has_horn: bool, speed: f32, rpm: f32, wobble: f32) -> f32 {
    if !has_horn { return rpm + wobble; }
    let mut g = 15.0f32;
    if 15.0 <= speed {
        let mut k = 2;
        loop {
            g = k as f32 * 15.0;
            if speed < g || k >= 9 { break; }
            k += 1;
        }
    }
    speed * 5000.0 / g + wobble
}

/// The traffic engine state (+0x230 rpm, +0x234 load, +0x238 the per-car pitch offset).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TrafficEngine { pub rpm: f32, pub load: f32, pub pitch_offset: f32 }

impl TrafficEngine {
    /// `FUN_002e5598` [M] after the base step: rpm slews 200/tick toward the target (10000 / 0 by
    /// throttle when a rear wheel has no contact); a drop past the step falls 100/tick and aims the
    /// load at 0.125, else at 1; the load slews 16/127 per tick. pitch = (rpm + 4000) / 7200 +
    /// offset. The En / Ex layers = load * +0x6c * group * 0.66 * (1 -/+ cone/2); the road voice =
    /// min(+0x6c * group * (speed - 2) / 15 * 0.5, 0.5) above speed 2. `audible` = +0x164.
    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, target_in: f32, throttle: f32, rear_free: bool, speed: f32, audible: bool, own: f32, group: f32, ticks: u32, l: &Listener, keys: &'static [&'static str; 3]) -> [VoiceOut; 3] {
        let t = ticks as f32;
        let target = if rear_free { if 0.0 < throttle { 10000.0 } else { 0.0 } } else { target_in };
        let step = t * 200.0;
        let mut aim = 1.0;
        let d = target - self.rpm;
        if d <= -step { aim = 0.125; self.rpm -= step * 0.5; } else if step <= d { self.rpm += step; } else { self.rpm = target; }
        let ls = t * 0.125_984_25;
        let dl = aim - self.load;
        if dl <= -ls { self.load -= ls } else if ls <= dl { self.load += ls } else { self.load = aim }
        let pitch = (self.rpm + 4000.0) / 7200.0 + self.pitch_offset;
        let vol = if audible { self.load * own * group } else { 0.0 };
        let en = (1.0 - l.cone * 0.5) * 0.66 * vol * l.gain;
        let ex = (l.cone * 0.5 + 1.0) * 0.66 * vol * l.gain;
        let road = if 2.0 < speed { (own * group * (speed - 2.0) * 0.066_666_67 * 0.5).min(0.5) } else { 0.0 };
        [
            VoiceOut { sfx: keys[0], volume: voice_gate(en), pitch: pitch * l.pitch },
            VoiceOut { sfx: keys[1], volume: voice_gate(ex), pitch: pitch * l.pitch },
            VoiceOut { sfx: keys[2], volume: voice_gate(road * l.gain), pitch: l.pitch },
        ]
    }
}

// ---------------------------------------------------------------------------------------------
// Car weapons (FUN_0020de20) and the vehicle classes
// ---------------------------------------------------------------------------------------------

/// `FUN_0020de20` [M]: the fire one-shot per car-weapon index (the switch after a shot is taken).
/// Other weapons fire through the projectile code (`FUN_0014ca60`: key `SFX_<weapon name>Fire`
/// from slot 4; its per-type jump table is not traced).
pub fn car_weapon_fire(weapon: i32) -> Option<OneShot> {
    let (slot, sfx, group) = match weapon {
        3 => (4, "SFX_boostFire", "Weapons Fire"),
        0xb => (4, "SFX_smkscrnFire", "Weapons Fire"),
        0xc => (0, "SFX_uploadcode", "Menu Sounds"),
        0xf => (0, "SFX_downloadcode", "Menu Sounds"),
        0x12 => (4, "SFX_twowheelfire", "Weapons Fire"),
        _ => return None,
    };
    Some(OneShot { slot, sfx, group })
}

/// `FUN_0020de20` [M]: the dry-fire click when the target only takes the EMP (and the weapon is
/// not 0 or 0xe): `SFX_DryFire`, slot 4, `Weapons Fire`.
pub const DRY_FIRE: OneShot = OneShot { slot: 4, sfx: "SFX_DryFire", group: "Weapons Fire" };

/// The +0xd0 sound class each vehicle ctor writes [M].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VehicleClass { Default = 0, Tank = 1, Heli = 2, Boat = 4, Sled = 5, Ultralight = 6, Suv = 7 }

/// The ctor voices of the vehicle classes (their per-frame steps: `vehicle_sounds.rs` and
/// `class_sounds.rs`) [M ctor strings]:
/// (ctor, class, keys). `FUN_002f0f10` tank, `FUN_002f1428` helicopter, `FUN_002e7020` boat,
/// `FUN_002ed7c0` snowmobile, `FUN_002e35b8` ultralight, `FUN_002e4980` turret.
pub const CLASS_VOICES: [(u32, &str, &[&str]); 6] = [
    (0x2f0f10, "tank", &["SFX_TankTread"]),
    (0x2f1428, "heli", &["SFX_helichange", "SFX_heli"]),
    (0x2e7020, "boat", &["SFX_load", "SFX_jetcav1", "SFX_rudcav1"]),
    (0x2ed7c0, "snowmobile", &["SFX_snomachfire", "SFX_snowRN", "SFX_roadRN", "SFX_roadcarve", "SFX_carve", "SFX_idle", "SFX_cruz"]),
    (0x2e35b8, "ultralight", &["SFX_bultralightDive", "SFX_BultraliteLdLW", "SFX_BultraliteLdHI", "SFX_umntmachfire"]),
    (0x2e4980, "turret", &["SFX_TurretLR", "SFX_TurretLRStop", "SFX_TurretUD", "SFX_TurretUDStop", "SFX_Kachung", "SFX_servo", "SFX_servodown"]),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn car(class: i32) -> ContactVehicle {
        ContactVehicle { class, smackable: false, speed: 10.0, wheels: [1; 4], prev_wheels: [1; 4], vel: [0.0; 3], landing: 0.0 }
    }

    #[test]
    fn game_clock_is_sixty_a_second_on_ntsc() {
        // 576000 / 9600 = 60 interrupts a second
        assert_eq!(clock_timer(CLOCK_RATES[1]), (2, 9600, 60));
        assert_eq!(clock_timer(CLOCK_RATES[2]), (2, 11520, 50));
        assert_eq!(clock_timer(0), (2, 5760, 100));
        assert_eq!(clock_timer(1000).0, 1);
        assert_eq!(GAME_CLOCK_HZ, clock_timer(CLOCK_RATES[1]).2 as i32);
    }

    #[test]
    fn rng_matches_the_carry_chain() {
        // first outputs from the boot words, worked by hand from the 0x25a5ac disassembly:
        // t4 = 0x9e353f7d + 0x6fdf3b64 = 0x0e147ae1 (carry 1); t3 = 0x0702c49c + 0x0e147ae1 + 1 =
        // 0x15173f7e; t2 = 0xc624dd2f + 0x15173f7e = 0xdb3c1cad; t1 = 0x883126e9 + 0xdb3c1cad =
        // 0x636d4396 (carry 1); t0 = 0xf22d0e56 + 0x636d4396 + 1 = 0x559a51ed.
        let mut r = DriveRng::boot();
        assert_eq!(r.next(), 0x559a_51ed);
        assert_eq!(r.w[5], 0x6fdf_3b65);
        assert_eq!(DriveRng::seeded(0), DriveRng::boot());
        assert_eq!(DriveRng::seeded(1).w[0], 0xf22d_0e57);
    }

    #[test]
    fn distance_attenuation_traced_values() {
        let l = Listener3d::look([0.0; 3], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0; 3]);
        // inside min (3): full gain
        assert_eq!(listen(&l, &Source3d::at([0.0, 0.0, 2.0], [0.0; 3])).gain, 1.0);
        // d = 100: (350 - 100) / (347 + 97 * 8) = 250 / 1123
        let o = listen(&l, &Source3d::at([0.0, 0.0, 100.0], [0.0; 3]));
        assert!((o.gain - 250.0 / 1123.0).abs() < 1e-6, "{}", o.gain);
        assert_eq!(o.distance, 100.0);
        assert!((o.send - 75.0 / 400.0).abs() < 1e-6);
        // past max: silent
        assert_eq!(listen(&l, &Source3d::at([0.0, 0.0, 351.0], [0.0; 3])).gain, 0.0);
        // doppler: source moving away at 40/tick -> 400 / 440
        let o = listen(&l, &Source3d::at([0.0, 0.0, 10.0], [0.0, 0.0, 40.0]));
        assert!((o.pitch - 400.0 / 440.0).abs() < 1e-6);
        // listener moving toward it at 40 -> 400 / 360
        let l2 = Listener3d { vel: [0.0, 0.0, 40.0], ..l };
        assert!((listen(&l2, &Source3d::at([0.0, 0.0, 10.0], [0.0; 3])).pitch - 400.0 / 360.0).abs() < 1e-6);
        // non-positional (max 0): gain 1, pan 0 for the index-2 listener, -0.25 / +0.25 for 0 / 1
        let s = Source3d { max: 0.0, ..Source3d::at([50.0, 0.0, 0.0], [0.0; 3]) };
        assert_eq!(listen(&l, &s).pan, 0.0);
        assert_eq!(listen(&Listener3d { index: 0, ..l }, &s).pan, -0.25);
        assert_eq!(listen(&Listener3d { index: 1, ..l }, &s).pan, 0.25);
    }

    #[test]
    fn pan_near_field_and_cone() {
        let l = Listener3d::look([0.0; 3], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0; 3]);
        // d < min/2 (1.5): pan 0
        assert_eq!(listen(&l, &Source3d::at([1.0, 0.0, 0.0], [0.0; 3])).pan, 0.0);
        // d = 2 on the right: 0.25 * (4 - 3) / 3
        let o = listen(&l, &Source3d::at([2.0, 0.0, 0.0], [0.0; 3]));
        assert!((o.pan - 0.25 / 3.0).abs() < 1e-6, "{}", o.pan);
        // far right: a quarter turn
        assert!((listen(&l, &Source3d::at([20.0, 0.0, 0.0], [0.0; 3])).pan - 0.25).abs() < 1e-6);
        // behind: half a turn
        assert!((listen(&l, &Source3d::at([0.0, 0.0, -20.0], [0.0; 3])).pan.abs() - 0.5).abs() < 1e-6);
        // cone = dot(listener->source direction, the source's facing (0,0,1))
        assert!((listen(&l, &Source3d::at([0.0, 0.0, 20.0], [0.0; 3])).cone - 1.0).abs() < 1e-6);
    }

    #[test]
    fn pan_law_stereo_levels() {
        // speaker 0 at 0x4000 -> VOLR; speaker 1 at 0xc000 -> VOLL
        assert_eq!(stereo_levels(0x4000), (0, 127));
        assert_eq!(stereo_levels(0xc000), (127, 0));
        // centre front: 63 each side through the curve -> 95
        assert_eq!(stereo_levels(0), (95, 95));
        assert_eq!(pan_curve(63), 95);
        // 45 degrees right: (to0, from1) = (0x2000, 0x6000) -> curve(95), curve(31)
        assert_eq!(stereo_levels(0x2000), (55, 119));
        assert_eq!(pan_azimuth(0.25, 0), 0x4000);
        assert_eq!(pan_azimuth(-0.25, 0), 0xc000);
        assert_eq!(patch_azimuth(64), Some(0));
    }

    #[test]
    fn a_source_on_the_listeners_right_is_in_the_right_channel() {
        // driving rule: right = up x forward. Facing +z with +y up, right is +x.
        let l = Listener3d::look([10.0, 2.0, 5.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [0.0; 3]);
        assert_eq!(l.right, [1.0, 0.0, 0.0]);
        let o = listen(&l, &Source3d::at([30.0, 2.0, 5.0], [0.0; 3]));
        let (left, right) = stereo_levels(pan_azimuth(o.pan, 0));
        assert!(right > left, "pan {} -> L {left} R {right}", o.pan);
        assert_eq!((left, right), (0, 127));
        let o = listen(&l, &Source3d::at([-10.0, 2.0, 5.0], [0.0; 3]));
        let (left, right) = stereo_levels(pan_azimuth(o.pan, 0));
        assert!(left > right);
        // a turned listener (facing +x): its right is -z
        let l = Listener3d::look([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0; 3]);
        let o = listen(&l, &Source3d::at([0.0, 0.0, -20.0], [0.0; 3]));
        let (left, right) = stereo_levels(pan_azimuth(o.pan, 0));
        assert!(right > left, "facing +x, -z is right");
    }

    #[test]
    fn the_chase_camera_basis_pans_its_right_to_the_right_channel() {
        use super::super::camera::{default_heli_cam, CarPose, ChaseCamera};
        let (p, arm) = default_heli_cam("vanquish");
        let car = CarPose::from_yaw([0.0, 0.0, 0.0], 0.7, [0.0; 3]);
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let v = cam.update(&p, &arm, &car, false, &mut |_, _| None);
        let l = Listener3d::from_view(&v, [0.0; 3]);
        let src = [v.eye[0] + v.right[0] * 15.0, v.eye[1] + v.right[1] * 15.0, v.eye[2] + v.right[2] * 15.0];
        let o = listen(&l, &Source3d::at(src, [0.0; 3]));
        let (left, right) = stereo_levels(pan_azimuth(o.pan, 0));
        assert!(right > left, "camera right -> right channel (pan {})", o.pan);
        // and that right is the driving rule's up x forward
        let r = norm(cross(v.up, v.fwd));
        assert!(dot(r, v.right) > 0.99);
    }

    #[test]
    fn impact_tables_cooldowns_and_scrapes() {
        let mut st = ImpactState::default();
        let mut sc = [0; 3];
        let v = car(0);
        // impulse 0.33: first entry above it is Med1 (0.35); 0.35 < 0.63 -> plays
        let s = impact(&v, &mut sc, 0.33, 0, [0.0; 3], [0.0; 3], 100, &mut st);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].sfx, s[0].group, s[0].slot), ("SFX_Med1", "Collisions", 1));
        assert_eq!(s[0].volume, 1.0);
        assert_eq!(st.cooldown[6], 105);
        assert_eq!(sc, [3, 0, 0]);
        // second hit while the rock counter is 3 -> 6 (> 3): a scrape, no impact
        assert!(impact(&v, &mut sc, 0.33, 0, [0.0; 3], [0.0; 3], 101, &mut st).is_empty());
        // counters decayed; Med1 is still cooling down -> the next slot, Med2 (0.40 < 0.63)
        let mut sc = [0; 3];
        let s = impact(&v, &mut sc, 0.33, 0, [0.0; 3], [0.0; 3], 102, &mut st);
        assert_eq!(s[0].sfx, "SFX_Med2");
        // metal (0xd) adds 0.35 for non-boat classes and raises the metal counter
        let mut sc = [0; 3];
        let mut st = ImpactState::default();
        let s = impact(&v, &mut sc, 0.0, 0xd, [0.0; 3], [0.0; 3], 0, &mut st);
        assert_eq!(s[0].sfx, "SFX_Med1");
        assert_eq!(sc, [0, 3, 0]);
        // boat (class 4): table B; on sand (5) table C
        let mut sc = [0; 3];
        let mut st = ImpactState::default();
        assert_eq!(impact(&car(4), &mut sc, 0.22, 0, [0.0; 3], [0.0; 3], 0, &mut st)[0].sfx, "SFX_Light4");
        let mut sc = [0; 3];
        assert_eq!(impact(&car(4), &mut sc, 0.22, 5, [0.0; 3], [0.0; 3], 0, &mut st)[0].sfx, "SFX_sandbump2");
        // sleds (class 5): nothing
        let mut sc = [0; 3];
        assert!(impact(&car(5), &mut sc, 0.5, 0, [0.0; 3], [0.0; 3], 0, &mut st).is_empty());
        // smackable: the metal gate first (alternating), then the table in Smackables; slow speed
        let mut st = ImpactState::default();
        let mut sc = [0; 3];
        let sm = ContactVehicle { smackable: true, speed: 1.0, ..car(0) };
        let s = impact(&sm, &mut sc, 0.33, 0, [0.0; 3], [0.0; 3], 0, &mut st);
        assert_eq!((s[0].sfx, s[0].slot, s[0].volume), ("SFX_MetalGate2", 2, 0.2));
        assert_eq!((s[1].sfx, s[1].group, s[1].volume), ("SFX_Med1", "Smackables", 0.5));
        let mut sc = [0; 3];
        assert_eq!(impact(&sm, &mut sc, 0.33, 0, [0.0; 3], [0.0; 3], 10, &mut st)[0].sfx, "SFX_MetalGate3");
        // code 1 with a wheel on the ground: the tyre thump only
        let mut sc = [0; 3];
        let s = impact(&v, &mut sc, 0.5, 1, [0.0; 3], [0.0; 3], 0, &mut st);
        assert_eq!((s.len(), s[0].sfx, s[0].group), (1, "SFX_TiresHeavy", "Landings"));
        // too soft for anything (0.04 < 0.05) once cooldown[0] is set
        let mut st = ImpactState { cooldown: [9; 16], gate: 0 };
        let mut sc = [0; 3];
        assert!(impact(&v, &mut sc, 0.04, 0, [0.0; 3], [0.0; 3], 0, &mut st).is_empty());
    }

    #[test]
    fn contact_codes_and_calls() {
        assert_eq!(contact_code(16), 13);
        assert_eq!(contact_code(80), 5);
        assert_eq!(contact_code(96), 15);
        assert_eq!(contact_code(6), 0);
        assert_eq!(contact_code_b(7, 0), 1);
        assert_eq!(contact_code_b(0, 0x10), 0xd);
        assert_eq!(contact_code_b(0, 3), 0xf);
        assert_eq!(contact_call_a(0, 0, 0xe), ContactCall::Landing);
        assert_eq!(contact_call_a(1, 0, 48), ContactCall::Impact(9));
        assert_eq!(contact_call_a(5, 0, 48), ContactCall::None);
    }

    #[test]
    fn landings() {
        let mut v = car(0);
        assert!(landing(&v, [0.0; 3]).is_empty(), "all wheels were down");
        v.prev_wheels = [0, 0, 1, 1];
        v.landing = 0.7;
        let s = landing(&v, [0.0; 3]);
        assert_eq!(s.iter().map(|x| x.sfx).collect::<Vec<_>>(), ["SFX_TiresHeavy", "SFX_Light4"]);
        assert!((s[0].volume - 0.8).abs() < 1e-6);
        v.prev_wheels = [0; 4];
        assert_eq!(landing(&v, [0.0; 3])[1].sfx, "SFX_Med3");
        v.landing = 0.2;
        v.wheels = [7, 8, 0, 0];
        assert_eq!(landing(&v, [0.0; 3])[0].sfx, "SFX_TiresWood");
        v.wheels = [0, 9, 0, 0];
        assert_eq!(landing(&v, [0.0; 3])[0].sfx, "SFX_TiresSnow");
    }

    #[test]
    fn horn_timing() {
        let ini = nf_formats::easnd::parse_mix_ini(b"Horns = 0.500, 4\r\n");
        let mut m = Mixer::driving(&ini, 100, 100);
        let l = Listener::default();
        let mut rng = DriveRng::boot();
        let mut h = Horn::new(1.0, 1.2);
        // not held: nothing happens
        assert_eq!(h.step(false, &mut rng, &l, &mut m).0, 0.0);
        assert_eq!(rng, DriveRng::boot());
        // held: count reloads (rand % 3 + 3), gap 0 -> a honk of rand % 60 + 5 starts
        let mut r2 = DriveRng::boot();
        let c = r2.next() % 3 + 3;
        let on = r2.next() % 60 + 5;
        let (v, p, _) = h.step(true, &mut rng, &l, &mut m);
        assert_eq!((h.count, h.on), (c, on));
        assert!((v - 0.5).abs() < 1e-6, "Horns 0.5 x the fader chain 1.0: {v}");
        assert!((p - 1.2).abs() < 1e-6);
        // the honk runs down regardless of the request, then a short gap (count still > 0)
        for _ in 0..on - 1 { h.step(false, &mut rng, &l, &mut m); }
        assert_eq!(h.on, 1);
        let g = r2.next() % 30 + 5;
        assert_eq!(h.step(false, &mut rng, &l, &mut m).0, 0.0);
        assert_eq!(h.gap, g);
    }

    #[test]
    fn traffic_engine_and_rpm() {
        assert_eq!(traffic_rpm(true, 10.0, 0.0, 0.0), 10.0 * 5000.0 / 15.0);
        assert_eq!(traffic_rpm(true, 20.0, 0.0, 0.0), 20.0 * 5000.0 / 30.0);
        assert_eq!(traffic_rpm(true, 50.0, 0.0, 0.0), 50.0 * 5000.0 / 60.0);
        assert_eq!(traffic_rpm(true, 500.0, 0.0, 0.0), 500.0 * 5000.0 / 135.0);
        assert_eq!(traffic_rpm(false, 50.0, 1200.0, 30.0), 1230.0);
        assert_eq!(traffic_voices("bmw")[0], "SFX_bmwLdEn");
        assert_eq!(traffic_voices("tank")[2], "SFX_tanktreads");
        static K: [&str; 3] = ["SFX_aLdEn", "SFX_aLdEx", "SFX_TrafficRN"];
        let mut e = TrafficEngine::default();
        let l = Listener { cone: 1.0, ..Listener::default() };
        let v = e.update(3000.0, 1.0, false, 17.0, true, 1.0, 1.0, 1, &l, &K);
        assert_eq!(e.rpm, 200.0);
        assert!((e.load - 0.125_984_25).abs() < 1e-7);
        assert!((v[0].pitch - 4200.0 / 7200.0).abs() < 1e-6);
        // cone 1: the exhaust layer 3x the engine layer
        assert!((v[1].volume / v[0].volume - 3.0).abs() < 1e-4);
        // road: (17 - 2)/15 * 0.5 = 0.5 (the cap)
        assert!((v[2].volume - 0.5).abs() < 1e-6);
        // a drop past 200/tick: -100/tick and the load heads for 0.125
        let mut e = TrafficEngine { rpm: 5000.0, load: 1.0, pitch_offset: 0.0 };
        e.update(0.0, 0.0, false, 0.0, true, 1.0, 1.0, 1, &l, &K);
        assert_eq!(e.rpm, 4900.0);
        assert!((e.load - (1.0 - 0.125_984_25)).abs() < 1e-6);
    }

    #[test]
    fn car_weapon_fire_table() {
        assert_eq!(car_weapon_fire(3).unwrap().sfx, "SFX_boostFire");
        assert_eq!(car_weapon_fire(0xc).unwrap().group, "Menu Sounds");
        assert!(car_weapon_fire(2).is_none());
    }
}
