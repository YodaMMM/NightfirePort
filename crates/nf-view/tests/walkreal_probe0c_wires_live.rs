//! walkreal probe #17 (level-0c-gate-band-crossing): every wire (0x3e) placement with the sim's
//! parsed grab segments, plus every script object (any SP) position at t0/t600/t1200 — looking
//! for ANY dynamic geometry over the band void x 55.6..66.3, z −209..−206.3, y 9..12.
//! Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_wires_and_script_objects() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== wires (scene) ===");
        for (i, wire) in sim.wires.iter().enumerate() {
            eprintln!("wire{i}: {:?}", wire);
        }
    }
    for t in [0u32, 600, 1200, 1800] {
        while w.total_ticks < t {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
            w.total_ticks += 1;
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== t{t}: script entities over the band box (x 50..72 z -212..-203 y 8..20) ===");
        for o in sim.drones.scripts.objects.list.iter().flatten() {
            if let nf_game::nis::Obj::Entity { hash, matrix, .. } = o {
                let (x, y, z) = (matrix[3][0], matrix[3][1], matrix[3][2]);
                if x >= 50.0 && x <= 72.0 && z >= -212.0 && z <= -203.0 && y >= 8.0 && y <= 20.0 {
                    eprintln!("  {hash:08x}@({x:.2},{y:.2},{z:.2})");
                }
            }
        }
        // Also: the doors' current positions (a door's leaf could bridge).
        for d in sim.drones.doors.list.iter() {
            let (x, y, z) = (d.pos[0], d.pos[1], d.pos[2]);
            if x >= 50.0 && x <= 72.0 && z >= -212.0 && z <= -203.0 {
                eprintln!("  door kind {} @({x:.2},{y:.2},{z:.2}) state {:#x} pos01 {:.2}", d.kind, d.state, d.pos01);
            }
        }
    }
    eprintln!("ok");
}
