//! probe16: the jib ride — the jib's walkable spots (ray-down along its rest direction),
//! then the full E #450 → walk onto the jib → the swing → the drop. Print-only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_real() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    w
}

fn pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

#[test]
#[ignore = "probe16: the jib walk map; run with --ignored --nocapture"]
fn probe16_jib_map() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    // Ray-down along the jib's rest direction (the tip at rest over (188.65, -268.92)).
    let (ax, az) = (200.48f32, -262.37f32);
    let (ux, uz) = ((188.65 - ax) / 13.5, (-268.92 - az) / 13.5);
    let sim = w.scene.sim.as_ref().unwrap();
    for r in [2.0f32, 3.5, 5.0, 6.5, 8.0, 9.5, 11.0, 12.5, 13.5] {
        let (x, z) = (ax + r * ux, az + r * uz);
        let mut row = format!("[probe16] jib r{r:>4.1} ({x:.1},{z:.1}): ");
        let mut top = 20.0f32;
        for _ in 0..6 {
            match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
                Some((t, _)) => {
                    let y = top - top * t;
                    row.push_str(&format!("{y:.2} "));
                    top = y - 0.05;
                }
                None => { row.push_str("-- "); break; }
            }
        }
        eprintln!("{row}");
    }
    // Perpendicular width at r=8: the jib's walkable width.
    let (px, pz) = (-uz, ux);
    for off in [-1.5f32, -0.75, 0.0, 0.75, 1.5] {
        let (x, z) = (ax + 8.0 * ux + off * px, az + 8.0 * uz + off * pz);
        let hit = sim.world.segment_hit([x, 20.0, z], [x, 0.0, z]).map(|(t, _)| 20.0 - 20.0 * t);
        eprintln!("[probe16] jib r8 off{off:+.2} ({x:.1},{z:.1}) top {hit:?}");
    }
}

/// The full ride: E #450 from the cab, walk onto the jib (r≈8), hold through the swing,
/// print every 30 ticks; then walk toward the dock edge and report.
#[test]
#[ignore = "probe16: the jib ride; run with --ignored --nocapture"]
fn probe16_jib_ride() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    // E #450 via the proven walk+use.
    let ok = w.try_walk_to(Wp { at: [198.93, 14.16, -264.15], act: Act::None, arrive: 0.8 }, "the cab spot");
    eprintln!("[probe16] cab walk ok {ok} at {:?}", pos(&w));
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 30 }), "E #450 (30 ticks)");
    eprintln!("[probe16] E ok {used} ch0xa {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0xa));
    // Immediately walk onto the jib (r≈8): (193.5, -266.3).
    let tojib = w.try_walk_to(Wp { at: [193.5, 14.5, -266.3], act: Act::None, arrive: 1.0 }, "onto the jib r8");
    eprintln!("[probe16] jib walk ok {tojib} at {:?}", pos(&w));
    // Hold and print the ride.
    for k in 0..30 {
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 0));
        let b = pos(&w);
        eprintln!("[probe16] ride t{} rot {:.3} bond ({:.2},{:.2},{:.2})", (k + 1) * 30,
            m.map(|m| m[0][2].atan2(m[0][0])).unwrap_or(f32::NAN), b[0], b[1], b[2]);
    }
    // After the swing: try walking to the dock (189, 9.6, -276) and report.
    let dock = w.try_walk_to(Wp { at: [189.0, 9.7, -276.0], act: Act::None, arrive: 1.5 }, "to the dock");
    eprintln!("[probe16] dock walk ok {dock} at {:?}", pos(&w));
    for f in &w.report.falls { eprintln!("[probe16] FALL {f}"); }
}
