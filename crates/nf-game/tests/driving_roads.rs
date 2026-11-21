//! The driving road network logic checked against the DRIVING.ELF decompile
//! (docs/systems/driving-ai.md §11): the curve object (FUN_001a8380 / FUN_001a85d0 with the
//! .data 0x37e4a0 basis) and the navigator's lane control points (FUN_00226538), the chase
//! chooser FUN_00223a30 and the follow chooser FUN_002235c8, the junction lights FUN_0013ef40 /
//! FUN_0013efc8, the yield FUN_001377c8 with FUN_00135530 / FUN_00135978, the road grid
//! FUN_0021cf80, the spawn director FUN_0013d618 / FUN_0013ca60 / FUN_0013c860 / FUN_0013c798, the
//! respawn FUN_00132808 / FUN_00132ba8 and the route count of FUN_00132d80. The fixtures are
//! hand-built format-exact RNgp articles; the expected numbers are worked by hand from the code.
//! The ISO-gated tests read every real track and run Paris traffic for 3600 ticks.

use nf_formats::crp::Crp;
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;

use nf_game::driving::ai::{
    cell_of, next_segment_chase, next_segment_follow, path_blocked, respawn_step, stop_short, traffic_yield, AiCar,
    JunctionLights, NavHint, Respawn, RoadGrid, RoadNav, SimCar, TrafficDirector, TrafficView, TrafficWorld, Vehicle,
    YieldWorld, NAV_KIND_TRAFFIC, PROFILE_PARKED, PROFILE_TRAFFIC,
};
use nf_game::driving::road::{bezier, lane_offset, LaneSet, RoadNet, RSF_CURVED};
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
    fn f3(&mut self, v: [f32; 3]) {
        for x in v {
            self.f32(x);
        }
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

fn rec(b: &mut Buf, tag: u32, short: bool, size: usize, count: u32) -> u32 {
    rec_skip(b, tag, short, size, 0, count)
}

fn rec_skip(b: &mut Buf, tag: u32, short: bool, size: usize, skip: u32, count: u32) -> u32 {
    let at = b.off();
    b.u32(tag);
    b.u32(((size as u32) << 8) | (skip << 5) | short as u32);
    b.u32(count);
    b.u32(0);
    at
}

#[derive(Clone)]
struct SegSpec {
    a: u16,
    b: u16,
    flags: u16,
    lanes: (i8, i8),
    mask: u8,
    route: i16,
    ctrl: Option<([f32; 3], [f32; 3], f32)>,
    junctions: (i16, i16),
}

fn seg(a: u16, b: u16) -> SegSpec {
    SegSpec { a, b, flags: 0, lanes: (1, 1), mask: 0b01, route: -1, ctrl: None, junctions: (-1, -1) }
}

struct JnSpec {
    pos: [f32; 3],
    neighbours: Vec<i16>,
    roads: Vec<i16>,
}

/// A format-exact RNgp `.crp`: nodes, segments (direction and side rows from the node positions:
/// side = (dir.z, 0, −dir.x)), junction (`ri`) and route (`rr`, cap at +0x2c) records.
fn build(nodes: &[([f32; 3], Vec<u16>)], segs: &[SegSpec], jns: &[JnSpec], route_caps: &[u8]) -> Vec<u8> {
    let mut b = Buf { b: Vec::new() };
    rec_skip(&mut b, 0x4341_5250, false, 0, 1, 0);
    b.put(12, &0x10u32.to_le_bytes());
    let win = 1 + nodes.len() + segs.len() + jns.len() + route_caps.len();
    let art = rec(&mut b, 0x524e_6770, false, 0, win as u32);
    b.put(art + 12, &0x20u32.to_le_bytes());
    let hd = rec(&mut b, 0x524e_6864, false, 10, 0);
    let rn: Vec<u32> = (0..nodes.len()).map(|i| rec(&mut b, 0x726e_0000 | i as u32, true, 0x18, 0)).collect();
    let rs: Vec<u32> = (0..segs.len()).map(|i| rec(&mut b, 0x7273_0000 | i as u32, true, 0x64, 0)).collect();
    let ri: Vec<u32> = (0..jns.len()).map(|i| rec(&mut b, 0x7269_0000 | i as u32, true, 0x30, 0)).collect();
    let rr: Vec<u32> = (0..route_caps.len()).map(|i| rec(&mut b, 0x7272_0000 | i as u32, true, 0x30, 0)).collect();
    let hp = b.off();
    for v in [nodes.len() as u16, segs.len() as u16, jns.len() as u16, route_caps.len() as u16, 0] {
        b.u16(v);
    }
    b.put(hd + 12, &hp.to_le_bytes());
    for (i, (p, links)) in nodes.iter().enumerate() {
        let at = b.off();
        b.f3(*p);
        b.u16(i as u16);
        b.b.push(links.len() as u8);
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
        let (c1, c2, chord) = s.ctrl.unwrap_or(([0.0; 3], [0.0; 3], l));
        b.f3(c1);
        b.u16(s.a);
        b.u16(s.b);
        b.f3(c2);
        b.b.extend_from_slice(&s.junctions.0.to_le_bytes());
        b.b.extend_from_slice(&s.junctions.1.to_le_bytes());
        b.f3(side);
        b.f32(chord); // +0x2c
        b.f3(side);
        b.pad_to(at, 0x3c);
        b.b.extend_from_slice(&s.route.to_le_bytes());
        b.u16(i as u16);
        b.f3(d);
        b.f32(25.0);
        b.u16(s.flags);
        b.b.push(s.lanes.0 as u8);
        b.b.push(s.lanes.1 as u8);
        b.pad_to(at, 0x58);
        b.b.push(s.mask);
        b.pad_to(at, 0x64);
        b.put(rs[i] + 12, &at.to_le_bytes());
    }
    for (i, j) in jns.iter().enumerate() {
        let at = b.off();
        b.f3(j.pos);
        b.u16(i as u16);
        b.b.push(j.neighbours.len() as u8);
        b.b.push(0);
        for n in &j.neighbours {
            b.b.extend_from_slice(&n.to_le_bytes());
        }
        b.pad_to(at, 0x18);
        b.b.push(j.roads.len() as u8);
        b.b.push(0);
        for r in &j.roads {
            b.b.extend_from_slice(&r.to_le_bytes());
        }
        b.pad_to(at, 0x30);
        b.put(ri[i] + 12, &at.to_le_bytes());
    }
    for (i, cap) in route_caps.iter().enumerate() {
        let at = b.off();
        b.pad_to(at, 0x2c);
        b.b.push(*cap);
        b.pad_to(at, 0x30);
        b.put(rr[i] + 12, &at.to_le_bytes());
    }
    b.b
}

fn parse(buf: &[u8]) -> RoadNet<'_> {
    let crp = Crp::parse(buf).expect("fixture CARP");
    RoadNet::parse_crp(&crp).expect("fixture RNgp")
}

