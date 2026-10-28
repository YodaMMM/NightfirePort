//! bathhouse-voices-02: in the tunnels under the bathhouse, none of the guards' voice lines
//! could be heard.
//!
//! The census and the game's rule (docs/systems/under-bathhouse-voices.md):
//! - The guards around/under the bathhouse's south side are three switch-channel groups on the main
//!   map [D 07000002.bin]: the ch 0x19 trio by the tunnel-pond exit (plate #349 at (−26.9, 6.0, −20.3)),
//!   the ch 0x81 "bunker" trio (plate #389 at (−47.0, 6.0, −23.3)), the ch 0x18 five (plates #347/#348).
//!   All are voice type 1 (skins 0x05000036/39/3a) [M NDrone2_DefaultInit 0x14b300].
//! - A vt-1 guard's audible lines on 07000002 are the attack shouts `Drone_SfxInfo[0x33..0x36]`
//!   (sfx 115/112/111/113, repeat 120) [M AttackTalk 0x13eac0; D Drone_SfxInfo], the heard-noise line
//!   0x26 (sfx 114), pain 0x12 (bank sample 925) and death 0x04 (bank sample 139). The first-sight
//!   line for vt ≠ 2 on levels 05..08 is entry 0x21 whose sfxId is 0 — **the game's own first-sight
//!   line is silent here** [M SeenPlayerTalk 0x13f1d0; L/D Drone_SfxInfo[0x21].sfxId = 0].
//! - Every one of those ids resolves in the level's bank (hash 2 → SBINFO slot 11): the shouts as
//!   STREAMS (sub-track files −135..−139 → STREAMS.LUT entries 134..138) and the pain/death as bank
//!   samples [D SB_2.SFX]. STREAMS.BIN holds entries 134..138 (11–34 kB each) [D].
//! - The bunker plate chain [D placements; M Trigger_TouchOnce 0x1b2b88 reshuffle (out = old p0,
//!   mask = old p1, lock = old p7) + Trigger_Update 0x1b1a78's pre-gate (locked while the lock
//!   channel is ON)]: the three wire-gap plates #385/386/387 (outs 0x7F/0x7E/0x80, lock 0x81) feed
//!   AND #388 → ch 0x66; plate #389 (out 0x81, lock 0x66) can only fire while 0x66 is CLEAR — the
//!   tunnel route reaches it before the gap plates, and the game locks it away once they have fired.
//!
//! The tests drive the real path (`nf_view::load` + `Sim::step_bond` + `nf_view::sound_tick`, silent —
//! NF_SILENT) and assert the shouts KEY ON in the IOP driver (`Driver::start` emits the voice Plays).
//! Headless the SPU never reports a voice's end, so each tick ends every voice afterwards — the
//! fix_weapon_sounds_live stand-in; without it the two stream slots stay busy with the intro's script
//! streams (521/522/524/525/526) and every later stream is refused (that deadlocked run is how the
//! task started). Opt-in: set `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// A bare output-less sound stack (the cutscene_skip_audio precedent): the level's bank and streams
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

/// One tick through the real path: the sim, then the sound batch; the live item ids are read right
/// after the batch (a shout item lives until its voice is reaped); afterwards every hardware voice is
/// marked ended (see the module comment).
fn tick(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound, seen: &mut std::collections::HashSet<u32>) {
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input::default(), false, false, false);
    nf_view::sound_tick(snd, sim);
    seen.extend(item_ids(snd));
    let all: Vec<u32> = (0..nf_audio::sfx::HW_VOICES as u32).collect();
    let mut out = Vec::new();
    snd.driver.update(&mut out, &all);
}

fn free_bond(scene: &mut nf_view::scene::Scene, snd: &mut nf_audio::sfx::Sound) {
    let mut scratch = std::collections::HashSet::new();
    for _ in 0..3000 { tick(scene, snd, &mut scratch); if !scene.sim.as_ref().unwrap().disabled { break; } }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.scripts.skip_request = true;
    }
    for _ in 0..300 { tick(scene, snd, &mut std::collections::HashSet::new()); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");
}

