//! The land car's step (DRIVING.ELF FUN_0018cef8 + FUN_0018c1b0 under FUN_00188200's
//! SUB_PHYSICS == 0 / IS_SNOWMOBILE == 0 / actor+0x2c8 == 0 branch [M 0x188bf8]): suspension
//! against the ground plane, the grip shares, the drive / brake / reverse forces, gravity and air
//! time, the walls. docs/systems/drive-physics.md §8 is the function-by-function table.
//! Every expected number below is derived from the [M]/[D] constants, so a drifted constant
//! fails the compare.

use nf_formats::crp_mesh::CollisionTri;
use nf_game::driving::{Car, CarInput, CarParams, CollisionGround, FlatGround, Ground, Rigid, DT};

fn vanquish_at(y: f32) -> Car {
    let mut c = Car::new(CarParams::vanquish(), [0.0, 0.0, 0.0], 0.0);
    c.pos[1] = y;
    c
}

fn settle<G: Ground>(car: &mut Car, g: &G, ticks: usize) {
    let r = Rigid::game_defaults();
    for _ in 0..ticks {
        car.step(&CarInput::default(), g, &r, DT);
    }
}

const FULL: CarInput = CarInput { throttle: 1.0, brake: 0.0, steer: 0.0, handbrake: false };

/// The wheel base points [M FUN_001f72d8: (∓hx ± X, −hy, ±hz + Z{F,R})] from the .atr X/ZF/ZR
/// offsets [D 0.16 / −0.894 / 1.006] and the vanquish.crp bbox half extents [D].
#[test]
fn wheel_points_are_the_box_corners_pulled_in_by_the_atr_offsets() {
    let p = CarParams::vanquish();
    // The suspension / grip literals of vanquish.atr [D].
    assert_eq!((p.spring_stiffness, p.spring_damping, p.spring_rest, p.spring_limit), ([80.0, 98.0], [400.0, 350.0], 0.24, 0.75));
    assert_eq!((p.accel_frw, p.coast_frw, p.brake_frw, p.reverse_frw, p.handbrake_frw), (0.25, 0.35, 0.65, 0.6, 0.9));
    assert_eq!((p.slope_scale, p.wheel_force_app_scale, p.counter_steer_scale, p.counter_steer_limit), (0.5, 0.5, 1.72, 2.0));
    let w = p.wheel_points();
    let [hx, hy, hz] = p.half_extents;
    assert!((hx - 0.919_357_66).abs() < 1e-6 && (hy - 0.577_840_7).abs() < 1e-6 && (hz - 2.316_776_3).abs() < 1e-6);
    assert!((w[0][0] - (-hx + 0.16)).abs() < 1e-6, "FL x = −hx + X");
    assert!((w[1][0] - (hx - 0.16)).abs() < 1e-6, "FR x = hx − X");
    assert!((w[0][2] - (hz - 0.894)).abs() < 1e-6, "front z = hz + ZF");
    assert!((w[2][2] - (-hz + 1.006)).abs() < 1e-6, "rear z = −hz + ZR");
    assert!(w.iter().all(|q| q[1] == -hy), "all four at −hy");
    // Sister: the wheelbase is 2hz − 1.9 = 2.7336 m, not the 1.9 m ZR − ZF the earlier pass used.
    let wb = w[0][2] - w[2][2];
    assert!((wb - 2.733_553).abs() < 1e-4, "wheelbase {wb}");
}

/// FUN_00210938's `hz ≤ 4` branch [M]: per unit mass, z: 1/(hx²+hy²); x = y:
/// 0.5/(0.08333·(2hz)² + 0.25·(hx²+hy²)).
#[test]
fn inverse_inertia_follows_the_box_formula() {
    let p = CarParams::vanquish();
    let ii = p.inv_inertia_per_mass();
    let [hx, hy, hz] = p.half_extents;
    let s = hx * hx + hy * hy;
    assert!((ii[2] - 1.0 / s).abs() < 1e-5);
    let xy = 0.5 / (0.08333 * (2.0 * hz) * (2.0 * hz) + 0.25 * s);
    assert!((ii[0] - xy).abs() < 1e-6 && ii[0] == ii[1]);
    assert!((ii[0] - 0.2399).abs() < 1e-3 && (ii[2] - 0.8481).abs() < 1e-3, "{ii:?}");
}

