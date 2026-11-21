//! The nearest-road join `FUN_00225c48` / `FUN_00225490`, the chasers' lane pick `FUN_00135f70`
//! with `FUN_00135d88` / `FUN_00135cf0` / `FUN_00135b10` / `FUN_001351d0` / `FUN_00222060`, the
//! target piece of `FUN_00131770`, `FUN_0012e468`'s join, and the car box `FUN_001c9778` →
//! `FUN_001f6e98` (docs/systems/driving-ai.md §12.5). The fixtures are format-exact RNgp articles;
//! every expected number is worked by hand from the decompile. The ISO-gated tests read the disc:
//! 200 random points near Paris roads, the lane masks in DRIVING.ELF and every car model's box.

use std::collections::HashMap;

use nf_formats::crp::{tag, Crp};
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;

use nf_game::driving::ai::{lookahead, stop_short, DriveRng, RoadGrid, RoadNav, TrafficWorld, BOX_FLOOR};
use nf_game::driving::join::{
    admitted, best_window, car_half_extents, chase_target_piece, floored_extents, lane_costs, lane_mask, lane_mask_index,
    lane_pick, makes_room_for, nav_for_pattern, nearest_cell, nearest_piece, project_clamped, road_width, round_half_away,
    scratch_join, snap_lane, speed_xz, JoinOutcome, LaneCar, LanePicker, LaneSelf, Nearest, ScratchNavs, LANE_MASKS_DEFAULT,
    LANE_MASKS_SNOW2A,
};
use nf_game::driving::road::{bezier, LaneSet, RoadNet, LANE_TABLE_DEFAULT, RSF_CURVED};

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
    fn f3(&mut self, v: [f32; 3]) {
        for x in v {
            self.b.extend_from_slice(&x.to_le_bytes());
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

fn rec(b: &mut Buf, tag: u32, short: bool, size: usize, skip: u32, count: u32) -> u32 {
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
    group: u8,
    /// rs+0x00 (the 0x8008 pieces' widths there).
    head: [f32; 3],
}

fn seg(a: u16, b: u16) -> SegSpec {
    SegSpec { a, b, flags: 0x200, lanes: (1, 1), group: 0, head: [0.0; 3] }
}

/// A format-exact RNgp `.crp` (side rows = (dir.z, 0, −dir.x), so a positive lane offset is −z
/// on a piece along +x).
fn build(nodes: &[([f32; 3], Vec<u16>)], segs: &[SegSpec]) -> Vec<u8> {
    let mut b = Buf { b: Vec::new() };
    rec(&mut b, 0x4341_5250, false, 0, 1, 0);
    b.put(12, &0x10u32.to_le_bytes());
    let win = 1 + nodes.len() + segs.len();
    let art = rec(&mut b, 0x524e_6770, false, 0, 0, win as u32);
    b.put(art + 12, &0x20u32.to_le_bytes());
    let hd = rec(&mut b, 0x524e_6864, false, 10, 0, 0);
    let rn: Vec<u32> = (0..nodes.len()).map(|i| rec(&mut b, 0x726e_0000 | i as u32, true, 0x18, 0, 0)).collect();
    let rs: Vec<u32> = (0..segs.len()).map(|i| rec(&mut b, 0x7273_0000 | i as u32, true, 0x64, 0, 0)).collect();
    let hp = b.off();
    for v in [nodes.len() as u16, segs.len() as u16, 0, 0, 0] {
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
        b.f3(s.head);
        b.u16(s.a);
        b.u16(s.b);
        b.f3([0.0; 3]);
        b.b.extend_from_slice(&(-1i16).to_le_bytes());
        b.b.extend_from_slice(&(-1i16).to_le_bytes());
        b.f3(side);
        b.b.extend_from_slice(&l.to_le_bytes());
        b.f3(side);
        b.pad_to(at, 0x3c);
        b.b.extend_from_slice(&(-1i16).to_le_bytes());
        b.u16(i as u16);
        b.f3(d);
        b.b.extend_from_slice(&25.0f32.to_le_bytes());
        b.u16(s.flags);
        b.b.push(s.lanes.0 as u8);
        b.b.push(s.lanes.1 as u8);
        b.pad_to(at, 0x58);
        b.b.push(0b01);
        b.pad_to(at, 0x5a);
        b.b.push(s.group);
        b.pad_to(at, 0x64);
        b.put(rs[i] + 12, &at.to_le_bytes());
    }
    b.b
}

fn parse(buf: &[u8]) -> RoadNet<'_> {
    RoadNet::parse_crp(&Crp::parse(buf).expect("fixture CARP")).expect("fixture RNgp")
}

fn near(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() <= eps)
}

/// One cell of 100 units at the origin listing `list`.
fn one_cell(list: Vec<i16>) -> RoadGrid {
    let mut cells = HashMap::new();
    cells.insert(0, list);
    RoadGrid { min_x: -50.0, min_z: -50.0, inv_cell: 0.01, rows: 1, cols: 1, cells }
}

/// Two parallel pieces along +x: piece 0 on z = 0, piece 1 on z = 10, 40 long.
fn two_roads(s0: SegSpec, s1: SegSpec) -> Vec<u8> {
    build(
        &[([0.0, 0.0, 0.0], vec![0]), ([40.0, 0.0, 0.0], vec![0]), ([0.0, 0.0, 10.0], vec![1]), ([40.0, 0.0, 10.0], vec![1])],
        &[s0, s1],
    )
}

fn traffic_nav() -> RoadNav {
    RoadNav::cleared(1, LaneSet::Default)
}

// ---------------------------------------------------------------- geometry

#[test]
fn the_clamped_projection_is_0x222a30() {
    let a = [0.0, 0.0, 0.0];
    let b = [10.0, 0.0, 0.0];
    assert_eq!(project_clamped(a, b, [4.0, 0.0, 3.0]), (0.4, [4.0, 0.0, 0.0]));
    assert_eq!(project_clamped(a, b, [-1.0, 0.0, 1.0]), (0.0, a));
    assert_eq!(project_clamped(a, b, [12.0, 0.0, 0.0]), (1.0, b));
    // Exactly on A: n(0) = 0 → s = 0 → (0, a).
    assert_eq!(project_clamped(a, b, a), (0.0, a));
}

