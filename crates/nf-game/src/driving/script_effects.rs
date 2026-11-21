//! The driving script's effect events: every action whose result is not part of the mission
//! rules (camera, audio, movie / cut-scene, AI, traffic, props), decoded from its argument block
//! so the systems that own those results can consume them (docs/systems/driving-missions.md §4).
//!
//! Evidence. Each creator (`DAT_0036fda0[id]`) copies argument words `+4..` into the event in
//! order; the handler (the event vtable's `+0xc`) reads them [M per creator / handler, the
//! addresses in [`crate::driving::script::ACTIONS`]]. Field names come from the game's own debug
//! prints where the handler has one (e.g. `ECameraLockOn`: "fElem, fTrigger, fWheelIndex,
//! fHenchmenIndex, fOffset Up Fw Side, fSmoothTransition, fNoAim" [M 0x148e30 format 0x36edb0]),
//! else from what the handler does with the word; words whose use was not read are named
//! `argN` (N = the word index). Word types (reference / int / float) are checked against the
//! eight tracks' data [D, `tests/driving_script_disc.rs`]. Reference words (`0x7372_nnnn`) name
//! `sr` path records of the track's `Map` window ([`Ref`]).
//!
//! Camera times (`smooth`, `latency`) are in 60 Hz frames as stored; the handlers rescale them
//! by `DAT_00339f4c / 60` when they are above 1 (else 1) [M 0x153700 / 0x14bdb8 / 0x1539a8 /
//! 0x149528].

use nf_formats::trackstart::{MapScript, TriggerAction, TAG_SR};

/// A reference argument: the raw word (the key [`crate::driving::script::Facts::elements`] uses)
/// and the `sr` path it names, e.g. `CARP::<<Map>>::{AIEl}::2304` or `CHAR::SFX_alarm`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ref {
    pub word: u32,
    pub path: String,
}

impl Ref {
    /// The text a `CHAR::` reference carries (sound, stream, channel, movie names).
    pub fn text(&self) -> &str { self.path.strip_prefix("CHAR::").unwrap_or(&self.path) }
}

