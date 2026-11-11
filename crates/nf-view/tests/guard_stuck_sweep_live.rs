//! The stuck-guard sweep: measures guards that stop moving while their state wants them to travel
//! (seen around 07000005's helipad and on other levels).
//!
//! Method per on-foot SP level: load through the real path (`nf_view::load`), let the level start,
//! then wake every live watching guard through the game's OWN alarm path — msg 0x1e (ALARM)
//! [M `DroneVision_EnemyAlerts` case 0x1e 0x177c18:113-114 — `+0x4F8 |= 0x4000000 | 0x40000000`,
//! SetState(0x56); the port's ALARM row stamps `alert_pending |= 0x40000000` and Attack] — and run
//! 60 s of fixed 60 Hz ticks. A drone is a HIT when it sits in a MOVING state — its mode/substate
//! wants it to travel:
//!   - ATTACK 0x56 — the chase/combat movement (the entry is `DroneFunc_FirstAttack` 0x148010, the
//!     tail runs the approach/strafe cycle) [M];
//!   - RUN_FOR_COVER 0x8c — run-to-cover [M `NDrone2_DSTATE_RunForCover` 0x169150];
//!   - ALERT_TO_POSITION 0x62 — walk to the alerted position [M `AlertToPosition` 0x165270];
//!   - SEARCH_AREA 0x65 — investigate [M `SearchArea` 0x165bf0];
//!   - HEARD_NOISE_AWARE/SUSPECT/ALERT 0xa3/0xa4/0xa5 — the sound-investigate family [M
//!     `HeardNoiseSuspect` 0x16b0e0];
//!   - GO_TO_GOAL 0x63 / GO_TO_GOAL_ALLY 0x28 — path follow (the port's GoToGoal, the
//!     GoToGoalPosition 0x63 arms) [M];
//!   - PATROL 0x07 and CIVILIAN_MISSION 0x16 — the mission-route walkers (`update_mission_route`'s
//!     `route.follow`, the `NDrone2_ReachedDestNode` 0x153358 machinery) [M];
//!   - INTEROGATE_ASSIST 0x39 — walk to the assist arrival point [M the TIMER_C walk 0x15add0 +
//!     0x1517d8:80-83];
//! yet its position moves < 0.1 over 3 s (180 ticks). Per hit the sweep logs: level, drone
//! index, skin, state, mode byte (+0xC5), position, the goal it holds (`+0x558`-family target),
//! `route_blocked` (the nav follow's BLOCKED/NO_PATH latch [M `Network::push_out` /
//! `update_mission_route`]), the goal distance, and whether it stands inside 07000005's helipad
//! courtyard box (the parked helicopter statics' spheres (−17.0, 43.0..44.8, −37.2..−43.6) [D
//! fix-portal5.md §6]). This test is LOG-ONLY (a measurement, not a pin); a follow-up fix turns
//! the exact rows into assertions.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::{anim_state, msg, state};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The on-foot SP levels the port loads (the census/walkreal coverage: 07000001–0d, 11–16, 1b(?),
/// 4a). `SWEEP_LEVELS` overrides (comma-separated) for a targeted run.
fn levels() -> Vec<String> {
    std::env::var("SWEEP_LEVELS").unwrap_or_else(|_| {
        "07000005,07000007,0700000c,07000001,07000002,07000004,07000006,07000008,07000009,0700000a,0700000b,0700000d,07000012,07000013,07000014,07000015,07000016".to_string()
    })
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect()
}

fn state_name(s: u8) -> &'static str {
    match s {
        state::ATTACK => "ATTACK 0x56 (chase)",
        state::RUN_FOR_COVER => "RUN_FOR_COVER 0x8c",
        state::ALERT_TO_POSITION => "ALERT_TO_POSITION 0x62",
        state::SEARCH_AREA => "SEARCH_AREA 0x65",
        state::HEARD_NOISE_AWARE => "HEARD_NOISE_AWARE 0xa3",
        state::HEARD_NOISE_SUSPECT => "HEARD_NOISE_SUSPECT 0xa4",
        state::HEARD_NOISE_ALERT => "HEARD_NOISE_ALERT 0xa5",
        state::GO_TO_GOAL => "GO_TO_GOAL 0x63",
        state::GO_TO_GOAL_ALLY => "GO_TO_GOAL_ALLY 0x28",
        state::PATROL => "PATROL 0x07 (route walk)",
        state::CIVILIAN_MISSION => "CIVILIAN_MISSION 0x16 (route walk)",
        state::INTEROGATE_ASSIST => "INTEROGATE_ASSIST 0x39",
        state::COMBAT_OUT_OF_RANGE => "COMBAT_OUT_OF_RANGE 0x5a (approach)",
        state::COMBAT_WAIT => "COMBAT_WAIT 0x5e (bunching wait)",
        state::AIM_BACKOFF => "AIM_BACKOFF 0x6d (back off)",
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
        _ => "?",
    }
}

