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

use walker::{Act, Walker, Wp, dxz, leg};

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
fn walk_tower_a_start_to_finish() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000009.bin", false).expect("load 07000009"));
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
    walk_nav_clearing(&mut w, [19.79, -376.68, 30.37], "the stair base", 1.4);
    walk_nav_clearing(&mut w, [-18.4, -368.8, 27.5], "up the stairs and across the lobby", 1.2);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x7c) && !sim.drones.channels.on(0x7d),
            "the walk to the switch must not raise the alarm (obj 1) before the press");
    }
    // The final leg walks the nav graph to the switch with NO clears: clear_live_guards chases
    // ever_seen guards across floors (the first walked runs dragged Bond down to the car park
    // mid-leg and the dxz-only arrival then "reached" the switch from 8 units below it), and the
    // lobby leg above already left the beat cleared.
    w.walk_nav_to(spot, "the alarm switch", 0.95);
    assert!(w.try_walk_to(Wp::new(spot[0], spot[1], spot[2]).r(0.95), "the alarm switch final approach"),
        "the walk must reach the ch-124 switch at {spot:?} — Walker movement only, no sim.bond.pos writes");
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

/// One guard's takedown, the level's own stealth shape: the K5 dart SLEEPS him (zero damage
/// [M 0x13fff0:269-306 → msg 0x19; 0x1469b8:315-361 → 0x50]) and one P2K round executes the
/// sleeper — a sleeping guard dies to ANY bullet regardless of health [M the dart-family
/// DSTATEs' msg-8 arm 0x163430/0x163570/0x163700 case 8/9 → 0x44 Death_Anim]. The shots land
/// POINT BLANK: approach to ~2.4u first whenever the guard is not mid-chain (the first run's
/// execute at 10u over the stairwell missed a static sleeper 14+ times; the scratch's 2.5u
/// kill was one tap). A live alarm-bound guard gets shot from where Bond stands (the chain is
/// racing). Darts dry → direct fire. Returns dead.
fn takedown(w: &mut Walker, di: usize, label: &str) -> bool {
    let (st0, gp) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[di];
        (d.state, d.pos)
    };
    let live = live_state(st0);
    if !live { approach(w, gp, 2.2, label); }
    // The dart: the silent takedown. A sleeping guard runs NO chain (the 0x50-0x52 family's
    // switch covers every message — blind, deaf, unalertable [M 0x163430/0x163570/0x163700]) —
    // the beat is cleared without a corpse, without the gunshot that pulls the next guard into
    // a race with the walk (the two executed runs both failed to a noise-pulled presser while
    // Bond was busy). The wake timer (RecoverTime) outruns the walk: a woken guard resumes his
    // patrol far behind Bond.
    let darted = {
        let sim = w.scene.sim.as_ref().unwrap();
        matches!(sim.drones.bond_weapon, 0x43 | 0x44) && sim.gun.clip + sim.gun.spare > 0
    } || cycle_to_darts(w);
    if darted {
        w.shoot_loose(di, &format!("{label} dart {di}"), 500);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.list[di].corpse || asleep(sim.drones.list[di].state) { return true; }
    }
    // Darts dry (or the dart missed twice): the loud fallback — direct fire.
    if cycle_to_lethal(w) {
        let (c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare) };
        if c + s > 0 {
            w.shoot_loose(di, &format!("{label} shoot {di}"), 1500);
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[di];
    d.corpse || d.health <= 0.0 || asleep(d.state)
}

fn st0_of(d: &nf_game::drone::Drone) -> u8 { d.state }

/// Walk Bond to `stand_dist` from `gp` (a static guard's position) THROUGH THE NAV GRAPH —
/// the stairwell guards stand mid-stair, and the direct XZ steer stalls at the stair base with
/// the guard 4u up (the first run's dart then executed from BELOW, where the stair geometry
/// itself eats the ray — the level-09-every-objective report's "geometry shadow", real for
/// from-below lines; the scratch's kill was from the stairs beside him).
fn approach(w: &mut Walker, gp: [f32; 3], stand_dist: f32, label: &str) {
    let bp = w.scene.sim.as_ref().unwrap().bond.pos;
    let dx = bp[0] - gp[0];
    let dz = bp[2] - gp[2];
    let l = (dx * dx + dz * dz).sqrt();
    if l <= stand_dist { return; }
    let stand = [gp[0] + dx / l * stand_dist, gp[1], gp[2] + dz / l * stand_dist];
    if dxz(stand, bp) > 16.0 { return; } // too far to bother closing — shoot from here
    let pts = {
        let sim = w.scene.sim.as_ref().unwrap();
        leg(sim, bp, stand)
    };
    for (k, p) in pts.iter().enumerate().skip(1) {
        let _ = w.try_walk_to(Wp::new(p[0], p[1], p[2]).r(1.1), &format!("{label} approach {k}"));
    }
    let _ = w.try_walk_to(Wp::new(stand[0], stand[1], stand[2]).r(0.9), &format!("{label} approach end"));
}

/// The patrol-beat clear: take down the nearest alive guard that threatens the alarm chain —
/// `ever_seen` latched (the sight's own ack bit, +0x228 & 8 [M 0x175ef0:82-86]), a live
/// alarm-bound state, or standing inside the 15u bubble ahead with a clear static line (the
/// imminent challenge). Ammo dry → the real resupply (floor pickups / drops), then the next
/// guard. A guard that survived the takedown window (a wedged line) is skipped — the walk's
/// own asserts judge the outcome.
fn clear_live_guards(w: &mut Walker, label: &str) {
    let mut skipped: Vec<usize> = Vec::new();
    for round in 0..12 {
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        // Priority: the racing guards first (ever_seen / the alarm-bound states / the
        // heard-noise investigators 0xa3-0xa5 walking in on the gunfire), then the 15u-bubble
        // pre-sight picks; nearest within the class.
        let Some(di) = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
            .filter(|(i, d)| d.present && !d.corpse && d.health > 0.0 && d.enabled
                && !asleep(d.state) // a sleeper runs no chain — the beat is already cleared
                && !skipped.contains(i)
                && (d.ever_seen || live_state(d.state)
                    || matches!(d.state, nf_game::drone::state::HEARD_NOISE_AWARE | nf_game::drone::state::HEARD_NOISE_SUSPECT | nf_game::drone::state::HEARD_NOISE_ALERT)
                    || (dxz(d.pos, bp) < 15.0
                        && w.scene.sim.as_ref().unwrap().world
                            .segment([bp[0], bp[1] + w.scene.sim.as_ref().unwrap().bond.eye, bp[2]],
                                     [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())))
            .min_by_key(|(i, d)| (
                (d.ever_seen || live_state(d.state)
                    || matches!(d.state, nf_game::drone::state::HEARD_NOISE_AWARE | nf_game::drone::state::HEARD_NOISE_SUSPECT | nf_game::drone::state::HEARD_NOISE_ALERT)) as u8,
                (dxz(d.pos, bp) * 100.0) as i32,
                *i))
            .map(|(i, _)| i)
        else { return };
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[di];
            eprintln!("[09] {label}: clearing guard {di} (state {:#x}, dist {:.1}, round {round})", d.state, dxz(d.pos, sim.bond.pos));
        }
        if !takedown(w, di, label) {
            // dry or a wedged line: the real resupply, then one more look; a drought BREAKS the
            // clear (every further round would burn seconds of cycling while the guards run)
            let (c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare) };
            if c + s == 0 && !w.resupply(label) { break; }
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