/// One decoded effect event.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    // ---- camera ----
    /// 0x3f `EPlayCameraAnim`: play a `Cams` animation (`FUN_001bc510(cam, cams, smooth,
    /// latency, arg4 != 0, arg5)`) [M 0x153700].
    CameraAnim { cams: Option<Ref>, smooth: u32, latency: u32, arg4: bool, arg5: i32 },
    /// 0x44 `EEndCameraAnim`: "smoothTransition, latency, carSpace, fCamID" [M 0x14bdb8].
    EndCameraAnim { smooth: u32, latency: u32, car_space: bool, cam_id: i32 },
    /// 0x61 `EPlayCarCameraAnim`: "fElem, smoothTransition, latency, system" [M 0x153c48].
    CarCameraAnim { element: Option<Ref>, smooth: u32, latency: u32, system: i32 },
    /// 0xcc `EPlayCameraSpline`: fly the camera along an `AISp` spline (`FUN_001bc890`) [M 0x1539a8].
    CameraSpline { spline: Option<Ref>, smooth: u32, latency: u32, arg4: i32 },
    /// 0xcf `EChangeCarCameraView`: "fCamID, fSmoothTransition, fLatency, fSetAsReturnToCam"
    /// [M 0x149528].
    CarCameraView { cam_id: i32, smooth: u32, latency: u32, set_as_return: bool },
    /// 0xf2 `ECameraLockOn`: aim the camera at an element, a trigger, a wheel or a henchman, with
    /// an offset (up, forward, side) [M 0x148e30].
    CameraLockOn {
        element: Option<Ref>,
        trigger: Option<usize>,
        wheel: i32,
        henchman: i32,
        offset_up: f32,
        offset_forward: f32,
        offset_side: f32,
        smooth: u32,
        no_aim: bool,
    },
    /// 0xf6 `ECameraZoom`: "fZoomPercent, fSmoothTransition" [M 0x149338].
    CameraZoom { percent: i32, smooth: u32 },
    /// 0xfb `ECameraShake`: "fShakeTime, fShakePower, fAffectAll" [M 0x149118].
    CameraShake { time: i32, power: i32, affect_all: bool },
    /// 0x4d `ESetAutoDriveCamOn` / 0xea `...OnMovie`: "fStartAngle, fStartPitch" in degrees
    /// (the handler passes them / 360) [M 0x1576e8 / 0x157838]; 0x4c / 0xe9 turn it off.
    AutoDriveCam { on: bool, movie: bool, start_angle: f32, start_pitch: f32 },
    /// 0x67 `ELetterBoxOff` / 0x68 `ELetterBoxOn` [M 0x150ce8 / 0x150dc8].
    LetterBox { on: bool, arg1: i32 },

    // ---- audio ----
    /// 0x14 `EStreamEvent`: play `stream` on `channel` ("NIS", "Music", "speech", ...); `alive_only`
    /// plays it only while the player's health fraction is above 0 (`FUN_00206b20(mgr, 1)`);
    /// `start` is passed as a float, `arg4` and `arg6` as flags [M 0x15d000].
    Stream { channel: Option<Ref>, stream: Option<Ref>, start: i32, arg4: bool, alive_only: bool, arg6: bool },
    /// 0x86 `EPlaySound`: a 2D sound from a bank (`FUN_002f4aa0(bank, name, "Power Ups", 2)`)
    /// [M 0x154658]; nothing plays when either is 0.
    Sound { bank: Option<Ref>, name: Option<Ref> },
    /// 0xa4 `EPlaySoundPos`: a one-shot "World Sounds" sound at a trigger's position, with the
    /// near / far distances (`+0x5c` = near, `+0x60` = far, `+0x64` = far²) [M 0x154798]; the
    /// handler plays nothing when the position, bank or name word is 0.
    SoundAt { at: Option<usize>, bank: Option<Ref>, name: Option<Ref>, arg4: i32, near: f32, far: f32 },
    /// 0xa6 `EStopSoundPos`: stop the sound started at that trigger (`FUN_002ec9a8`) [M 0x15cd90].
    StopSoundAt { at: Option<usize> },
    /// 0xf1 `ESetAudioMix`: channel level ×= 1 + percent / 100, clamped to [0, 1], over `fade`
    /// [M 0x157308].
    AudioMix { channel: Option<Ref>, fade: i32, percent: f32 },
    /// 0xf4 `ERevertAudioMix` [M 0x156a58].
    RevertAudioMix { channel: Option<Ref>, fade: i32 },
    /// 0x10a `EResetAudioMix` [M 0x1562c8].
    ResetAudioMix { fade: i32 },
    /// 0x110 `ESwitchStingerChannel`: "music" or "speech" [M 0x15d760].
    StingerChannel { channel: Option<Ref> },
    /// 0x111 `EReleaseStream`: release the prebuffered stream [M 0x1560d8].
    ReleaseStream,
    /// 0x10e `EPathSFX`: an "Object Sounds" loop + start sound on a moving instance [M 0x152200].
    PathSound { instance: Option<Ref>, loop_sound: Option<Ref>, start_sound: Option<Ref>, near: f32, far: f32 },
    /// 0xff `ESetCreakLevel` [M 0x157cf8].
    CreakLevel { arg1: i32, arg2: i32 },

    // ---- movie / cut-scene ----
    /// 0x4e `ESetVideo`: play a movie (`data\loading\<file>` through the "mpc" player) with an
    /// optional subtitle file [M 0x1592d8].
    Movie { file: Option<Ref>, subtitles: Option<Ref> },
    /// 0xf0 `ECustomTransition` (MIS01: "paris") [M 0x14b020].
    Transition { name: Option<Ref> },
    /// 0x70 `ESetSimRate`: the slow-motion rate [M 0x158d28].
    SimRate { rate: f32 },
    /// 0x103 `ESetSubtitle`: a text id for `secs` seconds [M 0x158fc0].
    Subtitle { text: u32, secs: f32 },

    // ---- AI ----
    /// 0x41 `EAICommand`: give an element an `AICo` command, with a target element [M 0x145ff8].
    AiCommand { command: Option<Ref>, target: Option<Ref>, element: Option<Ref>, arg4: i32 },
    /// 0x42 `EActivateAIElement`: spawn / wake an element (0 = the car that fired the trigger)
    /// [M 0x1476b8].
    AiActivate { element: Option<Ref>, other: Option<Ref> },
    /// 0x43 `EDeactivateAIElement`: remove it (`FUN_0012da78`) [M 0x14b510].
    AiDeactivate { element: Option<Ref> },
    /// 0x90 `EAIElementFireOff` / 0x91 `...FireOn` [M 0x1469f8 / 0x146b48].
    AiFire { element: Option<Ref>, on: bool },
    /// 0x9f `EAIElementSetAccuracy` [M 0x146ca8].
    AiAccuracy { element: Option<Ref>, accuracy: i32 },
    /// 0xa1 `EKillAIElement`; `credit` counts the kill for the player (`FUN_00206960`) [M 0x150748].
    AiKill { element: Option<Ref>, credit: bool },
    /// 0xc7 `EAIElementSetVehicleParams` [M 0x1470e8].
    AiVehicleParams { element: Option<Ref>, value: f32 },
    /// 0x108 `EAIElementSetWakeRange` (`FUN_0012b100(range, element)`) [M 0x1471e0].
    AiWakeRange { element: Option<Ref>, range: f32 },
    /// 0x109 `ETargetBeaconOnOff` [M 0x15d888].
    TargetBeacon { element: Option<Ref>, on: bool },
    /// 0xd1 `ESetNextAISpline` [M 0x158370].
    AiNextSpline { arg1: i32, spline: Option<Ref> },
    /// 0xf5 `ESetAvoidZone` [M 0x1579b8].
    AiAvoidZone { element: Option<Ref> },
    /// 0x94 `ESpawnSentry`: a gun on an instance; the 13 words after it go to `FUN_002004a0`
    /// [M 0x15b900].
    SpawnSentry { instance: Option<Ref>, args: Vec<u32> },
    /// 0xb4 `EDropMine` [M 0x14b9c8].
    DropMine { element: Option<Ref>, arg2: i32 },
    /// 0x2a `EControlToCPU` / 0x2b `EControlToPlayer`: who drives the player car [M 0x14ac38 /
    /// 0x14ad18].
    PlayerControl { cpu: bool },
    /// 0xca `EControlToSpline` / 0xef `EEndControlToSpline` [M 0x14ae60 / 0x14c110].
    ControlToSpline { arg1: i32, spline: Option<Ref> },
    EndControlToSpline { arg1: i32 },
    /// 0x6c `EAutoDriveSpeedChange` [M 0x1482f8].
    AutoDriveSpeed { factor: f32 },

    // ---- traffic ----
    /// 0x1c `ESpawnTraffic`: a traffic car at the firing trigger's position and `+0x30`
    /// direction (`FUN_0017b900(trigger)`, `(trigger + 0x30)`) [M 0x15c080].
    SpawnTraffic { at: Option<usize>, arg1: f32 },
    /// 0x78 `EChangeMaxTraffic`: the traffic cap `DAT_0032b750`, set in the constructor
    /// [M 0x149770]; 0 = no traffic.
    MaxTraffic { max: i32 },

    // ---- props ----
    /// 0x17 / 0x18 `EHide/ShowInstanceStatic` (a named instance) and 0x0b / 0x0c
    /// `EHide/ShowInstance` (the prop part running the list, `DAT_0032e708`) [M 0x14f218 /
    /// 0x159b38 / 0x14f168 / 0x159a88].
    InstanceVisible { instance: Option<Ref>, show: bool, static_ref: bool },
    /// 0x23 `EWakeupSmackable`: wake a smackable (with the firing context's car) [M 0x15de48].
    WakeSmackable { object: u32, instance: Option<Ref>, placement: Option<Ref> },
    /// 0x24 `EPlaySystemAnim`: start system animation `anim` on part `part` of an instance
    /// (`FUN_001a27a0`) [M 0x154ad8].
    SystemAnim { part: i32, anim: i32, instance: Option<Ref>, placement: Option<Ref> },
    /// 0x09 `EPlayEffect` [M 0x153f80].
    PlayEffect { effect: Option<Ref>, instance: Option<Ref>, placement: Option<Ref> },
    /// 0xad `ESuppressEffect` [M 0x15d320].
    SuppressEffect { effect: Option<Ref>, instance: Option<Ref>, placement: Option<Ref>, arg4: i32 },
    /// 0xfc / 0xfd `ESwitchEffectOff/On` [M 0x15d538 / 0x15d650].
    SwitchEffect { element: Option<Ref>, index: i32, on: bool },
    /// 0x82 `ESetCollisionGeometry` [M 0x157be0].
    CollisionGeometry { instance: Option<Ref>, arg2: i32 },
    /// 0x9a `ESpawnExplosionStatic`: an explosion at the position word 1 names (its record's
    /// `+0`; `trigger` when that is a `Trgr` record), or when word 1 is 0 at the instance's
    /// `+0x30` translation; words 3..7 go to `FUN_00159fb8` [M 0x15b010]. Word 1 also takes the
    /// form `TRIGGER0=INSTANCE_ELEMENT::n` on MIS11 [D]; what the loader makes of it is not traced.
    Explosion { at: Option<Ref>, trigger: Option<usize>, instance: Option<Ref>, args: Vec<u32> },

    // ---- player ----
    /// 0xc0 `ESetSpecialWeapon`: the player's special weapon `weapon` (MIS11: 0x14 at the
    /// three upload points) [M 0x158df0].
    SpecialWeapon { weapon: i32, trigger: Option<usize>, arg3: f32 },

    /// Any other id: the argument words 1.. as the creator reads them.
    Other { id: u32, words: Vec<u32> },
}

