//! The race (RACE.VIV, track `snow2a_race`): one rival against the player from the start line to
//! a finish line, built from the track's own script, AI and road data. The countdown itself is
//! in `intro.rs` / `script.rs`; this module reads the race out of the data ([`RaceDef`]), drives
//! the rival ([`Rival`]), keeps the race state the HUD reads ([`RaceState`], [`RaceHud`]) and
//! measures progress along the race road ([`RaceRoute`]).
//!
//! What the game has, from the data and the code [D the RACE track `.crp`, M DRIVING.ELF]:
//!
//! - **No race machine.** DRIVING.ELF has no lap, checkpoint, position or rubber-band code of
//!   its own: the strings `snow2a_race` are only the per-unit track-name tables and the
//!   speech-bank pick in `FUN_00176658`; the HUD draw list `FUN_0023cf88` draws the health
//!   gauge (`FUN_0023da30`), the weapon and ammo (`FUN_00240c98`), objectives, score and one
//!   mission timer (`FUN_0023e580`), and no position or lap readout [M-negative]. The race is
//!   the mission script plus one AI car.
//! - **One lap, point to point.** The player starts at trigger 212; the rival (`AIEl` 0,
//!   `paris_hench`, flags persistent | chase brain | uncounted, health 1250) waits beside it on
//!   its spawn command (`AICo` 3: hold). The countdown rule (timer 5 below 1 s) ends the car
//!   stop, sends the rival `AICo` 0 and starts the race clock: timer 0 set to 1 s counting up,
//!   drawn by `ETimerDrawOn(0)` [D rule 7].
//! - **The rival's racing line is the road network**: `AICo` 0/4/6 are pattern 4 (road
//!   follower) with profile 0xf, car group 0x3f; a kind-4 navigator joins only flag-8 pieces of
//!   its group [M 0x225490], and on RACE those 220 pieces form three one-way chains (start →
//!   bridge, bridge → ledge, ledge → finish) with no junction [D]. The chooser is
//!   `FUN_002235c8` ([`super::ai::next_segment_follow`]).
//! - **The rubber band is profile 0xf** (`hostile::chase_target_speed`, `FUN_0012e808`): the
//!   rival drives the piece's limit (40 on every race piece [D]); with the player ahead it
//!   drives `max(limit, 1.1·|v_player|)`, with the player behind 0.75 × the limit (0.85 when
//!   inside twice the follow distance), always capped at the command's cap × the MSet speed
//!   factor: 50 (`AICo` 0), 51 (`AICo` 4) or 60 (`AICo` 6) [D, M].
//! - **Sections**: triggers 215/221 (cap 51), 220 (cap 60, launched at 25 along its heading),
//!   226 (pattern 5: the bridge jump on spline 1, `path_STDVanquishBridgeJump`) and 227 (back to
//!   `AICo` 0) fire for kind-2 cars; the handler applies a command only when the car that fired
//!   is the rival's and then clears the trigger's enable bit [M 0x145ff8].
//! - **The finish**: trigger 228 (the player) and 229 (kind 2) are the same line 7 units apart.
//!   The player's crossing disables 229, adds 1 to score 1 and shows text 0x20e; the rival's
//!   disables 214 and 228 and loses the race. The player then reaches the wide box 214 (timer 9
//!   = 5 s, stage 2) and rule 9 wins below 1 s. Destroying the rival also wins (rule 8: text
//!   0x20e, timer 9 = 4 s, stage 2). Stopping (speed below 10) within 15 of the police car
//!   (`AIEl` 1) for 5 s loses (rules 2/3/11/12/13) [D].
//! - **Difficulty**: the traffic cap 6 / 8 / 10 (trigger 213 by default, rules 4/5 swap in 222
//!   or 223 at difficulty 1 / 2), the MSet health factors 1.0 / 1.2 / 1.4 on the rival and the
//!   speed factors (1.0 at every difficulty on RACE) [D].
//!
//! What the port adds and the game does not show: the race position ([`RaceState::position`])
//! is the port's ranking by progress along [`RaceRoute`] [S]; the game shows none. "Lap" and
//! "best lap" are the one lap of a point-to-point race: lap 1 of 1, and the best lap is the
//! finish time once the player has crossed [S, a reading, not a game value].

use nf_formats::driveai::{AiCommand, MapAi, AICO_RECORD, AIEL_RECORD};
use nf_formats::drivepath::MapPaths;
use nf_formats::trackstart::{MapScript, TriggerAction, TRIGGER_BY_KIND2, TRIGGER_BY_PLAYER, TRIGGER_ENABLED};

use super::ai::{lookahead, normalize, sub, DriveCommand, DriveRng, NavHint, RoadGrid, RoadNav, NAV_KIND_FOLLOW, STATE_CHASE};
use super::join::{nav_for_pattern, nearest_piece, JoinOutcome, Nearest};
use super::hostile::{chase_target_speed, ram_tick, ChaseSpeedInputs, RamState};
use super::intro::PathPlayer;
use super::patterns::{
    aim_point, chase_drive, dispatch, AimInputs, Arm, Body, ChaseDriveInputs, DispatchInputs, ReverseState, PATTERN_PATH,
    PATTERN_ROAD_FOLLOW,
};
use super::road::{LaneSet, RoadNet};
use super::script::{DriveScript, DriveState, TICKS_PER_SECOND};

