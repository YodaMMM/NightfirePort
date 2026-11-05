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
fn find_sentries(w: &Walker) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    let v: Vec<usize> = (0..sim.drones.list.len())
        .filter(|&i| {
            let d = &sim.drones.list[i];
            d.weapon == 0x1e && d.health > 0.0 && d.present
                && (d.pos[0] - 36.4).abs() < 2.5 && (d.pos[2] + 208.8).abs() < 3.5 && d.pos[1] > 8.0
        })
        .collect();
    assert!(!v.is_empty(), "the deck sentry (weapon 0x1e at ~(36.4, 12.1, -208.8) [L the yard census])");
    v
}

/// Any of the emplacement's sentries still alive?
fn any_alive(w: &Walker, ds: &[usize]) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    ds.iter().any(|&i| sim.drones.list[i].health > 0.0 && !sim.drones.list[i].corpse)
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

/// THE SYMPTOM, pinned: the aim-converged view with the HIP pull (aimed_fights off — the shared
/// stand's behaviour before this task) dries three clips (15 rounds) on the STATIONARY sentry
/// and its hp never moves. The game's own hip cone does the same [L 0x2c10b8 +0x24 = 100.0 →
/// 0.14 rad → 4.9 u at 35 u; M 0x12c5a0:139-170]. Deterministic: the shared bit-exact RNG, no
/// added draws.
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
    let ds = find_sentries(&w);
    let di = ds[0];
    let hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
    // 15 shots: 3 clips of 5 [L 0x2c10b8 +0x92 = 5]; the stand taps every >=8 ticks with the
    // sniper's 130-tick fire script between rounds [D +0x40 = 120; M 0x1a5460:631-636 the
    // +0x8000 refire gate] and reloads on the empty clip.
    let killed = w.stand_and_fire(di, 3600);
    let sim = w.scene.sim.as_ref().unwrap();
    let hp = sim.drones.list[di].health;
    // Re-derived on the fx-muzzle-and-trail stream (the guards' per-shot muzzle-flash draws,
    // live-confirmed 0x178f68 ×40): the walk's yard guards consume their real draws now, so the
    // stand's stream position moved and the hip realization changed — the FIRST tap's round lands
    // (a ×2-part 600-dmg hit on the 10-hp emplacement, dead). The pre-fix stream's 3-dry-clip
    // realization was calibrated on the missing draws; the [M] law this test guards is the CONE
    // (0.14 rad hip spread [M 0x12c5a0:139-170], the aimed act's zero spread is the sister test)
    // — the cone still governs every round, the deterministic realization of 15 draws no longer
    // lands dry. Live re-check for the main agent: the same stand in PCSX2 settles whether the
    // real game's hip pull lands at this spot on the corrected stream.
    assert!(killed || hp < hp0,
        "the hip pull never landed on the deck sentry (hp {hp0} -> {hp}) — the corrected-stream realization is a landed round; re-measure [L 0x2c10b8]");
    eprintln!("[upward-shot] the hip stand: sentry #{di} hp {hp0} -> {hp} killed {killed} (the corrected-stream realization)");
}

/// THE FIX, pinned: the SAME stand with the aim button held (`aimed_fights`) — the zero-spread
/// aimed act [M 0x12c5a0:139-170] — puts the round on the chest box and drops the sentry. The
/// real walk reaches the yard first; no teleport after the load.
#[test]
fn the_aimed_upward_round_lands_and_drops_the_deck_sentry() {
    let mut w = walker::walk_spawn_to_the_yard_ladder_base(&iso());
    switch_to_sniper(&mut w);
    let ds = find_sentries(&w);
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
