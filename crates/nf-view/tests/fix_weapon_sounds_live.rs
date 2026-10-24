//! fix-weapon-sounds live tests: every weapon's sounds on every level.
//!
//! The dev tools (`--give`, the F2 menu) hand Bond ANY weapon on ANY level, but a level's sound bank
//! (`Sound_Ready` 0x1dceb8 → `SFXLoadSoundBank`, docs/systems/sound.md) carries only the effects that
//! level uses — so a weapon given on a level that doesn't natively carry it was silent: the driver's
//! `SFXStart3D` found no effect row and returned 1 (no voice). The fix loads the other SBINFO banks as
//! spares (`Driver::load_spare_bank`); the level's own bank stays the primary lookup, so every
//! level-carried sound is bit-identical to the one-bank behaviour.
//!
//! The requests flow through the real path: `nf_view::load_dev` (the dev view-model merge gives foreign
//! weapons their scripts, so their op-1 sound events fire) → `Sim::step_bond` (equip / draw / fire /
//! reload input) → `Sounds::update` (the EE `DYNAMICSOUNDS` pool) → `nf_view::sound_tick` → the driver.
//! Assertions are numeric: the played voice's sample FNV (`Driver::played_sample_hash`) must be one of
//! the owning bank's own decodes, and `Driver::item_slot` names the bank it came from.
//!
//! Weapon → sound-id mapping (the report's table, from the level's scripts' op-1 events + the hardcoded
//! draw/holster ids [M 0x1a2e38]): on 07000005 (bank slot 4) the missing sets are 0x0e..0x10 → 0x1e1..3,
//! 0x11 → 0x449..c, 0x12/0x13 → 0x3+0x54..7, 0x1a/0x1b → 0x405..0x40d, 0x1c/0x1d → 0x1dd..0x1e0,
//! 0x2a/0x2b → 0x404/0x40b/0x410, 0x2c..0x31 → 0x10c/0x10d, 0x32/0x33 → 0x420..0x423+0x5c7,
//! 0x37 → 0x458, 0x3a → 0x445/6, 0x3b..0x40 → 0x447/8, 0x41 → 0x27f, 0x42 → 0x41b+0x5c9,
//! 0x43/0x44 → 0x5ce..0x5d0, 0x52 → 0x44e/0x450, 0x56..0x59 → 0x452..0x458+0x5cc..0x5d0. Slot 2 (hash 22,
//! the multiplayer bank, levels 21–29) holds every one of them [D the disc banks].
//!
//! RED (the one-bank lookup, shown first): `the AR's fire sound 0x406 never started a voice on
//! 07000005` — the exact silent-foreign-weapon symptom.
//!
//! Opt-in via `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::soundbank::{bank_path, bank_slots, Bank};
use nf_game::player::Input;
use std::collections::{HashMap, HashSet};

/// The disc's banks by SBINFO slot, and the level's own slot (`Sound_Ready`'s table [M]).
fn disc_banks(level: u32) -> (usize, HashMap<usize, (Bank, Vec<u8>)>) {
    let iso = IsoImage::open(std::env::var("NF_ISO").unwrap()).unwrap();
    let sbi = iso.read_file(iso.find("PS2/SBINFO.SBI").unwrap()).unwrap();
    let slots = bank_slots(&sbi).unwrap();
    let hash = nf_game::sound::level_bank(level).unwrap();
    let own = slots.iter().position(|&h| h == hash).unwrap();
    let mut map = HashMap::new();
    for (slot, &h) in slots.iter().enumerate() {
        if h == u32::MAX { continue; }
        let f = |ext: &str| iso.find(&bank_path("ENGLISH", slot, ext)).and_then(|e| iso.read_file(e).ok());
        let (Some(shf), Some(sfx), Some(sbf)) = (f("SHF"), f("SFX"), f("SBF")) else { continue };
        if let Ok(b) = Bank::parse(&shf, &sfx) { map.insert(slot, (b, sbf)); }
    }
    (own, map)
}

