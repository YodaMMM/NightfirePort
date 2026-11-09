//! Probe (scratch, level-0b-roof-sprint): the parachute take zone on 0700000b's roof deck.
//! The deck sprint reaches the take slot but ch 0x3F never fires on the agent stream while the
//! d1 walk takes it on the pass — this probe sweeps the documented open-face band (x 2.0-2.5,
//! z 41.0-41.5 [D probe on the s/fx-scrape tree]) for the spots where the touch law's distance +
//! LOS gates [M Pickup_Update 0x18e148:202-220] actually open, without any guard fire. Evidence
//! for the sprint lane, not a route stand-in. Run with --nocapture.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_0b_parachute_take_zone() {
    let mut scene = nf_view::load_with_carry(&iso(), "0700000b.bin", false, None).expect("load");
    scene.sim.as_mut().unwrap().drones.set_channel(0x2a);
    let pp = {
        let sim = scene.sim.as_ref().unwrap();
        let p = sim.drones.pickups.iter().find(|p| p.switch_channel == 0x3f)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]]).expect("OBJECT_Parachute");
        eprintln!("[take-probe] parachute at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        p
    };
    for x in [0.6f32, 1.0, 1.4, 1.8, 2.2, 2.6, 3.0, 3.4, 3.8] {
        for z0 in [40.0f32] {
            let mut scene = nf_view::load_with_carry(&iso(), "0700000b.bin", false, None).expect("load");
            scene.sim.as_mut().unwrap().drones.set_channel(0x2a);
            {
                let sim = scene.sim.as_mut().unwrap();
                sim.bond.pos = [x, pp[1] - 1.4, z0];
                sim.bond.vel = [0.0; 3];
                sim.bond.yaw = 0.0; // face north (+Z): the walk's yaw = atan2(dx, dz) convention
            }
            for i in 0..150 {
                let inp = nf_game::player::Input { forward: 1.0, ..Default::default() };
                scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
                if scene.sim.as_ref().unwrap().drones.channels.on(0x3f) {
                    let b = scene.sim.as_ref().unwrap().bond.pos;
                    eprintln!("[take-probe] push from ({:.1},{:.1}): TAKEN at tick {} at ({:.2},{:.2},{:.2})",
                        x, z0, i, b[0], b[1], b[2]);
                    break;
                }
                if i == 149 {
                    let b = scene.sim.as_ref().unwrap().bond.pos;
                    eprintln!("[take-probe] push from ({:.1},{:.1}): walled at ({:.2},{:.2},{:.2})",
                        x, z0, b[0], b[1], b[2]);
                }
            }
        }
    }
}
