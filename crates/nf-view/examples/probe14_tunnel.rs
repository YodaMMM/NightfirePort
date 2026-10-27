// level-14-full-walk scratch 5: the tunnel under the east vent lane — its full length,
// its floor levels (any steps up to the 2.40 lane?), its south exit, and the same for the
// west mirror. Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;

    // A. inside-tunnel down-grid from y 1.5 (below the lane plate): x 44.4..46.6 step .2,
    //    z 48.5 -> 41.5 step .25 — any intermediate floors = steps.
    println!("== A. tunnel down from y 1.5 -> -4: x 44.4..46.6 step .2, z 48.5->41.5 ==");
    let xs: Vec<f32> = (0..12).map(|i| 44.4 + i as f32 * 0.2).collect();
    print!("     z\\x");
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    let mut z = 48.5f32;
    while z >= 41.49 {
        print!("z {:>5}", z);
        for &x in &xs {
            match world.segment([x, 1.5, z], [x, -4.0, z]) {
                Some(t) => print!(" {:>5.2}", 1.5 - t * 5.5),
                None => print!("  open"),
            }
        }
        println!();
        z -= 0.25;
    }

    // B. the same for the WEST lane tunnel: x 40.0..41.7 step .2, z 48.5 -> 41.5.
    println!("== B. west tunnel down from y 1.5 -> -4: x 40.0..41.6 step .2 ==");
    let xs2: Vec<f32> = (0..9).map(|i| 40.0 + i as f32 * 0.2).collect();
    print!("     z\\x");
    for &x in &xs2 { print!(" {:>5}", x); }
    println!();
    let mut z = 48.5f32;
    while z >= 41.49 {
        print!("z {:>5}", z);
        for &x in &xs2 {
            match world.segment([x, 1.5, z], [x, -4.0, z]) {
                Some(t) => print!(" {:>5.2}", 1.5 - t * 5.5),
                None => print!("  open"),
            }
        }
        println!();
        z -= 0.25;
    }

    // C. tunnel ceilings: up from y -2.0 along x 45.2 and 40.8, z 48.5 -> 41.5 — the headroom.
    println!("== C. tunnel ceiling (up from y -2.0): x 45.2 east / x 40.8 west ==");
    let mut z = 48.5f32;
    while z >= 41.49 {
        let ce = world.segment([45.2, -2.0, z], [45.2, 3.0, z]).map(|t| -2.0 + t * 5.0);
        let cw = world.segment([40.8, -2.0, z], [40.8, 3.0, z]).map(|t| -2.0 + t * 5.0);
        println!("z {z:5.2}: east ceil {:?}  west ceil {:?}", ce.map(|v| format!("{v:.2}")), cw.map(|v| format!("{v:.2}")));
        z -= 0.25;
    }
}
