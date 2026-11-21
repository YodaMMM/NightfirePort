//! Unit tests for `nf_game::driving::sub_physics` (docs/systems/drive-physics.md §8-§9): the
//! SUB_PHYSICS pass `FUN_00190f88`, the VU0 rigid integrator (micro 0xad) and the pass dispatch
//! `FUN_00188200` 0x188bf8. Every number below is a DRIVING.ELF literal or a value derived from
//! one [M], or a shipped .atr line [D]; each test fails when its constant is changed.

use nf_game::driving::physics::{CarInput, CarParams, FlatWorld, Rigid, DT};
use nf_game::driving::sub_physics::*;

/// The SUB_PHYSICS lines of the shipped pvehicle files [D MIS11 vanquishsub.atr, MIS13A
/// ultralight.atr, MIS13C patrolboat.atr], cut to the keys the layer reads.
const VANQUISHSUB: &str = "[vanquishsub]\r\nMASS.f=1660.0\t\t\r\nBOOST_EXTRA_ACC.f=18.0\r\nBOOST_TIME.i=90\r\nIS_SUB=1\r\nSUB_PHYSICS=1\r\nHAS_TORPEDOS=1\r\n";
const ULTRALIGHT: &str = "[ultralight]\r\nMASS.f=1660.0\t\t\r\nBOOST_EXTRA_ACC.f=1.75\r\nBOOST_TIME.i=190\r\nSUB_PHYSICS=1\r\nNO_WORLD_COLLISIONS=1\r\nIS_FLYING=1\r\n";
const PATROLBOAT: &str = "[patrolboat]\r\nMASS.f=1660.0\t\t\r\nHAS_TURRET=1\r\nNO_WORLD_COLLISIONS=1\r\nIS_BOAT=1\r\n";

fn far() -> [f32; 3] { [10000.0, 0.0, 10000.0] } // no boundary sheet covers this point

fn craft() -> SubCraft { SubCraft::new(VehicleAttrs::parse_atr(VANQUISHSUB), [1.0, 0.5, 2.0], far(), 0.0, 0) }

#[test]
fn the_atr_flags_pick_the_pass() {
    let sub = VehicleAttrs::parse_atr(VANQUISHSUB);
    assert_eq!((sub.mass, sub.is_sub, sub.sub_physics, sub.boost_extra_acc, sub.boost_time), (1660.0, 1, 1, 18.0, 90));
    let ul = VehicleAttrs::parse_atr(ULTRALIGHT);
    assert_eq!((ul.sub_physics, ul.no_world_collisions, ul.is_sub), (1, 1, 0));
    let boat = VehicleAttrs::parse_atr(PATROLBOAT);
    assert_eq!((boat.sub_physics, boat.is_boat), (0, 1));
    // [M 0x188bf8-0x188c60]
    assert_eq!(drive_pass(&sub, 0, false), DrivePass::Sub);
    assert_eq!(drive_pass(&ul, 3, false), DrivePass::Sub);
    assert_eq!(drive_pass(&boat, 0, false), DrivePass::PlayerLand);
    assert_eq!(drive_pass(&boat, 1, false), DrivePass::AiLand);
    let mut snow = VehicleAttrs::game_defaults();
    snow.is_snowmobile = 1;
    assert_eq!(drive_pass(&snow, 0, false), DrivePass::Snowmobile);
    assert_eq!(drive_pass(&sub, 0, true), DrivePass::Path);
    assert_eq!(VehicleAttrs::game_defaults().mass, 2000.0);
}

#[test]
fn the_dispatcher_builds_the_vehicle_the_pass_names() {
    let p = CarParams::vanquish();
    assert!(matches!(Vehicle::player(VehicleAttrs::parse_atr(VANQUISHSUB), p.clone(), [1.0; 3], far(), 0.0, 0), Vehicle::Sub(_)));
    assert!(matches!(Vehicle::player(VehicleAttrs::parse_atr(PATROLBOAT), p, [1.0; 3], far(), 0.0, 0), Vehicle::Land(_)));
}

#[test]
fn trig_is_in_turns() {
    // FUN_002cbd50 / FUN_002cbeb0: sin/cos(2π·x) through the 9th-order polynomial [M].
    assert!((sin_turns(0.25) - 1.0).abs() < 1e-4);
    assert!((sin_turns(1.0 / 12.0) - 0.5).abs() < 1e-5);
    assert!((sin_turns(-0.25) + 1.0).abs() < 1e-4);
    assert!((sin_turns(1.25) - 1.0).abs() < 1e-4);
    assert!((cos_turns(0.0) - 1.0).abs() < 1e-4);
    // The polynomial is a plain Taylor series: near +-0.5 turn (2.6 rad here) it is off by ~1e-3,
    // as the game's is.
    assert!((cos_turns(1.0 / 6.0) - 0.5).abs() < 2e-3);
    assert!((cos_turns(1.0 / 6.0) - 0.5).abs() > 1e-5);
    assert!((cos_turns(-0.222) - (0.222f32 * std::f32::consts::TAU).cos()).abs() < 1e-4);
    // FUN_002104a8
    assert_eq!(approach(0.0, 1.0, 0.25), 0.25);
    assert_eq!(approach(0.9, 1.0, 0.25), 1.0);
    assert_eq!(approach(0.0, -0.1, 0.25), -0.1);
}

