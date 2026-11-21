//! The per-frame sound steps of the boat, the snowmobile, the turret and the tank (DRIVING.ELF
//! vtable +0x24 of each class). docs/systems/drive-audio.md is the write-up.
//!
//! Evidence (all [M], DRIVING.ELF decompile + disassembly, EE addresses; [D] disc data):
//! - Boat `FUN_002e7358` = base step `FUN_002e1768`, then `FUN_002e74d8` (load voice),
//!   `FUN_002e7d60` (servo one-shots), `FUN_002e76f8` (jet / rudder voices), `FUN_002e73b8`
//!   (creaks). Ctor `FUN_002e7020`: voices `SFX_load` (+0x228), `SFX_jetcav1` (+0x220),
//!   `SFX_rudcav1` (+0x224) from slot 9 (`PTR_DAT_003562c4`), group `Player Car`; +0x258 = 0.06
//!   (0x3d75c28f), the fade step of both the jet and the rudder.
//! - Boat inputs (the bridge `FUN_00188200`, 8-byte stride stores): +0x18c = car +0x278 > 0 (the
//!   GAMEACTION_GAS value), +0x190 = car +0x27c > 0 (GAMEACTION_BRAKE), +0x140 = car +0x288
//!   (|velocity|, `FUN_0018cef8`), +0x19c = car +0x240 (the ramped steer), +0x1a0 = car +0x244
//!   (the ramped vertical steer). +0x194 / +0x198 are raised by the car's action drain
//!   `FUN_0018add8` on GAMEACTION_SOUND_STEERLEFT..DOWN (0xc..0xf) / SOUND_STEERHORIZONTALRELEASE
//!   and SOUND_STEERVERTICALRELEASE (0x10 / 0x11) (names from the table at 0x32c098).
//! - Creaks: `ESetCreakLevel` (script action 0xff, creator `FUN_00157e68`, handler
//!   `FUN_00157cf8`) calls the boat's vtable +0x54 `FUN_002e8650(level = word 1 as s16,
//!   interval = word 2)` when the player car's status word (+0xbc of its vtable +0x1d4 block) is
//!   above 0. The interval jitter is the polar normal sample `FUN_002ece10` over libc `rand`
//!   (`FUN_002a8bd0`).
//! - Snowmobile `FUN_002edd88`: forward / side speeds = (+0x50) · the forward row (+0x20) / the
//!   right row (+0x30), the wind object `FUN_002feaa0` (or `FUN_002fed98` when not moving
//!   forward), the base step, `FUN_002ee768` (road / carve voices and the landing thumps),
//!   `FUN_002ede60` (engine idle / cruise / rev and the free-wheel one-shot). Ctor `FUN_002ed7c0`:
//!   `SFX_roadRN` +0x214, `SFX_snowRN` +0x218, `SFX_roadcarve` +0x21c, `SFX_carve` +0x220 (slot 0),
//!   `SFX_idle` +0x224, `SFX_cruz` +0x228 (slot 9); +0x230 = group `OffRoad`; +0x254 / +0x258 the
//!   front rest lengths 0.119527 / 0.119531. The wind object `FUN_002fe9c0`: `SFX_windy` from slot
//!   4 (`PTR_DAT_003562b0`), group `Player Wind`, level 0.4 / rate -0.1, pitch 1 / rate -0.1, pan
//!   0.5. Its random rates come from `FUN_002d0aa8` = the driving generator `FUN_0025a5ac` × 2^-32.
//! - The sound's +0xf4..+0x100 = car +0x210..+0x21c, the four wheels' compressions from the last
//!   wheel pass (`FUN_0018c1b0` / `FUN_0018e548` store them at the end of each wheel's step).
//! - Turret `FUN_002e4980(state, lr = +0xb8, ud = +0xbc, servo = +0xb0, listener)`, called from the
//!   base step for any vehicle with a turret at +0x1a8 and from the tank step. Ctor `FUN_002e4930`:
//!   group `Weapons Init`, every voice null.
//! - Tank `FUN_002f1000`: base step, then the engine object at +0x22c (`FUN_002faae8(4500, ..)`,
//!   the same 0x54-byte object and vtable +0xc `FUN_002ff320` → `FUN_002fa8f0` as the player car's),
//!   then the treads, then the turret at +0x210.
//!
//! Unproven (stated plainly): what the sound's +0x50 vector is (it comes from `FUN_001bcca0(1.0,
//! view set +4)`, the snowmobile reads it as a velocity); who writes the turret inputs +0xb8 /
//! +0xbc and the meaning of the bridge's +0xb0; a voice's "finished" test (its +0x60 word and the
//! per-channel words under `DAT_00355cd0`) is taken as "the sample has ended"; callers supply it;
//! `DAT_00355e5c` (copied from the mission manager's +0x3cc by `FUN_002065d0` / `FUN_00206620`)
//! is passed in as `muted`.

use super::audio::{voice_gate, EngineFrame, EngineSound, Listener, Mixer, VehicleFrame, VehicleInputs, VehicleSound, PLAYER_ENGINE_RPM};
use super::sound_events::{DriveRng, OneShot3d, Source3d, V3};
use super::vehicle_sounds::{tank_treads, PanVoice};

