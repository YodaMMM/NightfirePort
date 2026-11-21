//! The race logic without the disc (`nf_game::driving::race` and the kind-4 road follower in
//! `ai.rs` / `join.rs`): the junction chooser `FUN_002235c8`, the kind-4 join filters of
//! `FUN_00225490` and the join `FUN_00225c48`, the
//! carrot walk along a chain, the rival's road-race speed (profile 0xf of `FUN_0012e808`, the
//! rubber band) and the race state the HUD reads. Every number is the game's rule applied to a
//! small road built here; the disc test (`drive_race_disc.rs`) runs the real track.

use nf_formats::driveai::{AiCommand, MapAi, AICO_RECORD};
use nf_formats::drivepath::MapPaths;
use nf_formats::trackstart::MapScript;
use std::collections::HashMap;

use nf_game::driving::ai::{next_segment_follow, DriveRng, NavHint, RoadGrid, RoadNav, NAV_KIND_FOLLOW, NAV_KIND_TRAFFIC};
use nf_game::driving::join::{nearest_piece, JoinOutcome, Nearest};
use nf_game::driving::patterns::Body;
use nf_game::driving::race::{clock_text, RaceDef, RacePhase, RaceState, Racer, Rival, RivalDrive, RivalSection};
use nf_game::driving::road::{LaneSet, RoadNet, RoadNode, RoadSeg};
use nf_game::driving::script::DriveScript;

/// A node payload: position, own id, links [layout: road.rs, M 0x225c48 / 0x2247b0].
fn node(id: u16, p: [f32; 3], links: &[u16]) -> Vec<u8> {
    let mut b = vec![0u8; 0x10 + 2 * links.len().max(1)];
    for k in 0..3 {
        b[k * 4..k * 4 + 4].copy_from_slice(&p[k].to_le_bytes());
    }
    b[0x0c..0x0e].copy_from_slice(&id.to_le_bytes());
    b[0x0e] = links.len() as u8;
    for (i, l) in links.iter().enumerate() {
        b[0x10 + 2 * i..0x12 + 2 * i].copy_from_slice(&l.to_le_bytes());
    }
    b
}

/// A segment payload A → B with its direction, limit, flags and car group [layout: road.rs].
fn seg(id: u16, a: u16, b: u16, pa: [f32; 3], pb: [f32; 3], flags: u16, group: u8) -> Vec<u8> {
    let mut v = vec![0u8; 0x60];
    v[0x0c..0x0e].copy_from_slice(&a.to_le_bytes());
    v[0x0e..0x10].copy_from_slice(&b.to_le_bytes());
    v[0x3e..0x40].copy_from_slice(&id.to_le_bytes());
    let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let side = [d[2] / l, 0.0, -d[0] / l];
    for k in 0..3 {
        v[0x20 + 4 * k..0x24 + 4 * k].copy_from_slice(&side[k].to_le_bytes());
        v[0x30 + 4 * k..0x34 + 4 * k].copy_from_slice(&side[k].to_le_bytes());
        v[0x40 + 4 * k..0x44 + 4 * k].copy_from_slice(&(d[k] / l).to_le_bytes());
    }
    v[0x4c..0x50].copy_from_slice(&40.0f32.to_le_bytes());
    v[0x50..0x52].copy_from_slice(&flags.to_le_bytes());
    v[0x5a] = group;
    v
}

struct Net {
    nodes: Vec<Vec<u8>>,
    segs: Vec<Vec<u8>>,
}

impl Net {
    fn net(&self) -> RoadNet<'_> {
        RoadNet {
            nodes: self.nodes.iter().map(|d| RoadNode { data: d }).collect(),
            segs: self.segs.iter().map(|d| RoadSeg { data: d }).collect(),
            routes: Vec::new(),
            ri: Vec::new(),
            rj: Vec::new(),
            has_plain_road: true,
        }
    }
}

/// A race chain along +z: n0 (0,0,0) → n1 (0,0,100) → n2 (0,0,200), pieces 0 and 1, flag 8,
/// group 0x3f; node 1 also has a reversed piece 2 (n3 → n1, from (50,0,100)) and a branch piece 3
/// (n1 → n4 at (-100,0,100)), so node 1 has four links: 0, 1, 2, 3.
fn junction() -> Net {
    let p = [[0.0, 0.0, 0.0], [0.0, 0.0, 100.0], [0.0, 0.0, 200.0], [50.0, 0.0, 100.0], [-100.0, 0.0, 100.0]];
    Net {
        nodes: vec![node(0, p[0], &[0]), node(1, p[1], &[0, 1, 2, 3]), node(2, p[2], &[1]), node(3, p[3], &[2]), node(4, p[4], &[3])],
        segs: vec![
            seg(0, 0, 1, p[0], p[1], 8, 0x3f),
            seg(1, 1, 2, p[1], p[2], 8, 0x3f),
            seg(2, 3, 1, p[3], p[1], 8, 0x3f),
            seg(3, 1, 4, p[1], p[4], 8, 0x3f),
        ],
    }
}

