//! bathhouse-voices-02b lead 1: the IOP voice budget/priority reject does NOT refuse the
//! under-bathhouse trio's shout — the game's `SFXSetup` (0x4220) rejects a tracking-2 sound with
//! priority < 100 only while `proximity·priority < activeVoices·2000/budget` (budget =
//! 15 − started/400, clamped 2000, decaying ×9/10 per `SFXUpdate` 0x4ee4); the shouts
//! (ids 111..115 [D Drone_SfxInfo 0x33..0x36]) carry priority 99, inner 2, outer 50 [D SB_11.SFX]
//! and the guards are metres away, so 100·99 = 9900 dwarfs the reject threshold (4 ambient
//! voices → 533; even 20 voices → 2666). The port implements the same formula — the test pins
//! the outcome on the real path: the shout keys on and reaches the mixer at non-zero volume
//! while the map ambients hold their voices. Sister: the ch-0x15 bathhouse-floor five shout
//! through the same chain.
//!
//! The headless stand-in (every non-looping voice reports its end each tick — the stream slots
//! would otherwise stay busy with the intro's script streams) keeps the looped ambients
//! (`Driver::looping_voice_ids`): a looped sample never reports its end on hardware. The shout
//! item itself lives one tick under the stand-in, so the pin is the batch of its start tick:
//! the stream Play (STREAM pitch 0x759 = 1881 [M `StartSample` 0x3524]) at non-zero volume.
//! Opt-in: set `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn bare_sound(scene: &mut nf_view::scene::Scene) -> nf_audio::sfx::Sound {
    let mut snd = nf_audio::sfx::Sound {
        driver: Default::default(),
        output: None,
        captured: None,
        music_files: None,
        music: None,
        music_pending: Vec::new(),
    };
    if let Some((slot, bank, sbf)) = scene.sound_bank.take() { snd.driver.load_bank(slot, bank, sbf); }
    if let Some((lut, bin)) = scene.streams.take() { snd.driver.load_streams(lut, std::sync::Arc::new(bin)); }
    snd
}

/// One real-path tick with the stand-in's end report; returns (live ids BEFORE the stand-in reaps
/// the one-tick stream items, the batch's commands).
fn tick(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) -> (Vec<u32>, Vec<nf_audio::Cmd>) {
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input::default(), false, false, false);
    nf_view::sound_tick(snd, sim);
    let ids = snd.driver.ids();
    let keep = snd.driver.looping_voice_ids();
    let ended: Vec<u32> = (0..nf_audio::sfx::HW_VOICES as u32).filter(|v| !keep.contains(v)).collect();
    let mut out = Vec::new();
    snd.driver.update(&mut out, &ended);
    let cmds = match snd.captured.as_ref() {
        Some(c) => { let mut g = c.lock().unwrap(); std::mem::take(&mut *g) }
        None => Vec::new(),
    };
    (ids, cmds)
}

/// The loudest stream-pitch Play in a batch: (left, right).
fn stream_play(cmds: &[nf_audio::Cmd]) -> Option<(f32, f32)> {
    cmds.iter().filter_map(|c| match c {
        nf_audio::Cmd::Play { pitch: 1881, left, right, .. } => Some((*left, *right)), _ => None })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
}

/// The tunnel route arms the bunker trio (plate #389, ch 0x81 — the wire-gap chain's ch 0x66 is
/// still clear), the guards wake, enter Attack and shout; the shout's stream keys on and reaches
/// the mixer at non-zero volume, and the four map ambients keep their voices throughout.
#[test]
fn the_bunker_trio_shout_reaches_the_mixer_with_the_ambients_live() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound(&mut scene);
    let cap: std::sync::Arc<std::sync::Mutex<Vec<nf_audio::Cmd>>> = Default::default();
    snd.captured = Some(cap.clone());
    for _ in 0..3000 { let _ = tick(&mut scene, &mut snd); if !scene.sim.as_ref().unwrap().disabled { break; } }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.scripts.skip_request = true;
    }
    for _ in 0..300 { let _ = tick(&mut scene, &mut snd); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");

    // The tunnel route's order: step on the bunker plate with ch 0x66 clear.
    let c = {
        let sim = scene.sim.as_mut().unwrap();
        let c = sim.drones.triggers.iter().find(|t| t.out == 0x81).unwrap().centre();
        assert!(!sim.drones.channels.on(0x66), "the wire-gap chain has not fired on this route");
        [c[0], c[1] - 1.0, c[2]]
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = c;
    }
    // Stand with the trio for up to 20 s; the shout's start-tick batch carries the stream Play.
    let mut shout: Option<(u32, usize, f32, f32)> = None;
    for f in 0..1200 {
        let (ids, cmds) = tick(&mut scene, &mut snd);
        if let Some(&want) = ids.iter().find(|id| (111..=115).contains(*id)) {
            if let Some((l, r)) = stream_play(&cmds) {
                shout = Some((want, f, l, r));
                break;
            }
        }
    }
    let (id, f, l, r) = shout.expect("a bunker guard's attack shout keyed on and its stream Play reached the mixer");
    assert!(l > 0.0 && r > 0.0, "shout id {id} reached the mixer silent (l={l}, r={r}) at tick {f}");
    // Sister: the ambients keep their voices where the game keeps them — through the shout the
    // budget never steals them (their 4-8 voices sit far below the game's 15-voice budget) [M 0x4220].
    let (ids, _) = tick(&mut scene, &mut snd);
    for want in [36u32, 41, 92, 135] {
        assert!(ids.contains(&want), "map ambient {want} kept its voice through the shout (live: {ids:?})");
    }
    drop(snd);
    drop(scene);
}

/// Sister: the bathhouse-floor five (plate #341, ch 0x15) shout through the same chain — the
/// voices are not area-specific. The game's first-sight line is the silent entry 0x21 for vt 1
/// on this level [M 0x13f1d0], so Bond steps away and back to force a later Attack enter.
#[test]
fn the_bathhouse_floor_guards_shout_reaches_the_mixer_too() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound(&mut scene);
    let cap: std::sync::Arc<std::sync::Mutex<Vec<nf_audio::Cmd>>> = Default::default();
    snd.captured = Some(cap.clone());
    for _ in 0..3000 { let _ = tick(&mut scene, &mut snd); if !scene.sim.as_ref().unwrap().disabled { break; } }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.scripts.skip_request = true;
    }
    for _ in 0..300 { let _ = tick(&mut scene, &mut snd); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");
    let c = {
        let sim = scene.sim.as_mut().unwrap();
        let c = sim.drones.triggers.iter().find(|t| t.out == 0x15).expect("the floor plate exists").centre();
        [c[0], c[1] - 1.0, c[2]]
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = c;
    }
    let mut shout: Option<(u32, usize, f32, f32)> = None;
    'outer: for k in 0..4 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = if k % 2 == 1 { [-47.0, 6.0, -23.3] } else { c };
        }
        for _ in 0..300 {
            let (ids, cmds) = tick(&mut scene, &mut snd);
            if let Some(&want) = ids.iter().find(|id| (111..=115).contains(*id)) {
                if let Some((l, r)) = stream_play(&cmds) {
                    shout = Some((want, k, l, r));
                    break 'outer;
                }
            }
        }
    }
    let (id, k, l, r) = shout.expect("a bathhouse-floor guard's shout keyed on and reached the mixer");
    assert!(l > 0.0 && r > 0.0, "floor shout id {id} reached the mixer silent (l={l}, r={r}) at cycle {k}");
    drop(snd);
    drop(scene);
}