/// Action ids the race reads [D: the track's lists; names from `script::ACTIONS`].
pub mod act {
    pub const START_MISSION: u32 = 0x16;
    pub const DISPLAY_TEXT: u32 = 0x2c;
    pub const PLAYER_LOSE: u32 = 0x2d;
    pub const PLAYER_WIN: u32 = 0x2e;
    pub const SET_TIMER: u32 = 0x2f;
    pub const START_TIMER: u32 = 0x30;
    pub const DISABLE_TRIGGER: u32 = 0x35;
    pub const ENABLE_TRIGGER: u32 = 0x36;
    pub const AI_COMMAND: u32 = 0x41;
    pub const CHANGE_MAX_TRAFFIC: u32 = 0x78;
    pub const ADD_SCORE: u32 = 0x83;
    pub const END_CAR_STOP: u32 = 0xdb;
    pub const TIMER_DRAW_ON: u32 = 0x105;
}

/// The rule test types the race reads (`Rule` +0x00) [M 0x209318].
const RULE_DEATH: u32 = 3;
const RULE_TIMER: u32 = 9;
const RULE_DIFFICULTY: u32 = 13;

/// The record index an `sr` path names: `CARP::<<Map>>::{<table>}::<byte offset>` / `size`.
fn ref_index(map: &MapScript, word: u32, table: &str, size: usize) -> Option<usize> {
    let off: usize = map.ref_path(word)?.strip_prefix(&format!("CARP::<<Map>>::{{{table}}}::"))?.parse().ok()?;
    (off % size == 0).then_some(off / size)
}

/// One rival command change along the race: when the rival's car runs `trigger`, it gets
/// `AICo` `command` [D, M 0x145ff8].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RivalSection {
    pub trigger: usize,
    pub command: usize,
}

/// The race as the track's data defines it. Every field is found by its role in the script, not
/// by index, so the same reader runs on any track with this shape ([`RaceDef::from_track`]
/// returns `None` without it).
#[derive(Clone, Debug, PartialEq)]
pub struct RaceDef {
    /// The `EStartMission` trigger (RACE 212).
    pub start_trigger: usize,
    /// The countdown timer (5) and the rule that ends it (7: `EEndCarStop`).
    pub countdown_timer: usize,
    pub go_rule: usize,
    /// The countdown's texts: (seconds the timer passes downward, text id), first shown first:
    /// (4, 0x8ff), (3, 0x8fe), (2, 0x8fd), (1, 0x227c) on RACE [D rules 0/1/6/7].
    pub countdown_texts: Vec<(f32, u32)>,
    /// The race clock: timer 0, set to `clock_start` seconds and counted at `clock_rate` per
    /// tick by the go rule, drawn by `ETimerDrawOn` [D rule 7].
    pub clock_timer: usize,
    pub clock_start: f32,
    pub clock_rate: i32,
    /// The rival `AIEl` index (0), its selector word for the rules (`Facts::elements` key), its
    /// spawn command (3) and the command the go rule gives it (0).
    pub rival_element: usize,
    pub rival_selector: u32,
    pub rival_start_command: usize,
    pub rival_go_command: usize,
    /// The command changes along the race [D: 215 → 4, 220 → 6, 221 → 4, 226 → 1, 227 → 0].
    pub sections: Vec<RivalSection>,
    /// The finish line for the player (228) and for the rival (229), and the box that starts the
    /// win timer after the player's crossing (214).
    pub player_line: usize,
    pub rival_line: usize,
    pub win_zone: usize,
    /// The win timer (9), its seconds from the zone (5) and the rule threshold that wins (1 s).
    pub win_timer: usize,
    pub win_delay: f32,
    pub win_below: f32,
    /// What the player's crossing gives: the text id (0x20e) and the `EAddScoreAction` amount
    /// added to score 1 (1).
    pub finish_text: Option<u32>,
    pub finish_score: i32,
    /// A `SRuleDeath` rule on the rival leads to the win timer (rule 8).
    pub rival_death_wins: bool,
    /// The traffic cap by difficulty 0 / 1 / 2 (6 / 8 / 10).
    pub max_traffic: [i32; 3],
    /// MSet +0x2b8.. speed and +0x2c4.. health factors by difficulty [M 0x208468 / 0x2084a8].
    pub speed_factor: [f32; 3],
    pub health_factor: [f32; 3],
    /// The rival's base health (`AIEl` +0x4c, 1250).
    pub rival_health: f32,
}

