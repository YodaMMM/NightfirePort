//! Scratch (level-0c-display-hatch): standable-surface height map around the display vent
//! AFTER the four hinges are burnt and the hatch has fully opened (the SS meshes moved).
use nf_game::player::Input;
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
                Some((t, _)) => row.push_str(&format!("{:5.1}", top + t * (bot - top))),
                None => row.push_str("    ."),
            }
        }
        println!("{row}");
    }
}
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load(&path, "0700000c.bin", false).expect("load 0700000c");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        for k in 0..sim.drones.locks.len() {
            if (14..=17).contains(&sim.drones.locks[k].out) {
                sim.drones.locks[k].burn_through(&mut sim.drones.channels, 1);
            }
        }
        for _ in 0..300 { sim.step_bond(Input::default(), false, false, false); }
    }
    let scene = scene;
    let sim = scene.sim.as_ref().unwrap();
    println!("== OPEN state, HIGH: down from 19.5 to 10.5");
    probe(&sim.world, 19.5, 10.5, -114.0, -108.0);
    println!("== OPEN state, LOW: down from 11.4 to 4.5");
    probe(&sim.world, 11.4, 4.5, -118.0, -108.0);
}
