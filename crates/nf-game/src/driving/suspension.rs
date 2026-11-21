//! The per-wheel suspension spring-damper of DRIVING.ELF `FUN_0018fc00` (0x18fc00) —
//! docs/systems/drive-suspension.md. Its only caller is `FUN_00190510` [M call at :258],
//! which the controller `FUN_00188200` runs when `SUB_PHYSICS` is 0 and the vtbl+0x1d4
//! object's +0xc8 is non-zero [M 0x188bf8-0x188c30]; +0xc8 is `IS_SNOWMOBILE` in the pvehicle
//! schema [M FUN_001968f8 :265], set by supersnow / small_snowmobile [D]; the vtbl+0x1d4 getter
//! of the car class (vtable 0x379ef0, slot word 0x193170) is `FUN_00193170`, which returns
//! car+0x23c, the attr block [M]. `FUN_0018e548` repeats the same loop shape.
//!
//! Per wheel [M 0x18fc00 :51-74]: `x = wheel.w + SPRING_REST_LENGTH` (+0x60) is stored, the
//! ground query (vtbl+0x214, wheel index) runs, then `x = max(min(x, LIMIT (+0x64)), 0)`.
//! What the query writes into `x` is unproven: the port's `rest - distance` form is [S].
//! Contact = `x > 0 || prev > 0`, and the body's up-row y times the wheel vector's y must
//! exceed 0.2 (0x331970). Spring `F = x*STIFF + (x - prev)*DAMP`, wheels 0-1 front (+0x50 /
//! +0x58), 2-3 rear (+0x54 / +0x5c); `F = 0` when `x = 0` but the wheel is still in contact.

/// The spring attrs of one vehicle (pvehicle +0x50..+0x64).
#[derive(Clone, Copy, Debug)]
pub struct SusParams {
    pub stiffness_front: f32,
    pub stiffness_rear: f32,
    pub damping_front: f32,
    pub damping_rear: f32,
    pub rest_length: f32,
    pub compression_limit: f32,
}

impl SusParams {
    /// vanquish.atr [D]: SPRING_STIFFNESS 80/98, SPRING_DAMPING 400/350, REST 0.24.
    pub const VANQUISH: SusParams = SusParams {
        stiffness_front: 80.0,
        stiffness_rear: 98.0,
        damping_front: 400.0,
        damping_rear: 350.0,
        rest_length: 0.24,
        // vanquish.atr SPRING_COMPRESSION_LIMIT=0.75 [D MIS01.VIV]; the compiled-in default is
        // 2.0 [M FUN_001968f8 :168-172].
        compression_limit: 0.75,
    };

    /// The compiled-in pvehicle defaults [M FUN_001968f8 :149-172]: 1000 stiffness/damping
    /// front and rear, rest length 2.0, compression limit 2.0.
    pub const GAME_DEFAULTS: SusParams = SusParams {
        stiffness_front: 1000.0,
        stiffness_rear: 1000.0,
        damping_front: 1000.0,
        damping_rear: 1000.0,
        rest_length: 2.0,
        compression_limit: 2.0,
    };
}

/// One wheel's suspension state: this tick's compression and the previous tick's (the Δc term;
/// the game keeps the per-wheel array at `instance+528` [M 0x18fdbc]).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SusWheel {
    pub compression: f32,
    pub prev: f32,
}

/// Which pair of wheels: 0-1 front, 2-3 rear [M the `v39 < 2` axle split at 0x18fdd4].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axle {
    Front,
    Rear,
}

pub fn axle_of(wheel: usize) -> Axle {
    if wheel < 2 { Axle::Front } else { Axle::Rear }
}

/// The contact gate's attitude threshold [M DAT_00331970 = 0.2, read at 0x18fc00 :74]: a wheel
/// is live only when (body up-row y) * (wheel vector y) exceeds it.
pub const UPRIGHT_MIN: f32 = 0.2;

/// `DAT_00331970 < up_y * wheel_y` [M 0x18fc00 :74].
pub fn upright(up_y: f32, wheel_y: f32) -> bool {
    UPRIGHT_MIN < up_y * wheel_y
}

impl SusWheel {
    /// One wheel's spring [M 0x18fc00 :51-99]. `rest_offset` is the wheel vector's w,
    /// `ray_t` the ground query as a distance [S form]. Returns the spring force while the
    /// wheel is in contact (`x > 0` or last tick's `x > 0`): `x*k + (x - prev)*c` when
    /// `x > 0`, 0.0 when only the previous tick was compressed (the game zeroes +0x30c and
    /// still runs the wheel's tyre forces). None = no contact. The `upright` gate is the
    /// caller's (see `upright`). `None` for `ray_t` is the port's no-ground case: x = 0 [S].
    pub fn step(&mut self, rest_offset: f32, ray_t: Option<f32>, p: &SusParams, axle: Axle) -> Option<f32> {
        let x = match ray_t {
            Some(distance) => {
                let rest = rest_offset + p.rest_length;
                // [M :64-70]: min against SPRING_COMPRESSION_LIMIT, then floored at 0.
                (rest - distance).min(p.compression_limit).max(0.0)
            }
            None => 0.0,
        };
        self.compression = x;
        if !(x > 0.0 || self.prev > 0.0) {
            return None;
        }
        if x <= 0.0 {
            return Some(0.0);
        }
        // [M :78-87]: wheels 0-1 read +0x50/+0x58, 2-3 read +0x54/+0x5c.
        let (k, c) = match axle {
            Axle::Front => (p.stiffness_front, p.damping_front),
            Axle::Rear => (p.stiffness_rear, p.damping_rear),
        };
        Some(x * k + (x - self.prev) * c)
    }

