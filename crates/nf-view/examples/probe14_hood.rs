// level-14-full-walk-2 scratch: the mouths' hood underside band over the lane strip —
// up-facing rays (a down-facing underside is invisible to down-rays) + down-rays for the
// lane floor, so the walkable head-clearance corridor for leg 1f shows. Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;

    // A. up-rays from y 2.2 (just above the 2.38 lane) to y 5.5: first hit = the underside.
    println!("== A. up-rays y 2.2 -> 5.5 (the underside): x 44.4..46.2 step .3, z 38.5..43.7 step .4 ==");
    print!("     z\\x");
    let xs: Vec<f32> = (0..7).map(|i| 44.4 + i as f32 * 0.3).collect();
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    for zi in 0..14 {
        let z = 43.7f32 - zi as f32 * 0.4;
        print!("z {:>5}", z);
        for &x in &xs {
            match world.segment([x, 2.2, z], [x, 5.5, z]) {
                Some(t) => print!(" {:>5.2}", 2.2 + t * 3.3),
                None => print!("  open"),
            }
        }
        println!();
    }
    // B. down-rays over the same grid for the floor (the lane top / the connector).
    println!("== B. down-rays y 5.5 -> -3.0 (the floor) ==");
    for zi in 0..14 {
        let z = 43.7f32 - zi as f32 * 0.4;
        print!("z {:>5}", z);
        for &x in &xs {
            match world.segment([x, 5.5, z], [x, -3.0, z]) {
                Some(t) => print!(" {:>5.2}", 5.5 - t * 8.5),
                None => print!("  open"),
            }
        }
        println!();
    }
    // C. the west connector band: up-rays + down-rays at x 40.5..44.4 (the leg-1f west walk).
    println!("== C. the connector: x 40.5..44.4 step .45, z 38.3..40.5 step .4 ==");
    for xi in 0..9 {
        let x = 40.5f32 + xi as f32 * 0.45;
        for zi in 0..6 {
            let z = 40.5f32 - zi as f32 * 0.4;
            let up = world.segment([x, 2.0, z], [x, 5.5, z]).map(|t| 2.0 + t * 3.5);
            let down = world.segment([x, 5.5, z], [x, -3.0, z]).map(|t| 5.5 - t * 8.5);
            println!("x {x:5.2} z {z:5.2}: floor {:?} ceiling-up {:?}", down.map(|v| (v * 100.0).round() / 100.0), up.map(|v| (v * 100.0).round() / 100.0));
        }
    }
}
