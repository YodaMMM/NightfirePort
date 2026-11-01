//! walkreal probe (level-0c-secret-door-reach): the UPPER EAST lane scan. The hall's upper-east
//! lane (12.4) and the ST-1 walkway (14.6) may continue east above the x ~15 wall toward the
//! warehouse upper floor (8.7-9.8). Column scans x 13..48, z −170..−142 (all surfaces) + body-height
//! east rays at lane/walkway heights + north-south rays, to name the game's high route east.
//! Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_upper_lane() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();

    // 1. Column scans, z rows step 1.5, x cols step 2, all surfaces from y 18.
    let mut z = -170.0f32;
    while z <= -141.5 {
        let mut row = format!("z {z:7.1}:");
        let mut x = 13.0f32;
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
            row.push_str(&format!(" ({x:.0})[{}]", s.join(",")));
            x += 2.0;
        }
        eprintln!("{row}");
        z += 1.5;
    }

    // 2. East rays at lane/walkway heights from x 14 (z −158..−148): how far east is it walkable?
    eprintln!("=== east rays from (14.0, y, z) to x 30 ===");
    for z in [-158.0f32, -156.0, -154.0, -152.0, -150.0, -148.0] {
        for y in [11.6f32, 12.4, 13.0, 14.0, 14.7] {
            let hit = sim.world.segment_hit([14.0, y, z], [30.0, y, z]);
            let s = hit.map(|(t, _)| format!("hit at x {:.2}", 14.0 + t * 16.0)).unwrap_or("clear to 30".into());
            eprintln!("  z {z} y {y}: {s}");
        }
    }

    // 3. The 9.2 deck's extent (x 21..30, z −160..−150): is it a floor? down-rays + its edges.
    eprintln!("=== the 9.x deck scan ===");
    for z in [-159.0f32, -157.0, -155.0, -153.0, -151.0] {
        let mut row = format!("z {z:7.1}:");
        let mut x = 20.0f32;
        while x <= 34.0 {
            let hit = sim.world.segment_hit([x, 10.5, z], [x, 8.0, z]);
            let s = hit.map(|(t, _)| format!("{:.2}", 10.5 - 2.5 * t)).unwrap_or("....".into());
            row.push_str(&format!(" ({x:.0}){s}"));
            x += 1.0;
        }
        eprintln!("{row}");
    }

    // 4. The warehouse upper floor's west edge (x 44..48, z −165..−150) + the gap x 24..45.
    eprintln!("=== the 8.7 floor west edge scan (y 8..10) ===");
    for z in [-165.0f32, -160.0, -155.0, -150.0, -146.0] {
        let mut row = format!("z {z:7.1}:");
        let mut x = 24.0f32;
        while x <= 48.0 {
            let hit = sim.world.segment_hit([x, 10.0, z], [x, 8.0, z]);
            let s = hit.map(|(t, _)| format!("{:.2}", 10.0 - 2.0 * t)).unwrap_or("....".into());
            row.push_str(&format!(" ({x:.0}){s}"));
            x += 2.0;
        }
        eprintln!("{row}");
    }
    eprintln!("ok");
}
