//! smash-stat-gate-02: `Destroy_Smash` 0x12bb60:42-55's 07000002 stat arm — the named smashable
//! counts in Bond's accuracy as the game's does.
//!
//! The game's chain [M]: a smashed destroyable whose obj+0xD8 (the glist the smash swapped in,
//! the target's +0x50 SIBLING — 0x1d1b68:63,96 → 0x1d04fc → 0x12c318:25-30, the +0x40 == 0 arm
//! [L ee_snapshot.bin: 4/4 placement glists read 0]) equals `hashtable_getitem(0x020001d9)` —
//! the celglist pointer registered under the name [M 0x1d1508:28-29] — gets
//! `PlrStat_LogShotHitScenery(0)` AFTER the channel write [M 0x12bb60:40 then :55, disasm
//! 0x12bc2c-0x12bc48]. On 07000002 the named glist is "Computer2" [D map 51 object 12], the
//! sibling of the destroy placement pl[14]'s own target "Computer1" [D map 51 object 11,
//! pl[14] hash 0x020001d8, param 0 = 72 = 0x48] — Mayhew's computer. The port carries the
//! sibling hash on the leaf (one field, no name table) and the sim readers log the tally.
//!
//! Pinned through the real path (the headless stats convention, fix_progress_live's): the real
//! load, real `step_bond` trigger pulses — the bullet spawns (+1 tally [M `Player_WeaponInitBullet`
//! 0x1a4bf0:258]), the port's bullet pass contacts the leaf's bounds, the smash runs, the arm
//! drops the denominator by exactly one more. The tally arithmetic: `fired == presses − 1`
//! after the computer breaks (the warm-up air shot + the killing shot = 2 presses, one smash
//! decrement). The air shot decrements nothing [M the bullet path 0x12ce80/0x12e7b0 carries no
//! PlrStat call; the port's world-hit fold only fires on world contacts, and the sky has none].
//! The sister leg breaks ANOTHER smashable (a bunker cable light, smashed-model hash 0xffffffff
//! [D pl[394] → "cable light1b"]) — no extra decrement, the compare misses.
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}

/// The walker's `shoot_point` aim discipline, self-contained (the shared walker is untouched):
/// converge the view on `at`, tap the trigger when on target (`t % 16`), one bullet per tap
/// (the P2K is semi-auto — a fresh press per shot [M fix_progress_live's note]). Returns the
/// presses spent and whether `done` fired.
fn shoot_at(sim: &mut nf_view::scene::Sim, at: [f32; 3], done: impl Fn(&nf_view::scene::Sim) -> bool) -> (u32, bool) {
    let mut presses = 0u32;
    for t in 0..2400u32 {
        if done(sim) { return (presses, true); }
        let bp = sim.bond.pos;
        let dx = at[0] - bp[0];
        let dz = at[2] - bp[2];
        let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let desired = dx.atan2(dz);
        let err = wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let on_target = err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0;
        // HIP fire — the walker's own shoot_point discipline (its Input carries turn/look only;
        // the aim bit's raise gates the fire here, and the walkreal's computer shot succeeded
        // hip from this very vantage).
        let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        if on_target {
            // The press/release pair — the game's own trigger path through step_bond.
            sim.step_bond(input, true, false, false);
            sim.step_bond(Input::default(), false, false, false);
            presses += 1;
        } else {
            sim.step_bond(input, false, false, false);
        }
    }
    (presses, false)
}

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

/// The level-start cut-scene (SP #400, script 0x06000084 [D]) holds the player until its
/// stream 7 writes ch 0x47 at the scene tick 1539 [D event 0x0D(47,1); L the walkreal's run]
/// — any shot or teleport before that rides the scripted player and the fail chain restarts
/// the level. The walkreal's own wait, self-contained.