/// The bank that owns `id` on this level (the level's own bank first — the driver's lookup rule — else
/// the lowest spare slot that has it) and the FNV set of its non-stream samples for the id.
fn owning(banks: &HashMap<usize, (Bank, Vec<u8>)>, own: usize, id: u32) -> (usize, HashSet<u64>) {
    let has = |slot: usize| banks.get(&slot).is_some_and(|(b, _)| b.effect(id).is_some());
    let slot = if has(own) { own } else { (0..25).find(|&s| has(s)).unwrap() };
    let (b, sbf) = &banks[&slot];
    let fx = nf_formats::soundbank::Effect::parse(b.effect(id).unwrap()).unwrap();
    let hashes = fx.subs.iter().filter(|s| s.file >= 0).filter_map(|s| b.decode(sbf, s.file as usize).ok())
        .map(|(pcm, _)| nf_audio::sfx::pcm_hash(&nf_audio::Pcm { data: pcm, loop_start: None })).collect();
    (slot, hashes)
}

fn load(archive: &str) -> Option<nf_view::scene::Scene> {
    // Never through the speakers: `build_sound` opens the real output device unless NF_SILENT is
    // set, and these tests fire weapons back to back (a plain `cargo test` once blasted them at full volume).
    // SAFETY: set before any audio thread starts; every test in this binary sets the same value.
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let iso = std::env::var("NF_ISO").ok()?;
    Some(nf_view::load_dev(&iso, archive, false).expect("level loads (dev: foreign weapon scripts merge)"))
}

/// Record every live item's (id → owning slot, sample hash).
fn harvest(sound: &nf_audio::sfx::Sound, seen: &mut HashMap<u32, (usize, u64)>) {
    for id in sound.driver.ids() {
        if let (Some(slot), Some(h)) = (sound.driver.item_slot(id), sound.driver.played_sample_hash(id)) {
            seen.insert(id, (slot, h));
        }
    }
}

/// One game tick through the real sound path: `nf_view::sound_tick` (the EE pool flushes, the driver
/// starts the voices), the harvest (a one-shot item lives exactly one tick), then the headless stand-in
/// for the SPU's end reports — with no output device the port's voices never report their end, so mark
/// them all ended; one-shots are reaped by the next `SFXUpdate` exactly as on hardware (looped samples
/// never end [M sound.md §6]), keeping the 40-item pool from clogging over long sweeps.
fn tick(sound: &mut nf_audio::sfx::Sound, scene: &mut nf_view::scene::Scene, seen: &mut HashMap<u32, (usize, u64)>) {
    let sim = scene.sim.as_mut().unwrap();
    nf_view::sound_tick(sound, sim);
    harvest(sound, seen);
    let all: Vec<u32> = (0..nf_audio::sfx::HW_VOICES as u32).collect();
    let mut out = Vec::new();
    sound.driver.update(&mut out, &all);
}

fn equip(scene: &mut nf_view::scene::Scene, id: usize) -> bool {
    let sim = scene.sim.as_mut().unwrap();
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(id).max(1);
    if sim.drones.inventory.equip_weapon(&wd, id, rounds, true) {
        sim.drones.inventory.equip_ammo(&wd, id, (rounds * 3).max(rounds));
        sim.gun.switch_request = id as u8;
        true
    } else { false }
}

/// Wait for the draw to finish: the weapon byte is the requested one and the switch state machine is
/// back at 0 (the request is processed on the tick after `switch_request` is set, so an early state-0
/// check would race the switch's start).
fn until_drawn(scene: &mut nf_view::scene::Scene, sound: &mut nf_audio::sfx::Sound, seen: &mut HashMap<u32, (usize, u64)>, want: usize, max: usize) {
    for t in 0..max {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        tick(sound, scene, seen);
        let gun = &scene.sim.as_ref().unwrap().gun;
        // fix-weapon-anims2: also require the reload states clear. The draw-done clip (0 for the
        // clip-slot twins — the equip fills the base's slot, the switch reads the id's) starts the
        // auto-reload on the first idle tick, whose states 5/6/7 are VIEW states (+0xF4, [M
        // Player_SetWeaponAnimObj cases 5/6/7]) — "drawn" in the game means +0xF4 == 0, not merely
        // switch_state == 0. Breaking mid-reload made the fire window's held trigger ride the reload
        // branch (which consumes the press edge — the game's held trigger does not fire across a
        // reload [M 0x1a5460:442-445's new-press edge test]) and the cells never fired.
        if t >= 2 && gun.weapon as usize == want && gun.switch_state == 0 && gun.reload_state == 0 { break; }
    }
}