/// The plain chain: the same three nodes and pieces 0, 1 only (node 1 has two links).
fn chain() -> Net {
    let p = [[0.0, 0.0, 0.0], [0.0, 0.0, 100.0], [0.0, 0.0, 200.0]];
    Net {
        nodes: vec![node(0, p[0], &[0]), node(1, p[1], &[0, 1]), node(2, p[2], &[1])],
        segs: vec![seg(0, 0, 1, p[0], p[1], 8, 0x3f), seg(1, 1, 2, p[1], p[2], 8, 0x3f)],
    }
}

fn nav_on(net: &RoadNet, s: usize, t: f32) -> RoadNav {
    RoadNav::join(net, s, t, 0, true, NAV_KIND_FOLLOW, LaneSet::Snow2a).unwrap()
}

/// One grid cell over the whole test road listing every piece (the cell `FUN_00225490` reads).
fn one_cell(net: &RoadNet) -> RoadGrid {
    let mut cells = HashMap::new();
    cells.insert(0, (0..net.segs.len() as i16).collect());
    RoadGrid { min_x: -1000.0, min_z: -1000.0, inv_cell: 0.0001, rows: 1, cols: 1, cells }
}

/// `next_segment_follow` on a copy of `nav`: (piece, end), and the navigator after the call (the
/// chooser writes the end byte nav +0x3c, its `param_3`, in place) [M 0x2235c8].
fn follow(net: &RoadNet, nav: &RoadNav, hint: [f32; 3], rng: &mut dyn FnMut() -> u16) -> ((usize, u8), RoadNav) {
    let mut n = nav.clone();
    let mut h = hint;
    let s = next_segment_follow(net, &mut n, &mut h, rng).unwrap();
    ((s, n.end), n)
}

#[test]
fn kind4_chooser_follows_the_chain_and_turns_at_a_dead_end() {
    let n = chain();
    let net = n.net();
    let mut rng = || -> u16 { panic!("a two-link node draws nothing") };
    // Node 1 has two links: the other piece whose node[end] is not node 1 [M 0x2235c8].
    assert_eq!(follow(&net, &nav_on(&net, 0, 0.5), [0.0; 3], &mut rng).0, (1, 1));
    // Node 2 has one link: the same piece, end flipped.
    assert_eq!(follow(&net, &nav_on(&net, 1, 0.5), [0.0; 3], &mut rng).0, (1, 0));
}

/// Two links and neither continues the chain's sense (the other piece also ends at the exit
/// node): the chooser keeps the current piece and its end. The loop at 0x2239b0 falls out to
/// 0x2239f4, which returns s6, loaded with nav +0x3e (the current piece) at entry [M 0x2235c8
/// disassembly]; it does not take the other piece.
#[test]
fn kind4_chooser_at_a_two_link_node_with_no_continuing_piece_stays() {
    let p = [[0.0, 0.0, 0.0], [0.0, 0.0, 100.0], [50.0, 0.0, 100.0]];
    let n = Net {
        nodes: vec![node(0, p[0], &[0]), node(1, p[1], &[0, 1]), node(2, p[2], &[1])],
        segs: vec![seg(0, 0, 1, p[0], p[1], 8, 0x3f), seg(1, 2, 1, p[2], p[1], 8, 0x3f)],
    };
    let net = n.net();
    let mut rng = || -> u16 { panic!("a two-link node draws nothing") };
    assert_eq!(follow(&net, &nav_on(&net, 0, 0.5), [0.0; 3], &mut rng).0, (0, 1));
}

