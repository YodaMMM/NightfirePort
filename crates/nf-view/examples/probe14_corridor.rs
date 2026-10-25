// level-14-east-walkway scratch: the stairwell foot -> corridor x 49 -> mouths' lane, roofs excluded
// (rays from y 2.0: floors at/below 0.60 caught; the stairwell's sloped roof >= 2.26 missed).
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;
    let xs: Vec<f32> = (0..31).map(|i| 43.0 + i as f32 * 0.25).collect();
    print!("     z\\x");
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    for zi in 0..29 {
        let z = 42.0f32 + zi as f32 * 0.5;
        print!("z {:>5}", z);
        for &x in &xs {
            let t = world.segment([x, 2.0, z], [x, -3.0, z]);
            match t { Some(t) => print!(" {:>5.2}", 2.0 - t * 5.0), None => print!("  open") }
        }
        println!();
    }
    println!("== body-height eastward rays at y 0.4, from x 45.3 -> 49.6 ==");
    for zi in 0..25 {
        let z = 51.0f32 + zi as f32 * 0.25;
        match world.segment([45.3, 0.4, z], [49.6, 0.4, z]) {
            Some(t) => println!("z {z:>5}: blocked at x {:.2}", 45.3 + t * 4.3),
            None => println!("z {z:>5}: clear to x 49.6"),
        }
    }
}
