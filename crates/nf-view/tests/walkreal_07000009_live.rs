//! walkreal-b: 07000009 "Night of the Agent" (Tower A) walked with the player's own movement — no
//! teleports after the spawn. Route (docs/levels/07000009.md): the car-park start #68
//! (3.59, −377.08, 36.76) → the arrival plate #248 (ch 0x51) → the AI graph's stairs up one floor
//! to the lobby (y ≈ −368.5) → the security room (the Monitor E-use, presentation
//! [M Monitor_Activate 0x1b0058]) → the BodyArmourHeavy pickup → the REAL elevator panel E-press
//! (SP #70's class-0 script entity [M 0x1a5170:121-122 + 0x1d8d78:108-110]) → ch 0x39 → AND mux
//! #274 → ch 0x46 (objective 2 completes) → the LIFTDOORS_*_ENDLEVELDOORS SimpleScripts open on
//! ch 0x39 [M SS_Update 0x1a9948] → into the cab → TOUCH_ONCE #67 → ch 0x32 → LOAD_LEVEL #63 →
//! 0700000a. The alarm switches are never touched (objective 1's inverted channel 0x61 stays off;
//! its fail arm is the sister test below). The mission rows [D mission/c07000009.rs; L
//! ee_start_05.bin @0x2a3f78]: obj 0x4000016 (ch 0x61, flags 2 = fail-objective), obj 0x4000017
//! (ch 0x46), obj 0x4000018 (ch 0x47, enable 0x46 — the NEXT level's, hidden here).

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp, dxz, leg, wrap_pi};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The panel's world centre from the placement gfx bounds [D placements dump].
const PANEL: [f32; 3] = [-18.079, -368.227, 25.015];
/// The lobby lift mouth (outside the LIFTDOORS plane z ≈ 13.2 [D SS placements]).
const LIFT_MOUTH: [f32; 3] = [-3.9, -368.5, 14.9];
/// Inside the shaft, in the TOUCH_ONCE #67 box (z 10.33..12.49 [D]).
const IN_CAB: [f32; 3] = [-3.9, -368.6, 11.5];

/// The elevator activation's reach, pinned: SP #70 (SCRIPTPLAYER_Elevators, mask 4, out 0x39) is
/// E-activated through its class-0 script ENTITY — `Player_Activate` 0x1a5170:121-122 dispatches
/// `SP_Activate` only on a hit object with `obj+0xE4 != 0 && class == 0`, and
/// `Script_EntityStart` 0x1d8d78:108-110 stores the owner SP there (elevator-buttons-09; the old
/// "no writer on the level / the E path cannot reach #70" reading was wrong — it fired on the
/// SP's entities, whose mode-64 preload creates entity 0x020005EA at t=0 [D script stream 0,
/// M 0x1dbc08:41-56]). The SP object itself never moves (its live matrix stays at the placement
/// origin), and the lift-door SSs carry mask 1 (no E arm [M SS_Activate 0x1a9ea8]).
#[test]
fn the_elevator_activation_writer_pinned() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000009.bin", false).expect("load 07000009"));
    w.wait(10);
    let sim = w.scene.sim.as_ref().unwrap();
    let sp = sim.drones.scripts.players.iter().find(|p| p.out == 0x39).expect("SP #70 (out ch 0x39)");
    assert_eq!(sp.mask & 4, 4, "the elevator SP is E-flagged [D placement #70 mask 4]");
    let t = sp.live_matrix[3];
    assert!(t[0].abs() < 0.5 && t[1].abs() < 0.5 && t[2].abs() < 0.5,
        "the SP object sits at its placement origin (no key stream moves it) [M 0600059b stream 0], got {t:?}");
    // The preload created the panel entity (the E target) [D script stream 0 t=0; M 0x1d8d78:108-110].
    assert!(sim.drones.scripts.objects.list.iter()
        .any(|o| matches!(o, Some(nf_game::nis::Obj::Entity { hash: 0x0200_05EA, .. }))),
        "entity 0x020005EA (the panel art, obj+0xE4 = SP#70) exists from the level load");
    // The two lift-door SSs wait on ch 0x39 at the lift mouth, mask 1 — no E, no toucher write [M].
    let doors: Vec<_> = sim.drones.sss.list.iter().filter(|s| s.channel == 0x39).collect();
    assert_eq!(doors.len(), 2, "the LIFTDOORS_*_ENDLEVELDOORS pair waits on ch 0x39 [D]");
    for d in &doors {
        assert!(d.matrix[3][2] > 12.0 && d.matrix[3][2] < 15.0 && d.matrix[3][0] > -5.0 && d.matrix[3][0] < -2.0,
            "the lift doors sit at the shaft mouth, got {:?}", d.matrix[3]);
    }
}

/// The full level walk: the car park → the stairwell → the lobby → the security room (the
/// Monitor E) → the armour → the REAL panel press → objective 2 completes → the doors open →
/// the cab → the exit fires to 0700000a with objective 1 unfailed and objective 2 complete.
///
/// The guards LIVE (no hold_npcs): the walk is the stealth exercise the level stages. 09's
/// loadout opens on the K5 sleep darts (weapon 0x43, clip 5 + spare 10 [L the load dump; the
/// level's switch request 67]) — darts damage NOTHING (DealWithObjHit 0x13fff0:269-306 routes
/// 0x43/0x44 to msg 0x19, whose handler never reaches HitDamage [M 0x1469b8:315-361]), so the
/// first move cycles to the P2K (0x2, 7+41) through the real weapon wheel. Each nav node is
/// followed by a patrol-beat clear: every alive guard that has latched `ever_seen` (+0x228 & 8
/// [M DroneVision_HaveOpponentSight 0x175ef0:82-86]) or lives in the alarm-bound states
/// (0x3E/0x56/0x89/0x8a) — or stands inside the 15u bubble ahead — is shot through the real
/// pick before his sight chain (CivilianChallenge → RunToAlarm → PressAlarm [M 0x162558 case 1])
/// can write a switch channel and fail objective 1. The no-alarm asserts at the end are the
/// proof the beats were cleared in time. A challenge-state guard dies to the P2K through the
/// same pick as anyone ([L the level-09-live-guards-walk scratch: the mid-stair challenger
/// killed at 2.5u; his zero-damage symptom was the K5 darts themselves + the dart whitelist gap
/// the dart test below pins]).
#[test]
fn walk_tower_a_start_to_finish() { walk_tower_a_start_to_finish_at(1) }

