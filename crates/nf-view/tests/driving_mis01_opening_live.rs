//! MIS01 (Paris Prelude) opening in the driving view with no input (docs/systems/driving-mis01-opening.md).
//!
//! The script [D MIS01 `Map `]: trigger 644 on the flight path runs `ECallStage(42)`; stage 42's
//! rule 5 is `SRuleDifficulty` mask 6 (Agent, 00 Agent [M 0x20aac0]) with `EKillAIElement`,
//! `EPlayerLose`, `EDisableTrigger(712)`. Stage 19's rules 45 / 85 / 105 / 107 (the two pursuers
//! blown out or dead) disable 644 first. So on Agent the pursuers must be stopped before the
//! helicopter reaches 644; on Operative nothing loses there.
//!
//! The view's helicopter follows the spline at the controller's rate 0.5 [M 0x13da80, 0x1b1110]
//! (it ran at 1.0, reaching 644 at tick 425), with the path player's velocity
//! [M 0x1b14b8, 0x131620], and the front end's difficulty reaches the script (PTP+0x984 mapped
//! 1 -> 0, 2 -> 1, 3/4 -> 2 [M 0x174bf0:70-81]). Opt-in: NF_ISO.

use nf_game::driving::mission::MissionState;

struct Run {
    fired: Vec<(u32, usize)>,
    ended: Option<(u32, MissionState)>,
    max_vel_err: f32,
    end: [f32; 3],
}

fn run(game_difficulty: u32, ticks: u32) -> Run {
    let iso = std::env::var("NF_ISO").unwrap();
    let (_, mut d) = nf_view::driving::load_driving_scene_at(&iso, "MIS01", game_difficulty).expect("load MIS01");
    let mut r = Run { fired: Vec::new(), ended: None, max_vel_err: 0.0, end: [0.0; 3] };
    for tick in 1..=ticks {
        let p0 = d.car.pos;
        d.step(0.0, 0.0, 0.0, false);
        let on = d.intro.as_ref().is_some_and(|(i, p)| i.spline_position(p).is_some());
        if on && tick > 2 {
            let e = (0..3).map(|k| (d.car.vel[k] / 60.0 - (d.car.pos[k] - p0[k])).abs()).fold(0.0, f32::max);
            r.max_vel_err = r.max_vel_err.max(e);
        }
        let sc = d.mission.script.as_mut().expect("MIS01 script");
        r.fired.extend(sc.take_fired().into_iter().map(|(t, _)| (tick, t)));
        if d.mission_state != MissionState::Playing {
            r.ended = Some((tick, d.mission_state));
            break;
        }
    }
    r.end = d.car.pos;
    r
}

fn fired_at(r: &Run, t: usize) -> Option<u32> { r.fired.iter().find(|f| f.1 == t).map(|f| f.0) }

#[test]
fn drive_difficulty_maps_the_front_end_pick() {
    // The one mapping: nf_game::ptp::driving_difficulty (DRIVING main's remap [M 0x174bf0:70-81]).
    use nf_game::ptp::driving_difficulty;
    assert_eq!([1, 2, 3, 4].map(driving_difficulty), [0, 1, 2, 2]);
    assert_eq!(driving_difficulty(nf_view::driving::BOOT_DIFFICULTY), 1, "the boot default is Agent");
}

#[test]
fn agent_loses_at_trigger_644_after_the_helicopter_flies_there_at_half_rate() {
    if std::env::var("NF_ISO").is_err() { eprintln!("NF_ISO not set, skipping"); return }
    let r = run(2, 1500);
    let t644 = fired_at(&r, 644).expect("the flight reaches trigger 644");
    let (end, state) = r.ended.expect("Agent: stage 42's rule 5 loses with no pursuer stopped");
    assert_eq!(state, MissionState::Failed);
    assert_eq!(end, t644, "the lose is the stage-42 call 644 runs, on its tick");
    // Rate 0.5: the port's flight run reaches 644 at tick 922 with a 0.5 body radius
    // (driving_sniper_live); the Vanquish-sized body here touches it a little earlier. At rate
    // 1.0 it was tick 425.
    assert!((800..=960).contains(&t644), "644 at tick {t644}");
    assert!(r.max_vel_err < 1e-3, "on the spline the velocity is the path step x 60: off by {}", r.max_vel_err);
}

#[test]
fn operative_flies_the_whole_phase_with_no_lose() {
    if std::env::var("NF_ISO").is_err() { eprintln!("NF_ISO not set, skipping"); return }
    let r = run(1, 4000);
    assert_eq!(r.ended, None, "Operative: rule 5's mask 6 skips difficulty 0");
    assert!(fired_at(&r, 644).is_some() && fired_at(&r, 712).is_some(), "644 then 712 fire on the way");
    let t742 = fired_at(&r, 742).expect("742 (ESetNextAISpline path_Heli_Hover) fires");
    assert!(t742 > 3000, "742 at {t742}");
    // ESetNextAISpline: the helicopter ends on the hover track at the end of path_Helicopter02
    // (-143.0, 13.5, 542.4) [D], not wrapped back to the start of the same track.
    let dx = r.end[0] + 143.0;
    let dz = r.end[2] - 542.4;
    assert!((dx * dx + dz * dz).sqrt() < 5.0, "ends at {:?}", r.end);
    assert!(r.max_vel_err < 1e-3, "velocity off by {}", r.max_vel_err);
}