fn near(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= eps)
}

fn no_rng() -> impl FnMut() -> u16 {
    || panic!("this pick must not draw from the generator")
}

fn sim(pos: [f32; 3], vel: [f32; 3]) -> SimCar {
    SimCar { pos, vel, forward: [1.0, 0.0, 0.0], right: [0.0, 0.0, -1.0], alive: 1.0 }
}

// ---------------------------------------------------------------- curves

#[test]
fn the_curve_object_is_the_bezier_of_the_0x37e4a0_basis() {
    let p = [[0.0, 0.0, 0.0], [0.0, 0.0, 10.0], [10.0, 0.0, 20.0], [20.0, 0.0, 20.0]];
    assert_eq!(bezier(p, 0.0), p[0]);
    assert_eq!(bezier(p, 1.0), p[3]);
    // t = 0.5: weights 1/8, 3/8, 3/8, 1/8.
    assert!(near(bezier(p, 0.5), [6.25, 0.0, 13.75], 1e-5), "{:?}", bezier(p, 0.5));
    // t = 0.25: weights 27/64, 27/64, 9/64, 1/64.
    assert!(near(bezier(p, 0.25), [1.71875, 0.0, 7.34375], 1e-5), "{:?}", bezier(p, 0.25));
}

/// One curved piece n0 (0,0,0) → n1 (20,0,20) with its control points at rs+0x00 / rs+0x10.
fn curved_piece() -> Vec<u8> {
    let mut s = seg(0, 1);
    s.flags = RSF_CURVED;
    s.ctrl = Some(([0.0, 0.0, 10.0], [10.0, 0.0, 20.0], 28.284271));
    build(&[([0.0; 3], vec![0]), ([20.0, 0.0, 20.0], vec![0])], &[s], &[], &[])
}

#[test]
fn a_curved_piece_runs_through_both_control_points() {
    let buf = curved_piece();
    let net = parse(&buf);
    // P0 = node A, P1 = rs+0x00, P2 = rs+0x10, P3 = node B [M 0x222d40 → 0x1a8380].
    assert!(near(net.seg_point(0, 0.5).unwrap(), [6.25, 0.0, 13.75], 1e-5));
    assert!(near(net.curve_point(0, 0.25).unwrap(), [1.71875, 0.0, 7.34375], 1e-5));
}

/// A curved piece A (0,0,0) → B (10,0,0): side rows (0,0,−1); controls (3,0,4) / (7,0,4); the
/// +0x2c chord written as 20 (twice the node gap) so the scale is visible.
fn lane_curve() -> Vec<u8> {
    let mut s = seg(0, 1);
    s.flags = RSF_CURVED;
    s.ctrl = Some(([3.0, 0.0, 4.0], [7.0, 0.0, 4.0], 20.0));
    let mut next = seg(1, 2);
    next.flags = 0;
    build(&[([0.0; 3], vec![0]), ([10.0, 0.0, 0.0], vec![0, 1]), ([30.0, 0.0, 0.0], vec![1])], &[s, next], &[], &[])
}