#[test]
fn kind4_chooser_at_a_junction_draws_among_same_sense_links() {
    let n = junction();
    let net = n.net();
    // From piece 0 toward node 1: piece 2 ends at node 1 (its node[1] is node 1) and is not
    // eligible; pieces 1 and 3 are. (2 · 0xc000) >> 16 = 1 → the second eligible one, piece 3.
    let nav = nav_on(&net, 0, 0.5);
    let mut draws = 0;
    let r = |v: u16| {
        move || -> u16 {
            v
        }
    };
    let got = follow(&net, &nav, [0.0; 3], &mut || {
        draws += 1;
        0xc000
    })
    .0;
    assert_eq!((got, draws), ((3, 1), 1));
    assert_eq!(follow(&net, &nav, [0.0; 3], &mut r(0x7fff)).0, (1, 1));
    // A one-shot link index (+0xb0) wins at a node of three or more links and is used up.
    let mut forced = nav.clone();
    forced.slot = 2;
    let (got, after) = follow(&net, &forced, [0.0; 3], &mut r(0));
    assert_eq!((got, after.slot), ((2, 1), -1));
    // Out of range: link 0.
    forced.slot = 9;
    assert_eq!(follow(&net, &forced, [0.0; 3], &mut r(0)).0, (0, 1));
}

#[test]
fn kind4_chooser_with_a_hint_takes_the_best_aligned_link() {
    let n = junction();
    let net = n.net();
    let mut r = || -> u16 { panic!("the hinted pick draws nothing") };
    let mut nav = nav_on(&net, 0, 0.5);
    nav.hint_toward = true;
    // Hint −x: piece 3 runs from node 1 toward −x; end 1 (node 1 is its node A).
    assert_eq!(follow(&net, &nav, [-1.0, 0.0, 0.0], &mut r).0, (3, 1));
    // Hint +x: piece 2 ends at node 1, so its direction away from node 1 is −dir = +x; end 0.
    assert_eq!(follow(&net, &nav, [5.0, 0.0, 0.0], &mut r).0, (2, 0));
    // Hint +z: piece 1.
    nav.hint_toward = false;
    nav.hint_away = true;
    assert_eq!(follow(&net, &nav, [0.0, 0.0, 2.0], &mut r).0, (1, 1));
}

#[test]
fn kind4_join_takes_only_its_group_on_flag8_pieces() {
    let p = [[0.0, 0.0, 0.0], [0.0, 0.0, 100.0], [10.0, 0.0, 0.0], [10.0, 0.0, 100.0], [5.0, 0.0, 0.0], [5.0, 0.0, 100.0]];
    let n = Net {
        nodes: (0..6).map(|i| node(i as u16, p[i], &[(i / 2) as u16])).collect(),
        segs: vec![
            seg(0, 0, 1, p[0], p[1], 8, 0x3f),    // the race group
            seg(1, 2, 3, p[2], p[3], 0, 0),       // plain road
            seg(2, 4, 5, p[4], p[5], 0x108, 0x3f), // flag 0x100: never
        ],
    };
    let net = n.net();
    let at = [6.0, 0.0, 50.0];
    // The nearest piece (2, 1 unit off) has flag 0x100; kind 4 takes the group piece 0 (6 off),
    // kind 1 the plain piece 1 (4 off) [M 0x225490].
    let grid = one_cell(&net);
    let piece = |kind, group| match nearest_piece(&net, &grid, kind, group, at).unwrap() {
        Nearest::Piece { seg, t, .. } => Some((seg, t)),
        _ => None,
    };
    assert_eq!(piece(NAV_KIND_FOLLOW, 0x3f), Some((0, 0.5)));
    assert_eq!(piece(NAV_KIND_TRAFFIC, 0).map(|x| x.0), Some(1));
    assert_eq!(piece(NAV_KIND_FOLLOW, 0x20), None, "another group joins nothing");
    // The join on the centre line (avoidance off), heading -z: a kind-4 navigator with no hint
    // flag still takes the piece's own A->B sense [M 0x225c48].
    let mut nav = RoadNav::cleared(NAV_KIND_FOLLOW, LaneSet::Snow2a);
    nav.group = 0x3f;
    assert_eq!(nav.join_nearest(&net, &grid, at, [0.0, 0.0, -1.0], true).unwrap(), JoinOutcome::Joined);
    assert_eq!((nav.seg, nav.end, nav.kind, nav.group, nav.lane_off), (0, 1, NAV_KIND_FOLLOW, 0x3f, 0.0));
    assert_eq!(nav.pos, [0.0, 0.0, 50.0], "the carrot at the closest point");
}

