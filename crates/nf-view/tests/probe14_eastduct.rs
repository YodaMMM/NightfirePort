//! level-14 probe: the middle-channel pocket and the east duct — drop the crouched Bond into
//! (43.5, 2.9, 65.3) and (45.25, 2.9, 65.0), crawl/walk each direction, print trajectories.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn run(start: [f32; 3], yaw: f32, crouch: bool, label: &str) {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let mut traj = Vec::new();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = start;
        sim.bond.yaw = yaw;
        sim.bond.crouched = crouch;
        for t in 0..500 {
            let inp = Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
            if t % 50 == 0 { traj.push(sim.bond.pos); }
        }
        traj.push(sim.bond.pos);
    }
    let pts: Vec<String> = traj.iter().map(|p| format!("({:.2},{:.2},{:.2})", p[0], p[1], p[2])).collect();
    eprintln!("{label}: {}", pts.join(" "));
}

#[test]
fn probe_east_routes() {
    let south = std::f32::consts::PI;          // yaw π = −z
    let east = std::f32::consts::FRAC_PI_2;    // yaw π/2 = +x
    run([43.5, 2.9, 65.3], south, true, "pocket crouch south");
    run([45.25, 2.9, 65.5], south, true, "east duct crouch south");
    run([45.25, 2.9, 65.5], south, false, "east duct stand south");
    run([45.25, 2.9, 64.0], east, true, "east duct crouch east");
}