#[test]
fn the_navigator_offsets_the_control_points_by_the_chord_ratio() {
    let buf = lane_curve();
    let net = parse(&buf);
    // Lane 1, class 0 → offset 2: P0 = (0,0,−2), P1 = (10,0,−2), |P0 − P1| = 10, k = 20/10 = 2;
    // c1 = P0 + n(3,0,4)·5/2 = (1.5,0,0); c2 = P1 + n(−3,0,4)·5/2 = (8.5,0,0) [M 0x226538].
    let n = RoadNav::join(&net, 0, 0.5, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    assert!(near(n.p0, [0.0, 0.0, -2.0], 1e-6) && near(n.p1, [10.0, 0.0, -2.0], 1e-6));
    assert!(near(n.c1, [1.5, 0.0, 0.0], 1e-5), "{:?}", n.c1);
    assert!(near(n.c2, [8.5, 0.0, 0.0], 1e-5), "{:?}", n.c2);
    // The carrot at t = 0.5 is the Bezier of the lane points: (5, 0, −0.5), not the chord's
    // midpoint (5, 0, −2).
    assert!(near(n.pos, [5.0, 0.0, -0.5], 1e-5), "{:?}", n.pos);

    // Toward A (forward = false, t' = |1 − 0.5|): P0 = B − side·2 = (10,0,2), P1 = (0,0,2);
    // c1 from rs+0x10 / node B: (10,0,2) + n(−3,0,4)·2.5 = (8.5,0,4); c2 = (1.5,0,4).
    let n = RoadNav::join(&net, 0, 0.5, 1, false, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    assert!(near(n.c1, [8.5, 0.0, 4.0], 1e-5) && near(n.c2, [1.5, 0.0, 4.0], 1e-5), "{:?} {:?}", n.c1, n.c2);
    assert!(near(n.pos, [5.0, 0.0, 3.5], 1e-5), "{:?}", n.pos);
}

#[test]
fn advancing_on_a_curve_moves_along_the_bezier_and_crosses_by_the_chord() {
    let buf = lane_curve();
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 0, 0.0, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    n.advance(&net, 2.5, &mut no_rng()).unwrap();
    // t = 2.5 / |P0 − P1| = 0.25 → weights 27/64, 27/64, 9/64, 1/64.
    assert_eq!(n.t, 0.25);
    assert!(near(n.pos, [127.0 / 64.0, 0.0, -0.875], 1e-5), "{:?}", n.pos);
    // 10 more units: 7.5 finish the piece (by the chord), 2.5 carry into the straight one.
    n.advance(&net, 10.0, &mut no_rng()).unwrap();
    assert_eq!(n.seg, 1);
    assert!(near(n.pos, [12.5, 0.0, -2.0], 1e-5), "{:?}", n.pos);
}

// ---------------------------------------------------------------- the choosers

/// The traffic suite's junction: seg0 n0→n1; n1 links seg0, the connectors seg1 (→ n2, road seg3
/// to +z) and seg2 (→ n3, road seg4 to −z).
fn junction3(js: Vec<JnSpec>, cur_j: (i16, i16), r_js: [(i16, i16); 2]) -> Vec<u8> {
    let mut c0 = seg(0, 1);
    c0.junctions = cur_j;
    let mut c1 = seg(1, 2);
    c1.flags = 1;
    let mut c2 = seg(1, 3);
    c2.flags = 1;
    let mut r1 = seg(2, 4);
    r1.junctions = r_js[0];
    let mut r2 = seg(3, 5);
    r2.junctions = r_js[1];
    build(
        &[
            ([0.0, 0.0, 0.0], vec![0]),
            ([100.0, 0.0, 0.0], vec![0, 1, 2]),
            ([110.0, 0.0, 10.0], vec![1, 3]),
            ([110.0, 0.0, -10.0], vec![2, 4]),
            ([110.0, 0.0, 100.0], vec![3]),
            ([110.0, 0.0, -100.0], vec![4]),
        ],
        &[c0, c1, c2, r1, r2],
        &js,
        &[],
    )
}

#[test]
fn the_chase_chooser_takes_the_connector_whose_road_points_along_the_hint() {
    let buf = junction3(vec![], (-1, -1), [(-1, -1), (-1, -1)]);
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 0, 0.5, 1, true, 3, LaneSet::Default).unwrap();
    let mut h = [0.0, 0.0, -7.0];
    assert_eq!(next_segment_chase(&net, &mut n, &mut h, -1).unwrap(), (2, 1));
    assert_eq!(h, [0.0, 0.0, -1.0], "the hint is normalised in place");
    let mut h = [3.0, 0.0, 1.0];
    assert_eq!(next_segment_chase(&net, &mut n, &mut h, -1).unwrap(), (1, 1));
    // A pending slot (+0xb0) at a 3-link node wins over the hint and is cleared.
    n.slot = 2;
    let mut h = [0.0, 0.0, 1.0];
    assert_eq!(next_segment_chase(&net, &mut n, &mut h, -1).unwrap(), (2, 1));
    assert_eq!(n.slot, -1);
}

#[test]
fn the_chase_chooser_routes_toward_the_target_through_the_junction_records() {
    // J0 at n1 (the exit end of seg0), neighbours J1, J2; its roads seg3 (ends at J1) and seg4
    // (ends at J2). The target piece is seg4 (junction J2 at its node B): of J0's neighbours, J2
    // is an end of the target, seg4 ends at J2, and the exit link whose far node lists seg4 is
    // seg2 — taken although the hint points the other way [M 0x223a30].
    let js = vec![
        JnSpec { pos: [100.0, 0.0, 0.0], neighbours: vec![1, 2], roads: vec![3, 4] },
        JnSpec { pos: [110.0, 0.0, 100.0], neighbours: vec![0], roads: vec![3] },
        JnSpec { pos: [110.0, 0.0, -100.0], neighbours: vec![0], roads: vec![4] },
    ];
    let buf = junction3(js, (-1, 0), [(0, 1), (0, 2)]);
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 0, 0.5, 1, true, 3, LaneSet::Default).unwrap();
    let mut h = [0.0, 0.0, 1.0];
    assert_eq!(next_segment_chase(&net, &mut n, &mut h, 4).unwrap(), (2, 1));
    // No target → the hint decides (seg1, toward +z).
    let mut h = [0.0, 0.0, 1.0];
    assert_eq!(next_segment_chase(&net, &mut n, &mut h, -1).unwrap(), (1, 1));
    // The neighbour behind (rs+0x1c of seg0 when driving toward B) is skipped: make J2 the one
    // behind and the routing finds nothing → the hint decides.
    let js = vec![
        JnSpec { pos: [100.0, 0.0, 0.0], neighbours: vec![1, 2], roads: vec![3, 4] },
        JnSpec { pos: [110.0, 0.0, 100.0], neighbours: vec![0], roads: vec![3] },
        JnSpec { pos: [110.0, 0.0, -100.0], neighbours: vec![0], roads: vec![4] },
    ];
    let buf = junction3(js, (2, 0), [(0, 1), (0, 2)]);
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 0, 0.5, 1, true, 3, LaneSet::Default).unwrap();
    let mut h = [0.0, 0.0, 1.0];
    assert_eq!(next_segment_chase(&net, &mut n, &mut h, 4).unwrap(), (1, 1));
}

#[test]
fn the_follow_chooser_draws_once_among_the_continuing_links() {
    let buf = junction3(vec![], (-1, -1), [(-1, -1), (-1, -1)]);
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 0, 0.5, 1, true, 4, LaneSet::Default).unwrap();
    // Two links continue (seg1, seg2: node B ≠ n1); k = (2·r) >> 16.
    let mut draws = 0;
    let mut h = [0.0; 3];
    let s = next_segment_follow(&net, &mut n, &mut h, &mut || {
        draws += 1;
        0x8000
    })
    .unwrap();
    assert_eq!((s, draws), (2, 1));
    let s = next_segment_follow(&net, &mut n, &mut h, &mut || 0x7fff).unwrap();
    assert_eq!(s, 1);
    // A hint flag (+0xb1) picks by direction, no draw.
    n.hint_toward = true;
    let mut h = [0.1, 0.0, -1.0];
    assert_eq!(next_segment_follow(&net, &mut n, &mut h, &mut no_rng()).unwrap(), 2);
    assert_eq!(n.end, 1);
}

#[test]
fn chase_and_follow_navigators_advance_through_the_junction() {
    let buf = junction3(vec![], (-1, -1), [(-1, -1), (-1, -1)]);
    let net = parse(&buf);
    let mut n = RoadNav::join(&net, 0, 0.9, 0, true, 3, LaneSet::Default).unwrap();
    n.advance_with(&net, 15.0, NavHint { dir: [0.0, 0.0, -1.0], target: -1 }, &mut no_rng()).unwrap();
    assert_eq!(n.seg, 2, "kind 3 crossed onto the −z connector");
    let mut n = RoadNav::join(&net, 0, 0.9, 0, true, 4, LaneSet::Default).unwrap();
    n.advance_with(&net, 15.0, NavHint::NONE, &mut || 0).unwrap();
    assert_eq!(n.seg, 1, "kind 4 drew 0 → the first continuing link");
}

// ---------------------------------------------------------------- lights and the yield

#[test]
fn the_junction_lights_cycle_every_300_ticks_with_a_150_tick_all_red() {
    let mut l = JunctionLights::new(0);
    assert_eq!((l.phase, l.deadline), (0, 300));
    assert!(l.may_go([0.0; 3], 0, 3, 0, &[]));
    assert!(!l.may_go([0.0; 3], 1, 3, 0, &[]));
    assert!(l.may_go([0.0; 3], 0, 3, 150, &[]), "deadline − 150 = 150 is not < 150");
    assert!(!l.may_go([0.0; 3], 0, 3, 151, &[]), "the all-red tail");
    l.tick(300);
    assert_eq!(l.phase, 0, "300 is not past the deadline");
    l.tick(301);
    assert_eq!((l.phase, l.deadline), (1, 601));
    assert!(l.may_go([0.0; 3], 1, 3, 301, &[]));
    assert!(!l.may_go([0.0; 3], 0, 3, 301, &[]));
    // An object within 50 and not 5 or more below the junction holds every road.
    assert!(!l.may_go([0.0; 3], 1, 3, 301, &[[30.0, 0.0, 30.0]]));
    assert!(l.may_go([0.0; 3], 1, 3, 301, &[[40.0, 0.0, 40.0]]), "56.6 away");
    assert!(l.may_go([0.0; 3], 1, 3, 301, &[[10.0, -5.0, 0.0]]), "junction.y − obj.y = 5 is not < 5");
    assert!(!l.may_go([0.0; 3], 1, 3, 301, &[[10.0, 20.0, 0.0]]), "an object above counts");
}

