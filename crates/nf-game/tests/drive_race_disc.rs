//! The race on the disc (`nf_game::driving::race`, RACE.VIV / `snow2a_race`): the race read out
//! of the track's script and AI data, and one full race run through the game's own triggers and
//! rules with the rival on the ported AI and car physics.
//!
//! Reads the full retail image named by `NF_ISO`; skips cleanly when it is unset or the image has
//! no `DRIVING/` extents.

use nf_formats::crp::Crp;
use nf_formats::crp_mesh::CollisionTri;
use nf_formats::driveai::MapAi;
use nf_formats::drivepath::{map_paths, MapPaths};
use nf_formats::iso9660::IsoImage;
use nf_formats::trackstart::{map_script, MapScript, TRIGGER_ENABLED};
use nf_game::driving::ai::{DriveRng, RoadGrid};
use nf_game::driving::intro::IntroState;
use nf_game::driving::patterns::Body;
use nf_game::driving::race::{section_applies, RaceDef, RacePhase, RaceRoute, RaceState, Racer, Rival, RivalDrive, RivalSection};
use nf_game::driving::road::RoadNet;
use nf_game::driving::script::{CarBox, CarFacts, DriveScript, DriveState, ElementFacts, Facts, Output};
use nf_game::driving::script_effects::Effect;
use nf_game::driving::{Car, CarInput, CarParams, CollisionGround, Rigid, DT};

struct Track {
    crp: Vec<u8>,
    rival_atr: String,
}

fn load() -> Option<Track> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    let e = iso.find("/DRIVING/RACE.VIV")?;
    let g = nf_formats::disc::read_bigf(&iso, e).ok()?;
    let base = e.lba as u64 * 2048;
    let read = |pred: &dyn Fn(&str) -> bool| -> Option<Vec<u8>> {
        let x = g.entries.iter().find(|x| pred(&x.name.to_ascii_lowercase()))?;
        let raw = iso.read_bytes(base + x.offset as u64, x.size as u64).ok()?;
        if raw.iter().take(64).all(|&b| b == 0) {
            return None;
        }
        Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
    };
    let Some(crp) = read(&|n| n.starts_with("data\\track\\") && n.ends_with(".crp")) else {
        eprintln!("skip: no RACE track extent in this image");
        return None;
    };
    let rival_atr = String::from_utf8_lossy(&read(&|n| n == "data\\sim\\attrib\\pvehicle\\paris_hench.atr")?).into_owned();
    Some(Track { crp, rival_atr })
}

fn parts(crp: &Crp) -> (MapScript, MapAi, MapPaths, Vec<CollisionTri>) {
    (map_script(crp).unwrap(), MapAi::from_crp(crp).unwrap(), map_paths(crp).unwrap(), crp.extract_collision_tris().unwrap())
}

