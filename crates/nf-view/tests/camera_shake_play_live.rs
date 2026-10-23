//! camera-shake-exact (task 969): the explosion camera shake through the REAL input path —
//! `nf_view::load_weapons` → the panel give → `Sim::step_bond` — with the blast entering at the
//! `Explode_Create` 0x17cbd0 port (`Drones::pending_blasts` → `spawn_blast`) and via a thrown
//! frag. Numbers only; every game-behaviour assertion cites its evidence (docs/systems/
//! camera-shake.md):
//! - the set [M Camera_Shake 0x1bc430]: d to the viewer EYE (viewer+0x150, the port's
//!   `Drones::viewer_pos`), `min(2r·(1−(d/2r)²), 10)` as a FLOAT (min.s @0x1bc4d0 + swc1
//!   @0x1bc4d4 — no integer truncation), stored only when d ≤ 2r and the value > 0,
//!   overwriting any earlier amount. The set lands mid-tick; the tick's tail runs the
//!   `Camera_Update` block, so the POST-TICK amount is the decayed set value — uniform in
//!   (0.75·set, set] [M 0x1bb6d8:38-40]. The exact set value is unit-pinned in nf-game
//!   (`the_shake_amount_follows_camera_shake`).
//! - the update [M 0x1bb6d8:35-43]: while shake > 0 the block adds FOUR shared-stream draws
//!   per tick — three `Rand_FRand_MVar2_Vec` offsets (X, Y, Z, each uniform [−shake/60, 0)
//!   with shake the PRE-decay amount, 0x1e00d0) then one `Rand_FRand(shake·0.25)` decay. The
//!   exact 4-draw cadence is unit-pinned (`camera_shake_tick_draws_three_offsets_then_the_decay`);
//!   a live level's other systems share the stream, so here the offsets' window and the decay
//!   bounds carry the pin.
//! - the eye contract [M 0x1bb6d8:47-50 + Mat_World2ViewMat 0x1e3280]: viewer_pos next tick =
//!   this tick's end eye + this tick's offset (the game's +0x150 between Camera_Updates).
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_game::player::{Bond, Input};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Park Bond on the 05 spawn plain (the playtest spot, known open floor), frag (0x34) drawn.
fn scene_with_frag() -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load_weapons(&iso, "07000005.bin", false, &[0x34]).expect("load 05");
    let (msg, switch_to) = nf_view::panel::apply(&mut scene, &[
        ("do".to_string(), "weapon".to_string()), ("id".to_string(), 52.to_string()), ("rounds".to_string(), "5".to_string())]);
    let _ = msg;
    assert_eq!(switch_to, Some(0x34), "the panel must switch to the frag");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = Bond::new([105.97, 24.87, 110.12], 0.0);
    sim.bond.pitch = 0.0;
    sim.disabled = false;
    let mut drawn = false;
    for _ in 0..180 {
        sim.step_bond(Input { switch_to: Some(0x34), ..Default::default() }, false, false, false);
        let gun = &sim.gun;
        if gun.weapon == 0x34 && gun.switch_state == 0 && gun.reload_state == 0 { drawn = true; break; }
    }
    assert!(drawn, "the frag drew");
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

