//! One-shot scratch probe: g6's AIM_STAND_FIRE ask gates over time (07000012 idle).
//! Scratch; do not commit.
fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 1).expect("load 07000012");
    let mut last = 0u32;
    for t in 0..=840u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[6];
        if t - last > 40 && g.state == 0x68 {
            let p40 = (g.props[2] >> 0) & 1;
            let p11 = (g.props[0] >> 11) & 1;
            println!("t{t} g6 st {:#04x} firing {} burst_done {} burst {} stationary {} p40 {} p11 {} nb {:#04x} nb_hit {} aimed_at {} bond_armed {} clip {} anim {}./{} dist {:.1}",
                g.state, g.firing, g.burst_done, g.burst, g.stationary, p40, p11, g.no_bunching, g.no_bunching_hit,
                g.aimed_at, g.bond_armed, g.clip, g.anim.class, g.anim.state, g.dist);
            last = t;
        }
    }
}