fn moving(s: u8) -> bool {
    matches!(s, state::ATTACK | state::RUN_FOR_COVER | state::ALERT_TO_POSITION | state::SEARCH_AREA
        | state::HEARD_NOISE_AWARE | state::HEARD_NOISE_SUSPECT | state::HEARD_NOISE_ALERT
        | state::GO_TO_GOAL | state::GO_TO_GOAL_ALLY | state::INTEROGATE_ASSIST
        | state::PATROL | state::CIVILIAN_MISSION
        // The combat movers (stuck-guard-combat-sweep): the states whose game handlers walk, strafe
        // or back the guard.
        | state::COMBAT_OUT_OF_RANGE   // the approach chase [M CombatState router 0x164690:38-48]
        | state::COMBAT_WAIT           // DroneMove_NoBunching's parked mover [M 0x13cfb0:35-39]
        | state::AIM_BACKOFF           // the back-off walker [M CanBackoff 0x1501a0]
        | state::STEP_AIM_LEFT | state::STEP_AIM_RIGHT        // ChooseCombatMove's step pair
        | state::STRAFE_AIM_LEFT | state::STRAFE_AIM_RIGHT    // … strafe pair [M 0x150c18]
        | state::STRAFE_DODGE_LEFT | state::STRAFE_DODGE_RIGHT // … dodge pair
        | state::ROLL_LEFT_CROUCH | state::ROLL_RIGHT_CROUCH  // … roll pair
        | state::RUN_AWAY_FROM_OBJECT | state::HIDE_FROM_SCARY_OBJECT // the scare cover seek [M 0x168aa8/0x168d18]
        | state::UNDER_COVER_RETURN)   // walks back to cover while > 1.5 away [M 0x16a9c0 case 3]
}

/// The combat states that are NOT themselves movers but whose handlers pick combat moves /
/// move anims (`DroneAnim_SetCombatMoveAnim` runs inside them) [M the port's combat band,
/// drone.rs:3557-3559; DSTATE_Attack 0x164210, Combat 0x1731b8, AimStandFire 0x16cb70 family].
fn combat_band(s: u8) -> bool {
    matches!(s, state::ATTACK | state::COMBAT | state::COMBAT_NO_MOVE | state::COMBAT_OUT_OF_RANGE
        | state::COMBAT_NEW_SIGHTING | state::COMBAT_NO_SIGHT | state::COMBAT_WAIT | state::COMBAT_NO_ROUTE
        | state::AIM_STAND | state::AIM_STAND_FIRE | state::AIM_STAND_RELOAD | state::AIM_BACKOFF
        | state::CROUCH_COVER | state::AIM_CROUCH | state::AIM_CROUCH_FIRE | state::AIM_CROUCH_RELOAD
        | state::STEP_AIM_LEFT | state::STEP_AIM_RIGHT | state::STRAFE_AIM_LEFT | state::STRAFE_AIM_RIGHT)
}

/// The combat move anims `DroneAnim_SetCombatMoveAnim` (0x13cfb0) plays when the mover walks:
/// the quad anims — back 0x04, forward 0x08 (run-aim) / 0x0f (walk-aim), left 0x0c/0x29/0x5a,
/// right 0x0d/0x2a/0x5b [M 0x13cfb0:75-146/:193-244/:246-274/:276-392] — the rolls 0x53/0x55,
/// 0x54/0x56 [M :171-187/:374-390], and the plain run/walk the approach falls back to
/// [M moving_result's RUN arm; ALERT_WALK excluded — the search family's own walk].
fn combat_move_anim(a: u16) -> bool {
    matches!(a, anim_state::BACKOFF | anim_state::RUN_AIM | anim_state::WALK_AIM
        | anim_state::STRAFE_LEFT | anim_state::STRAFE_RIGHT
        | anim_state::STRAFE_LEFT_ALT | anim_state::STRAFE_RIGHT_ALT
        | anim_state::ROLL_LEFT | anim_state::ROLL_RIGHT
        | anim_state::ROLL_LEFT_ALT | anim_state::ROLL_RIGHT_ALT
        | anim_state::STEP_LEFT | anim_state::STEP_RIGHT
        | anim_state::RUN | anim_state::WALK)
}

