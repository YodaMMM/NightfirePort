//! reverb-zones live tests: the level's per-area reverb (docs/systems/reverb-zones.md).
//!
//! The game's chain [M]: the room cel carries the reverb value (`parseentity_fixup_entity`
//! 0x1d00c0: class 0xc047 -> cel+0x98 = placement param 9 as a raw u16; class 0xc023 -> the 0x9898
//! underwater sentinel; param 3 != 0 -> cel+0x3c & 0x10000000), `Camera_CheckLocation` 0x1bae90
//! keeps viewer+4 = the cel under the eye, and `Sound_UpdateListeners` 0x1dd278 sends
//! `SFXSetEnvironment(cel s16 +0x98, underwater = (cel+0x3c & 0x10000000) || env >= 20)` every
//! frame. The IOP (SFX.IRX 0x0db4/0x0e60/0xc840) clamps the target to 0..100 (the -26472 sentinel
//! -> SFXUnderWater + target 0), selects the indoors mix (50 = muffled / 100 = open), and fades
//! the applied depth +16 rising / -4 falling per frame, writing EVOL = ((r*80)/100)*0x3fff/100 to
//! both cores. The driver's `underwater` accessor flag is the IOP's SFXUnderWater: TRUE only for
//! the sentinel send, never for an ordinary env (the fix-room-sound reading).
//!
//! The pinned positions are on the indoor/outdoor pair of the Mayhew chain [D, the disc parse]:
//! **07000005** - the castle approach (outdoor): the spawn yard is env 0 dry, small interior
//! pockets are env 40 with the underwater flag; **07000006** - the castle interior (indoor): the
//! spawn hall is env 50, cellar pockets env 90, side rooms 30/70, and one env-0 cel that still
//! carries the underwater flag (the muffled mix with no reverb). The zone values are disc bytes
//! read through the game's own parse; every position test carries sister assertions at second
//! positions whose values would expose a broken zone pick.
//!
//! Tests are silent: the driver runs with `output: None` (NF_SILENT in the suite).

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// A bare output-less sound stack: the driver is what the assertions watch (the fix-room-sound
/// precedent).
fn bare_sound() -> nf_audio::sfx::Sound {
    nf_audio::sfx::Sound {
        driver: Default::default(),
        output: None,
        music_files: None,
        music: None,
        music_pending: Vec::new(),
    }
}

/// Moves Bond so his EYE lands on `p` (the game sends from the viewer's eye, viewer+0xc0
/// [M 0x1bae90]; the port sends from `sim.bond.head()`): keep the head offset, move the body.
fn place_eye_at(sim: &mut nf_view::scene::Sim, p: [f32; 3]) {
    let h = sim.bond.head();
    let off = [h[0] - sim.bond.pos[0], h[1] - sim.bond.pos[1], h[2] - sim.bond.pos[2]];
    sim.bond.pos = [p[0] - off[0], p[1] - off[1], p[2] - off[2]];
}

/// The EVOL integer `psiSetReverb` 0xc840 computes for a reverb depth `r`
/// [M SFX.IRX: ((r*0x50)/100)*0x3fff/100, integer] - what `Driver::update`'s returned fraction
/// encodes. Written here as the game's formula, not a call into the driver.
fn evol(r: i32) -> i32 { (r * 80) / 100 * 0x3fff / 100 }

/// The depth `Driver::update` returns, decoded back to the EVOL integer (the f32 round-trip is
/// safe: the values are <= 0x3fff, 14 significant bits).
fn evol_of(depth: f32) -> i32 { (depth * 0x3fff as f32).round() as i32 }

// ---------------------------------------------------------------- 07000005 (outdoor)

