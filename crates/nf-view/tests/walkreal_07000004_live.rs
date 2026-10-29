//! walkreal-a: 07000004 "Enemies Vanish" (Castle D) walked START TO FINISH with the player's own
//! movement — no teleports after the spawn. The route (docs/levels/07000004.md):
//!   the start (1.87, 5.84, 41.07) → the intro SP #3 + the story scene SP #2 settle by ~tick 1600
//!   (the AI re-enabled, ch 0x53 written; the story places Bond beside the target) → the armament
//!   walk (the StormUzi pickup #7 at (−10.66, 4.39, 42.66), the BodyArmourHeavy #9 at
//!   (−1.85, 4.37, 46.07) [D]) → back to the target (drone skin 0x0500001e, stationary) → the real
//!   fire path kills him → ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660] → the OUT trigger #46
//!   "MultiplexOut" (out 0x54, list [0x64] [D]) copies it into **ch 0x64 = channel 100 = the
//!   mission-complete channel** while Bond stands inside [M Trigger_MultiplexOut] → `Mission_Update`
//!   (0x182c60) internal 1 sees ch 100 → case 6 posts "Mission Complete" (LevelToEndTo
//!   **0x0730004**, the next arc's driving interlude [M 0x182c60:85-122]) → case 7 holds the 240-
//!   tick banner → case 4 hands `ResetMap_LevelToLoad(0x0730004)` [M 0x182c60:65-76] — the level's
//!   own exit (no 0xE8 trigger exists on this level [D placement probe]).
//! level-04-complete: the walk now carries the mission layer to the banner end and pins the events.
//! every-objective: the level's mission data lists EXACTLY ONE objective record — obj0 0x0400000d
//! "Defeat the assassin", fail none, hint 0x04000015, condition ch 0x54, enable ch 0x53, flags 0
//! [L ee_snapshot.bin 0x2a4350 + row 7*40 (level 07000004) → objective table 0x2a3f60, count 1;
//! matches MissionData symbol MissionData[7] [M ACTION.ELF 0x2a4350]]. So the objective list the
//! current tests already cover is {obj0}, covered COMPLETE by walk_the_full_mission_start_to_finish
//! (announced state 3 → the pickups → the kill → state 4 → the banner → the level's exit firing);
//! there are no further objectives to leg. That test now pins the record count itself.

#[path = "common/walker.rs"]
mod walker;

use nf_game::mission;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mission-table index of THIS level's record [L ee_start_05.bin 0x2a4350].
fn mission_row(level: u32) -> usize {
    mission::LEVELS.iter().position(|l| l.level == level).expect("the level's mission record")
}

/// Load and skip the intro + story cut-scenes the way the player does (the game's own skip action)
/// until the AI re-enables and ch 0x53 (the castle-C hand-off [D script 0x06000082 stream 9]) is
/// written. Returns the settled walker.
fn load_ready(iso: &str) -> Walker {
    let mut w = Walker::new(nf_view::load(iso, "07000004.bin", false).expect("load 07000004"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0004);
        assert!(sim.drones.triggers.iter().all(|t| t.kind != walker::trigger_kind::LOAD_LEVEL),
            "07000004 has no 0xE8 exit [D placement probe] — the mission layer IS the exit");
        let p = sim.bond.pos;
        assert!((p[0] - 1.87).abs() < 2.0 && (p[2] - 41.07).abs() < 2.0, "start #0 [D], got {p:?}");
    }
    let mut ready = false;
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { ready = true; break; }
    }
    assert!(ready, "the intro + story scenes must settle (the AI re-enabled, ch 0x53 written)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[04] settled at tick {} bond ({:.2},{:.2},{:.2}) ch 0x53 on", w.total_ticks, p[0], p[1], p[2]);
        assert!(!sim.drones.channels.on(0x54), "the objective channel starts off");
        assert_eq!(sim.drones.mission.state, 1, "the mission runs [M Mission_Init 0x182270]");
    }
    w
}

/// The OUT trigger #46's placement box (print helper) [D]. Unbounded (no collision mesh) — the
/// mux needs none [M 0x1b1a78 case 5].
#[allow(dead_code)]
fn out_zone(w: &Walker) -> [f32; 3] {
    let sim = w.scene.sim.as_ref().unwrap();
    let t = sim.drones.triggers.iter()
        .find(|t| t.kind == nf_game::triggers::kind::OUT)
        .expect("the MultiplexOut trigger #46 [D]");
    let c = [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5];
    eprintln!("[04] the MultiplexOut box lo {:?} hi {:?} → centre {:?}", t.lo, t.hi, c);
    c
}

