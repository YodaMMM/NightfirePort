//! probe (assertion-free, `--ignored`): a LOS sweep for the 0d yard leg — for a grid of Bond
//! spots (and heights) around the death area, which cells does drone #31 (skin 0x05000087 at
//! (161.1,6.9,-235.1), the walkreal killer) see? Prints "x z y0 los" lines for routing cover.
//! Run: NF_ISO=… cargo test -p nf-view --test probe_0d_los_grid -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_los_grid_for_sniper_31() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let idx = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| d.skin == 0x0500_0087
            && (d.pos[0] - 161.1).abs() < 1.0 && (d.pos[2] + 235.1).abs() < 1.0)
            .unwrap_or_else(|| sim.drones.list.iter().position(|d| d.skin == 0x0500_0087).expect("drone 31"))
    };
    eprintln!("[grid] drone idx {idx}");
    // Sweep the yard + the warehouse ground around the death leg.
    for x in [180.0, 184.0, 188.0, 192.0, 196.0, 200.0, 204.0, 208.0, 212.0, 216.0] {
        for z in [-268.0, -264.0, -260.0, -256.0, -252.0, -248.0, -244.0, -240.0, -236.0, -232.0, -228.0, -224.0] {
            for y in [-3.0f32, 0.0, 1.5] {
                {
                    let sim = w.scene.sim.as_mut().unwrap();
                    sim.bond.pos = [x, y, z];
                    sim.bond.vel = [0.0; 3];
                }
                w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
                let sim = w.scene.sim.as_ref().unwrap();
                let dr = &sim.drones.list[idx];
                // dr.los is refreshed by the drone tick for the CURRENT bond pos — one step above did it.
                eprintln!("[grid] x{x:.0} z{z:.0} y{y:.0} los {} dist {:.1} seen_ticks {}",
                    dr.los, dr.dist, dr.seen_ticks);
            }
        }
    }
}
