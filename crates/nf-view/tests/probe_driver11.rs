//! drone-bump-collision-11 one-off: where does the truck driver stall on his mission route?
#[path = "common/walker.rs"]
mod walker;
#[test]
#[ignore = "diagnostic; --ignored --nocapture"]
fn probe_driver_route_stall() {
    let iso = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load(&iso, "07000006.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    let di = sim.drones.list.iter().position(|dr| dr.skin == 0x0500_0035).unwrap();
    for _ in 0..920 { sim.step_bond(Default::default(), false, false, false); }
    eprintln!("after script: state {:#x} pos {:?}", sim.drones.list[di].state, sim.drones.list[di].pos);
    let mut last_route = String::new();
    for t in 0..6000u32 {
        sim.step_bond(Default::default(), false, false, false);
        let dr = &sim.drones.list[di];
        let r = format!("{:?} idx{}", dr.route.as_ref().map(|r| r.path), dr.route.as_ref().map(|r| r.index).unwrap_or(999));
        if r != last_route || t % 500 == 0 {
            eprintln!("t{t}: st {:#x} pos ({:.2},{:.2},{:.2}) route {r} blocked {:.2}", dr.state, dr.pos[0], dr.pos[1], dr.pos[2], dr.blocked_reach);
            last_route = r;
        }
        if dr.state == 0x34 { eprintln!("ARRIVED t{t}"); break; }
    }
    // The four clamp probes' blocking walls (the boundary_clamp shape).
    let sim = scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[di];
    let feet = [dr.pos[0], dr.pos[1] - 0.6, dr.pos[2]];
    let (s_, c_) = (nf_game::trig::sinf(dr.yaw), nf_game::trig::cosf(dr.yaw));
    let seg_hit = |a: [f32; 3], b: [f32; 3], w: &nf_game::nav::Wall| -> bool {
        // 2D segment-segment intersection (x,z).
        let d1 = [b[0] - a[0], b[2] - a[2]];
        let d2 = [w.b[0] - w.a[0], w.b[2] - w.a[2]];
        let den = d1[0] * d2[1] - d1[1] * d2[0];
        if den.abs() < 1e-9 { return false; }
        let t = ((w.a[0] - a[0]) * d2[1] - (w.a[2] - a[2]) * d2[0]) / den;
        let u = ((w.a[0] - a[0]) * d1[1] - (w.a[2] - a[2]) * d1[0]) / den;
        t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0
    };
    for (nm, ax, az) in [("fwd", 0.0f32, 0.5f32), ("back", 0.0, -0.5), ("right", 0.5, 0.0), ("left", -0.5, 0.0)] {
        let p = [feet[0] + ax * c_ + az * s_, feet[1], feet[2] - ax * s_ + az * c_];
        let ok = sim.nav.node_scan_ok(feet, p);
        let mut hit = String::from("none");
        for w in &sim.nav.walls {
            if seg_hit(feet, p, w) {
                let da = ((w.a[0] - feet[0]).powi(2) + (w.a[2] - feet[2]).powi(2)).sqrt();
                let db = ((w.b[0] - feet[0]).powi(2) + (w.b[2] - feet[2]).powi(2)).sqrt();
                hit = format!("wall a({:.2},{:.2}) b({:.2},{:.2}) da {:.2} db {:.2} flags {:x}", w.a[0], w.a[2], w.b[0], w.b[2], da, db, w.flags);
                break;
            }
        }
        eprintln!("probe {nm} ({:.2},{:.2}) ok={ok} blocking: {hit}", p[0], p[2]);
    }
}
