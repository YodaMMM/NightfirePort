//! stuck-guard-combat-sweep, part 2: 07000005's helipad courtyard, walked with the REAL input
//! path (no teleports after the spawn) — the follow-up the patrol/alarm sweep called for (the
//! helipad guards spawn trigger-gated, so the alarm-only pass measured 0 there).
//!
//! Method: load through the real path (`nf_view::load`), walk Bond from the cold start #26
//! (104.95, 25.12, 108.83) [D docs/levels/07000005.md §1] up the road (the walked spine the
//! first run measured) into the helipad side — the parked helicopter's statics sit at
//! (−17.0, 43.0..44.8, −37.2..−43.6) [D fix-portal5 §6], the pad courtyard guards at
//! (−31.3, 37.5, −37.05) / (−32.77, 37.5, −22.32) [L the load digest, placements 7/10] —
//! engage the guards: the walk trips the sight path; the trigger-gated pad guards are ENABLEd
//! through the game's own message (0x0e — WaitSwitch → the start state [M the msg 0x0e
//! handler]) and any still-idle watcher woken through the alarm path (msg 0x1e [M
//! `DroneVision_EnemyAlerts` case 0x1e 0x177c18:113-114]); then 60 s of fixed 60 Hz ticks log
//! every guard that sits in a MOVING combat state — a DSTATE mover (the extended combat-mover
//! set) or a combat-band state playing a `DroneAnim_SetCombatMoveAnim` (0x13cfb0) move anim —
//! whose position moves < 0.1 over 3 s (180 ticks).
//!
//! Per hit the drone's route/cover state is dumped (croute path/index/remaining/target, cover
//! node + point, goal, engage_range) so the report can pin the wedge mechanism. LOG-ONLY: the
//! fix turns the exact rows into asserted sisters.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::{anim_state, msg, state};
use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn state_name(s: u8) -> &'static str {
    match s {
        state::IDLE => "IDLE 0x04",
        state::ALERT => "ALERT 0x05",
        state::ATTACK => "ATTACK 0x56 (chase)",
        state::COMBAT => "COMBAT 0x58",
        state::COMBAT_NO_MOVE => "COMBAT_NO_MOVE 0x59",
        state::COMBAT_OUT_OF_RANGE => "COMBAT_OUT_OF_RANGE 0x5a (approach)",
        state::COMBAT_NEW_SIGHTING => "COMBAT_NEW_SIGHTING 0x5b",
        state::COMBAT_NO_SIGHT => "COMBAT_NO_SIGHT 0x5c",
        state::COMBAT_WAIT => "COMBAT_WAIT 0x5e (bunching wait)",
        state::COMBAT_NO_ROUTE => "COMBAT_NO_ROUTE 0x5f",
        state::AIM_STAND => "AIM_STAND 0x67",
        state::AIM_STAND_FIRE => "AIM_STAND_FIRE 0x68",
        state::AIM_STAND_RELOAD => "AIM_STAND_RELOAD 0x69",
        state::AIM_BACKOFF => "AIM_BACKOFF 0x6d (back off)",
        state::CROUCH_COVER => "CROUCH_COVER 0x6e",
        state::AIM_CROUCH => "AIM_CROUCH 0x6f",
        state::AIM_CROUCH_FIRE => "AIM_CROUCH_FIRE 0x70",
        state::AIM_CROUCH_RELOAD => "AIM_CROUCH_RELOAD 0x71",
        state::STEP_AIM_LEFT => "STEP_AIM_LEFT 0x73",
        state::STEP_AIM_RIGHT => "STEP_AIM_RIGHT 0x74",
        state::STRAFE_AIM_LEFT => "STRAFE_AIM_LEFT 0x75",
        state::STRAFE_AIM_RIGHT => "STRAFE_AIM_RIGHT 0x76",
        state::STRAFE_DODGE_LEFT => "STRAFE_DODGE_LEFT 0x77",
        state::STRAFE_DODGE_RIGHT => "STRAFE_DODGE_RIGHT 0x78",
        state::ROLL_LEFT_CROUCH => "ROLL_LEFT_CROUCH 0x79",
        state::ROLL_RIGHT_CROUCH => "ROLL_RIGHT_CROUCH 0x7a",
        state::RUN_AWAY_FROM_OBJECT => "RUN_AWAY_FROM_OBJECT 0x86 (cover seek)",
        state::HIDE_FROM_SCARY_OBJECT => "HIDE_FROM_SCARY_OBJECT 0x87 (cover seek)",
        state::UNDER_COVER_RETURN => "UNDER_COVER_RETURN 0x99 (walk to cover)",
        state::RUN_FOR_COVER => "RUN_FOR_COVER 0x8c",
        state::ALERT_TO_POSITION => "ALERT_TO_POSITION 0x62",
        state::SEARCH_AREA => "SEARCH_AREA 0x65",
        state::GO_TO_GOAL => "GO_TO_GOAL 0x63",
        state::PATROL => "PATROL 0x07 (route walk)",
        state::SNIPER_IDLE => "SNIPER_IDLE 0x29",
        state::SNIPER_AIM => "SNIPER_AIM 0x2a",
        state::SNIPER_FIRE => "SNIPER_FIRE 0x2b",
        _ => "?",
    }
}

