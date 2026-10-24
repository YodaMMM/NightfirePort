//! level-0a-office-hatch-4 probe 4: the port's glist `shapes` table near the spawn — the
//! script-player objects' collision (the lift car!) whose posed floor could be the game's
//! −0.0109 spawn surface.
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free dump; --ignored --nocapture"]
fn dump_shapes_near_spawn() {
    let iso = iso();
    let w = Walker::new(nf_view::load(&iso, "0700000a.bin", false).unwrap());
    let sim = w.scene.sim.as_ref().unwrap();
    // The script players' bound objects (the lift car!) + their posed matrices.
    for (pi, pl) in sim.drones.scripts.players.iter().enumerate() {
        println!("SP pl{pi} scripts {:08x}/{:08x} placement {}", pl.scripts[0], pl.scripts[1], pl.placement);
        if let Some(m) = sim.drones.scripts.objects.list.iter().flatten().next() {
            let _ = m;
        }
    }
    // Every running script stream binding with its object's matrix row3.
    for (pi, pl) in sim.drones.scripts.players.iter().enumerate() {
        for (slot, info) in pl.info.iter().enumerate() {
            if let Some(s) = info {
                for (si, (kind, obj, fk, lk)) in s.stream_bindings().iter().enumerate() {
                    if let Some(o) = obj {
                        if let Some(Some(nf_game::nis::Obj::Entity { hash, matrix, .. })) = sim.drones.scripts.objects.list.get(*o) {
                            println!("SP{pi} slot{slot} s{si} k{kind} fk{fk} lk{lk} obj{o} hash {hash:08x} pos ({:.4},{:.4},{:.4})",
                                matrix[3][0], matrix[3][1], matrix[3][2]);
                        }
                    }
                }
            }
        }
    }
    // The shapes table is private; go through the door/ss/hash surfaces the port exposes.
    // Instead: scan the DYNAMIC set's current poses (the doors at rest) for a -0.0109 face.
    println!("\n=== the dynamic set's objects near the spawn (bounds y -0.06..0.06) ===");
    let tris = sim.world.dynamic_tris_near([-6.0, -0.06, 8.0], [0.0, 0.06, 13.0]);
    println!("dynamic tris in the spawn box: {}", tris.len());
    for t in tris.iter().take(12) {
        println!("  tri ({:.3},{:.4},{:.3}) ({:.3},{:.4},{:.3}) ({:.3},{:.4},{:.3})", t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
}
