//! level-0c-guard-ammo: 0700000c guard #1's rounds at the ring, pinned through real input.
//!
//! The driver's symptom: on the current tree guard #1 (drone list index 1 = placement #3,
//! skin 0x5000093, weapon param 0x1e, at (13.04, 12.17, -198.00) [D docs/levels/0700000c-audit.md])
//! "keeps firing at the ring where he used to run dry" — the port is suspected of giving him
//! more rounds than the game allows. This test walks the shared 0c prologue to the yard
//! ladder, climbs it with the real input legs (copied verbatim from common/walkreal0c.rs's
//! `walk_spawn_to_the_band`), walks the patrol-ring legs, then holds Bond still at the ring
//! west edge — guard #1 closes and fires there ([L the yard census: "closes to ~1 u at the
//! ring NW corner and holds AimStandFire at ~4.6 u through the west-edge leg"]) — and samples
//! his clip/shots/state EVERY tick of the whole fight window.
//!
//! The decompiled rules this pins ([M]):
//! - clip 5 / reserve (+0xBBE) 5 at init for weapon-param 0x1e guards [M NDrone2_DefaultInit
//!   0x14b300:345-348 — the p9-keyed switch `case 0x1e/0x1f/0x24/0x25: uVar17 = 5`],
//! - every shot: `*(short*)(drone+0xBBC) -= 1` [M DroneWeap_FireWeapon 0x178f68:270],
//! - clip < 1 → combat state 0x69 AimStandReload [M DroneFunc_CombatState 0x148338:196-197],
//! - reload ENTER: `+0xBBC = +0xBBE` (refill to 5; the reserve itself is never decremented in
//!   the campaign path — the +0xBBE writers are only DefaultInit and the MP-gated BOTWEAP
//!   layer) [M NDrone2_DSTATE_AimStandReload 0x1666d8 case 1].
//! The 0x1e AmmoInGun gate [M 0x178f68:94-99] does NOT apply to guard #1: it keys on the mode
//! byte `Drone+0xC5 == 0x1e`, and guard #1's mode is `GetDTYPENEW(dtype 5, block 0)` = 0
//! [M NDrone2_GetDroneTypeAttackTypeFriend 0x14cbd8:23-27 + GetDTYPENEW 0x14c938 tail].

#[path = "common/walkreal0c.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

struct Sample { tick: u32, clip: i32, shots: usize, state: u8 }

fn sample(w: &Walker, di: usize, out: &mut Vec<Sample>) {
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[di];
    out.push(Sample { tick: w.total_ticks, clip: d.clip, shots: d.shots.len(), state: d.state });
}