/// The moving set: the patrol/alarm travellers PLUS the combat movers
/// (stuck-guard-combat-sweep; the cites live on the shared sweep's copy of this list).
fn moving(s: u8) -> bool {
    matches!(s, state::ATTACK | state::RUN_FOR_COVER | state::ALERT_TO_POSITION | state::SEARCH_AREA
        | state::HEARD_NOISE_AWARE | state::HEARD_NOISE_SUSPECT | state::HEARD_NOISE_ALERT
        | state::GO_TO_GOAL | state::GO_TO_GOAL_ALLY | state::INTEROGATE_ASSIST
        | state::PATROL | state::CIVILIAN_MISSION
        | state::COMBAT_OUT_OF_RANGE | state::COMBAT_WAIT | state::AIM_BACKOFF
        | state::STEP_AIM_LEFT | state::STEP_AIM_RIGHT
        | state::STRAFE_AIM_LEFT | state::STRAFE_AIM_RIGHT
        | state::STRAFE_DODGE_LEFT | state::STRAFE_DODGE_RIGHT
        | state::ROLL_LEFT_CROUCH | state::ROLL_RIGHT_CROUCH
        | state::RUN_AWAY_FROM_OBJECT | state::HIDE_FROM_SCARY_OBJECT
        | state::UNDER_COVER_RETURN)
}

/// The combat states whose handlers pick combat moves / move anims [M 0x13cfb0 runs inside
/// them; the port's combat band, drone.rs:3557-3559].
fn combat_band(s: u8) -> bool {
    matches!(s, state::ATTACK | state::COMBAT | state::COMBAT_NO_MOVE | state::COMBAT_OUT_OF_RANGE
        | state::COMBAT_NEW_SIGHTING | state::COMBAT_NO_SIGHT | state::COMBAT_WAIT | state::COMBAT_NO_ROUTE
        | state::AIM_STAND | state::AIM_STAND_FIRE | state::AIM_STAND_RELOAD | state::AIM_BACKOFF
        | state::CROUCH_COVER | state::AIM_CROUCH | state::AIM_CROUCH_FIRE | state::AIM_CROUCH_RELOAD
        | state::STEP_AIM_LEFT | state::STEP_AIM_RIGHT | state::STRAFE_AIM_LEFT | state::STRAFE_AIM_RIGHT)
}

/// The combat move anims `DroneAnim_SetCombatMoveAnim` (0x13cfb0) plays when the mover walks
/// [M 0x13cfb0:75-146/:193-244/:246-274/:276-392; the RUN/WALK fallback of the approach].
fn combat_move_anim(a: u16) -> bool {
    matches!(a, anim_state::BACKOFF | anim_state::RUN_AIM | anim_state::WALK_AIM
        | anim_state::STRAFE_LEFT | anim_state::STRAFE_RIGHT
        | anim_state::STRAFE_LEFT_ALT | anim_state::STRAFE_RIGHT_ALT
        | anim_state::ROLL_LEFT | anim_state::ROLL_RIGHT
        | anim_state::ROLL_LEFT_ALT | anim_state::ROLL_RIGHT_ALT
        | anim_state::STEP_LEFT | anim_state::STEP_RIGHT
        | anim_state::RUN | anim_state::WALK)
}

/// A drone whose mode wants travel: a DSTATE mover, or a combat-band state playing a move anim.
fn wants_travel(d: &nf_game::drone::Drone) -> bool {
    moving(d.state) || (combat_band(d.state) && combat_move_anim(d.anim.state))
}

/// 07000005's helipad courtyard box [D fix-portal5 §6 — the heli statics' spheres].
fn in_helipad_box(p: [f32; 3]) -> bool {
    (-30.0..2.0).contains(&p[0]) && p[1] > 30.0 && (-58.0..-22.0).contains(&p[2])
}

