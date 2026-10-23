//! copter-05-flight: the 07000005 gate-0 helicopter (#241) flies its CopterPaths route from the
//! load. The release 0.1.18 audit (proof/09-copters) rendered it FROZEN — frame 60 == frame 180
//! from (104.95, 24.98, 108.83) yaw −3.08 — because that cut predates the merged
//! s/copter-path-flight work (Copter_Move 0x17d628 + CopterPaths + SP_SetPosRot posing; task 958):
//! in the 0.1.18 tree the copter's look was a one-shot script stand-in whose keyless stream
//! entity froze at its creation-tick pose (the posing gap in docs/reports/copter-path-flight.md)
//! and nothing wrote obj+0x90. This file pins the flight the merged tree flies, through the real
//! path only: #241's gate is 0 (placement param 6 = 0 [D the 07000005 census — "no gate → active
//! from the start, the ambient fly-by"]), so the game flies it from the load with no channel
//! poke — every leg below runs on plain 60 Hz steps.
//!
//! The PINE diff points: the test prints the copter position at frames 60 / 120 / 180 / 720 from
//! the load (the audit's f60/f180 anchor included). The live read the main agent should make on a
//! fresh-boot PCSX2: the SP player 0x16df360 (#241's twin, +0x36 = 0 [L ee_start_05.bin]) —
//! SP+0xE0 → +0xC + *(ushort*)(SP+0xE0+0x2A)·4 = the playing SCRIPTINFO pointer
//! [M SP_SetPosRot 0x1dc9e8], whose first 64 bytes are the flight matrix `Script_SetPosRot`
//! writes every flown tick [M 0x1db920 — si+0x00..0x3F, translation si+0x30..0x3B], re-composed
//! against the keyed streams at si+0xA8 (count si+0xAAA). That matrix IS obj+0x90
//! [M Copter_Move 0x17d628:115].

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn the_gate0_copter_flies_its_route_from_the_load() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    // The gate-0 copter is #241 (the ambient fly-by; #242's gate is 66 = 0x42 [D the census]).
    let ci = sim.drones.copters.iter().position(|c| c.gate == 0).expect("the gate-0 copter parses");
    assert_eq!(sim.drones.copters[ci].placement, 241, "the gate-0 copter is placement #241 [D the census]");
    let p0 = sim.drones.copters[ci].pos();
    let near = (p0[0] - 95.92).powi(2) + (p0[1] - 65.28).powi(2) + (p0[2] - (-50.18)).powi(2);
    assert!(near < 4.0, "the copter sits at its placement spot (95.92, 65.28, −50.18) [D the census]; at {p0:?}");
    assert_eq!(sim.drones.copters[ci].state, 0, "obj+0xF4 starts 0 [M 0x17d340:63]");
    // The route: its own node is the [D] framelist — placement 241, kind 0, group 1, pick 0,
    // count 381, reset 0, step 0.09525, keys 382 (the census printout).
    let paths = sim.drones.copter_paths.clone();
    let pi = sim.drones.copters[ci].path.expect("rec+0x54 = its own node [M 0x17d340:52]");
    assert_eq!(paths[pi].placement, 241, "its node is #241's [D the census]");
    assert_eq!((paths[pi].count, paths[pi].keys.as_ref().map_or(0, |k| k.len())), (381, 382),
        "the route facts [D the census printout]");
    assert!((paths[pi].step - 0.09525).abs() < 1e-4, "the t step [D the census printout; M 0x17d340:33-41]");

    // Sister — the audit's frozen symptom: the copter is NOT static over the first 120 ticks.
    let mut p60 = p0;
    for n in 1..=120 {
        sim.step_bond(Default::default(), false, false, false);
        if n == 60 { p60 = sim.drones.copters[ci].pos(); }
    }
    let p1 = sim.drones.copters[ci].pos();
    eprintln!("[05-flight] f60 {p60:?} f120 {p1:?}");
    let moved = (p1[0] - p0[0]).powi(2) + (p1[1] - p0[1]).powi(2) + (p1[2] - p0[2]).powi(2);
    assert!(moved > 1.0, "the copter flew within 120 ticks (d2 = {moved}) — not the frozen f60==f180 render");
    assert_eq!(sim.drones.copters[ci].state, 1, "case 0 scanned, case 1 flies [M 0x17dbb8:115-137]");

    // The route tolerance: every flown tick's pose is the spline of the [D] framelist at the
    // record's own t [M Copter_Move 0x17d628:45-55] — the matrix is interpolated from the t
    // BEFORE this tick's increment, so the expectation is interp(t − step). The window (600
    // ticks ≈ Δt 57) never reaches the count-381 overflow, so no transition bookkeeping.
    let f0 = sim.drones.frame;
    for k in 0..600 {
        sim.step_bond(Default::default(), false, false, false);
        let c = &sim.drones.copters[ci];
        if c.t <= 0.0 { continue; }
        let keys = paths[pi].keys.as_ref().unwrap();
        if let Some(m) = nf_game::door::interp(keys, 0, paths[pi].count, c.t - paths[pi].step, true) {
            let want = [m[3][0], m[3][1], m[3][2]];
            let got = c.pos();
            let d = (want[0] - got[0]).powi(2) + (want[1] - got[1]).powi(2) + (want[2] - got[2]).powi(2);
            assert!(d < 1e-2, "tick {k}: obj+0x90 = the spline pose of t ({want:?} vs {got:?}) [M Copter_Move 0x17d628:50-55]");
        }
        let total = 120 + (sim.drones.frame - f0); // the load-relative frame (the sister ran 120)
        if total == 180 || total == 720 {
            eprintln!("[05-flight] f{} {:?} (t {:.3})", total, c.pos(), c.t);
        }
    }
    // The SP base rides the copter matrix: SP_SetPosRot(*(rec+0x2c), obj+0x90) every flown tick
    // [M 0x17d628:115] — the draw slot's posing source (the 0.1.18 freeze was exactly this beat
    // missing). #241's SP plays from the load (gate 0, +0x36 = 0 [L ee_start_05.bin]).
    let c1 = sim.drones.copters[ci].pos();
    let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 241)
        .expect("#241's ScriptPlayer exists from the create beat [M 0x17d340:66-70]");
    assert!(!sp.hidden, "the gate-0 SP plays [M 0x17d340:73-74's gate!=0 guard; L ee_start_05 +0x36 = 0]");
    let base = sp.info[sp.cur].as_ref().map(|s| s.matrix).unwrap_or(sp.matrix);
    let bp = [base[3][0], base[3][1], base[3][2]];
    let d = (bp[0] - c1[0]).powi(2) + (bp[1] - c1[1]).powi(2) + (bp[2] - c1[2]).powi(2);
    assert!(d < 1.0, "the SP base rides the copter matrix ({bp:?} vs {c1:?}) [M 0x17d628:115]");
}
