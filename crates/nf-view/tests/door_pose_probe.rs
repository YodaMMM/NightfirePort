//! door-state-collision-2 probe: dump the port's door leaves for 07000008 (the Secure Room
//! door #10/122 — closed key1 at yaw −90°, open ends 0°/180° [D the keys]) and 07000007 (the
//! hall door whose OPEN leaf is the sill bridge over the 1.1 doorway step at (94.9,43.0,-47.2))
//! — the parsed anim keys [D] and the create pose. The movement-collision solidity is
//! unconditional (see drone.rs set_dynamic); these dumps pin the leaf poses the game sweeps.
//! Assertion-free, --nocapture.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump(doors: &nf_game::door::Doors, tag: &str) {
    eprintln!("=== {tag}: {} doors ===", doors.list.len());
    for (i, d) in doors.list.iter().enumerate() {
        eprintln!(
            "[{tag}] #{i} placement {} kind {} grp {} ch_open {} ch_lock {} flags {:#06x} state {:#04x} pos01 {:.3} glist {:#010x} keys {}",
            d.placement, d.kind, d.group, d.ch_open, d.ch_lock, d.flags, d.state, d.pos01, d.glist,
            d.keys.as_ref().map_or("NONE".to_string(), |k| format!("{}", k.len())));
        eprintln!("    row3 ({:.3},{:.3},{:.3})", d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]);
        eprintln!("    ax  ({:.3},{:.3},{:.3})", d.matrix[0][0], d.matrix[0][1], d.matrix[0][2]);
        eprintln!("    ay  ({:.3},{:.3},{:.3})", d.matrix[1][0], d.matrix[1][1], d.matrix[1][2]);
        eprintln!("    az  ({:.3},{:.3},{:.3})", d.matrix[2][0], d.matrix[2][1], d.matrix[2][2]);
        if let Some(keys) = &d.keys {
            for (k, key) in keys.iter().enumerate() {
                eprintln!("    key{k} pos ({:.3},{:.3},{:.3}) quat ({:.4},{:.4},{:.4},{:.4})",
                    key.pos[0], key.pos[1], key.pos[2], key.quat[0], key.quat[1], key.quat[2], key.quat[3]);
            }
        }
    }
}

#[test]
fn dump_08_doors() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    w.wait_enabled(3000, "the party start");
    w.wait(30);
    dump(&w.scene.sim.as_ref().unwrap().drones.doors, "08 at start");
}

#[test]
fn dump_07_hall_door() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(3000, "the party start");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            if (d.matrix[3][0] - 94.9).abs() < 4.0 && (d.matrix[3][2] + 47.2).abs() < 4.0 {
                eprintln!("[07 at start] door #{i} placement {} kind {} grp {} ch_open {} ch_lock {} flags {:#06x} state {:#04x} pos01 {:.3} glist {:#010x}",
                    d.placement, d.kind, d.group, d.ch_open, d.ch_lock, d.flags, d.state, d.pos01, d.glist);
                eprintln!("    row3 ({:.3},{:.3},{:.3})", d.matrix[3][0], d.matrix[3][1], d.matrix[3][2]);
                eprintln!("    ax  ({:.3},{:.3},{:.3})", d.matrix[0][0], d.matrix[0][1], d.matrix[0][2]);
                eprintln!("    ay  ({:.3},{:.3},{:.3})", d.matrix[1][0], d.matrix[1][1], d.matrix[1][2]);
                eprintln!("    az  ({:.3},{:.3},{:.3})", d.matrix[2][0], d.matrix[2][1], d.matrix[2][2]);
                eprintln!("    sph ({:.3},{:.3},{:.3}) r {:.3}", d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1);
                if let Some(keys) = &d.keys {
                    for (k, key) in keys.iter().enumerate() {
                        eprintln!("    key{k} pos ({:.3},{:.3},{:.3}) quat ({:.4},{:.4},{:.4},{:.4})",
                            key.pos[0], key.pos[1], key.pos[2], key.quat[0], key.quat[1], key.quat[2], key.quat[3]);
                    }
                } else { eprintln!("    keys NONE"); }
            }
        }
    }
}