/// One straight-up warm-up shot from the spawn: +1 fired, no decrement anywhere (the bullet
/// expires in the air — no world contact, no smash) [M 0x1a4bf0:258; no PlrStat call on the
/// bullet path 0x12ce80/0x12e7b0]. This makes the smash's single decrement visible (a fresh
/// computer leaf dies to the first bullet: health = param 3 = 0 [D pl[14] params]).
fn warmup_air_shot(sim: &mut nf_view::scene::Sim) {
    // The intro cut-scene (SP #400, script 0x06000084 [D]) holds the player (`sim.disabled` —
    // the game's GS_PausePlayer) until it ends; a shot or teleport before that rides the
    // scripted player and the fail chain restarts the level. This test SKIPS it the game's own
    // way (the player's skip action [M Script_FFwd 0x1dac78's chain; the walker's
    // skip_cutscenes sets the same `drones.scripts.skip_request`]): the stream 7 write of
    // ch 0x47 lands during the fast-forward [D event 0x0D(47,1)], the reveal included.
    let mut revealed = false;
    for _ in 0..3600 {
        if sim.drones.channels.on(0x47) { revealed = true; }
        if revealed && !sim.disabled { break; }
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Default::default(), false, false, false);
    }
    assert!(revealed && !sim.disabled, "the intro cut-scene finished (ch 0x47 [D 07800002 stream7], the player freed)");
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.disabled, "the player stays free");
    sim.bond.pitch = 0.99; // near-vertical [M the walker's normalized-pitch convention]
    sim.bond.yaw = 0.0;
    for _ in 0..6 { sim.step_bond(Default::default(), false, false, false); }
    sim.step_bond(Default::default(), true, false, false);
    sim.step_bond(Default::default(), false, false, false);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
}

