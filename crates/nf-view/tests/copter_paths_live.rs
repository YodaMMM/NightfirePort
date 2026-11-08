//! The extraction copter flies the game's CopterPaths route —
//! `Copter_Move` 0x17d628 over the 0x1c nodes `Copter_Create` 0x17d340 builds from the level's
//! 0x2F placements — not the old one-shot script pass. Real-path pins on the four copter levels
//! (05, 08, 0b, 12 — the placement census's kind-0x2F set); the printed route facts are the [D]
//! record.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The four levels with kind-0x2F placements [D the placement census]. `explode` marks
/// GameState+0x12 ∈ {0x0700000b, 0x07000012} — the death goes to state 6 there
/// [M Copter_Update 0x17dbb8:360-363].
const LEVELS: [(&str, bool); 4] = [
    ("07000005.bin", false), // The Exchange I — the two courtyard patrol copters
    ("07000008.bin", false), // The Exchange II — the extraction
    ("0700000b.bin", true),
    ("07000012.bin", true),
];

#[test]
fn every_copter_owns_its_route_node_and_the_0x4b_arrival_stays_off_while_flying() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    for (name, _explode) in LEVELS {
        let mut scene = nf_view::load(&iso, name, false).unwrap_or_else(|e| panic!("load {name}: {e}"));
        let sim = scene.sim.as_mut().unwrap();
        eprintln!("== {name} ==");
        for (i, p) in sim.drones.copter_paths.iter().enumerate() {
            eprintln!(
                "path {i}: placement {} kind {} group {} pick {} count {} reset {} step {:.5} keys {}",
                p.placement, p.kind, p.group, p.pick, p.count, p.reset_t, p.step,
                p.keys.as_ref().map_or(0, |k| k.len())
            );
        }
        for (i, c) in sim.drones.copters.iter().enumerate() {
            eprintln!(
                "copter {i}: placement {} gate {} out {} arrival(0x74) {} countdown {} path {:?} scripts {:?} hp {}",
                c.placement, c.gate, c.out, c.arrival, c.countdown, c.path, c.scripts, c.hp
            );
            assert!(c.path.is_some(), "{name} copter {i}: rec+0x54 = its own node [M 0x17d340:52]");
            assert_eq!(c.state, 0, "{name} copter {i}: obj+0xF4 starts 0 [M 0x17d340:63]");
        }
        assert!(!sim.drones.copter_paths.is_empty(), "{name}: the 0x2F placements built CopterPaths");
    }
}