#[test]
fn the_cell_of_rounds_a_fifth_half_away_from_zero() {
    assert_eq!(cell_of([12.4, -12.4, 2.5]), [2, -2, 1]);
    assert_eq!(cell_of([12.6, -12.6, -2.5]), [3, -3, -1]);
}

#[test]
fn a_car_ahead_blocks_and_the_aim_stops_short_of_it() {
    let me = [0.0; 3];
    assert!(path_blocked(0.001, me, [20.0, 0.0, 0.0], [10.0, 0.0, 0.0]));
    assert!(!path_blocked(0.001, me, [20.0, 0.0, 0.0], [10.0, 0.0, 5.0]), "outside the 0.9 cone");
    assert!(!path_blocked(0.001, me, [20.0, 0.0, 0.0], [25.0, 0.0, 0.0]), "beyond the aim");
    assert!(path_blocked(6.0, me, [20.0, 0.0, 0.0], [25.0, 0.0, 0.0]), "within r of the aim");
    let mut aim = [20.0, 0.0, 0.0];
    stop_short(2.0, me, 1.0, &mut aim, [10.0, 0.0, 0.0]);
    assert_eq!(aim, [6.5, 0.0, 0.0], "10 − 2 − 1.5·1");
    let mut aim = [20.0, 0.0, 0.0];
    stop_short(0.0, me, 1.0, &mut aim, [-10.0, 0.0, 0.0]);
    assert_eq!(aim, [0.0; 3], "a point behind pulls the aim onto the car");
}

fn veh(pos: [f32; 3]) -> Vehicle {
    Vehicle { pos, box_x: 1.0, box_z: 2.0, kind: 3, status: 2, enabled: true, state: 4, alive: 1.0, lane: 1, end: 1 }
}

/// seg0 n0 (0,0,0) → n1 (100,0,0) with flag 0x80 into junction J0 (roads seg0, seg3, seg4).
fn light_net() -> Vec<u8> {
    let js = vec![JnSpec { pos: [100.0, 0.0, 0.0], neighbours: vec![], roads: vec![0, 3, 4] }];
    let mut c0 = seg(0, 1);
    c0.flags = 0x80;
    c0.junctions = (-1, 0);
    let mut c1 = seg(1, 2);
    c1.flags = 1;
    let mut c2 = seg(1, 3);
    c2.flags = 1;
    build(
        &[
            ([0.0, 0.0, 0.0], vec![0]),
            ([100.0, 0.0, 0.0], vec![0, 1, 2]),
            ([110.0, 0.0, 10.0], vec![1, 3]),
            ([110.0, 0.0, -10.0], vec![2, 4]),
            ([110.0, 0.0, 100.0], vec![3]),
            ([110.0, 0.0, -100.0], vec![4]),
        ],
        &[c0, c1, c2, seg(2, 4), seg(3, 5)],
        &js,
        &[],
    )
}

