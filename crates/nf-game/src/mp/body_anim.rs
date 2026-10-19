//! The third-person MP body's animation clocks (mp-body-anim): the game's per-body script list
//! as `PlayerAnimSetInit*` / `AnimSetUpdate` / `Player_ChangeSubState` / `AnimObjectUpdate` /
//! `AnimFrameResolve` build and tick it, kept per slot by the draw. **Draw-only** — never hashed,
//! never read by the sim; every input is state the lockstep already shares (the body's speeds,
//! stance, weapon, life and aim).
//!
//! The model is the game's own list of playing scripts ([M] throughout unless tagged):
//!
//! - **Set init** (`AnimSetInit` 0x1c6a48, reached through `PlayerAnimSetInit` 0x195b90 from
//!   `Player_SetWeaponAnim` 0x1a9338 on every weapon set, from `Player_ChangeSubState` 0x190630
//!   on a stance change, from `Player_SSWalk`/`SSCrouch` 0x192310/0x1924c0 when the transition
//!   hold ends, and from the respawn): every playing script is deleted (`AnimListBuild(obj, 1)` +
//!   `AnimListDelete`), the locomotion slot and the strafe direction reset to 0xff, and the walk
//!   distance `ANIMOBJ+0xdc` = 0.37 (`AnimSetAppend(0.37, 0.5, …)` from 0x18fcd8).
//! - **The locomotion pick** (`AnimSetUpdate` 0x1c0938, called each tick by SSWalk/SSCrouch with
//!   `BLData+0x18` forward and `+0x10` strafe): t = clamp(|fwd|, 0, 0.1)/0.1; slot 0 (idle) at
//!   t ≤ 0.0002, else slot 1 (walk with the run crossfaded at t). A slot change waits out a 4-tick
//!   debounce (`set+0x30`). A change into or out of idle fades every set script out over 8 ticks
//!   (`+0xac = +0xa8 = 8.0`, fade −1); the idle appends at full weight from frame 1 (a NON-looping
//!   150-frame script [D 060009dc et al.: header loop bit clear] — it holds its last frame); the walk
//!   appends with the distance clock (flags |= 0x8d000000, `+0xb4 = 1`) and the run synced to it
//!   (`+0xb4 = 2`). With a single set script left (`GlbAnimList == 1`) the walk distance resets to
//!   `set+0x24` = 0.37. The walk distance advances by `ANIMOBJ+0xd4 = fwd × 0.5` per tick, SIGNED —
//!   walking backwards runs the walk's distance table backwards (the walk plays in reverse).
//! - **The strafe overlay** (LAB_001c0dc8, MP only): dir 1 (strafe > 0.0002, positive = left) /
//!   dir 2 (< −0.0002) when |atan2(strafe/0.075, |fwd|/0.1)·2/π| ≥ 0.5; a dir change deletes the
//!   old overlay, forces the locomotion slot to 0xff (re-appended next tick) and appends the strafe
//!   word with `AnimScriptAppendBlend` (every other script fades out over 8 ticks) and flags
//!   |= 0x8d004000 (looping); each tick its speed `+0x98` = |ratio| and its weight `+0xa8` =
//!   |angle·2/π|. *[S]*: the weight is taken as written (the 0x4000 flag is read as excluding the
//!   overlay from `AnimFrameResolve`'s fade ramp, `AnimListBuild(obj, 0x6800, 0, 0, 2)`).
//! - **Stance change** (`Player_ChangeSubState` 0x190630 case 0/4, from `Player_HandleJump`
//!   0x1916b0 at the end of the tick): the new stance's set is built, then the transition script
//!   (`PlayerAnimStand2Crouch` 0x190198 / `PlayerAnimCrouch2Stand` 0x190068) replaces every script
//!   (`AnimScriptAdd`) at speed 4, and `BLData+0x960` = 60. SSWalk/SSCrouch skip `AnimSetUpdate`
//!   while it counts; it ends (and the set is rebuilt) when any script stopped
//!   (`AnimScriptIsStopped` 0x1c70d8) or the count reaches 1.
//! - **Death** (substates 0xd/0xe, `Player_ChangeSubState`): every script deleted, one script —
//!   0x6000151 (the water-level global is −9.9 from `Player_Init`; 0x60008e0 only when it is 0.0)
//!   at speed 1, non-looping (297 frames, holds the end). The respawn rebuilds the set.
//! - **Weights** (`AnimFrameResolve` 0x1c35f8): fade +1 → clock += 1 up to the blend length;
//!   fade −1 → clock −= 1, deleted at ≤ 0; weight = clock / length. The pose: the scripts in stamp
//!   order, `result = first`, then `result = blend(result, next, 1 − weight(previous))`
//!   (docs/systems/anim-engine.md).
//! - **Upper-body aim** (`AnimObjectAimAt` 0x1bfe58 from `psiBuildMatrixPalette` 0x111498 on
//!   bone 2): the two angles ease toward their targets with `Ang_RotateToBearing(cur, want, 0.2,
//!   π)` (the 0.2 is `AnimObjectSetAiming`'s rate, 0x1c6878 from `Player_StandAtNewPosition`
//!   0x190d88:122 for MP players); with the aim target gone they ease back to 0. *[S]*: the port
//!   eases the TARGET direction (the body-relative twist/pitch of the aim point) and solves the
//!   gun-to-target rotation at draw time; the game eases the solved rotation.

