//! Scratch probe (level-14-qclaw-climb): the 07000014 drop-slot lid script 0x06000A3B —
//! what entities it creates, with which obj_types, and where the port poses them
//! at load / after the E-open. Assertion-free.
use nf_formats::nis::Cmd;

fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();

    // The script's raw commands: every stream, decoded.
    if let Some(nis) = sim.drones.scripts.lib.get(&0x0600_0a3b) {
        println!("== script 0x06000A3B: {} streams ==", nis.streams.len());
        for (si, st) in nis.streams.iter().enumerate() {
            println!(" stream {si}: keys {}..{} commands {}", st.first_key, st.last_key, st.commands.len());
            for c in st.decode().iter().enumerate().take(24) {
                match c.1 {
                    Cmd::Entity { hash, obj_type, params, .. } => {
                        println!("   t{}: Entity {hash:08x} type {obj_type:#04x} params {params:?}", c.0)
                    }
                    other => println!("   t{}: {other:?}", c.0),
                }
            }
        }
    } else {
        println!("script 0x06000A3B NOT in lib; lib len {}", sim.drones.scripts.lib.len());
    }

    // Who plays it (the SP / the mask)?
    for (i, p) in sim.drones.scripts.players.iter().enumerate() {
        if p.scripts.iter().any(|s| *s == 0x0600_0a3b) {
            println!("lid SP players[{i}]: scripts {:?} state {} cur {} mask {} modes {:?}",
                p.scripts, p.state, p.cur, p.mask, p.modes);
        }
    }

    // One tick: the prime pass runs.
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    println!("-- after 1 tick --");
    for (i, p) in sim.drones.scripts.players.iter().enumerate() {
        if p.scripts.iter().any(|s| *s == 0x0600_0a3b) {
            println!("lid SP[{i}] after tick: state {} cur {} info[cur] {:?}",
                p.state, p.cur, p.info[p.cur].as_ref().map(|s| (s.clock, s.play)));
        }
    }
    // Live entities near the slot (x 36..46, z 58..66) and their matrices.
    dump_slot_entities(sim, "at load");
}

fn dump_slot_entities(sim: &nf_view::scene::Sim, tag: &str) {
    println!("-- entities near the slot ({tag}) --");
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let nf_game::nis::Obj::Entity { hash, kind, matrix, .. } = o {
            let (x, y, z) = (matrix[3][0], matrix[3][1], matrix[3][2]);
            if (36.0..46.0).contains(&x) && (56.0..68.0).contains(&z) {
                println!("  entity {hash:08x} kind {kind:#04x} at ({x:.3}, {y:.3}, {z:.3})");
            }
        }
    }
}