#[test]
fn walk_the_full_mission_start_to_finish() {
    let iso = iso();
    let mut w = load_ready(&iso);
    let row = mission_row(0x0700_0004);
    // The WHOLE objective list the mission data lists [L ee_snapshot.bin 0x2a4350 row 7 → table
    // 0x2a3f60 count 1]: one record, obj0. Completing it completes every objective of the level.
    assert_eq!(mission::LEVELS[row].objectives.len(), 1,
        "07000004's mission data lists exactly one objective record [L ee_snapshot.bin 0x2a4350 row 7 → 0x2a3f60 count 1]");
    // The monitor's enable arm: ch 0x53 (the story hand-off) is ON → obj0 announces once [M
    // 0x1827d8:55-67]. Two ticks so the fold's own tick has run.
    w.tick(Default::default(), false);
    w.tick(Default::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[row][0], 3,
            "obj0 'Defeat the assassin' announced through the real monitor (enable ch 0x53 on) [M 0x1827d8:55-67]");
    }
    // The armament walk (the route's own pickups [D #6/#7/#8 StormUzi + #9 BodyArmourHeavy]):
    // the open-lawn uzi #7 (−10.66, 4.39, 42.66), then the armour #9 (−1.85, 4.37, 46.07).
    let (uzi, armour) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let uzi = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && !p.gone && (p.start[2] - 42.66).abs() < 1.0)
            .or_else(|| sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone))
            .map(|p| p.start).expect("the StormUzi pickup");
        let armour = sim.drones.pickups.iter().find(|p| p.kind == 3 && !p.gone).map(|p| p.start)
            .expect("the BodyArmourHeavy pickup");
        (uzi, armour)
    };
    w.walk_nav_to(uzi, "the StormUzi pickup", 0.4);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.inventory.owned.get(18).copied().unwrap_or(false),
            "walking over the pickup owns weapon 18 [M Pickup_Handler 0x18e650 type-0 arm]");
    }
    w.walk_nav_to(armour, "the armour pickup", 0.4);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.pickups.iter().all(|p| p.kind != 3 || p.gone),
            "the armour case is consumed (respawn param 5 absent → deleted [D params; M 0x18e650:338-348])");
        assert!(sim.drones.health.armour > 0.0,
            "the take raised armour from 0 to 50 [M 0x18e650:220-225]; the settled assassin's bullet-type pain may chip it on the walk (48.5 measured) — the raise itself is the [M] claim");
    }
    // The assassination: walk back to the target and kill him with the player's own fire path.
    let target_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_001e).expect("the target drone exists");
    let killed = w.shoot(target_i, "the target");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(killed || sim.drones.list[target_i].corpse || sim.drones.list[target_i].health <= 0.0,
            "the target must die (the walk's own shots)");
    }
    // The death channel settles over the death anim; the objective folds through the REAL monitor.
    let mut raised = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x54) { raised = true; break; }
    }
    assert!(raised, "the target's death must raise ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660]");
    // The OUT mux (trigger #46, out 0x54 → list [0x64] = ch 100): Trigger_Update's case 5 copies
    // the OUT channel into every list channel EVERY tick with NO touch/hit check [M 0x1b1a78
    // case 5:147-160 — Trigger_Hit's result is not consulted; the placement has no collision mesh
    // [D unbounded bbox] and needs none]. The monitor runs at the next tick's top; give both their
    // tick, then pin the fold + the copy.
    for _ in 0..30u32 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[row][0], 4,
            "obj0 'Defeat the assassin' folds complete through Mission_MonitorObjectives [M 0x1827d8:78-85]");
        assert!(sim.drones.channels.on(100),
            "the OUT mux mirrors ch 0x54 into ch 100 = the mission-complete channel [D #46 list [0x64]; M 0x1b1a78 case 5]");
    }
    // The game's exit: Mission_Update's next tick sees ch 100 → case 6 (the "Mission Complete"
    // banner, LevelToEndTo 0x0730004) → case 7's 240-tick hold → case 4's ResetMap_LevelToLoad
    // [M 0x182c60:85-122, 65-76]. The walker's ticks drain the layer's events through the scene's
    // real hud path (`Sim::step_bond` ends in `hud_tick` [M scene.rs] — the game's own Game_Run
    // order), so the exit arrives as the viewer's `mission_end` latch: 0x0730004 is a driving
    // interlude code → `ptp_handover` + `mission_end` (scene.rs's LevelEnd arm — the port's end
    // sequence: outro → results). Wait for it and pin the layer's globals.
    let mut ended = false;
    for _ in 0..500u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission_end.is_some() { ended = true; break; }
    }
    assert!(ended, "the mission-complete sequence must land (case 6 banner → case 4 ResetMap)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission_end, Some(0x0730_0004),
            "LevelEnd 0x0730004 → the end sequence with LevelToEndTo 0x0730004 [M 0x182c60:65-76, 89-90]");
        assert_eq!(sim.drones.mission.state, 6, "MissionState 6 = complete [M 0x182c60:88]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0004, "LevelToEndTo [M 0x182c60:89-90]");
        assert_eq!(sim.drones.mission.states[row][0], 4, "the objective stays complete");
    }
    eprintln!("[04] walked the FULL mission in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[04] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[04] stuck: {s}"); }
}

/// Sister: without the assassination the level can never end. Bond walks the route (the pickups)
/// while the target stays ALIVE — ch 0x54 stays off, the OUT mux copies nothing, the mission layer
/// never leaves internal 1/MissionState 1 and no banner events queue. This is the assert that
/// fails on the old code if anything but the player's own kill completes the mission.
#[test]
fn without_the_assassination_the_mission_stays_open() {
    let iso = iso();
    let mut w = load_ready(&iso);
    let row = mission_row(0x0700_0004);
    // Walk the route's pickups (the arming alone must not complete anything).
    let (uzi, armour) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let uzi = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && !p.gone && (p.start[2] - 42.66).abs() < 1.0)
            .or_else(|| sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone))
            .map(|p| p.start).expect("the StormUzi pickup");
        let armour = sim.drones.pickups.iter().find(|p| p.kind == 3 && !p.gone).map(|p| p.start)
            .expect("the BodyArmourHeavy pickup");
        (uzi, armour)
    };
    w.walk_nav_to(uzi, "sister: the StormUzi", 0.4);
    w.walk_nav_to(armour, "sister: the armour", 0.4);
    // Long wait with the target alive (the assassin stalks Bond the whole time).
    for _ in 0..600u32 {
        w.tick(Default::default(), false);
        for _ in std::mem::take(&mut w.scene.sim.as_mut().unwrap().drones.mission_events) {}
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x54), "no shots → the target lives, ch 0x54 off");
    assert!(!sim.drones.channels.on(100), "the OUT zone copies nothing while 0x54 is off [M Trigger_MultiplexOut]");
    assert_ne!(sim.drones.mission.state, 6, "the mission never completes without the kill");
    assert_eq!(sim.drones.mission.states[row][0], 3,
        "obj0 sits ANNOUNCED, never complete — the condition ch 0x54 only rises on the kill [M 0x1827d8 state 3 arm]");
    // The wait above kept the layer in internal 1 the whole time.
    assert_eq!(sim.drones.mission.internal, 1, "Mission_Update never left the running arm [M 0x182c60 case 1]");
}

