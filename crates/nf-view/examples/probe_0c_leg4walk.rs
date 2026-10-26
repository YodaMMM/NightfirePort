//! Scratch (level-0c-display-hatch): real-input walk of the leg-4 candidate route.
//! Route [D]: through the open vent onto the balcony strip, then W off the balcony bridge
//! (x 4.2) onto the mid floor 11.27, S along it, off its north-edge region into the ground
//! pocket 6.27, N up the 7.2 ramp under the walkway (the display case).
use nf_game::player::Input;
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
    let yaw_s = std::f32::consts::PI;      // forward = (0, ·, −1)
    let yaw_w = -std::f32::consts::FRAC_PI_2; // forward = (sin, ·, cos) = (−1, ·, 0)
    let yaw_n = 0.0;                        // (0, ·, +1)
    let mut log = |s: &str, scene: &nf_view::scene::Scene| {
        let p = scene.sim.as_ref().unwrap().bond.pos;
        println!("{s}: pos ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);
    };
    // Bond on the balcony strip just south of the opening (as after legs 2-3).
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [5.8, 15.0, -112.6], yaw_s);
    drop(sim);
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    }
    log("start on the balcony strip", &scene);
    let mut phase = |scene: &mut nf_view::scene::Scene, yaw: f32, ticks: u32, tag: &str| {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.yaw = yaw;
        for t in 1..=ticks {
            sim.step_bond(Input { forward: 1.0, crouch: false, ..Default::default() }, false, false, false);
            if t % 60 == 0 {
                let p = sim.bond.pos;
                println!("  {tag} t{t}: ({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]);
            }
        }
    };
    phase(&mut scene, yaw_w, 240, "W");   // off the balcony bridge edge (x 4.2) onto the mid floor
    log("after W", &scene);
    phase(&mut scene, yaw_s, 240, "S");   // south along the mid floor to its end region
    log("after S", &scene);
    phase(&mut scene, yaw_w, 180, "W2");  // toward the mid floor's west end / off edges
    log("after W2", &scene);
    phase(&mut scene, yaw_n, 300, "N");   // north into the case pocket under the walkway
    log("after N", &scene);
    let p = scene.sim.as_ref().unwrap().bond.pos;
    println!("FINAL: ({:.2}, {:.2}, {:.2}) — case reached: y<9.5 && z<-111.5: {}", p[0], p[1], p[2], p[1] < 9.5 && p[2] < -111.5);
}
