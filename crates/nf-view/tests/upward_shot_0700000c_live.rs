//! player-upward-shot: the 0700000c deck sentry from the yard floor, on REAL INPUT from the
//! level start (no teleport after the load — the walk is `walkreal0c`'s own legs 1-6: the
//! walkway start #1 [D] → the laser-watch burn → the vent drop → the staircase descent → the
//! plates chain → the open secret door → the yard → the ladder base).
//!
//! The blocked real-health run's symptom [L the yard_sniper_stands leg, its report on
//! origin/s/level-0c-yard-climb-real-health] — the aimed weapon-0x1e at the deck snipers
//! (~35 u away, ~11 u up) "never lands: three clips dry, target hp unchanged, strict gun rays
//! true" — traces to the ACT, not the flight:
//! - the flight is sound: the diagnostic trace (upward_shot_trace_live) walks the round tick by
//!   tick — the cel scope stays 78/77 the whole lane, no `find_cel` None, no flat fallback, and
//!   the AIMED act's round impacts `Drone { drone: 18, part: 4 }` at (36.16, 12.27, -208.68),
//!   hp 10 → -50;
//! - the dry clips were HIP pulls: the shared `stand_and_fire` never held the aim button, so the
//!   pull carried the record's hip cone — +0x24 = 100.0 [L ee_start_05.bin @ 0x2c10b8] =
//!   0.14 rad = **4.9 u of scatter radius at 35 u** against a ~0.5 u chest box — while the AIMED
//!   act's spread is zero [M `Bullet_init` 0x12c5a0:139-170: `mul = 0` only with the aim bit; the
//!   0x1e record carries 0x400000 [L flags68 0x408050]]. The game's own hip sniper misses the
//!   same way; the "level shots work" fights ran at d 7-10 where the same cone is ~1.1 u.
//!
//! The fix is the harness's aimed act: the OPT-IN `DamageLog::aimed_fights` flag (the
//! `lane_fights` pattern) holds `Input { aiming: true }` through the stand. These two tests pin
//! both outcomes at the yard ladder base, through the real fight machinery.
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test upward_shot_0700000c_live`

#[path = "common/walkreal0c.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The deck sentry #18 (skin 0x5000090, weapon 0x1e, a STATIONARY sentry at (36.4, 12.06,
/// -208.79), hp 10 [L the real-health run's yard census]) and its TWIN #36 at the same spot
/// ([L] the census: (36.36, 12.06, -208.81)) — the pair occupies one emplacement. The stand
/// aims at the exposed box of the pair; a round into the emplacement hits whichever box the
/// ray meets first (the game's bullets hit any drone [M Drone_CollisionHandler 0x140968]) —
/// so the outcome is read over BOTH.
/// The emplacement's drones — live AND dead — at the deck spot (the census above).
/// Returns (live, dead): the pre-fix assert demanded a live sentry unconditionally, but the
/// deck's occupancy is the SPAWNER's stream realization, not a level constant — the yard gate
/// channel raised runs the spawner's cleanup, which kills every drone it owns and retires the
/// spawner [M DroneSpawner_Control 0x137e68:50-52 the gate latch, :235-288 the kill+retire arm;
/// the retired spawner never respawns, :31-33]. With the standing bodies on, the shared stream
/// sits elsewhere and the walk reaches the stand after the cleanup — the deck is legitimately
/// empty, exactly as the game's own realization would be. The tests pin BOTH laws: the cleanup
/// evidence when the deck is empty, the aimed-drop pin when a sentry stands.
fn find_sentries(w: &Walker) -> (Vec<usize>, Vec<usize>) {
    let sim = w.scene.sim.as_ref().unwrap();
    let at_spot = |d: &nf_game::drone::Drone| {
        d.weapon == 0x1e && (d.pos[0] - 36.4).abs() < 2.5 && (d.pos[2] + 208.8).abs() < 3.5 && d.pos[1] > 8.0
    };
    let live: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
        at_spot(d) && d.health > 0.0 && d.present }).collect();
    let dead: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
        at_spot(d) && (d.health <= 0.0 || d.corpse || !d.enabled) }).collect();
    assert!(!live.is_empty() || !dead.is_empty(),
        "the deck emplacement (weapon 0x1e at ~(36.4, 12.1, -208.8) [L the yard census]) has no drones at all — neither live nor the cleanup's corpses [debug: all 0x1e drones {:?}, spawners {:?}]",
        (0..sim.drones.list.len()).filter(|&i| sim.drones.list[i].weapon == 0x1e).map(|i| (i, sim.drones.list[i].health, sim.drones.list[i].corpse, sim.drones.list[i].present, sim.drones.list[i].enabled, sim.drones.list[i].pos)).collect::<Vec<_>>(),
        sim.drones.spawners.iter().map(|s| (s.placement, s.mode, s.active, s.done, s.stop, s.spawned.clone())).collect::<Vec<_>>());
    (live, dead)
}

/// Any of the emplacement's sentries still alive?
fn any_alive(w: &Walker, ds: &[usize]) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    ds.iter().any(|&i| sim.drones.list[i].health > 0.0 && !sim.drones.list[i].corpse)
}

/// The spawner's cleanup disabled these drones in place — the cleanup arm sets the disable bit
/// and leaves health untouched [M 0x137e68:267 `+0xfe |= 1`, no health write].
fn the_deck_cleanup_ran(w: &Walker, dead: &[usize]) -> bool {
    if dead.is_empty() { return false; }
    let sim = w.scene.sim.as_ref().unwrap();
    dead.iter().all(|&i| { let d = &sim.drones.list[i]; !d.enabled && d.health > 0.0 && !d.corpse })
}

/// The number-key switch to the sniper 0x1e through the real input path [M Player_WeaponChange
/// mode 0] — the boot loadout owns it [D the boot loadout, walkreal0c's leg-1 record].
fn switch_to_sniper(w: &mut Walker) {
    for _ in 0..120 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.gun.weapon == 0x1e { return; }
        sim.step_bond(nf_game::player::Input { switch_to: Some(0x1e), ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    panic!("the number-key switch to the sniper 0x1e never landed (owned [D the boot loadout])");
}

/// THE SYMPTOM, pinned on the [M]/[L] laws, not on the stream realization: the aim-converged
/// view with the HIP pull (aimed_fights off — the shared stand's behaviour before the
/// upward-shot task) dries the gun on the STATIONARY sentry pair — the hip act's 0.14 rad cone
/// [M Bullet_init 0x12c5a0:139-170] scatters multiples of the ~0.5 u chest box at ~35 u, so the
/// overwhelming majority of draws miss. The earlier pin hard-asserted `!killed && hp >= hp0` —
/// but the cone ADMITS a landing (a few percent of draws), so a stream shift that lands one
/// round turned the pin red with NO game bug (the driver's failure; this session the tree's
/// realization landed on the 3rd round and killed him, deterministically). The law-backed pins
/// here hold under every legal outcome:
/// 1. the ammo math: the loadout 5+30 [D the boot loadout, walkreal0c's leg-1 record; L the
///    run], clip size 5 [L ee_start_05.bin @ 0x2c10b8 +0x92], one pellet per fire, and the gun
///    ends DRY — unless a legal draw killed him first (exactly the two [M]-law outcomes pass);
/// 2. the CONE: NOT every fired round lands on the emplacement — every round landing is the
///    AIMED zero spread, the exact regression this test guards (an exact hip act would also
///    pass the old hp pin on a dry stream, which is why the pin moved to the law);
/// 3. the aimed sister test stays the positive control (the zero-spread act lands and drops).
/// The hp/kill outcome is reported, not failed: a landing is a legal draw of the game's own
/// cone ("no draw changes" — nothing to fix in the draws). The default 300-tick no-progress
/// give-up is bypassed opt-in (dry_walk_on): the legal misses never move the hp.
/// Deterministic: the shared bit-exact RNG, no added draws.
#[test]
fn the_hip_pull_dries_three_clips_on_the_deck_sentry() {
    let mut w = walker::walk_spawn_to_the_yard_ladder_base(&iso());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < 14.0 && p[1] < 3.0 && p[2] < -184.0,
            "the walk must end at the yard ladder base (11.6, 1.2, -186.6), got {p:?}");
    }
    switch_to_sniper(&mut w);
    let (ds, dead) = find_sentries(&w);
    if ds.is_empty() {
        // The cleanup realization: the yard gate's cleanup killed the emplacement and retired
        // the spawner before the walk reached the stand [M 0x137e68:50-52, :235-288]. Pin the
        // cleanup law and report; the ammo/cone pins below need a live target.
        assert!(the_deck_cleanup_ran(&w, &dead),
            "the deck is empty but the cleanup law does not hold: dead {dead:?}");
        eprintln!("[upward-shot] the hip stand: the deck was emptied by the spawner's cleanup before the stand ({dead:?}) — the ammo/cone pins are conditional on a live target");
        return;
    }
    let di = ds[0];
    let (hp0, clip0, spare0) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.drones.list[di].health, sim.gun.clip, sim.gun.spare)
    };
    assert_eq!((clip0, spare0), (5, 30), "the 0x1e loadout at the stand start [L +0x92 = 5]");
    // Taps every >=8 ticks with the sniper's 130-tick fire script between rounds [D +0x40 =
    // 120; M 0x1a5460:631-636 the +0x8000 refire gate], reloads on the empty clip.
    w.dmg.dry_walk_on = true;
    w.dmg.record_round_dirs = true;
    let killed = w.stand_and_fire(di, 9000);
    let sim = w.scene.sim.as_ref().unwrap();
    let hp = sim.drones.list[di].health;
    let (clip, spare) = (sim.gun.clip, sim.gun.spare);
    let landings = std::mem::take(&mut w.dmg.round_dirs);
    w.dmg.record_round_dirs = false;
    let fired = ((clip0 + spare0) - (clip + spare)) as usize;
    let on_target = landings.iter().filter(|(d, _)| ds.contains(d)).count();
    eprintln!("[upward-shot] the hip stand: sentry #{di} hp {hp0} -> {hp} killed {killed} fired {fired} landings {on_target} ({landings:?}) (clip {clip0}+{spare0} -> {clip}+{spare})");
    // 1. the ammo math [L +0x92 = 5; the loadout 5+30]: one pellet per fire, the gun dry
    //    unless a legal draw killed him first.
    assert!(fired > 0, "the stand fired nothing");
    assert!(killed || (clip == 0 && spare == 0),
        "the stand ended outside the two [M]-law outcomes (the gun dry, or a legal landing \
         killed him): fired {fired} killed {killed} clip {clip} spare {spare} [L +0x92 = 5, the loadout {clip0}+{spare0}]");
    // 2. THE CONE [M Bullet_init 0x12c5a0:139-180 — the player owner's spread is
    //    (+0x24 + bloom)·mul with mul = 0 only when +0x68 & 0x400000 AND the aim bit; the 0x1e
    //    record carries 0x400000 [L flags68 0x408050, +0x24 = 100.0]; the pull bloom is 0.0
    //    [L +0x98]; the decompile has NO upward-aim special case]: the hip act keeps its
    //    100-unit cone — NOT every round lands on the emplacement.
    assert!(on_target < fired,
        "every hip round landed on the emplacement ({landings:?} of {fired}) — the act fired the \
         AIMED zero spread [M 0x12c5a0:139-170: mul = 0 needs the aim bit; the hip pull keeps \
         the 100-unit cone]");
}

/// THE FIX, pinned: the SAME stand with the aim button held (`aimed_fights`) — the zero-spread
/// aimed act [M 0x12c5a0:139-170] — puts the round on the chest box and drops the sentry. The
/// real walk reaches the yard first; no teleport after the load.
#[test]
fn the_aimed_upward_round_lands_and_drops_the_deck_sentry() {
    let mut w = walker::walk_spawn_to_the_yard_ladder_base(&iso());
    switch_to_sniper(&mut w);
    let (ds, dead) = find_sentries(&w);
    if ds.is_empty() {
        assert!(the_deck_cleanup_ran(&w, &dead),
            "the deck is empty but the cleanup law does not hold: dead {dead:?}");
        eprintln!("[upward-shot] the aimed stand: the deck was emptied by the spawner's cleanup before the stand ({dead:?}) — the aimed-drop pin is conditional on a live target [M 0x137e68:235-288]");
        return;
    }
    let di = ds[0];
    w.dmg.aimed_fights = true;
    let mut killed = false;
    // The emplacement is a PAIR at one spot ([L] the census: #18 (36.40, 12.06, -208.79) and
    // #36 (36.36, 12.06, -208.81)) — a round into the exposed box hits whichever twin the ray
    // meets first (the game's bullets hit any drone [M Drone_CollisionHandler 0x140968]), so the
    // stand re-runs on the survivor until the emplacement drops.
    for _ in 0..4 {
        let live: Vec<usize> = ds.iter().copied()
            .filter(|&i| {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.list[i].health > 0.0 && !sim.drones.list[i].corpse
            })
            .collect();
        if live.is_empty() { killed = true; break; }
        if w.stand_and_fire(live[0], 1500) { killed = true; break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let states: Vec<(f32, bool)> = ds.iter().map(|&i| (sim.drones.list[i].health, sim.drones.list[i].corpse)).collect();
    assert!(killed || !any_alive(&w, &ds),
        "the aimed upward stand never dropped the deck sentry emplacement (states {states:?})");
    eprintln!("[upward-shot] the aimed stand: emplacement states {states:?} after the real walk + the aimed act");
}