fn pv(sfx: &'static str, volume: f32, pitch: f32, pan: f32) -> PanVoice { PanVoice { sfx, volume: voice_gate(volume), pitch, pan } }

/// The fixed per-step jet / rudder fade of the boat (ctor +0x258 = 0x3d75c28f).
pub const BOAT_FADE: f32 = 0.06;

/// The sound base's step ramp (+0x9c value, +0xa0 count, +0xa4 steps, +0xa8 running):
/// `FUN_002fcb88` starts it, `FUN_002fcba0` steps it [M].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ramp { pub value: f32, pub count: i32, pub steps: i32, pub running: bool }

impl Ramp {
    /// `FUN_002fcb88`: steps = n, running, count 0, value 0.
    pub fn start(&mut self, n: i32) { *self = Ramp { value: 0.0, count: 0, steps: n, running: true }; }
    /// `FUN_002fcba0`: not running → 1.0; else count < steps → count/steps after count += 1;
    /// at count == steps → 1.0 and stop.
    pub fn step(&mut self) -> f32 {
        if !self.running {
            self.value = 1.0;
        } else if self.steps <= self.count {
            self.count = 0;
            self.value = 1.0;
            self.running = false;
        } else {
            self.count += 1;
            self.value = self.count as f32 / self.steps as f32;
        }
        self.value
    }
}

/// `FUN_002ece10` [M]: the polar normal sample; `rand` = libc `rand()` (0..2^31). Returns one of a
/// pair and keeps the other for the next call (`DAT_00355e8c` / `DAT_00421c40`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gauss { pub spare: Option<f32> }

impl Gauss {
    pub fn sample(&mut self, mut rand: impl FnMut() -> i32) -> f32 {
        if let Some(v) = self.spare.take() { return v; }
        loop {
            let a = rand() as f32 * 4.656613e-10;
            let x = (a + a) - 1.0;
            let b = rand() as f32 * 4.656613e-10;
            let y = (b + b) - 1.0;
            let s = x * x + y * y;
            if s < 1.0 {
                let f = ((s as f64).ln() as f32 * -2.0 / s).sqrt();
                self.spare = Some(y * f);
                return x * f;
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Boat
// ---------------------------------------------------------------------------------------------

/// What the boat's step reads beyond the base state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoatInputs {
    /// +0x13c the smoothed speed (the base step's `VehicleSound::speed`).
    pub speed: f32,
    /// +0x11c rpm and +0x120 the base step's wobble.
    pub rpm: f32,
    pub wobble: f32,
    /// +0x18c gas pedal down, +0x190 brake pedal down.
    pub gas: bool,
    pub brake: bool,
    /// +0x140 |velocity| (car +0x288).
    pub body_speed: f32,
    /// +0x19c / +0x1a0 the ramped steer / vertical steer.
    pub steer: f32,
    pub steer_v: f32,
    /// +0x194 / +0x198 a steer-sound press / release this frame.
    pub servo_on: bool,
    pub servo_off: bool,
    /// +0x00 the boat's position (the one-shots copy it).
    pub pos: V3,
    /// +0x6c the sound's own volume (1.0 from the base ctor).
    pub own: f32,
}

/// The boat's sound state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BoatSound {
    /// +0x240 the load voice is up; the base ramp (+0x9c..).
    pub load_on: bool,
    pub ramp: Ramp,
    /// +0x24c / +0x250 the jet voice on / its level; +0x248 / +0x254 the rudder's.
    pub jet_on: bool,
    pub jet_level: f32,
    pub rudder_on: bool,
    pub rudder_level: f32,
    /// +0x238 / +0x234 / +0x236 the rudder-stop, servo-on, servo-off alternation counters.
    pub rudder_alt: i16,
    pub servo_on_alt: i16,
    pub servo_off_alt: i16,
    /// +0x25c creak level (1..4, 0 = off), +0x260 interval, +0x264 next wait, +0x268 counter.
    pub creak_level: i16,
    pub creak_interval: i32,
    pub creak_next: i32,
    pub creak_count: i32,
}

/// One boat step's output.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoatFrame {
    /// `SFX_load` (+0x228), when the step touched it.
    pub load: Option<PanVoice>,
    /// `SFX_jetcav1` (+0x220) / `SFX_rudcav1` (+0x224), when touched.
    pub jet: Option<PanVoice>,
    pub rudder: Option<PanVoice>,
    /// Positioned one-shots (slot 9, `One Shots`, min 3 / max 350, at the boat, no velocity).
    pub shots: Vec<OneShot3d>,
    /// A 2-D creak (slot 0, group `Ambience`).
    pub creak: Option<&'static str>,
}

pub const BOAT_VOICES: [&str; 3] = ["SFX_load", "SFX_jetcav1", "SFX_rudcav1"];
pub const BOAT_RUDDER_STOPS: [&str; 2] = ["SFX_rudcav2", "SFX_rudcav3"];
pub const BOAT_SERVO_ON: [&str; 2] = ["SFX_servon1", "SFX_servon2"];
pub const BOAT_SERVO_OFF: [&str; 2] = ["SFX_servoff1", "SFX_servoff2"];
/// The creak keys `FUN_002e73b8` indexes with level − 1 (0x3c31c0 [D]).
pub const BOAT_CREAKS: [&str; 4] = ["SFX_creak1", "SFX_creak2", "SFX_creak3", "SFX_creak4"];

/// The alternation the boat's one-shots use: counter + 1, wrapped to 0 above 1.
fn alt(c: &mut i16) -> i16 {
    *c += 1;
    if 1 < *c { *c = 0; }
    *c
}

fn boat_shot(sfx: &'static str, volume: f32, pos: V3) -> OneShot3d {
    OneShot3d { slot: 9, sfx, group: "One Shots", volume, src: Source3d::at(pos, [0.0; 3]) }
}

/// (rpm + wobble) × 0.0005556 + `add`, in double precision (`FUN_0024d078` / `FUN_0024cfb8`).
fn jet_pitch(rpm: f32, add: f64) -> f32 { ((rpm as f64) * f64::from_bits(0x3f42_34b5_ea32_3861) + add) as f32 }

impl BoatSound {
    /// `ESetCreakLevel` → `FUN_002e8650` [M].
    pub fn set_creak(&mut self, level: i16, interval: i32) { self.creak_level = level; self.creak_interval = interval; }

    /// `FUN_002e74d8` [M]: the load voice. Pitch = speed / 20 (× 0.85 with the brake down) ×
    /// doppler. Gas (or brake) down: the voice ramps up over 10 steps (started when it was off);
    /// both up: it ramps down over 10 steps, then goes silent and off.
    pub fn load_step(&mut self, i: &BoatInputs, group: f32, l: &Listener) -> Option<PanVoice> {
        let mut pitch = i.speed / 20.0;
        if i.brake { pitch *= 0.85; }
        let base = i.own * group * ((i.rpm + i.wobble) / 3300.0);
        let mut out = None;
        if i.gas || i.brake {
            if !self.load_on { self.ramp.start(10); }
            self.load_on = true;
            let r = self.ramp.step();
            out = Some(pv(BOAT_VOICES[0], base * r * l.gain, pitch * l.pitch, l.pan));
        }
        if !i.gas && !i.brake {
            if self.load_on {
                if !self.ramp.running { self.ramp.start(10); }
                let r = self.ramp.step();
                out = Some(pv(BOAT_VOICES[0], base * (1.0 - r) * l.gain, pitch * l.pitch, l.pan));
            }
            if !self.ramp.running {
                self.load_on = false;
                out = Some(pv(BOAT_VOICES[0], 0.0, l.pitch, l.pan));
            }
        }
        out
    }

    /// `FUN_002e7d60` [M]: a steer-sound press plays `SFX_servon2`, `SFX_servon1`, … in turn, a
    /// release `SFX_servoff2`, `SFX_servoff1`, … (volume own × group).
    pub fn servo_step(&mut self, i: &BoatInputs, group: f32, shots: &mut Vec<OneShot3d>) {
        if i.servo_on {
            let k = alt(&mut self.servo_on_alt);
            shots.push(boat_shot(BOAT_SERVO_ON[k as usize], i.own * group, i.pos));
        }
        if i.servo_off {
            let k = alt(&mut self.servo_off_alt);
            shots.push(boat_shot(BOAT_SERVO_OFF[k as usize], i.own * group, i.pos));
        }
    }

    /// `FUN_002e76f8` [M]: level = speed / 20 × own × group. The jet sounds while moving with the
    /// brake down (pitch (rpm + wobble) × 0.0005556 + 0.2), then fades by 0.06 a step. The rudder
    /// sounds while moving and steering, or on any vertical steer; on release it fades by 0.06 a
    /// step (that tail is not × the listener gain) and, once silent, plays `SFX_rudcav3`,
    /// `SFX_rudcav2`, … in turn at the level of that step.
    pub fn jet_rudder_step(&mut self, i: &BoatInputs, group: f32, l: &Listener, out: &mut BoatFrame) {
        let level = (i.speed / 20.0) * i.own * group;
        let rw = i.rpm + i.wobble;
        let v = i.body_speed;
        let jet_now = (0.0 < v && i.brake) || (v < 0.0 && i.gas);
        if jet_now {
            self.jet_on = true;
            out.jet = Some(pv(BOAT_VOICES[1], level * l.gain, jet_pitch(rw, f32::from_bits(0x3e4c_cccd) as f64) * l.pitch, l.pan));
            self.jet_level = level;
        } else if self.jet_on {
            self.jet_level -= BOAT_FADE;
            if 0.0 <= self.jet_level {
                out.jet = Some(pv(BOAT_VOICES[1], self.jet_level * l.gain, jet_pitch(rw, 0.2) * l.pitch, l.pan));
            } else {
                self.jet_level = 0.0;
                self.jet_on = false;
                out.jet = Some(pv(BOAT_VOICES[1], 0.0, l.pitch, l.pan));
            }
        }
        if (0.0 < v && i.steer != 0.0) || i.steer_v != 0.0 {
            self.rudder_on = true;
            out.rudder = Some(pv(BOAT_VOICES[2], level * l.gain, l.pitch, l.pan));
            self.rudder_level = level;
            return;
        }
        if !self.rudder_on { return; }
        self.rudder_level -= BOAT_FADE;
        if 0.0 <= self.rudder_level {
            out.rudder = Some(pv(BOAT_VOICES[2], self.rudder_level, l.pitch, l.pan));
            return;
        }
        self.rudder_level = 0.0;
        self.rudder_on = false;
        out.rudder = Some(pv(BOAT_VOICES[2], 0.0, l.pitch, l.pan));
        let k = alt(&mut self.rudder_alt);
        out.shots.push(boat_shot(BOAT_RUDDER_STOPS[k as usize], level, i.pos));
    }

    /// `FUN_002e73b8` [M]: with a creak level set, count steps; at the wait, draw the next wait
    /// = interval + round(N(0,1) × 0.1 × interval) (round half away from zero) and creak.
    pub fn creak_step(&mut self, gauss: impl FnOnce() -> f32) -> Option<&'static str> {
        if self.creak_level < 1 { return None; }
        if self.creak_count < self.creak_next {
            self.creak_count += 1;
            return None;
        }
        let x = gauss() * 0.1 * self.creak_interval as f32;
        let half = if x < 0.0 { -0.5 } else { 0.5 };
        self.creak_next = self.creak_interval + (half + x) as i32;
        self.creak_count = 0;
        // the game indexes a 4-entry stack table; a level above 4 reads past it (not modelled)
        BOAT_CREAKS.get(self.creak_level as usize - 1).copied()
    }

    /// `FUN_002e7358` after the base step: the four sub-steps in the game's order.
    pub fn update(&mut self, i: &BoatInputs, l: &Listener, mix: &mut Mixer, gauss: impl FnOnce() -> f32) -> BoatFrame {
        let group = mix.volume_of("Player Car");
        let mut out = BoatFrame { load: self.load_step(i, group, l), ..BoatFrame::default() };
        self.servo_step(i, group, &mut out.shots);
        self.jet_rudder_step(i, group, l, &mut out);
        out.creak = self.creak_step(gauss);
        out
    }
}

// ---------------------------------------------------------------------------------------------
// Snowmobile
// ---------------------------------------------------------------------------------------------

/// The wind object (`FUN_002fe9c0`, 0x24 bytes; field = word index).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wind {
    /// [3] level, [4] its rate per tick; [5] pitch, [6] its rate; [7] the forward speed; [8] pan.
    pub level: f32,
    pub level_rate: f32,
    pub pitch: f32,
    pub pitch_rate: f32,
    pub speed: f32,
    pub pan: f32,
}