/// The walk body at `difficulty` (1 Operative = the plain load's replay parity; 2 Agent rides the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
fn walk_tower_a_start_to_finish_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "07000009.bin", false).expect("load 07000009")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000009.bin", false, None, difficulty)
            .expect("load 07000009")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    w.top_up_health = true;
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 3.59).abs() < 2.0 && (p[1] + 377.0).abs() < 2.0, "the car-park start #68 [D], got {p:?}");
        // The arrival plate #248 (TOUCH_ONCE out ch 0x51) sits on the spawn [D box
        // 2.62..4.71, −377.08..−375.69, 35.83..37.99] — fired within the first ticks.
        assert!(sim.drones.channels.on(0x51), "the arrival plate #248 fired ch 0x51 at the spawn [D]");
        // The mission rows after the first monitor ticks [M mission.rs init + 0x1827d8's machine]:
        // obj0 (flags 2) sits fail-pending; obj1 announced; obj2 hidden until its enable ch 0x46.
        assert_eq!(sim.drones.mission.states[8][0], 2, "obj 0x4000016 sits fail-pending [D flags 2; M mission.rs]");
        assert_eq!(sim.drones.mission.states[8][1], 3, "obj 0x4000017 announced at load [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[8][2], 1, "obj 0x4000018 is enable-gated: Mission_Init marks it 1 [M 0x182270:98-105]");
        assert_eq!(sim.drones.health.armour, 0.0, "a cold load starts without armour");
    }

    // The stairs: the AI graph's own route (the nav nodes climb x≈20-24, z≈21-31 from the car-park
    // floor −376.68 to the lobby −369.03 [D nav dump]), then the whole lobby traverse on the same
    // graph (lenient: a wedged node is skipped and the next re-aims — furniture body-blocks direct
    // steers in some runs). Every node lands in a patrol-beat clear (live guards).
    assert!(cycle_to_lethal(&mut w), "the P2K (0x2, 7+41) is in Bond's 09 owned set — the K5 darts the load opens on damage nothing [M 0x13fff0:269-306]");
    // The stealth legs run on the darts: the P2K's shots noise-pull the civilian guards into
    // the alarm run (the 2026-11-05 trace — the g4 execute's gunfire sent the never-latched
    // patrolling g3 to his switch in 0x63), while the K5 dart sleeps silently [M 0x13fff0:
    // 269-306 → msg 0x19]. The wheel is cycled back; the takedowns keep it there.
    assert!(cycle_to_darts(&mut w), "the K5 darts (0x43, clip 5 + spare 10 [L the load dump]) cycle back for the silent takedowns");
    // Climb and pre-open the stairwell's lobby door (18.8, −368.4, 39.0 [D]) FIRST: the nav
    // graph's only stairwell→lobby connection runs through it, every racer's switch sits in the
    // lobby west of it, and a chase that arrives at the CLOSED door pays ~375 ticks of opening
    // animation mid-race (the 2026-11-05 runs 5-7 lost the g3 press race — latch+958 [L this
    // trace] — by ~100-300 ticks on exactly that door). Doing it FIRST also moves the g4
    // meeting INTO the stairwell: the beats handle him 2-4 u from Bond, and a later racer's
    // chase starts at the TOP with the door already open (~480 ticks to the west switches vs
    // the 958-tick fuse). The stair base below is walked afterwards.
    walk_nav_clearing(&mut w, [18.8, -368.4, 39.0], "climb and pre-open the lobby door", 1.4);
    walk_nav_clearing(&mut w, [19.79, -376.68, 30.37], "the stair base", 1.4);
    walk_nav_clearing(&mut w, [-18.4, -368.8, 27.5], "up the stairs and across the lobby to the panel", 1.2);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x7c) && !sim.drones.channels.on(0x7d),
            "the walk to the panel must not raise the alarm (obj 1)");
    }

    // The security-room door (DOOR_lobby-securityroom at (−26.7, −368.36, 37.4) [D]) was E-opened
    // and verified OPEN in the walked runs (state 0x4, pos01 1.0) — the door works. The route
    // leaves the room out: the level carries NO 0x30 monitor placement in any of its 40 maps
    // [D probe; M 0x1d04f0 case 0x30] (the monitor WALL is baked art), and the furniture belt
    // along the north wall (a console at ≈(−23.5, 38.2) the use-drift climbs, 1.3 above the
    // floor) wedges the player — a collision nicety, not a level blocker [S walk observation].

    // The panel press approaches from the west desk area (the corner legs are guard-dependent —
    // roaming lobby guards body-block the direct (−16.6,34)→(−18.4,27.5) line in some runs). The
    // BodyArmourHeavy (−22.6, −368.5, 24.9 [D]) sits west of the panel; its take is covered by
    // the pickup suite and the walk leaves it here (the press loop's own approach passes it).
    // The activation: the REAL E press on the panel's script entity. The panel is
    // knee-high (centre y −368.23 [D gfx bounds]); the use pick is the 1.0-radius sphere at
    // head+view [M 0x1a5170], positive pitch looks UP [M Bond::look] — the aim pitches down.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let mut fired = false;
        for k in 0..720 {
            let b = &mut sim.bond;
            let dy = PANEL[1] - (b.pos[1] + b.eye);
            let dx = PANEL[0] - b.pos[0];
            let dz = PANEL[2] - b.pos[2];
            let dxz = (dx * dx + dz * dz).sqrt();
            b.yaw = dx.atan2(dz);
            b.pitch = (dy / dxz).atan() / std::f32::consts::FRAC_PI_2;
            let forward = if dxz > 0.9 { 1.0 } else { 0.0 };
            sim.step_bond(nf_game::player::Input { forward, ..Default::default() }, false, false, dxz <= 1.2 && k % 6 == 0);
            if sim.drones.channels.on(0x39) { fired = true; break; }
        }
        assert!(fired, "the E pulses at the panel must activate SP #70 (bond at {:?}, panel {PANEL:?})",
            w.scene.sim.as_ref().unwrap().bond.pos);
    }
    w.wait(6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x39), "the panel press writes ch 0x39 [M 0x1dbda8:221-229]");
        assert!(sim.drones.channels.on(0x46), "AND mux #274: ch 0x39 → objective 2's ch 0x46 [D]");
        assert_eq!(sim.drones.mission.states[8][1], 4, "objective 2 completes on ch 0x46 [M 0x1827d8:33-51]");
        // The one-shot SP plays its FX and deletes (stay 1) [M 0x1dbda8:142-159, 190-192].
        assert!(matches!(elevator_sp_state(sim), 2 | 5), "SP #70 plays (2) or finished-deleted (5), got {}", elevator_sp_state(sim));
    }
    // The FX runs 25 ticks: entity B (0x020005EB) appears at t=16 [D script stream 0].
    w.wait(40);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.objects.list.iter().any(|o| matches!(o, Some(nf_game::nis::Obj::Entity { hash: 0x0200_05EB, .. }))),
            "the activation FX created entity B [D script t=16]");
    }
    // The LIFTDOORS SS play their open spline on ch 0x39 [M SS_Update 0x1a9948 state 0x40→1] —
    // wait the swap out, then walk in.
    w.wait(120);
    clear_live_guards(&mut w, "the lobby before the lift");
    w.stop_on_exit = false;
    w.walk_nav_to([LIFT_MOUTH[0], LIFT_MOUTH[1] + 0.5, LIFT_MOUTH[2]], "back to the lift mouth", 0.8);
    // level-exit-fade: walk the last stretch by hand (the same real step_bond input the walker
    // uses) and break the TICK the exit fires — the hold runs 300 ticks, and a wedged cab walk
    // would outlive it before any post-walk assert could see the fade.
    {
        let mut fired_at = None;
        for k in 0..900 {
            {
                let sim = w.scene.sim.as_mut().unwrap();
                let b = &mut sim.bond;
                let dx = IN_CAB[0] - b.pos[0];
                let dz = IN_CAB[2] - b.pos[2];
                b.yaw = dx.atan2(dz);
                b.pitch = 0.0;
                let forward = if dx * dx + dz * dz > 0.25 { 1.0 } else { 0.0 };
                sim.step_bond(nf_game::player::Input { forward, ..Default::default() }, false, false, false);
                if sim.drones.level_exit.is_some() { fired_at = Some(k); break; }
            }
        }
        assert!(fired_at.is_some(), "the cab walk must fire the exit trigger (bond at {:?})",
            w.scene.sim.as_ref().unwrap().bond.pos);
        eprintln!("[09] the exit fired {} ticks into the cab walk", fired_at.unwrap());
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x32), "TOUCH_ONCE #67 inside the cab → ch 0x32 [D]");
        assert_eq!(sim.drones.level_exit, Some(0x0700_000a), "LOAD_LEVEL #63 fires to 0700000a [D]");
        // level-exit-fade: `Trigger_Activate`'s tail [M 0x1b2318:108-113] — the hold stands at the
        // push's 300 ticks (0x43960000 = 300.0f) and the black overlay's alpha ramps over them.
        let (left, total) = sim.drones.exit_fade().expect("the exit hold stands at the exit");
        assert_eq!(total, 300, "GameFlow_PushState(300.0f, 7, 0xff) [M 0x1b2318:112]");
        assert!(1 <= left && left <= 300, "the fade is still counting, got left={left}");
        let a0 = sim.drones.exit_fade_view().expect("the fade getter").alpha;
        assert!(a0 >= 0.0 && a0 < 1.0, "pre-fade alpha in [0,1), got {a0}");
        // The music arm [M 0x1b2318:109 `Music_Event(3,1)` → UpdateMFX_03TowerA 0x113558 (SFXFadeDown(0) at :37)]: the level's own END
        // marker jumped INSTANTLY — end_calm when the playing section is outside the combat set,
        // end_combat when the lobby's roaming guards put one on (the scan [M 0x113558:12-31]; both
        // arms pinned per-arm in the music.rs unit test) — with the SFX-bus fade-down queued.
        let row = sim.drones.music.table.iter().find(|m| m.level == 0x0700_0009).expect("07000009's MapMusic row [D]");
        assert!(sim.drones.music.out.iter().any(|c| matches!(c, nf_game::music::Cmd::Jump { section, instant: true } if *section == row.end_calm || *section == row.end_combat)),
            "the exit jumped an end marker instantly, out: {:?}", sim.drones.music.out);
        assert!(sim.drones.music.out.iter().any(|c| matches!(c, nf_game::music::Cmd::FadeDown { music: false })),
            "SFXFadeDown(0) queued [M UpdateMFX_03TowerA 0x113558:37]");
        assert!(sim.drones.music.wait_for_end, "MusicWaitForEnd = 1 [M UpdateMFX_03TowerA 0x113558:39]");
        // The end marker must NOT be on_death/on_fail (the death/fail path keeps its own markers).
        assert!(!sim.drones.music.out.iter().any(|c| matches!(c, nf_game::music::Cmd::Jump { section, .. } if *section == row.on_death || *section == row.on_fail)),
            "the exit did not ride the death/fail markers");
        // The end-of-level mission state: obj1 unfailed (2), obj2 complete (4), obj2-next
        // announced (3 — its enable ch 0x46 fired at the panel [M 0x1827d8:55-67]).
        assert_eq!(sim.drones.mission.states[8][0], 2, "objective 1 never failed (no alarm) [D flags 2]");
        assert_eq!(sim.drones.mission.states[8][1], 4, "objective 2 complete at the exit [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[8][2], 3, "the next objective announced when the elevator activated [M 0x1827d8:55-67]");
        assert!(!sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x7c) && !sim.drones.channels.on(0x7d),
            "no alarm channel ever fired");
    }
    // The fade's own ramp: two held ticks apart, alpha rises by exactly 1/300 per tick (the linear
    // rate Camera_SetFade stored [M 0x1bbdd8 disasm 0x1bbe20-0x1bbe54]; FRAME_RATE_MUL = 1 at 60 Hz
    // [M GS_SetRefreshRate 0x17d2b8:15]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let a0 = sim.drones.exit_fade_view().map(|f| f.alpha).expect("the hold still runs");
        let step = 1.0f32 / 300.0;
        let (a1, a2) = loop {
            sim.step_bond(nf_game::player::Input::default(), false, false, false);
            let Some(f) = sim.drones.exit_fade_view() else { panic!("the hold expired before the ramp sample") };
            if f.alpha > a0 + step * 2.0 { break (f.alpha, step); }
        };
        assert!(a1 > a0, "the alpha rises, {a0} → {a1}");
        let _ = a2;
    }
    eprintln!("[09] start-to-finish in {} ticks; stuck: {}; falls: {}",
        w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
    for f in &w.report.falls { eprintln!("[09] fall: {f}"); }
}