use crate::trig::{rotate_to_bearing, PI};
use nf_formats::mpanim::{
    crouch2stand_script, stand2crouch_script, MpAnimSets, DEATH_SCRIPT, FWD_MAX, STRAFE_MAX,
};

/// `AnimSetAppend(0x3ebd70a4, …)` from `PlayerAnimSetInitNormal`/`Crouch` — the walk distance at
/// set init and `set+0x24` (the idle reset) [M 0x18fcd8, 0x1c6a48].
pub const SET_PHASE_INIT: f32 = 0.37;
/// `AnimSetAppend(…, 0x3f000000)` — `set+0x28`, the distance per unit of forward speed [M].
pub const SET_STRIDE: f32 = 0.5;
/// The blend-out length `AnimSetUpdate` / `AnimScriptAppendBlend` write (`0x41000000`) [M].
pub const BLEND_OUT: f32 = 8.0;
/// `set+0x30`'s debounce after a slot change [M 0x1c0938].
pub const SLOT_DEBOUNCE: u8 = 4;
/// The transitions' speed (`script+0x98 = 0x40800000`) [M 0x190068/0x190198].
pub const TRANSITION_SPEED: f32 = 4.0;
/// `BLData+0x960 = (u8)FRAME_RATE` on a stance change [M 0x190630].
pub const STANCE_HOLD: u8 = 60;
/// `AnimObjectSetAiming(…, 0.2, …)` — the aim ease [M 0x190d88:122, 0x1c6878].
pub const AIM_RATE: f32 = 0.2;
/// The 2/π the strafe weight uses (`0.63661975`) [M].
const TWO_OVER_PI: f32 = 0.636_619_75;

/// What a layer plays.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerKind {
    /// The set's idle word (clock-driven, non-looping).
    Idle(u32),
    /// The set's walk word (distance-driven from `MpBodyAnim::phase`) with the run word synced
    /// to it at `run_weight` (`run+0x88`).
    Walk { walk: u32, run: u32, run_weight: f32 },
    /// The strafe overlay word (clock-driven, looping, speed |ratio|).
    Strafe(u32),
    /// A transition or the death script (clock-driven, non-looping).
    OneShot(u32),
}

