//! level-0a-low-beam-ride-body-2 probe: the world spheres of the truck-chain and 0a laser triggers.
#[test]
#[ignore = "probe"]
fn dump_trigger_spheres() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    for (level, want) in [("07000005.bin", vec![0x5cu8, 0x70, 0x45]), ("0700000a.bin", vec![0x24u8, 0x10])] {
        let scene = nf_view::load(&iso, level, false).expect("load");
        let sim = scene.sim.as_ref().unwrap();
        for t in &sim.drones.triggers {
            if want.contains(&t.out) || t.placement == 305 || t.placement == 621 || t.placement == 607 {
                let sp = t.sphere.map(|(c, r)| format!("({:.2},{:.2},{:.2}) r {:.2}", c[0], c[1], c[2], r));
                println!("{} plc {} out {:#04x} mask {:#x} lo {:.2?} hi {:.2?} tris {} sphere {:?}",
                    level, t.placement, t.out, t.mask, t.lo, t.hi, t.tris.len(), sp);
            }
        }
    }
}

#[test]
#[ignore = "probe: which trigger does the old box vs the new arm touch at the 05 spawn"]
fn dump_spawn_touches() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let pos = [100.0f32, 26.0, 108.0];
    let old = crate_toucher(pos, pos[1] - 0.05, pos[1] + 1.26);
    let new = crate_toucher(pos, pos[1] + 0.275, pos[1] - 0.4847);
    for t in &sim.drones.triggers {
        let o = t.touched_by(&old);
        let n = t.touched_by(&new);
        if o || n {
            println!("plc {} out {:#04x} mask {:#x} script {:?} old {} new {} lo {:.2?} hi {:.2?} sphere {:?}",
                t.placement, t.out, t.mask, t.script, o, n, t.lo, t.hi, t.sphere.map(|(c, r)| (c, r)));
        }
    }
}
fn crate_toucher(pos: [f32; 3], a_y: f32, b_y: f32) -> nf_game::triggers::Toucher {
    nf_game::triggers::Toucher { a: [pos[0], a_y, pos[2]], b: [pos[0], b_y, pos[2]], r: 0.55, drone: false, skin: 0, contain: true }
}
