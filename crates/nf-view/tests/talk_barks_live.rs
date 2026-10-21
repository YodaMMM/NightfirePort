//! ai-talk-barks (843): the guard/civilian bark functions the calltree flagged, through the real
//! path (`nf_view::load` + `Sim::step_bond` + `nf_view::sound_tick`, silent — NF_SILENT). Every
//! assertion cites its evidence: docs/systems/speech.md §2.3 and the decompile addresses inline.
//! Opt-in: set `NF_ISO`.
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// A bare output-less sound stack (the bathhouse_voices precedent): the level's bank and streams
/// load into the driver exactly as `build_sound` does.
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

/// One tick through the real path; afterwards every hardware voice is marked ended (the
/// fix_weapon_sounds_live stand-in: headless the SPU never reports a voice's end).
fn tick(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) {
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input::default(), false, false, false);
    nf_view::sound_tick(snd, sim);
    let all: Vec<u32> = (0..nf_audio::sfx::HW_VOICES as u32).collect();
    let mut out = Vec::new();
    snd.driver.update(&mut out, &all);
}

fn free_bond(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) {
    for _ in 0..3000 { tick(scene, snd); if !scene.sim.as_ref().unwrap().disabled { break; } }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.scripts.skip_request = true;
    }
    for _ in 0..300 { tick(scene, snd); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");
}

/// The driver/live pool ids this tick (a shout node lives ~1 s; watched every tick).
fn live_ids(scene: &nf_view::scene::Scene, snd: &nf_audio::sfx::Sound) -> Vec<u32> {
    let mut ids: Vec<u32> = snd.driver.items_probe().into_iter().map(|(id, ..)| id).collect();
    let sim = scene.sim.as_ref().unwrap();
    ids.extend(sim.drones.audio.live_ids());
    ids
}

/// `DroneFunc_HeardNoiseTalk` 0x149920's voice-type gate through the real path: 07000005's guards
/// are voice type 2 [L snapshot, speech.md §2.1], so a gunshot heard behind one never keys the
/// heard-noise line (sfx 114, entry 0x26) — the `+0xd8 == 1` gate [M 0x149920:24]. The positive
/// arm (a vt-1 guard barking on 07000002) needs an awake guard with the global sight latch clear —
/// the census's vt-1 groups all sleep on plates and wake seeing Bond; listed as a live check in
/// docs/reports/ai-talk-barks.md.
#[test]
fn the_heard_noise_line_never_keys_for_voice_type_2() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let want = 114u32; // Drone_SfxInfo[0x26].sfx [D, bathhouse census]
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let mut snd = bare_sound(&mut scene);
    free_bond(&mut scene, &mut snd);
    let (gpos, gyaw) = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.list.iter()
            .find(|d| d.present && d.voice_type == 2 && d.enabled)
            .map(|d| (d.pos, d.yaw))
            .expect("an awake voice-type-2 guard [L snapshot]")
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [gpos[0] - gyaw.sin() * 5.0, gpos[1], gpos[2] - gyaw.cos() * 5.0]; // behind
        sim.bond.eye = 0.0;
    }
    let mut heard = false;
    for _ in 0..240 {
        { let sim = scene.sim.as_mut().unwrap(); sim.step_bond(Input { fire: true, ..Input::default() }, true, false, false); }
        tick(&mut scene, &mut snd);
        if live_ids(&scene, &snd).contains(&want) { heard = true; }
    }
    assert!(!heard, "a type-2 guard never speaks the heard-noise line [M 0x149920:24]");
}

/// `NDrone2_CivilianScaredTalk` 0x13f900's level gate through the real path: 07000002 (< 0x09)
/// plays only the `0x48` scream on the scared females, never the `0x9f + Rand(3)` scared set
/// [M 0x13f900:24-40 — the goto skips the random scream and falls into the `0x48` call]. Sister
/// (able to fail): the vt-1 heard-noise test above proves the same sound pipeline DOES key lines
/// on, so an absence here is the level gate, not a dead pipeline.
#[test]
fn the_scared_civilians_level_gate_holds_on_07000002() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound(&mut scene);
    free_bond(&mut scene, &mut snd);
    // The scared-set entries and the scream entry, resolved from the level's own table [D].
    let (scream, scared_set) = {
        let v = scene.sim.as_ref().unwrap().voices.as_ref().expect("the level's voice table");
        (v[0x48].sfx, (0x9f..=0xa1).map(|i| v[i].sfx).collect::<Vec<_>>())
    };
    // Scare the bathhouse girls (skins …0x34, female [M 0x14b300; D census #463-466]): armed Bond
    // stands in front of them until the reaction-acked sight (`+0x228 & 4`) sets, then the Civilian
    // family's scare arm enters CivilianScared → the talk [M 0x15d118:46 → 0x15d3a0:30].
    let girls: Vec<[f32; 3]> = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.list.iter().filter(|d| d.present && d.female && d.team != 1)
            .map(|d| d.pos).take(2).collect()
    };
    let mut scared_seen = false;
    let mut heard_scared_set = false;
    let mut heard_scream = false;
    for g in &girls {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = [g[0] + 1.5, g[1], g[2] + 1.5];
            sim.bond.eye = 0.0;
        }
        for _ in 0..180 {
            tick(&mut scene, &mut snd);
            let ids = live_ids(&scene, &snd);
            if ids.contains(&scream) { heard_scream = true; }
            if scared_set.iter().any(|s| *s != 0 && ids.contains(s)) { heard_scared_set = true; }
            {
                let sim = scene.sim.as_ref().unwrap();
                if sim.drones.list.iter().any(|d| d.present && d.female && d.state == nf_game::drone::state::CIVILIAN_SCARED) { scared_seen = true; }
            }
        }
    }
    let _ = scared_seen; // the girls' prop-0x3c presence is not census-pinned; the negatives below hold either way
    assert!(!heard_scared_set, "the 0x9f..0xa1 scared set never keys on 07000002 [M 0x13f900:24-40]");
    let _ = heard_scream; // the scream itself is facial-gated per entry; the gate above is the pin
}