/// One playing script (`sAnimScript`, the fields the body uses).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layer {
    pub kind: LayerKind,
    /// The creation stamp (`+0x84`), the blend order.
    pub stamp: u32,
    /// The script clock in frames from 1 (`+0x90`); unused for `Walk` (the distance table gives it).
    pub time: f32,
    /// Frames per tick (`+0x98`).
    pub speed: f32,
    /// The script's length in frames (`+0xb0`) and its loop flag (header bit or 0x80000000).
    pub length: f32,
    pub looping: bool,
    /// Finished (flag 0x10000000).
    pub stopped: bool,
    /// The fade (`+0xb6`: 1 in, −1 out) and the blend clock / length (`+0xa8` / `+0xac`).
    pub fade: i8,
    pub clock: f32,
    pub blend_len: f32,
    /// The set's tag (`+0x80` = the set's unique tag) — the scripts `AnimListBuild(obj, 0x200, tag)` counts.
    pub tagged: bool,
    /// The overlay's weight is written by `AnimSetUpdate` (flag 0x4000) — `Some(weight)`.
    pub manual: Option<f32>,
}

impl Layer {
    /// `+0x9c` = clock / length (or the overlay's written weight).
    pub fn weight(&self) -> f32 {
        match self.manual {
            Some(w) => w.clamp(0.0, 1.0),
            None if self.blend_len > 0.0 => (self.clock / self.blend_len).clamp(0.0, 1.0),
            None => 1.0,
        }
    }
}

/// One tick's input, from state every machine shares (draw-only use).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpBodyInput {
    /// The body is alive (not in the substate-0xd/0xe death).
    pub alive: bool,
    /// The held weapon (`ANIMOBJ+0x62`) and its anim-set class (`weapon_data[w]+0x84`, or 999 SP).
    pub weapon: u8,
    pub class: u32,
    /// Substate 4 (crouched).
    pub crouch: bool,
    /// `BLData+0x18` forward and `+0x10` strafe (positive = left), units per tick.
    pub fwd: f32,
    pub strafe: f32,
    /// The aim target as body-relative (twist, pitch) radians — twist positive toward the body's
    /// left (a larger yaw), pitch positive UP — or None (no target: ease back to 0).
    pub aim: Option<[f32; 2]>,
}

/// One body's script list and clocks.
#[derive(Clone, Debug, PartialEq)]
pub struct MpBodyAnim {
    /// The playing scripts in stamp order.
    pub layers: Vec<Layer>,
    /// `ANIMOBJ+0xdc`: the walk distance (unwrapped; the draw wraps it by the walk table's total).
    pub phase: f32,
    /// `ANIMOBJ+0xd4`: this tick's distance step.
    pub rate: f32,
    /// The smoothed aim (twist, pitch), radians, pitch positive up.
    pub aim: [f32; 2],
    /// The current stance (substate 4) and the transition hold (`BLData+0x960`).
    pub crouch: bool,
    pub hold: u8,
    /// In the death script.
    pub dead: bool,
    /// `set+0x2e` (−1 = 0xff), `set+0x30`, `set+0x2f`.
    slot: i8,
    debounce: u8,
    strafe_dir: u8,
    /// The set's walk / strafe scripts by stamp (`set+0x08` / `set+0x14`; 0 = none).
    set_walk: u32,
    set_strafe: u32,
    weapon: Option<u8>,
    class: u32,
    next_stamp: u32,
}

impl Default for MpBodyAnim {
    fn default() -> Self { Self::new() }
}

impl MpBodyAnim {
    pub fn new() -> MpBodyAnim {
        MpBodyAnim {
            layers: Vec::new(), phase: SET_PHASE_INIT, rate: 0.0, aim: [0.0; 2], crouch: false, hold: 0, dead: false,
            slot: -1, debounce: 0, strafe_dir: 0xff, set_walk: 0, set_strafe: 0, weapon: None, class: 0, next_stamp: 1,
        }
    }

