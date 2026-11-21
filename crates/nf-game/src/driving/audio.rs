//! The driving missions' sound logic, headless: which banks and speech archive a track loads, the
//! mix-group tree and its volumes, and the per-frame vehicle sound model (engine layers, road noise,
//! skids, scrapes, the rpm wobble) as voice volume/pitch outputs. The data side (banks, streams,
//! EA-XA) is `nf_formats::easnd`; docs/systems/drive-audio.md is the write-up.
//!
//! Evidence (all [M] unless tagged; DRIVING.ELF decompile/disassembly, EE addresses):
//! - Startup "Init Sound" `FUN_00176658`: the speech archive base per track, the language suffix
//!   table at 0x376bb0 (`en fr ge sp`), `driving\%s.spe` with the English fallback, and the music /
//!   effects volumes = PTP+0x968 / PTP+0x964 over 100.
//! - Audio system init `FUN_002eb7a8`: the mix tree (the parent pairs in [`MIX_PARENTS`]), forced
//!   volumes 1.0 on the three faders and `Speech`, `1:Music`/`1:Effects` = the menu volumes; the bank
//!   slots: `g_common.bnk` → slot 8, then `banks.ini` keys by slot index 0..9 ([`BANK_SLOTS`]).
//!   The slot index is the `PTR_DAT_003562a0[slot]` table every sound lookup goes through
//!   (`FUN_002f4b10`; the engine uses `PTR_DAT_003562c4` = slot 9, `Engine`).
//! - Mix `.ini` `FUN_002f3058`; a group's ctor `FUN_002f2d08` (volume 0); effective volume
//!   `FUN_002f34d0` (own × parent chain, with the fade step).
//! - Voice gate `FUN_002ddfd0`: volume ≤ 1/127 → 0, else min(v, 1), then × the sound's own +0x2c.
//! - Engine: ctor `FUN_002ff040` (ten layer voices from the Engine bank, group `Player Car`),
//!   `FUN_002faae8` (+0x14 = 1 / 4500 for the player car, `FUN_002f07c8`), base step
//!   `FUN_002fa8f0` (rpm/load slews, pitch), layer mix `FUN_002ff320` (interior/exterior blend,
//!   low/high crossfade, the vacuum one-shot and its pitch wobble — `FUN_002cbd50` is a sine).
//! - Vehicle base step `FUN_002e1768`: the ±150 rpm wobble (+0x120), the three scrape voices,
//!   the front/rear skid slews and the per-surface skid voices; speed smoothing `FUN_002df7c0`;
//!   the voice names from the ctor `FUN_002defd0`.
//! - Player car step `FUN_002f0900`: the 500 rpm floor, the road-noise voice and its surface switch.
//! - Sim → audio bridge `FUN_00188200` (lines 691-760 of the decompile): wheel surfaces at
//!   +0xd4..+0xe0 (0 when the wheel has no contact), throttle +0x118, rpm +0x11c (= `FUN_0018aa88`,
//!   ×1.25 for the player — the physics port's `Car::rpm`), skid targets +0x104/+0x108 (≤ 1).
//! - "Ticks" everywhere is `DAT_00355e60`, the sim-tick count since the last audio update
//!   (`FUN_00159d10` +1 per tick, `FUN_00147e30` resets it; read with `cvt.s.w`).
//!
//! Not ported / not traced (stated plainly): which mission-script event starts which music or
//! speech stream (the `CHAR::` names in the track `.crp` — e.g. `CHAR::snipeintro` — are read by
//! event records this port does not decode). The scrape counters' writer (`FUN_002e0890`), the
//! horn timing, the 3-D model, impacts, landings, the traffic engine and the car-weapon fire
//! one-shots are in `sound_events.rs`. No sound is played by the viewer from this module yet.

use nf_formats::easnd::MixLine;

// ---------------------------------------------------------------------------------------------
// Startup selection
// ---------------------------------------------------------------------------------------------

/// The language suffixes at 0x376bb0, indexed by the language id (`FUN_00249f98`).
pub const LANGS: [&str; 4] = ["en", "fr", "ge", "sp"];

/// `FUN_00176658` [M]: the speech/music archive base for a track name (0x1004f4). The chain of
/// case-insensitive compares; anything else gets `mis11`.
pub fn speech_base(track: &str) -> &'static str {
    let is = |s: &str| track.eq_ignore_ascii_case(s);
    if is("uw_mis11") || is("uw_map") { "mis11" }
    else if is("snow1a_mis3") || is("snow1a") { "mis3" }
    else if is("snow2a_mis4") || is("snow2a") || is("snow2a_race") { "mis4" }
    else if is("paris_mis01") || is("paris_map") { "mis01" }
    else if is("junglea_mis13a") || is("junglea") { "mis13a" }
    else if is("jungleb_mis13b") || is("jungleb") { "mis13b" }
    else if is("junglec_mis13c") || is("junglec") { "mis13c" }
    else { "mis11" }
}

/// `FUN_00176658` [M]: `driving\<base><lang>.spe` when it exists, else the English one
/// ("WARNING: localized speech not found, using English").
pub fn speech_archive(track: &str, lang: usize, exists: impl Fn(&str) -> bool) -> String {
    let base = speech_base(track);
    let p = format!("driving\\{base}{}.spe", LANGS.get(lang).copied().unwrap_or("en"));
    if exists(&p) { p } else { format!("driving\\{base}en.spe") }
}

/// `FUN_00176658` [M]: the menu volume ints (0..100) → the driving engine's 0..1 floats.
pub fn menu_volume(percent: i32) -> f32 { percent as f32 / 100.0 }

/// `FUN_002eb7a8`'s bank-slot switch [M]: slot index → `banks.ini` key; slot 8 is `g_common.bnk`.
pub const BANK_SLOTS: [(usize, &str); 9] = [
    (0, "Generic"), (1, "Collisions"), (2, "Smackables"), (3, "CarWeapons"), (4, "Weapons"),
    (5, "Ambience"), (6, "Misc"), (7, "Actors"), (9, "Engine"),
];
pub const SLOT_G_COMMON: usize = 8;