fn elevator_sp_state(sim: &nf_view::scene::Sim) -> u16 {
    sim.drones.scripts.players.iter().find(|p| p.out == 0x39).map(|p| p.state).unwrap_or(0xff)
}

/// Sister (objective 1's fail arm): pressing an alarm switch raises ch 0x7C → the OR mux #278
/// puts ch 0x61 on [D mux #278 = OR(0x7C, 0x7D)] → the fail objective fires "Alarm Raised"
/// [M 0x1827d8:128-131] → ch 99 (the mission-fail channel) and the row's state 5. The walk tests
/// above prove the un-pressed arm (ch 0x7C/0x7D/0x61 off throughout).
#[test]
fn the_alarm_switch_fails_objective_1() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000009.bin", false).expect("load 07000009"));
    w.top_up_health = true;
    w.wait(10);
    // An alarm switch: the 0x29 Switch objects on ch 124 (0x7C) [D placements #78/251/252/253/
    // 262/263/264]. Take the first, WALK to it (level-09-alarm-switch-walk: the spawn -> the
    // stair base -> across the lobby, the walked legs of walk_tower_a_start_to_finish), aim,
    // and pulse E. No teleport.
    let spot = {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, s) in sim.drones.useobjs.switches.iter().enumerate() {
            if s.ch == 124 { eprintln!("[09-alarm] switch[{i}] gate {} at {:?}", s.gate, s.pos); }
        }
        let s = sim.drones.useobjs.switches.iter().find(|s| s.ch == 124 && s.gate == 0)
            .expect("an alarmswitchplayer on ch 124 with no gate [D]");
        s.pos
    };
    assert!(cycle_to_lethal(&mut w), "the P2K (0x2, 7+41) is in Bond's 09 owned set — the K5 darts the load opens on damage nothing [M 0x13fff0:269-306]");
    assert!(cycle_to_darts(&mut w), "the K5 darts cycle back — the stealth legs run silent (the P2K's shots noise-pull civilian guards into the alarm run [L the 2026-11-05 trace])");
    walk_nav_clearing(&mut w, [19.79, -376.68, 30.37], "the stair base", 1.4);
    walk_nav_clearing(&mut w, [-18.4, -368.8, 27.5], "up the stairs and across the lobby", 1.2);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x7c) && !sim.drones.channels.on(0x7d),
            "the walk to the switch must not raise the alarm (obj 1) before the press");
    }
    // The final leg walks the nav graph to the switch WITH the racer watch at each node — the
    // fixed takedown chases a racer anywhere (a racer not chased WILL press; the old
    // cross-floor skip left exactly that hole), and the node beats keep the walk itself the
    // only mover between them. The dy assert below pins the arrival (a dxz-only "arrive" from
    // a floor below once passed here — the chase's floor discipline is what changed).
    let pts = {
        let sim = w.scene.sim.as_ref().unwrap();
        leg(sim, sim.bond.pos, spot)
    };
    for (k, p) in pts.iter().enumerate().skip(1) {
        clear_live_guards(&mut w, &format!("the alarm switch pre {k}"));
        let _ = w.try_walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("the alarm switch nav {k}"));
        clear_live_guards(&mut w, &format!("the alarm switch nav {k}"));
    }
    // The final approach, BEAT-SPLIT (spawn-pick-and-dart-stamps re-pin, by route): a
    // dart-stamped sleeper wakes 7200 ticks after his dart [L docs/systems/tranq-dart.md — the
    // K5's DroneFunc_RecoverTime 120 s], and a woken tower guard carries the alarm-run plumbing
    // the dart's stamp block armed [M 0x1469b8:331-357 — the game's own wake behaviour, the same
    // block the tase stamps]. The measured failure: the old single approach hop WEDGED ~3769
    // ticks at the switch's ~1.08 u floor (its "arrival" was the level-exit latch passing the
    // assert truthy) and the waker pressed inside that window. The hops accept the PRESS reach —
    // dxz < 1.4, the arrival assert below — so the loop stops at the same spot the old run
    // reached, minus the wedge that gifted the waker his window; each hop is preceded by a
    // clearing beat.
    let mut reached = false;
    loop {
        clear_live_guards(&mut w, "the alarm switch approach beat");
        {
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            if dxz(p, spot) < 1.4 && (p[1] - spot[1]).abs() < 2.0 { reached = true; break; }
        }
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        let (dx, dz) = (spot[0] - p[0], spot[2] - p[2]);
        let l = (dx * dx + dz * dz).sqrt();
        if l < 1.4 { reached = (p[1] - spot[1]).abs() < 2.0; break; }
        let step = 1.5_f32.min(l - 1.0); // stop the hop 1.0 short: never wedge on the final 1.08 floor
        let nx = p[0] + dx / l * step;
        let nz = p[2] + dz / l * step;
        if !w.try_walk_to(Wp::new(nx, spot[1], nz).r(0.8), "the alarm switch approach hop") { break; }
    }
    assert!(reached,
        "the walk must reach the ch-124 switch's press reach at {spot:?} — Walker movement only, no sim.bond.pos writes");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let d = dxz(p, spot);
        eprintln!("[09-alarm] walked to ({:.2},{:.2},{:.2}); switch at {:?}; final dxz {d:.2}", p[0], p[1], p[2], spot);
        assert!((p[1] - spot[1]).abs() < 2.0, "the walked position must be on the switch's floor (dy {:.2} — a dxz-only arrive lands underneath, 8 units down)", p[1] - spot[1]);
        assert!(d < 1.4, "the walked position must be within the press reach of the switch (dxz {d:.2})");
        assert!(!sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x7c) && !sim.drones.channels.on(0x7d),
            "the alarm is still off when Bond arrives — the E press below is the ch 0x7C writer");
    }
    let mut fired = false;
    for k in 0..120 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let b = &mut sim.bond;
            let dx = spot[0] - b.pos[0];
            let dz = spot[2] - b.pos[2];
            b.yaw = dx.atan2(dz);
            b.pitch = 0.0;
            sim.step_bond(nf_game::player::Input::default(), false, false, k % 6 == 0);
            if sim.drones.channels.on(0x7c) { fired = true; break; }
        }
    }
    assert!(fired, "the E pulses at the alarm switch must raise ch 0x7C [M Switch_Activate 0x1b02a8]");
    w.wait(8);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x61), "OR mux #278: ch 0x7C → the alarm ch 0x61 [D]");
        assert!(sim.drones.channels.on(99), "the mission-fail channel ch 99 [M 0x1827d8:128-131]");
        assert_eq!(sim.drones.mission.states[8][0], 5, "objective 1 FAILED (state 5) [M mission.rs states]");
        assert_eq!(sim.drones.mission.states[8][1], 3, "objective 2 was never completed (announced, not 4)");
        // level-exit-fade sister: the fail exit keeps ITS OWN path [M 0x182c60:30-33 — the ch 99 arm
        // posts Music_Event(7,1) and InternalState 2, then case 2 → the mission-fail MENU (:77-81);
        // the red fade + Bond's kill are the ch 0x62 DEATH arm (:35-42), not this one — daily audit
        // 2026-10-19]; it never pushes the
        // trigger exit's 300-tick black fade or requests a level load]: no hold, no exit, and the
        // board's jump was on_fail/on_death (the fail pulse [M 0x182c60:31]), never an end marker.
        assert!(sim.drones.exit_hold.is_none(), "the fail path runs no exit hold");
        assert_eq!(sim.drones.level_exit, None, "the fail path loads no level");
        let row = sim.drones.music.table.iter().find(|m| m.level == 0x0700_0009).expect("07000009's MapMusic row [D]");
        assert!(sim.drones.music.out.iter().any(|c| matches!(c, nf_game::music::Cmd::Jump { section, .. } if *section == row.on_fail || *section == row.on_death)),
            "the fail rode its own marker, out: {:?}", sim.drones.music.out);
        assert!(!sim.drones.music.out.iter().any(|c| matches!(c, nf_game::music::Cmd::Jump { section, .. } if *section == row.end_calm || *section == row.end_combat)),
            "the fail never jumped the exit end marker");
    }
}