/// 07000005 - the castle approach (outdoor). The spawn yard cel is env 0 with no underwater flag
/// (the open-air mix: reverb target 0, indoors 100); the small interior pockets are env 40 with
/// the underwater flag set (reverb 40, the muffled 50 mix) [D disc parse; the level's 15 dry / 4
/// env-40-flagged split is [L ee_start_05.bin] via fix_room_sound_live]. Sisters: the env-40
/// pocket and the small dry interior cel assert values that differ from the spawn yard's, so a
/// zone pick stuck on the spawn cel fails.
#[test]
fn outdoor_07000005_zones_at_known_positions() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let mut snd = bare_sound();
    let sim = scene.sim.as_mut().unwrap();

    // The spawn: Bond's eye in the yard cel (probe: head (104.95, 25.68, 108.83) -> cel 0, env 0,
    // dry). sound_tick is the game's per-frame send [M 0x1dd278].
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(sim.cel_at(sim.bond.head()), Some(0), "the spawn eye sits in the yard cel");
    assert_eq!(snd.driver.environment(), (0, false, 100),
        "the outdoor yard: env 0 dry -> reverb 0, indoors 100");
    // Sister: the yard cel really is the dry outdoor kind (the count split is pinned in
    // fix_room_sound_live; here the spawn's own cel).
    assert!(!sim.cels[0].underwater && sim.cels[0].env == 0, "the yard cel is env 0, dry [D]");

    // The env-40 underwater pocket by the spawn (cel 15: (103.2, 15.8, 93.8)-(114.0, 23.8, 104.4)
    // [D]; the point is inside and outside the overlapping cel 16's z-range, so find_cel's
    // smallest-volume rule picks 15 over the giant yard box).
    let p15 = [108.0, 19.5, 99.0];
    assert_eq!(sim.cel_at(p15), Some(15), "the pocket point resolves to the env-40 cel");
    place_eye_at(sim, p15);
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment(), (40, false, 50),
        "the env-40 pocket: reverb 40, the flagged send -> the muffled 50 mix");
    // Sister: the eye really moved (a stale pick would still report the yard's values above).
    assert_ne!(sim.cel_at(sim.bond.head()), Some(0), "the eye left the yard cel");

    // The small dry interior cel on the west side (cel 9: (-33.8, 36.5, -38.0)-(-20.5, 39.8,
    // -21.3) [D]): env 0, no flag -> the open mix again.
    let p9 = [-27.0, 38.0, -30.0];
    assert_eq!(sim.cel_at(p9), Some(9), "the west interior point resolves to the small dry cel");
    place_eye_at(sim, p9);
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment(), (0, false, 100),
        "the dry interior cel: env 0, no flag -> reverb 0, indoors 100");
    assert!(sim.cels[9].env == 0 && !sim.cels[9].underwater, "cel 9 is the dry kind [D]");
}

// ---------------------------------------------------------------- 07000006 (indoor)

/// 07000006 - the castle interior (indoor). The parse keeps the level's zone spread [D disc
/// parse]: 37 room cels with env {0x1, 20x10, 30x3, 35x2, 40x1, 50x3, 60x1, 70x2, 90x14} and 18
/// cels carrying the underwater flag (the lone env-0, one env-30, both env-70s and all 14 env-90
/// pockets). Pinned by value counts (parse-order-independent) plus three exact kinds.
#[test]
fn indoor_07000006_parse_pins_the_zone_spread() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    assert_eq!(scene.cels.len(), 37, "the level's room-cel count [D probe census]");
    let mut envs: std::collections::BTreeMap<i16, usize> = Default::default();
    for c in &scene.cels { *envs.entry(c.env).or_default() += 1; }
    let want: Vec<(i16, usize)> = [(0i16, 1usize), (20, 10), (30, 3), (35, 2), (40, 1), (50, 3), (60, 1), (70, 2), (90, 14)].into();
    assert_eq!(envs.into_iter().collect::<Vec<_>>(), want, "the env-value multiset [D]");
    assert_eq!(scene.cels.iter().filter(|c| c.underwater).count(), 18, "the underwater-flag cels [D]");
    // Sisters: three exact kinds with distinct (env, flag) combinations.
    assert!(scene.cels.iter().any(|c| c.env == 50 && !c.underwater), "a dry 50 hall exists [D]");
    assert!(scene.cels.iter().any(|c| c.env == 90 && c.underwater), "a wet 90 pocket exists [D]");
    assert!(scene.cels.iter().any(|c| c.env == 0 && c.underwater), "the env-0 flagged cel exists [D]");
}