/// A drone the extended moving set OR the combat-band anim check holds: its mode wants travel —
/// a DSTATE mover, or a combat-band state playing a combat move anim [M 0x13cfb0].
fn wants_travel(d: &nf_game::drone::Drone) -> bool {
    moving(d.state) || (combat_band(d.state) && combat_move_anim(d.anim.state))
}

/// 07000005's helipad courtyard box [D fix-portal5.md §6 — the heli statics' spheres].
fn in_helipad_box(level: &str, p: [f32; 3]) -> bool {
    level == "07000005" && (-30.0..2.0).contains(&p[0]) && p[1] > 30.0 && (-58.0..-22.0).contains(&p[2])
}

fn sweep_level(lv: &str) {
    let Ok(mut w) = nf_view::load(&iso(), &format!("{lv}.bin"), false)
        .map(walker::Walker::new) else { eprintln!("SWEEP {lv}: LOAD FAILED"); return; };
    if !w.wait_enabled(6000, "level start") { eprintln!("SWEEP {lv}: never enabled"); return; }
    w.wait(120); // the level's first second settles (spawners, scripts)
    // The game's own alarm path on every live watching guard: msg 0x1e [M 0x177c18:113-114].
    {
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
            // The alarm row's own state band [M 0x161860 case 0x1e — EnemyAlerts serves the unaware
            // family; the port's ALARM arm covers IDLE|ALERT|PATROL|ALERT_TO_POSITION|SEARCH_AREA|
            // HEARD_NOISE_*|INTEROGATE_ASSIST_WAIT]. The route walkers (PATROL/CIVILIAN_MISSION)
            // are already moving — they are swept as-is.
            if d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
                && matches!(d.state, state::IDLE | state::ALERT | state::PATROL) {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
                woken += 1;
            }
        }
        eprintln!("SWEEP {lv}: alarmed {woken} guards [M 0x177c18:113-114]");
    }
    // 60 s of ticks; per drone, an anchor (pos, tick) resets whenever it leaves a MOVING state or
    // moves ≥ 0.1; a hit is 180 consecutive ticks in a MOVING state within 0.1 of the anchor.
    const WINDOW: u32 = 180;
    let mut anchor: Vec<([f32; 3], u32)> = Vec::new();
    let mut hits: Vec<String> = Vec::new();
    for t in 0..3600u32 {
        w.tick(Input::default(), false);
        // A second alarm pass at t=1800: guards that lost the alert (the idle timeout msg 4 re-arms
        // [M 0x148db0:89-96]) re-engage, so a mid-level wedge in the second half still measures.
        if t == 1800 {
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
            for i in 0..sim.drones.list.len() {
                let d = &sim.drones.list[i];
                if d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
                    && matches!(d.state, state::IDLE | state::ALERT) {
                    sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
                }
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        if anchor.len() != sim.drones.list.len() { anchor.resize(sim.drones.list.len(), ([0.0; 3], 0)); }
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || !d.enabled || d.corpse || d.health <= 0.0 { continue; }
            if !wants_travel(d) { anchor[i].1 = 0; continue; }
            let moved = ((d.pos[0] - anchor[i].0[0]).powi(2) + (d.pos[1] - anchor[i].0[1]).powi(2) + (d.pos[2] - anchor[i].0[2]).powi(2)).sqrt();
            if moved >= 0.1 { anchor[i] = (d.pos, t); continue; }
            if anchor[i].1 == 0 { anchor[i] = (d.pos, t); continue; }
            if t - anchor[i].1 >= WINDOW {
                let gd = d.goal.map(|g| ((g[0] - d.pos[0]).powi(2) + (g[1] - d.pos[1]).powi(2) + (g[2] - d.pos[2]).powi(2)).sqrt());
                let row = format!(
                    "HIT {lv} drone#{i} skin {:#010x} state {} mode {:#04x} anim {:#04x} pos ({:.2},{:.2},{:.2}) goal {:?} goal_dist {:?} route_blocked {} helipad_box {}",
                    d.skin, state_name(d.state), d.mode, d.anim.state, d.pos[0], d.pos[1], d.pos[2],
                    d.goal.map(|g| (g[0], g[1], g[2])), gd.map(|x| (x * 100.0).round() / 100.0),
                    d.route_blocked, in_helipad_box(lv, d.pos));
                if !hits.contains(&row) { hits.push(row.clone()); eprintln!("{row}"); }
                anchor[i] = (d.pos, t); // re-anchor: one row per (drone, spot) wedge
            }
        }
    }
    eprintln!("SWEEP {lv}: {} hits over 60 s", hits.len());
}

