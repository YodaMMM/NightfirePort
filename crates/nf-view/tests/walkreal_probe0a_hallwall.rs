//! probe: the hall-wall rest scenario [L 2026-10-30]: placed (−15,1.2,16.3), yaw pi, forward 300.
#[path = "common/walker_b2.rs"]
mod walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
#[test]
fn probe_hall_wall_rest() {
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-15.0, 1.2, 16.3];
        sim.bond.yaw = std::f32::consts::PI;
        sim.bond.grounded = true;
    }
    for t in 0..300 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        if t % 50 == 0 || t == 299 {
            let p = scene.sim.as_ref().unwrap().bond.pos;
            eprintln!("[hall] t={t} pos ({:.4},{:.4},{:.4})", p[0], p[1], p[2]);
        }
    }
}
