//! speech-facial-fallback (945): `NDrone2_SpeechSFX` 0x141b00's hashtable-miss arm — a facial-id
//! line whose script is absent from the level plays DIRECT (facialAnimId := 0, permanent) and
//! returns a handle; a present script stays on the (unported) FACIAL path — no handle [M
//! 0x141b00; speech.md §1.3]. The presence test's port source: `Data.scripts`, the level
//! archive's kind-5 0x06 anim-script entries [D census (zz probe, 2026-10-22): the 61 speech
//! facial hashes are 0x06-namespace; 07000005 carries 0 of them, 07000007 carries 54 — the
//! absent set on 07000007 includes the bond-talk lines 0x70/0x72/0x73 (sfx 397/399/400)].
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
/// join]); every present-script draw (54 of the 61 facial scripts ride the level archive [D])
/// goes FACIAL — no handle, nothing sounds, the bookkeeping does not move [M :15-17].
#[test]
fn bond_talk_plays_the_absent_script_lines_and_stays_silent_on_present_ones() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let mut snd = bare_sound(&mut scene);
    free_bond(&mut scene, &mut snd);
    let absent = [397u32, 399, 400]; // set 0xd's absent-script lines on 07000007 [D census]
    let mut target: Option<usize> = None;
    let mut played: Vec<u32> = Vec::new();
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
            if d.bond_talk_count > 0 || d.bond_talk_cooldown > 0 { advanced = true; }
        }
        if played.len() >= 2 && advanced { break; }
    }
    assert!(target.is_some(), "the crosshair reached a type-13 civilian");
    assert!(advanced, "the bond-talk bookkeeping advanced on a miss level (the bug: speech_request always returned 0)");
    assert!(!played.is_empty(), "an absent-script bond-talk line played [M 0x141b00:18-28]");
    for sfx in &played {
        assert!(absent.contains(sfx), "sfx {sfx} played — only the absent-script lines {absent:?} may sound [D census; M 0x141b00:15-17 keeps present ones on the silent FACIAL path]");
    }
}

/// Sister (the same level, the same path, a present-script-only talker): voice type 14 runs
/// BondTalk case 0xe — lines `0x84 + rand(9)`, ALL of whose facial scripts ride 07000007 [D
/// census: 132..=140 all present] — so every draw takes the FACIAL path: no handle, no voice,
/// and `do_bond_talk`'s bookkeeping never advances (`h == 0 && !+0x53C` → return [M 0x141b00:15-17;
/// 0x1446b0 case 0xe → 0x144488:50-53]). The reach: Bond seeds at the nearest type-13 (whose
/// head-on ray a type-14 neighbour blocks — the census geometry), then locks onto the aimed
/// type-14.
#[test]
fn a_present_script_talker_never_plays_nor_advances() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let mut snd = bare_sound(&mut scene);
    free_bond(&mut scene, &mut snd);
    // the census sanity: every set-0xe line's script is present on this level [D]
    {
        let sim = scene.sim.as_ref().unwrap();
        let voices = sim.voices.as_deref().unwrap();
        for i in 132..=140 {
            assert!(voices[i].facial != 0 && sim.scripts.contains_key(&voices[i].facial),
                "line {i} must be facial with a present script [D census]");
        }
    }
    let mut target: Option<usize> = None;
    let mut any_voice = false;
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
            // The push dodge: the guest push [M NDrone2_DoCollision 0x149ec8:11-34 + Collide_Jointy
            // 0x1ec180:229-266] shoves Bond off a spot that sits inside a live capsule, and the
            // mid-tick displacement swings the aim ray off the talker. The real player's move is a
            // step-back: keep the SAME seed line (the vt-14 blocker sits on it — the census
            // geometry above) and step the radius out until the spot is clear of every live
            // capsule, so the ray forms and holds. The asserts are untouched.
            let clear_r = {
                let caps: Vec<([f32; 2], f32)> = sim.drones.list.iter()
                    .filter(|x| x.present && x.enabled && !x.corpse)
                    .map(|x| {
                        let r = if x.voice_type == 0xc { 0.55f32 } else { 0.4f32 };
                        ([x.pos[0], x.pos[2]], r)
                    })
                    .collect();
                let mut chosen: Option<f32> = None;
                let mut best = (2.2f32, -1.0f32);
                for &r_ext in &[2.2f32, 2.5, 2.8, 3.1, 3.4] {
                    let px = npos[0] + nyaw.sin() * r_ext;
                    let pz = npos[2] + nyaw.cos() * r_ext;
                    let mut min_d = f32::MAX;
                    for &(cp, cr) in &caps {
                        let dd = ((px - cp[0]).powi(2) + (pz - cp[1]).powi(2)).sqrt() - cr;
                        if dd < min_d { min_d = dd; }
                    }
                    if min_d >= 1.05 { chosen = Some(r_ext); break; }
                    if min_d > best.1 { best = (r_ext, min_d); }
                }
                chosen.unwrap_or(best.0)
            };
            let sim = scene.sim.as_mut().unwrap();
            let (s2, c2) = (nyaw.sin(), nyaw.cos());
            sim.bond.pos = [npos[0] + s2 * clear_r, npos[1], npos[2] + c2 * clear_r];
            sim.bond.yaw = (npos[0] - sim.bond.pos[0]).atan2(npos[2] - sim.bond.pos[2]);
            let dy = 0.4f32 - 1.6; // mid-body aim [M 0x1a7810's cylinder]
            sim.bond.pitch = (dy / clear_r.hypot(1.2)).asin();
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
            if d.voice.is_some() { any_voice = true; }
            assert_eq!(d.bond_talk_done, 0, "a present-script draw advanced the bookkeeping [M 0x141b00:15-17; 0x144488:50-53]");
        }
    }
    assert!(target.is_some(), "the crosshair reached a type-14 talker");
    assert!(!any_voice, "a present-script line played [D census; M 0x141b00:15-17]");
}
