//! probe02quads: dump the three stand-in quads (0x17 plate tr[003], 0x3C zone tr[032], 0x19 plate
//! tr[009]) — kind/flags/out/mask/centre/tris — plus segment checks along candidate approach lines,
//! so the real-input crossings can be planned from the actual geometry.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe02quads() {
    let iso = iso();
    let w = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load 07000002"));
    let sim = w.scene.sim.as_ref().unwrap();
    let want = [0x17u8, 0x3c, 0x19];
    for t in &sim.drones.triggers {
        if !want.contains(&t.out) { continue; }
        let c = t.centre();
        eprintln!("TRIG out {:#x} kind {:#x} mask {:#x} lock {:#x} gone {} ntris {} centre ({:.2},{:.2},{:.2})",
            t.out, t.kind, t.mask, t.lock, t.gone, t.tris.len(), c[0], c[1], c[2]);
        for (k, tri) in t.tris.iter().enumerate() {
            eprintln!("  tri{k} v0 ({:.2},{:.2},{:.2}) v1 ({:.2},{:.2},{:.2}) v2 ({:.2},{:.2},{:.2})",
                tri[0][0], tri[0][1], tri[0][2], tri[1][0], tri[1][1], tri[1][2], tri[2][0], tri[2][1], tri[2][2]);
        }
    }
    // Segment checks: candidates toward each quad from the route's own points.
    let checks: [(&str, [f32; 3], [f32; 3]); 9] = [
        ("0x17 from west-wing south", [-37.4, 5.9, 8.0], [-35.2, 6.5, 5.05]),
        ("0x17 from south hall west", [-31.2, 5.9, 22.1], [-35.2, 6.5, 5.05]),
        ("0x17 from west", [-38.5, 6.5, 5.0], [-35.2, 6.5, 5.05]),
        ("0x17 from north", [-35.2, 6.5, 8.0], [-35.2, 6.5, 5.05]),
        ("0x3C from east", [-42.0, 5.9, 11.7], [-45.4, 5.74, 11.7]),
        ("0x3C from south", [-45.4, 5.9, 14.0], [-45.4, 5.74, 11.7]),
        ("0x3C from north", [-45.4, 5.9, 9.0], [-45.4, 5.74, 11.7]),
        ("0x19 from west", [-30.0, 5.9, -21.5], [-26.9, 6.0, -20.3]),
        ("0x19 from south", [-26.9, 6.0, -17.0], [-26.9, 6.0, -20.3]),
    ];
    for (name, a, b) in checks {
        let seg = sim.world.segment(a, b);
        eprintln!("SEG {name}: ({:.1},{:.1},{:.1})->({:.1},{:.1},{:.1}) = {:?}",
            a[0], a[1], a[2], b[0], b[1], b[2], seg);
    }
    // The nav graph's own answer for each approach (leg = the walker's real routing).
    let starts = [[-31.2f32, 5.9, 22.1], [-37.4, 5.9, 8.0], [-42.0, 5.9, 11.7], [-30.0, 5.9, -21.5]];
    let dests = [[-35.2f32, 6.5, 5.05], [-45.4, 5.74, 11.7], [-26.9, 6.0, -20.3]];
    let starts: Vec<[f32; 3]> = starts.to_vec();
    let dests: Vec<[f32; 3]> = dests.to_vec();
    for s in &starts {
        for d in &dests {
            if walker::leg(sim, *s, *d).is_empty() { continue; }
            let route = walker::nav_route(sim, *s, *d);
            eprintln!("NAV ({:.1},{:.1},{:.1})->({:.1},{:.1},{:.1}) nodes {:?}", s[0], s[1], s[2], d[0], d[1], d[2],
                route.as_ref().map(|r| r.len()).unwrap_or(0));
        }
    }
    drop(sim);
    // Teleport experiment (probe only): does the port fire each quad AT ALL when Bond stands on it?
    let spots: [(&str, [f32; 3], u8); 3] = [
        ("0x17 plate", [-35.2, 6.52, 5.05], 0x17),
        ("0x3C zone", [-45.4, 5.9, 11.7], 0x3c),
        ("0x19 plate", [-26.9, 6.04, -20.28], 0x19),
    ];
    for (name, at, ch) in spots {
        let mut w2 = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load"));
        w2.skip_cutscenes = true;
        for _ in 0..60 { w2.tick(Default::default(), false); }
        {
            let sim = w2.scene.sim.as_mut().unwrap();
            sim.bond.pos = at;
        }
        for _ in 0..60 { w2.tick(Default::default(), false); }
        let on = w2.scene.sim.as_ref().unwrap().drones.channels.on(ch as u32);
        let p = w2.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("TELE {name}: ch {ch:#x} = {on} after standing at ({:.1},{:.1},{:.1}) (bond now {:.1},{:.1},{:.1})",
            at[0], at[1], at[2], p[0], p[1], p[2]);
    }
}
