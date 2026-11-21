//! The player car's recovery: the engine's own righting reset and the scripted placements
//! (docs/systems/drive-physics.md §10). Evidence: DRIVING.ELF, [M] code, [D] disc data, [S] the
//! port's reading where the code is hidden or not traced.
//!
//! What the engine does on its own [M FUN_00188200 0x188480-0x1884cc, 0x188bb0-0x188be4]: for
//! the player (actor `+0x2c8 == 0`) every controller tick reads the body's up row against
//! world up (`FUN_001f7c00`: `row1 · (0, 1, 0)`, the vector at 0x396568). Below 0.3 with any
//! hull point on the ground (instance `+0x6c`, `FUN_001f9500`) a counter (actor `+0x3e8`)
//! climbs to 30; otherwise it drops to 0 and the remembered heading (actor `+0x3f0`) moves a
//! tenth of the way to the body's forward row. At exactly 30 the car's reset method (vtable
//! `+0x158`, `FUN_00187b00`) rights it where it lies.
//!
//! What the engine does NOT do there: no fade, no damage, no mission fail, no respawn at a road
//! node or checkpoint, no height or bounding-box kill test [M-negative: `FUN_00187b00` and the
//! `+0x3e8` writers in `FUN_00188200` are the whole path; a grep of the 0x18xxxx-0x20xxxx
//! functions for `+0x14` compares against constants found only orientation tests — a narrow
//! search, not a proof that no kill plane exists elsewhere]. Water, river and out-of-area recoveries
//! are the tracks' own scripts: trigger zones whose event lists hold `EResetPlayerCarPos`
//! (kind 0x8c), handled by [`place_at`] [D: MIS01 has nine such triggers of radius 18-110 at
//! y −2.6..−16.2, all with speed factor 10.0]; `EPlayerLose` zones end the mission
//! (`script::Output::Lost`). The script event `EResetPlayerCar` (kind 98, `FUN_00156390`) calls
//! the same `+0x158` reset with argument 0 [M]; no shipped land track was checked for it here.

use super::physics::{Car, Ground};
use nf_formats::crp_mesh::CollisionTri;

/// `FUN_001f7c00(inst) < 0.3`: the up row's world-y below which the car counts as tipped [M 0x188480].
pub const TIP_UP_Y: f32 = 0.3;
/// The counter value that fires the reset: `actor+0x3e8 == 0x1e` [M 0x188bb8]; the counter
/// stops climbing there (`< 0x1e` guard, 0x1884a0).
pub const TIP_TICKS: i32 = 0x1e;
/// The heading memory's blend per upright tick: `h += (fwd − h) · 0.1` (0x3dcccccd) [M 0x1884ac].
pub const HEADING_BLEND: f32 = 0.1;
/// `FUN_00187eb8`'s initial spring compression: SPRING_COMPRESSION_LIMIT (attr +0x64) × 0.15 [M].
pub const PLACE_COMPRESSION: f32 = 0.15;
/// `FUN_00187eb8` with no ground under the point: the given y plus this [M `puStack_cc[1] + 1.0`].
pub const PLACE_NO_GROUND_LIFT: f32 = 1.0;

/// The per-car recovery state: actor `+0x3e8` and `+0x3f0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResetState {
    /// actor+0x3e8: consecutive ticks tipped and touching the ground.
    pub tip_ticks: i32,
    /// actor+0x3f0: the remembered heading (a lagged copy of the forward row). Its value at
    /// spawn is not traced; the port starts it at the spawn forward row [S].
    pub heading: [f32; 3],
}

impl ResetState {
    pub fn new(car: &Car) -> ResetState { ResetState { tip_ticks: 0, heading: car.rot[2] } }
}

/// Why the car was placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
    /// The engine's righting reset (`FUN_00187b00`).
    Righted,
    /// A script placement (`EResetPlayerCarPos` / `EStartMission` → `FUN_00187eb8`).
    Placed,
}

/// Where and how to put the car back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Respawn {
    pub cause: Cause,
    pub pos: [f32; 3],
    /// The orientation rows (right, up, forward).
    pub rot: [[f32; 3]; 3],
    pub vel: [f32; 3],
    /// actor+0x210.. (the wheels' last compression).
    pub spring_prev: [f32; 4],
}

impl Respawn {
    /// Puts the car there: `FUN_001f7880` (position, matrix, zero velocity and spin) and the
    /// state reset `FUN_00187350` (gear 1, last gear 0xff, shift inhibit 0, brake counter 0,
    /// wheel spin 0, spring memory 0) [M]. The wheels' cached planes are dropped [S: the
    /// `FUN_00187350` loop over actor+0x90 has its body hidden by the decompile].
    pub fn apply(&self, car: &mut Car) {
        car.place(self.pos, self.rot);
        car.vel = self.vel;
        car.speed = len(self.vel);
        car.spring_prev = self.spring_prev;
        car.gear = 1;
        car.last_gear = 0xff;
        car.shift_inhibit = 0;
        car.brake_ticks = 0;
        car.wheel_spin = 0.0;
        car.wheel_cache = [None; 4];
        car.contacts = 0;
        car.grounded = false;
    }
}