/// 07000006 position pins through the real `sound_tick` path: the spawn hall (env 50, dry), the
/// east cellar pocket (env 90, flagged), a west side room (env 30, dry) and the lone env-0
/// flagged cel. Sisters: each position asserts a different (reverb, indoors) pair, so a pick
/// stuck on one cel fails; the env-0/flagged pair inverts the outdoor level's dry readings.
#[test]
fn indoor_07000006_zones_at_known_positions() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    let mut snd = bare_sound();
    let sim = scene.sim.as_mut().unwrap();

    // The spawn: the great hall, env 50, no flag [D probe: pos (57.18, 39.27, 22.61) -> cel 24].
    // env 50 >= 20 forces the underwater ARGUMENT (the game's signed compare [M 0x1dd4b0]) even
    // though the cel's flag is clear -> the muffled 50 mix; the driver's SFXUnderWater stays off.
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment(), (50, false, 50),
        "the spawn hall: reverb 50, SFXUnderWater off, indoors 50");
    assert_eq!(sim.cel_at(sim.bond.head()).map(|i| sim.cels[i].env), Some(50),
        "the spawn eye's cel is the env-50 hall [D]");

    // The east cellar pocket (cel 32: (77.1, 34.0, 14.3)-(97.5, 38.1, 41.8) [D]; no other room
    // cel reaches x 87): env 90 with the flag -> (90, 50), SFXUnderWater still off (only the
    // sentinel sets it [M 0x0db4]).
    let p32 = [87.0, 36.0, 28.0];
    assert_eq!(sim.cel_at(p32), Some(32), "the cellar point resolves to the env-90 cel");
    place_eye_at(sim, p32);
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment(), (90, false, 50),
        "the env-90 cellar: reverb 90, the flagged send -> the muffled mix");

    // The west side room (cel 18: (-22.0, 38.0, -4.2)-(11.0, 46.0, 14.0) [D]): env 30, dry ->
    // (30, 50) - env >= 20 still forces the muffled argument [M 0x1dd4b0].
    let p18 = [-5.0, 42.0, 5.0];
    assert_eq!(sim.cel_at(p18), Some(18), "the side-room point resolves to the env-30 cel");
    place_eye_at(sim, p18);
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment(), (30, false, 50),
        "the env-30 side room: reverb 30, indoors still 50 (env >= 20)");

    // The lone env-0 cel that carries the flag (cel 2: (27.9, 40.0, -25.0)-(37.5, 45.0, -16.2)
    // [D]): no reverb, but the flag alone selects the muffled 50 mix - the pair the outdoor
    // level's dry cels invert.
    let p2 = [32.0, 42.0, -20.0];
    assert_eq!(sim.cel_at(p2), Some(2), "the flagged env-0 point resolves to its cel");
    place_eye_at(sim, p2);
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment(), (0, false, 50),
        "env 0 with the flag: reverb 0, the muffled mix (the flag alone decides)");
}

// ---------------------------------------------------------------- the crossfade