#[test]
fn guard1_rounds_at_the_ring_pinned_over_a_real_fight() {
    let iso = iso();
    // Real input from the spawn to the ladder base (the shared prologue, top-up on — Bond
    // never fires back, so the fight is unbounded and guard #1's rhythm is observable).
    let mut w = walker::walk_spawn_to_the_yard_ladder_base(&iso);
    let di = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter()
            .find(|d| d.placement == 3 && d.weapon == 0x1e)
            .expect("placement #3 (guard #1, weapon param 0x1e) must exist at load")
            .index
    };
    {
        let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
        eprintln!("[0c-ammo] guard #1 = drone index {} at load: clip {} clip_max {} weapon {:#x} shots {} state {:#x}",
            d.index, d.clip, d.clip_max, d.weapon, d.shots.len(), d.state);
    }
    let mut samples: Vec<Sample> = Vec::new();
    sample(&w, di, &mut samples);

    // Leg 7 (REAL, verbatim from walk_spawn_to_the_band): the ladder walk-in grab + climb.
    w.walk_to(Wp::new(10.3, 1.05, -186.8).r(0.7), "the ladder approach");
    let mut climbed = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbed = true; break; }
        let desired = (9.669741f32 - sim.bond.pos[0]).atan2(-187.3427f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: if err.abs() > 0.2 { err.clamp(-0.08, 0.08) } else { 0.0 },
            forward: if err.abs() > 0.2 { 0.3 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }
    assert!(climbed, "walking into the yard ladder #164 never grabbed it");
    let mut mantled = false;
    for _ in 0..2400 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate != nf_game::player::SS_CLIMB && sim.bond.pos[1] > 2.0 { mantled = true; break; }
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }
    assert!(mantled, "the yard climb never mantled past climb_top 12.139");
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.substate == nf_game::player::SS_WALK && sim.bond.pos[1] > 10.5 { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }

    // Leg 8 (REAL): the patrol-ring legs (the shared prologue's coordinates), sampled.
    w.walk_to(Wp::new(10.5, 11.1, -190.5).r(1.2), "the ring diagonal");
    sample(&w, di, &mut samples);
    w.walk_to(Wp::new(12.0, 11.1, -196.0).r(1.2), "the ring diagonal 2");
    sample(&w, di, &mut samples);
    w.walk_to(Wp::new(12.0, 11.1, -203.0).r(1.2), "the ring north-west corner");
    sample(&w, di, &mut samples);
    w.walk_to(Wp::new(4.0, 11.1, -207.5).r(1.2), "the ring west edge");
    sample(&w, di, &mut samples);

    // The fight window: hold Bond still at the west edge (tiny forward stick each tick so the
    // walk substate holds) and sample guard #1 every tick for 2400 ticks (~40 s of game time).
    for _ in 0..2400 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { forward: 0.05, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }

    // ---- the pin ----
    let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
    eprintln!("[0c-ammo] guard #1 after the window: clip {} clip_max {} shots {} state {:#x} alive {}",
        d.clip, d.clip_max, d.shots.len(), d.state, d.health > 0.0);

    // The clip pair [M 0x14b300:345-348].
    assert_eq!(d.clip_max, 5, "weapon-0x1e guard's clip_max must be the game's 5 [M 0x14b300:345-348]");
    assert!(d.clip >= 0 && d.clip <= 5, "clip must stay within 0..=5, saw {}", d.clip);

    // Reload ENTERs: the sampled state 0x69 windows [M 0x148338:196-197 + 0x1666d8 case 1].
    let mut reloads: Vec<(u32, i32, usize)> = Vec::new(); // (enter tick, clip at enter, shots so far)
    let mut prev_state = samples.first().map(|s| s.state).unwrap_or(0);
    for s in &samples {
        if s.state == 0x69 && prev_state != 0x69 {
            reloads.push((s.tick, s.clip, s.shots));
        }
        prev_state = s.state;
    }
    eprintln!("[0c-ammo] reload windows observed: {} ({:?})", reloads.len(),
        reloads.iter().map(|(t, c, sh)| (t, *c, *sh)).collect::<Vec<_>>());
    assert!(!reloads.is_empty(), "guard #1 never reloaded during the window — the clip<1 → 0x69 rule [M 0x148338:196-197] never fired");

    // Exactly 5 shots since the previous reload (the clip's capacity), and the ENTER sample's
    // clip is already the refilled 5 (the ENTER handler refills on the transition tick itself
    // [M 0x1666d8 case 1] — a clip read at the first 0x69 sample is post-refill).
    for (i, (tick, clip, shots)) in reloads.iter().enumerate() {
        let prev_shots = if i == 0 { samples.first().map(|s| s.shots).unwrap_or(0) } else { reloads[i - 1].2 };
        let per_clip = shots - prev_shots;
        assert_eq!(per_clip, 5,
            "reload #{} at tick {}: {} shots since the previous reload, the game's clip is 5 [M 0x14b300:345-348 + 0x178f68:270]",
            i, tick, per_clip);
        assert_eq!(*clip, 5, "reload #{} at tick {}: the ENTER refill must already show clip 5 [M 0x1666d8 case 1]", i, tick);
    }

    // The cadence + reload spans [L this run]: the game's 0x1e sniper law is burst 1, interval
    // +0x40 = 120 frames, BurstDelay_Max 60 forced at ≥30 u [M DroneWeap_NextBulletTime /
    // BurstDelay 0x179ba8, pinned by sniper-range-accuracy-0c-0d] — i.e. ~180-tick gaps at
    // range, tighter up close. Dump what the port does so the pin is checkable.
    let frames = &d.shots;
    let gaps: Vec<u32> = frames.windows(2).map(|p| p[1] - p[0]).collect();
    eprintln!("[0c-ammo] shot frames: {:?}", frames);
    eprintln!("[0c-ammo] shot gaps: {:?}", gaps);
    let mut spans: Vec<(u32, u32)> = Vec::new();
    let mut start: Option<u32> = None;
    let mut prev = 0u8;
    for s in &samples {
        if s.state == 0x69 && prev != 0x69 { start = Some(s.tick); }
        if s.state != 0x69 && prev == 0x69 { spans.push((start.unwrap(), s.tick)); }
        prev = s.state;
    }
    eprintln!("[0c-ammo] reload spans (enter,exit): {:?} — lengths {:?}", spans,
        spans.iter().map(|(a, b)| b - a).collect::<Vec<_>>());



    // No shots during a reload window: every sampled tick inside 0x69 keeps `shots` flat.
    let mut in_reload = false;
    let mut shots_at_enter = 0usize;
    for s in &samples {
        if s.state == 0x69 {
            if !in_reload { in_reload = true; shots_at_enter = s.shots; }
            assert_eq!(s.shots, shots_at_enter, "guard #1 fired during the AimStandReload window at tick {}", s.tick);
        } else {
            in_reload = false;
        }
    }

    // The fight really happened: guard #1 fired at the ring during the window.
    let fired = d.shots.len();
    eprintln!("[0c-ammo] total rounds fired by guard #1 over the walk+window: {}", fired);
    assert!(fired >= 10, "guard #1 fired only {} rounds — the fight window did not engage him", fired);
}