/// The per-tick check, run after the car's step: the `+0x3e8` counter and the `+0x3f0`
/// heading of `FUN_00188200` for the player [M 0x188480-0x1884cc], then the reset at 30
/// [M 0x188bb0]. `target` is the objective marker's point when one is set (`FUN_00207748(mgr,
/// 0)`: mgr+0x600 mode 1 = a point, 3 = an AI element's +0x30) — the reset then faces it.
/// Returns the placement when the reset fires; the state's counter is back at 0 then
/// (`FUN_00187350` zeroes +0x3e8 [M]).
pub fn check(car: &Car, tris: &[CollisionTri], state: &mut ResetState, target: Option<[f32; 3]>) -> Option<Respawn> {
    let up_y = car.rot[1][1];
    if up_y < TIP_UP_Y && car.hull_contacts != 0 {
        if state.tip_ticks < TIP_TICKS {
            state.tip_ticks += 1;
        }
    } else {
        // Tipped but airborne (`+0x6c == 0`) takes the same `goto` as upright [M 0x188494].
        state.tip_ticks = 0;
        let f = car.rot[2];
        for k in 0..3 {
            state.heading[k] += (f[k] - state.heading[k]) * HEADING_BLEND;
        }
    }
    if state.tip_ticks != TIP_TICKS {
        return None;
    }
    state.tip_ticks = 0;
    Some(righting(car, tris, state.heading, target))
}

/// `FUN_00187b00` for the player [M]: the direction is the objective marker's direction from
/// the car (normalised, `vrsqrt`) when one is set, else the remembered heading (the player's
/// vtable `+0x98` is `FUN_00193008` = `+0x2c8`, 0 → the `+0x3f0` branch). The car stays at its
/// x/z; y = the ground under its centre (`FUN_002151b8`, 0 when none) + TYRE_RADIUS +
/// SPRING_REST_LENGTH + the box half height (model `+0x4c4`, written from the half extents by
/// `FUN_001f6e98`). The `+0x2e4` path branch (a follower in state 2) is not ported [S: its
/// owner is not traced].
pub fn righting(car: &Car, tris: &[CollisionTri], heading: [f32; 3], target: Option<[f32; 3]>) -> Respawn {
    let p = car.pos;
    let dir = match target {
        Some(t) => [t[0] - p[0], t[1] - p[1], t[2] - p[2]],
        None => heading,
    };
    let ground = nf_formats::trackstart::ground_height(tris, p[0], p[1], p[2]).unwrap_or(0.0);
    let k = &car.params;
    Respawn {
        cause: Cause::Righted,
        pos: [p[0], ground + k.tyre_radius + k.spring_rest + k.half_extents[1], p[2]],
        rot: basis_from_dir(dir),
        vel: [0.0; 3],
        spring_prev: [0.0; 4],
    }
}

/// `FUN_00187eb8` (the player's vtable `+0x160`), the placement `EResetPlayerCarPos` and
/// `EStartMission` end in [M 0x156460 → 0x1fee68 then `+0x160`]: the matrix from `dir`; the
/// ground under `pos` (else `pos.y + 1`); each wheel's base point placed at that ground +
/// TYRE_RADIUS + SPRING_REST_LENGTH + half height, its ground plane queried (`FUN_001f6ae0`),
/// its height taken as `wheel.y + depth − 0.15·LIMIT` and its spring memory set to
/// `0.15·LIMIT`; the body y = the four heights' mean + the same three offsets [M 0x187eb8].
/// A wheel with no plane under it takes depth 0 [S]. The velocity is the unit `dir` times
/// `speed_factor` (the momentum write `dir·factor·mass` at 0x1565f0 [M]; the normalise is in
/// hidden VU0 code [S]).
pub fn place_at<G: Ground + ?Sized>(car: &Car, world: &G, tris: &[CollisionTri], pos: [f32; 3], dir: [f32; 3], speed_factor: f32) -> Respawn {
    let rot = basis_from_dir(dir);
    let k = &car.params;
    let lift = k.tyre_radius + k.spring_rest + k.half_extents[1];
    let ground = nf_formats::trackstart::ground_height(tris, pos[0], pos[1], pos[2]).unwrap_or(pos[1] + PLACE_NO_GROUND_LIFT);
    let base = [pos[0], ground + lift, pos[2]];
    let comp = k.spring_limit * PLACE_COMPRESSION;
    let mut sum = 0.0;
    for l in k.wheel_points() {
        let o = [0, 1, 2].map(|j| rot[0][j] * l[0] + rot[1][j] * l[1] + rot[2][j] * l[2]);
        let wp = [base[0] + o[0], base[1] + o[1], base[2] + o[2]];
        let depth = world.wheel_plane(wp).map_or(0.0, |pl| {
            (pl.v0[0] - wp[0]) * pl.normal[0] + (pl.v0[1] - wp[1]) * pl.normal[1] + (pl.v0[2] - wp[2]) * pl.normal[2]
        });
        sum += wp[1] + depth - comp;
    }
    let f = unit(dir);
    Respawn {
        cause: Cause::Placed,
        pos: [pos[0], sum * 0.25 + lift, pos[2]],
        rot,
        vel: [f[0] * speed_factor, f[1] * speed_factor, f[2] * speed_factor],
        spring_prev: [comp; 4],
    }
}

