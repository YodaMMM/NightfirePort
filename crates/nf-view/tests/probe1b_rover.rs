//! level-1b-complete probe: a westward rover. Walks Bond west from the spawn with real inputs,
//! dumping the local floor map every 600 ticks so the route's materializing geometry shows up.
//! Assertion-free.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::wrap_pi;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_rover_west() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load");
    // The arrival settle: wait out the shuttle-arrival cut-scene (the walkreal wait_enabled).
    for t in 0..3200u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if t % 400 == 0 { eprintln!("settle t{t}: disabled {} pos {:?}", scene.sim.as_ref().unwrap().disabled, scene.sim.as_ref().unwrap().bond.pos); }
    }

    // The rover: face the next waypoint, walk forward with small turns. A southern arc around
    // the pit (the platform's south deck first).
    let wps: [[f32; 2]; 14] = [
        [21.0, -0.5], [18.0, -0.5], [15.0, -0.5], [12.0, -0.5], [10.0, -0.5], [8.0, 0.0], [7.0, 0.0], [6.0, 0.0], [5.0, 0.0], [3.0, 0.0], [1.0, 0.0], [-2.0, 0.0], [-6.0, 0.0], [-7.3, 0.1],
    ];
    let mut wp = 0usize;
    for t in 0..5000u32 {
        {
            let sim = scene.sim.as_mut().unwrap();
            let p = sim.bond.pos;
            let (tx, tz) = (wps[wp][0], wps[wp][1]);
            let dx = tx - p[0]; let dz = tz - p[2];
            if dx * dx + dz * dz < 1.2 { wp = (wp + 1) % wps.len(); }
            let desired = dx.atan2(dz);
            let err = wrap_pi(desired - sim.bond.yaw);
            sim.step_bond(Input { forward: 1.0, turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
        }
        let sim = scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if t % 60 == 0 {
            let f = sim.world.floor_below([p[0], p[1], p[2]], 4.0);
            eprintln!("t{t}: pos ({:6.2},{:6.2},{:6.2}) grounded {} sub {} floor {f:?} wp{wp}",
                p[0], p[1], p[2], sim.bond.grounded, sim.bond.substate);
        }
        // The local floor map every 600 ticks: an x strip ±8, z rows ±4.
        if t % 500 == 499 {
            eprintln!("--- map around Bond (x {:6.2}, z {:6.2}, step 1.0, rows z+4..z-4) ---", p[0], p[2]);
            let mut z = p[2] + 4.0;
            while z >= p[2] - 4.0 {
                let mut row = format!("z {:6.1}: ", z);
                let mut x = p[0] - 8.0;
                while x <= p[0] + 8.0 {
                    match sim.world.floor_below([x, 0.5, z], 18.0) {
                        Some(f) if (-4.2..=0.0).contains(&f) => row.push('#'),
                        Some(_) => row.push('.'),
                        None => row.push(' '),
                    }
                    x += 1.0;
                }
                eprintln!("{row}");
                z -= 1.0;
            }
        }
        if p[1] < -14.0 { eprintln!("FELL into the pit at t{t}"); break; }
    }
}