#[test]
fn the_yield_stops_at_a_red_light_and_goes_on_green() {
    let buf = light_net();
    let net = parse(&buf);
    let nav = RoadNav::join(&net, 0, 0.5, 0, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    let s = sim([50.0, 0.0, 0.0], [0.5, 0.0, 0.0]);
    // Phase 1: road index 1 of 3 is green, this piece is index 0 → red.
    let lights = JunctionLights { phase: 1, deadline: 1000 };
    let w = YieldWorld { now: 0, lights, vehicles: &[], me: None, light_objects: &[] };
    let mut aim = [60.0, 0.0, 0.0];
    let mut fs = false;
    let r = traffic_yield(&net, &nav, &s, 0.001, &w, &mut aim, &mut fs).unwrap();
    assert_eq!(r, (true, true));
    assert_eq!(aim, [50.0, 0.0, 0.0], "the piece's other node is behind → the aim is the car");
    assert!(fs, "slow (0.5 ≤ 1) → the forced stop");
    // Green: nothing ahead → +0x194 clear, the forced stop clears.
    let w = YieldWorld { now: 0, lights: JunctionLights { phase: 3, deadline: 1000 }, vehicles: &[], me: None, light_objects: &[] };
    let mut aim = [60.0, 0.0, 0.0];
    let r = traffic_yield(&net, &nav, &s, 0.001, &w, &mut aim, &mut fs).unwrap();
    assert_eq!((r, aim, fs), ((false, false), [60.0, 0.0, 0.0], false));
    // A fast car is not forced to stop (only the aim moves).
    let w = YieldWorld { now: 0, lights, vehicles: &[], me: None, light_objects: &[] };
    let s = sim([50.0, 0.0, 0.0], [5.0, 0.0, 0.0]);
    let mut aim = [60.0, 0.0, 0.0];
    let mut fs = false;
    traffic_yield(&net, &nav, &s, 0.001, &w, &mut aim, &mut fs).unwrap();
    assert!(!fs);
}

#[test]
fn the_yield_stops_behind_the_nearest_car_in_its_path() {
    let buf = light_net();
    let net = parse(&buf);
    let nav = RoadNav::join(&net, 0, 0.5, 0, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    let s = sim([50.0, 0.0, 0.0], [0.5, 0.0, 0.0]);
    let green = JunctionLights { phase: 0, deadline: 1000 };
    // me (index 0) is skipped; a car 12 ahead (r = max(1, 2) = 2) and one 16 ahead; one parked
    // (status 3) at 8 and one outside the cell box (25 ahead: 5 cells).
    let mut parked = veh([58.0, 0.0, 0.0]);
    parked.status = 3;
    let v = [veh([50.0, 0.0, 0.0]), veh([66.0, 0.0, 0.0]), veh([62.0, 0.0, 0.0]), parked, veh([75.0, 0.0, 0.0])];
    let w = YieldWorld { now: 0, lights: green, vehicles: &v, me: Some(0), light_objects: &[] };
    let mut aim = [70.0, 0.0, 0.0];
    let mut fs = false;
    let r = traffic_yield(&net, &nav, &s, 1.0, &w, &mut aim, &mut fs).unwrap();
    assert_eq!(r, (true, false));
    // The 16-ahead car first: 16 − 2 − 1.5 = 12.5 → (62.5); then the 12-ahead one along the
    // working aim: 12 − 2 − 1.5 = 8.5 → (58.5), the nearer one wins.
    assert!(near(aim, [58.5, 0.0, 0.0], 1e-4), "{aim:?}");
    assert!(fs);
}

// ---------------------------------------------------------------- the spawn director

#[test]
fn the_grid_box_walks_columns_then_rows() {
    let g = RoadGrid { min_x: -100.0, min_z: -100.0, inv_cell: 0.1, rows: 20, cols: 20, cells: Default::default() };
    // x: (−15 + 100)·0.1 = 8.5 → 8 .. 11; z the same; cell = z·20 + x, x outer.
    let c = g.cells_in_box([0.0; 3], 15.0);
    assert_eq!(c.len(), 16);
    assert_eq!(&c[..5], &[168, 188, 208, 228, 169]);
    // Clamped at the grid edge.
    let c = g.cells_in_box([-100.0, 0.0, -100.0], 15.0);
    assert_eq!(c[0], 0);
    assert_eq!(c.len(), 4);
}

#[test]
fn the_range_tests_use_the_75_and_150_rings_and_the_view_cone() {
    let buf = light_net();
    let net = parse(&buf);
    let d = TrafficDirector::new(&net, 20, 0);
    let mut v = TrafficView { eye: [0.0; 3], forward: [1.0, 0.0, 0.0], view_test: false, spawning_off: false };
    let p = [0.0; 3];
    assert!(!d.out_of_range(p, &v, [-100.0, 0.0, 0.0]));
    assert!(d.out_of_range(p, &v, [-151.0, 0.0, 0.0]));
    v.view_test = true;
    assert!(d.out_of_range(p, &v, [-100.0, 0.0, 0.0]), "beyond 75 and behind the eye");
    assert!(!d.out_of_range(p, &v, [100.0, 0.0, 70.0]), "dot 0.819 ≥ 0.65: in view");
    assert!(d.out_of_range(p, &v, [100.0, 0.0, 120.0]), "dot 0.64 < 0.65");
    assert!(!d.out_of_range(p, &v, [-70.0, 0.0, 0.0]), "within 75 is always kept");
    v.spawning_off = true;
    assert!(!d.out_of_range(p, &v, [-500.0, 0.0, 0.0]));
    assert!(d.placement_ok(p, 0.001, [151.0, 0.0, 0.0]));
    assert!(!d.placement_ok(p, 1.5, [151.0, 0.0, 0.0]), "151 − 1.5 is not past 150");
}

/// Three straight roads far from each other, one route each (caps 1, 0, 1), and a grid that lists
/// them in cell 0.
fn spawn_net() -> Vec<u8> {
    let mut a = seg(0, 1);
    a.route = 0;
    let mut b = seg(2, 3);
    b.route = 1;
    let mut c = seg(4, 5);
    c.route = 2;
    c.lanes = (1, 2);
    c.mask = 0b010;
    build(
        &[
            ([200.0, 0.0, 0.0], vec![0]),
            ([260.0, 0.0, 0.0], vec![0]),
            ([0.0, 0.0, 200.0], vec![1]),
            ([0.0, 0.0, 260.0], vec![1]),
            ([-200.0, 0.0, 0.0], vec![2]),
            ([-260.0, 0.0, 0.0], vec![2]),
        ],
        &[a, b, c],
        &[],
        &[1, 0, 1],
    )
}

fn spawn_grid() -> RoadGrid {
    let mut g = RoadGrid { min_x: -1000.0, min_z: -1000.0, inv_cell: 0.001, rows: 2, cols: 2, cells: Default::default() };
    g.cells.insert(0, vec![0, 1, 2]);
    g
}

#[test]
fn a_spawn_draws_a_piece_then_a_start_and_honours_the_route_cap() {
    let buf = spawn_net();
    let net = parse(&buf);
    let grid = spawn_grid();
    let view = TrafficView { eye: [0.0; 3], forward: [1.0, 0.0, 0.0], view_test: true, spawning_off: false };
    let mut d = TrafficDirector::new(&net, 20, 0);
    // Draw 1 picks list[(3·r) >> 16]; 0x5556 → 1 (route 1, cap 0) → no spawn, no second draw.
    let mut seq = vec![0x5556u16].into_iter();
    assert!(d.pick(&net, &grid, [0.0; 3], 0, &view, &[], &mut || seq.next().unwrap()).unwrap().is_none());
    assert_eq!(d.list, vec![0, 1, 2]);
    assert_eq!(d.refresh_at, 0x59);
    // 0 → piece 0; the second draw 0x8000 → t = 0.5; lane slot k = (0 + 0) % 2 = 0 → lane 1, mask
    // bit 0 → toward B; the spot (230, 0, −2) is 230 from the player.
    let mut seq = vec![0u16, 0x8000].into_iter();
    let (p, spot) = d.pick(&net, &grid, [0.0; 3], 1, &view, &[], &mut || seq.next().unwrap()).unwrap().unwrap();
    assert_eq!((p.seg, p.lane, p.t, p.forward), (0, 1, 0.5, true));
    assert!(near(spot, [230.0, 0.0, -2.0], 1e-4), "{spot:?}");
    assert_eq!(d.spawn_count, 1);
    // A live traffic car within 4 cells of the spot fails it; the counter stays.
    let v = [veh([240.0, 0.0, 0.0])];
    let mut seq = vec![0u16, 0x8000].into_iter();
    assert!(d.pick(&net, &grid, [0.0; 3], 2, &view, &v, &mut || seq.next().unwrap()).unwrap().is_none());
    assert_eq!(d.spawn_count, 1);
    // Piece 2 (lanes −1/+2, mask 0b010): k = (0 + 1) % 3 = 1 → c = 1 → lane 2 − 1 = 1, bit 1 set
    // → toward B.
    let mut seq = vec![0xffffu16, 0].into_iter();
    let (p, _) = d.pick(&net, &grid, [0.0; 3], 3, &view, &[], &mut || seq.next().unwrap()).unwrap().unwrap();
    assert_eq!((p.seg, p.lane, p.forward), (2, 1, true));
    // A spot nearer than 75 to the player fails.
    let mut seq = vec![0u16, 0].into_iter();
    assert!(d.pick(&net, &grid, [200.0, 0.0, 0.0], 4, &view, &[], &mut || seq.next().unwrap()).unwrap().is_none());
}

#[test]
fn the_respawn_parks_an_out_of_range_car_and_places_it_on_a_spot() {
    let buf = spawn_net();
    let net = parse(&buf);
    let grid = spawn_grid();
    let view = TrafficView { eye: [0.0; 3], forward: [0.0, 0.0, -1.0], view_test: true, spawning_off: false };
    let mut d = TrafficDirector::new(&net, 20, 0);
    let mut car = AiCar::traffic(&net, 2, 0.5, 1, true, LaneSet::Default).unwrap();
    car.route_seen = 2;
    d.occupancy[2] = 1;
    d.live = 1;
    let mut status = 2i8;
    let mut seq = vec![0u16, 0].into_iter();
    // The car at (−230, 0, 0) is beyond 75 and outside the view cone (looking −z): parked, then
    // placed on piece 0 at t = 0 (spot (200, 0, −2), 200 from the player, not seen).
    let r = respawn_step(&mut d, &mut car, &mut status, [-230.0, 0.0, 0.0], &net, &grid, [0.0; 3], 10, &view, &[], &|_, _| false, &mut || seq.next().unwrap()).unwrap();
    match r {
        Respawn::Placed { pos, dir } => {
            assert!(near(pos, [200.0, 0.0, -2.0], 1e-4), "{pos:?}");
            assert_eq!(dir, [1.0, 0.0, 0.0]);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!((status, car.profile, car.nav.kind, d.live, d.occupancy[2], car.route_seen), (2, PROFILE_TRAFFIC, NAV_KIND_TRAFFIC, 1, 0, -1));
    assert_eq!(car.respawn_hold, 10 + 300);
    assert_eq!(d.park_slot, 50.0);
    // Within the hold nothing happens though it is still out of range.
    let r = respawn_step(&mut d, &mut car, &mut status, [-230.0, 0.0, 0.0], &net, &grid, [0.0; 3], 11, &view, &[], &|_, _| false, &mut no_rng()).unwrap();
    assert_eq!(r, Respawn::None);
    // A spot the view test sees is not used; the car stays parked.
    let mut seq = vec![0u16, 0].into_iter();
    let r = respawn_step(&mut d, &mut car, &mut status, [-230.0, 0.0, 0.0], &net, &grid, [0.0; 3], 400, &view, &[], &|_, _| true, &mut || seq.next().unwrap()).unwrap();
    assert_eq!(r, Respawn::Parked([0.0, 0.0, 40100.0]));
    assert_eq!((status, car.profile, d.live), (3, PROFILE_PARKED, 0));
    // At the cap no spot is drawn.
    d.cap = 0;
    let r = respawn_step(&mut d, &mut car, &mut status, [0.0, 0.0, 40100.0], &net, &grid, [0.0; 3], 401, &view, &[], &|_, _| false, &mut no_rng()).unwrap();
    assert_eq!(r, Respawn::None);
}

#[test]
fn the_route_count_follows_the_piece() {
    let buf = spawn_net();
    let net = parse(&buf);
    let mut d = TrafficDirector::new(&net, 20, 0);
    let mut car = AiCar::traffic(&net, 0, 0.5, 1, true, LaneSet::Default).unwrap();
    d.count_route(&net, &mut car).unwrap();
    assert_eq!((d.occupancy.clone(), car.route_seen), (vec![1, 0, 0], 0));
    d.count_route(&net, &mut car).unwrap();
    assert_eq!(d.occupancy, vec![1, 0, 0], "same route: no change");
    car.nav = RoadNav::join(&net, 2, 0.5, 1, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    d.count_route(&net, &mut car).unwrap();
    assert_eq!((d.occupancy.clone(), car.route_seen), (vec![0, 0, 1], 2));
}

// ---------------------------------------------------------------- the real disc (NF_ISO)

fn tracks(iso: &IsoImage<std::fs::File>) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for e in iso.entries().iter().filter(|e| e.path.starts_with("/DRIVING/") && e.path.ends_with(".VIV")) {
        let bigf = nf_formats::disc::read_bigf(iso, e).unwrap();
        let base = e.lba as u64 * 2048;
        let Some(ce) = bigf.entries.iter().find(|x| {
            let n = x.name.to_ascii_lowercase();
            n.contains("track") && n.ends_with(".crp")
        }) else {
            continue;
        };
        let raw = iso.read_bytes(base + ce.offset as u64, ce.size as u64).unwrap();
        let img = if refpack::is_refpack(&raw) { refpack::decompress(&raw).unwrap() } else { raw };
        out.push((e.path.clone(), img));
    }
    out
}

fn open_iso() -> Option<IsoImage<std::fs::File>> {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return None;
    };
    Some(IsoImage::open(&path).unwrap())
}

#[test]
fn real_curves_junctions_and_grids() {
    let Some(iso) = open_iso() else { return };
    // The basis matrices in the disc's DRIVING.ELF .data [D 0x37e4a0 / 0x37e4e0].
    if let Some(e) = iso.entries().iter().find(|e| e.path.ends_with("/DRIVING.ELF")) {
        let elf = iso.read_file(e).unwrap();
        let phoff = u32::from_le_bytes(elf[0x1c..0x20].try_into().unwrap()) as usize;
        let off = u32::from_le_bytes(elf[phoff + 4..phoff + 8].try_into().unwrap());
        let va = u32::from_le_bytes(elf[phoff + 8..phoff + 12].try_into().unwrap());
        let rd = |a: u32| -> f32 {
            let o = (a - va + off) as usize;
            f32::from_le_bytes(elf[o..o + 4].try_into().unwrap())
        };
        for r in 0..4 {
            for c in 0..4 {
                assert_eq!(rd(0x37e4a0 + (r * 4 + c) as u32 * 4), nf_game::driving::road::BEZIER_BASIS[r][c], "basis {r},{c}");
            }
        }
    }
    let mut checked = 0;
    for (path, img) in tracks(&iso) {
        let crp = Crp::parse(&img).unwrap();
        let net = RoadNet::parse_crp(&crp).unwrap();
        if net.segs.is_empty() {
            continue;
        }
        let mut curved = 0;
        let mut ordered = 0;
        let mut chord_ab = 0;
        for (i, s) in net.segs.iter().enumerate() {
            if s.flags() & RSF_CURVED == 0 {
                continue;
            }
            curved += 1;
            let a = net.node(s.node_a() as usize).unwrap().pos();
            let b = net.node(s.node_b() as usize).unwrap().pos();
            let d = |p: [f32; 3], q: [f32; 3]| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
            assert!(s.chord_len() + 1e-2 >= d(a, b), "{path} seg {i}: +0x2c {} is never shorter than |AB| {}", s.chord_len(), d(a, b));
            if (s.chord_len() - d(a, b)).abs() < 1e-2 {
                chord_ab += 1;
            }
            if d(s.curve_ctrl_a(), a) <= d(s.curve_ctrl_a(), b) && d(s.curve_ctrl(), b) <= d(s.curve_ctrl(), a) {
                ordered += 1;
            }
            assert!(near(net.seg_point(i, 0.0).unwrap(), a, 1e-3) && near(net.seg_point(i, 1.0).unwrap(), b, 1e-2));
        }
        assert!(ordered * 4 >= curved * 3, "{path}: rs+0x00 by node A and rs+0x10 by node B on most curves ({ordered}/{curved})");
        for (k, _) in net.ri.iter().enumerate() {
            let j = net.junction(k).unwrap();
            assert_eq!(j.own_id() as usize, k, "{path}: ri +0x0c is its index");
            assert!(j.data[0x0e] <= 4, "{path}: junction {k} lists {} neighbours", j.data[0x0e]);
            for r in j.roads() {
                let s = net.seg(r as usize).unwrap();
                assert!(s.junction_at(0) == k as i16 || s.junction_at(1) == k as i16, "{path}: junction {k} road {r} points back");
            }
        }
        let grid = RoadGrid::parse_crp(&crp).unwrap();
        assert!(grid.cols > 0 && grid.rows > 0 && grid.inv_cell > 0.0);
        let listed: usize = grid.cells.values().map(|l| l.iter().filter(|&&s| s >= 0 && (s as usize) < net.segs.len()).count()).sum();
        assert!(listed >= net.segs.len(), "{path}: every road piece sits in some cell");
        if path.ends_with("/MIS01.VIV") {
            assert_eq!((grid.cols, grid.rows, grid.min_x, grid.min_z), (108, 162, -1664.0, -2880.0));
            assert_eq!(grid.inv_cell, 1.0 / 24.0);
            assert_eq!((net.segs.len(), net.ri.len(), curved, chord_ab), (811, 130, 105, 33));
        }
        eprintln!("{path}: {} pieces, {curved} curved ({ordered} with the controls by their nodes, {chord_ab} with +0x2c = |AB|), {} junctions, grid {}x{} ({} cells)", net.segs.len(), net.ri.len(), grid.cols, grid.rows, grid.cells.len());
        checked += 1;
    }
    assert!(checked >= 6);
}

/// The traffic car parameters (hatchback.atr [D MIS01.VIV]; as the traffic suite uses).
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

/// XZ distance from p to the segment a-b.
fn seg_dist(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
    let l2 = dx * dx + dz * dz;
    let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[2] - a[2]) * dz) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (x, z) = (a[0] + dx * t - p[0], a[2] + dz * t - p[2]);
    (x * x + z * z).sqrt()
}


/// The road network as centre-line polylines (curved pieces sampled along their Bezier) in a
/// 16-unit hash, the XZ distance from a point to it, and the widest lane offset any piece carries.
/// Two limits use it: a navigator carrot must lie within the widest lane line (+0.05) — the road
/// logic itself; a car body within the widest lane line plus one turning-circle diameter of the
/// traffic car (`2·wheelbase / tan(MAXSTEERING)` = 18.8 for the hatchback) — the port's rail
/// physics cannot make Paris's tightest junction turns (connectors of 4 to 8 units at 90°) and
/// swings wide there before the carrot pulls it back [S: a port physics limit, not road logic].
fn road_check<'n>(net: &'n RoadNet) -> (impl Fn([f32; 3]) -> f32 + 'n, f32) {
    let p = hatchback();
    let turn = 2.0 * (p.wheel_zr_offset - p.wheel_zf_offset) / p.maxsteering.tan();
    // The road network as centre-line polylines (curved pieces sampled along their Bezier) in a
    // 16-unit hash, and the widest lane line any piece carries.
    let mut lines: Vec<([f32; 3], [f32; 3])> = Vec::new();
    let mut widest = 0.0f32;
    for i in 0..net.segs.len() {
        let s = net.seg(i).unwrap();
        let steps = if s.flags() & RSF_CURVED != 0 { 16 } else { 1 };
        for k in 0..steps {
            let a = net.seg_point(i, k as f32 / steps as f32).unwrap();
            let b = net.seg_point(i, (k + 1) as f32 / steps as f32).unwrap();
            lines.push((a, b));
        }
        let lanes = s.lanes_pos().max(s.lanes_neg()).max(0);
        widest = widest.max(lane_offset(LaneSet::Default, s.road_class(), lanes).abs());
    }
    let mut hash: std::collections::HashMap<(i32, i32), Vec<usize>> = Default::default();
    for (k, (a, b)) in lines.iter().enumerate() {
        let (x0, x1) = (a[0].min(b[0]) - 48.0, a[0].max(b[0]) + 48.0);
        let (z0, z1) = (a[2].min(b[2]) - 48.0, a[2].max(b[2]) + 48.0);
        for cx in (x0 / 16.0).floor() as i32..=(x1 / 16.0).floor() as i32 {
            for cz in (z0 / 16.0).floor() as i32..=(z1 / 16.0).floor() as i32 {
                hash.entry((cx, cz)).or_default().push(k);
            }
        }
    }
    let off_road = move |p: [f32; 3]| -> f32 {
        let key = ((p[0] / 16.0).floor() as i32, (p[2] / 16.0).floor() as i32);
        hash.get(&key).map(|v| v.iter().map(|&k| seg_dist(p, lines[k].0, lines[k].1)).fold(f32::MAX, f32::min)).unwrap_or(f32::MAX)
    };
    let _ = turn;
    (off_road, widest)
}


