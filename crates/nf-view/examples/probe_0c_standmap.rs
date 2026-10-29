//! Scratch (level-0c-display-hatch): standable-surface height map around the display vent.
//! For a grid of (x, z), ray-cast straight down and print the hit y (what Bond can stand on).
use nf_game::world::CollisionWorld;
fn probe(sim: &nf_game::world::CollisionWorld, top: f32, bot: f32, z0: f32, z1: f32) {
    let xs: String = (-1..38).map(|i| format!("{:5.0}", -1.0 + 0.5 * i as f32)).collect();
    println!("   x:      {}", xs);
    let n = ((z1 - z0) / 0.5).round() as i32;
    for k in 0..=n {
        let z = z0 + 0.5 * k as f32;
        let mut row = format!("z {:7.1}: ", z);
        for i in -1..38 {
            let x = -1.0 + 0.5 * i as f32;
            match sim.segment_hit([x, top, z], [x, bot, z]) {
                Some((t, _)) => {
                    let y = top + t * (bot - top);
                    row.push_str(&format!("{:5.1}", y));
                }
                None => row.push_str("    ."),
            }
        }
        println!("{row}");
    }
}
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let scene = nf_view::load(&path, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_ref().unwrap();
    println!("== HIGH: down from 19.5 to 10.5 (roof / walkway / bay floor / mid floor)");
    probe(&sim.world, 19.5, 10.5, -114.0, -106.0);
    println!("== LOW: down from 11.4 to 4.5 (under the mid floor, the case, the ground)");
    probe(&sim.world, 11.4, 4.5, -118.0, -106.0);
}
