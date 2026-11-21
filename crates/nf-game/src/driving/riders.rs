//! The seated rider gunners of the hostile vehicles (docs/systems/driving-ai.md §12.4).
//!
//! `FUN_0012cb08` seats each AIEl rider (health > 0) at its seat attribute in the car's `.atr`
//! and builds one AICharacter class per seat [M 0x12cb08:280-440]. This file ports, one
//! DRIVING.ELF function per item:
//!
//! - the seat → class table and the per-class constructor values (`FUN_001224b0` window,
//!   `FUN_00120b70` sunroof, `FUN_0011e490` snowmobile, `FUN_0011ff18` vehicle gun / driver,
//!   `FUN_0011aca8` Bond's seat) and the base `FUN_00118720` (range², clip, cadence);
//! - the skill scaling of the range (×4 / ×9 on +0x64) [M 0x12cb08:400-406];
//! - target pick and sight: `FUN_00122e20` (window), `FUN_0011f528` (snowmobile),
//!   `FUN_00121818` (sunroof), `FUN_00120478` (vehicle gun);
//! - the aim test shared by `FUN_00123058` / `FUN_0011f768` / `FUN_001219c0` / `FUN_00119290`
//!   (on target within 1/180 turn, +0x84 = which side);
//! - the reload test `FUN_00119270`;
//! - the fire cadence: `FUN_00122920` (window), `FUN_0011e9a0` (snowmobile), the fire arm of
//!   `FUN_00120fe0` (sunroof), with the window's acquire `FUN_00122780` and its per-tick
//!   `FUN_00122d28`.
//!
//! A shot here is the change to animation state 0xb with a picked fire animation (one draw of
//! the driving generator, `FUN_0017dc28(DAT_00339190)`). The round itself — its spawn point,
//! direction and spread — comes out of the character's animation layer (the event callbacks
//! handed to `FUN_0010b298`, e.g. 0x122358 for the window seat) and was not traced; the
//! helicopter gun's spread is ported in `heli.rs`.

/// The base AICharacter values `FUN_00118720` sets [M]: +0x64 the squared range (0x45afc800 =
/// 5625 = 75²), +0x7c shots / +0x80 clip 15, +0x8c ticks between shots 60, +0xac ready 1.
pub const BASE_RANGE_SQ: f32 = 5625.0;
pub const BASE_CLIP: i32 = 15;
pub const BASE_INTERVAL: i32 = 60;

/// One seat's class, as `FUN_0012cb08` builds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeatClass {
    /// The AIEl rider +0x00 seat code.
    pub seat: u32,
    /// The seat attribute read from the car's `.atr` (strings 0x36b178.. [D]).
    pub attr: &'static str,
    /// The class vtable and constructor.
    pub vtable: u32,
    pub ctor: u32,
    /// +0x78, the class's first AI state (the ctor's store).
    pub first_state: u32,
    /// The `FUN_00107c68(anim, a, b)` pair the seating plays.
    pub anim: (u32, u32),
    /// Whether the rider's weapon (+0x08 → the 0x327b90 name table) is passed to the class.
    pub armed: bool,
}

/// The seat switch of `FUN_0012cb08` [M 0x12cb08:303-385]. Any other seat code builds no rider.
/// The sunroof's +0x78 depends on its weapon (`sunroof_setup`); the value here is weapon 1's.
pub fn seat_class(seat: u32) -> Option<SeatClass> {
    Some(match seat {
        3 => SeatClass { seat, attr: "AIC_EWIND_POS", vtable: 0x369200, ctor: 0x1224b0, first_state: 0xd, anim: (0xd, 2), armed: true },
        4 => SeatClass { seat, attr: "AIC_ESUNR_POS", vtable: 0x368f08, ctor: 0x120b70, first_state: 0xb, anim: (0xb, 0x12), armed: true },
        0xe => SeatClass { seat, attr: "AIC_ESNOW_POS", vtable: 0x368950, ctor: 0x11e490, first_state: 8, anim: (8, 7), armed: true },
        0xf => SeatClass { seat, attr: "AIC_EVGUN_POS", vtable: 0x368c18, ctor: 0x11ff18, first_state: 1, anim: (1, 6), armed: false },
        0x10 => SeatClass { seat, attr: "AIC_EVDRI_POS", vtable: 0x368c18, ctor: 0x11ff18, first_state: 1, anim: (1, 1), armed: false },
        0x11 => SeatClass { seat, attr: "AIC_BOND_POS", vtable: 0x3681f0, ctor: 0x11aca8, first_state: 0, anim: (0, 8), armed: false },
        _ => return None,
    })
}

