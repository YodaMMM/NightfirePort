// quick scratch: the east stairwell descent — down-segs from y 3.5, fine grid
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let xs = [44.5, 45.0, 45.25, 45.5, 46.0, 46.5, 47.0, 47.5, 48.0, 49.0, 50.0];
    print!("     z\\x"); for x in xs { print!(" {:>6}", x); } println!();
    for zi in 0..21 {
        let z = 51.0f32 + zi as f32 * 0.5;
        print!("z {:>5}", z);
        for &x in xs.iter() {
            let p = [x, 3.5, z];
            let t = sim.world.segment(p, [p[0], p[1] - 5.0, p[2]]);
            match t { Some(t) => print!(" {:>6.2}", p[1] - t * 5.0), None => print!("   open") }
        }
        println!();
    }
}