    /// The loop tail stores this tick's x as next tick's prev for every wheel [M :345].
    pub fn end_tick(&mut self) {
        self.prev = self.compression;
    }
}

/// The handbrake term of `FUN_00190510` [M :168-179] (the pass `FUN_0018ecb8` :240 repeats
/// it): with the handbrake byte (+0x259) set, `2*speed + 1` below speed 1.0, else
/// `HANDBRAKE_FORCE * (speed*0.075 + 1.75)`; 0 when the byte is clear. `speed` is the
/// vehicle's +0x288. The function name predates this review: `FUN_00190510` is the
/// snowmobile-flag pass above, not a watercraft pass (subs run `FUN_00190f88`).
pub fn watercraft_handbrake_drag(speed: f32, handbrake_force: f32, handbrake: bool) -> f32 {
    if !handbrake {
        0.0
    } else if speed < 1.0 {
        2.0 * speed + 1.0
    } else {
        handbrake_force * (speed * 0.075 + 1.75)
    }
}

/// `FUN_00190510` :181-183 [M]: `+0x250 * MAXACC (+0x14) - +0x254 * MAXBRAKE (+0x1c)`. That
/// +0x250 / +0x254 are throttle / brake is unproven (the handbrake path zeroes both, :156-157).
pub fn watercraft_longitudinal_thrust(throttle: f32, max_acc: f32, brake: f32, max_brake: f32) -> f32 {
    throttle * max_acc - brake * max_brake
}