/// The rider's squared range after the skill scaling: skill 2 or 4 → ×4, 3 or 5 → ×9 (two
/// separate tests on the same +0x1c, so they never compound) [M 0x12cb08:400-406].
pub fn range_sq(skill: i32) -> f32 {
    let mut r = BASE_RANGE_SQ;
    if skill == 2 || skill == 4 {
        r *= 4.0;
    }
    if skill == 3 || skill == 5 {
        r *= 9.0;
    }
    r
}

/// The sunroof constructor `FUN_00120b70` [M]: the weapon index clamped to 1..2; weapon 1 →
/// clip +0x80 = 25, shots +0x7c = 25, +0x78 = 0xb; weapon 2 (`bazooka`) → clip 0 (no reload),
/// shots stay the base 15, +0x78 = 10. Returns (clip, shots, state).
pub fn sunroof_setup(weapon: i64) -> (i32, i32, u32) {
    let w = weapon.clamp(1, 2);
    if w == 1 { (0x19, 0x19, 0xb) } else { (0, BASE_CLIP, 10) }
}

/// `FUN_00119270` 0x119270: a reload is due when the class has a clip (+0x80 ≠ 0) and no shots
/// left (+0x7c < 1) [M].
pub fn needs_reload(clip: i32, shots: i32) -> bool {
    clip != 0 && shots < 1
}

/// The heading of `d` against the mount's forward row, in turns wrapped to [−0.5, 0.5]: `f(d.x,
/// d.z)/2π − f(fwd.x, fwd.z)/2π` with `f = FUN_002c0618`, read as `atan2` [M the 0.15915494
/// factor and the wrap; S that 0x2c0618 is atan2 with this argument order].
pub fn heading_turns(d: [f32; 3], fwd: [f32; 3]) -> f32 {
    let k = 0.159_154_94f32;
    let mut a = d[0].atan2(d[2]) * k - fwd[0].atan2(fwd[2]) * k;
    if a < -0.5 {
        a += 1.0;
    }
    if 0.5 < a {
        a -= 1.0;
    }
    a
}

/// The target every rider class picks [M 0x122e20 / 0x11f528 / 0x121818 / 0x123058 …]: the car's
/// controller (vtbl+0x5c) +0x1a4 target when it has one with +0x58 set (its position through
/// `FUN_0013e0b8`), else the player's position. The vehicle-gun sight `FUN_00120478` always uses
/// the player.
pub fn pick_target(controller_target: Option<[f32; 3]>, player: [f32; 3]) -> [f32; 3] {
    controller_target.unwrap_or(player)
}

fn dist_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// `FUN_00122e20` 0x122e20, the window seat's sight [M]: the mount (`FUN_00122238`: position
/// row 3, forward row 2) within the squared range of the target, and the target's heading
/// inside (−0.25, 0.3888889) turns — 90° to one side, 140° to the other.
pub fn window_sight(mount: [f32; 3], mount_fwd: [f32; 3], target: [f32; 3], range_sq: f32) -> bool {
    if !(dist_sq(target, mount) <= range_sq) {
        return false;
    }
    let a = heading_turns([target[0] - mount[0], 0.0, target[2] - mount[2]], mount_fwd);
    -0.25 < a && a < 0.388_888_9
}