/// The race as the RACE data defines it [D: every value read off the disc by `RaceDef`; the
/// rules and triggers are listed in race.rs's module doc].
#[test]
fn race_definition_reads_the_race_track() {
    let Some(t) = load() else { return };
    let crp = Crp::parse(&t.crp).unwrap();
    let (map, ai, _, _) = parts(&crp);
    let d = RaceDef::from_track(&map, &ai).expect("RACE has the race shape");
    assert_eq!((d.start_trigger, d.countdown_timer, d.go_rule), (212, 5, 7));
    assert_eq!(d.countdown_texts, vec![(4.0, 0x8ff), (3.0, 0x8fe), (2.0, 0x8fd), (1.0, 0x227c)]);
    assert_eq!((d.clock_timer, d.clock_start, d.clock_rate), (0, 1.0, 1));
    assert_eq!((d.rival_element, d.rival_start_command, d.rival_go_command), (0, 3, 0));
    assert_eq!(map.ref_path(d.rival_selector), Some("CARP::<<Map>>::{AIEl}::0"));
    let sec = |trigger, command| RivalSection { trigger, command };
    assert_eq!(d.sections, vec![sec(215, 4), sec(220, 6), sec(221, 4), sec(226, 1), sec(227, 0)]);
    assert_eq!((d.player_line, d.rival_line, d.win_zone), (228, 229, 214));
    assert_eq!((d.win_timer, d.win_delay, d.win_below), (9, 5.0, 1.0));
    assert_eq!((d.finish_text, d.finish_score, d.rival_death_wins), (Some(0x20e), 1, true));
    assert_eq!(d.max_traffic, [6, 8, 10]);
    assert_eq!(d.speed_factor, [1.0; 3]);
    assert_eq!(d.health_factor, [1.0, 1.2, 1.4]);
    assert_eq!(d.rival_health, 1250.0);
    assert_eq!(d.rival_health_at(2), 1750.0);
    // The rival's commands: hold at the start, then the road follower (profile 0xf, group 0x3f)
    // with its caps, and the bridge jump on a spline [D AICo].
    let c = |i: usize| &ai.commands[i];
    assert_eq!((c(3).pattern, c(3).profile), (1, 1), "the spawn command holds");
    for (i, cap) in [(0, 50.0), (4, 51.0), (6, 60.0)] {
        assert_eq!((c(i).pattern, c(i).profile, c(i).nav_byte, c(i).speed_cap, c(i).catchup), (4, 0xf, 0x3f, cap, 1.1), "AICo {i}");
        assert_eq!((c(i).follow, c(i).hint_toward, c(i).hint_away, c(i).lane), (1.0, 0, 0, -1), "AICo {i}");
    }
    assert_eq!(c(6).launch_a, 25.0);
    assert_eq!((c(1).pattern, map.ref_path(c(1).pattern_arg)), (5, Some("CARP::<<Map>>::{AISp}::112")));
    assert_eq!(ai.elements[0].name, "paris_hench");
}