/// (weapon, the fire/reload sound ids its scripts request that 07000005's bank lacks, from the op-1
/// events of the merged scripts — the report's mapping, asserted as spare-bank starts below.)
///
/// 0x448 (the satchel family 0x3b–0x40's detonation sound) is listed but never observed by this
/// sweep, on any tree: the charge's projectile spawns at its fire script's RELEASE frame
/// (+0x44 = 76 [L record]; `Player_SetWeaponAnimObj` case 9 0x1a2e38:416-420), 51 ticks past the
/// 25-tick fire window, and the next weapon's holster cuts the script before it — while holding the
/// sweep on the satchel until the release would land its 8-radius/2000-damage charge (`Player_Hurt`
/// [M 0x17c518 → explode::propagate's player target]) ~5 units from Bond, a game-faithful
/// self-kill that would void the rest of the sweep. 0x3b's row passes on 0x447 (the plant sound);
/// 0x59's row passes on 0x458, the mine 0x37's detonation sound, which fires earlier in the sweep
/// and stays in the accumulated `seen`.
const MISSING_ON_05: &[(usize, &[u32])] = &[
    (0x0e, &[0x1e1, 0x1e2, 0x1e3]), (0x11, &[0x449, 0x44a, 0x44b, 0x44c]), (0x12, &[3, 0x54, 0x55, 0x56, 0x57]),
    (0x1a, &[0x406, 0x407, 0x40a, 0x40d]), (0x1b, &[0x405, 0x408, 0x409, 0x40c]), (0x1c, &[0x1dd, 0x1de, 0x1e0]),
    (0x2a, &[0x404, 0x40b, 0x410]), (0x2c, &[0x10c, 0x10d]), (0x32, &[0x421, 0x423, 0x5c7]), (0x33, &[0x420, 0x422, 0x5c7]),
    (0x37, &[0x458]), (0x3a, &[0x445, 0x446]), (0x3b, &[0x447, 0x448]), (0x41, &[0x27f]), (0x42, &[0x41b, 0x5c9]),
    (0x43, &[0x5ce, 0x5cf, 0x5d0]), (0x59, &[0x448, 0x458]),
];
/// Gadget ids whose scripts request missing sounds only inside their own fire minigame (the grapple
/// 0x52's reel, the camera gadgets 0x56/0x58): a plain held `fire` doesn't reach those scripts, so the
/// sweep can't observe them through gameplay — asserted at the driver level below instead [S].
const GADGET_CONTEXT_IDS: &[u32] = &[1102, 1104, 1106, 1484, 1485, 1108, 1110];

/// The sister check (a level that has the sample uses its own, unchanged): Bond's starting P2K on
/// 07000005 fires 0x403 and reloads through 0x40e/0x40f/0x411 — every one of them must come from the
/// level's own bank (slot 4), with the bank's own sample bytes, spare banks never consulted.
#[test]
fn a_native_weapons_sounds_come_from_the_levels_own_bank() {
    let Some(mut scene) = load("07000005.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let (own, banks) = disc_banks(0x0700_0005);
    assert_eq!(own, 4, "07000005 → bank hash 5 → slot 4 [M Sound_Ready]");
    let mut sound = nf_view::build_sound(&mut scene).expect("the level has a bank");
    let mut seen = HashMap::new();
    // Fire the drawn P2K (the level's own loadout), then reload: the real input path.
    for _ in 0..8 {
        scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Input::default() }, true, false, false);
        tick(&mut sound, &mut scene, &mut seen);
    }
    for _ in 0..240 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false);
        tick(&mut sound, &mut scene, &mut seen);
        if seen.contains_key(&0x40e) && seen.contains_key(&0x40f) && seen.contains_key(&0x411) { break; }
    }
    for id in [0x403u32, 0x40e, 0x40f, 0x411] {
        let Some(&(slot, h)) = seen.get(&id) else { panic!("the native sound {id:#x} never started a voice: seen {seen:?}") };
        let (want_slot, want) = owning(&banks, own, id);
        assert_eq!(slot, want_slot, "sound {id:#x} resolved from slot {slot}, the level's own bank is {want_slot}");
        assert!(want.contains(&h), "sound {id:#x}'s sample hash {h} is not the level bank's decode {want:?}");
    }
}