/// The real-health fight (the full-damage scenario): the aimed-stands choreography
/// (`with_yard_aimed_stands` + `with_far_sniper_stands`) under `with_real_damage` — Bond
/// returns fire and the stands kill the deck snipers per the master real-damage walk. Guard
/// #1's rounds are dumped from his first shot to his death (the ring-corner stand shoots
/// him). The reload rhythm asserts stay in the top-up test above; this run OBSERVES and
/// prints so the fight-side behavior is checkable without depending on when the stand's
/// round lands.
#[test]
fn guard1_rounds_at_the_ring_real_health_fight() {
    walker::with_yard_aimed_stands(|| {
        walker::with_far_sniper_stands(|| walker::walker::with_real_damage(real_health_body))
    });
}

fn real_health_body() {
    let iso = iso();
    let mut w = walker::walk_spawn_to_the_yard_ladder_base(&iso);
    let di = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter()
            .find(|d| d.placement == 3 && d.weapon == 0x1e)
            .expect("placement #3 (guard #1, weapon param 0x1e) must exist at load")
            .index
    };
    let mut samples: Vec<Sample> = Vec::new();
    sample(&w, di, &mut samples);
    eprintln!("[0c-ammo/rh] guard #1 at the ladder base: clip {} shots {} state {:#x} health {:.1}",
        w.scene.sim.as_ref().unwrap().drones.list[di].clip,
        w.scene.sim.as_ref().unwrap().drones.list[di].shots.len(),
        w.scene.sim.as_ref().unwrap().drones.list[di].state,
        w.scene.sim.as_ref().unwrap().drones.list[di].health);

    // The climb + ring legs (verbatim from the shared prologue).
    w.walk_to(Wp::new(10.3, 1.05, -186.8).r(0.7), "the ladder approach");
    let mut climbed = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbed = true; break; }
        let desired = (9.669741f32 - sim.bond.pos[0]).atan2(-187.3427f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: if err.abs() > 0.2 { err.clamp(-0.08, 0.08) } else { 0.0 },
            forward: if err.abs() > 0.2 { 0.3 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }
    assert!(climbed, "walking into the yard ladder #164 never grabbed it");
    let mut mantled = false;
    for _ in 0..2400 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate != nf_game::player::SS_CLIMB && sim.bond.pos[1] > 2.0 { mantled = true; break; }
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }
    assert!(mantled, "the yard climb never mantled past climb_top 12.139");
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.substate == nf_game::player::SS_WALK && sim.bond.pos[1] > 10.5 { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }
    // The master choreography's ladder-top stands (the deck snipers that chew Bond on the
    // walkway — the real-damage walk runs them right after the mantle [L walkreal0c.rs]).
    walker::yard_sniper_stands(&mut w, "ladder-top", 3);
    w.walk_to(Wp::new(10.5, 11.1, -190.5).r(1.2), "the ring diagonal");
    sample(&w, di, &mut samples);
    w.walk_to(Wp::new(12.0, 11.1, -196.0).r(1.2), "the ring diagonal 2");
    sample(&w, di, &mut samples);
    w.walk_to(Wp::new(12.0, 11.1, -203.0).r(1.2), "the ring north-west corner");
    sample(&w, di, &mut samples);
    // The ring-corner stands: the master's "A player shoots the guard standing in front of
    // him before walking on" — the fight that kills guard #1 [L walkreal0c.rs].
    walker::yard_sniper_stands(&mut w, "ring-corner", 3);
    w.walk_to(Wp::new(4.0, 11.1, -207.5).r(1.2), "the ring west edge");
    sample(&w, di, &mut samples);
    walker::yard_sniper_stands(&mut w, "west-edge", 3);
    // The hold window (the fight continues while Bond stands at the west edge).
    for _ in 0..1800 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { forward: 0.05, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        sample(&w, di, &mut samples);
    }

    let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
    eprintln!("[0c-ammo/rh] guard #1 at the end: clip {} shots {} state {:#x} alive {}",
        d.clip, d.shots.len(), d.state, d.health > 0.0);
    let frames = &d.shots;
    let gaps: Vec<u32> = frames.windows(2).map(|p| p[1] - p[0]).collect();
    eprintln!("[0c-ammo/rh] shot frames: {:?}", frames);
    eprintln!("[0c-ammo/rh] shot gaps: {:?}", gaps);
    // Reload windows in this run.
    let mut reloads: Vec<(u32, i32, usize)> = Vec::new();
    let mut prev_state = samples.first().map(|s| s.state).unwrap_or(0);
    for s in &samples {
        if s.state == 0x69 && prev_state != 0x69 { reloads.push((s.tick, s.clip, s.shots)); }
        prev_state = s.state;
    }
    eprintln!("[0c-ammo/rh] reload ENTERs: {:?}", reloads.iter().map(|(t, c, sh)| (t, *c, *sh)).collect::<Vec<_>>());
    // The clip bookkeeping under fire: the clip may only drop when a shot fires, and every
    // reload ENTER must arrive with 5 shots since the previous one [M 0x178f68:270 +
    // 0x14b300:345-348] — a dead guard's last (partial) clip is exempt.
    let base = samples.first().map(|s| s.shots).unwrap_or(0);
    for (i, (tick, _clip, shots)) in reloads.iter().enumerate() {
        let prev_shots = if i == 0 { base } else { reloads[i - 1].2 };
        let per_clip = shots - prev_shots;
        assert_eq!(per_clip, 5,
            "[rh] reload #{} at tick {}: {} shots since the previous reload — the game's clip is 5 [M 0x14b300:345-348 + 0x178f68:270]",
            i, tick, per_clip);
    }
    eprintln!("[0c-ammo/rh] bond health at the end: {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
}