/// The original assassination walk (the channel-level pin), kept focused: kill → ch 0x54 → the
/// OUT mux mirrors it into ch 100. The full-mission test above adds the monitor fold + the banner
/// end.
#[test]
fn walk_the_assassination() {
    let iso = iso();
    let mut w = load_ready(&iso);
    let target_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_001e).expect("the target drone exists");
    let killed = w.shoot(target_i, "the target");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(killed || sim.drones.list[target_i].corpse || sim.drones.list[target_i].health <= 0.0,
            "the target must die (the walk's own shots)");
    }
    let mut raised = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x54) { raised = true; break; }
    }
    assert!(raised, "the target's death must raise ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660]");
    for _ in 0..30u32 { w.tick(Default::default(), false); }
    let copied = w.scene.sim.as_ref().unwrap().drones.channels.on(100);
    assert!(copied, "the OUT mux #46 must copy ch 0x54 to the mission-complete ch 100 [M 0x1b1a78 case 5]");
    eprintln!("[04] walked the assassination in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[04] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[04] stuck: {s}"); }
}

/// level-04-mission-complete-early: the kill leads into the game's outro — the REAL-input walk to
/// the ninja kill, then the game's end sequence, pinned step by step:
/// 1. the kill → ch 0x54 → the #46 mux → ch 100 → `Mission_Update` case 6 posts the "Mission
///    Complete" banner (0x2000006, type 3, 240 ticks) and enters case 7 [M 0x182c60:85-122];
/// 2. case 7 runs `GS_PausePlayer(1, 0)` EVERY tick of the hold [M 0x182c60:123-130] — Bond's
///    input is dead (the port's `mission_hold` gate) while the HUD keeps running (the banner's
///    message ticks down; no script camera holds the +0x2a gate);
/// 3. case 4 (success arm, MissionState 6) → `ResetMap_LevelToLoad(0x0730004, 0, 0)` +
///    `GameFlow_PushState(60.0, 7, …)` [M 0x182c60:65-76] — the sim latches
///    `mission_end = Some(0x0730004)`, the host's `demo_results` request;
/// 4. the host's end sequence (the port of the game's loader path [M `Menu_IsDrivingLevel`
///    0x202750 rejects 0x0730004 → the on-foot loader; `ResetMap_Load` 0x1d6360's movie branch:
///    SP_Create(0x600095d) + GameFlow state 6 + `psiStartBackgroundMovie(0x0730004)`;
///    `GetLevelToLoadFromMovieCode` 0x1d3560 returns 0x0700048): the front-end flow reaches the
///    movie state with the outro code, the movie's disc entry exists [D], and phase 2 resolves to
///    the front-end map with the queued results page opening at the load tail [M 0x1d6360:160-163].
/// "Destroy Mayhew's computer" (0x04000007) over gameplay is the game's own cold-boot announce
/// queue, not a port bug: the disc ships every record STATE 0 [D ACTION.ELF .data], the monitor's
/// case 0 announces the earlier parts' records once `record.level <= current` [M 0x1827d8:41-54],
/// and the game does the same on a fresh load [L a PCSX2 fresh 0700000b boot shows part-0/1
/// records announcing] — this test also pins that the level's own kill shows NO new-objective
/// write for 0x04000007 (its state moved 0→3 at boot, never back).
#[test]
fn the_kill_leads_into_the_games_outro() {
    let iso = iso();
    let mut w = load_ready(&iso);
    let row = mission_row(0x0700_0004);
    // The armament walk + the assassination (the full-mission test's route, the real input path).
    w.tick(Default::default(), false);
    w.tick(Default::default(), false);
    let (uzi, armour) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let uzi = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && !p.gone && (p.start[2] - 42.66).abs() < 1.0)
            .or_else(|| sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone))
            .map(|p| p.start).expect("the StormUzi pickup");
        let armour = sim.drones.pickups.iter().find(|p| p.kind == 3 && !p.gone).map(|p| p.start)
            .expect("the BodyArmourHeavy pickup");
        (uzi, armour)
    };
    w.walk_nav_to(uzi, "the StormUzi pickup", 0.4);
    w.walk_nav_to(armour, "the armour pickup", 0.4);
    let target_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_001e).expect("the target drone exists");
    let killed = w.shoot(target_i, "the target");
    assert!(killed || w.scene.sim.as_ref().unwrap().drones.list[target_i].corpse
        || w.scene.sim.as_ref().unwrap().drones.list[target_i].health <= 0.0, "the target must die");
    let mut raised = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x54) { raised = true; break; }
    }
    assert!(raised, "the kill must raise ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660]");

    // Case 6 → case 7: the "Mission Complete" banner + the GS_PausePlayer hold. The kill's own
    // objective writes: obj0 folds 3 → 4 with the 0x02000053 "OBJECTIVE COMPLETE" notice; NO
    // 0x04000007 write exists on the kill path (its record left state 0 at boot, case 0).
    let mut hold_seen = false;
    for _ in 0..60u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.mission.internal == 7 { hold_seen = true; break; }
    }
    assert!(hold_seen, "the mission layer must reach case 7 (the banner hold) [M 0x182c60:85-122]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.state, 6, "MissionState 6 = complete [M 0x182c60:88]");
        assert_eq!(sim.drones.mission.states[row][0], 4, "obj0 complete through the real monitor [M 0x1827d8:78-85]");
        let obj_labels: Vec<u32> = sim.hud.objective_msgs.iter().map(|m| m.label).collect();
        assert!(!obj_labels.contains(&0x0400_0007),
            "no 'Destroy Mayhew's computer' write on the kill path — its announce was the cold-boot case 0 [M 0x1827d8:41-54], queue now {obj_labels:?}");
        let banner = sim.hud.mission_msgs.front().expect("the Mission Complete banner is up");
        assert_eq!(banner.text, "Mission Complete", "the 0x2000006 type-3 banner [M 0x182c60:117-118]");
        eprintln!("[04-outro] the hold: banner life {:.0}, objective queue {:?}", banner.time,
            sim.hud.objective_msgs.iter().map(|m| m.label).collect::<Vec<_>>());
    }

    // The GS_PausePlayer hold: Bond's input is dead, the HUD alive.
    let start = w.scene.sim.as_ref().unwrap().bond.pos;
    let mut banner_life_seen = 0.0f32;
    for _ in 0..60u32 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(m) = sim.hud.mission_msgs.front() { banner_life_seen = banner_life_seen.max(m.time); }
        if sim.drones.mission.internal != 7 { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let dx = ((sim.bond.pos[0] - start[0]).powi(2) + (sim.bond.pos[2] - start[2]).powi(2)).sqrt();
        assert!(dx < 0.2,
            "case 7 pauses the player [M 0x182c60:124 GS_PausePlayer(1,0) per tick] — Bond drifted {dx:.3} under the banner");
        assert!(banner_life_seen > 0.0, "the banner keeps ticking through the hold (the HUD is not paused) [M 0x1b11d8]");
        assert!(!sim.drones.hud_frozen, "no script camera holds the +0x2a gate on this path");
    }

    // Case 4 → the level end: the sim latches mission_end (the host's demo_results request).
    let mut ended = false;
    for _ in 0..500u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission_end.is_some() { ended = true; break; }
    }
    assert!(ended, "case 4 must fire the level end [M 0x182c60:65-76]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission_end, Some(0x0730_0004), "LevelToEndTo 0x0730004 [M 0x182c60:89-90]");
        assert_eq!(sim.drones.mission.internal, 5, "case 4's terminal arm [M 0x182c60:75]");
    }

    // The host's end sequence (the port of the game's loader path): the front-end flow wraps the
    // outro code into the movie state, the movie's disc entry exists, phase 2 resolves to the
    // front-end map and the queued results page opens at the load tail.
    let mut fe = nf_view::menu::FrontEnd::new();
    fe.flow.current_level = 0x0700_0004;
    fe.flow.stack = vec![nf_view::menu::ST_RUNNER]; // the host's demo_results boots the runner state
    fe.mission_ended(nf_view::menu::Outcome::Complete, nf_view::menu::Stats::default());
    assert_eq!(fe.requested_level.take(), Some(0x0730_0004), "LevelToEndTo rides the end request [M 0x182c60:89-90]");
    assert!(fe.flow.movie_pending, "the 0x73 code wraps to a movie [M 0x1d6360's (code & 0xff0000) branch]");
    assert_eq!(fe.flow.page, 0x4000_0036, "the results page id queued [M 0x182c60:72]");
    assert_ne!(fe.page, nf_view::menu::Page::Results, "the results wait behind the outro [M 0x1d6360:160-163]");
    // The disc entry [D]: psiStartBackgroundMovie 0x1081a8 builds /MOVIES/30_FPS/<hash>.PSS.
    {
        let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
        let e = img.find("/MOVIES/30_FPS/07300004.PSS")
            .expect("the Double Cross outro movie entry exists in the image tree [D psiStartBackgroundMovie 0x1081a8]");
        eprintln!("[04-outro] the movie entry: size {} (0 on the slim on-foot image — the game's own BGFMVPlaying==0 arm pops state 6 then [M 0x1082e8])", e.size);
    }
    fe.flow.movie_playing = true; // BGFMVPlaying = 1 [M 0x1081a8] — the decoder started
    for _ in 0..70 { fe.tick(); } // the 60-frame fade [M disasm 0x183054-60] + the wrap
    assert_eq!(fe.flow.get_state(), nf_view::menu::ST_MOVIE, "the outro is up");
    assert_eq!(fe.flow.movie_code, 0x0730_0004, "the outro movie code [M 0x1d6360:101-112]");
    assert_ne!(fe.page, nf_view::menu::Page::Results, "no page over the outro [M 0x1c98c8 case 6]");
    fe.flow.movie_playing = false; // psiMovieFinished [M 0x1082e8]
    fe.tick();
    assert_eq!(fe.flow.phase2_resolved.take(), Some(nf_view::menu::FRONTEND_LEVEL),
        "the outro resolves to the front-end map [M 0x1d3560: 0x0730004 → 0x7000048]");
    assert!(fe.flow.load_tick(), "phase 3 done");
    assert!(fe.reopen_queued_page(), "the load tail's MenuManager_Create [M 0x1d6360:160-163]");
    assert_eq!(fe.page, nf_view::menu::Page::Results, "the results page, after the outro");
    eprintln!("[04-outro] the kill led into the game's end sequence in {} walk ticks", w.total_ticks);
}