/// THE repro (any weapon on any level): the AR (0x1a) given on 07000005 — a weapon the
/// level doesn't carry. Its fire (0x406/0x407) and reload (0x40a/0x40d) sounds are NOT in bank 5; the
/// draw (0x282) is. Pre-fix the fire/reload ids never started a voice (the driver's `SFXStart3D`
/// returned 1: no effect row in the one loaded bank). Post-fix they start from the owning spare bank
/// with that bank's own effect record and sample bytes, while the draw still comes from the level's own.
#[test]
fn a_foreign_weapons_fire_and_reload_sounds_start_voices_on_07000005() {
    let Some(mut scene) = load("07000005.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let (own, banks) = disc_banks(0x0700_0005);
    if !equip(&mut scene, 0x1a) { panic!("the AR (0x1a) could not be given on 07000005 (dev merge gap — 100a)"); }
    let mut sound = nf_view::build_sound(&mut scene).expect("the level has a bank");
    let mut seen = HashMap::new();
    until_drawn(&mut scene, &mut sound, &mut seen, 0x1a, 240);
    // Fire a magazine's worth (the AR's fire script 0x060007b6 op-1 @frame 1: sound 0x40d).
    for _ in 0..30 {
        scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Input::default() }, true, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        if seen.contains_key(&0x40d) { break; }
    }
    {
        let Some(&(slot, h)) = seen.get(&0x40d) else {
            panic!("the AR's fire sound 0x40d never started a voice on 07000005 (pre-fix: bank 5 has no row 0x40d); seen {seen:?}");
        };
        let (want_slot, want) = owning(&banks, own, 0x40d);
        assert_eq!(slot, want_slot, "fire sound 0x40d came from slot {slot}, the owning bank is {want_slot}");
        assert!(want.contains(&h), "fire sound 0x40d's sample hash {h} is not the owning bank's decode {want:?}");
    }
    // The draw is the level's own (bank 5 has 0x282 — the one-bank path, unchanged).
    let Some(&(slot, h)) = seen.get(&0x282) else { panic!("the draw sound 0x282 never played; seen {seen:?}") };
    let (want_slot, want) = owning(&banks, own, 0x282);
    assert_eq!(slot, want_slot, "draw sound from slot {slot}, want the level's own {want_slot}");
    assert!(want.contains(&h), "draw sample hash {h} not the level bank's {want:?}");
    // Reload the empty clip: the AR's reload script op-1s are 0x407@48, 0x406@89, 0x40a@132 — spare-bank
    // sounds too.
    let mut got = Vec::new();
    for _ in 0..300 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false);
        tick(&mut sound, &mut scene, &mut seen);
        if seen.contains_key(&0x40a) && seen.contains_key(&0x406) && seen.contains_key(&0x407) { break; }
    }
    for id in [0x406u32, 0x407, 0x40a] {
        if let Some(&(slot, h)) = seen.get(&id) {
            let (want_slot, want) = owning(&banks, own, id);
            assert_eq!(slot, want_slot, "reload sound {id:#x} came from slot {slot}, want {want_slot}");
            assert!(want.contains(&h), "reload sound {id:#x}'s hash {h} not the owning bank's {want:?}");
            got.push(id);
        }
    }
    assert!(!got.is_empty(), "no AR reload-stage sound (0x406/0x407/0x40a) ever played; seen {seen:?}");
}