impl Default for Wind {
    fn default() -> Self {
        Wind { level: f32::from_bits(0x3ecc_cccd), level_rate: f32::from_bits(0xbdcc_cccd), pitch: 1.0, pitch_rate: f32::from_bits(0xbdcc_cccd), speed: 1.0, pan: 0.5 }
    }
}

pub const WIND_VOICE: &str = "SFX_windy";

/// `FUN_002d0aa8` [M]: the driving generator's next word × 2^-32 (unsigned).
pub fn rand01(rng: &mut DriveRng) -> f32 { rng.next() as f32 * 2.328_306_4e-10 }

fn rate(r: f32, mul: f64, add: f64) -> f32 { ((r as f64) * mul + add) as f32 }

impl Wind {
    /// `FUN_002feaa0` [M], `speed` = the sled's smoothed speed (+0x13c, before this update's base
    /// step). Level and pitch drift by their rates × ticks; past 1.0 / below 0.4 the level's rate is
    /// redrawn as ∓(r·0.02 + 0.002), past 2.8 / below 1.0 the pitch's as −(r·0.01 + 0.001) /
    /// r·0.03 + 0.001. Volume = level × min((speed − 5) / 5, 1) × `Player Wind` × the forward
    /// speed × gain, pitch × doppler, pan 0.5.
    pub fn step(&mut self, ticks: u32, speed: f32, rng: &mut DriveRng, l: &Listener, mix: &mut Mixer) -> PanVoice {
        let t = ticks as f32;
        self.level += self.level_rate * t;
        self.pitch += self.pitch_rate * t;
        if 1.0 < self.level { self.level_rate = rate(rand01(rng), -0.02, -0.002); }
        if self.level < 0.4 { self.level_rate = rate(rand01(rng), 0.02, 0.002); }
        if 2.8 < self.pitch { self.pitch_rate = rate(rand01(rng), -0.01, -0.001); }
        if self.pitch < 1.0 { self.pitch_rate = rate(rand01(rng), 0.03, 0.001); }
        let mut ramp = 0.0;
        if 5.0 < speed {
            ramp = (speed - 5.0) * 0.2;
            if 1.0 < ramp { ramp = 1.0; }
        }
        let g = mix.volume_of("Player Wind");
        pv(WIND_VOICE, self.level * ramp * g * self.speed * l.gain, self.pitch * l.pitch, self.pan)
    }