#[test]
fn the_lane_snap_is_0x221f20() {
    let mut s = seg(0, 1);
    s.lanes = (3, 1);
    let buf = build(&[([0.0; 3], vec![0]), ([40.0, 0.0, 0.0], vec![0])], &[s]);
    let net = parse(&buf);
    // Class 0 (flags 0x200): table 0, 2, 6, 10 …; 7 < 6 + 2.5 → slot 2, one lane on +: slot 1.
    assert_eq!(snap_lane(&net, LaneSet::Default, 0, 7.0).unwrap(), (1, 2.0));
    // −7 with three lanes on −: slot 2 kept.
    assert_eq!(snap_lane(&net, LaneSet::Default, 0, -7.0).unwrap(), (-2, -6.0));
    // −13 → slot 3 (10 + 2.5 > 13? no: 14 + 2.5 → slot 4) clamped to the three − lanes.
    assert_eq!(snap_lane(&net, LaneSet::Default, 0, -13.0).unwrap(), (-3, -10.0));
    // Past every slot: lane 0, offset 0.
    assert_eq!(snap_lane(&net, LaneSet::Default, 0, 30.0).unwrap(), (0, 0.0));
    // 0 counts as positive: slot 1.
    assert_eq!(snap_lane(&net, LaneSet::Default, 0, 0.0).unwrap(), (1, 2.0));
}

#[test]
fn the_road_widths_are_0x221ac8_and_0x221cb0() {
    let mut s0 = seg(0, 1);
    s0.flags = 0x10 | 0x200;
    s0.lanes = (2, 1);
    let mut s1 = seg(2, 3);
    s1.flags = 0x8 | 0x200;
    s1.head = [3.0, 4.5, 0.0];
    let buf = two_roads(s0, s1);
    let net = parse(&buf);
    // Negative side: 5 (0x10) + 5 (0x200) + 2·5; positive: 5 + 1·5 (no 0x400).
    assert_eq!(road_width(&net, 0, false).unwrap(), 20.0);
    assert_eq!(road_width(&net, 0, true).unwrap(), 10.0);
    // A flag-8 piece reads rs+0x00 / rs+0x04.
    assert_eq!(road_width(&net, 1, false).unwrap(), 3.0);
    assert_eq!(road_width(&net, 1, true).unwrap(), 4.5);
    // Flag 0x1000: (2 + 1)·5·0.5 on each side.
    let mut s0 = seg(0, 1);
    s0.flags = 0x1000 | 0x10;
    s0.lanes = (2, 1);
    let buf = two_roads(s0, seg(2, 3));
    let net = parse(&buf);
    assert_eq!(road_width(&net, 0, false).unwrap(), 12.5);
    // Nothing at all: 0 → 5.
    let mut s1 = seg(2, 3);
    s1.flags = 0;
    s1.lanes = (0, 0);
    let buf = two_roads(seg(0, 1), s1);
    let net = parse(&buf);
    assert_eq!(road_width(&net, 1, true).unwrap(), 5.0);
}

#[test]
fn a_connector_takes_the_widest_plain_link() {
    // n0 —(piece 0, 2 lanes −)— n1 —(piece 1, connector)— n2 —(piece 2, 3 lanes −)— n3.
    let mut s0 = seg(0, 1);
    s0.lanes = (2, 1);
    let mut s1 = seg(1, 2);
    s1.flags = 1;
    let mut s2 = seg(2, 3);
    s2.lanes = (3, 1);
    let buf = build(
        &[([0.0; 3], vec![0]), ([20.0, 0.0, 0.0], vec![0, 1]), ([30.0, 0.0, 0.0], vec![1, 2]), ([50.0, 0.0, 0.0], vec![2])],
        &[s0, s1, s2],
    );
    let net = parse(&buf);
    assert_eq!(road_width(&net, 1, false).unwrap(), 5.0 + 15.0);
}

// ---------------------------------------------------------------- the nearest piece

#[test]
fn the_grid_cell_clamps_like_0x225490() {
    let g = RoadGrid { min_x: 0.0, min_z: 0.0, inv_cell: 0.1, rows: 2, cols: 3, cells: HashMap::new() };
    assert_eq!(nearest_cell(&g, [25.0, 0.0, 15.0]), 5);
    assert_eq!(nearest_cell(&g, [-5.0, 0.0, 15.0]), 3, "−0.5 truncates to 0");
    assert_eq!(nearest_cell(&g, [-15.0, 0.0, 15.0]), 3, "a negative index goes to 0");
    assert_eq!(nearest_cell(&g, [95.0, 0.0, 15.0]), 5, "past the columns: the last column");
    assert_eq!(nearest_cell(&g, [1e30, 0.0, 15.0]), 3, "an overflow saturates to 0x7fffffff, which goes to 0");
    assert_eq!(nearest_cell(&g, [25.0, 0.0, 99.0]), 5, "past the rows: the last row");
}

#[test]
fn the_nearest_piece_is_0x225490() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![1, 0, 1, 7]);
    // 3 from piece 0, 7 from piece 1.
    assert_eq!(nearest_piece(&net, &grid, 1, 0, [20.0, 0.0, 3.0]).unwrap(), Nearest::Piece { seg: 0, point: [20.0, 0.0, 0.0], t: 0.5 });
    // A tie: the first listed (piece 1) wins.
    assert_eq!(nearest_piece(&net, &grid, 1, 0, [20.0, 0.0, 5.0]).unwrap(), Nearest::Piece { seg: 1, point: [20.0, 0.0, 10.0], t: 0.5 });
    // An empty cell gives the EmptyCell case (piece 0 in the game).
    assert_eq!(nearest_piece(&net, &one_cell(vec![]), 1, 0, [20.0, 0.0, 3.0]).unwrap(), Nearest::EmptyCell);
    // Flag 0x100 is never joined.
    let mut s0 = seg(0, 1);
    s0.flags |= 0x100;
    let buf = two_roads(s0, seg(2, 3));
    let net = parse(&buf);
    assert!(matches!(nearest_piece(&net, &grid, 1, 0, [20.0, 0.0, 3.0]).unwrap(), Nearest::Piece { seg: 1, .. }));
}