// ---- the live-guard helpers ---------------------------------------------------------------------------------

/// The alarm-bound live states: a guard in any of these is on the sight chain that ends in a
/// switch press (CivilianChallenge 0x3E → RunToAlarm 0x89 → PressAlarm 0x8a; Attack 0x56 is the
/// chain's armed host) [M 0x15d610 case 3 → 0x164210 → 0x148010 → 0x162558 case 1].
fn live_state(st: u8) -> bool {
    matches!(st, nf_game::drone::state::CIVILIAN_CHALLENGE | nf_game::drone::state::ATTACK
        | nf_game::kiko::RUN_TO_ALARM | nf_game::kiko::PRESS_ALARM)
}

/// A guard on the alarm race in the wider sense: the chain states, the sight ack (`ever_seen`,
/// +0x228 & 8 [M 0x175ef0:82-86]), or RunToAlarm's flag (+0x16 [M 0x1736f8:19]). The flag is the
/// tell the state alone hides: RunToAlarm's ENTER hands the walk to GoToGoalPosition (0x63) with
/// the return state PressAlarm [M 0x1736f8:13-23] — and 0x63 is also the patrol's own nav state.
/// The 2026-11-05 trace (this branch): the P2K execute's shots noise-pulled the patrolling g3
/// (never latched: `seen false los false` on every traced tick) into an 890-tick 0x63 run to his
/// switch with `alarm_run` the only distinguishing field; the beat clears that only read
/// live_state ignored him until he ENTERED PressAlarm (0x8a) 31.8 u away — 210 ticks before his
/// press — and the capped takedown shot from 31.7 u for 1500 ticks while the fuse ran out.
fn racer(d: &nf_game::drone::Drone) -> bool {
    d.ever_seen || d.alarm_run || live_state(d.state)
}

/// Cycle the weapon wheel until a lethal gun is in hand — anything armed outside the
/// zero-damage ids (the 0x43/0x44 darts → msg 0x19 [M 0x13fff0:269-306], the 0x4a/0x4c taser →
/// msg 7 [M 0x1469b8 case 7]). The same cycle interact_07000009_live's kill test runs.
fn cycle_to_lethal(w: &mut Walker) -> bool {
    for _ in 0..12 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let lethal = (2..0x47).contains(&sim.drones.bond_weapon) && !matches!(sim.drones.bond_weapon, 0x43 | 0x44);
            if lethal && sim.gun.clip + sim.gun.spare > 0 { return true; }
        }
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); } // the holster/draw cycle
    }
    false
}

/// Cycle until the K5 sleep darts (0x43) are in hand with ammo — the level's stealth takedown.
fn cycle_to_darts(w: &mut Walker) -> bool {
    for _ in 0..12 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if matches!(sim.drones.bond_weapon, 0x43 | 0x44) && sim.gun.clip + sim.gun.spare > 0 { return true; }
        }
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    }
    false
}

fn asleep(st: u8) -> bool {
    matches!(st, nf_game::drone::state::STUN_DART_IMPACT | nf_game::drone::state::STUN_DART_LOOP | nf_game::drone::state::STUN_DART_RECOVER)
}