/// The banks a track loads: `(slot, path under data/audio/)` from its `banks.ini` section
/// (the archive's own track section lists exactly the banks it carries — disc test [D]).
pub fn track_banks(sections: &[(String, Vec<(String, String)>)], track: &str) -> Vec<(usize, String)> {
    let sec = sections.iter().find(|s| s.0.eq_ignore_ascii_case(track)).or_else(|| sections.iter().find(|s| s.0 == "default"));
    let mut out = vec![(SLOT_G_COMMON, "g_common.bnk".to_string())];
    if let Some((_, keys)) = sec {
        for &(slot, key) in &BANK_SLOTS {
            // FUN_002d9b88 per key: the first matching line [S — the ini reader is not decomposed]
            if let Some((_, v)) = keys.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)) { out.push((slot, v.clone())); }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Mix groups
// ---------------------------------------------------------------------------------------------

/// `FUN_002eb7a8` [M]: (group, parent) in the order the code links them.
pub const MIX_PARENTS: [(&str, &str); 48] = [
    ("0:Fader2", "0:Fader1"), ("0:Fader1", "0:Fader0"), ("0:Fader0", "1:Effects"), ("Speech", "1:Effects"),
    ("Ambience", "0:Fader2"), ("Asphalt", "0:Fader2"), ("Blowout", "0:Fader2"), ("Booster", "0:Fader2"),
    ("Bullet Impact", "0:Fader2"), ("Bullet Ricochet", "0:Fader2"), ("Characters", "0:Fader2"), ("Collisions", "0:Fader2"),
    ("Explosions", "0:Fader2"), ("ExplosionBass", "0:Fader2"), ("Friendly", "0:Fader2"), ("Gadgets", "0:Fader2"),
    ("Gatling Gun", "0:Fader2"), ("Giotto", "0:Fader2"), ("Helicopter", "0:Fader2"), ("Henchmen Fire", "0:Fader2"),
    ("Horns", "0:Fader2"), ("Landings", "0:Fader1"), ("Machine Gun", "0:Fader2"), ("Menu Sounds", "1:Effects"),
    ("Music", "1:Music"), ("NIS", "0:Fader2"), ("Object Sounds", "0:Fader2"), ("OffRoad", "0:Fader2"),
    ("Opponents", "0:Fader2"), ("Player Car", "0:Fader2"), ("Player Grunts", "0:Fader1"), ("Player Wind", "0:Fader1"),
    ("POV Weapons", "0:Fader2"), ("Power Ups", "0:Fader2"), ("Rail", "0:Fader2"), ("Scrapes", "0:Fader2"),
    ("Shell Casings", "0:Fader2"), ("Skids", "0:Fader1"), ("Smackables", "0:Fader2"), ("Tank Treads", "0:Fader2"),
    ("Tracking System", "0:Fader2"), ("Traffic Cars", "0:Fader2"), ("Weapons Fire", "0:Fader2"), ("Weapons Init", "0:Fader2"),
    ("Weapons Reload", "0:Fader2"), ("Weapons Transit", "0:Fader2"), ("Wood", "0:Fader2"), ("World Sounds", "0:Fader2"),
];

/// One `AMix` group (the 0x50-byte object; the fields the port uses).
#[derive(Clone, Debug, PartialEq)]
pub struct MixGroup {
    pub name: String,
    pub parent: Option<usize>,
    /// +4 the volume, +8 the previous volume (every setter in FUN_002eb7a8 moves +4 to +8)
    pub volume: f32,
    pub prev: f32,
    /// +0x14 fade target, +0x18 fade step per call, +0x1c fading
    pub target: f32,
    pub step: f32,
    pub fading: bool,
    /// +0x20 the mix `.ini`'s trailing int (meaning not traced)
    pub param: i32,
    /// +0x10 the ini volume (`FUN_002f3058` writes it with +4/+8/+0xc); the ctor `FUN_002f2d08`
    /// leaves -1.0. `ESetAudioMix` scales it and `FUN_002f3450` (`EResetAudioMix`) fades back to it.
    pub base: f32,
}

/// The mix-group registry (`FUN_002f46e8`'s map) — get-or-create by name like `FUN_002f2d50`.
#[derive(Clone, Debug, Default)]
pub struct Mixer { pub groups: Vec<MixGroup> }

impl Mixer {
    /// The group map's lookup (`FUN_002f3888` → `FUN_002f37e0`) compares with the
    /// case-insensitive `FUN_002a9198` [M], so the script's `music` is the `Music` group.
    pub fn find(&self, name: &str) -> Option<usize> { self.groups.iter().position(|g| g.name.eq_ignore_ascii_case(name)) }

    /// `FUN_002f2d50` [M]: the existing group, else a new one with the `FUN_002f2d08` ctor fields
    /// (volume 0, target 0, step 0, not fading, no parent).
    pub fn get_or_create(&mut self, name: &str) -> usize {
        if let Some(i) = self.find(name) { return i; }
        self.groups.push(MixGroup { name: name.into(), parent: None, volume: 0.0, prev: 0.0, target: 0.0, step: 0.0, fading: false, param: 0, base: -1.0 });
        self.groups.len() - 1
    }

    fn set(&mut self, name: &str, v: f32) {
        let i = self.get_or_create(name);
        let g = &mut self.groups[i];
        g.prev = g.volume;
        g.volume = v;
    }

    /// `FUN_002f3058` + `FUN_002eb7a8` [M]: the driving mix as the audio init builds it — the ini's
    /// groups (volume into +4/+8/+0xc/+0x10, the int into +0x20), the faders and `Speech` forced to 1.0,
    /// the parent links, then `1:Music` / `1:Effects` = the menu volumes (0..100).
    pub fn driving(ini: &[MixLine], music_percent: i32, effects_percent: i32) -> Mixer {
        let mut m = Mixer::default();
        for l in ini {
            let i = m.get_or_create(&l.name);
            let g = &mut m.groups[i];
            // +4/+8/+0xc/+0x10 = the volume (+0xc/+0x10 are not modelled), +0x20 = the int
            g.volume = l.volume;
            g.prev = l.volume;
            g.param = l.param;
            g.base = l.volume;
        }
        for n in ["1:Effects", "1:Music", "0:Fader2", "0:Fader1", "0:Fader0"] { m.get_or_create(n); }
        for n in ["0:Fader2", "0:Fader1", "0:Fader0", "Speech"] { m.set(n, 1.0); }
        for (child, parent) in MIX_PARENTS {
            let c = m.get_or_create(child);
            let p = m.get_or_create(parent);
            m.groups[c].parent = Some(p);
        }
        m.set("1:Music", menu_volume(music_percent));
        m.set("1:Effects", menu_volume(effects_percent));
        m
    }

    /// Starts a fade of a group toward `target` by `step` per volume read (the fields
    /// `FUN_002f34d0` consumes; the game's own fade setter is not traced — port API).
    pub fn fade(&mut self, idx: usize, target: f32, step: f32) {
        let g = &mut self.groups[idx];
        g.target = target;
        g.step = step;
        g.fading = true;
    }

    /// `FUN_002f34d0` [M]: the group's effective volume — its own (advancing a running fade by one
    /// step, clamped at the target in the step's direction; a zero step never finishes) times the
    /// parent chain's.
    pub fn volume(&mut self, idx: usize) -> f32 {
        let g = &mut self.groups[idx];
        if g.fading {
            let v = g.volume + g.step;
            g.volume = v;
            let done = if g.step < 0.0 { v <= g.target } else if g.step > 0.0 { v >= g.target } else { false };
            if done { g.volume = g.target; g.fading = false; }
        }
        let (own, parent) = (self.groups[idx].volume, self.groups[idx].parent);
        match parent { Some(p) => own * self.volume(p), None => own }
    }

    pub fn volume_of(&mut self, name: &str) -> f32 { self.find(name).map_or(0.0, |i| self.volume(i)) }

    /// `FUN_002f33a0(target, group, steps)` [M]: the target clamped to 0..1, +8 = +4, a fade of
    /// (target - volume) / max(steps, 1) per volume read.
    pub fn fade_to(&mut self, idx: usize, target: f32, steps: i32) {
        let t = if target < 0.0 { 0.0 } else if target > 1.0 { 1.0 } else { target };
        let g = &mut self.groups[idx];
        g.fading = true;
        g.prev = g.volume;
        g.target = t;
        g.step = (t - g.volume) / steps.max(1) as f32;
    }

    /// `FUN_002f3410(group, steps)` [M]: fade back to +8 (the volume before the last setter),
    /// leaving +8 as it is.
    pub fn revert(&mut self, idx: usize, steps: i32) {
        let g = &mut self.groups[idx];
        g.fading = true;
        g.target = g.prev;
        g.step = (g.prev - g.volume) / steps.max(1) as f32;
    }

    /// `FUN_002f3450(steps)` [M]: every registered group fades to its +0x10 ini volume (a group
    /// the ini lacks has -1 there, which `fade_to` clamps to 0).
    pub fn reset_all(&mut self, steps: i32) {
        for i in 0..self.groups.len() {
            let b = self.groups[i].base;
            self.fade_to(i, b, steps);
        }
    }

    /// A plain store into +4 that keeps the old one in +8 (the pattern `FUN_002f9180` and
    /// `FUN_002f8e00` use on `0:Fader2` / `0:Fader0`).
    pub fn store(&mut self, name: &str, v: f32) { self.set(name, v) }
}

// ---------------------------------------------------------------------------------------------
// Voices
// ---------------------------------------------------------------------------------------------

/// `FUN_002ddfd0` [M]: the per-voice gate — at most 1/127 is silence, else clamped to 1. (The
/// sound's own +0x2c factor is applied after this by the voice; the port leaves it to the player.)
pub fn voice_gate(v: f32) -> f32 { if v > 0.007874016 { v.min(1.0) } else { 0.0 } }

/// One looping voice's state for this audio update.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceOut {
    /// The bank key (`easnd::sfx_key` form) the voice was built with.
    pub sfx: &'static str,
    /// After [`voice_gate`].
    pub volume: f32,
    /// Pitch multiplier (1.0 = the sample's own rate).
    pub pitch: f32,
}

