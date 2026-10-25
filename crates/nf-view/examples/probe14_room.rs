// quick scratch: down-seg grid from y 3.5
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let xs = [38.0, 38.5, 39.0, 39.5, 40.0, 40.5, 41.0, 41.5, 42.0, 42.5, 43.0, 43.5, 44.0, 44.5, 45.0, 45.5, 46.0, 46.5, 47.0];
    print!("     z\\x"); for x in xs { print!(" {:>6}", x); } println!();
    for zi in 0..17 {
        let z = 59.0f32 + zi as f32 * 0.5;
        print!("z {:>5}", z);
        for &x in xs.iter() {
            let p = [x, 3.5, z];
            let t = sim.world.segment(p, [p[0], p[1] - 5.0, p[2]]);
            match t { Some(t) => print!(" {:>6.2}", p[1] - t * 5.0), None => print!("   open") }
        }
        println!();
    }
}
