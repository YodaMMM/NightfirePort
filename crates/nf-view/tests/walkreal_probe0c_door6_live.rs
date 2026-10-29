//! walkreal probe #6 (level-0c-door2turbine-3): the band crossing, part 4 — the upper catwalk.
//! Probe 5 found no mover and no serving wire; the top slice showed a ~15.73 surface running
//! x 50..72 at z -206.0 (over the pit) and ~16.08..17.08 at x 54 (z -207..-212), plus a type-2
//! zipline at y 17.85 (x 33..60.5, z -194.77). This probe: every ladder in the east half, the
//! catwalk's full extent (y 18 -> 14 dense over x 40..76, z -212..-194), the zipline ends' floor
//! probes, and the pit-side vertical faces at the band's east end (x 54..56) — how does a player
//! climb from the band to the catwalk? Diagnostic only, run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn down(sim: &nf_view::scene::Sim, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    sim.world.segment_hit([x, y0, z], [x, y1, z]).map(|(t, _)| y0 - (y0 - y1) * t)
}

#[test]
fn probe_catwalk() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();

    // 1. Every ladder with either end near the east half.
    eprintln!("--- ladders (centre in x 30..100, z -230..-190) ---");
    for (i, l) in sim.ladders.iter().enumerate() {
        if l.centre[0] > 30.0 && l.centre[0] < 100.0 && l.centre[2] > -230.0 && l.centre[2] < -190.0 {
            eprintln!("ladder[{i}] centre ({:.2},{:.2},{:.2}) top {:.2} bottom {:.2}", l.centre[0], l.centre[1], l.centre[2], l.top(), l.bottom());
        }
    }

    // 2. The catwalk map: y 18 -> 14, x 40..76 step 1, z -212..-194 step 1.
    eprintln!("=== catwalk map (y 18 -> 14) ===");
    eprintln!("        x: {}", (0..37).map(|i| format!("{:>6}", 40 + i)).collect::<String>());
    for zi in 0..19 {
        let z = -212.0f32 + zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..37 {
            let x = 40.0f32 + i as f32;
            match down(sim, x, 18.0, 14.0, z) {
                Some(y) => row.push_str(&format!("{y:5.2} ")),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }

    // 3. The band-end structure at x 54..56: vertical faces. Rays south->north at y 11..17,
    //    x 54.5, z -212 -> -204 (what rises from the band end?).
    eprintln!("=== the x 54.5 column, north rays (z -212 -> -204) ===");
    for y in [11.0f32, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0] {
        let hit = sim.world.segment_hit([54.5, y, -212.0], [54.5, y, -204.0]).map(|(t, _)| -212.0 + 8.0 * t);
        eprintln!("y {y:5.1}: first face at z {:?}", hit);
    }

    // 4. The zipline ends: floor below each (y 18 -> 1).
    eprintln!("=== zipline ends' floors ===");
    for (label, x, z) in [("A", 33.05f32, -194.77f32), ("mid", 46.79, -194.77), ("B", 60.53, -194.77)] {
        let f = down(sim, x, 18.0, 1.0, z);
        eprintln!("zipline {label} ({x:.2},{z:.2}) floor below: {:?}", f);
    }

    eprintln!("ok");
}