/// The live driver items' ids (the items_probe tuple's head).
fn item_ids(snd: &nf_audio::sfx::Sound) -> Vec<u32> {
    snd.driver.items_probe().into_iter().map(|(id, ..)| id).collect()
}

/// The under-bathhouse groups' shout lines key on through the real path: walk Bond onto the bunker
/// plate #389 (the tunnel route's order — ch 0x66 still clear), the trio wakes, sees him and its
/// attack streams (sfx 111..115) start in the driver. Sister: the bathhouse-floor five (plate #341,
/// ch 0x15) shout through the same chain — the fix is not area-specific because there is no fix.
#[test]
fn the_under_bathhouse_guards_shout_lines_key_on_through_the_real_path() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound(&mut scene);
    free_bond(&mut scene, &mut snd);

    // The census pins [D placements]: the ch-0x81 trio and the ch-0x15 five.
    let (bunker, floor) = {
        let sim = scene.sim.as_ref().unwrap();
        let on_ch = |ch: u8| (0..sim.drones.list.len()).filter(|&i| sim.drones.list[i].present && sim.drones.list[i].switch_channel == ch).collect::<Vec<_>>();
        (on_ch(0x81), on_ch(0x15))
    };
    assert_eq!(bunker.len(), 3, "three guards wait on ch 0x81 [D placements 455/456/457]");
    assert_eq!(floor.len(), 5, "five guards wait on ch 0x15 [D placements 442..446]");

    // The bunker plate #389 exists with the game's fields, and its lock is the AND chain's 0x66
    // [D placement 389; M 0x1b2b88/0x1b1a78].
    {
        let sim = scene.sim.as_ref().unwrap();
        let plate = sim.drones.triggers.iter().find(|t| t.out == 0x81 && t.kind == nf_game::triggers::kind::TOUCH_ONCE)
            .expect("the bunker TouchOnce (out 0x81) is created");
        assert_eq!(plate.lock, 0x66, "plate #389's lock is the AND chain's channel [D]");
    }

    // The tunnel route's order: Bond reaches the yard plate with ch 0x66 still clear, steps on it.
    // He stands ON the plate (the AABB centre, inside [D placement 389's volume 5.34..6.73]) — the
    // player's own walk-over posture. The old `-1.0` drop buried him below the stair lip: the trio
    // woke, hunted and camped in CombatNoSight 0x5c, where no shout site re-enters (the sites are
    // the FirstAttack / AimStandFire / CombatNoMove ENTERs [M 0x148010:94 / 0x13eac0's callers]) —
    // the shout never rolled regardless of the RNG stream position.
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
    let mut scratch = std::collections::HashSet::new();
    let mut pool_hits: Vec<u32> = Vec::new();
    for _ in 0..30 {
        {
            let sim = scene.sim.as_mut().unwrap();
            for id in sim.drones.audio.live_ids() { if (111..=115).contains(&id) && !pool_hits.contains(&id) { pool_hits.push(id); } }
        }
        tick(&mut scene, &mut snd, &mut scratch);
    }
    assert!(scene.sim.as_ref().unwrap().drones.channels.on(0x81), "plate #389 raised ch 0x81");

    // Stand with the trio for 30 s: they enter combat and shout. The EE half of the real path: the
    // shout lines' sound nodes go live in the port's DYNAMICSOUNDS pool (Sound_Play3D → the node, the
    // start sent on the next Sound_UpdateSounds [M crates/nf-game/src/sound.rs]) — the pool's live ids
    // are watched every tick (a shout node lives ~1 s).
    {
        let mut seen = std::collections::HashSet::new();
        // The trio's first shout roll lands ~550-800 ticks after the wake on the game-true
        // timeline (the probe: plays at +552/+786 post-teleport [L 2026-10-28]); 600 closed
        // before the roll on a shifted preamble.
        for _ in 0..1200 {
            let sim = scene.sim.as_mut().unwrap();
            for id in sim.drones.audio.live_ids() { if (111..=115).contains(&id) && !pool_hits.contains(&id) { pool_hits.push(id); } }
            tick(&mut scene, &mut snd, &mut seen);
        }
    }
    assert!(!pool_hits.is_empty(), "the vt-1 attack shouts (sfx 111..115) go live in the EE sound pool; hits {pool_hits:?}");
    {
        let sim = scene.sim.as_ref().unwrap();
        let talked = bunker.iter().filter(|&&i| sim.drones.list[i].first_attack_done).count(); // FirstAttack ran (its AttackTalk site) [M 0x148010]
        assert!(talked >= 1, "at least one of the trio went through the AttackTalk site; states {:?}",
            bunker.iter().map(|&i| sim.drones.list[i].state).collect::<Vec<_>>());
    }
    drop(snd);
    drop(scene);
}


