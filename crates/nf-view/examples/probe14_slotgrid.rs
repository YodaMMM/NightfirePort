//! Scratch probe (level-14-lid-and-climb): a static-floor grid over the slot area — where the
//! room floor is solid at y 2.4 vs open down to the trench (-0.6). Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);

    print!("     z\\x");
    let xs = [38.0, 38.5, 39.0, 39.5, 40.0, 40.5, 41.0, 41.5, 42.0, 42.5, 43.0, 43.5, 44.0, 44.5, 45.0, 45.5, 46.0, 46.5, 47.0];
    for x in xs {
        print!(" {:>6}", x);
    }
    println!();
    for z in [59.0, 60.0, 61.0, 62.0, 62.5, 63.0, 63.5, 63.9, 64.5, 65.0, 65.5, 66.0, 67.0] {
        print!("z {:>5}", z);
        for &x in xs.iter() {
            let p = [x, 2.4, z];
            let t = sim.world.segment(p, [p[0], p[1] - 5.0, p[2]]);
            let hit_y = t.map(|t| p[1] - t * 5.0);
            match hit_y {
                Some(y) => print!(" {:>6.2}", y),
                None => print!("   open"),
            }
        }
        println!();
    }
    println!("bond start: {:?}", sim.bond.pos);
    println!("(values = the static floor y under the spot; 'open' = nothing within 5 m down)");
}
