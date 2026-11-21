//! The driving vehicles' class sounds (tank treads, helicopter rotor, ultralight), the weapon fire /
//! projectile flight / explosion sounds, which matrix the audio listener uses, and the writers of
//! the landing strength and the horn request. docs/systems/drive-audio.md is the write-up.
//!
//! Evidence (all [M], DRIVING.ELF decompile / disassembly, EE addresses; [D] disc data):
//! - Class updates are vtable +0x24 of each class (+0xac): tank 0x3c47f8 → `FUN_002f1000`, heli
//!   0x3c4960 → `FUN_002f16b8`, ultralight 0x3c2b08 → `FUN_002e3c40` (→ vtable +0x54
//!   `FUN_002e3c90`), boat 0x3c3258 → `FUN_002e7358`, snowmobile 0x3c4118 → `FUN_002edd88`. The
//!   player car's `FUN_002f0900` sits in the same slot (0x3c469c). Every voice goes through the
//!   gate `FUN_002ddfd0(volume, pitch, pan, ..)`; +0x6c / +0x70 are the sound's own volume and
//!   pitch (1.0 from the base ctor `FUN_002f0480`), +0x84 its group (`Player Car` for all five).
//! - Listener `FUN_00147e30`: `DAT_00333ed8` (the render view set, made by `FUN_001c3db0` at
//!   start-up and cleared by `FUN_001c3ee8`) non-null → `FUN_002f4e00(**DAT_00333ed8, +0x40)`:
//!   the main view's camera matrix and the velocity beside it; null → the player body's matrix
//!   (body pool 0x339190 + index player+0x46, its +0x5c) with the position replaced by the car's
//!   (+0x10) and the car's velocity (+0x20). Both then store the car position at listener +0x60.
//! - Landing strength: the bridge `FUN_00188200` copies car +0x2ec into the sound's +0x150 (the
//!   `puVar12 + 0x2a` store, 8-byte stride). The car step `FUN_0018cef8` zeroes +0x2ec; the wheel
//!   pass (`FUN_0018c1b0`, `FUN_0018e548`) raises it to max(+0x2ec, compression × 5.0
//!   (0x396330)) for each wheel whose vtable +0x214 query returns 0; `FUN_0018cef8` then clamps it to
//!   1 when it is above 0.01. The query is `FUN_001931f8` (vtable 0x379ef0 / 0x37a9f8 +0x214):
//!   `car[+0x210 + 4*wheel] != 0`, the wheel's compression the pass stored at the end of that
//!   wheel's previous step (0x18c1b0 :644, 0x18e548 :363). So only wheels that had no compression
//!   last pass (were in the air) count: the strength is how hard the landing wheels hit.
//! - Horn request: the traffic driver's avoidance `FUN_001377c8` writes 1 into its own car's sound
//!   +0x154 when `FUN_00135530` reports another body in its way (grid cells within 3/2/3, the other
//!   body's +0x6d < 4 and +0x6e != 3) whose controller (vtable +0x5c) +0x70 is not 4; the sound base
//!   step `FUN_002e1768` passes it to the horn (`FUN_002dea48`) and clears it. The blocker test
//!   `FUN_00135530(range, driver, target, other)` (disassembly 0x135630-0x135740): see
//!   [`horn_blocker`]; range = max(other +0x5c -> +0x4c8, +0x4c0) (fields not named).
//! - Fire `FUN_0014ca60`: weapon = `FUN_0020e678` (the weapon whose barrel list holds the action),
//!   key `SFX_<stem>Fire` from slot 4 (`PTR_DAT_003562b0`), then the jump table 0x36f5a0 [D]:
//!   see [`fire_sound`].
//! - Flight `FUN_00180bc8`: an `ASound` (`FUN_002ed3d0`) per projectile, see [`transit_sound`].
//! - Hits `FUN_00182978`: the Q-Pulse plays `SFX_EMPdetonate` (POV Weapons, min 50 / max 500);
//!   other kinds call `FUN_00200088`, whose explosion event (`FUN_00159fb8`, handler
//!   `FUN_0015a098`, also made by `ESpawnExplosionStatic` `FUN_0015b010`) plays [`explosion`].
//! - Lock warning `FUN_00181e70`: a projectile homing on what the player car's vtable +0x29c returns
//!   beeps `SFX_Htargetlock` (slot 0, `Tracking System`, 2-D) once its +0x71c time is passed, then
//!   +0x71c += (int)(distance × fps × 0.0001).
//!
//! The boat, snowmobile, turret and tank steps are in `class_sounds.rs`. Not settled (stated
//! plainly): whether a projectile fired by an AI element with +0x138 bit 1 clear has a group name
//! (the code reads a stack buffer that this path never writes); what the other body's +0x4c0 /
//! +0x4c8 and the controller kind 4 are.

