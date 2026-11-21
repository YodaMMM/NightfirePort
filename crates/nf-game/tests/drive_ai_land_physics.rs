//! The AI cars' land pass and the land collision pieces of `Car::step`, without the disc:
//! `FUN_0018ecb8` / `FUN_0018e548` (the AI controller), `FUN_0018bb90` (the in-air levelling),
//! the land pass B response against a 2-D wall (`FUN_001fc2f0` → `FUN_001fb318`), and the VU0
//! integration's quaternion step (micro 0xd6-0x12e). Every test runs the same case on the old
//! path (the player pass, or no wall list) as its sister.

use nf_formats::crp_mesh::{CollisionTri, CollisionWall};
use nf_game::driving::physics::{quat_from_rows, rot_from_yaw, rows_from_quat};
use nf_game::driving::{Car, CarInput, CarParams, CollisionGround, DriverKind, FlatGround, Rigid, DT};

fn settled(kind: DriverKind, g: &FlatGround) -> Car {
    let mut car = Car::new(CarParams::vanquish(), [0.0, 0.0, 0.0], 0.0);
    car.pos[1] = g.height + car.rest_height();
    car.kind = kind;
    let rigid = Rigid::game_defaults();
    for _ in 0..120 {
        car.step(&CarInput::default(), g, &rigid, DT);
    }
    car
}

/// The AI drive is `throttle·max(MAXACC, 12.5)` on each rear wheel [M 0x18ecb8:256-261, the
/// 0x194/0x1a4 tuple slots], with no gearbox: the vanquish's MAXACC 3 is raised to 12.5, so two
/// rear wheels push 25 m/s². The player pass gives `3·0.65` per wheel times its grip share.
#[test]
fn the_ai_drive_is_the_floored_maxacc_on_both_rear_wheels() {
    let g = FlatGround::paved(0.0);
    let rigid = Rigid::game_defaults();
    let full = CarInput { throttle: 1.0, ..CarInput::default() };
    let mut ai = settled(DriverKind::Traffic, &g);
    let mut pl = settled(DriverKind::Player, &g);
    for _ in 0..6 {
        ai.step(&full, &g, &rigid, DT);
        pl.step(&full, &g, &rigid, DT);
    }
    let (v0a, v0p) = (ai.vel[2], pl.vel[2]);
    for _ in 0..30 {
        ai.step(&full, &g, &rigid, DT);
        pl.step(&full, &g, &rigid, DT);
    }
    let acc_ai = (ai.vel[2] - v0a) / (30.0 * DT);
    let acc_pl = (pl.vel[2] - v0p) / (30.0 * DT);
    eprintln!("AI pass {acc_ai:.2} m/s², player pass {acc_pl:.2} m/s²");
    assert!((acc_ai - 25.0).abs() < 1.5, "two rear wheels × 12.5 (got {acc_ai})");
    assert!(acc_pl < 0.9 * acc_ai, "the player pass (gear 1 ratio × grip share) is a different car (got {acc_pl})");
}

/// The AI steer is `input·max(MAXSTEERING, 0.2)` with no speed falloff [M 0x18ecb8:283-290];
/// the player pass falls off past MODSTEERSPEED [M 0x18cef8].
#[test]
fn the_ai_steer_has_the_0_2_floor_and_no_speed_falloff() {
    let g = FlatGround::paved(0.0);
    let rigid = Rigid::game_defaults();
    for (kind, want) in [(DriverKind::Hostile, 0.2f32), (DriverKind::Player, 0.09 * 0.425)] {
        let mut car = settled(kind, &g);
        car.vel = [0.0, 0.0, 30.0];
        car.step(&CarInput { steer: 1.0, ..CarInput::default() }, &g, &rigid, DT);
        assert!((car.steer_angle - want).abs() < 1e-4, "{kind:?}: steer {} want {want}", car.steer_angle);
    }
}

