//! The driving missions' scripted control and scripted camera: who drives the player car
//! (player, CPU, a spline, a forced stop), the path player that moves a car along a spline, the
//! scripted cameras, the letterbox bars, the cinematic flags and the sim rate
//! (docs/systems/driving-intros.md).
//!
//! [`IntroState::apply`] runs the [`Output::Effect`](super::script::Output::Effect) events
//! [`DriveScript`](super::script::DriveScript) reports, with their argument blocks; the state is
//! pure data, stepped by [`IntroState::step`] once per 60 Hz sim step. Evidence tags: [M] the
//! DRIVING.ELF decompile (EE addresses), [D] the eight tracks' data, [S] inferred and not
//! traced (each one is listed in the doc's last section).
//!
//! The race countdown is not a separate machine in the game: RACE's start list stops the car
//! (`EForceCarStop`), the trigger beside the start sets timer 5 to 5 s counting down, and four
//! stage-0 timer rules show the text ids 0x8ff, 0x8fe, 0x8fd as the timer passes 4, 3 and 2 s
//! and, below 1 s, show 0x227c, end the stop (`EEndCarStop`), send the rival its AI command and
//! start the race timer [D, rules 0/1/6/7]; [`DriveScript`](super::script::DriveScript) runs
//! those rules, this module turns the stop and the texts into state.

use nf_formats::drivepath::MapPaths;
use nf_formats::trackstart::{MapScript, TriggerAction};

/// Who the player car's driver obeys: the driver's `+0x70` (1 player, 2 CPU), written by
/// `FUN_00130f98` (`EControlToPlayer` 0x14ad18) and `FUN_00130e88` (`EControlToCPU` 0x14ac38)
/// [M]. The car is spawned as the player's [M 0x176658: type 0].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Player,
    Cpu,
}

/// The scripted camera the director runs (`DAT_00333ed8 + 4`).
#[derive(Clone, Debug, PartialEq)]
pub enum CameraMode {
    /// The normal chase camera (also what `EEndCameraAnim` returns to).
    Chase,
    /// `EPlayCameraAnim` 0x153700 → `FUN_001bc510`: a `Cams` placement.
    Anim { cam: usize, smooth: u32, latency: u32, flag: bool, fov: f32 },
    /// `EPlayCameraSpline` 0x1539a8 → `FUN_001bc890`: a camera on an `AISp` spline.
    Spline { spline: Option<usize>, smooth: u32, latency: u32, arg4: u32 },
}

/// `ELetterBoxOn` / `ELetterBoxOff` and the HUD's bar update `FUN_00245190` [M]: hud `+0x2b0`
/// state, `+0x2b4` counter (0..30).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LetterboxState {
    /// 0: fully in (counter 30).
    In,
    /// 1: no bars; the only state in which the HUD draws its gauges [M 0x23cf88].
    Off,
    /// 2: sliding in, counter +1 per update.
    SlidingIn,
    /// 3: sliding out, counter −1 per update.
    SlidingOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Letterbox {
    pub state: LetterboxState,
    pub counter: i32,
}

/// The bars' full count: `ELetterBoxOn(1)` and the slide's end both write 30 [M 0x244ea0 / 0x245190].
pub const LETTERBOX_FULL: i32 = 30;

impl Default for Letterbox {
    /// HUD init `FUN_0023c858`: state 1, counter 0 [M].
    fn default() -> Self { Letterbox { state: LetterboxState::Off, counter: 0 } }
}

