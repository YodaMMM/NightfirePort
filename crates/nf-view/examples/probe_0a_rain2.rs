//! Scratch probe 2 (level-0a-rain-indoors): put Bond at the reported office spot, tick, and dump
//! the env state there: active boxes, drop positions, the room cels containing the spot and their
//! overlap with the rain boxes. Run: NF_ISO=... cargo run --release -p nf-view --example probe_0a_rain2
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load(&path, "0700000a.bin", false).expect("load scene");
    let sim = scene.sim.as_mut().expect("sim");
    // The reported office-lounge state.
    sim.bond.pos = [-37.50, 1.11, -16.95];
    sim.bond.yaw = 2.827;
    sim.bond.pitch = -0.092;
    let input = nf_game::player::Input::default();
    for t in 1..=300u32 {
        sim.step_bond(input, false, false, false);
        if t % 100 == 0 {
            if let Some(env) = sim.drones.env.as_ref() {
                let alive = env.alive.iter().filter(|&&a| a).count();
                let near = env.drops.iter().zip(&env.alive).filter(|(p, &a)| a &&
                    (p[0] + 37.5).powi(2) + (p[1] - 1.11).powi(2) + (p[2] + 16.95).powi(2) < 25.0).count();
                println!("t{t}: bond {:?} centre {:?} alive {alive} near-office {near}",
                    sim.bond.pos, env.centre);
            }
        }
    }
    let env = sim.drones.env.as_ref().expect("env");
    let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
    for (p, &a) in env.drops.iter().zip(&env.alive) {
        if !a { continue; }
        for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); }
    }
    println!("drop bbox lo {:?} hi {:?}", lo, hi);
    println!("active boxes (cube-clipped, recomputed here):");
    {
        let c = env.centre;
        for b in &env.boxes {
            let lo = [b.min[0].max(c[0] - 15.0), b.min[1].max(c[1] - 15.0), b.min[2].max(c[2] - 15.0)];
            let hi = [b.max[0].min(c[0] + 15.0), b.max[1].min(c[1] + 15.0), b.max[2].min(c[2] + 15.0)];
            if hi[0] > lo[0] && hi[2] > lo[2] {
                println!("  ACTIVE lo {:?} hi {:?}", lo, hi);
            }
        }
    }
    // The room cels containing the spot (their bounds), and every cel overlapping box5.
    println!("cels containing the spot:");
    for (i, c) in scene.cels.iter().enumerate() {
        let inside = c.min[0] <= -37.5 && -37.5 <= c.max[0] && c.min[1] <= 1.11 && 1.11 <= c.max[1] && c.min[2] <= -16.95 && -16.95 <= c.max[2];
        if inside { println!("  cel {i} lo {:?} hi {:?}", c.min, c.max); }
    }
    println!("cels overlapping rainbox5 (x -81.7..-42.3, z -57.9..-20.5):");
    for (i, c) in scene.cels.iter().enumerate() {
        let ox = c.min[0] <= -42.265 && -81.669 <= c.max[0];
        let oz = c.min[2] <= -20.535 && -57.888 <= c.max[2];
        let oy = c.min[1] <= 18.53 && -96.91 <= c.max[1];
        if ox && oz && oy { println!("  cel {i} lo {:?} hi {:?}", c.min, c.max); }
    }
}