/// The argument words the creator of `id` reads (word 1 = `+4`) [M, each creator].
pub fn arg_words(id: u32) -> usize {
    match id {
        0x2a | 0x2b | 0x4c | 0x7d | 0xc1 | 0xdb | 0xe9 | 0xf8 | 0x111 | 0x119 | 0x0b | 0x0c | 0xb8 => 0,
        0x17 | 0x18 | 0x1c | 0x43 | 0x67 | 0x68 | 0x6c | 0x70 | 0x78 | 0x90 | 0x91 | 0xa6 | 0xbe | 0xdc | 0xef | 0xf0
        | 0xf5 | 0x101 | 0x10a | 0x10b | 0x10d | 0x110 | 0x113 | 0x114 | 0x117 | 0xa | 0x20 | 0x8a | 0xb5 | 0x116 => 1,
        0x19 | 0x2c | 0x38 | 0x3c | 0x42 | 0x4d | 0x4e | 0x82 | 0x86 | 0x95 | 0x96 | 0x9f | 0xa1 | 0xa2 | 0xb4 | 0xc4
        | 0xc7 | 0xca | 0xd1 | 0xe8 | 0xea | 0xf4 | 0xf6 | 0xf9 | 0xfa | 0xfc | 0xfd | 0xff | 0x102 | 0x103 | 0x108
        | 0x109 | 0x79 => 2,
        0x9 | 0x23 | 0x74 | 0x99 | 0xc0 | 0xd9 | 0xf1 | 0xfb | 0x1a | 0x1b => 3,
        0x24 | 0x41 | 0x44 | 0x61 | 0xad | 0xcc | 0xcf | 0x80 | 0x81 => 4,
        0x3f | 0xbc | 0xc2 | 0xd7 | 0xd8 | 0x10e => 5,
        0x14 | 0xa4 => 6,
        0x9a | 0x21 => 7,
        0xf2 => 9,
        0x94 => 14,
        _ => 0,
    }
}