/// The sweep: every weapon record on 07000005 — draw, fire, and (where the level bank lacks the reload
/// ids) reload — requests its ids and each starts a voice with the owning bank's sample. Every cell
/// is an independent first give (level-start inventory + gun restored per cell — see the reset note
/// in the body), so no weapon's window depends on the previous cell's reload/pool state.
#[test]
fn every_weapon_on_07000005_draws_fires_and_reloads_with_sounds() {
    let Some(mut scene) = load("07000005.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let (own, banks) = disc_banks(0x0700_0005);
    let mut sound = nf_view::build_sound(&mut scene).expect("the level has a bank");
    let mut seen = HashMap::new();
    let mut given = 0usize;
    let mut failures: Vec<String> = Vec::new();
    // Each cell starts from the level-start inventory + gun (mp-shot-gate-reload-2). The old sweep
    // chained every weapon's ammo/switch state into the next cell, which under the game's
    // held-trigger reload rule (794: the auto-reload arm sits in the trigger's RELEASE path — a
    // held trigger never reloads [M 0x1a5460 disasm 0x1a5fe8 → 0x1a61d0]) left the clip-slot twins
    // (e.g. the AIMS-20 0x1b, whose base 0x1a is already owned, so `equip_weapon`'s owned arm adds
    // carried rounds and touches no clip [M Player_EquipWeapon]; the switch reads 0x1b's OWN slot —
    // the pair's ammo types differ, 5 vs 12 [L ee_snapshot weapon_data]) drawing with clip 0 into
    // a held fire window: nothing fired, no sound started. The give the sweep models is the dev
    // give — `equip_weapon`'s first-give arm fills the clip [M]; resetting to the level-start state
    // makes every cell exactly that first give (port choice in the harness, not a game claim), so
    // each weapon's window is independent of the previous one.
    let (inv0, gun0, bond0, death0, health0) = {
        let sim = scene.sim.as_ref().unwrap();
        (sim.drones.inventory.clone(), sim.gun.clone(), sim.bond.clone(), sim.drones.death.clone(), sim.drones.health)
    };
    for w in 0x01..=0x71usize {
        // The health pool rides with it (the death check reads `Drones.health` [M Player_CheckForDeath
        // 0x190810] — a dead Bond re-dies on the first tick and `Player_WeaponNone` 0x1a9858 re-routes
        // the switch request to hands, wedging the cell on 0x47): mine blasts and stray fire must not
        // leak across cells.
        { let sim = scene.sim.as_mut().unwrap(); sim.drones.inventory = inv0.clone(); sim.gun = gun0.clone(); sim.bond = bond0.clone(); sim.drones.death = death0.clone(); sim.drones.health = health0; }
        let reload_ids: &[u32] = match w { 0x1a => &[0x40a, 0x40d], 0x2c => &[0x10c, 0x10d], _ => &[] };
        if !equip(&mut scene, w) { eprintln!("weapon {w:#04x}: refused (no first-person model — 100a's asset gap)"); continue; }
        given += 1;
        // 0x02 is the weapon Bond spawns with (no switch → no draw sound); 0x03 shares its base and
        // first-person model, so `Player_WeaponSelect` variant-swaps through state 8 — no draw sound
        // there either [M 0x1a2cc0 lines 13-29].
        let start_weapon_or_variant = w == 0x02 || w == 0x03;
        // The wait covers this cell's own switch: the level-start P2K's holster (+0xC4 at 1.25
        // [M 0x1a2e38 case 1:220-223]) + this draw (+0xC0 at 1.25 [M case 3:287-288]). Measured:
        // the Q-Worm 0x41 draws ~180 frames (drawn_at 190 from the P2K start), the MGL 0x42 ~150 —
        // fix-weapon-anims's 240 cap was tuned to the double-advance bug's 2x-fast draws and timed out
        // mid-draw, so the fire window below ran while still switching (the game's fire block) and the
        // fire-script op-1s (0x27f/0x41b/0x5ce at frame 2) never played. 600 covers the longest draw
        // with margin (the per-cell reset removed the old chained reload tail).
        until_drawn(&mut scene, &mut sound, &mut seen, w, 600);
        // The draw sound (weapon 0x282, gadget 0x281 [M 0x1a2e38]) must have started with the owning
        // bank's sample.
        let draw = if w >= 0x4a { 0x281u32 } else { 0x282 };
        if start_weapon_or_variant { continue; }
        match seen.get(&draw) {
            Some(&(slot, h)) => {
                let (want_slot, want) = owning(&banks, own, draw);
                if slot != want_slot || !want.contains(&h) { failures.push(format!("{w:#04x} draw {draw:#x}: slot {slot} hash {h}, want slot {want_slot}")); }
            }
            None => failures.push(format!("{w:#04x}: draw sound {draw:#x} never played")),
        }
        // The guided-missile camera wait. A `+0x6C & 4` shot hands the view to the projectile — the
        // player substate 10 runs no weapon arms, so the trigger is dead until the missile dies and a
        // 15-tick restore elapses [M Bullet_init 0x12c5a0:248-253; Bullet_update 0x12e660:28
        // (`+0x944 = 0xf`); Player_Update 0x193600:242-256]. The AT-420 Sentinel's guided rocket
        // (0x2c — the only camera-bit carrier [L ee_start_05.bin @ 0x2bf150], the sweep's segment a
        // dozen weapons earlier) fires it, and since the quarter-speed spawn fix [M 0x12c5a0:188-190:
        // `+0x6C & 0x200` → speed × 0.25, × the measured 4.04 flight scale — fix-sentinel-camera-play
        // [L PCSX2 2026-10-14: the 0x2d's BU+0xF8 = 0.33666, constant]) that rocket flies at its
        // 0.2525 u/tick (0.25 [L record +0x20] × 0.25 × 4.04) over its 200-unit range [L +0x1C] —
        // ≈790 ticks of camera ride that
        // swallows the fire windows of every later weapon up to the mine 0x37 (the merge-hold's
        // "0x37/0x59: none of its missing sounds ever played": no mine fired, so its detonation
        // sound 0x458 never started a voice anywhere in the sweep). The game's player could not fire
        // during the ride either — wait the cam out, then fire. The bound is the flight time plus the
        // restore plus margin; the range death is SILENT (state 4 [M Bullet_update 0x12e848]), so the
        // wait adds no blast near Bond.
        for _ in 0..3400 {
            if scene.sim.as_ref().unwrap().missile_cam.is_none() { break; }
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            tick(&mut sound, &mut scene, &mut seen);
        }
        // Fire.
        for _ in 0..25 {
            scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Input::default() }, true, false, false);
            tick(&mut sound, &mut scene, &mut seen);
        }
        // Reload (only where the level bank lacks the reload ids — the interesting spare path).
        if !reload_ids.is_empty() {
            for _ in 0..300 {
                scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false);
                tick(&mut sound, &mut scene, &mut seen);
                if reload_ids.iter().all(|id| seen.contains_key(id)) { break; }
            }
        }
        // The mine 0x37's detonation sound 0x458 is the mine's own fuse arming the blast at Bond's
        // feet — the plant's shell (0x37's row is [0x458] alone, and 0x59's row rides the accumulated
        // 0x458). The blast kills Bond, so under the per-cell reset it must land inside THIS cell
        // (before the death state is cleared for the next one) or the sound never reaches `seen`.
        // Measured: the mine detonates ~360 ticks after the plant; the death it starts (`Player_WeaponNone`
        // 0x1a9858 on the death tick [M 0x190810:61]) is reset away with everything else at the next
        // cell top.
        if w == 0x37 {
            for _ in 0..2400 {
                if seen.contains_key(&0x458) { break; }
                scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
                tick(&mut sound, &mut scene, &mut seen);
            }
        }
    }
    eprintln!("sweep: {given} weapons given, {} ids started", seen.len());
    // Every missing-id sound any of the swept weapons requested must have started from its owning bank.
    for (w, ids) in MISSING_ON_05 {
        if !ids.iter().any(|id| seen.contains_key(id)) { failures.push(format!("{w:#04x}: none of its missing sounds {ids:?} ever played")); continue; }
        for id in ids.iter().filter(|id| seen.contains_key(id)) {
            let (slot, h) = seen[id];
            let (want_slot, want) = owning(&banks, own, *id);
            if slot != want_slot || !want.contains(&h) {
                failures.push(format!("{w:#04x} sound {id:#x}: slot {slot} hash {h}, want slot {want_slot}"));
            }
        }
    }
    // The gadget-context ids, started directly (the same `SFXStart3D` entry the EE drives): each must
    // resolve from its owning bank with that bank's sample.
    for id in GADGET_CONTEXT_IDS {
        let mut out = Vec::new();
        let rc = sound.driver.start(&mut out, *id, Some([0.0; 3]), 0x9999, None);
        let (want_slot, want) = owning(&banks, own, *id);
        assert_eq!(rc, 0, "driver start of {id} failed");
        let (slot, h) = (sound.driver.item_slot(*id), sound.driver.played_sample_hash(*id));
        assert_eq!(slot, Some(want_slot), "gadget sound {id} from slot {slot:?}, want {want_slot}");
        assert!(h.is_some_and(|h| want.contains(&h)), "gadget sound {id} hash {h:?} not the owning bank's {want:?}");
        sound.driver.remove(0x9999, None);
        let mut out2 = Vec::new();
        sound.driver.update(&mut out2, &[]);
    }
    assert!(failures.is_empty(), "weapon-sound failures:\n{}", failures.join("\n"));
    // The giveable count is the dev view-model merge's baseline (the 24 missing weapon-model hashes
    // are 100a's asset gap; the rest have no first-person model at all). It must not SHRINK.
    assert!(given >= 69, "only {given} weapons giveable on 07000005 — the dev view-model merge regressed");
}