    /// The locomotion slot (`set+0x2e`: 0 idle, 1 walk, −1 = 0xff unset).
    pub fn slot(&self) -> i8 { self.slot }
    /// The strafe direction (`set+0x2f`: 0 none, 1 left, 2 right, 0xff unset).
    pub fn strafe_dir(&self) -> u8 { self.strafe_dir }

    /// `AnimSetInit` 0x1c6a48 (every script deleted, the slot/dir reset, the distance 0.37).
    fn rebuild(&mut self) {
        self.layers.clear();
        self.slot = -1;
        self.strafe_dir = 0xff;
        self.debounce = 0;
        self.phase = SET_PHASE_INIT;
        self.rate = 0.0;
        self.set_walk = 0;
        self.set_strafe = 0;
    }

    /// `AnimScriptAppend` + `AnimScriptInit` 0x1c1600: time 1, speed 1, fade in, blend clock =
    /// length = 1.0 (full weight at once), a new stamp.
    fn append(&mut self, kind: LayerKind, speed: f32, lens: &dyn Fn(u32) -> Option<(u16, bool)>, force_loop: bool, tagged: bool) -> u32 {
        let id = match kind { LayerKind::Idle(s) | LayerKind::Strafe(s) | LayerKind::OneShot(s) => s, LayerKind::Walk { walk, .. } => walk };
        let (len, lp) = lens(id).unwrap_or((1, false));
        let stamp = self.next_stamp;
        self.next_stamp = self.next_stamp.wrapping_add(1).max(1);
        self.layers.push(Layer {
            kind, stamp, time: 1.0, speed, length: len.max(1) as f32, looping: lp || force_loop, stopped: false,
            fade: 1, clock: 1.0, blend_len: 1.0, tagged, manual: None,
        });
        stamp
    }

    /// Fade every script out over 8 ticks (`AnimSetUpdate`'s tagged fade-out with `only_tagged`;
    /// `AnimScriptAppendBlend` 0x1c17c0 for all) — flags |= 0x500000, fade −1, clock = length = 8.
    fn fade_out(&mut self, only_tagged: bool) {
        for l in self.layers.iter_mut().filter(|l| !only_tagged || l.tagged) {
            l.fade = -1;
            l.clock = BLEND_OUT;
            l.blend_len = BLEND_OUT;
        }
    }

    /// One tick, in the game's order: `Player_Update`'s substate handler (the hold, `AnimSetUpdate`,
    /// the stance change at its end), then `AnimObjectUpdate` (the script clocks) and
    /// `AnimFrameResolve` (the weights). `lens` gives a script's (length, header loop bit).
    pub fn tick(&mut self, sets: &MpAnimSets, lens: &dyn Fn(u32) -> Option<(u16, bool)>, inp: &MpBodyInput) {
        // The aim ease runs every tick (AnimObjectAimAt's Ang_RotateToBearing pair).
        let want = inp.aim.filter(|_| inp.alive).unwrap_or([0.0; 2]);
        for k in 0..2 { self.aim[k] = rotate_to_bearing(self.aim[k], want[k], AIM_RATE, PI); }
        if !inp.alive {
            if !self.dead {
                // Player_ChangeSubState(0xd/0xe): AnimSetDeleteAll, then AnimScriptAdd(the death script).
                self.dead = true;
                self.hold = 0;
                self.rebuild();
                self.append(LayerKind::OneShot(DEATH_SCRIPT), 1.0, lens, false, false);
            }
            self.script_tick(lens);
            self.resolve();
            return;
        }
        if self.dead || self.weapon.is_none() {
            // The respawn (MP_ReSpawn: re-equip → the set rebuild, substate 0) or the first tick.
            self.dead = false;
            self.hold = 0;
            self.crouch = inp.crouch;
            self.weapon = Some(inp.weapon);
            self.class = inp.class;
            self.rebuild();
        } else if self.weapon != Some(inp.weapon) {
            // Player_SetWeaponAnim 0x1a9338 → PlayerAnimSetInit on every weapon set.
            self.weapon = Some(inp.weapon);
            self.class = inp.class;
            if self.hold == 0 { self.rebuild(); }
        }
        // SSWalk / SSCrouch: the transition hold.
        if self.hold > 0 {
            self.hold -= 1;
            let stopped = self.layers.iter().any(|l| l.stopped);
            if stopped || self.hold == 1 {
                self.hold = 0;
                self.rebuild();
            }
        }
        if self.hold == 0 { self.set_update(sets, lens, inp.fwd, inp.strafe); }
        // Player_HandleJump at the end of SSWalk/SSCrouch → Player_ChangeSubState(0 / 4).
        if inp.crouch != self.crouch {
            self.crouch = inp.crouch;
            self.rebuild();
            let s = if self.crouch { stand2crouch_script(self.class) } else { crouch2stand_script(self.class) };
            self.append(LayerKind::OneShot(s), TRANSITION_SPEED, lens, false, false);
            self.hold = STANCE_HOLD;
        }
        self.script_tick(lens);
        self.resolve();
    }