#[test]
fn the_kinds_admit_their_own_pieces() {
    let mut s0 = seg(0, 1);
    s0.flags |= 0x8;
    s0.group = 7;
    let mut s1 = seg(2, 3);
    s1.flags |= 0x8000;
    let buf = two_roads(s0, seg(2, 3));
    let net = parse(&buf);
    // Kind 4: only flag-8 pieces of its group; others never a flag-8 piece.
    assert!(admitted(&net, 0, 4, 7).unwrap());
    assert!(!admitted(&net, 0, 4, 6).unwrap());
    assert!(!admitted(&net, 1, 4, 7).unwrap());
    assert!(!admitted(&net, 0, 1, 7).unwrap());
    assert!(admitted(&net, 1, 1, 0).unwrap());
    let buf = two_roads(seg(0, 1), s1);
    let net = parse(&buf);
    // Kind 5: only flag-0x8000 pieces; others never one.
    assert!(admitted(&net, 1, 5, 0).unwrap());
    assert!(!admitted(&net, 0, 5, 0).unwrap());
    assert!(!admitted(&net, 1, 3, 0).unwrap());
    let grid = one_cell(vec![0, 1]);
    assert_eq!(nearest_piece(&net, &grid, 5, 0, [20.0, 0.0, 0.0]).unwrap(), Nearest::Piece { seg: 1, point: [20.0, 0.0, 10.0], t: 0.5 });
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    assert_eq!(nearest_piece(&net, &grid, 5, 0, [20.0, 0.0, 0.0]).unwrap(), Nearest::None);
}

// ---------------------------------------------------------------- the join