/// The per-update arguments every sound's update (vtable +0x24) gets: the output of the 3-D model
/// `FUN_002f1f40` (`sound_events::listen`) [M]. +0 gain (distance attenuation), +4 pitch (doppler),
/// +8 pan (turns, + = right), +0xc cone (direction . the sound's facing), +0x10 send ((d-25)/400),
/// +0x14 the listener distance (the engine's interior / exterior blend: <= 1 interior, >= 2
/// exterior), +0x18 the listener (its +0x5c index picks the voice channel). Default = a sound at
/// distance 2 straight ahead with no attenuation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Listener { pub gain: f32, pub pitch: f32, pub pan: f32, pub cone: f32, pub send: f32, pub distance: f32, pub index: u32 }

impl Default for Listener { fn default() -> Self { Listener { gain: 1.0, pitch: 1.0, pan: 0.0, cone: 0.0, send: 0.0, distance: 2.0, index: 2 } } }

/// The sim → audio inputs one vehicle carries (`FUN_00188200`'s writes into the audio object).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VehicleInputs {
    /// +0xd4/+0xd8/+0xdc/+0xe0: the four wheels' surface (`SURFACES` index), 0 = no contact.
    /// Order: front pair then rear pair (the skid code pairs d4/d8 with the front voice).
    pub wheel_surface: [u32; 4],
    /// +0x118 throttle (0..1).
    pub throttle: f32,
    /// +0x11c rpm (the physics' `Car::rpm`; the bridge multiplies the player's by 1.25 [M 0x188200:589]).
    pub rpm: f32,
    /// +0x104 / +0x108 front / rear skid targets (0..1).
    pub skid_front: f32,
    pub skid_rear: f32,
    /// |velocity| of the car (the vector at +0x10 the speed smoothing reads).
    pub speed: f32,
}

