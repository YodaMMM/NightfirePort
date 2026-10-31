//! walkreal probe (level-0c-secret-door-reach): the x-40 bridge + the ramp walk. Is the north-east
//! ground strip (x 33..39, y ~0.1, z −149..−143) connected to the warehouse ground (comp 2, x ≥
//! 40.8)? Fine surfaces x 36..44, z −152..−142 + knee-height rays; and the 13.3 deck → the 6.9
//! ramp edge geometry (the deck's south/east edges, the ramp's north tip). Diagnostic only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_bridge() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();

    // 1. The bridge zone: fine columns x 36..44 step 0.5, z −152..−142 step 1, all surfaces.
    eprintln!("=== the x-40 bridge zone ===");
    for z in [-152.0f32, -150.0, -148.0, -146.0, -144.0, -142.5] {
        let mut row = format!("z {z:7.1}:");
        let mut x = 36.0f32;
        while x <= 44.5 {
            let mut top = 12.0f32;
            let mut ys: Vec<f32> = vec![];
            for _ in 0..5 {
                match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
                    Some((t, _)) => {
                        let y = top - top * t;
                        ys.push(y);
                        top = y - 0.05;
                        if top <= 0.05 { break; }
                    }
                    None => break,
                }
            }
            let s: Vec<String> = ys.iter().map(|y| format!("{y:.2}")).collect();
            row.push_str(&format!(" ({x:.1})[{}]", s.join(",")));
            x += 0.5;
        }
        eprintln!("{row}");
    }

    // 2. Knee-height east rays across x 39..42 (is there a wall?) at y 0.9/1.4.
    eprintln!("=== knee rays west->east at y 0.9 and 1.4 ===");
    for z in [-151.0f32, -149.0, -147.0, -145.0, -143.5] {
        for y in [0.9f32, 1.4] {
            let hit = sim.world.segment_hit([38.5, y, z], [42.5, y, z]);
            let s = hit.map(|(t, _)| format!("hit at x {:.2}", 38.5 + t * 4.0)).unwrap_or("clear".into());
            eprintln!("  z {z} y {y}: {s}");
        }
    }

    // 3. The deck edges: down-rays around the 13.3 deck (x 18..23, z −162..−155) and the ramp's
    //    north tip (x 19..22, z −157..−154).
    eprintln!("=== the deck/ramp seam (z −162..−153, x 17..24) ===");
    let mut z = -162.0f32;
    while z <= -153.0 {
        let mut row = format!("z {z:7.1}:");
        let mut x = 17.0f32;
        while x <= 24.5 {
            let mut top = 15.0f32;
            let mut ys: Vec<f32> = vec![];
            for _ in 0..6 {
                match sim.world.segment_hit([x, top, z], [x, 0.0, z]) {
                    Some((t, _)) => {
                        let y = top - top * t;
                        ys.push(y);
                        top = y - 0.05;
                        if top <= 0.05 { break; }
                    }
                    None => break,
                }
            }
            let s: Vec<String> = ys.iter().map(|y| format!("{y:.1}")).collect();
            row.push_str(&format!(" ({x:.0})[{}]", s.join(",")));
            x += 1.0;
        }
        eprintln!("{row}");
        z += 1.0;
    }

    // 4. The ramp's bottom + the strip: down-rays x 28..34, z −152..−143.
    eprintln!("=== the ramp bottom / the strip top (x 28..34) ===");
    for z in [-152.0f32, -149.0, -146.0, -143.5] {
        let mut row = format!("z {z:7.1}:");
        let mut x = 28.0f32;
        while x <= 34.5 {
            let hit = sim.world.segment_hit([x, 8.0, z], [x, 0.0, z]);
            let s = hit.map(|(t, _)| format!("{:.2}", 8.0 - 8.0 * t)).unwrap_or("void".into());
            row.push_str(&format!(" ({x:.0}){s}"));
            x += 1.0;
        }
        eprintln!("{row}");
    }
    eprintln!("ok");
}