#[test]
fn the_07000008_extraction_flies_its_route_and_0x4b_waits_for_the_games_rule() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load 07000008");
    let sim = scene.sim.as_mut().unwrap();
    let ci = sim.drones.copters.iter().position(|c| c.gate == 5).expect("the ch-5-gated extraction copter [D #45]");
    // Sister: before ch 5 the copter is hidden and still — state 0, t 0, no arrival scanned.
    assert_eq!(sim.drones.copters[ci].t, 0.0, "rec+0x5c starts 0 [M 0x17d340:54]");
    assert_eq!(sim.drones.copters[ci].arrival_path, None, "case 0 has not run behind the gate");
    assert!(sim.script_entity((0x0600_0051, 0), 0x0200_035D).is_none(), "the SP is hidden [M 0x17d340:73-74]");
    let spawn = sim.drones.copters[ci].pos();
    let f = sim.drones.frame;
    sim.drones.channels.set(5, 1, f);
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.script_entity((0x0600_0051, 0), 0x0200_035D).is_some(), "the gate unhid the SP [M 0x17dbb8 head]");
    assert_eq!(sim.drones.copters[ci].state, 1, "case 0 scanned, case 1 flies [M 0x17dbb8:115-137]");
    // [D] 07000008 has NO kind-2 placement (the dump above): the scan finds none, so rec+0x58
    // stays None and the only group member is the copter's own path (rec+0x78 = 1) [M :115-135].
    assert_eq!(sim.drones.copters[ci].arrival_path, None, "no kind-2 path on the disc");
    assert_eq!(sim.drones.copters[ci].group_paths, 1, "the own path is the group's only kind-0 member");
    // The flight follows the route: the pose lands on the path's keys (the interpolation of the
    // current t over the copter's own framelist), far from the spawn within the flight window.
    let paths_t = sim.drones.copter_paths.clone();
    let mut seen_move = false;
    for _ in 0..600 {
        sim.step_bond(Default::default(), false, false, false);
        let c = &sim.drones.copters[ci];
        if c.t <= 0.0 { continue; }
        let pi = c.path.expect("flying → a current path");
        if let Some(keys) = &paths_t[pi].keys {
            // The matrix was interpolated from the t BEFORE this tick's increment (Copter_Move
            // interpolates, then advances t [M 0x17d628:50-53]).
            if let Some(m) = nf_game::door::interp(keys, 0, paths_t[pi].count, c.t - paths_t[pi].step, true) {
                let want = [m[3][0], m[3][1], m[3][2]];
                let got = c.pos();
                let d = (want[0] - got[0]).powi(2) + (want[1] - got[1]).powi(2) + (want[2] - got[2]).powi(2);
                assert!(d < 1e-2, "obj+0x90 = the spline pose of t ({want:?} vs {got:?}) [M Copter_Move 0x17d628:50-55]");
            }
        }
        let moved = (c.pos()[0] - spawn[0]).powi(2) + (c.pos()[1] - spawn[1]).powi(2) + (c.pos()[2] - spawn[2]).powi(2);
        if moved > 1.0 { seen_move = true; }
    }
    assert!(seen_move, "the copter left its spawn along the route (SP_SetPosRot poses the SP) [M 0x17d628:115]");
    // The game's arrival rule: while the route flies (no damage), ch 0x4B never fires — the old
    // one-shot port wrote it at 299 ticks [S the removed stand-in]; the real rule needs state 4
    // (hp < damage, disasm 0x17e390-0x17e3c8) → the arrival path adoption [M :244-254].
    assert!(!sim.drones.channels.on(0x4b), "ch 0x4B stays off while the route flies");
    assert_eq!(sim.drones.copters[ci].state, 1, "still case 1 after the window");
}

#[test]
fn the_flight_body_rides_the_copter_matrix() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load 07000008");
    let sim = scene.sim.as_mut().unwrap();
    let ci = sim.drones.copters.iter().position(|c| c.gate == 5).expect("the ch-5-gated copter");
    let f = sim.drones.frame;
    sim.drones.channels.set(5, 1, f);
    for _ in 0..40 { sim.step_bond(Default::default(), false, false, false); }
    // SP_SetPosRot(*(rec+0x2c), obj+0x90) [M 0x17d628:115]: the SP player and its playing
    // script's base carry the copter matrix while the flight flies. (The keyed streams recompose
    // against that base each tick — nis.rs interp `mul(&m, &self.matrix)`; a KEYLESS stream's
    // entity pose froze at its creation tick, the documented posing gap in the report — the base
    // fields are what this pin covers.)
    let c0 = sim.drones.copters[ci].pos();
    for _ in 0..120 { sim.step_bond(Default::default(), false, false, false); }
    let c1 = sim.drones.copters[ci].pos();
    let cd = (c1[0] - c0[0]).powi(2) + (c1[1] - c0[1]).powi(2) + (c1[2] - c0[2]).powi(2);
    assert!(cd > 0.25, "the copter flew between the samples (d2 = {cd})");
    let sp = sim.drones.scripts.players.iter().find(|p| p.placement == sim.drones.copters[ci].placement)
        .expect("the copter's SP player");
    let base = sp.info[sp.cur].as_ref().map(|s| s.matrix).unwrap_or(sp.matrix);
    let bp = [base[3][0], base[3][1], base[3][2]];
    let d = (bp[0] - c1[0]).powi(2) + (bp[1] - c1[1]).powi(2) + (bp[2] - c1[2]).powi(2);
    assert!(d < 1.0, "the SP base rides the copter matrix ({bp:?} vs {c1:?}) [M 0x17d628:115]");
}