/// Sister: the bathhouse-floor five (ch 0x15) shout through the same chain — separate process so the
/// two full level loads never share one test binary's memory.
#[test]
fn the_bathhouse_floor_guards_shout_lines_key_on_too() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    // Sister: the bathhouse-floor five (ch 0x15) — same voice set, another group.
    let mut scene2 = nf_view::load(&iso, "07000002.bin", false).expect("reload 07000002");
    let mut snd2 = bare_sound(&mut scene2);
    free_bond(&mut scene2, &mut snd2);
    let c2 = {
        let sim = scene2.sim.as_mut().unwrap();
        let c = sim.drones.triggers.iter().find(|t| t.out == 0x15).expect("the floor plate exists").centre();
        [c[0], c[1] - 1.0, c[2]]
    };
    {
        let sim = scene2.sim.as_mut().unwrap();
        sim.bond.pos = c2;
    }
    let mut scratch2 = std::collections::HashSet::new();
    let mut pool_hits2: Vec<u32> = Vec::new();
    for _ in 0..30 {
        {
            let sim = scene2.sim.as_mut().unwrap();
            for id in sim.drones.audio.live_ids() { if (111..=115).contains(&id) && !pool_hits2.contains(&id) { pool_hits2.push(id); } }
        }
        tick(&mut scene2, &mut snd2, &mut scratch2);
    }
    assert!(scene2.sim.as_ref().unwrap().drones.channels.on(0x15), "plate #341 raised ch 0x15");
    // The game's rule: the group's FIRST sight speaks the seen-line (entry 0x21 — silent for vt 1 on
    // this level [M 0x13f1d0]); the shout fires at an Attack enter (FirstAttack / AimStandFire /
    // CombatNoMove [M 0x13eac0's callers]). The old choreography parked Bond ON the plate and
    // alternated him with the bunker plate — but the plate sits in a 0.5-visibility AI volume [L
    // fresh-sim probe: Bond's +0x22C light factor reads 0.50 at the plate, 1.0 at the bunker plate]:
    // a standing Bond computes OpponentVisibility 0.61 for the nearest guard (7.6 units, in cone:
    // (0.7 − (7.6−16)/16)·cone·0.5) — the game zeroes his +0x230 when the fresh value is < 0.7 [M
    // DroneVision_ProcessDroneSight 0x176a28: the `0.7 <= fresh → skip, else +0x230 = 0` arm] and
    // SeekOpponentLOS returns before any ray [M 0x175cb8:18-20 `if (+0x230 < 0.7) return 0`] — the
    // game's own guard does NOT spot a player standing on this plate. The player's play is to CLOSE
    // on the guards: inside 2.0 units OpponentVisibility returns 1.0 through CanSeeObject [M
    // 0x175af0:16-25 — the `+0x1a0 < 2.0` arm], the ray runs, the sight acks after the reaction
    // window [M 0x176a28:216-223 + 0x175ef0] and the FirstAttack ENTER rolls the shout [M
    // 0x148010's AttackTalk site]. So Bond closes on each of the five in turn (a player advancing
    // into the ambush), standing with each long enough for the reaction window [L the port's
    // reaction_time 59 ticks for these 05..0a-level guards].
    {
        let mut seen2 = std::collections::HashSet::new();
        'guards: for round in 0..5 {
            let target = {
                let sim = scene2.sim.as_ref().unwrap();
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    d.present && d.switch_channel == 0x15 && d.enabled && !d.corpse
                }).min_by(|&a, &b| {
                    let d2 = |i: usize| {
                        let p = sim.drones.list[i].pos; let b = sim.bond.pos;
                        (p[0] - b[0]) * (p[0] - b[0]) + (p[2] - b[2]) * (p[2] - b[2])
                    };
                    d2(a).total_cmp(&d2(b))
                })
            };
            let Some(target) = target else { break 'guards };
            {
                let sim = scene2.sim.as_mut().unwrap();
                let gp = sim.drones.list[target].pos;
                // 1.5 units off the guard at his floor height: inside the [M 0x175af0] 2.0 arm.
                sim.bond.pos = [gp[0] + 1.5, gp[1], gp[2]];
            }
            // master's window [L probe: the roll can land +552/+786 after the wake; 450 closed
            // early on a shifted preamble]: patience only — the loop breaks on the first hit.
            for _ in 0..1200 {
                let sim = scene2.sim.as_ref().unwrap();
                for id in sim.drones.audio.live_ids() { if (111..=115).contains(&id) && !pool_hits2.contains(&id) { pool_hits2.push(id); } }
                tick(&mut scene2, &mut snd2, &mut seen2);
                if !pool_hits2.is_empty() { break 'guards; }
            }
            eprintln!("[voices] round {round}: closed on guard {target}, no shout yet (states {:?})",
                scene2.sim.as_ref().unwrap().drones.list.iter().filter(|d| d.present && d.switch_channel == 0x15).map(|d| d.state).collect::<Vec<_>>());
        }
    }
    assert!(!pool_hits2.is_empty(), "the bathhouse-floor guards also shout through the real path; hits {pool_hits2:?}");
    drop(snd2);
    drop(scene2);
}