impl Letterbox {
    /// `ELetterBoxOn` 0x150dc8 → `FUN_00244ea0(hud, instant)`: instant → state 0 with counter
    /// 30, else counter 0 and slide in [M].
    pub fn on(&mut self, instant: bool) {
        if instant {
            self.state = LetterboxState::In;
            self.counter = LETTERBOX_FULL;
        } else {
            self.counter = 0;
            self.state = LetterboxState::SlidingIn;
        }
    }
    /// `ELetterBoxOff` 0x150ce8 → `FUN_00244ec8(hud, instant)`: instant → state 1 (the counter
    /// is left as it is), else slide out from the current counter [M].
    pub fn off(&mut self, instant: bool) {
        self.state = if instant { LetterboxState::Off } else { LetterboxState::SlidingOut };
    }
    /// One `FUN_00245190` pass [M]: state 2 counts up and becomes 0 (counter 30) once above
    /// 29; state 3 counts down and becomes 1 (counter 0) below 1.
    pub fn tick(&mut self) {
        match self.state {
            LetterboxState::SlidingIn => {
                self.counter += 1;
                if self.counter > 0x1d {
                    self.state = LetterboxState::In;
                    self.counter = LETTERBOX_FULL;
                }
            }
            LetterboxState::SlidingOut => {
                self.counter -= 1;
                if self.counter < 1 {
                    self.counter = 0;
                    self.state = LetterboxState::Off;
                }
            }
            _ => {}
        }
    }
    /// Each bar's height in HUD pixels: `(int)(counter × 65 / 30)`, × 1.5 (truncated) when
    /// `DAT_00333760 + 0x58` is 1; `None` in state 1 (nothing drawn). The two bars are drawn at
    /// y 0 and at y = screen height − h, the HUD width wide, colour 0x7f000000 [M 0x245190].
    pub fn bar_height(&self, tall: bool) -> Option<f32> {
        if self.state == LetterboxState::Off {
            return None;
        }
        let mut h = ((self.counter as f32 * 65.0) / 30.0) as i32;
        if tall {
            h = (h as f32 * 1.5) as i32;
        }
        Some(h as f32)
    }
}

/// The path player (the class `FUN_001b1258` initialises and `FUN_001b14b8` updates; the
/// player driver's one is `FUN_0013e0b0(driver) + 0x60`, which `EPathSetDesiredThrottleStatic`
/// and `EPathSetTimeStatic` address through `FUN_00145d00(0, 0)`) [M]. Times are path units
/// (60 per second at the default clock scale, drivepath.rs).
#[derive(Clone, Debug, PartialEq)]
pub struct PathPlayer {
    /// `+0x98`: the `pt` instance playing, and its duration (`*anim`).
    pub anim: Option<(u16, f32)>,
    /// `+0x9c`: what plays after the end (`FUN_001b1258` sets it to the same animation, so a
    /// path wraps; `None` = stop at the end).
    pub next: Option<(u16, f32)>,
    /// `+0xa4`: the time rate.
    pub rate: f32,
    /// `+0xac`: the time.
    pub time: f32,
    /// `+0xbc` / `+0xc0` / `+0xc4`: speed, ramp step per update, target ("throttle").
    pub speed: f32,
    pub step: f32,
    pub target: f32,
    /// `+0xc8`: updates left before the ramp moves.
    pub delay: u32,
    /// `+0xcc`: playing.
    pub playing: bool,
}

impl Default for PathPlayer {
    /// `FUN_001b1258`'s values: speed, step and target 1.0, delay 0 [M].
    fn default() -> Self {
        PathPlayer { anim: None, next: None, rate: 1.0, time: 0.0, speed: 1.0, step: 1.0, target: 1.0, delay: 0, playing: false }
    }
}

impl PathPlayer {
    /// One `FUN_001b14b8` update over `dt` path units (no parent link; the `+0xa0` parent branch
    /// is not modelled) [M].
    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 || !self.playing {
            return;
        }
        let Some((_, dur)) = self.anim else { return };
        if self.delay == 0 {
            if self.speed != self.target {
                if self.speed < self.target {
                    self.speed = if self.step <= self.target - self.speed { self.speed + self.step } else { self.target };
                } else {
                    self.speed = if self.speed - self.target < self.step { self.target } else { self.speed - self.step };
                }
            } else if self.speed == 0.0 {
                self.playing = false;
            }
        } else {
            self.delay -= 1;
        }
        self.time += dt * self.speed * self.rate;
        let mut wrapped = false;
        if dur < self.time {
            wrapped = true;
            self.time -= dur;
        } else if self.time < 0.0 {
            wrapped = true;
        }
        if wrapped {
            match self.next {
                None => {
                    self.time = if self.time < 0.0 { 0.0 } else { dur };
                    self.speed = 0.0;
                    self.playing = false;
                }
                Some(n) => {
                    self.anim = Some(n);
                    if self.time < 0.0 {
                        self.time += n.1;
                    } else if n.1 < self.time {
                        self.time = n.1;
                    }
                }
            }
        }
    }
}

