//! Unit tests for `nf_game::driving::path_drive` (the spline-driven player vehicle: the
//! `FUN_00188200` pass choice, the `FUN_00131620` pose copy and the `FUN_00192498` path pass).
//! Expected values are worked by hand from the decompile [M] cited in the module.

use nf_game::driving::flight::{PathFollower, BASE_TURN, IDENTITY, Mat4};
use nf_game::driving::path_drive::*;
use nf_game::driving::sub_physics::{DrivePass, VehicleAttrs};
use nf_game::driving::{CarParams, FlatGround, WheelPlane};

fn heli_atr() -> &'static str {
    // The helicopter.atr keys these tests read [D MIS01.VIV].
    "[helicopter]\nSECONDARY_TYPE.s=vanquish\nCAR_WHEEL_X_OFFSET.f=0.13\nCAR_WHEEL_ZF_OFFSET.f=-0.8\n\
     CAR_WHEEL_ZR_OFFSET.f=1.17\nMASS.f=1660.0\nSPRING_REST_LENGTH=0.35\nSPRING_COMPRESSION_LIMIT=0.75\n\
     WHEEL_RADIUS=0.315\nIS_HELICOPTER.b=true\n"
}

fn heli_params() -> CarParams {
    let mut p = CarParams::parse_atr(heli_atr());
    p.half_extents = [1.0, 0.5, 2.0];
    p
}

fn follower_at(pose: Mat4, vel: [f32; 3]) -> PathFollower {
    let mut f = PathFollower::start(1, &Default::default(), 0.0);
    f.pose = pose;
    f.velocity = vel;
    f
}

/// No triangle anywhere: every wheel gets depth 0, so its spring sits at the rest length and
/// keeps its slip flag.
struct NoGround;
impl nf_game::driving::Ground for NoGround {
    fn wheel_plane(&self, _p: [f32; 3]) -> Option<WheelPlane> { None }
}

fn translate(m: Mat4, p: [f32; 3]) -> Mat4 {
    let mut m = m;
    m[3] = [p[0], p[1], p[2], 1.0];
    m
}

#[test]
fn the_helicopter_takes_the_land_pass_off_the_spline_and_the_path_pass_on_it() {
    let a = VehicleAttrs::parse_atr(heli_atr());
    assert_eq!((a.sub_physics, a.is_snowmobile), (0, 0), "IS_HELICOPTER is not in the schema");
    assert_eq!(player_pass(&a, 0, false, 1.0), DrivePass::PlayerLand);
    assert_eq!(player_pass(&a, 1, true, 1.0), DrivePass::Path);
    // EControlToSpline leaves car+0x2c8 = 1: a dead spline-driven car gets the AI land pass.
    assert_eq!(player_pass(&a, 1, true, 0.0), DrivePass::AiLand);
    assert_eq!(player_pass(&a, 1, false, 1.0), DrivePass::AiLand);
    assert!(!path_driven(false, true, 1.0));
    assert!(!path_driven(true, true, -0.5));
}

#[test]
fn the_body_takes_the_path_pose_velocity_and_momentum() {
    let f = follower_at(translate(BASE_TURN, [809.6, 4.2, 215.0]), [3.0, -1.0, 4.0]);
    let b = place_on_path(&f, &IDENTITY, 1660.0);
    assert_eq!(b.pos(), [809.6, 4.2, 215.0]);
    assert_eq!(b.row(0), [0.0, 0.0, -1.0]);
    assert_eq!(b.row(2), [1.0, 0.0, 0.0]);
    assert_eq!(b.velocity, [3.0, -1.0, 4.0]);
    assert_eq!(b.momentum, [4980.0, -1660.0, 6640.0]);
    assert_eq!(b.angular_velocity, [0.0; 3]);
    // The offset multiplies the rotation only: a translated offset does not move the body.
    let off = translate(IDENTITY, [100.0, 100.0, 100.0]);
    assert_eq!(place_on_path(&f, &off, 1.0).pos(), [809.6, 4.2, 215.0]);
    // A quarter-turn offset turns the rows.
    let b2 = place_on_path(&f, &BASE_TURN, 1.0);
    assert_eq!(b2.row(0), [-1.0, 0.0, 0.0], "BASE_TURN · BASE_TURN row 0");
}

