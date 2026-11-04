//! drone-block-swaps-g055-4 probe: dump 07000016's pickups (armour/ammo along the route). Prints only.
fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000016.bin", false).expect("load 07000016");
    let sim = scene.sim.as_mut().unwrap();
    println!("pickups (kind weapon rounds) at:");
    for p in &sim.drones.pickups {
        if p.gone { continue; }
        println!("  kind {:#06x} weapon {:#04x} rounds {} at ({:.2},{:.2},{:.2})",
            p.kind, p.weapon, p.rounds, p.m[3][0], p.m[3][1], p.m[3][2]);
    }
}