    /// `FUN_002fed98` [M]: not moving forward: silent at its pan, the listener pitch.
    pub fn idle(&self, l: &Listener) -> PanVoice { pv(WIND_VOICE, 0.0, l.pitch, self.pan) }
}

pub const SLED_ROAD: [&str; 4] = ["SFX_roadRN", "SFX_snowRN", "SFX_roadcarve", "SFX_carve"];
pub const SLED_ENGINE: [&str; 2] = ["SFX_idle", "SFX_cruz"];
pub const SLED_REVS: [&str; 2] = ["SFX_rev1", "SFX_rev2"];
pub const SLED_FREEWHEEL: &str = "SFX_freewheel";
/// The landing thumps `FUN_002ee768` plays from slot 1 in `Landings`, by the front compression
/// against the rest length × 2.5 / 1.67 / 1.25 (all that pass play).
pub const SLED_LANDINGS: [(f32, &str); 3] = [(2.5, "SFX_heavy1"), (1.67, "SFX_med2"), (1.25, "SFX_tiressnow")];

/// What the snowmobile's step reads beyond the base state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SledInputs {
    /// The base step's inputs (surfaces +0xd4..+0xe0, throttle +0x118, rpm +0x11c, speed).
    pub base: VehicleInputs,
    /// +0xe4..+0xf0 last frame's surfaces (the bridge copies +0xd4.. there first).
    pub prev_surface: [u32; 4],
    /// +0xf4..+0x100 the wheel compressions (car +0x210..).
    pub compression: [f32; 4],
    /// The sound's +0x50 vector and the forward (+0x20) / right (+0x30) rows of its matrix.
    pub motion: V3,
    pub fwd: V3,
    pub right: V3,
    /// +0x00 / +0x10 position and velocity (the one-shots copy them).
    pub pos: V3,
    pub vel: V3,
    /// +0x6c the own volume.
    pub own: f32,
    /// `DAT_00355e5c` (the mission manager's +0x3cc): silences the road / carve voices.
    pub muted: bool,
    /// The rev voice (+0x22c) that was playing at the start of this update has ended.
    pub rev_finished: bool,
}

/// The snowmobile's sound state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SledSound {
    pub base: VehicleSound,
    pub wind: Wind,
    /// +0x248 / +0x24c the forward / side speeds.
    pub fwd_speed: f32,
    pub side_speed: f32,
    /// +0x27c the smoothed rpm, +0x250 the engine pitch, +0x274 the cruise share, +0x270 the
    /// cruise scale (× 0.75 a step while a rev plays).
    pub rpm: f32,
    pub pitch: f32,
    pub load: f32,
    pub cruz_scale: f32,
    /// +0x268 / +0x26c rear / front compression averages; +0x254 / +0x258 the front rest lengths.
    pub rear_comp: f32,
    pub front_comp: f32,
    pub rest: [f32; 2],
    /// +0x240 the rear was unloaded last step.
    pub rear_free: bool,
    /// +0x238 last frame's airborne-wheel count as of the previous step; +0x23c a jump started.
    pub airborne: i16,
    pub jump: bool,
    /// +0x22c the rev voice, +0x244 a rev is playing.
    pub rev: Option<&'static str>,
    pub rev_active: bool,
}

impl SledSound {
    /// `FUN_002ed7c0` [M]: zeroed state, the rest lengths, the wind object.
    pub fn new(skid_multiple: f32) -> SledSound {
        SledSound {
            base: VehicleSound::new(false, false, skid_multiple),
            wind: Wind::default(),
            fwd_speed: 0.0, side_speed: 0.0, rpm: 0.0, pitch: 0.0, load: 0.0, cruz_scale: 0.0,
            rear_comp: 0.0, front_comp: 0.0, rest: [f32::from_bits(0x3df4_ca92), f32::from_bits(0x3df4_ccab)],
            rear_free: false, airborne: 0, jump: false, rev: None, rev_active: false,
        }
    }
}

/// `FUN_002df9b0` [M]: how many wheels had no surface last frame (+0xe4..+0xf0).
pub fn airborne_count(prev: [u32; 4]) -> i16 { prev.iter().filter(|&&s| s == 0).count() as i16 }

/// One snowmobile step's output.
#[derive(Clone, Debug, PartialEq)]
pub struct SledFrame {
    pub wind: PanVoice,
    pub base: VehicleFrame,
    /// In [`SLED_ROAD`] order.
    pub road: [PanVoice; 4],
    /// Landing thumps (slot 1, `Landings`) and the free-wheel (slot 9, `Offroad`).
    pub shots: Vec<OneShot3d>,
    /// [`SLED_ENGINE`] order: idle, cruise.
    pub engine: [PanVoice; 2],
    /// The rev voice this step (a new one starts when `rev_started`).
    pub rev: Option<PanVoice>,
    pub rev_started: bool,
    /// The rev ended: the cruise voice is rebuilt (it restarts).
    pub cruz_restart: bool,
}

fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn on(s: u32, set: &[u32]) -> bool { set.contains(&s) }

impl SledSound {
    /// `FUN_002ee768` [M]: the road / carve voices. Volume = `OffRoad` × speed / 20 (no listener
    /// gain). Inside (distance < 2) the pans follow the side speed; outside both pairs take the
    /// listener pan, and the step writes all four silent (they only sound with the listener within
    /// 2, the 3-D model's [5] slot). Front pair: a paved (1) front wheel → `SFX_roadcarve`, else snow / ice / dirt
    /// (3, 8, 9) → `SFX_carve`. Rear pair: the same test on +0xe0 and +0xe4 (the rear-right wheel
    /// now and the front-left wheel last frame, the game's offsets [M 0x2eefd8 / 0x2eefe8]) →
    /// `SFX_roadRN` / `SFX_snowRN`. Outside the listener range or `muted`: all four silent.
    pub fn road_step(&mut self, i: &SledInputs, l: &Listener, mix: &mut Mixer, shots: &mut Vec<OneShot3d>) -> [PanVoice; 4] {
        let vol = mix.volume_of("OffRoad") * (self.base.speed / 20.0);
        let (p12, p13);
        if l.distance < 2.0 {
            let a = if self.fwd_speed <= 0.0 {
                self.side_speed * 0.25 + 0.5
            } else if 0.0 <= self.side_speed {
                1.0 - self.side_speed * 0.25
            } else {
                self.side_speed.abs() * 0.25
            };
            p12 = a;
            p13 = 1.0 - a;
        } else {
            p12 = l.pan;
            p13 = l.pan;
        }
        // landing thumps: a wheel was off at the previous step's count and none is now
        if self.airborne != 0 && airborne_count(i.prev_surface) == 0 {
            let rest = (self.rest[0] + self.rest[1]) * 0.5;
            let g = mix.volume_of("Player Car");
            for (k, sfx) in SLED_LANDINGS {
                if rest * k <= self.front_comp {
                    shots.push(OneShot3d { slot: 1, sfx, group: "Landings", volume: i.own * g, src: Source3d::at(i.pos, i.vel) });
                }
            }
        }
        let [road_rn, snow_rn, road_carve, carve] = SLED_ROAD;
        let mut v = [pv(road_rn, 0.0, l.pitch, p12), pv(snow_rn, 0.0, l.pitch, p12), pv(road_carve, 0.0, l.pitch, p13), pv(carve, 0.0, l.pitch, p13)];
        if 2.0 <= l.distance || i.muted { return v; }
        let s = i.base.wheel_surface;
        let soft = [3, 8, 9];
        if s[0] == 1 || s[1] == 1 {
            v[2] = pv(road_carve, vol, l.pitch, p13);
        } else if on(s[0], &soft) || on(s[1], &soft) {
            v[3] = pv(carve, vol, l.pitch, p13);
        }
        let (a, b) = (s[3], i.prev_surface[0]);
        if a == 1 || b == 1 {
            v[0] = pv(road_rn, vol, l.pitch, p12);
        } else if on(a, &soft) || on(b, &soft) {
            v[1] = pv(snow_rn, vol, l.pitch, p12);
        }
        v
    }

