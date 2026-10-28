//! scratch: identify the bots near the 07000007 hall wedge. Assertion-free.
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
#[test]
fn who_stands_in_the_hall() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000007.bin", false).unwrap();
    for t in 0..400 {
        let mut s = scene.sim.as_ref().unwrap();
        drop(s);
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.list.iter().enumerate() {
        let dx = d.pos[0] - 79.57; let dz = d.pos[2] - (-50.34);
        if dx*dx + dz*dz < 40.0 {
            println!("drone {} skin {:#x} pos {:?} state {:#x} started {} enabled {} corpse {} hp {} down {} team {}",
                i, d.skin, d.pos, d.state, d.started, d.enabled, d.corpse, d.health, d.down, d.team);
        }
    }
    println!("bond {:?}", sim.bond.pos);
}