/// What the layer reports for the rest of the game.
#[derive(Clone, Debug, PartialEq)]
pub enum IntroOut {
    /// `EDisplayText` 0x14b8f8 → `FUN_00245890`: a HUD text id and its seconds [D: 0.5, 2, 5 …].
    Text { id: u32, secs: f32 },
    /// The cinematic flags went to 0 (`FUN_00206620`): programmer event 3 cleared, and a HUD
    /// fade that is up restarts over `fps / 2` ticks (`FUN_0023c728`) [M].
    CinematicEnd,
    /// `EControlToSpline` / `EEndControlToSpline` for an AI element (argument 1 non-zero).
    ElementSpline { element: u32, spline: Option<usize> },
}

/// The scripted control and camera state of one drive.
#[derive(Clone, Debug, PartialEq)]
pub struct IntroState {
    pub owner: Owner,
    /// The player car is on this spline (`EControlToSpline(0, spline)` until `EEndControlToSpline(0)`).
    pub spline: Option<usize>,
    pub path: PathPlayer,
    /// `EForceCarStop(a)`: the vehicle's vtable `+0x274` with 1 (a == 0) or 3 [M 0x14ede0];
    /// `EEndCarStop` calls `+0x27c` with 3 [M 0x14c048].
    pub stop: Option<u32>,
    /// `FUN_001bd198`: director `+0x354` → `+0x10` = 1 from `EControlToSpline(0)` and
    /// `EControlToCPU`, 0 from `EEndControlToSpline(0)` and `EControlToPlayer` [M]. Its reader
    /// is not in the decompile.
    pub autodrive_flag: bool,
    /// `EAutoDriveSpeedChange` 0x1482f8: driver `+0x134` [M].
    pub autodrive_speed: Option<f32>,
    /// Weapon manager `+0x350`, the auto-drive camera (POV) mode (driving-mis01-opening.md §3).
    pub pov: bool,
    pub pov_start_turns: Option<(f32, f32)>,
    pub camera: CameraMode,
    pub letterbox: Letterbox,
    /// Mission manager `+0x3cc` [M 0x2065d0 sets, 0x206620 clears]: bit 0 from
    /// `EPlayCameraAnim` / `EPlayCameraSpline`, bit 2 from `ELetterBoxOn`; `EEndCameraAnim`
    /// clears 3, `ELetterBoxOff` clears 7.
    pub cinematic: u32,
    /// `ESetSimRate` 0x158d28: `DAT_0032f2b0 + 0x24`, capped at 4.0 [M].
    pub sim_rate: f32,
    /// `EControlToPlayer` / `EStartMission` write `FUN_00167d80() + 4 = 1` [M 0x14ad18 / 0x15c738].
    pub control_word: bool,
    pub fps: u32,
    pub out: Vec<IntroOut>,
}

/// `FUN_00153700` / `FUN_0014bdb8` / `FUN_001539a8`: a frame count above 1 is rescaled to the
/// frame rate (`n × DAT_00339f4c / 60`, truncated); the smooth time otherwise becomes 1, the
/// latency stays [M].
pub fn rescale(n: u32, fps: u32, floor_one: bool) -> u32 {
    if n as f32 > 1.0 {
        ((n.wrapping_mul(fps)) as f32 / 60.0) as u32
    } else if floor_one {
        1
    } else {
        n
    }
}

/// `FUN_001bc510`: the camera's field value `+0xe8` is 33.0 unless the argument (an int) is
/// above 2 [M].
pub const CAMERA_DEFAULT_FOV: f32 = 33.0;

impl IntroState {
    pub fn new(fps: u32) -> IntroState {
        IntroState {
            owner: Owner::Player,
            spline: None,
            path: PathPlayer::default(),
            stop: None,
            autodrive_flag: false,
            autodrive_speed: None,
            pov: false,
            pov_start_turns: None,
            camera: CameraMode::Chase,
            letterbox: Letterbox::default(),
            cinematic: 0,
            sim_rate: 1.0,
            control_word: false,
            fps,
            out: Vec::new(),
        }
    }

