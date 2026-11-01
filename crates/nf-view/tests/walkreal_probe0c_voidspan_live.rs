//! walkreal probe (level-0c-secret-door-reach): the VOID SPAN surface scan. Every column over
//! x 14..48, z −160..−146 (step 1.0) multi-hit down-rayed from y 18, plus body-height east-west
//! rays from the display ground's east edge, to name what the span between ST-1 and the
//! warehouse actually holds. Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_void_span() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();

    // 1. Coarse column map over the span.
    let mut z = -160.0f32;
    while z <= -145.5 {
        let mut row = format!("z {z:7.1}: ");
        let mut x = 14.0f32;
        while x <= 47.5 {
            let mut top = 18.0f32;
            let mut ys: Vec<f32> = vec![];
            for _ in 0..8 {
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
            row.push_str(&format!("{:14}", format!("({x:.0})[{}]", s.join(","))));
            x += 3.0;
        }
        eprintln!("{row}");
        z += 1.5;
    }

    // 2. Body-height east-west rays from the display ground's east edge (x 14.5) at y 6.3..8,
    //    z −155..−150 — does anything at body height stop an eastward walk or crouch?
    eprintln!("=== east rays from (14.2, y, z) ===");
    for z in [-155.5f32, -154.0, -152.5, -151.5, -150.5] {
        for y in [6.4f32, 7.0, 7.5, 8.0, 9.0, 10.5] {
            let hit = sim.world.segment_hit([14.2, y, z], [20.0, y, z]);
            let s = hit.map(|(t, _)| format!("hit at x {:.2}", 14.2 + t * (20.0 - 14.2)))
                .unwrap_or("clear".into());
            eprintln!("  z {z} y {y}: {s}");
        }
    }

    // 3. What stands above the span floor? Up-rays from under the dots (x 25..38, z −153).
    eprintln!("=== up-rays from below (x, z −153), y 1 -> 14 ===");
    for x in [16.0f32, 20.0, 25.0, 30.0, 35.0, 38.0, 42.0, 46.0] {
        let hit = sim.world.segment_hit([x, 1.0, -153.0], [x, 14.0, -153.0]);
        let s = hit.map(|(t, _)| format!("first surface above y 1 at y {:.2}", 1.0 + t * 13.0))
            .unwrap_or("open to 14".into());
        eprintln!("  x {x}: {s}");
    }

    // 4. The mezzanine band dots' exact heights: down-rays at (25..38 step 1, −155..−151 step 1),
    //    y 3..7 band only.
    eprintln!("=== span floors in y 3..7 ===");
    for z in [-155.0f32, -154.0, -153.0, -152.0, -151.0] {
        let mut row = format!("z {z:7.1}: ");
        let mut x = 15.0f32;
        while x <= 47.5 {
            let hit = sim.world.segment_hit([x, 7.0, z], [x, 3.0, z]);
            let s = hit.map(|(t, _)| format!("{:.2}", 7.0 - 4.0 * t)).unwrap_or("....".into());
            row.push_str(&format!("{:>6}", s));
            x += 1.0;
        }
        eprintln!("{row}");
    }
    eprintln!("ok");
}