    /// `FUN_002ede60` [M]: the engine. rpm = 0.9 rpm + 0.1 (rpm + wobble); pitch = (rpm + 2000) /
    /// 7500; cruise share = min(rpm / 1000, 1). The free-wheel plays when the rear loads again with
    /// the throttle open. When the airborne count changes from 0 a jump starts; with the throttle
    /// open and no rev playing, `SFX_rev2` (both front wheels unloaded) or `SFX_rev1` (one) starts.
    /// While a rev plays the idle and cruise are silent; when it ends the cruise restarts.
    pub fn engine_step(&mut self, i: &SledInputs, l: &Listener, mix: &mut Mixer, out: &mut SledFrame) {
        let b = &i.base;
        self.rpm = self.rpm * 0.9 + (b.rpm + self.base.wobble) * 0.1;
        self.pitch = (self.rpm + 2000.0) / 7500.0;
        self.load = if self.rpm <= 1000.0 { self.rpm / 1000.0 } else { 1.0 };
        self.cruz_scale = if self.rev_active { self.cruz_scale * 0.75 } else { 1.0 };
        let c = i.compression;
        self.rear_comp = (c[2] + c[3]) * 0.5;
        self.front_comp = (c[0] + c[1]) * 0.5;
        let g = mix.volume_of("Player Car");
        if self.rear_free && 0.0 < self.rear_comp && 0.0 < b.throttle {
            out.shots.push(OneShot3d { slot: 9, sfx: SLED_FREEWHEEL, group: "Offroad", volume: i.own * g, src: Source3d::at(i.pos, i.vel) });
        }
        self.rear_free = self.rear_comp == 0.0;
        let n = airborne_count(i.prev_surface);
        let mut finished = i.rev_finished;
        if self.airborne != n {
            if self.airborne == 0 {
                if 0 < n { self.jump = true; }
            } else {
                self.jump = false;
            }
            self.airborne = n;
            if self.jump && (self.rev.is_none() || finished) && 0.0 < b.throttle {
                let key = if c[0] == 0.0 && c[1] == 0.0 {
                    Some(SLED_REVS[1])
                } else if c[0] != 0.0 && c[1] != 0.0 {
                    None
                } else {
                    Some(SLED_REVS[0])
                };
                if let Some(k) = key {
                    self.rev = Some(k);
                    finished = false; // a fresh voice has not ended
                    out.rev_started = true;
                    out.rev = Some(pv(k, i.own * g * l.gain, self.pitch * l.pitch, l.pan));
                    self.rev_active = true;
                }
            }
        }
        let [idle, cruz] = SLED_ENGINE;
        match self.rev {
            Some(k) if !finished => {
                out.rev = Some(pv(k, i.own * g * l.gain, self.pitch * l.pitch, l.pan));
                out.engine = [pv(idle, 0.0, l.pitch, l.pan), pv(cruz, 0.0, self.pitch * l.pitch, l.pan)];
            }
            _ => {
                out.engine = [
                    pv(idle, i.own * g * (1.0 - self.load) * l.gain, l.pitch, l.pan),
                    pv(cruz, i.own * g * self.load * self.cruz_scale * l.gain, self.pitch * l.pitch, l.pan),
                ];
                if self.rev.is_some() {
                    self.rev = None;
                    self.rev_active = false;
                    out.cruz_restart = true;
                }
            }
        }
    }

    /// `FUN_002edd88` [M]: one snowmobile sound update.
    pub fn update(&mut self, i: &SledInputs, ticks: u32, rng: &mut DriveRng, l: &Listener, mix: &mut Mixer) -> SledFrame {
        self.fwd_speed = dot(i.motion, i.fwd);
        self.side_speed = dot(i.motion, i.right);
        let wind = if 0.0 < self.fwd_speed {
            self.wind.speed = self.fwd_speed.abs();
            let s = self.base.speed;
            self.wind.step(ticks, s, rng, l, mix)
        } else {
            self.wind.idle(l)
        };
        let base = self.base.base_update(&i.base, ticks, l, mix);
        let mut shots = Vec::new();
        let road = self.road_step(i, l, mix, &mut shots);
        let mut out = SledFrame { wind, base, road, shots, engine: [pv("", 0.0, 0.0, 0.0); 2], rev: None, rev_started: false, cruz_restart: false };
        self.engine_step(i, l, mix, &mut out);
        out
    }
}

// ---------------------------------------------------------------------------------------------
// Turret and tank
// ---------------------------------------------------------------------------------------------

pub const TURRET_LR: [&str; 2] = ["SFX_TurretLR", "SFX_TurretLRStop"];
pub const TURRET_UD: [&str; 2] = ["SFX_TurretUD", "SFX_TurretUDStop"];
pub const TURRET_SERVO: [&str; 2] = ["SFX_servo", "SFX_servodown"];
/// The 2-D clunk when the servo starts (slot 4, group `Weapons Init`).
pub const TURRET_KACHUNG: &str = "SFX_Kachung";

/// The turret's sound state (`FUN_002e4930`): [1] servo, [2] LR, [3] UD voices; [4] / [5] / [6]
/// their on flags.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TurretSound {
    pub servo: Option<&'static str>,
    pub lr: Option<&'static str>,
    pub ud: Option<&'static str>,
    pub servo_on: bool,
    pub lr_on: bool,
    pub ud_on: bool,
}