    /// The player steers the car: driver owner 1, no spline, no forced stop, no POV mode.
    pub fn player_drives(&self) -> bool {
        self.owner == Owner::Player && self.spline.is_none() && self.stop.is_none() && !self.pov
    }

    fn clear_cinematic(&mut self, bits: u32) {
        self.cinematic &= !bits;
        // FUN_00206620 runs its "all clear" arm whenever the result is 0, even when it was 0.
        if self.cinematic == 0 {
            self.out.push(IntroOut::CinematicEnd);
        }
    }

    /// `EStartMission` [M 0x15c738]: the control word.
    pub fn start_mission(&mut self) { self.control_word = true; }

    /// Runs one event of the layer; other ids are ignored. `paths` resolves the spline and
    /// camera references.
    pub fn apply(&mut self, a: &TriggerAction, script: &MapScript, paths: &MapPaths) {
        let fps = self.fps;
        match a.id {
            // EControlToSpline 0x14ae60: only with a spline whose +0x40 animation resolved.
            0xca => {
                let s = paths.spline_ref(script, a.word(2));
                let anim = s.and_then(|s| paths.spline_anim(script, s));
                let Some(anim) = anim else { return };
                if a.word(1) == 0 {
                    self.spline = s;
                    // [S] the controller's vtable +0x24 (not decomposed) hands the spline's
                    // animation to the driver's path player; it starts at time 0 with the
                    // FUN_001b1258 defaults and wraps to itself. The time rate is the spline
                    // controller's: `FUN_0013da80` writes its path element's +0x18 = 0.5 and
                    // `FUN_001b1110` copies element +0x18 into the player's +0xa4 [M]; a reset
                    // (`FUN_0013dcc0` -> `FUN_001b1258`) leaves +0xa4 alone [M].
                    self.path = PathPlayer {
                        anim: Some((anim.instance, anim.duration)),
                        next: Some((anim.instance, anim.duration)),
                        rate: crate::driving::flight::CONTROLLER_RATE,
                        playing: true,
                        ..PathPlayer::default()
                    };
                    self.autodrive_flag = true;
                } else {
                    self.out.push(IntroOut::ElementSpline { element: a.word(1), spline: s });
                }
            }
            // ESetNextAISpline 0x158370 (word 1 = 0, the player; the spline needs its +0x40
            // animation) -> FUN_0013e058: with the controller in place FUN_0013dd28 ->
            // FUN_001b1838 sets the path player's next animation (+0x9c), played when this one
            // ends [M]. Without a controller the game builds one (FUN_0013da80); the port only
            // takes the case with the player already on a spline.
            0xd1 => {
                if a.word(1) == 0 && self.spline.is_some() {
                    if let Some(anim) = paths.spline_ref(script, a.word(2)).and_then(|s| paths.spline_anim(script, s)) {
                        self.path.next = Some((anim.instance, anim.duration));
                    }
                }
            }
            // EEndControlToSpline 0x14c110.
            0xef => {
                if a.word(1) == 0 {
                    self.spline = None;
                    self.path.playing = false;
                    self.autodrive_flag = false;
                } else {
                    self.out.push(IntroOut::ElementSpline { element: a.word(1), spline: None });
                }
            }
            0x2a => {
                self.owner = Owner::Cpu;
                self.autodrive_flag = true;
            }
            0x2b => {
                self.owner = Owner::Player;
                self.control_word = true;
                self.autodrive_flag = false;
            }
            0xdc => self.stop = Some(if a.word(1) == 0 { 1 } else { 3 }),
            // [S] that +0x27c(3) releases either stop mode.
            0xdb => self.stop = None,
            0x6c => self.autodrive_speed = Some(a.float(1)),
            // EPathSetDesiredThrottleStatic 0x152cb0 / EPathSetTimeStatic 0x153340 on the
            // player's path player (both selectors 0).
            0x74 => {
                if a.word(1) == 0 && a.word(2) == 0 {
                    self.path.target = a.float(3);
                }
            }
            0x99 => {
                if a.word(1) == 0 && a.word(2) == 0 {
                    if let Some((_, dur)) = self.path.anim {
                        self.path.time = dur * a.float(3);
                    }
                }
            }
            // ESetAutoDriveCamOn / ...Movie: only when the mode is off.
            0x4d | 0xea => {
                if !self.pov {
                    self.pov_start_turns = Some((a.float(1) / 360.0, a.float(2) / 360.0));
                    self.pov = true;
                }
            }
            0x4c | 0xe9 => self.pov = false,
            0x3f => {
                // FUN_001bc510 does nothing without a camera (MIS01 rule 33 passes 0 [D]); the
                // manager bit is set either way.
                let w5 = a.word(5) as i32 as f32;
                if let Some(cam) = paths.cam_ref(script, a.word(1)) {
                  self.camera = CameraMode::Anim {
                    cam,
                    smooth: rescale(a.word(2), fps, true),
                    latency: rescale(a.word(3), fps, false),
                    flag: a.word(4) != 0,
                    fov: if w5 > 2.0 { w5 } else { CAMERA_DEFAULT_FOV },
                  };
                }
                self.cinematic |= 1;
            }
            0xcc => {
                self.camera = CameraMode::Spline {
                    spline: paths.spline_ref(script, a.word(1)),
                    smooth: rescale(a.word(2), fps, true),
                    latency: rescale(a.word(3), fps, false),
                    arg4: a.word(4),
                };
                self.cinematic |= 1;
            }
            0x44 => {
                self.camera = CameraMode::Chase;
                self.clear_cinematic(3);
            }
            0x68 => {
                self.letterbox.on(a.word(1) != 0);
                self.cinematic |= 4;
            }
            0x67 => {
                self.letterbox.off(a.word(1) != 0);
                self.clear_cinematic(7);
            }
            0x70 => self.sim_rate = a.float(1).min(4.0),
            0x2c => self.out.push(IntroOut::Text { id: a.word(1), secs: a.float(2) }),
            _ => {}
        }
    }

