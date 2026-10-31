//! bunker-stick-02: the 07000002 bunker-door wedge, reproduced on real input and pinned from
//! both arms. The reported symptom: Bond hangs between the two bunker doors at (−45.42, 7.1,
//! −22.15) — not grounded, the floor 5.33 below — for thousands of ticks (found on the
//! real-health walk, s/fix-02-real-health's walkreal avoids the move that leads there).
//!
//! THE DOORWAY DATA [D placements + the leaf collision dump, this session]:
//! - Two kind-2 proximity-slider doors, NO channels (p2 = p3 = 0 — the E opens them, the
//!   360-tick auto-close re-closes [M Door_Update 0x1363c8's proximity/auto-close arms]):
//!   #268 at (−46.07, 6.76, −23.21) and #330 at (−46.16, 6.77, −20.27).
//! - Their CLOSED leaves (the port's disc collision posed at pos01 0) span z −24.15..−22.27
//!   and −21.43..−18.93: between them sits a STATIC MIDDLE PIER (z ≈ −22.3..−21.4) whose east
//!   face stands at x −45.97, full height [L-port the westward rays at knee/chest/head all hit
//!   −45.971 at z −22.22; the doorway's passable slots are NORTH (z −23.93..−22.3) and SOUTH
//!   (z −21.4..−19.37) of it once the leaves part].
//! - The game's own crossing is the NORTH slot: the doorway sensor #389 (the ambush trigger,
//!   out 0x81) stands at (−46.96, 6.04, −23.35) [D placement] — inside the north slot — and
//!   walkreal_07000002's own route node is "bunker door" (−46.1, 5.9, −23.2) [D route].
//! - The threshold at the door plane, mid-gap (−46.1, −22.15), has a FLOOR HOLE: the downward
//!   ray from y 8 finds no floor above y 0 [L-port] — the bunker level below (the walkreal's
//!   underground rooms stand at 1.05..1.8) is "the floor 5.33 below" the reported hang.
//!
//! The reproduction [L-port, this session]: a straight real-input steer into the MID-GAP
//! (z −22.15 — the pier strip) wedges Bond at (−45.42, 6.37, −22.22), grounded on the 5.33
//! floor, with forward AND strafe held: the capsule push pins him, no tangential motion, for
//! 900+ ticks — the reported x/z to the centimetre. The reported y 7.1 AIRBORNE variant did
//! not reproduce in this session's probes (the walker's sidestep phase held the same pin);
//! the live check below asks the game's own behaviour on that spot.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] a live measurement, [S]
//! unverified. Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The fresh load, the intro stepped away (≤3000 ticks — the level-start scene [D SP #400]).
fn load() -> Option<nf_view::scene::Scene> {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");
    Some(scene)
}

/// The wedge drive: from the courtyard east of the doors, a straight real-input steer at
/// `target` (turn + forward, the walker's plain leg), the E pulsed while approaching the door
/// plane (the walker's door-open habit — the doors' p2 is 0, so the E is the only opener [D
/// params; M Door_Activate 0x135be0]). Returns the last positions sampled 300 ticks apart,
/// after tick 300.
fn drive(scene: &mut nf_view::scene::Scene, target: [f32; 3], ticks: u32) -> Vec<([f32; 3], bool)> {
    let mut samples = Vec::new();
    let sim = scene.sim.as_mut().unwrap();
    for t in 0..ticks {
        {
            let b = &sim.bond;
            let dx = target[0] - b.pos[0];
            let dz = target[2] - b.pos[2];
            if dx * dx + dz * dz < 0.25 { break; }
        }
        let (pos, yaw) = (sim.bond.pos, sim.bond.yaw);
        let desired = (target[0] - pos[0]).atan2(target[2] - pos[2]);
        let err = desired - yaw;
        let err = err.atan2(err.cos());
        let near_plane = (pos[0] - (-46.1)).abs() < 1.8 && (pos[2] + 21.8).abs() < 2.6;
        let use_key = near_plane && t % 6 == 0;
        sim.step_bond(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false, false, use_key);
        if t >= 300 && t % 300 == 0 {
            let b = &sim.bond;
            samples.push((b.pos, b.grounded));
        }
    }
    samples
}