// ---------------------------------------------------------------------------------------------
// Engine (FUN_002ff040 / FUN_002fa8f0 / FUN_002ff320)
// ---------------------------------------------------------------------------------------------

/// The ten engine layer voices in `FUN_002ff040`'s slots: interior then exterior, each
/// idle / lowload / midload / hiload / cruz. (`vanquish.h` has no `midload` keys [D] — those two
/// voices resolve to no sound.)
pub const ENGINE_VOICES: [&str; 10] = [
    "SFX_IN-idle", "SFX_IN-lowload", "SFX_IN-midload", "SFX_IN-hiload", "SFX_IN-cruz",
    "SFX_OUT-idle", "SFX_OUT-lowload", "SFX_OUT-midload", "SFX_OUT-hiload", "SFX_OUT-cruz",
];
/// The one-shot `FUN_002ff320` starts when the load first passes 0.2.
pub const ENGINE_VACUUM: &str = "SFX_vaccuum";
/// The player car's engine normaliser (`FUN_002f07c8` → `FUN_002faae8(4500.0, …)`: +0x14 = 1/4500).
pub const PLAYER_ENGINE_RPM: f32 = 4500.0;

/// `FUN_002cbd50` [M]: sin(2π·x) — the argument folded to [-0.5, 0.5], then the odd series with the
/// game's coefficients.
pub fn game_sin_turns(mut x: f32) -> f32 {
    if x > 1.0 { x -= (x as i32) as f32; } else if x < 0.0 { x -= ((x as i32) - 1) as f32; }
    if x < -0.5 { x += 1.0; } else if x > 0.5 { x -= 1.0; }
    let a = x * 6.283_185_5;
    let a2 = a * a;
    let c3 = f32::from_bits(0xbe2a_aaa4);
    let c5 = f32::from_bits(0x3c08_873e);
    let c7 = f32::from_bits(0xb94f_b21f);
    let c9 = f32::from_bits(0x362e_9c14);
    let a3 = a * a2;
    let a5 = a3 * a2;
    let a7 = a5 * a2;
    let a9 = a7 * a2;
    a + a3 * c3 + a5 * c5 + a7 * c7 + a9 * c9
}

/// The engine sound object (the 0x54-byte `FUN_002ff040` object; field = float index).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineSound {
    /// [0] pitch
    pub pitch: f32,
    /// [1] sqrt(load), [2] sqrt(1 - load)
    pub w_load: f32,
    pub w_idle: f32,
    /// [3] time since the load passed 0.2 (ticks; 0 = the vacuum one-shot is due)
    pub vacuum_t: f32,
    /// [4] the smoothed rpm
    pub rpm: f32,
    /// [5] 1 / max rpm
    pub inv_max: f32,
    /// [6] the smoothed load (throttle)
    pub load: f32,
    /// [0x13] / [0x14] the low / high crossfade weights
    pub w_low: f32,
    pub w_high: f32,
}

/// One engine update's result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineFrame {
    pub voices: [VoiceOut; 10],
    /// The vacuum one-shot starts this update (group `Player Car`, distances 3 / 350 [M]).
    pub vacuum: bool,
}

impl EngineSound {
    /// `FUN_002ff040` + `FUN_002faae8` [M]: everything 0, +0x14 = 1 / `max_rpm`.
    pub fn new(max_rpm: f32) -> EngineSound {
        EngineSound { pitch: 0.0, w_load: 0.0, w_idle: 0.0, vacuum_t: 0.0, rpm: 0.0, inv_max: 1.0 / max_rpm, load: 0.0, w_low: 0.0, w_high: 0.0 }
    }

    /// `FUN_002fa8f0` [M]: slew the rpm toward the vehicle's (rpm + wobble − rear skid·1000; 10000 /
    /// 0 with throttle / without when a rear wheel has no contact), slew the load toward the
    /// throttle, then pitch = (rpm + 2000) / max and the two load weights.
    pub fn base_update(&mut self, rpm_target_in: f32, wobble: f32, rear_skid_level: f32, throttle: f32, rear_free: bool, ticks: u32) {
        let t = ticks as f32;
        let rate = t * 150.0;
        let mut step = t * 0.05;
        let mut thr = throttle;
        let target = if rear_free {
            if 0.0 < thr { 10000.0 } else { 0.0 }
        } else {
            (rpm_target_in + wobble) - rear_skid_level * 1000.0
        };
        let mut rpm = self.rpm;
        if target - rpm <= -rate {
            step *= 3.0;
            thr *= 0.125;
            rpm -= rate * 0.5;
        } else {
            rpm += (target - rpm) * 0.1;
        }
        self.rpm = rpm;
        let d = thr - self.load;
        if d <= -step { self.load -= step; } else if step <= d { self.load += step; } else { self.load = thr; }
        self.pitch = (self.rpm + 2000.0) * self.inv_max;
        self.w_load = self.load.sqrt();
        self.w_idle = (1.0 - self.load).sqrt();
    }

