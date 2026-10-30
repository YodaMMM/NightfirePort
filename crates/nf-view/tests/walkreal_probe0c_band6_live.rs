//! walkreal probe #12 (level-0c-gate-band-crossing): track the Phoenix forklift's script objects
//! in the running sim. Dumps, every 30 ticks for 2400 ticks, every Obj::Entity whose src script
//! is 0x06000762 (the forklift assembly) with its matrix translation, plus Bond's position.
//! Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_forklift_track() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    for t in 0..2400u32 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
        if t % 30 != 0 { continue; }
        let sim = w.scene.sim.as_ref().unwrap();
        let mut rows: Vec<String> = Vec::new();
        for o in sim.drones.scripts.objects.list.iter().flatten() {
            if let nf_game::nis::Obj::Entity { hash, src, matrix, .. } = o {
                if src.0 != 0x0600_0762 { continue; }
                rows.push(format!("{:08x}@({:.2},{:.2},{:.2})", hash, matrix[3][0], matrix[3][1], matrix[3][2]));
            }
        }
        // The SP players' states too.
        let sps: Vec<String> = sim.drones.scripts.players.iter().enumerate()
            .filter(|(_, p)| p.scripts[0] == 0x0600_0762)
            .map(|(i, p)| format!("sp{i} state {} cur {} deleted {}", p.state, p.cur, p.deleted))
            .collect();
        let b = sim.bond.pos;
        eprintln!("t{t:>5}: bond ({:.1},{:.1},{:.1}) {} {}", b[0], b[1], b[2], sps.join(" "), if rows.len() > 3 { format!("{} parts, body-ish {}", rows.len(), rows.iter().take(3).cloned().collect::<Vec<_>>().join(" ")) } else { rows.join(" ") });
    }
    eprintln!("ok");
}