/// Another guard (≠ di) already racing — the emergency that outranks finishing the current
/// quiet takedown: every tick volleyed into a doomed-or-sleeping guard is a tick off the
/// racer's ~670–890-tick run fuse [L PCSX2; the 2026-11-05 trace].
fn other_racer(w: &Walker, di: usize) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.list.iter().enumerate()
        .position(|(i, d)| i != di && d.present && !d.corpse && d.health > 0.0 && d.enabled
            && !asleep(d.state) && racer(d))
}

/// Steer Bond toward `target` by direct input, with one nav hop when the direct line stalls (a
/// wedged steer — furniture, a wall corner — gets the NEXT nav node of the leg graph toward the
/// target instead). The shared shape of the chase and the shadow. Returns false on the cap.
fn steer_to(w: &mut Walker, target: [f32; 3], arrive: f32, cap: u32, label: &str) -> bool {
    let start = w.total_ticks;
    let mut stalled = 0u32;
    let mut best = f32::MAX;
    let mut dry_hops = 0u32;
    loop {
        if w.total_ticks - start > cap { return false; }
        if w.stop_on_exit && w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return false; }
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let d = dxz(target, bp);
        let flat = (target[1] - bp[1]).abs() < 1.5;
        if d <= arrive && flat { return true; }
        if d > best - 0.03 { stalled += 1; } else { stalled = 0; best = best.min(d); }
        if stalled > 100 {
            // wedged: one nav hop toward the target; two consecutive dry hops → the steer cannot
            // make progress here (the target may sit behind furniture the graph does not serve)
            let pts = {
                let sim = w.scene.sim.as_ref().unwrap();
                leg(sim, bp, target)
            };
            if pts.len() > 1 {
                let _ = w.try_walk_to(Wp::new(pts[1][0], pts[1][1], pts[1][2]).r(1.1), &format!("{label} hop"));
                dry_hops = 0;
            } else {
                dry_hops += 1;
                if dry_hops >= 2 { return false; }
            }
            stalled = 0;
            best = f32::MAX;
            continue;
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (target[0] - sim.bond.pos[0]).atan2(target[2] - sim.bond.pos[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: err.clamp(-0.08, 0.08),
            forward: if err.abs() > 1.3 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        w.tick(input, false);
    }
}

/// Close on a RACING guard (already acked — stealth is moot; his press fuse is running): pursue
/// his LIVE position through the nav graph, replanning as he moves. A guard who reaches his
/// switch stands there for the PressAlarm fuse — 180 + 30 ticks from the ENTER to the channel
/// write [M 0x168ea8:30-31, 56-63] — stationary, inside dart range, exactly where the chase
/// wants him. The nav walk (whole legs at the walker's own pace, a node re-checked and the plan
/// re-issued whenever he drifts 2.5 u) is what the direct steer could not be: the 2026-11-05
/// run chased g3 from 12.5 u and still lost the 670-tick race crawling at ~1.5 u/s into
/// stairwell geometry between one-node hops. Returns true when he is dead/asleep or within
/// `close` on his floor.
fn chase_to(w: &mut Walker, di: usize, close: f32, cap: u32, label: &str) -> bool {
    let start = w.total_ticks;
    let mut last_goal = [f32::MAX; 3];
    loop {
        if w.total_ticks - start > cap {
            eprintln!("[09] {label}: chase cap g{di}");
            return false;
        }
        if w.stop_on_exit && w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return false; }
        let (st, dp, hp, corpse) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[di];
            (d.state, d.pos, d.health, d.corpse)
        };
        if corpse || hp <= 0.0 { return true; }
        if asleep(st) { return true; }
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let same_floor = (dp[1] - bp[1]).abs() < 3.5;
        if same_floor && dxz(dp, bp) <= close { return true; }
        // The chase runs to his DESTINATION, not his heels: FindAlarmPoint's assigned switch
        // (`alarm_point`, +0x738 pair [M 0x153008:26-30]) is STATIC — a straight nav plan beats
        // following him through doorways (the 2026-11-05 run followed g3's live pos through the
        // north lobby door and paid ~440 ticks of door animation for it). Arrive first, and the
        // guard walks INTO dart range for his own 210-tick press.
        let goal = w.scene.sim.as_ref().unwrap().drones.list[di].alarm_point.map(|p| p.1);
        let goal = goal.unwrap_or(dp);
        if dxz(goal, last_goal) <= 2.5 {
            // he holds his line: the direct close on a (near-)stationary target
            if steer_to(w, goal, close, 240, &format!("{label} chase g{di}")) { continue; }
        } else {
            last_goal = goal;
            // the whole nav leg to his switch, node by node, re-checked each node
            let pts = {
                let sim = w.scene.sim.as_ref().unwrap();
                leg(sim, bp, goal)
            };
            for p in pts.iter().skip(1) {
                if w.stop_on_exit && w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return false; }
                let (st, dp2, hp, corpse) = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let d = &sim.drones.list[di];
                    (d.state, d.pos, d.health, d.corpse)
                };
                if corpse || hp <= 0.0 { return true; }
                if asleep(st) { return true; }
                let bp2 = w.scene.sim.as_ref().unwrap().bond.pos;
                if (dp2[1] - bp2[1]).abs() < 3.5 && dxz(dp2, bp2) <= close { return true; }
                let goal2 = w.scene.sim.as_ref().unwrap().drones.list[di].alarm_point.map(|p| p.1).unwrap_or(dp2);
                if dxz(goal2, last_goal) > 2.5 { break; } // his switch changed — replan
                let _ = w.try_walk_to(Wp::new(p[0], p[1], p[2]).r(1.1), &format!("{label} chase g{di} node"));
            }
        }
        if w.total_ticks - start > cap { return false; }
    }
}

/// Close on a NOT-yet-racing guard from BEHIND: the guards' sight is the 12 u + ±1.57 rad
/// (half-fov) cone on their facing [D props; M 0x175ef0 family] — a 180° front hemisphere — so
/// the back hemisphere never latches, and a guard who never latches never challenges, never
/// runs, never presses. The shadow point trails his CURRENT yaw (a patroller's facing rotates
/// along his route; the re-aim follows). Bail (false) the tick he latches — the caller's racer
/// branch takes the chase — and when he is on another floor (the walk's next beat retries).
fn shadow_to(w: &mut Walker, di: usize, stand: f32, cap: u32, label: &str) -> bool {
    let start = w.total_ticks;
    loop {
        if w.total_ticks - start > cap {
            eprintln!("[09] {label}: shadow cap g{di}");
            return false;
        }
        let (st, dp, dyaw, hp, corpse, racing) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[di];
            (d.state, d.pos, d.yaw, d.health, d.corpse, racer(d))
        };
        if corpse || hp <= 0.0 { return true; }
        if asleep(st) { return true; }
        if racing { return false; } // latched mid-shadow: the chase takes over
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        if (dp[1] - bp[1]).abs() >= 3.5 { return false; } // another floor: not now
        if dxz(dp, bp) <= stand + 0.5 { return true; }
        // behind him: his facing is (sin yaw, cos yaw) in (x, z) [M the yaw convention of
        // atan2(dx, dz) everywhere the walker aims]
        let behind = [dp[0] - dyaw.sin() * stand, dp[1], dp[2] - dyaw.cos() * stand];
        let target = if dxz(behind, bp) > 0.8 { behind } else { dp };
        if !steer_to(w, target, 0.7, 220, &format!("{label} shadow g{di}")) {
            let (hp, corpse, st, racing) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                (d.health, d.corpse, d.state, racer(d))
            };
            if corpse || hp <= 0.0 || asleep(st) { return true; }
            if racing { return false; }
            if w.total_ticks - start > cap { return false; }
        }
    }
}

