//! level-14 probe (level-14-complete-2): the full vertical cross-section of the stairwell
//! slot at x=40.8 — every surface along each vertical line, so the slot's real shape is
//! read off the disc geometry instead of inferred from single down-rays. Prints only.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// All surfaces pierced by a downward ray from `y0` to `y1` at (x, z): walk the hits.
fn stack(sim: &nf_view::scene::Sim, x: f32, z: f32, y0: f32, y1: f32) -> Vec<f32> {
    let mut out = Vec::new();
    let mut y = y0;
    for _ in 0..12 {
        if y <= y1 { break; }
        match sim.world.segment([x, y, z], [x, y1, z]) {
            Some(t) => {
                let hy = y + (y1 - y) * t;
                out.push(hy);
                y = hy - 0.02;
            }
            None => break,
        }
    }
    out
}

#[test]
fn probe_xsect() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_ref().unwrap();
        // The vertical lines across the slot: the ramp foot, the trench, the wall plane, and
        // the corridor south of it.
        for x in [45.2f32, 44.8, 45.6] {
            for z in [56.5f32, 56.27, 56.1, 55.95, 55.8, 55.72, 55.65, 55.5, 55.2, 54.9, 54.6] {
                let hits = stack(sim, x, z, 3.8, -3.5);
                let s: Vec<String> = hits.iter().map(|y| format!("{y:.2}")).collect();
                eprintln!("x {x:.1} z {z:.2}: {}", if s.is_empty() { "OPEN 3.8..-3.5".into() } else { s.join(" / ") });
            }
            eprintln!("---");
        }
        // Horizontal clearance in the corridor band and in the slot: southward rays at the
        // heights between the wall's bottom edge and the corridor floor, and northward back.
        for y in [0.62f32, 0.55, 0.45, 0.42, 0.38, 0.35] {
            for (z0, z1) in [(56.3f32, 54.6f32), (54.6f32, 56.3)] {
                let hit = sim.world.segment([40.8, y, z0], [40.8, y, z1]);
                let where_ = hit.map(|t| format!("hit z {:.2}", z0 + (z1 - z0) * t)).unwrap_or_else(|| "CLEAR".into());
                eprintln!("y {y:.2} z {z0:.2}->{z1:.2}: {where_}");
            }
        }
        // The corridor's ceiling over its floor: how tall is the south corridor at x=40.8?
        for z in [55.5f32, 55.0, 54.6, 54.0, 53.0] {
            let hits = stack(sim, 40.8, z, 3.8, -3.5);
            let s: Vec<String> = hits.iter().map(|y| format!("{y:.2}")).collect();
            eprintln!("corridor x 40.8 z {z:.2}: {}", if s.is_empty() { "OPEN".into() } else { s.join(" / ") });
        }
    }
}
