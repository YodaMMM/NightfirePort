// real-health-14-00agent-4 scratch: the mezzanine west lip (x ~45.1, z ~39.9) raw-input probe.
// The -3 report measured x holding 45.1338 for 600 raw forward ticks under the spine fallback;
// this session the pushes at the lip print (0,0,0) (d = r exactly on both faces) — so what holds
// x? The probe poses Bond crouched AT the lip with `honest_ref` on and holds forward (west),
// printing the pose every 50 ticks, with and without the flag.
use nf_game::player::Input;

fn run(honest: bool) {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [45.13, 3.05, 39.87];
    sim.bond.yaw = -std::f32::consts::FRAC_2_PI; // west-facing candidate; the printout confirms
    sim.bond.crouched = true;
    sim.bond.cc = 0.65;
    sim.bond.vel = [0.0; 3];
    sim.bond.honest_ref = honest;
    sim.step_bond(Input::default(), false, false, false);
    println!("== honest={honest} yaw={} after settle: pos={:?} vel={:?}", sim.bond.yaw, sim.bond.pos, sim.bond.vel);
    for t in 0..600u32 {
        let input = Input { forward: 1.0, ..Input::default() }; // forward = the facing
        sim.step_bond(input, false, false, false);
        if t % 50 == 0 || t == 599 {
            println!("t {t:>3}: pos=({:.4},{:.4},{:.4}) vel=({:.4},{:.4},{:.4}) grounded={} crouched={}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.vel[0], sim.bond.vel[1], sim.bond.vel[2],
                sim.bond.grounded, sim.bond.crouched);
        }
        if sim.bond.pos[0] < 44.0 { println!("t {t}: the lip released (x < 44)"); break; }
    }
}

fn main() {
    run(true);
    run(false);
}