/// The per-wheel spring `x·STIFF + (x − prev)·DAMP` with x = d + REST [M FUN_0018c1b0]: at rest
/// on the flat plane the four springs carry exactly the bodge gravity (13 per unit mass), and the
/// rear (98) compresses less than the front (80). Sister: a 100 front stiffness would not balance.
#[test]
fn the_springs_hold_the_bodge_gravity_at_rest() {
    let mut car = vanquish_at(0.0);
    car.pos[1] = car.rest_height();
    settle(&mut car, &FlatGround::paved(0.0), 240);
    let p = &car.params;
    let [xf, _, xr, _] = car.spring_prev;
    // The springs push along n, whose y is clamped to 0.9999 [M FUN_001f6ae0].
    let load = (2.0 * p.spring_stiffness[0] * xf + 2.0 * p.spring_stiffness[1] * xr) * 0.9999;
    assert!((load - 13.0).abs() < 1e-4, "springs carry {load}, gravity 13");
    assert!(xr < xf, "the stiffer rear sags less ({xf} / {xr})");
    assert!(car.vel[1].abs() < 1e-4 && car.contacts == 4 && car.grounded);
    // The body sits at REST + hy minus the sag (n.y = 0.9999 on an up-facing tri [M 0x1f6ae0]).
    let expect_y = (p.spring_rest - (xf + xr) * 0.5 + p.half_extents[1]) / 0.9999;
    assert!((car.pos[1] - expect_y).abs() < 3e-3, "y {} vs {expect_y}", car.pos[1]);
    let wrong = (2.0 * 100.0 * xf + 2.0 * p.spring_stiffness[1] * xr) * 0.9999;
    assert!((wrong - 13.0).abs() > 0.5);
}

/// No floor at 0: a ground 50 m below the origin holds the car at its own height (the old step
/// clamped `pos.y` to 0 and kept vy = 0 above it).
#[test]
fn the_car_rests_on_collision_ground_below_zero() {
    let y0 = -50.0;
    let tris = vec![
        CollisionTri { v: [[-100.0, y0, -100.0], [-100.0, y0, 100.0], [100.0, y0, -100.0]], flags: 1 },
        CollisionTri { v: [[100.0, y0, -100.0], [-100.0, y0, 100.0], [100.0, y0, 100.0]], flags: 1 },
    ];
    let g = CollisionGround::new(tris);
    let mut car = vanquish_at(y0 + 1.0);
    settle(&mut car, &g, 240);
    assert!(car.contacts == 4, "all four wheels on the −50 ground");
    assert!(car.pos[1] < y0 + 1.0 && car.pos[1] > y0 + 0.7, "settled at y {}", car.pos[1]);
    // Class bit 0x04 tris are not ground [M 0x2138b0]: the same floor flagged 0x0401 drops the car.
    let skip = CollisionGround::new(vec![
        CollisionTri { v: [[-100.0, y0, -100.0], [-100.0, y0, 100.0], [100.0, y0, -100.0]], flags: 0x0401 },
        CollisionTri { v: [[100.0, y0, -100.0], [-100.0, y0, 100.0], [100.0, y0, 100.0]], flags: 0x0401 },
    ]);
    let mut fall = vanquish_at(y0 + 1.0);
    settle(&mut fall, &skip, 60);
    assert_eq!(fall.contacts, 0);
    assert!(fall.pos[1] < y0, "falls through the skipped class");
}