#[test]
fn paris_traffic_stays_on_the_road_network_for_3600_ticks() {
    let Some(iso) = open_iso() else { return };
    let (_, img) = tracks(&iso).into_iter().find(|(p, _)| p.ends_with("/MIS01.VIV")).expect("Paris");
    let crp = Crp::parse(&img).unwrap();
    let net = RoadNet::parse_crp(&crp).unwrap();
    let grid = RoadGrid::parse_crp(&crp).unwrap();

    let (off_road, widest) = road_check(&net);
    let hp = hatchback();
    let limit = widest + 2.0 * (hp.wheel_zr_offset - hp.wheel_zf_offset) / hp.maxsteering.tan();

    // The player drives the network on a kind-1 navigator at 20 units/s; the eye is the player.
    let start = (0..net.segs.len()).find(|&i| net.seg(i).unwrap().flags() & 0x8109 == 0).unwrap();
    let mut player_nav = RoadNav::join(&net, start, 0.0, 0, true, NAV_KIND_TRAFFIC, LaneSet::Default).unwrap();
    let mut prng = nf_game::driving::ai::DriveRng::game_reset();
    let mut world = TrafficWorld::new(&net, grid, hatchback(), 20, 20, 0).unwrap();
    let mut worst = 0.0f32;
    let mut live_ticks = 0usize;
    let mut placed = 0usize;
    let mut yield_ticks = 0usize;
    let mut red_ticks = 0usize;
    let mut moved = 0.0f32;
    let mut last: Vec<Option<[f32; 3]>> = vec![None; 20];
    for tick in 0..3600 {
        player_nav.advance(&net, 20.0 * DT, &mut || prng.next()).unwrap();
        let ppos = player_nav.pos;
        let player = Vehicle { pos: ppos, box_x: 1.0, box_z: 2.0, kind: 0, status: 0, enabled: true, state: 0, alive: 1.0, lane: 0, end: 1 };
        let view = TrafficView { eye: ppos, forward: player_nav.dir, view_test: true, spawning_off: false };
        let eye = ppos;
        let fwd = player_nav.dir;
        let visible = move |p: [f32; 3], _r: f32| {
            let d = [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            l < 400.0 && (d[0] * fwd[0] + d[1] * fwd[1] + d[2] * fwd[2]) / l.max(1e-6) >= 0.65
        };
        let before: Vec<i8> = world.cars.iter().map(|c| c.status).collect();
        world.step(&net, player, &view, &visible, DT).unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        for (i, c) in world.cars.iter().enumerate() {
            if c.status != 2 {
                last[i] = None;
                continue;
            }
            if before[i] != 2 {
                placed += 1;
            }
            live_ticks += 1;
            if c.drv.brain.yielding {
                yield_ticks += 1;
            }
            if c.drv.brain.junction_stop {
                red_ticks += 1;
            }
            let p = c.drv.car.pos;
            if let Some(q) = last[i] {
                moved += ((p[0] - q[0]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
            }
            last[i] = Some(p);
            let d = off_road(p);
            worst = worst.max(d);
            assert!(d <= limit, "tick {tick}: car {i} is {d:.2} off the road network at {p:?} (seg {}, limit {limit:.1})", c.drv.brain.nav.seg);
            let cd = off_road(c.drv.brain.nav.pos);
            assert!(cd <= widest + 0.05, "tick {tick}: car {i}'s carrot is {cd:.3} from a centre line");
        }
    }
    eprintln!(
        "Paris 3600 ticks: {placed} placements, {:.1} live cars on average, {:.0} units driven, worst {worst:.2} from a centre line (limit {limit:.1}), {yield_ticks} yielding car-ticks ({red_ticks} at a red light), {} spawn picks",
        live_ticks as f32 / 3600.0,
        moved,
        world.director.spawn_count
    );
    assert!(placed >= 5, "the respawn placed traffic ({placed})");
    assert!(moved > 100.0, "the placed traffic drives ({moved:.0} units)");
}

/// The road check for a dense Paris: 20 live traffic cars put straight onto pieces spread over
/// the city (spawning off — the world flag +0x3cc — so the director neither parks nor places),
/// running the lights, the yield and the junction choosers for 3600 ticks.
#[test]
fn twenty_paris_cars_keep_to_the_roads_through_lights_and_junctions() {
    let Some(iso) = open_iso() else { return };
    let (_, img) = tracks(&iso).into_iter().find(|(p, _)| p.ends_with("/MIS01.VIV")).expect("Paris");
    let crp = Crp::parse(&img).unwrap();
    let net = RoadNet::parse_crp(&crp).unwrap();
    let grid = RoadGrid::parse_crp(&crp).unwrap();
    let (off_road, widest) = road_check(&net);
    let p = hatchback();
    let limit = widest + 2.0 * (p.wheel_zr_offset - p.wheel_zf_offset) / p.maxsteering.tan();
    let mut world = TrafficWorld::new(&net, grid, hatchback(), 0, 20, 0).unwrap();
    let pieces: Vec<usize> = (0..net.segs.len())
        .filter(|&i| {
            let s = net.seg(i).unwrap();
            s.flags() & 0x810b == 0 && s.route() >= 0 && s.lanes_pos() + s.lanes_neg() > 0
        })
        .collect();
    for k in 0..20 {
        let seg = pieces[k * pieces.len() / 20];
        let mut drv = nf_game::driving::ai::AiDriver::on_segment(hatchback(), &net, seg, 1, k as i8).unwrap();
        drv.brain.respawn_hold = i32::MAX;
        world.cars.push(nf_game::driving::ai::TrafficCar { drv, status: 2 });
    }
    world.director.live = 20;
    let player = Vehicle { pos: [0.0, 0.0, 99999.0], box_x: 1.0, box_z: 2.0, kind: 0, status: 0, enabled: true, state: 0, alive: 1.0, lane: 0, end: 1 };
    let view = TrafficView { eye: player.pos, forward: [0.0, 0.0, 1.0], view_test: false, spawning_off: true };
    let mut worst = 0.0f32;
    let mut worst_carrot = 0.0f32;
    let mut off_lane_ticks = 0usize;
    let (mut red, mut yields, mut moved, mut pieces_entered) = (0usize, 0usize, 0.0f32, 0usize);
    let mut last: Vec<([f32; 3], usize)> = world.cars.iter().map(|c| (c.drv.car.pos, c.drv.brain.nav.seg)).collect();
    for tick in 0..3600 {
        world.step(&net, player, &view, &|_, _| false, DT).unwrap_or_else(|e| panic!("tick {tick}: {e}"));
        for (i, c) in world.cars.iter().enumerate() {
            assert_eq!(c.status, 2, "nothing parks with spawning off");
            let p = c.drv.car.pos;
            let d = off_road(p);
            worst = worst.max(d);
            assert!(d <= limit, "tick {tick}: car {i} is {d:.2} off the road network at {p:?} (seg {}, limit {limit:.1})", c.drv.brain.nav.seg);
            let cd = off_road(c.drv.brain.nav.pos);
            worst_carrot = worst_carrot.max(cd);
            assert!(cd <= widest + 0.05, "tick {tick}: car {i}'s carrot is {cd:.3} from a centre line (seg {}, lane offset {})", c.drv.brain.nav.seg, c.drv.brain.nav.lane_off);
            off_lane_ticks += (d > widest + 4.0) as usize;
            red += c.drv.brain.junction_stop as usize;
            yields += c.drv.brain.yielding as usize;
            moved += ((p[0] - last[i].0[0]).powi(2) + (p[2] - last[i].0[2]).powi(2)).sqrt();
            if c.drv.brain.nav.seg != last[i].1 {
                pieces_entered += 1;
            }
            last[i] = (p, c.drv.brain.nav.seg);
        }
    }
    eprintln!("dense Paris 3600 ticks: {moved:.0} units driven, {pieces_entered} pieces entered, worst {worst:.2} from a centre line (limit {limit:.1}), carrots within {worst_carrot:.3} (widest lane {widest:.1}), {off_lane_ticks} car-ticks more than 4 past the widest lane line, {yields} yielding car-ticks, {red} at a red light, light phase {}", world.director.lights.phase);
    // The lights hold a road for all but 150 of every n·300 ticks (one shared phase), so most of
    // the minute is spent waiting; the cars still cover thousands of units and many pieces.
    assert!(moved > 2000.0, "the cars drive ({moved:.0} units)");
    assert!(pieces_entered > 100, "the cars cross many pieces ({pieces_entered})");
    assert!(red > 0, "some car stopped at a red light");
    assert_eq!(world.director.lights.phase, 11, "3600 ticks = 11 light periods past the first");
}