/// The 07000005 patrols carry +0x74 = 100 (placement param 8 [D]). At the end of its route a
/// +0x74 == 100 copter sets obj+0xFE bit 0 [M Copter_Move 0x17d628:63-65] — the object loop's
/// delete flag [M 0x1336f8:77-87 -> control_delete_object] — so it flies its route ONCE and is
/// removed with its SP (`Copter_Delete` 0x17e568). No patrol writes its arrival channel.
#[test]
fn the_07000005_patrols_fly_their_route_once_and_are_deleted() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    assert!(sim.drones.copters.iter().any(|c| c.arrival == 100), "the Mayhew patrols' +0x74 = 100 [D placement param 8]");
    let flying: Vec<usize> = sim.drones.copters.iter().filter(|c| c.arrival == 100 && c.gate == 0).map(|c| c.placement).collect();
    assert!(!flying.is_empty(), "an ungated patrol flies from the load [D gate 0]");
    let outs: Vec<u16> = sim.drones.copters.iter().map(|c| c.out).collect();
    let mut gone_at = None;
    for t in 0..40_000u32 {
        sim.step_bond(Default::default(), false, false, false);
        for &o in &outs { assert!(o == 0 || !sim.drones.channels.on(o as u32), "no patrol fired its arrival channel"); }
        if !sim.drones.copters.iter().any(|c| flying.contains(&c.placement)) { gone_at = Some(t); break; }
    }
    assert!(gone_at.is_some(), "the ungated patrol is deleted at its route end [M 0x17d628:63-65, 0x1336f8:77-87]");
    // Sister: before the route end it was flying (not deleted at once).
    assert!(gone_at.unwrap() > 60, "it flew its route first (deleted at tick {gone_at:?})");
}

/// The flight rate on every copter level: each tick adds the node's +0xc step to t
/// [M Copter_Move 0x17d628:53], and that step is `count / scale` — `REC_FRAME_RATE` and
/// `FRAME_RATE` come from one `GS_SetRefreshRate` call and cancel [M 0x17d2b8:9-16; L
/// ee_start_05.bin: 0.1905 = 381/2000 on #241]. Prints the measured speed (units/s at 60 ticks/s,
/// heading deg/s) for each copter over 300 flown ticks.
#[test]
fn every_copter_route_advances_count_over_scale_per_tick() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    for (name, _explode) in LEVELS {
        let mut scene = nf_view::load(&iso, name, false).unwrap_or_else(|e| panic!("load {name}: {e}"));
        let sim = scene.sim.as_mut().unwrap();
        let paths = sim.drones.copter_paths.clone();
        for (ci, c) in sim.drones.copters.clone().iter().enumerate() {
            let pi = c.path.unwrap();
            if c.gate != 0 { let f = sim.drones.frame; sim.drones.channels.set(c.gate as u32, 1, f); }
            // Fly until case 1 has run one Copter_Move, then measure 300 ticks.
            for _ in 0..5 { sim.step_bond(Default::default(), false, false, false); }
            let (t0, p0, d0) = { let c = &sim.drones.copters[ci]; (c.t, c.pos(), { let m = c.matrix; [m[2][0], m[2][1], m[2][2]] }) };
            let mut dist = 0.0f32; let mut turn = 0.0f32; let mut prev = p0; let mut pd = d0;
            let mut n = 0u32;
            for _ in 0..300 {
                sim.step_bond(Default::default(), false, false, false);
                let c = &sim.drones.copters[ci];
                if c.path != Some(pi) || c.state != 1 { break; } // a transition or a state change ends the window
                let p = c.pos(); let d = { let m = c.matrix; [m[2][0], m[2][1], m[2][2]] };
                dist += ((p[0]-prev[0]).powi(2) + (p[1]-prev[1]).powi(2) + (p[2]-prev[2]).powi(2)).sqrt();
                let (a0, a1) = (pd[0].atan2(pd[2]), d[0].atan2(d[2]));
                let mut da = a1 - a0; while da > std::f32::consts::PI { da -= std::f32::consts::TAU; } while da < -std::f32::consts::PI { da += std::f32::consts::TAU; }
                turn += da.abs(); prev = p; pd = d; n += 1;
            }
            let t1 = sim.drones.copters[ci].t;
            let step = paths[pi].step;
            eprintln!("{name} copter {ci} (#{}): count {} step {step:.5} ticks {n} dt {:.3} speed {:.2} u/s turn {:.1} deg/s",
                c.placement, paths[pi].count, t1 - t0, dist / n.max(1) as f32 * 60.0, turn.to_degrees() / n.max(1) as f32 * 60.0);
            if n > 0 {
                assert!(((t1 - t0) - step * n as f32).abs() < 1e-2 * n as f32 * step + 1e-3,
                    "{name} copter {ci}: t advances one step per tick [M 0x17d628:53]");
            }
        }
    }
}
