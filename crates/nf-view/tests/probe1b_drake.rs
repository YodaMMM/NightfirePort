//! level-1b-complete probe: does spawner #187 produce Drake (skin 0x50_00ba) with the fight gate
//! ch 0x0a driven, from walk-reachable spots (the deck's west edge, the slit float's stall, the
//! island)? The arena floor may not be needed if the spawner's trigger reaches. Assertion-free.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_drake_spawn_from_walk_reachable_spots() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let spots: [(&str, [f32; 3]); 3] = [
        ("deck edge", [9.0, -1.9, -1.0]),
        ("slit mouth", [8.6, -1.9, -0.6]),
        ("stall", [5.0, -1.7, 0.0]),
    ];
    for (name, at) in spots {
        let _ = w.try_walk_to(walker::Wp::new(at[0], at[1], at[2]).r(1.2), name);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(0x0a, 1, sim.drones.frame);
        }
        let mut seen = None;
        for t in 0..900u32 {
            w.tick(Input { forward: 0.2, ..Default::default() }, false);
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                seen = Some((i, t));
                break;
            }
        }
        match seen {
            Some((i, t)) => {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[i];
                eprintln!("[{name}] DRAKE SPAWNED at t{t}: drone[{i}] pos ({:.2},{:.2},{:.2}) enabled {} health {:.0} mode {} state {:#x} dtype {:#x}",
                    d.pos[0], d.pos[1], d.pos[2], d.enabled, d.health, d.mode, d.state, d.dtype);
                if let Some(aim) = w.exposed_aim(i) {
                    eprintln!("[{name}] exposed aim {aim:?}");
                } else {
                    eprintln!("[{name}] NO exposed part; bond {:?}", sim.bond.pos);
                    // The manual LOS: the root-to-root segment vs the world.
                    let from = [sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]];
                    let to = [d.pos[0], d.pos[1], d.pos[2]];
                    let hit = sim.world.segment_bullet(from, to);
                    eprintln!("[{name}] root-root LOS t {hit:?} (None = clear)");
                    let hit2 = sim.world.segment([from[0], from[1] + 0.4, from[2]], [to[0], to[1] + 0.4, to[2]]);
                    eprintln!("[{name}] +0.4 LOS t {hit2:?}");
                }
            }
            None => {
                let sim = w.scene.sim.as_ref().unwrap();
                let n_present = sim.drones.list.iter().filter(|d| d.present).count();
                eprintln!("[{name}] no Drake in 900 ticks (ch 0x0a driven; present drones {n_present}; bond {:?})",
                    sim.bond.pos);
            }
        }
        // Reset the channel for the next spot (the game's own AND chain would hold it; the probe
        // re-drives it per spot).
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(0x0a, 0, sim.drones.frame);
        }
    }
}