impl RaceDef {
    /// Reads the race out of a track's script and AI set. `None` when the track has no race
    /// shape (a countdown rule with `EEndCarStop` and an `EAICommand`, a kind-2 trigger that
    /// loses, a player trigger that disables it).
    pub fn from_track(map: &MapScript, ai: &MapAi) -> Option<RaceDef> {
        let has = |l: &[TriggerAction], id: u32| l.iter().any(|a| a.id == id);
        let start_trigger = map.triggers.iter().position(|t| has(&t.actions, act::START_MISSION))?;
        let go = map.rules.iter().find(|r| r.word(0) == RULE_TIMER && has(&r.actions, act::END_CAR_STOP))?;
        let countdown_timer = go.word(0x10) as usize;
        let go_stage = go.word(8);
        let mut countdown_texts: Vec<(f32, u32)> = map
            .rules
            .iter()
            .filter(|r| r.word(0) == RULE_TIMER && r.word(8) == go_stage && r.word(0x10) as usize == countdown_timer)
            .filter_map(|r| r.actions.iter().find(|a| a.id == act::DISPLAY_TEXT).map(|a| (f32::from_bits(r.word(0x18)), a.word(1))))
            .collect();
        countdown_texts.sort_by(|a, b| b.0.total_cmp(&a.0));
        let set = go.actions.iter().find(|a| a.id == act::SET_TIMER)?;
        let clock_timer = set.word(1) as usize;
        let clock_rate = go.actions.iter().find(|a| a.id == act::START_TIMER && a.word(1) as usize == clock_timer)?.word(2) as i32;
        let cmd = go.actions.iter().find(|a| a.id == act::AI_COMMAND)?;
        let rival_go_command = ref_index(map, cmd.word(1), "AICo", AICO_RECORD)?;
        let rival_selector = cmd.word(3);
        let rival_element = ref_index(map, rival_selector, "AIEl", AIEL_RECORD)?;
        let el = ai.elements.get(rival_element)?;
        let rival_start_command = ref_index(map, el.command_ref, "AICo", AICO_RECORD)?;
        let mut sections = Vec::new();
        for t in &map.triggers {
            if t.flags & TRIGGER_BY_KIND2 == 0 {
                continue;
            }
            for a in t.actions.iter().filter(|a| a.id == act::AI_COMMAND) {
                if a.word(2) != 0 && a.word(3) == rival_selector {
                    if let Some(c) = ref_index(map, a.word(1), "AICo", AICO_RECORD) {
                        sections.push(RivalSection { trigger: t.index, command: c });
                    }
                }
            }
        }
        let rival_line = map
            .triggers
            .iter()
            .position(|t| t.flags & TRIGGER_BY_KIND2 != 0 && t.flags & TRIGGER_BY_PLAYER == 0 && has(&t.actions, act::PLAYER_LOSE))?;
        let disables = |t: usize| -> Vec<usize> {
            map.triggers[t].actions.iter().filter(|a| a.id == act::DISABLE_TRIGGER).filter_map(|a| map.trigger_ref(a.word(1))).collect()
        };
        let player_line = map
            .triggers
            .iter()
            .position(|t| t.flags & TRIGGER_BY_PLAYER != 0 && disables(t.index).contains(&rival_line))?;
        let win_zone = disables(rival_line).into_iter().find(|&t| t != player_line)?;
        let zone_set = map.triggers[win_zone].actions.iter().find(|a| a.id == act::SET_TIMER)?;
        let win_timer = zone_set.word(1) as usize;
        let win_delay = zone_set.float(2);
        let win_rule = map
            .rules
            .iter()
            .find(|r| r.word(0) == RULE_TIMER && r.word(0x10) as usize == win_timer && has(&r.actions, act::PLAYER_WIN))?;
        let win_below = f32::from_bits(win_rule.word(0x18));
        let pl = &map.triggers[player_line].actions;
        let finish_text = pl.iter().find(|a| a.id == act::DISPLAY_TEXT).map(|a| a.word(1));
        let finish_score = pl.iter().find(|a| a.id == act::ADD_SCORE).map_or(0, |a| a.word(1) as i32);
        let rival_death_wins = map.rules.iter().any(|r| {
            r.word(0) == RULE_DEATH
                && r.word(0x14) == rival_selector
                && r.actions.iter().any(|a| a.id == act::SET_TIMER && a.word(1) as usize == win_timer)
        });
        // Traffic: the initially enabled cap trigger, and the difficulty rules that swap one in.
        let traffic_of = |t: usize| map.triggers[t].actions.iter().find(|a| a.id == act::CHANGE_MAX_TRAFFIC).map(|a| a.word(1) as i32);
        let base = map
            .triggers
            .iter()
            .find(|t| t.flags & TRIGGER_ENABLED != 0 && has(&t.actions, act::CHANGE_MAX_TRAFFIC))
            .and_then(|t| traffic_of(t.index));
        let mut max_traffic = [base.unwrap_or(-1); 3];
        for r in map.rules.iter().filter(|r| r.word(0) == RULE_DIFFICULTY) {
            let mask = r.word(0x10);
            let Some(on) = r.actions.iter().filter(|a| a.id == act::ENABLE_TRIGGER).find_map(|a| map.trigger_ref(a.word(1))) else {
                continue;
            };
            let Some(v) = traffic_of(on) else { continue };
            for (d, slot) in max_traffic.iter_mut().enumerate() {
                if mask >> d & 1 != 0 {
                    *slot = v;
                }
            }
        }
        let mf = |o: usize| map.mset.as_ref().and_then(|m| m.get(o..o + 4)).map(|b| f32::from_le_bytes(b.try_into().unwrap()));
        let three = |o: usize| [mf(o).unwrap_or(1.0), mf(o + 4).unwrap_or(1.0), mf(o + 8).unwrap_or(1.0)];
        Some(RaceDef {
            start_trigger,
            countdown_timer,
            go_rule: go.index,
            countdown_texts,
            clock_timer,
            clock_start: set.float(2),
            clock_rate,
            rival_element,
            rival_selector,
            rival_start_command,
            rival_go_command,
            sections,
            player_line,
            rival_line,
            win_zone,
            win_timer,
            win_delay,
            win_below,
            finish_text,
            finish_score,
            rival_death_wins,
            max_traffic,
            speed_factor: three(0x2b8),
            health_factor: three(0x2c4),
            rival_health: el.health,
        })
    }