/// The set → tail-decay → eye-contract chain, through `step_bond`, with the blast entering at
/// the `Explode_Create` port. Determinism: the scenario runs twice from fresh loads and the
/// (shake, offset) series must match bit-for-bit.
#[test]
fn blast_shake_set_decay_and_eye_contract_through_the_real_tick() {
    if iso().is_none() { eprintln!("skipped: NF_ISO not set"); return; }
    let mut series = Vec::new();
    for _run in 0..2 {
        let mut scene = scene_with_frag();
        let sim = scene.sim.as_mut().unwrap();
        // The blast 1.8 east, 2.4 below the eye: d = 3.0 exactly (1.8² + 2.4² = 3²), radius 3
        // (range 6) → the set value 6·(1 − 0.25) = 4.5, under the cap.
        let eye0 = sim.bond.head();
        let blast = [eye0[0] + 1.8, eye0[1] - 2.4, eye0[2]];
        let mut local: Vec<(u32, f32, [f32; 3], [f32; 3], [f32; 3])> = Vec::new(); // (t, shake, off, viewer_pos, head_after)
        for t in 0..40 {
            if t == 0 { sim.drones.pending_blasts.push((blast, 3.0, 0.0, usize::MAX, 0, 0x0600_0052, 3.0)); }
            sim.step_bond(Input::default(), false, false, false);
            let h = sim.bond.head();
            local.push((t, sim.drones.shake, sim.drones.shake_off, sim.drones.viewer_pos, h));
        }
        // The SET [M 0x1bc430 through spawn_blast]: d against the eye at the tick's start
        // (viewer_pos == the untouched head on tick 0) = 3.0 → the float set 4.5.
        let d = nf_game::drone::vec_dist3d(blast, local[0].3);
        let want = 6.0 * (1.0 - (d / 6.0) * (d / 6.0));
        assert!((d - 3.0).abs() < 1.0e-5, "the fixture's d is 3.0 (got {d})");
        assert!((want - 4.5).abs() < 1.0e-5, "the float set is 4.5 — the old (int) port would have produced 4 [M min.s 0x1bc4d0]");
        // Post-tick the tail block has decayed it once: uniform (0.75·set, set] [M :38-40].
        assert!(local[0].1 <= want + 1.0e-6 && local[0].1 > 0.75 * want - 1.0e-5,
            "tick 0 shake {} outside the one-decay window ({}, {}]", local[0].1, 0.75 * want, want);
        // The offset window [M 0x1e00d0 with param_1 = param_2 = set/60 — the PRE-decay shake]:
        // each axis in [−set/60, 0], one-sided negative on the WORLD axes.
        let a0 = want / 60.0;
        for k in 0..3 {
            assert!(local[0].2[k] <= 0.0 && local[0].2[k] >= -a0 - 1.0e-6,
                "axis {k} offset {:e} outside [−set/60, 0] (a = {a0:e})", local[0].2[k]);
        }
        // The decay continues: monotone, each tick at most shake·0.25 down, never below 0.
        for w in local.windows(2) {
            let (s0, s1) = (w[0].1, w[1].1);
            if s0 > 0.0 {
                assert!(s1 <= s0 + 1.0e-6 && s1 > 0.0, "monotone decay {s0} → {s1}");
                assert!(s1 > 0.75 * s0 - 1.0e-4, "the decay draw removes at most shake·0.25 ({s0} → {s1})");
            } else {
                assert_eq!(s1, 0.0, "once at 0 it stays 0 without a new blast");
                assert_eq!(w[1].2, [0.0; 3], "no offset while shake == 0 [M 0x1bb6d8:35]");
            }
        }
        // 40 ticks of geometric decay (mean ×0.875) leave it well under half.
        assert!(local[39].1 < 0.5 * want, "40 ticks from {want}: {} (the decay must be visible-invisible)", local[39].1);
        // The EYE contract [M 0x1bb6d8:47-50]: viewer_pos(t+1) = head_after(t) + off(t).
        for w in local.windows(2) {
            for k in 0..3 {
                assert!((w[1].3[k] - (w[0].4[k] + w[0].2[k])).abs() < 1.0e-4,
                    "tick {}: viewer_pos[{}] {} vs head+off {} — the +0x150 port",
                    w[1].0, k, w[1].3[k], w[0].4[k] + w[0].2[k]);
            }
        }
        series.push(local);
    }
    // Determinism: two fresh loads produce the identical (shake, offset) series.
    for (a, b) in series[0].iter().zip(series[1].iter()) {
        assert_eq!((a.1, a.2), (b.1, b.2), "the series diverged at tick {}", a.0);
    }
}

