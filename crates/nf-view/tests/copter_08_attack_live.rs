//! The 07000008 extraction copter's attack (`Copter_Update` 0x17dbb8:138-242, copter-08-gondola-flight):
//! the spot check opens the attack passes, the copter's position keeps changing across them (the
//! route step is the decompile's `count · REC_FRAME_RATE · (FRAME_RATE / scale)` number), the
//! machine draws its rolls, and the mission completes only on the kill — an unshot copter never
//! completes it. The kill itself through real trigger pulls is `walkreal_07000008_live`'s
//! `walk_the_gondola_ride_to_the_end` (the walk accumulates the weapon contact damage into
//! rec+0x64 [M 0x17dbb8:277-353] and the death writes ch 0x4B → the mission end 0x0730008).
//!
//! The harness stand-ins (the same class `walkreal` documents): `Channels::set(5, …)` stands in
//! for the bottom-of-mountain cinematic's ch-5 write [D stream 10], and Bond is stood in the
//! copter's view cone each tick for "walking into its fire line" — the spot check's inputs (the
//! 1000-unit gate, the 45° cone, the LOS) are the game's.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The [D] route step of the ch-5 copter's own path (the copter-paths dump): count 8605,
/// scale 30 → step = 8605·(1/60)·(30/30) = 0.25309 t-units per flying tick [M 0x17d340:33-41].
const ROUTE_STEP: f32 = 0.25309;

#[test]
fn the_approach_keeps_moving_on_the_route_step() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load 07000008");
    {
        let sim = scene.sim.as_mut().unwrap();
        let ci = sim.drones.copters.iter().position(|c| c.gate == 5).expect("the ch-5 copter [D #45]");
        let f = sim.drones.frame;
        sim.drones.channels.set(5, 1, f);
        // The approach: the copter advances exactly the route step each flying tick, and every
        // 240-tick moving window displaces it by tens of units — the flight never parks.
        let mut last = sim.drones.copters[ci].pos();
        let mut window_dist = 0.0f32;
        let mut windows = Vec::new();
        for tick in 0..1440u32 {
            sim.step_bond(Default::default(), false, false, false);
            let c = &sim.drones.copters[ci];
            if tick == 0 { continue; } // case 0 scans; case 1 flies from the next tick
            assert!(
                (c.t - (tick as f32) * ROUTE_STEP).abs() < 0.06,
                "t advances by the [D] route step 0.25309/tick (t {} at tick {})", c.t, tick
            );
            let p = c.pos();
            window_dist += ((p[0] - last[0]).powi(2) + (p[1] - last[1]).powi(2) + (p[2] - last[2]).powi(2)).sqrt();
            last = p;
            if tick % 240 == 0 {
                windows.push(window_dist);
                eprintln!("window {}: the copter flew {:.1} units, at ({:.1},{:.1},{:.1})",
                    tick / 240, window_dist, p[0], p[1], p[2]);
                window_dist = 0.0;
            }
        }
        // The [D] route paces the approach at tens of units per 240-tick window (the dump: the
        // keys step ~1.2 units per t-unit over the first third). Far above the parked tail's ~0.
        for w in &windows {
            assert!(*w > 4.0, "the flight keeps moving: every 240-tick window displaces the copter (got {w:.1})");
        }
    }
}

#[test]
fn the_attack_passes_open_and_move_across_their_boundaries() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load 07000008");
    let draws_before;
    let placement;
    {
        let sim = scene.sim.as_mut().unwrap();
        let ci = sim.drones.copters.iter().position(|c| c.gate == 5).expect("the ch-5 copter [D #45]");
        placement = sim.drones.copters[ci].placement;
        let f = sim.drones.frame;
        sim.drones.channels.set(5, 1, f);
        for _ in 0..240 { sim.step_bond(Default::default(), false, false, false); }
        draws_before = sim.drones.rng.draws;
    }
    let mut saw_state2 = false;
    let mut pass_ticks = 0u32;
    let mut pos_at_pass_start: Option<[f32; 3]> = None;
    let mut moved_across = false;
    for _ in 0..4000u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            // The gate field zeroes on the unhide tick — index by the stable placement.
            let ci = sim.drones.copters.iter().position(|c| c.placement == placement)
                .expect("the copter stays listed while it flies [M Copter_Delete only on the death]");
            let c = sim.drones.copters[ci].clone();
            // Bond stands on the copter's facing line (Mat_GetDir, row 2 [M 0x1e3398]) — in the
            // 1000-unit gate and the 45° half-angle cone — until 100 pass ticks in, then behind
            // it (out of the cone) so the pass's hidden-arm exit runs [M :237, :241-242].
            let dir = [c.matrix[2][0], c.matrix[2][1], c.matrix[2][2]];
            let sign = if pass_ticks > 100 { -1.0 } else { 1.0 };
            sim.bond.pos = [
                c.pos()[0] + dir[0] * 60.0 * sign,
                c.pos()[1] + dir[1] * 60.0 * sign,
                c.pos()[2] + dir[2] * 60.0 * sign,
            ];
            sim.bond.vel = [0.0; 3];
            if c.state == 2 {
                saw_state2 = true;
                pass_ticks += 1;
                if pos_at_pass_start.is_none() { pos_at_pass_start = Some(c.pos()); }
            } else if let Some(start) = pos_at_pass_start.take() {
                // A pass boundary (the tick after the bail): the copter moved since the pass
                // started [M :190 — Copter_Move runs inside case 2] and the mission is still
                // waiting for the kill.
                let now = c.pos();
                let d2 = (now[0] - start[0]).powi(2) + (now[1] - start[1]).powi(2) + (now[2] - start[2]).powi(2);
                assert!(d2 > 0.04, "the copter moved across the pass boundary (d2 = {d2}) [M Copter_Move 0x17d628]");
                moved_across = true;
            }
            assert!(!sim.drones.channels.on(0x4b), "an unshot copter never completes the mission [M the death rule :358-370]");
        }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if moved_across && pass_ticks > 130 { break; }
    }
    assert!(saw_state2, "the spot check opened the attack (state 2) with Bond in the fire line [M :179-184]");
    assert!(pass_ticks > 6, "the pass held past the gun's 6-tick cadence ({} pass ticks)", pass_ticks);
    assert!(moved_across, "the out-of-cone arm bailed the pass back to state 1 [M :237, :241]");
    // The machine drew the attack rolls (attackmode at the entry, the missile pod draws in case 2).
    {
        let sim = scene.sim.as_ref().unwrap();
        let draws_after = sim.drones.rng.draws;
        assert!(draws_after > draws_before, "the attack drew the shared RNG (attackmode, the pod pick)");
    }
    eprintln!("attack: {pass_ticks} pass ticks over the phases, the position moved across the pass boundary");
}