use super::audio::{voice_gate, Listener, Mixer};
use super::sound_events::{Listener3d, V3};

/// One voice's gate inputs (`FUN_002ddfd0(volume, pitch, pan, ..)`), volume after [`voice_gate`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanVoice {
    pub sfx: &'static str,
    pub volume: f32,
    pub pitch: f32,
    /// Turns, + = right (0.75 is the same as -0.25).
    pub pan: f32,
}

fn pv(sfx: &'static str, volume: f32, pitch: f32, pan: f32) -> PanVoice { PanVoice { sfx, volume: voice_gate(volume), pitch, pan } }

// ---------------------------------------------------------------------------------------------
// Class updates
// ---------------------------------------------------------------------------------------------

/// `FUN_002f1000` [M]: the tank tread voice (`SFX_TankTread`, slot 0, group `Tank Treads`, at
/// +0x234). Level = 0 when either rear wheel (+0xdc / +0xe0) has no contact, else min(speed / 8,
/// 1) with speed = the smoothed +0x13c. Volume = level × `Tank Treads` × gain, pitch =
/// (level / 2 + 1/2) × doppler, the listener pan.
pub fn tank_treads(rear: [u32; 2], speed: f32, l: &Listener, mix: &mut Mixer) -> PanVoice {
    let level = if rear[0] == 0 || rear[1] == 0 { 0.0 } else if speed < 8.0 { speed * 0.125 } else { 1.0 };
    let g = mix.volume_of("Tank Treads");
    pv("SFX_TankTread", level * g * l.gain, (level * 0.5 + 0.5) * l.pitch, l.pan)
}

/// The helicopter's sound state: +0x228 the rotor blend.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeliSound { pub blend: f32 }

pub const HELI_VOICES: [&str; 2] = ["SFX_helichange", "SFX_heli"];

impl HeliSound {
    /// `FUN_002f16b8` [M]: blend = 0.9 × blend (+ 0.1 while |velocity| >= 1); three
    /// `SFX_helichange` voices × (1 - blend) and three `SFX_heli` × blend. Inside (listener
    /// distance < 2) they play un-attenuated at pans 0.25 / 0.75 / 0; outside only the first of
    /// each, × gain at the listener pan. `own` = (+0x6c, +0x70), group `Player Car`.
    pub fn update(&mut self, speed: f32, own: (f32, f32), l: &Listener, mix: &mut Mixer) -> [PanVoice; 6] {
        self.blend = if speed < 1.0 { self.blend * 0.9 } else { self.blend * 0.9 + 0.1 };
        let g = mix.volume_of("Player Car");
        let (v, p) = (own.0 * g, own.1 * l.pitch);
        let (a, b) = (1.0 - self.blend, self.blend);
        let [c, h] = HELI_VOICES;
        if l.distance < 2.0 {
            [pv(c, v * a, p, 0.25), pv(c, v * a, p, 0.75), pv(c, v * a, p, 0.0), pv(h, v * b, p, 0.25), pv(h, v * b, p, 0.75), pv(h, v * b, p, 0.0)]
        } else {
            [pv(c, v * a * l.gain, p, l.pan), pv(c, 0.0, p, l.pan), pv(c, 0.0, p, l.pan), pv(h, v * b * l.gain, p, l.pan), pv(h, 0.0, p, l.pan), pv(h, 0.0, p, l.pan)]
        }
    }
}

/// The ultralight's sound state (+0x234 pitch, +0x238 / +0x23c / +0x240 layer levels).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UltralightSound { pub pitch: f32, pub low: f32, pub dive: f32, pub high: f32 }

impl Default for UltralightSound {
    /// `FUN_002e35b8` [M]: +0x238 = 1, the rest 0.
    fn default() -> Self { UltralightSound { pitch: 0.0, low: 1.0, dive: 0.0, high: 0.0 } }
}

/// The three layers' keys (dive, low load, high load) in voice order +0x210.. [M ctor strings].
pub const ULTRALIGHT_VOICES: [&str; 3] = ["SFX_bultralightDive", "SFX_BultraliteLdLW", "SFX_BultraliteLdHI"];

impl UltralightSound {
    /// `FUN_002e3c90` [M], `vy` = the velocity's y (+0x14). Level flight (|vy| <= 5): low load
    /// rises (x 0.95 + 0.05), the others decay x 0.95, pitch target |vy| × 0.3 / 5 + 0.7. Climbing
    /// (vy > 5): high load rises, target 1.2, pitch capped at 1.2. Diving (vy < -5): the dive layer
    /// rises, high decays, low is left alone, target |vy / 12|, pitch kept within 1..1.7. Pitch =
    /// 0.95 × pitch + 0.05 × target. Outside (distance > 1): one voice per layer × gain at the
    /// listener pan; inside: all nine, pans 0.25 / 0.75 / 0, still × gain.
    pub fn update(&mut self, vy: f32, own: (f32, f32), l: &Listener, mix: &mut Mixer) -> [PanVoice; 9] {
        let target;
        if vy > 5.0 {
            target = 1.2;
            self.dive *= 0.95;
            self.low *= 0.95;
            self.high = self.high * 0.95 + 0.05;
        } else if vy < -5.0 {
            target = (vy / 12.0).abs();
            self.high *= 0.95;
            self.dive = self.dive * 0.95 + 0.05;
        } else {
            self.high *= 0.95;
            self.low = self.low * 0.95 + 0.05;
            target = vy.abs() * 0.3 / 5.0 + 0.7;
            self.dive *= 0.95;
        }
        self.pitch = self.pitch * 0.95 + target * 0.05;
        if vy > 5.0 && self.pitch > 1.2 { self.pitch = f32::from_bits(0x3f99999a); }
        if vy < -5.0 {
            if self.pitch < 1.0 { self.pitch = 1.0; }
            if self.pitch > 1.7 { self.pitch = f32::from_bits(0x3fd9999a); }
        }
        let g = mix.volume_of("Player Car");
        let p = own.1 * self.pitch * l.pitch;
        let lv = [self.dive, self.low, self.high];
        let v = |i: usize| own.0 * g * lv[i] * l.gain;
        let k = ULTRALIGHT_VOICES;
        if 1.0 < l.distance {
            [
                pv(k[0], v(0), p, l.pan), pv(k[1], v(1), p, l.pan), pv(k[2], v(2), p, l.pan),
                pv(k[0], 0.0, p, 0.75), pv(k[1], 0.0, p, 0.75), pv(k[2], 0.0, p, 0.75),
                pv(k[0], 0.0, p, 0.0), pv(k[1], 0.0, p, 0.0), pv(k[2], 0.0, p, 0.0),
            ]
        } else {
            [
                pv(k[0], v(0), p, 0.25), pv(k[1], v(1), p, 0.25), pv(k[2], v(2), p, 0.25),
                pv(k[0], v(0), p, 0.75), pv(k[1], v(1), p, 0.75), pv(k[2], v(2), p, 0.75),
                pv(k[0], v(0), p, 0.0), pv(k[1], v(1), p, 0.0), pv(k[2], v(2), p, 0.0),
            ]
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Listener, landing strength, horn request
// ---------------------------------------------------------------------------------------------

/// A body / camera matrix: rows right, up, forward and the position row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Basis { pub right: V3, pub up: V3, pub fwd: V3, pub pos: V3 }

/// `FUN_00147e30` [M]: the audio listener. `view` = the main render view's camera matrix and the
/// velocity stored after it (+0x40), when the view set exists; `car` = the player body's matrix,
/// position (+0x10) and velocity (+0x20). Returns the listener and its +0x60 point (the car).
pub fn audio_listener(view: Option<(Basis, V3)>, car: (Basis, V3)) -> (Listener3d, V3) {
    let (b, vel) = match view {
        Some(v) => v,
        None => car,
    };
    (Listener3d::new(b.pos, b.right, b.up, b.fwd, vel), car.0.pos)
}

/// The landing strength the bridge hands the sound (+0x150) [M `FUN_0018cef8` / `FUN_0018c1b0` /
/// `FUN_001931f8`]: max over the wheels whose previous-pass compression (car +0x210[i]) is 0 of
/// this pass's compression × 5, then min(·, 1) when above 0.01 (left as is otherwise).
pub fn landing_strength(compression: [f32; 4], prev_compression: [f32; 4]) -> f32 {
    let mut s = 0.0f32;
    for i in 0..4 {
        // FUN_001931f8: the query is `prev != 0.0`; the wheel counts when it returns 0
        if prev_compression[i] == 0.0 {
            let v = compression[i] * f32::from_bits(0x40a00000);
            if v > s { s = v; }
        }
    }
    if 0.01 < s && s > 1.0 { 1.0 } else { s }
}

/// `FUN_001377c8` [M]: the horn request a traffic driver raises for a blocker in its way — any
/// controller kind but 4.
pub fn horn_request(blocker_kind: i32) -> bool { blocker_kind != 4 }

/// `FUN_001377c8` [M 0x1377c8 grid loop]: the other body is looked at when its grid cell (+0x5c ->
/// +0x4f0 / +0x4f4 / +0x4f8) is within 3 / 2 / 3 of the driver's, its +0x6d < 4 and +0x6e != 3,
/// and it is not the driver's own body.
pub fn horn_candidate(own_cell: [i32; 3], other_cell: [i32; 3], other_6d: i8, other_6e: i8, same_body: bool) -> bool {
    let d = |k: usize| (own_cell[k] - other_cell[k]).abs();
    !same_body && other_6d < 4 && other_6e != 3 && d(0) < 4 && d(1) < 3 && d(2) < 4
}

fn sub3(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn len3(a: V3) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }

/// `FUN_00135530` [M, disassembly 0x135630-0x135740]: is `other` in the way of a car at `own`
/// heading for `target`? Only when the two directions from the car are within 0.9 (dot of the
/// unit vectors, about 26 degrees): then yes if `other` is closer than `range` to the target,
/// else yes if `other` is nearer the car than the target is (`c.olt.s |own-other|, |own-target|`).
pub fn horn_blocker(own: V3, target: V3, other: V3, range: f32) -> bool {
    let a = sub3(target, own);
    let b = sub3(other, own);
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / (len3(a) * len3(b));
    if dot < f32::from_bits(0x3f66_6666) { return false; }
    if len3(sub3(other, target)) < range { return true; }
    len3(sub3(own, other)) < len3(sub3(own, target))
}

// ---------------------------------------------------------------------------------------------
// Weapons
// ---------------------------------------------------------------------------------------------

/// Who fired, as `FUN_0014ca60` sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shooter {
    /// Event +0x14 set (an AI element); `bit1` = its +0x138 bit 1.
    Element { bit1: bool },
    /// A vehicle: `kind1` = its +0x40 is 1, `player` = it is the player car, `flying` = its
    /// `IS_FLYING` attribute is 1 (the heli gun, uStack_164; forced 0 for the player).
    Vehicle { kind1: bool, player: bool, flying: bool },
}

/// One fire / flight / hit sound request.
#[derive(Clone, Debug, PartialEq)]
pub struct WeaponSound {
    pub slot: usize,
    pub sfx: String,
    pub group: &'static str,
    /// 3-D min / max distance; `None` = a 2-D one-shot (`FUN_002f4a20`).
    pub range: Option<(f32, f32)>,
    /// The sound's +0x70 (written 0.75 for the flying gun; read as the pitch factor [S]).
    pub pitch: f32,
}

/// The counters `FUN_0014ca60` keeps (`DAT_0032bba4`, `DAT_0032bba0`, `DAT_0032bba8`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireCounters { pub element: u32, pub alternate: u32, pub vehicle: u32 }

/// The weapons whose fire takes the 2-D `POV Weapons` path (jump table 0x36f5a0 → 0x14e4f8 [D]).
pub const FIRE_POV: [usize; 10] = [3, 10, 11, 14, 18, 22, 25, 26, 28, 30];
/// → 0x14e9a8: a 3-D one-shot, min 50 / max 200.
pub const FIRE_3D: [usize; 6] = [1, 2, 5, 16, 19, 21];
/// → 0x14ebd8 ("WARNING: Audio not playing WEAPONTYPE!").
pub const FIRE_WARN: [usize; 8] = [8, 9, 12, 13, 15, 17, 20, 23];
/// → 0x14ebec directly: no fire sound in this event.
pub const FIRE_NONE: [usize; 6] = [4, 6, 7, 24, 27, 29];

/// `FUN_0014ca60` [M 0x14e490-0x14ebd4]: the fire sound of weapon `w` (stem from
/// `weapons::WEAPONS`). `mis3` = the track is `snow1a_mis3` (every other gun shot is skipped there).
pub fn fire_sound(w: usize, stem: &str, shooter: Shooter, mis3: bool, c: &mut FireCounters) -> Option<WeaponSound> {
    let key = format!("SFX_{stem}Fire");
    if FIRE_POV.contains(&w) {
        return Some(WeaponSound { slot: 4, sfx: key, group: "POV Weapons", range: None, pitch: 1.0 });
    }
    if w == 0 {
        return match shooter {
            Shooter::Element { bit1 } => {
                if bit1 { return None; }
                c.element = c.element.wrapping_add(1);
                c.element = if mis3 { c.element & 1 } else { 0 };
                if c.element != 0 { return None; }
                let k = ["SFX_HMachGun1", "SFX_HMachGun2"][(c.alternate & 1) as usize];
                c.alternate = c.alternate.wrapping_add(1);
                Some(WeaponSound { slot: 4, sfx: k.into(), group: "Henchmen Fire", range: Some((20.0, 200.0)), pitch: 1.0 })
            }
            Shooter::Vehicle { kind1, player, flying } => {
                if !kind1 || player { return None; }
                c.vehicle += 1;
                if 2 <= c.vehicle { c.vehicle = 0; }
                if !mis3 { c.vehicle = 0; }
                if c.vehicle != 0 { return None; }
                let k = if flying { "SFX_HumntmachFire" } else { "SFX_hmntmachfire" };
                Some(WeaponSound { slot: 4, sfx: k.into(), group: "Henchmen Fire", range: Some((20.0, 300.0)), pitch: if flying { 0.75 } else { 1.0 } })
            }
        };
    }
    if FIRE_3D.contains(&w) {
        let group = match shooter {
            Shooter::Element { bit1: true } => "Weapons Fire",
            // the group buffer is not written on this path [M 0x14e9c0]
            Shooter::Element { bit1: false } => return None,
            Shooter::Vehicle { player: true, .. } => "Weapons Fire",
            Shooter::Vehicle { .. } => "Henchmen Fire",
        };
        let sfx = match shooter {
            Shooter::Vehicle { kind1: true, player: false, .. } if w == 21 => "SFX_htorpedofire".to_string(),
            _ => key,
        };
        return Some(WeaponSound { slot: 4, sfx, group, range: Some((50.0, 200.0)), pitch: 1.0 });
    }
    None
}

/// `FUN_00180bc8` [M]: the projectile's flight loop (slot 4, `ASound`, min 15; the torpedo's max
/// 300, else the ctor's 350). `by_player` = the shooter is the player's body (uStack_e8 = 0).
/// Group `POV Weapons` for the Q-Pulse, else `Weapons Transit`. `None` = an empty key.
pub fn transit_sound(kind: usize, by_player: bool, track: &str) -> Option<WeaponSound> {
    let key = match kind {
        1 if track.eq_ignore_ascii_case("snow1a_mis3") || track.eq_ignore_ascii_case("snow1a") => "",
        2 | 0x1f => "SFX_MissleTrans",
        0xe => "SFX_EMPtrans",
        0x15 if by_player => "SFX_vtorpedoTrans",
        0x15 => "SFX_htorpedotrans",
        0x16 => "SFX_vgtorpTrans",
        0x19 => "",
        0x1c => "SFX_UMissileTrans",
        _ => "SFX_RocketTrans",
    };
    if key.is_empty() { return None; }
    let group = if kind == 0xe { "POV Weapons" } else { "Weapons Transit" };
    let max = if kind == 0x15 { 300.0 } else { 350.0 };
    Some(WeaponSound { slot: 4, sfx: key.into(), group, range: Some((15.0, max)), pitch: 1.0 })
}

/// `FUN_00182978` [M]: the Q-Pulse hit.
pub fn emp_detonate() -> WeaponSound {
    WeaponSound { slot: 4, sfx: "SFX_EMPdetonate".into(), group: "POV Weapons", range: Some((50.0, 500.0)), pitch: 1.0 }
}

/// `FUN_00181e70` [M]: the lock warning. Returns the next beep time when it beeps now.
pub fn lock_warning(target_is_player: bool, next_at: i32, now: i32, distance: f32, fps: i32) -> Option<i32> {
    (target_is_player && next_at < now).then(|| next_at + (distance * fps as f32 * 0.0001) as i32)
}
pub const LOCK_WARNING: (usize, &str, &str) = (0, "SFX_Htargetlock", "Tracking System");

/// The explosion keys `FUN_0015a098` rotates through (0x32bc10 [D]).
pub const EXPLOSIONS: [&str; 4] = ["SFX_HugeExp", "SFX_LrgExp", "SFX_MedExp", "SFX_SmallExp"];

/// The explosion sound state: `DAT_0032bbfc` (the slot cursor) and `DAT_0032bc00[4]` (cooldowns).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExplosionState { pub cursor: u32, pub until: [i32; 4] }

/// `FUN_0015a098` [M 0x15a098 :429-560]: from the cursor, the first of the four slots whose
/// cooldown has passed (none → no sound); that slot cools for 0x1e clock units. Two 3-D one-shots
/// from slot 4: the slot's key in `Explosions` (min 100k, max 400k) and `SFX_DistExp` in
/// `ExplosionBass` (min 1000k, max 5000k), k = 5 on `jungleb_mis13b`, else 1.
pub fn explosion(st: &mut ExplosionState, now: i32, track: &str) -> Vec<WeaponSound> {
    let start = st.cursor & 3;
    while now <= st.until[(st.cursor & 3) as usize] {
        st.cursor = st.cursor.wrapping_add(1);
        if st.cursor & 3 == start { return Vec::new(); }
    }
    let i = (st.cursor & 3) as usize;
    st.until[i] = now + 0x1e;
    let k = if track.eq_ignore_ascii_case("jungleb_mis13b") { 5.0 } else { 1.0 };
    st.cursor = st.cursor.wrapping_add(1);
    vec![
        WeaponSound { slot: 4, sfx: EXPLOSIONS[i].into(), group: "Explosions", range: Some((k * 100.0, k * 400.0)), pitch: 1.0 },
        WeaponSound { slot: 4, sfx: "SFX_DistExp".into(), group: "ExplosionBass", range: Some((k * 1000.0, k * 5000.0)), pitch: 1.0 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driving::weapons::WEAPONS;
    use nf_formats::easnd::MixLine;

    fn mix() -> Mixer {
        let ini = [MixLine { name: "Tank Treads".into(), volume: 0.5, param: 0 }, MixLine { name: "Player Car".into(), volume: 0.8, param: 0 }];
        Mixer::driving(&ini, 100, 100)
    }

    #[test]
    fn tank_treads_follow_speed_and_rear_contact() {
        let mut m = mix();
        let l = Listener { gain: 0.5, pitch: 1.1, pan: 0.2, ..Listener::default() };
        let v = tank_treads([1, 1], 4.0, &l, &mut m);
        assert!((v.volume - 0.5 * 0.5 * 0.5).abs() < 1e-6);
        assert!((v.pitch - 0.75 * 1.1).abs() < 1e-6);
        assert_eq!(v.pan, 0.2);
        let v = tank_treads([1, 0], 20.0, &l, &mut m);
        assert_eq!((v.volume, v.pitch), (0.0, 0.5 * 1.1));
        assert!((tank_treads([1, 1], 20.0, &l, &mut m).volume - 0.25).abs() < 1e-6);
    }

    #[test]
    fn heli_blend_and_interior_pans() {
        let mut m = mix();
        let mut h = HeliSound::default();
        let inside = Listener { gain: 0.1, distance: 1.0, ..Listener::default() };
        let v = h.update(5.0, (1.0, 1.0), &inside, &mut m);
        assert!((h.blend - 0.1).abs() < 1e-6);
        // inside: no gain, change voices × 0.9, heli × 0.1, pans .25/.75/0
        assert!((v[0].volume - 0.8 * 0.9).abs() < 1e-6 && v[0].pan == 0.25);
        assert_eq!((v[1].pan, v[2].pan), (0.75, 0.0));
        assert!((v[3].volume - 0.08).abs() < 1e-6 && v[3].sfx == "SFX_heli");
        let out = Listener { gain: 0.5, distance: 10.0, pan: -0.1, ..Listener::default() };
        let v = h.update(0.0, (1.0, 1.0), &out, &mut m);
        assert!((h.blend - 0.09).abs() < 1e-6);
        assert!((v[0].volume - 0.8 * 0.91 * 0.5).abs() < 1e-6 && v[0].pan == -0.1);
        assert_eq!(v[1].volume, 0.0);
    }

    #[test]
    fn ultralight_layers_by_climb_rate() {
        let mut m = mix();
        let out = Listener { distance: 5.0, ..Listener::default() };
        let mut u = UltralightSound::default();
        u.update(0.0, (1.0, 1.0), &out, &mut m);
        assert!((u.pitch - 0.035).abs() < 1e-6);
        assert!((u.low - 1.0).abs() < 1e-6);
        // climbing: high rises, target 1.2
        let mut u = UltralightSound { pitch: 1.19, ..UltralightSound::default() };
        let v = u.update(6.0, (1.0, 1.0), &out, &mut m);
        assert!((u.pitch - (1.19 * 0.95 + 0.06)).abs() < 1e-6);
        assert!((u.high - 0.05).abs() < 1e-6 && (u.low - 0.95).abs() < 1e-6);
        assert!((v[2].volume - 0.8 * 0.05).abs() < 1e-6);
        assert_eq!(v[3].volume, 0.0);
        // diving: pitch floored at 1 and capped at 1.7
        let mut u = UltralightSound::default();
        u.update(-60.0, (1.0, 1.0), &out, &mut m);
        assert_eq!(u.pitch, 1.0);
        let mut u = UltralightSound { pitch: 1.69, ..UltralightSound::default() };
        u.update(-60.0, (1.0, 1.0), &out, &mut m);
        assert_eq!(u.pitch, f32::from_bits(0x3fd9999a));
        assert_eq!(u.low, 1.0);
        assert!((u.dive - 0.05).abs() < 1e-6);
    }

    #[test]
    fn listener_is_the_view_when_there_is_one() {
        let cam = Basis { right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], fwd: [0.0, 0.0, 1.0], pos: [0.0, 5.0, -10.0] };
        let car = Basis { right: [0.0, 0.0, -1.0], up: [0.0, 1.0, 0.0], fwd: [1.0, 0.0, 0.0], pos: [3.0, 0.0, 4.0] };
        let (l, r) = audio_listener(Some((cam, [0.0, 0.0, 1.0])), (car, [9.0, 0.0, 0.0]));
        assert_eq!((l.pos, l.fwd, l.vel, r), (cam.pos, cam.fwd, [0.0, 0.0, 1.0], car.pos));
        let (l, _) = audio_listener(None, (car, [9.0, 0.0, 0.0]));
        assert_eq!((l.pos, l.fwd, l.right, l.vel), (car.pos, car.fwd, car.right, [9.0, 0.0, 0.0]));
    }

    #[test]
    fn landing_strength_is_the_largest_counted_compression_times_five() {
        // only wheels that had no compression last pass count (they are the ones landing)
        assert_eq!(landing_strength([0.1, 0.05, 0.3, 0.0], [0.0, 0.0, 0.2, 0.0]), 0.5);
        assert_eq!(landing_strength([0.5, 0.0, 0.0, 0.0], [0.0; 4]), 1.0);
        assert_eq!(landing_strength([0.001, 0.0, 0.0, 0.0], [0.0; 4]), 0.001f32 * 5.0);
        // all four were already down: no landing
        assert_eq!(landing_strength([0.9; 4], [0.1; 4]), 0.0);
        assert!(horn_request(1) && !horn_request(4));
    }

    #[test]
    fn horn_blocker_cone_range_and_nearer() {
        let own = [0.0, 0.0, 0.0];
        let target = [0.0, 0.0, 20.0];
        // straight ahead, 5 from the target, range 10: in the way
        assert!(horn_blocker(own, target, [0.0, 0.0, 15.0], 10.0));
        // ahead but past the target and beyond the range: not nearer than the target
        assert!(!horn_blocker(own, target, [0.0, 0.0, 40.0], 10.0));
        // ahead, between, out of range (range 1): nearer than the target
        assert!(horn_blocker(own, target, [0.0, 0.0, 10.0], 1.0));
        // past the target but within range: in the way
        assert!(horn_blocker(own, target, [0.0, 0.0, 25.0], 10.0));
        // off to the side (dot = cos 45 < 0.9)
        assert!(!horn_blocker(own, target, [10.0, 0.0, 10.0], 100.0));
        // just inside the cone: dot 0.95
        let z = 10.0f32;
        let x = z * (1.0f32 - 0.95 * 0.95).sqrt() / 0.95;
        assert!(horn_blocker(own, target, [x, 0.0, z], 1.0));
        assert!(horn_candidate([5, 1, 5], [8, 3, 2], 3, 0, false));
        assert!(!horn_candidate([5, 1, 5], [9, 1, 5], 3, 0, false));
        assert!(!horn_candidate([5, 1, 5], [5, 4, 5], 3, 0, false));
        assert!(!horn_candidate([5, 1, 5], [5, 1, 1], 3, 0, false));
        assert!(!horn_candidate([5, 1, 5], [5, 1, 5], 4, 0, false));
        assert!(!horn_candidate([5, 1, 5], [5, 1, 5], 3, 3, false));
        assert!(!horn_candidate([5, 1, 5], [5, 1, 5], 3, 0, true));
    }

    #[test]
    fn fire_sounds_by_weapon_and_shooter() {
        let mut c = FireCounters::default();
        let veh = Shooter::Vehicle { kind1: true, player: false, flying: false };
        let me = Shooter::Vehicle { kind1: true, player: true, flying: false };
        // every weapon is in exactly one class
        for w in 0..31 {
            let n = [FIRE_POV.contains(&w), FIRE_3D.contains(&w), FIRE_WARN.contains(&w), FIRE_NONE.contains(&w), w == 0].iter().filter(|&&b| b).count();
            assert_eq!(n, 1, "{w}");
        }
        let s = fire_sound(26, WEAPONS[26].sfx_stem, me, false, &mut c).unwrap();
        assert_eq!((s.sfx.as_str(), s.group, s.range), ("SFX_sniperFire", "POV Weapons", None));
        let s = fire_sound(1, WEAPONS[1].sfx_stem, me, false, &mut c).unwrap();
        assert_eq!((s.sfx.as_str(), s.group, s.range), ("SFX_RocketFire", "Weapons Fire", Some((50.0, 200.0))));
        assert_eq!(fire_sound(1, WEAPONS[1].sfx_stem, veh, false, &mut c).unwrap().group, "Henchmen Fire");
        assert_eq!(fire_sound(21, WEAPONS[21].sfx_stem, veh, false, &mut c).unwrap().sfx, "SFX_htorpedofire");
        assert_eq!(fire_sound(21, WEAPONS[21].sfx_stem, me, false, &mut c).unwrap().sfx, "SFX_vtorpedoFire");
        assert_eq!(fire_sound(13, WEAPONS[13].sfx_stem, me, false, &mut c), None);
        assert_eq!(fire_sound(24, WEAPONS[24].sfx_stem, me, false, &mut c), None);
        // the player's own gun: no sound here; an AI car's gun every shot, every other on MIS3
        assert_eq!(fire_sound(0, "mntmach", me, false, &mut c), None);
        assert_eq!(fire_sound(0, "mntmach", veh, false, &mut c).unwrap().sfx, "SFX_hmntmachfire");
        let fly = Shooter::Vehicle { kind1: true, player: false, flying: true };
        let s = fire_sound(0, "mntmach", fly, false, &mut c).unwrap();
        assert_eq!((s.sfx.as_str(), s.range, s.pitch), ("SFX_HumntmachFire", Some((20.0, 300.0)), 0.75));
        let mut c = FireCounters::default();
        let shots: Vec<bool> = (0..4).map(|_| fire_sound(0, "mntmach", veh, true, &mut c).is_some()).collect();
        assert_eq!(shots, [false, true, false, true]);
        // elements alternate HMachGun1 / 2
        let mut c = FireCounters::default();
        let e = Shooter::Element { bit1: false };
        let a: Vec<String> = (0..3).map(|_| fire_sound(0, "mntmach", e, false, &mut c).unwrap().sfx).collect();
        assert_eq!(a, ["SFX_HMachGun1", "SFX_HMachGun2", "SFX_HMachGun1"]);
        assert_eq!(fire_sound(0, "mntmach", Shooter::Element { bit1: true }, false, &mut c), None);
    }

    #[test]
    fn flight_explosion_and_lock_sounds() {
        assert_eq!(transit_sound(1, true, "paris_mis01").unwrap().sfx, "SFX_RocketTrans");
        assert_eq!(transit_sound(1, true, "snow1a_mis3"), None);
        assert_eq!(transit_sound(0x15, false, "x").unwrap().range, Some((15.0, 300.0)));
        assert_eq!(transit_sound(0x15, true, "x").unwrap().sfx, "SFX_vtorpedoTrans");
        assert_eq!(transit_sound(0xe, true, "x").unwrap().group, "POV Weapons");
        assert_eq!(transit_sound(0x19, true, "x"), None);
        assert_eq!(transit_sound(7, true, "x").unwrap().sfx, "SFX_RocketTrans");
        let mut st = ExplosionState::default();
        let a = explosion(&mut st, 10, "jungleb_mis13b");
        assert_eq!(a[0].sfx, "SFX_HugeExp");
        assert_eq!(a[0].range, Some((500.0, 2000.0)));
        assert_eq!(a[1].range, Some((5000.0, 25000.0)));
        assert_eq!(explosion(&mut st, 10, "x")[0].sfx, "SFX_LrgExp");
        explosion(&mut st, 10, "x");
        explosion(&mut st, 10, "x");
        // all four cooling: silent until the first cools (until = 10 + 30)
        assert!(explosion(&mut st, 40, "x").is_empty());
        assert_eq!(explosion(&mut st, 41, "x")[0].sfx, "SFX_HugeExp");
        assert_eq!(lock_warning(true, 100, 101, 500.0, 60), Some(103));
        assert_eq!(lock_warning(true, 100, 100, 500.0, 60), None);
        assert_eq!(lock_warning(false, 0, 101, 500.0, 60), None);
    }
}