#[test]
fn a_traffic_car_joins_its_own_side_lane() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    // 3 on the + side (−z), heading +x: offset 3 → snap → lane 1 at 2; carrot (20, 0, −2).
    let mut nav = traffic_nav();
    assert_eq!(nav.join_nearest(&net, &grid, [20.0, 0.0, -3.0], [1.0, 0.0, 0.0], false).unwrap(), JoinOutcome::Joined);
    assert_eq!((nav.seg, nav.end, nav.lane, nav.lane_off), (0, 1, 1, 2.0));
    assert!((nav.t - 0.5).abs() < 1e-6, "{}", nav.t);
    assert!(near(nav.pos, [20.0, 0.0, -2.0], 1e-6), "{:?}", nav.pos);
    // 3 on the − side: kind 1 takes max(2.5, −3) = 2.5 → still lane 1 at +2.
    let mut nav = traffic_nav();
    nav.join_nearest(&net, &grid, [20.0, 0.0, 3.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!((nav.lane, nav.lane_off), (1, 2.0));
    assert!(near(nav.pos, [20.0, 0.0, -2.0], 1e-6), "{:?}", nav.pos);
    // Heading −x: end 0, the + side is now +z: P0 = B − side_b·2 = (40, 0, 2).
    let mut nav = traffic_nav();
    nav.join_nearest(&net, &grid, [20.0, 0.0, -3.0], [-1.0, 0.0, 0.0], false).unwrap();
    assert_eq!((nav.end, nav.lane, nav.lane_off, nav.dir), (0, 1, 2.0, [-1.0, 0.0, 0.0]));
    assert!(near(nav.p0, [40.0, 0.0, 2.0], 1e-6) && near(nav.pos, [20.0, 0.0, 2.0], 1e-6), "{:?} {:?}", nav.p0, nav.pos);
    // A zero heading counts as along the piece.
    let mut nav = traffic_nav();
    nav.join_nearest(&net, &grid, [20.0, 0.0, -3.0], [0.0; 3], false).unwrap();
    assert_eq!(nav.end, 1);
    // param_4 = 1: the centre line.
    let mut nav = traffic_nav();
    nav.join_nearest(&net, &grid, [10.0, 0.0, -3.0], [1.0, 0.0, 0.0], true).unwrap();
    assert_eq!((nav.lane, nav.lane_off), (0, 0.0));
    assert!((nav.t - 0.25).abs() < 1e-6, "{}", nav.t);
    assert!(near(nav.pos, [10.0, 0.0, 0.0], 1e-6));
}

#[test]
fn a_chaser_keeps_its_offset_and_measures_it_laterally_behind_the_piece() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    // Behind P0 (−5 along) and 4 to the + side: the lateral 4, not the √41 = 6.4 to node A.
    let mut nav = RoadNav::cleared(3, LaneSet::Default);
    nav.join_nearest(&net, &grid, [-5.0, 0.0, -4.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!((nav.lane, nav.lane_off, nav.t), (0, 4.0, 0.0));
    assert!(near(nav.pos, [0.0, 0.0, -4.0], 1e-6), "{:?}", nav.pos);
    // Beside the piece: 7.5 on the − side, kept as it is (no snap).
    let mut nav = RoadNav::cleared(3, LaneSet::Default);
    nav.join_nearest(&net, &grid, [30.0, 0.0, 7.5], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!(nav.lane_off, -7.5);
    assert!((nav.t - 0.75).abs() < 1e-6, "{}", nav.t);
    assert!(near(nav.pos, [30.0, 0.0, 7.5], 1e-6), "{:?}", nav.pos);
}

#[test]
fn a_kind_4_follower_is_clamped_to_the_road_and_keeps_the_piece_direction() {
    let mut s0 = seg(0, 1);
    s0.flags = 0x8;
    s0.group = 7;
    s0.head = [3.0, 4.5, 0.0];
    let buf = two_roads(s0, seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0, 1]);
    let mut nav = RoadNav::cleared(4, LaneSet::Default);
    nav.group = 7;
    // +9 → min(9, rs+0x04 = 4.5); heading −x but kind 4 without +0xb1/+0xb2 drives A → B.
    nav.join_nearest(&net, &grid, [20.0, 0.0, -9.0], [-1.0, 0.0, 0.0], false).unwrap();
    assert_eq!((nav.seg, nav.end, nav.lane_off), (0, 1, 4.5));
    // −8 → max(−8, −rs+0x00 = −3).
    nav.join_nearest(&net, &grid, [20.0, 0.0, 8.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!(nav.lane_off, -3.0);
    // With +0xb1 the heading decides.
    nav.hint_toward = true;
    nav.join_nearest(&net, &grid, [20.0, 0.0, -1.0], [-1.0, 0.0, 0.0], false).unwrap();
    assert_eq!(nav.end, 0);
    // Another group finds no road.
    let mut other = RoadNav::cleared(4, LaneSet::Default);
    other.group = 6;
    assert_eq!(other.join_nearest(&net, &grid, [20.0, 0.0, -1.0], [1.0, 0.0, 0.0], false).unwrap(), JoinOutcome::NoRoad);
}

#[test]
fn a_pedestrian_navigator_joins_past_the_outer_lane() {
    let mut s0 = seg(0, 1);
    s0.lanes = (1, 2);
    let buf = two_roads(s0, seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    let mut nav = RoadNav::cleared(2, LaneSet::Default);
    // + side: lane 2 + 1, table[3] − 1 = 9.
    nav.join_nearest(&net, &grid, [20.0, 0.0, -3.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!((nav.lane, nav.lane_off), (3, 9.0));
    // − side: lane !1 = −2, −table[2] + 1 = −5.
    nav.join_nearest(&net, &grid, [20.0, 0.0, 3.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!((nav.lane, nav.lane_off), (-2, -5.0));
    // With no + lanes the − side goes 2 further out: −3.
    let mut s0 = seg(0, 1);
    s0.lanes = (1, 0);
    let buf = two_roads(s0, seg(2, 3));
    let net = parse(&buf);
    nav.join_nearest(&net, &grid, [20.0, 0.0, 3.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!((nav.lane, nav.lane_off), (-2, -3.0));
}

#[test]
fn reverse_is_0x225358() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    let mut nav = RoadNav::cleared(3, LaneSet::Default);
    nav.join_nearest(&net, &grid, [10.0, 0.0, -3.0], [1.0, 0.0, 0.0], false).unwrap();
    let (p0, p1) = (nav.p0, nav.p1);
    nav.reverse();
    assert_eq!((nav.end, nav.dir, nav.lane_off, nav.lane_to), (0, [-1.0, 0.0, 0.0], -3.0, -3.0));
    assert!((nav.t - 0.75).abs() < 1e-6, "{}", nav.t);
    assert_eq!((nav.p0, nav.p1), (p1, p0));
}

// ---------------------------------------------------------------- the lane pick

#[test]
fn the_lane_mask_index_and_mask_are_0x222060() {
    assert_eq!(lane_mask_index(0x200, 2), 0);
    assert_eq!(lane_mask_index(0, 2), 9, "no 0x200 and index 0 → 9");
    assert_eq!(lane_mask_index(0x200, 3), 1);
    assert_eq!(lane_mask_index(0x200, 1), 8);
    assert_eq!(lane_mask_index(0x10, 2), 6);
    assert_eq!(lane_mask_index(0x820, 4), 5);
    assert_eq!(lane_mask_index(0x800, 2), 2);
    let mut s1 = seg(2, 3);
    s1.flags = 0x8;
    s1.head = [3.0, 5.0, 0.0];
    let buf = two_roads(seg(0, 1), s1);
    let net = parse(&buf);
    let mut m = [0u8; 64];
    lane_mask(&net, LaneSet::Default, 0, 1, &mut m).unwrap();
    for k in 0..32 {
        assert_eq!((m[k], m[63 - k]), (LANE_MASKS_DEFAULT[0][k], LANE_MASKS_DEFAULT[0][k]), "slot {k} and its mirror");
    }
    // A flag-8 piece toward B: 3 open below the centre (slots 30, 31), 5 above (32..36).
    lane_mask(&net, LaneSet::Default, 1, 1, &mut m).unwrap();
    assert_eq!(&m[28..38], &[255, 255, 0, 0, 0, 0, 0, 0, 0, 255]);
    // Toward A the two sides swap.
    lane_mask(&net, LaneSet::Default, 1, 0, &mut m).unwrap();
    assert_eq!(&m[26..36], &[255, 255, 0, 0, 0, 0, 0, 0, 0, 255]);
}

/// A four-lane piece (flags 0x200, lanes 2/2 → mask 1: slots 24..39 open, 22/23 and 40/41 at 64).
fn wide_road() -> Vec<u8> {
    let mut s0 = seg(0, 1);
    s0.lanes = (2, 2);
    two_roads(s0, seg(2, 3))
}

#[test]
fn the_cost_map_is_0x135d88_and_the_window_0x135cf0() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    let mut nav = RoadNav::cleared(3, LaneSet::Default);
    nav.join_nearest(&net, &grid, [20.0, 0.0, -2.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!(nav.lane_off, 2.0);
    let mut h = [0u8; 64];
    lane_costs(&net, &nav, &mut h).unwrap();
    // Mask 0 (…64, 64, 0, 0, 0, 0 | 0, 0, 0, 0, 64, 64…) + 5 per slot from slot 34.
    assert_eq!(&h[25..39], &[255, 104, 99, 30, 25, 20, 15, 10, 5, 0, 5, 74, 79, 255]);
    // Four wide: slots 32..35 sum 20, the lowest → 32 + 2 = 34.
    assert_eq!(best_window(&h, 4), 34);
    // Three wide: 33..35 sums 10 → 33 + 1.
    assert_eq!(best_window(&h, 3), 34);
    // A zero window never beats 0 · 255: the default 32.
    assert_eq!(best_window(&h, 0), 32);
}

#[test]
fn room_is_made_for_cars_within_the_speed_radius_and_ahead() {
    let me = [0.0, 0.0, 0.0];
    let f = [1.0, 0.0, 0.0];
    assert!(makes_room_for(5.0, 0.0, me, [19.0, 0.0, 0.0], f, false), "r = max(2·5, 20)");
    assert!(!makes_room_for(5.0, 0.0, me, [21.0, 0.0, 0.0], f, false));
    assert!(makes_room_for(20.0, 0.0, me, [34.0, 0.0, 0.0], f, false), "r = min(40, 35)");
    assert!(!makes_room_for(20.0, 0.0, me, [36.0, 0.0, 0.0], f, false));
    assert!(!makes_room_for(10.0, 11.0, me, [16.0, 0.0, 0.0], f, false), "a faster car: r = 15");
    assert!(makes_room_for(10.0, 10.5, me, [16.0, 0.0, 0.0], f, false), "less than 1 faster: r = 20");
    assert!(!makes_room_for(5.0, 0.0, me, [-5.0, 0.0, 0.0], f, false), "behind");
    assert!(makes_room_for(5.0, 0.0, me, [0.0, 0.0, 5.0], f, false), "beside counts (dot = 0)");
    assert!(!makes_room_for(5.0, 0.0, me, [5.0, 6.0, 0.0], f, false), "6 above");
    assert!(makes_room_for(5.0, 0.0, me, [5.0, 6.0, 0.0], f, true), "both overtaking skip the height test");
    assert_eq!(speed_xz([3.0, 100.0, 4.0]), 5.0);
}

#[test]
fn rounding_is_half_away_from_zero() {
    assert_eq!([round_half_away(2.5), round_half_away(2.49), round_half_away(-2.5), round_half_away(-2.49), round_half_away(0.0)], [3, 2, -3, -2, 0]);
}

fn other_car(pos: [f32; 3], vel: [f32; 3]) -> LaneCar {
    LaneCar {
        sim_id: 2,
        kind: 2,
        status: 2,
        pos,
        vel,
        fwd: [1.0, 0.0, 0.0],
        box_z: 2.3,
        overtake: false,
        width: 4,
        profile: 0xc,
        state: 4,
        targets_asker: false,
        actor_clear: true,
        alive: 1.0,
        actor_9c: 0,
        scratch: ScratchNavs::new(LaneSet::Default),
    }
}

fn picker() -> LanePicker {
    LanePicker { hist: [0; 64], width: 4, overtake: false, avoid: true, hold: false, target: None, f204: false, f208: false, f148: false }
}

fn me_at(pos: [f32; 3]) -> LaneSelf {
    LaneSelf { sim_id: 1, pos, vel: [10.0, 0.0, 0.0], right: [0.0, 0.0, -1.0], fwd: [1.0, 0.0, 0.0] }
}

fn chaser(net: &RoadNet, grid: &RoadGrid, pos: [f32; 3]) -> RoadNav {
    let mut nav = RoadNav::cleared(3, LaneSet::Default);
    nav.join_nearest(net, grid, pos, [1.0, 0.0, 0.0], false).unwrap();
    nav
}

#[test]
fn the_lane_pick_steers_round_a_car_ahead_in_its_lane() {
    let buf = wide_road();
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    // Alone at offset 2 with a 4-wide window: the best window starts at 32 (sum 20) → 34 − 32 −
    // 0.5 = 1.5, over |2 − 1.5|·10·0.1.
    let mut nav = chaser(&net, &grid, [10.0, 0.0, -2.0]);
    let mut p = picker();
    let out = lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -2.0]), &mut nav, &mut [], 100).unwrap();
    assert_eq!(out.scratch_kind, Some(3));
    assert_eq!(out.change, Some((1.5, 0.5)));
    assert_eq!((nav.lane_to, nav.lane_off, nav.blend_len), (1.5, 2.0, 0.5));
    // A stopped car 10 ahead in the same lane (offset 2 → slot 34, window 4: slots 32..35 + 128):
    // the best window moves to 36..39 (sum 70) → 38 − 32 − 0.5 = 5.5.
    let mut nav = chaser(&net, &grid, [10.0, 0.0, -2.0]);
    let mut p = picker();
    let mut cars = [other_car([20.0, 0.0, -2.0], [0.0; 3])];
    let out = lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -2.0]), &mut nav, &mut cars, 100).unwrap();
    assert_eq!(&p.hist[30..42], &[20, 15, 138, 133, 128, 133, 10, 15, 20, 25, 94, 99]);
    assert_eq!(out.change, Some((5.5, 3.5)));
    assert_eq!(cars[0].scratch.stamp3, 100, "the other car's kind-3 scratch was joined now");
    // The same car chasing the asker (profile 0xe, alive, actor clear) weighs 0: as if alone.
    let mut nav = chaser(&net, &grid, [10.0, 0.0, -2.0]);
    let mut p = picker();
    let mut c = other_car([20.0, 0.0, -2.0], [0.0; 3]);
    c.targets_asker = true;
    c.profile = 0xe;
    let out = lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -2.0]), &mut nav, &mut [c], 100).unwrap();
    assert_eq!(out.change, Some((1.5, 0.5)));
    // A parked car (status 3) and a car 25 ahead (beyond r = 20) are not marked.
    let mut nav = chaser(&net, &grid, [10.0, 0.0, -2.0]);
    let mut p = picker();
    let mut c = other_car([20.0, 0.0, -2.0], [0.0; 3]);
    c.status = 3;
    let mut cars = [c, other_car([35.0, 0.0, -2.0], [0.0; 3])];
    let out = lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -2.0]), &mut nav, &mut cars, 100).unwrap();
    assert_eq!(out.change, Some((1.5, 0.5)));
}