fn digest(w: &Walker, what: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[05] {what}: bond ({:.2},{:.2},{:.2}) t{}", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], w.total_ticks);
    for (i, d) in sim.drones.list.iter().enumerate() {
        if !d.present { continue; }
        eprintln!("  drone#{i} skin {:#010x} en {} watch {} corpse {} state {} mode {:#04x} anim {:#04x} pos ({:.2},{:.2},{:.2}) hp {:.0}{}",
            d.skin, d.enabled as u8, d.watching as u8, d.corpse as u8,
            state_name(d.state), d.mode, d.anim.state, d.pos[0], d.pos[1], d.pos[2], d.health,
            if in_helipad_box(d.pos) { " HELIPAD" } else { "" });
    }
}

/// The alarm pass on every idle watcher [M 0x177c18:113-114] — the sweep's own opener.
fn alarm_idle(w: &mut Walker) -> usize {
    let sim = w.scene.sim.as_mut().unwrap();
    let difficulty = sim.drones.difficulty;
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
        ai_points: &sim.ai_points,
    };
    let frame = sim.drones.frame;
    let mut woken = 0;
    for i in 0..sim.drones.list.len() {
        let d = &sim.drones.list[i];
        if d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
            && matches!(d.state, state::IDLE | state::ALERT) {
            sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
            woken += 1;
        }
    }
    woken
}

/// The game's own enable message on the trigger-gated pad guards (0x0e: WaitSwitch → the
/// drone's start state [M the msg 0x0e handler]) — the stand-in for their spawn plate.
fn enable_wait_switch(w: &mut Walker) -> usize {
    let sim = w.scene.sim.as_mut().unwrap();
    let difficulty = sim.drones.difficulty;
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
        ai_points: &sim.ai_points,
    };
    let frame = sim.drones.frame;
    let mut woken = 0;
    for i in 0..sim.drones.list.len() {
        let d = &sim.drones.list[i];
        if d.present && !d.enabled && !d.corpse && d.health > 0.0 && d.team != 2 {
            sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ENABLE);
            woken += 1;
        }
    }
    woken
}

/// The per-hit mechanism dump: the route/cover state of one drone [M the fields are the port's
/// mirrors of Drone+0x854 (croute), +0xBB4 (cover), +0x710 (cover_point), +0x6F0 (goal)].
fn dump_route(d: &nf_game::drone::Drone, i: usize) {
    let r = d.croute.as_ref();
    eprintln!("  MECH drone#{i} state {} anim {:#04x} pos ({:.2},{:.2},{:.2}) goal {:?} route_blocked {} engage {:.1} dist {:.1} seen {} unseen {}",
        state_name(d.state), d.anim.state, d.pos[0], d.pos[1], d.pos[2],
        d.goal.map(|g| (g[0], g[1], g[2])), d.route_blocked, d.engage_range, d.dist, d.seen as u8, d.unseen_ticks);
    eprintln!("      croute {}", r.map_or("none".into(), |r| format!(
        "path {} idx {} remaining {:.2} nodes {} target ({:.2},{:.2},{:.2}) dest ({:.2},{:.2},{:.2})",
        r.path, r.index, r.remaining, r.nodes.len(), r.target[0], r.target[1], r.target[2], r.dest[0], r.dest[1], r.dest[2])));
    eprintln!("      cover {:?} cover_point {:?} cover_type {} cover_side {}", d.cover, d.cover_point, d.cover_type, d.cover_side);
}

