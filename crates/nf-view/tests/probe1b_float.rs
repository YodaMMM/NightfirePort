//! level-1b-drake-fight probe: the zero-G crossing route. From the slit mouth, scan the
//! world's westward segments at a grid of heights/z to find where the pit is open (the x 4.98
//! wall spans what?), then thrust-plan candidates. Assertion-free.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn scan_the_pit_westward_openings() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_ref().unwrap();
    println!("=== westward segments from (8.5,y,z) to (-8,y,z) — t = the first hit ===");
    for y in [-1.6f32, 6.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0] {
        for z in [0.0f32, -8.0, -10.0, -12.0, 6.0, 8.0, 10.0, 12.0] {
            let from = [8.5, y, z];
            let to = [-8.0, y, z];
            let hit = sim.world.segment(from, to);
            let hitb = sim.world.segment_bullet(from, to);
            println!("y {y:+.1} z {z:+.1}: seg t {hit:?} bullet {hitb:?}");
        }
    }
    println!("\n=== vertical segments (up from y 0 / down from y 28) at the shaft columns ===");
    for x in [8.5f32, 7.5, 6.5, 5.5, 3.5, 1.5, -3.0, -6.0] {
        for z in [0.0f32, -0.6, 0.6] {
            let up = sim.world.segment([x, 0.0, z], [x, 40.0, z]);
            let down = sim.world.segment([x, 40.0, z], [x, -8.0, z]);
            println!("x {x:+.1} z {z:+.1}: up(t from y0) {up:?} down(t from y40) {down:?}");
        }
    }
    println!("\n=== up-columns over the deck (find the slab's east edge) ===");
    for x in [8.8f32, 9.2, 9.6, 10.0, 11.0, 13.0, 16.0, 20.0, 24.0] {
        for z in [0.0f32, -8.0, 6.0] {
            let up = sim.world.segment([x, 0.0, z], [x, 40.0, z]);
            let down = sim.world.segment([x, 40.0, z], [x, -4.0, z]);
            println!("x {x:+.1} z {z:+.1}: up(t) {up:?} down(t from y40) {down:?}");
        }
    }
    println!("\n=== westward rows ABOVE the slab from over-deck starts ===");
    for y in [17.0f32, 18.0, 20.0, 22.0] {
        for (x0, z) in [(24.0f32, 0.0), (12.0, 0.0), (10.5, 0.0), (10.0, -0.6)] {
            let hit = sim.world.segment([x0, y, z], [-8.0, y, z]);
            println!("from x {x0:+.1} z {z:+.1} y {y:+.1}: west seg {hit:?}");
        }
    }
    println!("\n=== arena columns above Drake's spot ===");
    for (x, z) in [(-7.3f32, 0.06), (-5.0, 0.0), (-2.0, 0.0)] {
        let down = sim.world.segment([x, 24.0, z], [x, -8.0, z]);
        let up = sim.world.segment([x, 0.0, z], [x, 24.0, z]);
        println!("x {x:+.1} z {z:+.1}: down(t from y24) {down:?} up(t) {up:?}");
    }
    println!("\n=== fine westward rows around the mid-shaft script spots z ±8.82, y 4..15 ===");
    for y in [4.0f32, 6.0, 8.0, 9.45, 11.0, 13.0, 15.0] {
        for z in [-9.8f32, -9.3, -8.8, -8.3, -7.8, 7.8, 8.3, 8.8, 9.3, 9.8] {
            let hit = sim.world.segment([8.5, y, z], [-8.0, y, z]);
            let hitb = sim.world.segment_bullet([8.5, y, z], [-8.0, y, z]);
            println!("y {y:+.1} z {z:+.1}: seg {hit:?} bullet {hitb:?}");
        }
    }
}
