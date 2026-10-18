//! interactables-1b probe: dump every player-touchable placement on 0700001b (the space
//! station) by type, with its parameters and channel wiring — the census source for
//! docs/levels/0700001b.md. Diagnostic only — run with --nocapture. [D data via nf-formats;
//! the port's collections are what the game's parsemap_create_dynamic_objects 0x1d04f0
//! builds.]

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn p3(v: [f32; 3]) -> String { format!("({:.2},{:.2},{:.2})", v[0], v[1], v[2]) }

#[test]
fn dump_census() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 0700001b census ===");
    eprintln!("bond start {} yaw {:.3} disabled {}", p3(sim.bond.pos), sim.bond.yaw, sim.disabled);
    eprintln!("cels {} ladders {} wires {} creepwalls {} icons {} hints {}",
        scene.cels.len(), sim.ladders.len(), sim.wires.len(), sim.creepwalls.len(),
        sim.icons.len(), sim.hints.len());
    for (i, l) in sim.ladders.iter().enumerate() {
        eprintln!("ladder[{i}] at {}", p3(l.centre));
    }
    for (i, w) in sim.wires.iter().enumerate() {
        eprintln!("wire[{i}] at {}", p3(w.centre));
    }
    for (i, c) in sim.creepwalls.iter().enumerate() {
        eprintln!("creepwall[{i}] at {}", p3(c.pos));
    }
    for (i, h) in sim.hints.iter().enumerate() {
        eprintln!("hint[{i}] at {}", p3(h.pos));
    }
    eprintln!("--- triggers {} ---", sim.drones.triggers.len());
    for t in &sim.drones.triggers {
        let list = t.list.iter().map(|x| format!("{x:#x}")).collect::<Vec<_>>().join(",");
        eprintln!("trig kind={:2} out={:#04x} mask={:#04x} lock={} gate={} at={} list=[{}]",
            t.kind, t.out, t.mask, t.lock, t.gate, p3(t.centre()), list);
    }
    eprintln!("--- doors {} ---", sim.drones.doors.list.len());
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        eprintln!("door[{i}] at={} kind={} ch_open={:#x} ch_lock={:#x} lock_flag={} approach={} state={:#x}",
            p3(d.pos), d.kind, d.ch_open, d.ch_lock, d.locked_flag, d.approach, d.state);
    }
    eprintln!("--- locks {} ---", sim.drones.locks.len());
    for (i, l) in sim.drones.locks.iter().enumerate() {
        eprintln!("lock[{i}] at={} out={:#x} unlocked={} burned={}", p3(l.pos), l.out, l.unlocked, l.burned);
    }
    eprintln!("--- useobjs: pcs {} switches {} ---", sim.drones.useobjs.pcs.len(), sim.drones.useobjs.switches.len());
    for (i, q) in sim.drones.useobjs.pcs.iter().enumerate() {
        eprintln!("pc[{i}] at={} ch={:#x} e_usable={} armed={}", p3(q.pos), q.ch, q.e_usable, q.armed);
    }
    for (i, s) in sim.drones.useobjs.switches.iter().enumerate() {
        eprintln!("switch[{i}] at={} ch={:#x} rest={:#x}", p3(s.pos), s.ch, s.rest);
    }
    eprintln!("--- monitors {} ---", sim.drones.monitors.list.len());
    for (i, m) in sim.drones.monitors.list.iter().enumerate() {
        eprintln!("monitor[{i}] placement={} at={} group={:#x}", i, p3(m.pos), m.group);
    }
    eprintln!("--- fuseboxes {} ---", sim.drones.fuseboxes.len());
    for (i, f) in sim.drones.fuseboxes.iter().enumerate() {
        eprintln!("fusebox[{i}] placement={} at={} ch={:?}", i, p3(f.pos), f.ch);
    }
    eprintln!("--- sensors {} ---", sim.drones.sensors.list.len());
    for (i, s) in sim.drones.sensors.list.iter().enumerate() {
        eprintln!("sensor[{i}] placement={} at={} out={:#x} monitor={}", i, p3(s.pos), s.out, s.monitor_id);
    }
    eprintln!("--- grapple points {} ---", sim.drones.grapple_points.len());
    for (i, g) in sim.drones.grapple_points.iter().enumerate() {
        eprintln!("grapple[{i}] at={}", p3(g.pos));
    }
    eprintln!("--- searchlight parts {} ---", sim.drones.searchlights.parts.len());
    eprintln!("--- copters {} ---", sim.drones.copters.len());
    eprintln!("--- sound triggers {} ---", sim.drones.sound_triggers.list.len());
    eprintln!("--- props: photo {} destroy {} gunimps {} | breaks {} ---",
        sim.drones.props.subjects.len(), sim.drones.props.destroy.len(),
        sim.drones.props.gunimps.len(), sim.drones.breaks.list.len());
    for (i, s) in sim.drones.props.subjects.iter().enumerate() {
        eprintln!("photo[{i}] placement={} ch={:#x} centre={}",
            s.placement, s.channel, p3([(s.lo[0]+s.hi[0])/2.0, (s.lo[1]+s.hi[1])/2.0, (s.lo[2]+s.hi[2])/2.0]));
    }
    for (i, d) in sim.drones.props.destroy.iter().enumerate() {
        eprintln!("destroy[{i}] placement={} centre={}", d.placement,
            p3([(d.lo[0]+d.hi[0])/2.0, (d.lo[1]+d.hi[1])/2.0, (d.lo[2]+d.hi[2])/2.0]));
    }
    for (i, g) in sim.drones.props.gunimps.iter().enumerate() {
        eprintln!("gunimp[{i}] placement={} at={}", g.placement, p3(g.pos));
    }
    for (i, b) in sim.drones.breaks.list.iter().enumerate() {
        eprintln!("break[{i}] placement={} centre={} kind={}", b.placement,
            p3([(b.lo[0]+b.hi[0])/2.0, (b.lo[1]+b.hi[1])/2.0, (b.lo[2]+b.hi[2])/2.0]), b.kind);
    }
    eprintln!("--- worldobjs: rotors {} inverters {} shooters {} dynobjs {} thirdcams {} music {} ---",
        sim.drones.worldobjs.rotors.len(), sim.drones.worldobjs.inverters.len(),
        sim.drones.worldobjs.shooters.len(), sim.drones.worldobjs.dynobjs.len(),
        sim.drones.worldobjs.thirdcams.len(), sim.drones.worldobjs.music_triggers.len());
    eprintln!("--- pickups {} ---", sim.drones.pickups.len());
    for (i, p) in sim.drones.pickups.iter().enumerate() {
        eprintln!("pickup[{i}] kind={} weapon={:#x} rounds={} state={} at={}",
            p.kind, p.weapon, p.rounds, p.state, p3(p.start));
    }
    eprintln!("--- simple scripts {} ---", sim.drones.sss.list.len());
    for (i, s) in sim.drones.sss.list.iter().enumerate() {
        eprintln!("ss[{i}] placement={} ch={:#x} state={:#x} mask={:#x} keys={} at={}",
            s.placement, s.channel, s.state, s.mask, s.keys.len(), p3(s.pos()));
    }
    eprintln!("--- AI drones {} ---", sim.drones.list.len());
    for (i, d) in sim.drones.list.iter().enumerate() {
        eprintln!("drone[{i}] skin={:#x} at={} present={} enabled={} corpse={} group={}",
            d.skin, p3(d.pos), d.present, d.enabled, d.corpse, d.group);
    }
    eprintln!("--- channels on at load ---");
    let on: Vec<String> = (0..256u32).filter(|&c| sim.drones.channels.on(c)).map(|c| format!("{c:#x}")).collect();
    eprintln!("{}", on.join(","));
    eprintln!("=== end census ===");
}
