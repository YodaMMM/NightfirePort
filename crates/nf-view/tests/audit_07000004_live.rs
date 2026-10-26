//! level-04-audit: the 07000004 cut-scene dialogue pin — the intro (0x06000083) and the story
//! scene (0x06000082) play their scripted sound streams through the real path.
//!
//! [D] the archive's NIS decode (dump_07000004_audit): the intro's stream 8 starts sound 0x50a at
//! its frame 57 (flat 0, obj 0 → the previous stream's object, else flat); the story's streams
//! start 0x5d @43, 0x5e @298, 0xa5 @551 and @784, 0xaa @589, 0x5c3 @595, 0x255 @616, 0x134 @654,
//! 0xa9 @700 and the NIS music 0x172 @546/@710.
//! [M] `Script_SoundStart` 0x1d8820 — flat byte or no stream object → `Sound_Play` (2-D), else
//! `Sound_Play3D(100.0, id, obj+0xc0)`; the 0x172 start sets `SfxNisMusicTrigger` (the level's
//! music updater jumps) [M SFXStart 0x115a68 + the port's Request::Sound drain in drone.rs].
//! The port emits these through `Script_Run` case 0x16 [M 0x1d8548:92-95] → `Request::Sound`
//! [M nis.rs op 0x16] → the audio pool; this test pins the 04 lines' ids and tick windows.

use nf_game::sound::Cmd;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn the_intro_and_story_scenes_play_their_scripted_lines() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000004.bin", false).expect("load 07000004");
    let mut starts: Vec<(u32, u32)> = Vec::new(); // (tick, id) of every EE Start
    let mut music_trigger_at: Option<u32> = None;
    let mut story_live: Vec<(u32, u8)> = Vec::new(); // (tick, stream) the story's live script-sound entries
    for t in 0..1750u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.step_bond(nf_game::player::Input::default(), false, false, false);
            // The EE-side bookkeeping the viewer's sound_tick runs every frame: `Sounds::update`
            // emits the Start commands for the tick's new pool nodes (the test runs headless).
            let (pos, f) = (sim.bond.head(), sim.bond.look());
            let norm = { let l = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6); [f[2] / l, 0.0, -f[0] / l] };
            sim.drones.audio.update(sim.sfx.as_ref(), (pos, norm, [0.0, 1.0, 0.0], f), &|_| false);
            for c in sim.drones.audio.take_commands() {
                if let Cmd::Start { id, .. } = c { starts.push((t, id)); }
            }
            if music_trigger_at.is_none() && sim.drones.music.nis_trigger {
                music_trigger_at = Some(t);
            }
            for ((h, s), _, _) in sim.drones.script_sounds.iter().filter(|((h, _), _, _)| *h == 0x0600_0082) {
                story_live.push((t, *s));
            }
        }
    }
    let started = |id: u32| starts.iter().filter(|(_, i)| *i == id).map(|(t, _)| *t).collect::<Vec<_>>();
    // The intro's single line (stream 8 @57 [D]).
    let intro = started(0x50a);
    assert_eq!(intro.len(), 1, "the intro 0x06000083 starts its one line 0x50a exactly once [D stream 8; M Script_SoundStart 0x1d8820], got {intro:?}");
    assert!((50..70).contains(&intro[0]), "the intro line starts at its authored frame 57 (+ the load settle), got tick {}", intro[0]);
    // The story's dialogue lines, each at least once inside the scene [D streams 6/12-18].
    for id in [0x5d, 0x5e, 0xa5, 0xa9, 0xaa, 0x134, 0x255, 0x5c3] {
        let ts = started(id);
        assert!(!ts.is_empty(), "the story scene 0x06000082 starts sound {id:#x} at least once [D; M 0x1d8820]");
    }
    // The story's FIRST line (0x5d @43 of the scene [D stream 12]) — the intro runs 500 ticks
    // [D 0x06000083 end 500], so the line lands ≈543.
    let first = started(0x5d);
    assert!((520..620).contains(&first[0]),
        "the story's first line lands ≈500 + 43 (the scene starts when the intro writes ch 0x38 [D stream 6]), got tick {:?}", first.first());
    // The NIS music (0x172 @546 [D stream 6]) — the trigger for the level's music updater [M].
    assert!(!started(0x172).is_empty(), "the NIS music 0x172 starts [D stream 6]");
    assert!(music_trigger_at.is_some(), "the 0x172 start sets the NIS music trigger [M SFXStart 0x115a68; the Request::Sound drain]");
}
