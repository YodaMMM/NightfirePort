//! speech-facial-fallback (945), re-derived onto the game's advance-and-play behaviour: the
//! absent-script draws keep the hashtable-miss arm — `facialAnimId := 0` (permanent), the line
//! plays DIRECT and returns a handle [M 0x141b00:18-28] — while a present-script draw now takes
//! the ported FACIAL arm [M 0x141b00:15-35]: `AnimScriptPeekSfx` 0x1c6ca8 finds the script's own
//! op-1 sound, the script id queues at +0x4c with +0x37 = 1 and +0x48 = now + 0x14
//! (`SFXPreCache(sfx, 0x14)`), ControlSTANDARD appends it at T+21 [M speech.md §1.4], and the
//! line sounds from the FACIAL SCRIPT's own op-1 command — its sfx id, which the census shows
//! equals the entry's sfxId [D census (probe_facial_ops, this level): every present 07 BondTalk
//! facial script's first op-1 sfx == the entry's sfxId]. The facial arm returns 0 (no handle,
//! `+0x4D0` untouched [M :35]) and stamps the entry once +0x53c stands, so the caller's
//! bookkeeping advances [M 0x1446b0 case 0xd join; 0x144488:50-63]. The presence test's port
//! source: `Data.scripts`, the level archive's kind-5 0x06 anim-script entries [D census
//! (zz probe, 2026-10-22): the 61 speech facial hashes are 0x06-namespace; 07000005 carries 0 of
//! them, 07000007 carries 54 — the absent set on 07000007 includes the bond-talk lines
//! 0x70/0x72/0x73 (sfx 397/399/400)].
//! Through the real path (`nf_view::load` + `Sim::step_bond` + `nf_view::sound_tick`, silent).
//! Opt-in: set `NF_ISO`.
use nf_game::player::Input;
use nf_game::trig::angle_difference;

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

/// The 07000007 bond-talk path through the real path (`nf_view::load` + `Sim::step_bond` +
/// `nf_view::sound_tick`, silent): Bond stands 2.2 units from a voice-type-13 civilian (BondTalk
/// set 0xd [M 0x1446b0 case 0xd]) with the crosshair on him, cycling the approach angle until
/// the ray reaches HIM (a vt-14 crowd neighbour blocks the head-on ray). Every `rand(11)` draw
/// keys line `0x58 + r*4 + variant` [M]; with the NPC's variant != 1 the absent-script draws are
/// exactly lines 0x70/0x72/0x73 (sfx 397/399/400 [D census]) — those play DIRECT (the
/// hashtable-miss arm [M 0x141b00:18-28]) and advance the bookkeeping (h != 0 [M case 0xd
/// join]); every present-script draw takes the ported FACIAL arm — no handle, the line sounds
/// later from the facial script's own op-1 (== the entry's sfxId [D census]) and the
/// bookkeeping advances once +0x53c stands [M 0x141b00:32-35 + the case 0xd join].
#[test]
fn bond_talk_plays_the_absent_script_lines_direct_and_the_present_ones_through_the_facial() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let mut snd = bare_sound(&mut scene);
    free_bond(&mut scene, &mut snd);
    let absent = [397u32, 399, 400]; // set 0xd's absent-script lines on 07000007 [D census]
    // the present-script set-0xd sfx ids [D census: 0x58..0x6f + 0x71 — 463, 374..396, 398]
    let present: Vec<u32> = {
        let sim = scene.sim.as_ref().unwrap();
        let voices = sim.voices.as_deref().unwrap();
        (0x58..0x84).filter(|&i| sim.scripts.contains_key(&voices[i].facial))
            .map(|i| voices[i].sfx).collect()
    };
    let mut target: Option<usize> = None;
    let mut played: Vec<u32> = Vec::new();
    let mut facial: Vec<u32> = Vec::new();
    let mut advanced = false;
    for t in 0..4500u32 {
        {
            let sim = scene.sim.as_ref().unwrap();
            let i = target.unwrap_or_else(|| {
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.voice_type == 13
                        && d.health > 0.0 && d.skin & 0xffff != 0x22)
                    .min_by_key(|(_, d)| ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)) as u32)
                    .map(|(x, _)| x).expect("an awake type-13 civilian [D census]")
            });
            let d = &sim.drones.list[i];
            let (npos, nyaw) = (d.pos, d.yaw);
            let sim = scene.sim.as_mut().unwrap();
            let off = [0.0f32, 0.7, -0.7, 1.4, -1.4][(t / 90) as usize % 5]; // dodge crowd blockers
            let a = nyaw + off;
            let (s2, c2) = (a.sin(), a.cos());
            sim.bond.pos = [npos[0] + s2 * 2.2, npos[1], npos[2] + c2 * 2.2];
            sim.bond.yaw = (npos[0] - sim.bond.pos[0]).atan2(npos[2] - sim.bond.pos[2]);
            let dy = 0.4f32 - 1.6; // aim mid-body: the aimed-at cylinder spans feet-1.0..+1.8 [M 0x1a7810]
            sim.bond.pitch = (dy / 2.2f32.hypot(1.2)).asin();
        }
        tick(&mut scene, &mut snd);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(ai) = sim.drones.list.iter().position(|x| x.aimed_at) {
            let ad = &sim.drones.list[ai];
            if ad.voice_type == 13 && ad.skin & 0xffff != 0x22 { target = Some(ai); }
        }
        if let Some(i) = target {
            let d = &sim.drones.list[i];
            if let Some(v) = d.voice { if !played.contains(&v.1) { played.push(v.1); } }
            for s in &d.facial_played { if !facial.contains(s) { facial.push(*s); } }
            if d.bond_talk_count > 0 || d.bond_talk_cooldown > 0 { advanced = true; }
        }
        if played.len() >= 1 && facial.len() >= 2 && advanced { break; }
    }
    assert!(target.is_some(), "the crosshair reached a type-13 civilian");
    assert!(advanced, "the bond-talk bookkeeping advanced (both arms advance [M 0x1446b0 case 0xd join])");
    assert!(!played.is_empty(), "an absent-script bond-talk line played DIRECT [M 0x141b00:18-28]");
    for sfx in &played {
        assert!(absent.contains(sfx), "sfx {sfx} played — the DIRECT arm plays only the absent-script lines {absent:?} [D census]");
    }
    assert!(!facial.is_empty(), "a present-script line sounded through the facial pipeline [M 0x141b00:15-35]");
    for sfx in &facial {
        assert!(present.contains(sfx), "facial sfx {sfx} played — the FACIAL arm sounds the script's own op-1, == the entry's sfxId of a present line [D census; M 0x1c6ca8]");
        assert!(!absent.contains(sfx), "an absent-script line took the FACIAL arm (impossible: the miss arm goes DIRECT [M 0x141b00:18-19])");
    }
}