    /// The rival's starting health at `difficulty` (0..=2): `AIEl` +0x4c × the MSet health factor
    /// [M 0x12b8b0].
    pub fn rival_health_at(&self, difficulty: usize) -> f32 { self.rival_health * self.health_factor[difficulty.min(2)] }
}

// ------------------------------------------------------------------------------------------
// The rival driver
// ------------------------------------------------------------------------------------------

/// What [`Rival::step`] asks the caller to do with the rival's car this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RivalDrive {
    /// Drive the car with these pedals (`FUN_00133f60`, or the gated stop).
    Pedals(DriveCommand),
    /// Pattern 5: put the car at the path's position (the sim copies the path object's
    /// transform and velocity, `FUN_00130508`) [M].
    Path { pos: [f32; 3], vel: [f32; 3] },
    /// Nothing driven.
    Idle,
}

/// One tick's result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RivalStep {
    pub drive: RivalDrive,
    /// AI +0x8c this tick.
    pub target_speed: f32,
    /// The carrot (nav +0x20) after the advance.
    pub carrot: Option<[f32; 3]>,
}

/// The rival's AI: the command it runs, its kind-4 road navigator or path, and the chase state
/// the speed and drive functions keep (AIVehicle in state 5, the chase brain) [M 0x12cb08].
#[derive(Clone, Debug)]
pub struct Rival {
    /// AI +0x88: the command last applied (an index into `MapAi::commands`).
    pub command: Option<usize>,
    pub cmd: Option<AiCommand>,
    pub nav: Option<RoadNav>,
    pub path: Option<(PathPlayer, u16)>,
    pub reverse: ReverseState,
    pub ram: RamState,
    pub prev_aim: [f32; 3],
    pub prev_dist: f32,
    /// A launch the last command asked for (AICo +0x70: velocity = forward × it), taken by the
    /// caller [M 0x145ff8:276-306].
    pub launch: Option<f32>,
    /// The health (`AIEl` +0x4c × the difficulty factor); the caller lowers it on damage.
    pub health: f32,
    /// The lane table set (Snow2a on `snow2a_race` [M 0x220c28]).
    pub lane_set: LaneSet,
}

impl Rival {
    pub fn new(health: f32, fps: i32) -> Rival {
        Rival {
            command: None,
            cmd: None,
            nav: None,
            path: None,
            reverse: ReverseState::default(),
            ram: RamState::new(fps),
            prev_aim: [0.0; 3],
            prev_dist: 0.0,
            launch: None,
            health,
            lane_set: LaneSet::Snow2a,
        }
    }