#[test]
fn speed_spin_roll_steer_and_skid() {
    let p = heli_params();
    let a = VehicleAttrs::parse_atr(heli_atr());
    let g = NoGround;
    // Heading +z (IDENTITY rows), moving along +z at 20 m/s.
    let b = place_on_path(&follower_at(translate(IDENTITY, [0.0, 50.0, 0.0]), [0.0, 3.0, 20.0]), &IDENTITY, 1660.0);
    let mut d = PathDrive::default();
    d.step(&b, &p, &a, 60, &g);
    let speed = (9.0f32 + 400.0).sqrt();
    assert!((d.speed - speed).abs() < 1e-5, "{}", d.speed);
    let spin = 20.0 / (0.315 * 6.32 * 60.0);
    assert!((d.wheel_spin - spin).abs() < 1e-6, "{}", d.wheel_spin);
    assert_eq!(d.roll, [d.wheel_spin, d.wheel_spin]);
    assert!(!d.skid);
    assert!(d.steer.abs() < 1e-6);
    assert_eq!(d.slip, [0.0; 4]);
    // The roll phases wrap past 1.
    d.roll = [0.99, 0.5];
    d.step(&b, &p, &a, 60, &g);
    assert!((d.roll[0] - (0.99 + spin - 1.0)).abs() < 1e-5, "{:?}", d.roll);
    // Sideways (+x) travel: heading cosine 0 → skid, all four slip flags; steer = row0 · v̂ = 1.
    let side = place_on_path(&follower_at(translate(IDENTITY, [0.0, 50.0, 0.0]), [12.0, 0.0, 0.0]), &IDENTITY, 1.0);
    let mut d = PathDrive::default();
    d.step(&side, &p, &a, 60, &g);
    assert!(d.skid);
    assert_eq!(d.slip, [1.0; 4]);
    assert_eq!(d.steer, 1.0);
    // 60 degrees off: cos 0.5 → rear flags only.
    let (s, c) = (60f32.to_radians().sin(), 60f32.to_radians().cos());
    let off60 = place_on_path(&follower_at(translate(IDENTITY, [0.0, 50.0, 0.0]), [10.0 * s, 0.0, 10.0 * c]), &IDENTITY, 1.0);
    let mut d = PathDrive::default();
    d.step(&off60, &p, &a, 60, &g);
    assert!(d.skid);
    assert_eq!(d.slip, [0.0, 0.0, 1.0, 1.0]);
    // 74 degrees off: cos 0.2756 sits between the 0.25 front and 0.707 rear thresholds.
    let (s, c) = (74f32.to_radians().sin(), 74f32.to_radians().cos());
    let off74 = place_on_path(&follower_at(translate(IDENTITY, [0.0, 50.0, 0.0]), [10.0 * s, 0.0, 10.0 * c]), &IDENTITY, 1.0);
    let mut d = PathDrive::default();
    d.step(&off74, &p, &a, 60, &g);
    assert_eq!(d.slip, [0.0, 0.0, 1.0, 1.0]);
    // 76 degrees off: cos 0.2419, under 0.25: the front flags too.
    let (s, c) = (76f32.to_radians().sin(), 76f32.to_radians().cos());
    let off76 = place_on_path(&follower_at(translate(IDENTITY, [0.0, 50.0, 0.0]), [10.0 * s, 0.0, 10.0 * c]), &IDENTITY, 1.0);
    let mut d = PathDrive::default();
    d.step(&off76, &p, &a, 60, &g);
    assert_eq!(d.slip, [1.0; 4]);
    // Under 0.01 turns a tick the spin is 0; at least 1 it holds 1.
    let slow = place_on_path(&follower_at(IDENTITY, [0.0, 0.0, 1.0]), &IDENTITY, 1.0);
    let mut d = PathDrive::default();
    d.step(&slow, &p, &a, 60, &g);
    assert_eq!(d.wheel_spin, 0.0, "1 m/s is 0.0084 turns a tick");
    let fast = place_on_path(&follower_at(IDENTITY, [0.0, 0.0, 200.0]), &IDENTITY, 1.0);
    d.step(&fast, &p, &a, 60, &g);
    assert_eq!(d.wheel_spin, 1.0);
    // A plane far below unloads every wheel: the springs read 0 and the skid's slip flags clear.
    let mut d = PathDrive::default();
    d.step(&side, &p, &a, 60, &FlatGround::paved(-1000.0));
    assert!(d.skid);
    assert_eq!((d.spring, d.slip), ([0.0; 4], [0.0; 4]));
}