/// `FUN_00210540`: rows from a direction — forward = unit(dir), right = unit((0, 1, 0) × fwd),
/// up = fwd × right [M: the normalise, the two VU0 outer products against (0, 1, 0); which
/// output row holds which vector is hidden in the decompile — the port's rows (right, up,
/// forward) are the reading that keeps an upright `dir` upright [S]].
pub fn basis_from_dir(dir: [f32; 3]) -> [[f32; 3]; 3] {
    let f = unit(dir);
    let r = unit([f[2], 0.0, -f[0]]);
    let u = [f[1] * r[2] - f[2] * r[1], f[2] * r[0] - f[0] * r[2], f[0] * r[1] - f[1] * r[0]];
    [r, u, f]
}

fn len(a: [f32; 3]) -> f32 { (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt() }

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 1e-12 { [a[0] / l, a[1] / l, a[2] / l] } else { [0.0, 0.0, 1.0] }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driving::{CarParams, FlatGround};

    fn flat_tris(y: f32) -> Vec<CollisionTri> {
        vec![
            CollisionTri { v: [[-500.0, y, -500.0], [500.0, y, -500.0], [0.0, y, 500.0]], flags: 0x0001 },
        ]
    }

    fn tipped(car: &mut Car) {
        // On its roof: up = −y, forward kept.
        car.rot = [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]];
    }

    #[test]
    fn the_reset_fires_on_the_thirtieth_tipped_grounded_tick_not_before() {
        let mut car = Car::new(CarParams::vanquish(), [10.0, 3.0, 20.0], 0.0);
        let mut st = ResetState::new(&car);
        tipped(&mut car);
        car.hull_contacts = 2;
        let tris = flat_tris(-1.0);
        for t in 1..TIP_TICKS {
            assert_eq!(check(&car, &tris, &mut st, None), None, "tick {t}");
            assert_eq!(st.tip_ticks, t);
        }
        let r = check(&car, &tris, &mut st, None).expect("tick 30 fires");
        assert_eq!(st.tip_ticks, 0, "FUN_00187350 zeroes +0x3e8");
        assert_eq!(r.cause, Cause::Righted);
        // y = ground −1 + TYRE_RADIUS 0.065 + SPRING_REST_LENGTH 0.24 + half y 0.5778407 [D/M].
        assert!((r.pos[1] - (-1.0 + 0.065 + 0.24 + 0.577_840_7)).abs() < 1e-5, "{:?}", r.pos);
        assert_eq!((r.pos[0], r.pos[2]), (10.0, 20.0), "rights it where it lies");
        assert_eq!(r.vel, [0.0; 3]);
        assert!((r.rot[1][1] - 1.0).abs() < 1e-6, "upright again: {:?}", r.rot);
    }

    #[test]
    fn airborne_or_upright_ticks_restart_the_count() {
        let mut car = Car::new(CarParams::vanquish(), [0.0, 3.0, 0.0], 0.0);
        let mut st = ResetState::new(&car);
        let tris = flat_tris(0.0);
        tipped(&mut car);
        car.hull_contacts = 1;
        for _ in 0..29 { check(&car, &tris, &mut st, None); }
        assert_eq!(st.tip_ticks, 29);
        // One tick in the air (tipped, no hull contact): back to 0 [M 0x188494].
        car.hull_contacts = 0;
        assert_eq!(check(&car, &tris, &mut st, None), None);
        assert_eq!(st.tip_ticks, 0);
        // On its side at up.y 0.3 exactly is not tipped (`< 0.3`).
        car.hull_contacts = 3;
        car.rot = [[1.0, 0.0, 0.0], [0.0, 0.3, 0.953_939_2], [0.0, -0.953_939_2, 0.3]];
        for _ in 0..40 { assert_eq!(check(&car, &tris, &mut st, None), None); }
        assert_eq!(st.tip_ticks, 0);
    }

    #[test]
    fn the_heading_memory_lags_the_forward_row_by_a_tenth_per_tick() {
        let car0 = Car::new(CarParams::vanquish(), [0.0; 3], 0.0);
        let mut st = ResetState::new(&car0);
        assert_eq!(st.heading, [0.0, 0.0, 1.0]);
        let car = Car::new(CarParams::vanquish(), [0.0; 3], std::f32::consts::FRAC_PI_2);
        check(&car, &[], &mut st, None);
        // h = (0,0,1) + ((1,0,0) − (0,0,1))·0.1 = (0.1, 0, 0.9).
        assert!((st.heading[0] - 0.1).abs() < 1e-6 && (st.heading[2] - 0.9).abs() < 1e-6, "{:?}", st.heading);
        // A tipped, grounded tick leaves it alone.
        let mut t = car.clone();
        tipped(&mut t);
        t.hull_contacts = 1;
        let before = st.heading;
        check(&t, &[], &mut st, None);
        assert_eq!(st.heading, before);
    }

    #[test]
    fn the_righting_faces_the_remembered_heading_or_the_objective_marker() {
        let mut car = Car::new(CarParams::vanquish(), [5.0, 2.0, 5.0], 0.0);
        tipped(&mut car);
        let r = righting(&car, &[], [1.0, 0.0, 0.0], None);
        assert!((r.rot[2][0] - 1.0).abs() < 1e-6, "forward = the heading: {:?}", r.rot[2]);
        assert!((r.rot[0][2] + 1.0).abs() < 1e-6, "right = up × forward = (0,0,−1): {:?}", r.rot[0]);
        // No ground under it: height 0 + offsets [M 0x187b00 `afStack_70[0] = 0.0`].
        assert!((r.pos[1] - (0.065 + 0.24 + 0.577_840_7)).abs() < 1e-5);
        // A marker straight behind (−z): it faces −z.
        let r = righting(&car, &[], [1.0, 0.0, 0.0], Some([5.0, 2.0, -100.0]));
        assert!((r.rot[2][2] + 1.0).abs() < 1e-6, "{:?}", r.rot[2]);
    }

    #[test]
    fn a_car_on_its_roof_rests_on_the_ground_and_counts_hull_contacts() {
        // Pass C (FUN_001f9500): without it the wheels point at the sky, nothing holds the body
        // and it falls through the ground.
        let p = CarParams::vanquish();
        let hy = p.half_extents[1];
        let mut car = Car::new(p, [0.0, hy - 0.01, 0.0], 0.0);
        car.rot = [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]];
        let g = FlatGround::paved(0.0);
        let rigid = crate::driving::Rigid::game_defaults();
        let mut tris = Vec::new();
        tris.extend(flat_tris(0.0));
        let mut st = ResetState::new(&car);
        let mut fired = None;
        for tick in 1..=120 {
            car.step(&crate::driving::CarInput::default(), &g, &rigid, crate::driving::DT);
            assert!(car.pos[1] > hy - 0.05, "tick {tick}: the roof sank to {}", car.pos[1]);
            if let Some(r) = check(&car, &tris, &mut st, None) {
                fired = Some(tick);
                assert!(car.hull_contacts > 0);
                r.apply(&mut car);
                break;
            }
        }
        // Lying on its roof from tick 1, grounded every tick: righted on tick 30.
        assert_eq!(fired, Some(TIP_TICKS), "the reset tick");
        assert!(car.rot[1][1] > 0.999 && car.vel == [0.0; 3]);
    }

    #[test]
    fn the_script_placement_sets_the_springs_and_the_launch_speed() {
        let car = Car::new(CarParams::vanquish(), [0.0; 3], 0.0);
        let g = FlatGround::paved(2.0);
        let tris = flat_tris(2.0);
        let r = place_at(&car, &g, &tris, [1.0, 2.5, 1.0], [0.0, 0.0, 2.0], 10.0);
        assert_eq!(r.cause, Cause::Placed);
        let comp = 0.75 * 0.15;
        assert_eq!(r.spring_prev, [comp; 4]);
        assert!((r.vel[2] - 10.0).abs() < 1e-6 && r.vel[0] == 0.0, "unit dir × factor: {:?}", r.vel);
        // Flat ground at 2 (FlatGround's normal y is 0.9999): each wheel height ≈ 2 − 0.1125,
        // body = that + 0.065 + 0.24 + 0.5778.
        let want = 2.0 - comp + 0.065 + 0.24 + 0.577_840_7;
        assert!((r.pos[1] - want).abs() < 1e-3, "{} vs {want}", r.pos[1]);
        let mut c = car.clone();
        c.vel = [3.0, 0.0, 0.0];
        c.gear = 4;
        r.apply(&mut c);
        assert_eq!((c.pos, c.vel, c.gear, c.ang_vel), (r.pos, r.vel, 1, [0.0; 3]));
    }
}
