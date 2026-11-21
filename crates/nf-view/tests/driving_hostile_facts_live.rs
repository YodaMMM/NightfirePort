//! The driving view feeds the mission script the AI elements the game's way: every element the
//! rules name, live or not, with the record's hit points (`HostileWorld::script_elements`), and the
//! AI ground vehicles in the trigger walk as kind 2 (`trigger_cars`). An element destroyed by the
//! player's projectiles (the view's own hit test, `HostileWorld::damage`) fires the rule the track
//! carries for it [D the `Rule` entries, M 0x209820 / 0x2096d0]. Opt-in: NF_ISO.

use nf_game::driving::weapons as w;

/// One player projectile per call placed at element `e` (moving onto it), so the view's
/// `step_weapons` hit test lands it and books the hit through `HostileWorld::damage`.
fn shoot_at(d: &mut nf_view::driving::DrivingState, e: usize, n: usize) -> bool {
    let Some(p) = d.hostiles.as_ref().and_then(|h| h.world.live.iter().find(|v| v.element == e)).map(|v| v.pos()) else { return false };
    let now = d.ticks as i32;
    let wp = d.weapons.as_mut().expect("weapons");
    let k = wp.sys.secondary.max(0) as usize;
    for _ in 0..n {
        let pp = w::launch_params(k, 0.0, 0.0, 60, w::Launcher::Player);
        wp.shots.push(w::Projectile::launch(k, pp, [p[0], p[1] + 0.5, p[2]], [0.0, -1.0, 0.0], now));
    }
    true
}

fn health(d: &nf_view::driving::DrivingState, e: usize) -> f32 {
    d.hostiles.as_ref().unwrap().world.element_facts(e, d.car.pos).unwrap().health
}

#[test]
fn mis01_a_pursuer_destroyed_in_the_sniper_run_turns_off_the_t644_fail() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    // MIS01 [D]: list 686 (trigger 685) changes to stage 19. Stage 19's R105 / R107 are SRuleDeath
    // on AIEl 768 / 2816 (elements 3 and 11, the two pursuing henchman cars): each disables
    // trigger 644 and starts timer 4 (2 s); R113 (timer 4 below 1 s) calls stage 39. Trigger 644
    // calls stage 42, whose R5 (SRuleDifficulty mask 6: difficulty 1 or 2) kills element 23 and
    // runs EPlayerLose. The view plays difficulty 1.
    let (_, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS01").expect("load");
    let sc = d.mission.script.as_ref().unwrap();
    let r105 = sc.map.rules.iter().find(|r| r.index == 105).unwrap();
    assert_eq!((r105.word(0), r105.word(8) as i32, sc.map.ref_path(r105.word(0x14))), (3, 19, Some("CARP::<<Map>>::{AIEl}::768")));
    let mut killed_at = None;
    for t in 0..700u32 {
        if killed_at.is_none() && t >= 30 && health(&d, 3) > 0.0 {
            assert!(shoot_at(&mut d, 3, 10), "t{t}: element 3 is live");
        }
        d.step(0.0, 0.0, 0.0, false);
        if killed_at.is_none() && health(&d, 3) <= 0.0 {
            killed_at = Some(d.ticks);
        }
        assert!(!d.mission.script.as_ref().unwrap().lost(), "t{t}: lost (rules {:?})", d.rule_log);
    }
    let killed = killed_at.expect("element 3 destroyed");
    eprintln!("MIS01: element 3 at 0 hp on tick {killed}; rules {:?}; triggers {:?}", d.rule_log, d.trigger_log);
    let sc = d.mission.script.as_ref().unwrap();
    assert_eq!(d.rule_log.get(&105).map(|r| r.1), Some(1), "R105 fired once");
    assert_eq!(d.rule_log[&105].0, killed, "the frame the hit points reached 0");
    assert_eq!(sc.trigger_flags[644] & 1, 0, "trigger 644 disabled");
    assert!(!d.trigger_log.contains_key(&644), "trigger 644 never ran");
    assert!(!d.rule_log.contains_key(&5), "stage 42's R5 never ran");
    assert!(d.rule_log.get(&113).is_some_and(|r| r.0 > killed + 60), "R113 on timer 4 below 1 s, after the 2 s start");
    assert!(!d.rule_log.contains_key(&107), "element 11 lives");
}

#[test]
fn race_destroying_the_rival_wins() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    // RACE [D]: R8 (default list) is SRuleDeath on AIEl 0, the rival: HUD 0x2c, timer 9 = 4 s,
    // stage 2; stage 2's R9 wins below 1 s (the same end as the player's finish T214).
    let (_, mut d) = nf_view::driving::load_driving_scene(&iso, "RACE").expect("load");
    let sc = d.mission.script.as_ref().unwrap();
    let r8 = sc.map.rules.iter().find(|r| r.index == 8).unwrap();
    assert_eq!((r8.word(0), r8.word(8) as i32, sc.map.ref_path(r8.word(0x14))), (3, -1, Some("CARP::<<Map>>::{AIEl}::0")));
    let mut killed_at = None;
    for t in 0..800u32 {
        if killed_at.is_none() && t >= 30 && health(&d, 0) > 0.0 {
            assert!(shoot_at(&mut d, 0, 20), "t{t}: the rival is live");
        }
        d.step(0.0, 0.0, 0.0, false);
        if killed_at.is_none() && health(&d, 0) <= 0.0 {
            killed_at = Some(d.ticks);
        }
        if d.mission.script.as_ref().unwrap().state != nf_game::driving::script::DriveState::Playing {
            break;
        }
    }
    let killed = killed_at.expect("the rival destroyed");
    eprintln!("RACE: rival at 0 hp on tick {killed}; rules {:?}; state {:?}", d.rule_log, d.mission_state);
    assert_eq!(d.rule_log.get(&8).map(|r| r.0), Some(killed), "R8 the frame the rival's hit points reached 0");
    assert!(d.rule_log.contains_key(&9), "stage 2's R9");
    assert_eq!(d.mission.script.as_ref().unwrap().state, nf_game::driving::script::DriveState::Won);
}