    /// `FUN_002ff320` [M] (after the base step): the vacuum timer and pitch wobble, the low/high
    /// crossfade at 2000 rpm, the interior/exterior split by the listener distance, and the ten
    /// layer voices. `car_volume` = the `Player Car` group's effective volume × the vehicle's +0x6c.
    pub fn layer_update(&mut self, ticks: u32, l: &Listener, car_volume: f32) -> EngineFrame {
        let t = ticks as f32;
        let mut vacuum = false;
        if self.load < 0.2 {
            self.vacuum_t = 0.0;
        } else if self.vacuum_t < 60.0 {
            if self.vacuum_t == 0.0 { vacuum = true; }
            self.vacuum_t += t;
            if 0.5 < self.load {
                let mut x = self.vacuum_t / 10.0;
                while 1.0 < x { x -= 1.0; }
                self.pitch += game_sin_turns(x) * 0.3 * (2.0 / (self.vacuum_t + 2.0));
            }
        }
        if self.rpm < 2000.0 {
            let f = (self.rpm * 0.000_333_333_33).max(0.0);
            self.w_low = (1.0 - f).sqrt();
            self.w_high = f.sqrt();
        } else {
            self.w_low = 0.0;
            self.w_high = 1.0;
        }
        let (v_in, v_out) = if 2.0 < l.distance {
            (0.0, car_volume)
        } else if 1.0 < l.distance {
            let f = l.distance - 1.0;
            ((1.0 - f).sqrt() * car_volume, f.sqrt() * car_volume)
        } else {
            (car_volume, 0.0)
        };
        let mut voices = [VoiceOut { sfx: "", volume: 0.0, pitch: 0.0 }; 10];
        for (k, v) in [v_in, v_out].into_iter().enumerate() {
            let p = self.pitch * l.pitch;
            let layers = [
                (v * self.w_low * self.w_idle * l.gain, self.pitch * 1.1 * l.pitch),
                (v * self.w_load * l.gain, p),
                (v * self.w_high * self.w_load * l.gain, p),
                (v * self.w_high * self.w_load * l.gain, p),
                (v * self.w_high * self.w_idle * l.gain, p),
            ];
            for (j, (vol, pitch)) in layers.into_iter().enumerate() {
                voices[k * 5 + j] = VoiceOut { sfx: ENGINE_VOICES[k * 5 + j], volume: voice_gate(vol), pitch };
            }
        }
        EngineFrame { voices, vacuum }
    }
}

// ---------------------------------------------------------------------------------------------
// The vehicle base (FUN_002e1768 / FUN_002df7c0) and the player car (FUN_002f0900)
// ---------------------------------------------------------------------------------------------

/// The bank slots `FUN_002defd0` builds the vehicle voices from [M]: the road noise, the paved /
/// grass / off-road / ice skids from slot 0 (`Generic`, `PTR_DAT_003562a0`); the scrapes (and the
/// `Squeal*` skids of non-0xE vehicles) from slot 1 (`Collisions`, `PTR_DAT_003562a4` at +0x1b0).
pub const VEHICLE_SLOT: usize = 0;
pub const SCRAPE_SLOT: usize = 1;
/// The engine layers' slot (`PTR_DAT_003562c4`, `FUN_002ff040`).
pub const ENGINE_SLOT: usize = 9;

/// The three scrape voices in the order `FUN_002e1768` updates them (level +0x144 → voice +0x1e4,
/// +0x148 → +0x1e0, +0x14c → +0x1e8; names from `FUN_002defd0`).
pub const SCRAPE_VOICES: [&str; 3] = ["SFX_ScrapeRock", "SFX_ScrapeMetal", "SFX_scrapesand"];

/// The skid voice pairs (front, rear) and the surfaces that sound them [M FUN_002e1768]. The paved
/// pair is `SkidFront1`/`SkidBack1` for the class-0xE vehicles (player and traffic, `FUN_002defd0`)
/// and `SquealFront1`/`SquealBack1` otherwise. The rear grass and off-road voices play at 0.75 pitch.
pub const SKID_SETS: [(&str, &str, &[u32], f32); 4] = [
    ("SFX_SkidFront1", "SFX_SkidBack1", &[1, 4], 1.0),
    ("SFX_GrassSkid", "SFX_GrassSkid", &[3, 10], 0.75),
    ("SFX_OffRoadSkid", "SFX_OffRoadSkid", &[5, 2, 12], 0.75),
    ("SFX_iceskidfront", "SFX_iceskidback", &[8, 9], 1.0),
];

/// `FUN_002f0900` [M]: the player car's road-noise voice for a wheel surface (`None` = none).
pub fn road_noise(surface: u32) -> Option<&'static str> {
    match surface {
        1 => Some("SFX_PavedRN"),
        2 => Some("SFX_GravelRN"),
        4 => Some("SFX_CobbleRN"),
        8 => Some("SFX_IceRN"),
        9 => Some("SFX_SnowRN"),
        10 | 15 => Some("SFX_OffRN"),
        _ => None,
    }
}

/// The slew every level in `FUN_002e1768` uses: step toward `target` by at most `step`.
fn slew(cur: f32, target: f32, step: f32) -> f32 {
    let d = target - cur;
    if d <= -step { cur - step } else if step <= d { cur + step } else { target }
}

/// The vehicle audio object's state the base step keeps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleSound {
    /// +0x120 the rpm wobble, +0x124 its direction (ctor `FUN_002def58`: 0 and +1).
    pub wobble: f32,
    pub wobble_dir: f32,
    /// +0x13c the smoothed speed
    pub speed: f32,
    /// +0x144/+0x148/+0x14c the scrape levels
    pub scrape_level: [f32; 3],
    /// +0x17c/+0x180/+0x184 the scrape counters (ticks; written by the contact code, which is not
    /// traced — callers set them)
    pub scrape: [i32; 3],
    /// +0x10c / +0x110 the front / rear skid levels
    pub skid_front: f32,
    pub skid_rear: f32,
    /// +0x114 the skid multiple (`SFX_SKIDMULTIPLE` from the car's attributes when ≥ 1; ctor 1.0)
    pub skid_multiple: f32,
    /// whether the paved skid pair is the `Skid*` (class 0xE) or `Squeal*` set
    pub class_e: bool,
    /// +0x1f8 a horn voice exists (it changes the speed smoothing)
    pub has_horn: bool,
    /// the player's road-noise voice (the last wheel whose surface changed picks it; the ctor
    /// `FUN_002f07c8` starts with `SFX_PavedRN`)
    pub road_noise: Option<&'static str>,
    pub last_surface: [u32; 4],
}

/// The base step's voice outputs.
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleFrame {
    /// scrape voices in [`SCRAPE_VOICES`] order
    pub scrapes: [VoiceOut; 3],
    /// (front, rear) per [`SKID_SETS`] row
    pub skids: [(VoiceOut, VoiceOut); 4],
    /// the player's road noise (volume already × the `Asphalt` group)
    pub road: Option<VoiceOut>,
    pub engine: Option<EngineFrame>,
}