    /// `AnimSetUpdate` 0x1c0938 for this stance's set.
    fn set_update(&mut self, sets: &MpAnimSets, lens: &dyn Fn(u32) -> Option<(u16, bool)>, fwd: f32, strafe: f32) {
        let set = sets.set(self.class, self.crouch);
        let p2 = if fwd.abs() <= 0.0002 { 0.0 } else { fwd };
        self.rate = p2 * SET_STRIDE;
        let t = p2.abs().min(FWD_MAX) / FWD_MAX;
        let (new_slot, blend) = if t <= 0.0002 { (0i8, 0.0) } else { (1i8, t) };
        if self.debounce > 0 { self.debounce -= 1; }
        if new_slot != self.slot && self.debounce == 0 {
            self.debounce = SLOT_DEBOUNCE;
            let old = self.slot;
            self.slot = new_slot;
            if old == 0 || new_slot == 0 { self.fade_out(true); }
            if new_slot == 0 {
                self.append(LayerKind::Idle(set.idle), 1.0, lens, false, true);
            } else {
                if old != 0 {
                    let w = self.set_walk;
                    self.layers.retain(|l| l.stamp != w);
                }
                // Walk (mode 1) + run (mode 2, synced): flags |= 0x8d000000 (looping).
                self.set_walk = self.append(LayerKind::Walk { walk: set.walk, run: set.run, run_weight: 0.0 }, 1.0, lens, true, true);
            }
        }
        if self.slot != 0 {
            let w = self.set_walk;
            if let Some(l) = self.layers.iter_mut().find(|l| l.stamp == w) {
                if let LayerKind::Walk { run_weight, .. } = &mut l.kind { *run_weight = blend; }
            }
        }
        // `GlbAnimList` over the set's tag: the walk layer is two scripts (walk + its synced run).
        let tagged: usize = self.layers.iter().filter(|l| l.tagged).map(|l| if matches!(l.kind, LayerKind::Walk { .. }) { 2 } else { 1 }).sum();
        if tagged == 1 { self.phase = SET_PHASE_INIT; }
        // The MP strafe overlay (LAB_001c0dc8).
        let ratio = strafe / STRAFE_MAX;
        let angle = ratio.atan2((p2 / FWD_MAX).abs());
        let weight = (angle * TWO_OVER_PI).abs();
        let dir = if (strafe > 0.0002 || strafe < -0.0002) && weight >= 0.5 { if strafe > 0.0 { 1 } else { 2 } } else { 0 };
        if dir != self.strafe_dir {
            self.strafe_dir = dir;
            self.slot = -1;
            let s = self.set_strafe;
            self.layers.retain(|l| l.stamp != s);
            self.set_strafe = 0;
            if dir != 0 {
                let script = if dir == 1 { set.strafe_left } else { set.strafe_right };
                self.fade_out(false);
                let st = self.append(LayerKind::Strafe(script), ratio.abs(), lens, true, false);
                if let Some(l) = self.layers.last_mut() { l.manual = Some(weight); }
                self.set_strafe = st;
            }
        }
        if dir != 0 {
            let s = self.set_strafe;
            if let Some(l) = self.layers.iter_mut().find(|l| l.stamp == s) {
                l.speed = ratio.abs();
                l.manual = Some(weight);
            }
        }
    }