/// `FUN_0011f528` 0x11f528, the snowmobile seat's sight [M]: as the window's with the window
/// (−1/3, 1/3); the heading is stored at +0x164 whenever the range holds. Returns (seen, the
/// stored heading).
pub fn snow_sight(mount: [f32; 3], mount_fwd: [f32; 3], target: [f32; 3], range_sq: f32) -> (bool, Option<f32>) {
    if !(dist_sq(target, mount) <= range_sq) {
        return (false, None);
    }
    let a = heading_turns([target[0] - mount[0], 0.0, target[2] - mount[2]], mount_fwd);
    (-0.333_333_34 < a && a < 0.333_333_34, Some(a))
}

/// `FUN_00121818` 0x121818, the sunroof seat's sight [M]: the character's own position (+0x30)
/// within the squared range, and neither of the car's other two riders (+0x170 / +0x174, filled
/// by the seating scan) has its animation word +0x30 at 1. `others_up` = those two words == 1.
pub fn sunroof_sight(own: [f32; 3], target: [f32; 3], range_sq: f32, others_up: [bool; 2]) -> bool {
    dist_sq(target, own) <= range_sq && !others_up[0] && !others_up[1]
}

/// `FUN_00120478` 0x120478, the vehicle gun's sight: the player within the squared range of the
/// character's position [M].
pub fn vgun_sight(own: [f32; 3], player: [f32; 3], range_sq: f32) -> bool {
    dist_sq(player, own) <= range_sq
}

/// The aim test of every armed class (`FUN_00123058` window, `FUN_0011f768` snowmobile,
/// `FUN_001219c0` sunroof, `FUN_00119290` vehicle gun) [M]: the heading of the target from the
/// gun bone (`FUN_00108ec8`) against the bone's forward row; on target when inside ±1/180 turn
/// (2°); +0x84 = heading > 0 (the sweep direction `FUN_00122a68` reads). Returns (on target,
/// +0x84).
pub fn aim_test(bone: [f32; 3], bone_fwd: [f32; 3], target: [f32; 3]) -> (bool, bool) {
    let a = heading_turns([target[0] - bone[0], 0.0, target[2] - bone[2]], bone_fwd);
    let w = 0.005_555_555_7f32;
    (-w < a && a < w, 0.0 < a)
}

/// One rider's fire state (the AICharacter fields the cadence reads).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gunner {
    /// +0x4c the AI state.
    pub state: u32,
    /// +0x7c shots left, +0x80 the clip.
    pub shots: i32,
    pub clip: i32,
    /// +0x8c ticks between shots, +0x90 the tick of the next allowed shot.
    pub interval: i32,
    pub next: i32,
    /// +0xac the ready word.
    pub ready: bool,
    /// +0x164: the window seat's aim hold (ticks), set to fps·300 on acquiring the target.
    pub hold: i32,
}

impl Gunner {
    /// The base values of `FUN_00118720` [M].
    pub fn base() -> Gunner {
        Gunner { state: 0, shots: BASE_CLIP, clip: BASE_CLIP, interval: BASE_INTERVAL, next: 0, ready: true, hold: 0 }
    }
}

/// What a cadence tick decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fire {
    /// Nothing changed this tick.
    Wait,
    /// State 0xb with the fire animation at this index of the class's table (the window's at
    /// 0x327488, 5 entries; the snowmobile's at 0x326eb0; the sunroof's at 0x326f98 + 6·weapon).
    Shot(u32),
    /// State 0xd: the gun is put away (the window/snowmobile stow animation).
    Stow,
    /// State 0xc: reload (sunroof).
    Reload,
}

/// `FUN_00122780` 0x122780, the window seat's acquire [M]: when the car is not disabled (its
/// vtbl+0x6c query) and the sight holds: state 2, +0x50 = 8, the aim hold = fps·300.
pub fn window_acquire(g: &mut Gunner, car_disabled: bool, sight: bool, fps: i32) -> bool {
    if !car_disabled && sight {
        g.state = 2;
        g.hold = fps * 300;
        return true;
    }
    false
}