#[test]
fn the_lane_pick_is_skipped_off_or_mid_change_and_reuses_a_held_map() {
    let buf = wide_road();
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    let mut nav = chaser(&net, &grid, [10.0, 0.0, -2.0]);
    let mut p = picker();
    p.avoid = false;
    assert_eq!(lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -2.0]), &mut nav, &mut [], 0).unwrap().change, None);
    let mut p = picker();
    nav.lane_to = 6.0;
    assert_eq!(lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -2.0]), &mut nav, &mut [], 0).unwrap().change, None);
    // Held (+0x94): no rebuild; the stored map's best window (slots 8..11 at 0) → 10 − 32 − 0.5.
    let mut nav = chaser(&net, &grid, [10.0, 0.0, -2.0]);
    let mut p = picker();
    p.hold = true;
    p.hist = [200; 64];
    for k in 8..12 {
        p.hist[k] = 0;
    }
    let out = lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -2.0]), &mut nav, &mut [], 0).unwrap();
    assert_eq!((out.scratch_kind, out.change), (None, Some((-22.5, 24.5))));
}

#[test]
fn a_kind_4_car_with_nothing_near_heads_back_to_the_centre() {
    let mut s0 = seg(0, 1);
    s0.flags = 0x8 | 0x200;
    s0.group = 3;
    s0.head = [12.0, 12.0, 0.0];
    let buf = two_roads(s0, seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    let mut nav = RoadNav::cleared(4, LaneSet::Default);
    nav.group = 3;
    nav.join_nearest(&net, &grid, [10.0, 0.0, -6.0], [1.0, 0.0, 0.0], false).unwrap();
    assert_eq!(nav.lane_off, 6.0);
    let mut p = picker();
    let out = lane_pick(&net, &grid, &mut p, &me_at([10.0, 0.0, -6.0]), &mut nav, &mut [], 0).unwrap();
    // To half the offset over |6 − 3|·10·0.1; the centre slots +0xb7..+0xb9 zeroed.
    assert_eq!((out.scratch_kind, out.change), (Some(4), Some((3.0, 3.0))));
    assert_eq!(&p.hist[0x1f..0x22], &[0, 0, 0]);
    // Within 2 of the centre it goes straight to 0.
    let mut nav = RoadNav::cleared(4, LaneSet::Default);
    nav.group = 3;
    nav.join_nearest(&net, &grid, [10.0, 0.0, -1.5], [1.0, 0.0, 0.0], false).unwrap();
    let out = lane_pick(&net, &grid, &mut picker(), &me_at([10.0, 0.0, -1.5]), &mut nav, &mut [], 0).unwrap();
    assert_eq!(out.change, Some((0.0, 1.5)));
}

#[test]
fn the_scratch_join_refreshes_every_15_ticks_and_turns_to_the_asker() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    let mut sc = ScratchNavs::new(LaneSet::Default);
    let f = [1.0, 0.0, 0.0];
    // Joined at pos + vel·0.25 = (14, 0, −3).
    let n = scratch_join(&net, &grid, &mut sc, [10.0, 0.0, -3.0], [16.0, 0.0, 0.0], f, f, false, 0, 10).unwrap();
    assert_eq!(n.lane_off, 3.0);
    assert!((n.t - 0.35).abs() < 1e-6, "{}", n.t);
    // 14 ticks later the old join stands though the car moved.
    let n = scratch_join(&net, &grid, &mut sc, [10.0, 0.0, -5.0], [0.0; 3], f, f, false, 0, 24).unwrap();
    assert_eq!(n.lane_off, 3.0);
    // At 25 it re-joins.
    let n = scratch_join(&net, &grid, &mut sc, [10.0, 0.0, -5.0], [0.0; 3], f, f, false, 0, 25).unwrap();
    assert_eq!(n.lane_off, 5.0);
    // Asked by a car heading the other way: turned round, the offset negated, and kept.
    let n = scratch_join(&net, &grid, &mut sc, [10.0, 0.0, -5.0], [0.0; 3], f, [-1.0, 0.0, 0.0], false, 0, 26).unwrap();
    assert_eq!((n.lane_off, n.end), (-5.0, 0));
    assert_eq!(sc.nav3.lane_off, -5.0);
}