/// Gravity and air time [M FUN_001f7c60 :136-144, FUN_001ff3d8's D-then-E order]: the body
/// integrates last tick's accumulator first, so after N ticks from rest vy = g·(N−1)·dt; in the
/// air the gearbox revs in neutral (`+0x299 == 0` → throttle·7000 [M 0x18aa88:32]).
#[test]
fn air_time_falls_at_the_bodge_gravity_and_lands() {
    let r = Rigid::game_defaults();
    let w = FlatGround::paved(0.0);
    let mut car = vanquish_at(20.0);
    let half = CarInput { throttle: 0.5, ..Default::default() };
    for _ in 0..60 {
        car.step(&half, &w, &r, DT);
    }
    assert_eq!(car.contacts, 0);
    assert!(!car.grounded);
    assert!((car.vel[1] - (-13.0 * 59.0 * DT)).abs() < 1e-3, "vy {}", car.vel[1]);
    assert_eq!(car.rpm, 0.5 * 7000.0, "neutral rev in the air");
    // jungle_truck's GRAVITY −34 [M :140]: the same fall is 34/13 faster.
    let mut truck = vanquish_at(20.0);
    truck.gravity = r.gravity;
    for _ in 0..60 {
        truck.step(&half, &w, &r, DT);
    }
    assert!((truck.vel[1] - (-34.0 * 59.0 * DT)).abs() < 1e-3);
    for _ in 0..600 {
        car.step(&CarInput::default(), &w, &r, DT);
    }
    assert_eq!(car.contacts, 4, "landed");
    assert!(car.vel[1].abs() < 1e-2 && car.pos[1] > 0.6 && car.pos[1] < 0.85, "settled y {}", car.pos[1]);
}

/// The drive [M FUN_0018cef8]: rear wheels only (IS_RALLY 0), each pushing
/// `MAXACC·RATIO[g]·(1 − ACCEL_FRW)·TYREGRIP`, against the per-wheel rolling vector
/// `vel·ROLLING_RESISTANCE` [M FUN_0018c1b0 `−= param_7`]. Top speed = the balance
/// 2·3·0.52·0.75·8 = 0.4·v → 46.8 m/s in gear 3, short of the 50 m/s LIMIT3. Sister: the old
/// flat model ran past 54 m/s.
#[test]
fn full_throttle_tops_out_at_the_drive_vs_rolling_balance() {
    let r = Rigid::game_defaults();
    let w = FlatGround::paved(0.0);
    let mut car = vanquish_at(0.0);
    car.pos[1] = car.rest_height();
    settle(&mut car, &w, 120);
    let p = car.params.clone();
    let mut order = Vec::new();
    for _ in 0..3600 {
        car.step(&FULL, &w, &r, DT);
        if order.last() != Some(&car.gear) { order.push(car.gear); }
    }
    assert_eq!(order, vec![1, 2, 3], "the auto box walks 1 → 2 → 3 [M 0x18aa88]");
    let drive = 2.0 * p.maxacc * p.gear_ratio[3] * (1.0 - p.accel_frw) * p.tyre_grip_factor;
    let v_top = drive / (4.0 * 0.1);
    assert!((car.speed - v_top).abs() < 0.05, "speed {} vs balance {v_top}", car.speed);
    assert!((v_top - 46.8).abs() < 1e-3);
    assert_eq!(car.yaw, 0.0, "a straight run stays straight");
}

/// The first ticks from rest [M]: in gear 1 the two rear wheels give 2·1.95·0.75·8 = 23.4 per unit
/// mass, landing one tick late (the controller's forces integrate on the next tick).
#[test]
fn launch_acceleration_is_the_gear_one_rear_drive() {
    let r = Rigid::game_defaults();
    let w = FlatGround::paved(0.0);
    let mut car = vanquish_at(0.0);
    car.pos[1] = car.rest_height();
    settle(&mut car, &w, 240);
    car.step(&FULL, &w, &r, DT); // accumulates
    let v0 = car.vel[2];
    car.step(&FULL, &w, &r, DT); // integrates it
    let a = (car.vel[2] - v0) / DT;
    assert!((a - 23.4).abs() < 0.05, "launch accel {a}");
}