#[test]
fn inertia_from_half_extents() {
    // FUN_00210938 [M]: hz > 4 → the box form with mass·0.3333.
    let m = 1660.0;
    let a = inv_inertia(m, [1.0, 2.0, 5.0]);
    let k = m * 0.3333;
    assert!((a[0] - 1.0 / (k * 29.0)).abs() < 1e-9 && (a[1] - 1.0 / (k * 26.0)).abs() < 1e-9 && (a[2] - 1.0 / (k * 5.0)).abs() < 1e-9);
    // hz <= 4: z = 0.5/(m·0.5·(hx²+hy²)); x = y = 0.5/(m·0.08333·(2hz)² + m·0.25·(hx²+hy²)).
    let b = inv_inertia(m, [1.0, 0.5, 2.0]);
    assert!((b[2] - 0.5 / (m * 0.5 * 1.25)).abs() < 1e-9);
    assert!((b[0] - 0.5 / (m * 0.08333 * 16.0 + m * 0.25 * 1.25)).abs() < 1e-9 && b[0] == b[1]);
}

#[test]
fn thrust_and_drag_set_the_top_speed() {
    // Thrust (throttle − brake)·7, doubled, along the nose [M 0x190f88:344-367]; drag −0.5·v for
    // the player [M :368-379]: 14 = 0.5·v → 28 m/s.
    let mut c = craft();
    let rigid = Rigid::game_defaults();
    let input = SubInput { throttle: 1.0, ..Default::default() };
    c.step(&input, &rigid, DT);
    assert!((c.body.vel[2] - 14.0 * DT).abs() < 1e-6, "first tick: {:?}", c.body.vel);
    for _ in 0..3000 {
        c.step(&input, &rigid, DT);
    }
    assert!((c.body.vel[2] - 28.0).abs() < 0.01, "top speed {:?}", c.body.vel);
    assert!(c.body.vel[0].abs() < 1e-3 && c.body.vel[1].abs() < 1e-3);
    // An AI driver's drag is −0.2·v → 70 m/s.
    let mut ai = craft();
    ai.driver = 1;
    for _ in 0..6000 {
        ai.step(&input, &rigid, DT);
    }
    assert!((ai.body.vel[2] - 70.0).abs() < 0.05, "AI top speed {:?}", ai.body.vel);
}

#[test]
fn no_gravity_only_the_bob() {
    // SUB_PHYSICS == 1 skips the gravity force [M 0x1f7c60]; the height moves only by the bob
    // (sin(p1) + 1 + sin(p2))·0.075, phases +0.0022 / +0.0035 per tick [M 0x191ff8-0x1920c0].
    let mut c = craft();
    let y0 = c.body.pos[1];
    let rigid = Rigid::game_defaults();
    c.step(&SubInput::default(), &rigid, DT);
    assert!((c.body.pos[1] - (y0 + 0.075)).abs() < 1e-6, "seed 0: first bob = (0+1+0)·0.075");
    assert!((c.bob_phase[0] - 0.0022).abs() < 1e-7 && (c.bob_phase[1] - 0.0035).abs() < 1e-7);
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    for _ in 0..2000 {
        c.step(&SubInput::default(), &rigid, DT);
        let d = c.body.pos[1] - y0;
        lo = lo.min(d);
        hi = hi.max(d);
    }
    assert!(lo >= -0.0751 && hi <= 0.2251, "{lo} {hi}");
    assert!(hi - lo > 0.2, "the bob swings: {lo} {hi}");
    assert!(c.body.vel[1].abs() < 1e-6);
}

#[test]
fn lateral_axle_forces_clamp_at_12_5_and_9_375() {
    // Sliding sideways at 10 m/s, heading +z, no input: front −clamp(5·u, 12.5) after the
    // y-rotation by −slip (0 here), rear −clamp(5·u, 9.375) [M :490-591], drag −0.5·10.
    let mut c = craft();
    c.body.vel = [10.0, 0.0, 0.0];
    let mut inp = SubInput::default();
    c.controller(&mut inp);
    let a = c.body.force[0] / c.body.mass;
    assert!((a - (-5.0 - 12.5 - 9.375)).abs() < 1e-4, "{a}");
    assert!(c.body.force[1].abs() < 1e-3, "world y lane zeroed [M :524-531]");
}