/// The sweep itself: log-only. Every HIT line is a row of the report's table; a level that fails
/// to load or panics is a SKIP row (its absence from the table is explicit, not silent).
#[test]
fn the_sweep_logs_every_frozen_moving_drone() {
    for lv in levels() {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sweep_level(&lv)));
        if r.is_err() { eprintln!("SWEEP {lv}: SKIP (panic — see the report's skip list)"); }
    }
}

/// guard-stuck-0c-14: the 07000014 door-leaf walker slides along the leaf. The sweep's HIT row
/// (07000014 drone#0, skin 0x05000090, GO_TO_GOAL 0x63, pos (50.34,1.00,53.29), goal
/// (56.06,0.01,42.91) 11.9 u away, route_blocked false) is a guard whose route creeps toward a
/// SPECIAL (door) node at (51.54, 52.34) across the leaf of door object 0x100_0000_001A
/// (hinge (50.82, 1.19, 52.92), width along world z [L the probe's HitHost + matrix dump]): the
/// mover's 0.04 step is rejected every tick (`blocked_reach` 0.040 [L the probe ledger]) and the
/// guard stands 1.5 u short of the node forever. The game keeps the along-leaf part of a
/// leaf-blocked step [M Drone_CollisionHandler 0x140968:204-208] — `door_leaf_slide` (the port's
/// stand-in) now runs on this level, so the guard rounds the hinge southward: z 53.29 → 52.34
/// [L the post-fix sweep row]. RED on the old code: the gate held the slide to 07000007 and the
/// guard never left (50.34, 53.29). The REMAINING corner (pure-east steps have no tangent and
/// the leaf's z-band still covers the creep line) is the next round's item — the game's
/// LinkCreep_AtSpecialNode tier-2 (0x159dd0 → NDrone2_DroneNearPos 0x143e68 walks the DRONE
/// list; the port asks the player) is the divergence candidate.
#[test]
fn the_countdown_leaf_walker_slides_along_the_leaf() {
    let Ok(mut w) = nf_view::load(&iso(), "07000014.bin", false).map(walker::Walker::new)
        else { eprintln!("07000014: LOAD FAILED"); return; };
    assert!(w.wait_enabled(6000, "level start"), "07000014: never enabled");
    w.wait(120);
    // The game's own alarm path on every live watching guard: msg 0x1e [M 0x177c18:113-114].
    {
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
        for i in 0..sim.drones.list.len() {
            let d = &sim.drones.list[i];
            if d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
                && matches!(d.state, state::IDLE | state::ALERT | state::PATROL) {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
            }
        }
    }
    // The leaf walker: the GO_TO_GOAL guard whose goal is the disc's (56.06, 0.01, 42.91)
    // [L the sweep row; the goal comes from the level's own script/placement data].
    let mut found = false;
    let mut reached_leaf = false;
    let mut slid = false;
    for t in 0..1800u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        for d in sim.drones.list.iter() {
            if !d.present || !d.enabled || d.corpse { continue; }
            if d.state != state::GO_TO_GOAL { continue; }
            let Some(g) = d.goal else { continue };
            if ((g[0] - 56.06).abs() > 0.5) || ((g[2] - 42.91).abs() > 0.5) { continue; }
            found = true;
            // The leaf plane is x ≈ 50.82: the walker's approach brings it within 0.5 of the plane.
            if (d.pos[0] - 50.82).abs() < 0.5 { reached_leaf = true; }
            // The along-leaf slide carries it south past the hinge's z (52.92 [L the matrix]).
            if d.pos[2] < 52.90 && d.pos[2] > 51.5 { slid = true; }
        }
    }
    assert!(found, "the 07000014 leaf walker (goal (56.06, 0.01, 42.91) [L the sweep row]) never took the alarm in GO_TO_GOAL");
    assert!(reached_leaf, "the walker never reached the leaf's plane (x 50.82 [L the probe's HitHost]): the leaf-slide law [M 0x140968:204-208] is not applied on this level");
    assert!(slid, "the walker reached the leaf but never slid along it (z stayed at 53.29): the leaf-slide law [M 0x140968:204-208] is not applied on this level");
}