/// Braking at speed [M]: state 2 puts BRAKE_FRW·BRAKING_FRICTION on the front grip; the front
/// wheels push `−MAXBRAKE·0.65·8·0.3` each (the rear carry no brake), plus the rolling drag.
#[test]
fn braking_decelerates_by_the_front_brake_force_plus_rolling() {
    let r = Rigid::game_defaults();
    let w = FlatGround::paved(0.0);
    let mut car = vanquish_at(0.0);
    car.pos[1] = car.rest_height();
    settle(&mut car, &w, 120);
    for _ in 0..600 {
        car.step(&FULL, &w, &r, DT);
    }
    let brake = CarInput { brake: 1.0, ..Default::default() };
    car.step(&brake, &w, &r, DT);
    let v0 = car.vel[2];
    car.step(&brake, &w, &r, DT);
    let a = (car.vel[2] - v0) / DT;
    let p = &car.params;
    let expect = -(2.0 * p.maxbrake * p.brake_frw * p.tyre_grip_factor * p.braking_friction) - 0.4 * v0;
    assert!((a - expect).abs() < 0.3, "brake decel {a} vs {expect}");
    assert!(!car.reversing);
}

/// Reverse [M 0x18cef8 :184-208, :401-407, :425-429]: the brake-hold counter reads 30 while the
/// brake is up, so a brake pressed at rest (longitudinal speed ≤ 0) reverses at once; braking from
/// speed resets it to 0 above 2.5 m/s and the reverse waits for 30 more held ticks; the front wheels then push back with
/// `rev·REVERSE_FRW·TYREGRIP`, rev = 2.4 − (v − 10)·0.075. The balance 9.6·(3.15 − 0.075v) = 0.4v
/// → 27.0 m/s.
#[test]
fn reverse_engages_from_rest_and_tops_out_at_the_cutoff_balance() {
    let r = Rigid::game_defaults();
    let w = FlatGround::paved(0.0);
    let mut car = vanquish_at(0.0);
    car.pos[1] = car.rest_height();
    settle(&mut car, &w, 120);
    let brake = CarInput { brake: 1.0, ..Default::default() };
    // From speed: brake to a stop, then the 30-tick hold before the reverse.
    for _ in 0..120 {
        car.step(&FULL, &w, &r, DT);
    }
    let mut stopped = None;
    let mut first = None;
    for t in 0..1800 {
        car.step(&brake, &w, &r, DT);
        if stopped.is_none() && car.vel[2] < 2.5 { stopped = Some(t); }
        if car.reversing && first.is_none() { first = Some(t); }
    }
    let (s, f) = (stopped.unwrap(), first.unwrap());
    assert!(f >= s + 30, "reverse {f} only after the 30-tick hold from {s}");
    assert!(car.vel[2] < 0.0);
    let p = &car.params;
    let k = 2.0 * p.reverse_frw * p.tyre_grip_factor;
    let v = k * (p.maxrevacc + p.speed_cutoff_reverse * p.speed_cutoff_rate_reverse) / (0.4 + k * p.speed_cutoff_rate_reverse);
    assert!((car.speed - v).abs() < 0.05, "reverse top {} vs {v}", car.speed);
}

/// The reverse drive term [M 0x18cef8 :425-429]: in the reverse state the THROTTLE still pushes
/// with the engine (MAXACC·RATIO), and the brake pulls with the decayed reverse accel.
#[test]
fn drive_accel_in_reverse_is_throttle_engine_minus_brake_reverse() {
    let mut car = vanquish_at(0.0);
    car.reversing = true;
    car.speed = 20.0;
    let rev = 2.4 - 10.0 * 0.075;
    assert!((car.drive_accel(&CarInput { brake: 1.0, ..Default::default() }) + rev).abs() < 1e-5);
    assert!((car.drive_accel(&CarInput { throttle: 1.0, ..Default::default() }) - 3.0 * 0.65).abs() < 1e-6);
    car.speed = 50.0;
    assert_eq!(car.drive_accel(&CarInput { brake: 1.0, ..Default::default() }), 0.0, "floored at 0");
}

/// The friction modifier actor+0x2e8 [M]: 1 to FRIC_MOD_SPEED 10, then 1 − (v−10)/75, ≥ 0.45.
#[test]
fn fric_mod_falls_from_10_to_the_0_45_floor() {
    let car = vanquish_at(0.0);
    assert_eq!(car.fric_mod(10.0), 1.0);
    assert!((car.fric_mod(47.5) - 0.5).abs() < 1e-6);
    assert!((car.fric_mod(48.0) - 0.493_333).abs() < 1e-5);
    assert_eq!(car.fric_mod(60.0), 0.45);
    assert_eq!(car.fric_mod(200.0), 0.45);
}

