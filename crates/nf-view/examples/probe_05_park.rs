//! Scratch (level-0a-office-hatch-6): the ch-0x70 parking plate's box, straight from the ride —
//! the disc's own TouchOnce entity (script stream 6 key 8 [D], y 37.7418) created by the truck
//! script's stream, through the port's trigger load. Prints the box at the park tick with Bond's
//! root and the touch-capsule relation (the miss side).
use nf_game::player::{Bond, Input};
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load(&path, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = Bond::new([100.0, 26.0, 108.0], sim.bond.yaw);
    let mut prev_dis = sim.disabled;
    let mut crouch_armed = false;
    let mut input = Input::default();
    for t in 1..=7000u32 {
        if !sim.disabled && prev_dis && sim.bond.pos[0] < 95.0 { crouch_armed = true; }
        prev_dis = sim.disabled;
        if crouch_armed && !sim.bond.crouched { input.crouch = true; crouch_armed = false; } else { input.crouch = false; }
        sim.step_bond(input, false, false, false);
        let t70 = sim.drones.triggers.iter().find(|tr| tr.out == 0x70);
        if let Some(tr) = t70 {
            if t % 200 == 0 || sim.bond.grounded && sim.bond.pos[1] > 37.5 {
                println!("[t{t}] bond {:?} crouched {} grounded {}  plate lo {:?} hi {:?} tris {}",
                    sim.bond.pos, sim.bond.crouched, sim.bond.grounded, tr.lo, tr.hi, tr.tris.len());
            }
        }
        if sim.drones.channels.on(0x70) {
            println!("[park t{t}] bond {:?}", sim.bond.pos);
            let tr = sim.drones.triggers.iter().find(|tr| tr.out == 0x70).unwrap();
            println!("[plate] lo {:?} hi {:?}  tris:", tr.lo, tr.hi);
            for tri in tr.tris.iter().take(8) { println!("  {:?}", tri); }
            break;
        }
    }
}
