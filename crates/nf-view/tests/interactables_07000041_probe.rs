//! interactables-41 probe: dump every player-touchable placement on 07000041 (the space-station
//! approach wrapper) by type, with its parameters and channel wiring — the census source for
//! docs/levels/07000041.md. Diagnostic only — run with --nocapture. [D data via nf-formats;
//! the port's collections are what the game's parsemap_create_dynamic_objects 0x1d04f0 builds.]

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn p3(v: [f32; 3]) -> String { format!("({:.2},{:.2},{:.2})", v[0], v[1], v[2]) }

#[test]
fn dump_census() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000041.bin", false).expect("load 41");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 07000041 census ===");
    eprintln!("bond start {} yaw {:.3} disabled {}", p3(sim.bond.pos), sim.bond.yaw, sim.disabled);
    eprintln!("cels {} ladders {} wires {} creepwalls {} icons {} hints {}",
        scene.cels.len(), sim.ladders.len(), sim.wires.len(), sim.creepwalls.len(),
        sim.icons.len(), sim.hints.len());
    for (i, l) in sim.ladders.iter().enumerate() { eprintln!("ladder[{i}] at {}", p3(l.centre)); }
    for (i, w) in sim.wires.iter().enumerate() { eprintln!("wire[{i}] at {}", p3(w.centre)); }
    for (i, c) in sim.creepwalls.iter().enumerate() { eprintln!("creepwall[{i}] at {}", p3(c.pos)); }
    for (i, h) in sim.icons.iter().enumerate() { eprintln!("icon[{i}] at {}", p3(h.pos)); }
    eprintln!("--- triggers {} ---", sim.drones.triggers.len());
    for t in &sim.drones.triggers {
        let list = t.list.iter().map(|x| format!("{x:#x}")).collect::<Vec<_>>().join(",");
        eprintln!("trig kind={:2} out={:#04x} mask={:#04x} lock={} gate={} at={} list=[{}]",
            t.kind, t.out, t.mask, t.lock, t.gate, p3(t.centre()), list);
    }
    eprintln!("--- searchlight parts {} ---", sim.drones.searchlights.parts.len());
    eprintln!("--- fuseboxes {} ---", sim.drones.fuseboxes.len());
    for (i, f) in sim.drones.fuseboxes.iter().enumerate() {
        eprintln!("fusebox[{i}] placement={} at={} ch={:?}", i, p3(f.pos), f.ch);
    }
    eprintln!("--- grapple points {} ---", sim.drones.grapple_points.len());
    for (i, g) in sim.drones.grapple_points.iter().enumerate() { eprintln!("grapple[{i}] at {}", p3(g.pos)); }
    eprintln!("--- worldobjs: rotors {} inverters {} shooters {} dynobjs {} thirdcams {} music {} ---",
        sim.drones.worldobjs.rotors.len(), sim.drones.worldobjs.inverters.len(),
        sim.drones.worldobjs.shooters.len(), sim.drones.worldobjs.dynobjs.len(),
        sim.drones.worldobjs.thirdcams.len(), sim.drones.worldobjs.music_triggers.len());
    for (i, s) in sim.drones.worldobjs.shooters.iter().enumerate() { eprintln!("shooter[{i}] at={} arm={:#x} weapon={:#x} fired={}", p3(s.pos), s.arm, s.weapon, s.fired); }
    for (i, d) in sim.drones.worldobjs.dynobjs.iter().enumerate() { eprintln!("dynobj[{i}] at={} ch_a={:#x} ch_b={:#x} mode={}", p3(d.pos), d.ch_a, d.ch_b, d.mode); }
    for (i, t) in sim.drones.worldobjs.thirdcams.iter().enumerate() { eprintln!("thirdcam[{i}] cam_id={} flag={} keys={}", t.cam_id, t.flag, t.key_count); }
    eprintln!("--- sound triggers {} ---", sim.drones.sound_triggers.list.len());
    eprintln!("--- pickups {} ---", sim.drones.pickups.len());
    for (i, p) in sim.drones.pickups.iter().enumerate() {
        eprintln!("pickup[{i}] kind={} weapon={:#x} rounds={} state={} at={}", p.kind, p.weapon, p.rounds, p.state, p3(p.start));
    }
    eprintln!("--- breaks {} ---", sim.drones.breaks.list.len());
    for (i, b) in sim.drones.breaks.list.iter().enumerate() {
        eprintln!("break[{i}] placement={} centre={} kind={}", b.placement,
            p3([(b.lo[0]+b.hi[0])/2.0, (b.lo[1]+b.hi[1])/2.0, (b.lo[2]+b.hi[2])/2.0]), b.kind);
    }
    eprintln!("--- AI drones {} ---", sim.drones.list.len());
    eprintln!("--- script players {} ---", sim.drones.scripts.players.len());
    for (i, s) in sim.drones.scripts.players.iter().enumerate() {
        eprintln!("sp[{i}] placement={} script={:#x}/{:#x} mode={:?} state={} out={:#x} mask={:#x} gate={:#x} hp={}",
            s.placement, s.scripts[0], s.scripts[1], s.modes, s.state, s.out, s.mask, s.gate, s.hit_points);
    }
    eprintln!("--- mines {} ---", sim.drones.mines.len());
    for (i, m) in sim.drones.mines.iter().enumerate() {
        eprintln!("mine[{i}] placement={} at={} damage={} radius={} script={:#x} tris={}", m.placement, p3(m.pos), m.damage, m.radius, m.script, m.tris.len());
    }
    eprintln!("--- channels on at load ---");
    let on: Vec<String> = (0..256u32).filter(|&c| sim.drones.channels.on(c)).map(|c| format!("{c:#x}")).collect();
    eprintln!("{}", on.join(","));
    eprintln!("=== end census ===");
}