/// Sister (the same level, the same path, a present-script-only talker): voice type 14 runs
/// BondTalk case 0xe — lines `0x84 + rand(9)`, ALL of whose facial scripts ride 07000007 [D
/// census: 132..=140 all present, each carrying its own op-1 == the entry's sfxId] — so every
/// draw takes the FACIAL arm: the line sounds through the facial script's own op-1, the
/// bookkeeping ADVANCES once +0x53c stands [M 0x141b00:32-35; 0x1446b0 case 0xe → 0x144488:50-63],
/// and `+0x4D0` stays clear — a facial line never stores a handle [M 0x141b00:35]. The reach:
/// Bond seeds at the nearest type-13 (whose head-on ray a type-14 neighbour blocks — the census
/// geometry), then locks onto the aimed type-14.
#[test]
fn a_present_script_talker_plays_through_the_facial_and_advances() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let mut snd = bare_sound(&mut scene);
    free_bond(&mut scene, &mut snd);
    // the census sanity: every set-0xe line's script is present on this level, and its own op-1
    // sound == the entry's sfxId [D census; M 0x1c6ca8's first-op-1 walk]
    {
        let sim = scene.sim.as_ref().unwrap();
        let voices = sim.voices.as_deref().unwrap();
        for i in 132..=140 {
            assert!(voices[i].facial != 0 && sim.scripts.contains_key(&voices[i].facial),
                "line {i} must be facial with a present script [D census]");
            let op1 = sim.scripts.get(&voices[i].facial).and_then(|s| s.commands.iter()
                .find(|c| c.op == 1).and_then(|c| c.args.get(1)).copied()).unwrap_or(0) as u32;
            assert_eq!(op1, voices[i].sfx as u32, "line {i}'s facial script's own op-1 sfx [D census]");
        }
    }
    let mut target: Option<usize> = None;
    let mut any_facial = false;
    let mut done = 0u32;
    for t in 0..2700u32 {
        {
            let sim = scene.sim.as_ref().unwrap();
            let i = target.unwrap_or_else(|| {
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.voice_type == 13
                        && d.health > 0.0 && d.skin & 0xffff != 0x22)
                    .min_by_key(|(_, d)| ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)) as u32)
                    .map(|(x, _)| x).expect("an awake type-13 seed [D census]")
            });
            let d = &sim.drones.list[i];
            let (npos, nyaw) = (d.pos, d.yaw);
            let sim = scene.sim.as_mut().unwrap();
            let (s2, c2) = (nyaw.sin(), nyaw.cos());
            sim.bond.pos = [npos[0] + s2 * 2.2, npos[1], npos[2] + c2 * 2.2];
            sim.bond.yaw = (npos[0] - sim.bond.pos[0]).atan2(npos[2] - sim.bond.pos[2]);
            let dy = 0.4f32 - 1.6; // mid-body aim [M 0x1a7810's cylinder]
            sim.bond.pitch = (dy / 2.2f32.hypot(1.2)).asin();
        }
        tick(&mut scene, &mut snd);
        let sim = scene.sim.as_ref().unwrap();
        if target.is_none() {
            if let Some(ai) = sim.drones.list.iter().position(|x| x.aimed_at) {
                if sim.drones.list[ai].voice_type == 14 { target = Some(ai); }
            }
        }
        if let Some(i) = target {
            let d = &sim.drones.list[i];
            any_facial |= !d.facial_played.is_empty();
            done = done.max(d.bond_talk_done);
            assert!(d.voice.is_none(), "a facial line stored +0x4D0 (the facial arm never returns or stores a handle [M 0x141b00:35])");
        }
        if any_facial && done > 0 { break; }
    }
    assert!(target.is_some(), "the crosshair reached a type-14 talker");
    assert!(any_facial, "a present-script line sounded through the facial script's own op-1 [M 0x141b00:15-35; D census op-1 == sfxId]");
    assert!(done > 0, "the bookkeeping advanced once +0x53c stood [M 0x144488:50-63]");
}