// ---------------------------------------------------------------- what drives the choosers

#[test]
fn the_target_piece_follows_the_goal_or_the_target_car() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0, 1]);
    let f = [1.0, 0.0, 0.0];
    let mut scratch = RoadNav::cleared(0, LaneSet::Default);
    // Not pattern 3, or profile 0xe/0xf, or not tracking: −1. Held: unchanged.
    assert_eq!(chase_target_piece(&net, &grid, 0xd, 4, true, false, 3, f, [20.0, 0.0, 9.0], &mut scratch, None, 0).unwrap(), Some(-1));
    assert_eq!(chase_target_piece(&net, &grid, 0xe, 3, true, false, 3, f, [20.0, 0.0, 9.0], &mut scratch, None, 0).unwrap(), Some(-1));
    assert_eq!(chase_target_piece(&net, &grid, 0xd, 3, false, false, 3, f, [20.0, 0.0, 9.0], &mut scratch, None, 0).unwrap(), Some(-1));
    assert_eq!(chase_target_piece(&net, &grid, 0xd, 3, true, true, 3, f, [20.0, 0.0, 9.0], &mut scratch, None, 0).unwrap(), None);
    // The goal by piece 1: the scratch joins it as kind 3 (own kind 3) or 4 (own kind 4).
    assert_eq!(chase_target_piece(&net, &grid, 0xd, 3, true, false, 3, f, [20.0, 0.0, 9.0], &mut scratch, None, 0).unwrap(), Some(1));
    assert_eq!((scratch.kind, scratch.seg), (3, 1));
    // A target car by piece 0: its kind-3 scratch gives the piece.
    let mut sc = ScratchNavs::new(LaneSet::Default);
    let r = chase_target_piece(&net, &grid, 0xd, 3, true, false, 3, f, [20.0, 0.0, 9.0], &mut scratch, Some((&mut sc, [20.0, 0.0, 1.0], [0.0; 3], f)), 5).unwrap();
    assert_eq!((r, sc.stamp3), (Some(0), 5));
}

#[test]
fn the_pattern_sets_the_navigator_kind_and_join() {
    let buf = two_roads(seg(0, 1), seg(2, 3));
    let net = parse(&buf);
    let grid = one_cell(vec![0]);
    let f = [1.0, 0.0, 0.0];
    let mut nav = RoadNav::cleared(9, LaneSet::Default);
    // Traffic: any pattern but 1 → kind 1, no join.
    assert_eq!(nav_for_pattern(&net, &grid, &mut nav, 4, 3, true, [20.0, 0.0, -3.0], f).unwrap(), None);
    assert_eq!(nav.kind, 1);
    assert_eq!(nav_for_pattern(&net, &grid, &mut nav, 4, 1, true, [20.0, 0.0, -3.0], f).unwrap(), None);
    assert_eq!(nav.kind, 0);
    // Pattern 3: kind 3 with the offset.
    assert_eq!(nav_for_pattern(&net, &grid, &mut nav, 5, 3, true, [20.0, 0.0, -3.0], f).unwrap(), Some(JoinOutcome::Joined));
    assert_eq!((nav.kind, nav.lane_off), (3, 3.0));
    // Pattern 2 leaves the kind.
    nav_for_pattern(&net, &grid, &mut nav, 5, 2, true, [20.0, 0.0, -3.0], f).unwrap();
    assert_eq!(nav.kind, 3);
    // Pattern 4 with the avoidance off: the centre line (no flag-8 piece here → no road).
    assert_eq!(nav_for_pattern(&net, &grid, &mut nav, 5, 4, false, [20.0, 0.0, -3.0], f).unwrap(), Some(JoinOutcome::NoRoad));
    assert_eq!(nav.kind, 4);
}