/// `FUN_0018bb90`: a kind-2 car in the air with up.y ≥ 0.4 blends toward the level frame
/// pitched NOSE_JUMP_ANGLE down, one NUM_BLEND_STEPS-th per tick; a traffic car (kind 3) is not
/// levelled [M 0x18ecb8:563-583].
#[test]
fn a_hostile_car_levels_in_the_air_and_traffic_does_not() {
    let g = FlatGround::paved(-1000.0);
    let rigid = Rigid::game_defaults();
    let pitched = {
        let (s, c) = 0.5f32.sin_cos();
        // nose up 0.5 rad about the body x axis
        [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
    };
    let mut ups = Vec::new();
    for kind in [DriverKind::Hostile, DriverKind::Traffic] {
        let mut car = Car::new(CarParams::vanquish(), [0.0, 100.0, 0.0], 0.0);
        car.kind = kind;
        car.place([0.0, 100.0, 0.0], pitched);
        for _ in 0..90 {
            car.step(&CarInput::default(), &g, &rigid, DT);
        }
        ups.push((car.rot[1][1], car.rot[2][1]));
    }
    eprintln!("hostile up.y/fwd.y {:?}, traffic {:?}", ups[0], ups[1]);
    let want_fwd_y = -(0.03f32).sin();
    assert!((ups[0].1 - want_fwd_y).abs() < 0.02, "the hostile's nose settles {want_fwd_y} (fwd.y {})", ups[0].1);
    assert!((ups[1].1 - 0.5f32.sin()).abs() < 0.02, "traffic keeps its pitch (fwd.y {})", ups[1].1);
}

fn floor() -> Vec<CollisionTri> {
    let s = 200.0;
    vec![
        CollisionTri { v: [[-s, 0.0, -s], [s, 0.0, s], [s, 0.0, -s]], flags: 1 },
        CollisionTri { v: [[-s, 0.0, -s], [-s, 0.0, s], [s, 0.0, s]], flags: 1 },
    ]
}

/// The land pass B against a 2-D wall [M FUN_001fc2f0 → FUN_00219328 → FUN_001fb318]: a car
/// driving at 15 m/s into a wall across its path stops at it; with no wall list the same car
/// drives through (the floor has no steep triangle for the stand-in to find).
#[test]
fn a_car_driven_into_a_2d_wall_stops_at_it() {
    let wall = CollisionWall { a: [-20.0, -1.0, 20.0], b: [20.0, 3.0, 20.0], surface: 1, class: 0 };
    let rigid = Rigid::game_defaults();
    let mut ends = Vec::new();
    for g in [CollisionGround::with_walls(floor(), vec![wall]), CollisionGround::new(floor())] {
        let mut car = Car::new(CarParams::vanquish(), [0.0, 0.0, 0.0], 0.0);
        car.kind = DriverKind::Hostile;
        car.pos[1] = car.rest_height();
        for _ in 0..60 {
            car.step(&CarInput::default(), &g, &rigid, DT);
        }
        car.vel = [0.0, 0.0, 15.0];
        let mut max_z = f32::MIN;
        for _ in 0..240 {
            car.step(&CarInput { throttle: 0.3, ..CarInput::default() }, &g, &rigid, DT);
            max_z = max_z.max(car.pos[2]);
        }
        ends.push((max_z, car.rot[1][1]));
    }
    eprintln!("with the wall: max z {:.2} up.y {:.3}; without: max z {:.2}", ends[0].0, ends[0].1, ends[1].0);
    let hz = CarParams::vanquish().half_extents[2];
    assert!(ends[0].0 < 20.0 - hz + 0.6, "the nose stays this side of the wall (centre max z {})", ends[0].0);
    assert!(ends[0].1 > 0.9, "and the car stays on its wheels");
    assert!(ends[1].0 > 25.0, "without the wall list the car passes z = 20 (max z {})", ends[1].0);
}

/// The quaternion the VU0 program keeps and the rows it rebuilds [M micro 0x110-0x12e] round
/// trip, and the rows are the port's yaw convention.
#[test]
fn rows_and_quaternions_round_trip() {
    for yaw in [0.0f32, 0.7, -2.4, 3.1] {
        let r = rot_from_yaw(yaw);
        let back = rows_from_quat(quat_from_rows(&r));
        for i in 0..3 {
            for j in 0..3 {
                assert!((back[i][j] - r[i][j]).abs() < 1e-5, "yaw {yaw}: row {i} {back:?} vs {r:?}");
            }
        }
    }
    let q = [0.0, (0.25f32).sin(), 0.0, (0.25f32).cos()]; // 0.5 rad about +y
    let r = rows_from_quat(q);
    let w = rot_from_yaw(0.5);
    assert!((r[2][0] - w[2][0]).abs() < 1e-5 && (r[2][2] - w[2][2]).abs() < 1e-5, "+y rotation = +yaw: {r:?} vs {w:?}");
}