/// One turret step's output: (slot, voice) for each live voice, and the clunk.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurretFrame { pub voices: Vec<(usize, PanVoice)>, pub kachung: bool }

impl TurretSound {
    /// `FUN_002e4980` [M]. Each axis: moving and off → its loop (slot 0) starts; stopped and on →
    /// its stop sample replaces it. The servo: on and off → `SFX_Kachung` plus the `SFX_servo` loop
    /// (slot 4); off and on → `SFX_servodown`. Voices play at `Weapons Init` × gain (the two axes ×
    /// 0.5); a voice that has ended (`done`, for [servo, lr, ud] voices not replaced this step) is
    /// dropped.
    pub fn update(&mut self, lr: bool, ud: bool, servo: bool, done: [bool; 3], l: &Listener, mix: &mut Mixer) -> TurretFrame {
        let g = mix.volume_of("Weapons Init");
        let mut out = TurretFrame::default();
        let mut fresh = [false; 3];
        let axis = |moving: bool, on: &mut bool, v: &mut Option<&'static str>, keys: [&'static str; 2], fresh: &mut bool| {
            if !moving {
                if *on { *v = Some(keys[1]); *on = false; *fresh = true; }
            } else if !*on {
                *v = Some(keys[0]);
                *on = true;
                *fresh = true;
            }
        };
        axis(lr, &mut self.lr_on, &mut self.lr, TURRET_LR, &mut fresh[1]);
        if let Some(k) = self.lr {
            out.voices.push((0, pv(k, g * 0.5 * l.gain, l.pitch, l.pan)));
            if done[1] && !fresh[1] { self.lr = None; }
        }
        axis(ud, &mut self.ud_on, &mut self.ud, TURRET_UD, &mut fresh[2]);
        if let Some(k) = self.ud {
            out.voices.push((0, pv(k, g * 0.5 * l.gain, l.pitch, l.pan)));
            if done[2] && !fresh[2] { self.ud = None; }
        }
        if !servo {
            if self.servo_on { self.servo = Some(TURRET_SERVO[1]); self.servo_on = false; fresh[0] = true; }
        } else if !self.servo_on {
            out.kachung = true;
            self.servo = Some(TURRET_SERVO[0]);
            self.servo_on = true;
            fresh[0] = true;
        }
        if let Some(k) = self.servo {
            out.voices.push((4, pv(k, g * l.gain, l.pitch, l.pan)));
            if done[0] && !fresh[0] { self.servo = None; }
        }
        out
    }
}

/// The tank's sound (`FUN_002f0f10`): the vehicle base, the engine object (max rpm 4500), the
/// turret.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TankSound { pub base: VehicleSound, pub engine: EngineSound, pub turret: TurretSound }

/// One tank step's output.
#[derive(Clone, Debug, PartialEq)]
pub struct TankFrame { pub base: VehicleFrame, pub engine: EngineFrame, pub treads: PanVoice, pub turret: TurretFrame }

impl TankSound {
    pub fn new(skid_multiple: f32) -> TankSound {
        TankSound { base: VehicleSound::new(false, false, skid_multiple), engine: EngineSound::new(PLAYER_ENGINE_RPM), turret: TurretSound::default() }
    }

