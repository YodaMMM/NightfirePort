//! The seal walk probe: raw full-forward real input at the two candidate lanes (x −31.5 west
//! notch, x −26.0 east notch) from the approach slope south onto the guardhouse roof (36.5).
//! The disc's tri chain [D probe_sealtris05]: approach slope (ny 0.89, up to ~35.3 at z −20) →
//! the 45° ramps (ny 0.70-0.95, z −21..−20, top ~36.0-36.55) → the notch panels (top 36.86) →
//! the capping slopes (36.86→37.0) → the guardhouse roof 36.5 (its north lip only spans
//! x −30.62..−20.5, so x −33.75..−30.62 is open roof edge). Scratch probe — evidence, not a test.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn run_lane(scene: &mut nf_view::scene::Scene, start: [f32; 3], aim_x: f32, label: &str, jump_at: f32) {
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = start;
    sim.bond.vel = [0.0; 3];
    sim.bond.yaw = std::f32::consts::PI; // south
    sim.bond.pitch = 0.0;
    let mut crested = false;
    let mut stall = 0u32;
    let mut last_z = start[2];
    for t in 0..700u32 {
        let (p, yaw) = {
            let s = scene.sim.as_ref().unwrap();
            (s.bond.pos, s.bond.yaw)
        };
        let feet = p[1] - 1.0847;
        // crest: south of the roof edge, feet near the roof 36.5
        if p[2] < -21.7 && feet > 35.8 && feet < 37.8 { crested = true; }
        let desired = (-21.9f32 - p[2]).atan2(aim_x - p[0]);
        // steer toward (aim_x, z −21.9): yaw = atan2(dx, dz)
        let want = (aim_x - p[0]).atan2(-21.9 - p[2]);
        let err = want - yaw;
        let err = err - std::f32::consts::PI * (err / std::f32::consts::PI).round() * 0.0; // keep signed
        let err = if err > std::f32::consts::PI { err - 2.0 * std::f32::consts::PI } else if err < -std::f32::consts::PI { err + 2.0 * std::f32::consts::PI } else { err };
        let jump = jump_at > -90.0 && p[2] < jump_at && p[2] > jump_at - 1.2 && t % 6 == 0;
        let input = Input { turn: err.clamp(-0.07, 0.07), forward: 1.0, jump, ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if t % 10 == 0 || (p[2] - last_z).abs() < 1e-4 {
            eprintln!("[{label} t{t}] pos {:?} feet {feet:.2} yaw {yaw:.2} g {} ny {:.2} bump {} jump {}",
                s.bond.pos, s.bond.grounded, s.bond.floor_ny, s.bond.head_bump, s.bond.jump);
        }
        if (p[2] - last_z).abs() < 1e-3 { stall += 1; } else { stall = 0; }
        last_z = p[2];
        if stall > 200 { eprintln!("[{label}] STALL at {:?}", s.bond.pos); break; }
        if crested { eprintln!("[{label}] CRESTED at tick {t} pos {:?}", s.bond.pos); break; }
    }
    let s = scene.sim.as_ref().unwrap();
    println!("{label}: crested {crested} end pos {:?} feet {:.2} g {} ny {:.2}",
        s.bond.pos, s.bond.pos[1] - 1.0847, s.bond.grounded, s.bond.floor_ny);
}

fn main() {
    let iso_path = iso();
    let scene = &mut nf_view::load(&iso_path, "07000005.bin", false).expect("load 07000005");
    // settle the load
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    // lane A: the west notch (x −31.75..−30.25, pillar −31.38..−30.63 → aim the east gap −30.44)
    run_lane(scene, [-31.0, 35.5 + 1.0847, -18.2], -31.5, "A west-notch", -90.0);
    // lane B: the east notch (x −26.5..−25.24, pillar −26.38..−25.62 → aim −26.0)
    run_lane(scene, [-26.0, 34.0 + 1.0847, -18.2], -26.0, "B east-notch", -90.0);
    // lane C: the east notch with a JUMP pressed at the ramp's top third (z −20.6..−21.0)
    run_lane(scene, [-26.0, 34.0 + 1.0847, -18.2], -26.0, "C east-notch+jump", -20.7);
    // lane D: the west notch with a JUMP
    run_lane(scene, [-31.0, 35.5 + 1.0847, -18.2], -31.5, "D west-notch+jump", -20.7);
}