    /// `FUN_00145ff8` for the rival (the parts a road racer uses) [M]: the same command again
    /// does nothing; else nav +0x04 = AICo +0x7f, +0xb0 = +0x7e when ≥ 0, +0xb1/+0xb2 =
    /// +0x84/+0x85, then `FUN_0012e468` ([`nav_for_pattern`], state 5): pattern 4 → a kind-4
    /// navigator joined at the car and its heading (`FUN_00225c48`, [`RoadNav::join_nearest`]: the
    /// one grid cell under the car, the centre line when the avoidance +0x154 is off, else the
    /// lateral offset clamped to the road width), pattern 5 → the spline the pattern argument names on the path player
    /// (`FUN_0013dfd8` → `FUN_0013da80`: `FUN_001b1258` defaults, wrapping to itself), other
    /// patterns drop the navigator; and the launch (+0x70 other than 0 and 1.5). Returns whether
    /// it applied.
    pub fn apply(&mut self, index: usize, ai: &MapAi, net: &RoadNet, grid: &RoadGrid, map: &MapScript, paths: &MapPaths, car_pos: [f32; 3], car_fwd: [f32; 3]) -> bool {
        if self.command == Some(index) {
            return false;
        }
        let Some(c) = ai.commands.get(index).cloned() else { return false };
        self.command = Some(index);
        self.launch = (c.launch_a != 0.0 && c.launch_a != 1.5).then_some(c.launch_a);
        self.path = None;
        match c.pattern as u32 {
            PATTERN_ROAD_FOLLOW => {
                // FUN_00145ff8 writes +0x04, +0xb0 and +0xb1/+0xb2 before FUN_0012e468 joins, so
                // the join's group filter and its end pick (a kind-4 navigator with neither hint
                // flag keeps the piece's A→B sense) see the new values [M 0x145ff8:259-266].
                let mut n = self.nav.take().unwrap_or_else(|| RoadNav::cleared(NAV_KIND_FOLLOW, self.lane_set));
                n.group = c.nav_byte;
                if c.lane >= 0 {
                    n.slot = c.lane;
                }
                n.hint_toward = c.hint_toward != 0;
                n.hint_away = c.hint_away != 0;
                let joined = nav_for_pattern(net, grid, &mut n, STATE_CHASE, PATTERN_ROAD_FOLLOW, c.no_avoid == 0, car_pos, car_fwd);
                self.nav = matches!(joined, Ok(Some(JoinOutcome::Joined))).then_some(n);
            }
            PATTERN_PATH => {
                self.nav = None;
                let anim = paths.spline_ref(map, c.pattern_arg).and_then(|s| paths.spline_anim(map, s));
                self.path = anim.map(|a| {
                    // [S] `FUN_001b1258` leaves +0x9c on the same animation (the path wraps); on
                    // RACE the jump's path ends 10 units before the trigger that hands the rival
                    // back to the road (227), so the port stops the path at its end and lets the
                    // car coast on ([`RivalDrive::Idle`]); what the game's car does past the end
                    // is not traced.
                    let pp = PathPlayer { anim: Some((a.instance, a.duration)), next: None, playing: true, ..PathPlayer::default() };
                    (pp, a.instance)
                });
            }
            _ => self.nav = None,
        }
        self.cmd = Some(c);
        true
    }

    /// One 60 Hz AI tick for the rival chasing `player` (position, velocity, forward):
    /// the ram roll (`ram_tick`, one driving draw per 5 s), the aim (`aim_point`), the target
    /// speed (`chase_target_speed`, profile 0xf is the rubber band), the carrot advance
    /// (`FUN_00130140`: when the car is nearer the carrot than the lookahead the carrot moves on
    /// by the difference, with the aim direction as the chooser's hint), the dispatch and the
    /// drive (`chase_drive`), or the path copy for pattern 5 [M per function].
    /// Taken as given [S]: the player is always seen (the sight rays are the caller's), no bump
    /// probe hit, no avoidance steering (the lane pick 0x135f70, `join::lane_pick`, is not called here: the
    /// offset the join keeps stays as it is).
    pub fn step(&mut self, net: &RoadNet, paths: &MapPaths, body: &Body, player: ([f32; 3], [f32; 3], [f32; 3]), speed_factor: f32, rng: &mut DriveRng) -> RivalStep {
        let Some(c) = self.cmd.clone() else {
            return RivalStep { drive: RivalDrive::Idle, target_speed: 0.0, carrot: None };
        };
        let (ppos, pvel, pfwd) = player;
        let fps = TICKS_PER_SECOND;
        ram_tick(&mut self.ram, false, false, true, true, c.ram_chance, fps, rng);
        let pattern = c.pattern as u32;
        let profile = c.profile as u32;
        let carrot = self.nav.as_ref().map(|n| n.pos).unwrap_or(body.pos);
        let aim = aim_point(&AimInputs {
            profile,
            pattern,
            target_mode: c.target_mode as u32,
            goal: c.goal,
            has_target: true,
            target_pos: ppos,
            target_vel: pvel,
            lost: false,
            self_body: *body,
            lead: c.lead,
            follow: c.follow,
            prev_dist: self.prev_dist,
            prev_aim: self.prev_aim,
            nav_carrot: carrot,
            nav_10: carrot,
        });
        self.prev_aim = aim.point;
        self.prev_dist = aim.dist;
        let seg_limit = self.nav.as_ref().and_then(|n| net.seg(n.seg).ok()).map_or(0.0, |s| s.speed_limit());
        let target = chase_target_speed(&ChaseSpeedInputs {
            state: 5,
            profile,
            pattern,
            alive: self.health,
            stop_query: false,
            stop_byte: false,
            self_pos: body.pos,
            self_vel: body.vel,
            self_forward: body.fwd,
            self_box: body.box_z,
            has_target: true,
            target_is_player: true,
            target_vel: pvel,
            target_box: body.box_z,
            aim: aim.point,
            dist: aim.dist,
            dir: aim.dir,
            dir_210: aim.nav_dir,
            follow: c.follow,
            catchup: c.catchup,
            cap: c.speed_cap,
            speed_mul: 1.0,
            ramming: self.ram.ram > 0,
            lost: false,
            element_target: false,
            follow_target: c.follow_target != 0,
            spawn_delay: self.ram.spawn_delay,
            seg_limit,
            speed_factor,
        })
        .speed;
        if pattern == PATTERN_PATH {
            let Some((pp, inst)) = self.path.as_mut() else {
                return RivalStep { drive: RivalDrive::Idle, target_speed: target, carrot: None };
            };
            if !pp.playing {
                return RivalStep { drive: RivalDrive::Idle, target_speed: target, carrot: None };
            }
            let anim = paths.anims.get(inst);
            let before = anim.and_then(|a| a.position(pp.time));
            pp.update(1.0);
            if !pp.playing {
                // The end of the path this tick: the car keeps the velocity it had [S].
                return RivalStep { drive: RivalDrive::Idle, target_speed: target, carrot: None };
            }
            let now = anim.and_then(|a| a.position(pp.time));
            return match now {
                Some(p) => {
                    let b = before.unwrap_or(p);
                    let k = fps as f32;
                    RivalStep { drive: RivalDrive::Path { pos: p, vel: [(p[0] - b[0]) * k, (p[1] - b[1]) * k, (p[2] - b[2]) * k] }, target_speed: target, carrot: None }
                }
                None => RivalStep { drive: RivalDrive::Idle, target_speed: target, carrot: None },
            };
        }
        let la = lookahead(body.speed(), body.box_z);
        if let Some(n) = self.nav.as_mut() {
            let d = ((body.pos[0] - n.pos[0]).powi(2) + (body.pos[1] - n.pos[1]).powi(2) + (body.pos[2] - n.pos[2]).powi(2)).sqrt();
            if d < la {
                // FUN_00130140's hint: n(target − car) with +0xb1, n(car − target) with +0xb2,
                // else AI +0x1e0 [M 0x130140]; the port stands in the aim direction for +0x1e0
                // [S]. The kind-4 chooser reads it only with +0xb1/+0xb2 set (none on RACE [D]).
                let dir = if n.hint_toward {
                    normalize(sub(ppos, body.pos))
                } else if n.hint_away {
                    normalize(sub(body.pos, ppos))
                } else {
                    aim.dir
                };
                let _ = n.advance_with(net, (la - d).abs(), NavHint { dir, target: -1 }, &mut || rng.next());
            }
        }
        let carrot = self.nav.as_ref().map(|n| n.pos);
        let arm = dispatch(&DispatchInputs {
            state: 5,
            pattern,
            alive: self.health,
            stop_query: false,
            stop_byte: false,
            target_speed: target,
            avoid_on: c.no_avoid == 0,
            nav_repeat: self.nav.as_ref().is_some_and(|n| n.repeated),
            self_pos: body.pos,
            dir: aim.dir,
            lookahead: la,
            carrot: carrot.unwrap_or(body.pos),
        });
        let drive = match arm {
            Arm::Chase { speed, aim: at, .. } => {
                let i = ChaseDriveInputs {
                    profile,
                    pattern,
                    spawn_delay: self.ram.spawn_delay,
                    hold_160: false,
                    hold_15c: false,
                    nav_repeat: self.nav.as_ref().is_some_and(|n| n.repeated),
                    has_target: true,
                    target_is_player: true,
                    target_pos: ppos,
                    target_vel: pvel,
                    target_fwd: pfwd,
                    dir: aim.dir,
                    held_steer: 0.0,
                    fps,
                };
                RivalDrive::Pedals(chase_drive(body, at, speed, &mut self.reverse, &i, &mut |_| false).drive())
            }
            Arm::Gated => RivalDrive::Pedals(DriveCommand::IDLE),
            _ => RivalDrive::Idle,
        };
        RivalStep { drive, target_speed: target, carrot }
    }
}