/// mission-results-page: the results page WORKS — the walked kill runs the real end path to the
/// results page, then Del opens the Mission Stats page with the mission's own tallies, Tab opens
/// the Dossier, Enter continues, and the score/medal/reward bits follow the tracked stats:
/// 1. the mission's tallies accumulate through the real fire path — shots [M PlrStat_LogShotFired
///    0x1998d8 ← Player_WeaponInitBullet 0x1a4bf0:258], the kill [M PlrStat_LogEnemyDispatched
///    0x1996a8 ← NDrone2_LogDroneDeath 0x14ab00:29], the elapsed clock every tick [M
///    PlarStat_LogUpdateElapsedTime 0x1995a0 ← Player_Update 0x193600:132];
/// 2. `demo_results` builds the front end from those tallies (nf_view::mission_end_stats — the
///    old `menu::Stats::default()` zeroed them) and commits the score through `results_enter`
///    [M P_NFRESULTS 0x208e78:77 GetScore(0) + :169 Menu_SetLevelBonus];
/// 3. Del → page 0x40000037 [M :225-230], whose 0x4c runs `PlrStat_GetScore(0)` [M P_NFSTATS
///    0x209938:36] — the rows carry the same tallies; Tab → 0x4000002b [M :222-224]; Enter →
///    the new-reward/mission-list routing [M 0x208e78 case 0x50's 0x44 sends + P_NFBONUS
///    0x209670 case 0x4c's DAT_002e1c2c text gate].
#[test]
fn the_results_page_opens_stats_and_dossier_and_scores_the_real_tallies() {
    let iso = iso();
    let mut w = load_ready(&iso);
    // The armament walk + the assassination — the outro test's route, the real input path.
    w.tick(Default::default(), false);
    w.tick(Default::default(), false);
    let (uzi, armour) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let uzi = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && !p.gone && (p.start[2] - 42.66).abs() < 1.0)
            .or_else(|| sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone))
            .map(|p| p.start).expect("the StormUzi pickup");
        let armour = sim.drones.pickups.iter().find(|p| p.kind == 3 && !p.gone).map(|p| p.start)
            .expect("the BodyArmourHeavy pickup");
        (uzi, armour)
    };
    w.walk_nav_to(uzi, "the StormUzi pickup", 0.4);
    w.walk_nav_to(armour, "the armour pickup", 0.4);
    let target_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_001e).expect("the target drone exists");
    let killed = w.shoot(target_i, "the target");
    assert!(killed || w.scene.sim.as_ref().unwrap().drones.list[target_i].corpse
        || w.scene.sim.as_ref().unwrap().drones.list[target_i].health <= 0.0, "the target must die");
    let mut raised = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x54) { raised = true; break; }
    }
    assert!(raised, "the kill must raise ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660]");
    let mut ended = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission_end.is_some() { ended = true; break; }
    }
    assert!(ended, "case 4 must fire the level end [M 0x182c60:65-76]");

    // ── demo_results' steps (nf-view lib.rs) against the LIVE sim ──
    // The tallies the mission leaves behind (the fire path's own shots + the kill + the clock):
    let stats = nf_view::mission_end_stats(&w.scene.sim.as_ref().unwrap().drones.progress);
    eprintln!("[04-results] the tracked stats: fired {} hit {} acc {}% time {}s moves {} dispatched {} subdued {} surrendered {} bonus {}",
        stats.shots_fired, stats.shots_hit, stats.accuracy_pct, stats.time_s, stats.bond_moves,
        stats.dispatched, stats.subdued, stats.surrendered, stats.bonus_pickups);
    assert!(stats.shots_fired > 0, "the walk's real shots logged [M 0x1998d8 ← 0x1a4bf0:258]");
    assert!(stats.shots_hit > 0, "the kill's hits logged [M 0x199840 ← NDrone2_HitDamage 0x145558:60]");
    assert!(stats.dispatched >= 1, "the kill dispatched [M 0x1996a8 ← 0x14ab00:29]");
    assert!(stats.time_s > 0, "the elapsed clock accumulated per tick [M 0x1995a0 ← 0x193600:132]");

    let mut fe = nf_view::menu::FrontEnd::new();
    fe.flow.current_level = 0x0700_0004;
    fe.flow.stack = vec![nf_view::menu::ST_RUNNER]; // the host's demo_results boots the runner state
    fe.mission_ended(nf_view::menu::Outcome::Complete, stats.clone());
    let (base, diff) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.drones.mission.base_map, sim.drones.progress.difficulty)
    };
    assert_eq!(base, 0x0700_0001, "Double Cross part 2's base map is the mission's first level [L the mission table; M Mission_Init 0x182270:52]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert!(fe.results_enter(&mut sim.drones.progress, base, 6, diff, None),
            "the results enter commits [M 0x208e78:77 GetScore(0)]");
        // The host's mission-end snapshot (the sim dies with the front-end load; the game's
        // globals are BSS [M 0x209938:36, 0x182270:65]).
        let mut snap = sim.drones.progress.clone();
        let idx = snap.table.iter().position(|e| e.hash == base).unwrap();
        let last = snap.table[idx].last;
        let th = snap.table[idx].thresholds;
        assert!(last > 0, "the commit landed a score [M 0x1984b0:309-315 last = total]");
        let expect_medal = if last >= th[3] { 4u32 } else if last >= th[2] { 3 }
            else if last >= th[1] { 2 } else if last >= th[0] { 1 } else { 5 };
        eprintln!("[04-results] committed score {last} vs thresholds {th:?} → medal {expect_medal}");
        // The outro plays, the load tail opens the page (the reward_after_outro walk).
        fe.flow.movie_playing = true;
        for _ in 0..70 { fe.tick(); }
        fe.flow.movie_playing = false;
        fe.tick();
        assert!(fe.flow.phase2_resolved.take().is_some());
        assert!(fe.flow.load_tick());
        assert!(fe.reopen_queued_page(), "the load tail's MenuManager_Create [M 0x1d6360:160-163]");
        assert_eq!(fe.page, nf_view::menu::Page::Results, "the results page is up");
        // The page's fields are the commit's: the score is entry+0x34 [M :96-97] and the medal
        // compare is the game's own threshold chain [M :98-142].
        let rf = fe.results.as_ref().expect("the fields built on the enter");
        assert_eq!(rf.score, last, "the score line is the committed last [M :96-97 SeparateNumber(+0x34)]");
        assert_eq!(rf.medal, expect_medal, "the medal follows the thresholds [M :98-142]");
        // The reward bits: the medal loop commits tiers medal..1 [M Menu_SetLevelBonus 0x1ffa10
        // :70-82 — the counter loop]; a first medal is a NEW reward [M :76-79 DAT_002e1c2c = 1].
        for tier in 1u32..=4 {
            let has = snap.has_medal(base, tier, 0);
            assert_eq!(has, tier <= expect_medal && expect_medal < 5,
                "reward bit tier {tier} follows the medal [M 0x1ffa10:70-82]");
        }
        assert_eq!(fe.new_reward, expect_medal < 5, "the first medal is a new reward [M 0x1ffa10:76-79]");

        // ── Del opens the Mission Stats page and it builds from the same tallies ──
        assert!(fe.action(nf_view::menu::Action::Extra),
            "Del opens the Stats page [M 0x208e78:225-230 msg 0x5e → 0x40000037]");
        assert_eq!(fe.page, nf_view::menu::Page::Stats);
        assert_eq!(fe.flow.page, nf_view::menu::PAGE_STATS, "the page id [M 0x40000037]");
        // The host's menu_after hook — the game's page-enter 0x4c build [M 0x209938:31-38].
        fe.results_subpages_enter(&mut snap, None);
        let rows = fe.mission_stats.as_ref().expect("the stats rows built [M 0x209938:31-38]");
        let find = |id: u32| rows.iter().find(|r| r.id == id).unwrap_or_else(|| panic!("row {id:#x} missing"));
        assert_eq!(find(0x15b).current, stats.dispatched.to_string(),
            "the Dispatched row reads the same tally [M 0x209938:60-62 ← GetScore's refill +0x0C]");
        assert_eq!(find(0x165).current, stats.bond_moves.to_string(),
            "the Bond Moves row [M 0x209938:57-59 ← +0x20]");
        assert_eq!(find(0x15e).current, format!("{}%", stats.accuracy_pct),
            "the Accuracy row's pct is the row-5 ratio [M 0x209938:100-102]");
        assert!(find(0x160).current != "00:00", "the Time row carries the elapsed clock [M 0x209938:103-108 ← +0x30]");
        // Back to the results.
        assert!(fe.action(nf_view::menu::Action::Back));
        assert_eq!(fe.page, nf_view::menu::Page::Results, "Back returns to the results");

        // ── Tab opens the Dossier ──
        assert!(fe.action(nf_view::menu::Action::Side),
            "Tab opens the Dossier [M 0x208e78:222-224 msg 0x5d → 0x4000002b]");
        assert_eq!(fe.page, nf_view::menu::Page::Dossier);
        assert_eq!(fe.flow.page, nf_view::menu::PAGE_DOSSIER, "the page id [M 0x4000002b]");
        assert!(fe.action(nf_view::menu::Action::Back));
        assert_eq!(fe.page, nf_view::menu::Page::Results, "Back returns (the deep navigation is menu-dossier-navigation's)");

        // ── Enter continues: the new-reward routing, then the mission list ──
        assert!(fe.action(nf_view::menu::Action::Confirm), "Enter acts [M 0x208e78 case 0x50]");
        if fe.new_reward {
            assert_eq!(fe.page, nf_view::menu::Page::NfBonus,
                "a new reward routes to the 007 Bonus page [M 0x208e78 case 0x50's 0x44 → 0x40000038]");
            // The host's hook — P_NFBONUS's 0x4c: the tier text only when DAT_002e1c2c [M 0x209670:16-18].
            fe.results_subpages_enter(&mut snap, None);
            let bonus = fe.bonus.as_ref().expect("the bonus fields built [M 0x209670:14-18]");
            assert!(bonus.has_text && !bonus.text_labels.is_empty(),
                "the tier text composed for the new reward [M 0x209670:22-41]");
            for (i, l) in bonus.lines.iter().enumerate() {
                assert_eq!(l.has, (i as u32 + 1) <= expect_medal, "tier {} line follows the medal [M Menu_GetLevelBonuses]", i + 1);
            }
            assert!(fe.action(nf_view::menu::Action::Confirm), "Enter on the bonus continues [M 0x209670 case 0x4b]");
        }
        assert_eq!(fe.page, nf_view::menu::Page::MissionSelect,
            "the results flow continues to the mission list [M 0x208e78:40-43 / 0x209670:50-53 just_completed == 0 → 0x4000001c]");
        assert_eq!(fe.flow.page, nf_view::menu::PAGE_MISSION_SELECT);
    }
}

