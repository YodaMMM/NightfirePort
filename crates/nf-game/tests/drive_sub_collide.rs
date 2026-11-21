//! Unit tests for `nf_game::driving::sub_collide` (DRIVING.ELF `FUN_001fc2f0` / `FUN_001fb318` /
//! `FUN_001fc198`) and the start speed (`FUN_0015c738`). Every expected value is written from the
//! decompile with the game's constants as literals, so changing a constant in the port turns a
//! test red.

use nf_game::driving::physics::{Tuning, DT};
use nf_game::driving::sub_collide::*;
use nf_game::driving::sub_physics::*;

const I3: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn close(a: f32, b: f32, eps: f32) -> bool { (a - b).abs() <= eps }
fn close3(a: [f32; 3], b: [f32; 3], eps: f32) -> bool { (0..3).all(|k| close(a[k], b[k], eps)) }

fn floor(y: f32, flags: u16) -> ([[f32; 3]; 3], u16) { ([[-50.0, y, -50.0], [50.0, y, -50.0], [0.0, y, 50.0]], flags) }

#[test]
fn hull_has_the_flush_corners_with_the_018_pad_then_the_nose_and_side_points() {
    let o = hull_offsets(&I3, [1.0, 2.0, 3.0]);
    // [M FUN_001f8cf0 :88-200] corner order and the +0.18 on x; [M FUN_001fc2f0 :152-185] 8-10.
    let want = [
        [-1.18, -2.0, -3.0],
        [1.18, -2.0, -3.0],
        [1.18, -2.0, 3.0],
        [-1.18, -2.0, 3.0],
        [-1.18, 2.0, 3.0],
        [1.18, 2.0, 3.0],
        [1.18, 2.0, -3.0],
        [-1.18, 2.0, -3.0],
        [0.0, 0.0, 3.05],
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
    ];
    for i in 0..11 {
        assert!(close3(o[i], want[i], 1e-6), "point {i}: {:?} want {:?}", o[i], want[i]);
    }
    // A body turned 90° about y (nose = +x): the nose point follows row 2.
    let rows = [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
    let o = hull_offsets(&rows, [1.0, 2.0, 3.0]);
    assert!(close3(o[8], [3.05, 0.0, 0.0], 1e-6));
    assert!(close3(o[2], [3.0, -2.0, -1.18], 1e-6));
    assert!(close(body_radius([1.0, 2.0, 2.0]), 3.0, 1e-6));
}

#[test]
fn segment_query_normal_is_turned_up_capped_and_toward_the_start() {
    let m = WorldMesh::new([floor(0.0, 0)]);
    let h = m.segment([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], QUERY_CLASS_MASK).unwrap();
    assert!(close(h.t, 0.5, 1e-6) && close3(h.point, [0.0, 0.0, 0.0], 1e-6));
    // y >= 0.9999 → 0x3f7ff972 [M 0x21699c-0x2169b8].
    assert_eq!(h.normal[1].to_bits(), 0x3f7f_f972);
    // From below: turned toward the start [M 0x216a38-0x216ab4].
    let h = m.segment([0.0, -1.0, 0.0], [0.0, 1.0, 0.0], QUERY_CLASS_MASK).unwrap();
    assert_eq!(h.normal[1], -f32::from_bits(0x3f7f_f972));
    // A segment that ends short of the plane: no hit.
    assert!(m.segment([0.0, 1.0, 0.0], [0.0, 0.1, 0.0], QUERY_CLASS_MASK).is_none());
    // A sloped face keeps its own normal (only a near-vertical one is capped).
    let slope = WorldMesh::new([([[-10.0, -10.0, -10.0], [10.0, 10.0, -10.0], [0.0, 0.0, 10.0]], 0)]);
    let h = slope.segment([0.0, 5.0, 0.0], [0.0, -5.0, 0.0], QUERY_CLASS_MASK).unwrap();
    let s = 0.5f32.sqrt();
    assert!(close3(h.normal, [-s, s, 0.0], 1e-5), "{:?}", h.normal);
}

#[test]
fn segment_query_skips_class_0x20_and_lets_a_surface_just_behind_the_start_hide_one_ahead() {
    // class byte 0x20 (flags high byte) is masked [M FUN_001fc2f0 :251 + FUN_00213d68].
    let masked = WorldMesh::new([floor(0.0, 0x20 << 8)]);
    assert!(masked.segment([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], QUERY_CLASS_MASK).is_none());
    let other = WorldMesh::new([floor(0.0, 0x10 << 8 | 7)]);
    let h = other.segment([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], QUERY_CLASS_MASK).unwrap();
    assert_eq!((h.surface, h.class), (7, 0x10));
    // A plane 0.5 behind the start (d = −0.5 > −1) wins and then fails 0 <= t: no hit
    // [M FUN_00213d68 `-1.0 < d`, FUN_00231af0 `t < 0 → 0`].
    let two = WorldMesh::new([floor(0.0, 0), floor(1.5, 0)]);
    assert!(two.segment([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], QUERY_CLASS_MASK).is_none());
    // 1.5 behind (d = −1.5): ignored, the floor ahead is the hit.
    let two = WorldMesh::new([floor(0.0, 0), floor(2.5, 0)]);
    assert!(close(two.segment([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], QUERY_CLASS_MASK).unwrap().t, 0.5, 1e-6));
    // The nearer of two ahead.
    let two = WorldMesh::new([floor(-0.5, 0), floor(0.5, 0)]);
    assert!(close(two.segment([0.0, 1.0, 0.0], [0.0, -1.0, 0.0], QUERY_CLASS_MASK).unwrap().t, 0.25, 1e-6));
}

#[test]
fn building_tuning_is_the_shipped_default_tun_and_the_restitution_slope_is_coeff_over_micro() {
    let b = Building::game_defaults();
    assert_eq!(b, Building {
        micro_threshold: 25.0,
        friction_factor: 0.0045,
        coeff_restitution: 0.01,
        friction_limit: 0.125,
        force_limit: 10.0,
        pf_scale: 10.0,
    });
    let t = Tuning::parse("MICRO_THRESHOLD=10\r\nBUILDING_MICRO_THRESHOLD=25\r\nBUILDING_FRICTION_FACTOR=0.0045\r\nBUILDING_COEFF_RESTITUTION=0.01\r\nBUILDING_FRICTION_LIMIT=0.125\r\nBUILDING_FORCE_LIMIT=10\r\nPF_SCALE=10\r\n");
    assert_eq!(Building::from_tun(&t), b);
    assert!(close(b.restitution_slope(), 0.0004, 1e-9));
}

fn rest_body(mass: f32, half: [f32; 3], pos: [f32; 3]) -> Body { Body::new(mass, inv_inertia(mass, half), pos, 0.0) }

#[test]
fn response_impulse_matches_fun_001fb318_with_the_game_constants() {
    let half = [0.8, 0.5, 2.0];
    let mut b = rest_body(1000.0, half, [0.0, 1.0, 0.0]);
    b.vel = [0.0, -5.0, 0.0];
    let inv = b.inv_inertia;
    let n = [0.0, 1.0, 0.0];
    let hit = [0.5, 0.0, 0.5];
    let depth = 0.2;
    let s = respond(&mut b, half[0], depth, 1.0, n, hit, &Building::game_defaults());
    // r = (0.5, −1, 0.5), u = v, vn = −5; W = diag(inv) (identity rows).
    // (W(r×n))×r · n = 0.25·(ix + iz); h = sqrt(0.5 − 0)/hx.
    let den = 0.25 * (inv[0] + inv[2]) + 0.5f32.sqrt() / 0.8;
    let j = -(0.01 / 25.0 * 5.0 + 1.0) * (-5.0 - depth * 10.0) / den;
    assert!(j < 10.0, "the case stays under BUILDING_FORCE_LIMIT: {j}");
    // friction: u·0.0045 = (0, −0.0225, 0), inside ±0.125.
    let vy = -5.0 + j + 0.0225;
    assert!(close3(b.vel, [0.0, vy, 0.0], 1e-4), "{:?} want vy {vy}", b.vel);
    assert!(close(s, j * 0.5 / 10.0, 1e-5), "strength {s}");
    // τ = r × m·Δv, local z ×0.15; ω = W·L.
    let dp = 1000.0 * (j + 0.0225);
    let tq = [-(0.5) * dp, 0.0, 0.5 * dp * 0.15];
    assert!(close3(b.ang_mom, tq, 1e-2), "{:?} want {:?}", b.ang_mom, tq);
    assert!(close3(b.ang_vel, [tq[0] * inv[0], 0.0, tq[2] * inv[2]], 1e-5));
}

#[test]
fn response_clamps_the_impulse_and_the_friction_and_ignores_a_separating_point() {
    let half = [1.0, 0.5, 2.0];
    // Fast and deep: j above 10 → Δv.n = 10; sideways 40 m/s → friction 40·0.0045 = 0.18 → 0.125.
    let mut b = rest_body(1000.0, half, [0.0, 1.0, 0.0]);
    b.vel = [40.0, -20.0, 0.0];
    let s = respond(&mut b, half[0], 1.0, 1.0, [0.0, 1.0, 0.0], [0.5, 0.0, 0.5], &Building::game_defaults());
    assert!(close3(b.vel, [40.0 - 0.125, -20.0 + 10.0 + 0.09, 0.0], 1e-4), "{:?}", b.vel);
    assert_eq!(s, 1.0);
    // Moving away from the surface: untouched, strength 0 [M 0x1fb3e8].
    let mut b = rest_body(1000.0, half, [0.0, 1.0, 0.0]);
    b.vel = [0.0, 3.0, 0.0];
    let s = respond(&mut b, half[0], 0.5, 1.0, [0.0, 1.0, 0.0], [0.5, 0.0, 0.5], &Building::game_defaults());
    assert_eq!((s, b.vel, b.ang_mom), (0.0, [0.0, 3.0, 0.0], [0.0; 3]));
    // A point straight below the centre: r×n = 0 and h = 0 → den 0 → no normal impulse, only
    // the friction term [M 0x1fb54c `c.eq.s` → j = 0].
    let mut b = rest_body(1000.0, half, [0.0, 1.0, 0.0]);
    b.vel = [0.0, -5.0, 0.0];
    let s = respond(&mut b, half[0], 0.5, 1.0, [0.0, 1.0, 0.0], [0.0, 0.0, 0.0], &Building::game_defaults());
    assert!(close3(b.vel, [0.0, -5.0 + 0.0225, 0.0], 1e-6) && s == 0.0, "{:?} {s}", b.vel);
}

#[test]
fn damage_is_sixty_times_the_strength_above_02() {
    assert_eq!(world_damage(0.2), None);
    assert_eq!(world_damage(0.1), None);
    assert!(close(world_damage(0.5).unwrap(), 30.0, 1e-5));
    assert!(close(world_damage(1.0).unwrap(), 60.0, 1e-5));
}

fn wall(z: f32) -> ([[f32; 3]; 3], u16) { ([[-30.0, -30.0, z], [30.0, -30.0, z], [0.0, 30.0, z]], 0) }

#[test]
fn nose_into_a_wall_at_low_speed_sets_the_0x29e_flag_and_fast_does_not() {
    let half = [1.0, 0.5, 2.0];
    let m = WorldMesh::new([wall(2.02)]);
    let mut b = rest_body(1660.0, half, [0.0, 0.0, 0.0]);
    b.vel = [0.0, 0.0, 2.0];
    let out = collide_world(&mut b, half, &m, &Building::game_defaults(), DT);
    let pts: Vec<usize> = out.contacts.iter().map(|c| c.point_index).collect();
    // Corner 2 reaches z 2 + 2/60 past the wall first; its response slows the body, and the
    // velocity is re-read per point [M :219-232], so corners 3-5 then stop short; the nose point
    // (2.05 + v·dt) still reaches.
    assert_eq!(pts, vec![2, 8], "{out:?}");
    assert!(out.nose_stuck);
    assert!(b.vel[2] < 2.0, "pushed back: {:?}", b.vel);
    for c in &out.contacts {
        assert!(close3(c.hit.normal, [0.0, 0.0, -1.0], 1e-6));
    }
    let mut b = rest_body(1660.0, half, [0.0, 0.0, 0.0]);
    b.vel = [0.0, 0.0, 8.0];
    let out = collide_world(&mut b, half, &m, &Building::game_defaults(), DT);
    assert!(!out.contacts.is_empty() && !out.nose_stuck);
    // Surface 0x0e: an event, no response.
    let m = WorldMesh::new([([[-30.0, -30.0, 2.02], [30.0, -30.0, 2.02], [0.0, 30.0, 2.02]], 0x0e)]);
    let mut b = rest_body(1660.0, half, [0.0, 0.0, 0.0]);
    b.vel = [0.0, 0.0, 2.0];
    let out = collide_world(&mut b, half, &m, &Building::game_defaults(), DT);
    assert_eq!((out.contacts.len(), out.event_hits.clone(), b.vel), (0, vec![2, 3, 4, 5, 8], [0.0, 0.0, 2.0]));
}

#[test]
fn side_points_respond_at_a_quarter_and_never_hurt() {
    let half = [1.0, 0.5, 2.0];
    // A small plate at x = 1.1 around the side point only (the corners at y ±0.5, z ±2 miss it).
    let m = WorldMesh::new([([[1.1, -0.2, -0.2], [1.1, -0.2, 0.3], [1.1, 0.3, 0.0]], 0)]);
    let mut b = rest_body(1660.0, half, [0.0, 0.0, 0.0]);
    b.vel = [9.0, 0.0, 0.0];
    let before = b.clone();
    let out = collide_world(&mut b, half, &m, &Building::game_defaults(), DT);
    let side = out.contacts.iter().find(|c| c.point_index == 9).expect("point 9 hits");
    assert_eq!(side.damage, None);
    assert_eq!(out.contacts.len(), 1);
    // Weight 0.25 and a quarter of the depth [M FUN_001fc2f0 :348-356].
    let end = [1.0 + 9.0 * DT, 0.0, 0.0];
    let d = ((end[0] - side.hit.point[0]).powi(2) + (end[1] - side.hit.point[1]).powi(2) + (end[2] - side.hit.point[2]).powi(2)).sqrt();
    let mut want = before.clone();
    let ws = respond(&mut want, half[0], d * 0.25, 0.25, side.hit.normal, side.hit.point, &Building::game_defaults());
    assert!(close(side.strength, ws, 1e-6) && close3(b.vel, want.vel, 1e-6), "{} vs {ws}", side.strength);
    let mut full = before.clone();
    let fs = respond(&mut full, half[0], d, 1.0, side.hit.normal, side.hit.point, &Building::game_defaults());
    assert!(!close(fs, ws, 1e-4));
    assert!(out.contacts.iter().all(|c| c.point_index != 10));
}

#[test]
fn start_mission_sets_the_velocity_to_the_direction_times_the_argument() {
    let attrs = VehicleAttrs { mass: 1660.0, sub_physics: 1, ..VehicleAttrs::game_defaults() };
    let mut c = SubCraft::new(attrs, [5.344, 1.19, 2.998], [0.0; 3], 0.0, 0);
    c.start_mission([3.0, 0.0, 4.0], 35.0);
    assert!(close3(c.body.vel, [21.0, 0.0, 28.0], 1e-5));
    // The momentum the game wrote (dir·arg·mass) over the mass: the first integration keeps it
    // (35 < M_LIMIT 100 per axis).
    c.step(&SubInput::default(), &nf_game::driving::Rigid::game_defaults(), DT);
    assert!(c.body.vel[2] > 27.0 && c.body.vel[2] < 28.0);
}

#[test]
fn step_world_keeps_a_sinking_craft_above_a_floor() {
    let attrs = VehicleAttrs { mass: 1660.0, sub_physics: 1, ..VehicleAttrs::game_defaults() };
    let half = [0.919, 0.578, 2.317];
    let m = WorldMesh::new([floor(0.0, 0)]);
    let mut c = SubCraft::new(attrs, half, [0.0, 1.0, 0.0], 0.0, 0);
    c.body.vel = [0.0, -6.0, 0.0];
    let rigid = nf_game::driving::Rigid::game_defaults();
    let b = Building::game_defaults();
    let mut hits = 0;
    for _ in 0..240 {
        let prev = c.body.pos;
        hits += c.step_world(&SubInput { throttle: 1.0, ..Default::default() }, &rigid, &b, &m, DT).contacts.len();
        assert!(m.crossing(prev, c.body.pos, QUERY_CLASS_MASK).is_none(), "passed through at {:?}", c.body.pos);
        assert!(c.body.pos[1] > 0.0);
    }
    assert!(hits > 0);
}

#[test]
fn step_world_stops_full_throttle_at_a_wall_that_the_plain_step_flies_through() {
    let attrs = VehicleAttrs { mass: 1660.0, sub_physics: 1, ..VehicleAttrs::game_defaults() };
    let half = [0.919, 0.578, 2.317];
    let m = WorldMesh::new([wall(30.0)]);
    let rigid = nf_game::driving::Rigid::game_defaults();
    let b = Building::game_defaults();
    let input = SubInput { throttle: 1.0, ..Default::default() };
    let mut c = SubCraft::new(attrs.clone(), half, [0.0, 0.0, 0.0], 0.0, 0);
    let mut hits = 0;
    for _ in 0..600 {
        let prev = c.body.pos;
        hits += c.step_world(&input, &rigid, &b, &m, DT).contacts.len();
        assert!(m.crossing(prev, c.body.pos, QUERY_CLASS_MASK).is_none(), "passed through at {:?}", c.body.pos);
    }
    assert!(hits > 0 && c.body.pos[2] < 30.0, "{:?}", c.body.pos);
    let mut c = SubCraft::new(attrs, half, [0.0, 0.0, 0.0], 0.0, 0);
    for _ in 0..600 {
        c.step(&input, &rigid, DT);
    }
    assert!(c.body.pos[2] > 30.0, "{:?}", c.body.pos);
}