/// Steering at speed turns the car the way the input points (positive steer → yaw increases, the
/// port's convention), and the turn holds a steady yaw rate under the floored 0.03825 rad angle.
#[test]
fn steering_turns_the_car_to_a_steady_rate() {
    let r = Rigid::game_defaults();
    let w = FlatGround::paved(0.0);
    let mut car = vanquish_at(0.0);
    car.pos[1] = car.rest_height();
    settle(&mut car, &w, 120);
    for _ in 0..600 {
        car.step(&FULL, &w, &r, DT);
    }
    let right = CarInput { steer: 1.0, ..FULL };
    for _ in 0..120 {
        car.step(&right, &w, &r, DT);
    }
    let y1 = car.yaw;
    let w1 = car.yaw_rate;
    for _ in 0..60 {
        car.step(&right, &w, &r, DT);
    }
    assert!(y1 > 0.0 && car.yaw > y1, "turning right");
    assert!((car.yaw_rate - w1).abs() < 0.01, "steady rate {w1} → {}", car.yaw_rate);
    assert!((car.steer_angle - 0.09 * 0.425).abs() < 1e-6, "the MINSTEER floor at speed");
    assert_eq!(car.contacts, 4, "all four stay down in the turn");
}

/// The handbrake [M 0x18cef8 handbrake arm]: at speed it adds the HANDBRAKE_FORCE drag per wheel
/// and swaps to the 0.9 / 0.1 grip shares × TOTAL_HANDBRAKE_FRICTION_SCALE; parked
/// (4 wheels, |v| < 0.08, no throttle) the horizontal velocity zeroes.
#[test]
fn handbrake_slows_harder_than_coasting_and_parks() {
    let r = Rigid::game_defaults();
    let w = FlatGround::paved(0.0);
    let mut base = vanquish_at(0.0);
    base.pos[1] = base.rest_height();
    settle(&mut base, &w, 120);
    for _ in 0..300 {
        base.step(&FULL, &w, &r, DT);
    }
    let mut coast = base.clone();
    let mut hb = base.clone();
    let hbi = CarInput { handbrake: true, ..Default::default() };
    for _ in 0..60 {
        coast.step(&CarInput::default(), &w, &r, DT);
        hb.step(&hbi, &w, &r, DT);
    }
    assert!(hb.speed < coast.speed, "handbrake {} < coast {}", hb.speed, coast.speed);
    for _ in 0..1200 {
        hb.step(&hbi, &w, &r, DT);
    }
    assert_eq!((hb.vel[0], hb.vel[2]), (0.0, 0.0), "parked");
}

/// Pass B [S response, M sweep]: a wall across the road stops the car; it never passes the wall.
#[test]
fn a_wall_stops_the_car() {
    let mut tris = vec![
        CollisionTri { v: [[-100.0, 0.0, -100.0], [-100.0, 0.0, 200.0], [100.0, 0.0, -100.0]], flags: 1 },
        CollisionTri { v: [[100.0, 0.0, -100.0], [-100.0, 0.0, 200.0], [100.0, 0.0, 200.0]], flags: 1 },
    ];
    let zw = 60.0;
    tris.push(CollisionTri { v: [[-100.0, -5.0, zw], [100.0, -5.0, zw], [-100.0, 10.0, zw]], flags: 1 });
    tris.push(CollisionTri { v: [[100.0, -5.0, zw], [100.0, 10.0, zw], [-100.0, 10.0, zw]], flags: 1 });
    let g = CollisionGround::new(tris);
    let r = Rigid::game_defaults();
    let mut car = vanquish_at(1.0);
    settle(&mut car, &g, 60);
    let hz = car.params.half_extents[2];
    for _ in 0..900 {
        car.step(&FULL, &g, &r, DT);
        // The sweep uses the velocity before this tick's accel lands (B runs before D), so the
        // nose may run on by up to acc·dt² (< 0.1 m).
        assert!(car.pos[2] + hz <= zw + 0.1, "nose past the wall at z {}", car.pos[2]);
    }
    assert!(car.pos[2] + hz > zw - 1.0, "it reached the wall");
}