/// The fade between presets, through the real path: `sound_tick` sets the target and runs one
/// `SFXUpdate` (`Sound::batch` -> `Driver::update`); extra `driver.update` calls are the
/// following frames. The rise is +16/frame, the fall -4/frame [M SFX.IRX 0x0e60], and the depth
/// returned is `psiSetReverb`'s EVOL [M 0xc840]. Sisters: the first fall frame must NOT be the
/// target (a snap would fail), and the rise must pass through the intermediate depths.
#[test]
fn the_zone_change_fades_at_the_game_rates() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    let mut snd = bare_sound();
    let sim = scene.sim.as_mut().unwrap();

    // Enter the env-50 hall from a fresh driver (current 0). The tick's own batch already runs
    // one SFXUpdate frame (`Sound::batch` -> `Driver::update`): 0 -> 16 happened inside it. The
    // following frames give 32, 48, then the clamp at 50.
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment().0, 50, "the hall's target is 50");
    assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(32), "rise frame 2");
    assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(48), "rise frame 3");
    assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(50), "rise frame 4 clamps at the target");
    assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(50), "the current holds at the target");

    // Into the env-0 flagged cel (cel 2's bbox [D]): the target drops to 0 and the fall is
    // -4/frame (the tick's own frame took 50 -> 46).
    let p2 = [32.0, 42.0, -20.0];
    assert_eq!(sim.cel_at(p2), Some(2), "the env-0 cel resolves");
    place_eye_at(sim, p2);
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment().0, 0, "the new target is 0");
    assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(42),
        "the fall continues -4, not the target"); // sister: a snap-to-0 fails here
    assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(38), "fall frame 3");
    for _ in 0..10 { snd.driver.update(&mut Vec::new(), &[]); }
    assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(0), "the fall lands on 0");

    // Into the env-90 cellar [D]: 0 -> 90 takes six +16 frames (the tick's own frame did the
    // first, 16; these five pass through 32/48/64/80 and clamp at 90).
    let p32 = [87.0, 36.0, 28.0];
    assert_eq!(sim.cel_at(p32), Some(32), "the env-90 cel resolves");
    place_eye_at(sim, p32);
    nf_view::sound_tick(&mut snd, sim);
    for want in [32, 48, 64, 80, 90] {
        assert_eq!(evol_of(snd.driver.update(&mut Vec::new(), &[])), evol(want),
            "the rise to 90 passes through {want}");
    }
}

// ---------------------------------------------------------------- the 0xc023 sentinel, live

/// The 0xc023 sentinel through a real level: 07000002 (The Exchange) is the only retail level
/// with 0xc023 cel placements [D probe census - 5 of them, param 3 = 0/1]. Their env reads back
/// -26472 (the 0x9898 the game writes, also the allocator fill a classless cel keeps [M
/// 0x1d00c0/0x1c93f8]); the IOP turns that value into SFXUnderWater + reverb target 0 [M SFX.IRX
/// 0x0db4]. Sister: the level also has ordinary 0xc047 cels, asserted present.
#[test]
fn the_07000002_water_cels_send_the_underwater_sentinel() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut snd = bare_sound();
    let sim = scene.sim.as_mut().unwrap();
    let idx = scene.cels.iter().position(|c| c.env == -0x6768).expect("a sentinel (0xc023) cel exists");
    assert!(scene.cels.iter().any(|c| c.env >= 0), "sister: the level also has ordinary 0xc047 cels");

    // Stand inside the sentinel cel (bbox-fraction scan: the smallest-volume rule can lose a
    // centre point to an overlapping room, the fix-room-sound precedent).
    let (cmin, cmax) = { let c = &scene.cels[idx]; (c.min, c.max) };
    let mut placed = false;
    'scan: for fx in [0.5, 0.25, 0.75, 0.4, 0.6] {
        for fy in [0.5, 0.3, 0.7] {
            let p = [cmin[0] + (cmax[0] - cmin[0]) * fx,
                     cmin[1] + (cmax[1] - cmin[1]) * fy,
                     cmin[2] + (cmax[2] - cmin[2]) * fx];
            if sim.cel_at(p) == Some(idx) { place_eye_at(sim, p); placed = true; break 'scan; }
        }
    }
    assert!(placed, "a point inside the sentinel cel resolves to it");
    nf_view::sound_tick(&mut snd, sim);
    assert_eq!(snd.driver.environment(), (0, true, 50),
        "the sentinel: SFXUnderWater on, reverb target 0, the muffled mix [M 0x0db4]");
    // Sister: the picked cel IS the sentinel kind (a normal cel here would contradict the parse).
    assert_eq!(sim.cel_at(sim.bond.head()), Some(idx), "the eye is in the sentinel cel");
}
