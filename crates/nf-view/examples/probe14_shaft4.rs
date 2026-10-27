// level-14-full-walk scratch 4: vertical cross-sections — floor/ceiling pairs along the
// passage z 52 -> 46.5 at x = 44.2/44.4/44.6/45.0/45.4/46.0/46.3, plus the west mirror.
// Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    let world = &sim.world;

    for &x in &[44.2f32, 44.4, 44.6, 45.0, 45.4, 46.0, 46.3] {
        println!("== x {x}: z from 52.0 south to 46.5 step 0.25 — [floor|ceiling] from y 0.2 ==");
        let mut z = 52.0f32;
        while z >= 46.49 {
            let f = world.segment([x, 0.2, z], [x, -6.0, z]).map(|t| 0.2 - t * 6.2);
            let c = world.segment([x, 0.2, z], [x, 4.4, z]).map(|t| 0.2 + t * 4.2);
            let s = match (f, c) {
                (Some(fv), Some(cv)) => format!("[{fv:.2}|{cv:.2}]"),
                (Some(fv), None) => format!("[{fv:.2}|open]"),
                (None, Some(cv)) => format!("[----|{cv:.2}]"),
                (None, None) => "[----|----]".to_string(),
            };
            print!("{s} ");
            z -= 0.25;
        }
        println!();
    }
    // The lip fine structure: x 44.7..45.9 step 0.15, z 48.9..47.9 step 0.1 — floor from y 3.2
    // (sees the lane top) and from y 1.6 (sees below-lip geometry).
    println!("== lip fine: down from y 3.2, x 44.7..45.9 step .15, z 48.9..47.9 step .1 ==");
    let mut z = 48.9f32;
    while z >= 47.89 {
        print!("z {z:5.2}: ");
        let mut x = 44.7f32;
        while x <= 45.91 {
            match world.segment([x, 3.2, z], [x, 1.0, z]) {
                Some(t) => print!("{:.2} ", 3.2 - t * 2.2),
                None => print!("open "),
            }
            x += 0.15;
        }
        println!();
        z -= 0.1;
    }
}
