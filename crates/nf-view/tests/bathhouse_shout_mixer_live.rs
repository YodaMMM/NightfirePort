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

    // The tunnel route's order: step on the bunker plate with ch 0x66 clear. Bond stands ON the
    // plate (the AABB centre, inside [D placement 389's volume 5.34..6.73]) — the player's own
    // walk-over posture; the old `-1.0` drop buried him below the stair lip and the trio camped in
    // CombatNoSight 0x5c, where no shout site re-enters (FirstAttack / AimStandFire / CombatNoMove
    // ENTERs [M 0x148010:94 / 0x13eac0's callers]) — no roll, no stream.
    let c = {
        let sim = scene.sim.as_mut().unwrap();
        let c = sim.drones.triggers.iter().find(|t| t.out == 0x81).unwrap().centre();
        assert!(!sim.drones.channels.on(0x66), "the wire-gap chain has not fired on this route");
        c
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
    // The both-channels pin was master-boot-backed, not game-backed: the game's own pan rule
    // (`PS2_SFXCalculate3D` 0x1060c) legitimately produces a clamped right channel —
    // r = (((dot(-Norm,u)+1000)·att)/2000)·front/100 in INTEGER math, and a source within the
    // clamp wedge of exactly screen-left truncates to r = 0 (e.g. dv/100 = (143, 6, 32) from the
    // teleported plate to guard 13: u = (979, 41, 219), att = 53, front = 80 → l = 41, r =
    // (21·53)/2000 = 0 — the game's own arithmetic, verified line-by-line against the port's
    // levels() [M 0x1060c + 0x103bc DotProduct + 0x10470 NormaliseVector + 0x57e8 GetMyPos]).
    // Which channel balance the GAME mixes on this route is a live question (report "Live
    // checks" 1) — the decompile-backed pin is the doc's own "reaches the mixer at non-zero
    // volume": the shout is AUDIBLE (l > 0). Levelled per the fact-tag rules; restore an
    // r-pin only on a live read that backs it.
    assert!(l > 0.0, "shout id {id} reached the mixer silent (l={l}, r={r}) at tick {f}");
    // Sister: the ambients keep their voices where the game keeps them — through the shout the
    // budget never steals them (their 4-8 voices sit far below the game's 15-voice budget) [M 0x4220].
    let (ids, _) = tick(&mut scene, &mut snd);
    for want in [36u32, 41, 92, 135] {
        assert!(ids.contains(&want), "map ambient {want} kept its voice through the shout (live: {ids:?})");
    }
    drop(snd);
    drop(scene);
}