// ---------------------------------------------------------------- the car box

#[test]
fn the_box_floor_and_the_stop_behind_rule_use_the_half_length() {
    assert_eq!(floored_extents([0.0, 0.5, 2.3]), [BOX_FLOOR, 0.5, 2.3]);
    // stop_short pulls the aim back by 1.5·box_z: the hatchback-sized 2.2 stops 3.3 short.
    let mut aim = [30.0, 0.0, 0.0];
    stop_short(0.0, [0.0; 3], 2.2, &mut aim, [20.0, 0.0, 0.0]);
    assert!(near(aim, [20.0 - 3.3, 0.0, 0.0], 1e-5), "{aim:?}");
    assert_eq!(lookahead(0.0, 2.2), 2.2 + 7.5);
}

// ---------------------------------------------------------------- the real disc (NF_ISO)

fn open_iso() -> Option<IsoImage<std::fs::File>> {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return None;
    };
    Some(IsoImage::open(&path).unwrap())
}

fn viv_files(iso: &IsoImage<std::fs::File>, viv: &str, pred: &dyn Fn(&str) -> bool) -> Vec<(String, Vec<u8>)> {
    let Some(e) = iso.find(viv) else { return Vec::new() };
    let g = nf_formats::disc::read_bigf(iso, e).unwrap();
    let mut out = Vec::new();
    for x in &g.entries {
        let n = x.name.to_ascii_lowercase();
        if !pred(&n) {
            continue;
        }
        let raw = iso.read_bytes(e.lba as u64 * 2048 + x.offset as u64, x.size as u64).unwrap();
        if raw.iter().take(64).all(|&b| b == 0) {
            continue; // slim image
        }
        out.push((n, if refpack::is_refpack(&raw) { refpack::decompress(&raw).unwrap() } else { raw }));
    }
    out
}

#[test]
fn the_lane_masks_are_the_disc_elf_tables() {
    let Some(iso) = open_iso() else { return };
    let e = iso.entries().iter().find(|e| e.path.ends_with("/DRIVING.ELF")).expect("DRIVING.ELF");
    let elf = iso.read_file(e).unwrap();
    let phoff = u32::from_le_bytes(elf[0x1c..0x20].try_into().unwrap()) as usize;
    let off = u32::from_le_bytes(elf[phoff + 4..phoff + 8].try_into().unwrap());
    let va = u32::from_le_bytes(elf[phoff + 8..phoff + 12].try_into().unwrap());
    let rd = |a: u32| elf[(a - va + off) as usize];
    for k in 0..10 {
        for j in 0..32 {
            assert_eq!(rd(0x33a6d8 + k * 32 + j as u32), LANE_MASKS_SNOW2A[k as usize][j], "snow2a mask {k} slot {j}");
            assert_eq!(rd(0x33a818 + k * 32 + j as u32), LANE_MASKS_DEFAULT[k as usize][j], "default mask {k} slot {j}");
        }
    }
}