/// The race road: the rival group's kind-4 pieces are three one-way chains with no junction, the
/// whole course about 5.3 km from the start to past the finish [D RNgp].
#[test]
fn race_road_is_three_chains_from_the_start_past_the_finish() {
    let Some(t) = load() else { return };
    let crp = Crp::parse(&t.crp).unwrap();
    let (map, _, _, _) = parts(&crp);
    let net = RoadNet::parse_crp(&crp).unwrap();
    let grid = RoadGrid::parse_crp(&crp).unwrap();
    for n in &net.nodes {
        assert!(n.link_count() <= 2, "a race node with a junction");
    }
    let start = map.triggers[212].pos;
    let route = RaceRoute::build(&net, &grid, 0x3f, start).unwrap();
    eprintln!("route: {} chains, {} points, {:.0} long", route.chains, route.points.len(), route.length());
    assert_eq!(route.chains, 3);
    let near = |p: [f32; 3]| route.points.iter().map(|q| ((p[0] - q[0]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()).fold(f32::MAX, f32::min);
    assert!(near(start) < 5.0, "the route starts at the start line");
    let line = map.triggers[228].pos;
    assert!(near(line) < 20.0, "the route crosses the finish line ({})", near(line));
    let at_line = route.progress(line, route.length() * 0.9, (route.length(), route.length()));
    assert!(at_line > route.length() * 0.8 && at_line < route.length(), "the line is near the end: {at_line}");
    assert!((5000.0..5800.0).contains(&route.length()), "{}", route.length());
}

fn body_of(car: &Car) -> Body {
    let (s, c) = car.yaw.sin_cos();
    let h = car.params.half_extents;
    Body { pos: car.pos, vel: car.vel, side: [c, 0.0, -s], up: [0.0, 1.0, 0.0], fwd: [s, 0.0, c], box_z: h[2], box_x: h[0] }
}

fn route_point(r: &RaceRoute, s: f32) -> ([f32; 3], [f32; 3]) {
    let s = s.clamp(0.0, r.length());
    let k = r.dist.iter().position(|&d| d >= s).unwrap_or(r.points.len() - 1).max(1);
    let (a, b) = (r.points[k - 1], r.points[k]);
    let l = (r.dist[k] - r.dist[k - 1]).max(1e-6);
    let t = ((s - r.dist[k - 1]) / l).clamp(0.0, 1.0);
    let d = [(b[0] - a[0]) / l, (b[1] - a[1]) / l, (b[2] - a[2]) / l];
    ([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t], d)
}

#[derive(Debug)]
struct Outcome {
    phase: RacePhase,
    first: Option<Racer>,
    ticks: i32,
    go_tick: Option<i32>,
    finish_clock: Option<i32>,
    rival_progress: f32,
    player_progress: f32,
    score: i32,
    /// The (trigger, command) changes the rival took, in order.
    sections: Vec<(usize, usize)>,
    /// The countdown texts and the tick each went up.
    texts: Vec<(u32, i32)>,
    places: std::collections::BTreeSet<u8>,
    laps: std::collections::BTreeSet<(u8, u8)>,
    clock_backwards: bool,
    /// The rival's largest distance from the race road (off the bridge path).
    worst_rival_off_road: f32,
}

/// One race. The player is the harness's stand-in [S]: a point that waits out the car stop, then
/// runs along [`RaceRoute`] at `player_speed`, rides the bridge spline from trigger 224 to the
/// path's end, coasts on to trigger 225 and re-joins the route. The rival is the port's `Rival`
/// AI; its car is a point the AI's pedals drive (`physics == false`, [S]: the turn, acceleration
/// and brake rates are the harness's) or the port's `Car` on the track's collision ground. The
/// script's own triggers and rules decide the race.
fn run_race(t: &Track, player_speed: f32, max_ticks: i32, physics: bool, trace: bool) -> Outcome {
    let crp = Crp::parse(&t.crp).unwrap();
    let (map, ai, paths, tris) = parts(&crp);
    let net = RoadNet::parse_crp(&crp).unwrap();
    let grid = RoadGrid::parse_crp(&crp).unwrap();
    let def = RaceDef::from_track(&map, &ai).unwrap();
    let route = RaceRoute::build(&net, &grid, 0x3f, map.triggers[def.start_trigger].pos).unwrap();
    let ground = CollisionGround::new(tris);
    let rigid = Rigid::game_defaults();
    let mut rng = DriveRng::game_reset();
    let mut script = DriveScript::new(map.clone());
    let mut intro = IntroState::new(60);
    let mut state = RaceState::default();
    // The rival: its AIEl placement and its spawn command (hold) [D AIEl 0 / AICo 3].
    let el = &ai.elements[def.rival_element];
    let mut car = Car::new(CarParams::parse_atr(&t.rival_atr), el.pos, el.facing[0].atan2(el.facing[2]));
    car.pos[1] += 1.0;
    let mut rival = Rival::new(def.rival_health_at(0), 60);
    rival.apply(def.rival_start_command, &ai, &net, &grid, &map, &paths, car.pos, body_of(&car).fwd);
    let half = car.params.half_extents;
    let radius = (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt();
    let mut s = route.progress(map.triggers[def.start_trigger].pos, 0.0, (10.0, 50.0));
    let (mut ppos, mut pdir) = route_point(&route, s);
    let mut pvel = [0.0f32; 3];
    let (mut on_spline, mut spline_done, mut spline_t) = (false, false, 0.0f32);
    let mut rs = 0.0f32;
    let mut o = Outcome {
        phase: RacePhase::Countdown,
        first: None,
        ticks: 0,
        go_tick: None,
        finish_clock: None,
        rival_progress: 0.0,
        player_progress: 0.0,
        score: 0,
        sections: Vec::new(),
        texts: Vec::new(),
        places: Default::default(),
        laps: Default::default(),
        clock_backwards: false,
        worst_rival_off_road: 0.0,
    };
    let mut last_clock = 0;
    let mut tick = 0;
    while tick < max_ticks {
        tick += 1;
        // The player stand-in.
        let before = ppos;
        if on_spline && !spline_done && intro.spline_position(&paths).is_some() {
            if intro.path.time < spline_t {
                spline_done = true; // the path wrapped: coast on [S]
            } else {
                ppos = intro.spline_position(&paths).unwrap();
            }
            spline_t = intro.path.time;
        } else if on_spline && spline_done {
            ppos[0] += pvel[0] * DT;
            ppos[2] += pvel[2] * DT;
        } else if state.go_tick.is_some() && intro.stop.is_none() {
            s += player_speed * DT;
            let (p, d) = route_point(&route, s);
            ppos = p;
            pdir = d;
        }
        if !(on_spline && spline_done) {
            pvel = [(ppos[0] - before[0]) / DT, (ppos[1] - before[1]) / DT, (ppos[2] - before[2]) / DT];
        }
        // The facts the script reads: the player, the rival as the one kind-2 car (walk slot 1).
        let rspeed = (car.vel[0].powi(2) + car.vel[1].powi(2) + car.vel[2].powi(2)).sqrt();
        let mut facts = Facts {
            player_pos: ppos,
            player_vel: pvel,
            player_radius: 2.5,
            player_box: Some(CarBox::upright(pdir[0].atan2(pdir[2]), [0.92, 0.58, 2.32])),
            cars: vec![CarFacts { kind: 2, pos: car.pos, vel: car.vel, radius, bbox: Some(CarBox::upright(car.yaw, half)) }],
            difficulty: 0,
            ..Facts::default()
        };
        facts.elements.insert(def.rival_selector, ElementFacts { health: rival.health, speed: rspeed, pos: car.pos, ..ElementFacts::default() });
        script.step(&facts);
        let fired = script.take_fired();
        for out in script.take_output() {
            if let Output::Effect { effect, trigger, action, .. } = &out {
                intro.apply(action, &map, &paths);
                // The go rule's command to the rival (a rule: no trigger) [D rule 7].
                if let (Effect::AiCommand { command: Some(c), .. }, None) = (effect, trigger) {
                    if let Some(i) = c.path.rsplit("::").next().and_then(|x| x.parse::<usize>().ok()).map(|x| x / 0xa0) {
                        rival.apply(i, &ai, &net, &grid, &map, &paths, car.pos, body_of(&car).fwd);
                    }
                }
            }
        }
        for &(tr, slot) in &fired {
            if let Some(cmd) = section_applies(&def, tr, slot, 1) {
                if rival.apply(cmd, &ai, &net, &grid, &map, &paths, car.pos, body_of(&car).fwd) {
                    o.sections.push((tr, cmd));
                }
                script.trigger_flags[tr] &= !TRIGGER_ENABLED;
            }
            if tr == 224 && slot == 0 {
                on_spline = true;
            }
            if tr == 225 && slot == 0 && on_spline {
                on_spline = false;
                s = route.progress(ppos, s, (50.0, 400.0));
            }
        }
        intro.step();
        state.observe(&def, &script, &fired, 1);
        if let Some((id, at, _)) = state.countdown_text {
            if o.texts.last() != Some(&(id, at)) {
                o.texts.push((id, at));
            }
        }
        // The rival.
        let st = rival.step(&net, &paths, &body_of(&car), (ppos, pvel, pdir), def.speed_factor[0], &mut rng);
        if let Some(v) = rival.launch.take() {
            let (sn, cs) = car.yaw.sin_cos();
            car.vel = [sn * v, car.vel[1], cs * v];
        }
        match st.drive {
            RivalDrive::Pedals(c) if physics => {
                car.step(&CarInput { throttle: c.throttle, brake: c.brake, steer: c.steer, handbrake: c.handbrake }, &ground, &rigid, DT);
                // As `ai::AiDriver::step`: the velocity follows the heading [S].
                let (sn, cs) = car.yaw.sin_cos();
                let along = car.vel[0] * sn + car.vel[2] * cs;
                car.vel[0] = sn * along;
                car.vel[2] = cs * along;
            }
            RivalDrive::Pedals(c) => {
                // The point the pedals drive [S]: turn steer × 1.6 rad/s (less below 10 u/s, the
                // other way when rolling back), throttle × 14 u/s², brake × 22 u/s² and below
                // zero the reverse gear to −7.5 u/s; the height of the race road under it.
                let (sn, cs) = car.yaw.sin_cos();
                let v = car.vel[0] * sn + car.vel[2] * cs;
                let nv = if v > 0.0 || c.throttle > 0.0 {
                    (v + (c.throttle * 14.0 - c.brake * 22.0) * DT).max(if v > 0.0 { 0.0 } else { v })
                } else {
                    (v - c.brake * 10.0 * DT).max(-7.5)
                };
                car.yaw += c.steer * (v / 10.0).clamp(-1.0, 1.0) * 1.6 * DT;
                let (sn, cs) = car.yaw.sin_cos();
                car.vel = [sn * nv, 0.0, cs * nv];
                car.pos[0] += car.vel[0] * DT;
                car.pos[2] += car.vel[2] * DT;
                let y = route_point(&route, route.progress(car.pos, rs, (60.0, 300.0))).0[1];
                car.vel[1] = (y - car.pos[1]) / DT;
                car.pos[1] = y;
            }
            RivalDrive::Path { pos, vel } => {
                car.pos = pos;
                car.vel = vel;
                if vel[0] != 0.0 || vel[2] != 0.0 {
                    car.yaw = vel[0].atan2(vel[2]);
                }
            }
            // Past the end of the bridge path: the car coasts on [S].
            RivalDrive::Idle if rival.path.is_some() => {
                car.pos[0] += car.vel[0] * DT;
                car.pos[2] += car.vel[2] * DT;
            }
            RivalDrive::Idle => {}
        }
        assert!(car.pos.iter().all(|x| x.is_finite()), "tick {tick}: rival position");
        rs = route.progress(car.pos, rs, (60.0, 300.0));
        state.progress = [s, rs];
        if state.go_tick.is_some() && rival.path.is_none() {
            let (q, _) = route_point(&route, rs);
            let off = ((q[0] - car.pos[0]).powi(2) + (q[2] - car.pos[2]).powi(2)).sqrt();
            o.worst_rival_off_road = o.worst_rival_off_road.max(off);
        }
        let hud = state.hud(&def, &script);
        if state.go_tick.is_some() {
            o.places.insert(hud.position);
            o.laps.insert((hud.lap, hud.laps));
            o.clock_backwards |= hud.clock_ticks < last_clock;
            last_clock = hud.clock_ticks;
        }
        if trace && tick % 600 == 0 {
            eprintln!("tick {tick}: {:?} clock {:?} place {} player {s:.0} rival {rs:.0} AICo {:?} target {:.1}", state.phase, hud.clock_text, hud.position, rival.command, st.target_speed);
        }
        if matches!(script.state, DriveState::Won | DriveState::Lost) {
            break;
        }
    }
    o.phase = state.phase;
    o.first = state.first_over;
    o.ticks = tick;
    o.go_tick = state.go_tick;
    o.finish_clock = state.finish_clock;
    o.rival_progress = rs;
    o.player_progress = s;
    o.score = script.score[1];
    o
}

fn check_common(o: &Outcome, route_len: f32) {
    // The countdown as the script runs it (texts at 63, 123, 183 and the go at 243, as
    // driving_intro_disc.rs pins them).
    assert_eq!(o.texts, vec![(0x8ff, 63), (0x8fe, 123), (0x8fd, 183), (0x227c, 243)]);
    assert_eq!(o.go_tick, Some(243));
    assert_eq!(o.laps.iter().copied().collect::<Vec<_>>(), vec![(1, 1)]);
    assert!(o.places.iter().all(|p| (1..=2).contains(p)), "{:?}", o.places);
    assert!(!o.clock_backwards, "the race clock never runs back");
    // The sections the rival ran come in the course's order (a car can miss a box it passes
    // beside); one that got past the bridge (3400 along) ran the jump and the hand-back.
    let order = [(215, 4), (226, 1), (227, 0), (220, 6), (221, 4)];
    let mut k = 0;
    for x in &o.sections {
        k += order[k..].iter().position(|y| y == x).unwrap_or_else(|| panic!("section {x:?} out of order: {:?}", o.sections)) + 1;
    }
    if o.rival_progress > 3400.0 {
        assert!(o.sections.starts_with(&[(226, 1), (227, 0)]), "{:?}", o.sections);
    }
    assert!(o.rival_progress > route_len * 0.5, "the rival got round: {} of {route_len}", o.rival_progress);
}

/// A fast player (60 u/s): the rival behind drives at most its cap (50, 60 after trigger 220)
/// and loses ground; the player crosses line 228 first, gets the finish score, reaches box 214
/// and the win rule ends the race 4 s later (timer 9 = 5 s, rule 9 below 1 s).
#[test]
fn full_race_a_fast_player_wins() {
    let Some(t) = load() else { return };
    let o = run_race(&t, 60.0, 60 * 300, false, false);
    eprintln!("60 u/s: {o:?}");
    check_common(&o, 5366.0);
    assert_eq!((o.phase, o.first, o.score), (RacePhase::Won, Some(Racer::Player), 1));
    let fc = o.finish_clock.expect("the finish time");
    assert!((5000..6000).contains(&fc), "finish clock {fc}");
    assert!(o.rival_progress < o.player_progress, "the rival behind at the end");
}

/// The player at the rival's own pace (38 u/s, under the road limit): the rubber band keeps the
/// two together (the rival drives 1.1 × the player's speed when behind, 30 when ahead) and the
/// lead changes hands; the race still ends by a line.
#[test]
fn full_race_at_the_rivals_pace_is_close() {
    let Some(t) = load() else { return };
    let o = run_race(&t, 38.0, 60 * 300, false, false);
    eprintln!("38 u/s: {o:?}");
    check_common(&o, 5366.0);
    assert!(matches!(o.phase, RacePhase::Won | RacePhase::Lost), "{:?}", o.phase);
    assert_eq!(o.places, [1, 2].into_iter().collect(), "the lead changed hands");
    // The gap is a number of this harness, not of the game [S]. The rival's kind-4 join keeps the
    // lateral offset to the road width (avoidance on: AICo +0x7c = 0 → AI +0x154 = 1 →
    // `FUN_00225c48(…, 0)`, M 0x145ff8:251, 0x12e468), and the lane pick that would steer that
    // offset (`join::lane_pick`, FUN_00135f70) is not wired into the rival; with the centre-line
    // join the gap was under 1 unit, with the offset kept it is about 460.
    assert!((o.player_progress - o.rival_progress).abs() < 600.0, "a close finish: {} vs {}", o.player_progress, o.rival_progress);
}

/// A slow player (25 u/s): the rival leads, drives 0.75 × the road limit (30) while ahead, and
/// crosses its line 229 first: the script loses the race at once.
#[test]
fn full_race_the_rival_wins_against_a_slow_player() {
    let Some(t) = load() else { return };
    let o = run_race(&t, 25.0, 60 * 400, false, false);
    eprintln!("25 u/s: {o:?}");
    check_common(&o, 5366.0);
    assert_eq!((o.phase, o.first, o.score, o.finish_clock), (RacePhase::Lost, Some(Racer::Rival), 0, None));
    assert!(o.player_progress < o.rival_progress, "the rival ahead at the end");
}

/// The rival on the port's car physics (`Car` over the RACE collision ground) for the first ten
/// seconds after the go: it leaves the start under its own pedals along the race road.
#[test]
fn rival_on_car_physics_leaves_the_start_along_the_road() {
    let Some(t) = load() else { return };
    let o = run_race(&t, 38.0, 243 + 600, true, false);
    eprintln!("physics: rival at {:.0} after 10 s, off the road at most {:.1}", o.rival_progress, o.worst_rival_off_road);
    assert!(o.rival_progress > 150.0, "{}", o.rival_progress);
    assert!(o.worst_rival_off_road < 16.0, "{}", o.worst_rival_off_road);
}