/// Sister [M 0x1bc49c-a4]: a blast OUTSIDE its range leaves the standing amount — the store is
/// skipped (the old port shape wrote 0 outside).
#[test]
fn blast_outside_the_range_leaves_the_standing_amount() {
    if iso().is_none() { eprintln!("skipped: NF_ISO not set"); return; }
    let mut scene = scene_with_frag();
    let sim = scene.sim.as_mut().unwrap();
    // With nothing standing, an out-of-range blast (radius 1 = range 2, d 6) sets nothing.
    let eye = sim.bond.head();
    sim.drones.pending_blasts.push(([eye[0] + 6.0, eye[1], eye[2]], 1.0, 0.0, usize::MAX, 0, 0x0600_0052, 1.0));
    sim.step_bond(Input::default(), false, false, false);
    assert_eq!(sim.drones.shake, 0.0, "out of range and nothing standing → still 0");
    // A standing amount survives an out-of-range blast (only the tail decay touches it).
    sim.drones.shake = 2.5;
    let eye = sim.bond.head();
    sim.drones.pending_blasts.push(([eye[0] + 60.0, eye[1], eye[2]], 1.0, 0.0, usize::MAX, 0, 0x0600_0052, 1.0));
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.shake > 0.75 * 2.5 - 1.0e-4 && sim.drones.shake <= 2.5,
        "the standing 2.5 only decayed (got {})", sim.drones.shake);
    // The negative control: the same radius IN range fires the store (2·(1 − 0.25) = 1.5 set,
    // then one decay → (1.125, 1.5]).
    let eye = sim.bond.head();
    sim.drones.pending_blasts.push(([eye[0] + 1.0, eye[1], eye[2]], 1.0, 0.0, usize::MAX, 0, 0x0600_0052, 1.0));
    sim.step_bond(Input::default(), false, false, false);
    assert!(sim.drones.shake > 1.125 - 1.0e-4 && sim.drones.shake <= 1.5 + 1.0e-6,
        "in range the store fired (got {})", sim.drones.shake);
}

/// The real throw: a frag popped a few units away shakes — the offsets stay in the one-sided
/// window for every shaking tick and the amount decays to nothing.
#[test]
fn frag_pop_shakes_through_the_throw_pipeline() {
    if iso().is_none() { eprintln!("skipped: NF_ISO not set"); return; }
    let mut scene = scene_with_frag();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pitch = -0.55; // aim at the ground a few units ahead
    sim.step_bond(Input { fire: true, ..Default::default() }, true, false, false);
    let mut pops = 0;
    let mut shook = false;
    let mut peak = 0.0f32;
    let mut pop_radius = 0.0f32;
    for t in 0..600 {
        sim.step_bond(Input::default(), false, false, false);
        if pops == 0 && !sim.drones.explosions.is_empty() {
            pops = t + 1;
            pop_radius = sim.drones.explosions.iter().map(|(_, r, ..)| *r).fold(0.0, f32::max);
        }
        if sim.drones.shake > 0.0 {
            shook = true;
            peak = peak.max(sim.drones.shake);
            // The offsets were drawn with the PRE-decay shake; the post-tick read gives
            // post ∈ (0.75·pre, pre] → pre < post/0.75 [M 0x1bb6d8:38-40].
            let a_max = sim.drones.shake / 60.0 / 0.75;
            for k in 0..3 {
                assert!(sim.drones.shake_off[k] <= 0.0 && sim.drones.shake_off[k] >= -a_max - 1.0e-6,
                    "tick {t}: axis {k} offset {:e} outside [−shake_pre/60, 0] (bound {a_max:e})", sim.drones.shake_off[k]);
            }
        } else if shook {
            assert_eq!(sim.drones.shake_off, [0.0; 3], "no offset once the shake died");
        }
        if pops > 0 && t > pops + 240 { break; }
    }
    assert!(pops > 0, "the frag popped");
    assert!(shook, "the pop shook — the blast reached the camera-shake set");
    assert!(peak > 0.5, "a pop a few units from the eye shakes visibly (peak {peak})");
    // The pop's radius gates the call [M 0x17cc28].
    assert!(pop_radius >= 0.5, "the pop's blast radius {pop_radius} passed the gate");
}