#[test]
fn every_car_model_has_a_base_box() {
    let Some(iso) = open_iso() else { return };
    let mut n = 0;
    let mut seen = std::collections::BTreeMap::new();
    for e in iso.entries().iter().filter(|e| e.path.starts_with("/DRIVING/") && e.path.ends_with(".VIV")) {
        for (name, b) in viv_files(&iso, &e.path, &|n| n.starts_with(r"data\car\model\") && n.ends_with(".crp")) {
            let crp = Crp::parse(&b).unwrap();
            let h = car_half_extents(&crp).unwrap_or_else(|err| panic!("{} {name}: {err}", e.path));
            // The same numbers straight off the Base record.
            let arti = crp.pool().unwrap().into_iter().find(|r| r.tag == tag::ARTI).unwrap();
            let (s, en) = arti.window(crp.data).unwrap();
            let base = crp.window_records(s, en).unwrap().into_iter().find(|r| r.tag == 0x4261_7365).unwrap();
            let p = crp.payload(&base).unwrap();
            let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
            assert_eq!(h, [(f(0x20) - f(0x10)) * 0.5, (f(0x24) - f(0x14)) * 0.5, (f(0x28) - f(0x18)) * 0.5]);
            assert!(h.iter().all(|&x| x > BOX_FLOOR), "{name}: {h:?}");
            seen.insert(name.clone(), h);
            n += 1;
        }
    }
    for (k, v) in &seen {
        eprintln!("{k}: half extents {v:?}");
    }
    assert!(n >= 60, "{n} car models");
    assert_eq!(seen[r"data\car\model\paris_henchbigbox.crp"], [0.97594196, 0.6050807, 2.3213403]);
}

/// 200 random points near Paris roads (DriveRng picks the piece, t, the side distance and the
/// heading): each joins as a traffic car (kind 1) and as a chaser (kind 3); every join must land
/// on a lane line of the piece `nearest_piece` names, and that piece must be the nearest admitted
/// piece of the point's cell by a brute-force search.
#[test]
fn paris_joins_land_on_lanes() {
    let Some(iso) = open_iso() else { return };
    let files = viv_files(&iso, "/DRIVING/MIS01.VIV", &|n| n.contains("track") && n.ends_with(".crp"));
    let Some((_, img)) = files.into_iter().next() else { return };
    let crp = Crp::parse(&img).unwrap();
    let net = RoadNet::parse_crp(&crp).unwrap();
    let grid = RoadGrid::parse_crp(&crp).unwrap();
    let mut rng = DriveRng::game_reset();
    let mut r01 = || rng.next() as f32 / 65536.0;
    let plain: Vec<usize> = (0..net.segs.len()).filter(|&s| admitted(&net, s, 1, 0).unwrap()).collect();
    let (mut joined, mut no_road, mut empty, mut other_piece) = (0, 0, 0, 0);
    let mut max_side = 0.0f32;
    for i in 0..200 {
        let seg = plain[(r01() * plain.len() as f32) as usize % plain.len()];
        let t = r01();
        let side = (r01() - 0.5) * 24.0;
        let s = net.seg(seg).unwrap();
        let c = net.seg_point(seg, t).unwrap();
        let sa = s.side_a();
        let p = [c[0] + sa[0] * side, c[1] + sa[1] * side, c[2] + sa[2] * side];
        let d = s.dir();
        let heading = if r01() < 0.5 { d } else { [-d[0], -d[1], -d[2]] };
        for kind in [1u32, 3] {
            let mut nav = RoadNav::cleared(kind, LaneSet::Default);
            match nav.join_nearest(&net, &grid, p, heading, false).unwrap() {
                JoinOutcome::NoRoad => {
                    no_road += 1;
                    continue;
                }
                JoinOutcome::EmptyCell => {
                    empty += 1;
                    continue;
                }
                JoinOutcome::Joined => joined += 1,
            }
            let js = net.seg(nav.seg).unwrap();
            // The piece is the cell's nearest admitted piece (brute force).
            let list = &grid.cells[&nearest_cell(&grid, p)];
            let dist = |k: usize| {
                let r = net.seg(k).unwrap();
                let a = net.node(r.node_a() as usize).unwrap().pos();
                let b = net.node(r.node_b() as usize).unwrap().pos();
                let (tt, mut q) = project_clamped(a, b, p);
                if r.flags() & RSF_CURVED != 0 {
                    q = net.seg_point(k, tt).unwrap();
                }
                ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
            };
            let best = list.iter().filter(|&&k| k >= 0 && (k as usize) < net.segs.len() && admitted(&net, k as usize, kind, 0).unwrap()).map(|&k| dist(k as usize)).fold(f32::MAX, f32::min);
            assert!(dist(nav.seg) <= best + 1e-4, "point {i} kind {kind}: piece {} at {} but {best} exists", nav.seg, dist(nav.seg));
            if nav.seg != seg {
                other_piece += 1;
            }
            // On the lane line: P0/P1 are the piece's end points at the offset, the carrot on the
            // line (lerp) or the Bezier at t.
            let (na, nb) = (net.node(js.node_a() as usize).unwrap().pos(), net.node(js.node_b() as usize).unwrap().pos());
            let (sa, sb) = (js.side_a(), js.side_b());
            let o = nav.lane_off;
            let (e0, e1) = if nav.end == 1 {
                ([na[0] + sa[0] * o, na[1] + sa[1] * o, na[2] + sa[2] * o], [nb[0] + sb[0] * o, nb[1] + sb[1] * o, nb[2] + sb[2] * o])
            } else {
                ([nb[0] - sb[0] * o, nb[1] - sb[1] * o, nb[2] - sb[2] * o], [na[0] - sa[0] * o, na[1] - sa[1] * o, na[2] - sa[2] * o])
            };
            assert!(near(nav.p0, e0, 1e-3) && near(nav.p1, e1, 1e-3), "point {i} kind {kind}: lane ends");
            let on = if js.flags() & RSF_CURVED != 0 {
                bezier([nav.p0, nav.c1, nav.c2, nav.p1], nav.t)
            } else {
                [e0[0] + (e1[0] - e0[0]) * nav.t, e0[1] + (e1[1] - e0[1]) * nav.t, e0[2] + (e1[2] - e0[2]) * nav.t]
            };
            assert!(near(nav.pos, on, 1e-3), "point {i} kind {kind}: carrot {:?} off its lane {:?}", nav.pos, on);
            assert!((0.0..=1.0).contains(&nav.t));
            let travel_dir = if nav.end == 1 { js.dir() } else { [-js.dir()[0], -js.dir()[1], -js.dir()[2]] };
            assert_eq!(nav.dir, travel_dir);
            if kind == 1 {
                // A traffic car sits on a table lane of its own side that the piece has.
                let t = &LANE_TABLE_DEFAULT[js.road_class()];
                assert!(nav.lane >= 0 && nav.lane <= js.lanes_pos().max(1), "point {i}: lane {} of {}", nav.lane, js.lanes_pos());
                assert_eq!(o, t[nav.lane as usize], "point {i}: offset {o} is lane {}'s", nav.lane);
            } else {
                assert_eq!(nav.lane, 0);
                max_side = max_side.max(o.abs());
                assert!(o.abs() <= dist(nav.seg) + 1e-3, "point {i}: a chaser's offset is at most its distance");
            }
        }
    }
    eprintln!("Paris: {joined} joins, {no_road} no road, {empty} empty cells, {other_piece} on another piece than the one the point was made from, widest chaser offset {max_side}");
    assert_eq!(joined + no_road + empty, 400);
    assert!(joined >= 390, "most points near a road join it ({joined}/400)");
}

#[test]
fn traffic_cars_take_the_model_box() {
    let Some(iso) = open_iso() else { return };
    let files = viv_files(&iso, "/DRIVING/MIS01.VIV", &|n| n.contains("track") && n.ends_with(".crp") || n == r"data\car\model\hatchback.crp");
    let Some((_, track)) = files.iter().find(|(n, _)| n.contains("track")) else { return };
    let Some((_, hb)) = files.iter().find(|(n, _)| n.contains("hatchback")) else { return };
    let crp = Crp::parse(track).unwrap();
    let net = RoadNet::parse_crp(&crp).unwrap();
    let grid = RoadGrid::parse_crp(&crp).unwrap();
    let h = car_half_extents(&Crp::parse(hb).unwrap()).unwrap();
    let params = nf_game::driving::CarParams::vanquish();
    let mut w = TrafficWorld::new(&net, grid, params, 2, 20, 0).unwrap();
    assert_eq!((w.box_x, w.cars[0].drv.brain.box_z), (BOX_FLOOR, BOX_FLOOR));
    w.set_extents(h);
    assert_eq!((w.box_x, w.cars[0].drv.brain.box_z, w.cars[1].drv.brain.box_z), (h[0], h[2], h[2]));
    assert_eq!(h, [0.81680465, 0.7761159, 1.9520375], "hatchback [D MIS01.VIV Base box]");
    eprintln!("hatchback half extents {h:?}");
}
