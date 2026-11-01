//! probe18: does the crane glist's COLLISION rotate with the visual? Ray-downs at the rest and
//! the rotated jib walkway spots, before and after the swing. Print-only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "probe18: the glist collision rotation; run with --ignored --nocapture"]
fn probe18_collision_rotation() {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let rays = |w: &Walker, tag: &str| {
        let sim = w.scene.sim.as_ref().unwrap();
        let spots = [
            ("rest jib r7", [194.15f32, -265.9]),
            ("end jib r7", [201.55, -269.3]),
            ("rest deck r3", [198.0, -264.4]),
            ("end deck r3", [200.3, -261.0]),
        ];
        for (name, xz) in spots { let (x, z) = (xz[0], xz[1]);
            let mut row = String::new();
            let mut top = 20.0f32;
            for _ in 0..4 {
                match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
                    Some((t, _)) => { let y = top - top * t; row.push_str(&format!("{y:.2} ")); top = y - 0.05; }
                    None => { row.push_str("-- "); break; }
                }
            }
            eprintln!("[p18] {tag} {name} ({x},{z}): {row}");
        }
    };
    rays(&w, "BEFORE");
    assert!(w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 30 }), "E #450"));
    w.wait(200);
    rays(&w, "MID(200)");
    w.wait(300);
    rays(&w, "MID(500)");
    w.wait(400);
    rays(&w, "AFTER(900)");
    let rot = w.scene.sim.as_ref().unwrap().drones.scripts.entity_matrix((0x0600_0a08, 0))
        .map(|m| m[0][2].atan2(m[0][0]));
    eprintln!("[p18] final rot {rot:?}");
}
