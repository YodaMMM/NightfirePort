//! drone-block-swaps-g055-4 probe: dump 07000016's team-1 guards at spawn — the positions,
//! skins, weapons and health the ramp-mouth pre-clear and the fight lever design ride on.
use nf_game::player::Input;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000016.bin", false).expect("load 07000016");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..300 { sim.step_bond(Input::default(), false, false, false); }
    println!("team-1 guards at +300 ticks:");
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.team == 1 && d.present && !d.corpse {
            println!("  #{i} placement {} pos ({:.2},{:.2},{:.2}) skin {:#x} weapon {:#x} hp {} state {:#x} enabled {}",
                d.placement, d.pos[0], d.pos[1], d.pos[2], d.skin, d.weapon, d.health, d.state, d.enabled);
        }
    }
    println!("weapon defs of interest: 0x11 (Delta Repeater), 0x14 (the guards' AR), 0x47/0x48/0x4a (the chain kit)");
}