    /// `AnimScriptTick` 0x1c2630 + `AnimScriptEnd` 0x1c1930 per playing script; the walk's
    /// distance clock (`AnimDistanceTableDistanceToFrame2` 0x1c4270 stores walked + step back).
    fn script_tick(&mut self, _lens: &dyn Fn(u32) -> Option<(u16, bool)>) {
        let mut walked = false;
        for l in self.layers.iter_mut() {
            if l.stopped { continue; }
            if let LayerKind::Walk { .. } = l.kind { walked = true; continue; }
            let mut t = l.time + l.speed;
            if t > l.length || t < 1.0 {
                if l.looping {
                    t = if l.speed < 0.0 { t + l.length } else { t - l.length };
                } else {
                    l.stopped = true;
                    t = if l.speed < 0.0 { 1.0 } else { l.length };
                }
                t = t.clamp(1.0, l.length);
            }
            l.time = t;
        }
        if walked { self.phase += self.rate; }
    }

    /// `AnimFrameResolve` 0x1c35f8's weight pass and the faded-out deletes.
    fn resolve(&mut self) {
        for l in self.layers.iter_mut() {
            if l.manual.is_some() { continue; }
            if l.fade > 0 { l.clock = (l.clock + 1.0).min(l.blend_len); }
            else if l.fade < 0 { l.clock -= 1.0; }
        }
        self.layers.retain(|l| !(l.fade < 0 && l.clock <= 0.0));
    }

