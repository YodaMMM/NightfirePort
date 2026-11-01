//! Probe (assertion-free, ignored): the float against the shaft's south end-wall (z −20.40).
#[path = "common/walker.rs"]
mod walker;
use nf_game::player::Input;
use walker::Walker;

fn thrust_to(sim: &nf_view::scene::Sim, at: [f32; 3], g: f32) -> Input {
    let p = sim.bond.pos;
    let d = [at[0] - p[0], at[1] - p[1], at[2] - p[2]];
    let m = sim.bond.zero_g_rot;
    let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    Input { strafe: (dot(m[0]) * g).clamp(-1.0, 1.0), lift: (dot(m[1]) * g).clamp(-1.0, 1.0), forward: (dot(m[2]) * g).clamp(-1.0, 1.0), ..Default::default() }
}

fn fly(w: &mut Walker, at: [f32; 3], n: u32, label: &str) {
    let p0 = w.scene.sim.as_ref().unwrap().bond.pos;
    let mut mins = [f32::MAX; 3]; let mut maxs = [f32::MIN; 3];
    for t in 0..n {
        let input = if std::env::var_os("PROBE_YAW").is_some() && label != "leg0" {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (at[0] - p[0]).atan2(at[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let lift = ((at[1] - p[1]) * 2.0).clamp(-1.0, 1.0);
            Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() < 0.35 { 1.0 } else { 0.15 }, lift, ..Default::default() }
        } else { thrust_to(w.scene.sim.as_ref().unwrap(), at, 0.6) };
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        for k in 0..3 { mins[k] = mins[k].min(p[k]); maxs[k] = maxs[k].max(p[k]); }
        if t % 100 == 0 { eprintln!("  [{label}] t{t} pos ({:.3},{:.3},{:.3}) local {:?} lift {:.3}", p[0], p[1], p[2], w.scene.sim.as_ref().unwrap().bond.local, w.scene.sim.as_ref().unwrap().bond.zero_g_lift); }
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[{label}] -> ({:.2},{:.2},{:.2}) from ({:.2},{:.2},{:.2}) to ({:.3},{:.3},{:.3}) min {mins:?} max {maxs:?}", at[0], at[1], at[2], p0[0], p0[1], p0[2], p[0], p[1], p[2]);
}

#[test]
#[ignore]
fn probe_endwall() {
    let iso = std::env::var("NF_ISO").unwrap();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "arrival");
    w.wait(120);
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("spawn {:?} substate {}", sim.bond.pos, sim.bond.substate);
    let route: Vec<[f32; 3]> = std::env::var("PROBE_ROUTE").ok().map(|s| s.split(';').map(|t| { let v: Vec<f32> = t.split(',').map(|x| x.trim().parse().unwrap()).collect(); [v[0], v[1], v[2]] }).collect())
        .unwrap_or(vec![[20.0, 0.0, 1.0], [8.0, 0.0, -1.0], [7.0, 0.0, -9.6], [7.0, -3.6, -11.5], [7.5, -3.4, -15.5], [8.0, -1.2, -17.0], [8.0, -1.2, -22.0], [14.0, -1.2, -19.9], [2.0, -1.2, -19.9], [8.0, 1.0, -19.9], [8.0, -4.0, -19.9], [8.0, -1.2, -15.0]]);
    let n: u32 = std::env::var("PROBE_N").ok().and_then(|s| s.parse().ok()).unwrap_or(700);
    for (i, at) in route.iter().enumerate() { fly(&mut w, *at, n, &format!("leg{i}")); }
}
