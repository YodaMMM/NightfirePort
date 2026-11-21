//! The driving traffic car checked against the DRIVING.ELF decompile, function by function
//! (docs/systems/driving-ai.md §9): the drive control FUN_00133ab0, the target speed
//! FUN_0012e808, the lookahead FUN_00132f80, the spawn lane FUN_0013c5e8, the navigator join
//! FUN_002257c0 / advance FUN_00224e00 / lane points FUN_002267d8, the kind-1 next-segment pick
//! FUN_00223f98 and the driving generator FUN_0017dc28. The fixtures are hand-built format-exact
//! RNgp articles; the ISO-gated test reads every real track and the ELF's lane tables.

use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;

use nf_game::driving::ai::{
    drive_control, gated_stop, lookahead, next_segment_traffic, spawn_lane, target_speed, AiCar,
    AiDriver, DriveCommand, DriveRng, RoadNav, SimCar, SpeedInputs, BOX_FLOOR, NAV_KIND_TRAFFIC,
    PROFILE_ROAD_LIMIT, PROFILE_TRAFFIC, STATE_CHASE, STATE_STANDBY, STATE_TRAFFIC, TRAFFIC_SPEED,
};
use nf_game::driving::road::{lane_offset, LaneSet, RoadNet, LANE_TABLE_DEFAULT, LANE_TABLE_SNOW2A};
use nf_game::driving::{CarParams, DT};

// ---------------------------------------------------------------- the fixture builder

struct Buf {
    b: Vec<u8>,
}

impl Buf {
    fn off(&self) -> u32 {
        self.b.len() as u32
    }
    fn u32(&mut self, v: u32) {
        self.b.extend_from_slice(&v.to_le_bytes());
    }
    fn u16(&mut self, v: u16) {
        self.b.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.b.extend_from_slice(&v.to_le_bytes());
    }
    fn pad_to(&mut self, start: u32, rel: u32) {
        while self.off() < start + rel {
            self.b.push(0);
        }
    }
    fn put(&mut self, at: u32, v: &[u8]) {
        self.b[at as usize..at as usize + v.len()].copy_from_slice(v);
    }
}

fn rec(b: &mut Buf, tag: u32, short: bool, size: usize, skip: usize, count: u32, dref: u32) -> u32 {
    let at = b.off();
    b.u32(tag);
    b.u32(((size as u32) << 8) | ((skip as u32) << 5) | short as u32);
    b.u32(count);
    b.u32(dref);
    at
}

#[derive(Clone)]
struct SegSpec {
    a: u16,
    b: u16,
    limit: f32,
    flags: u16,
    lanes: (i8, i8),
    mask: u8,
    route: i16,
}

fn seg(a: u16, b: u16) -> SegSpec {
    SegSpec { a, b, limit: 25.0, flags: 0, lanes: (1, 1), mask: 0b01, route: -1 }
}