    /// The blend factors for composing the layers' poses in stamp order: `out[0]` is unused
    /// (the first pose is the start), `out[i]` is `1 − weight(layer i−1)` — the factor the i-th
    /// pose is blended over the running result with (`AnimFrameResolve`).
    pub fn blend_factors(&self) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.layers.len());
        for i in 0..self.layers.len() {
            out.push(if i == 0 { 0.0 } else { 1.0 - self.layers[i - 1].weight() });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::mpanim::{MpAnimSet, ANIM_SET_COUNT};

    fn sets() -> MpAnimSets {
        let mut s = [MpAnimSet { idle: 0, walk: 0, run: 0, strafe_left: 0, strafe_right: 0 }; ANIM_SET_COUNT];
        for (i, x) in s.iter_mut().enumerate() {
            let b = 0x0600_0000 + (i as u32) * 0x10;
            *x = MpAnimSet { idle: b + 1, walk: b + 2, run: b + 3, strafe_left: b + 4, strafe_right: b + 5 };
        }
        MpAnimSets { sets: s }
    }
    fn lens(id: u32) -> Option<(u16, bool)> {
        if id == DEATH_SCRIPT { return Some((297, false)); }
        Some(match id & 0xf { 1 => (150, false), 2 => (86, false), 3 => (50, false), 4 | 5 => (43, false), _ => (50, false) })
    }
    fn inp(fwd: f32, strafe: f32) -> MpBodyInput {
        MpBodyInput { alive: true, weapon: 2, class: 1, crouch: false, fwd, strafe, aim: None }
    }

    #[test]
    fn idle_plays_from_frame_one_and_holds_its_end() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        // Tick 1 appends the idle; the same update's strafe pass sees dir 0 ≠ 0xff and forces the
        // slot to 0xff, so once the 4-tick debounce runs out (tick 5) the idle re-appends from
        // frame 1 over the first one fading out — the game's own first ticks after a set init.
        a.tick(&s, &lens, &inp(0.0, 0.0));
        assert_eq!(a.layers[0].time, 2.0, "appended at time 1, ticked once");
        for _ in 0..3 { a.tick(&s, &lens, &inp(0.0, 0.0)); }
        assert_eq!(a.layers.len(), 1);
        a.tick(&s, &lens, &inp(0.0, 0.0));
        assert_eq!(a.layers.len(), 2, "tick 5: the re-append over the fading first idle");
        assert_eq!(a.layers[1].time, 2.0);
        for _ in 0..20 { a.tick(&s, &lens, &inp(0.0, 0.0)); }
        assert_eq!(a.slot(), 0);
        assert_eq!(a.layers.len(), 1);
        assert!(matches!(a.layers[0].kind, LayerKind::Idle(_)));
        assert_eq!(a.layers[0].time, 22.0);
        for _ in 0..300 { a.tick(&s, &lens, &inp(0.0, 0.0)); }
        assert_eq!(a.layers[0].time, 150.0, "the non-looping idle holds its last frame");
        assert!(a.layers[0].stopped);
        assert_eq!(a.phase, SET_PHASE_INIT, "a lone set script resets the walk distance");
    }

    #[test]
    fn walk_distance_is_signed_and_the_slot_change_crossfades() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        for _ in 0..20 { a.tick(&s, &lens, &inp(0.0, 0.0)); }
        a.tick(&s, &lens, &inp(0.05, 0.0));
        assert_eq!(a.slot(), 1);
        // The idle fades out over 8 ticks under the new walk.
        assert_eq!(a.layers.len(), 2);
        let f = a.blend_factors();
        assert!((f[1] - (1.0 - 7.0 / 8.0)).abs() < 1e-6, "factor {f:?}");
        let p0 = a.phase;
        for _ in 0..20 { a.tick(&s, &lens, &inp(0.05, 0.0)); }
        assert!((a.phase - (p0 + 20.0 * 0.025)).abs() < 1e-4, "forward walks the distance up");
        assert_eq!(a.layers.len(), 1, "the idle faded out and was deleted");
        if let LayerKind::Walk { run_weight, .. } = a.layers[0].kind { assert!((run_weight - 0.5).abs() < 1e-6); } else { panic!() }
        let p1 = a.phase;
        for _ in 0..20 { a.tick(&s, &lens, &inp(-0.04, 0.0)); }
        assert!((a.phase - (p1 - 20.0 * 0.02)).abs() < 1e-4, "backwards walks the distance down: {} vs {}", a.phase, p1);
        assert!(matches!(a.layers.last().unwrap().kind, LayerKind::Walk { .. }), "backwards keeps the walk slot (|fwd| picks it)");
    }

    #[test]
    fn debounce_holds_a_quick_flip() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        a.tick(&s, &lens, &inp(0.0, 0.0));
        a.tick(&s, &lens, &inp(0.05, 0.0)); // idle → walk is blocked: the first append set debounce 4
        assert_ne!(a.slot(), 1);
        for _ in 0..3 { a.tick(&s, &lens, &inp(0.05, 0.0)); }
        assert_eq!(a.slot(), 1, "the change lands once the 4-tick debounce ran out");
    }

    #[test]
    fn strafe_overlay_appends_by_direction_and_weight() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        for _ in 0..20 { a.tick(&s, &lens, &inp(0.0, 0.0)); }
        a.tick(&s, &lens, &inp(0.0, 0.075));
        let st = a.layers.iter().find(|l| matches!(l.kind, LayerKind::Strafe(_))).expect("overlay");
        assert_eq!(st.kind, LayerKind::Strafe(s.set(1, false).strafe_left), "positive strafe = dir 1");
        assert_eq!(st.manual, Some(1.0));
        assert!(st.looping);
        assert_eq!(a.strafe_dir(), 1);
        for _ in 0..12 { a.tick(&s, &lens, &inp(0.0, -0.0375)); }
        let st = a.layers.iter().find(|l| matches!(l.kind, LayerKind::Strafe(_))).unwrap();
        assert_eq!(st.kind, LayerKind::Strafe(s.set(1, false).strafe_right));
        assert!((st.speed - 0.5).abs() < 1e-6, "speed = |ratio|");
        // Diagonal under 45° from forward: no overlay.
        for _ in 0..12 { a.tick(&s, &lens, &inp(0.1, 0.03)); }
        assert!(a.layers.iter().all(|l| !matches!(l.kind, LayerKind::Strafe(_))));
        assert_eq!(a.strafe_dir(), 0);
    }

    #[test]
    fn crouch_plays_the_transition_then_the_crouch_set() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        for _ in 0..20 { a.tick(&s, &lens, &inp(0.0, 0.0)); }
        let mut c = inp(0.0, 0.0);
        c.crouch = true;
        a.tick(&s, &lens, &c);
        assert_eq!(a.hold, STANCE_HOLD);
        assert_eq!(a.layers.len(), 1);
        assert_eq!(a.layers[0].kind, LayerKind::OneShot(stand2crouch_script(1)));
        assert_eq!(a.layers[0].time, 5.0, "speed 4");
        let mut n = 0;
        while a.hold > 0 { a.tick(&s, &lens, &c); n += 1; assert!(n < 61); }
        // 50 frames at 4/tick: stopped on the 13th script tick, the hold ends on the next tick.
        assert_eq!(n, 13);
        assert_eq!(a.layers.len(), 1);
        assert_eq!(a.layers[0].kind, LayerKind::Idle(s.set(1, true).idle), "the crouch set's idle");
        c.crouch = false;
        a.tick(&s, &lens, &c);
        assert_eq!(a.layers[0].kind, LayerKind::OneShot(crouch2stand_script(1)));
    }

    #[test]
    fn death_replaces_everything_and_the_respawn_rebuilds() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        for _ in 0..20 { a.tick(&s, &lens, &inp(0.08, 0.0)); }
        let mut d = inp(0.0, 0.0);
        d.alive = false;
        a.tick(&s, &lens, &d);
        assert!(a.dead);
        assert_eq!(a.layers.len(), 1);
        assert_eq!(a.layers[0].kind, LayerKind::OneShot(DEATH_SCRIPT));
        for _ in 0..100 { a.tick(&s, &lens, &d); }
        assert_eq!(a.layers[0].time, 102.0);
        for _ in 0..300 { a.tick(&s, &lens, &d); }
        assert_eq!(a.layers[0].time, 297.0, "the non-looping death holds its end");
        a.tick(&s, &lens, &inp(0.0, 0.0));
        assert!(!a.dead);
        assert!(matches!(a.layers[0].kind, LayerKind::Idle(_)));
    }

    #[test]
    fn weapon_change_rebuilds_the_set() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        for _ in 0..20 { a.tick(&s, &lens, &inp(0.0, 0.0)); }
        let mut w = inp(0.0, 0.0);
        w.weapon = 0x12;
        w.class = 2;
        a.tick(&s, &lens, &w);
        assert_eq!(a.layers.len(), 1);
        assert_eq!(a.layers[0].kind, LayerKind::Idle(s.set(2, false).idle));
        assert_eq!(a.layers[0].time, 2.0, "restarted");
    }

    #[test]
    fn aim_eases_toward_the_target_and_back() {
        let s = sets();
        let mut a = MpBodyAnim::new();
        let mut i = inp(0.0, 0.0);
        i.aim = Some([0.0, 0.5]);
        a.tick(&s, &lens, &i);
        assert!((a.aim[1] - 0.1).abs() < 1e-6, "0.2 of the way: {}", a.aim[1]);
        for _ in 0..60 { a.tick(&s, &lens, &i); }
        assert!((a.aim[1] - 0.5).abs() < 1e-3);
        i.aim = None;
        for _ in 0..60 { a.tick(&s, &lens, &i); }
        assert!(a.aim[1].abs() < 1e-3);
    }
}