/// Whether the AI command handler applies a trigger's `EAICommand` to the rival: the gate
/// element (word 2) must be the rival and the car that ran the trigger the rival's car; it then
/// clears the trigger's enable bit [M 0x145ff8:88-140].
pub fn section_applies(def: &RaceDef, trigger: usize, car_slot: i32, rival_slot: i32) -> Option<usize> {
    let s = def.sections.iter().find(|s| s.trigger == trigger)?;
    (car_slot == rival_slot).then_some(s.command)
}

// ------------------------------------------------------------------------------------------
// The race road (the port's progress measure)
// ------------------------------------------------------------------------------------------

/// The race road as one polyline: the kind-4 chains of the rival's car group, from the piece
/// nearest the start (`nearest_piece`: the start's grid cell), each chain followed A→B to its end and the next chain taken as the unused
/// one whose first node is nearest the last chain's end (the jumps between chains are bridged by
/// a straight step) [S: a port measure built from the road data; the game keeps none].
#[derive(Clone, Debug, PartialEq)]
pub struct RaceRoute {
    pub points: Vec<[f32; 3]>,
    /// Arc length at each point.
    pub dist: Vec<f32>,
    /// How many chains it joins (3 on RACE).
    pub chains: usize,
}

impl RaceRoute {
    pub fn build(net: &RoadNet, grid: &RoadGrid, group: u8, start: [f32; 3]) -> Option<RaceRoute> {
        let race: Vec<usize> = (0..net.segs.len())
            .filter(|&i| net.seg(i).is_ok_and(|s| s.flags() & 8 != 0 && s.flags() & 0x100 == 0 && s.car_group() as u8 == group))
            .collect();
        if race.is_empty() {
            return None;
        }
        let by_a = |node: u16, used: &[bool]| race.iter().position(|&i| !used[i] && net.seg(i).is_ok_and(|s| s.node_a() == node));
        let mut used = vec![false; net.segs.len()];
        let npos = |n: u16| net.node(n as usize).map(|x| x.pos()).unwrap_or([0.0; 3]);
        let d2 = |a: [f32; 3], b: [f32; 3]| (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
        let Ok(Nearest::Piece { seg: first, .. }) = nearest_piece(net, grid, NAV_KIND_FOLLOW, group, start) else { return None };
        let mut points: Vec<[f32; 3]> = Vec::new();
        let mut chains = 0;
        let mut cur = Some(first);
        while let Some(mut seg) = cur {
            chains += 1;
            // Walk back to the chain's head only for the first chain's start piece: the race
            // starts on the piece nearest the start, so the walk goes forward from it.
            points.push(npos(net.seg(seg).ok()?.node_a()));
            loop {
                used[seg] = true;
                let s = net.seg(seg).ok()?;
                points.push(npos(s.node_b()));
                match by_a(s.node_b(), &used) {
                    Some(k) => seg = race[k],
                    None => break,
                }
            }
            let tail = *points.last()?;
            cur = race
                .iter()
                .copied()
                .filter(|&i| !used[i])
                .filter(|&i| race.iter().all(|&j| !net.seg(j).is_ok_and(|s| s.node_b() == net.seg(i).map(|x| x.node_a()).unwrap_or(u16::MAX))))
                .min_by(|&a, &b| {
                    let pa = npos(net.seg(a).map(|s| s.node_a()).unwrap_or(0));
                    let pb = npos(net.seg(b).map(|s| s.node_a()).unwrap_or(0));
                    d2(pa, tail).total_cmp(&d2(pb, tail))
                });
        }
        let mut dist = Vec::with_capacity(points.len());
        let mut acc = 0.0f32;
        for (k, p) in points.iter().enumerate() {
            if k > 0 {
                acc += d2(*p, points[k - 1]).sqrt();
            }
            dist.push(acc);
        }
        Some(RaceRoute { points, dist, chains })
    }

    pub fn length(&self) -> f32 { self.dist.last().copied().unwrap_or(0.0) }

    /// The arc length of the point on the route nearest `pos`, searched within `window` of `near`
    /// (the last progress) so the overlapping parts of the course do not swap.
    pub fn progress(&self, pos: [f32; 3], near: f32, window: (f32, f32)) -> f32 {
        let mut best = (f32::MAX, near);
        for k in 1..self.points.len() {
            if self.dist[k] < near - window.0 || self.dist[k - 1] > near + window.1 {
                continue;
            }
            let (a, b) = (self.points[k - 1], self.points[k]);
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let l2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
            let t = if l2 > 0.0 { (((pos[0] - a[0]) * ab[0] + (pos[1] - a[1]) * ab[1] + (pos[2] - a[2]) * ab[2]) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let p = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
            let d = (pos[0] - p[0]).powi(2) + (pos[1] - p[1]).powi(2) + (pos[2] - p[2]).powi(2);
            if d < best.0 {
                best = (d, self.dist[k - 1] + l2.sqrt() * t);
            }
        }
        best.1
    }
}

// ------------------------------------------------------------------------------------------
// The race state and its HUD numbers
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Racer {
    Player,
    Rival,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RacePhase {
    /// Before the go rule: the car stop holds, the countdown texts show.
    Countdown,
    /// The race clock runs.
    Racing,
    /// Someone crossed the line; the script has not ended the drive yet.
    Finishing,
    /// The script ended the drive (`EPlayerWin` / `EPlayerLose`).
    Won,
    Lost,
}

/// The race as it stands. [`RaceState::observe`] reads the script after each step.
#[derive(Clone, Debug, PartialEq)]
pub struct RaceState {
    pub phase: RacePhase,
    /// Who crossed the finish line first (the trigger that ran: player line or rival line).
    pub first_over: Option<Racer>,
    /// The race clock (timer ticks) at the go and at the player's crossing.
    pub go_tick: Option<i32>,
    pub finish_clock: Option<i32>,
    /// Progress along [`RaceRoute`] for the player and the rival [S, the port's measure].
    pub progress: [f32; 2],
    /// The countdown text up now and the tick it went up (each shows 0.5 s, the last 2 s [D]).
    pub countdown_text: Option<(u32, i32, f32)>,
    /// The next countdown text to show (an index into [`RaceDef::countdown_texts`]).
    pub countdown_next: usize,
}

impl Default for RaceState {
    fn default() -> Self { RaceState { phase: RacePhase::Countdown, first_over: None, go_tick: None, finish_clock: None, progress: [0.0; 2], countdown_text: None, countdown_next: 0 } }
}

/// The numbers a race HUD shows this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct RaceHud {
    /// The player's place (1 or 2) of `racers` [S: the port's ranking; the game draws none].
    pub position: u8,
    pub racers: u8,
    /// Lap 1 of 1: the race is point to point [D].
    pub lap: u8,
    pub laps: u8,
    /// The race clock in timer ticks (60 a second), and its text as `FUN_0023e580` draws it:
    /// `"%2d:%02d"` of the whole seconds, only while they are above 0 [M 0x23e580; D the format
    /// at 0x3aab48].
    pub clock_ticks: i32,
    pub clock_text: Option<String>,
    /// The finish time once the player has crossed (the one lap's time) [S].
    pub best_lap_ticks: Option<i32>,
    /// The countdown text id up now.
    pub countdown_text: Option<u32>,
}

/// `FUN_0023e580`'s clock face: whole seconds = ticks / 60; nothing at 0 or below; minutes and
/// seconds through `"%2d:%02d"` [M].
pub fn clock_text(ticks: i32) -> Option<String> {
    let s = ticks / TICKS_PER_SECOND;
    (s > 0).then(|| format!("{:2}:{:02}", s / 60, s % 60))
}

impl RaceState {
    /// Reads one step of the script: the trigger log (`DriveScript::take_fired`, with the walk
    /// slot of the rival's car), the go (the clock timer running), the countdown texts, the
    /// finish line and the end.
    pub fn observe(&mut self, def: &RaceDef, script: &DriveScript, fired: &[(usize, i32)], rival_slot: i32) {
        if self.first_over.is_none() {
            let player = fired.iter().any(|&(t, car)| t == def.player_line && car == 0);
            let rival = fired.iter().any(|&(t, car)| t == def.rival_line && car == rival_slot);
            // Both lines in one step: the walk runs both lists before either's `EDisableTrigger`
            // is handled, and the rival's `EPlayerLose` loses the drive, so the rival has it
            // [M 0x22d240 queues, the handlers run in the drain; the game's order among hits in
            // one frame is not traced].
            if rival {
                self.first_over = Some(Racer::Rival);
            } else if player {
                self.first_over = Some(Racer::Player);
                self.finish_clock = Some(script.timers[def.clock_timer].peek(script.now));
            }
        }
        let cd = script.timers[def.countdown_timer];
        if cd.running && self.go_tick.is_none() {
            let secs = cd.peek(script.now) as f32 / TICKS_PER_SECOND as f32;
            while let Some(&(thr, id)) = def.countdown_texts.get(self.countdown_next) {
                if !(secs < thr) {
                    break;
                }
                // The text's own seconds from its EDisplayText (0.5 s, the last 2 s) [D].
                let dur = if self.countdown_next + 1 == def.countdown_texts.len() { 2.0 } else { 0.5 };
                self.countdown_text = Some((id, script.now, dur));
                self.countdown_next += 1;
            }
        }
        if let Some((_, at, dur)) = self.countdown_text {
            if (script.now - at) as f32 >= dur * TICKS_PER_SECOND as f32 {
                self.countdown_text = None;
            }
        }
        if self.go_tick.is_none() && script.timers[def.clock_timer].running {
            self.go_tick = Some(script.now);
        }
        self.phase = match script.state {
            DriveState::Won => RacePhase::Won,
            DriveState::Lost => RacePhase::Lost,
            _ if self.first_over.is_some() => RacePhase::Finishing,
            _ if self.go_tick.is_some() => RacePhase::Racing,
            _ => RacePhase::Countdown,
        };
    }

    /// The player's place: fixed by the line once someone has crossed, else by progress [S].
    pub fn position(&self) -> u8 {
        match self.first_over {
            Some(Racer::Player) => 1,
            Some(Racer::Rival) => 2,
            None => {
                if self.progress[1] > self.progress[0] {
                    2
                } else {
                    1
                }
            }
        }
    }

    pub fn hud(&self, def: &RaceDef, script: &DriveScript) -> RaceHud {
        let clock = script.timers[def.clock_timer].peek(script.now);
        let running = self.go_tick.is_some();
        RaceHud {
            position: self.position(),
            racers: 2,
            lap: 1,
            laps: 1,
            clock_ticks: if running { clock } else { 0 },
            clock_text: if running { clock_text(clock) } else { None },
            best_lap_ticks: self.finish_clock,
            countdown_text: self.countdown_text.map(|c| c.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_face_is_whole_seconds_as_the_hud_prints_them() {
        // FUN_0023e580: ticks / 60, nothing at 0, "%2d:%02d" [M; D 0x3aab48].
        assert_eq!(clock_text(0), None);
        assert_eq!(clock_text(59), None);
        assert_eq!(clock_text(61), Some(" 0:01".into()));
        assert_eq!(clock_text(60 * 75 + 59), Some(" 1:15".into()));
        assert_eq!(clock_text(60 * 600), Some("10:00".into()));
        assert_eq!(clock_text(-5), None);
    }

    #[test]
    fn the_line_fixes_the_place_over_progress() {
        let mut s = RaceState::default();
        s.progress = [100.0, 200.0];
        assert_eq!(s.position(), 2, "the rival further along");
        s.progress = [300.0, 200.0];
        assert_eq!(s.position(), 1);
        s.first_over = Some(Racer::Rival);
        assert_eq!(s.position(), 2, "the rival crossed first: progress no longer counts");
        s.first_over = Some(Racer::Player);
        s.progress = [0.0, 500.0];
        assert_eq!(s.position(), 1);
    }
}