/// A format-exact RNgp `.crp`: nodes (position + links), segments with their direction and side
/// rows computed from the node positions (side = (dir.z, 0, −dir.x), the relation every straight
/// real segment shows [D the ISO-gated test]).
fn build(nodes: &[([f32; 3], Vec<u16>)], segs: &[SegSpec]) -> Vec<u8> {
    let mut b = Buf { b: Vec::new() };
    rec(&mut b, 0x4341_5250, false, 0, 1, 0, 0x10);
    let win = 1 + nodes.len() + segs.len();
    rec(&mut b, 0x524e_6770, false, 0, 0, win as u32, 0x20);
    let hd = rec(&mut b, 0x524e_6864, false, 10, 0, 0, 0);
    let rn: Vec<u32> = (0..nodes.len()).map(|i| rec(&mut b, 0x726e_0000 | i as u32, true, 0x18, 0, 0, 0)).collect();
    let rs: Vec<u32> = (0..segs.len()).map(|i| rec(&mut b, 0x7273_0000 | i as u32, true, 0x64, 0, 0, 0)).collect();
    let hp = b.off();
    for v in [nodes.len() as u16, segs.len() as u16, 0, 0, 0] {
        b.u16(v);
    }
    b.put(hd + 12, &hp.to_le_bytes());
    for (i, (p, links)) in nodes.iter().enumerate() {
        let at = b.off();
        b.f32(p[0]);
        b.f32(p[1]);
        b.f32(p[2]);
        b.u16(i as u16); // +0x0c own id
        b.b.push(links.len() as u8); // +0x0e
        b.b.push(0);
        for l in links {
            b.u16(*l);
        }
        b.pad_to(at, 0x18);
        b.put(rn[i] + 12, &at.to_le_bytes());
    }
    for (i, s) in segs.iter().enumerate() {
        let pa = nodes[s.a as usize].0;
        let pb = nodes[s.b as usize].0;
        let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let d = [d[0] / l, d[1] / l, d[2] / l];
        let side = [d[2], 0.0, -d[0]];
        let at = b.off();
        b.pad_to(at, 0x0c);
        b.u16(s.a);
        b.u16(s.b);
        b.pad_to(at, 0x20);
        for v in side {
            b.f32(v);
        }
        b.pad_to(at, 0x30);
        for v in side {
            b.f32(v);
        }
        b.pad_to(at, 0x3c);
        b.b.extend_from_slice(&s.route.to_le_bytes());
        b.u16(i as u16); // +0x3e own id
        for v in d {
            b.f32(v);
        }
        b.f32(s.limit); // +0x4c
        b.u16(s.flags); // +0x50
        b.b.push(s.lanes.0 as u8); // +0x52
        b.b.push(s.lanes.1 as u8); // +0x53
        b.pad_to(at, 0x58);
        b.b.push(s.mask); // +0x58
        b.pad_to(at, 0x64);
        b.put(rs[i] + 12, &at.to_le_bytes());
    }
    b.b
}

fn parse(buf: &[u8]) -> RoadNet<'_> {
    let crp = Crp::parse(buf).expect("fixture CARP");
    RoadNet::parse_crp(&crp).expect("fixture RNgp")
}

/// A straight two-piece road: n0 (0,0,0) — n1 (100,0,0) — n2 (150,0,0); n2 is a dead end.
fn straight() -> Vec<u8> {
    build(
        &[([0.0, 0.0, 0.0], vec![0]), ([100.0, 0.0, 0.0], vec![0, 1]), ([150.0, 0.0, 0.0], vec![1])],
        &[seg(0, 1), seg(1, 2)],
    )
}

fn sim(pos: [f32; 3], vel: [f32; 3]) -> SimCar {
    // facing +x: forward = +x, right = d(forward)/d(yaw) at yaw = π/2 = (0, 0, −1)
    SimCar { pos, vel, forward: [1.0, 0.0, 0.0], right: [0.0, 0.0, -1.0], alive: 1.0 }
}

fn no_rng() -> impl FnMut() -> u16 {
    || panic!("this pick must not draw from the generator")
}

// ---------------------------------------------------------------- FUN_00133ab0