/// The shared engagement flow: the cold start → the road walk (the waypoints the first run's
/// nav legs actually arrived at) → the pad-side legs → the ENABLE + alarm engagement. Both the
/// log-only measurement and the sister assertion run exactly this (the step loop only arises
/// with the walked context — the alarm-only flow never reproduced it).
fn walk_and_engage(verbose: bool) -> Walker {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 104.95).abs() < 2.0 && (p[2] - 108.83).abs() < 2.0, "cold start #26 [D], got {p:?}");
    }
    w.wait_enabled(6000, "level start");
    w.wait(120);
    if verbose { digest(&w, "after the level's first second"); }

    // The walk up the road: the waypoints ride the road's own lane (the single long leg cut the
    // corner and fell off the road edge at (48, 34, 55.66) — floor_below 28.93 under a y-34 Bond).
    for (name, at) in [
        ("the courtyard west", [62.0f32, 26.0, 96.0]),
        ("the road bend", [48.53, 28.20, 96.17]),
        ("the road climb", [37.61, 30.04, 93.92]),
        ("the wall gate", [32.17, 31.78, 88.42]),
        ("the road shelf", [30.50, 32.62, 85.08]),
        ("the road head", [30.12, 36.21, 66.31]),
        ("the truck park", [29.85, 37.74, 50.30]),
    ] {
        w.walk_nav_to(at, name, 2.5);
    }
    if verbose { digest(&w, "after the road walk"); }

    // The pad side: west past the gate along the wall. Lenient legs — the exact lane is
    // measured as we go (the digests carry the positions).
    for (name, at) in [
        ("west along the wall", [20.0f32, 37.7, 46.0]),
        ("the pad approach", [5.0, 38.5, 30.0]),
        ("the pad courtyard", [-25.0, 38.0, -10.0]),
        ("the helipad", [-17.0, 44.5, -40.0]),
    ] {
        w.walk_nav_to(at, name, 3.0);
        if verbose { digest(&w, &format!("after {name}")); }
    }
    let arrived = in_helipad_box(w.scene.sim.as_ref().unwrap().bond.pos);
    eprintln!("[05] helipad courtyard reached: {arrived}");

    // Engage: the trigger-gated pad guards through the game's own ENABLE msg, 10 s of sight
    // reaction, then the alarm path on whoever is still idle, then 10 s more.
    let enabled = enable_wait_switch(&mut w);
    eprintln!("[05] ENABLE pass woke {enabled} trigger-gated guards [M the msg 0x0e handler]");
    w.wait(600);
    let woken = alarm_idle(&mut w);
    eprintln!("[05] alarm pass woke {woken} idle watchers [M 0x177c18:113-114]");
    w.wait(600);
    if verbose { digest(&w, "after the engage"); }
    w
}

/// The 07000005 helipad engagement walk + the frozen-combat-mover log. LOG-ONLY.
#[test]
fn the_helipad_walk_engages_the_guards_and_logs_frozen_combat_movers() {
    let mut w = walk_and_engage(true);

    // The frozen-mover log: 60 s of ticks, the anchor logic of the shared sweep; every new HIT
    // row also dumps the drone's route/cover mechanism.
    const WINDOW: u32 = 180;
    let n = w.scene.sim.as_ref().unwrap().drones.list.len();
    let mut anchor: Vec<([f32; 3], u32)> = vec![([0.0; 3], 0); n];
    let mut hits: Vec<String> = Vec::new();
    for t in 0..3600u32 {
        w.tick(Input::default(), false);
        if t == 1800 {
            let again = alarm_idle(&mut w);
            eprintln!("[05] mid-log re-alarm: {again}");
        }
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || !d.enabled || d.corpse || d.health <= 0.0 { continue; }
            if !wants_travel(d) { anchor[i].1 = 0; continue; }
            let moved = ((d.pos[0] - anchor[i].0[0]).powi(2) + (d.pos[1] - anchor[i].0[1]).powi(2) + (d.pos[2] - anchor[i].0[2]).powi(2)).sqrt();
            if moved >= 0.1 { anchor[i] = (d.pos, t); continue; }
            if anchor[i].1 == 0 { anchor[i] = (d.pos, t); continue; }
            if t - anchor[i].1 >= WINDOW {
                let gd = d.goal.map(|g| ((g[0] - d.pos[0]).powi(2) + (g[1] - d.pos[1]).powi(2) + (g[2] - d.pos[2]).powi(2)).sqrt());
                let row = format!(
                    "HIT 05 drone#{i} skin {:#010x} state {} mode {:#04x} anim {:#04x} pos ({:.2},{:.2},{:.2}) goal {:?} goal_dist {:?} route_blocked {} helipad_box {}",
                    d.skin, state_name(d.state), d.mode, d.anim.state, d.pos[0], d.pos[1], d.pos[2],
                    d.goal.map(|g| (g[0], g[1], g[2])), gd.map(|x| (x * 100.0).round() / 100.0),
                    d.route_blocked, in_helipad_box(d.pos));
                if !hits.contains(&row) {
                    hits.push(row.clone());
                    eprintln!("{row}");
                    dump_route(d, i);
                }
                anchor[i] = (d.pos, t);
            }
        }
    }
    digest(&w, "at the end of the log window");
    eprintln!("[05] {} frozen-combat-mover hits over 60 s", hits.len());
}