#[test]
fn kind4_carrot_walks_on_to_the_next_piece_keeping_its_offset() {
    let n = chain();
    let net = n.net();
    let mut nav = nav_on(&net, 0, 0.9);
    nav.advance_with(&net, 30.0, NavHint { dir: [0.0, 0.0, 1.0], target: -1 }, &mut || 0).unwrap();
    assert_eq!((nav.seg, nav.end), (1, 1));
    assert!((nav.t - 0.2).abs() < 1e-5, "10 units used on piece 0, 20 into piece 1: t {}", nav.t);
    assert!((nav.pos[2] - 120.0).abs() < 1e-3, "{:?}", nav.pos);
    // Kind 2 (the pedestrian chooser 0x2247b0) is not ported and is refused.
    let mut other = nav_on(&net, 0, 0.5);
    other.kind = 2;
    assert!(other.advance(&net, 10.0, &mut || 0).is_err());
}

/// An AICo record with the road-racer fields [D: RACE AICo 0: pattern 4, profile 0xf, follow 1,
/// catch-up 1.1, cap 50, group 0x3f, +0x7d 1, lane −1].
fn racer_command(cap: f32) -> Vec<u8> {
    let mut b = vec![0u8; AICO_RECORD];
    let f = |b: &mut Vec<u8>, o: usize, v: f32| b[o..o + 4].copy_from_slice(&v.to_le_bytes());
    f(&mut b, 0x4c, 1.0);
    f(&mut b, 0x5c, 1.1);
    f(&mut b, 0x60, cap);
    f(&mut b, 0x68, 0.3);
    f(&mut b, 0x6c, 0.2);
    b[0x79] = 4;
    b[0x7a] = 0xf;
    b[0x7d] = 1;
    b[0x7e] = 0xff;
    b[0x7f] = 0x3f;
    b
}

fn body(pos: [f32; 3], speed: f32) -> Body {
    Body { pos, vel: [0.0, 0.0, speed], side: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], fwd: [0.0, 0.0, 1.0], box_z: 2.3, box_x: 0.9 }
}

/// The rubber band on a 40-limit road: player ahead → max(40, 1.1·|v|) capped at 50; player
/// behind → 0.75 × 40 = 30 (0.85 inside twice the follow distance) [M 0x12e808 profile 0xf].
#[test]
fn rival_speed_is_the_road_limit_with_the_catch_up_band() {
    let n = chain();
    let net = n.net();
    let ai = MapAi { elements: Vec::new(), commands: vec![AiCommand::parse(&racer_command(50.0)).unwrap(), AiCommand::parse(&racer_command(60.0)).unwrap()], ..MapAi::default() };
    let (map, paths) = (MapScript::default(), MapPaths::default());
    let grid = one_cell(&net);
    let me = [0.0, 0.0, 50.0];
    let mut rng = DriveRng::game_reset();
    let target = |rv: &mut Rival, ppos: [f32; 3], pspeed: f32, rng: &mut DriveRng| {
        rv.step(&net, &paths, &body(me, 30.0), (ppos, [0.0, 0.0, pspeed], [0.0, 0.0, 1.0]), 1.0, rng).target_speed
    };
    let mut rv = Rival::new(1250.0, 60);
    assert!(rv.apply(0, &ai, &net, &grid, &map, &paths, me, [0.0, 0.0, 1.0]));
    assert!(!rv.apply(0, &ai, &net, &grid, &map, &paths, me, [0.0, 0.0, 1.0]), "the same command again does nothing");
    assert_eq!(rv.nav.as_ref().map(|n| (n.seg, n.group)), Some((0, 0x3f)));
    let ahead = [0.0, 0.0, 120.0];
    assert_eq!(target(&mut rv, ahead, 20.0, &mut rng), 40.0, "slow player ahead: the road limit");
    assert_eq!(target(&mut rv, ahead, 42.0, &mut rng), 42.0 * 1.1, "faster player ahead: 1.1 × its speed");
    assert_eq!(target(&mut rv, ahead, 70.0, &mut rng), 50.0, "capped at AICo +0x60");
    assert_eq!(target(&mut rv, [0.0, 0.0, 10.0], 45.0, &mut rng), 30.0, "player behind: 0.75 × the limit");
    assert_eq!(target(&mut rv, [0.0, 0.0, 48.5], 45.0, &mut rng), (40.0f32 * 0.85).max(45.0 * 0.85), "behind within 2 × follow");
    // AICo 6's cap of 60.
    assert!(rv.apply(1, &ai, &net, &grid, &map, &paths, me, [0.0, 0.0, 1.0]));
    assert_eq!(target(&mut rv, ahead, 70.0, &mut rng), 60.0);
    // The drive aims at the carrot ahead on the road with the throttle on.
    match rv.step(&net, &paths, &body(me, 10.0), (ahead, [0.0, 0.0, 45.0], [0.0, 0.0, 1.0]), 1.0, &mut rng).drive {
        RivalDrive::Pedals(c) => assert!(c.throttle > 0.0 && c.brake == 0.0, "{c:?}"),
        d => panic!("{d:?}"),
    }
}