/// `FUN_00122d28` 0x122d28, the window seat's per-tick bookkeeping [M]: a dead car (health ≤ 0)
/// moves a rider outside states 0xf/0x16 to 0xf (state 2 keeps); the aim hold counts down to 0.
pub fn window_tick(g: &mut Gunner, car_health: f32) {
    if !(0.0 < car_health || g.state == 0xf || g.state == 0x16) && g.state != 2 {
        g.state = 0xf;
        return;
    }
    if 0 < g.hold {
        g.hold -= 1;
    }
}

/// `FUN_00122920` 0x122920, the window seat's fire state [M]: with the car not disabled and
/// (the aim hold running or the sight holding): the sweep (`FUN_00122a68`, `sweep`), then when
/// ready and `now ≥ next`: state 0xb, shots − 1, next = now + interval, the animation
/// `rand·5 >> 16` (one draw). Otherwise state 0xd (stow). The window seat fires whether or not
/// the aim test holds, and never reloads (shots go negative) [M: no +0x124 / +0x11c call].
pub fn window_fire(g: &mut Gunner, now: i32, car_disabled: bool, sight: bool, sweep: &mut dyn FnMut(), rand: &mut dyn FnMut() -> u16) -> Fire {
    if car_disabled || (g.hold < 1 && !sight) {
        g.state = 0xd;
        return Fire::Stow;
    }
    sweep();
    if !g.ready || now < g.next {
        return Fire::Wait;
    }
    g.state = 0xb;
    g.shots -= 1;
    g.next = now + g.interval;
    Fire::Shot(((rand() as i32 * 5) >> 16) as u32)
}

/// `FUN_0011e9a0` 0x11e9a0, the snowmobile seat's fire state [M]: no sight → state 0xd (stow);
/// else when ready and `now ≥ next`: state 0xb, shots − 1, next = now + interval, one draw
/// `rand·5 >> 16`, the animation index forced to 0 unless the aim test holds.
pub fn snow_fire(g: &mut Gunner, now: i32, sight: bool, aim_on: bool, rand: &mut dyn FnMut() -> u16) -> Fire {
    if !sight {
        g.state = 0xd;
        return Fire::Stow;
    }
    if !g.ready || now < g.next {
        return Fire::Wait;
    }
    g.state = 0xb;
    g.shots -= 1;
    g.next = now + g.interval;
    let i = ((rand() as i32 * 5) >> 16) as u32;
    Fire::Shot(if aim_on { i } else { 0 })
}

/// The fire arm of `FUN_00120fe0` 0x120fe0, the sunroof seat [M]: with the car not disabled and
/// the sight holding and `FUN_00121c10` clear (`special` = its verdict, an animation branch not
/// ported): a due reload (`FUN_00119270`) refills the shots from the clip (state 0xc); else the
/// sweep, then when ready, the aim test holds and `now ≥ next`: state 0xb, shots − 1, next = now +
/// interval, the animation `(rand·6 >> 16) + 6·weapon` (one draw). The not-disabled/no-sight
/// exit (stow with the car's vtbl+0x74(0) and event 0x13/9) is reported as `Stow`.
#[allow(clippy::too_many_arguments)]
pub fn sunroof_fire(g: &mut Gunner, now: i32, car_disabled: bool, sight: bool, special: bool, aim_on: bool, weapon: u32, sweep: &mut dyn FnMut(), rand: &mut dyn FnMut() -> u16) -> Fire {
    if car_disabled || !sight {
        g.state = 0xd;
        return Fire::Stow;
    }
    if special {
        g.state = 0x10;
        return Fire::Wait;
    }
    if needs_reload(g.clip, g.shots) {
        g.shots = g.clip;
        g.state = 0xc;
        return Fire::Reload;
    }
    sweep();
    if !g.ready || !aim_on || now < g.next {
        return Fire::Wait;
    }
    g.state = 0xb;
    g.shots -= 1;
    g.next = now + g.interval;
    Fire::Shot(((rand() as i32 * 6) >> 16) as u32 + 6 * weapon)
}
