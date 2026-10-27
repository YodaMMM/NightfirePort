// level-14-full-walk scratch: the east shaft entry fine grid — how the 0.60 corridor/stairwell
// level connects UP to the east vent lane (2.40) through the shaft throat (x 44.75..45.75,
// z 49..50 [D probe14_south]). Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;

    // A. fine down grid from y 3.2 (below the 3.27 lip): x 43.8..47.4 step 0.2, z 47..52 step 0.25.
    println!("== A. down from y 3.2 -> -6: x 43.8..47.4 step .2, z 52->47 step .25 ==");
    let xs: Vec<f32> = (0..19).map(|i| 43.8 + i as f32 * 0.2).collect();
    print!("     z\\x");
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    for zi in 0..21 {
        let z = 52.0f32 - zi as f32 * 0.25;
        print!("z {:>5}", z);
        for &x in &xs {
            match world.segment([x, 3.2, z], [x, -6.0, z]) {
                Some(t) => print!(" {:>5.2}", 3.2 - t * 9.2),
                None => print!("  open"),
            }
        }
        println!();
    }

    // B. the same grid from y 1.4 (corridor band) -> -6.
    println!("== B. down from y 1.4 -> -6: same grid ==");
    for zi in 0..21 {
        let z = 52.0f32 - zi as f32 * 0.25;
        print!("z {:>5}", z);
        for &x in &xs {
            match world.segment([x, 1.4, z], [x, -6.0, z]) {
                Some(t) => print!(" {:>5.2}", 1.4 - t * 7.4),
                None => print!("  open"),
            }
        }
        println!();
    }

    // C. north-south rays at walker chest heights: x in {44.4,44.6,44.9,45.3,45.7,46.1},
    //    y in {0.8,1.2,1.6,2.0,2.3}, z from 52.5 south to 46.5 — first hit = the blocker.
    println!("== C. southward rays (z 52.5 -> 46.5): first-hit z at chest heights ==");
    for &x in &[44.4f32, 44.6, 44.9, 45.3, 45.7, 46.1] {
        for &y in &[0.8f32, 1.2, 1.6, 2.0, 2.3] {
            let hit = world.segment([x, y, 52.5], [x, y, 46.5]);
            match hit {
                Some(t) => println!("x {x:5.2} y {y:4.2}: blocked at z {:.2}", 52.5 - t * 6.0),
                None => println!("x {x:5.2} y {y:4.2}: CLEAR z 52.5..46.5"),
            }
        }
    }

    // D. the same southward rays on the WEST shaft (the old route's shape, x 39.6..42.2).
    println!("== D. west shaft southward rays (z 52.5 -> 46.5) ==");
    for &x in &[39.9f32, 40.1, 40.4, 40.8, 41.2, 41.6] {
        for &y in &[0.8f32, 1.2, 1.6, 2.0, 2.3] {
            let hit = world.segment([x, y, 52.5], [x, y, 46.5]);
            match hit {
                Some(t) => println!("x {x:5.2} y {y:4.2}: blocked at z {:.2}", 52.5 - t * 6.0),
                None => println!("x {x:5.2} y {y:4.2}: CLEAR z 52.5..46.5"),
            }
        }
    }

    // E. the mouth lane: the east vent lane south end + the lid band (z 42.5..41) fine,
    //    x 44.4..46.6 step .2 — floor and the south wall face.
    println!("== E. the east mouth: down from y 3.2 -> -1, x 44.4..46.6 step .2, z 43.5..40.75 step .25 ==");
    let xs2: Vec<f32> = (0..12).map(|i| 44.4 + i as f32 * 0.2).collect();
    print!("     z\\x");
    for &x in &xs2 { print!(" {:>5}", x); }
    println!();
    for zi in 0..12 {
        let z = 43.5f32 - zi as f32 * 0.25;
        print!("z {:>5}", z);
        for &x in &xs2 {
            match world.segment([x, 3.2, z], [x, -1.0, z]) {
                Some(t) => print!(" {:>5.2}", 3.2 - t * 4.2),
                None => print!("  open"),
            }
        }
        println!();
    }
}