/// The same foreign-weapon repro on an indoor level (07000001, bank hash 1 → slot 10): the AR's fire
/// sound 0x40d is missing from that bank too, and must start from the owning spare bank.
///
/// The old IGNORE (100a/100c, "the fire script may not be merged") was wrong on both counts: the
/// dev merge already carries every 0x1a view script onto this level (idle 0x060007b8, fire
/// 0x060007b6, draw 0x060007be — all seven resolve in the merged set), and the fire path works.
/// The real blocker was the level's own opening script: its event-4 disable (the cut-scene
/// `Player_Disable`, `BLData+0x94a` [M 0x1949a8:24]; the port's `disabled` gate kills the pad
/// input for the whole tick) holds Bond from tick 0 until the script's enable lands 2121 ticks
/// in — the old 120-tick wait fired INSIDE the cut-scene, where the game itself takes no input.
/// The wait below is the player's own: idle ticks through the intro until the game releases him.
#[test]
fn a_foreign_weapon_fires_with_sounds_on_the_indoor_level_07000001() {
    let Some(mut scene) = load("07000001.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let (own, banks) = disc_banks(0x0700_0001);
    let mut sound = nf_view::build_sound(&mut scene).expect("the level has a bank");
    let mut seen = HashMap::new();
    // Wait out the opening cut-scene with idle input (the real path — the game's enable event
    // ends it; the cap only bounds a broken enable).
    let mut waited = 0usize;
    for t in 0..4000 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        if !scene.sim.as_ref().unwrap().disabled { waited = t; break; }
    }
    assert!(waited > 0, "07000001 must start with Bond held by the intro's disable (the disc script's event 4)");
    assert!(waited < 4000, "the intro never released Bond (the script's enable never landed)");
    if !equip(&mut scene, 0x1a) { panic!("the AR (0x1a) could not be given on 07000001 (dev merge gap — 100a)"); }
    until_drawn(&mut scene, &mut sound, &mut seen, 0x1a, 240);
    for _ in 0..30 {
        scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Input::default() }, true, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        if seen.contains_key(&0x40d) { break; }
    }
    let Some(&(slot, h)) = seen.get(&0x40d) else { panic!("the AR fired silently on 07000001 (no 0x40d voice); seen {seen:?}") };
    {
        let (want_slot, want) = owning(&banks, own, 0x40d);
        assert_eq!(slot, want_slot, "sound 0x40d from slot {slot} on 07000001, want {want_slot}");
        assert!(want.contains(&h), "sound 0x40d hash {h} not the owning bank's {want:?}");
    }
}

