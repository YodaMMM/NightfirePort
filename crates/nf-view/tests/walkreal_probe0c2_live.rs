//! walkreal probe 2 for 0700000c: fine heightmaps — the yard/ring (north band), the zipline
//! ends, the ST-1 south drop zone, the lower walkway strip, the warehouse ground walk, and the
//! trigger volumes the route needs (skylight plates, CamSubjects, plates). Diagnostic only.

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_0700000c_fine() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_ref().unwrap();

    // Heightmap: down-segment from y 19 to y -2 at each grid node; print the FIRST hit's y.
    eprintln!("--- heightmap yard/ring (x 0..100 step 4, z -186..-222 step 4; first hit y in 19..-2) ---");
    for z in (-222..=-186).step_by(4) {
        let z = z as f32;
        let mut row = String::new();
        for x in (0..=100).step_by(4) {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 19.0, z], [x, -2.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 19.0 - 21.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:        {}", (0..=100).step_by(4).map(|x| format!("{x:>4}")).collect::<String>());

    eprintln!("--- heightmap ST-1 south/east (x 2..30 step 2, z -148..-160 step 2) ---");
    for z in (-160..=-148).step_by(2) {
        let z = z as f32;
        let mut row = String::new();
        for x in (2..=30).step_by(2) {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 17.0, z], [x, 4.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 17.0 - 13.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:        {}", (2..=30).step_by(2).map(|x| format!("{x:>4}")).collect::<String>());

    eprintln!("--- heightmap lower walkway strip (x 3..10 step 1, z -110..-157 step 3) ---");
    for z in (-157..=-110).rev().step_by(3) {
        let z = z as f32;
        let mut row = String::new();
        for x in 3..=10 {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 16.0, z], [x, 10.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 16.0 - 6.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:        {}", (3..=10).map(|x| format!("{x:>4}")).collect::<String>());

    eprintln!("--- heightmap warehouse ground walk (x 5..55 step 5, z -112..-176 step 4, y 8..-1) ---");
    for z in (-176..=-112).step_by(4) {
        let z = z as f32;
        let mut row = String::new();
        for x in (5..=55).step_by(5) {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 8.0, z], [x, -1.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 8.0 - 9.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:        {}", (5..=55).step_by(5).map(|x| format!("{x:>4}")).collect::<String>());

    eprintln!("--- zipline/wire end platforms ---");
    let ends: &[(&str, [f32; 3])] = &[
        ("zipline A", [33.05, 17.85, -194.77]),
        ("zipline B", [60.53, 17.85, -194.77]),
        ("zipline mid", [46.79, 17.85, -194.77]),
        ("wire283 A", [27.45, 14.44, -208.76]),
        ("wire283 B", [38.03, 14.44, -217.64]),
        ("wire282 A", [2.80, 14.11, -228.20]),
        ("wire282 B", [2.80, 14.11, -218.11]),
        ("wire284 A", [-1.50, 13.67, -216.12]),
        ("wire284 B", [-11.60, 13.67, -210.28]),
    ];
    for (name, at) in ends {
        // What is within 1.5 below each end?
        let hit = sim.world.segment_hit([at[0], at[1] + 1.0, at[2]], [at[0], at[1] - 3.0, at[2]]);
        let below = hit.map(|(t, _)| at[1] + 1.0 - 4.0 * t);
        // Any floor wider around: 4 offsets
        let mut around = Vec::new();
        for (dx, dz) in [(-1.5f32, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5)] {
            let h = sim.world.segment_hit([at[0] + dx, at[1] + 1.0, at[2] + dz], [at[0] + dx, at[1] - 3.0, at[2] + dz]);
            around.push(h.map(|(t, _)| (at[1] + 1.0 - 4.0 * t)));
        }
        eprintln!("{name}: floor below={below:?} around={around:?}");
    }

    eprintln!("--- trigger volumes of interest (tri bounds) ---");
    for t in &sim.drones.triggers {
        let c = t.centre();
        // Only the route-relevant ones: out 0x1b, 0x02/0x03, 0x33, 0x65, 0x43, 0x66, and the load gate
        if t.out != 0x1b && t.out != 0x02 && t.out != 0x03 && t.out != 0x33 && t.out != 0x65 && t.out != 0x43 && t.out != 0x66 && t.out != 0xb4 { continue; }
        let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
        for tri in &t.tris {
            for v in tri { for a in 0..3 { lo[a] = lo[a].min(v[a]); hi[a] = hi[a].max(v[a]); } }
        }
        if t.tris.is_empty() { eprintln!("trig out={:#04x} kind={} mask={:#x}: NO TRIS", t.out, t.kind, t.mask); }
        else { eprintln!("trig out={:#04x} kind={} mask={:#x}: vol x {:.2}..{:.2} y {:.2}..{:.2} z {:.2}..{:.2}", t.out, t.kind, t.mask, lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]); }
        let _ = c;
    }
    eprintln!("--- subjects (camera) ---");
    for s in &sim.drones.props.subjects {
        eprintln!("subject ch={:#x} lo={:?} hi={:?}", s.channel, s.lo, s.hi);
    }
    eprintln!("--- pickups ---");
    for p in walker::pickups(sim) {
        eprintln!("pickup at=({:.1},{:.1},{:.1}) kind={:#x} ch={:#x}", p.0[0], p.0[1], p.0[2], p.1, p.2);
    }
}