#[test]
fn springs_hang_at_rest_in_the_air_and_compress_on_the_ground() {
    let p = heli_params();
    let a = VehicleAttrs::parse_atr(heli_atr());
    let b = place_on_path(&follower_at(translate(IDENTITY, [0.0, 50.0, 0.0]), [0.0, 0.0, 20.0]), &IDENTITY, 1.0);
    let mut d = PathDrive::default();
    d.step(&b, &p, &a, 60, &NoGround);
    assert_eq!(d.spring, [0.35; 4], "no plane: depth 0, the rest length");
    assert_eq!(d.ground[0], ([0.0, 1.0, 0.0], 0.0));
    // Body at y = 0.3, wheel points at -hy = -0.5 → world y -0.2; ground plane y = 0 (n.y 0.9999):
    // depth = 0.2 · 0.9999, spring = depth + 0.35.
    let low = place_on_path(&follower_at(translate(IDENTITY, [0.0, 0.3, 0.0]), [0.0, 0.0, 20.0]), &IDENTITY, 1.0);
    d.step(&low, &p, &a, 60, &FlatGround::paved(0.0));
    let want = 0.2f32 * 0.9999 + 0.35;
    for s in d.spring {
        assert!((s - want).abs() < 1e-5, "{:?}", d.spring);
    }
    // Deep under the ground: clamped to the 0.75 compression limit.
    let deep = place_on_path(&follower_at(translate(IDENTITY, [0.0, -3.0, 0.0]), [0.0, 0.0, 20.0]), &IDENTITY, 1.0);
    d.step(&deep, &p, &a, 60, &FlatGround::paved(0.0));
    assert_eq!(d.spring, [0.75; 4]);
    // High above a plane: negative length → 0 and the slip flag cleared.
    let mut d = PathDrive::default();
    d.slip = [1.0; 4];
    let high = place_on_path(&follower_at(translate(IDENTITY, [0.0, 10.0, 0.0]), [0.0, 0.0, 20.0]), &IDENTITY, 1.0);
    d.step(&high, &p, &a, 60, &FlatGround::paved(0.0));
    assert_eq!(d.spring, [0.0; 4]);
    assert_eq!(d.slip, [0.0; 4]);
}

#[test]
fn the_wheel_points_follow_the_body_rows() {
    let p = heli_params();
    let a = VehicleAttrs::parse_atr(heli_atr());
    // A ground that reports the queried point as its plane origin with surface 0: the depth is 0
    // only at that exact point, so record the points instead.
    struct Rec(std::cell::RefCell<Vec<[f32; 3]>>);
    impl nf_game::driving::Ground for Rec {
        fn wheel_plane(&self, p: [f32; 3]) -> Option<WheelPlane> { self.0.borrow_mut().push(p); None }
    }
    let r = Rec(Default::default());
    let b = place_on_path(&follower_at(translate(BASE_TURN, [10.0, 20.0, 30.0]), [0.0, 0.0, 0.0]), &IDENTITY, 1.0);
    PathDrive::default().step(&b, &p, &a, 60, &r);
    let pts = r.0.borrow();
    // FL local (-hx + X, -hy, hz + ZF) = (-0.87, -0.5, 1.2); BASE_TURN rows: x → (0,0,-1), z → (1,0,0).
    let fl = pts[0];
    assert!((fl[0] - (10.0 + 1.2)).abs() < 1e-5 && (fl[1] - 19.5).abs() < 1e-5 && (fl[2] - (30.0 + 0.87)).abs() < 1e-5, "{fl:?}");
    // RR local (hx - X, -hy, -hz + ZR) = (0.87, -0.5, -0.83).
    let rr = pts[3];
    assert!((rr[0] - (10.0 - 0.83)).abs() < 1e-5 && (rr[2] - (30.0 - 0.87)).abs() < 1e-5, "{rr:?}");
}

#[test]
fn the_surface_bump_and_the_boat_lift() {
    assert_eq!(surface_bump(1, [0.3, 0.0, 0.7]), 0.0, "paved: no bump");
    let p = [0.1f32, 0.0, 0.2];
    let tau = 6.283_185_5f32;
    let gravel = ((0.2 * 2.0 * tau).sin() + (0.1 * 2.0 * tau).cos()) * 0.01;
    assert!((surface_bump(2, p) - gravel).abs() < 1e-7);
    assert!((surface_bump(4, p) - gravel * 1.5).abs() < 1e-6);
    assert_eq!(surface_bump(6, p), -10.0, "surface 6: a flat -10");
    let pl = WheelPlane { v0: [0.0, 1.0, 0.0], normal: [0.0, 1.0, 0.0], surface: 1 };
    assert_eq!(wheel_depth(Some(&pl), [5.0, 0.5, 5.0], false).1, 0.5);
    assert_eq!(wheel_depth(Some(&pl), [5.0, 0.5, 5.0], true).1, 10.5);
}

#[test]
fn snowmobile_and_sub_variants() {
    let p = heli_params();
    let b = place_on_path(&follower_at(translate(IDENTITY, [0.0, 0.3, 0.0]), [5.0, 0.0, 0.0]), &IDENTITY, 1.0);
    let mut snow = VehicleAttrs::game_defaults();
    snow.is_snowmobile = 1;
    let mut d = PathDrive::default();
    d.spring = [0.4; 4];
    d.step(&b, &p, &snow, 60, &FlatGround::paved(0.0));
    assert_eq!(d.spring, [0.0; 4]);
    assert_eq!(d.slip, [0.0; 4], "the snowmobile arm zeroes the slip flags after the skid set them");
    assert_eq!(d.speed, 5.0);
    let mut sub = VehicleAttrs::game_defaults();
    sub.sub_physics = 1;
    let mut d = PathDrive::default();
    d.step(&b, &p, &sub, 60, &FlatGround::paved(0.0));
    assert_eq!(d, PathDrive::default(), "SUB_PHYSICS: the pass returns at once");
}
