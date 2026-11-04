//! Headless NF_ISO tests for the 07000006 start's driver dialogue subtitles ("Can't I rest a
//! moment?..." / "Then it's six hours back, old man!..."): the spoken-line path
//! (`Sound_DoSubtitle` 0x1dde58) fed by the drones' start-script anim sound events
//! (`AnimProcessScriptCmds` 0x1c1b38 case 1 → `Sound_Play3D(100.0f, −1, −1, id, bone-0, …)`).
//!
//! The lines ride `Snd2Lbl` (0x2d25d8) — id 0xac → label 0x0600006e, id 0xae → 0x0600006f, both
//! rows carrying the 0x80000000 always-show bit [D Snd2Lbl idx 78/79] — so they post with the
//! Subtitles option OFF (the fresh-boot default [L ee_start_*.bin @0x2a37c8 = 0; M bootup
//! 0x1c7e78:34 clears it]), which is what the PS2's 07000006 capture shows. The sources:
//! driver #494 (skin 0x05000035) start script 0x0600011d op-1 frame 5 = sfx 0xac; escort #450
//! (skin 0x050002e) start script 0x0600011b op-1 frame 295 = sfx 0xae [D 07000006.bin kind-5
//! entries + the census; both wait ch 0xb9 raised by the truck-park TouchOnces that fire at the
//! spawn]. Durations: `fptoui(g_fSubtitleDurationModification × SFXOutputData[id]+0x10 × 60)` with
//! the modifier 0.92 [L ee_start_05.bin @0x30d084] and 0xac → 3.8312931 s, 0xae → 3.3215871 s
//! [D SFXOutputData] → 211 / 183 ticks.
//!
//! Opt-in via `NF_ISO`.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn step(sim: &mut nf_view::scene::Sim, ticks: u32) {
    for _ in 0..ticks { sim.step_bond(Default::default(), false, false, false); }
}

/// The driver conversation posts at the start scripts' sound-event ticks with the Subtitles option
/// OFF — the always-show rows' game rule (`Sound_DoSubtitle` 0x1dde58's own gate passes the bit;
/// the id-0xff `Text_AddMsg` add rides the `(type == 4 && id == 0xff)` disjunct [M 0x1b0c30]).
#[test]
fn driver_conversation_posts_with_the_option_off() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let Ok(mut scene) = nf_view::load(&path, "07000006.bin", false) else { eprintln!("07000006.bin does not load headless — skipped"); return };
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(!sim.drones.subtitle_track.enabled, "the fresh default is Off [M profile.rs; L ee_start_*]");
        // The context attached `Sound_DoSubtitle`'s tables from ACTION.ELF.
        assert!(sim.drones.snd2lbl.iter().any(|&(l, s)| l == 0x8600_006e && s == 0xac), "Snd2Lbl idx 78 [D]");
        assert!(sim.drones.snd2lbl.iter().any(|&(l, s)| l == 0x8600_006f && s == 0xae), "Snd2Lbl idx 79 [D]");
    }
    step(scene.sim.as_mut().unwrap(), 700);
    let sim = scene.sim.as_ref().unwrap();
    let track = &sim.drones.subtitle_track;
    let line = |label: u32| track.lines().iter().find(|l| l.label == label);
    // The whole gate conversation posts in script order through the FIFO chain [M Text_UpdateMsg
    // 0x1b11d8: only the displayed head's timer runs; a later add queues behind it]. The wall
    // guards' scripts (#487/#488) have no wait channel (p3 = 0 [D census]) so the frame-1 sound
    // 0xdc posts at the boot tick; the driver/escort wake on the spawn plates' ch 0xb9.
    let opener = line(0x0600_0070).expect("the wall guard's opener queued (frame-1 event of 0x06000124)");
    assert!(opener.start <= 3, "no wait channel: the script starts at boot (got start {})", opener.start);
    assert_eq!(opener.end - opener.start, 196, "0.92 × SFXOutputData[0xdc]+0x10 × 60 [D; L 0x30d084]");
    // The driver's line: start script 0x0600011d, op-1 sound 0xac at frame 5 [D anim entry]; the
    // start tick is the wake chain's (the ch-0xb9 plates → DSTATE wake → the script), so the pin
    // is the range + the duration, not a hand-computed tick.
    let driver = line(0x0600_006e).expect("the driver's line queued despite the option Off");
    assert!(opener.end <= driver.start && driver.start < 300, "after the opener, within the wake window (got start {})", driver.start);
    assert_eq!(driver.end - driver.start, 211, "0.92 × 3.8312931 s × 60 [D SFXOutputData; L 0x30d084]");
    // The escort's reply: script 0x0600011b's op-1 at frame 295 [D]; its sound fired while the
    // queue still displayed, so the FIFO starts it at a later line's end [M 0x1b11d8].
    let escort = line(0x0600_006f).expect("the escort's reply queued");
    assert!(escort.start >= driver.end, "the FIFO chained the reply behind the busy head [M 0x1b11d8]");
    assert_eq!(escort.end - escort.start, 183, "0.92 × 3.3215871 s × 60 [D]");
    // The wall guards' hip-flask exchange follows (0x06000126 frame 250, 0x06000124 frame 408).
    assert!(line(0x0600_0071).is_some() && line(0x0600_0072).is_some(), "the guards' chat queued");
    // The per-tick query the draw code calls.
    assert_eq!(track.active_subtitle(driver.start), Some(0x0600_006e));
    assert_eq!(track.active_subtitle(driver.start + 210), Some(0x0600_006e));
    assert_ne!(track.active_subtitle(driver.start + 211), Some(0x0600_006e), "the head's timer hit 0 and the queue advanced [M Text_UpdateMsg 0x1b11d8]");
    assert_eq!(track.active_subtitle(escort.start), Some(0x0600_006f));
    // The HUD drain took the same adds (the game's single Text_AddMsg call).
    assert!(sim.hud.subs.iter().flatten().next().is_some(), "the spoken lines reached the HUD drain");
}

/// Sister: the same lines with the option ON — the track is identical (the spoken adds' gate is
/// `Sound_DoSubtitle`'s, not the queue's: the id-0xff add passes either way [M 0x1b0c30; the
/// option only reaches DoSubtitle's own gate [M 0x1dde58:21-23], which the always-show bit
/// bypasses]).
#[test]
fn the_option_on_leaves_the_always_show_lines_unchanged() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let Ok(mut scene) = nf_view::load(&path, "07000006.bin", false) else { eprintln!("07000006.bin does not load headless — skipped"); return };
    scene.sim.as_mut().unwrap().drones.subtitle_track.enabled = true;
    step(scene.sim.as_mut().unwrap(), 700);
    let sim = scene.sim.as_ref().unwrap();
    let track = &sim.drones.subtitle_track;
    let driver = track.lines().iter().find(|l| l.label == 0x0600_006e).expect("the driver's line queued");
    let escort = track.lines().iter().find(|l| l.label == 0x0600_006f).expect("the escort's reply queued");
    assert!(driver.start < 300 && escort.start >= driver.end);
    assert_eq!((driver.end - driver.start, escort.end - escort.start), (211, 183));
}

