//! Probe 2 (level-14-tripbomb-real-route): approach the rack wall from the west and take a
//! tripbomb. Prints the floor grid around the armoury, then Bond walks east into the rack wall at
//! pickup #24's z and we watch the take.

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_tripbomb_approach() {
    let iso_path = iso();
    let scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    for z in [30.0f32, 30.7, 31.1, 32.0, 33.0, 34.0, 35.0, 35.9, 36.4, 37.0] {
        let mut row = String::new();
        for x in [-36.0f32, -35.0, -34.0, -33.5, -33.0, -32.6, -32.3, -32.0, -31.5] {
            let cell = match sim.world.floor_below([x, 6.0, z], 12.0) {
                Some(v) => format!("({x},{v:.2}) "),
                None => format!("({x},--) "),
            };
            row.push_str(&cell);
        }
        eprintln!("[probe2] z {z}: {row}");
    }
    drop(sim);
    // Bond walks east along the lane to pickup #25's z, then into the rack.
    let mut w = Walker::new(nf_view::load(&iso_path, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-34.5f32, 1.0, 33.6];
        sim.bond.yaw = 0.0; // face east (+x) at the lane
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    }
    let before = w.scene.sim.as_ref().unwrap().drones.taken.len();
    let r = w.try_walk_to(Wp::new(-33.0, 0.85, 35.86).r(0.35), "the rack front at pickup #25");
    eprintln!("[probe2] walk_to the rack front: {r} bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    for i in 0..120 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.taken.len() > before { eprintln!("[probe2] TAKE at tick {i}"); break; }
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[probe2] final bond {:?} taken {:?} trips left {}",
        sim.bond.pos,
        sim.drones.taken.iter().skip(before).map(|(_, k)| format!("{k:?}")).collect::<Vec<_>>(),
        sim.drones.pickups.iter().filter(|p| p.weapon == 0x3a && !p.gone).count());
}
