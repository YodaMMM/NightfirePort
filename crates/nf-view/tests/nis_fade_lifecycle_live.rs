//! audit-nis-ops: the two fade behaviours the script-player audit found unmodelled, pinned on
//! 07000004's real intro → story chain through the real load + sim path (no teleport).
//!
//! [M] The game's `Script_Play` 0x1db588:24-26 latches the global `ScriptCam` to the hashcode of
//! every camera-flag script it plays; `Script_Update` 0x1da998:44-79 arms
//! `Camera_SetFade(40, 4, 0xff, 0)` every tick once a playing camera script's clock passes
//! `length − 20` (the death NISs and the no-fade hashes {0x60004fa, 0x60008c8, 0x6000080,
//! 0x6000994} excepted). The type byte (the call's 4th argument, stored at viewer+0x234) is 0 —
//! and `Camera_SetFade` 0x1bbdd8:14 refuses a new fade only while the target viewer's own object
//! runs a TYPE-1 fade, so the every-tick near-end re-arm overwrites itself mid-ramp: the veil
//! never leaves ~1/40 alpha while the script runs. `Script_Free` 0x1da880:49-51 clears ScriptCam
//! and the hand-back drops the script viewer's fade with the cut-scene.
//! [M] The skip (`Script_FFwd` 0x1dac78:49-57) arms `+0xAB4 = min(remaining, 45)` and calls
//! `Camera_SetFade(+0xAB4, 4, 0xff, 1)` — a TYPE-1 fade-out to black over up to 45 ticks while
//! the scene keeps playing; the catch-up fast-forward starts only once the script fade clock
//! passes +0xAB4 (Script_Update 0x1da998's loop guard), and the tail 0x1da998:101-110 fades the
//! GAMEPLAY viewer back in (`Camera_SetFade(−22, 0, 0xff, 1)`).
//! [D] 07000004's intro 0x06000083: camera flag set, length 500, no op-0x13 in the pack (the
//! census: 07000004 carries N14 only); the story 0x06000082: camera flag set, length 830. Both
//! non-looping, so the near-end window is crossed exactly once per script [D the audit census].

use nf_game::player::Input;

fn the_intro_and_story_near_end_fades() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000004.bin", false).expect("load 07000004");
    // (tick, alpha, rate, mode, script-viewer?) per tick the overlay state is live.
    let mut windows: Vec<(u32, f32, f32, u8, bool)> = Vec::new();
    let mut cleared: Vec<u32> = Vec::new();
    let mut live = false;
    for t in 0..1500u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        match sim.drones.view_fade {
            // Only the near-end shape: mode 0, the script viewer, rate 1/40.
            Some(f) if f.mode == 0 => {
                if !live { live = true; }
                windows.push((t, f.alpha, f.rate, f.mode, sim.drones.view_fade_script));
            }
            Some(_) => {}
            None => { if live { live = false; cleared.push(t); } }
        }
    }
    // Two windows: the intro's (clock ≥ 480 of 500) and the story's (clock ≥ 810 of 830, which
    // starts when the intro writes ch 0x38 at its end tick — ≈ tick 500 + the load settle).
    assert!(windows.len() >= 2, "both camera scripts must arm the near-end fade [M 0x1da998:44-79], got {} ticks live: first {:?} last {:?}",
        windows.len(), windows.first(), windows.last());
    let first = windows[0];
    assert!((460..560).contains(&first.0), "the intro's window opens at its clock 480 (+ the load settle), got tick {}", first.0);
    assert!((first.2 - 1.0 / 40.0).abs() < 1e-6, "the near-end fade is Camera_SetFade(40, ..) — rate 1/40 [M 0x1da998:69], got {}", first.2);
    assert!(first.4, "the near-end fade arms on the script viewer [M 0x1bbdd8 slot 4]");
    // The every-tick re-arm overwrites itself (type 0 never blocks): the veil never develops.
    let worst = windows.iter().take(18).map(|w| w.1).fold(0.0f32, f32::max);
    assert!(worst < 0.1, "the type-0 re-arm resets the alpha each tick [M 0x1bbdd8:14] — the veil stays near zero, worst {worst}");
    // The hand-back/completion ends it: the intro's last re-arm is at its end tick (~500), then
    // the un-re-armed ramp completes within its 40-tick length and the idle object stops drawing
    // [M 0x1bada0 state 2 → 0] — gone well before the story's own window (~1310).
    let first_clear = cleared[0];
    assert!(first_clear > first.0 && first_clear - first.0 < 70,
        "the near-end fade ends with its script's window + one completion ramp [M 0x1bada0], opened {} cleared {}", first.0, first_clear);
    let second = windows.iter().find(|w| w.0 > first_clear).expect("the story scene arms its own near-end fade");
    assert!(first_clear < second.0, "the intro's fade must not shadow the story scene's window");
    assert!((1250..1450).contains(&second.0), "the story's window opens ≈500 + 810 (+ settle), got tick {}", second.0);
}