/// The computer leg: the real load, the real trigger, the tally arithmetic
/// `fired == presses − 1` [M 0x1a4bf0:258 +1/bullet, 0x12bb60:55 −1 on the named smash].
#[test]
fn the_named_computer_smash_drops_the_accuracy_denominator_once() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.mission.status() < 2, "the mission is playing (the stat gate is open [M 0x198408:13-19])");

    // The named leaf: the port's sibling-hash carry locates Mayhew's computer [D map 51
    // objects 11/12; pl[14] hash 0x020001d8, param 0 = 0x48].
    let ci = sim.drones.props.destroy.iter()
        .position(|d| d.smashed_hash == nf_game::props::SMASH_STAT_HASH)
        .expect("07000002's destroy leaves carry the named smashed-model hash (pl[14] → \"Computer2\")");
    let centre = [(sim.drones.props.destroy[ci].lo[0] + sim.drones.props.destroy[ci].hi[0]) / 2.0,
                  (sim.drones.props.destroy[ci].lo[1] + sim.drones.props.destroy[ci].hi[1]) / 2.0,
                  (sim.drones.props.destroy[ci].lo[2] + sim.drones.props.destroy[ci].hi[2]) / 2.0];
    assert!(!sim.drones.props.destroy[ci].dead, "the computer stands before the shot");

    // The warm-up air shot (+1, no decrement), then the proven walkway vantage
    // [L walkreal_07000002_live's probe: the ray eye (−55.1, 7.45, −23.1) → centre is clear to
    // the box itself] and the real trigger until the death channel rises [D pl[14] param 0].
    warmup_air_shot(sim);
    let before = sim.drones.progress.stats[0].shots_fired;
    // The corridor vantage [L this session's sweep: from (−54.0, 5.9, −23.1) the settled stance
    // (−53.49, 6.40, −23.09) puts ONE press into the leaf — dead + ch 0x48; the (−55.1, …)
    // walkway end's muzzle sits in the walkway's trim and never spawns]. Settle first, then aim.
    sim.bond.pos = [-54.0, 5.9, -23.1];
    for _ in 0..12 { sim.step_bond(Default::default(), false, false, false); }
    {
        let bp = sim.bond.pos;
        let dy = centre[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        sim.bond.yaw = (centre[0] - bp[0]).atan2(centre[2] - bp[2]);
        sim.bond.pitch = (dy / dxz(bp, centre).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
    }
    let (presses, done) = shoot_at(sim, centre, |sim| sim.drones.channels.on(0x48));
    assert!(done, "the computer broke (ch 0x48 [D pl[14] param 0; M 0x12bd48]) within the cap");
    assert!(sim.drones.props.destroy[ci].dead, "the leaf is dead after the smash");
    assert!(sim.drones.mission.status() < 2, "Bond alive, the mission still playing (the gate was open at the smash)");
    eprintln!("[smash-stat] computer: presses {presses}, fired {} → {} (delta {})",
        before, sim.drones.progress.stats[0].shots_fired, sim.drones.progress.stats[0].shots_fired as i64 - before as i64);
    // The game's arithmetic: +1 per bullet spawned [M 0x1a4bf0:258], −1 on the named smash
    // [M 0x12bb60:55]. The leaf-killing bullet's own contact is not a decrement (the port's
    // world-hit fold does not fire on the destroy contact — [L this session's sweep: one press
    // at the leaf read delta 0 = the spawn and the smash only]). The warm-up air shot and every
    // miss ride their own spawns/folds inside `before`.
    assert_eq!(sim.drones.progress.stats[0].shots_fired, before + presses - 1,
        "the accuracy denominator nets the named smash off [M Destroy_Smash 0x12bb60:55]");
    assert!(sim.drones.props.stat_hit == false, "the scratch is consumed");
}

/// The sister leg: ANOTHER smashable's break changes nothing — the cable light's smashed-model
/// hash is 0xffffffff [D pl[394] → "cable light1b"], the game's compare misses [M 0x12bc3c-40],
/// the tally keeps every bullet.
#[test]
fn another_smashables_break_leaves_the_tally_alone() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }

    // A non-named destroy leaf near the bunker room: the cable light trio (kind 1, channel 0
    // for pl[390]/pl[394] [D params]), the one nearest (−59.9, 7.5, −28).
    let li = sim.drones.props.destroy.iter().enumerate()
        .filter(|(_, d)| !d.dead && d.smashed_hash != nf_game::props::SMASH_STAT_HASH)
        .min_by(|a, b| {
            let d = |x: &nf_game::props::Destroy| (x.pos[0] + 59.9).powi(2) + (x.pos[1] - 7.5).powi(2) + (x.pos[2] + 28.0).powi(2);
            d(a.1).total_cmp(&d(b.1))
        })
        .map(|(i, _)| i)
        .expect("07000002 has non-named destroy leaves (the cable lights/kitchen pairs [D pl[299]/301/390/392/394])");
    let centre = [(sim.drones.props.destroy[li].lo[0] + sim.drones.props.destroy[li].hi[0]) / 2.0,
                  (sim.drones.props.destroy[li].lo[1] + sim.drones.props.destroy[li].hi[1]) / 2.0,
                  (sim.drones.props.destroy[li].lo[2] + sim.drones.props.destroy[li].hi[2]) / 2.0];
    assert!(sim.drones.props.destroy[li].smashed_hash != nf_game::props::SMASH_STAT_HASH);

    warmup_air_shot(sim);
    let before = sim.drones.progress.stats[0].shots_fired;
    // The bunker room far end — under the cable light's walkway [D the walkreal's leg-5 room] —
    // the aim SET before the first tick (the same thin-floor rule as the computer leg).
    sim.bond.pos = [-59.0, 5.4, -26.0];
    {
        let dy = centre[1] - (sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop);
        sim.bond.yaw = (centre[0] - sim.bond.pos[0]).atan2(centre[2] - sim.bond.pos[2]);
        sim.bond.pitch = (dy / dxz(sim.bond.pos, centre).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
    }
    let (presses, _done) = shoot_at(sim, centre, |sim| sim.drones.props.destroy[li].dead);
    assert!(sim.drones.props.destroy[li].dead, "the cable light broke (the leaf's own death mark)");
    assert!(sim.drones.mission.status() < 2, "the mission still playing");
    eprintln!("[smash-stat] cable light: presses {presses}, fired {} → {} (delta {})",
        before, sim.drones.progress.stats[0].shots_fired, sim.drones.progress.stats[0].shots_fired as i64 - before as i64);
    // No smash decrement: +1 per bullet, nothing else [M 0x12bb60:51-54 — the compare
    // `obj+0xD8 != hashtable_getitem(0x020001d9)` skips the call; the port's leaf carries
    // 0xffffffff ≠ the named hash].
    assert_eq!(sim.drones.progress.stats[0].shots_fired, before + presses,
        "another smashable's break changes nothing [M 0x12bb60:51-54]");
    assert!(sim.drones.props.stat_hit == false, "the scratch never set");
}
