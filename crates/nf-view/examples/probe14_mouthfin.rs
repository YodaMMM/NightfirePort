// level-14-full-walk scratch 6: the mouth approach fine structure — is there a stair from the
// under-mouth corridor (-0.60) up to the lane/mouth slabs (2.40)? And the exact ramp/lip shape
// at the well's south edge. Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;

    // A. fine: x 44.9..46.1 step .1, z 42.5..41.2 step .1, down from y 3.2 -> -2.5.
    println!("== A. east mouth fine: down y 3.2 -> -2.5 ==");
    let xs: Vec<f32> = (0..13).map(|i| 44.9 + i as f32 * 0.1).collect();
    print!("     z\\x");
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    let mut z = 42.5f32;
    while z >= 41.19 {
        print!("z {:>5}", z);
        for &x in &xs {
            match world.segment([x, 3.2, z], [x, -2.5, z]) {
                Some(t) => print!(" {:>5.2}", 3.2 - t * 5.7),
                None => print!("  open"),
            }
        }
        println!();
        z -= 0.1;
    }

    // B. the ramp/lip at the well's south edge: x 44.5..46.1 step .1, z 48.8..47.9 step .1,
    //    down from y 3.2 -> 0.0 (sees ramp top), and up from y -2.0 (sees underside).
    println!("== B. lip fine down y 3.2 -> 0 ==");
    let xs2: Vec<f32> = (0..17).map(|i| 44.5 + i as f32 * 0.1).collect();
    print!("     z\\x");
    for &x in &xs2 { print!(" {:>5}", x); }
    println!();
    let mut z = 48.8f32;
    while z >= 47.89 {
        print!("z {:>5}", z);
        for &x in &xs2 {
            match world.segment([x, 3.2, z], [x, 0.0, z]) {
                Some(t) => print!(" {:>5.2}", 3.2 - t * 3.2),
                None => print!("  open"),
            }
        }
        println!();
        z -= 0.1;
    }

    // C. what is north of the west mouth / west lane at 2.40 — the west lobe and the #301 hole:
    //    x 39.4..41.9 step .2, z 42.5..37 step .25, down from y 3.2 -> -1.
    println!("== C. west lobe/mezzanine: down y 3.2 -> -1 ==");
    let xs3: Vec<f32> = (0..13).map(|i| 39.4 + i as f32 * 0.2).collect();
    print!("     z\\x");
    for &x in &xs3 { print!(" {:>5}", x); }
    println!();
    let mut z = 42.5f32;
    while z >= 36.99 {
        print!("z {:>5}", z);
        for &x in &xs3 {
            match world.segment([x, 3.2, z], [x, -1.0, z]) {
                Some(t) => print!(" {:>5.2}", 3.2 - t * 4.2),
                None => print!("  open"),
            }
        }
        println!();
        z -= 0.25;
    }

    // D. the east lobe -> the strip -> the west lobe at mezzanine level: x 41.5..45.0 step .25,
    //    z 41.0 -> 37.0 step .25, down from y 3.2 -> -1.
    println!("== D. mezzanine crossing: down y 3.2 -> -1 ==");
    let xs4: Vec<f32> = (0..15).map(|i| 41.5 + i as f32 * 0.25).collect();
    print!("     z\\x");
    for &x in &xs4 { print!(" {:>5}", x); }
    println!();
    let mut z = 41.0f32;
    while z >= 36.99 {
        print!("z {:>5}", z);
        for &x in &xs4 {
            match world.segment([x, 3.2, z], [x, -1.0, z]) {
                Some(t) => print!(" {:>5.2}", 3.2 - t * 4.2),
                None => print!("  open"),
            }
        }
        println!();
        z -= 0.25;
    }
}