/// The skip: the intro fades OUT to black over min(remaining, 45) ticks (type 1), the scene keeps
/// playing under it, the catch-up fast-forwards at black, and the gameplay viewer's fade-in
/// (−22, type 1) takes the screen back.
#[test]
fn the_skip_fades_out_then_the_handback_fades_in() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000004.bin", false).expect("load 07000004");
    for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        // One skip press through the real input path (the game's Input_Action(-1, 0x19, 1) poll).
        let sim = scene.sim.as_mut().unwrap();
        assert!(sim.drones.scripts.players.iter().any(|p| p.state == 2 && p.info[p.cur].as_ref().is_some_and(|i| i.playing())),
            "the intro's SP must be playing when the skip lands");
        sim.drones.scripts.skip_request = true;
    }
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    let Some(f) = scene.sim.as_ref().unwrap().drones.view_fade else { panic!("the skip must arm the view fade-out [M 0x1dac78:56-57]") };
    assert_eq!(f.mode, 1, "the skip's fade-out is a TYPE-1 fade [M 0x1dac78:57]");
    assert!((f.rate - 1.0 / 45.0).abs() < 1e-6, "remaining ≈ 460 → fade_len 45, rate 1/45 [M 0x1dac78:49-52], got {}", f.rate);
    assert!(f.alpha < 0.1, "the fade-out starts transparent");
    // Under the fade the scene keeps playing; at black the catch-up runs and the hand-back
    // fades the gameplay viewer back in. The black frame is observable either as the fade-out's
    // parked end (state 2, one drawn tick) or as the fade-in's first frame (alpha 1.0) — the
    // drain replaces the parked fade on the same tick, so track the peak alpha seen.
    let mut fade_in: Option<(f32, f32, bool)> = None;
    let mut peak = 0.0f32;
    for _ in 0..90 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(f) = sim.drones.view_fade {
            peak = peak.max(f.alpha);
            if f.rate < 0.0 { fade_in = Some((f.alpha, f.rate, sim.drones.view_fade_script)); break; }
        }
    }
    assert!(peak >= 0.999, "the skip must reach black before the fade-in [M 0x1dac78:49-57 + the catch-up guard], peak {peak}");
    let (alpha, rate, on_gameplay) = fade_in.expect("the tail's fade-in must follow the catch-up [M 0x1da998:101-110]");
    assert!((rate - -1.0 / 22.0).abs() < 1e-5, "the tail is Camera_SetFade(−22, 0, 0xff, 1) — rate −1/22 [M 0x1da998:106], got {rate}");
    assert!((alpha - 1.0).abs() < 1e-6, "a fade-in starts at alpha 1 (from black)");
    assert!(!on_gameplay, "the tail's fade-in arms on the GAMEPLAY viewer [M 0x1da998:106 — slot 0]");
    // The scene really skipped: the intro's ch 0x38 hand-back fired and the story SP took over.
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.scripts.players.iter().any(|p| p.scripts.contains(&0x0600_0082)),
        "the story scene 0x06000082 must load after the skipped intro [D 0x06000083 stream 6 event 0xd]");
}

#[test]
fn the_near_end_fade_of_both_camera_scripts_arms_and_dies_with_the_scene() {
    the_intro_and_story_near_end_fades()
}