fn race_def() -> RaceDef {
    RaceDef {
        start_trigger: 0,
        countdown_timer: 5,
        go_rule: 7,
        countdown_texts: vec![(4.0, 0x8ff), (3.0, 0x8fe), (2.0, 0x8fd), (1.0, 0x227c)],
        clock_timer: 0,
        clock_start: 1.0,
        clock_rate: 1,
        rival_element: 0,
        rival_selector: 0,
        rival_start_command: 3,
        rival_go_command: 0,
        sections: vec![RivalSection { trigger: 3, command: 4 }],
        player_line: 1,
        rival_line: 2,
        win_zone: 4,
        win_timer: 9,
        win_delay: 5.0,
        win_below: 1.0,
        finish_text: Some(0x20e),
        finish_score: 1,
        rival_death_wins: true,
        max_traffic: [6, 8, 10],
        speed_factor: [1.0; 3],
        health_factor: [1.0, 1.2, 1.4],
        rival_health: 1250.0,
    }
}

/// The countdown texts as timer 5 runs down from 5 s, the go when the clock starts, the clock
/// face, and the finish attributed to the car that ran its line first.
#[test]
fn race_state_follows_the_countdown_the_clock_and_the_line() {
    let def = race_def();
    let mut s = DriveScript::new(MapScript::default());
    let mut st = RaceState::default();
    // Timer 5 armed as the trigger beside the start does: 5 s (301 ticks), counting down.
    s.timers[5].value = (5.0 * 60.0 + 1.0) as i32;
    s.timers[5].rate = -1;
    s.timers[5].running = true;
    s.timers[5].start = 0;
    let mut shown = Vec::new();
    for _ in 0..300 {
        s.now += 1;
        if s.now == 243 {
            // The go rule: the clock set to 1 s and started [D rule 7].
            s.timers[0].value = 61;
            s.timers[0].rate = 1;
            s.timers[0].start = s.now;
            s.timers[0].running = true;
        }
        st.observe(&def, &s, &[], 1);
        if let Some((id, at, _)) = st.countdown_text {
            if shown.last() != Some(&(id, at)) {
                shown.push((id, at));
            }
        }
    }
    // Below 4 s at tick 62 (value 301 − 62 = 239 < 240), 3 s at 122, 2 s at 182, 1 s at 242.
    assert_eq!(shown, vec![(0x8ff, 62), (0x8fe, 122), (0x8fd, 182), (0x227c, 242)]);
    assert_eq!((st.phase, st.go_tick), (RacePhase::Racing, Some(243)));
    let h = st.hud(&def, &s);
    assert_eq!((h.lap, h.laps, h.racers, h.position), (1, 1, 2, 1));
    assert_eq!(h.clock_ticks, 61 + 57);
    assert_eq!(h.clock_text.as_deref(), Some(" 0:01"));
    assert_eq!(clock_text(61 + 60 * 83), Some(" 1:24".into()));
    // The rival (walk slot 1) runs its line first; the player's line afterwards changes nothing.
    st.progress = [500.0, 400.0];
    st.observe(&def, &s, &[(2, 1)], 1);
    assert_eq!((st.first_over, st.position(), st.phase), (Some(Racer::Rival), 2, RacePhase::Finishing));
    st.observe(&def, &s, &[(1, 0)], 1);
    assert_eq!((st.first_over, st.finish_clock), (Some(Racer::Rival), None));
    // A fresh race: the player first, the finish clock kept as the one lap's time.
    let mut st2 = RaceState::default();
    st2.go_tick = Some(243);
    st2.observe(&def, &s, &[(2, 7), (1, 0)], 1);
    // Both lines in one step: the rival's list loses the drive.
    let mut st3 = RaceState::default();
    st3.observe(&def, &s, &[(1, 0), (2, 1)], 1);
    assert_eq!((st3.first_over, st3.finish_clock), (Some(Racer::Rival), None));
    assert_eq!((st2.first_over, st2.finish_clock), (Some(Racer::Player), Some(61 + 57)), "slot 7 is not the rival's car");
    assert_eq!(st2.hud(&def, &s).best_lap_ticks, Some(118));
    assert_eq!(def.rival_health_at(1), 1500.0);
}