/// guard-leaf-pushout: the 07000014 leaf press rests IN CONTACT (the game's move-then-pushout
/// order), and the pressed walker completes his mission once the player opens the door.
///
/// The freeze facts [L this round's probe runs]: the Countdown leaf walker (GO_TO_GOAL 0x63,
/// goal (56.06, 0.01, 42.91) — the ALARM node [M 0x168ea8: the route's reached node enters
/// PressAlarm 0x8a, the flow run parks him there]) presses the closed 0x1A leaf's west face.
/// The leaf geometry [L the posed-tri AABB dump]: a double SLIDE door (kind 2), both leaves
/// hinged at (50.82, 1.19, 52.92); 0x1A covers z 51.66..53.00, its west face x 50.803-50.813;
/// the twin 0x19 covers z 52.84..54.18 — the doorway seals shut. The game's order [M
/// Drone_CollisionHandler 0x140968:204-208 — obj+0x30 += the delta BEFORE the contact pass;
/// Collide_Intersect 0x1ebaa8:353-373 — the pass's DeltaP resolves the capsule]: the body rests
/// IN CONTACT, the collide sphere radius 0.4 [M NDrone2_Collision 0x140e90:91-93, `+0x45c`]—
/// i.e. the center at the face minus 0.4 = 50.403. The old reject left it at 50.378 (the probe's
/// 0.4 overshoot + the step), out of contact — the press the door's own `Door_ReactToHit`
/// (0x135980) reads from obj+0xD0 never registered. RED on the old code: the walker's pressed
/// rest x stays <= 50.39; GREEN: it crosses 50.39 (the contact).
///
/// The door cannot open by itself [M]: `Door_ReactToHit`'s kind-2 arm reverses a CLOSING leaf
/// only (a closed one returns); ch_open/ch_lock are 0 [L the door dump]; the route's door node
/// reach needs dist < 0.4, or 0.4-1.0 with a peer/player on the node [M LinkCreep_AtSpecialNode
/// 0x159dd0] — the pressed walker stands 1.13 from the node (50.403 vs 51.537, [L]), outside
/// the band, and no other drone ever comes within 0.4 ([L] the 60 s two-drone log: drone#1
/// strafes 1.05-3.5 off). So the no-input press is the GAME's own behavior — the pin for the
/// game-law fix is the press position itself, plus the REAL-PLAY reach: the player's
/// `Door_Activate` (0x135be0 — the same call the use path makes, both leaves, group 1) opens
/// the door, and the walker flows through the opening leaves and walks the rest of his route to
/// the alarm ([L the flow run: x 50.55 → 51.14 → 52.06 …, the route advances idx 5→6→7→8, the
/// PressAlarm entry at (55.18, 1.00, 44.48), 1.80 from the goal]).
#[test]
fn the_countdown_leaf_walker_presses_in_contact_and_reaches_his_goal_once_the_door_opens() {
    let Ok(mut w) = nf_view::load(&iso(), "07000014.bin", false).map(walker::Walker::new)
        else { eprintln!("07000014: LOAD FAILED"); return; };
    assert!(w.wait_enabled(6000, "level start"), "07000014: never enabled");
    w.wait(120);
    // The game's own alarm path on every live watching guard: msg 0x1e [M 0x177c18:113-114].
    {
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
        for i in 0..sim.drones.list.len() {
            let d = &sim.drones.list[i];
            if d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
                && matches!(d.state, state::IDLE | state::ALERT | state::PATROL) {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
            }
        }
    }
    // Phase 1 (t 0..600): the walk to the leaf and the press. Track the pressed rest.
    let mut found = false;
    let mut pressed_rest_x = 0.0f32;
    // guard-leaf-press-07 (the door-side pin): during the press, doors.collide_step must see the
    // pressing guard in door 0x1A's contacts — the obj+0xD0 list Door_ReactToHit (0x135980)
    // reads — because his capsule rests IN CONTACT under the move-then-pushout order. The
    // contact's centre is the mover's root pos [M Drones::tick's DoorMover build: centre = x.pos;
    // Doors::collide_step pushes DoorContact { centre: m.centre } on the capsule-meets-tris hit].
    let mut press_contact_seen = false;
    for t in 0..600u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        for d in sim.drones.list.iter() {
            if !d.present || !d.enabled || d.corpse { continue; }
            if d.state != state::GO_TO_GOAL { continue; }
            let Some(g) = d.goal else { continue };
            if ((g[0] - 56.06).abs() > 0.5) || ((g[2] - 42.91).abs() > 0.5) { continue; }
            found = true;
            // The leaf zone [L the sweep row]: once within 0.5 of the leaf plane x 50.82,
            // the walker's rest x IS the press position.
            if (d.pos[0] - 50.82).abs() < 0.5 {
                pressed_rest_x = pressed_rest_x.max(d.pos[0]);
                if !press_contact_seen
                    && sim
                        .drones
                        .doors
                        .contact_centres(26)
                        .iter()
                        .any(|(c, pl)| !*pl && (c[0] - d.pos[0]).abs() < 0.05 && (c[2] - d.pos[2]).abs() < 0.05)
                {
                    press_contact_seen = true;
                }
            }
        }
    }
    assert!(found, "the 07000014 leaf walker (goal (56.06, 0.01, 42.91) [L the sweep row]) never took the alarm in GO_TO_GOAL");
    // THE PRESS [M 0x140968:204-208 + 0x1ebaa8:353-373, radius 0.4 0x140e90:91-93]: the rest is
    // IN CONTACT — the center within 0.42 of the face x 50.803-50.813 [L the probe's HitHost],
    // i.e. x > 50.39. The old reject rested 50.378 (0.435 out) [L the pre-fix probe ledger].
    assert!(pressed_rest_x > 50.39, "the leaf walker's pressed rest x {:.3} never entered contact (the face 50.803 [L] minus the collide radius 0.4 [M 0x140e90:91-93] = 50.403; the old reject held 50.378) — the move-then-pushout order [M 0x140968:204-208] is not applied", pressed_rest_x);
    // THE DOOR SIDE [M Door_ReactToHit 0x135980 reads obj+0xD0; the collide step's contact push
    // on the capsule-meets-tris hit]: during the press the pressing guard sits in door 0x1A's
    // (list index 26) contact list — the door's own react path sees him every press tick.
    assert!(press_contact_seen, "during the press, door 0x1A's obj+0xD0 contacts never carried the pressing guard (centre = his root pos [M the DoorMover build]) — Door_ReactToHit's read [M 0x135980] would see an empty list");
    // Phase 2: the player's act — Door_Activate (0x135be0) on both leaves (group 1), then the
    // walker flows through and walks his route to the alarm [L the flow run].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let actors = vec![(sim.bond.pos, 0.4, true)];
        let mut evs = Vec::new();
        for di in [25usize, 26] {
            sim.drones.doors.activate(di, &actors, true, &mut sim.drones.channels, sim.drones.frame, &mut evs);
        }
    }
    let mut crossed = false;
    let mut at_alarm = false;
    for _ in 0..1800u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        for d in sim.drones.list.iter() {
            if !d.present || !d.enabled || d.corpse { continue; }
            let Some(g) = d.goal else { continue };
            if ((g[0] - 56.06).abs() > 0.5) || ((g[2] - 42.91).abs() > 0.5) { continue; }
            if d.pos[0] > 51.3 && (51.0..54.0).contains(&d.pos[2]) { crossed = true; }
            // The PressAlarm entry park [L the flow run: (55.18, 1.00, 44.48)] — the route's
            // last node, 1.80 from the alarm spot.
            if d.state == 0x8a && d.pos[0] > 54.5 && d.pos[2] < 45.5 { at_alarm = true; }
        }
        if crossed && at_alarm { break; }
    }
    assert!(crossed, "the leaf walker never crossed the opened doorway (x > 51.3) — the opened double door did not free him");
    assert!(at_alarm, "the leaf walker crossed but never reached his route's end (the PressAlarm park at (55.18, 1.00, 44.48) [L the flow run]) — the goal (56.06, 0.01, 42.91) [L the sweep row] is the alarm node");
}