struct Args<'a> {
    a: &'a TriggerAction,
    map: &'a MapScript,
}

impl Args<'_> {
    fn u(&self, i: usize) -> u32 { self.a.word(i) }
    fn i(&self, i: usize) -> i32 { self.a.word(i) as i32 }
    fn f(&self, i: usize) -> f32 { self.a.float(i) }
    fn b(&self, i: usize) -> bool { self.a.word(i) != 0 }
    fn r(&self, i: usize) -> Option<Ref> {
        let w = self.a.word(i);
        if w >> 16 != TAG_SR >> 16 {
            return None;
        }
        Some(Ref { word: w, path: self.map.ref_path(w).unwrap_or("").to_string() })
    }
    fn t(&self, i: usize) -> Option<usize> { self.map.trigger_ref(self.a.word(i)) }
    fn rest(&self, from: usize, to: usize) -> Vec<u32> { (from..=to).map(|i| self.a.word(i)).collect() }
}

/// Decodes action `a` (fired by trigger `trigger`, `None` for rules and prop keyframes) into its
/// effect event.
pub fn decode(a: &TriggerAction, trigger: Option<usize>, map: &MapScript) -> Effect {
    let p = Args { a, map };
    match a.id {
        0x3f => Effect::CameraAnim { cams: p.r(1), smooth: p.u(2), latency: p.u(3), arg4: p.b(4), arg5: p.i(5) },
        0x44 => Effect::EndCameraAnim { smooth: p.u(1), latency: p.u(2), car_space: p.b(3), cam_id: p.i(4) },
        0x61 => Effect::CarCameraAnim { element: p.r(1), smooth: p.u(2), latency: p.u(3), system: p.i(4) },
        0xcc => Effect::CameraSpline { spline: p.r(1), smooth: p.u(2), latency: p.u(3), arg4: p.i(4) },
        0xcf => Effect::CarCameraView { cam_id: p.i(1), smooth: p.u(2), latency: p.u(3), set_as_return: p.b(4) },
        0xf2 => Effect::CameraLockOn {
            element: p.r(1),
            trigger: p.t(2),
            wheel: p.i(3),
            henchman: p.i(4),
            offset_up: p.f(5),
            offset_forward: p.f(6),
            offset_side: p.f(7),
            smooth: p.u(8),
            no_aim: p.b(9),
        },
        0xf6 => Effect::CameraZoom { percent: p.i(1), smooth: p.u(2) },
        0xfb => Effect::CameraShake { time: p.i(1), power: p.i(2), affect_all: p.b(3) },
        0x4c | 0xe9 => Effect::AutoDriveCam { on: false, movie: a.id == 0xe9, start_angle: 0.0, start_pitch: 0.0 },
        0x4d | 0xea => Effect::AutoDriveCam { on: true, movie: a.id == 0xea, start_angle: p.f(1), start_pitch: p.f(2) },
        0x67 | 0x68 => Effect::LetterBox { on: a.id == 0x68, arg1: p.i(1) },

        0x14 => Effect::Stream { channel: p.r(1), stream: p.r(2), start: p.i(3), arg4: p.b(4), alive_only: p.b(5), arg6: p.b(6) },
        0x86 => Effect::Sound { bank: p.r(1), name: p.r(2) },
        0xa4 => Effect::SoundAt { at: p.t(1), bank: p.r(2), name: p.r(3), arg4: p.i(4), near: p.f(5), far: p.f(6) },
        0xa6 => Effect::StopSoundAt { at: p.t(1) },
        0xf1 => Effect::AudioMix { channel: p.r(1), fade: p.i(2), percent: p.f(3) },
        0xf4 => Effect::RevertAudioMix { channel: p.r(1), fade: p.i(2) },
        0x10a => Effect::ResetAudioMix { fade: p.i(1) },
        0x110 => Effect::StingerChannel { channel: p.r(1) },
        0x111 => Effect::ReleaseStream,
        0x10e => Effect::PathSound { instance: p.r(1), loop_sound: p.r(2), start_sound: p.r(3), near: p.f(4), far: p.f(5) },
        0xff => Effect::CreakLevel { arg1: p.i(1), arg2: p.i(2) },

        0x4e => Effect::Movie { file: p.r(1), subtitles: p.r(2) },
        0xf0 => Effect::Transition { name: p.r(1) },
        0x70 => Effect::SimRate { rate: p.f(1) },
        0x103 => Effect::Subtitle { text: p.u(1), secs: p.f(2) },

        0x41 => Effect::AiCommand { command: p.r(1), target: p.r(2), element: p.r(3), arg4: p.i(4) },
        0x42 => Effect::AiActivate { element: p.r(1), other: p.r(2) },
        0x43 => Effect::AiDeactivate { element: p.r(1) },
        0x90 | 0x91 => Effect::AiFire { element: p.r(1), on: a.id == 0x91 },
        0x9f => Effect::AiAccuracy { element: p.r(1), accuracy: p.i(2) },
        0xa1 => Effect::AiKill { element: p.r(1), credit: p.b(2) },
        0xc7 => Effect::AiVehicleParams { element: p.r(1), value: p.f(2) },
        0x108 => Effect::AiWakeRange { element: p.r(1), range: p.f(2) },
        0x109 => Effect::TargetBeacon { element: p.r(1), on: p.b(2) },
        0xd1 => Effect::AiNextSpline { arg1: p.i(1), spline: p.r(2) },
        0xf5 => Effect::AiAvoidZone { element: p.r(1) },
        0x94 => Effect::SpawnSentry { instance: p.r(1), args: p.rest(2, 14) },
        0xb4 => Effect::DropMine { element: p.r(1), arg2: p.i(2) },
        0x2a | 0x2b => Effect::PlayerControl { cpu: a.id == 0x2a },
        0xca => Effect::ControlToSpline { arg1: p.i(1), spline: p.r(2) },
        0xef => Effect::EndControlToSpline { arg1: p.i(1) },
        0x6c => Effect::AutoDriveSpeed { factor: p.f(1) },

        0x1c => Effect::SpawnTraffic { at: trigger, arg1: p.f(1) },
        0x78 => Effect::MaxTraffic { max: p.i(1) },

        0x17 | 0x18 => Effect::InstanceVisible { instance: p.r(1), show: a.id == 0x18, static_ref: true },
        0x0b | 0x0c => Effect::InstanceVisible { instance: None, show: a.id == 0x0c, static_ref: false },
        0x23 => Effect::WakeSmackable { object: p.u(1), instance: p.r(2), placement: p.r(3) },
        0x24 => Effect::SystemAnim { part: p.i(1), anim: p.i(2), instance: p.r(3), placement: p.r(4) },
        0x09 => Effect::PlayEffect { effect: p.r(1), instance: p.r(2), placement: p.r(3) },
        0xad => Effect::SuppressEffect { effect: p.r(1), instance: p.r(2), placement: p.r(3), arg4: p.i(4) },
        0xfc | 0xfd => Effect::SwitchEffect { element: p.r(1), index: p.i(2), on: a.id == 0xfd },
        0x82 => Effect::CollisionGeometry { instance: p.r(1), arg2: p.i(2) },
        0x9a => Effect::Explosion { at: p.r(1), trigger: p.t(1), instance: p.r(2), args: p.rest(3, 7) },

        0xc0 => Effect::SpecialWeapon { weapon: p.i(1), trigger: p.t(2), arg3: p.f(3) },

        id => Effect::Other { id, words: p.rest(1, arg_words(id)) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn act(id: u32, words: &[u32]) -> TriggerAction {
        let mut args = id.to_le_bytes().to_vec();
        for w in words {
            args.extend_from_slice(&w.to_le_bytes());
        }
        TriggerAction { id, args }
    }

    #[test]
    fn camera_lock_on_takes_the_debug_print_order() {
        // [M 0x148e30 format 0x36edb0]: fElem, fTrigger, fWheelIndex, fHenchmenIndex, Up, Fw,
        // Side, fSmoothTransition, fNoAim. MIS01's words [D]: (AIEl 1280, 0, 0, 0, -0.35, -0.2,
        // 0, 200, 0).
        let mut map = MapScript::default();
        map.refs.insert(0x7372_0010, "CARP::<<Map>>::{AIEl}::1280".into());
        let a = act(0xf2, &[0x7372_0010, 0, 0, 0, (-0.35f32).to_bits(), (-0.2f32).to_bits(), 0, 200, 0]);
        assert_eq!(
            decode(&a, None, &map),
            Effect::CameraLockOn {
                element: Some(Ref { word: 0x7372_0010, path: "CARP::<<Map>>::{AIEl}::1280".into() }),
                trigger: None,
                wheel: 0,
                henchman: 0,
                offset_up: -0.35,
                offset_forward: -0.2,
                offset_side: 0.0,
                smooth: 200,
                no_aim: false,
            }
        );
    }

    #[test]
    fn sound_at_a_trigger_and_audio_mix() {
        // EPlaySoundPos: word 1 the position (a trigger), 2 the bank, 3 the name, 5 near, 6 far
        // [M 0x154798: +0x5c = word 5, +0x60 = word 6].
        let mut map = MapScript::default();
        map.triggers.push(nf_formats::trackstart::Trigger {
            index: 0,
            pos: [0.0; 3],
            radius: 1.0,
            kind: 0,
            shape: 2,
            flags: 0,
            height: 0.0,
            axis: [0.0; 3],
            width: 0.0,
            dir: [0.0; 3],
            length: 0.0,
            actions: vec![],
        });
        map.refs.insert(0x7372_0001, "CARP::<<Map>>::{Trgr}::0".into());
        map.refs.insert(0x7372_0002, "CHAR::snox/g3_snomb.bnk".into());
        map.refs.insert(0x7372_0003, "CHAR::SFX_dooropen".into());
        map.refs.insert(0x7372_0004, "CHAR::Ambience".into());
        let e = decode(&act(0xa4, &[0x7372_0001, 0x7372_0002, 0x7372_0003, 0, 10f32.to_bits(), 200f32.to_bits()]), None, &map);
        let Effect::SoundAt { at, name, near, far, .. } = e else { panic!("{e:?}") };
        assert_eq!((at, name.unwrap().text().to_string(), near, far), (Some(0), "SFX_dooropen".into(), 10.0, 200.0));
        let e = decode(&act(0xf1, &[0x7372_0004, 0x2d, (-100f32).to_bits()]), None, &map);
        assert_eq!(e, Effect::AudioMix { channel: Some(Ref { word: 0x7372_0004, path: "CHAR::Ambience".into() }), fade: 0x2d, percent: -100.0 });
    }

    #[test]
    fn traffic_takes_the_firing_trigger_and_the_cap_is_word_1() {
        let map = MapScript::default();
        assert_eq!(decode(&act(0x1c, &[5f32.to_bits()]), Some(7), &map), Effect::SpawnTraffic { at: Some(7), arg1: 5.0 });
        assert_eq!(decode(&act(0x78, &[3]), None, &map), Effect::MaxTraffic { max: 3 });
        assert_eq!(decode(&act(0x90, &[0]), None, &map), Effect::AiFire { element: None, on: false });
        assert_eq!(decode(&act(0x0c, &[]), None, &map), Effect::InstanceVisible { instance: None, show: true, static_ref: false });
    }
}
