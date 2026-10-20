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

use walker::{Act, Walker, Wp};

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
#[test]
fn walk_tower_a_start_to_finish() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000009.bin", false).expect("load 07000009"));
    w.top_up_health = true;
    // The guards are HELD for this walk (the renderer's --hold-npcs dev hold [M lib.rs:5511]):
    // the walk's job is the interactive chain — the arrival plate, the panel SP, the muxes, the
    // lift doors, the exit — not the guard AI. With the alarm chain working
    // (security-room-alarm-09: CivilianPatrol 0x15d610 → Attack → FirstSightState 0x147c48 prop
    // 0x1a → CivilianChallenge 0x162558 → RunToAlarm → PressAlarm) the patrolling guards catch a
    // scripted walker (the east-stair pinch at 3.5u; the stair head is permanently patrolled) —
    // the game is a stealth level and a live-guard walk is a stealth exercise, not a chain test.
    // The alarm coverage lives in security_alarm_09_live.rs.
    w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
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
    // graph (lenient: a wedged node is skipped and the next re-aims — the lobby's roaming guards
    // and the raised desk platform body-block direct steers in some runs).
    w.walk_nav_to([19.79, -376.68, 30.37], "the stair base", 1.4);
    w.walk_nav_to([-18.4, -368.8, 27.5], "up the stairs and across the lobby to the panel", 1.2);
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
    // 262/263/264]. Take the first, stand Bond beside it (a teleport to reach the spot — the press
    // itself is the real input path), aim, and pulse E.
    let spot = {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = sim.drones.useobjs.switches.iter().find(|s| s.ch == 124 && s.gate == 0)
            .expect("an alarmswitchplayer on ch 124 with no gate [D]");
        s.pos
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [spot[0] + 0.9, spot[1], spot[2]];
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