impl VehicleSound {
    pub fn new(class_e: bool, has_horn: bool, skid_multiple: f32) -> VehicleSound {
        VehicleSound {
            wobble: 0.0, wobble_dir: 1.0, speed: 0.0, scrape_level: [0.0; 3], scrape: [0; 3], skid_front: 0.0, skid_rear: 0.0,
            // FUN_00186878: the attribute replaces the ctor's 1.0 only when it is >= 1.0
            skid_multiple: if 1.0 <= skid_multiple { skid_multiple } else { 1.0 },
            class_e, has_horn, road_noise: None, last_surface: [0; 4],
        }
    }

    /// `FUN_002df7c0` [M]: the speed the sound reads. With a horn voice a speed in (5, 14) reads 14
    /// and the value slews by ticks·0.1; without one it is the raw |velocity|.
    pub fn smooth_speed(&mut self, raw: f32, ticks: u32) {
        let mut s = raw;
        if self.has_horn {
            if 5.0 < s && s < 14.0 { s = 14.0; }
            let step = ticks as f32 * 0.1;
            let d = s - self.speed;
            if d <= -step { self.speed -= step; return; }
            if step <= d { self.speed += step; return; }
        }
        self.speed = s;
    }

    /// `FUN_002e1768` [M]: the wobble, scrapes and skids. `mix` supplies the group volumes
    /// (`Scrapes` = +0x208, `Skids` = +0x204).
    pub fn base_update(&mut self, inp: &VehicleInputs, ticks: u32, l: &Listener, mix: &mut Mixer) -> VehicleFrame {
        self.smooth_speed(inp.speed, ticks);
        let t = ticks as f32;
        let step = t * 0.04;
        // ±150 rpm triangle at 5 rpm per tick (stored before the bounds test, flipped after)
        self.wobble += t * 5.0 * self.wobble_dir;
        if 150.0 < self.wobble { self.wobble_dir = -1.0; } else if self.wobble < -150.0 { self.wobble_dir = 1.0; }
        let scrapes_vol = mix.volume_of("Scrapes");
        let mut scrapes = [VoiceOut { sfx: "", volume: 0.0, pitch: 0.0 }; 3];
        for i in 0..3 {
            let mut target = 0.0;
            if self.scrape[i] >= 1 {
                self.scrape[i] = (self.scrape[i] - ticks as i32).max(0);
                if 3 < self.scrape[i] { target = if self.speed < 15.0 { self.speed * 0.066_666_67 } else { 1.0 }; }
            }
            self.scrape_level[i] = slew(self.scrape_level[i], target, step);
            scrapes[i] = VoiceOut { sfx: SCRAPE_VOICES[i], volume: voice_gate(self.scrape_level[i] * scrapes_vol * l.gain), pitch: l.pitch };
        }
        self.skid_front = slew(self.skid_front, inp.skid_front, step);
        let skids_vol = mix.volume_of("Skids");
        let front = self.skid_front * skids_vol * self.skid_multiple;
        self.skid_rear = slew(self.skid_rear, inp.skid_rear, step);
        let skids_vol = mix.volume_of("Skids");
        let rear = self.skid_rear * skids_vol * self.skid_multiple;
        let w = inp.wheel_surface;
        let mut skids = [(VoiceOut { sfx: "", volume: 0.0, pitch: 0.0 }, VoiceOut { sfx: "", volume: 0.0, pitch: 0.0 }); 4];
        for (k, (fname, rname, surfs, rear_pitch)) in SKID_SETS.iter().enumerate() {
            let (fname, rname) = if k == 0 && !self.class_e { ("SFX_SquealFront1", "SFX_SquealBack1") } else { (*fname, *rname) };
            let on_f = surfs.contains(&w[0]) || surfs.contains(&w[1]);
            let on_r = surfs.contains(&w[2]) || surfs.contains(&w[3]);
            skids[k] = (
                VoiceOut { sfx: fname, volume: if on_f { voice_gate(front * l.gain) } else { 0.0 }, pitch: l.pitch },
                VoiceOut { sfx: rname, volume: if on_r { voice_gate(rear * l.gain) } else { 0.0 }, pitch: l.pitch * rear_pitch },
            );
        }
        VehicleFrame { scrapes, skids, road: None, engine: None }
    }
}

/// The player car's sound: the vehicle base plus the engine and the road noise (`FUN_002f0900`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerCarSound { pub base: VehicleSound, pub engine: EngineSound }

impl PlayerCarSound {
    pub fn new(skid_multiple: f32) -> PlayerCarSound {
        let mut base = VehicleSound::new(true, false, skid_multiple);
        base.road_noise = Some("SFX_PavedRN");
        PlayerCarSound { base, engine: EngineSound::new(PLAYER_ENGINE_RPM) }
    }

    /// One audio update (`ticks` sim ticks since the last one).
    pub fn update(&mut self, inp: &VehicleInputs, ticks: u32, l: &Listener, mix: &mut Mixer) -> VehicleFrame {
        let mut inp = *inp;
        // the 500 rpm floor on +0x11c (tested on rpm + wobble, before the base step) [M 0x2f0900]
        if inp.rpm + self.base.wobble < 500.0 { inp.rpm = 500.0; }
        let mut f = self.base.base_update(&inp, ticks, l, mix);
        // the engine object's +0xc update: FUN_002ff320 → FUN_002fa8f0
        let rear_free = inp.wheel_surface[2] == 0 || inp.wheel_surface[3] == 0;
        self.engine.base_update(inp.rpm, self.base.wobble, self.base.skid_rear, inp.throttle, rear_free, ticks);
        // car volume: the engine's own group `Player Car` × the owner's +0x6c (1.0 [S: not traced past the ctor])
        let car = mix.volume_of("Player Car");
        f.engine = Some(self.engine.layer_update(ticks, l, car));
        // road noise: the current voice gets its volume first, then a changed wheel surface
        // replaces the voice (the last changed wheel wins; the new voice sounds from the next update)
        if let Some(sfx) = self.base.road_noise {
            let s = self.base.speed;
            let ramp = if s < 15.0 { 0.0 } else if s < 50.0 { (s - 15.0) * 0.028_571_429 } else { 1.0 };
            // the road-noise voice is built in the player's +0x1fc group (`Asphalt`, FUN_002defd0)
            let asphalt = mix.volume_of("Asphalt");
            f.road = Some(VoiceOut { sfx, volume: voice_gate(ramp * asphalt * l.gain), pitch: l.pitch });
        }
        for i in 0..4 {
            if inp.wheel_surface[i] != self.base.last_surface[i] {
                self.base.last_surface[i] = inp.wheel_surface[i];
                self.base.road_noise = road_noise(inp.wheel_surface[i]);
            }
        }
        f
    }
}