/// The wheel-roll step shared by `FUN_00192498` :126-142, `FUN_0018cef8` :389 and
/// `FUN_0018ecb8` :246 [M]: `speed / (WHEEL_RADIUS * 6.32 * fps)` with `fps` the integer frame
/// rate at 0x339f4c (`(float)DAT_00339f4c`), i.e. wheel turns per frame (6.32 is close to
/// 2*pi); below 0.01 it is 0, and it is capped at 1.0. With WHEEL_RADIUS <= 0 the speed
/// passes through unscaled. `dt` is the port's step; fps = 1/dt.
pub fn rotor_wheelspin(speed: f32, wheel_radius: f32, dt: f32) -> f32 {
    let fps = if dt > 0.0 { (1.0 / dt).round() } else { 0.0 };
    let w = if wheel_radius > 0.0 && fps > 0.0 { speed / (wheel_radius * 6.32 * fps) } else { speed };
    let w = if w < 0.01 { 0.0 } else { w };
    w.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_axle_split_is_wheels_01_front_23_rear() {
        assert_eq!(axle_of(0), Axle::Front);
        assert_eq!(axle_of(1), Axle::Front);
        assert_eq!(axle_of(2), Axle::Rear);
        assert_eq!(axle_of(3), Axle::Rear);
        // Sister: the boundary between the pairs is what the game's `v39 < 2` draws.
        assert_ne!(axle_of(1), axle_of(2));
    }

    #[test]
    fn the_spring_force_is_kx_plus_c_delta_x_with_vanquish_values() {
        let p = SusParams::VANQUISH;
        let mut w = SusWheel::default();
        // First contact: compression 0.1 (rest 0.34 above ground), no history — Δc = 0.1.
        let f = w.step(0.10, Some(0.24), &p, Axle::Front).expect("front wheel in contact");
        assert!((f - (0.10 * 80.0 + 0.10 * 400.0)).abs() < 1e-5);
        // The same wheel settles: second tick at equal compression — the damping term vanishes.
        w.end_tick();
        let f2 = w.step(0.10, Some(0.24), &p, Axle::Front).expect("still in contact");
        assert!((f2 - 0.10 * 80.0).abs() < 1e-5);
        // Sister: at steady state (damping gone) the rear axle's stiffer spring answers more —
        // the k split by `v39 < 2` is what differs (first-contact forces cross because front
        // damping 400 > rear 350).
        let mut r = SusWheel::default();
        r.step(0.10, Some(0.24), &p, Axle::Rear).unwrap();
        r.end_tick();
        let fr = r.step(0.10, Some(0.24), &p, Axle::Rear).unwrap();
        assert!(fr > f2, "rear stiffness 98 > front 80");
        assert_ne!(f, fr);
    }

    #[test]
    fn compression_clamps_at_the_limit_and_floors_at_zero() {
        let mut p = SusParams::VANQUISH;
        p.compression_limit = 0.5;
        let mut w = SusWheel::default();
        // A deep hit (rest far below ground): the compression saturates at the limit.
        let f = w.step(5.0, Some(0.24), &p, Axle::Front).expect("saturated contact");
        assert!((f - (0.5 * p.stiffness_front + 0.5 * p.damping_front)).abs() < 1e-4);
        assert_eq!(w.compression, 0.5);
        // Sister: one ulp past the limit would exceed it — the min bites (an unclamped step
        // would return 5.0·k here, not 0.5·k).
        assert!(f < 5.0 * p.stiffness_front);
        // A hit beyond the rest extension: no compression, no contact.
        let mut idle = SusWheel::default();
        assert!(idle.step(-1.0, Some(0.24), &p, Axle::Front).is_none());
        assert_eq!(idle.compression, 0.0);
        // And the ray missing entirely hangs the wheel unloaded (the game's no-hit arm).
        let mut air = SusWheel::default();
        assert!(air.step(0.10, None, &p, Axle::Front).is_none());
    }

    #[test]
    fn the_delta_term_fires_on_compression_changes_only() {
        let p = SusParams::VANQUISH;
        let mut w = SusWheel::default();
        w.step(0.20, Some(0.24), &p, Axle::Front).unwrap();
        w.end_tick();
        // Deeper this tick: Δc > 0 adds damping on top of the spring.
        let deeper = w.step(0.30, Some(0.24), &p, Axle::Front).unwrap();
        w.end_tick();
        let steady = w.step(0.30, Some(0.24), &p, Axle::Front).unwrap();
        assert!(deeper > steady, "Δc contributes only on the change");
        // Sister: a tick with no change reproduces the steady force exactly.
        w.end_tick();
        let again = w.step(0.30, Some(0.24), &p, Axle::Front).unwrap();
        assert!((again - steady).abs() < 1e-6);
    }

    #[test]
    fn vanquish_compression_limit_is_the_atr_value_and_defaults_are_the_schema() {
        // [D vanquish.atr SPRING_COMPRESSION_LIMIT=0.75] — the PR kept the 2.0 default.
        assert_eq!(SusParams::VANQUISH.compression_limit, 0.75);
        let mut w = SusWheel::default();
        let f = w.step(5.0, Some(0.24), &SusParams::VANQUISH, Axle::Front).unwrap();
        assert!((f - (0.75 * 80.0 + 0.75 * 400.0)).abs() < 1e-4, "{f}");
        let d = SusParams::GAME_DEFAULTS;
        assert_eq!((d.stiffness_front, d.damping_rear, d.rest_length, d.compression_limit), (1000.0, 1000.0, 2.0, 2.0));
    }

    #[test]
    fn a_wheel_that_just_lifted_stays_in_contact_with_zero_spring_force() {
        // [M 0x18fc00 :73, :99]: contact = x > 0 || prev > 0; x == 0 -> F (+0x30c) = 0.
        let p = SusParams::VANQUISH;
        let mut w = SusWheel::default();
        w.step(0.10, Some(0.24), &p, Axle::Rear).unwrap();
        w.end_tick();
        assert_eq!(w.step(-1.0, Some(0.24), &p, Axle::Rear), Some(0.0), "prev > 0 keeps contact");
        w.end_tick();
        assert_eq!(w.step(-1.0, Some(0.24), &p, Axle::Rear), None, "two ticks off the ground: no contact");
    }

    #[test]
    fn the_upright_gate_is_0_2() {
        // [M DAT_00331970 = 0x3e4ccccd]: strict `0.2 < up_y * wheel_y`.
        assert!(upright(1.0, 0.3));
        assert!(!upright(1.0, 0.2));
        assert!(!upright(-1.0, 1.0), "an upside-down body loses every wheel");
    }

    #[test]
    fn wheel_roll_is_turns_per_frame_at_60_and_caps_at_one() {
        // [M 0x18cef8 :389 / 0x192498 :128]: speed / (r * 6.32 * fps). 10 m/s on the vanquish
        // wheel: 10 / (0.365 * 6.32 * 60) = 0.07225. The PR multiplied by fps (dt in the divisor).
        let w = rotor_wheelspin(10.0, 0.365, 1.0 / 60.0);
        assert!((w - 10.0 / (0.365 * 6.32 * 60.0)).abs() < 1e-6, "{w}");
        assert_eq!(rotor_wheelspin(1000.0, 0.365, 1.0 / 60.0), 1.0, "capped at one turn a frame [M :138]");
        assert_eq!(rotor_wheelspin(0.1, 0.365, 1.0 / 60.0), 0.0, "below 0.01 -> 0 [M :134]");
    }

    #[test]
    fn watercraft_and_aerial_passes_match_decompile() {
        assert_eq!(watercraft_handbrake_drag(10.0, 0.1, false), 0.0);
        assert!((watercraft_handbrake_drag(0.5, 0.1, true) - 2.0).abs() < 1e-6);
        assert!((watercraft_handbrake_drag(10.0, 2.0, true) - 5.0).abs() < 1e-6);
        assert!((watercraft_longitudinal_thrust(1.0, 3.0, 0.5, 2.0) - 2.0).abs() < 1e-6);
        assert_eq!(rotor_wheelspin(0.0, 0.365, 1.0 / 60.0), 0.0);
    }
}
