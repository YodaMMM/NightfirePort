//! Scratch: the spawn settle — tick the port from the 0a spawn with no input and log the root.
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let cold = nf_view::load_with_carry(&path, "0700000a.bin", false, None).expect("load");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut scene = nf_view::load_with_carry(&path, "0700000a.bin", false, Some(&save)).expect("warm");
    let sim = scene.sim.as_mut().unwrap();
    println!("spawn root bits = {:#x} ({:.7})", sim.bond.pos[1].to_bits(), sim.bond.pos[1]);
    let input = nf_game::player::Input::default();
    for t in 1..=12u32 {
        sim.step_bond(input, false, false, false);
        println!("t{t}: root bits {:#x} ({:.7}) grounded {} vel {:?}", sim.bond.pos[1].to_bits(), sim.bond.pos[1], sim.bond.grounded, sim.bond.vel);
    }
}