// ---------------------------------------------------------------------------------------------
// One-shots
// ---------------------------------------------------------------------------------------------

/// A one-shot request: `FUN_002f4b10(slot, sfx, group, …)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OneShot { pub slot: usize, pub sfx: &'static str, pub group: &'static str }

/// `FUN_00124650` [M]: the sniper bolt — with the weapon in state 3, an animation time strictly
/// inside (13.9, 14.1) plays `SFX_sniperbolt` from the Weapons bank (slot 4) in `POV Weapons`
/// (the bank's `SFX_par_sniperBolt.ptc` — the lookup is case-insensitive).
pub fn sniper_bolt(weapon_state: i32, anim_time: f32) -> Option<OneShot> {
    (weapon_state == 3 && 13.9 < anim_time && anim_time < 14.1).then_some(OneShot { slot: 4, sfx: "SFX_sniperbolt", group: "POV Weapons" })
}

/// The stream channels `FUN_0015d000` knows: `speech` streams go through the `SpeechVsAmbience`
/// fader (played only while the mission is not in a state `FUN_00206788` reports), `nis` likewise
/// (no fade-in, looped flag forced), any other channel name is a mix group (`music` → `Music`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamChannel { Speech, Nis, Group }

pub fn stream_channel(name: &str) -> StreamChannel {
    if name.eq_ignore_ascii_case("speech") { StreamChannel::Speech }
    else if name.eq_ignore_ascii_case("nis") { StreamChannel::Nis }
    else { StreamChannel::Group }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speech_archives_per_track() {
        assert_eq!(speech_base("paris_mis01"), "mis01");
        assert_eq!(speech_base("PARIS_MAP"), "mis01");
        assert_eq!(speech_base("snow2a_race"), "mis4");
        assert_eq!(speech_base("junglec"), "mis13c");
        assert_eq!(speech_base("anything"), "mis11");
        assert_eq!(speech_archive("snow1a_mis3", 1, |_| false), "driving\\mis3en.spe");
        assert_eq!(speech_archive("snow1a_mis3", 1, |p| p.ends_with("fr.spe")), "driving\\mis3fr.spe");
    }

    #[test]
    fn mix_tree_volumes() {
        let ini = nf_formats::easnd::parse_mix_ini(b"Player Car = 0.350, 5\r\nSkids = 0.800, 4\r\nMusic = 1.000, 6\r\n");
        let mut m = Mixer::driving(&ini, 80, 50);
        // Player Car -> Fader2 -> Fader1 -> Fader0 -> 1:Effects (0.5)
        assert!((m.volume_of("Player Car") - 0.35 * 0.5).abs() < 1e-6);
        assert!((m.volume_of("Skids") - 0.8 * 0.5).abs() < 1e-6);
        assert!((m.volume_of("Music") - 0.8).abs() < 1e-6);
        // a group the ini lacks keeps the ctor's 0 volume
        assert_eq!(m.volume_of("Horns"), 0.0);
        // fades: +0.25 per read, clamped at the target
        let i = m.find("Skids").unwrap();
        m.fade(i, 1.0, 0.25);
        m.volume(i);
        assert_eq!(m.groups[i].volume, 1.0);
        assert!(!m.groups[i].fading);
    }

    #[test]
    fn voice_gate_quantum() {
        assert_eq!(voice_gate(0.007874016), 0.0);
        assert_eq!(voice_gate(0.0079), 0.0079);
        assert_eq!(voice_gate(3.0), 1.0);
    }

    #[test]
    fn engine_base_step_traced_values() {
        let mut e = EngineSound::new(PLAYER_ENGINE_RPM);
        // rising toward 3000: rpm += 10 % of the gap, load slews 0.05 per tick
        e.base_update(3000.0, 0.0, 0.0, 1.0, false, 1);
        assert_eq!(e.rpm, 300.0);
        assert!((e.load - 0.05).abs() < 1e-7);
        assert!((e.pitch - 2300.0 / 4500.0).abs() < 1e-6);
        // a drop of more than 150/tick: rpm falls by 75/tick and the load step triples toward throttle/8
        let mut e = EngineSound { rpm: 4000.0, load: 1.0, ..EngineSound::new(PLAYER_ENGINE_RPM) };
        e.base_update(1000.0, 0.0, 0.0, 1.0, false, 1);
        assert_eq!(e.rpm, 3925.0);
        assert!((e.load - 0.85).abs() < 1e-6);
        // rear wheel off the ground with throttle: target 10000
        let mut e = EngineSound::new(PLAYER_ENGINE_RPM);
        e.base_update(0.0, 0.0, 0.0, 0.5, true, 2);
        assert_eq!(e.rpm, 1000.0);
        // rear skid pulls the target down by 1000 per unit
        let mut e = EngineSound { rpm: 2000.0, ..EngineSound::new(PLAYER_ENGINE_RPM) };
        e.base_update(3000.0, 100.0, 0.5, 0.0, false, 1);
        assert_eq!(e.rpm, 2060.0);
    }

    #[test]
    fn engine_layers_crossfade_and_distance() {
        let mut e = EngineSound { rpm: 1500.0, load: 0.36, w_load: 0.6, w_idle: 0.8, pitch: 0.5, ..EngineSound::new(PLAYER_ENGINE_RPM) };
        let f = e.layer_update(1, &Listener { distance: 0.5, ..Listener::default() }, 0.5);
        // vacuum fires on the first update past load 0.2; load <= 0.5 so no pitch wobble
        assert!(f.vacuum);
        assert_eq!(e.vacuum_t, 1.0);
        let lo = (1.0f32 - 1500.0 * 0.000_333_333_33).sqrt();
        let hi = (1500.0f32 * 0.000_333_333_33).sqrt();
        assert!((f.voices[0].volume - 0.5 * lo * 0.8).abs() < 1e-6, "IN-idle");
        assert!((f.voices[0].pitch - 0.55).abs() < 1e-6, "idle pitch × 1.1");
        assert!((f.voices[1].volume - 0.5 * 0.6).abs() < 1e-6, "IN-lowload");
        assert!((f.voices[3].volume - 0.5 * hi * 0.6).abs() < 1e-6, "IN-hiload");
        assert!((f.voices[4].volume - 0.5 * hi * 0.8).abs() < 1e-6, "IN-cruz");
        assert!(f.voices[5..].iter().all(|v| v.volume == 0.0), "interior distance: exterior silent");
        // distance 1.5: sqrt(0.5) each side
        let mut e2 = EngineSound { rpm: 2500.0, load: 0.1, w_load: 1.0, w_idle: 0.0, ..EngineSound::new(PLAYER_ENGINE_RPM) };
        let f = e2.layer_update(1, &Listener { distance: 1.5, ..Listener::default() }, 1.0);
        assert!((f.voices[1].volume - 0.5f32.sqrt()).abs() < 1e-6);
        assert!((f.voices[6].volume - 0.5f32.sqrt()).abs() < 1e-6);
        assert_eq!(f.voices[0].volume, 0.0, "rpm >= 2000: no low layer");
        assert!(!f.vacuum);
    }

    #[test]
    fn game_sine_matches_sin() {
        for i in -40..40 {
            let x = i as f32 * 0.037;
            // the game's 9th-order series is off by up to 0.0033 near half a turn [M coefficients]
            assert!((game_sin_turns(x) - (x * std::f32::consts::TAU).sin()).abs() < 4e-3, "{x}");
        }
        assert!((game_sin_turns(0.5) - 0.0).abs() > 1e-3, "the series' own error at half a turn is kept");
    }

    #[test]
    fn vacuum_pitch_wobble() {
        let mut e = EngineSound { load: 0.8, vacuum_t: 4.0, pitch: 1.0, ..EngineSound::new(PLAYER_ENGINE_RPM) };
        e.layer_update(1, &Listener::default(), 1.0);
        // t = 5: sin(2π·0.5) ≈ 0
        assert!((e.pitch - 1.0).abs() < 1e-3);
        let mut e = EngineSound { load: 0.8, vacuum_t: 1.5, pitch: 1.0, ..EngineSound::new(PLAYER_ENGINE_RPM) };
        e.layer_update(1, &Listener::default(), 1.0);
        // t = 2.5: sin(2π·0.25) = 1 → +0.3·2/4.5
        assert!((e.pitch - (1.0 + 0.3 * 2.0 / 4.5)).abs() < 1e-3);
    }

    #[test]
    fn wobble_skids_scrapes_road_noise() {
        let ini = nf_formats::easnd::parse_mix_ini(b"Skids = 1.000, 4\r\nScrapes = 1.000, 3\r\nAsphalt = 0.490, 4\r\nPlayer Car = 0.350, 5\r\n");
        let mut m = Mixer::driving(&ini, 100, 100);
        let mut p = PlayerCarSound::new(0.0);
        let inp = VehicleInputs { wheel_surface: [1, 1, 3, 3], throttle: 1.0, rpm: 3000.0, skid_front: 1.0, skid_rear: 0.5, speed: 32.5 };
        let l = Listener::default();
        p.base.scrape[0] = 10;
        let f = p.update(&inp, 1, &l, &mut m);
        assert_eq!(p.base.wobble, 5.0);
        // skids slew 0.04 per tick: front paved voice at 0.04, rear grass voice at 0.04 with pitch 0.75
        assert!((f.skids[0].0.volume - 0.04).abs() < 1e-6 && f.skids[0].1.volume == 0.0);
        assert!((f.skids[1].1.volume - 0.04).abs() < 1e-6 && f.skids[1].1.pitch == 0.75 && f.skids[1].0.volume == 0.0);
        // scrape 0: counter 10 -> 9 (> 3) so target = 1 (speed >= 15); level 0.04
        assert_eq!(p.base.scrape[0], 9);
        assert!((f.scrapes[0].volume - 0.04).abs() < 1e-6);
        // road noise: the ctor's paved voice, speed 32.5 -> (32.5-15)/35 = 0.5, × Asphalt 0.49
        let r = f.road.unwrap();
        assert_eq!(r.sfx, "SFX_PavedRN");
        assert!((r.volume - 0.5 * 0.49).abs() < 1e-5);
        // wheel 3 changed last (0 -> 3, grass): no road-noise voice from the next update on
        assert_eq!(p.base.road_noise, None);
        // the wobble turns at +150 after 31 ticks of 5
        for _ in 0..30 { p.update(&inp, 1, &l, &mut m); }
        assert_eq!(p.base.wobble, 155.0);
        assert_eq!(p.base.wobble_dir, -1.0);
        assert_eq!(road_noise(15), Some("SFX_OffRN"));
        assert_eq!(road_noise(3), None);
    }

    #[test]
    fn speed_smoothing_with_a_horn() {
        let mut v = VehicleSound::new(true, true, 1.0);
        v.smooth_speed(8.0, 1);
        assert!((v.speed - 0.1).abs() < 1e-7, "(5,14) reads 14, slewed 0.1/tick");
        let mut v = VehicleSound::new(true, false, 1.0);
        v.smooth_speed(8.0, 1);
        assert_eq!(v.speed, 8.0);
    }

    #[test]
    fn sniper_bolt_window() {
        assert!(sniper_bolt(3, 14.0).is_some());
        assert!(sniper_bolt(3, 13.9).is_none());
        assert!(sniper_bolt(2, 14.0).is_none());
        assert_eq!(stream_channel("Speech"), StreamChannel::Speech);
        assert_eq!(stream_channel("music"), StreamChannel::Group);
    }
}