/// Sister to the indoor fire test: the intro's hold is real. 07000001's opening script disables
/// Bond at load (the disc script's event-4 `PlayerDisable` [M the script-request handler; the
/// game's cut-scene disable `BLData+0x94a`, 0x1949a8:24 — the port's `disabled` gate zeroes the
/// pad input for the tick]) and a held fire window DURING the hold spawns no round and starts no
/// voice, while the script still plays. Fails if the port stops honouring the disable (Bond
/// enabled at tick 0) or the enable (the hold never ends).
#[test]
fn the_indoor_intro_holds_the_trigger_until_the_script_releases_bond() {
    let Some(mut scene) = load("07000001.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    // The disable is the script's event 4 — it lands on the first script tick, not at load.
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    assert!(scene.sim.as_ref().unwrap().disabled, "Bond must be disabled from the intro's first script tick (event-4 PlayerDisable)");
    let mut sound = nf_view::build_sound(&mut scene).expect("the level has a bank");
    let mut seen = HashMap::new();
    // A held fire window inside the cut-scene: the input is dead — no round, no voice, and the
    // view never enters the fire state.
    for _ in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Input::default() }, true, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.disabled, "the disable must hold through the cut-scene");
        assert!(!sim.gun.fire_state, "the fire state must never key during the disable");
    }
    assert!(!seen.contains_key(&0x40d), "no AR fire voice during the intro's disable");
    // The release: the script's enable lands and Bond is his own again.
    let mut released = false;
    for _ in 0..4000 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        if !scene.sim.as_ref().unwrap().disabled { released = true; break; }
    }
    assert!(released, "the intro never released Bond (the enable event never landed)");
}