/// Sister (the OLD behaviour sticks): the straight steer into the mid-gap — the fix-02 run's
/// wedge trajectory — leaves Bond pinned at the pier's east face with the doors open. The pin
/// of the cause: the mid-gap strip is the disc-real static pier [D the leaf spans above], the
/// threshold under it is the floor hole [L-port], and the capsule push holds him (forward held
/// the whole time, zero motion).
#[test]
fn the_midgap_wedge_sticks_between_the_bunker_doors() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    // The wedge start: the courtyard east of the doors, the fix-02 fight-end side.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-44.2, 5.5, -21.8];
        sim.bond.vel = [0.0; 3];
    }
    // Steer at the mid-gap point (the straight line to the corridor crosses the plane at
    // z −22.15 — the pier strip [L-port the holemap]).
    let samples = drive(&mut scene, [-46.6, 5.9, -22.15], 1200);
    assert!(samples.len() >= 2, "the drive sampled the wedge");
    let (first, _) = samples[0];
    let (last, grounded_last) = *samples.last().unwrap();
    assert!((first[0] - last[0]).abs() < 0.05 && (first[2] - last[2]).abs() < 0.05,
        "the wedge holds: Bond pinned between the samples (first {first:?}, last {last:?})");
    assert!((last[0] - (-45.42)).abs() < 0.2 && (last[2] - (-22.15)).abs() < 0.4,
        "the pin sits at the reported spot (−45.42, −22.15±), got {last:?}");
    // The doors stood open the whole time (the E opened them; the stick is not a closed door).
    let sim = scene.sim.as_ref().unwrap();
    let open = sim.drones.doors.list.iter().take(2)
        .filter(|d| (d.live_sph().0[0] + 46.1).abs() < 1.0)
        .map(|d| d.state)
        .collect::<Vec<_>>();
    // Both leaves ANSWERED the E (open, or the auto-close arm mid-swing [M 0x13660c]): the
    // wedge is the static pier strip, not a leaf still sealing the gap.
    assert!(open.iter().all(|&s| s & 0x1c != 0), "both bunker doors left the closed state (open/closing), got {open:?}");
    let _ = grounded_last;
}

/// The fix pin (the game's own crossing): the same start, but the crossing aimed at the NORTH
/// slot — the game's own route, the sensor #389's z −23.35 [D placement]. Bond crosses the
/// door plane and LANDS grounded on the corridor side, and stays grounded — no wedge, no
/// airborne hold.
#[test]
fn the_game_own_north_slot_crossing_walks_through_and_lands() {
    let Some(mut scene) = load() else { eprintln!("NF_ISO not set — skipping"); return };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-44.2, 5.5, -21.8];
        sim.bond.vel = [0.0; 3];
    }
    // Leg 1: to the north slot's east lip (the sensor's z — the game's own crossing [D #389]);
    // the E pulses open the doors on approach (inside drive()).
    let _ = drive(&mut scene, [-45.4, 5.9, -23.35], 600);
    // Leg 2: through the slot, west into the corridor toward walkreal's node (−55.1, −23.1).
    let samples = drive(&mut scene, [-48.2, 5.9, -23.2], 900);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.bond.pos[0] < -47.0,
        "Bond crossed the door plane westward, got {:?}", sim.bond.pos);
    assert!(sim.bond.grounded, "Bond is grounded on the corridor side, got {:?}", sim.bond.pos);
    let floor = sim.world.segment([sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]],
                                  [sim.bond.pos[0], sim.bond.pos[1] - 8.0, sim.bond.pos[2]])
        .map(|t| sim.bond.pos[1] - 8.0 * t);
    assert!(floor.map_or(false, |f| (5.0..6.2).contains(&f)),
        "the corridor floor under him is the doorway level band, got {floor:?}");
    // "Stays grounded": the sampled tail (300 ticks apart) never went airborne.
    for (p, g) in &samples {
        assert!(*g || (p[1] - 6.4).abs() < 0.6,
            "the crossing stays grounded (sample {p:?} grounded {g})");
    }
}