    /// `FUN_002f1000` [M]: base step; the engine (`FUN_002ff320` → `FUN_002fa8f0`, the player car's
    /// code with no 500 rpm floor); the treads; the turret (`turret` = +0xb8, +0xbc, +0xb0).
    pub fn update(&mut self, inp: &VehicleInputs, ticks: u32, l: &Listener, mix: &mut Mixer, turret: (bool, bool, bool), done: [bool; 3]) -> TankFrame {
        let base = self.base.base_update(inp, ticks, l, mix);
        let rear_free = inp.wheel_surface[2] == 0 || inp.wheel_surface[3] == 0;
        self.engine.base_update(inp.rpm, self.base.wobble, self.base.skid_rear, inp.throttle, rear_free, ticks);
        let car = mix.volume_of("Player Car");
        let engine = self.engine.layer_update(ticks, l, car);
        let rear = [inp.wheel_surface[2], inp.wheel_surface[3]];
        let treads = tank_treads(rear, self.base.speed, l, mix);
        let turret = self.turret.update(turret.0, turret.1, turret.2, done, l, mix);
        TankFrame { base, engine, treads, turret }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::easnd::MixLine;

    fn mix() -> Mixer {
        let ini = [
            MixLine { name: "Player Car".into(), volume: 0.8, param: 0 },
            MixLine { name: "OffRoad".into(), volume: 0.5, param: 0 },
            MixLine { name: "Player Wind".into(), volume: 0.25, param: 0 },
            MixLine { name: "Weapons Init".into(), volume: 0.6, param: 0 },
            MixLine { name: "Tank Treads".into(), volume: 0.5, param: 0 },
        ];
        Mixer::driving(&ini, 100, 100)
    }
    fn near(a: f32, b: f32) -> bool { (a - b).abs() < 1e-5 }

    #[test]
    fn ramp_counts_up_then_stops_at_one() {
        let mut r = Ramp::default();
        assert_eq!(r.step(), 1.0);
        r.start(10);
        let v: Vec<f32> = (0..11).map(|_| r.step()).collect();
        assert!(near(v[0], 0.1) && near(v[9], 1.0) && v[10] == 1.0 && !r.running);
    }

    #[test]
    fn boat_load_ramps_up_on_gas_and_down_after() {
        let mut m = mix();
        let l = Listener { gain: 0.5, pitch: 1.0, ..Listener::default() };
        let mut b = BoatSound::default();
        let i = BoatInputs { speed: 10.0, rpm: 3000.0, wobble: 300.0, gas: true, own: 1.0, ..BoatInputs::default() };
        let v = b.update(&i, &l, &mut m, || 0.0).load.unwrap();
        // 0.8 × (3300 / 3300) × 0.1 × 0.5
        assert!(near(v.volume, 0.04), "{}", v.volume);
        assert!(near(v.pitch, 0.5));
        let ib = BoatInputs { brake: true, gas: false, ..i };
        assert!(near(b.update(&ib, &l, &mut m, || 0.0).load.unwrap().pitch, 0.5 * 0.85));
        for _ in 0..20 { b.update(&i, &l, &mut m, || 0.0); }
        let off = BoatInputs { gas: false, ..i };
        // released: × (1 − 0.1) on the first step down
        assert!(near(b.update(&off, &l, &mut m, || 0.0).load.unwrap().volume, 0.8 * 0.9 * 0.5));
        for _ in 0..9 { b.update(&off, &l, &mut m, || 0.0); }
        assert!(b.load_on);
        let last = b.update(&off, &l, &mut m, || 0.0).load.unwrap();
        assert_eq!(last.volume, 0.0);
        assert!(!b.load_on);
        // idle: the game keeps writing a silent voice every step
        let idle = b.update(&off, &l, &mut m, || 0.0).load.unwrap();
        assert_eq!((idle.volume, idle.pitch), (0.0, 1.0));
    }

    #[test]
    fn boat_jet_on_brake_and_rudder_tail_and_stops() {
        let mut m = mix();
        let l = Listener { gain: 0.5, pitch: 2.0, ..Listener::default() };
        let mut b = BoatSound::default();
        let i = BoatInputs { speed: 20.0, rpm: 1000.0, wobble: 0.0, brake: true, body_speed: 3.0, steer: 0.5, own: 1.0, ..BoatInputs::default() };
        let f = b.update(&i, &l, &mut m, || 0.0);
        let jet = f.jet.unwrap();
        assert!(near(jet.volume, 0.8 * 0.5));
        assert!(near(jet.pitch, (1000.0 * 0.0005556 + 0.2) * 2.0), "{}", jet.pitch);
        assert!(near(f.rudder.unwrap().volume, 0.4));
        // release everything: the tails fade by 0.06; the rudder tail ignores the gain
        let rel = BoatInputs { brake: false, steer: 0.0, ..i };
        let f = b.update(&rel, &l, &mut m, || 0.0);
        assert!(near(f.jet.unwrap().volume, (0.8 - 0.06) * 0.5));
        assert!(near(f.rudder.unwrap().volume, 0.8 - 0.06));
        let mut shots = Vec::new();
        for _ in 0..20 { shots.extend(b.update(&rel, &l, &mut m, || 0.0).shots); }
        assert_eq!(shots.len(), 1);
        assert_eq!((shots[0].sfx, shots[0].slot, shots[0].group), ("SFX_rudcav3", 9, "One Shots"));
        assert!(near(shots[0].volume, 0.8));
        // a vertical steer alone also drives the rudder; at rest no jet
        let vs = BoatInputs { steer_v: 1.0, body_speed: 0.0, brake: true, ..rel };
        let f = b.update(&vs, &l, &mut m, || 0.0);
        assert!(f.rudder.unwrap().volume > 0.0 && f.jet.is_none());
    }

    #[test]
    fn boat_servo_alternates_and_creaks_jitter() {
        let mut m = mix();
        let l = Listener::default();
        let mut b = BoatSound::default();
        let on = BoatInputs { servo_on: true, own: 1.0, ..BoatInputs::default() };
        let k: Vec<&str> = (0..3).map(|_| b.update(&on, &l, &mut m, || 0.0).shots[0].sfx).collect();
        assert_eq!(k, ["SFX_servon2", "SFX_servon1", "SFX_servon2"]);
        let off = BoatInputs { servo_off: true, own: 1.0, ..BoatInputs::default() };
        assert_eq!(b.update(&off, &l, &mut m, || 0.0).shots[0].sfx, "SFX_servoff2");
        // creaks: level 2, interval 30, N = 1.0 → next = 30 + round(3.0) = 33
        let mut c = BoatSound::default();
        assert_eq!(c.creak_step(|| 1.0), None);
        c.set_creak(2, 30);
        assert_eq!(c.creak_step(|| 1.0), Some("SFX_creak2"));
        assert_eq!(c.creak_next, 33);
        let n = (0..33).filter(|_| c.creak_step(|| 1.0).is_some()).count();
        assert_eq!(n, 0);
        assert_eq!(c.creak_step(|| -1.5), Some("SFX_creak2"));
        // 30 × -0.15 = -4.5 → rounds away from zero to -5
        assert_eq!(c.creak_next, 25);
    }

    #[test]
    fn gauss_pairs_and_spares() {
        let mut g = Gauss::default();
        let mut seq = [0x6000_0000i32, 0x3000_0000].into_iter().cycle();
        let a = g.sample(|| seq.next().unwrap());
        let x = 0x6000_0000 as f32 * 4.656613e-10 * 2.0 - 1.0;
        let y = 0x3000_0000 as f32 * 4.656613e-10 * 2.0 - 1.0;
        let s = x * x + y * y;
        let f = (-2.0 * s.ln() / s).sqrt();
        assert!(near(a, x * f));
        assert!(near(g.sample(|| panic!("spare first")), y * f));
    }

    #[test]
    fn wind_drifts_and_redraws_rates() {
        let mut m = mix();
        let l = Listener { gain: 1.0, pitch: 1.0, ..Listener::default() };
        let mut w = Wind { speed: 10.0, ..Wind::default() };
        let mut rng = DriveRng::boot();
        let v = w.step(1, 6.0, &mut rng, &l, &mut m);
        // level 0.4 − 0.1 = 0.3 < 0.4 → new rate r·0.02 + 0.002; pitch 0.9 < 1 → r·0.03 + 0.001
        let mut r2 = DriveRng::boot();
        let (a, b) = (rand01(&mut r2), rand01(&mut r2));
        assert!(near(w.level, 0.3) && near(w.level_rate, a * 0.02 + 0.002) && near(w.pitch_rate, b * 0.03 + 0.001));
        assert!(near(v.volume, 0.3 * 0.2 * 0.25 * 10.0), "{}", v.volume);
        assert_eq!(v.pan, 0.5);
        assert_eq!(w.step(1, 5.0, &mut rng, &l, &mut m).volume, 0.0);
    }

    fn sled_in() -> SledInputs {
        let mut i = SledInputs { own: 1.0, motion: [0.0, 0.0, 10.0], fwd: [0.0, 0.0, 1.0], right: [1.0, 0.0, 0.0], ..SledInputs::default() };
        i.base.wheel_surface = [9; 4];
        i.prev_surface = [9; 4];
        i.base.speed = 20.0;
        i.compression = [0.1; 4];
        i
    }

    #[test]
    fn sled_carve_voices_by_surface_and_the_rear_quirk() {
        let mut m = mix();
        // the road / carve voices sound only with the listener within 2 (the [5] distance slot)
        let l = Listener { gain: 0.5, distance: 1.0, pan: 0.1, ..Listener::default() };
        let mut s = SledSound::new(1.0);
        let i = sled_in();
        let f = s.update(&i, 1, &mut DriveRng::boot(), &l, &mut m);
        // speed 20 (no horn: raw); OffRoad 0.5 × 20/20, no listener gain
        assert!(near(f.road[3].volume, 0.5) && f.road[2].volume == 0.0, "{:?}", f.road);
        assert!(near(f.road[1].volume, 0.5) && f.road[0].volume == 0.0);
        // inside, moving straight forward: rear pair pan 1 − 0 = 1, front pair 0
        assert_eq!((f.road[1].pan, f.road[3].pan), (1.0, 0.0));
        let far = Listener { distance: 5.0, ..l };
        let f = s.update(&i, 1, &mut DriveRng::boot(), &far, &mut m);
        assert!(f.road.iter().all(|v| v.volume == 0.0 && v.pan == 0.1));
        // the rear pair reads the rear-right wheel and last frame's front-left one
        let mut j = sled_in();
        j.base.wheel_surface = [1, 1, 1, 0];
        j.prev_surface = [0, 1, 1, 1];
        let f = s.update(&j, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(f.road[2].volume > 0.0 && f.road[0].volume == 0.0 && f.road[1].volume == 0.0);
        j.prev_surface = [1, 1, 1, 1];
        let f = s.update(&j, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(f.road[0].volume > 0.0);
        // muted: silent
        let f = s.update(&SledInputs { muted: true, ..sled_in() }, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(f.road.iter().all(|v| v.volume == 0.0));
        let mut k = sled_in();
        // moving forward: side >= 0 → 1 − side/4, side < 0 → |side|/4
        k.motion = [-0.4, 0.0, 10.0];
        let f = s.update(&k, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(near(f.road[1].pan, 0.1) && near(f.road[3].pan, 0.9));
        k.motion = [0.4, 0.0, 10.0];
        let f = s.update(&k, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(near(f.road[1].pan, 0.9) && near(f.road[3].pan, 0.1));
        // not moving forward: side/4 + 0.5
        k.motion = [0.4, 0.0, -10.0];
        let f = s.update(&k, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(near(f.road[1].pan, 0.6) && near(f.road[3].pan, 0.4));
    }

    #[test]
    fn sled_engine_rev_on_takeoff_and_landing_thumps() {
        let mut m = mix();
        let l = Listener { gain: 1.0, pitch: 1.0, distance: 5.0, ..Listener::default() };
        let mut s = SledSound::new(1.0);
        let mut i = sled_in();
        i.base.rpm = 1000.0;
        i.base.throttle = 1.0;
        let f = s.update(&i, 1, &mut DriveRng::boot(), &l, &mut m);
        // the base step moved the wobble to 5 before the engine reads it
        assert!(near(s.rpm, 100.5) && near(s.pitch, 2100.5 / 7500.0) && near(s.load, 0.1005));
        assert!(near(f.engine[0].volume, 0.8 * (1.0 - 0.1005)) && near(f.engine[1].volume, 0.8 * 0.1005));
        assert!(f.rev.is_none());
        // take-off: last frame two wheels off, both front compressions 0 → SFX_rev2
        i.prev_surface = [0, 0, 9, 9];
        i.compression = [0.0, 0.0, 0.1, 0.1];
        let f = s.update(&i, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(f.rev_started && f.rev.unwrap().sfx == "SFX_rev2" && s.rev_active);
        assert!(f.engine.iter().all(|v| v.volume == 0.0));
        // while it plays the cruise scale shrinks
        let f = s.update(&i, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(!f.rev_started && near(s.cruz_scale, 0.75));
        // it ends: idle / cruise back, cruise rebuilt
        let f = s.update(&SledInputs { rev_finished: true, ..i }, 1, &mut DriveRng::boot(), &l, &mut m);
        assert!(f.cruz_restart && s.rev.is_none() && f.engine[0].volume > 0.0);
        // landing: the count from the previous step is 2, all wheels down now, front compression
        // ≥ 2.5 × rest → all three thumps
        let mut land = sled_in();
        land.compression = [0.35; 4];
        s.update(&land, 1, &mut DriveRng::boot(), &l, &mut m); // stores front_comp 0.35, airborne 0
        s.airborne = 2;
        let f = s.update(&land, 1, &mut DriveRng::boot(), &l, &mut m);
        let k: Vec<&str> = f.shots.iter().map(|o| o.sfx).collect();
        assert_eq!(k, ["SFX_heavy1", "SFX_med2", "SFX_tiressnow"]);
        // a single-wheel rev and the free-wheel when the rear loads again under throttle
        let mut t = SledSound::new(1.0);
        let mut a = sled_in();
        a.base.throttle = 1.0;
        a.compression = [0.0, 0.1, 0.0, 0.0];
        t.update(&a, 1, &mut DriveRng::boot(), &l, &mut m);
        a.prev_surface = [0, 9, 9, 9];
        a.compression = [0.0, 0.1, 0.2, 0.2];
        let f = t.update(&a, 1, &mut DriveRng::boot(), &l, &mut m);
        assert_eq!(f.rev.unwrap().sfx, "SFX_rev1");
        assert_eq!(f.shots.iter().map(|o| (o.sfx, o.slot, o.group)).collect::<Vec<_>>(), [("SFX_freewheel", 9, "Offroad")]);
    }

    #[test]
    fn turret_loops_stops_and_clunk() {
        let mut m = mix();
        let l = Listener { gain: 0.5, ..Listener::default() };
        let mut t = TurretSound::default();
        let f = t.update(true, false, true, [false; 3], &l, &mut m);
        assert!(f.kachung);
        assert_eq!(f.voices.iter().map(|v| (v.0, v.1.sfx)).collect::<Vec<_>>(), [(0, "SFX_TurretLR"), (4, "SFX_servo")]);
        assert!(near(f.voices[0].1.volume, 0.6 * 0.5 * 0.5) && near(f.voices[1].1.volume, 0.3));
        // a finished loop is dropped and not restarted while still moving
        let f = t.update(true, false, true, [true, true, false], &l, &mut m);
        assert!(!f.kachung && f.voices.len() == 2);
        assert_eq!((t.lr, t.servo), (None, None));
        assert!(t.update(true, false, true, [false; 3], &l, &mut m).voices.is_empty());
        let f = t.update(false, false, false, [false; 3], &l, &mut m);
        assert_eq!(f.voices.iter().map(|v| v.1.sfx).collect::<Vec<_>>(), ["SFX_TurretLRStop", "SFX_servodown"]);
    }

    #[test]
    fn tank_engine_has_no_rpm_floor_and_treads_follow() {
        let mut m = mix();
        let l = Listener { gain: 1.0, distance: 5.0, ..Listener::default() };
        let mut t = TankSound::new(1.0);
        let inp = VehicleInputs { wheel_surface: [1; 4], throttle: 0.0, rpm: 0.0, speed: 4.0, ..VehicleInputs::default() };
        let f = t.update(&inp, 1, &l, &mut m, (false, false, false), [false; 3]);
        // target = 0 + wobble 5 → rpm 0.5 (the player car would floor at 500)
        assert!(near(t.engine.rpm, 0.5), "{}", t.engine.rpm);
        assert!(near(f.treads.volume, 0.5 * 0.5));
        assert_eq!(f.engine.voices[5].sfx, "SFX_OUT-idle");
    }
}