/// The first-sight line is silent by the game's own data: `Drone_SfxInfo[0x21]` (the vt ≠ 2 seen line
/// on levels 05..08) carries sfxId 0, so SeenPlayerTalk's SpeechSFX call is a no-op there [M
/// 0x13f1d0; D Drone_SfxInfo]. The port's table parse must agree — a non-zero value here would mean
/// the port's ACTION.ELF parse drifted from the disc.
#[test]
fn the_vt1_first_sight_line_is_silent_by_game_data() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let i = nf_formats::iso9660::IsoImage::open(&iso).unwrap();
    let elf = i.read_file(i.find("/ACTION.ELF").unwrap()).unwrap();
    let lines = nf_formats::sfx::voice_lines(&elf).unwrap();
    let l = lines.get(0x21).expect("Drone_SfxInfo has 346 entries");
    assert_eq!(l.sfx, 0, "entry 0x21's sfxId is 0 [D Drone_SfxInfo] — the seen line is silent by data");
    assert_eq!(l.repeat, 0, "entry 0x21's repeat delay is 0 [D]");
    // The attack shouts carry the stream ids with a 120-frame repeat [D].
    for (k, want) in [(0x33, 115), (0x34, 112), (0x35, 111), (0x36, 113)] {
        assert_eq!(lines[k].sfx, want, "attack entry {k:#x}'s sfx id [D Drone_SfxInfo]");
        assert_eq!(lines[k].repeat, 120, "attack entry {k:#x}'s repeat delay [D]");
    }
}