/// The sister (the fixed class → an assertion). Pre-fix, the WALKED engagement on 07000005
/// parked EIGHT guards in STEP_AIM_LEFT/RIGHT (0x73/0x74, the step anims 0x5a/0x5b) that inched
/// < 0.1 per 3 s for the whole 60 s window (the first run's digests: the pair at (53-56, 27,
/// 96-97), the gate-level pair at (29-32, 34-37, 54-74), the spawn-area runners) — the
/// combat-move CHOICE probe (`can_move`, knee/chest rays) passed where the MOVER (`move_by`,
/// feet/knee rays + the 0.4 overshoot) rejected, so every completed step anim re-picked the
/// same blocked step: 0x73/0x74 ↔ 0x68 forever. The game cannot diverge this way — its choice
/// probe (`NDrone2_CanStepLeft` 0x14f640 family) and its mover share the one `NDrone2_MoveTest`
/// + `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0 machinery [M]; the port now consults
/// `probe_move` (the mover's own test) in the choice's pair gates, the evasive dodge probe and
/// the strafe UPDATE's blocked exit [S consistency — which ray set is the game's is
/// unverified; the agreement is the port's requirement].
///
/// Asserted here on the real walked path: the walk to the gate ledge (the leg that ends standing
/// BELOW the gate guards' plateau — Bond at (40.1, 34.7, 54.5), the guards at y 37 — the exact
/// context the first run froze eight guards in), the alarm pass, 60 s of ticks — no guard may
/// sit in a step/strafe combat mover (0x73-0x78) with < 0.1 of movement over 3 s (180 ticks).
/// COMBAT_WAIT is excluded: the bunching wait is the game's own parked-mover state, not this
/// class. (The alarm-only flow and the road waypoint walk do NOT reproduce the loop — the
/// sister runs the ledge-leg context the measurement froze.)
#[test]
fn no_guard_loops_a_blocked_combat_step() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.wait_enabled(6000, "level start");
    w.wait(120);
    // The first run's exact flow: the road leg (Bond ends on the ledge below the gate guards,
    // (40.10, 34.71, 54.55)), the pad-side attempt (its wedge parks Bond at (34.64, 34.72,
    // 49.58) directly below the gate pair), 10 s of sight engagement, the alarm pass, 10 s more
    // — the step loop the first run measured develops across those ticks.
    w.walk_nav_to([40.0, 37.0, 52.0], "the gate ledge", 2.5);
    w.walk_nav_to([-17.0, 44.5, -40.0], "the pad attempt", 3.0);
    eprintln!("[05-sister] bond at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w.wait(600);
    let woken = alarm_idle(&mut w);
    eprintln!("[05-sister] alarm pass woke {woken} idle watchers [M 0x177c18:113-114]");
    w.wait(600);
    const WINDOW: u32 = 180;
    let n = w.scene.sim.as_ref().unwrap().drones.list.len();
    let mut anchor: Vec<([f32; 3], u32)> = vec![([0.0; 3], 0); n];
    let mut fails: Vec<String> = Vec::new();
    for t in 0..3600u32 {
        w.tick(Input::default(), false);
        if t == 1800 { alarm_idle(&mut w); }
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || !d.enabled || d.corpse || d.health <= 0.0 { continue; }
            let looping = matches!(d.state, state::STEP_AIM_LEFT | state::STEP_AIM_RIGHT
                | state::STRAFE_AIM_LEFT | state::STRAFE_AIM_RIGHT
                | state::STRAFE_DODGE_LEFT | state::STRAFE_DODGE_RIGHT);
            if !looping { anchor[i].1 = 0; continue; }
            let moved = ((d.pos[0] - anchor[i].0[0]).powi(2) + (d.pos[1] - anchor[i].0[1]).powi(2) + (d.pos[2] - anchor[i].0[2]).powi(2)).sqrt();
            if moved >= 0.1 { anchor[i] = (d.pos, t); continue; }
            if anchor[i].1 == 0 { anchor[i] = (d.pos, t); continue; }
            if t - anchor[i].1 >= WINDOW {
                let row = format!("drone#{i} state {} anim {:#04x} pos ({:.2},{:.2},{:.2}) frozen from t{}",
                    state_name(d.state), d.anim.state, d.pos[0], d.pos[1], d.pos[2], anchor[i].1);
                if !fails.contains(&row) { fails.push(row); }
                anchor[i] = (d.pos, t);
            }
        }
    }
    assert!(fails.is_empty(), "guards looping a blocked combat step (the probe/mover disagreement the fix removes): {fails:?}");
}
