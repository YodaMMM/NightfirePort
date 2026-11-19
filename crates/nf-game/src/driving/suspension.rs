//! The per-wheel suspension spring–damper (DRIVING.ELF `sub_18FC00` 0x18fc00 and its kind
//! siblings) — docs/systems/drive-suspension.md. Decoded from the IDA database of
//! `work/elfs/DRIVING.ELF`: per wheel, a ground raycast (vtbl+532) against
//! `rest = axle[3] + SPRING_REST_LENGTH`, compression clamped by `SPRING_COMPRESSION_LIMIT`,
//! and the force `F = compression·STIFFNESS[axle] + Δcompression·DAMPING[axle]` — the axle
//! split by wheel index (0-1 front, 2-3 rear [M 0x18fdd4-0x18fdcc's `v39 < 2`]). The flat
//! `FlatWorld` makes the raycast analytic here; the general raycast consumer is the
//! mission-runner work.

/// The spring attrs of one car (the `.atr` values ride `CarParams`; the game's compiled-in
/// defaults are 1000/1000/1000/1000/2/[limit] [M drive-physics.md §2 — vanquish carries
/// 80/98 · 400/350 · 0.24 [D]]).
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
    /// vanquish.atr [D drive-physics.md §2's measured set].
    pub const VANQUISH: SusParams = SusParams {
        stiffness_front: 80.0,
        stiffness_rear: 98.0,
        damping_front: 400.0,
        damping_rear: 350.0,
        rest_length: 0.24,
        compression_limit: 2.0, // the game's compiled-in default [M]; no vanquish override on file
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

impl SusWheel {
    /// §2.2-§2.6 of the doc [M structure]: `rest_offset` is the axle descriptor's rest term
    /// (`axle[3]`), `ray_t` the ground query's return (the `[S]` semantic of drive-suspension.md
    /// §5 — `Some(distance)` where smaller = closer ground; `None` = the ray missed and the
    /// wheel hangs at full rest extension, the game's no-hit path at 0x18fd08). Returns the
    /// spring force when the wheel is in contact (the game's `scratch+0x30C`), `None` when the
    /// compression gate (`> 0`, plus the dead-zone skip at 0x18fd80) leaves the wheel unloaded.
    pub fn step(&mut self, rest_offset: f32, ray_t: Option<f32>, p: &SusParams, axle: Axle) -> Option<f32> {
        let Some(distance) = ray_t else {
            // The no-hit arm keeps the accumulator at the rest value (0x18fd08's max into
            // `scratch+804`) and records no contact.
            self.compression = 0.0;
            return None;
        };
        let rest = rest_offset + p.rest_length;
        // 0x18fd1c-0x18fd34: min against SPRING_COMPRESSION_LIMIT, floored at 0.
        let compression = ((rest - distance).min(p.compression_limit)).max(0.0);
        self.compression = compression;
        if compression <= 0.0 {
            return None;
        }
        // 0x18fdbc-0x18fde8: Δc against the previous tick's compression, then
        // `F = compression·STIFFNESS[axle] + Δc·DAMPING[axle]`.
        let (k, c) = match axle {
            Axle::Front => (p.stiffness_front, p.damping_front),
            Axle::Rear => (p.stiffness_rear, p.damping_rear),
        };
        Some(compression * k + (compression - self.prev) * c)
    }

    /// The tick's bookkeeping the game does after the force lands (the `prev` array walk at the
    /// loop tail, `a5 += 2` / `v108 += 4` at 0x1902d0-0x1902e8).
    pub fn end_tick(&mut self) {
        self.prev = self.compression;
    }
}

/// Watercraft / submersible pass (`DRIVING.ELF` `sub_190510`, `MIS11` `vanquishsub` & `MIS13C` `patrolboat`) [M]:
/// handbrake drag `HANDBRAKE_FORCE * (speed * 0.075 + 1.75)` when `speed >= 1.0`, else `2*speed + 1.0`.
pub fn watercraft_handbrake_drag(speed: f32, handbrake_force: f32, handbrake: bool) -> f32 {
    if !handbrake {
        0.0
    } else if speed < 1.0 {
        2.0 * speed + 1.0
    } else {
        handbrake_force * (speed * 0.075 + 1.75)
    }
}

/// Watercraft / submersible longitudinal thrust (`DRIVING.ELF` `sub_190510:183-184`) [M]:
/// `throttle * params+20 - brake * params+28`.
pub fn watercraft_longitudinal_thrust(throttle: f32, max_acc: f32, brake: f32, max_brake: f32) -> f32 {
    throttle * max_acc - brake * max_brake
}

/// Aerial / rotor wheelspin pass (`DRIVING.ELF` `sub_192498:96-102`) [M]:
/// `speed / (wheel_radius * 6.32 * dt)`, floored to `0.0` below `0.01`.
pub fn rotor_wheelspin(speed: f32, wheel_radius: f32, dt: f32) -> f32 {
    let w = if wheel_radius > 0.0 && dt > 0.0 {
        speed / (wheel_radius * 6.32 * dt)
    } else {
        speed
    };
    if w < 0.01 { 0.0 } else { w }
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
    fn watercraft_and_aerial_passes_match_decompile() {
        assert_eq!(watercraft_handbrake_drag(10.0, 0.1, false), 0.0);
        assert!((watercraft_handbrake_drag(0.5, 0.1, true) - 2.0).abs() < 1e-6);
        assert!((watercraft_handbrake_drag(10.0, 2.0, true) - 5.0).abs() < 1e-6);
        assert!((watercraft_longitudinal_thrust(1.0, 3.0, 0.5, 2.0) - 2.0).abs() < 1e-6);
        assert_eq!(rotor_wheelspin(0.0, 0.365, 1.0 / 60.0), 0.0);
        assert!(rotor_wheelspin(10.0, 0.365, 1.0 / 60.0) > 100.0);
    }
}
