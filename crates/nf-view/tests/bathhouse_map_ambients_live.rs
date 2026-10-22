//! bathhouse-voices-02b: the map ambients on 07000002 were silent because the port built its
//! `MapSounds` from the MAIN map's block 0x28 — which is empty there — while the game's
//! `GMapSounds` global is reset and refilled by EVERY map's block during the create pass
//! (`Sound_LoadMapSounds` 0x1ddff8, called from `parsemap_handle_block_id` 0x1d1020 case 0x28
//! for each block `parsemap_parsenextblock` 0x1d12e8 walks; maps parse in archive order, so the
//! LAST block wins). 07000002's block lives on map 51: four ambients, among them the waterfall
//! id 0x5c (92) the playtest should hear under the bathhouse. The lead-1 side (the IOP voice
//! budget/priority reject, `SFXSetup` 0x4220) matches the port line for line — see
//! docs/systems/map-ambient-sounds.md for the rule and the shout's margin math.
//!
//! The tests drive the real path (`nf_view::load` + `Sim::step_bond` + `nf_view::sound_tick`,
//! NF_SILENT); `Sound.captured` exposes each batch's mixer commands. The headless stand-in ends
//! every non-looping voice each tick (the fix_weapon_sounds_live one, refined: a looped sample
//! never reports its end on hardware, so the looped ambients the game keeps are kept here too —
//! `Driver::looping_voice_ids`). Opt-in: set `NF_ISO`.

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

/// One real-path tick: the sim, the sound batch (captured), then the stand-in's end report for
/// every non-looping hardware voice. Returns the batch's commands.
fn tick(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) -> Vec<nf_audio::Cmd> {
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input::default(), false, false, false);
    nf_view::sound_tick(snd, sim);
    let keep = snd.driver.looping_voice_ids();
    let ended: Vec<u32> = (0..nf_audio::sfx::HW_VOICES as u32).filter(|v| !keep.contains(v)).collect();
    let mut out = Vec::new();
    snd.driver.update(&mut out, &ended);
    match snd.captured.as_ref() {
        Some(c) => { let mut g = c.lock().unwrap(); std::mem::take(&mut *g) }
        None => Vec::new(),
    }
}

/// The loudest 941-pitch Play in a batch (the bank's ambient samples 111/107/198/207 are all
/// pitch 941 [D SB_11.SHF]); the mixer's front gain for that voice.
fn front_941(cmds: &[nf_audio::Cmd]) -> Option<(f32, f32)> {
    cmds.iter().filter_map(|c| match c {
        nf_audio::Cmd::Play { pitch: 941, left, right, .. } => Some((*left, *right)), _ => None })
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
}

/// The parse-level pin: `Level::map_sounds` is the game's GMapSounds — the LAST block 0x28's
/// records, not the main map's. 07000002 carries two blocks (map 17's degenerate single id-0
/// record, then map 51's four); the game's global ends as map 51's four [M 0x1ddff8 + 0x1d1020].
#[test]
fn map_sounds_are_the_last_block_0x28_not_the_main_maps() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let i = nf_formats::iso9660::IsoImage::open(&iso).unwrap();
    let bin = {
        let (entries, files_bin) = nf_formats::disc::files_bin_entries(&i).unwrap();
        let fb = i.read_file(files_bin).unwrap();
        let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000002.bin")).expect("07000002 in FILES.BIN");
        fb[e.offset as usize..(e.offset + e.size) as usize].to_vec()
    };
    let level = nf_formats::level::Level::parse(&bin).expect("parse");
    // The main map's own list is empty here — the old source; the game's global is not.
    assert!(level.maps[0].sounds.is_empty(), "07000002's main map carries no block 0x28 [D]");
    assert_eq!(level.map_sounds.len(), 4, "the game's GMapSounds holds map 51's four ambients [D Sound_LoadMapSounds 0x1ddff8]");
    let ids: Vec<u32> = level.map_sounds.iter().map(|m| m.id & 0xfffff).collect();
    assert_eq!(ids, vec![0x24, 0x29, 0x5c, 0x87], "ids in archive order; 0x5c is the waterfall [D]");
    let w = level.map_sounds.iter().position(|m| m.id & 0xfffff == 0x5c).expect("the waterfall is one of the four");
    assert!((level.map_sounds[w].pos[0] - -54.96).abs() < 0.1 && level.map_sounds[w].pos[1].abs() < 0.1
        && (level.map_sounds[w].pos[2] - 1.01).abs() < 0.1, "the waterfall sits at the pool (−55, 0, 1) [D]");
    assert!((level.map_sounds[w].radius - 91.0).abs() < 0.5 && (level.map_sounds[w].inner - 81.9).abs() < 0.5,
        "start radius 91, inner 81.9 = 0.9·radius [D]");
    // Sister: the degenerate earlier block (map 17's single id-0 record) did NOT win — "first
    // non-empty" would have made GMapSounds that one-record list instead.
    let map17 = level.maps.iter().find(|m| m.sounds.len() == 1 && m.sounds[0].id & 0xfffff == 0).expect("map 17's degenerate block exists [D]");
    assert_eq!(map17.sounds.len(), 1);
    assert_ne!(level.map_sounds[0].id & 0xfffff, 0, "the last block 0x28 wins, not the first [M 0x1ddff8 resets per block]");
}