#[test]
fn pitch_stick_pushes_along_the_up_axis() {
    // At 10 m/s along the nose with full pitch: gain (1 − 10/15)·2.5 + 1 = 1.833, ×0.125 →
    // 0.229 → clamped to 0.222 turns; rotX(−0.222) puts c·0 − s·10 = 9.848 in the y lane;
    // −4× that clamps to −12 (front); the rear sees 0 [M :592-704].
    let mut c = craft();
    c.body.vel = [0.0, 0.0, 10.0];
    let mut inp = SubInput { pitch: 1.0, ..Default::default() };
    c.controller(&mut inp);
    let a = c.body.force[1] / c.body.mass;
    assert!((a + 12.0).abs() < 1e-4, "{a}");
    // With a steer term ≥ 0.01 and no pitch the block is skipped: no y force at all.
    let mut c = craft();
    c.body.vel = [0.0, 0.0, 10.0];
    let mut inp = SubInput { steer: 0.5, ..Default::default() };
    c.controller(&mut inp);
    assert!(c.body.force[1].abs() < 1e-3, "{:?}", c.body.force);
    assert!(c.slip > 0.0 && (c.slip - 0.125 * 0.125 * ((1.0 - 10.0 / 15.0) * 3.0 + 1.0)).abs() < 1e-6, "slip {}", c.slip);
}

#[test]
fn the_reverse_latch() {
    // Brake held at a standstill: the counter reloads 30 on release, so the first braked tick at
    // forward speed ≤ 0 sets reversing [M 0x190fe8-0x1910b4].
    let mut c = craft();
    let mut inp = SubInput { brake: 1.0, ..Default::default() };
    c.controller(&mut inp);
    assert!(!c.reversing, "counter starts at 0: counts first");
    let mut c = craft();
    c.controller(&mut SubInput::default());
    assert_eq!(c.brake_count, 0x1e);
    let mut inp = SubInput { brake: 1.0, ..Default::default() };
    c.controller(&mut inp);
    assert!(c.reversing);
    // Moving forward ≥ 2.5 m/s clears the counter.
    let mut c = craft();
    c.body.vel = [0.0, 0.0, 3.0];
    let mut inp = SubInput { brake: 1.0, ..Default::default() };
    c.controller(&mut inp);
    assert_eq!(c.brake_count, 0);
}

#[test]
fn the_integrator_clamps_and_rotates() {
    let rigid = Rigid::game_defaults();
    // M_LIMIT 100: the momentum clamps per axis BEFORE the force [M micro 0xad/0xb1].
    let mut b = Body::new(1660.0, [1e-3; 3], far(), 0.0);
    b.vel = [150.0, -150.0, 50.0];
    b.integrate(&rigid, DT);
    assert_eq!(b.vel, [100.0, -100.0, 50.0]);
    // AM_LIMIT 10 on ω [M micro 0xc0/0xc4], and L is rebuilt from the clamped ω.
    let mut b = Body::new(1660.0, [1e-3; 3], far(), 0.0);
    b.ang_mom = [0.0, 1e6, 0.0];
    b.integrate(&rigid, DT);
    assert!((b.ang_vel[1] - 10.0).abs() < 1e-5, "{:?}", b.ang_vel);
    assert!((b.ang_mom[1] - 10.0 / 1e-3).abs() < 1e-1, "{:?}", b.ang_mom);
    // ω = 1 rad/s about +y for one second turns the nose by ~1 rad (q += ½(ω dt)⊗q).
    let mut b = Body::new(1660.0, [1e-3; 3], far(), 0.0);
    b.ang_mom = [0.0, 1.0 / 1e-3, 0.0];
    for _ in 0..60 {
        b.integrate(&rigid, DT);
    }
    let yaw = b.rows[2][0].atan2(b.rows[2][2]);
    assert!((yaw - 1.0).abs() < 1e-2, "yaw {yaw}");
    // Position uses the NEW velocity (semi-implicit): one tick of a = 6 from rest.
    let mut b = Body::new(1.0, [1.0; 3], [0.0; 3], 0.0);
    b.add_accel([6.0, 0.0, 0.0]);
    b.integrate(&rigid, DT);
    assert!((b.pos[0] - 6.0 * DT * DT).abs() < 1e-8);
}

#[test]
fn the_roll_leveller_rights_a_banked_craft() {
    // A craft banked 30° with no input rolls back toward level: the −10·m·(up'·r0) torque on the
    // body z axis plus the −2·L drag [M :705-858].
    let rigid = Rigid::game_defaults();
    let mut c = craft();
    let a = 30f32.to_radians() * 0.5;
    c.body.quat = [0.0, 0.0, a.sin(), a.cos()];
    c.body.integrate(&rigid, 0.0); // rebuild the rows from the quaternion
    let bank0 = c.body.rows[0][1].abs();
    for _ in 0..240 {
        c.step(&SubInput::default(), &rigid, DT);
    }
    let bank = c.body.rows[0][1].abs();
    assert!(bank0 > 0.45 && bank < 0.05, "bank {bank0} → {bank}");
}

#[test]
fn a_land_vehicle_still_takes_the_car_step() {
    let mut v = Vehicle::player(VehicleAttrs::parse_atr(PATROLBOAT), CarParams::vanquish(), [1.0; 3], [0.0; 3], 0.0, 0);
    let input = CarInput { throttle: 1.0, ..Default::default() };
    for _ in 0..60 {
        v.step(&input, 0.0, &FlatWorld::paved(), &Rigid::game_defaults(), DT);
    }
    assert!(v.pos()[2] > 0.0);
}