    /// One 60 Hz sim step: the path player advances one path unit × the clock scale (taken as
    /// 1, [S]) and the letterbox runs one update (the HUD draws once per frame, [S]).
    pub fn step(&mut self) {
        self.path.update(1.0);
        self.letterbox.tick();
    }

    /// The car's position on its spline now (the path player's matrix row 3 is written into the
    /// car's matrix, [S]).
    pub fn spline_position(&self, paths: &MapPaths) -> Option<[f32; 3]> {
        let (inst, _) = self.path.anim?;
        self.spline?;
        paths.anims.get(&inst)?.position(self.path.time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letterbox_slides_in_over_thirty_updates_and_out_again() {
        let mut l = Letterbox::default();
        assert_eq!(l.bar_height(false), None);
        l.on(false);
        assert_eq!((l.state, l.counter), (LetterboxState::SlidingIn, 0));
        for _ in 0..29 {
            l.tick();
        }
        assert_eq!((l.state, l.counter), (LetterboxState::SlidingIn, 29));
        assert_eq!(l.bar_height(false), Some(62.0)); // (int)(29*65/30) = 62
        l.tick();
        assert_eq!((l.state, l.counter), (LetterboxState::In, 30));
        assert_eq!(l.bar_height(false), Some(65.0));
        assert_eq!(l.bar_height(true), Some(97.0)); // (int)(65*1.5)
        l.off(false);
        for _ in 0..29 {
            l.tick();
        }
        assert_eq!((l.state, l.counter), (LetterboxState::SlidingOut, 1));
        l.tick();
        assert_eq!((l.state, l.counter), (LetterboxState::Off, 0));
        // Instant on / off.
        l.on(true);
        assert_eq!((l.state, l.counter), (LetterboxState::In, 30));
        l.off(true);
        assert_eq!((l.state, l.counter), (LetterboxState::Off, 30));
    }

    #[test]
    fn camera_times_rescale_above_one_frame() {
        assert_eq!(rescale(0, 60, true), 1);
        assert_eq!(rescale(1, 60, true), 1);
        assert_eq!(rescale(0, 60, false), 0);
        assert_eq!(rescale(25, 60, true), 25);
        assert_eq!(rescale(25, 30, true), 12);
        assert_eq!(rescale(0x78, 30, false), 60);
    }

    #[test]
    fn path_player_ramps_to_the_throttle_and_wraps_or_stops() {
        let mut p = PathPlayer { anim: Some((1, 10.0)), next: None, playing: true, ..PathPlayer::default() };
        p.target = 0.25;
        p.step = 0.5;
        p.update(1.0); // 1.0 -> 0.5 (step 0.5), time 0.5
        assert_eq!((p.speed, p.time), (0.5, 0.5));
        p.update(1.0); // 0.5 -> 0.25 (within a step), time 0.75
        assert_eq!((p.speed, p.time), (0.25, 0.75));
        for _ in 0..40 {
            p.update(1.0);
        }
        // Past the end with no next: clamp at the end, stop.
        assert_eq!((p.time, p.speed, p.playing), (10.0, 0.0, false));
        let mut w = PathPlayer { anim: Some((1, 10.0)), next: Some((1, 10.0)), playing: true, ..PathPlayer::default() };
        for _ in 0..12 {
            w.update(1.0);
        }
        assert_eq!((w.time, w.playing), (2.0, true));
        // Throttle 0: the speed reaches 0, then the next update stops the player.
        let mut z = PathPlayer { anim: Some((1, 10.0)), playing: true, target: 0.0, ..PathPlayer::default() };
        z.update(1.0);
        assert_eq!((z.speed, z.time, z.playing), (0.0, 0.0, true));
        z.update(1.0);
        assert!(!z.playing);
    }

    fn act(id: u32, words: &[u32]) -> TriggerAction {
        let mut args = id.to_le_bytes().to_vec();
        for w in words {
            args.extend_from_slice(&w.to_le_bytes());
        }
        TriggerAction { id, args }
    }

    #[test]
    fn control_owner_stop_and_cinematic_bits() {
        let (m, p) = (MapScript::default(), MapPaths::default());
        let mut s = IntroState::new(60);
        assert!(s.player_drives());
        s.apply(&act(0xdc, &[1]), &m, &p);
        assert_eq!(s.stop, Some(3));
        assert!(!s.player_drives());
        s.apply(&act(0xdb, &[]), &m, &p);
        assert!(s.player_drives());
        s.apply(&act(0x2a, &[]), &m, &p);
        assert_eq!((s.owner, s.autodrive_flag), (Owner::Cpu, true));
        s.apply(&act(0x2b, &[]), &m, &p);
        assert_eq!((s.owner, s.autodrive_flag, s.control_word), (Owner::Player, false, true));
        // Camera anim (bit 0) + letterbox (bit 2); ending the camera leaves bit 2, so no end yet.
        s.apply(&act(0x3f, &[0, 1, 0, 1, 0]), &m, &p);
        s.apply(&act(0x68, &[1]), &m, &p);
        assert_eq!(s.cinematic, 5);
        s.apply(&act(0x44, &[0, 0, 0, 0]), &m, &p);
        assert_eq!((s.cinematic, s.camera.clone()), (4, CameraMode::Chase));
        assert!(s.out.is_empty());
        s.apply(&act(0x67, &[0]), &m, &p);
        assert_eq!(s.cinematic, 0);
        assert_eq!(s.out, vec![IntroOut::CinematicEnd]);
        // ESetSimRate caps at 4.
        s.apply(&act(0x70, &[9.0f32.to_bits()]), &m, &p);
        assert_eq!(s.sim_rate, 4.0);
        s.apply(&act(0x70, &[0.6f32.to_bits()]), &m, &p);
        assert_eq!(s.sim_rate, 0.6);
        // POV on only when off; the start angles in turns.
        s.apply(&act(0x4d, &[(-60.0f32).to_bits(), (-20.0f32).to_bits()]), &m, &p);
        s.apply(&act(0x4d, &[90.0f32.to_bits(), 0]), &m, &p);
        assert_eq!(s.pov_start_turns, Some((-60.0 / 360.0, -20.0 / 360.0)));
        s.apply(&act(0xe9, &[]), &m, &p);
        assert!(!s.pov);
    }
}
