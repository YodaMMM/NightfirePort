// level-14-east-walkway scratch: the west->east crossing across the vent-room slot.
// (a) fine down-seg grid z 63.4..65.4 / x 38..47.5;
// (b) horizontal rays at body heights y 2.8 / 3.3 from x 41.2 eastward at each z —
//     a hit = a wall (the down-grid cannot see walls);
// (c) the same westward from x 45.2.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);

    println!("== (a) down-seg floors, y 3.5 -> -2.5, step .25 x / .2 z ==");
    let xs: Vec<f32> = (0..38).map(|i| 38.0 + i as f32 * 0.25).collect();
    print!("     z\\x");
    for &x in &xs { print!(" {:>5}", x); }
    println!();
    for zi in 0..11 {
        let z = 63.4f32 + zi as f32 * 0.2;
        print!("z {:>5}", z);
        for &x in &xs {
            let p = [x, 3.5, z];
            let t = sim.world.segment(p, [p[0], -2.5, p[2]]);
            match t { Some(t) => print!(" {:>5.2}", 3.5 - t * 6.0), None => print!("  open") }
        }
        println!();
    }

    for y in [2.8f32, 3.3] {
        println!("== (b) eastward rays at y {y}: x 41.2 -> 44.8 ==");
        for zi in 0..17 {
            let z = 63.0f32 + zi as f32 * 0.25;
            let from = [41.2, y, z];
            let to = [44.8, y, z];
            match sim.world.segment(from, to) {
                Some(t) => {
                    let hx = 41.2 + t * (44.8 - 41.2);
                    let fl = sim.world.segment([hx + 0.05, 3.5, z], [hx + 0.05, -2.5, z]).map(|t| 3.5 - t * 6.0);
                    println!("z {z:>5}: wall at x {hx:.2} (floor past wall {fl:?})");
                }
                None => println!("z {z:>5}: clear"),
            }
        }
        println!("== (c) westward rays at y {y}: x 45.2 -> 41.4 ==");
        for zi in 0..17 {
            let z = 63.0f32 + zi as f32 * 0.25;
            let from = [45.2, y, z];
            let to = [41.4, y, z];
            match sim.world.segment(from, to) {
                Some(t) => {
                    let hx = 45.2 + t * (41.4 - 45.2);
                    let fl = sim.world.segment([hx - 0.05, 3.5, z], [hx - 0.05, -2.5, z]).map(|t| 3.5 - t * 6.0);
                    println!("z {z:>5}: wall at x {hx:.2} (floor past wall {fl:?})");
                }
                None => println!("z {z:>5}: clear"),
            }
        }
    }
}
