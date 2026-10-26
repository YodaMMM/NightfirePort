// level-14-east-walkway scratch: the south half — the corridor (x 44..50, z 42..62), the shaft(s)
// up to the south vent room (y 2.4), and the east mouth lane (x 45.3, z 42..47.5).
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;

    println!("== (a) down-seg from y 3.5: x 39.5..46.5 step .25, z 41.5..51 step .5 ==");
    let xs: Vec<f32> = (0..29).map(|i| 39.5 + i as f32 * 0.25).collect();
    print!("     z\\x");
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    for zi in 0..20 {
        let z = 41.5f32 + zi as f32 * 0.5;
        print!("z {:>5}", z);
        for &x in &xs {
            let t = world.segment([x, 3.5, z], [x, -3.0, z]);
            match t { Some(t) => print!(" {:>5.2}", 3.5 - t * 6.5), None => print!("  open") }
        }
        println!();
    }

    println!("== (b) down-seg from y 1.0 (corridor band, below the vent floor): same grid ==");
    for zi in 0..20 {
        let z = 41.5f32 + zi as f32 * 0.5;
        print!("z {:>5}", z);
        for &x in &xs {
            let t = world.segment([x, 1.0, z], [x, -3.0, z]);
            match t { Some(t) => print!(" {:>5.2}", 1.0 - t * 4.0), None => print!("  open") }
        }
        println!();
    }

    println!("== (c) the east stairwell fine: x 44.4..46.4 step .2, z 50.5..59 step .25, from y 4 ==");
    let xs2: Vec<f32> = (0..11).map(|i| 44.4 + i as f32 * 0.2).collect();
    print!("     z\\x");
    for &x in &xs2 { print!(" {:>5}", x); }
    println!();
    for zi in 0..35 {
        let z = 59.0f32 - zi as f32 * 0.25;
        print!("z {:>5}", z);
        for &x in &xs2 {
            let t = world.segment([x, 4.0, z], [x, -3.0, z]);
            match t { Some(t) => print!(" {:>5.2}", 4.0 - t * 7.0), None => print!("  open") }
        }
        println!();
    }
}