#[test]
fn drive_control_steers_by_the_side_row_not_the_facing() {
    let s = sim([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    // Aim dead ahead: lat = dot(dir, row0) = 0 → no steer [M 0x133ab0:57-63,135]. The old port
    // steered by dot(dir, forward) = 1 here (full lock on a straight road).
    let c = drive_control(&s, [50.0, 0.0, 0.0], 20.0, false);
    assert_eq!(c.steer, 0.0, "aim on the nose → steer 0");
    // Aim 45° to the right row: lat = dot(normalize(30,0,−30), (0,0,−1)) = √½.
    let c = drive_control(&s, [30.0, 0.0, -30.0], 20.0, false);
    assert!((c.steer - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6, "steer = lat, got {}", c.steer);
    // Aim behind (fa = dot(forward, dir) ≤ −0.2): full lock to lat's side [M 0x133ab0:131-133].
    let c = drive_control(&s, [-30.0, 0.0, -10.0], 20.0, false);
    assert_eq!(c.steer, 1.0, "behind and right → +1");
    let c = drive_control(&s, [-30.0, 0.0, 10.0], 20.0, false);
    assert_eq!(c.steer, -1.0, "behind and left → −1");
    // fa = −0.1 (> −0.2): no lock, steer = lat.
    let dir = [-0.1f32, 0.0, -(1.0f32 - 0.01).sqrt()];
    let c = drive_control(&s, [dir[0] * 40.0, 0.0, dir[2] * 40.0], 20.0, false);
    assert!((c.steer - (-dir[2])).abs() < 1e-5, "steer = lat below the lock threshold, got {}", c.steer);
}

#[test]
fn drive_control_pedals_match_the_disassembly() {
    let aim = [100.0, 0.0, 0.0];
    // ramp = max(min(|Δ|,7.5)/7.5, 0.05) on the throttle while speed < target'.
    let c = drive_control(&sim([0.0; 3], [20.0, 0.0, 0.0]), aim, 25.0, false);
    assert!((c.throttle - 5.0 / 7.5).abs() < 1e-6 && c.brake == 0.0, "{c:?}");
    // The brake hold: span ≤ 1 with target' ≥ 2.5 → no pedal [M 0x133ab0:158-160].
    let c = drive_control(&sim([0.0; 3], [20.0, 0.0, 0.0]), aim, 19.5, false);
    assert_eq!((c.throttle, c.brake), (0.0, 0.0));
    // span ≤ 1 with target' < 2.5 → brake: speed 2.9, target 2.0 → span 0.9 → ramp 0.12.
    let c = drive_control(&sim([0.0; 3], [2.9, 0.0, 0.0]), aim, 2.0, false);
    assert!((c.brake - 0.9 / 7.5).abs() < 1e-6 && c.throttle == 0.0, "{c:?}");
    // At rest inside the arrival radius: target' = 0, speed 0 → not < → va = 0 ≤ 0 → THROTTLE
    // at the 0.05 floor [0x133ea8 c.olt.s 0,va / bc1fl → 0x133f00]. The old port issued nothing.
    let c = drive_control(&sim([97.0, 0.0, 0.0], [0.0; 3]), aim, 7.5, false);
    assert_eq!(c, DriveCommand { steer: 0.0, throttle: 0.05, brake: 0.0, handbrake: false });
    // Rolling backwards at 3 toward a far aim with target 0.5 (floor): speed 3 ≥ 0.5 and va = −1
    // → throttle(min(2.5, 7.5)/7.5), never the brake.
    let c = drive_control(&sim([0.0; 3], [-3.0, 0.0, 0.0]), aim, 0.0, false);
    assert!((c.throttle - 2.5 / 7.5).abs() < 1e-6 && c.brake == 0.0, "{c:?}");
    // The forced stop [M 0x133ab0:173-180].
    let c = drive_control(&sim([0.0; 3], [10.0, 0.0, 0.0]), aim, 7.5, true);
    assert_eq!(c, DriveCommand { steer: 0.0, throttle: 0.0, brake: 0.0, handbrake: true });
}

#[test]
fn the_gated_stop_creeps_at_a_quarter() {
    // FUN_00130508:74-176: ≥ 0.1 → min(|v|, 0.25) on the brake (forward) or throttle (backward).
    assert_eq!(gated_stop(&sim([0.0; 3], [0.05, 0.0, 0.0])), DriveCommand::IDLE);
    assert_eq!(gated_stop(&sim([0.0; 3], [0.2, 0.0, 0.0])).brake, 0.2);
    assert_eq!(gated_stop(&sim([0.0; 3], [5.0, 0.0, 0.0])).brake, 0.25);
    assert_eq!(gated_stop(&sim([0.0; 3], [-5.0, 0.0, 0.0])).throttle, 0.25);
}

// ---------------------------------------------------------------- FUN_0012e808 / FUN_00132f80

#[test]
fn traffic_targets_7_5_not_the_segment_limit() {
    let base = SpeedInputs {
        state: STATE_TRAFFIC,
        profile: PROFILE_TRAFFIC,
        alive: 1.0,
        stop_query: false,
        stop_byte: false,
        seg_limit: 25.0,
        speed_mul: 1.0,
        follow_target: false,
    };
    // Profile 0xc stores 0x40f00000 [M 0x12e808]; the old port returned rs+0x4c (25 here, and 0
    // on 451 of Paris's 811 pieces [D]).
    assert_eq!(target_speed(&base), Some(TRAFFIC_SPEED));
    assert_eq!(TRAFFIC_SPEED, 7.5);
    assert_eq!(target_speed(&SpeedInputs { profile: PROFILE_ROAD_LIMIT, speed_mul: 0.8, ..base }), Some(20.0));
    assert_eq!(target_speed(&SpeedInputs { profile: 0x5, ..base }), Some(0.0), "unlisted profile → 0");
    assert_eq!(target_speed(&SpeedInputs { profile: 0xd, ..base }), None, "chase arm not carried");
    assert_eq!(target_speed(&SpeedInputs { alive: 0.4, ..base }), Some(0.0));
    assert_eq!(target_speed(&SpeedInputs { state: STATE_CHASE, alive: 0.0, ..base }), Some(0.0));
    assert_eq!(target_speed(&SpeedInputs { stop_query: true, ..base }), Some(0.0));
    assert_eq!(target_speed(&SpeedInputs { stop_byte: true, ..base }), Some(0.0));
}

#[test]
fn lookahead_is_the_larger_of_speed_and_box() {
    assert_eq!(lookahead(10.0, BOX_FLOOR), BOX_FLOOR + 7.5);
    assert_eq!(lookahead(40.0, BOX_FLOOR), 40.0 * 0.35);
    assert_eq!(lookahead(10.0, 2.0), 9.5);
}

// ---------------------------------------------------------------- lanes

#[test]
fn lane_offsets_and_spawn_lanes() {
    assert_eq!(lane_offset(LaneSet::Default, 0, 1), 2.0);
    assert_eq!(lane_offset(LaneSet::Default, 0, -2), -6.0);
    assert_eq!(lane_offset(LaneSet::Snow2a, 0, 1), 2.5);
    assert_eq!(lane_offset(LaneSet::Default, 1, 1), 3.0);

    let mut s = seg(0, 1);
    s.lanes = (2, 2);
    s.mask = 0b0011;
    let buf = build(&[([0.0; 3], vec![0]), ([100.0, 0.0, 0.0], vec![0])], &[s]);
    let net = parse(&buf);
    // k = count % 4; c = k + (k ≥ 2); lane = 2 − c; bit k of 0b0011 → forward, else negated
    // and B→A [M 0x13c5e8].
    let got: Vec<_> = (0..4).map(|k| spawn_lane(&net, 0, k).unwrap().unwrap()).collect();
    assert_eq!(got, vec![(2, true), (1, true), (1, false), (2, false)]);
}

// ---------------------------------------------------------------- the navigator

#[test]
fn join_puts_the_carrot_on_the_lane_line() {
    let buf = straight();
    let net = parse(&buf);
    // side = (dir.z, 0, −dir.x) = (0,0,−1) for +x; lane 1, class 0 → offset 2.
    let n = RoadNav::join(&net, 0, 0.25, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    assert_eq!((n.end, n.t, n.lane_off), (1, 0.25, 2.0));
    assert_eq!(n.p0, [0.0, 0.0, -2.0]);
    assert_eq!(n.p1, [100.0, 0.0, -2.0]);
    assert_eq!(n.pos, [25.0, 0.0, -2.0]);
    assert_eq!(n.dir, [1.0, 0.0, 0.0]);
    // B→A: t' = |1 − t|, P0 = B − side_b·off, P1 = A − side_a·off [M 0x2257c0:77-91, 0x2267d8].
    let n = RoadNav::join(&net, 0, 0.25, 1, false, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    assert_eq!((n.end, n.t), (0, 0.75));
    assert_eq!(n.p0, [100.0, 0.0, 2.0]);
    assert_eq!(n.pos, [25.0, 0.0, 2.0], "same A→B place, the other lane side");
    assert_eq!(n.dir, [-1.0, 0.0, 0.0]);
}

#[test]
fn advance_carries_the_leftover_distance_into_the_next_piece() {
    let buf = straight();
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 0, 0.9, 0, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    // 10 units finish seg0, 10 more go on seg1 (50 long) → t = 0.2 [M 0x224e00:120-182]. The
    // plain link is taken without a draw.
    n.advance(&net, 20.0, &mut no_rng()).unwrap();
    assert_eq!(n.seg, 1);
    assert_eq!(n.end, 1, "seg1 leaves n1 through node A → toward B");
    assert!((n.t - 0.2).abs() < 1e-5, "t = {}", n.t);
    assert!((n.pos[0] - 110.0).abs() < 1e-3, "x = {}", n.pos[0]);
    assert!(!n.repeated);
    // Within a piece the carrot is lerp(P0, P1, t), clamped at 1 only by the crossing.
    n.advance(&net, 5.0, &mut no_rng()).unwrap();
    assert!((n.pos[0] - 115.0).abs() < 1e-3);
}

#[test]
fn a_dead_end_turns_the_navigator_around_and_the_car_stops() {
    let buf = straight();
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 1, 0.9, 0, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    // n2 has one link → same piece, end flips, +0x94 set [M 0x223f98:120-123, 0x224e00:145-147].
    n.advance(&net, 10.0, &mut no_rng()).unwrap();
    assert_eq!((n.seg, n.end, n.repeated), (1, 0, true));
    assert_eq!(n.dir, [-1.0, 0.0, 0.0]);
    // The brain aims at the car itself once the navigator repeats [M 0x130508:297-303].
    let mut car = AiCar::traffic(&net, 1, 0.0, 0, true, LaneSet::Default).unwrap();
    car.nav = n;
    let s = sim([120.0, 0.0, 0.0], [7.0, 0.0, 0.0]);
    let cmd = car.step(&net, &s, &mut no_rng()).unwrap().unwrap();
    assert!(cmd.brake > 0.0 && cmd.throttle == 0.0, "arrival at its own position → brake: {cmd:?}");
}

/// A junction: the car comes in on seg0 (n0→n1); n1's other links are connectors (flag bit 0)
/// to n2 and n3, each continued by a plain road.
fn junction(count2: bool) -> Vec<u8> {
    let mut c1 = seg(1, 2);
    c1.flags = 0x0001;
    let mut c2 = seg(1, 3);
    c2.flags = 0x0001;
    let r1 = seg(2, 4);
    let r2 = seg(3, 5);
    let n1_links = if count2 { vec![0, 1] } else { vec![0, 1, 2] };
    build(
        &[
            ([0.0, 0.0, 0.0], vec![0]),
            ([100.0, 0.0, 0.0], n1_links),
            ([110.0, 0.0, 10.0], vec![1, 3]),
            ([110.0, 0.0, -10.0], vec![2, 4]),
            ([110.0, 0.0, 100.0], vec![3]),
            ([110.0, 0.0, -100.0], vec![4]),
        ],
        &[seg(0, 1), c1, c2, r1, r2],
    )
}

#[test]
fn a_plain_link_is_taken_before_any_junction_logic() {
    // n1 links: seg0 (current), seg1 (connector), seg2 (plain) → seg2, no draw [M 0x2270e8].
    let mut c1 = seg(1, 2);
    c1.flags = 1;
    let buf = build(
        &[([0.0; 3], vec![0]), ([100.0, 0.0, 0.0], vec![0, 1, 2]), ([110.0, 0.0, 10.0], vec![1]), ([200.0, 0.0, 0.0], vec![2])],
        &[seg(0, 1), c1, seg(1, 3)],
    );
    let net = parse(&buf);
    let n = RoadNav::join(&net, 0, 0.5, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    let p = next_segment_traffic(&net, &n, &mut no_rng()).unwrap();
    assert_eq!((p.seg, p.end, p.lane_change), (2, 1, false));
}

#[test]
fn a_junction_pick_draws_and_takes_at_three_tenths() {
    let buf = junction(true);
    let net = parse(&buf);
    let n = RoadNav::join(&net, 0, 0.5, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    // Two links (current + one connector): the "straight" block takes the connector on a draw
    // below 0.3·65536; a draw at or above it falls to the slot-1 rule (lane 1 on a flag-free
    // road → kept) and draws again [M 0x223f98].
    let mut draws = 0;
    let p = next_segment_traffic(&net, &n, &mut || {
        draws += 1;
        0
    })
    .unwrap();
    assert_eq!((p.seg, draws), (1, 1), "taken on the first draw");
    let mut draws = 0;
    let p = next_segment_traffic(&net, &n, &mut || {
        draws += 1;
        19661 // 19661/65536 = 0.30000 ≥ 0.3 → not taken
    })
    .unwrap();
    assert_eq!((p.seg, draws), (1, 2), "the last eligible candidate stands after two draws");
    assert_eq!(p.end, 1, "the connector leaves n1 through its node A");
    // 19660/65536 = 0.29999 < 0.3 → taken.
    let mut draws = 0;
    next_segment_traffic(&net, &n, &mut || {
        draws += 1;
        19660
    })
    .unwrap();
    assert_eq!(draws, 1);

    // A closed lane mask on the road past the connector bars it: parity 1 (entered at its
    // node A) needs bit 0 set → mask 0 → no candidate → the current piece repeats.
    let mut c1 = seg(1, 2);
    c1.flags = 1;
    let mut r1 = seg(2, 3);
    r1.mask = 0;
    let buf = build(
        &[([0.0; 3], vec![0]), ([100.0, 0.0, 0.0], vec![0, 1]), ([110.0, 0.0, 10.0], vec![1, 2]), ([110.0, 0.0, 100.0], vec![2])],
        &[seg(0, 1), c1, r1],
    );
    let net = parse(&buf);
    let n = RoadNav::join(&net, 0, 0.5, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    let p = next_segment_traffic(&net, &n, &mut no_rng()).unwrap();
    assert_eq!(p.seg, 0, "no open road past the junction → the same piece");
}

#[test]
fn the_three_way_junction_uses_the_slot_rules() {
    let buf = junction(false);
    let net = parse(&buf);
    let n = RoadNav::join(&net, 0, 0.5, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    // n1 links [seg0, seg1, seg2]: slot 1 → the near rule (lane 1 kept), slot 2 of 3 → the far
    // rule (lane 1 = cur lanes+ → r lanes+ = 1). Never-take draws: both are eligible, two draws,
    // the last one (slot 2) stands.
    let mut draws = 0;
    let p = next_segment_traffic(&net, &n, &mut || {
        draws += 1;
        0xffff
    })
    .unwrap();
    assert_eq!((p.seg, draws, p.lane), (2, 2, 1));
    // An always-take draw stops at slot 1.
    let p = next_segment_traffic(&net, &n, &mut || 0).unwrap();
    assert_eq!(p.seg, 1);
}

#[test]
fn the_driving_generator_matches_fun_0017dc28() {
    let mut r = DriveRng { state: 1, mult: 0x075b_cd15, draws: 0 };
    // p = 1·0x075bcd15 → state = 0xcd15, out = (p >> 8) & 0xffff = 0x5bcd.
    assert_eq!(r.next(), 0x5bcd);
    assert_eq!(r.state, 0xcd15);
    let r = DriveRng::game_reset();
    assert_eq!(r.mult % 50, 39, "the reset warm-up count");
    assert!(r.state <= 0xffff, "the state is masked after the warm-up");
}

// ---------------------------------------------------------------- the wired car

/// A civilian traffic car: the steering/engine keys of the shipped `hatchback.atr` (MAXSTEERING
/// 0.2, MODSTEERSPEED 100, MINSTEER 1, MAXACC 13.29, WHEEL_ZF/ZR −0.82/1.08 [D MIS01.VIV]); the
/// keys not listed keep the vanquish numbers.
fn hatchback() -> CarParams {
    CarParams {
        maxsteering: 0.2,
        modsteer_speed: 100.0,
        min_steer: 1.0,
        maxacc: 13.29,
        wheel_zf_offset: -0.82,
        wheel_zr_offset: 1.08,
        ..CarParams::vanquish()
    }
}

/// An L corner: n0 (0,0,0) → n1 (100,0,0) → n2 (100,0,100), single lanes driving A→B.
fn corner() -> Vec<u8> {
    build(
        &[([0.0; 3], vec![0]), ([100.0, 0.0, 0.0], vec![0, 1]), ([100.0, 0.0, 100.0], vec![1])],
        &[seg(0, 1), seg(1, 2)],
    )
}

fn dist_to_lane(p: [f32; 3]) -> f32 {
    // The lane-1 line: offset 2 on the side row (dir.z, 0, −dir.x) — seg0 runs (0,−2)→(100,−2),
    // seg1 (102,0)→(102,100); the carrot jumps the gap between them at the crossing.
    let pts = [[0.0f32, -2.0], [100.0, -2.0], [102.0, 0.0], [102.0, 100.0]];
    let q = [p[0], p[2]];
    let mut best = f32::MAX;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let ab = [b[0] - a[0], b[1] - a[1]];
        let t = (((q[0] - a[0]) * ab[0] + (q[1] - a[1]) * ab[1]) / (ab[0] * ab[0] + ab[1] * ab[1])).clamp(0.0, 1.0);
        let c = [a[0] + ab[0] * t, a[1] + ab[1] * t];
        best = best.min(((q[0] - c[0]).powi(2) + (q[1] - c[1]).powi(2)).sqrt());
    }
    best
}

#[test]
fn a_traffic_car_follows_the_corner_at_seven_and_a_half() {
    let buf = corner();
    let net = parse(&buf);
    let mut d = AiDriver::on_segment(hatchback(), &net, 0, 1, 0).unwrap();
    // spawn count 0 on a (1,1) piece, mask bit 0 set → lane 1 forward [M 0x13c5e8].
    assert_eq!((d.brain.nav.lane, d.brain.nav.end), (1, 1));
    assert_eq!(d.car.pos, [0.0, 0.0, -2.0], "placed on the carrot");
    let mut worst = 0.0f32;
    let mut top = 0.0f32;
    let mut reached = false;
    for _ in 0..(60 * 40) {
        d.step(&net, DT).unwrap();
        worst = worst.max(dist_to_lane(d.car.pos));
        top = top.max(d.car.speed);
        if d.car.pos[2] > 60.0 {
            reached = true;
            break;
        }
    }
    assert!(reached, "the car drove round the corner: pos {:?}", d.car.pos);
    assert_eq!(d.brain.nav.seg, 1);
    assert!(worst < 6.0, "the car keeps near its lane line: worst {worst}");
    assert!(top > 6.5 && top < 8.5, "the cruise is the 7.5 traffic target: top {top}");
}

#[test]
fn standby_commands_nothing() {
    let buf = corner();
    let net = parse(&buf);
    let mut d = AiDriver::on_segment(hatchback(), &net, 0, 1, 0).unwrap();
    d.brain.state = STATE_STANDBY;
    for _ in 0..60 {
        assert_eq!(d.step(&net, DT).unwrap(), DriveCommand::IDLE);
    }
    assert_eq!(d.car.speed, 0.0);
}

// ---------------------------------------------------------------- the real disc (NF_ISO)

#[test]
fn real_tracks_and_lane_tables() {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return;
    };
    let iso = IsoImage::open(&path).unwrap();

    // The lane tables in the ELF's .data [D 0x33a618 / 0x33a678, the single PT_LOAD].
    if let Some(e) = iso.entries().iter().find(|e| e.path.ends_with("/DRIVING.ELF")) {
        let elf = iso.read_file(e).unwrap();
        let phoff = u32::from_le_bytes(elf[0x1c..0x20].try_into().unwrap()) as usize;
        let off = u32::from_le_bytes(elf[phoff + 4..phoff + 8].try_into().unwrap());
        let va = u32::from_le_bytes(elf[phoff + 8..phoff + 12].try_into().unwrap());
        let rd = |a: u32| -> f32 {
            let o = (a - va + off) as usize;
            f32::from_le_bytes(elf[o..o + 4].try_into().unwrap())
        };
        for c in 0..4 {
            for k in 0..6 {
                assert_eq!(rd(0x33a618 + (c * 6 + k) as u32 * 4), LANE_TABLE_SNOW2A[c][k], "snow2a {c},{k}");
                assert_eq!(rd(0x33a678 + (c * 6 + k) as u32 * 4), LANE_TABLE_DEFAULT[c][k], "default {c},{k}");
            }
        }
        let mult_rd = |a: u32| -> u32 {
            let o = (a - va + off) as usize;
            u32::from_le_bytes(elf[o..o + 4].try_into().unwrap())
        };
        // FUN_0017db68: lui v1,0x075b (0x17db68) / ori v1,v1,0xcd15 (0x17db70) — the multiplier.
        let w0 = mult_rd(0x17db68);
        let w1 = mult_rd(0x17db70);
        assert_eq!((w0 & 0xffff, w1 & 0xffff), (0x075b, 0xcd15), "the generator ctor constant");
        eprintln!("{}: lane tables and the generator constant match", e.path);
    } else {
        eprintln!("no DRIVING.ELF on this image — the table check is skipped");
    }

    let mut tracks = 0;
    for e in iso.entries().iter().filter(|e| e.path.starts_with("/DRIVING/") && e.path.ends_with(".VIV")) {
        let bigf = nf_formats::disc::read_bigf(&iso, e).unwrap();
        let base = e.lba as u64 * 2048;
        let Some(ce) = bigf.entries.iter().find(|x| {
            let n = x.name.to_ascii_lowercase();
            n.contains("track") && n.ends_with(".crp")
        }) else {
            continue;
        };
        let raw = iso.read_bytes(base + ce.offset as u64, ce.size as u64).unwrap();
        let img = if refpack::is_refpack(&raw) { refpack::decompress(&raw).unwrap() } else { raw };
        let crp = Crp::parse(&img).unwrap();
        let net = RoadNet::parse_crp(&crp).unwrap();
        if net.segs.is_empty() {
            continue;
        }
        tracks += 1;
        for (i, n) in net.nodes.iter().enumerate() {
            assert_eq!(n.own_id() as usize, i, "{}: node +0x0c is its index", e.path);
        }
        for s in &net.segs {
            for v in [s.side_a(), s.side_b()] {
                let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
                assert!((l - 1.0).abs() < 1e-3, "{}: side rows are unit, got {l}", e.path);
            }
        }
        // A traffic car spawned on every lane-bearing piece runs 10 s through the real network
        // (junction connectors, dead ends, lane masks) without a chooser error, and moves.
        let mut moved = 0;
        let mut spawned = 0;
        for i in (0..net.segs.len()).step_by(7) {
            if spawn_lane(&net, i, 0).unwrap().is_none() {
                continue;
            }
            let mut d = AiDriver::on_segment(hatchback(), &net, i, 1, 0).unwrap();
            spawned += 1;
            let start = d.car.pos;
            for _ in 0..600 {
                d.step(&net, DT).unwrap_or_else(|err| panic!("{} seg {i}: {err}", e.path));
            }
            let p = d.car.pos;
            let moved_by = ((p[0] - start[0]).powi(2) + (p[2] - start[2]).powi(2)).sqrt();
            if moved_by > 20.0 {
                moved += 1;
            }
        }
        let ys: Vec<f32> = net.nodes.iter().map(|n| n.pos()[1]).collect();
        let span = ys.iter().cloned().fold(f32::MIN, f32::max) - ys.iter().cloned().fold(f32::MAX, f32::min);
        eprintln!("{}: {} pieces, {spawned} spawned, {moved} moved > 20 units in 10 s, node height span {span:.1}", e.path, net.segs.len());
        // The flat test world holds the car at y = 0, so the carrot gap includes the road's
        // height; only Paris (the traffic city, a flat road net) is held to the drive check.
        if e.path.ends_with("/MIS01.VIV") {
            assert!(moved * 4 >= spawned * 3, "{}: most traffic cars drive ({moved}/{spawned})", e.path);
        }
    }
    assert!(tracks >= 6, "the driving tracks with a road network: {tracks}");
}
