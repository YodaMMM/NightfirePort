// level-14-full-walk scratch 2: INSIDE the east well (x 44.75..45.75, z 49..50.25) — its bottom,
// its four walls' openings at every depth, and where the -2.40 pit lane (x 42..44.5) leads.
// Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;

    // A. the well's bottom: down from y 1.0 at the well centre to y -20.
    for &(x, z) in &[(45.0f32, 49.5f32), (45.25, 49.0), (45.5, 49.75), (44.9, 50.1)] {
        match world.segment([x, 1.0, z], [x, -20.0, z]) {
            Some(t) => println!("A. down ({x}, {z}): floor at y {:.2}", 1.0 - t * 21.0),
            None => println!("A. down ({x}, {z}): open to -20"),
        }
    }

    // B. rays OUT of the well interior (45.25, z 49.6) in the 4 directions at depths
    //    y -5..2.4 step 0.35: first hit distance. Length 4.0 each way.
    let (cx, cz) = (45.25f32, 49.6f32);
    let dirs: [(&str, [f32; 3]); 4] = [
        ("west ", [-1.0, 0.0, 0.0]),
        ("east ", [1.0, 0.0, 0.0]),
        ("north", [0.0, 0.0, 1.0]),
        ("south", [0.0, 0.0, -1.0]),
    ];
    for yi in 0..21 {
        let y = 2.4f32 - yi as f32 * 0.35;
        let mut row = format!("B. y {y:6.2}:");
        for (name, d) in &dirs {
            let hit = world.segment([cx, y, cz], [cx + d[0] * 4.0, y + d[1] * 4.0, cz + d[2] * 4.0]);
            match hit {
                Some(t) => row.push_str(&format!(" {name} {:.2}  ", t * 4.0)),
                None => row.push_str(&format!(" {name} open  ")),
            }
        }
        println!("{row}");
    }

    // C. the -2.40 pit lane (x 42..44.5, z 44.5..48): where does it go? Down-grid from y 0.5
    //    x 41.6..44.7 step .2, z 49..43 step .5.
    println!("== C. pit lane down from y 0.5 -> -6: x 41.6..44.6 step .2, z 49->43 step .5 ==");
    let xs: Vec<f32> = (0..16).map(|i| 41.6 + i as f32 * 0.2).collect();
    print!("     z\\x");
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    for zi in 0..13 {
        let z = 49.0f32 - zi as f32 * 0.5;
        print!("z {:>5}", z);
        for &x in &xs {
            match world.segment([x, 0.5, z], [x, -6.0, z]) {
                Some(t) => print!(" {:>5.2}", 0.5 - t * 6.5),
                None => print!("  open"),
            }
        }
        println!();
    }

    // D. the under-strip corridor (z 42..44, y -0.6): rays UP to find any opening to the
    //    mezzanine/vent above; and rays at y 0..2 northward (z 44 -> 48.5) at x 42.5..44.5.
    println!("== D. under-strip: up-rays from y -0.4 to y 3.2 (first hit) at x 42..44.6, z 42.5..44 ==");
    for &x in &[42.0f32, 42.5, 43.0, 43.5, 44.0, 44.4] {
        for &z in &[42.5f32, 43.0, 43.5, 44.0] {
            let hit = world.segment([x, -0.4, z], [x, 3.2, z]);
            match hit {
                Some(t) => println!("D. up ({x}, {z}): ceiling at y {:.2}", -0.4 + t * 3.6),
                None => println!("D. up ({x}, {z}): open to 3.2"),
            }
        }
    }
    println!("== D2. northward rays (z 44.4 -> 49) at y 0.0/0.8/1.6: x 42.5..44.6 ==");
    for &x in &[42.5f32, 43.0, 43.5, 44.0, 44.3, 44.6] {
        for &y in &[0.0f32, 0.8, 1.6] {
            let hit = world.segment([x, y, 44.4], [x, y, 49.0]);
            match hit {
                Some(t) => println!("D2. north ({x}, y {y}): blocked at z {:.2}", 44.4 + t * 4.6),
                None => println!("D2. north ({x}, y {y}): CLEAR z 44.4..49"),
            }
        }
    }
}