/// One guard's takedown, the level's own stealth shape, rebuilt on the 2026-11-05 trace:
/// - the darts land POINT BLANK only — the 6.5 u volley on g5 missed all 15 (clip 5 + spare 10
///   → clip 5 spare 0) while the scratch's 2.5 u kill was one tap [L both, this branch] — so
///   the dart fires at ≤ 3.4 u, after the shadow/chase closed;
/// - a not-yet-racing guard is shadowed from BEHIND (never latches, never challenges);
/// - a RACING guard (ever_seen / alarm_run / the chain states) is chased to his switch — he
///   STOPS there for the 210-tick PressAlarm fuse [M 0x168ea8], stationary, inside dart range —
///   instead of being shot from beyond the old 16 u approach cap (the trace: g3 at 31.7 u ate
///   1500 ticks of P2K for zero damage on his 1.0 health);
/// - the dart SLEEPS (zero damage [M 0x13fff0:269-306 → msg 0x19]) — no corpse, no gunshot, no
///   noise pull of the next guard (the old loud fallback pulled the patrolling g3 into the run
///   that failed the walk). GO_TO_GOAL carries the dart case: a racer falls asleep MID-RUN
///   [M dart_impact_state_ok; the Tower band sets his wake resume to CIVILIAN_PATROL
///   M 0x1469b8:331-357]. Darts dry → return false; the caller resupplies, and only a real
///   drought falls back to the loud P2K.
/// Returns dead-or-asleep.
fn takedown(w: &mut Walker, di: usize, label: &str) -> bool {
    let start = w.total_ticks;
    let mut volleys = 0u32;
    loop {
        if w.total_ticks - start > 3000 {
            eprintln!("[09] {label}: takedown cap g{di}");
            return false;
        }
        let (st, dp, hp, corpse, racing) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[di];
            (d.state, d.pos, d.health, d.corpse, racer(d))
        };
        if corpse || hp <= 0.0 { return true; }
        if asleep(st) { return true; }
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let same_floor = (dp[1] - bp[1]).abs() < 3.5;
        let dist = dxz(dp, bp);
        // Get point-blank first. A racer is chased to wherever his chain takes him (the switch);
        // a quiet guard is shadowed from behind so he never latches at all.
        if !same_floor || dist > 3.4 {
            // an emergent racer elsewhere outranks closing on this quiet guard
            if !racing {
                if let Some(ri) = other_racer(w, di) {
                    eprintln!("[09] {label}: g{ri} raced while closing on g{di} — the racer takes the round");
                    return false;
                }
            }
            let ok = if racing {
                chase_to(w, di, 3.0, 1500, label)
            } else {
                let shadowed = shadow_to(w, di, 2.6, 1500, label);
                if !shadowed {
                    // the shadow could not reach him (wedge, another floor): a bounded direct
                    // rush still beats a far volley for the tap race
                    chase_to(w, di, 2.6, 500, label)
                } else { true }
            };
            if !ok { return false; }
            continue;
        }
        // Point blank: the silent dart. The aim tracks his live position per tick (the loose
        // gate ≈ the torso box at this range [M shoot_loose's doc]); the sleep takes him out of
        // every chain. The volley is SHORT — 110 ticks ≈ the aim settle + 2–3 taps + the dart
        // flight: the sleep the last dart causes lands IN FLIGHT after the volley returns, and
        // the re-read below catches it (the 320-tick cap fired half a clip into already-
        // sleeping guards while the next guard's race ran — the 2026-11-05 run 2 budget burn).
        let darted = { let sim = w.scene.sim.as_ref().unwrap();
            matches!(sim.drones.bond_weapon, 0x43 | 0x44) && sim.gun.clip + sim.gun.spare > 0 } || cycle_to_darts(w);
        if darted {
            w.shoot_loose(di, &format!("{label} dart {di}"), 80);
            volleys += 1;
            let (st, hp, corpse) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                (d.state, d.health, d.corpse)
            };
            if corpse || hp <= 0.0 || asleep(st) { return true; }
            // an emergent racer elsewhere outranks another volley here
            if let Some(ri) = other_racer(w, di) {
                eprintln!("[09] {label}: g{ri} raced while volleying g{di} — the racer takes the round");
                return false;
            }
            if volleys >= 4 {
                eprintln!("[09] {label}: g{di} survived {volleys} point-blank volleys — skipped");
                return false;
            }
            continue; // he drifted or the volley missed: re-close (tighter) and tap again
        }
        return false; // darts dry — the caller resupplies or falls back
    }
}

/// The patrol-beat clear: take down the nearest alive guard that threatens the alarm chain —
/// `racer()` (the sight ack, RunToAlarm's flag, the chain states), a guard LATCHING NOW
/// (`seen_ticks > 0`: inside the 12 u + FOV gate this tick — the reaction window from the first
/// held-LOS tick to the ack is the race the dart must win), a heard-noise investigator, or
/// standing inside the 15 u bubble ahead with a clear static line (the imminent challenge).
/// Ammo dry → the real resupply (floor pickups / drops), then the next guard; a real dart
/// drought with no floor resupply falls back to the loud point-blank P2K (the asserts judge the
/// noise it pulls). A guard that survived the takedown window (a wedged line) is skipped — the
/// walk's own asserts judge the outcome.
fn clear_live_guards(w: &mut Walker, label: &str) {
    let mut skipped: Vec<usize> = Vec::new();
    for round in 0..12 {
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        // Priority: racers and latching guards first (the press fuse runs on them), then the
        // bubble/heard-noise pre-sight picks; nearest within the class.
        let Some(di) = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
            .filter(|(i, d)| d.present && !d.corpse && d.health > 0.0 && d.enabled
                && !asleep(d.state) // a sleeper runs no chain — the beat is already cleared
                && !skipped.contains(i)
                && (racer(d)
                    || d.seen_ticks > 0
                    || matches!(d.state, nf_game::drone::state::HEARD_NOISE_AWARE | nf_game::drone::state::HEARD_NOISE_SUSPECT | nf_game::drone::state::HEARD_NOISE_ALERT)
                    || (dxz(d.pos, bp) < 15.0
                        && w.scene.sim.as_ref().unwrap().world
                            .segment([bp[0], bp[1] + w.scene.sim.as_ref().unwrap().bond.eye, bp[2]],
                                     [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())))
            .min_by_key(|(i, d)| (
                (!(racer(d) || d.seen_ticks > 0)) as u8,
                (dxz(d.pos, bp) * 100.0) as i32,
                *i))
            .map(|(i, _)| i)
        else { return };
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[di];
            eprintln!("[09] {label}: clearing guard {di} (state {:#x}, dist {:.1}, racer {} [ever_seen {} alarm_run {}], seen_ticks {}, alarm_point {:?}, round {round})",
                d.state, dxz(d.pos, sim.bond.pos), racer(d), d.ever_seen, d.alarm_run, d.seen_ticks, d.alarm_point);
        }
        if !takedown(w, di, label) {
            // darts dry: the real resupply, then one more look; a DROUGHT with no floor
            // resupply falls back to the loud point-blank kill (the asserts judge the noise);
            // anything else (a wedged line, a survived volley) just skips him
            let (wp, c, s) = { let sim = w.scene.sim.as_ref().unwrap();
                (sim.drones.bond_weapon, sim.gun.clip, sim.gun.spare) };
            let darts_dry = matches!(wp, 0x43 | 0x44) && c + s == 0;
            if darts_dry && w.resupply(label) { continue; }
            if darts_dry && cycle_to_lethal(w) {
                let (c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare) };
                if c + s > 0 {
                    eprintln!("[09] {label}: darts dry, no resupply — the loud point-blank fallback on g{di}");
                    w.shoot_loose(di, &format!("{label} shoot {di}"), 900);
                }
            }
            if w.scene.sim.as_ref().unwrap().drones.list[di].health > 0.0
                && !w.scene.sim.as_ref().unwrap().drones.list[di].corpse {
                // he lived through the bounded windows — stop feeding rounds into the wall
                skipped.push(di);
            }
        }
    }
}