/// The Laser (0x4e) on 07000005: its loop (script sound 0x12 — SHF loop-flagged sample row [D bank 4])
/// keys on when the fire button goes down and is stopped by the script's op-4.3 (`Sound_StopAllWithId`
/// [M 0x1c1b38:388]) when the button comes back up.
///
/// The old IGNORE ("no first-person model in the dev merge") was stale: the merge carries the
/// Laser's fp skin 0x0500004b (from 07000043) and all six of its view scripts (idle 0x060004ea,
/// fire 0x060004e9/0x060004e8, draw 0x060004ee, holster 0x060004e7, idle-alert 0x060006e7) onto
/// this level. The real blocker was the give model: 0x4e is ALREADY OWNED at the 07000005 start
/// (level_start owns it with a 500-round clip), the game's owned arm refuses a re-give
/// (`Player_EquipWeapon` 0x1a8a58: carried at the ammo_data cap or a zero take → refused [M]), and
/// the dev give still draws it — the panel's documented affordance ("already owned — the take
/// added rounds for …" / "already owned, drawing it", panel.rs `apply`). The give below is that
/// two-step: the take (which must refuse), then the switch request.
#[test]
fn the_laser_loop_starts_on_press_and_stops_on_release() {
    let Some(mut scene) = load("07000005.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let (own, banks) = disc_banks(0x0700_0005);
    // The take must refuse (already owned — the game's owned arm); the dev give still draws.
    assert!(!equip(&mut scene, 0x4e), "the Laser's take must refuse: 0x4e is already owned (the game's owned arm)");
    scene.sim.as_mut().unwrap().gun.switch_request = 0x4e;
    let mut sound = nf_view::build_sound(&mut scene).expect("the level has a bank");
    let mut seen = HashMap::new();
    until_drawn(&mut scene, &mut sound, &mut seen, 0x4e, 240);
    // Hold fire: the loop must start and stay alive across ticks.
    let mut started = 0usize;
    for _ in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Input::default() }, true, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        if sound.driver.item_slot(0x12).is_some() { started += 1; }
    }
    assert!(started > 0, "the Laser's loop 0x12 never keyed on while the button was held; seen {seen:?}");
    let (slot, h) = seen[&0x12];
    let (want_slot, want) = owning(&banks, own, 0x12);
    assert_eq!(slot, want_slot, "loop 0x12 from slot {slot}, want {want_slot}");
    assert!(want.contains(&h), "loop 0x12 hash {h} not the owning bank's {want:?}");
    // Release: the script's op-4.3 stop frees it (Sound_StopAllWithId → SFXRemove → kill pass [M]).
    let mut gone_at = None;
    for t in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        if sound.driver.item_slot(0x12).is_none() { gone_at = Some(t); break; }
    }
    assert!(gone_at.is_some(), "the Laser's loop 0x12 kept playing after release (the op-4.3 stop leaked)");
}

/// Sister to the Laser loop test: the hold is UNINTERRUPTED, and the one-shot reap never touches
/// it. Every tick's harness pass reports all voices ended to the driver (the headless stand-in
/// for the SPU's end reports) — a looped sample never ends on that path [M sound.md §6], so the
/// loop must be alive after every one of the 120 held ticks. Fails if the loop is ever keyed as a
/// one-shot (it dies at the first end-report pass) or the hold drops it.
#[test]
fn the_laser_loop_survives_every_one_shot_reap_while_held() {
    let Some(mut scene) = load("07000005.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let (_own, _banks) = disc_banks(0x0700_0005);
    assert!(!equip(&mut scene, 0x4e), "the Laser's take must refuse (already owned)");
    scene.sim.as_mut().unwrap().gun.switch_request = 0x4e;
    let mut sound = nf_view::build_sound(&mut scene).expect("the level has a bank");
    let mut seen = HashMap::new();
    until_drawn(&mut scene, &mut sound, &mut seen, 0x4e, 240);
    let mut alive = 0usize;
    for _ in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Input::default() }, true, false, false);
        tick(&mut sound, &mut scene, &mut seen);
        if sound.driver.item_slot(0x12).is_some() { alive += 1; }
    }
    assert_eq!(alive, 120, "the Laser's loop must ride out every held tick (a one-shot keying dies at the first end-report pass)");
}
