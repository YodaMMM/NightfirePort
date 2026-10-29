//! Scratch (level-06-every-objective): what happens at the gallery deck's north edge (x 35,
//! z −64.8, deck 49.0 → shelf 45.0)? Settle Bond on the deck and hold forward north — pure
//! evidence for the walkreal leg (this probe may teleport; the test may not).
use nf_game::player::Input;

fn run(sim: &mut nf_view::scene::Sim, label: &str, jump: bool) {
    println!("== {label}");
    // Settle on the deck at x 35, z −65.6 (deck 49.0), facing north (+z).
    sim.bond.pos = [35.0, 49.2, -65.6];
    sim.bond.yaw = 0.0;
    let mut input = Input { forward: 1.0, ..Default::default() };
    input.jump = jump;
    for t in 0..300 {
        sim.step_bond(input, false, false, false);
        if t % 20 == 0 {
            println!("  t{:3}: pos {:?} grounded {} floor_ny {:.2}", t, sim.bond.pos, sim.bond.grounded, sim.bond.floor_ny);
        }
    }
    println!("  end: pos {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
}

fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load(&path, "07000006.bin", false).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    run(sim, "walk north off the deck edge (no jump)", false);
    run(sim, "walk+jump north off the deck edge", true);
}