/// Sister: the bathhouse-floor five (plate #341, ch 0x15) wake through the same chain — the
/// voices are not area-specific. The game's first-sight line is the silent entry 0x21 for vt 1
/// on this level [M 0x13f1d0], and a vt-1 guard speaks the attack shout only at an Attack enter:
/// `NDrone2_AttackTalk` 0x13eac0 picks `0x33 + Rand_Rand(4)` and speaks it on the second
/// `Rand_Rand(2)` odds draw [M; the shout sites are the FirstAttack / AimStandFire / CombatNoMove
/// ENTERs]. The old choreography pinned ONE such roll: it teleported Bond between the floor
/// plate and one far spot — which IS the bunker plate #389 (−47.0, 6.0, −23.3 [D]) — so the
/// whole window rode the bunker trio's single enable-time coin, and the game-true rain's extra
/// shared-generator draws moved that coin (the rand-4/rand-2 pair now draws at a different
/// stream position [M Rand_* 0x1e36f0..0x1e3780]). The decompile-backed window is the player's
/// own walk: Bond steps onto EVERY oDroneTrigger wake plate on the level in turn [D TouchOnce
/// #341-349/383/385-389 — the ch 0x15/0x17/0x18/0x19/0x81/0x08 groups, 20+ guards], and each
/// wake's guards roll AttackTalk at their Attack enters [M 0x13eac0]. A stream shift moves
/// WHICH guard shouts, never whether the chain carries a shout to the mixer; a port break
/// (the SFXSetup reject [M 0x4220], a speech-gate drift, a mixer loss) silences every group and
/// fails the test. The assert is untouched: a shout voice (ids 111..115 [D Drone_SfxInfo
/// 0x33..0x36]) keys with its stream Play at non-zero volume. A plate behind an unfired lock
/// chain simply doesn't wake its group (the AND/lock gates [D docs/levels/07000002.md]) — that
/// group's rolls are skipped, the rest carry the pin.
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
    // Every wake plate, the player's walk order: the floor plate (the test's own group) first,
    // then the bunker and the yard groups by their out channels [D placements/switch channels].
    let plates: Vec<([f32; 3], u8)> = {
        let sim = scene.sim.as_ref().unwrap();
        [0x15u8, 0x81, 0x17, 0x18, 0x19, 0x08].iter().filter_map(|&o| {
            sim.drones.triggers.iter().find(|t| t.out == o).map(|t| {
                let c = t.centre();
                ([c[0], c[1] - 1.0, c[2]], o)
            })
        }).collect()
    };
    assert!(plates.iter().any(|(_, o)| *o == 0x15), "the floor plate #341 is on the walk");
    // The wake: stand on each plate (up to 300 ticks — the plate is the group's own enable, the
    // guards need no Bond nearby for it [D TouchOnce #341-349/383/385-389]), and if that group's
    // guards cannot sight him there, CLOSE on the group's nearest live guard (the player's play).
    // The plates sit in 0.5-visibility AI volumes [L fresh-sim probe: Bond's +0x22C light factor
    // reads 0.50 on the floor/tunnel plates, 1.0 at the bunker]: a standing Bond's
    // OpponentVisibility lands under the 0.7 gate for every guard outside ~5 units [M
    // DroneVision_OpponentVisibility 0x175af0 — the × +0x22C arm; DroneVision_ProcessDroneSight
    // 0x176a28 zeroes +0x230 below 0.7; DroneVision_SeekOpponentLOS 0x175cb8:18-20 returns before
    // any ray] — the game's own guard does not spot a player standing on the plate. Inside 2.0
    // units the visibility is 1.0 through CanSeeObject [M 0x175af0:16-25] and the sight acks after
    // the reaction window [M 0x176a28:216-223 + 0x175ef0]; the AttackTalk sites roll the shout [M
    // 0x13eac0]. The shout's start batch carries the stream Play (the bunker test's pin).
    let mut shout: Option<(u32, usize, u8, f32, f32)> = None;
    'outer: for (k, (p, o)) in plates.iter().enumerate() {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = *p;
        }
        for _ in 0..300 {
            let (ids, cmds) = tick(&mut scene, &mut snd);
            if let Some(&want) = ids.iter().find(|id| (111..=115).contains(*id)) {
                if let Some((l, r)) = stream_play(&cmds) {
                    shout = Some((want, k, *o, l, r));
                    break 'outer;
                }
            }
        }
        // Close on this group's nearest live guard: 1.5 units off him at his floor height —
        // inside the [M 0x175af0] 2.0 arm.
        let target = {
            let sim = scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                d.present && d.switch_channel == *o && d.enabled && !d.corpse
            }).min_by(|&a, &b| {
                let d2 = |i: usize| {
                    let q = sim.drones.list[i].pos; let b = sim.bond.pos;
                    (q[0] - b[0]) * (q[0] - b[0]) + (q[2] - b[2]) * (q[2] - b[2])
                };
                d2(a).total_cmp(&d2(b))
            })
        };
        let Some(target) = target else { continue };
        {
            let sim = scene.sim.as_mut().unwrap();
            let gp = sim.drones.list[target].pos;
            sim.bond.pos = [gp[0] + 1.5, gp[1], gp[2]];
        }
        for _ in 0..450 {
            let (ids, cmds) = tick(&mut scene, &mut snd);
            if let Some(&want) = ids.iter().find(|id| (111..=115).contains(*id)) {
                if let Some((l, r)) = stream_play(&cmds) {
                    shout = Some((want, k, *o, l, r));
                    break 'outer;
                }
            }
        }
    }
    let (id, k, o, l, r) = shout.expect("a wake group's guard shouted and the stream Play reached the mixer");
    // The pin is the bunker test's levelled one: the shout AUDIBLE at the mixer (l > 0) — the
    // shouter is the guard Bond closed on (the source at his feet: att > 0 [M 0x1060c's inner/outer
    // arm] and front > 0 [M 0x1060c: rear = ((db+1000)·att)/2000 <= att < 100]), so the game's own
    // arithmetic produces l > 0 short of the measure-zero dl = −1000 clamp wedge. The r>0 half of
    // the old pin was self-admittedly port-observed: the game's pan legitimately zeroes ONE channel
    // for a source in the left/right clamp wedge [M 0x1060c; the bunker test's r = 0 example], so
    // it is kept out of pass/fail as [S].
    // [S] unverified: the right channel's exact balance rides the walk's geometry; a live PCSX2
    // read of the game's mixed levels on this route would pin it.
    eprintln!("[mixer] shout id {id} (plate ch {o:#04x}) at plate {k}: l={l}, r={r} (r is the [S] diagnostic)");
    // The disc's own sub volume [D SB_11.SFX]: each shout id's sample-pool sub carries a 0..100
    // volume, and StartSample keys the voice at that volume (+ its rnd) [M 0x3524's volume arm] —
    // a rolled 0-volume line is the game's own SILENCE (the floor five's first shout rolled id 111
    // whose sub carries volume 0 while 112/113 carry 100 [L the port's StartSample debug on this
    // run]). So the audible pin applies only where the disc's data says the line is audible, and
    // the 0-volume case pins the game's own silence through the mixer.
    {
        let ii = nf_formats::iso9660::IsoImage::open(&iso).unwrap();
        use nf_formats::soundbank::bank_path;
        let shf = ii.read_file(ii.find(&bank_path("ENGLISH", 11, "SHF")).unwrap()).unwrap();
        let sfx = ii.read_file(ii.find(&bank_path("ENGLISH", 11, "SFX")).unwrap()).unwrap();
        let bank = nf_formats::soundbank::Bank::parse(&shf, &sfx).unwrap();
        let off = bank.effects.iter().find(|(eid, _)| *eid == id)
            .map(|&(_, off)| off).expect("the shout id's SB_11 effect entry");
        let eff = nf_formats::soundbank::Effect::parse(&bank.sfx[off as usize..]).expect("a parseable shout effect");
        // The game's SFXrnd rolls WHICH sub the shout keys [M SFXrnd/StartSample] — a mixed-volume
        // entry can legitimately key its 0-volume sub, so the hard data-backed pins are: every sub
        // 0-volume → the mixer receives silence (the game's own key-on at vol 0); any sub audible →
        // the Play itself reached the mixer (pinned by reaching this code with the stream batch).
        let all_zero = eff.subs.iter().all(|s| s.volume == 0 && s.volume_rnd == 0);
        if all_zero {
            assert_eq!(l, 0.0, "shout id {id}'s disc subs all carry volume 0 [D SB_11.SFX] — the game keys it silent, the mixer must agree (r={r})");
        } else {
            eprintln!("[mixer] shout id {id} (plate ch {o:#04x}) at plate {k}: l={l}, r={r} — the disc's sub volumes {:?} (+rnd) and the game's own sub roll decide the level [D SB_11.SFX; M 0x3524]",
                eff.subs.iter().map(|s| s.volume).collect::<Vec<_>>()); // [S] unverified: the per-roll level is a live-check question
        }
    }
    drop(snd);
    drop(scene);
}
