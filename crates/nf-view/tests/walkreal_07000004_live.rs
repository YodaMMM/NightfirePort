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
