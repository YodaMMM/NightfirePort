//! level-13-guards-start-alerted-8: the 07000014 delta-exit leg-6 wedge probe. Teleports Bond to the
//! stuck spot the -7 report pinned ((23.31,1.14,-24.64), cel 16, segment_to_target Some(0.1227)),
//! dumps the world triangles / doors / the ray to the target, then replays the walk tick-by-tick with
//! a per-second position trace. Assertion-free, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker_b2;

use nf_game::player::Input;
use nf_game::world::V3;
use walker_b2::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_the_delta_leg6_wedge() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).expect("load 07000014"));
    w.wait_enabled(3000, "the spawn");
    // Teleport to the wedge spot (the -7 dump's exact position).
    w.scene.sim.as_mut().unwrap().bond.pos = [23.31, 1.14, -24.64];
    for _ in 0..30 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[leg6] settle pos ({:.3},{:.3},{:.3}) grounded {} cel {:?} floor_below {:?}",
            p[0], p[1], p[2], sim.bond.grounded, sim.world.find_cel(p), sim.world.floor_below(p, 6.0));
        // The world triangles in the approach box.
        let tris = sim.world.tris_near([22.3, 0.0, -25.8], [24.6, 2.6, -23.8]);
        eprintln!("[leg6] tris_near count {}", tris.len());
        for t in tris.iter().filter(|t| t.iter().any(|v| v[0] > 23.4 && v[0] < 24.3 && v[2] > -25.6 && v[2] < -24.0)) {
            eprintln!("  tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
                t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
        }
        // The ray to the target and the doors.
        let hit = sim.world.segment_hit_scoped(Some(16), [23.31, 1.14, -24.64], [27.79, 1.28, -25.32]);
        eprintln!("[leg6] segment to target: {:?}", hit);
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            if dxz2(d.pos, [23.31, -24.64]) < 6.0 {
                eprintln!("[leg6] door #{i} placement {} kind {} state {:#x} pos01 {:.2} pos {:?} ch_open {}",
                    d.placement, d.kind, d.state, d.pos01, d.pos, d.ch_open);
            }
        }
    }
    // Now replay the walk with a position trace.
    let mut ticks = 0u32;
    while ticks < 2400 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let d = dxz2(p, [27.79, -25.32]);
        if d < 0.9 { eprintln!("[leg6] ARRIVED at +{ticks} pos ({p:?})"); return; }
        let desired = (27.79f32 - p[0]).atan2(-25.32 - p[2]);
        let err = walker_b2::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: if d > 2.2 { 1.0 } else { 0.45 }, ..Default::default() };
        w.tick(input, false);
        ticks += 1;
        if ticks % 60 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            eprintln!("[leg6 t{ticks}] pos ({:.2},{:.2},{:.2}) grounded {} cel {:?} yaw {:.2}",
                p[0], p[1], p[2], sim.bond.grounded, sim.world.find_cel(p), sim.bond.yaw);
        }
    }
    eprintln!("[leg6] no arrival in 2400 ticks — the wedge reproduces from the teleport");
}

fn dxz2(p: V3, q: [f32; 2]) -> f32 { ((p[0] - q[0]).powi(2) + (p[2] - q[1]).powi(2)).sqrt() }

#[test]
fn probe_the_delta_lift_boxes() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).expect("load 07000014"));
    w.wait_enabled(3000, "the spawn");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[lift] spawn-state dynamic boxes near the wedge:");
        for (id, lo, hi) in sim.world.dynamic_boxes() {
            if lo[0] > 20.0 && lo[0] < 30.0 && lo[2] > -28.0 && lo[2] < -20.0 {
                eprintln!("  box id {id:#x} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
            }
        }
        eprintln!("[lift] useobj/placement scan: doors near (24,-22.8) dumped above; script players:");
        for p in sim.drones.scripts.players.iter() { let _ = (&p.scripts, &p.state);
            eprintln!("  sp scripts {:?} state {}", p.scripts, p.state);
        }
    }
    // Teleport Bond onto the wedge spot and re-dump (the settle difference).
    w.scene.sim.as_mut().unwrap().bond.pos = [23.31, 1.14, -24.64];
    for _ in 0..60 { w.tick(Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[lift] after settle: pos {:?} grounded {} floor_below {:?}", sim.bond.pos, sim.bond.grounded, sim.world.floor_below(sim.bond.pos, 6.0));
    for (id, lo, hi) in sim.world.dynamic_boxes() {
        if lo[0] > 20.0 && lo[0] < 30.0 && lo[2] > -28.0 && lo[2] < -20.0 {
            eprintln!("  box id {id:#x} lo ({:.2},{:.2},{:.2}) hi ({:.2},{:.2},{:.2})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        }
    }
}

#[test]
fn probe_the_delta_nav_route() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).expect("load 07000014"));
    w.wait_enabled(3000, "the spawn");
    // Stand at leg 5's end (the wake touch) and ask the nav graph for the exit.
    w.scene.sim.as_mut().unwrap().bond.pos = [20.3, 0.9, -23.5];
    for _ in 0..30 { w.tick(Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let pts = walker_b2::nav_route(sim, [20.3, 0.9, -23.5], [27.79, 1.28, -25.32]);
        match pts {
            Some(p) => { eprintln!("[navroute] {} nodes:", p.len()); for q in &p { eprintln!("  ({:.2},{:.2},{:.2})", q[0], q[1], q[2]); } }
            None => eprintln!("[navroute] NO ROUTE"),
        }
    }
    // Walk it with the lenient b2 walker (no panic on a wedge).
    w.lenient = true;
    let r = w.follow_nav_route([27.79, 1.28, -25.32], "nav to the exit #944");
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[navroute] follow_nav_route returned {r} bond ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
}

#[test]
fn probe_the_delta_real_approach() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000014.bin", false).expect("load 07000014"));
    w.wait_enabled(3000, "the spawn");
    // The real leg-6 start: the resupply spot the kill ended at (the run's own trace).
    w.scene.sim.as_mut().unwrap().bond.pos = std::env::var("LEG6_START").ok().map(|s| { let v: Vec<f32> = s.split(',').map(|x| x.parse().unwrap()).collect(); [v[0], v[1], v[2]] }).unwrap_or([21.39, 1.04, -22.57]);
    for _ in 0..30 { w.tick(Input::default(), false); }
    w.lenient = true;
    // The player's real approach: stay on the y≈1.0 floor, E the double slides, through, then the exit.
    w.walk_to(Wp::new(22.9, 1.0, -22.8).r(1.2), "the doorway approach");
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[approach] after approach leg: ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    w.walk_to(Wp { at: [24.0, 1.2, -22.8], act: Act::Use { use_ticks: 240 }, arrive: 1.1 }, "E the delta slides");
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[approach] after E leg: ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    w.walk_to(Wp::new(27.79, 1.28, -25.32).r(1.0), "to the Delta exit #944");
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[approach] after exit leg: ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[approach] doors: {}", sim.drones.doors.list.iter().enumerate()
        .filter(|(_, d)| dxz2(d.pos, [24.0, -22.8]) < 2.0)
        .map(|(i, d)| format!("#{i} st {:#x} p01 {:.2}", d.state, d.pos01)).collect::<Vec<_>>().join(" | "));
}
