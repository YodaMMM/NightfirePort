// real-health-14-00agent-5 scratch: the 1.72-landing float wedge.
//
// The d2 real-damage walk wedges with Bond hanging AIRBORNE at (33.90, 0.68, 31.14)/(33.90,
// 0.68, 35.38) — grounded=false, floor_ny 1.0, the real floor below at y≈0.0, a scoped floor
// read at 0.34 — for thousands of ticks (the walker's recovery hop can't fire: never grounded).
// This probe poses Bond AT the float spots and answers:
//   1. does a bare pose hang (the wedge is state-local) or fall (the wedge needs the approach)?
//   2. what do the floor ray / the scoped floors read at the spot?
//   3. does the westward steer move at all (the resupply target is 1.65 west)?
use nf_game::player::Input;

fn run_spot(iso: &str, pos: [f32; 3], label: &str, steer_west: bool) {
    let mut scene = nf_view::load(iso, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = pos;
    sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
    sim.bond.crouched = false;
    sim.bond.vel = [0.0; 3];
    // one settle step like the probes
    sim.step_bond(Input::default(), false, false, false);
    println!("== {label} posed ({:.3},{:.3},{:.3}) cc {:.3} -> after 1 settle: ({:.4},{:.4},{:.4}) grounded {} vel {:?}",
        pos[0], pos[1], pos[2], sim.bond.cc, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded, sim.bond.vel);
    for t in 0..600u32 {
        let inp = if steer_west {
            Input { forward: 1.0, ..Default::default() }
        } else {
            Input::default()
        };
        sim.step_bond(inp, false, false, false);
        if t % 100 == 0 || t == 599 {
            let p = sim.bond.pos;
            println!("  {label} t {t:>3}: ({:.4},{:.4},{:.4}) vel {:?} grounded {} floor_ny {:.2}",
                p[0], p[1], p[2], sim.bond.vel, sim.bond.grounded, sim.bond.floor_ny);
        }
    }
}

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    run_spot(&iso, [33.90, 0.68, 31.14], "floatA(resupply)", false);
    run_spot(&iso, [33.90, 0.68, 35.38], "floatB(leg)", false);
    run_spot(&iso, [33.90, 0.68, 31.14], "floatA-steer", true);
}