/// mission-stats-rows: EVERY Mission Stats cell of the real 07000004 finish — value, target and
/// points, each computed inline with the game's own formulas and pinned against the rows the
/// page builds — plus the Dossier Records row the finish writes. The walk is the same real-input
/// route (the pickups, the kill, the end sequence) as the results test above.
///
/// Row sources [M PlrStat_GetScore 0x1984b0:66-84]; formulas [M :118-268]; formats [M
/// Menu_AddRow 0x201c10 / Menu_AddRowPercentage 0x201df0 / SeparateNumber 0x1dec78 /
/// Timer_Seconds2String 0x1deaa8]. Records reader [M C_RBDSRECORDS 0x20bf00:19-44].
#[test]
fn every_stats_row_matches_the_game_formulas_from_the_real_finish() {
    let iso = iso();
    let mut w = load_ready(&iso);
    w.tick(Default::default(), false);
    w.tick(Default::default(), false);
    let (uzi, armour) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let uzi = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && !p.gone && (p.start[2] - 42.66).abs() < 1.0)
            .or_else(|| sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone))
            .map(|p| p.start).expect("the StormUzi pickup");
        let armour = sim.drones.pickups.iter().find(|p| p.kind == 3 && !p.gone).map(|p| p.start)
            .expect("the BodyArmourHeavy pickup");
        (uzi, armour)
    };
    w.walk_nav_to(uzi, "the StormUzi pickup", 0.4);
    w.walk_nav_to(armour, "the armour pickup", 0.4);
    let target_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_001e).expect("the target drone exists");
    let killed = w.shoot(target_i, "the target");
    assert!(killed || w.scene.sim.as_ref().unwrap().drones.list[target_i].corpse
        || w.scene.sim.as_ref().unwrap().drones.list[target_i].health <= 0.0, "the target must die");
    let mut ended = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission_end.is_some() { ended = true; break; }
    }
    assert!(ended, "case 4 must fire the level end [M 0x182c60:65-76]");

    let (base, diff) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.drones.mission.base_map, sim.drones.progress.difficulty)
    };
    assert_eq!(base, 0x0700_0001, "part 4 scores under the Double Cross base [L mission table 0x2a4350 row 7; M Mission_Init 0x182270:52]");

    // The results enter's commit (the game's results page 0x4c [M 0x208e78:77]) — the SAME
    // progress object the stats page then re-commits against (P_NFSTATS 0x209938:36).
    let mut fe = nf_view::menu::FrontEnd::new();
    fe.flow.current_level = 0x0700_0004;
    fe.flow.stack = vec![nf_view::menu::ST_RUNNER];
    fe.mission_ended(nf_view::menu::Outcome::Complete, nf_view::menu::Stats::default());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert!(fe.results_enter(&mut sim.drones.progress, base, 6, diff, None),
            "the results commit [M 0x208e78:77]");
        // mission-stats-rows: the host's session layer (lib.rs end_mission) — the game's BSS
        // table the Records page reads with no save in play [M C_RBDSRECORDS 0x20bf00:19-26].
        fe.session_progress = Some(sim.drones.progress.clone());
    }
    let snap = fe.session_progress.clone().unwrap();
    let idx = snap.table.iter().position(|e| e.hash == base).unwrap();
    let e = snap.table[idx];
    let s0 = &snap.stats[0]; // player 0's PlrMissionStats [M 0x2bcc20's port]

    // The game's own cell build [M 0x209938:57-123 through Menu_AddRow 0x201c10]:
    // ceil₁ₑ₋₅(f) = (int)f + (|f − (int)f| ≥ 1e-5) [M 0x1984b0:178-183];
    // pct(f) = round½↑(f × 100) [M 0x209938:92/100/106].
    let ceil_1e5 = |f: f32| { let i = f as i64; (i as f32 + ((f - i as f32).abs() >= 1e-5) as i64 as f32) as u32 };
    let pct = |f: f32| format!("{}%", (f * 100.0 + 0.5) as u32);
    let sep = nf_game::stats::separate_number;
    let timer = nf_game::stats::timer_string;

    // The tallies → each row's value [M 0x1984b0:66-84]; the static counts [L ee_start_05 entry 4].
    let (bond, disp, sub, sur, det) = (s0.bond_moments, s0.dispatched, s0.disabled, s0.surrendered, s0.detected);
    let (fired, hit) = (s0.shots_fired, s0.shots_hit_enemy);
    let acc = if fired > 0 { e.rows[5].count as u64 * hit as u64 / fired as u64 } else { e.rows[5].count as u64 };
    let health = s0.health as u32; // fptoui [M row 6]
    let elapsed = s0.elapsed;
    eprintln!("[04-rows] bond {bond} disp {disp} sub {sub} sur {sur} det {det} fired {fired} hit {hit} acc {acc} hp {health} elapsed {elapsed} diff {diff}");

    // The per-row points [M 0x1984b0:118-268], on the entry's own design data.
    let raw = [
        ceil_1e5(e.rows[0].target as f32 * bond.min(e.rows[0].count) as f32 / e.rows[0].count as f32), // 0
        ceil_1e5(e.rows[1].target as f32 * disp.min(e.rows[1].count) as f32 / e.rows[1].count as f32), // 1
        ceil_1e5(e.rows[2].target as f32 * sub.min(e.rows[2].count) as f32 / e.rows[2].count as f32), // 2
        ceil_1e5(e.rows[3].target as f32 * sur.min(e.rows[3].count) as f32 / e.rows[3].count as f32), // 3
        if det > e.rows[4].count { // 4: the penalty ramp [M :201-224]
            let pen = if e.penalty == 0.0 { 0.03 } else { e.penalty };
            ceil_1e5(((1.0 - (det - e.rows[4].count) as f32 * pen).max(0.0)) * e.rows[4].target as f32)
        } else { e.rows[4].target }, // within par: the full max [M :225-233]
        ceil_1e5(e.rows[5].target as f32 * acc.min(e.rows[5].count as u64) as f32 / e.rows[5].count as f32), // 5
        ceil_1e5(e.rows[6].target as f32 * health.min(e.rows[6].count) as f32 / e.rows[6].count as f32), // 6
        ceil_1e5(e.rows[7].target as f32 * e.rows[7].count as f32 / (elapsed / 100).max(1) as f32), // 7: INTEGER seconds [M :120-131]
        if e.rows[8].count <= s0.bonus_pickups { e.rows[8].target } else { 0 }, // 8: all-or-nothing [M :255-268]
    ];
    let base_u: f32 = raw.iter().map(|r| *r as f32).sum(); // the f32 accumulate [M :288-293]
    let base_score = base_u as u32;
    let mult = match diff { 1 => 0.0f32, 2 => 1.0, _ => 2.0 }; // ScoringTableDifMul [M :294-300]
    let bonus = (base_u * mult) as u32;
    let total = base_score + bonus; // [M :309-315]

    let rows = nf_game::stats::stats_rows(&mut w.scene.sim.as_mut().unwrap().drones.progress, base, 6, diff, None)
        .expect("the Double Cross base is a ScoringTable hash");
    let find = |id: u32| rows.iter().find(|r| r.id == id).unwrap_or_else(|| panic!("row {id:#x} missing"));

    // Row by row — current, target, TOTAL [M 0x209938:57-123].
    assert_eq!(find(0x165).current, bond.to_string(), "Bond value [M :66 ← +0x20]");
    assert_eq!(find(0x165).target, e.rows[0].count.to_string(), "Bond target = the static count [L entry 4 row 0]");
    assert_eq!(find(0x165).total, sep(raw[0]), "Bond points [M 0x201c10:99-101]");
    assert_eq!(find(0x15b).current, disp.to_string(), "Dispatched value [M :68]");
    assert_eq!(find(0x15b).target, "45", "Dispatched target = the design 45 [L entry 4 row 1 — the game's own mission-wide value]");
    assert_eq!(find(0x15b).total, sep(raw[1]), "Dispatched points [M :171-185]");
    assert_eq!(find(0x15c).current, sub.to_string(), "Subdued value [M :70]");
    assert_eq!(find(0x15c).target, e.rows[2].count.to_string(),
        "Subdued target = the design count 3 [L entry 4 row 2; M 0x209938:77 — the builder passes rows+0x30 (row2.count) as the target arg, same shape as Dispatched]");
    assert_eq!(find(0x15c).total, sep(raw[2]), "Subdued points = fptoui(row2.raw) [M 0x209938:77-79]");
    assert_eq!(find(0x16e).current, sur.to_string(), "Surrendered value [M :72]");
    assert_eq!(find(0x16e).target, e.rows[3].count.to_string(),
        "Surrendered target = the design count 3 [L entry 4 row 3; M 0x209938:82 — rows+0x44 (row3.count) as the target arg]");
    assert_eq!(find(0x16e).total, sep(raw[3]), "Surrendered points = fptoui(row3.raw) [M 0x209938:82-84]");
    // Sister: the Subdued/Surrendered/Stealth rows are CONDITIONAL on the entry's new_scores
    // flag (0 for the 0x09 maps) [M 0x209938:74-86 `if (*(char *)(score_table + 0x38) != 0)`;
    // the flag GetScore set at 0x1984b0:38-43] — 07000001 sets it, so all three show.
    assert!(rows.iter().any(|r| r.id == 0x15c) && rows.iter().any(|r| r.id == 0x16e) && rows.iter().any(|r| r.id == 0x15d),
        "the SP flag shows the Subdued/Surrendered/Stealth rows [M 0x209938:74]");
    assert_eq!(find(0x15d).current, pct(e.rows[4].ratio), "Stealth pct = fptoui(ratio·100+0.5) [M :84]");
    assert_eq!(find(0x15d).target, "----", "Stealth's −1 target [M 0x201df0:78-88]");
    assert_eq!(find(0x15d).total, sep(raw[4]), "Stealth points [M :201-233]");
    assert_eq!(find(0x15e).current, format!("{}%", acc), "Accuracy = hit·par/fired [M :74-78]");
    assert_eq!(find(0x15e).target, "100%", "the accuracy par [L entry 4 row 5]");
    assert_eq!(find(0x15e).total, sep(raw[5]), "Accuracy points [M :171-185]");
    assert_eq!(find(0x15f).current, pct(e.rows[6].ratio), "Health pct = fptoui(ratio·100+0.5) [M :92]");
    assert_eq!(find(0x15f).target, "100%", "the literal [M :105-108]");
    assert_eq!(find(0x15f).total, sep(raw[6]), "Health points [M :171-185]");
    assert_eq!(find(0x160).current, timer(elapsed as i64), "Time current [M :103-108]");
    assert_eq!(find(0x160).target, "08:00", "the 480 s par [L entry 4 row 7]");
    assert_eq!(find(0x160).total, sep(raw[7]), "Time points = max·par/max(int seconds, 1) [M :118-170]");
    assert_eq!(find(0x161).current, s0.bonus_pickups.to_string(), "Bonus value [M :110 ← +0x24]");
    assert_eq!(find(0x161).target, "7", "Bonus target [L entry 4 row 8]");
    assert_eq!(find(0x161).total, sep(raw[8]), "Bonus points [M :255-268]");
    // The tail rows [M :117-123] and the commit [M :301-315].
    assert_eq!(find(0x162).total, sep(base_score), "Subtotal = Σ raw (f32 accumulate, trunc) [M :288-293]");
    assert_eq!(find(0x164).total, sep(bonus), "Difficulty Bonus = base × the multiplier [M :294-300]");
    assert_eq!(find(0x163).total, sep(total), "TOTAL = base + bonus [M :309-315]");
    assert_eq!(e.last, total, "the results commit stamped last [M :309-315]");
    assert_eq!(e.best, total, "best = max(best, last) — the first finish stamps best [M :312-315]");

    // ── The Dossier Records row (C_RBDSRECORDS 0x20bf00:19-44): the finished mission's values —
    // the session layer (no profile), then the same through a profile (the save).
    assert_eq!(fe.record_row(4), (total, bond as u32,
        (1..=4u32).filter(|&k| snap.has_medal(base, k, 0)).count() as u32),
        "High Score = entry+0x30, Bond Moves = rows[0].value, Upgrades = the medal census [M 0x20bf00:26-44]");
    let medal = if total >= e.thresholds[3] { 4u32 } else if total >= e.thresholds[2] { 3 }
        else if total >= e.thresholds[1] { 2 } else if total >= e.thresholds[0] { 1 } else { 5 };
    let want = match medal { 4 => Some((0x307, 0x0300_013e)), 3 => Some((0x306, 0x0300_013f)),
        2 => Some((0x308, 0x0300_0140)), 1 => Some((0x309, 0x0300_013d)), _ => None };
    assert_eq!(fe.record_medal(4), want, "the medal chain follows the four bonus flags [M 0x20bf00:58-110]");
    // The profile path: the save's restored table reads the same [M PlrStats_SetupScoreTable 0x1990e0].
    let mut prof = nf_game::profile::Profile::default_codename("STATSROWS");
    prof.progress = snap.clone();
    fe.profile = Some(prof);
    assert_eq!(fe.record_row(4).0, total, "the profile's restored table carries the same commit [M 0x1990e0]");
    fe.profile = None;
    fe.session_progress = None;
    assert_eq!(fe.record_row(4), (0, 0, 0), "no save and no session → the fresh-boot zeros [M 0x1984b0's zeroed table]");
}