/// A nav walk with a patrol-beat clear after every node (the live-guard shape of walk_nav_to).
fn walk_nav_clearing(w: &mut Walker, target: [f32; 3], label: &str, final_arrive: f32) {
    let pts = {
        let sim = w.scene.sim.as_ref().unwrap();
        leg(sim, sim.bond.pos, target)
    };
    for (k, p) in pts.iter().enumerate().skip(1) {
        clear_live_guards(w, &format!("{label} pre {k}"));
        let _ = w.try_walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("{label} nav {k}"));
        clear_live_guards(w, &format!("{label} nav {k}"));
    }
    let _ = w.try_walk_to(Wp { at: target, act: Act::None, arrive: final_arrive }, label);
    clear_live_guards(w, label);
}

/// The dart pin for the root-caused open item: a CHALLENGE-state guard hit by the K5 sleep
/// darts (the level's own loadout) falls asleep — STUN_DART_IMPACT 0x50 / LOOP 0x51 — and loses
/// NO health: 0x162558's group {7, 9, 0x17, 0x18, 0x19} forwards to `DroneFunc_HandleImpact(
/// …, 0x89, 1)` [M 0x162558:40-46] whose case 0x19 (0x1469b8:315-361) runs the dart sleep and
/// never reaches `NDrone2_HitDamage`; the zero damage itself is `DealWithObjHit` 0x13fff0:269-306
/// routing the 0x43/0x44 darts to msg 0x19. The pre-fix port dropped the dart here (the
/// whitelist lacked the state) — the challenge-state guard shrugged 15 darts with no flinch,
/// the open item the level-09-every-objective report left. Sister pin: the alarm stays OFF —
/// the sleeping guard never reaches PressAlarm.
#[test]
fn the_challenge_state_guard_sleeps_to_a_dart_and_loses_no_health() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000009.bin", false).expect("load 07000009"));
    w.top_up_health = true;
    w.wait(10);
    // Bond beside the lobby panel: the panel-side guard's patrol crosses his sight inside the
    // window (security_alarm_09_live's chain, challenge ~t2400 there — here the guard walks TO
    // the stationary Bond, the same 12.0 sight [D props]).
    let sim = w.scene.sim.as_mut().unwrap();
    sim.bond.pos = [-18.3, -368.4, 27.0];
    sim.bond.yaw = 3.1416;
    let mut challenge = None;
    for t in 0..4500u32 {
        w.tick(nf_game::player::Input::default(), false);
        if let Some(i) = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.present && !d.corpse && d.state == nf_game::drone::state::CIVILIAN_CHALLENGE) {
            challenge = Some((t, i));
            break;
        }
    }
    let Some((t_ch, di)) = challenge else { panic!("no guard challenged the panel Bond within the window"); };
    let (hp0, clip0, spare0) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.drones.list[di].health, sim.gun.clip, sim.gun.spare)
    };
    eprintln!("[09-dart] challenge at +{t_ch}: drone {di} hp {hp0}, K5 clip {clip0} spare {spare0}");
    // The darts ARE the load weapon (0x43) — fire from here through the real path.
    let shot = w.shoot(di, "the challenger");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[di];
    eprintln!("[09-dart] after the darts: shot {shot} hp {:.1} state {:#x} corpse {}", d.health, d.state, d.corpse);
    assert!(matches!(d.state, nf_game::drone::state::STUN_DART_IMPACT | nf_game::drone::state::STUN_DART_LOOP | nf_game::drone::state::STUN_DART_RECOVER)
        || d.corpse, "the challenge-state guard sleeps to the darts (the 0x50-0x52 family [M 0x1469b8 case 0x19]), got {:#x}", d.state);
    assert_eq!(d.health, hp0, "the darts lose NO health (msg 0x19 never reaches HitDamage [M 0x13fff0:269-306])");
    assert!(!sim.drones.channels.on(0x61), "no alarm while the challenger sleeps (the press never ran)");
}

/// The dart's Tower stamp block [M 0x1469b8:331-357 — the case-0x19 copy of the block the tase's
/// case-7 arm runs at :103-128]: a SIGHTED tower guard who takes a dart gets the whole
/// alert-plumbing block stamped — first-attack latch off (`+0x4F8 &= ~0x02000000`), alt state
/// 0x89 RunToAlarm (`+0x5A4`), `+0x16 = 0`, alert bits 0 (`+0x4FC`), the `+0xC48 == 100`
/// first-behaviour scratch, mode 9 / alt mode 7 (`+0xC5/+0xC6`), sight latch off
/// (`+0x4F8 &= ~0x04000000`), return state 0x14 (`+0x5A2`) — and the impact resume 0x15. The
/// challenger scenario IS the sighted arm: he walked to the panel Bond and challenged him, so
/// the sight latch is set when the dart lands. The pre-fix port picked only the resume and
/// skipped the block body (the tase ran it whole — docs/systems/tranq-dart.md "the Tower band"),
/// so a darted guard woke without the 0x89 alarm run the tasEd one makes.
#[test]
fn the_dart_stamps_the_tower_alert_block_like_the_tase() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000009.bin", false).expect("load 07000009"));
    w.top_up_health = true;
    w.wait(10);
    let sim = w.scene.sim.as_mut().unwrap();
    sim.bond.pos = [-18.3, -368.4, 27.0];
    sim.bond.yaw = 3.1416;
    let mut challenge = None;
    for t in 0..4500u32 {
        w.tick(nf_game::player::Input::default(), false);
        if let Some(i) = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.present && !d.corpse && d.state == nf_game::drone::state::CIVILIAN_CHALLENGE) {
            challenge = Some((t, i));
            break;
        }
    }
    let Some((t_ch, di)) = challenge else { panic!("no guard challenged the panel Bond within the window"); };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[di];
        eprintln!("[09-dart-stamp] challenge at +{t_ch}: drone {di} state {:#x} sighted {} mode {} alt_state {:#x}",
            d.state, d.sighted as u8, d.mode, d.alt_state);
        assert!(d.sighted, "the challenger has seen Bond — the sighted arm's gate [+0x4F8 & 0x4000000]");
    }
    // The darts ARE the load weapon (0x43, selected at the level start [M 0x1a7ef8 case
    // 0x7000009]) — fire through the real path.
    let shot = w.shoot(di, "the challenger");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[di];
    eprintln!("[09-dart-stamp] after the darts (shot {shot}): state {:#x} mode {} alt_mode {} alt_state {:#x} sighted {} return_state {:#x} resume {:#x} alarm_run {} alert_bits {:#x} first_attack {}",
        d.state, d.mode, d.alt_mode, d.alt_state, d.sighted as u8, d.return_state, d.impact_resume, d.alarm_run as u8, d.alert_bits, d.first_attack_done as u8);
    assert!(matches!(d.state, nf_game::drone::state::STUN_DART_IMPACT | nf_game::drone::state::STUN_DART_LOOP | nf_game::drone::state::STUN_DART_RECOVER)
        || d.corpse, "the dart sleeps him (the 0x50-0x52 family [M 0x1469b8 case 0x19]), got {:#x}", d.state);
    // THE STAMP BLOCK [M 0x1469b8:331-357, the sighted arm]:
    assert!(!d.first_attack_done, "+0x4F8 &= ~0x02000000: the first-attack latch cleared [M :337]");
    assert_eq!(d.alt_state, nf_game::kiko::RUN_TO_ALARM, "+0x5A4 = 0x89 RunToAlarm [M :338]");
    assert!(!d.alarm_run, "+0x16 = 0 [M :339]");
    assert_eq!(d.alert_bits, 0, "+0x4FC = 0 [M :340]");
    assert_eq!(d.mode, 9, "+0xC5 = 9 [M :350]");
    assert_eq!(d.alt_mode, 7, "+0xC6 = 7 [M :351]");
    assert!(!d.sighted, "+0x4F8 &= ~0x04000000: the sight latch cleared [M :352-353]");
    assert_eq!(d.return_state, nf_game::drone::state::CIVILIAN_PATROL, "+0x5A2 = 0x14 [M :354]");
    assert_eq!(d.impact_resume, nf_game::drone::state::RETURN_TO_PATROL_PATH, "SMI+0x10 = 0x15 [M :355]");
}