/// The waterfall and the level's other three map ambients key on through the real path now, each
/// with a non-zero mixer volume, and the game keeps their voices (looped samples never end, the
/// stop gate is 1.2·R² away) [M 0x1dd118; the volumes are the mixer's gain for the voice].
#[test]
fn the_map_ambients_including_the_waterfall_key_on_through_the_real_path() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound(&mut scene);
    let cap: std::sync::Arc<std::sync::Mutex<Vec<nf_audio::Cmd>>> = Default::default();
    snd.captured = Some(cap.clone());
    // Every ambient keys on (during the intro or right after), its batch carrying a Play at the
    // bank samples' pitch 941 with a non-zero front volume. Watch from tick 0: the intro's
    // scripted listener already crosses the start gates.
    let mut seen: Vec<(u32, usize, f32, f32)> = Vec::new();
    let (mut free, mut extra) = (false, 0u32);
    for f in 0..3600 {
        let cmds = tick(&mut scene, &mut snd);
        let ids = snd.driver.ids();
        for want in [36u32, 41, 92, 135] {
            if ids.contains(&want) && !seen.iter().any(|(w, ..)| *w == want) {
                let (l, r) = front_941(&cmds).unwrap_or((0.0, 0.0));
                seen.push((want, f, l, r));
            }
        }
        if !free && !scene.sim.as_ref().unwrap().disabled {
            free = true;
            let sim = scene.sim.as_mut().unwrap();
            sim.drones.scripts.skip_request = true;
        }
        if free { extra += 1; if extra > 300 && seen.len() == 4 { break; } }
    }
    assert!(free, "Bond is free after the intro");
    let got: Vec<u32> = seen.iter().map(|(w, ..)| *w).collect();
    // The key-on rule: `HandleMapSoundAllocation` 0x1dd118 (run per tick by `Sound_UpdateListeners`
    // 0x1dd278) starts every still-inactive GMapSounds entry whose listener squared distance is
    // inside the record's radius², one `Sound_Play3D(100.0f, pos, …)` per entry [M 0x1dd118:66-74],
    // the entries being the four records `Sound_LoadMapSounds` 0x1ddff8 loaded from map 51's block
    // 0x28 [D the four positions/radii above]. No per-id or budget carve-out exempts an ambient:
    // priority 100 tops the steal scan [M SFXSetup 0x4220, docs/systems/map-ambient-sounds.md §2-3].
    assert_eq!(got.len(), 4, "all four map ambients key on through the real path [M 0x1dd118:66-74, the records D]; got {seen:?}");
    for (want, f, l, r) in &seen {
        // The front gain rule: `PS2_SFXCalculate3D` 0x1060c (SFX.IRX) — the distance term clamps
        // to 0 inside the inner radius (dist − inner·10 < 0 → 0) so att = 100 [M 0001060c:55-72],
        // and the front gain is ((dot+1000)·att²/2000)·front·vol/100 [M :82-85]; the key-on
        // volume is 100.0f [M 0x1dd118:70] — so a keyed-on ambient not exactly at the rear pole
        // reaches the mixer with a positive front gain on both channels.
        assert!(*l > 0.0 && *r > 0.0, "ambient {want} keyed on at tick {f} but its front voice reached the mixer silent (l={l}, r={r}) [M PS2_SFXCalculate3D 0x1060c:55-85, vol 100 0x1dd118:70]");
    }
    // Sister: the game keeps their voices where it keeps them — the listener is inside the 1.2·R²
    // stop gate and the budget has room (4 ambient voices ≪ 15), so all four are STILL live after
    // further seconds; a wrong budget steal or stop gate would drop them [M 0x1dd118, 0x4220].
    for _ in 0..240 { tick(&mut scene, &mut snd); }
    let ids = snd.driver.ids();
    for want in [36u32, 41, 92, 135] {
        assert!(ids.contains(&want), "ambient {want} still live seconds later (the game keeps looped ambients; live: {ids:?})");
    }
    drop(snd);
    drop(scene);
}