/// Real damage: the same walk with the health top-up off for every walker it makes (the
/// tests' own `top_up_health = true` included) and the walker's return fire on — Bond's death
/// panics with the death spot and the last hits (common/damage_log.rs), so a pass pins that the
/// walk completes on the port's own damage.
#[test]
fn walk_tower_a_start_to_finish_real_damage() {
    walker::with_real_damage(walk_tower_a_start_to_finish);
}

/// The lobby guards' effective health is 1.0, pinned through the real load path. 07000009's only
/// stats section sits on drone placement #319 (hash 02000637; in-map #62, the census's global #319)
/// [D]: tag 0x80010006 → three 2-word records decoded by the disassembly-verified
/// `behaviour_util_get` (widths 0x303330 = 8,3,8,8,5,8,8,1 [D ACTION.ELF]; field f → word f via the
/// 0x303350 jump table [D]; LSB-first; kind advance `(tag&0xffff)/3` words [M 0x1c7cac+0x1c7d74];
/// each field `& 0xff` [M 0x1c7d00]) → type 0xD k0 60/0/1/60/2/60/60/0, k1 80/1/1/80/3/80/80/1,
/// k2 100/2/1/100/3/100/100/0. That decoder is the one the shared snapshots prove live: its decode
/// of 07000001's #158 (type 2) equals ee_start_07000001.bin's `drone_stats` 0x31aa30 type-2 record
/// byte-for-byte [L], and ee_02b.bin pins the same [L, enemy_accuracy.rs #442]. No live snapshot
/// holds a type-0xD record (all three ee_start_*.bin hold only the boot level's types 2 and 12 [L]),
/// so kind 1's consumer is pinned from code: `DoModeSettingsNEW` reads kind 1 only [M 0x14c704
/// `addiu a1,zero,1`] and writes record word 2 to +0xAC as `(float)(int)w` [M 0x14c70c-0x14c730] —
/// raw HP, not an index or multiplier: `NDrone2_HitDamage` 0x145558:154 subtracts the damage from
/// +0xAC and :156's `> 0` is the survival gate, with no rescale off the stats path (DefaultInit's
/// 0x14b300:556-561 multiply is the captain branch; 0xD is not a captain kind 1/2/3/6/7). Every 09
/// type-0xD placement runs DoModeSettingsNEW (param 5 = 0x64 ≥ 0x24 [D census]; [M 0x14b300 disasm
/// 0x14bccc-d4: `sltiu v0,a1,36` on `*(setup+0x48)` = param 5]) and #319 precedes #321/#322/#323/
/// #507/#518, so all six load health 1.0 / accuracy 80 / reaction 80 / recover 80: one landed round
/// of ≥ 1.0 damage kills a lobby guard (the P2K round is 1.5 × difficulty [D weapon_data]).
/// Sister: accuracy 80 puts `DoBulletAccuracy`'s p at 100 − 400 < 0 — the guards never take the
/// aimed zero-offset arm, they always wobble (legal; the live hit counts in enemy_accuracy_live's
/// model absorb it through the wobble ray).
#[test]
fn the_lobby_guards_load_with_the_type_0xd_stats_health_1() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000009.bin", false).expect("load 07000009"));
    w.wait(10);
    let sim = w.scene.sim.as_ref().unwrap();
    let lobby: Vec<&nf_game::drone::Drone> = sim.drones.list.iter().filter(|d| d.dtype == 0xd).collect();
    assert!(lobby.len() >= 6, "the six type-0xD lobby guards load (param 2 = 1 ≤ every difficulty) [D census], got {}", lobby.len());
    for d in &lobby {
        assert_eq!((d.health, d.max_health), (1.0, 1.0),
            "a type-0xD guard loads +0xAC = (float)1 from #319's kind-1 word 2 [M 0x14c70c-30; D census], got {}/{}", d.health, d.max_health);
        assert_eq!(d.accuracy, 80, "kind-1 word 0 → +0xB4 [M 0x14c750-54], got {}", d.accuracy);
        assert_eq!(d.aggression, 1, "kind-1 word 1 → +0xB5 [M 0x14c758-5c], got {}", d.aggression);
        assert_eq!((d.reaction, d.recover), (80, 80), "kind-1 words 5/6 → +0xB8/+0xB9 [M 0x14c600:105-106], got {}/{}", d.reaction, d.recover);
    }
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-08-09-0a-0b): the
/// same walked car-park → lobby → security-room → panel → cab route at the menu's Agent pick —
/// GameState+0x28 = 2 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e = "Agent"]. On this level group
/// (09-0b) the pick scales Bond's pain by Plr_DMod_Normal 0.8 instead of Easy 0.6 and the
/// guards' hit chance by DroneFiring_Accuracy_Normal 1.0 instead of Easy 0.8 [M Player_HandlePain
/// 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 09-0b group ← ReadTuningVars
/// 0x1d3a80]; the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. Real input
/// only — the load path is the only difference.
#[test]
fn walk_tower_a_start_to_finish_real_damage_agent() {
    walker::with_real_damage(|| walk_tower_a_start_to_finish_at(2))
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-08-09-0a-0b): the same
/// walked car-park → lobby → security-room → panel → cab route at the menu's 00 Agent pick —
/// GameState+0x28 = 3 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 2 carries 3 [D]; Txt 0x14f = "00 Agent"]. On this level
/// group (09-0b) the pick scales Bond's pain by Plr_DMod_Hard 1.0 (Easy 0.6, Agent 0.8); the
/// guards' hit chance stays DroneFiring_Accuracy_Hard 1.0 — the SAME row Agent runs (Easy 0.8)
/// — and the guards' base bullet damage rides DroneDamage_Hard 0.8 (Agent 1.0) [M Player_HandlePain
/// 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; ReadTuningVars 0x1d3a80 cases
/// 0x7000009-a-b; tuning.rs 09-0b group]; the min-difficulty placements pass Drone_Create's gate
/// at d2 AND d3 [M 0x136be0:49]. Real input only — the load path is the only difference (plus
/// the d3-gated tactics the deaths force, inside this file).
#[test]
fn walk_tower_a_start_to_finish_real_damage_00agent() {
    walker::with_real_damage(|| walk_tower_a_start_to_finish_at(3))
}
